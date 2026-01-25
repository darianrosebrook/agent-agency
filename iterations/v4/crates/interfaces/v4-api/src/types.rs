//! API Request/Response Types
//!
//! Types for HTTP API communication with timing metrics for LLM probing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use v4_types::task::{Environment, TaskConstraints, TaskPriority};

/// Request to submit a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskRequest {
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Task priority
    #[serde(default)]
    pub priority: TaskPriority,
    /// Environment to execute in
    #[serde(default)]
    pub environment: Environment,
    /// Optional constraints
    pub constraints: Option<TaskConstraints>,
    /// Client-provided request ID for tracing
    pub request_id: Option<String>,
}

/// Response from task submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskResponse {
    /// Assigned task ID
    pub task_id: String,
    /// Whether the task was authorized
    pub authorized: bool,
    /// Authorization score (0.0-1.0)
    pub score: f64,
    /// Denial reason if not authorized
    pub denial_reason: Option<String>,
    /// Routing decision
    pub routing: Option<RoutingInfo>,
    /// Timing metrics for LLM probing
    pub timing: TimingMetrics,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

/// Routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    /// Worker type selected
    pub worker_type: String,
    /// Confidence in routing decision
    pub confidence: f64,
    /// Primary reasoning
    pub reason: String,
}

/// Timing metrics for measuring LLM integration performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    /// Total request processing time (ms)
    pub total_ms: u64,
    /// Time for symbolic reasoning / proposal generation (ms)
    pub reasoning_ms: u64,
    /// Time for council evaluation (ms)
    pub council_ms: u64,
    /// Time for gate checking (ms)
    pub gates_ms: u64,
    /// Time for certificate generation (ms)
    pub certificate_ms: u64,
    /// Timestamp when processing started
    pub started_at: DateTime<Utc>,
    /// Timestamp when processing completed
    pub completed_at: DateTime<Utc>,
}

impl TimingMetrics {
    /// Create a new timing metrics instance
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            total_ms: 0,
            reasoning_ms: 0,
            council_ms: 0,
            gates_ms: 0,
            certificate_ms: 0,
            started_at: now,
            completed_at: now,
        }
    }

    /// Record completion
    pub fn complete(&mut self) {
        self.completed_at = Utc::now();
        self.total_ms = (self.completed_at - self.started_at).num_milliseconds() as u64;
    }
}

impl Default for TimingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to get task status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskRequest {
    /// Task ID to query
    pub task_id: String,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    /// Task ID
    pub task_id: String,
    /// Current status
    pub status: String,
    /// Authorized status
    pub authorized: bool,
    /// Council verdict summary
    pub council_summary: Option<CouncilSummary>,
    /// Timing from original evaluation
    pub timing: Option<TimingMetrics>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Council verdict summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSummary {
    /// Constitutional judge score
    pub constitutional_score: f64,
    /// Technical judge score
    pub technical_score: f64,
    /// Quality judge score
    pub quality_score: f64,
    /// Aggregate score
    pub aggregate_score: f64,
    /// Whether any judge vetoed
    pub vetoed: bool,
    /// Reasoning
    pub reasoning: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Component health
    pub components: ComponentHealth,
    /// Last request timestamp
    pub last_request: Option<DateTime<Utc>>,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Arbiter health
    pub arbiter: bool,
    /// Database health (if configured)
    pub database: Option<bool>,
    /// LLM provider health (if configured)
    pub llm_provider: Option<bool>,
}

/// Metrics response for LLM probing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests authorized
    pub authorized_count: u64,
    /// Requests denied
    pub denied_count: u64,
    /// Average total latency (ms)
    pub avg_latency_ms: f64,
    /// Average reasoning latency (ms)
    pub avg_reasoning_ms: f64,
    /// Average council latency (ms)
    pub avg_council_ms: f64,
    /// P50 latency (ms)
    pub p50_latency_ms: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: f64,
    /// P99 latency (ms)
    pub p99_latency_ms: f64,
    /// Collected since
    pub collected_since: DateTime<Utc>,
}

/// LLM probe request for testing inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeRequest {
    /// Prompt to send to LLM
    pub prompt: String,
    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature (0.0-1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Whether to stream response
    #[serde(default)]
    pub stream: bool,
}

fn default_max_tokens() -> u32 {
    256
}

fn default_temperature() -> f32 {
    0.7
}

