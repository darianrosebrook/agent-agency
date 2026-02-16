//! Inference provider trait
//!
//! Abstraction layer for different inference backends (Mock, CoreML, etc.)

use std::path::PathBuf;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::InferenceConfig;
use crate::error::InferenceError;
use crate::types::{InferenceRequest, InferenceResponse, ModelInfo, ProviderStatus};

/// Source from which to load a model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ModelSource {
    /// Load from a local directory containing config.json, tokenizer.json, and *.safetensors
    Local {
        /// Path to the model directory
        path: PathBuf,
    },
    /// Download from HuggingFace Hub
    HuggingFace {
        /// Model ID (e.g., "minghuaw/Mistral-7B-v0.1")
        model_id: String,
    },
}

impl ModelSource {
    /// Create a local model source
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::Local { path: path.into() }
    }

    /// Create a HuggingFace Hub model source
    pub fn huggingface(model_id: impl Into<String>) -> Self {
        Self::HuggingFace {
            model_id: model_id.into(),
        }
    }

    /// Validate that the source exists and has required files
    pub fn validate(&self) -> Result<(), InferenceError> {
        match self {
            Self::Local { path } => {
                if !path.exists() {
                    return Err(InferenceError::LoadFailed(format!(
                        "Model directory does not exist: {}",
                        path.display()
                    )));
                }
                if !path.is_dir() {
                    return Err(InferenceError::LoadFailed(format!(
                        "Model path is not a directory: {}",
                        path.display()
                    )));
                }

                // Check for config
                let has_config =
                    path.join("config.json").exists() || path.join("params.json").exists();
                if !has_config {
                    return Err(InferenceError::LoadFailed(
                        "Model directory missing config.json or params.json".to_string(),
                    ));
                }

                // Check for tokenizer
                if !path.join("tokenizer.json").exists() {
                    return Err(InferenceError::LoadFailed(
                        "Model directory missing tokenizer.json".to_string(),
                    ));
                }

                // Check for weights
                let has_weights = std::fs::read_dir(path)
                    .map(|entries| {
                        entries
                            .flatten()
                            .any(|e| e.path().extension().is_some_and(|ext| ext == "safetensors"))
                    })
                    .unwrap_or(false);
                if !has_weights {
                    return Err(InferenceError::LoadFailed(
                        "Model directory missing .safetensors weight files".to_string(),
                    ));
                }

                Ok(())
            }
            Self::HuggingFace { model_id } => {
                if model_id.is_empty() {
                    return Err(InferenceError::ConfigError(
                        "HuggingFace model ID cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }
}

/// Inference provider trait
///
/// All inference backends implement this trait, enabling swappable providers
/// for different environments (development, testing, production).
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Check if the provider is available
    async fn is_available(&self) -> bool;

    /// Load a model
    async fn load_model(&self) -> Result<ModelInfo, InferenceError>;

    /// Unload the model
    async fn unload_model(&self) -> Result<(), InferenceError>;

    /// Check if model is loaded
    fn is_model_loaded(&self) -> bool;

    /// Get current model info
    fn model_info(&self) -> Option<&ModelInfo>;

    /// Run inference
    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError>;

    /// Get provider status
    async fn status(&self) -> ProviderStatus;

    /// Health check
    async fn health_check(&self) -> Result<(), InferenceError> {
        if !self.is_available().await {
            return Err(InferenceError::ProviderNotAvailable(self.name().to_string()));
        }
        Ok(())
    }
}

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider from configuration
    pub fn create(config: &InferenceConfig) -> Box<dyn InferenceProvider> {
        match config.provider {
            crate::config::ProviderType::Mock => {
                Box::new(crate::mock::MockProvider::new(config.clone()))
            }
            crate::config::ProviderType::CoreML => {
                // CoreML provider requires the coreml feature
                #[cfg(feature = "coreml")]
                {
                    Box::new(crate::coreml::CoreMLProvider::new(config.clone()))
                }
                #[cfg(not(feature = "coreml"))]
                {
                    tracing::warn!(
                        "CoreML requested but feature not enabled, falling back to mock"
                    );
                    Box::new(crate::mock::MockProvider::new(config.clone()))
                }
            }
            crate::config::ProviderType::MLX => {
                // MLX provider requires the mlx feature and Apple Silicon
                #[cfg(feature = "mlx")]
                {
                    Box::new(crate::mlx::MLXProvider::new(config.clone()))
                }
                #[cfg(not(feature = "mlx"))]
                {
                    tracing::warn!(
                        "MLX requested but feature not enabled, falling back to mock"
                    );
                    Box::new(crate::mock::MockProvider::new(config.clone()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InferenceConfig;

    #[test]
    fn test_factory_creates_mock() {
        let config = InferenceConfig::mock();
        let provider = ProviderFactory::create(&config);
        assert_eq!(provider.name(), "mock");
    }

    #[test]
    fn test_model_source_local() {
        let source = ModelSource::local("/tmp/my-model");
        assert!(matches!(source, ModelSource::Local { .. }));
    }

    #[test]
    fn test_model_source_huggingface() {
        let source = ModelSource::huggingface("minghuaw/Mistral-7B-v0.1");
        assert!(matches!(source, ModelSource::HuggingFace { .. }));
    }

    #[test]
    fn test_model_source_validate_nonexistent_path() {
        let source = ModelSource::local("/nonexistent/path/to/model");
        let result = source.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn test_model_source_validate_empty_hf_id() {
        let source = ModelSource::huggingface("");
        let result = source.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_model_source_validate_valid_hf_id() {
        let source = ModelSource::huggingface("minghuaw/Mistral-7B-v0.1");
        assert!(source.validate().is_ok());
    }

    #[test]
    fn test_model_source_serialization() {
        let source = ModelSource::local("/tmp/model");
        let json = serde_json::to_string(&source).unwrap();
        let parsed: ModelSource = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ModelSource::Local { .. }));
    }
}
