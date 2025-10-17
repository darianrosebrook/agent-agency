//! Task Router
//!
//! Routes tasks to appropriate workers based on capabilities, load, and other factors.

use crate::types::*;
use crate::{RoutingAlgorithm, LoadBalancingStrategy};
use agent_agency_council::types::RiskTier;
use agent_agency_council::models::TaskSpec;
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Task router implementation
#[derive(Debug)]
pub struct TaskRouter {
    routing_algorithm: RoutingAlgorithm,
    capability_threshold: f32,
    load_balancing_strategy: LoadBalancingStrategy,
}

impl TaskRouter {
    /// Create a new task router
    pub fn new() -> Self {
        Self {
            routing_algorithm: RoutingAlgorithm::Hybrid,
            capability_threshold: 0.7,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
        }
    }

    /// Create a task router with configuration
    pub fn with_config(
        algorithm: RoutingAlgorithm,
        capability_threshold: f32,
        load_balancing: LoadBalancingStrategy,
    ) -> Self {
        Self {
            routing_algorithm: algorithm,
            capability_threshold,
            load_balancing_strategy: load_balancing,
        }
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
            return Err(anyhow::anyhow!("No suitable workers found for task requirements"));
        }

        // Apply routing algorithm
        let selected_workers = match self.routing_algorithm {
            RoutingAlgorithm::CapabilityBased => {
                self.route_by_capability(&candidates, &requirements).await?
            }
            RoutingAlgorithm::LoadBalanced => {
                self.route_by_load_balancing(&candidates, &requirements).await?
            }
            RoutingAlgorithm::RoundRobin => {
                self.route_by_round_robin(&candidates, &requirements).await?
            }
            RoutingAlgorithm::LeastBusy => {
                self.route_by_least_busy(&candidates, &requirements).await?
            }
            RoutingAlgorithm::Hybrid => {
                self.route_by_hybrid(&candidates, &requirements).await?
            }
        };

        // Calculate estimated completion time
        let estimated_completion = self.calculate_estimated_completion_time(&selected_workers, &requirements);

        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(&selected_workers, &requirements);

        let result = TaskRoutingResult {
            task_id: task_spec.id,
            selected_workers,
            routing_reasoning: format!("Used {:?} algorithm with {:.1}% confidence", 
                self.routing_algorithm, confidence_score * 100.0),
            estimated_completion_time: estimated_completion,
            confidence_score,
        };

        debug!("Task routing completed: {} workers selected", result.selected_workers.len());
        Ok(result)
    }

    /// Convert task spec to requirements
    fn task_spec_to_requirements(&self, task_spec: &TaskSpec) -> TaskRequirements {
        // Extract languages from scope and context
        let mut required_languages = Vec::new();
        let mut required_frameworks = Vec::new();
        let mut required_domains = task_spec.scope.domains.clone();

        // Analyze task description and context for technology requirements
        let description = &task_spec.description.to_lowercase();
        let context_json = serde_json::to_string(&task_spec.context).unwrap_or_default().to_lowercase();

        // Detect programming languages
        if description.contains("rust") || context_json.contains("rust") {
            required_languages.push("rust".to_string());
        }
        if description.contains("typescript") || description.contains("javascript") || context_json.contains("typescript") {
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
                            load_factor
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
        let best_candidate = candidates.iter()
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
        _requirements: &TaskRequirements,
    ) -> Result<Vec<WorkerAssignment>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // TODO: Implement actual round robin with persistent state
        // For now, just select the first candidate
        let candidate = &candidates[0];
        let assignment = WorkerAssignment {
            worker_id: candidate.worker.id,
            worker_name: candidate.worker.name.clone(),
            capability_match_score: candidate.capability_score,
            estimated_execution_time_ms: candidate.estimated_execution_time_ms,
            reasoning: "Round robin selection (first available)".to_string(),
            load_factor: candidate.load_factor,
        };

        Ok(vec![assignment])
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
        let best_candidate = candidates.iter()
            .min_by(|a, b| a.worker.performance_metrics.current_load
                .partial_cmp(&b.worker.performance_metrics.current_load)
                .unwrap())
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
        let complexity_factor = 1.0 + (requirements.required_languages.len() + 
                                      requirements.required_frameworks.len()) as f64 * 0.1;

        (base_time as f64 * speed_factor * context_factor * complexity_factor) as u64
    }

    /// Calculate load factor for a worker
    fn calculate_load_factor(&self, worker: &Worker) -> f32 {
        // Combine current load with historical performance
        let current_load = worker.performance_metrics.current_load;
        let recent_tasks = worker.performance_metrics.total_tasks.min(10) as f32;
        let busy_factor = if recent_tasks > 0 {
            worker.performance_metrics.busy_workers as f32 / recent_tasks
        } else {
            0.0
        };

        (current_load * 0.7 + busy_factor * 0.3).min(1.0)
    }

    /// Calculate combined score for worker selection
    fn calculate_combined_score(&self, capability_score: f32, estimated_time: u64, load_factor: f32) -> f32 {
        // Normalize execution time (shorter is better)
        let time_score = 1.0 / (estimated_time as f32 / 10000.0 + 0.1);
        
        // Invert load factor (lower load is better)
        let load_score = 1.0 - load_factor;
        
        // Weighted combination
        capability_score * 0.5 + time_score * 0.3 + load_score * 0.2
    }

    /// Calculate estimated completion time
    fn calculate_estimated_completion_time(&self, assignments: &[WorkerAssignment], _requirements: &TaskRequirements) -> chrono::DateTime<chrono::Utc> {
        if assignments.is_empty() {
            return chrono::Utc::now();
        }

        let max_time = assignments.iter()
            .map(|a| a.estimated_execution_time_ms)
            .max()
            .unwrap_or(0);

        chrono::Utc::now() + chrono::Duration::milliseconds(max_time as i64)
    }

    /// Calculate confidence score for the routing decision
    fn calculate_confidence_score(&self, assignments: &[WorkerAssignment], requirements: &TaskRequirements) -> f32 {
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
        
        (capability_confidence * 0.6 + availability_confidence * 0.2 + load_confidence * 0.2).min(1.0)
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
            context: TaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            worker_output: WorkerOutput {
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
        
        assert!(requirements.required_languages.contains(&"rust".to_string()));
        assert!(requirements.required_frameworks.contains(&"tokio".to_string()));
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
            context: TaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            worker_output: WorkerOutput {
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

        let candidates = vec![
            WorkerCandidate {
                worker,
                capability_score: 0.9,
                estimated_execution_time_ms: 5000,
                load_factor: 0.3,
                combined_score: 0.8,
            }
        ];

        let assignments = router.route_by_capability(&candidates, &requirements).await.unwrap();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].capability_match_score, 0.9);
    }
}
