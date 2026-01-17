#![no_std]

//! LLM API client library for moteOS.
//!
//! This crate provides common types and traits for interacting with various
//! Large Language Model (LLM) providers in a `no_std` environment.

extern crate alloc;

pub mod error;
pub mod providers;
pub mod streaming;
pub mod types;

pub use error::LlmError;
pub use providers::{AnthropicClient, GroqClient, OpenAiClient, XaiClient};
pub use types::{CompletionResult, FinishReason, GenerationConfig, Message, ModelInfo, Role};

/// Trait for LLM providers.
///
/// This trait defines the interface that all LLM providers must implement.
/// It supports both streaming and non-streaming completions through a callback
/// mechanism suitable for `no_std` environments.
pub trait LlmProvider: Send {
    /// Get the name of this provider (e.g., "OpenAI", "Anthropic").
    fn name(&self) -> &str;

    /// Get a list of available models for this provider.
    fn models(&self) -> &[ModelInfo];

    /// Get the default model identifier for this provider.
    fn default_model(&self) -> &str;

    /// Generate a completion for the given messages.
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation history
    /// * `model` - The model identifier to use
    /// * `config` - Generation configuration parameters
    /// * `on_token` - Callback function called for each token as it's generated
    ///
    /// # Returns
    ///
    /// Returns a `CompletionResult` on success, or an `LlmError` on failure.
    ///
    /// # Note
    ///
    /// The `on_token` callback is called for each token as it's generated,
    /// enabling streaming responses. For non-streaming providers, this may be
    /// called once with the complete response.
    fn complete(
        &mut self,
        messages: &[Message],
        model: &str,
        config: &GenerationConfig,
        on_token: impl FnMut(&str),
    ) -> Result<CompletionResult, LlmError>;

    /// Validate that the API key is valid and the provider is accessible.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the API key is valid, or an `LlmError` if validation fails.
    fn validate_api_key(&self) -> Result<(), LlmError>;
}
