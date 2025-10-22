//! Task decomposition engine and pattern analysis

pub mod analyzer;
pub mod dependency_graph;
pub mod strategies;

pub use analyzer::*;
pub use dependency_graph::*;
pub use strategies::*;

// Re-export types from types module that are used in decomposition
pub use crate::types::{ComplexTask, TaskAnalysis, TaskPattern, Dependency, SubtaskScores, SubTask, TaskId, SubTaskId, TaskScope, QualityRequirements, WorkerSpecialty, Priority};

// Re-export from progress module
pub use crate::progress::WorkerStatus;

// Re-export error types
pub use crate::error::DecompositionError;

/// Main decomposition engine that coordinates all decomposition activities
pub struct DecompositionEngine {
    pattern_recognizer: PatternRecognizer,
    dependency_analyzer: DependencyAnalyzer,
    complexity_scorer: ComplexityScorer,
}

impl DecompositionEngine {
    pub fn new() -> Self {
        Self {
            pattern_recognizer: PatternRecognizer::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            complexity_scorer: ComplexityScorer::new(),
        }
    }

    /// Analyze task to determine decomposition strategy
    pub async fn analyze(
        &self,
        task: &ComplexTask,
    ) -> Result<TaskAnalysis, DecompositionError> {
        // Identify problem patterns
        let patterns = self.pattern_recognizer.identify_patterns(task)?;

        // Map dependencies between potential subtasks
        let dependencies = self.dependency_analyzer.analyze(task)?;

        // Score decomposition opportunities
        let subtask_scores = self.complexity_scorer.score_subtasks(task, &patterns)?;

        let recommended_workers = subtask_scores.complexity_scores.len().min(8); // Cap at 8 workers
        let should_parallelize = subtask_scores.parallelization_score > 0.6;

        // Validate decomposition strategy with council (if available)
        // TODO: Integrate with council for consensus validation of decomposition strategy
        // This would involve:
        // 1. Creating a council task spec from the analysis
        // 2. Getting council consensus on the decomposition approach
        // 3. Adjusting recommended_workers based on council feedback

        Ok(TaskAnalysis {
            patterns,
            dependencies,
            subtask_scores,
            recommended_workers,
            should_parallelize,
        })
    }

