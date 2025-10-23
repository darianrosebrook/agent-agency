//! Integration utilities for connecting parallel workers with other v3 systems

use crate::{ComplexTask, ParallelCoordinatorConfig};
use std::path::PathBuf;

/// Determine if a task should be routed to parallel execution
pub fn should_route_to_parallel(
    description: &str,
    complexity_score: f32,
    config: &ParallelCoordinatorConfig,
) -> bool {
    // Check if parallel execution is enabled
    if !config.enabled {
        return false;
    }

    // Recalculate complexity including keyword analysis for routing decision
    let keyword_boost = estimate_parallelization_benefit(description, None);
    let effective_complexity = complexity_score.max(keyword_boost);

    // Check if effective complexity exceeds threshold
    if effective_complexity < config.complexity_threshold {
        return false;
    }

    // Check for keywords that indicate good parallelization candidates
    let parallel_keywords = [
        "fix", "error", "bug", "refactor", "multiple", "complex",
        "large", "scale", "optimize", "parallel", "concurrent",
    ];

    let desc_lower = description.to_lowercase();
    let keyword_count = parallel_keywords
        .iter()
        .filter(|&keyword| desc_lower.contains(keyword))
        .count();

    // Route to parallel if complexity is high OR multiple parallel keywords found
    complexity_score >= config.complexity_threshold || keyword_count >= 2
}

/// Estimate the parallelization benefit (0.0-1.0 scale)
pub fn estimate_parallelization_benefit(description: &str, _config: Option<&ParallelCoordinatorConfig>) -> f32 {
    let desc_lower = description.to_lowercase();

    // Base benefit from error-related tasks (highly parallelizable)
    let mut benefit = if desc_lower.contains("error") || desc_lower.contains("fix") || desc_lower.contains("bug") {
        0.8
    } else {
        0.4
    };

    // Additional benefit from complexity indicators
    let complexity_indicators = [
        "refactor", "multiple", "complex", "large", "scale", "optimize",
        "parallel", "concurrent", "many", "several", "various",
    ];

    let indicator_count = complexity_indicators
        .iter()
        .filter(|&indicator| desc_lower.contains(indicator))
        .count() as f32;

    benefit += (indicator_count / 5.0).min(0.3); // Max 0.3 from indicators

    // Length-based complexity (longer descriptions tend to be more complex)
    let length_factor = (description.len() as f32 / 1000.0).min(0.2);

    benefit += length_factor;

    benefit.min(1.0)
}

/// Convert a task description into a ComplexTask for parallel execution
pub fn convert_to_complex_task(description: String, workspace_root: PathBuf) -> ComplexTask {
    // Calculate basic complexity score
    let complexity_score = estimate_parallelization_benefit(&description, None);

    ComplexTask {
        id: crate::TaskId::new(),
        description,
        context: crate::TaskContext {
            working_directory: workspace_root,
            environment_variables: std::env::vars().collect(),
            timeout: Some(std::time::Duration::from_secs(300)),
        },
        complexity_score,
        estimated_subtasks: Some((complexity_score * 5.0).max(2.0) as usize), // Rough estimate
        scope: crate::TaskScope::default(),
        quality_requirements: crate::QualityRequirements::default(),
    }
}

/// Stub implementation for orchestrator handle integration
/// TODO: Replace with actual OrchestratorHandle when orchestration integration is complete
pub trait OrchestratorHandle: Send + Sync {
    fn execute_sequential(&self, task: ComplexTask) -> impl std::future::Future<Output = Result<crate::TaskResult, crate::ParallelError>> + Send;
}

/// Stub implementation for testing
pub struct StubOrchestratorHandle;

impl OrchestratorHandle for StubOrchestratorHandle {
    async fn execute_sequential(&self, _task: ComplexTask) -> Result<crate::TaskResult, crate::ParallelError> {
        Err(crate::ParallelError::Worker(crate::WorkerError::NotImplemented(
            "Sequential execution via orchestrator not yet integrated".to_string()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_route_to_parallel() {
        let config = ParallelCoordinatorConfig {
            enabled: true,
            max_concurrent_workers: 8,
            max_subtasks_per_task: 20,
            task_timeout_seconds: 300,
            complexity_threshold: 0.6,
            enable_quality_gates: true,
            enable_dependency_resolution: true,
        };

        // High complexity should route to parallel
        assert!(should_route_to_parallel("Fix 50 compilation errors across multiple files", 0.8, &config));

        // Low complexity should not route
        assert!(!should_route_to_parallel("Simple hello world task", 0.3, &config));

        // Parallel keywords should route even with moderate complexity
        assert!(should_route_to_parallel("Refactor multiple modules", 0.5, &config));

        // Disabled config should not route
        let disabled_config = ParallelCoordinatorConfig { enabled: false, ..config };
        assert!(!should_route_to_parallel("Fix 50 compilation errors", 0.8, &disabled_config));
    }

    #[test]
    fn test_estimate_parallelization_benefit() {
        // Error-related tasks should have high benefit
        assert!(estimate_parallelization_benefit("Fix compilation errors", None) > 0.7);

        // Complex refactoring should have moderate-high benefit
        assert!(estimate_parallelization_benefit("Refactor complex system", None) > 0.5);

        // Simple tasks should have lower benefit
        assert!(estimate_parallelization_benefit("Print hello world", None) < 0.5);
    }
}
