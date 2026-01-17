use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub enum TensorData {
    F32(Vec<f32>),
    Q4K(Vec<u8>), // Q4_K_M / block_q4_K
}

#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: TensorData,
    pub shape: Vec<usize>,
}

impl Tensor {
    pub fn new_f32(data: Vec<f32>, shape: Vec<usize>) -> Self {
        Self {
            data: TensorData::F32(data),
            shape,
        }
    }

    pub fn new_q4k(data: Vec<u8>, shape: Vec<usize>) -> Self {
        Self {
            data: TensorData::Q4K(data),
            shape,
        }
    }

    pub fn elements(&self) -> usize {
        self.shape.iter().product()
    }
}

/// block_q4_K structure from llama.cpp
/// Block size is 256
pub const QK_K: usize = 256;

#[repr(C, packed)]
pub struct BlockQ4K {
    pub d: f32,             // super-block scale (using f32 for simplicity in no_std)
    pub dmin: f32,          // super-block scale for quantized mins
    pub scales: [u8; 12],   // scales and mins, quantized with 6 bits
    pub qs: [u8; 128],      // 4-bit quarters
}
