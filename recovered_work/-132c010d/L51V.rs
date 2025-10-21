//! Core ML provider for Apple Silicon optimized models

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;

use super::{ModelProvider, ModelContext, ModelResponse, ModelInfo, ModelCapabilities, HealthStatus, ModelError};
use crate::types::IterationContext;

// Tokenizer imports
use tokenizers::tokenizer::{Tokenizer, EncodeInput, Encoding};
use tokenizers::models::bpe::BPE;
use tokenizers::normalizers::{NormalizerWrapper, Sequence, Lowercase, NFD, StripAccents};
use tokenizers::pre_tokenizers::{PreTokenizerWrapper, Whitespace, ByteLevel};
use tokenizers::processors::PostProcessorWrapper;
use tokenizers::decoders::{DecoderWrapper, ByteLevel as ByteLevelDecoder};
use std::path::Path;
use once_cell::sync::Lazy;

/// Advanced tokenizer configuration for Core ML models
#[derive(Debug, Clone)]
pub struct TokenizerConfig {
    pub vocab_size: usize,
    pub max_length: usize,
    pub pad_token: String,
    pub eos_token: String,
    pub bos_token: String,
    pub unk_token: String,
    pub model_type: TokenizerModelType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenizerModelType {
    BPE,
    WordPiece,
    Unigram,
    ByteLevel,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            vocab_size: 30000,
            max_length: 512,
            pad_token: "[PAD]".to_string(),
            eos_token: "[EOS]".to_string(),
            bos_token: "[BOS]".to_string(),
            unk_token: "[UNK]".to_string(),
            model_type: TokenizerModelType::BPE,
        }
    }
}

/// Tokenizer wrapper with caching and performance optimizations
#[derive(Debug)]
pub struct CoreMLTokenizer {
    tokenizer: Arc<Tokenizer>,
    config: TokenizerConfig,
    cache: HashMap<String, Vec<i32>>,
    max_cache_size: usize,
}

impl CoreMLTokenizer {
    /// Create a new tokenizer with BPE model
    pub fn new(config: TokenizerConfig) -> Result<Self> {
        let tokenizer = Self::create_tokenizer(&config)?;

        Ok(Self {
            tokenizer: Arc::new(tokenizer),
            config,
            cache: HashMap::new(),
            max_cache_size: 1000,
        })
    }

    /// Create tokenizer from configuration
    fn create_tokenizer(config: &TokenizerConfig) -> Result<Tokenizer> {
        match config.model_type {
            TokenizerModelType::BPE => Self::create_bpe_tokenizer(config),
            TokenizerModelType::ByteLevel => Self::create_byte_level_tokenizer(config),
            _ => {
                // Fallback to BPE for unsupported types
                warn!("Unsupported tokenizer type {:?}, falling back to BPE", config.model_type);
                Self::create_bpe_tokenizer(config)
            }
        }
    }

    /// Create BPE tokenizer
    fn create_bpe_tokenizer(config: &TokenizerConfig) -> Result<Tokenizer> {
        // Create a basic BPE tokenizer with common settings
        // In production, this would load from a pre-trained tokenizer file
        let mut tokenizer = Tokenizer::new(
            tokenizers::models::bpe::BPE::default()
        );

        // Add normalizers
        let normalizer = NormalizerWrapper::Sequence(Sequence::new(vec![
            NormalizerWrapper::NFD(NFD::default()),
            NormalizerWrapper::Lowercase(Lowercase::default()),
            NormalizerWrapper::StripAccents(StripAccents::default()),
        ]));
        tokenizer.with_normalizer(normalizer);

        // Add pre-tokenizer
        tokenizer.with_pre_tokenizer(PreTokenizerWrapper::Whitespace(Whitespace::default()));

        // Add decoder
        tokenizer.with_decoder(DecoderWrapper::ByteLevel(ByteLevelDecoder::default()));

        // Add special tokens
        tokenizer.add_special_tokens(&[
            tokenizers::AddedToken::from(&config.pad_token, true),
            tokenizers::AddedToken::from(&config.eos_token, true),
            tokenizers::AddedToken::from(&config.bos_token, true),
            tokenizers::AddedToken::from(&config.unk_token, true),
        ]);

        // Set max length
        tokenizer.with_truncation(Some(tokenizers::tokenizer::TruncationParams {
            max_length: config.max_length,
            ..Default::default()
        }))?;

        Ok(tokenizer)
    }

