//! Model providers and registry for the self-prompting agent
//!
//! Supports multiple model providers (Ollama, CoreML, etc.) with unified interface.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::types::SelfPromptingAgentError;

/// Unified model provider trait
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Generate text from a prompt
    async fn generate(&self, prompt: &str, options: &GenerationOptions) -> Result<String, SelfPromptingAgentError>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Check if provider is available
    async fn is_available(&self) -> bool { true }
}

/// Generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub stop_sequences: Vec<String>,
    pub model_name: Option<String>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(2048),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stop_sequences: vec![],
            model_name: None,
        }
    }
}

/// Model registry managing multiple providers
pub struct ModelRegistry {
    providers: HashMap<String, Arc<dyn ModelProvider>>,
    default_provider: Option<String>,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    /// Register a model provider
    pub fn register_provider(&mut self, name: String, provider: Arc<dyn ModelProvider>) {
        self.providers.insert(name.clone(), provider);
        if self.default_provider.is_none() {
            self.default_provider = Some(name);
        }
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn ModelProvider>> {
        self.providers.get(name).cloned()
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> Option<Arc<dyn ModelProvider>> {
        self.default_provider.as_ref()
            .and_then(|name| self.providers.get(name).cloned())
    }

    /// List available providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Generate text using the default provider
    pub async fn generate(&self, prompt: &str, options: &GenerationOptions) -> Result<String, SelfPromptingAgentError> {
        let provider = self.get_default_provider()
            .ok_or_else(|| SelfPromptingAgentError::ModelProvider("No default provider configured".to_string()))?;

        provider.generate(prompt, options).await
    }

    /// Generate text using a specific provider
    pub async fn generate_with_provider(&self, provider_name: &str, prompt: &str, options: &GenerationOptions) -> Result<String, SelfPromptingAgentError> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| SelfPromptingAgentError::ModelProvider(format!("Provider '{}' not found", provider_name)))?;

        provider.generate(prompt, options).await
    }
}

/// Ollama model provider
pub struct OllamaProvider {
    base_url: String,
    default_model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(base_url: String, default_model: String) -> Self {
        Self {
            base_url,
            default_model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    async fn generate(&self, prompt: &str, options: &GenerationOptions) -> Result<String, SelfPromptingAgentError> {
        let model_name = options.model_name.as_ref()
            .unwrap_or(&self.default_model);

        let request_body = serde_json::json!({
            "model": model_name,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": options.temperature.unwrap_or(0.7),
                "top_p": options.top_p.unwrap_or(0.9),
                "num_predict": options.max_tokens.unwrap_or(2048),
                "stop": options.stop_sequences,
            }
        });

        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SelfPromptingAgentError::ModelProvider(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SelfPromptingAgentError::ModelProvider(format!("HTTP error: {}", response.status())));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| SelfPromptingAgentError::ModelProvider(format!("Failed to parse response: {}", e)))?;

        response_json["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| SelfPromptingAgentError::ModelProvider("Invalid response format".to_string()))
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.client.get(&url).send().await
            .map(|resp| resp.status().is_success())
            .unwrap_or(false)
    }
}

/// Expert selection router for choosing the best model for a task
pub struct ExpertSelectionRouter {
    registry: Arc<ModelRegistry>,
    // Expert selection logic would go here
}

impl ExpertSelectionRouter {
    pub fn new(registry: Arc<ModelRegistry>) -> Self {
        Self { registry }
    }

    pub async fn select_expert(&self, task_description: &str) -> Result<String, SelfPromptingAgentError> {
        // Stub implementation - would analyze task and select best model
        // For now, just return the first available provider
        let providers = self.registry.list_providers();
        providers.into_iter().next()
            .ok_or_else(|| SelfPromptingAgentError::ModelProvider("No providers available".to_string()))
    }
}

/// Consensus builder for combining multiple model outputs
pub struct ConsensusBuilder {
    registry: Arc<ModelRegistry>,
    num_models: usize,
}

impl ConsensusBuilder {
    pub fn new(registry: Arc<ModelRegistry>, num_models: usize) -> Self {
        Self { registry, num_models }
    }

    pub async fn build_consensus(&self, prompt: &str, options: &GenerationOptions) -> Result<String, SelfPromptingAgentError> {
        // Stub implementation - would generate with multiple models and combine results
        self.registry.generate(prompt, options).await
    }
}

/// Shadow router for A/B testing and model comparison
pub struct ShadowRouter {
    registry: Arc<ModelRegistry>,
    shadow_provider: String,
    shadow_percentage: f64,
}

impl ShadowRouter {
    pub fn new(registry: Arc<ModelRegistry>, shadow_provider: String, shadow_percentage: f64) -> Self {
        Self {
            registry,
            shadow_provider,
            shadow_percentage,
        }
    }

    pub async fn route_with_shadow(&self, prompt: &str, options: &GenerationOptions) -> Result<String, SelfPromptingAgentError> {
        // Stub implementation - would route some traffic to shadow model
        self.registry.generate(prompt, options).await
    }
}

/// Offline evaluator for model performance analysis
pub struct OfflineEvaluator {
    // Would store evaluation data and metrics
}

impl OfflineEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn evaluate_model(&self, model_name: &str, test_cases: Vec<(String, String)>) -> Result<f64, SelfPromptingAgentError> {
        // Stub implementation - would run evaluation on test cases
        Ok(0.85) // Mock score
    }
}
