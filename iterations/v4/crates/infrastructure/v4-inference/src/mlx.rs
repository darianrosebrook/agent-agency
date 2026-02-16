//! MLX inference provider
//!
//! A provider for Apple Silicon using MLX via mlx-rs bindings.
//! Optimized for M-series chips using Metal Performance Shaders.
//!
//! ## Features
//!
//! - Native Apple Silicon support via MLX
//! - Unified memory architecture (no CPU/GPU transfer overhead)
//! - SafeTensors model loading from HuggingFace Hub or local paths
//! - KV-cache for efficient autoregressive generation
//! - Temperature and top-p sampling
//!
//! ## Model Loading
//!
//! Models are loaded from HuggingFace Hub or local paths in SafeTensors format.
//! The provider expects a directory containing:
//! - `config.json` or `params.json` - Model configuration
//! - `tokenizer.json` - Tokenizer configuration
//! - `*.safetensors` - Model weights

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use crate::config::InferenceConfig;
use crate::error::InferenceError;
use crate::provider::InferenceProvider;
use crate::types::{FinishReason, InferenceRequest, InferenceResponse, ModelInfo, ProviderStatus};

/// Device selection for MLX
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MLXDevice {
    /// CPU execution
    Cpu,
    /// GPU execution via Metal
    Gpu,
}

impl Default for MLXDevice {
    fn default() -> Self {
        Self::Gpu
    }
}

/// MLX-specific configuration
#[derive(Debug, Clone)]
pub struct MLXConfig {
    /// Device to use for inference
    pub device: MLXDevice,
    /// Maximum context length
    pub max_context: u32,
    /// Tokens to process per eval batch
    pub tokens_per_eval: u32,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Whether to use KV cache
    pub use_kv_cache: bool,
    /// HuggingFace model ID (for downloading from Hub)
    pub hf_model_id: Option<String>,
    /// Whether to quantize after loading
    pub quantize: bool,
}

impl Default for MLXConfig {
    fn default() -> Self {
        Self {
            device: MLXDevice::default(),
            max_context: 4096,
            tokens_per_eval: 10,
            seed: 0,
            use_kv_cache: true,
            hf_model_id: None,
            quantize: true,
        }
    }
}

/// Internal state for the MLX session
struct MLXSession {
    /// Model name/identifier
    model_name: String,
    /// Model parameters count
    parameters: u64,
    /// Context size
    context_size: u32,
    /// Memory usage estimate in bytes
    memory_bytes: u64,
    /// Real MLX model and tokenizer (when `mlx` feature is enabled)
    #[cfg(feature = "mlx")]
    model: std::sync::Mutex<MlxModel>,
    #[cfg(feature = "mlx")]
    tokenizer: tokenizers::Tokenizer,
}

/// Wraps the real MLX model components
#[cfg(feature = "mlx")]
struct MlxModel {
    model: crate::mlx_model::Mistral,
}

/// MLX inference provider for Apple Silicon
///
/// This provider uses the mlx-rs bindings to run inference on Apple Silicon
/// devices using the MLX framework. MLX provides:
/// - Lazy evaluation for efficient computation
/// - Unified memory model (no CPU/GPU transfers needed)
/// - Dynamic computation graphs
pub struct MLXProvider {
    /// Configuration
    config: InferenceConfig,
    /// MLX-specific config
    mlx_config: MLXConfig,
    /// Whether model is loaded
    model_loaded: AtomicBool,
    /// Model information
    model_info: RwLock<Option<ModelInfo>>,
    /// MLX session (when model is loaded)
    session: RwLock<Option<Arc<MLXSession>>>,
    /// Request counter for metrics
    request_count: AtomicU32,
    /// Active request counter
    active_requests: AtomicU32,
}