    /// Create byte-level tokenizer (simplified)
    fn create_byte_level_tokenizer(config: &TokenizerConfig) -> Result<Tokenizer> {
        let mut tokenizer = Tokenizer::new(
            tokenizers::models::bpe::BPE::default()
        );

        // Add byte-level pre-tokenizer
        tokenizer.with_pre_tokenizer(PreTokenizerWrapper::ByteLevel(ByteLevel::default()));

        // Add decoder
        tokenizer.with_decoder(DecoderWrapper::ByteLevel(ByteLevelDecoder::default()));

        // Add special tokens
        tokenizer.add_special_tokens(&[
            tokenizers::AddedToken::from(&config.pad_token, true),
            tokenizers::AddedToken::from(&config.eos_token, true),
            tokenizers::AddedToken::from(&config.bos_token, true),
            tokenizers::AddedToken::from(&config.unk_token, true),
        ]);

        Ok(tokenizer)
    }

    /// Tokenize text with caching and performance optimizations
    pub fn tokenize(&mut self, text: &str) -> Result<Vec<i32>> {
        // Check cache first
        if let Some(tokens) = self.cache.get(text) {
            return Ok(tokens.clone());
        }

        // Tokenize the text
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let token_ids: Vec<i32> = encoding.get_ids().iter()
            .map(|&id| id as i32)
            .collect();

        // Cache the result if cache isn't too large
        if self.cache.len() < self.max_cache_size {
            self.cache.insert(text.to_string(), token_ids.clone());
        }

        Ok(token_ids)
    }

    /// Batch tokenize multiple texts
    pub fn tokenize_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<i32>>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let tokens = self.tokenize(text)?;
            results.push(tokens);
        }

        Ok(results)
    }

    /// Detokenize tokens back to text
    pub fn detokenize(&self, tokens: &[i32]) -> Result<String> {
        let token_ids: Vec<u32> = tokens.iter().map(|&t| t as u32).collect();
        let text = self.tokenizer.decode(&token_ids, true)
            .map_err(|e| anyhow!("Detokenization failed: {}", e))?;

        Ok(text)
    }

    /// Get tokenizer vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    /// Get special token IDs
    pub fn special_tokens(&self) -> Result<HashMap<String, i32>> {
        let mut specials = HashMap::new();

        if let Some(pad_id) = self.tokenizer.token_to_id(&self.config.pad_token) {
            specials.insert("pad".to_string(), pad_id as i32);
        }
        if let Some(eos_id) = self.tokenizer.token_to_id(&self.config.eos_token) {
            specials.insert("eos".to_string(), eos_id as i32);
        }
        if let Some(bos_id) = self.tokenizer.token_to_id(&self.config.bos_token) {
            specials.insert("bos".to_string(), bos_id as i32);
        }
        if let Some(unk_id) = self.tokenizer.token_to_id(&self.config.unk_token) {
            specials.insert("unk".to_string(), unk_id as i32);
        }

        Ok(specials)
    }

    /// Clear tokenization cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_cache_size)
    }
}

/// Global tokenizer instance for CoreML models
static GLOBAL_TOKENIZER: Lazy<Result<CoreMLTokenizer>> = Lazy::new(|| {
    let config = TokenizerConfig::default();
    CoreMLTokenizer::new(config)
});

/// Core ML provider configuration
#[derive(Debug, Clone)]
pub struct CoreMLConfig {
    pub model_name: String,
    pub model_path: Option<String>,
    pub compute_units: ComputeUnits,
    pub use_ane: bool,
    pub cache_size_mb: usize,
}

impl Default for CoreMLConfig {
    fn default() -> Self {
        Self {
            model_name: "phi-3-mini-4k-instruct".to_string(),
            model_path: None,
            compute_units: ComputeUnits::All,
            use_ane: true,
            cache_size_mb: 512,
        }
    }
}

/// Compute units for Core ML
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeUnits {
    CpuOnly,
    CpuAndGpu,
    CpuAndNeuralEngine,
    All,
}

impl From<ComputeUnits> for agent_agency_apple_silicon::inference::ComputeUnits {
    fn from(units: ComputeUnits) -> Self {
        match units {
            ComputeUnits::CpuOnly => agent_agency_apple_silicon::inference::ComputeUnits::CpuOnly,
            ComputeUnits::CpuAndGpu => agent_agency_apple_silicon::inference::ComputeUnits::CpuAndGpu,
            ComputeUnits::CpuAndNeuralEngine => agent_agency_apple_silicon::inference::ComputeUnits::CpuAndNeuralEngine,
            ComputeUnits::All => agent_agency_apple_silicon::inference::ComputeUnits::All,
        }
    }
}

