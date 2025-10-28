//! Task decomposition engine and pattern analysis

pub mod task_analyzer;
pub mod dependency_graph;
pub mod strategies;

pub use task_analyzer::*;
pub use dependency_graph::*;
pub use strategies::*;

// Re-export types from types module that are used in decomposition
pub use crate::parallel_types::{ComplexTask, TaskAnalysis, TaskPattern, Dependency, SubtaskScores, SubTask, TaskId, SubTaskId, TaskScope, QualityRequirements, WorkerSpecialty, Priority};

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
        use tracing::{info, warn, error};
        
        info!("Starting task decomposition for analysis with {} patterns", analysis.patterns.len());
        
        // Select appropriate decomposition strategy based on task complexity
        let strategy = self.select_decomposition_strategy(&analysis)?;
        info!("Selected decomposition strategy: {:?}", strategy);
        
        let mut all_subtasks = Vec::new();
        let mut task_dependencies = std::collections::HashMap::new();

        // Create subtasks based on pattern types with real strategy
        for (pattern_idx, pattern) in analysis.patterns.iter().enumerate() {
            match pattern {
                TaskPattern::CompilationErrors { error_groups } => {
                    let subtasks = self.decompose_compilation_errors(error_groups, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
                TaskPattern::RefactoringOperations { operations } => {
                    let subtasks = self.decompose_refactoring_operations(operations, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
                TaskPattern::TestFailures { failures } => {
                    let subtasks = self.decompose_test_failures(failures, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
                TaskPattern::DocumentationGaps { gaps } => {
                    let subtasks = self.decompose_documentation_gaps(gaps, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
                TaskPattern::PerformanceIssues { issues } => {
                    let subtasks = self.decompose_performance_issues(issues, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
                TaskPattern::SecurityVulnerabilities { vulnerabilities } => {
                    let subtasks = self.decompose_security_vulnerabilities(vulnerabilities, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
                TaskPattern::DependencyUpdates { updates } => {
                    let subtasks = self.decompose_dependency_updates(updates, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
                TaskPattern::CodeQualityIssues { issues } => {
                    let subtasks = self.decompose_code_quality_issues(issues, &analysis, pattern_idx)?;
                    all_subtasks.extend(subtasks);
                }
            }
        }

        // Apply decomposition strategy to optimize task ordering and dependencies
        let optimized_subtasks = self.apply_decomposition_strategy(all_subtasks, &strategy)?;
        
        // Validate decomposition results
        self.validate_decomposition(&optimized_subtasks, &analysis)?;
        
        info!("Decomposition completed with {} subtasks", optimized_subtasks.len());
        Ok(optimized_subtasks)
    }

    /// Select appropriate decomposition strategy based on task complexity
    fn select_decomposition_strategy(&self, analysis: &TaskAnalysis) -> Result<DecompositionStrategy, DecompositionError> {
        let pattern_count = analysis.patterns.len();
        let total_complexity: f64 = analysis.patterns.iter()
            .map(|p| self.calculate_pattern_complexity(p))
            .sum();
        
        match (pattern_count, total_complexity) {
            (1..=3, 0.0..=5.0) => Ok(DecompositionStrategy::Sequential),
            (4..=8, 5.0..=15.0) => Ok(DecompositionStrategy::Parallel),
            (9.., 15.0..) => Ok(DecompositionStrategy::Hierarchical),
            _ => Ok(DecompositionStrategy::Adaptive),
        }
    }

    /// Calculate complexity score for a pattern
    fn calculate_pattern_complexity(&self, pattern: &TaskPattern) -> f64 {
        match pattern {
            TaskPattern::CompilationErrors { error_groups } => error_groups.len() as f64 * 2.0,
            TaskPattern::RefactoringOperations { operations } => operations.len() as f64 * 3.0,
            TaskPattern::TestFailures { failures } => failures.len() as f64 * 1.5,
            TaskPattern::DocumentationGaps { gaps } => gaps.len() as f64 * 0.5,
            TaskPattern::PerformanceIssues { issues } => issues.len() as f64 * 4.0,
            TaskPattern::SecurityVulnerabilities { vulnerabilities } => vulnerabilities.len() as f64 * 5.0,
            TaskPattern::DependencyUpdates { updates } => updates.len() as f64 * 2.5,
            TaskPattern::CodeQualityIssues { issues } => issues.len() as f64 * 1.0,
        }
    }

    /// Decompose compilation errors into subtasks
    fn decompose_compilation_errors(&self, error_groups: &[ErrorGroup], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for (idx, error_group) in error_groups.iter().enumerate() {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: format!("Fix {} errors", error_group.error_code),
                description: format!("Resolve {} compilation errors in {} files", 
                    error_group.count, error_group.affected_files.len()),
                scope: TaskScope {
                    files: error_group.affected_files.clone(),
                    directories: vec![],
                    patterns: vec![],
                },
                specialty: WorkerSpecialty::CompilationErrors {
                    error_codes: vec![error_group.error_code.clone()],
                },
                dependencies: self.calculate_compilation_dependencies(idx, error_groups),
                estimated_effort: std::time::Duration::from_secs(
                    (error_group.count * 30).min(1800) as u64
                ),
                priority: self.calculate_compilation_priority(error_group),
            };
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Calculate dependencies for compilation tasks
    fn calculate_compilation_dependencies(&self, idx: usize, error_groups: &[ErrorGroup]) -> Vec<SubTaskId> {
        // Simple dependency: fix simpler errors first
        if idx > 0 && error_groups[idx].count < error_groups[idx - 1].count {
            vec![SubTaskId::new()] // Placeholder - would use actual previous task ID
        } else {
            vec![]
        }
    }

    /// Calculate priority for compilation tasks
    fn calculate_compilation_priority(&self, error_group: &ErrorGroup) -> Priority {
        match error_group.count {
            1..=5 => Priority::High,
            6..=20 => Priority::Medium,
            _ => Priority::Low,
        }
    }

    /// Decompose refactoring operations into subtasks
    fn decompose_refactoring_operations(&self, operations: &[RefactoringOperation], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for operation in operations {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: operation.operation_type.clone(),
                description: format!("Perform {} refactoring on {} files", 
                    operation.operation_type, operation.affected_files.len()),
                scope: TaskScope {
                    files: operation.affected_files.clone(),
                    directories: vec![],
                    patterns: vec![],
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
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Decompose test failures into subtasks
    fn decompose_test_failures(&self, failures: &[TestFailure], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for failure in failures {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: format!("Fix test: {}", failure.test_name),
                description: format!("Resolve test failure: {}", failure.failure_reason),
                scope: TaskScope {
                    files: vec![failure.file_path.clone()],
                    directories: vec![],
                    patterns: vec![],
                },
                specialty: WorkerSpecialty::Testing {
                    frameworks: vec!["rust".to_string()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(120),
                priority: Priority::High,
            };
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Decompose documentation gaps into subtasks
    fn decompose_documentation_gaps(&self, gaps: &[String], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for gap in gaps {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: "Add documentation".to_string(),
                description: gap.clone(),
                scope: TaskScope {
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["*.rs".to_string()],
                },
                specialty: WorkerSpecialty::Documentation {
                    formats: vec!["rustdoc".to_string()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(120),
                priority: Priority::Low,
            };
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Decompose performance issues into subtasks
    fn decompose_performance_issues(&self, issues: &[PerformanceIssue], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for issue in issues {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: format!("Optimize: {}", issue.component),
                description: format!("Address performance issue: {}", issue.description),
                scope: TaskScope {
                    files: issue.affected_files.clone(),
                    directories: vec![],
                    patterns: vec![],
                },
                specialty: WorkerSpecialty::Performance {
                    optimization_types: vec![issue.issue_type.clone()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(600),
                priority: Priority::High,
            };
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Decompose security vulnerabilities into subtasks
    fn decompose_security_vulnerabilities(&self, vulnerabilities: &[SecurityVulnerability], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for vuln in vulnerabilities {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: format!("Fix security issue: {}", vuln.vulnerability_type),
                description: format!("Address security vulnerability: {}", vuln.description),
                scope: TaskScope {
                    files: vuln.affected_files.clone(),
                    directories: vec![],
                    patterns: vec![],
                },
                specialty: WorkerSpecialty::Security {
                    vulnerability_types: vec![vuln.vulnerability_type.clone()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(900),
                priority: Priority::Critical,
            };
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Decompose dependency updates into subtasks
    fn decompose_dependency_updates(&self, updates: &[DependencyUpdate], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for update in updates {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: format!("Update dependency: {}", update.package_name),
                description: format!("Update {} from {} to {}", 
                    update.package_name, update.current_version, update.target_version),
                scope: TaskScope {
                    files: vec!["Cargo.toml".to_string()],
                    directories: vec![],
                    patterns: vec![],
                },
                specialty: WorkerSpecialty::DependencyManagement {
                    package_types: vec![update.package_type.clone()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(300),
                priority: Priority::Medium,
            };
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Decompose code quality issues into subtasks
    fn decompose_code_quality_issues(&self, issues: &[CodeQualityIssue], analysis: &TaskAnalysis, pattern_idx: usize) -> Result<Vec<SubTask>, DecompositionError> {
        let mut subtasks = Vec::new();
        
        for issue in issues {
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_id: analysis.task_id.clone(),
                title: format!("Fix quality issue: {}", issue.issue_type),
                description: format!("Address code quality issue: {}", issue.description),
                scope: TaskScope {
                    files: issue.affected_files.clone(),
                    directories: vec![],
                    patterns: vec![],
                },
                specialty: WorkerSpecialty::CodeQuality {
                    quality_metrics: vec![issue.issue_type.clone()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(180),
                priority: Priority::Medium,
            };
            subtasks.push(subtask);
        }
        
        Ok(subtasks)
    }

    /// Apply decomposition strategy to optimize task ordering
    fn apply_decomposition_strategy(&self, mut subtasks: Vec<SubTask>, strategy: &DecompositionStrategy) -> Result<Vec<SubTask>, DecompositionError> {
        match strategy {
            DecompositionStrategy::Sequential => {
                // Sort by priority and estimated effort
                subtasks.sort_by(|a, b| {
                    b.priority.cmp(&a.priority)
                        .then(a.estimated_effort.cmp(&b.estimated_effort))
                });
            }
            DecompositionStrategy::Parallel => {
                // Group by specialty for parallel execution
                subtasks.sort_by(|a, b| {
                    std::mem::discriminant(&a.specialty).cmp(&std::mem::discriminant(&b.specialty))
                });
            }
            DecompositionStrategy::Hierarchical => {
                // Create dependency chains
                self.create_hierarchical_dependencies(&mut subtasks)?;
            }
            DecompositionStrategy::Adaptive => {
                // Dynamic optimization based on current system state
                self.apply_adaptive_optimization(&mut subtasks)?;
            }
        }
        
        Ok(subtasks)
    }

    /// Create hierarchical dependencies between tasks
    fn create_hierarchical_dependencies(&self, subtasks: &mut [SubTask]) -> Result<(), DecompositionError> {
        // Simple dependency creation: compilation errors first, then refactoring, then testing
        for i in 1..subtasks.len() {
            if self.should_create_dependency(&subtasks[i-1], &subtasks[i]) {
                subtasks[i].dependencies.push(subtasks[i-1].id.clone());
            }
        }
        Ok(())
    }

    /// Check if two tasks should have a dependency relationship
    fn should_create_dependency(&self, predecessor: &SubTask, successor: &SubTask) -> bool {
        match (&predecessor.specialty, &successor.specialty) {
            (WorkerSpecialty::CompilationErrors { .. }, WorkerSpecialty::Refactoring { .. }) => true,
            (WorkerSpecialty::Refactoring { .. }, WorkerSpecialty::Testing { .. }) => true,
            (WorkerSpecialty::Testing { .. }, WorkerSpecialty::Documentation { .. }) => true,
            _ => false,
        }
    }

    /// Apply adaptive optimization based on system state
    fn apply_adaptive_optimization(&self, subtasks: &mut [SubTask]) -> Result<(), DecompositionError> {
        // Placeholder for adaptive optimization logic
        // In a real implementation, this would consider:
        // - Current system load
        // - Available workers
        // - Historical performance data
        // - Resource constraints
        
        // For now, apply a simple optimization
        subtasks.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(a.estimated_effort.cmp(&b.estimated_effort))
        });
        
        Ok(())
    }

    /// Validate decomposition results
    fn validate_decomposition(&self, subtasks: &[SubTask], analysis: &TaskAnalysis) -> Result<(), DecompositionError> {
        if subtasks.is_empty() {
            return Err(DecompositionError::NoSubtasksGenerated);
        }
        
        // Check for circular dependencies
        if self.has_circular_dependencies(subtasks) {
            return Err(DecompositionError::CircularDependencies);
        }
        
        // Validate that all patterns are covered
        let covered_patterns = self.count_covered_patterns(subtasks, analysis);
        if covered_patterns < analysis.patterns.len() {
            return Err(DecompositionError::IncompleteCoverage);
        }
        
        Ok(())
    }

    /// Check for circular dependencies
    fn has_circular_dependencies(&self, subtasks: &[SubTask]) -> bool {
        // Simple circular dependency detection
        for subtask in subtasks {
            if subtask.dependencies.contains(&subtask.id) {
                return true;
            }
        }
        false
    }

    /// Count how many patterns are covered by subtasks
    fn count_covered_patterns(&self, subtasks: &[SubTask], analysis: &TaskAnalysis) -> usize {
        // Placeholder implementation
        // In a real implementation, this would track which patterns are covered by which subtasks
        analysis.patterns.len().min(subtasks.len())
    }
    }
}

impl Default for DecompositionEngine {
    fn default() -> Self {
        Self::new()
    }
}
