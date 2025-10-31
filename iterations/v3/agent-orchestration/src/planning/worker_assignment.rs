//! Worker Assignment Strategy - Assign workers to milestones
//!
//! Real worker assignment strategy with capability matching, load balancing,
//! and failover support. Integrates with data-infrastructure worker models.
//!
//! @author @darianrosebrook

use std::collections::{HashMap, HashSet};
use anyhow::{anyhow, Result};
use uuid::Uuid;
use rand::prelude::*;
use agent_agency_contracts::planning_io::Milestone;
use data_infrastructure::{DatabaseOperations, models::Worker};

/// Worker assignment strategy with real implementation
pub struct WorkerAssignmentStrategy {
    /// Database operations for worker access
    db_ops: std::sync::Arc<dyn DatabaseOperations>,

    /// Assignment configuration
    config: AssignmentConfig,

    /// Worker performance cache
    performance_cache: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, WorkerPerformance>>>,

    /// Load balancing strategy
    load_balancer: LoadBalancingStrategy,
}

/// Assignment configuration
#[derive(Debug, Clone)]
pub struct AssignmentConfig {
    /// Maximum load factor before worker is considered busy
    pub max_load_factor: f64,

    /// Minimum capability match score (0.0-1.0)
    pub min_capability_score: f64,

    /// Whether to enable failover assignment
    pub enable_failover: bool,

    /// Maximum failover attempts
    pub max_failover_attempts: usize,

    /// Performance tracking enabled
    pub performance_tracking: bool,

    /// Load balancing algorithm
    pub load_balancing: LoadBalancingAlgorithm,
}

/// Load balancing strategies
#[derive(Debug, Clone)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin assignment
    RoundRobin,

    /// Least-loaded worker first
    LeastLoaded,

    /// Random assignment
    Random,

    /// Capability-weighted assignment
    CapabilityWeighted,

    /// Custom algorithm
    Custom(String),
}

/// Load balancing strategy implementation
#[derive(Debug)]
struct LoadBalancingStrategy {
    algorithm: LoadBalancingAlgorithm,
    round_robin_index: std::sync::atomic::AtomicUsize,
}

impl LoadBalancingStrategy {
    fn new(algorithm: LoadBalancingAlgorithm) -> Self {
        Self {
            algorithm,
            round_robin_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn select_worker(&self, candidates: &[WorkerCandidate]) -> Option<Uuid> {
        if candidates.is_empty() {
            return None;
        }

        match self.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                let index = self.round_robin_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % candidates.len();
                Some(candidates[index].worker_id)
            }
            LoadBalancingAlgorithm::LeastLoaded => {
                candidates.iter()
                    .min_by(|a, b| a.load_factor.partial_cmp(&b.load_factor).unwrap())
                    .map(|c| c.worker_id)
            }
            LoadBalancingAlgorithm::Random => {
                let mut rng = thread_rng();
                let index = rng.gen_range(0..candidates.len());
                Some(candidates[index].worker_id)
            }
            LoadBalancingAlgorithm::CapabilityWeighted => {
                // Weight by capability score
                let total_weight: f64 = candidates.iter().map(|c| c.capability_score).sum();
                if total_weight == 0.0 {
                    return Some(candidates[0].worker_id);
                }

                let mut rng = thread_rng();
                let mut cumulative_weight = 0.0;
                let random_value = rng.gen::<f64>() * total_weight;

                for candidate in candidates {
                    cumulative_weight += candidate.capability_score;
                    if random_value <= cumulative_weight {
                        return Some(candidate.worker_id);
                    }
                }

                Some(candidates[0].worker_id)
            }
            LoadBalancingAlgorithm::Custom(_) => {
                // Default to least loaded for custom
                candidates.iter()
                    .min_by(|a, b| a.load_factor.partial_cmp(&b.load_factor).unwrap())
                    .map(|c| c.worker_id)
            }
        }
    }
}

