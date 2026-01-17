use alloc::collections::BTreeMap;
use alloc::fmt;
use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{ModelError, ParseError};

/// GGUF file magic number
const GGUF_MAGIC: &[u8; 4] = b"GGUF";
/// Supported GGUF version
const GGUF_VERSION: u32 = 3;

/// GGUF file parser and container
///
/// This struct parses and provides access to GGUF (GPT-Generated Unified Format) files.
/// GGUF is the format used by llama.cpp and related projects for storing model weights
/// and metadata.
///
/// # Example
///
/// ```no_run
/// use inference::GgufFile;
///
/// let file_data = std::fs::read("model.gguf").unwrap();
/// let gguf = GgufFile::parse(file_data).unwrap();
///
/// // Access metadata
/// if let Some(version) = gguf.get_metadata("general.version") {
///     println!("Model version: {:?}", version);
/// }
///
/// // Access tensor data
/// let tensor_data = gguf.get_tensor("blk.0.attn_q.weight").unwrap();
/// ```
pub struct GgufFile {
    /// File version
    version: u32,
    /// Metadata key-value pairs
    metadata: BTreeMap<String, MetadataValue>,
    /// Tensor information
    tensors: Vec<TensorInfo>,
    /// Raw file data (needed for tensor access)
    data: Vec<u8>,
    /// Offset where tensor data section starts
    tensor_data_offset: usize,
}

/// Metadata value types
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    /// UTF-8 string
    String(String),
    /// 32-bit unsigned integer
    UInt32(u32),
    /// 32-bit signed integer
    Int32(i32),
    /// 32-bit float
    Float32(f32),
    /// 64-bit unsigned integer
    UInt64(u64),
    /// 64-bit signed integer
    Int64(i64),
    /// 64-bit float
    Float64(f64),
    /// Boolean
    Bool(bool),
    /// Array of metadata values
    Array(Vec<MetadataValue>),
}

/// Tensor information
#[derive(Debug, Clone)]
pub struct TensorInfo {
    /// Tensor name
    pub name: String,
    /// Tensor dimensions
    pub dimensions: Vec<u64>,
    /// Tensor type (GGUF tensor type enum)
    pub tensor_type: u32,
    /// Offset in the tensor data section
    pub offset: u64,
    /// Size of tensor data in bytes
    pub size: usize,
}

/// Metadata value type IDs (from GGUF spec)
#[repr(u32)]
enum MetadataType {
    UInt8 = 0,
    Int8 = 1,
    UInt16 = 2,
    Int16 = 3,
    UInt32 = 4,
    Int32 = 5,
    Float32 = 6,
    Bool = 7,
    String = 8,
    Array = 9,
    UInt64 = 10,
    Int64 = 11,
    Float64 = 12,
}

impl GgufFile {
    /// Parse a GGUF file from raw bytes
    pub fn parse(data: Vec<u8>) -> Result<Self, ModelError> {
        let mut offset = 0;

        // Parse magic number
        if data.len() < 4 {
            return Err(ParseError::UnexpectedEof.into());
        }
        if &data[0..4] != GGUF_MAGIC {
            return Err(ParseError::InvalidMagic.into());
        }
        offset += 4;

        // Parse version
        let version = Self::read_u32(&data, &mut offset)?;
        if version != GGUF_VERSION {
            return Err(ParseError::UnsupportedVersion(version).into());
        }

        // Parse tensor count
        let tensor_count = Self::read_u64(&data, &mut offset)?;

        // Parse metadata KV count
        let metadata_kv_count = Self::read_u64(&data, &mut offset)?;

        // Parse metadata section
        let mut metadata = BTreeMap::new();
        for _ in 0..metadata_kv_count {
            let (key, value) = Self::parse_metadata_kv(&data, &mut offset)?;
            metadata.insert(key, value);
        }

        // Parse tensor info section
        let mut tensors = Vec::new();
        let mut tensor_data_offset = offset;

        for _ in 0..tensor_count {
            // Parse tensor name
            let name_len = Self::read_u64(&data, &mut offset)?;
            let name = Self::read_string(&data, &mut offset, name_len as usize)?;

            // Parse dimensions
            let n_dims = Self::read_u32(&data, &mut offset)?;
            let mut dimensions = Vec::with_capacity(n_dims as usize);
            for _ in 0..n_dims {
                dimensions.push(Self::read_u64(&data, &mut offset)?);
            }

            // Parse tensor type
            let tensor_type = Self::read_u32(&data, &mut offset)?;

            // Parse offset
            let tensor_offset = Self::read_u64(&data, &mut offset)?;

            // Calculate tensor size
            let size = Self::calculate_tensor_size(&dimensions, tensor_type)?;

            tensors.push(TensorInfo {
                name,
                dimensions,
                tensor_type,
                offset: tensor_offset,
                size,
            });
        }

        // Find the actual start of tensor data (after all tensor info entries)
        // Tensor data starts at the offset of the first tensor
        if !tensors.is_empty() {
            tensor_data_offset = tensors[0].offset as usize;
        }

        Ok(GgufFile {
            version,
            metadata,
            tensors,
            data,
            tensor_data_offset,
        })
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&MetadataValue> {
        self.metadata.get(key)
    }

    /// Get all metadata keys
    pub fn metadata_keys(&self) -> impl Iterator<Item = &String> {
        self.metadata.keys()
    }

    /// Get tensor data by name
    pub fn get_tensor(&self, name: &str) -> Result<&[u8], ModelError> {
        let tensor = self
            .tensors
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| ModelError::TensorNotFound(String::from(name)))?;

        let start = self.tensor_data_offset + tensor.offset as usize;
        let end = start + tensor.size;

        if end > self.data.len() {
            let msg = fmt::format(format_args!("Tensor {} extends beyond file end", name));
            return Err(ModelError::InvalidTensorAccess(msg));
        }

        Ok(&self.data[start..end])
    }

