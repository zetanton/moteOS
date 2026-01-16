#![no_std]
#![feature(alloc)]

extern crate alloc;

pub mod gguf;
pub mod error;

pub use gguf::{GgufFile, MetadataValue, TensorInfo};
pub use error::{ModelError, ParseError};
