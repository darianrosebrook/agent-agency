//! Inference configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Inference provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Provider type
    pub provider: ProviderType,
    /// Model configuration
    pub model: ModelConfig,
    /// Generation defaults
    pub generation: GenerationDefaults,
    /// Resource limits
    pub limits: ResourceLimits,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            provider: ProviderType::Mock,
            model: ModelConfig::default(),
            generation: GenerationDefaults::default(),
            limits: ResourceLimits::default(),
        }
    }
}

impl InferenceConfig {
    /// Create config for mock provider (development/testing)
    pub fn mock() -> Self {
        Self {
            provider: ProviderType::Mock,
            ..Default::default()
        }
    }

    /// Create config for CoreML provider (legacy)
    pub fn coreml(model_path: impl Into<PathBuf>) -> Self {
        Self {
            provider: ProviderType::CoreML,
            model: ModelConfig {
                path: Some(model_path.into()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create config for MLX provider (recommended for Apple Silicon)
    pub fn mlx(model_path: impl Into<PathBuf>) -> Self {
        Self {
            provider: ProviderType::MLX,
            model: ModelConfig {
                path: Some(model_path.into()),
                format: ModelFormat::SafeTensors,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create config from environment variables
    pub fn from_env() -> Self {
        let provider = match std::env::var("V4_INFERENCE_PROVIDER")
            .unwrap_or_else(|_| "mock".to_string())
            .to_lowercase()
            .as_str()
        {
            "coreml" => ProviderType::CoreML,
            "mlx" => ProviderType::MLX,
            _ => ProviderType::Mock,
        };

        let model_path = std::env::var("V4_MODEL_PATH").ok().map(PathBuf::from);

        let model_name = std::env::var("V4_MODEL_NAME").unwrap_or_else(|_| "mistral-7b".to_string());

        Self {
            provider,
            model: ModelConfig {
                name: model_name,
                path: model_path,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Mock provider for testing
    Mock,
    /// CoreML for Apple Silicon (legacy)
    CoreML,
    /// MLX for Apple Silicon (recommended)
    MLX,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name/identifier
    pub name: String,
    /// Path to model file (for local models)
    pub path: Option<PathBuf>,
    /// Model format
    pub format: ModelFormat,
    /// Quantization type
    pub quantization: Quantization,
    /// Context window size
    pub context_size: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "mock-model".to_string(),
            path: None,
            format: ModelFormat::CoreML,
            quantization: Quantization::Int8,
            context_size: 2048,
        }
    }
}

/// Model format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFormat {
    /// CoreML mlpackage
    CoreML,
    /// GGUF format
    GGUF,
    /// SafeTensors
    SafeTensors,
}

/// Quantization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quantization {
    /// No quantization (FP32)
    None,
    /// FP16
    Fp16,
    /// INT8 weights with FP16 activations
    Int8,
    /// 4-bit quantization
    Int4,
}

/// Generation parameter defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationDefaults {
    /// Default max tokens
    pub max_tokens: u32,
    /// Default temperature
    pub temperature: f32,
    /// Default top-p (nucleus sampling)
    pub top_p: f32,
    /// Default top-k
    pub top_k: u32,
    /// Default repetition penalty
    pub repetition_penalty: f32,
}

impl Default for GenerationDefaults {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.95,
            top_k: 40,
            repetition_penalty: 1.1,
        }
    }
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum inference timeout
    #[serde(with = "duration_serde")]
    pub inference_timeout: Duration,
    /// Maximum tokens per request
    pub max_tokens_per_request: u32,
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Maximum context tokens
    pub max_context_tokens: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            inference_timeout: Duration::from_secs(60),
            max_tokens_per_request: 4096,
            max_concurrent_requests: 4,
            max_context_tokens: 8192,
        }
    }
}

/// Serde helper for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = InferenceConfig::default();
        assert_eq!(config.provider, ProviderType::Mock);
        assert_eq!(config.generation.max_tokens, 256);
    }

    #[test]
    fn test_mock_config() {
        let config = InferenceConfig::mock();
        assert_eq!(config.provider, ProviderType::Mock);
    }

    #[test]
    fn test_coreml_config() {
        let config = InferenceConfig::coreml("/models/mistral.mlpackage");
        assert_eq!(config.provider, ProviderType::CoreML);
        assert!(config.model.path.is_some());
    }

    #[test]
    fn test_config_serialization() {
        let config = InferenceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: InferenceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider, config.provider);
    }

    #[test]
    fn test_mlx_config() {
        let config = InferenceConfig::mlx("/models/mistral");
        assert_eq!(config.provider, ProviderType::MLX);
        assert!(config.model.path.is_some());
        assert_eq!(config.model.format, ModelFormat::SafeTensors);
    }

    #[test]
    fn test_provider_type_serialization() {
        let mlx = ProviderType::MLX;
        let json = serde_json::to_string(&mlx).unwrap();
        assert_eq!(json, "\"mlx\"");

        let parsed: ProviderType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ProviderType::MLX);
    }
}
