//! Judge types and configurations
//!
//! Core types for judge configuration, health metrics, and review contexts.

use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Configuration for a judge instance
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub name: String,
    pub specialization: String,
    pub max_response_time_ms: u64,
    pub health_check_interval_ms: u64,
}

/// Health metrics for a judge
#[derive(Debug, Clone)]
pub struct JudgeHealthMetrics {
    pub is_healthy: bool,
    pub response_time_p95_ms: u64,
    pub error_rate: f64,
    pub last_health_check: DateTime<Utc>,
}

/// Context for a review session
#[derive(Debug, Clone)]
pub struct ReviewContext {
    pub session_id: String,
    pub working_spec: String,
    pub previous_reviews: Vec<PreviousReview>,
    pub constraints: HashMap<String, String>,
}

/// Previous review information
#[derive(Debug, Clone)]
pub struct PreviousReview {
    pub judge_name: String,
    pub timestamp: DateTime<Utc>,
    pub verdict_summary: String,
}

/// Judge contribution to a review
#[derive(Debug, Clone)]
pub struct JudgeContribution {
    pub judge_name: String,
    pub specialization: String,
    pub confidence: f64,
    pub reasoning: String,
}

/// Summary of a verdict
#[derive(Debug, Clone)]
pub struct VerdictSummary {
    pub verdict: String,
    pub confidence: f64,
    pub reasoning: String,
}
