//! Task Router
//!
//! Routes tasks to appropriate workers based on capabilities, load, and other factors.

use crate::types::*;
use crate::{LoadBalancingStrategy, RoutingAlgorithm};
use agent_agency_council::models::{RiskTier, TaskSpec};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundRobinState {
    current_weights: HashMap<Uuid, f64>,
    base_weights: HashMap<Uuid, f64>,
    last_selected: Option<Uuid>,
    updated_at: DateTime<Utc>,
}

impl RoundRobinState {
    fn new() -> Self {
        Self {
            current_weights: HashMap::new(),
            base_weights: HashMap::new(),
            last_selected: None,
            updated_at: Utc::now(),
        }
    }

    fn reconcile(&mut self, base_weights: &HashMap<Uuid, f64>) {
        self.current_weights
            .retain(|id, _| base_weights.contains_key(id));
        self.base_weights
            .retain(|id, _| base_weights.contains_key(id));

        for (id, weight) in base_weights {
            self.base_weights.insert(*id, *weight);
            self.current_weights.entry(*id).or_insert(0.0);
        }
    }

    fn select_next(
        &mut self,
        base_weights: &HashMap<Uuid, f64>,
        total_weight: f64,
    ) -> Result<(Uuid, f64)> {
        for (id, weight) in base_weights {
            let entry = self.current_weights.entry(*id).or_insert(0.0);
            *entry += *weight;
        }

        let mut selected_id: Option<Uuid> = None;
        let mut selected_current = f64::MIN;

        for (id, current) in &self.current_weights {
            if let Some(base) = base_weights.get(id) {
                if selected_id.is_none()
                    || *current > selected_current
                    || ((*current - selected_current).abs() < f64::EPSILON
                        && base > base_weights.get(&selected_id.unwrap()).unwrap_or(&0.0))
                {
                    selected_id = Some(*id);
                    selected_current = *current;
                }
            }
        }

        let selected_id =
            selected_id.ok_or_else(|| anyhow!("No round robin candidate available"))?;
        if let Some(entry) = self.current_weights.get_mut(&selected_id) {
            *entry -= total_weight;
        }

        Ok((selected_id, *base_weights.get(&selected_id).unwrap_or(&0.0)))
    }
}

#[derive(Clone)]
struct RoundRobinStateStore {
    state: Arc<Mutex<HashMap<String, RoundRobinState>>>,
    storage_path: PathBuf,
    persist: bool,
}

impl std::fmt::Debug for RoundRobinStateStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoundRobinStateStore")
            .field("storage_path", &self.storage_path)
            .field("persist", &self.persist)
            .finish()
    }
}

impl RoundRobinStateStore {
    fn with_default_path() -> Result<Self> {
        let path = std::env::var("TASK_ROUTER_STATE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".caws/runtime/task_router_round_robin.json"));
        Self::new(path)
    }

    fn new(path: PathBuf) -> Result<Self> {
        let snapshot = if path.exists() {
            Self::load_snapshot(&path)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            state: Arc::new(Mutex::new(snapshot)),
            storage_path: path,
            persist: true,
        })
    }