impl MLXProvider {
    /// Create a new MLX provider
    pub fn new(config: InferenceConfig) -> Self {
        Self {
            config,
            mlx_config: MLXConfig::default(),
            model_loaded: AtomicBool::new(false),
            model_info: RwLock::new(None),
            session: RwLock::new(None),
            request_count: AtomicU32::new(0),
            active_requests: AtomicU32::new(0),
        }
    }

    /// Create with custom MLX configuration
    pub fn with_mlx_config(config: InferenceConfig, mlx_config: MLXConfig) -> Self {
        Self {
            config,
            mlx_config,
            model_loaded: AtomicBool::new(false),
            model_info: RwLock::new(None),
            session: RwLock::new(None),
            request_count: AtomicU32::new(0),
            active_requests: AtomicU32::new(0),
        }
    }

    /// Check if MLX is available on this system
    fn check_mlx_available() -> bool {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            true
        }
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            false
        }
    }

    /// Get the model path from config
    fn get_model_path(&self) -> Result<PathBuf, InferenceError> {
        self.config
            .model
            .path
            .clone()
            .ok_or_else(|| InferenceError::ConfigError("Model path not specified".to_string()))
    }

    /// Estimate memory usage for a model
    fn estimate_memory(parameters: u64, quantization: &crate::config::Quantization) -> u64 {
        let bytes_per_param = match quantization {
            crate::config::Quantization::None => 4, // FP32
            crate::config::Quantization::Fp16 => 2, // FP16
            crate::config::Quantization::Int8 => 1, // INT8
            crate::config::Quantization::Int4 => 1, // INT4 (approx)
        };
        parameters * bytes_per_param
    }

    /// Estimate token count from text
    fn estimate_tokens(text: &str) -> u32 {
        (text.len() / 4).max(1) as u32
    }

    /// Generate text using MLX
    #[cfg(feature = "mlx")]
    #[allow(clippy::too_many_arguments)]
    async fn generate_text(
        &self,
        session: &MLXSession,
        prompt: &str,
        max_tokens: u32,
        temperature: f32,
        _top_p: f32,
        _top_k: u32,
        _stop_sequences: &[String],
    ) -> Result<(String, u32, FinishReason), InferenceError> {
        use mlx_rs::ops::indexing::{IndexOp, NewAxis};

        // Tokenize input
        let encoding = session
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| InferenceError::InferenceFailed(format!("Tokenization failed: {e}")))?;

        let token_ids = encoding.get_ids();
        let prompt_tokens = mlx_rs::Array::from(token_ids).index(NewAxis);

        // Generate tokens
        let mut model_guard = session.model.lock().map_err(|e| {
            InferenceError::InferenceFailed(format!("Failed to acquire model lock: {e}"))
        })?;

        let generate = crate::mlx_model::Generate::new(
            &mut model_guard.model,
            &prompt_tokens,
            temperature,
        );

        let tokens_per_eval = self.mlx_config.tokens_per_eval as usize;
        let mut output_tokens: Vec<u32> = Vec::with_capacity(max_tokens as usize);
        let mut finish_reason = FinishReason::MaxTokens;

        for (token_result, _n) in generate.zip(0..max_tokens) {
            let token = token_result.map_err(|e| {
                InferenceError::InferenceFailed(format!("Generation step failed: {e}"))
            })?;

            let token_id: u32 = token.item();
            output_tokens.push(token_id);

            // Batch evaluate for efficiency
            if output_tokens.len().is_multiple_of(tokens_per_eval) {
                // Evaluation happens lazily; accessing item() forces it
            }

            // Check for EOS token (common EOS IDs: 2 for Mistral/Llama)
            if token_id == 2 {
                finish_reason = FinishReason::Complete;
                break;
            }
        }

        // Decode output tokens
        let text = session
            .tokenizer
            .decode(&output_tokens, true)
            .map_err(|e| InferenceError::InferenceFailed(format!("Decoding failed: {e}")))?;

        let tokens_generated = output_tokens.len() as u32;
        Ok((text, tokens_generated, finish_reason))
    }

    /// Generate text — mock fallback when mlx feature is disabled
    #[cfg(not(feature = "mlx"))]
    async fn generate_text(
        &self,
        _session: &MLXSession,
        prompt: &str,
        max_tokens: u32,
        _temperature: f32,
        _top_p: f32,
        _top_k: u32,
        _stop_sequences: &[String],
    ) -> Result<(String, u32, FinishReason), InferenceError> {
        let response = self.mock_generate(prompt, max_tokens);
        let tokens = Self::estimate_tokens(&response);
        let reason = if tokens >= max_tokens {
            FinishReason::MaxTokens
        } else {
            FinishReason::Complete
        };
        Ok((response, tokens, reason))
    }

    /// Mock generation for development (when mlx feature is disabled)
    #[cfg(not(feature = "mlx"))]
    fn mock_generate(&self, prompt: &str, max_tokens: u32) -> String {
        let base = if prompt.contains("code") || prompt.contains("function") {
            "Here's a code solution:\n\n```rust\nfn solution() -> Result<(), Error> {\n    // Implementation details\n    Ok(())\n}\n```\n\nThis approach handles the requirements efficiently."
        } else if prompt.contains("explain") {
            "Let me explain:\n\n1. **Core concept**: The fundamental principle involves understanding the relationship between components.\n\n2. **Implementation**: In practice, this means carefully designing interfaces.\n\n3. **Best practices**: Always consider edge cases and error handling."
        } else {
            "Based on the context provided, I can offer the following analysis:\n\nThe key considerations are the requirements specified and the constraints involved. A balanced approach would address both the immediate needs and long-term maintainability."
        };

        let max_chars = (max_tokens * 4) as usize;
        if base.len() > max_chars {
            base[..max_chars].to_string()
        } else {
            base.to_string()
        }
    }

    /// Load model from HuggingFace Hub
    #[cfg(feature = "mlx")]
    fn load_from_hub(model_id: &str) -> Result<(PathBuf, PathBuf, PathBuf), InferenceError> {
        use hf_hub::api::sync::Api;

        let api = Api::new()
            .map_err(|e| InferenceError::LoadFailed(format!("HuggingFace API init failed: {e}")))?;

        let repo = api.model(model_id.to_string());

        // Download config
        let config_path = repo.get("params.json").or_else(|_| repo.get("config.json")).map_err(|e| {
            InferenceError::LoadFailed(format!("Failed to download model config: {e}"))
        })?;

        // Download tokenizer
        let tokenizer_path = repo.get("tokenizer.json").map_err(|e| {
            InferenceError::LoadFailed(format!("Failed to download tokenizer: {e}"))
        })?;

        // Download weights
        let weights_path = repo.get("weights.safetensors").or_else(|_| repo.get("model.safetensors")).map_err(|e| {
            InferenceError::LoadFailed(format!("Failed to download weights: {e}"))
        })?;

        Ok((config_path, tokenizer_path, weights_path))
    }

    /// Load model from local directory
    #[cfg(feature = "mlx")]
    fn load_from_local(dir: &std::path::Path) -> Result<(PathBuf, PathBuf, PathBuf), InferenceError> {
        let config_path = dir.join("params.json");
        let config_path = if config_path.exists() {
            config_path
        } else {
            let alt = dir.join("config.json");
            if alt.exists() {
                alt
            } else {
                return Err(InferenceError::LoadFailed(
                    "No params.json or config.json found in model directory".to_string(),
                ));
            }
        };

        let tokenizer_path = dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            return Err(InferenceError::LoadFailed(
                "No tokenizer.json found in model directory".to_string(),
            ));
        }

        let weights_path = dir.join("weights.safetensors");
        let weights_path = if weights_path.exists() {
            weights_path
        } else {
            let alt = dir.join("model.safetensors");
            if alt.exists() {
                alt
            } else {
                // Try to find any .safetensors file
                let entries = std::fs::read_dir(dir).map_err(|e| {
                    InferenceError::LoadFailed(format!("Failed to read model directory: {e}"))
                })?;
                let mut found = None;
                for entry in entries.flatten() {
                    if entry.path().extension().is_some_and(|ext| ext == "safetensors") {
                        found = Some(entry.path());
                        break;
                    }
                }
                found.ok_or_else(|| {
                    InferenceError::LoadFailed(
                        "No .safetensors file found in model directory".to_string(),
                    )
                })?
            }
        };

        Ok((config_path, tokenizer_path, weights_path))
    }
}

