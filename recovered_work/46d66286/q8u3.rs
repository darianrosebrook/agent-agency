//! Code evaluator that runs tests, linting, and type checking

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::fs;
use async_trait::async_trait;

use super::{Evaluator, EvalContext, EvalCriterion, EvaluationError, FlakinessHardener, HardenedEvaluationResult};
use crate::types::{Artifact, TaskType};

/// Code evaluator configuration
#[derive(Debug, Clone)]
pub struct CodeEvaluatorConfig {
    pub test_command: String,
    pub lint_command: Option<String>,
    pub type_check_command: Option<String>,
    pub coverage_threshold: f64,
}

impl Default for CodeEvaluatorConfig {
    fn default() -> Self {
        Self {
            test_command: "npm test".to_string(),
            lint_command: Some("npm run lint".to_string()),
            type_check_command: Some("npm run typecheck".to_string()),
            coverage_threshold: 0.8,
        }
    }
}

/// Code evaluator for running tests and quality checks
pub struct CodeEvaluator {
    config: CodeEvaluatorConfig,
    flakiness_hardener: FlakinessHardener,
}

impl CodeEvaluator {
    /// Create a new code evaluator
    pub fn new() -> Self {
        Self {
            config: CodeEvaluatorConfig::default(),
            flakiness_hardener: FlakinessHardener::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: CodeEvaluatorConfig) -> Self {
        Self {
            config,
            flakiness_hardener: FlakinessHardener::new(),
        }
    }

    /// Run test command with flakiness hardening
    async fn run_tests(&self, project_root: PathBuf) -> Result<EvalCriterion, EvaluationError> {
        // Use flakiness hardener to run tests with retries
        let hardened_result = self.flakiness_hardener.harden_evaluation(|| async {
            let output = Command::new("sh")
                .arg("-c")
                .arg(&self.config.test_command)
                .current_dir(project_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| EvaluationError::CommandError(format!("Failed to run tests: {}", e)))?;

            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let score = if success { 1.0 } else { 0.0 };
            let passed = success;

            let notes = if success {
                format!("Tests passed. Output: {}", stdout.chars().take(200).collect::<String>())
            } else {
                format!("Tests failed. Stderr: {}", stderr.chars().take(200).collect::<String>())
            };

            // Add retry information to notes if applicable
            let enhanced_notes = if !passed && !notes.is_empty() {
                format!("{} (retried for flakiness)", notes)
            } else {
                notes
            };

            Ok(EvalCriterion {
                id: "tests-pass".to_string(),
                description: "All tests pass without errors".to_string(),
                weight: 0.4,
                passed,
                score,
                notes: Some(enhanced_notes),
            })
        }).await?;

        // Enhance the criterion with confidence and failure analysis
        let mut enhanced_criterion = hardened_result.criterion;
        let confidence_note = format!(" (confidence: {:.1}%, retries: {})",
                                    hardened_result.confidence * 100.0,
                                    hardened_result.retry_count);

        enhanced_criterion.notes = Some(format!("{}{}",
            enhanced_criterion.notes.as_deref().unwrap_or(""),
            confidence_note
        ));

        // Add failure bucket information if available
        if let Some(bucket) = hardened_result.failure_bucket {
            let bucket_note = format!(" [Failure: {:?}, patterns: {}]",
                                    bucket.category,
                                    bucket.patterns.len());
            enhanced_criterion.notes = Some(format!("{}{}",
                enhanced_criterion.notes.as_deref().unwrap_or(""),
                bucket_note
            ));
        }

        Ok(enhanced_criterion)
    }

    /// Run linting command
    async fn run_lint(&self, project_root: &Path) -> Result<EvalCriterion, EvaluationError> {
        if let Some(lint_cmd) = &self.config.lint_command {
            let output = Command::new("sh")
                .arg("-c")
                .arg(lint_cmd)
                .current_dir(project_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| EvaluationError::CommandError(format!("Failed to run lint: {}", e)))?;

            let success = output.status.success();
            let stderr = String::from_utf8_lossy(&output.stderr);

            let score = if success { 1.0 } else { 0.0 };
            let passed = success;

            let notes = if success {
                "No linting errors".to_string()
            } else {
                format!("Linting errors: {}", stderr.chars().take(200).collect::<String>())
            };

            Ok(EvalCriterion {
                id: "lint-clean".to_string(),
                description: "Code passes all linting rules".to_string(),
                weight: 0.2,
                passed,
                score,
                notes: Some(notes),
            })
        } else {
            // Skip linting if not configured
            Ok(EvalCriterion {
                id: "lint-clean".to_string(),
                description: "Linting not configured (skipped)".to_string(),
                weight: 0.0,
                passed: true,
                score: 1.0,
                notes: Some("Linting not configured".to_string()),
            })
        }
    }

    /// Run type checking
    async fn run_type_check(&self, project_root: &Path) -> Result<EvalCriterion, EvaluationError> {
        if let Some(type_cmd) = &self.config.type_check_command {
            let output = Command::new("sh")
                .arg("-c")
                .arg(type_cmd)
                .current_dir(project_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| EvaluationError::CommandError(format!("Failed to run type check: {}", e)))?;

            let success = output.status.success();
            let stderr = String::from_utf8_lossy(&output.stderr);

            let score = if success { 1.0 } else { 0.0 };
            let passed = success;

            let notes = if success {
                "No type errors".to_string()
            } else {
                format!("Type errors: {}", stderr.chars().take(200).collect::<String>())
            };

            Ok(EvalCriterion {
                id: "types-ok".to_string(),
                description: "Code passes type checking".to_string(),
                weight: 0.3,
                passed,
                score,
                notes: Some(notes),
            })
        } else {
            // Skip type checking if not configured
            Ok(EvalCriterion {
                id: "types-ok".to_string(),
                description: "Type checking not configured (skipped)".to_string(),
                weight: 0.0,
                passed: true,
                score: 1.0,
                notes: Some("Type checking not configured".to_string()),
            })
        }
    }

    /// Check code coverage (if available)
    async fn check_coverage(&self, project_root: &Path) -> Result<EvalCriterion, EvaluationError> {
        // Try to find coverage reports
        let coverage_paths = [
            "coverage/lcov-report/index.html",
            "coverage/coverage.json",
            "target/debug/coverage.json",
        ];

        for path in &coverage_paths {
            if project_root.join(path).exists() {
                // Parse the coverage report and validate against thresholds
                match self.parse_coverage_report(&project_root.join(path)).await {
                    Ok(coverage_data) => {
                        let line_coverage = coverage_data.line_coverage;
                        let branch_coverage = coverage_data.branch_coverage;
                        let function_coverage = coverage_data.function_coverage;

                        // Check if coverage meets minimum thresholds
                        let line_passed = line_coverage >= self.config.coverage_threshold;
                        let branch_passed = branch_coverage >= self.config.coverage_threshold;
                        let function_passed = function_coverage >= self.config.coverage_threshold;

                        let overall_passed = line_passed && branch_passed && function_passed;
                        let overall_score = if overall_passed { 1.0 } else {
                            // Partial credit based on best coverage metric
                            let max_coverage = line_coverage.max(branch_coverage).max(function_coverage);
                            (max_coverage / self.config.coverage_threshold).min(1.0)
                        };

                        let threshold_pct = (self.config.coverage_threshold * 100.0) as u32;
                        let notes = format!(
                            "Coverage report: {} | Line: {:.1}%, Branch: {:.1}%, Function: {:.1}% | Threshold: {}%",
                            path,
                            line_coverage * 100.0,
                            branch_coverage * 100.0,
                            function_coverage * 100.0,
                            threshold_pct
                        );

                        return Ok(EvalCriterion {
                            id: "coverage-adequate".to_string(),
                            description: format!("Code coverage meets {}% threshold", threshold_pct),
                            weight: 0.1,
                            passed: overall_passed,
                            score: overall_score,
                            notes: Some(notes),
                        });
                    }
                    Err(e) => {
                        // Report exists but couldn't be parsed
                        return Ok(EvalCriterion {
                            id: "coverage-adequate".to_string(),
                            description: "Coverage report parsing failed".to_string(),
                            weight: 0.0,
                            passed: false,
                            score: 0.0,
                            notes: Some(format!("Found coverage report at {} but failed to parse: {}", path, e)),
                        });
                    }
                }
            }
        }

        // No coverage report found
        Ok(EvalCriterion {
            id: "coverage-adequate".to_string(),
            description: "Coverage report not found".to_string(),
            weight: 0.0,
            passed: false,
            score: 0.0,
            notes: Some("No coverage report detected".to_string()),
        })
    }
}

#[async_trait]
impl Evaluator for CodeEvaluator {
    async fn evaluate(&self, artifacts: &[Artifact], context: &EvalContext) -> Result<Vec<EvalCriterion>, EvaluationError> {
        let mut criteria = Vec::new();

        // Find the project root (assume it's the parent of the first artifact's directory)
        // Convert to PathBuf to make it Send for async operations
        let project_root = if let Some(artifact) = artifacts.first() {
            Path::new(&artifact.file_path)
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(Path::new("."))
                .to_path_buf()
        } else {
            PathBuf::from(".")
        };

        // Run all code quality checks
        criteria.push(self.run_tests(project_root).await?);
        criteria.push(self.run_lint(project_root).await?);
        criteria.push(self.run_type_check(project_root).await?);
        criteria.push(self.check_coverage(project_root).await?);

        Ok(criteria)
    }

