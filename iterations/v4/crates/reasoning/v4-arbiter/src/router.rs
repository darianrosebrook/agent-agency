//! Task Routing with Reasoning
//!
//! Implements INV-CORE-08: No Hidden Routers - All task routing decisions
//! must be logged with reasoning.

use serde::{Deserialize, Serialize};
use v4_types::task::{Environment, TaskPriority, TaskRequest};

/// Routing decision with full reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Decision ID
    pub decision_id: String,
    /// Task being routed
    pub task_id: String,
    /// Selected worker type
    pub worker_type: WorkerType,
    /// Worker ID (if specific worker selected)
    pub worker_id: Option<String>,
    /// Reasoning for the routing decision
    pub reasoning: RoutingReasoning,
    /// Routing factors considered
    pub factors: Vec<RoutingFactor>,
    /// Timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub decided_at: chrono::DateTime<chrono::Utc>,
}

impl RoutingDecision {
    /// Get a summary of the decision
    pub fn summary(&self) -> String {
        format!(
            "Route task {} to {:?} worker. Reason: {}",
            self.task_id, self.worker_type, self.reasoning.primary_reason
        )
    }
}

/// Types of workers tasks can be routed to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerType {
    /// Local execution worker
    Local,
    /// Sandboxed execution worker
    Sandboxed,
    /// Remote execution worker
    Remote,
    /// GPU-accelerated worker
    GpuAccelerated,
    /// Manual review (human in the loop)
    ManualReview,
}

impl WorkerType {
    /// Get a description of this worker type
    pub fn description(&self) -> &'static str {
        match self {
            Self::Local => "Local execution with standard permissions",
            Self::Sandboxed => "Isolated sandbox with restricted permissions",
            Self::Remote => "Remote execution on cluster",
            Self::GpuAccelerated => "GPU-accelerated execution",
            Self::ManualReview => "Requires human review before execution",
        }
    }
}

/// Reasoning for a routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingReasoning {
    /// Primary reason for the routing
    pub primary_reason: String,
    /// Secondary reasons
    pub secondary_reasons: Vec<String>,
    /// Alternatives considered
    pub alternatives_considered: Vec<AlternativeRoute>,
    /// Why alternatives were rejected
    pub rejection_reasons: Vec<String>,
}

/// An alternative route that was considered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeRoute {
    /// Worker type
    pub worker_type: WorkerType,
    /// Why it was considered
    pub consideration: String,
    /// Score (0.0 to 1.0)
    pub score: f64,
}

/// A factor considered in routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingFactor {
    /// Factor name
    pub name: String,
    /// Factor value
    pub value: String,
    /// Weight in decision (0.0 to 1.0)
    pub weight: f64,
    /// How it influenced the decision
    pub influence: FactorInfluence,
}

/// How a factor influenced the routing decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactorInfluence {
    /// Strongly favored the chosen route
    StronglyFavored,
    /// Mildly favored the chosen route
    Favored,
    /// Neutral
    Neutral,
    /// Opposed the chosen route
    Opposed,
    /// Strongly opposed (but overridden)
    StronglyOpposed,
}

/// Worker routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRouting {
    /// The routing decision
    pub decision: RoutingDecision,
    /// Priority for worker queue
    pub queue_priority: u32,
    /// Resource requirements
    pub resources: ResourceRequirements,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to retry on failure
    pub retry_on_failure: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
}

/// Resource requirements for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum memory in MB
    pub min_memory_mb: u64,
    /// Maximum memory in MB
    pub max_memory_mb: u64,
    /// CPU cores requested
    pub cpu_cores: u32,
    /// Whether GPU is required
    pub gpu_required: bool,
    /// Disk space in MB
    pub disk_mb: u64,
    /// Network access required
    pub network_access: bool,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_memory_mb: 256,
            max_memory_mb: 4096,
            cpu_cores: 1,
            gpu_required: false,
            disk_mb: 1024,
            network_access: false,
        }
    }
}

