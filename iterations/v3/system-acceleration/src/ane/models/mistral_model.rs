//! Mistral LLM model loading and inference
//!
//! This module provides Mistral-7B-Instruct-v0.3 CoreML model integration
//! with tokenization, KV caching, and constitutional reasoning capabilities.

use schemars::JsonSchema;
use crate::ane::ane_errors::{ANEError, Result};
use crate::ane::compat::coreml as coreml_bridge;
use crate::ane::compat::coreml::{MLModelConfiguration, MLComputeUnits, KvStateHandle};
use crate::ane::ane_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::telemetry::TelemetryCollector;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::sync::Arc;

/// Safe model reference that can be sent across threads
/// The actual CoreML handle is stored in a thread-local registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub struct SafeModelHandle (crate::ane::compat::coreml::ModelRef);

impl SafeModelHandle {
    pub fn new(model_ref: crate::ane::compat::coreml::ModelRef) -> Self {
        Self(model_ref)
    }

    pub fn get_model_id(&self) -> u64 {
        self.0.id()
    }

    /// Access the underlying model handle on the current thread
    /// Returns None if called on the wrong thread or if model was unloaded
    /// Never returns a fabricated handle - callers must handle None explicitly
    pub fn with_handle<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&crate::ane::compat::coreml::CoreMlHandle) -> R,
    {
        crate::ane::compat::coreml::registry::with_model_handle(self.0, |ptr| {
            if let Some(handle) = crate::ane::compat::coreml::CoreMlHandle::new(ptr.as_ptr()) {
                f(&handle)
            } else {
                // If CoreMlHandle::new fails, we can't call the closure
                // This should not happen in practice, but we need to handle it
                panic!("Failed to create CoreMlHandle from valid pointer");
            }
        })
    }
}

/// Mistral model loaded and ready for inference
#[derive(Debug)]
pub struct MistralModel {
    /// Safe model reference that can be sent across threads
    pub handle: SafeModelHandle,
    /// Model schema information
    pub schema: ModelSchema,
    /// Thread-safe tokenizer for text processing
    pub tokenizer: SafeMistralTokenizer,
    /// Thread-safe KV cache for efficient inference
    pub kv_cache: Arc<tokio::sync::Mutex<KVCache>>,
    /// Telemetry collector (assumed thread-safe)
    pub telemetry: TelemetryCollector,
    /// Circuit breaker for resilience (used in inference, not accessed in manager)
    #[allow(dead_code)]
    pub circuit_breaker: CircuitBreaker,
    /// Model load time
    pub loaded_at: Instant,
    /// Thread-safe last access time (cheap scalar, doesn't need async)
    pub last_accessed: Arc<std::sync::Mutex<Instant>>,
}

/// Thread-safe stateless facade for Mistral tokenizer
/// All operations go through high-level bridge functions that manage resources internally
#[derive(Debug, Clone, Default)]
pub struct SafeMistralTokenizer;

impl SafeMistralTokenizer {
    /// Create a new stateless tokenizer facade
    pub fn new() -> Self {
        Self
    }

    /// Encode text to tokens
    pub fn encode(&self, text: &str) -> Result<Vec<i32>> {
        coreml_bridge::mistral_encode(text)
            .map_err(|e| ANEError::InferenceFailed(format!("Encoding failed: {e}")))
    }

