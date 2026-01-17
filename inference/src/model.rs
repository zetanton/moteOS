use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::transformer::{Transformer, KvCache, ModelConfig, ModelWeights};
use crate::tokenizer::Tokenizer;
use crate::sampling::sample;
use crate::error::ModelError;

use llm::{LlmProvider, ModelInfo, Message, Role, GenerationConfig, CompletionResult, FinishReason, LlmError};

/// Local LLM model for inference
pub struct LocalModel {
    transformer: Transformer,
    tokenizer: Tokenizer,
    kv_cache: KvCache,
}

impl LocalModel {
    /// Create a new LocalModel
    pub fn new(weights: ModelWeights, config: ModelConfig, tokenizer: Tokenizer) -> Self {
        let kv_cache = KvCache::new(
            config.num_layers,
            config.max_seq_len,
            config.num_heads,
            config.head_dim,
        );
        
        Self {
            transformer: Transformer::new(weights, config),
            tokenizer,
            kv_cache,
        }
    }

    /// Format messages into a prompt string (ChatML format)
    fn format_prompt(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();
        
        for msg in messages {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            
            prompt.push_str("<|im_start|>");
            prompt.push_str(role_str);
            prompt.push_str("\n");
            prompt.push_str(&msg.content);
            prompt.push_str("<|im_end|>\n");
        }
        
        // Add the start of the assistant's response if the last message wasn't from assistant
        if messages.last().map(|m| m.role != Role::Assistant).unwrap_or(true) {
            prompt.push_str("<|im_start|>assistant\n");
        }
        
        prompt
    }

    /// Generate text based on a prompt
    pub fn generate(
        &mut self,
        prompt: &str,
        max_tokens: Option<usize>,
        temperature: f32,
        top_p: Option<f32>,
        top_k: Option<usize>,
        stop_sequences: &[String],
        rng_seed: u64,
        mut on_token: impl FnMut(&str),
    ) -> Result<(String, FinishReason), ModelError> {
        // 1. Tokenize prompt
        let tokens = self.tokenizer.encode(prompt);
        if tokens.is_empty() {
            return Err(ModelError::InvalidInput("Empty prompt".into()));
        }

        // 2. Reset KV cache for new generation
        self.kv_cache.reset();

        // 3. Prefill phase
        // Process all prompt tokens except the last one to fill KV cache
        if tokens.len() > 1 {
            self.transformer.forward(&tokens[..tokens.len() - 1], &mut self.kv_cache)?;
        }

        // Process the last token of the prompt to get the first generation logits
        let last_token = *tokens.last().unwrap();
        let mut last_logits = self.transformer.forward(&[last_token], &mut self.kv_cache)?;

        // 4. Generation loop
        let mut generated_tokens = Vec::new();
        let mut generated_text = String::new();
        let max_gen = max_tokens.unwrap_or(self.transformer.config().max_seq_len - tokens.len());
        let mut current_seed = rng_seed;
        let mut finish_reason = FinishReason::Length;

        for _ in 0..max_gen {
            // Sample next token
            let next_token = sample(
                &mut last_logits,
                temperature,
                top_p,
                top_k,
                current_seed,
            );
            
            // Advance seed for next token
            current_seed = xorshift64(current_seed);

            // Check for EOS
            if Some(next_token) == self.tokenizer.eos_token() {
                finish_reason = FinishReason::Stop;
                break;
            }

            // Decode and stream the token
            let token_str = self.tokenizer.decode(&[next_token]);
            
            // Check for stop sequences
            let mut found_stop = false;
            for stop in stop_sequences {
                if token_str.contains(stop) || (generated_text.clone() + &token_str).contains(stop) {
                    found_stop = true;
                    break;
                }
            }
            
            if found_stop {
                finish_reason = FinishReason::Stop;
                break;
            }

            on_token(&token_str);
            
            generated_text.push_str(&token_str);
            generated_tokens.push(next_token);

            // Check if we've reached the max sequence length
            if self.kv_cache.current_pos() >= self.transformer.config().max_seq_len {
                finish_reason = FinishReason::Length;
                break;
            }

            // Forward pass for the next token
            last_logits = self.transformer.forward(&[next_token], &mut self.kv_cache)?;
        }

        Ok((generated_text, finish_reason))
    }
}

impl LlmProvider for LocalModel {
    fn name(&self) -> &str {
        "Local (SmolLM)"
    }

    fn models(&self) -> &[ModelInfo] {
        // Return empty slice as String static initialization is not possible
        &[]
    }

    fn default_model(&self) -> &str {
        "smollm-360m"
    }

    fn complete(
        &mut self,
        messages: &[Message],
        _model: &str,
        config: &GenerationConfig,
        on_token: impl FnMut(&str),
    ) -> Result<CompletionResult, LlmError> {
        let prompt = self.format_prompt(messages);
        
        // Use a fixed seed for reproducibility or a pseudo-random one if we had a clock
        let seed = 42; 

        match self.generate(
            &prompt,
            config.max_tokens,
            config.temperature,
            config.top_p,
            config.top_k,
            &config.stop_sequences,
            seed,
            on_token,
        ) {
            Ok((text, finish_reason)) => {
                Ok(CompletionResult {
                    text,
                    tokens_used: Some(self.kv_cache.current_pos()),
                    finish_reason,
                })
            }
            Err(e) => Err(LlmError::Other(format!("Inference error: {:?}", e))),
        }
    }

    fn validate_api_key(&self) -> Result<(), LlmError> {
        // Local model doesn't need an API key
        Ok(())
    }
}

fn xorshift64(mut seed: u64) -> u64 {
    seed ^= seed << 13;
    seed ^= seed >> 7;
    seed ^= seed << 17;
    seed
}
