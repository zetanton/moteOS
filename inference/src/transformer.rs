use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::ops::{matmul_f32, matmul_q4k, add, mul, rms_norm, rope, silu, softmax};
use crate::tensor::{Tensor, TensorData};
use crate::error::ModelError;
use micromath::F32Ext;

/// Model configuration parameters
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub head_dim: usize,
    pub intermediate_size: usize,
    pub max_seq_len: usize,
    pub rope_freq_base: f32,
    pub norm_eps: f32,
}

impl ModelConfig {
    /// Create a default config for SmolLM-360M
    pub fn smollm_360m() -> Self {
        Self {
            vocab_size: 32000,
            hidden_size: 1024,
            num_layers: 24,
            num_heads: 16,
            head_dim: 64,
            intermediate_size: 2816,
            max_seq_len: 2048,
            rope_freq_base: 10000.0,
            norm_eps: 1e-6,
        }
    }
}

/// Embedding weights
#[derive(Debug, Clone)]
pub struct EmbeddingWeights {
    pub weight: Tensor, // (vocab_size, hidden_size)
}

/// Transformer layer weights
#[derive(Debug, Clone)]
pub struct TransformerLayerWeights {
    /// Attention layer norm (RMS norm)
    pub attention_norm: Vec<f32>, // (hidden_size,)
    
    /// QKV projection weights (combined)
    /// Shape: (hidden_size, 3 * hidden_size)
    pub attention_qkv: Tensor,
    
    /// Attention output projection weights
    /// Shape: (hidden_size, hidden_size)
    pub attention_output: Tensor,
    
    /// FFN layer norm (RMS norm)
    pub ffn_norm: Vec<f32>, // (hidden_size,)
    
    /// FFN gate projection weights
    /// Shape: (hidden_size, intermediate_size)
    pub ffn_gate: Tensor,
    
    /// FFN up projection weights
    /// Shape: (hidden_size, intermediate_size)
    pub ffn_up: Tensor,
    
    /// FFN down projection weights
    /// Shape: (intermediate_size, hidden_size)
    pub ffn_down: Tensor,
}

/// Output layer weights
#[derive(Debug, Clone)]
pub struct OutputWeights {
    pub weight: Tensor, // (vocab_size, hidden_size)
}

/// Complete model weights
#[derive(Debug, Clone)]
pub struct ModelWeights {
    pub embedding: EmbeddingWeights,
    pub layers: Vec<TransformerLayerWeights>,
    pub output: OutputWeights,
}

/// KV Cache for efficient autoregressive generation
pub struct KvCache {
    /// Key cache: [layer][seq_pos * head_dim]
    k_cache: Vec<Vec<f32>>,
    /// Value cache: [layer][seq_pos * head_dim]
    v_cache: Vec<Vec<f32>>,
    /// Current sequence position
    current_pos: usize,
    /// Number of layers
    num_layers: usize,
    /// Maximum sequence length
    max_seq_len: usize,
    /// Head dimension
    head_dim: usize,
    /// Number of heads
    num_heads: usize,
}

impl KvCache {
    /// Create a new KV cache
    pub fn new(num_layers: usize, max_seq_len: usize, num_heads: usize, head_dim: usize) -> Self {
        let cache_size = max_seq_len * num_heads * head_dim;
        let mut k_cache = Vec::with_capacity(num_layers);
        let mut v_cache = Vec::with_capacity(num_layers);
        
        for _ in 0..num_layers {
            k_cache.push(vec![0.0; cache_size]);
            v_cache.push(vec![0.0; cache_size]);
        }
        
        Self {
            k_cache,
            v_cache,
            current_pos: 0,
            num_layers,
            max_seq_len,
            head_dim,
            num_heads,
        }
    }
    
    /// Append K and V vectors for a specific layer
    pub fn append(&mut self, layer: usize, k: &[f32], v: &[f32]) {
        if layer >= self.num_layers {
            return;
        }
        
        if self.current_pos >= self.max_seq_len {
            return;
        }
        
        let pos_offset = self.current_pos * self.num_heads * self.head_dim;
        let cache_size = self.num_heads * self.head_dim;
        
        if pos_offset + cache_size <= self.k_cache[layer].len() {
            self.k_cache[layer][pos_offset..pos_offset + cache_size].copy_from_slice(k);
            self.v_cache[layer][pos_offset..pos_offset + cache_size].copy_from_slice(v);
        }
    }
    
