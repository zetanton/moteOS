use alloc::vec::Vec;
use crate::ops::{softmax, xorshift64};

/// Sample a token from logits using various techniques
pub fn sample(
    logits: &mut [f32],
    temperature: f32,
    top_p: Option<f32>,
    top_k: Option<usize>,
    rng_seed: u64,
) -> u32 {
    // 1. Apply temperature
    if temperature > 0.0 && temperature != 1.0 {
        for val in logits.iter_mut() {
            *val /= temperature;
        }
    }
    
    // 2. Apply Softmax to get probabilities
    softmax(logits);
    
    // 3. Top-K filtering
    if let Some(k) = top_k {
        if k > 0 && k < logits.len() {
            // Find the k-th largest value
            // For simplicity in no_std, we'll do a partial sort or just find threshold
            // In a more optimized version, use a min-heap
            let mut indexed_logits: Vec<(usize, f32)> = logits.iter().cloned().enumerate().collect();
            indexed_logits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
            
            let threshold = indexed_logits[k - 1].1;
            for val in logits.iter_mut() {
                if *val < threshold {
                    *val = 0.0;
                }
            }
            
            // Re-normalize after zeroing
            let sum: f32 = logits.iter().sum();
            if sum > 0.0 {
                for val in logits.iter_mut() {
                    *val /= sum;
                }
            }
        }
    }
    
    // 4. Top-P (Nucleus) sampling
    if let Some(p) = top_p {
        if p > 0.0 && p < 1.0 {
            let mut indexed_logits: Vec<(usize, f32)> = logits.iter().cloned().enumerate().collect();
            indexed_logits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
            
            let mut cumulative_prob = 0.0;
            let mut last_idx = indexed_logits.len() - 1;
            
            for (i, &(_, prob)) in indexed_logits.iter().enumerate() {
                cumulative_prob += prob;
                if cumulative_prob >= p {
                    last_idx = i;
                    break;
                }
            }
            
            let threshold = indexed_logits[last_idx].1;
            for val in logits.iter_mut() {
                if *val < threshold {
                    *val = 0.0;
                }
            }
            
            // Re-normalize after zeroing
            let sum: f32 = logits.iter().sum();
            if sum > 0.0 {
                for val in logits.iter_mut() {
                    *val /= sum;
                }
            }
        }
    }
    
    // 5. Sample from the distribution
    // For no_std, we need a simple RNG. We'll use a LCG or Xorshift.
    let random_val = xorshift64(rng_seed) as f32 / core::u64::MAX as f32;
    
    let mut cumulative_prob = 0.0;
    for (i, &prob) in logits.iter().enumerate() {
        cumulative_prob += prob;
        if random_val <= cumulative_prob {
            return i as u32;
        }
    }
    
    // Fallback to the most likely token
    logits.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0 as u32
}