/// Core ML model provider
pub struct CoreMLProvider {
    config: CoreMLConfig,
    backend: Arc<agent_agency_apple_silicon::core_ml_backend::CoreMLBackend>,
    model_info: ModelInfo,
    prepared_model: Option<Arc<agent_agency_apple_silicon::inference::PreparedModel>>,
    tokenizer: CoreMLTokenizer,
}

impl CoreMLProvider {
    /// Create a new Core ML provider
    pub async fn new(model_name: &str) -> Result<Self, ModelError> {
        Self::with_config(CoreMLConfig {
            model_name: model_name.to_string(),
            ..Default::default()
        }).await
    }

    /// Create a new Core ML provider with custom config
    pub async fn with_config(config: CoreMLConfig) -> Result<Self, ModelError> {
        let backend = Arc::new(agent_agency_apple_silicon::core_ml_backend::CoreMLBackend::new());

        // Check if we're on macOS with Apple Silicon
        if !Self::is_supported_platform() {
            return Err(ModelError::ModelUnavailable("Core ML requires macOS with Apple Silicon".to_string()));
        }

        // Initialize tokenizer with default config
        let tokenizer_config = TokenizerConfig::default();
        let tokenizer = CoreMLTokenizer::new(tokenizer_config)
            .map_err(|e| ModelError::ModelUnavailable(format!("Failed to initialize tokenizer: {}", e)))?;

        let model_info = ModelInfo {
            id: format!("coreml:{}", config.model_name),
            name: config.model_name.clone(),
            provider: "coreml".to_string(),
            capabilities: ModelCapabilities {
                max_context: 4096, // Conservative estimate, varies by model
                supports_streaming: false, // Core ML doesn't support streaming yet
                // TODO: Implement function calling support for CoreML models
                // - Add function schema definition and validation
                // - Implement function call parsing from model outputs
                // - Add function execution environment and sandboxing
                // - Support function result formatting and injection
                // - Implement function call safety and validation
                // - Add function calling metrics and monitoring
                supports_function_calling: false, // PLACEHOLDER: Not implemented
                // TODO: Implement vision capabilities for CoreML models
                // - Add image preprocessing and feature extraction
                // - Implement vision model loading and inference
                // - Support multiple image formats and sizes
                // - Add vision-specific prompt formatting
                // - Implement vision result interpretation
                // - Add vision model performance optimization
                supports_vision: false, // PLACEHOLDER: Text-only implementation
            },
        };

        Ok(Self {
            config,
            backend,
            model_info,
            prepared_model: None,
            tokenizer,
        })
    }

    /// Check if the current platform supports Core ML
    fn is_supported_platform() -> bool {
        cfg!(target_os = "macos") && std::env::consts::ARCH == "aarch64"
    }

    /// Prepare the model for inference
    async fn prepare_model(&mut self) -> Result<(), ModelError> {
        if self.prepared_model.is_some() {
            return Ok(());
        }

        // Load model from path or use built-in model
        let model_path = match &self.config.model_path {
            Some(path) => path.clone(),
            None => {
                // Use built-in model path - this would need to be configured
                return Err(ModelError::ConfigError("Model path not specified".to_string()));
            }
        };

        // Create model artifact
        let model_artifact = agent_agency_apple_silicon::inference::ModelArtifact {
            path: model_path.into(),
            model_type: agent_agency_apple_silicon::inference::ModelType::TextGeneration,
            metadata: HashMap::new(),
        };

        // Prepare model
        let prepare_options = agent_agency_apple_silicon::inference::PrepareOptions {
            compute_units: self.config.compute_units.into(),
            use_ane: self.config.use_ane,
            cache_size_mb: self.config.cache_size_mb,
        };

        let prepared = self.backend.prepare_model(model_artifact, prepare_options)
            .await
            .map_err(|e| ModelError::ModelUnavailable(format!("Failed to prepare model: {}", e)))?;

        self.prepared_model = Some(Arc::new(prepared));
        Ok(())
    }

    /// Generate text using the prepared model
    async fn generate_text(&self, input_text: &str, context: &ModelContext) -> Result<String, ModelError> {
        let prepared = self.prepared_model.as_ref()
            .ok_or_else(|| ModelError::ModelUnavailable("Model not prepared".to_string()))?;

        // Create input tensors
        let mut inputs = HashMap::new();

        // Tokenize input text using the proper tokenizer
        let input_ids = self.tokenize_text(input_text)?;

        // Create tensor batch
        let batch = agent_agency_apple_silicon::inference::TensorBatch {
            tensors: vec![agent_agency_apple_silicon::inference::Tensor {
                name: "input_ids".to_string(),
                data: input_ids,
                shape: vec![1, input_ids.len() as i64],
                dtype: agent_agency_apple_silicon::inference::DType::I32,
            }],
        };

        // Run inference
        let outputs = self.backend.run_inference(prepared.as_ref(), batch)
            .await
            .map_err(|e| ModelError::ModelUnavailable(format!("Inference failed: {}", e)))?;

        // Extract output text (simplified - would need proper detokenization)
        self.detokenize_output(outputs)
    }