    fn in_memory() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            storage_path: PathBuf::new(),
            persist: false,
        }
    }

    fn load_snapshot(path: &Path) -> Result<HashMap<String, RoundRobinState>> {
        let data = fs::read(path)
            .with_context(|| format!("reading round robin state {}", path.display()))?;
        let snapshot = serde_json::from_slice(&data)
            .with_context(|| format!("parsing round robin state {}", path.display()))?;
        Ok(snapshot)
    }

    fn next_candidate(
        &self,
        key: &str,
        base_weights: &HashMap<Uuid, f64>,
    ) -> Result<(Uuid, Option<Uuid>, f64)> {
        if base_weights.is_empty() {
            bail!("No round robin weights provided");
        }

        let total_weight: f64 = base_weights.values().copied().sum();
        if total_weight <= f64::EPSILON {
            bail!("Total round robin weight is zero");
        }

        let mut guard = self.state.lock();
        let mut state = guard.get(key).cloned().unwrap_or_else(RoundRobinState::new);
        let previous = state.last_selected;
        state.reconcile(base_weights);
        let (selected_id, selected_weight) = state.select_next(base_weights, total_weight)?;
        state.last_selected = Some(selected_id);
        state.updated_at = Utc::now();
        guard.insert(key.to_string(), state);

        let snapshot = if self.persist {
            Some((*guard).clone())
        } else {
            None
        };
        drop(guard);

        if let Some(snapshot) = snapshot {
            if let Err(err) = self.write_snapshot(&snapshot) {
                warn!(error = ?err, "Failed to persist round robin state");
            }
        }

        Ok((selected_id, previous, selected_weight))
    }

    fn remove(&self, key: &str) {
        let mut guard = self.state.lock();
        let removed = guard.remove(key);
        let snapshot = if removed.is_some() && self.persist {
            Some((*guard).clone())
        } else {
            None
        };
        drop(guard);

        if let Some(snapshot) = snapshot {
            if let Err(err) = self.write_snapshot(&snapshot) {
                warn!(error = ?err, "Failed to persist round robin state after removal");
            }
        }
    }

    fn write_snapshot(&self, snapshot: &HashMap<String, RoundRobinState>) -> Result<()> {
        if !self.persist {
            return Ok(());
        }

        if let Some(parent) = self.storage_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "creating directory for round robin state {}",
                        parent.display()
                    )
                })?;
            }
        }

        let data = serde_json::to_vec_pretty(snapshot)?;
        fs::write(&self.storage_path, data).with_context(|| {
            format!(
                "writing round robin state to {}",
                self.storage_path.display()
            )
        })?;
        Ok(())
    }
}

/// Task router implementation
#[derive(Debug)]
pub struct TaskRouter {
    routing_algorithm: RoutingAlgorithm,
    capability_threshold: f32,
    load_balancing_strategy: LoadBalancingStrategy,
    round_robin_state: RoundRobinStateStore,
}

impl TaskRouter {
    /// Create a new task router
    pub fn new() -> Self {
        let round_robin_state = RoundRobinStateStore::with_default_path().unwrap_or_else(|err| {
            warn!(
                error = ?err,
                "Failed to initialize persistent round robin state; using in-memory store"
            );
            RoundRobinStateStore::in_memory()
        });

        Self {
            routing_algorithm: RoutingAlgorithm::Hybrid,
            capability_threshold: 0.7,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
            round_robin_state,
        }
    }

    /// Create a task router with configuration
    pub fn with_config(
        algorithm: RoutingAlgorithm,
        capability_threshold: f32,
        load_balancing: LoadBalancingStrategy,
    ) -> Self {
        let round_robin_state = RoundRobinStateStore::with_default_path().unwrap_or_else(|err| {
            warn!(
                error = ?err,
                "Failed to initialize persistent round robin state; using in-memory store"
            );
            RoundRobinStateStore::in_memory()
        });

        Self {
            routing_algorithm: algorithm,
            capability_threshold,
            load_balancing_strategy: load_balancing,
            round_robin_state,
        }
    }

    /// Create a task router with explicit round robin storage path
    pub fn with_round_robin_state(
        algorithm: RoutingAlgorithm,
        capability_threshold: f32,
        load_balancing: LoadBalancingStrategy,
        storage_path: PathBuf,
    ) -> Result<Self> {
        let round_robin_state = RoundRobinStateStore::new(storage_path)?;
        Ok(Self {
            routing_algorithm: algorithm,
            capability_threshold,
            load_balancing_strategy: load_balancing,
            round_robin_state,
        })
    }

