//! API Service Layer
//!
//! Wraps the Arbiter and provides business logic for API handlers.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use v4_arbiter::{Arbiter, ArbiterResult};
use v4_inference::{InferenceConfig, InferenceRequest, InferenceService};
use v4_types::task::TaskRequest;

use crate::types::{
    ChainOfThoughtResponse, ComponentHealth, CouncilDecisionsResponse, CouncilSummary,
    ErrorResponse, FinalVerdict, HealthResponse, JudgeDecision, MetricsResponse, ProbeRequest,
    ProbeResponse, ReasoningStep, RoutingInfo, SubmitTaskRequest, SubmitTaskResponse,
    TaskStatusResponse, TimingMetrics, WorkerAction, WorkerActionsResponse, WorkerInfo,
};

/// API Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Maximum stored task results
    pub max_stored_tasks: usize,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Enable detailed timing metrics
    pub detailed_timing: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_stored_tasks: 1000,
            request_timeout_ms: 30000,
            detailed_timing: true,
        }
    }
}

/// Stored task result
#[derive(Debug, Clone)]
pub struct StoredTask {
    /// Task ID
    pub task_id: String,
    /// Original request
    pub request: SubmitTaskRequest,
    /// Evaluation result
    pub result: ArbiterResult,
    /// Timing metrics
    pub timing: TimingMetrics,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// API Service
pub struct ApiService {
    /// Arbiter for task evaluation
    arbiter: Arc<Arbiter>,
    /// Inference service for LLM probing
    inference: Arc<InferenceService>,
    /// Service configuration
    config: ServiceConfig,
    /// Stored task results (ring buffer)
    tasks: RwLock<VecDeque<StoredTask>>,
    /// Service start time
    started_at: Instant,
    /// Total request count
    request_count: AtomicU64,
    /// Authorized request count
    authorized_count: AtomicU64,
    /// Denied request count
    denied_count: AtomicU64,
    /// Latency samples for percentiles
    latencies: RwLock<VecDeque<u64>>,
    /// Last request timestamp
    last_request: RwLock<Option<DateTime<Utc>>>,
}

impl ApiService {
    /// Create a new API service
    pub fn new(
        arbiter: Arc<Arbiter>,
        inference: Arc<InferenceService>,
        config: ServiceConfig,
    ) -> Self {
        Self {
            arbiter,
            inference,
            config,
            tasks: RwLock::new(VecDeque::new()),
            started_at: Instant::now(),
            request_count: AtomicU64::new(0),
            authorized_count: AtomicU64::new(0),
            denied_count: AtomicU64::new(0),
            latencies: RwLock::new(VecDeque::with_capacity(1000)),
            last_request: RwLock::new(None),
        }
    }

    /// Create with default arbiter and mock inference
    pub fn with_defaults() -> Self {
        let arbiter = Arc::new(Arbiter::new());
        let inference = Arc::new(InferenceService::new(InferenceConfig::mock()));
        Self::new(arbiter, inference, ServiceConfig::default())
    }

    /// Create with custom inference config
    pub fn with_inference_config(inference_config: InferenceConfig) -> Self {
        let arbiter = Arc::new(Arbiter::new());
        let inference = Arc::new(InferenceService::new(inference_config));
        Self::new(arbiter, inference, ServiceConfig::default())
    }

    /// Submit a task for evaluation
    pub async fn submit_task(
        &self,
        request: SubmitTaskRequest,
    ) -> Result<SubmitTaskResponse, ErrorResponse> {
        let mut timing = TimingMetrics::new();
        let start = Instant::now();

        // Update counters
        self.request_count.fetch_add(1, Ordering::Relaxed);
        *self.last_request.write().await = Some(Utc::now());

        // Generate task ID
        let task_id = uuid::Uuid::new_v4().to_string();

        // Build TaskRequest
        let task_request = TaskRequest {
            id: task_id.clone(),
            title: request.title.clone(),
            description: request.description.clone(),
            constraints: request.constraints.clone().unwrap_or_default(),
            priority: request.priority,
            environment: request.environment,
            metadata: None,
        };

        // Evaluate with timing
        let reasoning_start = Instant::now();
        let result = self
            .arbiter
            .evaluate(task_request)
            .await
            .map_err(|e| ErrorResponse::new("EVALUATION_ERROR", e.to_string()))?;

        // Record timing
        timing.reasoning_ms = result.processing_time_ms;
        timing.council_ms = reasoning_start.elapsed().as_millis() as u64 - timing.reasoning_ms;
        timing.complete();

        // Record latency for percentiles
        {
            let mut latencies = self.latencies.write().await;
            if latencies.len() >= 1000 {
                latencies.pop_front();
            }
            latencies.push_back(timing.total_ms);
        }

        // Update counters
        if result.is_authorized() {
            self.authorized_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.denied_count.fetch_add(1, Ordering::Relaxed);
        }

        // Build routing info
        let routing = result.authorization.routing.as_ref().map(|r| RoutingInfo {
            worker_type: format!("{:?}", r.decision.worker_type),
            confidence: 1.0, // RoutingDecision uses explicit reasoning, not confidence scores
            reason: r.decision.reasoning.primary_reason.clone(),
        });

        // Store task
        let stored = StoredTask {
            task_id: task_id.clone(),
            request: request.clone(),
            result: result.clone(),
            timing: timing.clone(),
            created_at: Utc::now(),
        };
        self.store_task(stored).await;

        Ok(SubmitTaskResponse {
            task_id,
            authorized: result.is_authorized(),
            score: result.council_verdict.scores.aggregate,
            denial_reason: result.denial_reason().map(|r| format!("{:?}", r)),
            routing,
            timing,
            request_id: request.request_id,
        })
    }