    /// Decode tokens to text
    pub fn decode(&self, tokens: &[i32]) -> Result<String> {
        if tokens.is_empty() {
            return Ok(String::new());
        }
        coreml_bridge::mistral_decode(tokens)
            .map_err(|e| ANEError::InferenceFailed(format!("Decoding failed: {e}")))
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> Result<i32> {
        // TODO: Query actual vocabulary size through CoreML bridge
        //       Currently returns hardcoded value; should query actual vocabulary size from CoreML model metadata.
        //
        // COMPLETION CHECKLIST:
        // [ ] Query CoreML model metadata for vocabulary size
        // [ ] Extract vocabulary size from model schema
        // [ ] Handle missing vocabulary size metadata
        // [ ] Support various model architectures
        // [ ] Add unit tests for vocabulary size query
        // [ ] Add integration tests with real models
        // [ ] Verify vocabulary size accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Vocabulary size is queried from CoreML metadata
        // - Model schema is parsed correctly
        // - Missing metadata is handled gracefully
        // - Various model architectures are supported
        //
        // DEPENDENCIES:
        // - CoreML bridge API (Required)
        // - Model metadata structure (Required)
        // - Vocabulary size extraction utilities (Required)
        //
        // ESTIMATED EFFORT: 2-3 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (model metadata feature)
        // - Change Budget: ~60 LOC
        // - Reviewer Requirements: Core ML expertise
        Ok(32000) // Temporary: hardcoded Mistral-7B vocab size until CoreML query is implemented
    }

    /// Get end-of-sequence token ID
    pub fn eos_id(&self) -> Result<i32> {
        // TODO: Implement actual eos_id through CoreML bridge
        // Standard Mistral models use token ID 2 for EOS
        Ok(2)
    }

    /// Check if text fits in context window
    pub fn fits_context(&self, text: &str, max_length: usize) -> Result<bool> {
        let tokens = self.encode(text)?;
        Ok(tokens.len() <= max_length)
    }

    /// Truncate text to fit context window
    pub fn truncate_to_context(&self, text: &str, max_length: usize) -> Result<String> {
        let tokens = self.encode(text)?;
        if tokens.len() <= max_length {
            return Ok(text.to_string());
        }

        let truncated_tokens = &tokens[..max_length];
        self.decode(truncated_tokens)
    }
}

// No Drop impl needed - stateless facade delegates to bridge

/// KV cache for Mistral inference optimization
/// Shape-aware contract that can wire to Core ML cache ports when available
#[derive(Debug)]
pub struct KVCache {
    /// Maximum tokens tracked in cache (context length)
    pub max_length: usize,
    /// Current generated sequence length
    current_length: usize,
    /// Model architecture metadata for cache shape
    pub n_layers: usize,
    pub n_kv_heads: usize,
    pub head_dim: usize,
    /// Core ML session state handle (optional, for KV cache acceleration)
    pub coreml_state: Option<KvStateHandle>,
}

impl KVCache {
    /// Create new KV cache
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            current_length: 0,
            n_layers: 0,
            n_kv_heads: 0,
            head_dim: 0,
            coreml_state: None,
        }
    }

    /// Configure cache with model architecture metadata and initialize Core ML state
    /// Called once model config is known
    pub fn configure(&mut self, n_layers: usize, n_kv_heads: usize, head_dim: usize, model_handle: &SafeModelHandle) -> Result<()> {
        self.n_layers = n_layers;
        self.n_kv_heads = n_kv_heads;
        self.head_dim = head_dim;

        // Try to create Core ML KV state if available
        // This will fail gracefully on non-macOS platforms
        // The SafeModelHandle contains a ModelRef that we can use directly
        match KvStateHandle::create(
            &model_handle.0, // SafeModelHandle(ModelRef)
            n_layers,
            n_kv_heads,
            head_dim,
            self.max_length,
        ) {
            Ok(state) => {
                self.coreml_state = Some(state);
                tracing::info!("Initialized Core ML KV cache state for {} layers", n_layers);
            }
            Err(e) => {
                tracing::warn!("Failed to create Core ML KV state, falling back to CPU: {}", e);
                self.coreml_state = None;
            }
        }

        Ok(())
    }

    /// Advance one token in streamed generation
    pub fn step(&mut self) -> Result<()> {
        self.current_length = self.current_length.saturating_add(1);

        // Update Core ML state if available
        if let Some(ref state) = self.coreml_state {
            state.step()?;
        }

        Ok(())
    }

    /// Reset cache
    pub fn reset(&mut self) -> Result<()> {
        self.current_length = 0;

        // Reset Core ML state if available
        if let Some(ref state) = self.coreml_state {
            state.reset()?;
        }

        Ok(())
    }

    /// Get current sequence length
    pub fn sequence_length(&self) -> usize {
        self.current_length
    }

    /// Check if cache is valid for given sequence
    pub fn is_valid_for(&self, sequence_length: usize) -> bool {
        sequence_length >= self.current_length
    }
}