    /// Get K cache for a specific layer and position range
    pub fn get_k(&self, layer: usize, start_pos: usize, end_pos: usize) -> &[f32] {
        if layer >= self.num_layers || end_pos > self.max_seq_len {
            return &[];
        }
        
        let start_offset = start_pos * self.num_heads * self.head_dim;
        let end_offset = end_pos * self.num_heads * self.head_dim;
        
        if end_offset <= self.k_cache[layer].len() {
            &self.k_cache[layer][start_offset..end_offset]
        } else {
            &[]
        }
    }
    
    /// Get V cache for a specific layer and position range
    pub fn get_v(&self, layer: usize, start_pos: usize, end_pos: usize) -> &[f32] {
        if layer >= self.num_layers || end_pos > self.max_seq_len {
            return &[];
        }
        
        let start_offset = start_pos * self.num_heads * self.head_dim;
        let end_offset = end_pos * self.num_heads * self.head_dim;
        
        if end_offset <= self.v_cache[layer].len() {
            &self.v_cache[layer][start_offset..end_offset]
        } else {
            &[]
        }
    }
    
    /// Get current position
    pub fn current_pos(&self) -> usize {
        self.current_pos
    }
    
    /// Advance position (for next token)
    pub fn advance(&mut self) {
        if self.current_pos < self.max_seq_len {
            self.current_pos += 1;
        }
    }
    
    /// Reset cache (for new sequence)
    pub fn reset(&mut self) {
        self.current_pos = 0;
        for layer in 0..self.num_layers {
            self.k_cache[layer].fill(0.0);
            self.v_cache[layer].fill(0.0);
        }
    }
}

/// Transformer model for inference
pub struct Transformer {
    weights: ModelWeights,
    config: ModelConfig,
}

impl Transformer {
    /// Create a new transformer with weights and config
    pub fn new(weights: ModelWeights, config: ModelConfig) -> Self {
        Self { weights, config }
    }
    
    /// Forward pass through the transformer
    /// 
    /// # Arguments
    /// * `tokens` - Input token IDs (sequence of token indices)
    /// * `kv_cache` - KV cache for storing attention states
    /// 
    /// # Returns
    /// Logits over vocabulary (vocab_size)
    pub fn forward(&self, tokens: &[u32], kv_cache: &mut KvCache) -> Result<Vec<f32>, ModelError> {
        let seq_len = tokens.len();
        if seq_len == 0 {
            return Err(ModelError::InvalidInput("Empty token sequence".into()));
        }
        
        // 1. Embedding lookup
        let mut x = self.embedding_lookup(tokens)?;
        
        // 2. Process through each transformer layer
        for layer_idx in 0..self.config.num_layers {
            x = self.transformer_layer(&x, layer_idx, kv_cache)?;
        }
        
        // 3. Output projection (vocab_size, hidden_size)
        // Note: Some models apply a final norm before output projection, but for simplicity
        // we'll use the last token's representation directly
        let logits = self.output_projection(&x)?;
        
        Ok(logits)
    }
    
    /// Embedding lookup
    fn embedding_lookup(&self, tokens: &[u32]) -> Result<Vec<f32>, ModelError> {
        let seq_len = tokens.len();
        let hidden_size = self.config.hidden_size;
        let mut embeddings = Vec::with_capacity(seq_len * hidden_size);
        
        match &self.weights.embedding.weight.data {
            TensorData::F32(weight_data) => {
                for &token_id in tokens {
                    if token_id as usize >= self.config.vocab_size {
                        return Err(ModelError::InvalidInput(format!("Token ID {} out of range", token_id)));
                    }
                    let offset = (token_id as usize) * hidden_size;
                    if offset + hidden_size > weight_data.len() {
                        return Err(ModelError::InvalidInput("Embedding weight out of bounds".into()));
                    }
                    embeddings.extend_from_slice(&weight_data[offset..offset + hidden_size]);
                }
            }
            TensorData::Q4K(_) => {
                // Quantized embeddings would need dequantization
                return Err(ModelError::InvalidInput("Quantized embeddings not yet supported".into()));
            }
        }
        
        Ok(embeddings)
    }
    
