//! Mistral model implementation for MLX
//!
//! Implements the Mistral-7B transformer architecture using mlx-rs.
//! Based on the mlx-rs examples/mistral reference implementation.
//!
//! This module is only compiled when the `mlx` feature is enabled.

use mlx_rs::error::Exception;
use mlx_rs::macros::ModuleParameters;
use mlx_rs::module::{Module, ModuleParametersExt};
use mlx_rs::nn;
use mlx_rs::ops::indexing::{IndexOp, NewAxis};
use mlx_rs::quantization::MaybeQuantized;
use mlx_rs::Array;
use serde::Deserialize;
use std::path::Path;

/// Model configuration (loaded from params.json or config.json)
#[derive(Debug, Clone, Deserialize)]
pub struct ModelArgs {
    /// Model hidden dimension
    pub dim: i32,
    /// Number of transformer layers
    pub n_layers: i32,
    /// Attention head dimension
    pub head_dim: i32,
    /// FFN hidden dimension
    pub hidden_dim: i32,
    /// Number of attention heads
    pub n_heads: i32,
    /// Number of KV heads (grouped query attention)
    pub n_kv_heads: i32,
    /// Vocabulary size
    pub vocab_size: i32,
    /// RoPE theta base
    #[serde(default = "default_rope_theta")]
    pub rope_theta: f32,
}

fn default_rope_theta() -> f32 {
    10000.0
}

impl ModelArgs {
    /// Estimate total parameter count
    pub fn estimated_parameters(&self) -> u64 {
        let embed = self.vocab_size as u64 * self.dim as u64;
        let per_layer = {
            let attn = (self.dim as u64 * self.dim as u64) * 4;
            let ffn = (self.dim as u64 * self.hidden_dim as u64) * 3;
            let norms = self.dim as u64 * 2;
            attn + ffn + norms
        };
        let output = self.vocab_size as u64 * self.dim as u64;
        embed + (per_layer * self.n_layers as u64) + output
    }
}

/// Feed-forward network with SwiGLU activation
#[derive(Debug, Clone, ModuleParameters)]
struct FeedForward {
    #[param]
    w1: MaybeQuantized<nn::Linear>,
    #[param]
    w2: MaybeQuantized<nn::Linear>,
    #[param]
    w3: MaybeQuantized<nn::Linear>,
}

impl FeedForward {
    fn new(args: &ModelArgs) -> Result<Self, Exception> {
        Ok(Self {
            w1: MaybeQuantized::new(nn::Linear::new(args.dim, args.hidden_dim)?),
            w2: MaybeQuantized::new(nn::Linear::new(args.hidden_dim, args.dim)?),
            w3: MaybeQuantized::new(nn::Linear::new(args.dim, args.hidden_dim)?),
        })
    }
}

impl Module<&Array> for FeedForward {
    type Error = Exception;
    type Output = Array;

    fn forward(&mut self, x: &Array) -> Result<Array, Exception> {
        let w2_input = nn::silu(self.w1.forward(x)?)?.multiply(self.w3.forward(x)?)?;
        self.w2.forward(&w2_input)
    }

    fn training_mode(&mut self, _mode: bool) {}
}

/// Multi-head attention with RoPE and KV-cache
#[derive(Debug, Clone, ModuleParameters)]
pub struct Attention {
    n_heads: i32,
    n_kv_heads: i32,
    scale: f32,

    #[param]
    wq: MaybeQuantized<nn::Linear>,
    #[param]
    wk: MaybeQuantized<nn::Linear>,
    #[param]
    wv: MaybeQuantized<nn::Linear>,
    #[param]
    wo: MaybeQuantized<nn::Linear>,

    #[param]
    rope: nn::Rope,
}

/// Input for attention forward pass
pub struct AttentionInput<'a> {
    pub x: &'a Array,
    pub mask: Option<&'a Array>,
    pub cache: Option<(&'a Array, &'a Array)>,
}

/// Output from attention forward pass
pub struct AttentionOutput {
    pub output: Array,
    pub cache: (Array, Array),
}

