//! Decomposition strategies for different types of tasks

use crate::error::*;
use crate::parallel_types::*;
use crate::worker_types::{Priority, SubTaskId, TaskScope};
use std::collections::HashMap;

/// Decomposition strategy interface
#[async_trait::async_trait]
pub trait DecompositionStrategy: Send + Sync {
    /// Check if this strategy applies to the given task
    fn applies_to(&self, task: &ComplexTask) -> bool;

    /// Decompose the task into subtasks
    async fn decompose(
        &self,
        task: &ComplexTask,
        analysis: &TaskAnalysis,
    ) -> DecompositionResult<Vec<SubTask>>;
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
        task.description.to_lowercase().contains("compile")
            || task.description.to_lowercase().contains("error")
            || task.description.to_lowercase().contains("build")
    }

    async fn decompose(
        &self,
        task: &ComplexTask,
        analysis: &TaskAnalysis,
    ) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for compilation error patterns in the analysis
        for pattern in &analysis.patterns {
            if let TaskPattern::CompilationErrors { error_groups } = pattern {
                for (i, error_group) in error_groups.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId::new(),
                        parent_task_id: task.id.clone(),
                        parent_id: task.id.clone(),
                        title: format!("Fix {} compilation errors", error_group.error_count),
                        description: format!(
                            "Resolve {} compilation errors in {}",
                            error_group.error_count, error_group.file_path
                        ),
                        complexity: 0.8,
                        dependencies: vec![], // Independent by default
                        assigned_worker: None,
                        status: SubTaskStatus::Pending,
                        priority: Priority::High,
                        estimated_duration: std::time::Duration::from_secs(120), // 2 minutes
                        scope: TaskScope {
                            domains: vec![],
                            files_affected: vec![error_group.file_path.clone()],
                            files: vec![],
                            directories: vec![],
                            patterns: vec![],
                            max_files: None,
                            max_loc: None,
                        },
                        specialty: WorkerSpecialty::Compilation,
                        estimated_effort: 120.0, // 2 minutes in seconds
                        metadata: HashMap::new(),
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create a general compilation subtask
        if subtasks.is_empty() {
            subtasks.push(SubTask {
                id: SubTaskId::new(),
                parent_task_id: task.id.clone(),
                parent_id: task.id.clone(),
                title: "Fix compilation errors".to_string(),
                description: "Resolve all compilation errors in the codebase".to_string(),
                complexity: 0.9,
                dependencies: vec![],
                assigned_worker: None,
                status: SubTaskStatus::Pending,
                priority: Priority::Critical,
                estimated_duration: std::time::Duration::from_secs(300), // 5 minutes
                scope: TaskScope {
                    domains: vec![],
                    files_affected: vec![],
                    files: vec![],
                    directories: vec![],
                    patterns: vec![],
                    max_files: None,
                    max_loc: None,
                },
                specialty: WorkerSpecialty::Compilation,
                estimated_effort: 300.0, // 5 minutes in seconds
                metadata: HashMap::new(),
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
        task.description.to_lowercase().contains("refactor")
            || task.description.to_lowercase().contains("rename")
            || task.description.to_lowercase().contains("extract")
            || task.description.to_lowercase().contains("move")
    }

    async fn decompose(
        &self,
        task: &ComplexTask,
        analysis: &TaskAnalysis,
    ) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for refactoring patterns
        for pattern in &analysis.patterns {
            if let TaskPattern::RefactoringOperations { operations } = pattern {
                for (i, operation) in operations.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId::new(),
                        parent_task_id: task.id.clone(),
                        parent_id: task.id.clone(),
                        title: format!("{} operation", operation.operation_type),
                        description: operation.description.clone(),
                        complexity: operation.complexity,
                        dependencies: vec![], // Will be set by dependency analysis
                        assigned_worker: None,
                        status: SubTaskStatus::Pending,
                        priority: Priority::Medium,
                        estimated_duration: std::time::Duration::from_secs(
                            (operation.complexity * 300.0) as u64,
                        ),
                        scope: TaskScope {
                            domains: vec![],
                            files_affected: vec![],
                            files: vec![],
                            directories: vec![],
                            patterns: vec![],
                            max_files: None,
                            max_loc: None,
                        },
                        specialty: WorkerSpecialty::Refactoring {
                            patterns: vec!["code_cleanup".to_string(), "optimization".to_string()],
                        },
                        estimated_effort: (operation.complexity * 300.0),
                        metadata: HashMap::new(),
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create a general refactoring subtask
        if subtasks.is_empty() {
            subtasks.push(SubTask {
                id: SubTaskId::new(),
                parent_task_id: task.id.clone(),
                parent_id: task.id.clone(),
                title: "General refactoring".to_string(),
                description: "Perform general refactoring operations".to_string(),
                complexity: 0.7,
                dependencies: vec![],
                assigned_worker: None,
                status: SubTaskStatus::Pending,
                priority: Priority::Medium,
                estimated_duration: std::time::Duration::from_secs(300), // 5 minutes
                scope: TaskScope {
                    domains: vec![],
                    files_affected: vec![],
                    files: vec![],
                    directories: vec![],
                    patterns: vec![],
                    max_files: None,
                    max_loc: None,
                },
                specialty: WorkerSpecialty::Refactoring { patterns: vec![] },
                estimated_effort: 300.0,
                metadata: HashMap::new(),
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
        task.description.to_lowercase().contains("test")
            || task.description.to_lowercase().contains("coverage")
            || task.description.to_lowercase().contains("spec")
    }

    async fn decompose(
        &self,
        task: &ComplexTask,
        analysis: &TaskAnalysis,
    ) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for testing patterns
        for pattern in &analysis.patterns {
            if let TaskPattern::TestingGaps { missing_tests } = pattern {
                for (i, test_gap) in missing_tests.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId(uuid::Uuid::new_v4()),
                        parent_task_id: task.id.clone(),
                        parent_id: task.id.clone(),
                        title: format!("Add {}", test_gap),
                        description: test_gap.clone(),
                        complexity: 0.6,
                        dependencies: vec![],
                        assigned_worker: None,
                        status: SubTaskStatus::Pending,
                        priority: Priority::High,
                        estimated_duration: std::time::Duration::from_secs(180), // 3 minutes
                        scope: TaskScope {
                            domains: vec![],
                            files_affected: vec![],
                            files: vec![],
                            directories: vec![],
                            patterns: vec!["*.rs".to_string(), "*test*.rs".to_string()],
                            max_files: None,
                            max_loc: None,
                        },
                        specialty: WorkerSpecialty::Testing {
                            frameworks: vec!["rust".to_string()], // Could be parameterized
                        },
                        estimated_effort: 180.0, // 3 minutes in seconds
                        metadata: HashMap::new(),
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create general testing subtasks
        if subtasks.is_empty() {
            // Unit tests
            let unit_test_id = SubTaskId::new();
            subtasks.push(SubTask {
                id: unit_test_id.clone(),
                parent_task_id: task.id.clone(),
                parent_id: task.id.clone(),
                title: "Add unit tests".to_string(),
                description: "Add unit tests for functions and methods".to_string(),
                complexity: 0.7,
                dependencies: vec![],
                assigned_worker: None,
                status: SubTaskStatus::Pending,
                priority: Priority::High,
                estimated_duration: std::time::Duration::from_secs(600), // 10 minutes
                scope: TaskScope {
                    domains: vec![],
                    files_affected: vec![],
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["src/**/*.rs".to_string()],
                    max_files: None,
                    max_loc: None,
                },
                specialty: WorkerSpecialty::Testing {
                    frameworks: vec!["rust".to_string()],
                },
                estimated_effort: 600.0, // 10 minutes in seconds
                metadata: HashMap::new(),
            });

            // Integration tests
            subtasks.push(SubTask {
                id: SubTaskId::new(),
                parent_task_id: task.id.clone(),
                parent_id: task.id.clone(),
                title: "Add integration tests".to_string(),
                description: "Add integration tests for component interactions".to_string(),
                complexity: 0.8,
                dependencies: vec![unit_test_id], // Depends on unit tests
                assigned_worker: None,
                status: SubTaskStatus::Pending,
                priority: Priority::High,
                estimated_duration: std::time::Duration::from_secs(900), // 15 minutes
                scope: TaskScope {
                    domains: vec![],
                    files_affected: vec![],
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["tests/**/*.rs".to_string()],
                    max_files: None,
                    max_loc: None,
                },
                specialty: WorkerSpecialty::Testing {
                    frameworks: vec!["rust".to_string()],
                },
                estimated_effort: 900.0, // 15 minutes in seconds
                metadata: HashMap::new(),
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
        task.description.to_lowercase().contains("doc")
            || task.description.to_lowercase().contains("readme")
            || task.description.to_lowercase().contains("comment")
    }

    async fn decompose(
        &self,
        task: &ComplexTask,
        analysis: &TaskAnalysis,
    ) -> DecompositionResult<Vec<SubTask>> {
        let mut subtasks = Vec::new();

        // Look for documentation patterns
        for pattern in &analysis.patterns {
            if let TaskPattern::DocumentationNeeds { files_needing_docs } = pattern {
                for (i, doc_need) in files_needing_docs.iter().enumerate() {
                    let subtask = SubTask {
                        id: SubTaskId(uuid::Uuid::new_v4()),
                        parent_task_id: task.id.clone(),
                        parent_id: task.id.clone(),
                        title: format!("Add {}", doc_need),
                        description: doc_need.clone(),
                        complexity: 0.5,
                        dependencies: vec![],
                        assigned_worker: None,
                        status: SubTaskStatus::Pending,
                        priority: Priority::Low,
                        estimated_duration: std::time::Duration::from_secs(120), // 2 minutes
                        scope: TaskScope {
                            domains: vec![],
                            files_affected: vec![],
                            files: vec![],
                            directories: vec![],
                            patterns: vec!["*.rs".to_string(), "*.md".to_string()],
                            max_files: None,
                            max_loc: None,
                        },
                        specialty: WorkerSpecialty::Documentation {
                            formats: vec!["markdown".to_string(), "rustdoc".to_string()],
                        },
                        estimated_effort: 120.0, // 2 minutes in seconds
                        metadata: HashMap::new(),
                    };

                    subtasks.push(subtask);
                }
            }
        }

        // If no specific patterns found, create general documentation subtasks
        if subtasks.is_empty() {
            // API documentation
            let api_docs_id = SubTaskId::new();
            subtasks.push(SubTask {
                id: api_docs_id.clone(),
                parent_task_id: task.id.clone(),
                parent_id: task.id.clone(),
                title: "Add API documentation".to_string(),
                description: "Add documentation comments to public APIs".to_string(),
                complexity: 0.6,
                dependencies: vec![],
                assigned_worker: None,
                status: SubTaskStatus::Pending,
                priority: Priority::Low,
                estimated_duration: std::time::Duration::from_secs(180), // 3 minutes
                scope: TaskScope {
                    domains: vec![],
                    files_affected: vec![],
                    files: vec![],
                    directories: vec![],
                    patterns: vec!["src/**/*.rs".to_string()],
                    max_files: None,
                    max_loc: None,
                },
                specialty: WorkerSpecialty::Documentation {
                    formats: vec!["rustdoc".to_string()],
                },
                estimated_effort: 180.0, // 3 minutes in seconds
                metadata: HashMap::new(),
            });

            // README updates
            subtasks.push(SubTask {
                id: SubTaskId::new(),
                parent_task_id: task.id.clone(),
                parent_id: task.id.clone(),
                title: "Update README".to_string(),
                description: "Update README with usage examples and API documentation".to_string(),
                complexity: 0.5,
                dependencies: vec![api_docs_id.clone()], // Depends on API docs
                assigned_worker: None,
                status: SubTaskStatus::Pending,
                priority: Priority::Low,
                estimated_duration: std::time::Duration::from_secs(120), // 2 minutes
                scope: TaskScope {
                    domains: vec![],
                    files_affected: vec!["README.md".to_string()],
                    files: vec!["README.md".to_string()],
                    directories: vec![],
                    patterns: vec!["README.md".to_string()],
                    max_files: None,
                    max_loc: None,
                },
                specialty: WorkerSpecialty::Documentation {
                    formats: vec!["markdown".to_string()],
                },
                estimated_effort: 120.0, // 2 minutes in seconds
                metadata: HashMap::new(),
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
    pub fn find_applicable_strategies(
        &self,
        task: &ComplexTask,
    ) -> Vec<&dyn DecompositionStrategy> {
        self.strategies
            .iter()
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