    /// Process a single transformer layer
    fn transformer_layer(
        &self,
        x: &[f32],
        layer_idx: usize,
        kv_cache: &mut KvCache,
    ) -> Result<Vec<f32>, ModelError> {
        if layer_idx >= self.weights.layers.len() {
            return Err(ModelError::InvalidInput(format!("Layer index {} out of range", layer_idx)));
        }
        
        let layer = &self.weights.layers[layer_idx];
        let hidden_size = self.config.hidden_size;
        
        // 1. Pre-attention RMS norm
        let mut x_norm = vec![0.0; hidden_size];
        rms_norm(&mut x_norm, x, &layer.attention_norm, self.config.norm_eps);
        
        // 2. Attention layer
        let attn_out = self.attention_layer(&x_norm, layer_idx, layer, kv_cache)?;
        
        // 3. Residual connection
        let x_after_attn = add(x, &attn_out);
        
        // 4. Pre-FFN RMS norm
        let mut x_norm2 = vec![0.0; hidden_size];
        rms_norm(&mut x_norm2, &x_after_attn, &layer.ffn_norm, self.config.norm_eps);
        
        // 5. FFN layer
        let ffn_out = self.ffn_layer(&x_norm2, layer)?;
        
        // 6. Residual connection
        let output = add(&x_after_attn, &ffn_out);
        
        Ok(output)
    }
    
    /// Multi-head self-attention layer
    fn attention_layer(
        &self,
        x: &[f32],
        layer_idx: usize,
        layer: &TransformerLayerWeights,
        kv_cache: &mut KvCache,
    ) -> Result<Vec<f32>, ModelError> {
        let hidden_size = self.config.hidden_size;
        let num_heads = self.config.num_heads;
        let head_dim = self.config.head_dim;
        let seq_len = if kv_cache.current_pos() == 0 {
            x.len() / hidden_size
        } else {
            1 // Autoregressive: processing one token at a time
        };
        
        // 1. QKV projection
        // Input: x (seq_len * hidden_size)
        // Output: qkv (seq_len * 3 * hidden_size)
        let qkv = self.qkv_projection(x, &layer.attention_qkv)?;
        
        // 2. Reshape and split into Q, K, V
        // qkv shape: (seq_len, 3 * hidden_size) = (seq_len, 3 * num_heads * head_dim)
        let qkv_len = seq_len * 3 * hidden_size;
        if qkv.len() != qkv_len {
            return Err(ModelError::InvalidInput("QKV projection size mismatch".into()));
        }
        
        // Split QKV: each token has [Q, K, V] concatenated
        // For each token: [Q (hidden_size), K (hidden_size), V (hidden_size)]
        let mut q = Vec::with_capacity(seq_len * hidden_size);
        let mut k = Vec::with_capacity(seq_len * hidden_size);
        let mut v = Vec::with_capacity(seq_len * hidden_size);
        
        for i in 0..seq_len {
            let token_start = i * 3 * hidden_size;
            q.extend_from_slice(&qkv[token_start..token_start + hidden_size]);
            k.extend_from_slice(&qkv[token_start + hidden_size..token_start + 2 * hidden_size]);
            v.extend_from_slice(&qkv[token_start + 2 * hidden_size..token_start + 3 * hidden_size]);
        }
        
        // 3. Apply RoPE to Q and K
        let mut q_rope = q;
        let mut k_rope = k;
        
        for pos in 0..seq_len {
            let abs_pos = kv_cache.current_pos() + pos;
            let q_pos = &mut q_rope[pos * hidden_size..(pos + 1) * hidden_size];
            let k_pos = &mut k_rope[pos * hidden_size..(pos + 1) * hidden_size];
            
            rope(q_pos, abs_pos, head_dim, self.config.rope_freq_base);
            rope(k_pos, abs_pos, head_dim, self.config.rope_freq_base);
        }
        
        // 4. Get cached K and V (for previous positions)
        let cache_start = 0;
        let cache_end = kv_cache.current_pos();
        let cached_k = if cache_end > 0 {
            kv_cache.get_k(layer_idx, cache_start, cache_end)
        } else {
            &[]
        };
        let cached_v = if cache_end > 0 {
            kv_cache.get_v(layer_idx, cache_start, cache_end)
        } else {
            &[]
        };
        
        // 5. Concatenate cached K/V with current K/V
        let total_seq_len = cache_end + seq_len;
        let mut k_full = Vec::with_capacity(total_seq_len * num_heads * head_dim);
        let mut v_full = Vec::with_capacity(total_seq_len * num_heads * head_dim);
        
        if !cached_k.is_empty() {
            k_full.extend_from_slice(cached_k);
        }
        k_full.extend_from_slice(&k_rope);
        
        if !cached_v.is_empty() {
            v_full.extend_from_slice(cached_v);
        }
        v_full.extend_from_slice(&v);
        
        // 6. Store current K and V in cache
        for pos in 0..seq_len {
            let k_slice = &k_rope[pos * num_heads * head_dim..(pos + 1) * num_heads * head_dim];
            let v_slice = &v[pos * num_heads * head_dim..(pos + 1) * num_heads * head_dim];
            kv_cache.append(layer_idx, k_slice, v_slice);
        }
        
        // Advance cache position
        for _ in 0..seq_len {
            kv_cache.advance();
        }
        
        // 7. Compute attention scores: Q @ K^T / sqrt(head_dim)
        // Q: (seq_len, num_heads, head_dim)
        // K: (total_seq_len, num_heads, head_dim)
        // Scores: (seq_len, num_heads, total_seq_len)
        let mut scores = Vec::with_capacity(seq_len * num_heads * total_seq_len);
        let scale = 1.0 / (head_dim as f32).sqrt();
        
        for i in 0..seq_len {
            for h in 0..num_heads {
                let q_head = &q_rope[(i * num_heads + h) * head_dim..(i * num_heads + h + 1) * head_dim];
                for j in 0..total_seq_len {
                    let k_head = &k_full[(j * num_heads + h) * head_dim..(j * num_heads + h + 1) * head_dim];
                    let mut score = 0.0;
                    for d in 0..head_dim {
                        score += q_head[d] * k_head[d];
                    }
                    scores.push(score * scale);
                }
            }
        }
        
        // 8. Apply softmax to scores
        for head in 0..num_heads {
            for i in 0..seq_len {
                let start = (i * num_heads + head) * total_seq_len;
                let end = start + total_seq_len;
                softmax(&mut scores[start..end]);
            }
        }
        
        // 9. Apply attention to values: scores @ V
        // Scores: (seq_len, num_heads, total_seq_len)
        // V: (total_seq_len, num_heads, head_dim)
        // Output: (seq_len, num_heads, head_dim)
        let mut attn_out = vec![0.0; seq_len * num_heads * head_dim];
        
        for i in 0..seq_len {
            for h in 0..num_heads {
                let score_start = (i * num_heads + h) * total_seq_len;
                let out_head = &mut attn_out[(i * num_heads + h) * head_dim..(i * num_heads + h + 1) * head_dim];
                
                for j in 0..total_seq_len {
                    let score = scores[score_start + j];
                    let v_head = &v_full[(j * num_heads + h) * head_dim..(j * num_heads + h + 1) * head_dim];
                    for d in 0..head_dim {
                        out_head[d] += score * v_head[d];
                    }
                }
            }
        }
        
        // 10. Reshape and concatenate heads: (seq_len, num_heads, head_dim) -> (seq_len, hidden_size)
        let mut attn_reshaped = Vec::with_capacity(seq_len * hidden_size);
        for i in 0..seq_len {
            for h in 0..num_heads {
                let head_start = (i * num_heads + h) * head_dim;
                attn_reshaped.extend_from_slice(&attn_out[head_start..head_start + head_dim]);
            }
        }
        
        // 11. Output projection
        let output = self.attention_output_projection(&attn_reshaped, &layer.attention_output)?;
        
        Ok(output)
    }
    
