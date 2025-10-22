use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
    /// Uniquely identify this call; enables counterfactual replay
    pub request_id: Uuid,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,

    // NEW: commonly supported knobs (leave as Option to keep BC)
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub seed: Option<u64>,
    pub model_name: Option<String>,

    // NEW: analysis/stratification
    /// Lightweight hash of the normalized prompt for grouping
    pub prompt_hash: Option<u64>,
    /// Schema version to manage migrations of UsedParameters/telemetry
    pub schema_version: Option<u16>,
}

/// LLM generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub request_id: Uuid,              // NEW: Match request
    pub content: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub parameters_used: UsedParameters,
}

/// Parameters actually used during generation (for learning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsedParameters {
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: Option<f32>,            // NEW
    pub frequency_penalty: Option<f32>, // NEW
    pub presence_penalty: Option<f32>,  // NEW
    pub stop_sequences: Vec<String>,
    pub seed: Option<u64>,             // NEW
    pub schema_version: u16,           // NEW: Migration tracking
    pub origin: String,                // NEW: "bandit:thompson@v0.3.1"
    pub policy_version: String,        // NEW: Semver of learner
    pub timestamp: DateTime<Utc>,
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

        let mut payload = json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(self.config.max_tokens),
            "temperature": request.temperature.unwrap_or(self.config.temperature),
            "stop": request.stop_sequences,
        });

        // Add optional parameters if present
        if let Some(top_p) = request.top_p {
            payload["top_p"] = json!(top_p);
        }
        if let Some(frequency_penalty) = request.frequency_penalty {
            payload["frequency_penalty"] = json!(frequency_penalty);
        }
        if let Some(presence_penalty) = request.presence_penalty {
            payload["presence_penalty"] = json!(presence_penalty);
        }
        if let Some(seed) = request.seed {
            payload["seed"] = json!(seed);
        }

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

        // Create UsedParameters from the request
        let parameters_used = UsedParameters {
            model_name: self.config.model.clone(),
            temperature: request.temperature.unwrap_or(self.config.temperature),
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens),
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop_sequences: request.stop_sequences.unwrap_or_default(),
            seed: request.seed,
            schema_version: request.schema_version.unwrap_or(1),
            origin: "openai_client".to_string(),
            policy_version: "1.0.0".to_string(),
            timestamp: Utc::now(),
        };

        Ok(GenerationResponse {
            request_id: request.request_id,
            content,
            usage,
            finish_reason,
            parameters_used,
        })
    }

    async fn health_check(&self) -> Result<()> {
        // Simple health check by making a minimal request
        let request = GenerationRequest {
            request_id: Uuid::new_v4(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: Some(1),
            temperature: Some(0.0),
            stop_sequences: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            seed: None,
            model_name: None,
            prompt_hash: None,
            schema_version: None,
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

        let mut options = json!({
            "num_predict": request.max_tokens.unwrap_or(self.config.max_tokens),
            "temperature": request.temperature.unwrap_or(self.config.temperature),
        });

        // Add optional parameters if present
        if let Some(top_p) = request.top_p {
            options["top_p"] = json!(top_p);
        }
        if let Some(seed) = request.seed {
            options["seed"] = json!(seed as i64);
        }

        let payload = json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": options
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

        // Create UsedParameters from the request
        let parameters_used = UsedParameters {
            model_name: self.config.model.clone(),
            temperature: request.temperature.unwrap_or(self.config.temperature),
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens),
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop_sequences: request.stop_sequences.unwrap_or_default(),
            seed: request.seed,
            schema_version: request.schema_version.unwrap_or(1),
            origin: "ollama_client".to_string(),
            policy_version: "1.0.0".to_string(),
            timestamp: Utc::now(),
        };

        Ok(GenerationResponse {
            request_id: request.request_id,
            content,
            usage,
            finish_reason,
            parameters_used,
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