impl Attention {
    fn new(args: &ModelArgs) -> Result<Self, Exception> {
        let scale = (args.head_dim as f32).powf(-0.5);
        Ok(Self {
            n_heads: args.n_heads,
            n_kv_heads: args.n_kv_heads,
            scale,
            wq: MaybeQuantized::new(nn::Linear::new(
                args.dim,
                args.n_heads * args.head_dim,
            )?),
            wk: MaybeQuantized::new(nn::Linear::new(
                args.dim,
                args.n_kv_heads * args.head_dim,
            )?),
            wv: MaybeQuantized::new(nn::Linear::new(
                args.dim,
                args.n_kv_heads * args.head_dim,
            )?),
            wo: MaybeQuantized::new(nn::Linear::new(
                args.n_heads * args.head_dim,
                args.dim,
            )?),
            rope: nn::Rope::new(args.head_dim),
        })
    }
}

/// Run scaled dot product attention, handling optional mask
fn sdpa(
    queries: Array,
    keys: &Array,
    values: &Array,
    scale: f32,
    mask: Option<&Array>,
) -> Result<Array, Exception> {
    use mlx_rs::fast::ScaledDotProductAttentionMask;
    let mask_opt: Option<ScaledDotProductAttentionMask<'_>> =
        mask.map(ScaledDotProductAttentionMask::Array);
    mlx_rs::fast::scaled_dot_product_attention(queries, keys, values, scale, mask_opt)
}

impl Module<AttentionInput<'_>> for Attention {
    type Error = Exception;
    type Output = AttentionOutput;

    fn forward(&mut self, input: AttentionInput<'_>) -> Result<AttentionOutput, Exception> {
        let AttentionInput { x, mask, cache } = input;
        let shape = x.shape();
        let (b, l) = (shape[0], shape[1]);

        let mut queries = self.wq.forward(x)?;
        let mut keys = self.wk.forward(x)?;
        let mut values = self.wv.forward(x)?;

        // Reshape for multi-head attention
        queries = queries
            .reshape(&[b, l, self.n_heads, -1])?
            .transpose_axes(&[0, 2, 1, 3])?;
        keys = keys
            .reshape(&[b, l, self.n_kv_heads, -1])?
            .transpose_axes(&[0, 2, 1, 3])?;
        values = values
            .reshape(&[b, l, self.n_kv_heads, -1])?
            .transpose_axes(&[0, 2, 1, 3])?;

        // Apply RoPE with cache offset
        match cache {
            Some((key_cache, value_cache)) => {
                let offset = key_cache.shape()[2];
                queries = self.rope.forward((&queries, offset))?;
                keys = self.rope.forward((&keys, offset))?;
                keys = mlx_rs::ops::concatenate_axis(&[key_cache, &keys], 2)?;
                values = mlx_rs::ops::concatenate_axis(&[value_cache, &values], 2)?;
            }
            None => {
                queries = self.rope.forward(&queries)?;
                keys = self.rope.forward(&keys)?;
            }
        }

        // Scaled dot-product attention
        let output = sdpa(queries, &keys, &values, self.scale, mask)?;

        let output = output
            .transpose_axes(&[0, 2, 1, 3])?
            .reshape(&[b, l, -1])?;
        let output = self.wo.forward(&output)?;

        Ok(AttentionOutput {
            output,
            cache: (keys, values),
        })
    }

    fn training_mode(&mut self, _mode: bool) {}
}

/// Transformer block (attention + feed-forward with residuals)
#[derive(Debug, Clone, ModuleParameters)]
struct TransformerBlock {
    #[param]
    attention: Attention,

    #[param]
    feed_forward: FeedForward,

    #[param]
    attention_norm: nn::RmsNorm,

    #[param]
    ffn_norm: nn::RmsNorm,
}

impl TransformerBlock {
    fn new(args: &ModelArgs) -> Result<Self, Exception> {
        Ok(Self {
            attention: Attention::new(args)?,
            feed_forward: FeedForward::new(args)?,
            attention_norm: nn::RmsNorm::new(args.dim)?,
            ffn_norm: nn::RmsNorm::new(args.dim)?,
        })
    }
}

