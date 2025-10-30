//! Engine contracts for judge inference backends
//!
//! This module defines the stable, platform-agnostic interfaces that
//! different inference engines (CoreML, GGML, Remote APIs) must implement.
//!
//! The contracts enable:
//! - Hexagonal architecture (council depends only on contracts)
//! - Testability (engines can be mocked via traits)
//! - Interchangeability (different engines for different platforms)
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::judge_io::{JudgePrompt, JudgeVerdict, JudgeType, RubricItem, WorkingSpecEvidence, VerdictLabel, Violation, Severity};

/// Core trait for judge inference engines
/// Platform-agnostic interface for LLM inference backends
#[async_trait::async_trait]
pub trait JudgeEngine: Send + Sync + std::fmt::Debug {
    /// Complete a judge prompt and return structured verdict
    async fn complete(&self, req: EngineRequest) -> Result<EngineResponse, EngineError>;

    /// Get engine capabilities and metadata
    fn capabilities(&self) -> EngineCaps;
}

/// Request for judge inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRequest {
    /// The judge prompt with rubric and evidence
    pub prompt: JudgePrompt,

    /// Maximum tokens to generate in response
    pub max_tokens: usize,

    /// Sampling temperature (0.0 = deterministic, 1.0 = creative)
    pub temperature: f64,

    /// Optional random seed for reproducible results
    pub seed: Option<u64>,
}

/// Response from judge inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResponse {
    /// Raw text output from the model
    pub raw_text: String,

    /// Parsed and validated judge verdict
    pub parsed: JudgeVerdict,

    /// Token usage statistics
    pub usage: TokenUsage,
}


/// Engine capabilities and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCaps {
    /// Model identifier (e.g., "mistral-7b")
    pub model_id: String,

    /// Model family (e.g., "mistral", "llama", "gpt")
    pub family: String,

    /// Maximum context length in tokens
    pub max_ctx: usize,

    /// Maximum tokens to generate in response
    pub max_tokens_out: usize,

    /// Quantization level (e.g., "int4", "fp16", "fp32")
    pub quant: String,

    /// Acceleration technologies available
    pub acceleration: Vec<String>, // ["ANE", "GPU", "CPU"]
}

/// Token usage statistics from inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,

    /// Tokens generated in response
    pub completion_tokens: u32,

    /// Total tokens processed
    pub total_tokens: u32,
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

/// Engine-specific errors
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Model not available: {model_id}")]
    ModelNotAvailable { model_id: String },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Inference failed: {message}")]
    InferenceFailed { message: String },

    #[error("Response parsing failed: {message}")]
    ParseError { message: String },

    #[error("JSON validation failed: {message}")]
    ValidationError { message: String },

    #[error("Timeout exceeded: {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal engine error: {message}")]
    Internal { message: String },
}

impl TokenUsage {
    /// Create token usage from raw text (approximate estimation)
    pub fn from_text(text: &str) -> Self {
        // Rough estimation: ~4 characters per token
        let estimated_tokens = (text.len() / 4) as u32;
        Self {
            prompt_tokens: 0, // Not available from text alone
            completion_tokens: estimated_tokens,
            total_tokens: estimated_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::judge_io::{JudgeVerdict, VerdictLabel, Violation, Severity};

    #[test]
    fn test_engine_request_serialization() {
        let req = EngineRequest {
            prompt: JudgePrompt {
                role: JudgeType::Constitutional,
                objective: "Test ethical compliance".to_string(),
                rubric: vec![],
                evidence: WorkingSpecEvidence {
                    spec_text: "test".to_string(),
                    acceptance_criteria: vec![],
                    risk_tier: "low".to_string(),
                    context: HashMap::new(),
                },
                output_schema: "{}".to_string(),
            },
            max_tokens: 128,
            temperature: 0.1,
            seed: Some(42),
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: EngineRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_tokens, req.max_tokens);
        assert_eq!(deserialized.temperature, req.temperature);
    }

    #[test]
    fn test_engine_caps_validation() {
        let caps = EngineCaps {
            model_id: "mistral-7b-instruct".to_string(),
            family: "mistral".to_string(),
            max_ctx: 4096,
            max_tokens_out: 1024,
            quant: "int4".to_string(),
            acceleration: vec!["ANE".to_string(), "GPU".to_string()],
        };

        assert_eq!(caps.model_id, "mistral-7b-instruct");
        assert!(caps.acceleration.contains(&"ANE".to_string()));
        assert!(caps.max_ctx > 0);
    }
}
