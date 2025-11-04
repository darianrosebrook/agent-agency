//! Inference manager for coordinating backend execution

use crate::types::*;
use crate::ModelManagementError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Trait for inference backends
#[async_trait]
pub trait InferenceBackend: Send + Sync + std::fmt::Debug {
    /// Execute inference request
    async fn execute(&self, request: InferenceInput) -> Result<InferenceOutput, ModelManagementError>;

    /// Check if backend supports the given model type
    fn supports_model(&self, model_type: &str) -> bool;

    /// Get backend name
    fn name(&self) -> &str;

    /// Get backend identifier
    fn id(&self) -> &str;

    /// Get backend capabilities
    fn capabilities(&self) -> BackendCapabilities;
}

/// Backend capabilities
#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    /// Supported models
    pub supported_models: Vec<String>,

    /// Maximum batch size
    pub max_batch_size: usize,

    /// Supports async execution
    pub supports_async: bool,

    /// Quantization support
    pub quantization_support: Vec<String>,
}

/// Inference manager for coordinating backends
#[derive(Debug)]
pub struct InferenceManager {
    /// Registered backends
    backends: Arc<RwLock<HashMap<String, Arc<dyn InferenceBackend>>>>,

    /// Backend selection cache
    backend_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl InferenceManager {
    /// Create a new inference manager
    pub fn new() -> Self {
        Self {
            backends: Arc::new(RwLock::new(HashMap::new())),
            backend_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an inference backend
    pub async fn register_backend(&self, backend: Arc<dyn InferenceBackend>) -> Result<(), ModelManagementError> {
        let backend_id = backend.id().to_string();

        let mut backends = self.backends.write().await;
        if backends.contains_key(&backend_id) {
            warn!("Backend {} already registered, replacing", backend_id);
        }

        backends.insert(backend_id.clone(), backend);
        info!("Registered inference backend: {}", backend_id);

        Ok(())
    }

    /// Get or create a backend for the given model type
    pub async fn get_or_create_backend(&self, model_type: &str) -> Result<Arc<dyn InferenceBackend>, ModelManagementError> {
        // Check cache first
        {
            let cache = self.backend_cache.read().await;
            if let Some(backend_id) = cache.get(model_type) {
                let backends = self.backends.read().await;
                if let Some(backend) = backends.get(backend_id) {
                    return Ok(backend.clone());
                }
            }
        }

        // Find suitable backend
        let backends = self.backends.read().await;
        for (backend_id, backend) in backends.iter() {
            if backend.supports_model(model_type) {
                // Update cache
                let mut cache = self.backend_cache.write().await;
                cache.insert(model_type.to_string(), backend_id.clone());

                debug!("Selected backend {} for model type {}", backend_id, model_type);
                return Ok(backend.clone());
            }
        }

        Err(ModelManagementError::InferenceError(
            format!("No backend available for model type: {}", model_type)
        ))
    }

    /// Execute inference using the appropriate backend
    pub async fn execute_inference(&self, request: &InferenceInput) -> Result<InferenceOutput, ModelManagementError> {
        let backend = self.get_or_create_backend(&request.model_id).await?;

        debug!("Executing inference for model {} using backend {}",
               request.model_id, backend.name());

        let start_time = std::time::Instant::now();
        let result = backend.execute(request.clone()).await;
        let execution_time = start_time.elapsed();

        match &result {
            Ok(output) => {
                debug!("Inference completed in {:?} for model {}",
                       execution_time, request.model_id);
                Ok(output.clone())
            }
            Err(e) => {
                warn!("Inference failed for model {}: {}", request.model_id, e);
                Err(ModelManagementError::InferenceError(e.to_string()))
            }
        }
    }

    /// Get all registered backends
    pub async fn list_backends(&self) -> Result<Vec<String>, ModelManagementError> {
        let backends = self.backends.read().await;
        Ok(backends.keys().cloned().collect())
    }

    /// Get backend capabilities
    pub async fn get_backend_capabilities(&self, backend_id: &str) -> Result<Option<BackendCapabilities>, ModelManagementError> {
        let backends = self.backends.read().await;
        match backends.get(backend_id) {
            Some(backend) => Ok(Some(backend.capabilities())),
            None => Ok(None),
        }
    }

    /// Remove a backend
    pub async fn remove_backend(&self, backend_id: &str) -> Result<(), ModelManagementError> {
        let mut backends = self.backends.write().await;
        if backends.remove(backend_id).is_some() {
            // Clear cache entries for this backend
            let mut cache = self.backend_cache.write().await;
            cache.retain(|_, id| id != backend_id);

            info!("Removed backend: {}", backend_id);
            Ok(())
        } else {
            Err(ModelManagementError::InferenceError(
                format!("Backend not found: {}", backend_id)
            ))
        }
    }
}

/// Simple CPU-based inference backend for testing
#[derive(Debug)]
pub struct CpuInferenceBackend {
    id: String,
    name: String,
    supported_models: Vec<String>,
}

impl CpuInferenceBackend {
    pub fn new(id: String, name: String, supported_models: Vec<String>) -> Self {
        Self {
            id,
            name,
            supported_models,
        }
    }
}

#[async_trait]
impl InferenceBackend for CpuInferenceBackend {
    async fn execute(&self, request: InferenceInput) -> Result<InferenceOutput, ModelManagementError> {
        // Simulate inference execution
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let metadata = InferenceMetadata {
            backend: self.name.clone(),
            model_version: "1.0.0".to_string(),
            executed_at: chrono::Utc::now(),
            tokens_processed: None,
        };

        let performance = InferencePerformance {
            total_latency_ms: 100,
            model_execution_ms: 80,
            preprocessing_ms: 10,
            postprocessing_ms: 10,
            memory_usage_mb: 50,
        };

        Ok(InferenceOutput {
            data: request.data,
            metadata,
            performance,
        })
    }

    fn supports_model(&self, model_type: &str) -> bool {
        self.supported_models.contains(&model_type.to_string())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supported_models: self.supported_models.clone(),
            max_batch_size: 1,
            supports_async: true,
            quantization_support: vec!["none".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_inference_manager() {
        let manager = InferenceManager::new();

        // Register a backend
        let backend = Arc::new(CpuInferenceBackend::new(
            "cpu-1".to_string(),
            "CPU Backend".to_string(),
            vec!["bert".to_string(), "gpt".to_string()],
        ));
        manager.register_backend(backend).await.unwrap();

        // Test backend selection
        let selected = manager.get_or_create_backend("bert").await.unwrap();
        assert_eq!(selected.name(), "CPU Backend");

        // Test inference execution
        let input = InferenceInput {
            model_id: "bert".to_string(),
            data: serde_json::json!({"text": "Hello world"}),
            parameters: InferenceParameters::default(),
        };

        let result = manager.execute_inference(&input).await.unwrap();
        assert_eq!(result.metadata.backend, "CPU Backend");
    }
}
