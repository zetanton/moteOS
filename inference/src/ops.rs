use alloc::vec::Vec;
use micromath::F32Ext;
use crate::tensor::{BlockQ4K, QK_K};

/// SiLU (Sigmoid Linear Unit) activation function: x * sigmoid(x)
pub fn silu(x: &mut [f32]) {
    for val in x.iter_mut() {
        *val = *val / (1.0 + (-*val).exp());
    }
}

/// Softmax activation function
pub fn softmax(x: &mut [f32]) {
    let mut max = x[0];
    for &val in x.iter().skip(1) {
        if val > max {
            max = val;
        }
    }

    let mut sum = 0.0;
    for val in x.iter_mut() {
        *val = (*val - max).exp();
        sum += *val;
    }

    let inv_sum = 1.0 / sum;
    for val in x.iter_mut() {
        *val *= inv_sum;
    }
}

/// RMS Norm (Root Mean Square Layer Normalization)
pub fn rms_norm(out: &mut [f32], x: &[f32], weight: &[f32], eps: f32) {
    let mut sum = 0.0;
    for &val in x.iter() {
        sum += val * val;
    }
    let mean = sum / x.len() as f32;
    let inv_std = 1.0 / (mean + eps).sqrt();

    for i in 0..x.len() {
        out[i] = x[i] * inv_std * weight[i];
    }
}

/// Layer Norm
pub fn layer_norm(out: &mut [f32], x: &[f32], weight: &[f32], bias: &[f32], eps: f32) {
    let mut sum = 0.0;
    for &val in x.iter() {
        sum += val;
    }
    let mean = sum / x.len() as f32;

    let mut var = 0.0;
    for &val in x.iter() {
        let diff = val - mean;
        var += diff * diff;
    }
    let inv_std = 1.0 / (var / x.len() as f32 + eps).sqrt();

    for i in 0..x.len() {
        out[i] = (x[i] - mean) * inv_std * weight[i] + bias[i];
    }
}

/// RoPE (Rotary Positional Embedding)
pub fn rope(x: &mut [f32], pos: usize, head_dim: usize, freq_base: f32) {
    let n_heads = x.len() / head_dim;
    for h in 0..n_heads {
        let head_x = &mut x[h * head_dim..(h + 1) * head_dim];
        for i in 0..head_dim / 2 {
            let freq = 1.0 / freq_base.powf((2 * i) as f32 / head_dim as f32);
            let val = pos as f32 * freq;
            let f_cos = val.cos();
            let f_sin = val.sin();

            let x0 = head_x[i];
            let x1 = head_x[i + head_dim / 2];

            head_x[i] = x0 * f_cos - x1 * f_sin;
            head_x[i + head_dim / 2] = x0 * f_sin + x1 * f_cos;
        }
    }
}

/// Element-wise addition
pub fn add(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

/// Element-wise multiplication
pub fn mul(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
}

/// Matrix multiplication (F32)
/// out = A * B
/// A: (m, k), B: (k, n), out: (m, n)
pub fn matmul_f32(a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(m * n);
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0;
            for l in 0..k {
                sum += a[i * k + l] * b[l * n + j];
            }
            out.push(sum);
        }
    }
    out
}

/// Matrix multiplication (Q4_K)
/// A: (m, k) quantized, B: (k, n) f32, out: (m, n) f32
pub fn matmul_q4k(a: &[u8], b: &[f32], m: usize, n: usize, k: usize) -> Vec<f32> {
    let blocks_per_row = k / QK_K;
    let block_size = core::mem::size_of::<BlockQ4K>();
    let mut out = Vec::with_capacity(m * n);
    
    for i in 0..m {
        for j in 0..n {
            let mut row_sum = 0.0;
            let row_start = i * blocks_per_row * block_size;
            
            for l in 0..blocks_per_row {
                let block_offset = row_start + l * block_size;
                let block = unsafe { &*(a.as_ptr().add(block_offset) as *const BlockQ4K) };
                
                // Extract 32 elements at a time
                for group in 0..8 {
                    let group_scale = get_scale(block, group);
                    let group_min = get_min(block, group);
                    let d = block.d * group_scale;
                    let m = block.dmin * group_min;
                    
                    for r in 0..32 {
                        let idx = group * 32 + r;
                        let q = if idx % 2 == 0 {
                            block.qs[idx / 2] & 0x0F
                        } else {
                            block.qs[idx / 2] >> 4
                        };
                        
                        let val = (q as f32) * d - m;
                        row_sum += val * b[(l * QK_K + idx) * n + j];
                    }
                }
            }
            out.push(row_sum);
        }
    }
    out
}

