//! Inference provider trait
//!
//! Abstraction layer for different inference backends (Mock, CoreML, etc.)

use async_trait::async_trait;

use crate::config::InferenceConfig;
use crate::error::InferenceError;
use crate::types::{InferenceRequest, InferenceResponse, ModelInfo, ProviderStatus};

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
}
