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

const DEFAULT_BASE_URL: &str = "https://api.x.ai";
const CHAT_COMPLETIONS_PATH: &str = "/v1/chat/completions";
const SUPPORTED_MODELS: [&str; 2] = ["grok-2", "grok-2-mini"];

#[derive(Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Deserialize)]
struct ChatCompletionChoice {
    delta: ChatCompletionDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionDelta {
    content: Option<String>,
}

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
        mut on_token: impl FnMut(&str),
    ) -> Result<CompletionResult, LlmError> {
        if self.api_key.trim().is_empty() {
            return Err(LlmError::AuthError("missing API key".into()));
        }
        if !Self::is_supported_model(model) {
            return Err(LlmError::InvalidModel(model.into()));
        }

        let url = self.endpoint_url();
        let body = build_openai_compatible_request_body(messages, model, config, true);

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
            if done {
                return;
            }
            if data == "[DONE]" {
                finish_reason = FinishReason::Stop;
                done = true;
                return;
            }

            let Ok(chunk) = miniserde::json::from_str::<ChatCompletionChunk>(data) else {
                return;
            };

            let Some(choice) = chunk.choices.first() else {
                return;
            };

            if let Some(reason) = choice.finish_reason.as_deref() {
                finish_reason = match reason {
                    "stop" => FinishReason::Stop,
                    "length" => FinishReason::Length,
                    "content_filter" => FinishReason::ContentFilter,
                    other => FinishReason::Other(other.to_string()),
                };
            }

            if let Some(content) = choice.delta.content.as_deref() {
                on_token(content);
                full_text.push_str(content);
            }
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

fn build_openai_compatible_request_body(
    messages: &[Message],
    model: &str,
    config: &GenerationConfig,
    stream: bool,
) -> String {
    let mut out = String::new();
    out.push_str("{\"model\":\"");
    push_json_escaped(&mut out, model);
    out.push_str("\",\"messages\":[");

    for (i, message) in messages.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str("{\"role\":\"");
        out.push_str(role_to_str(message.role));
        out.push_str("\",\"content\":\"");
        push_json_escaped(&mut out, &message.content);
        out.push_str("\"}");
    }
    out.push_str("],\"temperature\":");
    out.push_str(&format!("{}", config.temperature));

    if let Some(max_tokens) = config.max_tokens {
        out.push_str(",\"max_tokens\":");
        out.push_str(&format!("{}", max_tokens));
    }

    if let Some(top_p) = config.top_p {
        out.push_str(",\"top_p\":");
        out.push_str(&format!("{}", top_p));
    }

    if let Some(top_k) = config.top_k {
        out.push_str(",\"top_k\":");
        out.push_str(&format!("{}", top_k));
    }

    if !config.stop_sequences.is_empty() {
        out.push_str(",\"stop\":[");
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

fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
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
