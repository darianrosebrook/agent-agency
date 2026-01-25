//! Inference service
//!
//! High-level service that wraps providers and adds metrics, caching, and retries.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::InferenceConfig;
use crate::error::InferenceError;
use crate::provider::{InferenceProvider, ProviderFactory};
use crate::types::{InferenceRequest, InferenceResponse, ModelInfo, ProviderStatus};

/// Inference service
///
/// Provides a high-level interface for LLM inference with:
/// - Provider abstraction
/// - Request tracking
/// - Basic metrics
pub struct InferenceService {
    provider: Arc<RwLock<Box<dyn InferenceProvider>>>,
    config: InferenceConfig,
    metrics: Arc<RwLock<InferenceMetrics>>,
}

impl InferenceService {
    /// Create a new inference service
    pub fn new(config: InferenceConfig) -> Self {
        let provider = ProviderFactory::create(&config);
        Self {
            provider: Arc::new(RwLock::new(provider)),
            config,
            metrics: Arc::new(RwLock::new(InferenceMetrics::default())),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &InferenceConfig {
        &self.config
    }

    /// Load the model
    pub async fn load_model(&self) -> Result<ModelInfo, InferenceError> {
        let provider = self.provider.read().await;
        provider.load_model().await
    }

    /// Unload the model
    pub async fn unload_model(&self) -> Result<(), InferenceError> {
        let provider = self.provider.read().await;
        provider.unload_model().await
    }

    /// Check if model is loaded
    pub async fn is_model_loaded(&self) -> bool {
        let provider = self.provider.read().await;
        provider.is_model_loaded()
    }

    /// Run inference
    pub async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        let start = std::time::Instant::now();

        // Check timeout
        let timeout = self.config.limits.inference_timeout;
        let provider = self.provider.read().await;

        let result = tokio::time::timeout(timeout, provider.infer(request)).await;

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        match result {
            Ok(Ok(response)) => {
                metrics.successful_requests += 1;
                metrics.total_tokens_generated += response.tokens_generated as u64;
                metrics.total_latency_ms += start.elapsed().as_millis() as u64;
                Ok(response)
            }
            Ok(Err(e)) => {
                metrics.failed_requests += 1;
                Err(e)
            }
            Err(_) => {
                metrics.failed_requests += 1;
                Err(InferenceError::Timeout(timeout.as_millis() as u64))
            }
        }
    }

    /// Get provider status
    pub async fn status(&self) -> ProviderStatus {
        let provider = self.provider.read().await;
        provider.status().await
    }

    /// Get service metrics
    pub async fn metrics(&self) -> InferenceMetrics {
        self.metrics.read().await.clone()
    }

    /// Health check
    pub async fn health_check(&self) -> Result<(), InferenceError> {
        let provider = self.provider.read().await;
        provider.health_check().await
    }
}

/// Inference metrics
#[derive(Debug, Clone, Default)]
pub struct InferenceMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total tokens generated
    pub total_tokens_generated: u64,
    /// Total latency (ms)
    pub total_latency_ms: u64,
}

impl InferenceMetrics {
    /// Calculate average latency
    pub fn avg_latency_ms(&self) -> f64 {
        if self.successful_requests > 0 {
            self.total_latency_ms as f64 / self.successful_requests as f64
        } else {
            0.0
        }
    }

    /// Calculate average tokens per request
    pub fn avg_tokens_per_request(&self) -> f64 {
        if self.successful_requests > 0 {
            self.total_tokens_generated as f64 / self.successful_requests as f64
        } else {
            0.0
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.successful_requests as f64 / self.total_requests as f64
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_lifecycle() {
        let service = InferenceService::new(InferenceConfig::mock());

        assert!(!service.is_model_loaded().await);

        service.load_model().await.unwrap();
        assert!(service.is_model_loaded().await);

        service.unload_model().await.unwrap();
        assert!(!service.is_model_loaded().await);
    }

    #[tokio::test]
    async fn test_service_inference() {
        let service = InferenceService::new(InferenceConfig::mock());
        service.load_model().await.unwrap();

        let request = InferenceRequest::new("Hello, world!");
        let response = service.infer(request).await.unwrap();

        assert!(!response.text.is_empty());

        let metrics = service.metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_service_metrics() {
        let service = InferenceService::new(InferenceConfig::mock());
        service.load_model().await.unwrap();

        // Make several requests
        for i in 0..5 {
            let request = InferenceRequest::new(format!("Request {}", i));
            service.infer(request).await.unwrap();
        }

        let metrics = service.metrics().await;
        assert_eq!(metrics.total_requests, 5);
        assert_eq!(metrics.successful_requests, 5);
        assert!(metrics.avg_latency_ms() > 0.0);
    }

    #[test]
    fn test_metrics_calculations() {
        let metrics = InferenceMetrics {
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            total_tokens_generated: 9500,
            total_latency_ms: 95000,
        };

        assert!((metrics.success_rate() - 0.95).abs() < 0.001);
        assert!((metrics.avg_tokens_per_request() - 100.0).abs() < 0.001);
        assert!((metrics.avg_latency_ms() - 1000.0).abs() < 0.001);
    }
}