    /// Route a task to appropriate workers
    pub async fn route_task(
        &self,
        task_spec: &TaskSpec,
        workers: &DashMap<Uuid, Worker>,
    ) -> Result<TaskRoutingResult> {
        info!("Routing task: {} ({})", task_spec.title, task_spec.id);

        // Convert task spec to requirements
        let requirements = self.task_spec_to_requirements(task_spec);

        // Get candidate workers
        let candidates = self.get_candidate_workers(&requirements, workers).await;

        if candidates.is_empty() {
            return Err(anyhow::anyhow!(
                "No suitable workers found for task requirements"
            ));
        }

        // Apply routing algorithm
        let selected_workers = match self.routing_algorithm {
            RoutingAlgorithm::CapabilityBased => {
                self.route_by_capability(&candidates, &requirements).await?
            }
            RoutingAlgorithm::LoadBalanced => {
                self.route_by_load_balancing(&candidates, &requirements)
                    .await?
            }
            RoutingAlgorithm::RoundRobin => {
                self.route_by_round_robin(&candidates, &requirements)
                    .await?
            }
            RoutingAlgorithm::LeastBusy => {
                self.route_by_least_busy(&candidates, &requirements).await?
            }
            RoutingAlgorithm::Hybrid => self.route_by_hybrid(&candidates, &requirements).await?,
        };

        // Calculate estimated completion time
        let estimated_completion =
            self.calculate_estimated_completion_time(&selected_workers, &requirements);

        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(&selected_workers, &requirements);

        let result = TaskRoutingResult {
            task_id: task_spec.id,
            selected_workers,
            routing_reasoning: format!(
                "Used {:?} algorithm with {:.1}% confidence",
                self.routing_algorithm,
                confidence_score * 100.0
            ),
            estimated_completion_time: estimated_completion,
            confidence_score,
        };

        debug!(
            "Task routing completed: {} workers selected",
            result.selected_workers.len()
        );
        Ok(result)
    }

    /// Convert task spec to requirements
    fn task_spec_to_requirements(&self, task_spec: &TaskSpec) -> TaskRequirements {
        // Extract languages from scope and context
        let mut required_languages = Vec::new();
        let mut required_frameworks = Vec::new();
        let required_domains = task_spec.scope.domains.clone();

        // Analyze task description and context for technology requirements
        let description = &task_spec.description.to_lowercase();
        let context_json = serde_json::to_string(&task_spec.context)
            .unwrap_or_default()
            .to_lowercase();

        // Detect programming languages
        if description.contains("rust") || context_json.contains("rust") {
            required_languages.push("rust".to_string());
        }
        if description.contains("typescript")
            || description.contains("javascript")
            || context_json.contains("typescript")
        {
            required_languages.push("typescript".to_string());
        }
        if description.contains("python") || context_json.contains("python") {
            required_languages.push("python".to_string());
        }
        if description.contains("go") || context_json.contains("golang") {
            required_languages.push("go".to_string());
        }

        // Detect frameworks
        if description.contains("react") || context_json.contains("react") {
            required_frameworks.push("react".to_string());
        }
        if description.contains("tokio") || context_json.contains("tokio") {
            required_frameworks.push("tokio".to_string());
        }
        if description.contains("django") || context_json.contains("django") {
            required_frameworks.push("django".to_string());
        }
        if description.contains("fastapi") || context_json.contains("fastapi") {
            required_frameworks.push("fastapi".to_string());
        }

        // Set minimum scores based on risk tier
        let (min_quality_score, min_caws_awareness) = match task_spec.risk_tier {
            RiskTier::Tier1 => (0.9, 0.95), // Critical tasks need high quality
            RiskTier::Tier2 => (0.8, 0.85), // Standard tasks need good quality
            RiskTier::Tier3 => (0.7, 0.75), // Low-risk tasks can be more lenient
        };

        // Estimate context length based on task complexity
        let context_length_estimate = self.estimate_context_length(task_spec);

        TaskRequirements {
            required_languages,
            required_frameworks,
            required_domains,
            min_quality_score,
            min_caws_awareness,
            max_execution_time_ms: task_spec.scope.max_loc.map(|loc| loc as u64 * 100), // Rough estimate
            preferred_worker_type: None, // Let router decide
            context_length_estimate,
        }
    }

    /// Get candidate workers that can handle the task
    async fn get_candidate_workers(
        &self,
        requirements: &TaskRequirements,
        workers: &DashMap<Uuid, Worker>,
    ) -> Vec<WorkerCandidate> {
        let mut candidates = Vec::new();

        for entry in workers.iter() {
            let worker = entry.value();

            // Check if worker can handle the task
            if worker.can_handle_task(requirements) {
                let capability_score = worker.calculate_capability_score(requirements);

                // Only include workers above threshold
                if capability_score >= self.capability_threshold {
                    let estimated_time = self.estimate_execution_time(worker, requirements);
                    let load_factor = self.calculate_load_factor(worker);

                    candidates.push(WorkerCandidate {
                        worker: worker.clone(),
                        capability_score,
                        estimated_execution_time_ms: estimated_time,
                        load_factor,
                        combined_score: self.calculate_combined_score(
                            capability_score,
                            estimated_time,
                            load_factor,
                        ),
                    });
                }
            }
        }

        // Sort by combined score (higher is better)
        candidates.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

        candidates
    }

