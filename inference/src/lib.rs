#![no_std]

extern crate alloc;

pub mod gguf;
pub mod error;
pub mod tokenizer;
pub mod tensor;
pub mod ops;
pub mod simd;
pub mod transformer;
pub mod sampling;
pub mod model;

pub use gguf::{GgufFile, MetadataValue, TensorInfo};
pub use error::{ModelError, ParseError, TokenizerError};
pub use tokenizer::{Tokenizer, SpecialTokens};
pub use tensor::{Tensor, TensorData, BlockQ4K, QK_K};
pub use transformer::{
    Transformer, ModelConfig, ModelWeights, TransformerLayerWeights,
    EmbeddingWeights, OutputWeights, KvCache,
};
pub use model::LocalModel;
