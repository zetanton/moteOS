//! BPE tokenizer implementation for loading vocabulary from GGUF files
//!
//! This module provides a byte-pair encoding (BPE) tokenizer that can load
//! vocabulary and merge rules from GGUF model files and encode/decode text.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::error::{ModelError, TokenizerError};
use crate::gguf::{GgufFile, MetadataValue};

/// BPE (Byte Pair Encoding) tokenizer
///
/// This tokenizer implements the Byte Pair Encoding algorithm for converting
/// text to token IDs and vice versa. It loads vocabulary and merge rules from
/// GGUF model files.
///
/// # Example
///
/// ```no_run
/// use inference::{GgufFile, Tokenizer};
///
/// let gguf = GgufFile::parse(model_data).unwrap();
/// let tokenizer = Tokenizer::from_gguf(&gguf).unwrap();
///
/// // Encode text to tokens
/// let tokens = tokenizer.encode("Hello, world!");
///
/// // Decode tokens back to text
/// let text = tokenizer.decode(&tokens);
/// ```
pub struct Tokenizer {
    /// Vocabulary mapping from token string to token ID
    vocab: BTreeMap<String, u32>,
    /// Reverse mapping from token ID to token string
    id_to_token: BTreeMap<u32, String>,
    /// BPE merge rules (pairs of tokens to merge)
    merges: Vec<(String, String)>,
    /// Special tokens
    special_tokens: SpecialTokens,
}

/// Special tokens used by the tokenizer
#[derive(Debug, Clone)]
pub struct SpecialTokens {
    /// Beginning of sequence token
    pub bos_token: Option<u32>,
    /// End of sequence token
    pub eos_token: Option<u32>,
    /// Padding token
    pub pad_token: Option<u32>,
    /// Unknown token
    pub unk_token: Option<u32>,
}

impl Tokenizer {
    /// Create a new tokenizer from vocabulary and merges
    pub fn new(
        vocab: BTreeMap<String, u32>,
        merges: Vec<(String, String)>,
        special_tokens: SpecialTokens,
    ) -> Self {
        // Build reverse mapping
        let mut id_to_token = BTreeMap::new();
        for (token, id) in vocab.iter() {
            id_to_token.insert(*id, token.clone());
        }

        Self {
            vocab,
            id_to_token,
            merges,
            special_tokens,
        }
    }

