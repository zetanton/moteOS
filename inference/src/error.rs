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

impl From<ParseError> for ModelError {
    fn from(err: ParseError) -> Self {
        ModelError::Parse(err)
    }
}

impl core::fmt::Display for ModelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModelError::Parse(e) => write!(f, "Parse error: {}", e),
            ModelError::TensorNotFound(name) => write!(f, "Tensor not found: {}", name),
            ModelError::MetadataNotFound(key) => write!(f, "Metadata not found: {}", key),
            ModelError::InvalidTensorAccess(msg) => write!(f, "Invalid tensor access: {}", msg),
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

#[cfg(feature = "std")]
impl std::error::Error for ModelError {}
#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