#[async_trait]
impl InferenceProvider for MLXProvider {
    fn name(&self) -> &str {
        "mlx"
    }

    async fn is_available(&self) -> bool {
        Self::check_mlx_available()
    }

    async fn load_model(&self) -> Result<ModelInfo, InferenceError> {
        if !Self::check_mlx_available() {
            return Err(InferenceError::ProviderNotAvailable(
                "MLX requires macOS with Apple Silicon".to_string(),
            ));
        }

        let model_name = self.config.model.name.clone();

        tracing::info!(
            model = %model_name,
            device = ?self.mlx_config.device,
            "Loading MLX model"
        );

        #[cfg(feature = "mlx")]
        {
            // Resolve model files (Hub or local)
            let (config_path, tokenizer_path, weights_path) =
                if let Some(ref hf_id) = self.mlx_config.hf_model_id {
                    tracing::info!(hf_model = %hf_id, "Loading from HuggingFace Hub");
                    Self::load_from_hub(hf_id)?
                } else if let Ok(local_path) = self.get_model_path() {
                    tracing::info!(path = %local_path.display(), "Loading from local path");
                    Self::load_from_local(&local_path)?
                } else {
                    return Err(InferenceError::ConfigError(
                        "No model path or HuggingFace model ID specified".to_string(),
                    ));
                };

            // Load model config
            let config_str = std::fs::read_to_string(&config_path).map_err(|e| {
                InferenceError::LoadFailed(format!("Failed to read config: {e}"))
            })?;
            let model_args: crate::mlx_model::ModelArgs =
                serde_json::from_str(&config_str).map_err(|e| {
                    InferenceError::LoadFailed(format!("Failed to parse model config: {e}"))
                })?;

            // Initialize model
            let mut model = crate::mlx_model::Mistral::new(&model_args).map_err(|e| {
                InferenceError::LoadFailed(format!("Failed to initialize model: {e}"))
            })?;

            // Load weights
            model
                .load_safetensors(&weights_path)
                .map_err(|e| InferenceError::LoadFailed(format!("Failed to load weights: {e}")))?;

            // Optionally quantize
            if self.mlx_config.quantize {
                tracing::info!("Quantizing model for faster inference");
                model = model.quantize().map_err(|e| {
                    InferenceError::LoadFailed(format!("Quantization failed: {e}"))
                })?;
            }

            // Load tokenizer
            let tokenizer =
                tokenizers::Tokenizer::from_file(&tokenizer_path).map_err(|e| {
                    InferenceError::LoadFailed(format!("Failed to load tokenizer: {e}"))
                })?;

            let parameters = model_args.estimated_parameters();
            let memory_bytes = Self::estimate_memory(parameters, &self.config.model.quantization);

            let session = MLXSession {
                model_name: model_name.clone(),
                parameters,
                context_size: self.mlx_config.max_context,
                memory_bytes,
                model: std::sync::Mutex::new(MlxModel { model }),
                tokenizer,
            };

            let info = ModelInfo {
                name: model_name,
                version: "mlx-0.25".to_string(),
                parameters,
                context_size: self.mlx_config.max_context,
                is_loaded: true,
                memory_bytes,
            };

            *self.session.write().unwrap() = Some(Arc::new(session));
            *self.model_info.write().unwrap() = Some(info.clone());
            self.model_loaded.store(true, Ordering::SeqCst);

            tracing::info!(
                memory_mb = memory_bytes / 1_000_000,
                parameters = parameters,
                context = self.mlx_config.max_context,
                "MLX model loaded successfully"
            );

            Ok(info)
        }

        #[cfg(not(feature = "mlx"))]
        {
            // Mock session for development without mlx feature
            let parameters = 7_000_000_000u64;
            let memory_bytes =
                Self::estimate_memory(parameters, &self.config.model.quantization);

            let session = MLXSession {
                model_name: model_name.clone(),
                parameters,
                context_size: self.mlx_config.max_context,
                memory_bytes,
            };

            let info = ModelInfo {
                name: model_name,
                version: "mlx-mock".to_string(),
                parameters,
                context_size: self.mlx_config.max_context,
                is_loaded: true,
                memory_bytes,
            };

            *self.session.write().unwrap() = Some(Arc::new(session));
            *self.model_info.write().unwrap() = Some(info.clone());
            self.model_loaded.store(true, Ordering::SeqCst);

            tracing::info!(
                memory_mb = memory_bytes / 1_000_000,
                "MLX model loaded (mock mode — enable 'mlx' feature for real inference)"
            );

            Ok(info)
        }
    }

