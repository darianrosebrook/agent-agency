//! Model abstraction layer for hot-swappable AI providers

pub mod ollama;
pub mod selection;

pub use ollama::OllamaProvider;
pub use selection::{ModelRegistry, ModelSelectionPolicy};

use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::types::{ModelResponse, ModelInfo, ModelCapabilities, HealthStatus, IterationContext};

/// Context for model generation
#[derive(Debug, Clone)]
pub struct ModelContext {
    pub task_history: Vec<IterationContext>,
    pub temperature: f64,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
}

/// Trait for AI model providers
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Generate a response from the model
    async fn generate(&self, prompt: &str, context: &ModelContext) -> Result<ModelResponse, ModelError>;

    /// Check if the model is healthy and available
    async fn health_check(&self) -> Result<HealthStatus, ModelError>;

    /// Get information about this model
    fn model_info(&self) -> ModelInfo;

    /// Get model capabilities
    fn capabilities(&self) -> ModelCapabilities;
}

/// Model performance statistics
#[derive(Debug, Clone)]
pub struct ModelPerformanceStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
    pub last_used: DateTime<Utc>,
}

/// Errors from model operations
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Model unavailable: {0}")]
    ModelUnavailable(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
