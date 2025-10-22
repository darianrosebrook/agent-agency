//! Problem pattern analysis for task decomposition

use crate::types::*;
use crate::error::*;
use std::collections::HashMap;

/// Pattern recognizer for identifying decomposition opportunities
pub struct PatternRecognizer;

impl PatternRecognizer {
    pub fn new() -> Self {
        Self
    }

    /// Identify patterns in a complex task
    pub fn identify_patterns(&self, task: &ComplexTask) -> Result<Vec<TaskPattern>, DecompositionError> {
        let mut patterns = Vec::new();

        // Analyze task description and context for patterns
        let description = &task.description;
        let context = &task.context;

        // Check for compilation-related patterns
        if self.is_compilation_task(description) {
            let compilation_patterns = self.identify_compilation_patterns(description, context)?;
            if !compilation_patterns.is_empty() {
                patterns.push(TaskPattern::CompilationErrors { error_groups: compilation_patterns });
            }
        }

        // Check for refactoring patterns
        if self.is_refactoring_task(description) {
            let refactoring_patterns = self.identify_refactoring_patterns(description, context)?;
            if !refactoring_patterns.is_empty() {
                patterns.push(TaskPattern::RefactoringOperations { operations: refactoring_patterns });
            }
        }

        // Check for testing patterns
        if self.is_testing_task(description) {
            patterns.push(TaskPattern::TestingGaps {
                missing_tests: self.identify_testing_gaps(description, context)?,
            });
        }

        // Check for documentation patterns
        if self.is_documentation_task(description) {
            patterns.push(TaskPattern::DocumentationNeeds {
                missing_docs: self.identify_documentation_needs(description, context)?,
            });
        }

        Ok(patterns)
    }

    /// Check if task is compilation-related
    fn is_compilation_task(&self, description: &str) -> bool {
        let compilation_keywords = [
            "compile", "compilation", "build", "cargo check", "error", "E0",
            "rustc", "linking", "undefined reference", "missing",
        ];

        compilation_keywords.iter()
            .any(|keyword| description.to_lowercase().contains(keyword))
    }

    /// Check if task is refactoring-related
    fn is_refactoring_task(&self, description: &str) -> bool {
        let refactoring_keywords = [
            "refactor", "rename", "extract", "move", "restructure",
            "reorganize", "clean", "simplify", "optimize",
        ];

        refactoring_keywords.iter()
            .any(|keyword| description.to_lowercase().contains(keyword))
    }

    /// Check if task is testing-related
    fn is_testing_task(&self, description: &str) -> bool {
        let testing_keywords = [
            "test", "testing", "coverage", "spec", "assert", "mock",
            "fixture", "unit test", "integration test",
        ];

        testing_keywords.iter()
            .any(|keyword| description.to_lowercase().contains(keyword))
    }

    /// Check if task is documentation-related
    fn is_documentation_task(&self, description: &str) -> bool {
        let documentation_keywords = [
            "doc", "document", "readme", "comment", "api docs",
            "user guide", "tutorial", "example",
        ];

        documentation_keywords.iter()
            .any(|keyword| description.to_lowercase().contains(keyword))
    }

    /// Identify compilation error patterns
    fn identify_compilation_patterns(
        &self,
        description: &str,
        context: &TaskContext,
    ) -> Result<Vec<ErrorGroup>, DecompositionError> {
        let mut error_groups = Vec::new();

        // Try to extract error information from description or working directory
        let error_codes = self.extract_error_codes(description);
        let affected_files = self.find_rust_files(&context.working_directory)?;

        // Group by error types
        let mut error_map: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();

        for error_code in error_codes {
            let affected = self.files_likely_affected_by_error(&error_code, &affected_files);
            if !affected.is_empty() {
                error_map.insert(error_code, affected);
            }
        }

        // Convert to ErrorGroup structs
        for (error_code, files) in error_map {
            error_groups.push(ErrorGroup {
                error_code,
                count: files.len(),
                affected_files: files,
            });
        }

        Ok(error_groups)
    }