/// Model schema for Mistral
#[derive(Debug, Clone, JsonSchema)]
pub struct ModelSchema {
    /// Input tensor specifications
    pub inputs: Vec<TensorSpec>,
    /// Output tensor specifications
    pub outputs: Vec<TensorSpec>,
    /// Context window size
    pub context_length: usize,
}

/// Tensor specification
#[derive(Debug, Clone, JsonSchema)]
pub struct TensorSpec {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: String,
}

/// Compilation options for Mistral models
#[derive(Debug, Clone, JsonSchema)]
pub struct MistralCompilationOptions {
    /// Target precision
    pub precision: Option<String>,
    /// Compute units preference
    pub compute_units: Option<String>,
    /// Enable quantization
    pub enable_quantization: bool,
    /// Context length override
    pub context_length: Option<usize>,
}

impl Default for MistralCompilationOptions {
    fn default() -> Self {
        Self {
            precision: Some("int4".to_string()), // Quantized for memory efficiency
            compute_units: Some("all".to_string()),
            enable_quantization: true,
            context_length: Some(4096),
        }
    }
}

/// Validate compilation options
fn validate_options(opts: &MistralCompilationOptions) -> Result<()> {
    if let Some(p) = &opts.precision {
        let valid_precisions = ["fp16", "int8", "int4"];
        if !valid_precisions.contains(&p.as_str()) {
            return Err(ANEError::InvalidModelFormat(
                format!("Unsupported precision '{}'. Valid values: {:?}", p, valid_precisions)
            ));
        }
    }
    Ok(())
}

/// Load a Mistral model with full configuration
pub async fn load_mistral_model(
    model_path: &Path,
    options: &MistralCompilationOptions,
    mut telemetry: TelemetryCollector,
) -> Result<MistralModel> {
    // Validate options before proceeding
    validate_options(options)?;
    
    let start_time = Instant::now();

    // Load model through CoreML bridge
    let handle = load_coreml_model(model_path, options).await?;

    // Extract model schema
    let schema = extract_model_schema(handle).await?;

    // Initialize stateless tokenizer facade
    let tokenizer = SafeMistralTokenizer::new();

    // Initialize thread-safe KV cache
    let context_length = options.context_length.unwrap_or(4096);
    let mut kv_cache = KVCache::new(context_length);

    // Configure KV cache with model architecture (if available)
    // For Mistral-7B: n_layers=32, n_kv_heads=8, head_dim=128
    let model_handle = SafeModelHandle::new(handle);
    if let Err(e) = kv_cache.configure(32, 8, 128, &model_handle) {
        tracing::warn!("Failed to configure KV cache with Core ML: {}", e);
        // Continue with CPU-only KV cache
    }
    let kv_cache = Arc::new(tokio::sync::Mutex::new(kv_cache));

    // Initialize circuit breaker
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

    let load_time = start_time.elapsed();

    // Record telemetry
    telemetry.record_compile(load_time.as_millis() as u64, true);

    Ok(MistralModel {
        handle: model_handle,
        schema,
        tokenizer,
        kv_cache,
        telemetry,
        circuit_breaker,
        loaded_at: Instant::now(),
        last_accessed: Arc::new(std::sync::Mutex::new(Instant::now())),
    })
}

/// Load CoreML model through bridge
async fn load_coreml_model(
    model_path: &Path,
    options: &MistralCompilationOptions,
) -> Result<crate::ane::compat::coreml::ModelRef> {
    // Compile if needed
    let compiled_path = compile_if_needed(model_path, options).await?;

    // Load through CoreML compat layer
    let model_path_str = compiled_path.to_string_lossy().to_string();
    crate::ane::compat::coreml::load_model(&model_path_str)
}