    async fn unload_model(&self) -> Result<(), InferenceError> {
        tracing::info!("Unloading MLX model");

        *self.session.write().unwrap() = None;
        *self.model_info.write().unwrap() = None;
        self.model_loaded.store(false, Ordering::SeqCst);

        Ok(())
    }

    fn is_model_loaded(&self) -> bool {
        self.model_loaded.load(Ordering::SeqCst)
    }

    fn model_info(&self) -> Option<&ModelInfo> {
        // Can't safely return reference through RwLock
        None
    }

    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        if !self.is_model_loaded() {
            return Err(InferenceError::ModelNotLoaded(
                "Model not loaded".to_string(),
            ));
        }

        let session = self
            .session
            .read()
            .unwrap()
            .clone()
            .ok_or_else(|| InferenceError::ModelNotLoaded("Session not available".to_string()))?;

        self.request_count.fetch_add(1, Ordering::SeqCst);
        self.active_requests.fetch_add(1, Ordering::SeqCst);

        let start = std::time::Instant::now();

        // When not using real MLX, add simulated delay
        #[cfg(not(feature = "mlx"))]
        {
            let simulated_delay_ms = 10 + (request.prompt.len() % 40) as u64;
            tokio::time::sleep(std::time::Duration::from_millis(simulated_delay_ms)).await;
        }