    /// Create optimized subtasks from analysis
    pub fn decompose(
        &self,
        analysis: TaskAnalysis,
    ) -> Result<Vec<SubTask>, DecompositionError> {
        // For now, create a simple decomposition based on patterns
        // TODO: Implement proper strategy-based decomposition
        let mut all_subtasks = Vec::new();

        // Create subtasks based on pattern types
        for pattern in &analysis.patterns {
            match pattern {
                TaskPattern::CompilationErrors { error_groups } => {
                    for error_group in error_groups {
                        let subtask = SubTask {
                            id: SubTaskId::new(),
                            parent_id: TaskId::new(), // TODO: Pass actual task ID
                            title: format!("Fix {} errors", error_group.error_code),
                            description: format!("Resolve {} compilation errors", error_group.count),
                            scope: TaskScope {
                                included_files: error_group.affected_files.clone(),
                                excluded_files: vec![],
                                included_patterns: vec![],
                                excluded_patterns: vec![],
                                time_budget: std::time::Duration::from_secs(300),
                                quality_requirements: QualityRequirements::default(),
                            },
                            specialty: WorkerSpecialty::CompilationErrors {
                                error_codes: vec![error_group.error_code.clone()],
                            },
                            dependencies: vec![],
                            estimated_effort: std::time::Duration::from_secs(180),
                            priority: Priority::High,
                        };
                        all_subtasks.push(subtask);
                    }
                }
                TaskPattern::RefactoringOperations { operations } => {
                    for operation in operations {
                        let subtask = SubTask {
                            id: SubTaskId::new(),
                            parent_id: TaskId::new(),
                            title: operation.operation_type.clone(),
                            description: format!("Perform {} refactoring", operation.operation_type),
                            scope: TaskScope {
                                included_files: operation.affected_files.clone(),
                                excluded_files: vec![],
                                included_patterns: vec![],
                                excluded_patterns: vec![],
                                time_budget: std::time::Duration::from_secs(300),
                                quality_requirements: QualityRequirements::default(),
                            },
                            specialty: WorkerSpecialty::Refactoring {
                                strategies: vec![operation.operation_type.clone()],
                            },
                            dependencies: vec![],
                            estimated_effort: std::time::Duration::from_secs(
                                (operation.complexity * 300.0) as u64
                            ),
                            priority: Priority::Medium,
                        };
                        all_subtasks.push(subtask);
                    }
                }
                TaskPattern::TestingGaps { missing_tests } => {
                    for test_gap in missing_tests {
                        let subtask = SubTask {
                            id: SubTaskId::new(),
                            parent_id: TaskId::new(),
                            title: "Add tests".to_string(),
                            description: test_gap.clone(),
                            scope: TaskScope {
                                included_files: vec![],
                                excluded_files: vec![],
                                included_patterns: vec!["*.rs".to_string()],
                                excluded_patterns: vec![],
                                time_budget: std::time::Duration::from_secs(300),
                                quality_requirements: QualityRequirements {
                                    min_test_coverage: Some(0.8),
                                    ..QualityRequirements::default()
                                },
                            },
                            specialty: WorkerSpecialty::Testing {
                                frameworks: vec!["rust".to_string()],
                            },
                            dependencies: vec![],
                            estimated_effort: std::time::Duration::from_secs(180),
                            priority: Priority::High,
                        };
                        all_subtasks.push(subtask);
                    }
                }
                TaskPattern::DocumentationNeeds { missing_docs } => {
                    for doc_need in missing_docs {
                        let subtask = SubTask {
                            id: SubTaskId::new(),
                            parent_id: TaskId::new(),
                            title: "Add documentation".to_string(),
                            description: doc_need.clone(),
                            scope: TaskScope {
                                included_files: vec![],
                                excluded_files: vec![],
                                included_patterns: vec!["*.rs".to_string()],
                                excluded_patterns: vec![],
                                time_budget: std::time::Duration::from_secs(180),
                                quality_requirements: QualityRequirements {
                                    documentation_required: true,
                                    ..QualityRequirements::default()
                                },
                            },
                            specialty: WorkerSpecialty::Documentation {
                                formats: vec!["rustdoc".to_string()],
                            },
                            dependencies: vec![],
                            estimated_effort: std::time::Duration::from_secs(120),
                            priority: Priority::Low,
                        };
                        all_subtasks.push(subtask);
                    }
                }
            }
        }

        // If no patterns were found, create a generic subtask
        if all_subtasks.is_empty() {
            all_subtasks.push(self.create_generic_subtask(&analysis));
        }

        Ok(all_subtasks)
    }

    /// Select appropriate decomposition strategy for a pattern
    fn select_strategy_for_pattern(&self, pattern: &TaskPattern) -> Box<dyn DecompositionStrategy> {
        // For now, return a strategy based on pattern type
        // This should be more sophisticated - we need a task parameter
        match pattern {
            TaskPattern::CompilationErrors { .. } => {
                Box::new(strategies::CompilationErrorStrategy::new())
            }
            TaskPattern::RefactoringOperations { .. } => {
                Box::new(strategies::RefactoringStrategy::new())
            }
            TaskPattern::TestingGaps { .. } => {
                Box::new(strategies::TestingStrategy::new())
            }
            TaskPattern::DocumentationNeeds { .. } => {
                Box::new(strategies::DocumentationStrategy::new())
            }
        }
    }

    /// Create a generic subtask when no specific patterns are identified
    fn create_generic_subtask(&self, analysis: &TaskAnalysis) -> SubTask {
        SubTask {
            id: SubTaskId::new(),
            parent_id: TaskId::new(), // This should be passed in
            title: "Generic task execution".to_string(),
            description: "Execute task using general-purpose approach".to_string(),
            scope: TaskScope::default(),
            specialty: WorkerSpecialty::Custom {
                domain: "general".to_string(),
                capabilities: vec!["execution".to_string()],
            },
            dependencies: vec![],
            estimated_effort: std::time::Duration::from_secs(300),
            priority: Priority::Medium,
        }
    }
}

impl Default for DecompositionEngine {
    fn default() -> Self {
        Self::new()
    }
}