    /// Route by capability matching (highest capability score wins)
    async fn route_by_capability(
        &self,
        candidates: &[WorkerCandidate],
        _requirements: &TaskRequirements,
    ) -> Result<Vec<WorkerAssignment>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Select the best candidate
        let best_candidate = &candidates[0];
        let assignment = WorkerAssignment {
            worker_id: best_candidate.worker.id,
            worker_name: best_candidate.worker.name.clone(),
            capability_match_score: best_candidate.capability_score,
            estimated_execution_time_ms: best_candidate.estimated_execution_time_ms,
            reasoning: format!(
                "Best capability match: {:.1}% (capabilities: {})",
                best_candidate.capability_score * 100.0,
                best_candidate.worker.capabilities.languages.join(", ")
            ),
            load_factor: best_candidate.load_factor,
        };

        Ok(vec![assignment])
    }

    /// Route by load balancing
    async fn route_by_load_balancing(
        &self,
        candidates: &[WorkerCandidate],
        _requirements: &TaskRequirements,
    ) -> Result<Vec<WorkerAssignment>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Find worker with lowest load
        let best_candidate = candidates
            .iter()
            .min_by(|a, b| a.load_factor.partial_cmp(&b.load_factor).unwrap())
            .unwrap();

        let assignment = WorkerAssignment {
            worker_id: best_candidate.worker.id,
            worker_name: best_candidate.worker.name.clone(),
            capability_match_score: best_candidate.capability_score,
            estimated_execution_time_ms: best_candidate.estimated_execution_time_ms,
            reasoning: format!(
                "Load balanced selection: {:.1}% load (capability: {:.1}%)",
                best_candidate.load_factor * 100.0,
                best_candidate.capability_score * 100.0
            ),
            load_factor: best_candidate.load_factor,
        };

        Ok(vec![assignment])
    }

    /// Route by round robin
    async fn route_by_round_robin(
        &self,
        candidates: &[WorkerCandidate],
        requirements: &TaskRequirements,
    ) -> Result<Vec<WorkerAssignment>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let mut eligible: Vec<WorkerCandidate> = candidates
            .iter()
            .filter(|candidate| matches!(candidate.worker.status, WorkerStatus::Available))
            .cloned()
            .collect();

        if eligible.is_empty() {
            eligible = candidates.to_vec();
        }

        let key = self.compute_round_robin_key(requirements, &eligible);
        let mut weights = HashMap::new();
        for candidate in &eligible {
            let weight = Self::compute_round_robin_weight(candidate);
            weights.insert(candidate.worker.id, weight);
        }

        let fallback = eligible
            .iter()
            .min_by(|a, b| {
                a.load_factor
                    .partial_cmp(&b.load_factor)
                    .unwrap_or(Ordering::Equal)
            })
            .cloned()
            .unwrap();

        let positive_weight = weights.values().any(|w| *w > f64::EPSILON);
        let (selected, selected_weight, previous) = if positive_weight {
            match self.round_robin_state.next_candidate(&key, &weights) {
                Ok((selected_id, previous, weight)) => {
                    if let Some(candidate) = eligible
                        .iter()
                        .find(|c| c.worker.id == selected_id)
                        .cloned()
                    {
                        (candidate, weight, previous)
                    } else {
                        // Candidate disappeared since last selection – reset state and fall back.
                        self.round_robin_state.remove(&key);
                        (
                            fallback.clone(),
                            weights
                                .get(&fallback.worker.id)
                                .copied()
                                .unwrap_or_default(),
                            None,
                        )
                    }
                }
                Err(err) => {
                    warn!(
                        error = ?err,
                        "Round robin state unavailable; falling back to load-based selection"
                    );
                    self.round_robin_state.remove(&key);
                    (
                        fallback.clone(),
                        weights
                            .get(&fallback.worker.id)
                            .copied()
                            .unwrap_or_default(),
                        None,
                    )
                }
            }
        } else {
            self.round_robin_state.remove(&key);
            (fallback.clone(), 0.0, None)
        };

        let previous_name = previous.and_then(|id| {
            eligible
                .iter()
                .find(|candidate| candidate.worker.id == id)
                .map(|candidate| candidate.worker.name.clone())
        });

        let reasoning = format!(
            "Round robin selection: weight {:.3}, load {:.1}%, capability {:.1}%{}",
            selected_weight,
            selected.load_factor * 100.0,
            selected.capability_score * 100.0,
            previous_name
                .map(|name| format!(" (previous: {name})"))
                .unwrap_or_default()
        );

        let assignment = WorkerAssignment {
            worker_id: selected.worker.id,
            worker_name: selected.worker.name.clone(),
            capability_match_score: selected.capability_score,
            estimated_execution_time_ms: selected.estimated_execution_time_ms,
            reasoning,
            load_factor: selected.load_factor,
        };

        Ok(vec![assignment])
    }

    fn compute_round_robin_key(
        &self,
        requirements: &TaskRequirements,
        candidates: &[WorkerCandidate],
    ) -> String {
        let mut ids: Vec<String> = candidates
            .iter()
            .map(|candidate| candidate.worker.id.to_string())
            .collect();
        ids.sort();

        let preferred = requirements
            .preferred_worker_type
            .as_ref()
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|| "any".to_string());

        format!(
            "{}|{}|{}|{}|{}",
            requirements.required_languages.join(","),
            requirements.required_frameworks.join(","),
            requirements.required_domains.join(","),
            preferred,
            ids.join(",")
        )
    }

    fn compute_round_robin_weight(candidate: &WorkerCandidate) -> f64 {
        let capability = candidate.capability_score.clamp(0.0, 1.0) as f64;
        let capacity = (1.0 - candidate.load_factor).clamp(0.0, 1.0) as f64;
        let efficiency =
            1.0 / (1.0 + (candidate.estimated_execution_time_ms as f64 / 30_000.0).max(0.0));
        let health = match candidate.worker.health_status {
            WorkerHealthStatus::Healthy => 1.0,
            WorkerHealthStatus::Degraded => 0.6,
            WorkerHealthStatus::Unhealthy => 0.2,
        };

        // Weighted blend that favors capability while accounting for capacity, efficiency, and health
        let weight = capability * 0.5 + capacity * 0.3 + efficiency * 0.15 + health * 0.05;
        weight.max(0.01)
    }

    /// Route by least busy worker
    async fn route_by_least_busy(
        &self,
        candidates: &[WorkerCandidate],
        _requirements: &TaskRequirements,
    ) -> Result<Vec<WorkerAssignment>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Find worker with lowest current load
        let best_candidate = candidates
            .iter()
            .min_by(|a, b| {
                a.worker
                    .performance_metrics
                    .current_load
                    .partial_cmp(&b.worker.performance_metrics.current_load)
                    .unwrap()
            })
            .unwrap();

        let assignment = WorkerAssignment {
            worker_id: best_candidate.worker.id,
            worker_name: best_candidate.worker.name.clone(),
            capability_match_score: best_candidate.capability_score,
            estimated_execution_time_ms: best_candidate.estimated_execution_time_ms,
            reasoning: format!(
                "Least busy worker: {:.1}% current load",
                best_candidate.worker.performance_metrics.current_load * 100.0
            ),
            load_factor: best_candidate.load_factor,
        };

        Ok(vec![assignment])
    }

    /// Route using hybrid algorithm (capability + load balancing)
    async fn route_by_hybrid(
        &self,
        candidates: &[WorkerCandidate],
        _requirements: &TaskRequirements,
    ) -> Result<Vec<WorkerAssignment>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Use combined score for selection
        let best_candidate = &candidates[0]; // Already sorted by combined score

        let assignment = WorkerAssignment {
            worker_id: best_candidate.worker.id,
            worker_name: best_candidate.worker.name.clone(),
            capability_match_score: best_candidate.capability_score,
            estimated_execution_time_ms: best_candidate.estimated_execution_time_ms,
            reasoning: format!(
                "Hybrid selection: {:.1}% combined score (capability: {:.1}%, load: {:.1}%)",
                best_candidate.combined_score * 100.0,
                best_candidate.capability_score * 100.0,
                best_candidate.load_factor * 100.0
            ),
            load_factor: best_candidate.load_factor,
        };

        Ok(vec![assignment])
    }

    /// Estimate context length for a task
    fn estimate_context_length(&self, task_spec: &TaskSpec) -> u32 {
        let base_length = 2000; // Base context length

        // Add length based on scope
        let scope_length = task_spec.scope.files_affected.len() as u32 * 500;

        // Add length based on description complexity
        let description_length = task_spec.description.len() as u32;

        // Add length based on risk tier
        let risk_multiplier = match task_spec.risk_tier {
            RiskTier::Tier1 => 2.0,
            RiskTier::Tier2 => 1.5,
            RiskTier::Tier3 => 1.0,
        };

        ((base_length + scope_length + description_length) as f32 * risk_multiplier) as u32
    }

    /// Estimate execution time for a worker and task
    fn estimate_execution_time(&self, worker: &Worker, requirements: &TaskRequirements) -> u64 {
        let base_time = 5000; // 5 seconds base time

        // Adjust based on worker speed score
        let speed_factor = 1.0 / (worker.capabilities.speed_score + 0.1); // Avoid division by zero

        // Adjust based on context length
        let context_factor = (requirements.context_length_estimate as f64 / 4000.0).max(0.5);

        // Adjust based on number of requirements
        let complexity_factor = 1.0
            + (requirements.required_languages.len() + requirements.required_frameworks.len())
                as f64
                * 0.1;

        (base_time as f64 * speed_factor as f64 * context_factor as f64 * complexity_factor as f64)
            as u64
    }

    /// Calculate load factor for a worker
    fn calculate_load_factor(&self, worker: &Worker) -> f32 {
        // Combine current load with historical performance
        let current_load = worker.performance_metrics.current_load;
        let recent_tasks = worker.performance_metrics.total_tasks.min(10) as f32;
        let busy_factor = if recent_tasks > 0.0 {
            worker.performance_metrics.total_tasks as f32 / recent_tasks
        } else {
            0.0
        };

        (current_load * 0.7 + busy_factor * 0.3).min(1.0)
    }

    /// Calculate combined score for worker selection
    fn calculate_combined_score(
        &self,
        capability_score: f32,
        estimated_time: u64,
        load_factor: f32,
    ) -> f32 {
        // Normalize execution time (shorter is better)
        let time_score = 1.0 / (estimated_time as f32 / 10000.0 + 0.1);

        // Invert load factor (lower load is better)
        let load_score = 1.0 - load_factor;

        // Weighted combination
        capability_score * 0.5 + time_score * 0.3 + load_score * 0.2
    }

    /// Calculate estimated completion time
    fn calculate_estimated_completion_time(
        &self,
        assignments: &[WorkerAssignment],
        _requirements: &TaskRequirements,
    ) -> chrono::DateTime<chrono::Utc> {
        if assignments.is_empty() {
            return chrono::Utc::now();
        }

        let max_time = assignments
            .iter()
            .map(|a| a.estimated_execution_time_ms)
            .max()
            .unwrap_or(0);

        chrono::Utc::now() + chrono::Duration::milliseconds(max_time as i64)
    }

    /// Calculate confidence score for the routing decision
    fn calculate_confidence_score(
        &self,
        assignments: &[WorkerAssignment],
        requirements: &TaskRequirements,
    ) -> f32 {
        if assignments.is_empty() {
            return 0.0;
        }

        let best_assignment = &assignments[0];

        // Base confidence on capability match
        let capability_confidence = best_assignment.capability_match_score;

        // Adjust based on number of candidates (more candidates = higher confidence)
        let availability_confidence = (assignments.len() as f32 / 5.0).min(1.0);

        // Adjust based on load factor (lower load = higher confidence)
        let load_confidence = 1.0 - best_assignment.load_factor;

        (capability_confidence * 0.6 + availability_confidence * 0.2 + load_confidence * 0.2)
            .min(1.0)
    }
}

