//! Inference types
//!
//! Core types for LLM inference requests and responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Request ID
    pub request_id: String,
    /// Prompt text
    pub prompt: String,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature (0.0-1.0)
    pub temperature: f32,
    /// Top-p (nucleus sampling)
    pub top_p: f32,
    /// Top-k sampling
    pub top_k: u32,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// Whether to stream output
    pub stream: bool,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

impl InferenceRequest {
    /// Create a new inference request
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            prompt: prompt.into(),
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.95,
            top_k: 40,
            stop_sequences: Vec::new(),
            stream: false,
            created_at: Utc::now(),
        }
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp.clamp(0.0, 2.0);
        self
    }

    /// Set streaming
    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Add stop sequence
    pub fn with_stop(mut self, stop: impl Into<String>) -> Self {
        self.stop_sequences.push(stop.into());
        self
    }

    /// Compute hash of the request (for caching/fingerprinting)
    pub fn content_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.prompt.as_bytes());
        hasher.update(self.max_tokens.to_le_bytes());
        hasher.update(self.temperature.to_le_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Request ID (matches request)
    pub request_id: String,
    /// Generated text
    pub text: String,
    /// Number of tokens generated
    pub tokens_generated: u32,
    /// Number of prompt tokens
    pub prompt_tokens: u32,
    /// Time to first token (milliseconds)
    pub time_to_first_token_ms: u64,
    /// Total generation time (milliseconds)
    pub total_time_ms: u64,
    /// Tokens per second
    pub tokens_per_second: f64,
    /// Model used
    pub model: String,
    /// Finish reason
    pub finish_reason: FinishReason,
    /// Response timestamp
    pub created_at: DateTime<Utc>,
}

impl InferenceResponse {
    /// Create a new response
    pub fn new(request_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            text: text.into(),
            tokens_generated: 0,
            prompt_tokens: 0,
            time_to_first_token_ms: 0,
            total_time_ms: 0,
            tokens_per_second: 0.0,
            model: String::new(),
            finish_reason: FinishReason::Complete,
            created_at: Utc::now(),
        }
    }

    /// Compute content hash
    pub fn content_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.request_id.as_bytes());
        hasher.update(self.text.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Reason generation stopped
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Completed naturally
    Complete,
    /// Hit max tokens
    MaxTokens,
    /// Hit stop sequence
    StopSequence,
    /// Timeout
    Timeout,
    /// Error occurred
    Error,
}

/// Streaming token event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEvent {
    /// Request ID
    pub request_id: String,
    /// Token text
    pub token: String,
    /// Token index
    pub index: u32,
    /// Is final token
    pub is_final: bool,
    /// Cumulative text so far
    pub text_so_far: String,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    /// Model version
    pub version: String,
    /// Parameters count
    pub parameters: u64,
    /// Context window size
    pub context_size: u32,
    /// Is model loaded
    pub is_loaded: bool,
    /// Memory usage (bytes)
    pub memory_bytes: u64,
}

/// Provider status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    /// Provider name
    pub name: String,
    /// Is available
    pub available: bool,
    /// Currently loaded models
    pub loaded_models: Vec<ModelInfo>,
    /// Concurrent request capacity
    pub capacity: u32,
    /// Current active requests
    pub active_requests: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let request = InferenceRequest::new("Hello, world!")
            .with_max_tokens(100)
            .with_temperature(0.5)
            .with_stop("\n");

        assert_eq!(request.prompt, "Hello, world!");
        assert_eq!(request.max_tokens, 100);
        assert!((request.temperature - 0.5).abs() < 0.001);
        assert_eq!(request.stop_sequences, vec!["\n"]);
    }

    #[test]
    fn test_temperature_clamping() {
        let request = InferenceRequest::new("test").with_temperature(5.0);
        assert!((request.temperature - 2.0).abs() < 0.001);

        let request = InferenceRequest::new("test").with_temperature(-1.0);
        assert!(request.temperature.abs() < 0.001);
    }

    #[test]
    fn test_content_hash() {
        let request = InferenceRequest::new("test");
        let hash = request.content_hash();
        assert_eq!(hash.len(), 64); // SHA256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_response_creation() {
        let response = InferenceResponse::new("req-123", "Generated text");
        assert_eq!(response.request_id, "req-123");
        assert_eq!(response.text, "Generated text");
        assert_eq!(response.finish_reason, FinishReason::Complete);
    }

    #[test]
    fn test_finish_reason_serialization() {
        let reason = FinishReason::MaxTokens;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"max_tokens\"");
    }
}