/// Task router that makes routing decisions
pub struct TaskRouter {
    /// Default worker type
    default_worker: WorkerType,
    /// Risk tier threshold for sandboxing
    sandbox_threshold: u8,
    /// Risk tier threshold for manual review
    review_threshold: u8,
}

impl TaskRouter {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            default_worker: WorkerType::Local,
            sandbox_threshold: 3,
            review_threshold: 5,
        }
    }

    /// Set the sandbox threshold
    pub fn with_sandbox_threshold(mut self, threshold: u8) -> Self {
        self.sandbox_threshold = threshold;
        self
    }

    /// Set the review threshold
    pub fn with_review_threshold(mut self, threshold: u8) -> Self {
        self.review_threshold = threshold;
        self
    }

    /// Route a task
    pub fn route(&self, task: &TaskRequest, risk_tier: u8) -> RoutingDecision {
        let mut factors = Vec::new();
        let mut alternatives = Vec::new();

        // Factor: Risk tier
        factors.push(RoutingFactor {
            name: "risk_tier".to_string(),
            value: risk_tier.to_string(),
            weight: 0.4,
            influence: if risk_tier >= self.review_threshold {
                FactorInfluence::StronglyOpposed
            } else if risk_tier >= self.sandbox_threshold {
                FactorInfluence::Opposed
            } else {
                FactorInfluence::Neutral
            },
        });

        // Factor: Priority
        factors.push(RoutingFactor {
            name: "priority".to_string(),
            value: format!("{:?}", task.priority),
            weight: 0.2,
            influence: match task.priority {
                TaskPriority::Critical => FactorInfluence::StronglyFavored,
                TaskPriority::High => FactorInfluence::Favored,
                TaskPriority::Normal => FactorInfluence::Neutral,
                TaskPriority::Low => FactorInfluence::Neutral,
            },
        });

        // Factor: Environment
        factors.push(RoutingFactor {
            name: "environment".to_string(),
            value: format!("{:?}", task.environment),
            weight: 0.2,
            influence: match task.environment {
                Environment::Production => FactorInfluence::StronglyOpposed,
                Environment::Staging => FactorInfluence::Opposed,
                Environment::Development => FactorInfluence::Favored,
            },
        });

        // Factor: Breaking changes allowed
        factors.push(RoutingFactor {
            name: "breaking_changes".to_string(),
            value: task.constraints.allow_breaking_changes.to_string(),
            weight: 0.2,
            influence: if task.constraints.allow_breaking_changes {
                FactorInfluence::Opposed
            } else {
                FactorInfluence::Neutral
            },
        });

        // Determine worker type based on factors
        let worker_type = if risk_tier >= self.review_threshold {
            WorkerType::ManualReview
        } else if risk_tier >= self.sandbox_threshold
            || task.environment == Environment::Production
        {
            WorkerType::Sandboxed
        } else {
            self.default_worker
        };

        // Consider alternatives
        if worker_type != WorkerType::Local {
            alternatives.push(AlternativeRoute {
                worker_type: WorkerType::Local,
                consideration: "Lower overhead".to_string(),
                score: if risk_tier < self.sandbox_threshold {
                    0.7
                } else {
                    0.2
                },
            });
        }
        if worker_type != WorkerType::Sandboxed {
            alternatives.push(AlternativeRoute {
                worker_type: WorkerType::Sandboxed,
                consideration: "Better isolation".to_string(),
                score: 0.5,
            });
        }

        // Build reasoning
        let primary_reason = match worker_type {
            WorkerType::ManualReview => format!(
                "Risk tier {} requires human review",
                risk_tier
            ),
            WorkerType::Sandboxed => {
                if task.environment == Environment::Production {
                    "Production environment requires sandbox isolation".to_string()
                } else {
                    format!("Risk tier {} requires sandbox isolation", risk_tier)
                }
            }
            WorkerType::Local => "Standard execution appropriate for task risk level".to_string(),
            WorkerType::Remote => "Task requires remote execution".to_string(),
            WorkerType::GpuAccelerated => "Task requires GPU acceleration".to_string(),
        };

        let mut rejection_reasons = Vec::new();
        for alt in &alternatives {
            if alt.score < 0.5 {
                rejection_reasons.push(format!(
                    "{:?} rejected: score {:.2} ({})",
                    alt.worker_type, alt.score, alt.consideration
                ));
            }
        }

        RoutingDecision {
            decision_id: uuid::Uuid::new_v4().to_string(),
            task_id: task.id.clone(),
            worker_type,
            worker_id: None,
            reasoning: RoutingReasoning {
                primary_reason,
                secondary_reasons: vec![],
                alternatives_considered: alternatives,
                rejection_reasons,
            },
            factors,
            decided_at: chrono::Utc::now(),
        }
    }

    /// Create full routing information
    pub fn create_routing(&self, task: &TaskRequest, risk_tier: u8) -> WorkerRouting {
        let decision = self.route(task, risk_tier);

        let resources = ResourceRequirements {
            network_access: task.constraints.allowed_paths.iter().any(|p| p.contains("http")),
            ..Default::default()
        };

        let timeout_seconds = task.constraints.max_duration_seconds.unwrap_or(3600);

        WorkerRouting {
            decision,
            queue_priority: match task.priority {
                TaskPriority::Critical => 100,
                TaskPriority::High => 75,
                TaskPriority::Normal => 50,
                TaskPriority::Low => 25,
            },
            resources,
            timeout_seconds,
            retry_on_failure: task.environment != Environment::Production,
            max_retries: if task.environment == Environment::Production {
                0
            } else {
                3
            },
        }
    }
}