    /// Get tensor information by name
    pub fn get_tensor_info(&self, name: &str) -> Option<&TensorInfo> {
        self.tensors.iter().find(|t| t.name == name)
    }

    /// Get all tensor names
    pub fn tensor_names(&self) -> impl Iterator<Item = &String> {
        self.tensors.iter().map(|t| &t.name)
    }

    /// Get version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get number of tensors
    pub fn tensor_count(&self) -> usize {
        self.tensors.len()
    }

    // Helper functions for parsing

    fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32, ParseError> {
        if *offset + 4 > data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &data[*offset..*offset + 4];
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *offset += 4;
        Ok(value)
    }

    fn read_u64(data: &[u8], offset: &mut usize) -> Result<u64, ParseError> {
        if *offset + 8 > data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &data[*offset..*offset + 8];
        let value = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        *offset += 8;
        Ok(value)
    }

    fn read_i32(data: &[u8], offset: &mut usize) -> Result<i32, ParseError> {
        if *offset + 4 > data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &data[*offset..*offset + 4];
        let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *offset += 4;
        Ok(value)
    }

    fn read_i64(data: &[u8], offset: &mut usize) -> Result<i64, ParseError> {
        if *offset + 8 > data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &data[*offset..*offset + 8];
        let value = i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        *offset += 8;
        Ok(value)
    }

    fn read_f32(data: &[u8], offset: &mut usize) -> Result<f32, ParseError> {
        if *offset + 4 > data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &data[*offset..*offset + 4];
        let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *offset += 4;
        Ok(value)
    }

    fn read_f64(data: &[u8], offset: &mut usize) -> Result<f64, ParseError> {
        if *offset + 8 > data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &data[*offset..*offset + 8];
        let value = f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        *offset += 8;
        Ok(value)
    }

    fn read_string(data: &[u8], offset: &mut usize, len: usize) -> Result<String, ParseError> {
        if *offset + len > data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &data[*offset..*offset + len];
        *offset += len;

        // GGUF strings are null-terminated, but we read the full length
        // Find null terminator if present
        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(len);
        let str_bytes = &bytes[..null_pos];

        String::from_utf8(str_bytes.to_vec()).map_err(|_| ParseError::InvalidStringEncoding)
    }