/// Dot product of a Q4_K block and an F32 vector (for vector-matrix mult)
pub fn dot_product_q4k_f32(block: &BlockQ4K, b: &[f32]) -> f32 {
    let mut sum = 0.0;
    
    for i in 0..8 { // 8 groups of 32 elements = 256
        let group_scale = get_scale(block, i);
        let group_min = get_min(block, i);
        
        let d = block.d * group_scale;
        let m = block.dmin * group_min;
        
        for j in 0..32 {
            let idx = i * 32 + j;
            let q = if idx % 2 == 0 {
                block.qs[idx / 2] & 0x0F
            } else {
                block.qs[idx / 2] >> 4
            };
            
            let val = (q as f32) * d - m;
            sum += val * b[idx];
        }
    }
    
    sum
}

// Helper to extract 6-bit scales from the 12-byte scales array
fn get_scale(block: &BlockQ4K, i: usize) -> f32 {
    let bit_idx = i * 6;
    let byte_idx = bit_idx / 8;
    let bit_shift = bit_idx % 8;
    let val = if bit_shift <= 2 {
        (block.scales[byte_idx] >> bit_shift) & 0x3F
    } else {
        let low = block.scales[byte_idx] >> bit_shift;
        let high = (block.scales[byte_idx + 1] as u32) << (8 - bit_shift);
        (low as u32 | high) as u8 & 0x3F
    };
    val as f32
}

fn get_min(block: &BlockQ4K, i: usize) -> f32 {
    let bit_idx = 48 + i * 6;
    let byte_idx = bit_idx / 8;
    let bit_shift = bit_idx % 8;
    let val = if bit_shift <= 2 {
        (block.scales[byte_idx] >> bit_shift) & 0x3F
    } else {
        let low = block.scales[byte_idx] >> bit_shift;
        let high = (block.scales[byte_idx + 1] as u32) << (8 - bit_shift);
        (low as u32 | high) as u8 & 0x3F
    };
    val as f32
}

/// Simple 64-bit Xorshift random number generator
pub fn xorshift64(mut seed: u64) -> u64 {
    seed ^= seed << 13;
    seed ^= seed >> 7;
    seed ^= seed << 17;
    seed
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;

    #[test]
    fn test_silu() {
        let mut x = [0.0, 1.0, -1.0];
        silu(&mut x);
        assert_eq!(x[0], 0.0);
        assert!((x[1] - 0.7310586).abs() < 1e-6);
    }

    #[test]
    fn test_softmax() {
        let mut x = [1.0, 2.0, 3.0];
        softmax(&mut x);
        let sum: f32 = x.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert!(x[2] > x[1]);
        assert!(x[1] > x[0]);
    }

    #[test]
    fn test_rms_norm() {
        let x = [1.0, 2.0, 3.0];
        let weight = [1.0, 1.0, 1.0];
        let mut out = [0.0; 3];
        rms_norm(&mut out, &x, &weight, 1e-6);
        
        let mut sum_sq = 0.0;
        for &val in out.iter() {
            sum_sq += val * val;
        }
        let rms = (sum_sq / out.len() as f32).sqrt();
        assert!((rms - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_matmul_f32() {
        let a = [1.0, 2.0, 3.0, 4.0]; // 2x2
        let b = [1.0, 2.0]; // 2x1
        let out = matmul_f32(&a, &b, 2, 1, 2);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], 5.0);
        assert_eq!(out[1], 11.0);
    }

    #[test]
    fn test_add_mul() {
        let a = [1.0, 2.0];
        let b = [3.0, 4.0];
        let c = add(&a, &b);
        assert_eq!(c, [4.0, 6.0]);
        let d = mul(&a, &b);
        assert_eq!(d, [3.0, 8.0]);
    }
}