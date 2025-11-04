//! Judge configuration and type definitions
//!
//! Core judge types, configuration, health metrics,
//! and session management structures.

use schemars::JsonSchema;
use crate::judge_backup::verdicts::JudgeVerdict;
use std::collections::HashMap;
use uuid::Uuid;

/// Judge type specialization

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum JudgeType {
    Constitutional,     // CAWS compliance and constitutional analysis
    Technical,          // Technical implementation analysis
    Quality,            // Quality assessment (alias for QualityAssurance)
    QualityAssurance,
    Security,
    Performance,
    Architecture,
    Testing,
    Compliance,
    DomainExpert,
    Ethics, // Advanced ethical reasoning judge
}

/// Judge configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct JudgeConfig {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub expertise_areas: Vec<String>,
    pub bias_tendencies: HashMap<String, f64>,
}

/// Judge contribution in a council session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct JudgeContribution {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub verdict: JudgeVerdict,
    pub processing_time_ms: u64,
    pub model_version: String,
    pub token_usage: Option<TokenUsage>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage statistics

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Review context provided to judges

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ReviewContext {
    pub working_spec: agent_agency_contracts::working_spec::WorkingSpec,
    pub planning_metadata: Option<PlanningMetadata>,
    pub previous_reviews: Vec<PreviousReview>,
    pub risk_tier: agent_agency_contracts::task_request::RiskTier,
    pub session_id: String,
    pub judge_instructions: HashMap<String, String>,
}

/// Planning metadata from the planning agent

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct PlanningMetadata {
    pub planning_duration: std::time::Duration,
    pub refinement_iterations: u32,
    pub caws_compliance_score: f64,
    pub validation_issues: Vec<String>,
}

/// Previous review in the session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct PreviousReview {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub verdict_summary: VerdictSummary,
    pub key_insights: Vec<String>,
}

/// Verdict summary for previous reviews

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum VerdictSummary {
    Approved { confidence: f64 },
    RequestedRefinement { change_count: usize },
    Rejected { critical_issue_count: usize },
}

impl std::fmt::Display for VerdictSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerdictSummary::Approved { confidence } =>
                write!(f, "Approved ({:.2} confidence)", confidence),
            VerdictSummary::RequestedRefinement { change_count } =>
                write!(f, "Requested {} changes", change_count),
            VerdictSummary::Rejected { critical_issue_count } =>
                write!(f, "Rejected ({} critical issues)", critical_issue_count),
        }
    }
}

/// Judge health metrics for monitoring

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct JudgeHealthMetrics {
    pub judge_id: String,
    pub response_time_avg_ms: u64,
    pub success_rate: f64,
    pub error_rate: f64,
    #[schemars(with = "String")]
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub consecutive_failures: u32,
    pub total_evaluations: u64,
    pub health_status: JudgeHealthStatus,
}

/// Judge health status

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum JudgeHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Judge performance metrics

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
struct JudgePerformanceStats {
    pub total_evaluations: u64,
    pub successful_evaluations: u64,
    pub failed_evaluations: u64,
    pub average_response_time_ms: u64,
    pub average_confidence: f64,
    #[schemars(with = "Option<String>")]
    pub last_evaluation: Option<chrono::DateTime<chrono::Utc>>,
}

/// Judge evaluation context

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct JudgeEvaluationContext {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub session_id: String,
    pub judge_config: JudgeConfig,
    pub review_context: ReviewContext,
    pub timeout: std::time::Duration,
    pub retry_count: u32,
}

/// Judge evaluation result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum JudgeEvaluationResult {
    Success(JudgeContribution),
    RetryableFailure {
        error: String,
        retry_after: std::time::Duration,
    },
    PermanentFailure {
        error: String,
    },
}


