#![allow(unused_attributes)]
#![no_std]

extern crate alloc;

use crate::providers::openai_compat::{apply_chunk_to_text, build_request_body};
use crate::streaming::for_each_sse_data;
use crate::types::{CompletionResult, FinishReason, GenerationConfig, Message, ModelInfo};
use crate::{LlmError, LlmProvider};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use network::{get_network_stack, HttpClient};
use smoltcp::wire::Ipv4Address;

const DEFAULT_BASE_URL: &str = "https://api.x.ai";
const CHAT_COMPLETIONS_PATH: &str = "/v1/chat/completions";
const SUPPORTED_MODELS: [&str; 2] = ["grok-2", "grok-2-mini"];

pub struct XaiClient {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    get_time_ms: fn() -> i64,
    sleep_ms: Option<fn(i64)>,
    models: Vec<ModelInfo>,
}

impl XaiClient {
    pub fn new(
        api_key: String,
        dns_server: Ipv4Address,
        get_time_ms: fn() -> i64,
        sleep_ms: Option<fn(i64)>,
    ) -> Self {
        Self::new_with_base_url(api_key, dns_server, DEFAULT_BASE_URL.to_string(), get_time_ms, sleep_ms)
    }

    pub fn new_with_base_url(
        api_key: String,
        dns_server: Ipv4Address,
        base_url: String,
        get_time_ms: fn() -> i64,
        sleep_ms: Option<fn(i64)>,
    ) -> Self {
        let models = Vec::from([
            ModelInfo::new("grok-2".into(), "Grok 2".into(), 128_000, true),
            ModelInfo::new("grok-2-mini".into(), "Grok 2 Mini".into(), 128_000, true),
        ]);

        Self {
            api_key,
            http_client: HttpClient::new(dns_server),
            base_url,
            get_time_ms,
            sleep_ms,
            models,
        }
    }

    fn endpoint_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{base}{CHAT_COMPLETIONS_PATH}")
    }

    fn is_supported_model(model: &str) -> bool {
        SUPPORTED_MODELS.iter().any(|m| *m == model)
    }
}

impl LlmProvider for XaiClient {
    fn name(&self) -> &str {
        "xAI"
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn default_model(&self) -> &str {
        "grok-2"
    }

    fn complete(
        &mut self,
        messages: &[Message],
        model: &str,
        config: &GenerationConfig,
        mut on_token: &mut dyn FnMut(&str),
    ) -> Result<CompletionResult, LlmError> {
        if self.api_key.trim().is_empty() {
            return Err(LlmError::AuthError("missing API key".into()));
        }
        if !Self::is_supported_model(model) {
            return Err(LlmError::InvalidModel(model.into()));
        }

        let url = self.endpoint_url();
        let body = build_request_body(messages, model, config, true);

        let auth_header = format!("Bearer {}", self.api_key);
        let headers = [
            ("Authorization", auth_header.as_str()),
            ("Accept", "text/event-stream"),
        ];

        let mut guard = get_network_stack();
        let stack = guard
            .as_mut()
            .ok_or_else(|| LlmError::NetworkError("network stack not initialized".into()))?;

        let response = self
            .http_client
            .post_json(stack, &url, &body, &headers, self.get_time_ms, self.sleep_ms)
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if response.status == 401 || response.status == 403 {
            return Err(LlmError::AuthError("unauthorized".into()));
        }
        if response.status == 429 {
            let retry_after = response
                .header("Retry-After")
                .and_then(|v| v.trim().parse::<u64>().ok());
            return Err(LlmError::RateLimitError { retry_after });
        }
        if response.status >= 400 {
            let body_str = core::str::from_utf8(&response.body)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "<non-utf8 body>".into());
            return Err(LlmError::HttpError {
                status: response.status,
                body: body_str,
            });
        }

        let body_str = core::str::from_utf8(&response.body)
            .map_err(|e| LlmError::ParseError(format!("invalid utf-8 SSE body: {e}")))?;

        let mut full_text = String::new();
        let mut finish_reason = FinishReason::Stop;
        let mut done = false;

        for_each_sse_data(body_str, |data| {
            apply_chunk_to_text(data, &mut full_text, &mut finish_reason, &mut done, &mut on_token);
        });

        Ok(CompletionResult::new(
            full_text,
            None,
            finish_reason,
        ))
    }

    fn validate_api_key(&self) -> Result<(), LlmError> {
        if self.api_key.trim().is_empty() {
            return Err(LlmError::AuthError("missing API key".into()));
        }
        Ok(())
    }
}
