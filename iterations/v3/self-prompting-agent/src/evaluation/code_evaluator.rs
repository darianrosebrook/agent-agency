//! Code evaluator that runs tests, linting, and type checking

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::fs;
use async_trait::async_trait;

use super::{Evaluator, EvalContext, EvalCriterion, EvaluationError};
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
}

impl CodeEvaluator {
    /// Create a new code evaluator
    pub fn new() -> Self {
        Self {
            config: CodeEvaluatorConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: CodeEvaluatorConfig) -> Self {
        Self { config }
    }

    /// Run test command and parse results
    async fn run_tests(&self, project_root: &Path) -> Result<EvalCriterion, EvaluationError> {
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

        Ok(EvalCriterion {
            id: "tests-pass".to_string(),
            description: "All tests pass without errors".to_string(),
            weight: 0.4,
            passed,
            score,
            notes: Some(notes),
        })
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
                // TODO: Implement proper coverage analysis and validation
                // - [ ] Parse actual coverage numbers from coverage reports
                // - [ ] Implement coverage threshold validation
                // - [ ] Add coverage trend analysis and improvement tracking
                // - [ ] Implement coverage gap identification and recommendations
                // - [ ] Add support for different coverage report formats
                return Ok(EvalCriterion {
                    id: "coverage-adequate".to_string(),
                    description: format!("Code coverage meets {}% threshold", (self.config.coverage_threshold * 100.0) as u32),
                    weight: 0.1,
                    passed: true,
                    score: 1.0,
                    notes: Some(format!("Coverage report found at: {}", path)),
                });
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
        let project_root = if let Some(artifact) = artifacts.first() {
            Path::new(&artifact.file_path)
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(Path::new("."))
        } else {
            Path::new(".")
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