    /// Extract error codes from description
    fn extract_error_codes(&self, description: &str) -> Vec<String> {
        let error_pattern = regex::Regex::new(r"E\d{4}").unwrap();
        error_pattern.find_iter(description)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Find Rust files in directory
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

    /// Determine which files are likely affected by a specific error
    fn files_likely_affected_by_error(&self, error_code: &str, files: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
        // This is a simplified heuristic - in practice, you'd analyze the actual error
        match error_code {
            "E0063" => files.iter().filter(|f| f.to_string_lossy().contains("struct")).cloned().collect(),
            "E0277" => files.iter().filter(|f| f.to_string_lossy().contains("trait") || f.to_string_lossy().contains("impl")).cloned().collect(),
            "E0308" => files.iter().filter(|f| f.to_string_lossy().contains("fn") || f.to_string_lossy().contains("let")).cloned().collect(),
            _ => files.to_vec(), // Default to all files
        }
    }

    /// Identify refactoring operation patterns
    fn identify_refactoring_patterns(
        &self,
        description: &str,
        context: &TaskContext,
    ) -> Result<Vec<RefactoringOp>, DecompositionError> {
        let mut operations = Vec::new();

        // Look for common refactoring patterns in description
        if description.to_lowercase().contains("rename") {
            operations.push(RefactoringOp {
                operation_type: "rename".to_string(),
                affected_files: self.find_rust_files(&context.working_directory)?,
                complexity: 0.7, // Moderate complexity
            });
        }

        if description.to_lowercase().contains("extract") {
            operations.push(RefactoringOp {
                operation_type: "extract".to_string(),
                affected_files: self.find_rust_files(&context.working_directory)?,
                complexity: 0.8, // Higher complexity
            });
        }

        if description.to_lowercase().contains("move") {
            operations.push(RefactoringOp {
                operation_type: "move".to_string(),
                affected_files: self.find_rust_files(&context.working_directory)?,
                complexity: 0.6, // Lower complexity
            });
        }

        Ok(operations)
    }

    /// Identify testing gaps
    fn identify_testing_gaps(
        &self,
        description: &str,
        context: &TaskContext,
    ) -> Result<Vec<String>, DecompositionError> {
        let mut gaps = Vec::new();

        // Look for untested components
        if description.to_lowercase().contains("coverage") {
            gaps.push("Increase test coverage for critical paths".to_string());
        }

        if description.to_lowercase().contains("unit test") {
            gaps.push("Add missing unit tests for functions".to_string());
        }

        if description.to_lowercase().contains("integration") {
            gaps.push("Add integration tests for component interactions".to_string());
        }

        Ok(gaps)
    }

    /// Identify documentation needs
    fn identify_documentation_needs(
        &self,
        description: &str,
        context: &TaskContext,
    ) -> Result<Vec<String>, DecompositionError> {
        let mut needs = Vec::new();

        if description.to_lowercase().contains("api docs") {
            needs.push("Document public API functions".to_string());
        }

        if description.to_lowercase().contains("readme") {
            needs.push("Update README with usage examples".to_string());
        }

        if description.to_lowercase().contains("comments") {
            needs.push("Add code comments for complex logic".to_string());
        }

        Ok(needs)
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Complexity scorer for decomposition decisions
pub struct ComplexityScorer;

impl ComplexityScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score subtasks for decomposition potential
    pub fn score_subtasks(
        &self,
        task: &ComplexTask,
        patterns: &[TaskPattern],
    ) -> DecompositionResult<SubtaskScores> {
        let mut parallelization_score = 0.0;
        let mut complexity_scores = Vec::new();

        // Analyze patterns for parallelization potential
        for pattern in patterns {
            match pattern {
                TaskPattern::CompilationErrors { error_groups } => {
                    // Compilation errors are highly parallelizable
                    parallelization_score += 0.8;
                    for error_group in error_groups {
                        let score = self.score_error_group(error_group);
                        complexity_scores.push(score);
                    }
                }
                TaskPattern::RefactoringOperations { operations } => {
                    // Refactoring has moderate parallelization potential
                    parallelization_score += 0.6;
                    for operation in operations {
                        complexity_scores.push(operation.complexity);
                    }
                }
                TaskPattern::TestingGaps { .. } => {
                    // Testing can be somewhat parallelized
                    parallelization_score += 0.5;
                    complexity_scores.push(0.4); // Moderate complexity
                }
                TaskPattern::DocumentationNeeds { .. } => {
                    // Documentation is highly parallelizable
                    parallelization_score += 0.9;
                    complexity_scores.push(0.3); // Low complexity
                }
            }
        }

        // Factor in task size and existing complexity score
        parallelization_score *= task.complexity_score;

        // Estimate durations based on complexity scores
        let estimated_durations = complexity_scores.iter()
            .map(|&score| self.estimate_duration(score))
            .collect();

        Ok(SubtaskScores {
            parallelization_score: parallelization_score.min(1.0),
            complexity_scores,
            estimated_durations,
        })
    }

    /// Score an error group for complexity
    fn score_error_group(&self, error_group: &ErrorGroup) -> f32 {
        let base_score = match error_group.error_code.as_str() {
            "E0063" => 0.3, // Missing fields - straightforward
            "E0277" => 0.7, // Trait bounds - complex
            "E0308" => 0.5, // Type mismatch - moderate
            _ => 0.6,       // Default moderate complexity
        };

        // Adjust based on number of affected files
        let file_factor = (error_group.affected_files.len() as f32).sqrt() / 2.0;
        (base_score + file_factor).min(1.0)
    }

    /// Estimate duration based on complexity score
    fn estimate_duration(&self, complexity_score: f32) -> std::time::Duration {
        // Simple heuristic: higher complexity = longer duration
        let base_minutes = 5.0; // 5 minutes base
        let complexity_factor = complexity_score * 10.0; // Up to 10x longer
        let total_minutes = base_minutes + complexity_factor;

        std::time::Duration::from_secs((total_minutes * 60.0) as u64)
    }
}

impl Default for ComplexityScorer {
    fn default() -> Self {
        Self::new()
    }
}
