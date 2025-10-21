use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// API endpoint URL
    pub api_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Model name to use
    pub model: String,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature for generation (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum retries on failure
    pub max_retries: u32,
}

/// Message role in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Message in LLM conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// LLM generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
}

/// LLM generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Reason why generation finished
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    Error,
}

/// LLM client trait for abstraction over different providers
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Generate text using the LLM
    async fn generate(&self, request: &GenerationRequest) -> Result<GenerationResponse>;

    /// Check if the client is healthy and available
    async fn health_check(&self) -> Result<()>;

    /// Get the model name being used
    fn model_name(&self) -> &str;

    /// Get the provider name (OpenAI, Anthropic, etc.)
    fn provider_name(&self) -> &str;
}

/// OpenAI-compatible LLM client
pub struct OpenAIClient {
    config: LLMConfig,
    client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn generate(&self, request: &GenerationRequest) -> Result<GenerationResponse> {
        use serde_json::{json, Value};

        let messages: Vec<Value> = request.messages.iter().map(|msg| {
            json!({
                "role": match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                },
                "content": msg.content
            })
        }).collect();

        let payload = json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(self.config.max_tokens),
            "temperature": request.temperature.unwrap_or(self.config.temperature),
            "stop": request.stop_sequences,
        });

        let response = self.client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let data: Value = response.json().await?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format: missing content"))?
            .to_string();

        let usage = TokenUsage {
            prompt_tokens: data["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: data["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: data["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        let finish_reason = match data["choices"][0]["finish_reason"].as_str() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            _ => FinishReason::Error,
        };

        Ok(GenerationResponse {
            content,
            usage,
            finish_reason,
        })
    }

    async fn health_check(&self) -> Result<()> {
        // Simple health check by making a minimal request
        let request = GenerationRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: Some(1),
            temperature: Some(0.0),
            stop_sequences: None,
        };

        self.generate(&request).await?;
        Ok(())
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    fn provider_name(&self) -> &str {
        "OpenAI"
    }
}

/// Ollama-compatible LLM client for local models
pub struct OllamaClient {
    config: LLMConfig,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }
}

#[async_trait]
impl LLMClient for OllamaClient {
    async fn generate(&self, request: &GenerationRequest) -> Result<GenerationResponse> {
        use serde_json::{json, Value};

        // Ollama expects a different format
        let prompt = request.messages.iter()
            .map(|msg| format!("{}: {}", match msg.role {
                MessageRole::System => "System",
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
            }, msg.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let payload = json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": request.max_tokens.unwrap_or(self.config.max_tokens),
                "temperature": request.temperature.unwrap_or(self.config.temperature),
            }
        });

        let response = self.client
            .post(&self.config.api_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }

        let data: Value = response.json().await?;

        let content = data["response"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format: missing response"))?
            .to_string();

        // Ollama doesn't provide detailed token usage, so we estimate
        let estimated_tokens = (content.len() / 4) as u32; // Rough estimate

        let usage = TokenUsage {
            prompt_tokens: estimated_tokens / 2,
            completion_tokens: estimated_tokens / 2,
            total_tokens: estimated_tokens,
        };

        let finish_reason = if data["done"].as_bool().unwrap_or(false) {
            FinishReason::Stop
        } else {
            FinishReason::Error
        };

        Ok(GenerationResponse {
            content,
            usage,
            finish_reason,
        })
    }

    async fn health_check(&self) -> Result<()> {
        let response = self.client
            .get(&format!("{}/api/tags", self.config.api_url.trim_end_matches('/')))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Ollama service not available"));
        }

        Ok(())
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    fn provider_name(&self) -> &str {
        "Ollama"
    }
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;