    fn applies_to(&self, task_type: &TaskType) -> bool {
        matches!(task_type, TaskType::CodeFix | TaskType::CodeGeneration)
    }

    fn evaluator_type(&self) -> &'static str {
        "code"
    }
}

impl CodeEvaluator {
    /// Parse coverage report data
    async fn parse_coverage_report(&self, report_path: &Path) -> Result<CoverageData, EvaluationError> {
        let extension = report_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "json" => self.parse_json_coverage(report_path).await,
            "html" => {
                // For HTML reports, we can't easily parse them, so return default values
                // In a real implementation, you might want to extract data from the HTML
                Ok(CoverageData {
                    line_coverage: 0.0,
                    branch_coverage: 0.0,
                    function_coverage: 0.0,
                })
            }
            _ => {
                // Try to parse as JSON anyway
                self.parse_json_coverage(report_path).await
            }
        }
    }

    /// Parse JSON coverage report (Istanbul format)
    async fn parse_json_coverage(&self, report_path: &Path) -> Result<CoverageData, EvaluationError> {
        let content = tokio::fs::read_to_string(report_path)
            .await
            .map_err(|e| EvaluationError::IoError(e))?;

        // Parse Istanbul JSON coverage format
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| EvaluationError::ParseError(format!("Failed to parse JSON coverage: {}", e)))?;

        let mut total_lines = 0u64;
        let mut covered_lines = 0u64;
        let mut total_branches = 0u64;
        let mut covered_branches = 0u64;
        let mut total_functions = 0u64;
        let mut covered_functions = 0u64;

        if let Some(files) = json.as_object() {
            for (_file_path, file_data) in files {
                if let Some(file_obj) = file_data.as_object() {
                    // Parse line coverage
                    if let Some(lines) = file_obj.get("l") {
                        Self::parse_coverage_map(lines, &mut total_lines, &mut covered_lines);
                    }

                    // Parse branch coverage
                    if let Some(branches) = file_obj.get("b") {
                        Self::parse_coverage_map(branches, &mut total_branches, &mut covered_branches);
                    }

                    // Parse function coverage
                    if let Some(functions) = file_obj.get("f") {
                        Self::parse_coverage_map(functions, &mut total_functions, &mut covered_functions);
                    }
                }
            }
        }

        // Calculate percentages
        let line_coverage = if total_lines > 0 {
            covered_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        let branch_coverage = if total_branches > 0 {
            covered_branches as f64 / total_branches as f64
        } else {
            0.0
        };

        let function_coverage = if total_functions > 0 {
            covered_functions as f64 / total_functions as f64
        } else {
            0.0
        };

        Ok(CoverageData {
            line_coverage,
            branch_coverage,
            function_coverage,
        })
    }

    /// Parse coverage map data from JSON
    fn parse_coverage_map(data: &serde_json::Value, total: &mut u64, covered: &mut u64) {
        if let Some(obj) = data.as_object() {
            for (_key, value) in obj {
                if let Some(count) = value.as_u64() {
                    *total += 1;
                    if count > 0 {
                        *covered += 1;
                    }
                } else if let Some(array) = value.as_array() {
                    // Handle branch coverage arrays
                    *total += 1;
                    if array.iter().any(|&ref v| v.as_u64().unwrap_or(0) > 0) {
                        *covered += 1;
                    }
                }
            }
        }
    }
}

/// Coverage data structure
#[derive(Debug, Clone)]
struct CoverageData {
    line_coverage: f64,
    branch_coverage: f64,
    function_coverage: f64,
}