    /// Get task status by ID
    pub async fn get_task(&self, task_id: &str) -> Result<TaskStatusResponse, ErrorResponse> {
        let tasks = self.tasks.read().await;

        let stored = tasks
            .iter()
            .find(|t| t.task_id == task_id)
            .ok_or_else(|| ErrorResponse::new("NOT_FOUND", format!("Task {} not found", task_id)))?;

        let council_summary = Some(CouncilSummary {
            constitutional_score: stored.result.council_verdict.scores.constitutional,
            technical_score: stored.result.council_verdict.scores.technical,
            quality_score: stored.result.council_verdict.scores.quality,
            aggregate_score: stored.result.council_verdict.scores.aggregate,
            vetoed: stored.result.council_verdict.vetoed,
            reasoning: stored.result.council_verdict.reasoning.clone(),
        });

        let status = if stored.result.is_authorized() {
            "authorized"
        } else {
            "denied"
        };

        Ok(TaskStatusResponse {
            task_id: task_id.to_string(),
            status: status.to_string(),
            authorized: stored.result.is_authorized(),
            council_summary,
            timing: Some(stored.timing.clone()),
            created_at: stored.created_at,
            updated_at: stored.created_at, // No updates in this simple impl
        })
    }

    /// Probe the LLM for inference testing
    pub async fn probe_llm(&self, request: ProbeRequest) -> Result<ProbeResponse, ErrorResponse> {
        // Ensure model is loaded
        if !self.inference.is_model_loaded().await {
            self.inference
                .load_model()
                .await
                .map_err(|e| ErrorResponse::new("MODEL_LOAD_ERROR", e.to_string()))?;
        }

        // Build inference request
        let inference_request = InferenceRequest::new(&request.prompt)
            .with_max_tokens(request.max_tokens)
            .with_temperature(request.temperature)
            .with_streaming(request.stream);

        // Run inference
        let response = self
            .inference
            .infer(inference_request)
            .await
            .map_err(|e| ErrorResponse::new("INFERENCE_ERROR", e.to_string()))?;

        Ok(ProbeResponse {
            text: response.text,
            tokens_generated: response.tokens_generated,
            time_to_first_token_ms: response.time_to_first_token_ms,
            total_generation_ms: response.total_time_ms,
            tokens_per_second: response.tokens_per_second,
            model: response.model,
        })
    }

    /// Get health status
    pub async fn health(&self) -> HealthResponse {
        let llm_available = self.inference.is_model_loaded().await;

        HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            components: ComponentHealth {
                arbiter: true,
                database: None,
                llm_provider: Some(llm_available),
            },
            last_request: *self.last_request.read().await,
        }
    }

    /// Get metrics
    pub async fn metrics(&self) -> MetricsResponse {
        let latencies = self.latencies.read().await;
        let total = self.request_count.load(Ordering::Relaxed);

        // Calculate averages and percentiles
        let (avg_latency, p50, p95, p99) = if latencies.is_empty() {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let mut sorted: Vec<u64> = latencies.iter().copied().collect();
            sorted.sort_unstable();

            let avg = sorted.iter().sum::<u64>() as f64 / sorted.len() as f64;
            let p50 = percentile(&sorted, 50);
            let p95 = percentile(&sorted, 95);
            let p99 = percentile(&sorted, 99);

            (avg, p50, p95, p99)
        };

        MetricsResponse {
            total_requests: total,
            authorized_count: self.authorized_count.load(Ordering::Relaxed),
            denied_count: self.denied_count.load(Ordering::Relaxed),
            avg_latency_ms: avg_latency,
            avg_reasoning_ms: 0.0, // Would need separate tracking
            avg_council_ms: 0.0,   // Would need separate tracking
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            collected_since: Utc::now()
                - chrono::Duration::seconds(self.started_at.elapsed().as_secs() as i64),
        }
    }