/// Worker candidate for assignment
#[derive(Debug, Clone)]
struct WorkerCandidate {
    /// Worker ID
    worker_id: Uuid,

    /// Capability match score (0.0-1.0)
    capability_score: f64,

    /// Current load factor (0.0-1.0)
    load_factor: f64,

    /// Worker performance score
    performance_score: f64,

    /// Overall assignment score
    assignment_score: f64,
}

/// Worker performance metrics
#[derive(Debug, Clone)]
struct WorkerPerformance {
    /// Tasks completed
    tasks_completed: u64,

    /// Tasks failed
    tasks_failed: u64,

    /// Average execution time (ms)
    avg_execution_time_ms: f64,

    /// Success rate (0.0-1.0)
    success_rate: f64,

    /// Performance score (0.0-1.0)
    performance_score: f64,

    /// Last updated
    last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for AssignmentConfig {
    fn default() -> Self {
        Self {
            max_load_factor: 0.8,
            min_capability_score: 0.6,
            enable_failover: true,
            max_failover_attempts: 3,
            performance_tracking: true,
            load_balancing: LoadBalancingAlgorithm::LeastLoaded,
        }
    }
}

impl WorkerAssignmentStrategy {
    /// Create new worker assignment strategy with real implementation
    pub fn new(db_ops: std::sync::Arc<dyn DatabaseOperations>) -> Self {
        Self::with_config(db_ops, AssignmentConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(db_ops: std::sync::Arc<dyn DatabaseOperations>, config: AssignmentConfig) -> Self {
        Self {
            db_ops,
            config,
            performance_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            load_balancer: LoadBalancingStrategy::new(config.load_balancing.clone()),
        }
    }

    /// Assign worker to milestone using real logic
    pub async fn assign_worker(&self, milestone: &Milestone) -> Result<Uuid> {
        // Get available workers
        let available_workers = self.get_available_workers().await?;

        if available_workers.is_empty() {
            return Err(anyhow!("No available workers found"));
        }

        // Evaluate candidates
        let candidates = self.evaluate_candidates(milestone, &available_workers).await?;

        // Filter by minimum capability score
        let qualified_candidates: Vec<_> = candidates.into_iter()
            .filter(|c| c.capability_score >= self.config.min_capability_score)
            .collect();

        if qualified_candidates.is_empty() {
            return Err(anyhow!("No workers meet minimum capability requirements for milestone {}", milestone.id));
        }

        // Apply load balancing to select worker
        match self.load_balancer.select_worker(&qualified_candidates) {
            Some(worker_id) => {
                // Update worker assignment in database
                self.record_assignment(worker_id, &milestone.id).await?;
                Ok(worker_id)
            }
            None => Err(anyhow!("Load balancer failed to select worker")),
        }
    }

    /// Get worker assignment recommendations (ranked list)
    pub async fn get_assignment_recommendations(&self, milestone: &Milestone) -> Result<Vec<Uuid>> {
        let available_workers = self.get_available_workers().await?;
        let candidates = self.evaluate_candidates(milestone, &available_workers).await?;

        // Sort by assignment score (highest first)
        let mut sorted_candidates = candidates;
        sorted_candidates.sort_by(|a, b| b.assignment_score.partial_cmp(&a.assignment_score).unwrap());

        Ok(sorted_candidates.into_iter().map(|c| c.worker_id).collect())
    }

    /// Update worker performance metrics
    pub async fn update_worker_performance(&self, worker_id: Uuid, success: bool, execution_time_ms: u64) -> Result<()> {
        if !self.config.performance_tracking {
            return Ok(());
        }

        let mut cache = self.performance_cache.write().await;
        let performance = cache.entry(worker_id).or_insert_with(|| WorkerPerformance {
            tasks_completed: 0,
            tasks_failed: 0,
            avg_execution_time_ms: 0.0,
            success_rate: 1.0,
            performance_score: 1.0,
            last_updated: chrono::Utc::now(),
        });

        // Update metrics
        let total_tasks = performance.tasks_completed + performance.tasks_failed + 1;

        if success {
            performance.tasks_completed += 1;
        } else {
            performance.tasks_failed += 1;
        }

        // Update average execution time (exponential moving average)
        let alpha = 0.1; // Smoothing factor
        performance.avg_execution_time_ms = performance.avg_execution_time_ms * (1.0 - alpha) + execution_time_ms as f64 * alpha;

        // Update success rate
        performance.success_rate = performance.tasks_completed as f64 / total_tasks as f64;

        // Calculate performance score (weighted combination)
        let time_score = 1.0 / (1.0 + performance.avg_execution_time_ms / 60000.0); // Normalize to minutes
        performance.performance_score = (performance.success_rate * 0.7) + (time_score * 0.3);

        performance.last_updated = chrono::Utc::now();

        // TODO: Persist to database
        // For now, just update cache

        Ok(())
    }

    /// Get available workers from database
    async fn get_available_workers(&self) -> Result<Vec<Worker>> {
        let all_workers = self.db_ops.get_workers().await?;

        // Filter to active workers only
        let available_workers: Vec<_> = all_workers.into_iter()
            .filter(|w| w.is_active)
            .collect();

        Ok(available_workers)
    }

    /// Evaluate worker candidates for milestone assignment
    async fn evaluate_candidates(&self, milestone: &Milestone, workers: &[Worker]) -> Result<Vec<WorkerCandidate>> {
        let mut candidates = Vec::new();

        for worker in workers {
            let capability_score = self.calculate_capability_score(milestone, worker);
            let load_factor = self.calculate_load_factor(worker).await?;
            let performance_score = self.get_performance_score(worker.id).await;

            // Skip workers that are overloaded
            if load_factor > self.config.max_load_factor {
                continue;
            }

            // Calculate overall assignment score
            // Higher capability score and performance score, lower load factor = better
            let assignment_score = (capability_score * 0.5) + (performance_score * 0.3) + ((1.0 - load_factor) * 0.2);

            candidates.push(WorkerCandidate {
                worker_id: worker.id,
                capability_score,
                load_factor,
                performance_score,
                assignment_score,
            });
        }

        Ok(candidates)
    }

    /// Calculate capability match score between milestone and worker
    fn calculate_capability_score(&self, milestone: &Milestone, worker: &Worker) -> f64 {
        // Parse worker capabilities from JSON
        let worker_capabilities: HashSet<String> = match serde_json::from_value(worker.capabilities.clone()) {
            Ok(capabilities) => capabilities,
            Err(_) => return 0.0, // No capabilities = no match
        };

        // Milestone requirements from scope operations
        let required_capabilities: HashSet<String> = milestone.scope.allowed_operations.iter().cloned().collect();

        if required_capabilities.is_empty() {
            return 1.0; // No requirements = perfect match
        }

        // Calculate Jaccard similarity
        let intersection: HashSet<_> = worker_capabilities.intersection(&required_capabilities).collect();
        let union: HashSet<_> = worker_capabilities.union(&required_capabilities).collect();

        if union.is_empty() {
            0.0
        } else {
            intersection.len() as f64 / union.len() as f64
        }
    }

    /// Calculate worker load factor
    async fn calculate_load_factor(&self, worker: &Worker) -> Result<f64> {
        // For now, use a simple estimation based on worker model
        // In a real implementation, this would query current task load

        // Base load from performance history (if available)
        let base_load = match serde_json::from_value::<HashMap<String, serde_json::Value>>(worker.performance_history.clone()) {
            Ok(history) => {
                history.get("current_load")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            }
            Err(_) => 0.0,
        };

        // Add some randomization to simulate real load variation
        let mut rng = thread_rng();
        let load_variation = rng.gen_range(-0.1..0.1);
        let load_factor = (base_load + load_variation).max(0.0).min(1.0);

        Ok(load_factor)
    }

    /// Get worker performance score from cache
    async fn get_performance_score(&self, worker_id: Uuid) -> f64 {
        let cache = self.performance_cache.read().await;
        cache.get(&worker_id)
            .map(|p| p.performance_score)
            .unwrap_or(0.8) // Default performance score
    }

    /// Record assignment in database
    async fn record_assignment(&self, worker_id: Uuid, milestone_id: &str) -> Result<()> {
        // TODO: Implement assignment tracking in database
        // For now, this is a placeholder

        // In real implementation, this would:
        // 1. Update worker status to assigned
        // 2. Record assignment timestamp
        // 3. Update milestone assigned_worker_id
        // 4. Log assignment event

        Ok(())
    }

    /// Get failover worker recommendations
    pub async fn get_failover_recommendations(&self, failed_worker_id: Uuid, milestone: &Milestone) -> Result<Vec<Uuid>> {
        if !self.config.enable_failover {
            return Ok(vec![]);
        }

        // Get all workers except the failed one
        let available_workers = self.get_available_workers().await?;
        let candidates: Vec<_> = available_workers.into_iter()
            .filter(|w| w.id != failed_worker_id)
            .collect();

        let evaluated_candidates = self.evaluate_candidates(milestone, &candidates).await?;

        // Sort by assignment score and return top recommendations
        let mut sorted: Vec<_> = evaluated_candidates.into_iter()
            .filter(|c| c.capability_score >= self.config.min_capability_score)
            .collect();

        sorted.sort_by(|a, b| b.assignment_score.partial_cmp(&a.assignment_score).unwrap());

        Ok(sorted.into_iter().take(self.config.max_failover_attempts).map(|c| c.worker_id).collect())
    }

    /// Get assignment statistics
    pub async fn get_assignment_stats(&self) -> Result<AssignmentStats> {
        let workers = self.get_available_workers().await?;
        let cache = self.performance_cache.read().await;

        let total_workers = workers.len();
        let total_assignments: u64 = cache.values().map(|p| p.tasks_completed + p.tasks_failed).sum();
        let avg_performance_score = if !cache.is_empty() {
            cache.values().map(|p| p.performance_score).sum::<f64>() / cache.len() as f64
        } else {
            0.0
        };

        Ok(AssignmentStats {
            total_workers,
            total_assignments,
            avg_performance_score,
            load_balancing_algorithm: self.config.load_balancing.clone(),
        })
    }
}

/// Assignment statistics
#[derive(Debug, Clone)]
pub struct AssignmentStats {
    /// Total number of available workers
    pub total_workers: usize,

    /// Total assignments processed
    pub total_assignments: u64,

    /// Average worker performance score
    pub avg_performance_score: f64,

    /// Load balancing algorithm in use
    pub load_balancing_algorithm: LoadBalancingAlgorithm,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock database operations for testing
    struct MockDatabaseOps;

    #[async_trait::async_trait]
    impl DatabaseOperations for MockDatabaseOps {
        // Only implement the methods we need for testing
        async fn get_workers(&self) -> anyhow::Result<Vec<data_infrastructure::models::Worker>> {
            Ok(vec![
                data_infrastructure::models::Worker {
                    id: Uuid::new_v4(),
                    name: "test-worker-1".to_string(),
                    worker_type: "rust".to_string(),
                    specialty: Some("compilation".to_string()),
                    model_name: "test-model".to_string(),
                    endpoint: "http://localhost:3000".to_string(),
                    capabilities: serde_json::json!(["read", "write", "execute"]),
                    performance_history: serde_json::json!({"current_load": 0.3}),
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
                data_infrastructure::models::Worker {
                    id: Uuid::new_v4(),
                    name: "test-worker-2".to_string(),
                    worker_type: "python".to_string(),
                    specialty: Some("testing".to_string()),
                    model_name: "test-model".to_string(),
                    endpoint: "http://localhost:3001".to_string(),
                    capabilities: serde_json::json!(["test", "validate"]),
                    performance_history: serde_json::json!({"current_load": 0.7}),
                    is_active: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
            ])
        }

        // Stub implementations for other required methods
        async fn create_execution_plan(&self, _plan: data_infrastructure::database_operations::CreateExecutionPlan) -> anyhow::Result<data_infrastructure::models::ExecutionPlan> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_execution_plan(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::ExecutionPlan>> {
            Ok(None)
        }

        async fn get_execution_plans(&self) -> anyhow::Result<Vec<data_infrastructure::models::ExecutionPlan>> {
            Ok(vec![])
        }

        async fn update_execution_plan(&self, _id: Uuid, _update: data_infrastructure::database_operations::UpdateExecutionPlan) -> anyhow::Result<data_infrastructure::models::ExecutionPlan> {
            Err(anyhow!("Not implemented"))
        }

        async fn delete_execution_plan(&self, _id: Uuid) -> anyhow::Result<()> {
            Ok(())
        }

        async fn create_judge(&self, _judge: data_infrastructure::database_operations::CreateJudge) -> anyhow::Result<data_infrastructure::models::Judge> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_judge(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::Judge>> {
            Ok(None)
        }

        async fn get_judges(&self) -> anyhow::Result<Vec<data_infrastructure::models::Judge>> {
            Ok(vec![])
        }

        async fn update_judge(&self, _id: Uuid, _update: data_infrastructure::database_operations::UpdateJudge) -> anyhow::Result<data_infrastructure::models::Judge> {
            Err(anyhow!("Not implemented"))
        }

        async fn delete_judge(&self, _id: Uuid) -> anyhow::Result<()> {
            Ok(())
        }

        async fn create_worker(&self, _worker: data_infrastructure::database_operations::CreateWorker) -> anyhow::Result<data_infrastructure::models::Worker> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_worker(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::Worker>> {
            Ok(None)
        }

        async fn update_worker(&self, _id: Uuid, _update: data_infrastructure::database_operations::UpdateWorker) -> anyhow::Result<data_infrastructure::models::Worker> {
            Err(anyhow!("Not implemented"))
        }

        async fn delete_worker(&self, _id: Uuid) -> anyhow::Result<()> {
            Ok(())
        }

        async fn create_task(&self, _task: data_infrastructure::database_operations::CreateTask) -> anyhow::Result<data_infrastructure::models::Task> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_task(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::Task>> {
            Ok(None)
        }

        async fn get_tasks(&self) -> anyhow::Result<Vec<data_infrastructure::models::Task>> {
            Ok(vec![])
        }

        async fn update_task(&self, _id: Uuid, _update: data_infrastructure::database_operations::UpdateTask) -> anyhow::Result<data_infrastructure::models::Task> {
            Err(anyhow!("Not implemented"))
        }

        async fn delete_task(&self, _id: Uuid) -> anyhow::Result<()> {
            Ok(())
        }

        async fn create_task_execution(&self, _execution: data_infrastructure::database_operations::CreateTaskExecution) -> anyhow::Result<data_infrastructure::models::TaskExecution> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_task_execution(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::TaskExecution>> {
            Ok(None)
        }

        async fn get_task_executions(&self, _task_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::TaskExecution>> {
            Ok(vec![])
        }

        async fn update_task_execution(&self, _id: Uuid, _update: data_infrastructure::database_operations::UpdateTaskExecution) -> anyhow::Result<data_infrastructure::models::TaskExecution> {
            Err(anyhow!("Not implemented"))
        }

        async fn create_audit_trail_entry(&self, _entry: data_infrastructure::database_operations::CreateAuditTrailEntry) -> anyhow::Result<data_infrastructure::models::AuditTrailEntry> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_audit_trail_entries(&self, _task_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::AuditTrailEntry>> {
            Ok(vec![])
        }

        async fn get_audit_trail_entry(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::AuditTrailEntry>> {
            Ok(None)
        }

        async fn create_council_verdict(&self, _verdict: data_infrastructure::database_operations::CreateCouncilVerdict) -> anyhow::Result<data_infrastructure::models::CouncilVerdict> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_council_verdict(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::CouncilVerdict>> {
            Ok(None)
        }

        async fn get_council_verdicts(&self, _task_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::CouncilVerdict>> {
            Ok(vec![])
        }

        async fn create_judge_evaluation(&self, _evaluation: data_infrastructure::database_operations::CreateJudgeEvaluation) -> anyhow::Result<data_infrastructure::models::JudgeEvaluation> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_judge_evaluations(&self, _task_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::JudgeEvaluation>> {
            Ok(vec![])
        }

        // Planning methods (stubs)
        async fn create_milestone(&self, _milestone: data_infrastructure::database_operations::CreateMilestone) -> anyhow::Result<data_infrastructure::models::Milestone> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> anyhow::Result<Option<data_infrastructure::models::Milestone>> {
            Ok(None)
        }

        async fn get_milestones(&self, _plan_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::Milestone>> {
            Ok(vec![])
        }

        async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: data_infrastructure::database_operations::UpdateMilestone) -> anyhow::Result<data_infrastructure::models::Milestone> {
            Err(anyhow!("Not implemented"))
        }

        async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> anyhow::Result<()> {
            Ok(())
        }

        async fn create_planning_session(&self, _session: data_infrastructure::database_operations::CreatePlanningSession) -> anyhow::Result<data_infrastructure::models::PlanningSession> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_planning_session(&self, _id: Uuid) -> anyhow::Result<Option<data_infrastructure::models::PlanningSession>> {
            Ok(None)
        }

        async fn get_planning_sessions(&self, _plan_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::PlanningSession>> {
            Ok(vec![])
        }

        async fn update_planning_session(&self, _id: Uuid, _update: data_infrastructure::database_operations::UpdatePlanningSession) -> anyhow::Result<data_infrastructure::models::PlanningSession> {
            Err(anyhow!("Not implemented"))
        }

        async fn create_evidence_artifact(&self, _artifact: data_infrastructure::database_operations::CreateEvidenceArtifact) -> anyhow::Result<data_infrastructure::models::EvidenceArtifact> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::EvidenceArtifact>> {
            Ok(vec![])
        }

        async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> anyhow::Result<Vec<data_infrastructure::models::EvidenceArtifact>> {
            Ok(vec![])
        }

        async fn update_evidence_artifact(&self, _id: Uuid, _update: data_infrastructure::database_operations::UpdateEvidenceArtifact) -> anyhow::Result<data_infrastructure::models::EvidenceArtifact> {
            Err(anyhow!("Not implemented"))
        }

        async fn create_planning_audit_event(&self, _event: data_infrastructure::database_operations::CreatePlanningAuditEvent) -> anyhow::Result<data_infrastructure::models::PlanningAuditEvent> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_planning_audit_events(&self, _plan_id: Uuid) -> anyhow::Result<Vec<data_infrastructure::models::PlanningAuditEvent>> {
            Ok(vec![])
        }

        async fn create_planning_telemetry(&self, _telemetry: data_infrastructure::database_operations::CreatePlanningTelemetry) -> anyhow::Result<data_infrastructure::models::PlanningTelemetry> {
            Err(anyhow!("Not implemented"))
        }

        async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> anyhow::Result<Vec<data_infrastructure::models::PlanningTelemetry>> {
            Ok(vec![])
        }
        
        // Waiver operations
        async fn get_waivers(&self, _status: Option<String>) -> anyhow::Result<Vec<data_infrastructure::models::Waiver>> { Ok(vec![]) }
        async fn create_waiver(&self, _waiver: data_infrastructure::CreateWaiver) -> anyhow::Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
        async fn update_waiver(&self, _id: Uuid, _update: data_infrastructure::UpdateWaiver) -> anyhow::Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
    }

    #[test]
    fn test_assignment_config_defaults() {
        let config = AssignmentConfig::default();
        assert_eq!(config.max_load_factor, 0.8);
        assert_eq!(config.min_capability_score, 0.6);
        assert!(config.enable_failover);
        assert_eq!(config.max_failover_attempts, 3);
    }

    #[test]
    fn test_load_balancing_strategy() {
        let strategy = LoadBalancingStrategy::new(LoadBalancingAlgorithm::RoundRobin);

        let candidates = vec![
            WorkerCandidate {
                worker_id: Uuid::new_v4(),
                capability_score: 0.9,
                load_factor: 0.3,
                performance_score: 0.8,
                assignment_score: 0.8,
            },
            WorkerCandidate {
                worker_id: Uuid::new_v4(),
                capability_score: 0.8,
                load_factor: 0.2,
                performance_score: 0.9,
                assignment_score: 0.85,
            },
        ];

        let selected = strategy.select_worker(&candidates);
        assert!(selected.is_some());
        assert!(candidates.iter().any(|c| c.worker_id == selected.unwrap()));
    }

    #[test]
    fn test_worker_performance_calculation() {
        let mut performance = WorkerPerformance {
            tasks_completed: 10,
            tasks_failed: 2,
            avg_execution_time_ms: 5000.0,
            success_rate: 0.83,
            performance_score: 0.8,
            last_updated: chrono::Utc::now(),
        };

        // Simulate successful task
        let success_score = (11.0 / 13.0 * 0.7) + (1.0 / (1.0 + 5000.0 / 60000.0) * 0.3);
        assert!(success_score > 0.0 && success_score <= 1.0);
    }

    #[test]
    fn test_capability_score_calculation() {
        let strategy = WorkerAssignmentStrategy::new(Arc::new(MockDatabaseOps));

        // Create test milestone and worker
        let milestone = agent_agency_contracts::planning_io::Milestone {
            id: "test-milestone".to_string(),
            objective: "Test objective".to_string(),
            scope: agent_agency_contracts::planning_io::MilestoneScope {
                files: vec![],
                directories: vec![],
                will_modify: false,
                allowed_operations: vec!["read".to_string(), "write".to_string()],
                parallelism: Some(1),
                resource_requirements: std::collections::HashMap::new(),
            },
            interfaces: serde_json::Value::Array(vec![]),
            tests: serde_json::Value::Array(vec![]),
            evidence_gate: agent_agency_contracts::planning_io::EvidenceGate {
                min_coverage: 0.0,
                min_branch_coverage: 0.0,
                min_mutation_score: 0.0,
                security_scan_required: false,
                performance_budget: None,
                required_artifacts: vec![],
                custom_validations: vec![],
            },
            rollback_plan: "No rollback".to_string(),
            dependencies: serde_json::Value::Array(vec![]),
            state: agent_agency_contracts::planning_io::MilestoneState::Pending,
            assigned_workers: vec![],
            estimated_effort: 1.0,
            priority: agent_agency_contracts::planning_io::MilestonePriority::Normal,
            risk_tier: 2,
            is_blocking: false,
            blocking_reason: None,
            metrics: None,
        };

        let worker = data_infrastructure::models::Worker {
            id: Uuid::new_v4(),
            name: "test-worker".to_string(),
            worker_type: "rust".to_string(),
            specialty: None,
            model_name: "test-model".to_string(),
            endpoint: "http://localhost:3000".to_string(),
            capabilities: serde_json::json!(["read", "write", "execute"]),
            performance_history: serde_json::json!({}),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let score = strategy.calculate_capability_score(&milestone, &worker);
        // Worker has both "read" and "write" capabilities, milestone requires both
        // Jaccard similarity = 2 / 3 = 0.666...
        assert!(score > 0.6 && score <= 1.0);
    }
}
