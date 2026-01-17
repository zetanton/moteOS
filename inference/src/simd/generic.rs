pub fn dot_product_f32(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub fn matmul_f32(out: &mut [f32], a: &[f32], b: &[f32], m: usize, k: usize) {
    for i in 0..m {
        let row = &a[i * k..(i + 1) * k];
        out[i] = dot_product_f32(row, b);
    }
}
