//! Mock inference provider
//!
//! A mock provider for development and testing that simulates LLM behavior
//! without requiring actual model weights.

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;

use crate::config::InferenceConfig;
use crate::error::InferenceError;
use crate::provider::InferenceProvider;
use crate::types::{FinishReason, InferenceRequest, InferenceResponse, ModelInfo, ProviderStatus};

/// Mock inference provider for development and testing
pub struct MockProvider {
    config: InferenceConfig,
    model_loaded: AtomicBool,
    model_info: RwLock<Option<ModelInfo>>,
    request_count: AtomicU32,
}

impl MockProvider {
    /// Create a new mock provider
    pub fn new(config: InferenceConfig) -> Self {
        Self {
            config,
            model_loaded: AtomicBool::new(false),
            model_info: RwLock::new(None),
            request_count: AtomicU32::new(0),
        }
    }

    /// Generate mock response text based on the prompt
    fn generate_response(&self, prompt: &str, max_tokens: u32) -> String {
        // Simple mock generation based on prompt content
        let base_response = if prompt.contains("code") || prompt.contains("function") {
            "Here's a code snippet that addresses your request:\n\n```rust\nfn example() -> Result<(), Error> {\n    // Implementation\n    Ok(())\n}\n```"
        } else if prompt.contains("explain") {
            "Let me explain this concept:\n\nThe key points to understand are:\n1. First principle\n2. Second principle\n3. Practical application"
        } else if prompt.contains("list") {
            "Here's a list:\n- Item one\n- Item two\n- Item three"
        } else if prompt.contains("?") {
            "Based on my analysis, the answer involves several considerations. The primary factor is the context of your question, which suggests a nuanced approach is needed."
        } else {
            "I understand your request. Here's my response based on the provided context and requirements. This mock response simulates what a real LLM would generate."
        };

        // Truncate to approximate token count (rough: 4 chars per token)
        let max_chars = (max_tokens * 4) as usize;
        if base_response.len() > max_chars {
            base_response[..max_chars].to_string()
        } else {
            base_response.to_string()
        }
    }

    /// Estimate token count (rough approximation)
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Rough estimate: ~4 characters per token
        (text.len() / 4).max(1) as u32
    }
}

#[async_trait]
impl InferenceProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn load_model(&self) -> Result<ModelInfo, InferenceError> {
        tracing::info!(model = %self.config.model.name, "Loading mock model");

        let info = ModelInfo {
            name: self.config.model.name.clone(),
            version: "1.0.0-mock".to_string(),
            parameters: 7_000_000_000, // 7B params (mock)
            context_size: self.config.model.context_size,
            is_loaded: true,
            memory_bytes: 0, // Mock uses no memory
        };

        *self.model_info.write().unwrap() = Some(info.clone());
        self.model_loaded.store(true, Ordering::SeqCst);

        tracing::info!("Mock model loaded successfully");
        Ok(info)
    }

    async fn unload_model(&self) -> Result<(), InferenceError> {
        tracing::info!("Unloading mock model");
        *self.model_info.write().unwrap() = None;
        self.model_loaded.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_model_loaded(&self) -> bool {
        self.model_loaded.load(Ordering::SeqCst)
    }

    fn model_info(&self) -> Option<&ModelInfo> {
        // Can't return reference to RwLock content safely
        None
    }

    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        if !self.is_model_loaded() {
            return Err(InferenceError::ModelNotLoaded(
                "Model not loaded".to_string(),
            ));
        }

        self.request_count.fetch_add(1, Ordering::SeqCst);

        let start = std::time::Instant::now();

        // Simulate some processing time (5-50ms)
        let delay_ms = 5 + (request.prompt.len() % 45) as u64;
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

        let text = self.generate_response(&request.prompt, request.max_tokens);
        let tokens_generated = self.estimate_tokens(&text);
        let prompt_tokens = self.estimate_tokens(&request.prompt);

        let total_time_ms = start.elapsed().as_millis() as u64;
        let tokens_per_second = if total_time_ms > 0 {
            (tokens_generated as f64 / total_time_ms as f64) * 1000.0
        } else {
            0.0
        };

        let finish_reason = if tokens_generated >= request.max_tokens {
            FinishReason::MaxTokens
        } else {
            FinishReason::Complete
        };

        Ok(InferenceResponse {
            request_id: request.request_id,
            text,
            tokens_generated,
            prompt_tokens,
            time_to_first_token_ms: delay_ms / 2, // Mock: half of total
            total_time_ms,
            tokens_per_second,
            model: self.config.model.name.clone(),
            finish_reason,
            created_at: chrono::Utc::now(),
        })
    }

    async fn status(&self) -> ProviderStatus {
        let loaded_models = if self.is_model_loaded() {
            vec![ModelInfo {
                name: self.config.model.name.clone(),
                version: "1.0.0-mock".to_string(),
                parameters: 7_000_000_000,
                context_size: self.config.model.context_size,
                is_loaded: true,
                memory_bytes: 0,
            }]
        } else {
            vec![]
        };

        ProviderStatus {
            name: "mock".to_string(),
            available: true,
            loaded_models,
            capacity: self.config.limits.max_concurrent_requests,
            active_requests: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_lifecycle() {
        let config = InferenceConfig::mock();
        let provider = MockProvider::new(config);

        assert!(!provider.is_model_loaded());

        let info = provider.load_model().await.unwrap();
        assert!(provider.is_model_loaded());
        assert!(info.is_loaded);

        provider.unload_model().await.unwrap();
        assert!(!provider.is_model_loaded());
    }

    #[tokio::test]
    async fn test_mock_inference() {
        let config = InferenceConfig::mock();
        let provider = MockProvider::new(config);

        provider.load_model().await.unwrap();

        let request = InferenceRequest::new("Explain the concept of recursion")
            .with_max_tokens(100)
            .with_temperature(0.7);

        let response = provider.infer(request).await.unwrap();

        assert!(!response.text.is_empty());
        assert!(response.tokens_generated > 0);
        assert!(response.total_time_ms > 0);
    }

    #[tokio::test]
    async fn test_mock_requires_loaded_model() {
        let config = InferenceConfig::mock();
        let provider = MockProvider::new(config);

        let request = InferenceRequest::new("test");
        let result = provider.infer(request).await;

        assert!(matches!(result, Err(InferenceError::ModelNotLoaded(_))));
    }

    #[tokio::test]
    async fn test_mock_code_generation() {
        let config = InferenceConfig::mock();
        let provider = MockProvider::new(config);
        provider.load_model().await.unwrap();

        let request = InferenceRequest::new("Write a function to sort a list");
        let response = provider.infer(request).await.unwrap();

        assert!(response.text.contains("```"));
    }

    #[tokio::test]
    async fn test_mock_status() {
        let config = InferenceConfig::mock();
        let provider = MockProvider::new(config);

        let status = provider.status().await;
        assert!(status.available);
        assert!(status.loaded_models.is_empty());

        provider.load_model().await.unwrap();
        let status = provider.status().await;
        assert_eq!(status.loaded_models.len(), 1);
    }
}