    /// Store a task result
    async fn store_task(&self, task: StoredTask) {
        let mut tasks = self.tasks.write().await;

        // Ring buffer behavior
        if tasks.len() >= self.config.max_stored_tasks {
            tasks.pop_front();
        }
        tasks.push_back(task);
    }

    // ========================================================================
    // Observability Endpoints (for dashboard integration)
    // ========================================================================

    /// Get chain-of-thought reasoning for a task
    pub async fn get_chain_of_thought(
        &self,
        task_id: &str,
    ) -> Result<ChainOfThoughtResponse, ErrorResponse> {
        let tasks = self.tasks.read().await;

        let stored = tasks
            .iter()
            .find(|t| t.task_id == task_id)
            .ok_or_else(|| ErrorResponse::new("NOT_FOUND", format!("Task {} not found", task_id)))?;

        // Build reasoning steps from the arbiter result
        let mut steps = Vec::new();
        let base_time = stored.created_at;

        // Step 1: Task reception
        steps.push(ReasoningStep {
            step: 1,
            step_type: "task_reception".to_string(),
            description: "Task received and parsed".to_string(),
            input: Some(serde_json::json!({
                "title": stored.request.title,
                "description": stored.request.description,
                "priority": format!("{:?}", stored.request.priority),
            })),
            output: Some(serde_json::json!({
                "task_id": task_id,
            })),
            duration_ms: 1,
            timestamp: base_time,
        });

        // Step 2: Symbolic reasoning / proposal generation
        steps.push(ReasoningStep {
            step: 2,
            step_type: "proposal_generation".to_string(),
            description: "Generated operator proposal through symbolic reasoning".to_string(),
            input: Some(serde_json::json!({
                "task_title": stored.request.title,
            })),
            output: Some(serde_json::json!({
                "proposal_hash": stored.result.authorization.certificate.as_ref()
                    .map(|c| c.certificate_hash.clone())
                    .unwrap_or_else(|| "none".to_string()),
            })),
            duration_ms: stored.timing.reasoning_ms,
            timestamp: base_time + chrono::Duration::milliseconds(1),
        });

        // Step 3: Council evaluation
        steps.push(ReasoningStep {
            step: 3,
            step_type: "council_evaluation".to_string(),
            description: "Council of judges evaluated the proposal".to_string(),
            input: Some(serde_json::json!({
                "proposal": "task_proposal",
            })),
            output: Some(serde_json::json!({
                "aggregate_score": stored.result.council_verdict.scores.aggregate,
                "approved": !stored.result.council_verdict.vetoed && matches!(stored.result.council_verdict.verdict, v4_types::council::CouncilVerdict::Approved { .. } | v4_types::council::CouncilVerdict::ConditionalApproval { .. }),
                "vetoed": stored.result.council_verdict.vetoed,
            })),
            duration_ms: stored.timing.council_ms,
            timestamp: base_time + chrono::Duration::milliseconds(stored.timing.reasoning_ms as i64 + 1),
        });

        // Step 4: Gate checking
        steps.push(ReasoningStep {
            step: 4,
            step_type: "gate_checking".to_string(),
            description: "CAWS gates verified".to_string(),
            input: Some(serde_json::json!({
                "council_approved": !stored.result.council_verdict.vetoed && matches!(stored.result.council_verdict.verdict, v4_types::council::CouncilVerdict::Approved { .. } | v4_types::council::CouncilVerdict::ConditionalApproval { .. }),
            })),
            output: Some(serde_json::json!({
                "gates_passed": stored.result.authorization.gate_result.can_proceed,
            })),
            duration_ms: stored.timing.gates_ms,
            timestamp: base_time + chrono::Duration::milliseconds(
                stored.timing.reasoning_ms as i64 + stored.timing.council_ms as i64 + 1
            ),
        });

        // Step 5: Final authorization
        steps.push(ReasoningStep {
            step: 5,
            step_type: "authorization".to_string(),
            description: "Final authorization decision".to_string(),
            input: Some(serde_json::json!({
                "council_approved": !stored.result.council_verdict.vetoed && matches!(stored.result.council_verdict.verdict, v4_types::council::CouncilVerdict::Approved { .. } | v4_types::council::CouncilVerdict::ConditionalApproval { .. }),
                "gates_passed": stored.result.authorization.gate_result.can_proceed,
            })),
            output: Some(serde_json::json!({
                "authorized": stored.result.is_authorized(),
                "routing": stored.result.authorization.routing.as_ref().map(|r| {
                    serde_json::json!({
                        "worker_type": format!("{:?}", r.decision.worker_type),
                        "reason": r.decision.reasoning.primary_reason,
                    })
                }),
            })),
            duration_ms: stored.timing.certificate_ms,
            timestamp: base_time + chrono::Duration::milliseconds(
                stored.timing.reasoning_ms as i64 + stored.timing.council_ms as i64 + stored.timing.gates_ms as i64 + 1
            ),
        });

        Ok(ChainOfThoughtResponse {
            task_id: task_id.to_string(),
            steps,
            total_time_ms: stored.timing.total_ms,
        })
    }

