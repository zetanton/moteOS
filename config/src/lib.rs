#![no_std]

extern crate alloc;

pub mod toml;
pub mod error;

pub use toml::{TomlParser, Value};
pub use error::ConfigError;