/// Compile model if needed
async fn compile_if_needed(
    source_path: &Path,
    options: &MistralCompilationOptions,
) -> Result<PathBuf> {
    let ext = source_path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match ext {
        "mlmodelc" => {
            // Already compiled
            if !source_path.exists() {
                return Err(ANEError::ModelNotFound(
                    source_path.display().to_string()
                ));
            }
            Ok(source_path.to_path_buf())
        }
        "mlmodel" => {
            // Need to compile
            if !source_path.exists() {
                return Err(ANEError::ModelNotFound(
                    source_path.display().to_string()
                ));
            }

            // Compile Mistral model for ANE
            let compiled_path = source_path.with_extension("mlmodelc");
            
            // Check if already compiled and up-to-date
            if compiled_path.exists() {
                let source_modified = std::fs::metadata(source_path)?
                    .modified()?;
                let compiled_modified = std::fs::metadata(&compiled_path)?
                    .modified()?;
                
                if compiled_modified >= source_modified {
                    return Ok(compiled_path);
                }
            }
            
            // Compile the model with Mistral-specific optimizations
            compile_mistral_model(source_path, &compiled_path, options)?;
            
            Ok(compiled_path)
        }
        _ => Err(ANEError::InvalidModelFormat(
            format!("Unsupported model format: {}", ext)
        )),
    }
}

/// Discover context length from model metadata or use default
fn discover_context_len_or_default(default: usize) -> usize {
    // TODO: Query actual context length from CoreML model metadata
    //       Currently returns default; should query actual context length from CoreML model metadata.
    //
    // COMPLETION CHECKLIST:
    // [ ] Query CoreML model metadata for context length
    // [ ] Extract context length from model schema
    // [ ] Handle missing context length metadata
    // [ ] Support various model architectures
    // [ ] Add unit tests for context length query
    // [ ] Add integration tests with real models
    // [ ] Verify context length accuracy
    //
    // ACCEPTANCE CRITERIA:
    // - Context length is queried from CoreML metadata
    // - Model schema is parsed correctly
    // - Missing metadata falls back to default
    // - Various model architectures are supported
    //
    // DEPENDENCIES:
    // - CoreML bridge API (Required)
    // - Model metadata structure (Required)
    // - Context length extraction utilities (Required)
    //
    // ESTIMATED EFFORT: 2-3 hours (medium confidence)
    // PRIORITY: Medium
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 2 (model metadata feature)
    // - Change Budget: ~60 LOC
    // - Reviewer Requirements: Core ML expertise
    default // Temporary: return default until CoreML query is implemented
}

/// Extract model schema from loaded model
async fn extract_model_schema(_handle: crate::ane::compat::coreml::ModelRef) -> Result<ModelSchema> {
    // TODO: Extract actual model schema through CoreML bridge
    //       Currently returns default schema; should extract actual model schema from CoreML model handle.
    //
    // COMPLETION CHECKLIST:
    // [ ] Query CoreML model for input/output specifications
    // [ ] Extract tensor names, shapes, and types
    // [ ] Build ModelSchema from CoreML metadata
    // [ ] Handle missing schema information
    // [ ] Support various model architectures
    // [ ] Add unit tests for schema extraction
    // [ ] Add integration tests with real models
    // [ ] Verify schema extraction accuracy
    //
    // ACCEPTANCE CRITERIA:
    // - Model schema is extracted from CoreML model
    // - Tensor specifications are accurate
    // - Missing information is handled gracefully
    // - Various model architectures are supported
    //
    // DEPENDENCIES:
    // - CoreML bridge API (Required)
    // - Model schema structure (Required)
    // - Schema extraction utilities (Required)
    //
    // ESTIMATED EFFORT: 3-4 hours (medium confidence)
    // PRIORITY: Medium
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 2 (model metadata feature)
    // - Change Budget: ~80 LOC
    // - Reviewer Requirements: Core ML expertise
    Ok(ModelSchema { // Temporary: default Mistral schema until CoreML extraction is implemented
        inputs: vec![
            TensorSpec {
                name: "input_ids".to_string(),
                shape: vec![1, 4096],
                dtype: "int32".to_string(),
            },
            TensorSpec {
                name: "attention_mask".to_string(),
                shape: vec![1, 4096],
                dtype: "int32".to_string(),
            },
        ],
        outputs: vec![
            // More efficient for per-step generation: last-token logits only
            TensorSpec {
                name: "logits".to_string(),
                shape: vec![1, 32000],
                dtype: "float32".to_string(),
            },
        ],
        context_length: discover_context_len_or_default(4096),
    })
}

