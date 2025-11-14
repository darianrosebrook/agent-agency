//! Error types for research operations
//!
//! Uses stable error codes instead of stringly-typed errors.
//! Keep thiserror and any tracing deps OUT of contracts.

#[cfg(feature = "serde")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddingErrorCode {
    ProviderUnavailable,
    RateLimited,
    InvalidInput,
    Internal,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct RetryHint {
    pub retryable: bool,
    pub after_ms: Option<u64>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct EmbeddingError {
    pub code: EmbeddingErrorCode,
    pub message: String, // Human-readable
    pub transient: bool, // Retry hint
    pub hint: Option<RetryHint>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeErrorCode {
    NotFound,
    Failed,
    RateLimited,
    InvalidInput,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct KnowledgeError {
    pub code: KnowledgeErrorCode,
    pub message: String,
    pub transient: bool,
    pub hint: Option<RetryHint>,
}