    /// QKV projection
    fn qkv_projection(&self, x: &[f32], qkv_weight: &Tensor) -> Result<Vec<f32>, ModelError> {
        let seq_len = x.len() / self.config.hidden_size;
        let hidden_size = self.config.hidden_size;
        
        match &qkv_weight.data {
            TensorData::F32(weight) => {
                // x: (seq_len, hidden_size)
                // weight: (hidden_size, 3 * hidden_size)
                // out: (seq_len, 3 * hidden_size)
                let qkv = matmul_f32(x, weight, seq_len, 3 * hidden_size, hidden_size);
                Ok(qkv)
            }
            TensorData::Q4K(_weight) => {
                // Quantized QKV projection
                // Note: Proper quantized matrix multiplication for x * weight (where weight is quantized)
                // requires either:
                // 1. Weight stored in transposed format
                // 2. A specialized matmul function for f32 * quantized
                // 3. Dequantizing the weight (memory intensive)
                // 
                // For now, we'll return an error indicating this needs proper support.
                // In a production implementation, weights would be loaded in the appropriate format
                // or a specialized matmul function would be used.
                return Err(ModelError::InvalidInput(
                    "Quantized QKV projection requires proper weight format or specialized matmul".into()
                ));
            }
        }
    }
    
    /// Attention output projection
    fn attention_output_projection(&self, x: &[f32], out_weight: &Tensor) -> Result<Vec<f32>, ModelError> {
        let seq_len = x.len() / self.config.hidden_size;
        let hidden_size = self.config.hidden_size;
        
        match &out_weight.data {
            TensorData::F32(weight) => {
                // x: (seq_len, hidden_size)
                // weight: (hidden_size, hidden_size)
                // out: (seq_len, hidden_size)
                let out = matmul_f32(x, weight, seq_len, hidden_size, hidden_size);
                Ok(out)
            }
            TensorData::Q4K(weight) => {
                // x: (seq_len, hidden_size) = (m, k)
                // weight: (hidden_size, hidden_size) quantized = (k, n)
                // Transpose x to (hidden_size, seq_len) for matmul_q4k
                let x_transposed = self.transpose_matrix(x, seq_len, hidden_size);
                let out_transposed = matmul_q4k(weight, &x_transposed, hidden_size, seq_len, hidden_size);
                let out = self.transpose_matrix(&out_transposed, hidden_size, seq_len);
                Ok(out)
            }
        }
    }
    
