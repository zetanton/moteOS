#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(target_arch = "aarch64")]
pub unsafe fn dot_product_f32_neon(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len();
    let mut sumv = vdupq_n_f32(0.0);
    let mut i = 0;
    
    while i + 4 <= n {
        let va = vld1q_f32(a.as_ptr().add(i));
        let vb = vld1q_f32(b.as_ptr().add(i));
        sumv = vfmaq_f32(sumv, va, vb);
        i += 4;
    }
    
    let mut final_sum = vaddvq_f32(sumv);
    
    while i < n {
        final_sum += a[i] * b[i];
        i += 1;
    }
    
    final_sum
}

#[cfg(target_arch = "aarch64")]
pub fn matmul_f32_optimized(out: &mut [f32], a: &[f32], b: &[f32], m: usize, k: usize) {
    for i in 0..m {
        let row = &a[i * k..(i + 1) * k];
        unsafe {
            out[i] = dot_product_f32_neon(row, b);
        }
    }
}
