#![allow(unused_attributes)]
#![no_std]

extern crate alloc;

use crate::streaming::for_each_sse_data;
use crate::types::{CompletionResult, FinishReason, GenerationConfig, Message, ModelInfo, Role};
use crate::{LlmError, LlmProvider};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use miniserde::Deserialize;
use network::{get_network_stack, HttpClient};
use smoltcp::wire::Ipv4Address;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const MESSAGES_PATH: &str = "/v1/messages";
const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";
const SUPPORTED_MODELS: [&str; 3] = [
    "claude-sonnet-4-20250514",
    "claude-opus-4-20250514",
    "claude-haiku-3-5-20241022",
];

#[derive(Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicDelta>,
}

#[derive(Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
}

pub struct AnthropicClient {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    anthropic_version: String,
    get_time_ms: fn() -> i64,
    sleep_ms: Option<fn(i64)>,
    models: Vec<ModelInfo>,
}

impl AnthropicClient {
    pub fn new(
        api_key: String,
        dns_server: Ipv4Address,
        get_time_ms: fn() -> i64,
        sleep_ms: Option<fn(i64)>,
    ) -> Self {
        Self::new_with_base_url(
            api_key,
            dns_server,
            DEFAULT_BASE_URL.into(),
            DEFAULT_ANTHROPIC_VERSION.into(),
            get_time_ms,
            sleep_ms,
        )
    }

    pub fn new_with_base_url(
        api_key: String,
        dns_server: Ipv4Address,
        base_url: String,
        anthropic_version: String,
        get_time_ms: fn() -> i64,
        sleep_ms: Option<fn(i64)>,
    ) -> Self {
        let models = Vec::from([
            ModelInfo::new(
                "claude-sonnet-4-20250514".into(),
                "Claude Sonnet 4".into(),
                200_000,
                true,
            ),
            ModelInfo::new(
                "claude-opus-4-20250514".into(),
                "Claude Opus 4".into(),
                200_000,
                true,
            ),
            ModelInfo::new(
                "claude-haiku-3-5-20241022".into(),
                "Claude Haiku 3.5".into(),
                200_000,
                true,
            ),
        ]);

        Self {
            api_key,
            http_client: HttpClient::new(dns_server),
            base_url,
            anthropic_version,
            get_time_ms,
            sleep_ms,
            models,
        }
    }

    fn endpoint_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{base}{MESSAGES_PATH}")
    }

    fn is_supported_model(model: &str) -> bool {
        SUPPORTED_MODELS.iter().any(|m| *m == model)
    }
}

impl LlmProvider for AnthropicClient {
    fn name(&self) -> &str {
        "Anthropic"
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn default_model(&self) -> &str {
        "claude-sonnet-4-20250514"
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
        let body = build_anthropic_request_body(messages, model, config, true);

        let headers = [
            ("x-api-key", self.api_key.as_str()),
            ("anthropic-version", self.anthropic_version.as_str()),
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
            if done {
                return;
            }

            let Ok(event) = miniserde::json::from_str::<AnthropicStreamEvent>(data) else {
                return;
            };

            match event.event_type.as_str() {
                "content_block_delta" => {
                    let Some(delta) = event.delta else { return };
                    if delta.delta_type.as_deref() != Some("text_delta") {
                        return;
                    }
                    let Some(text) = delta.text.as_deref() else { return };
                    on_token(text);
                    full_text.push_str(text);
                }
                "message_stop" => {
                    finish_reason = FinishReason::Stop;
                    done = true;
                }
                _ => {}
            }
        });

        Ok(CompletionResult::new(full_text, None, finish_reason))
    }

    fn validate_api_key(&self) -> Result<(), LlmError> {
        if self.api_key.trim().is_empty() {
            return Err(LlmError::AuthError("missing API key".into()));
        }
        Ok(())
    }
}

fn build_anthropic_request_body(
    messages: &[Message],
    model: &str,
    config: &GenerationConfig,
    stream: bool,
) -> String {
    let mut system = String::new();
    let mut non_system: Vec<&Message> = Vec::new();
    for message in messages {
        if message.role == Role::System {
            if !system.is_empty() {
                system.push('\n');
            }
            system.push_str(&message.content);
        } else {
            non_system.push(message);
        }
    }

    let max_tokens = config.max_tokens.unwrap_or(1024);

    let mut out = String::new();
    out.push_str("{\"model\":\"");
    push_json_escaped(&mut out, model);
    out.push_str("\",\"max_tokens\":");
    out.push_str(&format!("{}", max_tokens));

    if !system.is_empty() {
        out.push_str(",\"system\":\"");
        push_json_escaped(&mut out, &system);
        out.push('"');
    }

    out.push_str(",\"messages\":[");
    for (i, message) in non_system.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str("{\"role\":\"");
        out.push_str(match message.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "user",
        });
        out.push_str("\",\"content\":\"");
        push_json_escaped(&mut out, &message.content);
        out.push_str("\"}");
    }
    out.push(']');

    out.push_str(",\"temperature\":");
    out.push_str(&format!("{}", config.temperature));

    if !config.stop_sequences.is_empty() {
        out.push_str(",\"stop_sequences\":[");
        for (i, stop) in config.stop_sequences.iter().enumerate() {
            if i != 0 {
                out.push(',');
            }
            out.push('"');
            push_json_escaped(&mut out, stop);
            out.push('"');
        }
        out.push(']');
    }

    out.push_str(",\"stream\":");
    out.push_str(if stream { "true" } else { "false" });
    out.push('}');
    out
}

fn push_json_escaped(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str("\\u00");
                let b = c as u8;
                out.push(nibble_to_hex((b >> 4) & 0x0F));
                out.push(nibble_to_hex(b & 0x0F));
            }
            c => out.push(c),
        }
    }
}

fn nibble_to_hex(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + (n - 10)) as char,
        _ => '0',
    }
}

