//! Problem pattern analysis for task decomposition

use crate::parallel_types::*;
use crate::worker_types::{SubTaskId, Priority, TaskScope};
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

        // Analyze task description for patterns
        let description = &task.description;

        // Check for compilation-related patterns
        if self.is_compilation_task(description) {
            let compilation_patterns = self.identify_compilation_patterns(description)?;
            if !compilation_patterns.is_empty() {
                patterns.push(TaskPattern::CompilationErrors { error_groups: compilation_patterns });
            }
        }

        // Check for refactoring patterns
        if self.is_refactoring_task(description) {
            let refactoring_patterns = self.identify_refactoring_patterns(description)?;
            if !refactoring_patterns.is_empty() {
                patterns.push(TaskPattern::RefactoringOperations { operations: refactoring_patterns });
            }
        }

        // Check for testing patterns
        if self.is_testing_task(description) {
            patterns.push(TaskPattern::TestingGaps {
                missing_tests: self.identify_testing_gaps(description)?,
            });
        }

        // Check for documentation patterns
        if self.is_documentation_task(description) {
            patterns.push(TaskPattern::DocumentationNeeds {
                files_needing_docs: self.identify_documentation_needs(description)?,
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
    ) -> Result<Vec<ErrorGroup>, DecompositionError> {
        let mut error_groups = Vec::new();

        // Try to extract error information from description
        let error_codes = self.extract_error_codes(description);

        // Group by error types and create error groups
        for error_code in error_codes {
            // TODO: Implement proper file detection from error messages
            //       Currently uses placeholder file path; should detect actual file paths from error messages and compiler output.
            //
            // COMPLETION CHECKLIST:
            // [ ] Parse error messages for file paths
            // [ ] Extract file paths from compiler output
            // [ ] Map error codes to affected files
            // [ ] Handle multiple files per error
            // [ ] Validate file paths exist
            // [ ] Add unit tests for file detection
            // [ ] Add integration tests with real compiler output
            // [ ] Verify file detection accuracy
            //
            // ACCEPTANCE CRITERIA:
            // - File paths are detected from error messages correctly
            // - Multiple files per error are handled
            // - File paths are validated
            // - File detection works with various error formats
            //
            // DEPENDENCIES:
            // - Error message parsing utilities (Required)
            // - Compiler output parsing utilities (Required)
            // - File path validation utilities (Required)
            //
            // ESTIMATED EFFORT: 3-4 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (error analysis feature)
            // - Change Budget: ~80 LOC
            // - Reviewer Requirements: Compiler output parsing expertise
            error_groups.push(ErrorGroup {
                file_path: format!("unknown_file.rs"), // Temporary: placeholder until file detection is implemented
                // TODO: Count actual error occurrences per file
                //       Currently uses placeholder count; should count actual occurrences of each error code per file.
                //
                // COMPLETION CHECKLIST:
                // [ ] Count error occurrences per file
                // [ ] Aggregate error counts across files
                // [ ] Track error frequency statistics
                // [ ] Handle duplicate error detection
                // [ ] Add unit tests for error counting
                // [ ] Add integration tests with real errors
                // [ ] Verify error counting accuracy
                //
                // ACCEPTANCE CRITERIA:
                // - Error counts are accurate per file
                // - Error frequency is tracked correctly
                // - Duplicate errors are handled appropriately
                // - Error counting reflects actual occurrences
                //
                // DEPENDENCIES:
                // - Error tracking infrastructure (Required)
                // - Error counting utilities (Required)
                // - Error aggregation utilities (Required)
                //
                // ESTIMATED EFFORT: 2-3 hours (medium confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (error analysis feature)
                // - Change Budget: ~50 LOC
                // - Reviewer Requirements: Error tracking expertise
                error_count: 1, // Temporary: placeholder count until actual counting is implemented
                severity: ErrorSeverity::High, // Default severity
                error_code: error_code.clone(),
                count: 1,
                affected_files: vec![format!("unknown_file.rs")], // Temporary: placeholder until file detection
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
        // TODO: Analyze actual error to determine affected files
        //       Currently uses basic heuristic; should analyze actual error message and compiler output to determine affected files.
        //
        // COMPLETION CHECKLIST:
        // [ ] Parse error message for file references
        // [ ] Analyze compiler output for file dependencies
        // [ ] Use static analysis to find affected files
        // [ ] Consider transitive dependencies
        // [ ] Handle various error types appropriately
        // [ ] Add unit tests for file detection
        // [ ] Add integration tests with real errors
        // [ ] Verify file detection accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Affected files are determined from actual error analysis
        // - File dependencies are considered
        // - Transitive dependencies are handled
        // - Various error types are handled correctly
        //
        // DEPENDENCIES:
        // - Error message parsing utilities (Required)
        // - Static analysis utilities (Required)
        // - Dependency analysis utilities (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (error analysis feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Compiler and static analysis expertise
        match error_code { // Temporary: basic heuristic until actual error analysis
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
    ) -> Result<Vec<RefactoringOperation>, DecompositionError> {
        let mut operations = Vec::new();

        // Look for common refactoring patterns in description
        if description.to_lowercase().contains("rename") {
            // TODO: Extract actual file path from description or context
            //       Currently uses placeholder; should extract actual file path from description or task context.
            //
            // COMPLETION CHECKLIST:
            // [ ] Parse description for file path references
            // [ ] Extract file paths from task context
            // [ ] Validate file paths exist
            // [ ] Handle multiple file references
            // [ ] Support relative and absolute paths
            // [ ] Add unit tests for file path extraction
            // [ ] Add integration tests with various descriptions
            // [ ] Verify file path extraction accuracy
            //
            // ACCEPTANCE CRITERIA:
            // - File paths are extracted from description correctly
            // - File paths are validated
            // - Multiple file references are handled
            // - Relative and absolute paths are supported
            //
            // DEPENDENCIES:
            // - Description parsing utilities (Required)
            // - File path extraction utilities (Required)
            // - Path validation utilities (Required)
            //
            // ESTIMATED EFFORT: 2-3 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (refactoring analysis feature)
            // - Change Budget: ~60 LOC
            // - Reviewer Requirements: Text parsing expertise
            operations.push(RefactoringOperation {
                operation_type: "rename".to_string(),
                file_path: "unknown_file.rs".to_string(), // Temporary: placeholder until file path extraction
                complexity: 0.7, // Moderate complexity
                description: "Rename operation".to_string(),
            });
        }

        if description.to_lowercase().contains("extract") {
            operations.push(RefactoringOperation {
                operation_type: "extract".to_string(),
                file_path: "unknown_file.rs".to_string(), // Temporary: placeholder until file path extraction
                complexity: 0.8, // Higher complexity
                description: "Extract method/variable".to_string(),
            });
        }

        if description.to_lowercase().contains("move") {
            operations.push(RefactoringOperation {
                operation_type: "move".to_string(),
                file_path: "unknown_file.rs".to_string(), // Temporary: placeholder until file path extraction
                complexity: 0.6, // Lower complexity
                description: "Move code between modules".to_string(),
            });
        }

        Ok(operations)
    }

    /// Identify testing gaps
    fn identify_testing_gaps(
        &self,
        description: &str,
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
                        complexity_scores.push(score as f64);
                    }
                }
                TaskPattern::RefactoringOperations { operations } => {
                    // Refactoring has moderate parallelization potential
                    parallelization_score += 0.6;
                    for operation in operations {
                        complexity_scores.push(operation.complexity as f64);
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
        let estimated_durations: Vec<std::time::Duration> = complexity_scores.iter()
            .map(|&score| self.estimate_duration(score as f32))
            .collect();

        Ok(SubtaskScores {
            parallelization_score: parallelization_score.min(1.0),
            complexity_scores: complexity_scores.into_iter().map(|s| s as f64).collect(),
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