impl Default for TaskRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Worker candidate for routing
#[derive(Debug, Clone)]
struct WorkerCandidate {
    worker: Worker,
    capability_score: f32,
    estimated_execution_time_ms: u64,
    load_factor: f32,
    combined_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_task_router_creation() {
        let router = TaskRouter::new();
        assert_eq!(router.routing_algorithm, RoutingAlgorithm::Hybrid);
        assert_eq!(router.capability_threshold, 0.7);
    }

    #[tokio::test]
    async fn test_task_spec_to_requirements() {
        let router = TaskRouter::new();

        let task_spec = TaskSpec {
            id: Uuid::new_v4(),
            title: "Implement Rust API".to_string(),
            description: "Create a REST API in Rust with tokio".to_string(),
            risk_tier: RiskTier::Tier2,
            scope: TaskScope {
                files_affected: vec!["src/api.rs".to_string()],
                max_files: Some(5),
                max_loc: Some(1000),
                domains: vec!["backend".to_string()],
            },
            acceptance_criteria: vec![],
            context: CouncilTaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: ConfigEnvironment::Development,
            },
            worker_output: CouncilWorkerOutput {
                content: "".to_string(),
                files_modified: vec![],
                rationale: "".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.0,
                    quality_score: 0.0,
                    confidence: 0.0,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: std::collections::HashMap::new(),
            },
            caws_spec: None,
        };