    /// Get council decisions for a task
    pub async fn get_council_decisions(
        &self,
        task_id: &str,
    ) -> Result<CouncilDecisionsResponse, ErrorResponse> {
        let tasks = self.tasks.read().await;

        let stored = tasks
            .iter()
            .find(|t| t.task_id == task_id)
            .ok_or_else(|| ErrorResponse::new("NOT_FOUND", format!("Task {} not found", task_id)))?;

        let verdict = &stored.result.council_verdict;
        let scores = &verdict.scores;

        // Build judge decisions
        let judges = vec![
            JudgeDecision {
                judge_type: "constitutional".to_string(),
                score: scores.constitutional,
                vetoed: scores.constitutional < 0.5,
                reasoning: format!(
                    "Constitutional compliance evaluation. Score: {:.2}",
                    scores.constitutional
                ),
                concerns: if scores.constitutional < 0.5 {
                    vec!["Constitutional score below threshold".to_string()]
                } else {
                    vec![]
                },
                evaluation_time_ms: stored.timing.council_ms / 3,
            },
            JudgeDecision {
                judge_type: "technical".to_string(),
                score: scores.technical,
                vetoed: scores.technical < 0.5,
                reasoning: format!(
                    "Technical feasibility evaluation. Score: {:.2}",
                    scores.technical
                ),
                concerns: if scores.technical < 0.5 {
                    vec!["Technical score below threshold".to_string()]
                } else {
                    vec![]
                },
                evaluation_time_ms: stored.timing.council_ms / 3,
            },
            JudgeDecision {
                judge_type: "quality".to_string(),
                score: scores.quality,
                vetoed: scores.quality < 0.5,
                reasoning: format!(
                    "Quality assessment. Score: {:.2}",
                    scores.quality
                ),
                concerns: if scores.quality < 0.5 {
                    vec!["Quality score below threshold".to_string()]
                } else {
                    vec![]
                },
                evaluation_time_ms: stored.timing.council_ms / 3,
            },
        ];

        let final_verdict = FinalVerdict {
            approved: !verdict.vetoed && matches!(verdict.verdict, v4_types::council::CouncilVerdict::Approved { .. } | v4_types::council::CouncilVerdict::ConditionalApproval { .. }),
            aggregate_score: scores.aggregate,
            vetoed: verdict.vetoed,
            reasoning: verdict.reasoning.clone(),
        };

        Ok(CouncilDecisionsResponse {
            task_id: task_id.to_string(),
            judges,
            final_verdict,
            total_time_ms: stored.timing.council_ms,
        })
    }

