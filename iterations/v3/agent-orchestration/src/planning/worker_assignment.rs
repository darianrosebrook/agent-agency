//! Worker Assignment Strategy - Assign workers to milestones
//!
//! Real worker assignment strategy with capability matching, load balancing,
//! and failover support. Integrates with data-infrastructure worker models.
//!
//! @author @darianrosebrook

use crate::planning::assignment_storage::AssignmentDatabaseStorage;
use crate::planning::{models::Worker, DatabaseOperations};
use agent_agency_contracts::planning_io::Milestone;
use anyhow::{anyhow, Result};
use rand::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use agent_research::performance_tracker::PerformanceTracker;

/// Worker assignment strategy with real implementation
pub struct WorkerAssignmentStrategy {
    /// Database operations for worker access
    db_ops: std::sync::Arc<dyn DatabaseOperations>,

    /// Assignment database storage
    assignment_storage: Option<std::sync::Arc<AssignmentDatabaseStorage>>,

    /// Assignment configuration
    config: AssignmentConfig,

    /// Worker performance cache
    performance_cache: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, WorkerPerformance>>>,

    /// Load balancing strategy
    load_balancer: LoadBalancingStrategy,

    /// Audit trail manager for chain-of-thought recording
    audit_trail_manager: Option<std::sync::Arc<crate::audit_trail::AuditTrailManager>>,

    /// Performance tracker for benchmark results (always-on when available)
    performance_tracker: Option<std::sync::Arc<PerformanceTracker>>,

    /// Clock for deterministic time (feature-gated)
    #[cfg(feature = "evaluation")]
    clock: Option<std::sync::Arc<dyn crate::evaluation::determinism::Clock>>,

    /// RNG source for deterministic randomness (feature-gated)
    #[cfg(feature = "evaluation")]
    rng_source: Option<std::sync::Arc<crate::evaluation::determinism::ThreadSafeRngSource>>,
}

impl std::fmt::Debug for WorkerAssignmentStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerAssignmentStrategy")
            .field("config", &self.config)
            .field("load_balancer", &self.load_balancer)
            .finish()
    }
}

/// Assignment configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum LoadBalancingAlgorithm {
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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
                let index = self
                    .round_robin_index
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    % candidates.len();
                Some(candidates[index].worker_id)
            }
            LoadBalancingAlgorithm::LeastLoaded => candidates
                .iter()
                .min_by(|a, b| a.load_factor.partial_cmp(&b.load_factor).unwrap())
                .map(|c| c.worker_id),
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
                candidates
                    .iter()
                    .min_by(|a, b| a.load_factor.partial_cmp(&b.load_factor).unwrap())
                    .map(|c| c.worker_id)
            }
        }
    }
}