        let requirements = router.task_spec_to_requirements(&task_spec);

        assert!(requirements
            .required_languages
            .contains(&"rust".to_string()));
        assert!(requirements
            .required_frameworks
            .contains(&"tokio".to_string()));
        assert_eq!(requirements.min_quality_score, 0.8); // Tier2
        assert_eq!(requirements.min_caws_awareness, 0.85); // Tier2
    }

    #[tokio::test]
    async fn test_estimate_context_length() {
        let router = TaskRouter::new();

        let task_spec = TaskSpec {
            id: Uuid::new_v4(),
            title: "Test task".to_string(),
            description: "A simple test task".to_string(),
            risk_tier: RiskTier::Tier2,
            scope: TaskScope {
                files_affected: vec!["file1.rs".to_string(), "file2.rs".to_string()],
                max_files: Some(5),
                max_loc: Some(1000),
                domains: vec!["backend".to_string()],
            },
            acceptance_criteria: vec![],
            context: CouncilTaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: ConfigEnvironment::Development,
            },
            worker_output: CouncilWorkerOutput {
                content: "".to_string(),
                files_modified: vec![],
                rationale: "".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.0,
                    quality_score: 0.0,
                    confidence: 0.0,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: std::collections::HashMap::new(),
            },
            caws_spec: None,
        };

        let context_length = router.estimate_context_length(&task_spec);
        assert!(context_length > 2000); // Should be more than base length
    }

    #[tokio::test]
    async fn test_route_by_capability() {
        let router = TaskRouter::new();

        let mut capabilities = WorkerCapabilities::default();
        capabilities.languages = vec!["rust".to_string()];
        capabilities.quality_score = 0.9;

        let worker = Worker::new(
            "test-worker".to_string(),
            WorkerType::Generalist,
            "llama3.3:7b".to_string(),
            "http://localhost:11434".to_string(),
            capabilities,
        );

        let requirements = TaskRequirements {
            required_languages: vec!["rust".to_string()],
            required_frameworks: vec![],
            required_domains: vec![],
            min_quality_score: 0.8,
            min_caws_awareness: 0.7,
            max_execution_time_ms: Some(30000),
            preferred_worker_type: None,
            context_length_estimate: 4000,
        };

        let candidates = vec![WorkerCandidate {
            worker,
            capability_score: 0.9,
            estimated_execution_time_ms: 5000,
            load_factor: 0.3,
            combined_score: 0.8,
        }];

        let assignments = router
            .route_by_capability(&candidates, &requirements)
            .await
            .unwrap();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].capability_match_score, 0.9);
    }

    fn make_candidate(name: &str, capability: f32, load: f32) -> WorkerCandidate {
        let mut capabilities = WorkerCapabilities::default();
        capabilities.languages = vec!["rust".to_string()];
        capabilities.quality_score = capability;
        capabilities.caws_awareness = capability;

        let mut worker = Worker::new(
            name.to_string(),
            WorkerType::Generalist,
            "llama3.3:7b".to_string(),
            "http://localhost:11434".to_string(),
            capabilities,
        );
        worker.performance_metrics.current_load = load;
        worker.health_status = WorkerHealthStatus::Healthy;
        worker.health_metrics = None;
        worker.last_health_check = Some(chrono::Utc::now());

        WorkerCandidate {
            worker,
            capability_score: capability,
            estimated_execution_time_ms: 5000,
            load_factor: load,
            combined_score: capability * (1.0 - load),
        }
    }

    fn basic_requirements() -> TaskRequirements {
        TaskRequirements {
            required_languages: vec!["rust".to_string()],
            required_frameworks: vec![],
            required_domains: vec![],
            min_quality_score: 0.6,
            min_caws_awareness: 0.6,
            max_execution_time_ms: None,
            preferred_worker_type: None,
            context_length_estimate: 2048,
        }
    }

    #[tokio::test]
    async fn test_round_robin_rotates_workers() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path().join("rr_state.json");
        let router = TaskRouter::with_round_robin_state(
            RoutingAlgorithm::RoundRobin,
            0.7,
            LoadBalancingStrategy::ResourceBased,
            storage_path.clone(),
        )
        .expect("router");

        let candidate_a = make_candidate("worker-a", 0.8, 0.2);
        let candidate_b = make_candidate("worker-b", 0.8, 0.2);
        let candidates = vec![candidate_a.clone(), candidate_b.clone()];
        let requirements = basic_requirements();

        let first = router
            .route_by_round_robin(&candidates, &requirements)
            .await
            .expect("first selection");
        let second = router
            .route_by_round_robin(&candidates, &requirements)
            .await
            .expect("second selection");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(
            first[0].worker_id, second[0].worker_id,
            "Round robin should rotate between workers"
        );
    }

    #[tokio::test]
    async fn test_weighted_round_robin_prefers_high_weight() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path().join("rr_state_weighted.json");
        let router = TaskRouter::with_round_robin_state(
            RoutingAlgorithm::RoundRobin,
            0.7,
            LoadBalancingStrategy::ResourceBased,
            storage_path.clone(),
        )
        .expect("router");

        let strong = make_candidate("strong-worker", 0.95, 0.1);
        let weaker = make_candidate("support-worker", 0.75, 0.2);
        let candidates = vec![strong.clone(), weaker.clone()];
        let requirements = basic_requirements();

        let mut counts = std::collections::HashMap::new();
        for _ in 0..6 {
            let assignment = router
                .route_by_round_robin(&candidates, &requirements)
                .await
                .expect("selection");
            let id = assignment[0].worker_id;
            *counts.entry(id).or_insert(0usize) += 1;
        }

        let strong_count = counts.get(&strong.worker.id).copied().unwrap_or(0);
        let weak_count = counts.get(&weaker.worker.id).copied().unwrap_or(0);
        assert!(
            strong_count > weak_count,
            "Higher capability worker should be selected more often (strong: {}, weak: {})",
            strong_count,
            weak_count
        );
        assert!(
            weak_count > 0,
            "Lower weight worker should still receive assignments occasionally"
        );
    }

    #[tokio::test]
    async fn test_round_robin_state_persists_across_instances() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path().join("rr_state_persist.json");

        let router_one = TaskRouter::with_round_robin_state(
            RoutingAlgorithm::RoundRobin,
            0.7,
            LoadBalancingStrategy::ResourceBased,
            storage_path.clone(),
        )
        .expect("router one");

        let candidate_a = make_candidate("worker-a", 0.8, 0.1);
        let candidate_b = make_candidate("worker-b", 0.8, 0.1);
        let candidates = vec![candidate_a.clone(), candidate_b.clone()];
        let requirements = basic_requirements();

        let first = router_one
            .route_by_round_robin(&candidates, &requirements)
            .await
            .expect("first selection");
        let first_worker = first[0].worker_id;

        // Drop router_one and create a new router with same storage
        drop(router_one);
        let router_two = TaskRouter::with_round_robin_state(
            RoutingAlgorithm::RoundRobin,
            0.7,
            LoadBalancingStrategy::ResourceBased,
            storage_path.clone(),
        )
        .expect("router two");

        let second = router_two
            .route_by_round_robin(&candidates, &requirements)
            .await
            .expect("second selection");
        let second_worker = second[0].worker_id;

        assert_ne!(
            first_worker, second_worker,
            "Round robin state should persist and continue rotation after restart"
        );
    }
}