    /// Simple tokenization (placeholder - would use proper tokenizer)
    fn tokenize_text(&mut self, text: &str) -> Result<Vec<i32>, ModelError> {
        // Use the proper tokenizer with BPE and special token support
        self.tokenizer.tokenize(text)
            .map_err(|e| ModelError::InferenceError(format!("Tokenization failed: {}", e)))
    }

    /// Detokenize model output using proper tokenizer
    fn detokenize_output(&self, outputs: agent_agency_apple_silicon::inference::TensorMap) -> Result<String, ModelError> {
        // Extract token IDs from the output tensor
        if let Some(output_tensor) = outputs.tensors.first() {
            // Use the tokenizer to decode the tokens back to text
            self.tokenizer.detokenize(&output_tensor.data)
                .map_err(|e| ModelError::InferenceError(format!("Detokenization failed: {}", e)))
        } else {
            Err(ModelError::InvalidResponse("No output tensors".to_string()))
        }
    }

    /// Build the full prompt with context
    fn build_prompt(&self, base_prompt: &str, context: &ModelContext) -> String {
        let mut full_prompt = String::new();

        // Add system instructions
        full_prompt.push_str("You are an autonomous AI agent working on iterative code improvement tasks.\n");
        full_prompt.push_str("You will receive feedback on your previous outputs and should improve them.\n\n");

        // Add task history for context
        if !context.task_history.is_empty() {
            full_prompt.push_str("Previous iterations:\n");
            for (i, iteration) in context.task_history.iter().enumerate() {
                full_prompt.push_str(&format!("Iteration {}:\n", i + 1));
                full_prompt.push_str(&format!("Output: {}\n", iteration.previous_output));
                full_prompt.push_str(&format!("Feedback: {}\n\n", iteration.eval_report.score));
            }
        }

        // Add current task
        full_prompt.push_str("Current task:\n");
        full_prompt.push_str(base_prompt);
        full_prompt.push_str("\n\n");

        // Add output instructions
        full_prompt.push_str("Provide your response as a unified diff when making code changes, or as plain text for other tasks.\n");

        full_prompt
    }
}

#[async_trait]
impl ModelProvider for CoreMLProvider {
    async fn generate(&self, prompt: &str, context: &ModelContext) -> Result<ModelResponse, ModelError> {
        // Prepare model if needed
        if self.prepared_model.is_none() {
            return Err(ModelError::ModelUnavailable("Model not prepared. Call prepare_model() first.".to_string()));
        }

        let full_prompt = self.build_prompt(prompt, context);

        let start_time = std::time::Instant::now();

        // Generate text
        let output_text = self.generate_text(&full_prompt, context).await?;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Estimate token usage (simplified)
        let prompt_tokens = full_prompt.split_whitespace().count();
        let completion_tokens = output_text.split_whitespace().count();
        let tokens_used = prompt_tokens + completion_tokens;

        Ok(ModelResponse {
            text: output_text,
            model_id: self.model_info.id.clone(),
            tokens_used,
            latency_ms,
            finish_reason: Some("completed".to_string()),
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, ModelError> {
        // Check if platform is supported
        if !Self::is_supported_platform() {
            return Ok(HealthStatus {
                healthy: false,
                last_check: Utc::now(),
                error_message: Some("Core ML requires macOS with Apple Silicon".to_string()),
            });
        }

        // Check if model can be prepared
        let mut temp_provider = Self::with_config(self.config.clone()).await
            .map_err(|e| ModelError::ModelUnavailable(format!("Model setup failed: {}", e)))?;

        match temp_provider.prepare_model().await {
            Ok(_) => Ok(HealthStatus {
                healthy: true,
                last_check: Utc::now(),
                error_message: None,
            }),
            Err(e) => Ok(HealthStatus {
                healthy: false,
                last_check: Utc::now(),
                error_message: Some(format!("Model preparation failed: {}", e)),
            }),
        }
    }

    fn model_info(&self) -> ModelInfo {
        self.model_info.clone()
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.model_info.capabilities.clone()
    }
}