/// LLM probe response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResponse {
    /// Generated text
    pub text: String,
    /// Tokens generated
    pub tokens_generated: u32,
    /// Time to first token (ms)
    pub time_to_first_token_ms: u64,
    /// Total generation time (ms)
    pub total_generation_ms: u64,
    /// Tokens per second
    pub tokens_per_second: f64,
    /// Model used
    pub model: String,
}

// ============================================================================
// Observability Types (for dashboard integration)
// ============================================================================

/// Chain-of-thought response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfThoughtResponse {
    /// Task ID
    pub task_id: String,
    /// Reasoning steps
    pub steps: Vec<ReasoningStep>,
    /// Total reasoning time (ms)
    pub total_time_ms: u64,
}

/// A single reasoning step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    /// Step number
    pub step: u32,
    /// Step type (e.g., "proposal", "validation", "routing")
    pub step_type: String,
    /// Description of what happened
    pub description: String,
    /// Input to this step
    pub input: Option<serde_json::Value>,
    /// Output from this step
    pub output: Option<serde_json::Value>,
    /// Duration (ms)
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Council decisions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilDecisionsResponse {
    /// Task ID
    pub task_id: String,
    /// Individual judge decisions
    pub judges: Vec<JudgeDecision>,
    /// Final verdict
    pub final_verdict: FinalVerdict,
    /// Total council deliberation time (ms)
    pub total_time_ms: u64,
}

/// A single judge's decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeDecision {
    /// Judge type
    pub judge_type: String,
    /// Score (0.0-1.0)
    pub score: f64,
    /// Whether this judge vetoed
    pub vetoed: bool,
    /// Reasoning for the decision
    pub reasoning: String,
    /// Specific concerns raised
    pub concerns: Vec<String>,
    /// Evaluation time (ms)
    pub evaluation_time_ms: u64,
}

/// Final verdict from council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalVerdict {
    /// Whether approved
    pub approved: bool,
    /// Aggregate score
    pub aggregate_score: f64,
    /// Whether vetoed by any judge
    pub vetoed: bool,
    /// Summary reasoning
    pub reasoning: String,
}

/// Worker actions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerActionsResponse {
    /// Task ID
    pub task_id: String,
    /// Worker assignment info
    pub worker: Option<WorkerInfo>,
    /// Actions taken
    pub actions: Vec<WorkerAction>,
    /// Current status
    pub status: String,
    /// Total execution time (ms)
    pub total_time_ms: u64,
}

/// Worker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Worker ID
    pub worker_id: String,
    /// Worker type
    pub worker_type: String,
    /// Assigned at timestamp
    pub assigned_at: DateTime<Utc>,
}

/// A single worker action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAction {
    /// Action ID
    pub action_id: String,
    /// Action type (e.g., "tool_call", "llm_query", "file_write")
    pub action_type: String,
    /// Description
    pub description: String,
    /// Input parameters
    pub input: Option<serde_json::Value>,
    /// Output result
    pub output: Option<serde_json::Value>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration (ms)
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Error Types
// ============================================================================

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            request_id: None,
            timestamp: Utc::now(),
        }
    }

    /// Add request ID
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_metrics() {
        let mut timing = TimingMetrics::new();
        timing.reasoning_ms = 50;
        timing.council_ms = 30;
        timing.complete();

        // total_ms is u64, so just verify it has a value
        let _ = timing.total_ms;
        assert!(timing.completed_at >= timing.started_at);
    }

    #[test]
    fn test_submit_request_serialization() {
        let request = SubmitTaskRequest {
            title: "Test task".to_string(),
            description: "Test description".to_string(),
            priority: TaskPriority::Normal,
            environment: Environment::Development,
            constraints: None,
            request_id: Some("req-123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: SubmitTaskRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.title, "Test task");
        assert_eq!(parsed.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse::new("VALIDATION_ERROR", "Invalid task title")
            .with_request_id("req-456");

        assert_eq!(error.code, "VALIDATION_ERROR");
        assert_eq!(error.request_id, Some("req-456".to_string()));
    }

    #[test]
    fn test_probe_request_defaults() {
        let json = r#"{"prompt": "Hello"}"#;
        let request: ProbeRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.prompt, "Hello");
        assert_eq!(request.max_tokens, 256);
        assert!((request.temperature - 0.7).abs() < 0.01);
        assert!(!request.stream);
    }
}