impl Default for TaskRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::task::TaskConstraints;

    fn make_task(priority: TaskPriority, environment: Environment) -> TaskRequest {
        TaskRequest {
            id: "task-123".to_string(),
            title: "Test task".to_string(),
            description: "A test task".to_string(),
            constraints: TaskConstraints::default(),
            priority,
            environment,
            metadata: None,
        }
    }

    #[test]
    fn test_low_risk_local_routing() {
        let router = TaskRouter::new();
        let task = make_task(TaskPriority::Normal, Environment::Development);

        let decision = router.route(&task, 1);

        assert_eq!(decision.worker_type, WorkerType::Local);
        assert!(!decision.reasoning.primary_reason.is_empty());
        assert!(!decision.factors.is_empty());
    }

    #[test]
    fn test_high_risk_sandboxed_routing() {
        let router = TaskRouter::new();
        let task = make_task(TaskPriority::Normal, Environment::Development);

        let decision = router.route(&task, 4);

        assert_eq!(decision.worker_type, WorkerType::Sandboxed);
    }

    #[test]
    fn test_very_high_risk_manual_review() {
        let router = TaskRouter::new();
        let task = make_task(TaskPriority::Normal, Environment::Development);

        let decision = router.route(&task, 5);

        assert_eq!(decision.worker_type, WorkerType::ManualReview);
    }

    #[test]
    fn test_production_sandboxed() {
        let router = TaskRouter::new();
        let task = make_task(TaskPriority::Normal, Environment::Production);

        let decision = router.route(&task, 1); // Even low risk

        assert_eq!(decision.worker_type, WorkerType::Sandboxed);
    }

    #[test]
    fn test_routing_has_reasoning() {
        let router = TaskRouter::new();
        let task = make_task(TaskPriority::Normal, Environment::Development);

        let decision = router.route(&task, 1);

        // INV-CORE-08: Must have reasoning
        assert!(!decision.reasoning.primary_reason.is_empty());
        assert!(!decision.factors.is_empty());
    }

    #[test]
    fn test_full_routing() {
        let router = TaskRouter::new();
        let task = make_task(TaskPriority::High, Environment::Staging);

        let routing = router.create_routing(&task, 2);

        assert_eq!(routing.queue_priority, 75);
        assert!(routing.retry_on_failure);
        assert_eq!(routing.max_retries, 3);
    }

    #[test]
    fn test_production_no_retry() {
        let router = TaskRouter::new();
        let task = make_task(TaskPriority::Normal, Environment::Production);

        let routing = router.create_routing(&task, 1);

        assert!(!routing.retry_on_failure);
        assert_eq!(routing.max_retries, 0);
    }
}