    fn parse_metadata_kv(
        data: &[u8],
        offset: &mut usize,
    ) -> Result<(String, MetadataValue), ParseError> {
        // Read key length
        let key_len = Self::read_u64(data, offset)?;

        // Read key
        let key = Self::read_string(data, offset, key_len as usize)?;

        // Read value type
        let value_type = Self::read_u32(data, offset)?;

        // Read value based on type
        let value = match value_type {
            t if t == MetadataType::UInt8 as u32 => {
                if *offset + 1 > data.len() {
                    return Err(ParseError::UnexpectedEof);
                }
                let v = data[*offset];
                *offset += 1;
                MetadataValue::UInt32(v as u32)
            }
            t if t == MetadataType::Int8 as u32 => {
                if *offset + 1 > data.len() {
                    return Err(ParseError::UnexpectedEof);
                }
                let v = data[*offset] as i8;
                *offset += 1;
                MetadataValue::Int32(v as i32)
            }
            t if t == MetadataType::UInt16 as u32 => {
                if *offset + 2 > data.len() {
                    return Err(ParseError::UnexpectedEof);
                }
                let bytes = &data[*offset..*offset + 2];
                let v = u16::from_le_bytes([bytes[0], bytes[1]]) as u32;
                *offset += 2;
                MetadataValue::UInt32(v)
            }
            t if t == MetadataType::Int16 as u32 => {
                if *offset + 2 > data.len() {
                    return Err(ParseError::UnexpectedEof);
                }
                let bytes = &data[*offset..*offset + 2];
                let v = i16::from_le_bytes([bytes[0], bytes[1]]) as i32;
                *offset += 2;
                MetadataValue::Int32(v)
            }
            t if t == MetadataType::UInt32 as u32 => {
                MetadataValue::UInt32(Self::read_u32(data, offset)?)
            }
            t if t == MetadataType::Int32 as u32 => {
                MetadataValue::Int32(Self::read_i32(data, offset)?)
            }
            t if t == MetadataType::Float32 as u32 => {
                MetadataValue::Float32(Self::read_f32(data, offset)?)
            }
            t if t == MetadataType::Bool as u32 => {
                if *offset + 1 > data.len() {
                    return Err(ParseError::UnexpectedEof);
                }
                let v = data[*offset] != 0;
                *offset += 1;
                MetadataValue::Bool(v)
            }
            t if t == MetadataType::String as u32 => {
                let str_len = Self::read_u64(data, offset)?;
                let s = Self::read_string(data, offset, str_len as usize)?;
                MetadataValue::String(s)
            }
            t if t == MetadataType::Array as u32 => {
                let array_type = Self::read_u32(data, offset)?;
                let array_len = Self::read_u64(data, offset)?;
                let mut array = Vec::with_capacity(array_len as usize);

                for _ in 0..array_len {
                    let value = match array_type {
                        t if t == MetadataType::UInt32 as u32 => {
                            MetadataValue::UInt32(Self::read_u32(data, offset)?)
                        }
                        t if t == MetadataType::Int32 as u32 => {
                            MetadataValue::Int32(Self::read_i32(data, offset)?)
                        }
                        t if t == MetadataType::Float32 as u32 => {
                            MetadataValue::Float32(Self::read_f32(data, offset)?)
                        }
                        t if t == MetadataType::Bool as u32 => {
                            if *offset + 1 > data.len() {
                                return Err(ParseError::UnexpectedEof);
                            }
                            let v = data[*offset] != 0;
                            *offset += 1;
                            MetadataValue::Bool(v)
                        }
                        t if t == MetadataType::String as u32 => {
                            let str_len = Self::read_u64(data, offset)?;
                            let s = Self::read_string(data, offset, str_len as usize)?;
                            MetadataValue::String(s)
                        }
                        _ => {
                            return Err(ParseError::InvalidMetadataType(array_type));
                        }
                    };
                    array.push(value);
                }
                MetadataValue::Array(array)
            }
            t if t == MetadataType::UInt64 as u32 => {
                MetadataValue::UInt64(Self::read_u64(data, offset)?)
            }
            t if t == MetadataType::Int64 as u32 => {
                MetadataValue::Int64(Self::read_i64(data, offset)?)
            }
            t if t == MetadataType::Float64 as u32 => {
                MetadataValue::Float64(Self::read_f64(data, offset)?)
            }
            _ => {
                return Err(ParseError::InvalidMetadataType(value_type));
            }
        };

        Ok((key, value))
    }

    /// Calculate tensor size in bytes based on dimensions and type
    ///
    /// Note: For quantized types (Q4_K_M, Q5_K, etc.), the actual size
    /// calculation is more complex due to block structures. This implementation
    /// provides a simplified calculation. For production use, you may want to
    /// implement the full quantized tensor size calculation.
    fn calculate_tensor_size(dimensions: &[u64], tensor_type: u32) -> Result<usize, ParseError> {
        // Calculate total elements
        let elements: u64 = dimensions.iter().product();

        // Get element size based on tensor type
        // GGUF tensor types (from llama.cpp/ggml.h)
        // Note: Quantized types have block structures, so this is approximate
        let element_size = match tensor_type {
            0 => 4,   // F32 (4 bytes per element)
            1 => 2,   // F16 (2 bytes per element)
            2 => 18,  // Q4_0 (block size 32, 18 bytes per block)
            3 => 20,  // Q4_1 (block size 32, 20 bytes per block)
            4 => 24,  // Q4_K (block size 32, variable)
            5 => 22,  // Q5_0 (block size 32, 22 bytes per block)
            6 => 24,  // Q5_1 (block size 32, 24 bytes per block)
            7 => 28,  // Q5_K (block size 32, variable)
            8 => 30,  // Q6_K (block size 32, variable)
            9 => 34,  // Q8_0 (block size 32, 34 bytes per block)
            10 => 36, // Q8_1 (block size 32, 36 bytes per block)
            11 => 18, // Q2_K (block size 256, variable)
            12 => 20, // Q3_K (block size 256, variable)
            13 => 24, // Q4_K_M (block size 256, variable)
            14 => 28, // Q5_K_M (block size 256, variable)
            15 => 30, // Q6_K_M (block size 256, variable)
            _ => {
                // Default to 4 bytes for unknown types (conservative)
                4
            }
        };

        // For quantized types, calculate based on blocks
        // Most quantized types use blocks of 32 or 256 elements
        let size = if tensor_type >= 2 && tensor_type <= 15 {
            // Quantized types: calculate based on block structure
            // This is simplified - actual calculation depends on block size
            let block_size = if tensor_type >= 11 { 256 } else { 32 };
            let blocks = (elements + block_size - 1) / block_size; // Ceiling division
            (blocks as usize) * element_size
        } else {
            // Non-quantized types: simple multiplication
            (elements as usize) * element_size
        };

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require actual GGUF file data
    // For now, they're placeholders

    #[test]
    fn test_magic_validation() {
        // This would test with invalid magic
        // Would need actual test data
    }
}
