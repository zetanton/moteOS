# Inference Crate

This crate provides GGUF file format parsing and local model inference capabilities for moteOS.

## Features

- **GGUF Parser**: Parse GGUF header, extract metadata, and load tensor data
- **Metadata Access**: Query model metadata (version, architecture, tokenizer info, etc.)
- **Tensor Access**: Load tensor weights by name with proper offset calculation
- **no_std Compatible**: Designed for embedded/unikernel environments

## GGUF Format Support

The parser supports GGUF version 3 format with:
- Full metadata parsing (strings, integers, floats, booleans, arrays)
- Tensor information extraction
- Tensor data loading
- Support for various tensor types (F32, F16, quantized types)

## Usage

```rust
use inference::GgufFile;

// Load GGUF file
let file_data = /* read from file or embedded */;
let gguf = GgufFile::parse(file_data)?;

// Access metadata
if let Some(version) = gguf.get_metadata("general.version") {
    match version {
        MetadataValue::String(v) => println!("Version: {}", v),
        _ => {}
    }
}

// Get tokenizer info
if let Some(vocab_size) = gguf.get_metadata("tokenizer.ggml.vocab_size") {
    if let MetadataValue::UInt32(size) = vocab_size {
        println!("Vocabulary size: {}", size);
    }
}

// Load tensor data
let tensor_data = gguf.get_tensor("blk.0.attn_q.weight")?;
// tensor_data is a &[u8] slice containing the raw tensor bytes
```

## Tensor Types

The parser supports common GGUF tensor types:
- F32, F16 (unquantized)
- Q4_0, Q4_1, Q4_K, Q4_K_M (4-bit quantized)
- Q5_0, Q5_1, Q5_K, Q5_K_M (5-bit quantized)
- Q6_K, Q6_K_M (6-bit quantized)
- Q8_0, Q8_1 (8-bit quantized)
- Q2_K, Q3_K (2-bit and 3-bit quantized)

## Implementation Notes

- Tensor size calculation for quantized types is simplified. For production use, you may want to implement the full quantized tensor block structure calculations.
- The parser loads the entire file into memory. For very large models, consider implementing streaming or memory-mapped access.

## See Also

- Section 3.6.2 of `docs/TECHNICAL_SPECIFICATIONS.md` for the full specification
- Section 6.2 of `docs/TECHNICAL_SPECIFICATIONS.md` for the interface contract
