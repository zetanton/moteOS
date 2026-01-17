use alloc::string::String;

/// Errors that can occur during model operations
#[derive(Debug, Clone)]
pub enum ModelError {
    /// Parser errors
    Parse(ParseError),
    /// Invalid tensor name
    TensorNotFound(String),
    /// Invalid metadata key
    MetadataNotFound(String),
    /// Invalid tensor data access
    InvalidTensorAccess(String),
    /// Tokenizer errors
    Tokenizer(TokenizerError),
}

/// Errors that can occur during GGUF parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Invalid magic number
    InvalidMagic,
    /// Unsupported version
    UnsupportedVersion(u32),
    /// Unexpected end of file
    UnexpectedEof,
    /// Invalid metadata type
    InvalidMetadataType(u32),
    /// Invalid string encoding
    InvalidStringEncoding,
    /// Invalid tensor type
    InvalidTensorType(u32),
    /// Invalid alignment
    InvalidAlignment,
    /// General parse error with message
    General(String),
}

/// Errors that can occur during tokenization
#[derive(Debug, Clone)]
pub enum TokenizerError {
    /// Missing vocabulary data in GGUF file
    MissingVocab,
    /// Invalid vocabulary format
    InvalidVocabFormat,
    /// Invalid merge rules format
    InvalidMergeFormat,
    /// Token not found in vocabulary
    TokenNotFound(String),
    /// General tokenizer error with message
    General(String),
}

impl From<ParseError> for ModelError {
    fn from(err: ParseError) -> Self {
        ModelError::Parse(err)
    }
}

impl From<TokenizerError> for ModelError {
    fn from(err: TokenizerError) -> Self {
        ModelError::Tokenizer(err)
    }
}

impl core::fmt::Display for ModelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModelError::Parse(e) => write!(f, "Parse error: {}", e),
            ModelError::TensorNotFound(name) => write!(f, "Tensor not found: {}", name),
            ModelError::MetadataNotFound(key) => write!(f, "Metadata not found: {}", key),
            ModelError::InvalidTensorAccess(msg) => write!(f, "Invalid tensor access: {}", msg),
            ModelError::Tokenizer(e) => write!(f, "Tokenizer error: {}", e),
        }
    }
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::InvalidMagic => write!(f, "Invalid GGUF magic number"),
            ParseError::UnsupportedVersion(v) => write!(f, "Unsupported GGUF version: {}", v),
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidMetadataType(t) => write!(f, "Invalid metadata type: {}", t),
            ParseError::InvalidStringEncoding => write!(f, "Invalid string encoding"),
            ParseError::InvalidTensorType(t) => write!(f, "Invalid tensor type: {}", t),
            ParseError::InvalidAlignment => write!(f, "Invalid alignment"),
            ParseError::General(msg) => write!(f, "{}", msg),
        }
    }
}

impl core::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TokenizerError::MissingVocab => write!(f, "Missing vocabulary data in GGUF file"),
            TokenizerError::InvalidVocabFormat => write!(f, "Invalid vocabulary format"),
            TokenizerError::InvalidMergeFormat => write!(f, "Invalid merge rules format"),
            TokenizerError::TokenNotFound(token) => write!(f, "Token not found: {}", token),
            TokenizerError::General(msg) => write!(f, "{}", msg),
        }
    }
}

// std::error::Error is only available with std feature
// In no_std, we only implement Display
