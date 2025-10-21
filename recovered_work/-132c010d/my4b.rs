//! Core ML provider for Apple Silicon optimized models

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;

use super::{ModelProvider, ModelContext, ModelResponse, ModelInfo, ModelCapabilities, HealthStatus, ModelError};
use crate::types::IterationContext;

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

        let model_info = ModelInfo {
            id: format!("coreml:{}", config.model_name),
            name: config.model_name.clone(),
            provider: "coreml".to_string(),
            capabilities: ModelCapabilities {
                max_context: 4096, // Conservative estimate, varies by model
                supports_streaming: false, // Core ML doesn't support streaming yet
                supports_function_calling: false, // Not implemented
                supports_vision: false, // Text-only for now
            },
        };

        Ok(Self {
            config,
            backend,
            model_info,
            prepared_model: None,
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

        // Convert text to tokens (simplified - would need proper tokenization)
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
    fn tokenize_text(&self, text: &str) -> Result<Vec<i32>, ModelError> {
        // This is a placeholder implementation
        // In reality, you'd use the model's tokenizer
        Ok(text.chars().map(|c| c as i32).collect())
    }

    /// Simple detokenization (placeholder - would use proper detokenizer)
    fn detokenize_output(&self, outputs: agent_agency_apple_silicon::inference::TensorMap) -> Result<String, ModelError> {
        // This is a placeholder implementation
        // In reality, you'd convert tokens back to text
        if let Some(output_tensor) = outputs.tensors.first() {
            let text: String = output_tensor.data.iter()
                .filter_map(|&token| char::from_u32(token as u32))
                .collect();
            Ok(text)
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
