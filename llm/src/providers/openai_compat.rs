#![allow(unused_attributes)]
#![no_std]

extern crate alloc;

use crate::types::{FinishReason, GenerationConfig, Message, Role};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use miniserde::Deserialize;

#[derive(Deserialize)]
pub struct ChatCompletionChunk {
    pub choices: Vec<ChatCompletionChoice>,
}

#[derive(Deserialize)]
pub struct ChatCompletionChoice {
    pub delta: ChatCompletionDelta,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize)]
pub struct ChatCompletionDelta {
    pub content: Option<String>,
}

pub fn build_request_body(
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

pub fn apply_chunk_to_text(
    data: &str,
    full_text: &mut String,
    finish_reason: &mut FinishReason,
    done: &mut bool,
    mut on_token: impl FnMut(&str),
) {
    if *done {
        return;
    }

    if data == "[DONE]" {
        *finish_reason = FinishReason::Stop;
        *done = true;
        return;
    }

    let Ok(chunk) = miniserde::json::from_str::<ChatCompletionChunk>(data) else {
        return;
    };

    let Some(choice) = chunk.choices.first() else {
        return;
    };

    if let Some(reason) = choice.finish_reason.as_deref() {
        *finish_reason = match reason {
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

