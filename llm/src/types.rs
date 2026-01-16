extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Represents a message in a conversation with an LLM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    /// Create a new message with the given role and content.
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }
}

/// Represents the role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// System message - typically used for instructions or context
    System,
    /// User message - input from the user
    User,
    /// Assistant message - response from the LLM
    Assistant,
}

/// Configuration for text generation parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationConfig {
    /// Temperature for sampling (0.0-2.0). Higher values make output more random.
    pub temperature: f32,
    /// Maximum number of tokens to generate. None means no limit.
    pub max_tokens: Option<usize>,
    /// Sequences that will stop generation when encountered.
    pub stop_sequences: Vec<String>,
    /// Top-p (nucleus) sampling parameter (0.0-1.0). 
    /// Samples from tokens with cumulative probability up to this value.
    pub top_p: Option<f32>,
    /// Top-k sampling parameter. Only sample from the top K most likely tokens.
    pub top_k: Option<usize>,
}

impl GenerationConfig {
    /// Create a new generation config with default values.
    pub fn new() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: None,
            stop_sequences: Vec::new(),
            top_p: None,
            top_k: None,
        }
    }

    /// Create a generation config with the specified temperature.
    pub fn with_temperature(temperature: f32) -> Self {
        Self {
            temperature,
            ..Self::new()
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about an LLM model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    /// Unique identifier for the model (e.g., "gpt-4o", "claude-sonnet-4-20250514")
    pub id: String,
    /// Human-readable name for the model (e.g., "GPT-4o", "Claude Sonnet 4")
    pub name: String,
    /// Maximum context length in tokens that this model supports.
    pub context_length: usize,
    /// Whether this model supports streaming responses.
    pub supports_streaming: bool,
}

impl ModelInfo {
    /// Create a new model info.
    pub fn new(id: String, name: String, context_length: usize, supports_streaming: bool) -> Self {
        Self {
            id,
            name,
            context_length,
            supports_streaming,
        }
    }
}

/// Result of a completion request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionResult {
    /// The generated text.
    pub text: String,
    /// Number of tokens used in the completion (if available).
    pub tokens_used: Option<usize>,
    /// Reason why the generation stopped.
    pub finish_reason: FinishReason,
}

impl CompletionResult {
    /// Create a new completion result.
    pub fn new(text: String, tokens_used: Option<usize>, finish_reason: FinishReason) -> Self {
        Self {
            text,
            tokens_used,
            finish_reason,
        }
    }
}

/// Reason why text generation stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinishReason {
    /// Generation stopped because a stop sequence was encountered.
    Stop,
    /// Generation stopped because the maximum token limit was reached.
    Length,
    /// Generation stopped due to content filtering.
    ContentFilter,
    /// Generation stopped for another reason (with description).
    Other(String),
}