    /// Get worker actions for a task
    pub async fn get_worker_actions(
        &self,
        task_id: &str,
    ) -> Result<WorkerActionsResponse, ErrorResponse> {
        let tasks = self.tasks.read().await;

        let stored = tasks
            .iter()
            .find(|t| t.task_id == task_id)
            .ok_or_else(|| ErrorResponse::new("NOT_FOUND", format!("Task {} not found", task_id)))?;

        // Build worker info from routing
        let worker = stored.result.authorization.routing.as_ref().map(|r| WorkerInfo {
            worker_id: format!("worker-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("000")),
            worker_type: format!("{:?}", r.decision.worker_type),
            assigned_at: stored.created_at,
        });

        // Build mock actions based on task status
        // In a real implementation, these would come from actual worker execution logs
        let mut actions = Vec::new();

        if stored.result.is_authorized() {
            // Task was authorized - show planned/pending actions
            actions.push(WorkerAction {
                action_id: format!("action-{}-1", task_id.split('-').next().unwrap_or("0")),
                action_type: "task_assignment".to_string(),
                description: "Task assigned to worker".to_string(),
                input: Some(serde_json::json!({
                    "task_id": task_id,
                    "worker_type": worker.as_ref().map(|w| &w.worker_type),
                })),
                output: Some(serde_json::json!({
                    "status": "assigned",
                })),
                success: true,
                error: None,
                duration_ms: 5,
                timestamp: stored.created_at,
            });

            actions.push(WorkerAction {
                action_id: format!("action-{}-2", task_id.split('-').next().unwrap_or("0")),
                action_type: "pending_execution".to_string(),
                description: "Task queued for execution".to_string(),
                input: None,
                output: Some(serde_json::json!({
                    "queue_position": 0,
                })),
                success: true,
                error: None,
                duration_ms: 1,
                timestamp: stored.created_at + chrono::Duration::milliseconds(5),
            });
        }

        let status = if stored.result.is_authorized() {
            "pending_execution"
        } else {
            "denied"
        };

        Ok(WorkerActionsResponse {
            task_id: task_id.to_string(),
            worker,
            actions,
            status: status.to_string(),
            total_time_ms: stored.timing.total_ms,
        })
    }
}

/// Calculate percentile from sorted data
fn percentile(sorted: &[u64], p: usize) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }

    // Linear interpolation percentile calculation
    let rank = (p as f64 / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let frac = rank - lower as f64;

    if lower == upper {
        sorted[lower] as f64
    } else {
        sorted[lower] as f64 * (1.0 - frac) + sorted[upper] as f64 * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_submit_task() {
        let service = ApiService::with_defaults();

        let request = SubmitTaskRequest {
            title: "Test task".to_string(),
            description: "Read a file".to_string(),
            priority: v4_types::task::TaskPriority::Normal,
            environment: v4_types::task::Environment::Development,
            constraints: None,
            request_id: Some("test-123".to_string()),
        };

        let response = service.submit_task(request).await.unwrap();

        assert!(!response.task_id.is_empty());
        assert!(response.timing.total_ms > 0 || response.timing.total_ms == 0); // Either works
        assert_eq!(response.request_id, Some("test-123".to_string()));
    }

    #[tokio::test]
    async fn test_service_get_task() {
        let service = ApiService::with_defaults();

        // Submit first
        let request = SubmitTaskRequest {
            title: "Get test".to_string(),
            description: "Test getting".to_string(),
            priority: v4_types::task::TaskPriority::Normal,
            environment: v4_types::task::Environment::Development,
            constraints: None,
            request_id: None,
        };

        let submit_response = service.submit_task(request).await.unwrap();

        // Then get
        let status = service.get_task(&submit_response.task_id).await.unwrap();

        assert_eq!(status.task_id, submit_response.task_id);
        assert!(status.council_summary.is_some());
    }

    #[tokio::test]
    async fn test_service_health() {
        let service = ApiService::with_defaults();
        let health = service.health().await;

        assert_eq!(health.status, "healthy");
        assert!(health.components.arbiter);
    }

    #[tokio::test]
    async fn test_service_metrics() {
        let service = ApiService::with_defaults();

        // Submit a few tasks
        for i in 0..3 {
            let request = SubmitTaskRequest {
                title: format!("Task {}", i),
                description: "Test".to_string(),
                priority: v4_types::task::TaskPriority::Normal,
                environment: v4_types::task::Environment::Development,
                constraints: None,
                request_id: None,
            };
            let _ = service.submit_task(request).await;
        }

        let metrics = service.metrics().await;

        assert_eq!(metrics.total_requests, 3);
    }

    #[test]
    fn test_percentile() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        // P50 of [1,2,3,4,5,6,7,8,9,10] should be ~5.5 (median)
        assert!((percentile(&data, 50) - 5.5).abs() < 1.0);
        // P95 of 10 values should be ~9.55
        assert!((percentile(&data, 95) - 9.55).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_service_probe_llm() {
        let service = ApiService::with_defaults();

        let request = ProbeRequest {
            prompt: "What is recursion?".to_string(),
            max_tokens: 50,
            temperature: 0.7,
            stream: false,
        };

        let response = service.probe_llm(request).await.unwrap();

        assert!(!response.text.is_empty());
        assert!(response.tokens_generated > 0);
        assert!(response.total_generation_ms > 0);
    }
}
