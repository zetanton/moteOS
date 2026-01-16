#![no_std]

extern crate alloc;

pub mod toml;
pub mod error;
pub mod storage;

pub use toml::{TomlParser, Value};
pub use error::ConfigError;
pub use storage::{ConfigStorage, efi::EfiConfigStorage};
