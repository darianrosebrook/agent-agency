//! Ollama model provider for local AI models

use std::collections::HashMap;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use super::{ModelProvider, ModelContext, ModelResponse, ModelInfo, ModelCapabilities, HealthStatus, ModelError};
use crate::types::IterationContext;

/// Ollama provider configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_name: String,
    pub timeout_seconds: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model_name: "gemma:3n".to_string(),
            timeout_seconds: 300,
        }
    }
}

/// Ollama API request
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

/// Ollama API response
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    #[serde(rename = "created_at")]
    created_at: String,
    response: String,
    done: bool,
    #[serde(rename = "total_duration")]
    total_duration: Option<u64>,
    #[serde(rename = "load_duration")]
    load_duration: Option<u64>,
    #[serde(rename = "prompt_eval_count")]
    prompt_eval_count: Option<u32>,
    #[serde(rename = "eval_count")]
    eval_count: Option<u32>,
    #[serde(rename = "eval_duration")]
    eval_duration: Option<u64>,
}

/// Ollama options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f64,
    #[serde(rename = "num_predict")]
    num_predict: usize,
    stop: Vec<String>,
}

/// Ollama model provider
pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
    model_info: ModelInfo,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(model_name: &str) -> Result<Self, ModelError> {
        Self::with_config(OllamaConfig {
            model_name: model_name.to_string(),
            ..Default::default()
        })
    }

    /// Create a new Ollama provider with custom config
    pub fn with_config(config: OllamaConfig) -> Result<Self, ModelError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(ModelError::ConfigError)?;

        let model_info = ModelInfo {
            id: format!("ollama:{}", config.model_name),
            name: config.model_name.clone(),
            provider: "ollama".to_string(),
            capabilities: ModelCapabilities {
                max_context: 8192, // Conservative estimate, varies by model
                supports_streaming: true,
                supports_function_calling: false, // Ollama doesn't support function calling yet
                supports_vision: false, // Basic Ollama doesn't support vision
            },
        };

        Ok(Self {
            client,
            config,
            model_info,
        })
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
impl ModelProvider for OllamaProvider {
    async fn generate(&self, prompt: &str, context: &ModelContext) -> Result<ModelResponse, ModelError> {
        let full_prompt = self.build_prompt(prompt, context);

        let request = OllamaRequest {
            model: self.config.model_name.clone(),
            prompt: full_prompt,
            stream: false,
            options: OllamaOptions {
                temperature: context.temperature,
                num_predict: context.max_tokens,
                stop: context.stop_sequences.clone(),
            },
        };

        let start_time = std::time::Instant::now();

        let response = self.client
            .post(&format!("{}/api/generate", self.config.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModelError::ModelUnavailable(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Calculate tokens used (approximate)
        let tokens_used = ollama_response.prompt_eval_count.unwrap_or(0) as usize
            + ollama_response.eval_count.unwrap_or(0) as usize;

        Ok(ModelResponse {
            text: ollama_response.response,
            model_id: self.model_info.id.clone(),
            tokens_used,
            latency_ms,
            finish_reason: Some("completed".to_string()),
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, ModelError> {
        // Try to get model info to check if Ollama is running and model is available
        let response = self.client
            .get(&format!("{}/api/tags", self.config.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(HealthStatus {
                healthy: false,
                last_check: Utc::now(),
                error_message: Some(format!("HTTP {}", response.status())),
            });
        }

        // Parse response to check if our model is available
        let tags_response: serde_json::Value = response.json().await
            .map_err(|e| ModelError::InvalidResponse(e.to_string()))?;

        let models = tags_response["models"]
            .as_array()
            .ok_or_else(|| ModelError::InvalidResponse("Invalid models response".to_string()))?;

        let model_available = models.iter().any(|model| {
            model["name"].as_str() == Some(&self.config.model_name)
        });

        Ok(HealthStatus {
            healthy: model_available,
            last_check: Utc::now(),
            error_message: if model_available {
                None
            } else {
                Some(format!("Model '{}' not available", self.config.model_name))
            },
        })
    }

    fn model_info(&self) -> ModelInfo {
        self.model_info.clone()
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.model_info.capabilities.clone()
    }
}