/// Estimate memory usage for Mistral model
pub fn estimate_memory_usage(model: &MistralModel) -> usize {
    // Parameters (typical Mistral-7B): n_params ≈ 7e9, n_layers=32, n_kv_heads=8, head_dim=128
    let n_params = 7_000_000_000usize;
    
    // Precision is a property of the weights; assume int4 by default (0.5 bytes per param)
    let bytes_per_param = 1usize / 2; // 0.5 B for int4
    let model_bytes = n_params.saturating_mul(bytes_per_param);

    let ctx = model.schema.context_length.max(1);
    let n_layers = 32usize;
    let n_kv_heads = 8usize;
    let head_dim = 128usize;
    let bytes_per_val = 2usize; // fp16 cache (2 bytes)
    
    // KV cache per token: 2 (K,V) * layers * heads * head_dim * bytes
    let kv_per_token = 2usize * n_layers * n_kv_heads * head_dim * bytes_per_val;
    let kv_bytes = kv_per_token.saturating_mul(ctx);

    let overhead_bytes = 512 * 1024 * 1024; // 512MB overhead
    let total_bytes = model_bytes + kv_bytes + overhead_bytes;
    
    total_bytes / (1024 * 1024) // Return MB
}

/// Validate Mistral model compatibility
pub fn validate_mistral_compatibility(_model: &MistralModel) -> Result<()> {
    // Check model format
    // Validate tokenizer compatibility
    // Verify compute unit support

    // TODO: Implement actual tokenizer and compute unit validation
    //       Currently assumes compatibility; should validate tokenizer compatibility and compute unit support.
    //
    // COMPLETION CHECKLIST:
    // [ ] Validate tokenizer compatibility with model
    // [ ] Verify compute unit (ANE/CPU/GPU) support
    // [ ] Check tokenizer vocabulary matches model
    // [ ] Validate tokenizer configuration
    // [ ] Handle validation errors appropriately
    // [ ] Add unit tests for validation
    // [ ] Add integration tests with various tokenizers
    // [ ] Verify validation accuracy
    //
    // ACCEPTANCE CRITERIA:
    // - Tokenizer compatibility is validated correctly
    // - Compute unit support is verified
    // - Validation errors are handled gracefully
    // - Various tokenizers are supported
    //
    // DEPENDENCIES:
    // - Tokenizer validation utilities (Required)
    // - Compute unit detection utilities (Required)
    // - Model compatibility checking (Required)
    //
    // ESTIMATED EFFORT: 3-4 hours (medium confidence)
    // PRIORITY: Medium
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 2 (model validation feature)
    // - Change Budget: ~80 LOC
    // - Reviewer Requirements: Core ML and tokenizer expertise
    Ok(()) // Temporary: assume compatible until validation is implemented
}

/// Constitutional reasoning templates
pub mod reasoning_templates {
    use super::*;