    /// Feed-Forward Network layer
    fn ffn_layer(&self, x: &[f32], layer: &TransformerLayerWeights) -> Result<Vec<f32>, ModelError> {
        let seq_len = x.len() / self.config.hidden_size;
        let hidden_size = self.config.hidden_size;
        let intermediate_size = self.config.intermediate_size;
        
        // 1. Gate projection
        let gate = self.ffn_projection(x, &layer.ffn_gate, hidden_size, intermediate_size, seq_len)?;
        
        // 2. Up projection
        let up = self.ffn_projection(x, &layer.ffn_up, hidden_size, intermediate_size, seq_len)?;
        
        // 3. SiLU activation on gate
        let mut gate_activated = gate;
        silu(&mut gate_activated);
        
        // 4. Element-wise multiplication: gate * up
        let ffn_intermediate = mul(&gate_activated, &up);
        
        // 5. Down projection
        let output = self.ffn_projection(&ffn_intermediate, &layer.ffn_down, intermediate_size, hidden_size, seq_len)?;
        
        Ok(output)
    }
    
    /// FFN projection helper
    fn ffn_projection(
        &self,
        x: &[f32],
        weight: &Tensor,
        in_dim: usize,
        out_dim: usize,
        seq_len: usize,
    ) -> Result<Vec<f32>, ModelError> {
        match &weight.data {
            TensorData::F32(weight_data) => {
                // x: (seq_len, in_dim)
                // weight: (in_dim, out_dim)
                // out: (seq_len, out_dim)
                let out = matmul_f32(x, weight_data, seq_len, out_dim, in_dim);
                Ok(out)
            }
            TensorData::Q4K(weight_data) => {
                // x: (seq_len, in_dim) = (m, k)
                // weight: (in_dim, out_dim) quantized = (k, n)
                // Transpose x to (in_dim, seq_len) for matmul_q4k
                let x_transposed = self.transpose_matrix(x, seq_len, in_dim);
                let out_transposed = matmul_q4k(weight_data, &x_transposed, in_dim, seq_len, out_dim);
                let out = self.transpose_matrix(&out_transposed, out_dim, seq_len);
                Ok(out)
            }
        }
    }
    
    /// Output projection to vocabulary
    fn output_projection(&self, x: &[f32]) -> Result<Vec<f32>, ModelError> {
        let seq_len = x.len() / self.config.hidden_size;
        let hidden_size = self.config.hidden_size;
        let vocab_size = self.config.vocab_size;
        
        match &self.weights.output.weight.data {
            TensorData::F32(weight) => {
                // x: (seq_len, hidden_size)
                // weight: (vocab_size, hidden_size)
                // out: (seq_len, vocab_size)
                // Note: Usually we only need the last token's logits
                let last_token = &x[(seq_len - 1) * hidden_size..];
                let logits = matmul_f32(last_token, weight, 1, vocab_size, hidden_size);
                Ok(logits)
            }
            TensorData::Q4K(weight) => {
                // last_token: (hidden_size,) = (k,)
                // weight: (vocab_size, hidden_size) quantized = (m, k)
                // For matmul_q4k: (m, k) quantized * (k, n) f32
                // We need to reshape last_token to (hidden_size, 1)
                let last_token = &x[(seq_len - 1) * hidden_size..];
                let logits = matmul_q4k(weight, last_token, vocab_size, 1, hidden_size);
                Ok(logits)
            }
        }
    }
    
    /// Transpose a matrix (helper for quantized operations)
    fn transpose_matrix(&self, matrix: &[f32], rows: usize, cols: usize) -> Vec<f32> {
        let mut transposed = Vec::with_capacity(rows * cols);
        for j in 0..cols {
            for i in 0..rows {
                transposed.push(matrix[i * cols + j]);
            }
        }
        transposed
    }
    
    
    /// Get model configuration
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
}
