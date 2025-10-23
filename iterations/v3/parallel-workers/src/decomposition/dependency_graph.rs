//! Dependency analysis and graph construction for task decomposition

use crate::types::*;
use crate::error::*;
use std::collections::{HashMap, HashSet};

/// Dependency analyzer for understanding relationships between subtasks
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze dependencies between potential subtasks
    pub fn analyze(&self, task: &ComplexTask) -> DecompositionResult<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        // Analyze based on task type and context
        match self.infer_task_type(&task.description) {
            TaskType::Compilation => {
                dependencies.extend(self.analyze_compilation_dependencies(task)?);
            }
            TaskType::Refactoring => {
                dependencies.extend(self.analyze_refactoring_dependencies(task)?);
            }
            TaskType::Testing => {
                dependencies.extend(self.analyze_testing_dependencies(task)?);
            }
            TaskType::Documentation => {
                dependencies.extend(self.analyze_documentation_dependencies(task)?);
            }
            TaskType::General => {
                // For general tasks, assume minimal dependencies
                dependencies = vec![];
            }
        }

        Ok(dependencies)
    }

    /// Infer task type from description
    fn infer_task_type(&self, description: &str) -> TaskType {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("compile") || desc_lower.contains("build") || desc_lower.contains("error") {
            TaskType::Compilation
        } else if desc_lower.contains("refactor") || desc_lower.contains("rename") || desc_lower.contains("move") {
            TaskType::Refactoring
        } else if desc_lower.contains("test") || desc_lower.contains("coverage") {
            TaskType::Testing
        } else if desc_lower.contains("doc") || desc_lower.contains("readme") || desc_lower.contains("comment") {
            TaskType::Documentation
        } else {
            TaskType::General
        }
    }

    /// Analyze compilation dependencies
    fn analyze_compilation_dependencies(&self, task: &ComplexTask) -> DecompositionResult<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        // Compilation typically has dependencies based on module structure
        // lib.rs must compile before other modules
        // Types must compile before implementations that use them

        let rust_files = self.find_rust_files(&task.context.working_directory)?;

        // Create subtask IDs for each file
        let subtask_ids: Vec<_> = rust_files.iter()
            .enumerate()
            .map(|(i, _)| SubTaskId(format!("compile-{}", i)))
            .collect();

        // Add dependencies: lib.rs/mod.rs must come first
        for (i, file) in rust_files.iter().enumerate() {
            if self.is_library_root(file) {
                // Library root has no dependencies
                continue;
            }

            // Non-root files depend on the library root
            for (j, other_file) in rust_files.iter().enumerate() {
                if self.is_library_root(other_file) && i != j {
                    dependencies.push(Dependency {
                        from_subtask: subtask_ids[i].clone(),
                        to_subtask: subtask_ids[j].clone(),
                        dependency_type: DependencyType::CompilationOrder,
                        blocking: true,
                    });
                    break;
                }
            }
        }

        Ok(dependencies)
    }

    /// Analyze refactoring dependencies
    fn analyze_refactoring_dependencies(&self, task: &ComplexTask) -> DecompositionResult<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        // Refactoring operations often have ordering dependencies
        // e.g., extract function before moving it

        let operations = self.extract_refactoring_operations(&task.description);

        for (i, op) in operations.iter().enumerate() {
            let from_id = SubTaskId(format!("refactor-{}", i));

            // Check for dependencies based on operation type
            for (j, other_op) in operations.iter().enumerate() {
                if i != j {
                    let to_id = SubTaskId(format!("refactor-{}", j));

                    if self.operations_have_dependency(op, other_op) {
                        dependencies.push(Dependency {
                            from_subtask: from_id.clone(),
                            to_subtask: to_id,
                            dependency_type: DependencyType::DataDependency,
                            blocking: true,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Analyze testing dependencies
    fn analyze_testing_dependencies(&self, task: &ComplexTask) -> DecompositionResult<Vec<Dependency>> {
        // Testing usually has minimal dependencies - tests can run in parallel
        // Some integration tests might depend on unit tests passing first

        let mut dependencies = Vec::new();

        if task.description.to_lowercase().contains("integration") {
            // Integration tests depend on unit tests
            dependencies.push(Dependency {
                from_subtask: SubTaskId("integration-tests".to_string()),
                to_subtask: SubTaskId("unit-tests".to_string()),
                dependency_type: DependencyType::DataDependency,
                blocking: true,
            });
        }

        Ok(dependencies)
    }

    /// Analyze documentation dependencies
    fn analyze_documentation_dependencies(&self, task: &ComplexTask) -> DecompositionResult<Vec<Dependency>> {
        // Documentation usually has no dependencies - can be done in parallel
        Ok(vec![])
    }

    /// Find Rust files in the working directory
    fn find_rust_files(&self, dir: &std::path::Path) -> DecompositionResult<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        fn visit_dir(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() && path.file_name().unwrap_or_default() != "target" {
                        visit_dir(&path, files)?;
                    } else if path.extension().unwrap_or_default() == "rs" {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        visit_dir(dir, &mut files).map_err(|e| DecompositionError::FileAnalysis {
            path: dir.to_path_buf(),
            message: e.to_string(),
        })?;

        Ok(files)
    }

    /// Check if a file is a library root (lib.rs or main.rs)
    fn is_library_root(&self, file: &std::path::Path) -> bool {
        file.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with("lib") ||
        file.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with("main")
    }

    /// Extract refactoring operations from description
    fn extract_refactoring_operations(&self, description: &str) -> Vec<String> {
        let desc_lower = description.to_lowercase();
        let mut operations = Vec::new();

        if desc_lower.contains("rename") {
            operations.push("rename".to_string());
        }
        if desc_lower.contains("extract") {
            operations.push("extract".to_string());
        }
        if desc_lower.contains("move") {
            operations.push("move".to_string());
        }
        if desc_lower.contains("restructure") {
            operations.push("restructure".to_string());
        }

        operations
    }

    /// Check if two operations have a dependency
    fn operations_have_dependency(&self, op1: &str, op2: &str) -> bool {
        match (op1, op2) {
            ("extract", "move") => true, // Must extract before moving
            ("rename", "move") => true,  // Must rename before moving
            _ => false,
        }
    }
}

/// Task types for dependency analysis
#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskType {
    Compilation,
    Refactoring,
    Testing,
    Documentation,
    General,
}

/// Dependency graph for managing subtask relationships
pub struct DependencyGraph {
    nodes: HashMap<SubTaskId, SubTask>,
    edges: HashMap<SubTaskId, Vec<Dependency>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a subtask to the graph
    pub fn add_subtask(&mut self, subtask: SubTask) {
        let subtask_id = subtask.id.clone();
        self.nodes.insert(subtask_id.clone(), subtask);
        self.edges.entry(subtask_id).or_default();
    }

    /// Add a dependency between subtasks
    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.edges.entry(dependency.from_subtask.clone())
            .or_default()
            .push(dependency);
    }

    /// Get all subtasks with no dependencies (ready to execute)
    pub fn get_ready_subtasks(&self) -> Vec<SubTaskId> {
        self.nodes.keys()
            .filter(|&subtask_id| {
                self.get_dependencies(subtask_id).is_empty()
            })
            .cloned()
            .collect()
    }

    /// Get dependencies for a subtask
    pub fn get_dependencies(&self, subtask_id: &SubTaskId) -> Vec<&Dependency> {
        self.edges.get(subtask_id)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// Check if a subtask is ready to execute
    pub fn is_ready(&self, subtask_id: &SubTaskId) -> bool {
        self.get_dependencies(subtask_id).is_empty()
    }

    /// Mark dependencies as satisfied when a subtask completes
    pub fn complete_subtask(&mut self, completed_id: &SubTaskId) {
        // Remove the completed subtask from the graph
        self.nodes.remove(completed_id);
        self.edges.remove(completed_id);

        // Remove dependencies on the completed subtask
        for dependencies in self.edges.values_mut() {
            dependencies.retain(|dep| &dep.to_subtask != completed_id);
        }
    }

    /// Get a topological sort of subtasks (execution order)
    pub fn topological_sort(&self) -> DecompositionResult<Vec<SubTaskId>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for subtask_id in self.nodes.keys() {
            if !visited.contains(subtask_id) {
                self.dfs(subtask_id, &mut visited, &mut visiting, &mut result)?;
            }
        }

        result.reverse(); // Reverse to get correct execution order
        Ok(result)
    }

    /// Depth-first search for topological sort
    fn dfs(
        &self,
        subtask_id: &SubTaskId,
        visited: &mut HashSet<SubTaskId>,
        visiting: &mut HashSet<SubTaskId>,
        result: &mut Vec<SubTaskId>,
    ) -> DecompositionResult<()> {
        if visiting.contains(subtask_id) {
            return Err(DecompositionError::CircularDependency {
                subtask_ids: vec![subtask_id.clone()],
            });
        }

        if visited.contains(subtask_id) {
            return Ok(());
        }

        visiting.insert(subtask_id.clone());

        // Visit all dependencies
        for dependency in self.get_dependencies(subtask_id) {
            self.dfs(&dependency.to_subtask, visited, visiting, result)?;
        }

        visiting.remove(subtask_id);
        visited.insert(subtask_id.clone());
        result.push(subtask_id.clone());

        Ok(())
    }

    /// Detect cycles in the dependency graph
    pub fn has_cycles(&self) -> bool {
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for subtask_id in self.nodes.keys() {
            if !visited.contains(subtask_id)
                && self.has_cycle(subtask_id, &mut visited, &mut visiting) {
                    return true;
                }
        }

        false
    }

    /// Check for cycles starting from a node
    fn has_cycle(
        &self,
        subtask_id: &SubTaskId,
        visited: &mut HashSet<SubTaskId>,
        visiting: &mut HashSet<SubTaskId>,
    ) -> bool {
        if visiting.contains(subtask_id) {
            return true;
        }

        if visited.contains(subtask_id) {
            return false;
        }

        visiting.insert(subtask_id.clone());

        for dependency in self.get_dependencies(subtask_id) {
            if self.has_cycle(&dependency.to_subtask, visited, visiting) {
                return true;
            }
        }

        visiting.remove(subtask_id);
        visited.insert(subtask_id.clone());

        false
    }

    /// Get the number of subtasks in the graph
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}
