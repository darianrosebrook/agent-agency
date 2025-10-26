//! Inference backend implementations

use crate::inference::manager::*;
use crate::types::*;
use crate::ModelManagementError;
use async_trait::async_trait;

/// Mock backend for testing
#[derive(Debug)]
pub struct MockInferenceBackend {
    id: String,
    name: String,
    supported_models: Vec<String>,
    latency_ms: u64,
}

impl MockInferenceBackend {
    pub fn new(id: String, name: String, supported_models: Vec<String>, latency_ms: u64) -> Self {
        Self {
            id,
            name,
            supported_models,
            latency_ms,
        }
    }
}

#[async_trait]
impl InferenceBackend for MockInferenceBackend {
    async fn execute(&self, request: InferenceInput) -> Result<InferenceOutput, ModelManagementError> {
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(self.latency_ms)).await;

        // Mock response based on input
        let output_data = match request.data.get("text") {
            Some(text) => serde_json::json!({
                "processed_text": format!("MOCK: {}", text),
                "confidence": 0.95
            }),
            _ => serde_json::json!({"result": "mock_processed"}),
        };

        let metadata = InferenceMetadata {
            backend: self.name.clone(),
            model_version: "1.0.0".to_string(),
            executed_at: chrono::Utc::now(),
            tokens_processed: Some(50),
        };

        let performance = InferencePerformance {
            total_latency_ms: self.latency_ms,
            model_execution_ms: self.latency_ms * 8 / 10,
            preprocessing_ms: self.latency_ms / 10,
            postprocessing_ms: self.latency_ms / 10,
            memory_usage_mb: 100,
        };

        Ok(InferenceOutput {
            data: output_data,
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
            max_batch_size: 32,
            supports_async: true,
            quantization_support: vec!["none".to_string(), "int8".to_string()],
        }
    }
}

/// HTTP-based inference backend for remote models
#[derive(Debug)]
pub struct HttpInferenceBackend {
    id: String,
    name: String,
    supported_models: Vec<String>,
    endpoint: String,
    client: reqwest::Client,
}

impl HttpInferenceBackend {
    pub fn new(id: String, name: String, supported_models: Vec<String>, endpoint: String) -> Self {
        Self {
            id,
            name,
            supported_models,
            endpoint,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl InferenceBackend for HttpInferenceBackend {
    async fn execute(&self, request: InferenceInput) -> Result<InferenceOutput, ModelManagementError> {
        let start_time = std::time::Instant::now();

        // Prepare request payload
        let payload = serde_json::json!({
            "model": request.model_id,
            "data": request.data,
            "parameters": request.parameters
        });

        // Make HTTP request
        let response = self.client
            .post(&self.endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ModelManagementError::Http(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ModelManagementError::Http(format!(
                "HTTP {}: {}", status, error_text
            )));
        }

        // Parse response
        let response_data: serde_json::Value = response.json().await
            .map_err(|e| ModelManagementError::Http(format!("Failed to parse JSON response: {}", e)))?;

        let execution_time = start_time.elapsed();

        let metadata = InferenceMetadata {
            backend: self.name.clone(),
            model_version: "remote".to_string(),
            executed_at: chrono::Utc::now(),
            tokens_processed: response_data.get("tokens_used")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
        };

        let performance = InferencePerformance {
            total_latency_ms: execution_time.as_millis() as u64,
            model_execution_ms: execution_time.as_millis() as u64 * 9 / 10,
            preprocessing_ms: execution_time.as_millis() as u64 / 20,
            postprocessing_ms: execution_time.as_millis() as u64 / 20,
            memory_usage_mb: 0, // Unknown for remote backend
        };

        Ok(InferenceOutput {
            data: response_data,
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
            max_batch_size: 1, // HTTP backend typically doesn't support batching
            supports_async: true,
            quantization_support: vec!["unknown".to_string()],
        }
    }
}