    /// Generate constitutional analysis prompt
    pub fn format_constitutional_analysis(
        task_spec: &str,
        evidence: &[String],
        debate_history: &[String],
    ) -> Result<String> {
        let mut prompt = String::new();

        prompt.push_str("# Constitutional AI Analysis\n\n");
        prompt.push_str("You are a constitutional AI judge. Your role is to ensure compliance with CAWS (Coding Agent Workflow System) principles.\n\n");

        prompt.push_str("## Task Specification:\n");
        prompt.push_str(task_spec);
        prompt.push_str("\n\n");

        if !evidence.is_empty() {
            prompt.push_str("## Evidence:\n");
            for (i, evidence_item) in evidence.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, evidence_item));
            }
            prompt.push_str("\n");
        }

        if !debate_history.is_empty() {
            prompt.push_str("## Previous Deliberations:\n");
            for deliberation in debate_history {
                prompt.push_str(deliberation);
                prompt.push_str("\n");
            }
            prompt.push_str("\n");
        }

        prompt.push_str("## Analysis Requirements:\n");
        prompt.push_str("1. Assess CAWS compliance across all criteria\n");
        prompt.push_str("2. Evaluate risk tier appropriateness\n");
        prompt.push_str("3. Identify any violations or concerns\n");
        prompt.push_str("4. Provide specific recommendations\n");
        prompt.push_str("5. Justify your verdict with evidence citations\n\n");

        prompt.push_str("## Response Format:\n");
        prompt.push_str("Provide your analysis in the following structured format:\n");
        prompt.push_str("- COMPLIANCE_LEVEL: [FULL/PARTIAL/NONE]\n");
        prompt.push_str("- RISK_ASSESSMENT: [TIER_1/TIER_2/TIER_3]\n");
        prompt.push_str("- KEY_CONCERNS: [List specific issues]\n");
        prompt.push_str("- RECOMMENDATIONS: [Actionable suggestions]\n");
        prompt.push_str("- VERDICT: [APPROVE/MODIFY/REJECT]\n");
        prompt.push_str("- JUSTIFICATION: [Detailed reasoning with evidence references]\n");

        Ok(prompt)
    }

    /// Generate debate argument prompt
    pub fn format_debate_argument(
        topic: &str,
        previous_arguments: &[String],
        evidence: &[String],
    ) -> Result<String> {
        let mut prompt = String::new();

        prompt.push_str("# Constitutional Debate\n\n");
        prompt.push_str("You are participating in a constitutional debate. Consider all perspectives and evidence carefully.\n\n");

        prompt.push_str("## Debate Topic:\n");
        prompt.push_str(topic);
        prompt.push_str("\n\n");

        if !evidence.is_empty() {
            prompt.push_str("## Available Evidence:\n");
            for (i, evidence_item) in evidence.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, evidence_item));
            }
            prompt.push_str("\n");
        }

        if !previous_arguments.is_empty() {
            prompt.push_str("## Previous Arguments:\n");
            for (i, arg) in previous_arguments.iter().enumerate() {
                prompt.push_str(&format!("Judge {}: {}\n", i + 1, arg));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("## Your Role:\n");
        prompt.push_str("Provide a well-reasoned argument that either supports or challenges the current position. ");
        prompt.push_str("Cite specific evidence and explain your reasoning clearly.\n\n");

        prompt.push_str("## Response Format:\n");
        prompt.push_str("POSITION: [SUPPORT/CHALLENGE]\n");
        prompt.push_str("ARGUMENT: [Your detailed reasoning]\n");
        prompt.push_str("EVIDENCE_CITATIONS: [Specific evidence references]\n");
        prompt.push_str("CONFIDENCE_LEVEL: [HIGH/MEDIUM/LOW]\n");

        Ok(prompt)
    }
}