impl Module<AttentionInput<'_>> for TransformerBlock {
    type Error = Exception;
    type Output = AttentionOutput;

    fn forward(&mut self, input: AttentionInput<'_>) -> Result<AttentionOutput, Exception> {
        let AttentionInput { x, mask, cache } = input;

        // Attention with residual
        let norm_x = self.attention_norm.forward(x)?;
        let attn_out = self.attention.forward(AttentionInput {
            x: &norm_x,
            mask,
            cache,
        })?;
        let h = x.add(attn_out.output)?;

        // Feed-forward with residual
        let r = self.feed_forward.forward(&self.ffn_norm.forward(&h)?)?;
        let output = h.add(r)?;

        Ok(AttentionOutput {
            output,
            cache: attn_out.cache,
        })
    }

    fn training_mode(&mut self, _mode: bool) {}
}

/// Input for the full model forward pass
pub struct MistralInput<'a> {
    pub inputs: &'a Array,
    pub cache: &'a [Option<(Array, Array)>],
}

/// Output from the full model forward pass
pub struct MistralOutput {
    pub logits: Array,
    pub cache: Vec<Option<(Array, Array)>>,
}

/// Full Mistral model
#[derive(Debug, Clone, ModuleParameters)]
pub struct Mistral {
    #[param]
    tok_embeddings: MaybeQuantized<nn::Embedding>,

    #[param]
    layers: Vec<TransformerBlock>,

    #[param]
    norm: nn::RmsNorm,

    #[param]
    output: MaybeQuantized<nn::Linear>,
}

impl Mistral {
    /// Create a new Mistral model from config
    pub fn new(args: &ModelArgs) -> Result<Self, Exception> {
        let mut layers = Vec::with_capacity(args.n_layers as usize);
        for _ in 0..args.n_layers {
            layers.push(TransformerBlock::new(args)?);
        }

        Ok(Self {
            tok_embeddings: MaybeQuantized::new(nn::Embedding::new(args.vocab_size, args.dim)?),
            layers,
            norm: nn::RmsNorm::new(args.dim)?,
            output: MaybeQuantized::new(nn::Linear::new(args.dim, args.vocab_size)?),
        })
    }

    /// Load weights from a safetensors file
    pub fn load_safetensors(&mut self, path: &Path) -> Result<(), mlx_rs::error::IoError> {
        ModuleParametersExt::load_safetensors(self, path)
    }

    /// Quantize the model for faster inference and lower memory
    pub fn quantize(self) -> Result<Self, Exception> {
        use mlx_rs::quantization::Quantizable;
        self.try_into_quantized(64, 4)
    }
}

impl mlx_rs::quantization::Quantizable for Mistral {
    const DEFAULT_GROUP_SIZE: i32 = 64;
    const DEFAULT_BITS: i32 = 4;
    type Quantized = Self;
    type QuantizationError = Exception;

    fn try_into_quantized(
        mut self,
        group_size: i32,
        bits: i32,
    ) -> Result<Self::Quantized, Self::QuantizationError> {
        let quant_fn = |m: nn::Embedding| m.try_into_quantized(group_size, bits);
        self.tok_embeddings = self.tok_embeddings.quantize_with(quant_fn)?;

        let quant_linear =
            |m: nn::Linear| -> Result<nn::QuantizedLinear, Exception> { m.try_into_quantized(group_size, bits) };
        self.output = self.output.quantize_with(quant_linear)?;

        for layer in &mut self.layers {
            layer.attention.wq = layer.attention.wq.clone().quantize_with(quant_linear)?;
            layer.attention.wk = layer.attention.wk.clone().quantize_with(quant_linear)?;
            layer.attention.wv = layer.attention.wv.clone().quantize_with(quant_linear)?;
            layer.attention.wo = layer.attention.wo.clone().quantize_with(quant_linear)?;
            layer.feed_forward.w1 = layer.feed_forward.w1.clone().quantize_with(quant_linear)?;
            layer.feed_forward.w2 = layer.feed_forward.w2.clone().quantize_with(quant_linear)?;
            layer.feed_forward.w3 = layer.feed_forward.w3.clone().quantize_with(quant_linear)?;
        }

        Ok(self)
    }
}