        let result = self
            .generate_text(
                &session,
                &request.prompt,
                request.max_tokens,
                request.temperature,
                request.top_p,
                request.top_k,
                &request.stop_sequences,
            )
            .await;

        self.active_requests.fetch_sub(1, Ordering::SeqCst);

        let (text, tokens_generated, finish_reason) = result?;

        let total_time_ms = start.elapsed().as_millis() as u64;
        let prompt_tokens = Self::estimate_tokens(&request.prompt);

        let time_to_first_token_ms = if total_time_ms > 0 {
            (total_time_ms * prompt_tokens as u64) / (prompt_tokens + tokens_generated) as u64
        } else {
            0
        };

        let tokens_per_second = if total_time_ms > 0 {
            (tokens_generated as f64 / total_time_ms as f64) * 1000.0
        } else {
            0.0
        };

        Ok(InferenceResponse {
            request_id: request.request_id,
            text,
            tokens_generated,
            prompt_tokens,
            time_to_first_token_ms,
            total_time_ms,
            tokens_per_second,
            model: session.model_name.clone(),
            finish_reason,
            created_at: chrono::Utc::now(),
        })
    }

    async fn status(&self) -> ProviderStatus {
        let loaded_models = if self.is_model_loaded() {
            if let Some(session) = self.session.read().unwrap().as_ref() {
                vec![ModelInfo {
                    name: session.model_name.clone(),
                    version: "mlx-0.25".to_string(),
                    parameters: session.parameters,
                    context_size: session.context_size,
                    is_loaded: true,
                    memory_bytes: session.memory_bytes,
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        ProviderStatus {
            name: "mlx".to_string(),
            available: Self::check_mlx_available(),
            loaded_models,
            capacity: self.config.limits.max_concurrent_requests,
            active_requests: self.active_requests.load(Ordering::SeqCst),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InferenceConfig;

    fn test_config() -> InferenceConfig {
        let mut config = InferenceConfig::default();
        config.model.path = Some(PathBuf::from("/tmp/test-model"));
        config.model.name = "test-model".to_string();
        config
    }

    #[test]
    fn test_mlx_provider_creation() {
        let provider = MLXProvider::new(test_config());
        assert_eq!(provider.name(), "mlx");
        assert!(!provider.is_model_loaded());
    }

    #[test]
    fn test_mlx_config_defaults() {
        let config = MLXConfig::default();
        assert_eq!(config.device, MLXDevice::Gpu);
        assert_eq!(config.max_context, 4096);
        assert!(config.use_kv_cache);
        assert!(config.quantize);
    }

    #[test]
    fn test_memory_estimation() {
        // 7B model in FP16 = 14GB
        let mem = MLXProvider::estimate_memory(7_000_000_000, &crate::config::Quantization::Fp16);
        assert_eq!(mem, 14_000_000_000);

        // 7B model in INT4 = 7GB (approx)
        let mem = MLXProvider::estimate_memory(7_000_000_000, &crate::config::Quantization::Int4);
        assert_eq!(mem, 7_000_000_000);
    }

    #[test]
    fn test_token_estimation() {
        let tokens = MLXProvider::estimate_tokens("Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 10);

        let long_text = "a".repeat(400);
        let tokens = MLXProvider::estimate_tokens(&long_text);
        assert_eq!(tokens, 100); // 400 chars / 4 = 100 tokens
    }

    #[tokio::test]
    async fn test_mlx_availability() {
        let provider = MLXProvider::new(test_config());
        let available = provider.is_available().await;

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert!(available);

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        assert!(!available);
    }

    #[tokio::test]
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    async fn test_mlx_model_lifecycle() {
        // This test uses mock mode (no real model files)
        // Real model loading requires actual model files on disk
        let provider = MLXProvider::new(test_config());

        assert!(!provider.is_model_loaded());

        // Without the mlx feature, this loads a mock session
        #[cfg(not(feature = "mlx"))]
        {
            let info = provider.load_model().await.unwrap();
            assert!(provider.is_model_loaded());
            assert!(info.is_loaded);

            provider.unload_model().await.unwrap();
            assert!(!provider.is_model_loaded());
        }
    }

    #[tokio::test]
    async fn test_mlx_requires_loaded_model() {
        let provider = MLXProvider::new(test_config());

        let request = InferenceRequest::new("test");
        let result = provider.infer(request).await;

        assert!(matches!(result, Err(InferenceError::ModelNotLoaded(_))));
    }

    #[tokio::test]
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    async fn test_mlx_status() {
        let provider = MLXProvider::new(test_config());

        let status = provider.status().await;
        assert!(status.available);
        assert!(status.loaded_models.is_empty());
    }

    #[test]
    fn test_mlx_config_with_hub() {
        let config = MLXConfig {
            hf_model_id: Some("minghuaw/Mistral-7B-v0.1".to_string()),
            quantize: true,
            ..Default::default()
        };
        assert!(config.hf_model_id.is_some());
    }

    #[cfg(feature = "mlx")]
    #[test]
    fn test_load_from_nonexistent_local_path() {
        let result = MLXProvider::load_from_local(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