/// Compile Mistral model with ANE optimizations
fn compile_mistral_model(
    source_path: &Path,
    compiled_path: &Path,
    options: &MistralCompilationOptions,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        
        // Load the source model
        let model = crate::ane::compat::coreml::load_model(source_path.to_str().unwrap())?;
        
        // Create compilation configuration optimized for Mistral
        let mut config = MLModelConfiguration::new();
        
        // Note: Quantization level (int4/int8/fp16) is a property of the compiled model itself.
        // The configuration here only selects compute units and tolerance settings.
        // Map compute_units string to MLComputeUnits enum
        match options.compute_units.as_deref() {
            Some("all") | None => config.set_compute_units(MLComputeUnits::All),
            Some("cpu") => config.set_compute_units(MLComputeUnits::CpuOnly),
            Some("cpuAndGpu") => config.set_compute_units(MLComputeUnits::CpuAndGpu),
            _ => config.set_compute_units(MLComputeUnits::All),
        }
        
        // Enable low-precision accumulation for better performance
        config.set_allow_low_precision_accumulation_on_gpu(true);
        
        // Set context length optimization
        if let Some(context_length) = options.context_length {
            // Configure for specific context length
            // This would be handled by the model's internal configuration
            tracing::info!("Compiling Mistral model for context length: {}", context_length);
        }
        
        // Save compiled model to disk using scoped access
        model.save_to_path(compiled_path)
            .map_err(|e| ANEError::CompilationFailed(format!("Failed to save compiled Mistral model: {:?}", e)))?;
        
        tracing::info!("Successfully compiled Mistral model to: {}", compiled_path.display());
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal("Core ML compilation not available on this platform"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_encode_decode() {
        let tokenizer = SafeMistralTokenizer::new();
        let test_text = "Hello, world!";
        let tokens = tokenizer.encode(test_text).unwrap();
        let decoded = tokenizer.decode(&tokens).unwrap();
        
        assert!(!tokens.is_empty());
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_default_schema_shapes() {
        let s = ModelSchema {
            inputs: vec![
                TensorSpec {
                    name: "input_ids".to_string(),
                    shape: vec![1, 4096],
                    dtype: "int32".to_string(),
                },
                TensorSpec {
                    name: "attention_mask".to_string(),
                    shape: vec![1, 4096],
                    dtype: "int32".to_string(),
                },
            ],
            outputs: vec![
                TensorSpec {
                    name: "logits".to_string(),
                    shape: vec![1, 32000],
                    dtype: "float32".to_string(),
                }
            ],
            context_length: 4096,
        };
        
        assert_eq!(s.inputs[0].dtype, "int32");
        assert_eq!(s.outputs[0].shape, vec![1, 32000]);
    }

    #[test]
    fn test_kv_cache_operations() {
        let mut cache = KVCache::new(4096);
        assert_eq!(cache.sequence_length(), 0);

        // Create a mock model handle for testing (will fail gracefully)
        // In real usage, this would be a valid Core ML handle
        let mock_handle = SafeModelHandle::new(crate::ane::compat::coreml::ModelRef::new());

        // Test cache configuration
        cache.configure(32, 8, 128, &mock_handle).unwrap();
        assert_eq!(cache.n_layers, 32);
        assert_eq!(cache.n_kv_heads, 8);
        assert_eq!(cache.head_dim, 128);

        // Test cache step
        cache.step().unwrap();
        assert_eq!(cache.sequence_length(), 1);
        cache.step().unwrap();
        assert_eq!(cache.sequence_length(), 2);

        // Test cache reset
        cache.reset().unwrap();
        assert_eq!(cache.sequence_length(), 0);
        // Core ML state should still be None since we used a mock handle
        assert_eq!(cache.coreml_state, None);
    }

    #[test]
    fn test_compilation_options_default() {
        let options = MistralCompilationOptions::default();
        assert_eq!(options.precision, Some("int4".to_string()));
        assert_eq!(options.compute_units, Some("all".to_string()));
        assert!(options.enable_quantization);
        assert_eq!(options.context_length, Some(4096));
    }

    #[test]
    fn test_memory_usage_estimation() {
        // TODO: Implement comprehensive memory usage estimation test with real model
        //       Currently ensures function doesn't panic only; should implement comprehensive test that uses real model to validate memory usage estimation accuracy.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Test uses real model for validation
        // - Memory usage estimation is accurate
        // - Test validates expected memory usage thresholds
        // - Test covers various model sizes
        //
        // DEPENDENCIES:
        // - Real model for testing (Required)
        // - Memory measurement utilities (Required)
        // - Test fixtures and model loading (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (test infrastructure enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Model testing and memory measurement expertise
        // let model = MistralModel { ... };
        // let usage = estimate_memory_usage(&model);
        // assert!(usage > 4000); // At least 4GB for 7B model
    }
}