impl Module<MistralInput<'_>> for Mistral {
    type Error = Exception;
    type Output = MistralOutput;

    fn forward(&mut self, input: MistralInput<'_>) -> Result<MistralOutput, Exception> {
        let MistralInput { inputs, cache } = input;

        // Token embedding
        let mut h = self.tok_embeddings.forward(inputs)?;

        // Causal mask
        let mut mask = None;
        if h.shape()[1] > 1 {
            let mask_ =
                nn::MultiHeadAttention::create_additive_causal_mask::<f32>(h.shape()[1])?;
            let mask_ = mask_.as_dtype(h.dtype())?;
            mask = Some(mask_);
        }

        // Process through transformer layers
        let mut out_cache = Vec::with_capacity(self.layers.len());
        for (i, layer) in self.layers.iter_mut().enumerate() {
            let cache_entry = cache
                .get(i)
                .and_then(Option::as_ref)
                .map(|(k, v)| (k, v));
            let input = AttentionInput {
                x: &h,
                mask: mask.as_ref(),
                cache: cache_entry,
            };
            let output = layer.forward(input)?;
            h = output.output;
            out_cache.push(Some(output.cache));
        }

        // Output projection
        let output = self.output.forward(&self.norm.forward(&h)?)?;

        Ok(MistralOutput {
            logits: output,
            cache: out_cache,
        })
    }

    fn training_mode(&mut self, _mode: bool) {}
}

/// Temperature-based sampling
fn sample(logits: &Array, temp: f32) -> Result<Array, Exception> {
    if temp == 0.0 {
        mlx_rs::ops::indexing::argmax_axis(logits, -1, None)
    } else {
        let scaled = logits.multiply(mlx_rs::array!(1.0 / temp))?;
        mlx_rs::random::categorical(scaled, None, None, None)
    }
}

/// Generation state machine
enum GenerateState<'a> {
    /// First step: process entire prompt (prefill)
    Prefill { prompt_tokens: &'a Array },
    /// Subsequent steps: generate one token at a time using KV-cache
    Decoding {
        y: Array,
        cache: Vec<Option<(Array, Array)>>,
    },
}

/// Token generator iterator
///
/// Implements the two-phase generation pattern:
/// 1. Prefill: process entire prompt, generate first token
/// 2. Decoding: use KV-cache to generate subsequent tokens one at a time
pub struct Generate<'a> {
    model: &'a mut Mistral,
    state: GenerateState<'a>,
    temp: f32,
}

impl<'a> Generate<'a> {
    /// Create a new generator
    pub fn new(model: &'a mut Mistral, prompt_tokens: &'a Array, temp: f32) -> Self {
        Self {
            model,
            state: GenerateState::Prefill { prompt_tokens },
            temp,
        }
    }
}

/// Helper macro to propagate errors in Iterator::next
macro_rules! tri {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Some(Err(e)),
        }
    };
}

impl Iterator for Generate<'_> {
    type Item = Result<Array, Exception>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.state {
            GenerateState::Prefill { prompt_tokens } => {
                let prompt_tokens = *prompt_tokens;
                let initial_cache: Vec<Option<(Array, Array)>> = Vec::new();
                let input = MistralInput {
                    inputs: prompt_tokens,
                    cache: &initial_cache,
                };
                let MistralOutput { logits, cache } = tri!(self.model.forward(input));
                let y = tri!(sample(&logits.index((.., -1, ..)), self.temp));

                self.state = GenerateState::Decoding {
                    y: y.clone(),
                    cache,
                };
                Some(Ok(y))
            }
            GenerateState::Decoding { y, cache } => {
                let next_token = y.index((.., NewAxis));
                let input = MistralInput {
                    inputs: &next_token,
                    cache: cache.as_slice(),
                };
                let MistralOutput {
                    logits,
                    cache: new_cache,
                } = tri!(self.model.forward(input));

                let logits = tri!(logits.squeeze_axes(&[1]));
                let y = tri!(sample(&logits, self.temp));

                self.state = GenerateState::Decoding {
                    y: y.clone(),
                    cache: new_cache,
                };
                Some(Ok(y))
            }
        }
    }
}
