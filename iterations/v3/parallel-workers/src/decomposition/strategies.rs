//! Decomposition strategies for different types of tasks

use crate::types::*;
use crate::error::*;

/// Decomposition strategy interface
#[async_trait::async_trait]
pub trait DecompositionStrategy: Send + Sync {
    /// Check if this strategy applies to the given task
    fn applies_to(&self, task: &ComplexTask) -> bool;

    /// Decompose the task into subtasks
    async fn decompose(&self, task: &ComplexTask, analysis: &TaskAnalysis) -> DecompositionResult<Vec<SubTask>>;
}

/// Compilation error decomposition strategy
pub struct CompilationErrorStrategy;

impl Default for CompilationErrorStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilationErrorStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DecompositionStrategy for CompilationErrorStrategy {
    fn applies_to(&self, task: &ComplexTask) -> bool {
        task.description.to_lowercase().contains("compile") ||
        task.description.to_lowercase().contains("error") ||
        task.description.to_lowercase().contains("build")
    }

    async fn decompose(&self, task: &ComplexTask, analysis: &TaskAnalysis) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for compilation error patterns in the analysis
        for pattern in &analysis.patterns {
            if let TaskPattern::CompilationErrors { error_groups } = pattern {
                for (i, error_group) in error_groups.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId(format!("compile-fix-{}-{}", error_group.error_code, i)),
                        parent_id: task.id.clone(),
                        title: format!("Fix {} {} errors", error_group.count, error_group.error_code),
                        description: format!(
                            "Resolve {} compilation errors of type {} in {} files",
                            error_group.count, error_group.error_code, error_group.affected_files.len()
                        ),
                        scope: TaskScope {
                            files: error_group.affected_files.clone(),
                            directories: vec![],
                    patterns: vec![format!("*{}*", error_group.error_code)],
                            // time_budget: std::time::Duration::from_secs(300), // 5 minutes
                            // quality_requirements: QualityRequirements::default(),
                        },
                        specialty: WorkerSpecialty::CompilationErrors {
                            error_codes: vec![error_group.error_code.clone()],
                        },
                        dependencies: vec![], // Independent by default
                        estimated_effort: std::time::Duration::from_secs(120), // 2 minutes
                        priority: Priority::High,
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create a general compilation subtask
        if subtasks.is_empty() {
            subtasks.push(SubTask {
                id: SubTaskId("compile-general".to_string()),
                parent_id: task.id.clone(),
                title: "Fix compilation errors".to_string(),
                description: "Resolve all compilation errors in the codebase".to_string(),
                scope: TaskScope {
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["*.rs".to_string()],
                    // time_budget: std::time::Duration::from_secs(600), // 10 minutes
                    // quality_requirements: QualityRequirements::default(),
                },
                specialty: WorkerSpecialty::CompilationErrors {
                    error_codes: vec![],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(300), // 5 minutes
                priority: Priority::Critical,
            });
        }

        Ok(subtasks)
    }
}

/// Refactoring decomposition strategy
pub struct RefactoringStrategy;

impl Default for RefactoringStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DecompositionStrategy for RefactoringStrategy {
    fn applies_to(&self, task: &ComplexTask) -> bool {
        task.description.to_lowercase().contains("refactor") ||
        task.description.to_lowercase().contains("rename") ||
        task.description.to_lowercase().contains("extract") ||
        task.description.to_lowercase().contains("move")
    }

    async fn decompose(&self, task: &ComplexTask, analysis: &TaskAnalysis) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for refactoring patterns
        for pattern in &analysis.patterns {
            if let TaskPattern::RefactoringOperations { operations } = pattern {
                for (i, operation) in operations.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId(format!("refactor-{}-{}", operation.operation_type, i)),
                        parent_id: task.id.clone(),
                        title: format!("{} operation", operation.operation_type),
                        description: format!(
                            "Perform {} refactoring operation across {} files",
                            operation.operation_type, operation.affected_files.len()
                        ),
                        scope: TaskScope {
                            files: operation.affected_files.clone(),
                            directories: vec![],
                patterns: vec![],
                            // time_budget: std::time::Duration::from_secs(300), // 5 minutes
                            // quality_requirements: QualityRequirements::default(),
                        },
                        specialty: WorkerSpecialty::Refactoring {
                            strategies: vec![operation.operation_type.clone()],
                        },
                        dependencies: vec![], // Will be set by dependency analysis
                        estimated_effort: std::time::Duration::from_secs(
                            (operation.complexity * 300.0) as u64
                        ),
                        priority: Priority::Medium,
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create a general refactoring subtask
        if subtasks.is_empty() {
            subtasks.push(SubTask {
                id: SubTaskId("refactor-general".to_string()),
                parent_id: task.id.clone(),
                title: "General refactoring".to_string(),
                description: "Perform general refactoring operations".to_string(),
                scope: TaskScope {
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["*.rs".to_string()],
                    // time_budget: std::time::Duration::from_secs(600), // 10 minutes
                    // quality_requirements: QualityRequirements::default(),
                },
                specialty: WorkerSpecialty::Refactoring {
                    strategies: vec!["general".to_string()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(300), // 5 minutes
                priority: Priority::Medium,
            });
        }

        Ok(subtasks)
    }
}

/// Testing decomposition strategy
pub struct TestingStrategy;

impl Default for TestingStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl TestingStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DecompositionStrategy for TestingStrategy {
    fn applies_to(&self, task: &ComplexTask) -> bool {
        task.description.to_lowercase().contains("test") ||
        task.description.to_lowercase().contains("coverage") ||
        task.description.to_lowercase().contains("spec")
    }

    async fn decompose(&self, task: &ComplexTask, analysis: &TaskAnalysis) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for testing patterns
        for pattern in &analysis.patterns {
            if let TaskPattern::TestingGaps { missing_tests } = pattern {
                for (i, test_gap) in missing_tests.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId(format!("test-{}", i)),
                        parent_id: task.id.clone(),
                        title: format!("Add {}", test_gap),
                        description: test_gap.clone(),
                        scope: TaskScope {
                            files: vec![],
                            directories: vec![],
                            patterns: vec!["*.rs".to_string(), "*test*.rs".to_string()],
                        },
                        specialty: WorkerSpecialty::Testing {
                            frameworks: vec!["rust".to_string()], // Could be parameterized
                        },
                        dependencies: vec![],
                        estimated_effort: std::time::Duration::from_secs(180), // 3 minutes
                        priority: Priority::High,
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create general testing subtasks
        if subtasks.is_empty() {
            // Unit tests
            subtasks.push(SubTask {
                id: SubTaskId("unit-tests".to_string()),
                parent_id: task.id.clone(),
                title: "Add unit tests".to_string(),
                description: "Add unit tests for functions and methods".to_string(),
                scope: TaskScope {
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["src/**/*.rs".to_string()],
                },
                specialty: WorkerSpecialty::Testing {
                    frameworks: vec!["rust".to_string()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(300), // 5 minutes
                priority: Priority::High,
            });

            // Integration tests
            subtasks.push(SubTask {
                id: SubTaskId("integration-tests".to_string()),
                parent_id: task.id.clone(),
                title: "Add integration tests".to_string(),
                description: "Add integration tests for component interactions".to_string(),
                scope: TaskScope {
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["tests/**/*.rs".to_string()],
                },
                specialty: WorkerSpecialty::Testing {
                    frameworks: vec!["rust".to_string()],
                },
                dependencies: vec![SubTaskId("unit-tests".to_string())], // Depends on unit tests
                estimated_effort: std::time::Duration::from_secs(300), // 5 minutes
                priority: Priority::Medium,
            });
        }

        Ok(subtasks)
    }
}

/// Documentation decomposition strategy
pub struct DocumentationStrategy;

impl Default for DocumentationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentationStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DecompositionStrategy for DocumentationStrategy {
    fn applies_to(&self, task: &ComplexTask) -> bool {
        task.description.to_lowercase().contains("doc") ||
        task.description.to_lowercase().contains("readme") ||
        task.description.to_lowercase().contains("comment")
    }

    async fn decompose(&self, task: &ComplexTask, analysis: &TaskAnalysis) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for documentation patterns
        for pattern in &analysis.patterns {
            if let TaskPattern::DocumentationNeeds { missing_docs } = pattern {
                for (i, doc_need) in missing_docs.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId(format!("doc-{}", i)),
                        parent_id: task.id.clone(),
                        title: format!("Add {}", doc_need),
                        description: doc_need.clone(),
                        scope: TaskScope {
                            files: vec![],
                            directories: vec![],
                            patterns: vec!["*.rs".to_string(), "*.md".to_string()],
                        },
                        specialty: WorkerSpecialty::Documentation {
                            formats: vec!["markdown".to_string(), "rustdoc".to_string()],
                        },
                        dependencies: vec![],
                        estimated_effort: std::time::Duration::from_secs(120), // 2 minutes
                        priority: Priority::Low,
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create general documentation subtasks
        if subtasks.is_empty() {
            // API documentation
            subtasks.push(SubTask {
                id: SubTaskId("api-docs".to_string()),
                parent_id: task.id.clone(),
                title: "Add API documentation".to_string(),
                description: "Add documentation comments to public APIs".to_string(),
                scope: TaskScope {
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["src/**/*.rs".to_string()],
                },
                specialty: WorkerSpecialty::Documentation {
                    formats: vec!["rustdoc".to_string()],
                },
                dependencies: vec![],
                estimated_effort: std::time::Duration::from_secs(180), // 3 minutes
                priority: Priority::Low,
            });

            // README updates
            subtasks.push(SubTask {
                id: SubTaskId("readme-docs".to_string()),
                parent_id: task.id.clone(),
                title: "Update README".to_string(),
                description: "Update README with usage examples and API documentation".to_string(),
                scope: TaskScope {
                    files: vec!["README.md".into()],
                    directories: vec![],
                    patterns: vec!["README.md".to_string()],
                },
                specialty: WorkerSpecialty::Documentation {
                    formats: vec!["markdown".to_string()],
                },
                dependencies: vec![SubTaskId("api-docs".to_string())], // Depends on API docs
                estimated_effort: std::time::Duration::from_secs(120), // 2 minutes
                priority: Priority::Low,
            });
        }

        Ok(subtasks)
    }
}

/// Strategy registry for managing decomposition strategies
pub struct StrategyRegistry {
    strategies: Vec<Box<dyn DecompositionStrategy>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            strategies: Vec::new(),
        };

        // Register built-in strategies
        registry.register_strategy(Box::new(CompilationErrorStrategy::new()));
        registry.register_strategy(Box::new(RefactoringStrategy::new()));
        registry.register_strategy(Box::new(TestingStrategy::new()));
        registry.register_strategy(Box::new(DocumentationStrategy::new()));

        registry
    }

    /// Register a new decomposition strategy
    pub fn register_strategy(&mut self, strategy: Box<dyn DecompositionStrategy>) {
        self.strategies.push(strategy);
    }

    /// Find applicable strategies for a task
    pub fn find_applicable_strategies(&self, task: &ComplexTask) -> Vec<&dyn DecompositionStrategy> {
        self.strategies.iter()
            .filter(|strategy| strategy.applies_to(task))
            .map(|strategy| strategy.as_ref())
            .collect()
    }

    /// Get all registered strategies
    pub fn all_strategies(&self) -> &[Box<dyn DecompositionStrategy>] {
        &self.strategies
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}