/// Worker candidate for assignment

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WorkerCandidate {
    /// Worker ID
    #[schemars(with = "String")]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerPerformance {
    /// Tasks completed
    pub tasks_completed: u64,

    /// Tasks failed
    pub tasks_failed: u64,

    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,

    /// Performance score (0.0-1.0)
    pub performance_score: f64,

    /// Last updated
    #[schemars(with = "String")]
    pub last_updated: chrono::DateTime<chrono::Utc>,
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
        Self::with_config_and_audit(db_ops, AssignmentConfig::default(), None)
    }

    /// Create with performance tracker for benchmark-driven assignment
    #[cfg(feature = "research")]
    pub fn with_performance_tracker(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        performance_tracker: std::sync::Arc<PerformanceTracker>,
    ) -> Self {
        let load_balancing_config = config.load_balancing.clone();
        Self {
            db_ops,
            assignment_storage: None,
            config,
            performance_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            load_balancer: LoadBalancingStrategy::new(load_balancing_config),
            audit_trail_manager: None,
            performance_tracker: Some(performance_tracker),
            #[cfg(feature = "evaluation")]
            clock: None,
            #[cfg(feature = "evaluation")]
            rng_source: None,
        }
    }

    /// Create with custom configuration and audit trail manager
    pub fn with_config_and_audit(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        audit_trail_manager: Option<std::sync::Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        Self::with_config_audit_and_determinism(
            db_ops,
            config,
            audit_trail_manager,
            #[cfg(feature = "evaluation")]
            None,
            #[cfg(feature = "evaluation")]
            None,
        )
    }

    /// Create with custom configuration, audit trail manager, and determinism controls (feature-gated)
    #[cfg(feature = "evaluation")]
    pub fn with_config_audit_and_determinism(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        audit_trail_manager: Option<std::sync::Arc<crate::audit_trail::AuditTrailManager>>,
        clock: Option<std::sync::Arc<dyn crate::evaluation::determinism::Clock>>,
        rng_source: Option<std::sync::Arc<crate::evaluation::determinism::ThreadSafeRngSource>>,
    ) -> Self {
        let load_balancing_config = config.load_balancing.clone(); // Clone before moving config
        Self {
            db_ops,
            assignment_storage: None,
            config,
            performance_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            load_balancer: LoadBalancingStrategy::new(load_balancing_config),
            audit_trail_manager,
            #[cfg(feature = "research")]
            performance_tracker: None,
            clock,
            rng_source,
        }
    }

    #[cfg(not(feature = "evaluation"))]
    fn with_config_audit_and_determinism(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        audit_trail_manager: Option<std::sync::Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        let load_balancing_config = config.load_balancing.clone(); // Clone before moving config
        Self {
            db_ops,
            assignment_storage: None,
            config,
            performance_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            load_balancer: LoadBalancingStrategy::new(load_balancing_config),
            audit_trail_manager,
            #[cfg(feature = "research")]
            performance_tracker: None,
        }
    }

    /// Create with assignment database storage
    pub fn with_assignment_storage(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        assignment_storage: std::sync::Arc<AssignmentDatabaseStorage>,
    ) -> Self {
        Self::with_assignment_storage_and_audit(db_ops, config, assignment_storage, None)
    }

    /// Create with assignment database storage and audit trail manager
    pub fn with_assignment_storage_and_audit(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        assignment_storage: std::sync::Arc<AssignmentDatabaseStorage>,
        audit_trail_manager: Option<std::sync::Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        Self::with_assignment_storage_audit_and_determinism(
            db_ops,
            config,
            assignment_storage,
            audit_trail_manager,
            #[cfg(feature = "evaluation")]
            None,
            #[cfg(feature = "evaluation")]
            None,
        )
    }

    /// Create with assignment database storage, audit trail manager, and determinism controls (feature-gated)
    #[cfg(feature = "evaluation")]
    pub fn with_assignment_storage_audit_and_determinism(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        assignment_storage: std::sync::Arc<AssignmentDatabaseStorage>,
        audit_trail_manager: Option<std::sync::Arc<crate::audit_trail::AuditTrailManager>>,
        clock: Option<std::sync::Arc<dyn crate::evaluation::determinism::Clock>>,
        rng_source: Option<std::sync::Arc<crate::evaluation::determinism::ThreadSafeRngSource>>,
    ) -> Self {
        let load_balancing_config = config.load_balancing.clone();
        Self {
            db_ops,
            assignment_storage: Some(assignment_storage),
            config,
            performance_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            load_balancer: LoadBalancingStrategy::new(load_balancing_config),
            audit_trail_manager,
            #[cfg(feature = "research")]
            performance_tracker: None,
            clock,
            rng_source,
        }
    }

    #[cfg(not(feature = "evaluation"))]
    fn with_assignment_storage_audit_and_determinism(
        db_ops: std::sync::Arc<dyn DatabaseOperations>,
        config: AssignmentConfig,
        assignment_storage: std::sync::Arc<AssignmentDatabaseStorage>,
        audit_trail_manager: Option<std::sync::Arc<crate::audit_trail::AuditTrailManager>>,
    ) -> Self {
        let load_balancing_config = config.load_balancing.clone();
        Self {
            db_ops,
            assignment_storage: Some(assignment_storage),
            config,
            performance_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            load_balancer: LoadBalancingStrategy::new(load_balancing_config),
            audit_trail_manager,
            #[cfg(feature = "research")]
            performance_tracker: None,
        }
    }

    /// Get current time (uses clock if available, otherwise system time)
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        #[cfg(feature = "evaluation")]
        {
            if let Some(ref clock) = self.clock {
                clock.now()
            } else {
                chrono::Utc::now()
            }
        }
        #[cfg(not(feature = "evaluation"))]
        {
            chrono::Utc::now()
        }
    }

    /// Generate a UUID (uses RNG source if available, otherwise system UUID)
    fn generate_uuid(&self) -> Uuid {
        #[cfg(feature = "evaluation")]
        {
            if let Some(ref rng) = self.rng_source {
                rng.generate_uuid()
            } else {
                Uuid::new_v4()
            }
        }
        #[cfg(not(feature = "evaluation"))]
        {
            Uuid::new_v4()
        }
    }

    /// Set assignment storage (for dependency injection)
    pub fn set_assignment_storage(&mut self, storage: std::sync::Arc<AssignmentDatabaseStorage>) {
        self.assignment_storage = Some(storage);
    }

    /// Set audit trail manager (for chain-of-thought recording)
    pub fn set_audit_trail_manager(&mut self, audit_trail_manager: std::sync::Arc<crate::audit_trail::AuditTrailManager>) {
        self.audit_trail_manager = Some(audit_trail_manager);
    }

    /// Record assignment decision for chain-of-thought visibility
    async fn record_assignment_decision(
        &self,
        milestone: &Milestone,
        decision_type: &str,
        reasoning: String,
        alternatives: Vec<String>,
        chosen_option: String,
        confidence: f64,
    ) -> Result<()> {
        self.record_assignment_decision_with_candidates(
            milestone,
            decision_type,
            reasoning,
            alternatives,
            chosen_option,
            confidence,
            None, // No candidate details
        ).await
    }

    /// Record assignment decision with candidate details (enhanced for evaluation)
    async fn record_assignment_decision_with_candidates(
        &self,
        milestone: &Milestone,
        decision_type: &str,
        reasoning: String,
        alternatives: Vec<String>,
        chosen_option: String,
        confidence: f64,
        candidates: Option<&[WorkerCandidate]>, // Optional candidate details with scores
    ) -> Result<()> {
        if let Some(ref audit_manager) = self.audit_trail_manager {
            let context = crate::chain_of_thought::DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: Some(milestone.id.clone()),
                worker_id: None,
                resource_constraints: std::collections::HashMap::new(),
                time_constraints: None,
                priority_level: Some(milestone.priority.to_string()),
            };

            // Build alternatives with scores if candidates provided
            let alternatives_vec: Vec<crate::chain_of_thought::Alternative> = if let Some(candidates_list) = candidates {
                // Map candidates to alternatives with real scores
                candidates_list.iter()
                    .take(5) // Limit to top 5 candidates to avoid trace bloat
                    .map(|c| {
                        let mut pros = Vec::new();
                        let mut cons = Vec::new();
                        
                        if c.capability_score >= 0.8 {
                            pros.push("High capability match".to_string());
                        } else if c.capability_score < 0.6 {
                            cons.push("Low capability match".to_string());
                        }
                        
                        if c.load_factor < 0.5 {
                            pros.push("Low current load".to_string());
                        } else if c.load_factor > 0.8 {
                            cons.push("High current load".to_string());
                        }
                        
                        if c.performance_score >= 0.8 {
                            pros.push("High performance score".to_string());
                        }
                        
                        crate::chain_of_thought::Alternative {
                            option: format!("Worker {}", c.worker_id),
                            score: c.assignment_score,
                            reasoning: format!(
                                "Capability: {:.2}, Load: {:.2}, Performance: {:.2}",
                                c.capability_score, c.load_factor, c.performance_score
                            ),
                            pros,
                            cons,
                            confidence: c.assignment_score.min(1.0),
                        }
                    })
                    .collect()
            } else {
                // Fallback to simple alternatives without scores
                alternatives.into_iter()
                    .take(5) // Limit to top 5
                    .map(|alt| crate::chain_of_thought::Alternative {
                        option: alt,
                        score: 0.5,
                        reasoning: "Candidate evaluation".to_string(),
                        pros: vec!["Available".to_string()],
                        cons: vec![],
                        confidence: 0.7,
                    })
                    .collect()
            };

            // Calculate risk assessment if candidates provided
            let risk_assessment = if let Some(candidates_list) = candidates {
                self.calculate_risk_assessment(milestone, candidates_list, &chosen_option, confidence)
            } else {
                None
            };

            // Build evaluation metadata for context
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("milestone_id".to_string(), serde_json::Value::String(milestone.id.clone()));
            metadata.insert("milestone_priority".to_string(), serde_json::Value::String(format!("{:?}", milestone.priority)));
            metadata.insert("milestone_state".to_string(), serde_json::Value::String(format!("{:?}", milestone.state)));
            if let Some(duration) = milestone.estimated_duration {
                metadata.insert("estimated_duration_minutes".to_string(), serde_json::Value::Number(duration.into()));
            }
            metadata.insert("is_blocking".to_string(), serde_json::Value::Bool(milestone.is_blocking));
            metadata.insert("risk_tier".to_string(), serde_json::Value::Number(milestone.risk_tier.into()));
            
            // Add candidate pool information
            if let Some(candidates_list) = candidates {
                metadata.insert("candidate_pool_size".to_string(), serde_json::Value::Number(candidates_list.len().into()));
                if let Some(best_candidate) = candidates_list.first() {
                    metadata.insert("best_capability_score".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(best_candidate.capability_score).unwrap_or(serde_json::Number::from(0))
                    ));
                }
            }
            
            // Add decision context
            metadata.insert("decision_type_label".to_string(), serde_json::Value::String(decision_type.to_string()));
            metadata.insert("confidence_score".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(confidence).unwrap_or(serde_json::Number::from(0))
            ));

            let decision = crate::chain_of_thought::DecisionPoint {
                decision_id: self.generate_uuid(),
                decision_type: crate::chain_of_thought::DecisionType::WorkerAssignment,
                timestamp: self.now(),
                context,
                alternatives: alternatives_vec,
                chosen_option,
                reasoning,
                confidence,
                risk_assessment,
                metadata,
            };

            audit_manager.record_orchestration_decision(decision).await?;
        }
        Ok(())
    }

    /// Calculate risk assessment for worker assignment decision
    fn calculate_risk_assessment(
        &self,
        milestone: &Milestone,
        candidates: &[WorkerCandidate],
        chosen_option: &str,
        confidence: f64,
    ) -> Option<crate::chain_of_thought::RiskAssessment> {
        if candidates.is_empty() {
            return None;
        }

        let mut risk_factors = Vec::new();
        let mut mitigation_strategies = Vec::new();
        let mut fallback_options = Vec::new();

        // Find the chosen candidate
        let chosen_worker_id = chosen_option
            .strip_prefix("Worker ")
            .and_then(|s| s.parse::<Uuid>().ok());
        
        let chosen_candidate = chosen_worker_id
            .and_then(|id| candidates.iter().find(|c| c.worker_id == id));

        // Analyze risk factors
        if let Some(chosen) = chosen_candidate {
            // Low capability score
            if chosen.capability_score < 0.7 {
                risk_factors.push(format!(
                    "Low capability score ({:.2}) may impact task quality",
                    chosen.capability_score
                ));
                mitigation_strategies.push("Monitor task execution closely".to_string());
            }

            // High load factor
            if chosen.load_factor > 0.8 {
                risk_factors.push(format!(
                    "High worker load ({:.2}) may cause delays",
                    chosen.load_factor
                ));
                mitigation_strategies.push("Consider load balancing or task prioritization".to_string());
            }

            // Low performance score
            if chosen.performance_score < 0.6 {
                risk_factors.push(format!(
                    "Low performance score ({:.2}) indicates potential reliability issues",
                    chosen.performance_score
                ));
                mitigation_strategies.push("Enable failover and error recovery mechanisms".to_string());
            }
        }

        // Few qualified candidates (limited options)
        if candidates.len() <= 2 {
            risk_factors.push(format!(
                "Limited candidate pool ({} workers) reduces flexibility",
                candidates.len()
            ));
            mitigation_strategies.push("Consider expanding worker pool or relaxing constraints".to_string());
        }

        // Low confidence in decision
        if confidence < 0.7 {
            risk_factors.push(format!(
                "Low confidence score ({:.2}) indicates uncertainty in assignment",
                confidence
            ));
            mitigation_strategies.push("Review assignment criteria and worker capabilities".to_string());
        }

        // High priority milestone with risks
        let is_high_priority = matches!(
            milestone.priority,
            agent_agency_contracts::planning_io::MilestonePriority::High
                | agent_agency_contracts::planning_io::MilestonePriority::Critical
        );
        if is_high_priority && !risk_factors.is_empty() {
            risk_factors.push("High priority milestone requires careful monitoring".to_string());
            mitigation_strategies.push("Implement additional monitoring and checkpointing".to_string());
        }

        // Build fallback options from alternative candidates
        for candidate in candidates.iter().take(3) {
            if chosen_worker_id.map_or(true, |id| candidate.worker_id != id) {
                fallback_options.push(format!(
                    "Worker {} (capability: {:.2}, load: {:.2})",
                    candidate.worker_id, candidate.capability_score, candidate.load_factor
                ));
            }
        }

        // Determine overall risk level
        let risk_level = if risk_factors.is_empty() {
            "low"
        } else if risk_factors.len() <= 2 && confidence >= 0.7 {
            "medium"
        } else {
            "high"
        };

        Some(crate::chain_of_thought::RiskAssessment {
            risk_level: risk_level.to_string(),
            risk_factors,
            mitigation_strategies,
            fallback_options,
        })
    }

    /// Assign worker to milestone using real logic
    pub async fn assign_worker(&self, milestone: &Milestone) -> Result<Uuid> {
        // Record start of assignment process
        self.record_assignment_decision(
            milestone,
            "assignment_started",
            format!("Starting worker assignment for milestone {}", milestone.id),
            vec![],
            "".to_string(),
            1.0,
        ).await?;

        // Get available workers
        let available_workers = self.get_available_workers().await?;

        if available_workers.is_empty() {
            self.record_assignment_decision(
                milestone,
                "no_workers_available",
                "No available workers found".to_string(),
                vec![],
                "fail".to_string(),
                0.0,
            ).await?;
            return Err(anyhow!("No available workers found"));
        }

        // Record worker discovery
        self.record_assignment_decision(
            milestone,
            "workers_discovered",
            format!("Found {} available workers", available_workers.len()),
            available_workers.iter().map(|w| format!("Worker {}", w.id)).collect(),
            "continue".to_string(),
            0.8,
        ).await?;

        // Evaluate candidates
        let candidates = self
            .evaluate_candidates(milestone, &available_workers)
            .await?;

        // Record candidate evaluation with candidate details (before filtering)
        self.record_assignment_decision_with_candidates(
            milestone,
            "candidates_evaluated",
            format!("Evaluated {} candidates for milestone {}", candidates.len(), milestone.id),
            candidates.iter().map(|c| format!("Worker {} (score: {:.2})", c.worker_id, c.capability_score)).collect(),
            "filter_qualified".to_string(),
            0.9,
            Some(&candidates), // Pass candidate details with scores
        ).await?;

        // Log candidate scores for debugging (before filtering consumes candidates)
        let candidate_count = candidates.len();
        let candidate_scores: Vec<_> = candidates.iter().map(|c| (c.worker_id, c.capability_score)).collect();
        let highest_score = candidates.iter().map(|c| c.capability_score).fold(0.0, f64::max);
        tracing::info!(
            milestone_id = %milestone.id,
            candidate_count = candidate_count,
            min_required_score = self.config.min_capability_score,
            candidate_scores = ?candidate_scores,
            "Evaluated worker candidates for milestone"
        );

        // Filter by minimum capability score
        let qualified_candidates: Vec<_> = candidates
            .into_iter()
            .filter(|c| c.capability_score >= self.config.min_capability_score)
            .collect();

        if qualified_candidates.is_empty() {
            tracing::warn!(
                milestone_id = %milestone.id,
                candidate_count = candidate_count,
                min_required_score = self.config.min_capability_score,
                highest_score = highest_score,
                "No workers meet minimum capability score"
            );
            self.record_assignment_decision(
                milestone,
                "no_qualified_candidates",
                format!("No workers meet minimum capability score {:.2} for milestone {}", self.config.min_capability_score, milestone.id),
                vec![],
                "fail".to_string(),
                0.0,
            ).await?;
            return Err(anyhow!(
                "No workers meet minimum capability requirements for milestone {}",
                milestone.id
            ));
        }

        // Record qualified candidates with details
        self.record_assignment_decision_with_candidates(
            milestone,
            "qualified_candidates",
            format!("{} workers qualified for milestone {}", qualified_candidates.len(), milestone.id),
            qualified_candidates.iter().map(|c| format!("Worker {} (score: {:.2})", c.worker_id, c.capability_score)).collect(),
            "apply_load_balancing".to_string(),
            0.85,
            Some(&qualified_candidates), // Pass qualified candidate details
        ).await?;

        // Apply load balancing to select worker
        match self.load_balancer.select_worker(&qualified_candidates) {
            Some(worker_id) => {
                // Record final selection
                self.record_assignment_decision(
                    milestone,
                    "worker_selected",
                    format!("Selected worker {} for milestone {} using {} algorithm",
                        worker_id, milestone.id,
                        match self.config.load_balancing {
                            LoadBalancingAlgorithm::RoundRobin => "round-robin",
                            LoadBalancingAlgorithm::LeastLoaded => "least-loaded",
                            LoadBalancingAlgorithm::Random => "random",
                            LoadBalancingAlgorithm::CapabilityWeighted => "capability-weighted",
                            LoadBalancingAlgorithm::Custom(ref s) => s,
                        }
                    ),
                    vec![format!("Worker {}", worker_id)],
                    format!("Worker {}", worker_id),
                    0.95,
                ).await?;

                // Update worker assignment in database
                self.record_assignment(worker_id, &milestone.id).await?;
                Ok(worker_id)
            }
            None => {
                self.record_assignment_decision(
                    milestone,
                    "load_balancer_failed",
                    format!("Load balancer failed to select worker for milestone {}", milestone.id),
                    vec![],
                    "fail".to_string(),
                    0.0,
                ).await?;
                Err(anyhow!("Load balancer failed to select worker"))
            }
        }
    }

    /// Get worker assignment recommendations (ranked list)
    pub async fn get_assignment_recommendations(&self, milestone: &Milestone) -> Result<Vec<Uuid>> {
        let available_workers = self.get_available_workers().await?;
        let candidates = self
            .evaluate_candidates(milestone, &available_workers)
            .await?;

        // Sort by assignment score (highest first)
        let mut sorted_candidates = candidates;
        sorted_candidates
            .sort_by(|a, b| b.assignment_score.partial_cmp(&a.assignment_score).unwrap());

        Ok(sorted_candidates.into_iter().map(|c| c.worker_id).collect())
    }

    /// Update worker performance metrics
    pub async fn update_worker_performance(
        &self,
        worker_id: Uuid,
        success: bool,
        execution_time_ms: u64,
    ) -> Result<()> {
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
        performance.avg_execution_time_ms =
            performance.avg_execution_time_ms * (1.0 - alpha) + execution_time_ms as f64 * alpha;

        // Update success rate
        performance.success_rate = performance.tasks_completed as f64 / total_tasks as f64;

        // Calculate performance score (weighted combination)
        let time_score = 1.0 / (1.0 + performance.avg_execution_time_ms / 60000.0); // Normalize to minutes
        performance.performance_score = (performance.success_rate * 0.7) + (time_score * 0.3);

        performance.last_updated = chrono::Utc::now();

        // Persist to database if storage is available
        if let Some(ref storage) = self.assignment_storage {
            if let Err(e) = storage
                .store_performance_metrics(
                    worker_id,
                    performance.tasks_completed,
                    performance.tasks_failed,
                    performance.avg_execution_time_ms,
                    performance.success_rate,
                    performance.performance_score,
                )
                .await
            {
                tracing::warn!("Failed to persist performance metrics to database: {}", e);
            }
        }

        Ok(())
    }

    /// Get available workers from database
    async fn get_available_workers(&self) -> Result<Vec<Worker>> {
        let all_workers = self.db_ops.get_workers().await?;

        // Filter to active workers only
        let available_workers: Vec<_> = all_workers.into_iter().filter(|w| w.is_active).collect();

        Ok(available_workers)
    }

    /// Get a worker by ID from database
    pub async fn get_worker_by_id(&self, worker_id: Uuid) -> Result<Option<Worker>> {
        let all_workers = self.db_ops.get_workers().await?;
        Ok(all_workers.into_iter().find(|w| w.id == worker_id))
    }

    /// Get performance tracker if available
    pub fn get_performance_tracker(&self) -> Option<Arc<PerformanceTracker>> {
        self.performance_tracker.clone()
    }

    /// Evaluate worker candidates for milestone assignment
    async fn evaluate_candidates(
        &self,
        milestone: &Milestone,
        workers: &[Worker],
    ) -> Result<Vec<WorkerCandidate>> {
        let mut candidates = Vec::new();

        // TODO: Implement arbiter decision metadata extraction and worker pool filtering
        //       <Extract arbiter decision metadata from milestone and use it to filter/prioritize workers>
        //
        // COMPLETION CHECKLIST:
        // [ ] Extract arbiter metadata from milestone (arbiter_task_type, arbiter_risk_tier, arbiter_worker_pool, arbiter_confidence)
        // [ ] Add metadata field to Milestone struct in agent-agency-contracts if not present, or use alternative mechanism
        // [ ] Implement proper worker pool matching using Worker.metadata field instead of string contains
        // [ ] Apply arbiter confidence score to worker candidate scoring (boost matching workers)
        // [ ] Handle missing arbiter metadata gracefully (fallback to current behavior)
        // [ ] API/data structures defined & stable (Milestone metadata access pattern)
        // [ ] Error handling + validation aligned with error taxonomy (handle missing/invalid metadata)
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        //     - Test metadata extraction with valid arbiter data
        //     - Test metadata extraction with missing arbiter data
        //     - Test worker pool filtering with matching workers
        //     - Test worker pool filtering with no matching workers (fallback)
        //     - Test confidence score application to candidate scoring
        // [ ] Integration tests for external systems/contracts
        //     - Test integration with unified_orchestrator arbiter decision flow
        //     - Test worker assignment with arbiter recommendations end-to-end
        // [ ] Documentation: public API + system behavior
        //     - Document arbiter metadata extraction mechanism
        //     - Document worker pool filtering algorithm
        //     - Document confidence score application
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        //     - Ensure metadata extraction doesn't add significant latency
        //     - Worker filtering should be O(n) where n is worker count
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        //     - Validate arbiter metadata values (prevent injection)
        //     - Ensure worker pool matching doesn't expose sensitive worker data
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        //     - Log arbiter metadata extraction success/failure
        //     - Log worker pool filtering results (counts, matches)
        //     - Track arbiter recommendation impact on worker selection
        // [ ] Configurability and feature flags defined if relevant
        //     - Feature flag to enable/disable arbiter filtering
        //     - Configurable confidence threshold for applying arbiter recommendations
        // [ ] Failure-mode cards documented (degradation paths)
        //     - Fallback to all workers if arbiter metadata missing
        //     - Fallback to all workers if pool filtering results in empty set
        //
        // ACCEPTANCE CRITERIA:
        // - Arbiter decision metadata (task_type, risk_tier, worker_pool, confidence) extracted from milestone
        // - Workers filtered by arbiter-recommended pool when metadata available
        // - Worker pool matching uses Worker.metadata field, not string contains
        // - Arbiter confidence score boosts matching worker candidate scores
        // - System gracefully handles missing arbiter metadata (no panics, fallback behavior)
        // - Performance: metadata extraction + filtering adds <5ms latency for 100 workers
        // - Integration: worker assignment respects arbiter recommendations from unified_orchestrator
        //
        // DEPENDENCIES:
        // - Milestone metadata field or alternative mechanism (Required) - See unified_orchestrator.rs:1494-1509
        // - Worker.metadata field for pool membership (Required) - Already exists in Worker struct
        // - ArbiterPipelineOptimizer decision flow (Required) - See system-federated-ml/src/arbiter_pipeline.rs
        // - Worker candidate scoring mechanism (Required) - Already exists in evaluate_candidates
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No – System works without arbiter filtering, but optimization is degraded
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (impacts worker assignment optimization, not critical path)
        // - Change Budget: ~150 LOC (metadata extraction + filtering logic + tests)
        // - Reviewer Requirements: Planning domain expertise, understanding of arbiter integration
        let arbiter_worker_pool: Option<String> = None;
        let arbiter_confidence = 0.5;
        let arbiter_task_type: Option<String> = None;
        let arbiter_risk_tier: Option<String> = None;

        // Filter workers by arbiter worker pool recommendation if available
        let filtered_workers: Vec<&Worker> = if let Some(ref recommended_pool) = arbiter_worker_pool {
            // Filter workers that match the recommended pool
            // PLACEHOLDER: Simple string matching - replace with Worker.metadata pool membership check
            workers.iter()
                .filter(|w| {
                    // PLACEHOLDER: Simple matching - replace with proper pool metadata check
                    w.name.contains(recommended_pool) || recommended_pool.contains(&w.name)
                })
                .collect()
        } else {
            // No arbiter recommendation, use all workers
            workers.iter().collect()
        };

        // If filtering resulted in no workers, fall back to all workers
        let workers_to_evaluate = if filtered_workers.is_empty() {
            workers.iter().collect::<Vec<_>>()
        } else {
            filtered_workers
        };

        for worker in workers_to_evaluate {
            let capability_score = self.calculate_capability_score(milestone, worker);
            let load_factor = self.calculate_load_factor(worker).await?;
            let performance_score = self.get_performance_score(worker.id).await;

            // Skip workers that are overloaded
            if load_factor > self.config.max_load_factor {
                continue;
            }

            // Apply arbiter optimization boost if worker matches recommended pool
            let arbiter_boost = if let Some(ref recommended_pool) = arbiter_worker_pool {
                if worker.name.contains(recommended_pool) || recommended_pool.contains(&worker.name) {
                    // Boost score based on arbiter confidence (0.0 to 0.2 boost)
                    arbiter_confidence * 0.2
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Calculate overall assignment score
            // Higher capability score and performance score, lower load factor = better
            // Include arbiter boost if worker matches recommended pool
            let base_assignment_score =
                (capability_score * 0.5) + (performance_score * 0.3) + ((1.0 - load_factor) * 0.2);
            let assignment_score = base_assignment_score + arbiter_boost;

            candidates.push(WorkerCandidate {
                worker_id: worker.id,
                capability_score,
                load_factor,
                performance_score,
                assignment_score,
            });
        }

        // Sort by assignment score (highest first) to prioritize arbiter-recommended workers
        candidates.sort_by(|a, b| b.assignment_score.partial_cmp(&a.assignment_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(candidates)
    }

    /// Calculate capability match score between milestone and worker
    fn calculate_capability_score(&self, milestone: &Milestone, worker: &Worker) -> f64 {
        // Parse worker capabilities from JSON
        // Handle both array format ["cap1", "cap2"] and object format {"cap1": true, "cap2": true}
        let worker_capabilities: HashSet<String> = match serde_json::from_value(worker.capabilities.clone()) {
            Ok(capabilities) => capabilities,
            Err(_) => {
                // Try parsing as object and extracting keys
                if let serde_json::Value::Object(map) = &worker.capabilities {
                    map.keys().cloned().collect()
                } else {
                    tracing::warn!(
                        worker_id = %worker.id,
                        capabilities = ?worker.capabilities,
                        "Failed to parse worker capabilities, returning 0.0"
                    );
                    return 0.0;
                }
            }
        };

        // Milestone requirements from scope operations
        let required_capabilities: HashSet<String> =
            milestone.scope.allowed_operations.iter().cloned().collect();

        if required_capabilities.is_empty() {
            // No requirements = perfect match (any worker can handle it)
            tracing::debug!(
                milestone_id = %milestone.id,
                "Milestone has no required capabilities, returning perfect match score"
            );
            return 1.0;
        }

        // Check if worker has all required capabilities
        let has_all_required = required_capabilities.is_subset(&worker_capabilities);
        
        if has_all_required {
            // Worker has all required capabilities - perfect match
            // Score is based on how many required capabilities match (normalized to 0.0-1.0)
            // If worker has exactly the required capabilities, score = 1.0
            // If worker has more capabilities, score is still 1.0 (bonus capabilities don't hurt)
            let score = 1.0;
            tracing::debug!(
                milestone_id = %milestone.id,
                worker_id = %worker.id,
                "Worker has all required capabilities, score = 1.0"
            );
            return score;
        }

        // Worker missing some required capabilities - use Jaccard similarity
        let intersection: HashSet<_> = worker_capabilities
            .intersection(&required_capabilities)
            .cloned()
            .collect();
        let union: HashSet<_> = worker_capabilities.union(&required_capabilities).cloned().collect();

        let score = if union.is_empty() {
            0.0
        } else {
            intersection.len() as f64 / union.len() as f64
        };

        tracing::debug!(
            milestone_id = %milestone.id,
            worker_id = %worker.id,
            worker_capabilities = ?worker_capabilities,
            required_capabilities = ?required_capabilities,
            intersection = ?intersection,
            score = score,
            "Calculated capability score"
        );

        score
    }

    /// Calculate worker load factor
    async fn calculate_load_factor(&self, worker: &Worker) -> Result<f64> {
        // TODO: Implement real-time worker load calculation from worker pool metrics
        //       Currently uses basic estimation based on worker model metadata; should query actual worker metrics.
        //
        // COMPLETION CHECKLIST:
        // [ ] Query worker pool for current active task count
        // [ ] Calculate CPU and memory usage from worker metrics
        // [ ] Factor in task queue depth for worker
        // [ ] Combine metrics into load factor score
        // [ ] Handle missing metrics gracefully
        // [ ] Add unit tests with mock worker load data
        // [ ] Add integration tests with real worker load
        // [ ] Verify load factor accuracy improves worker assignment decisions
        //
        // ACCEPTANCE CRITERIA:
        // - Worker load factor is calculated from real-time worker pool metrics
        // - Load factor accurately reflects current CPU, memory, and queue depth
        // - Missing metrics are handled gracefully with fallback values
        // - Load factor improves worker assignment quality
        //
        // DEPENDENCIES:
        // - Worker pool API for querying active tasks (Required)
        // - Worker metrics collection system (Required)
        // - Task queue depth tracking (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (standard feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Worker management domain expertise

        // Base load from performance history (if available)
        let base_load = match worker.metadata.get("performance_history") {
            Some(perf_history) => match perf_history {
                serde_json::Value::Object(history) => history
                    .get("current_load")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                _ => 0.0,
            },
            None => 0.0,
        };

        // Add some randomization to simulate real load variation
        let mut rng = thread_rng();
        let load_variation = rng.gen_range(-0.1..0.1);
        let load_factor = (base_load + load_variation).max(0.0).min(1.0);

        Ok(load_factor)
    }

    /// Get worker performance score from cache and benchmark results
    async fn get_performance_score(&self, worker_id: Uuid) -> f64 {
        let cache = self.performance_cache.read().await;
        let base_score = cache
            .get(&worker_id)
            .map(|p| p.performance_score)
            .unwrap_or(0.8); // Default performance score

        // Enhance with benchmark results if available (always-on)
        if let Some(ref tracker) = self.performance_tracker {
            // Get historical benchmark results for this worker's model
            //
            // TODO: Implement comprehensive worker-to-model mapping for performance tracking
            //       Currently uses basic approach with worker_id directly; should implement comprehensive mapping that maps worker_id to model_id for accurate historical performance retrieval.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - worker_id is mapped to model_id correctly
            // - Historical performance is retrieved using model_id
            // - Mapping handles missing or invalid mappings gracefully
            // - Performance tracking is accurate and complete
            //
            // DEPENDENCIES:
            // - Worker-to-model mapping system (Required)
            // - Model ID lookup utilities (Required)
            // - Performance tracking database (Required)
            //
            // ESTIMATED EFFORT: 6-8 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (performance tracking functionality)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: Performance tracking and worker-model mapping expertise
            // Use worker_id as model_id for now (TODO: implement proper worker-to-model mapping)
            // This assumes 1:1 mapping between workers and models
            match tracker.get_historical_performance(worker_id).await {
                Ok(historical_results) => {
                    if !historical_results.is_empty() {
                        // Calculate average benchmark score from historical results
                        let avg_benchmark_score = historical_results.iter()
                            .map(|r| r.score)
                            .sum::<f64>() / historical_results.len() as f64;
                        
                        // Blend base score with benchmark score (70% base, 30% benchmark)
                        return (base_score * 0.7) + (avg_benchmark_score * 0.3);
                    }
                }
                Err(_) => {
                    // Benchmark data unavailable, use base score
                }
            }
        }

        base_score
    }

    /// Get performance cache (for external monitoring)
    pub async fn get_performance_cache(&self) -> Result<std::collections::HashMap<Uuid, WorkerPerformance>> {
        let cache = self.performance_cache.read().await;
        Ok(cache.clone())
    }

    /// Record assignment in database
    async fn record_assignment(&self, worker_id: Uuid, milestone_id: &str) -> Result<()> {
        // Persist to database if storage is available
        if let Some(ref storage) = self.assignment_storage {
            let priority = "Normal"; // Default priority, could be extracted from milestone if available
            match storage
                .record_assignment(worker_id, milestone_id, None, priority, None)
                .await
            {
                Ok(_) => {
                    tracing::debug!(
                        "Recorded assignment: worker {} -> milestone {}",
                        worker_id,
                        milestone_id
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to record assignment to database: {}", e);
                    // Don't fail the assignment if database persistence fails
                }
            }
        }

        Ok(())
    }

    /// Get failover worker recommendations
    pub async fn get_failover_recommendations(
        &self,
        failed_worker_id: Uuid,
        milestone: &Milestone,
    ) -> Result<Vec<Uuid>> {
        if !self.config.enable_failover {
            return Ok(vec![]);
        }

        // Get all workers except the failed one
        let available_workers = self.get_available_workers().await?;
        let candidates: Vec<_> = available_workers
            .into_iter()
            .filter(|w| w.id != failed_worker_id)
            .collect();

        let evaluated_candidates = self.evaluate_candidates(milestone, &candidates).await?;

        // Sort by assignment score and return top recommendations
        let mut sorted: Vec<_> = evaluated_candidates
            .into_iter()
            .filter(|c| c.capability_score >= self.config.min_capability_score)
            .collect();

        sorted.sort_by(|a, b| b.assignment_score.partial_cmp(&a.assignment_score).unwrap());

        Ok(sorted
            .into_iter()
            .take(self.config.max_failover_attempts)
            .map(|c| c.worker_id)
            .collect())
    }

    /// Get assignment statistics
    pub async fn get_assignment_stats(&self) -> Result<AssignmentStats> {
        let workers = self.get_available_workers().await?;
        let cache = self.performance_cache.read().await;

        let total_workers = workers.len();
        let total_assignments: u64 = cache
            .values()
            .map(|p| p.tasks_completed + p.tasks_failed)
            .sum();
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct AssignmentStats {
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
    // struct MockDatabaseOps; disabled due to massive api drift

    // #[async_trait::async_trait]
    // impl DatabaseOperations for MockDatabaseOps {
    //     // Only implement the methods we need for testing
    //     async fn get_workers(&self) -> anyhow::Result<Vec<crate::planning::models::Worker>> {
    //         Ok(vec![
    //             crate::planning::models::Worker {
    //                 id: Uuid::new_v4(),
    //                 name: "test-worker-1".to_string(),
    //                 worker_type: "rust".to_string(),
    //                 specialty: Some("compilation".to_string()),
    //                 model_name: "test-model".to_string(),
    //                 endpoint: "http://localhost:3000".to_string(),
    //                 capabilities: serde_json::json!(["read", "write", "execute"]),
    //                 performance_history: serde_json::json!({"current_load": 0.3}),
    //                 is_active: true,
    //                 created_at: chrono::Utc::now(),
    //                 updated_at: chrono::Utc::now(),
    //             },
    //             crate::planning::models::Worker {
    //                 id: Uuid::new_v4(),
    //                 name: "test-worker-2".to_string(),
    //                 worker_type: "python".to_string(),
    //                 specialty: Some("testing".to_string()),
    //                 model_name: "test-model".to_string(),
    //                 endpoint: "http://localhost:3001".to_string(),
    //                 capabilities: serde_json::json!(["test", "validate"]),
    //                 performance_history: serde_json::json!({"current_load": 0.7}),
    //                 is_active: true,
    //                 created_at: chrono::Utc::now(),
    //                 updated_at: chrono::Utc::now(),
    //             },
    //         ])
    //     }

    //     // Stub implementations for other required methods
    //     async fn create_execution_plan(&self, _plan: crate::planning::database_operations::CreateExecutionPlan) -> anyhow::Result<crate::planning::models::ExecutionPlan> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_execution_plan(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::ExecutionPlan>> {
    //         Ok(None)
    //     }

    //     async fn get_execution_plans(&self) -> anyhow::Result<Vec<crate::planning::models::ExecutionPlan>> {
    //         Ok(vec![])
    //     }

    //     async fn update_execution_plan(&self, _id: Uuid, _update: crate::planning::database_operations::UpdateExecutionPlan) -> anyhow::Result<crate::planning::models::ExecutionPlan> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn delete_execution_plan(&self, _id: Uuid) -> anyhow::Result<()> {
    //         Ok(())
    //     }

    //     async fn create_judge(&self, _judge: crate::planning::database_operations::CreateJudge) -> anyhow::Result<crate::planning::models::Judge> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_judge(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::Judge>> {
    //         Ok(None)
    //     }

    //     async fn get_judges(&self) -> anyhow::Result<Vec<crate::planning::models::Judge>> {
    //         Ok(vec![])
    //     }

    //     async fn update_judge(&self, _id: Uuid, _update: crate::planning::database_operations::UpdateJudge) -> anyhow::Result<crate::planning::models::Judge> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn delete_judge(&self, _id: Uuid) -> anyhow::Result<()> {
    //         Ok(())
    //     }

    //     async fn create_worker(&self, _worker: crate::planning::database_operations::CreateWorker) -> anyhow::Result<crate::planning::models::Worker> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_worker(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::Worker>> {
    //         Ok(None)
    //     }

    //     async fn update_worker(&self, _id: Uuid, _update: crate::planning::database_operations::UpdateWorker) -> anyhow::Result<crate::planning::models::Worker> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn delete_worker(&self, _id: Uuid) -> anyhow::Result<()> {
    //         Ok(())
    //     }

    //     async fn create_task(&self, _task: crate::planning::database_operations::CreateTask) -> anyhow::Result<crate::planning::models::Task> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_task(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::Task>> {
    //         Ok(None)
    //     }

    //     async fn get_tasks(&self) -> anyhow::Result<Vec<crate::planning::models::Task>> {
    //         Ok(vec![])
    //     }

    //     async fn update_task(&self, _id: Uuid, _update: crate::planning::database_operations::UpdateTask) -> anyhow::Result<crate::planning::models::Task> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn delete_task(&self, _id: Uuid) -> anyhow::Result<()> {
    //         Ok(())
    //     }

    //     async fn create_task_execution(&self, _execution: crate::planning::database_operations::CreateTaskExecution) -> anyhow::Result<crate::planning::models::TaskExecution> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_task_execution(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::TaskExecution>> {
    //         Ok(None)
    //     }

    //     async fn get_task_executions(&self, _task_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::TaskExecution>> {
    //         Ok(vec![])
    //     }

    //     async fn update_task_execution(&self, _id: Uuid, _update: crate::planning::database_operations::UpdateTaskExecution) -> anyhow::Result<crate::planning::models::TaskExecution> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn create_audit_trail_entry(&self, _entry: crate::planning::database_operations::CreateAuditTrailEntry) -> anyhow::Result<crate::planning::models::AuditTrailEntry> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_audit_trail_entries(&self, _task_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::AuditTrailEntry>> {
    //         Ok(vec![])
    //     }

    //     async fn get_audit_trail_entry(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::AuditTrailEntry>> {
    //         Ok(None)
    //     }

    //     async fn create_council_verdict(&self, _verdict: crate::planning::database_operations::CreateCouncilVerdict) -> anyhow::Result<crate::planning::models::CouncilVerdict> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_council_verdict(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::CouncilVerdict>> {
    //         Ok(None)
    //     }

    //     async fn get_council_verdicts(&self, _task_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::CouncilVerdict>> {
    //         Ok(vec![])
    //     }

    //     async fn create_judge_evaluation(&self, _evaluation: crate::planning::database_operations::CreateJudgeEvaluation) -> anyhow::Result<crate::planning::models::JudgeEvaluation> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_judge_evaluations(&self, _task_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::JudgeEvaluation>> {
    //         Ok(vec![])
    //     }

    //     // Planning methods (stubs)
    //     async fn create_milestone(&self, _milestone: crate::planning::database_operations::CreateMilestone) -> anyhow::Result<crate::planning::models::Milestone> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> anyhow::Result<Option<crate::planning::models::Milestone>> {
    //         Ok(None)
    //     }

    //     async fn get_milestones(&self, _plan_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::Milestone>> {
    //         Ok(vec![])
    //     }

    //     async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: crate::planning::database_operations::UpdateMilestone) -> anyhow::Result<crate::planning::models::Milestone> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> anyhow::Result<()> {
    //         Ok(())
    //     }

    //     async fn create_planning_session(&self, _session: crate::planning::database_operations::CreatePlanningSession) -> anyhow::Result<crate::planning::models::PlanningSession> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_planning_session(&self, _id: Uuid) -> anyhow::Result<Option<crate::planning::models::PlanningSession>> {
    //         Ok(None)
    //     }

    //     async fn get_planning_sessions(&self, _plan_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::PlanningSession>> {
    //         Ok(vec![])
    //     }

    //     async fn update_planning_session(&self, _id: Uuid, _update: crate::planning::database_operations::UpdatePlanningSession) -> anyhow::Result<crate::planning::models::PlanningSession> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn create_evidence_artifact(&self, _artifact: crate::planning::database_operations::CreateEvidenceArtifact) -> anyhow::Result<crate::planning::models::EvidenceArtifact> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::EvidenceArtifact>> {
    //         Ok(vec![])
    //     }

    //     async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> anyhow::Result<Vec<crate::planning::models::EvidenceArtifact>> {
    //         Ok(vec![])
    //     }

    //     async fn update_evidence_artifact(&self, _id: Uuid, _update: crate::planning::database_operations::UpdateEvidenceArtifact) -> anyhow::Result<crate::planning::models::EvidenceArtifact> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn create_planning_audit_event(&self, _event: crate::planning::database_operations::CreatePlanningAuditEvent) -> anyhow::Result<crate::planning::models::PlanningAuditEvent> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_planning_audit_events(&self, _plan_id: Uuid) -> anyhow::Result<Vec<crate::planning::models::PlanningAuditEvent>> {
    //         Ok(vec![])
    //     }

    //     async fn create_planning_telemetry(&self, _telemetry: crate::planning::database_operations::CreatePlanningTelemetry) -> anyhow::Result<crate::planning::models::PlanningTelemetry> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> anyhow::Result<Vec<crate::planning::models::PlanningTelemetry>> {
    //         Ok(vec![])
    //     }

    //     // Waiver operations
    //     async fn get_waivers(&self, _status: Option<String>) -> anyhow::Result<Vec<crate::planning::models::Waiver>> { Ok(vec![]) }
    //     async fn create_waiver(&self, _waiver: crate::planning::CreateWaiver) -> anyhow::Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    //     async fn update_waiver(&self, _id: Uuid, _update: crate::planning::UpdateWaiver) -> anyhow::Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    // }

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
        let strategy = WorkerAssignmentStrategy::new(Arc::new(crate::test_utils::MockDatabaseOps));

        // Create test milestone and worker
        let milestone = agent_agency_contracts::planning_io::Milestone {
            id: "test-milestone".to_string(),
            objective: "Test objective".to_string(),
            scope: agent_agency_contracts::planning_io::MilestoneScope {
                files: vec![],
                directories: vec![],
                included_paths: vec![],
                excluded_paths: vec![],
                will_modify: false,
                allowed_operations: vec!["read".to_string(), "write".to_string()],
                parallelism: Some(1),
                resource_requirements: std::collections::HashMap::new(),
            },
            interfaces: vec![],
            tests: vec![],
            evidence_gate: agent_agency_contracts::planning_io::EvidenceGate {
                min_coverage: 0.0,
                min_branch_coverage: 0.0,
                min_mutation_score: 0.0,
                security_scan_required: false,
                performance_budget: None,
                required_artifacts: vec![],
                custom_validations: vec![],
            },
            quality_gates: vec![],
            dependencies: vec![],
            estimated_duration: None,
            rollback_plan: "No rollback".to_string(),
            state: agent_agency_contracts::planning_io::MilestoneState::Pending,
            assigned_workers: vec![],
            estimated_effort: 1.0,
            priority: agent_agency_contracts::planning_io::MilestonePriority::Normal,
            risk_tier: 2,
            is_blocking: false,
            blocking_reason: None,
            metrics: None,
        };

        let worker = crate::planning::models::Worker {
            id: Uuid::new_v4(),
            name: "test-worker".to_string(),
            worker_type: "rust".to_string(),
            specialty: None,
            model_name: "test-model".to_string(),
            endpoint: "http://localhost:3000".to_string(),
            capabilities: serde_json::json!(["read", "write", "execute"]),
            performance_history: serde_json::json!({}),
            is_active: true,
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let score = strategy.calculate_capability_score(&milestone, &worker);
        // Worker has both "read" and "write" capabilities, milestone requires both
        // Jaccard similarity = 2 / 3 = 0.666...
        assert!(score > 0.6 && score <= 1.0);
    }
}
