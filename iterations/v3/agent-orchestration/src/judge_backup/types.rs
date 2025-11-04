//! Judge types and configurations
//!
//! Core types for judge configuration, health metrics, and review contexts.

use schemars::JsonSchema;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::judge_backup::backup_types::JudgeType;
use crate::judge_backup::verdicts::JudgeVerdict;

/// Configuration for a judge instance

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeConfig {
    /// Judge identifier
    pub judge_id: String,
    /// Judge name
    pub name: String,
    /// Judge specialization type
    pub judge_type: JudgeType,
    /// Judge specialization area
    pub specialization: String,
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    /// Health check interval in milliseconds
    pub health_check_interval_ms: u64,
}

/// Health metrics for a judge

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeHealthMetrics {
    pub is_healthy: bool,
    pub response_time_p95_ms: u64,
    pub error_rate: f64,
    #[schemars(with = "String")]

    pub last_health_check: DateTime<Utc>,
}

/// Context for a review session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReviewContext {
    /// Session identifier
    pub session_id: String,
    /// Working specification content
    pub working_spec: String,
    /// Risk tier for the review
    pub risk_tier: u8,
    /// Previous reviews for context
    pub previous_reviews: Vec<PreviousReview>,
    /// Constraints for the review
    pub constraints: HashMap<String, String>,
}

/// Previous review information

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PreviousReview {
    pub judge_name: String,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub verdict_summary: String,
}

/// Judge contribution to a review

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeContribution {
    /// Judge identifier
    pub judge_id: String,
    /// Judge name for display
    pub judge_name: String,
    /// Judge specialization type
    pub judge_type: JudgeType,
    /// Judge's verdict/decision
    pub verdict: JudgeVerdict,
    /// Judge's confidence in their decision (0.0-1.0)
    pub confidence: f64,
    /// Detailed reasoning for the decision
    pub reasoning: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Model version used for the decision
    pub model_version: String,
    /// Token usage for the decision
    pub token_usage: u32,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Summary of a verdict

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerdictSummary {
    pub verdict: String,
    pub confidence: f64,
    pub reasoning: String,
}