    /// Load tokenizer from GGUF file
    ///
    /// This reads the tokenizer vocabulary and configuration from the GGUF
    /// metadata section. The GGUF format stores tokenizer data in the following keys:
    /// - `tokenizer.ggml.model`: The tokenizer type (should be "gpt2" or "llama")
    /// - `tokenizer.ggml.tokens`: Array of token strings
    /// - `tokenizer.ggml.token_type`: Array of token types (optional)
    /// - `tokenizer.ggml.merges`: Array of BPE merge rules (optional)
    /// - `tokenizer.ggml.bos_token_id`: Beginning of sequence token ID
    /// - `tokenizer.ggml.eos_token_id`: End of sequence token ID
    /// - `tokenizer.ggml.unknown_token_id`: Unknown token ID (optional)
    /// - `tokenizer.ggml.padding_token_id`: Padding token ID (optional)
    pub fn from_gguf(gguf: &GgufFile) -> Result<Self, ModelError> {
        // Extract tokens array
        let tokens = gguf
            .get_metadata("tokenizer.ggml.tokens")
            .ok_or_else(|| ModelError::Tokenizer(TokenizerError::MissingVocab))?;

        let token_strings = match tokens {
            MetadataValue::Array(arr) => {
                let mut strings = Vec::new();
                for item in arr {
                    match item {
                        MetadataValue::String(s) => strings.push(s.clone()),
                        _ => return Err(ModelError::Tokenizer(TokenizerError::InvalidVocabFormat)),
                    }
                }
                strings
            }
            _ => return Err(ModelError::Tokenizer(TokenizerError::InvalidVocabFormat)),
        };

        // Build vocabulary mapping
        let mut vocab = BTreeMap::new();
        for (id, token) in token_strings.iter().enumerate() {
            vocab.insert(token.clone(), id as u32);
        }

        // Extract merge rules (optional, may not exist for all models)
        let merges = if let Some(merges_meta) = gguf.get_metadata("tokenizer.ggml.merges") {
            match merges_meta {
                MetadataValue::Array(arr) => {
                    let mut merge_rules = Vec::new();
                    for item in arr {
                        match item {
                            MetadataValue::String(s) => {
                                // Merge rules are typically in format "token1 token2"
                                let parts: Vec<&str> = s.split_whitespace().collect();
                                if parts.len() == 2 {
                                    merge_rules.push((parts[0].to_string(), parts[1].to_string()));
                                }
                            }
                            _ => {}
                        }
                    }
                    merge_rules
                }
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        // Extract special tokens
        let bos_token = Self::extract_token_id(gguf, "tokenizer.ggml.bos_token_id");
        let eos_token = Self::extract_token_id(gguf, "tokenizer.ggml.eos_token_id");
        let pad_token = Self::extract_token_id(gguf, "tokenizer.ggml.padding_token_id");
        let unk_token = Self::extract_token_id(gguf, "tokenizer.ggml.unknown_token_id");

        let special_tokens = SpecialTokens {
            bos_token,
            eos_token,
            pad_token,
            unk_token,
        };

        Ok(Self::new(vocab, merges, special_tokens))
    }

    /// Helper to extract a token ID from metadata
    fn extract_token_id(gguf: &GgufFile, key: &str) -> Option<u32> {
        match gguf.get_metadata(key) {
            Some(MetadataValue::UInt32(id)) => Some(*id),
            Some(MetadataValue::Int32(id)) if *id >= 0 => Some(*id as u32),
            Some(MetadataValue::UInt64(id)) => Some(*id as u32),
            Some(MetadataValue::Int64(id)) if *id >= 0 => Some(*id as u32),
            _ => None,
        }
    }

    /// Encode text to token IDs
    ///
    /// This performs BPE tokenization by:
    /// 1. Splitting text into individual bytes/characters
    /// 2. Applying BPE merge rules iteratively
    /// 3. Converting final tokens to IDs
    ///
    /// # Arguments
    /// * `text` - The input text to tokenize
    ///
    /// # Returns
    /// A vector of token IDs representing the input text
    pub fn encode(&self, text: &str) -> Vec<u32> {
        if text.is_empty() {
            return Vec::new();
        }

        // Convert text to initial byte-level tokens
        // In proper BPE, we start with each byte as a separate token
        let mut tokens = Vec::new();
        for &byte in text.as_bytes() {
            // First, try byte-level representation (most GGUF models use <0xXX> format)
            let byte_token = format!("<0x{:02X}>", byte);

            if self.vocab.contains_key(&byte_token) {
                // Use byte-level token if it exists
                tokens.push(byte_token);
            } else if byte < 128 {
                // For ASCII bytes (0-127), try single-character UTF-8 representation
                // This is safe because ASCII bytes are valid UTF-8
                let char_str = format!("{}", byte as char);
                if self.vocab.contains_key(&char_str) {
                    tokens.push(char_str);
                } else {
                    // Fallback to byte token format even if not in vocab
                    tokens.push(byte_token);
                }
            } else {
                // For non-ASCII bytes (128-255), always use byte-level representation
                // Don't cast to char as this can produce invalid UTF-8
                tokens.push(byte_token);
            }
        }

        // Apply BPE merge rules with proper priority ordering
        if !self.merges.is_empty() {
            tokens = self.apply_merges(tokens);
        }

        // Convert tokens to IDs
        let mut token_ids = Vec::new();
        for token in tokens {
            if let Some(&id) = self.vocab.get(&token) {
                token_ids.push(id);
            } else if let Some(unk_id) = self.special_tokens.unk_token {
                // Use unknown token for out-of-vocabulary tokens
                token_ids.push(unk_id);
            }
            // If no unknown token is defined, skip the token
        }

        token_ids
    }

    /// Apply BPE merge rules to a sequence of tokens
    ///
    /// This implements the BPE algorithm by repeatedly applying merge rules
    /// in priority order. The merge rules in the `merges` vector are ordered
    /// by priority (earlier merges have higher priority).
    ///
    /// The algorithm:
    /// 1. For each merge rule (in order)
    /// 2. Scan through the token sequence
    /// 3. Merge all occurrences of the pair
    /// 4. Continue until all merge rules are applied
    fn apply_merges(&self, mut tokens: Vec<String>) -> Vec<String> {
        // Apply each merge rule in priority order
        // The order of merges in the vector represents their priority
        for (left, right) in &self.merges {
            let mut i = 0;

            // Apply this merge rule to all matching pairs in the sequence
            while i < tokens.len().saturating_sub(1) {
                if tokens[i] == *left && tokens[i + 1] == *right {
                    // Merge the two tokens
                    let merged = format!("{}{}", left, right);
                    tokens[i] = merged;
                    tokens.remove(i + 1);

                    // Don't increment i here - check if the merged token can be merged again
                    // This handles cases where a merge creates a new mergeable pair
                } else {
                    i += 1;
                }
            }
        }

        tokens
    }

    /// Decode token IDs back to text
    ///
    /// This converts a sequence of token IDs back to the original text by:
    /// 1. Looking up each token ID in the vocabulary
    /// 2. Concatenating the token strings
    /// 3. Handling special tokens appropriately
    ///
    /// Special token handling:
    /// - BOS (beginning of sequence): Skipped in output
    /// - EOS (end of sequence): Stops decoding when encountered
    /// - PAD (padding): Skipped in output
    /// - UNK (unknown): Included in output if present
    ///
    /// # Arguments
    /// * `token_ids` - The token IDs to decode
    ///
    /// # Returns
    /// The decoded text string
    pub fn decode(&self, token_ids: &[u32]) -> String {
        let mut result = String::new();

        for &token_id in token_ids {
            // Stop decoding if we encounter EOS token
            if self.special_tokens.eos_token == Some(token_id) {
                break;
            }

            // Skip BOS and PAD tokens in output
            if self.special_tokens.bos_token == Some(token_id)
                || self.special_tokens.pad_token == Some(token_id)
            {
                continue;
            }

            // Include all other tokens (including UNK if it appears in the sequence)
            if let Some(token) = self.id_to_token.get(&token_id) {
                result.push_str(token);
            }
            // Skip unknown token IDs that aren't in vocabulary
        }

        result
    }

    /// Check if a token ID is a special token (BOS, EOS, PAD, or UNK)
    pub fn is_special_token(&self, token_id: u32) -> bool {
        self.special_tokens.bos_token == Some(token_id)
            || self.special_tokens.eos_token == Some(token_id)
            || self.special_tokens.pad_token == Some(token_id)
            || self.special_tokens.unk_token == Some(token_id)
    }

    /// Get the beginning of sequence token ID
    pub fn bos_token(&self) -> Option<u32> {
        self.special_tokens.bos_token
    }

    /// Get the end of sequence token ID
    pub fn eos_token(&self) -> Option<u32> {
        self.special_tokens.eos_token
    }

    /// Get the padding token ID
    pub fn pad_token(&self) -> Option<u32> {
        self.special_tokens.pad_token
    }

    /// Get the unknown token ID
    pub fn unk_token(&self) -> Option<u32> {
        self.special_tokens.unk_token
    }

    /// Get the vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    /// Check if a token string exists in the vocabulary
    pub fn contains_token(&self, token: &str) -> bool {
        self.vocab.contains_key(token)
    }

    /// Get the token ID for a specific token string
    pub fn token_to_id(&self, token: &str) -> Option<u32> {
        self.vocab.get(token).copied()
    }

    /// Get the token string for a specific token ID
    pub fn id_to_token_str(&self, id: u32) -> Option<&str> {
        self.id_to_token.get(&id).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_creation() {
        let mut vocab = BTreeMap::new();
        vocab.insert("hello".to_string(), 0);
        vocab.insert("world".to_string(), 1);

        let special_tokens = SpecialTokens {
            bos_token: Some(2),
            eos_token: Some(3),
            pad_token: None,
            unk_token: None,
        };

        let tokenizer = Tokenizer::new(vocab, Vec::new(), special_tokens);

        assert_eq!(tokenizer.vocab_size(), 2);
        assert_eq!(tokenizer.bos_token(), Some(2));
        assert_eq!(tokenizer.eos_token(), Some(3));
    }

    #[test]
    fn test_token_lookup() {
        let mut vocab = BTreeMap::new();
        vocab.insert("hello".to_string(), 0);
        vocab.insert("world".to_string(), 1);

        let special_tokens = SpecialTokens {
            bos_token: None,
            eos_token: None,
            pad_token: None,
            unk_token: None,
        };

        let tokenizer = Tokenizer::new(vocab, Vec::new(), special_tokens);

        assert_eq!(tokenizer.token_to_id("hello"), Some(0));
        assert_eq!(tokenizer.token_to_id("world"), Some(1));
        assert_eq!(tokenizer.token_to_id("unknown"), None);

        assert_eq!(tokenizer.id_to_token_str(0), Some("hello"));
        assert_eq!(tokenizer.id_to_token_str(1), Some("world"));
        assert_eq!(tokenizer.id_to_token_str(999), None);
    }
}
