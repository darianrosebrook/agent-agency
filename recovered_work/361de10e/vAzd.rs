//! Quality Gates Implementation
//!
//! Individual quality gate definitions and execution logic for linting,
//! testing, coverage, mutation analysis, and CAWS compliance.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::timeout;

use crate::planning::types::ExecutionArtifacts;

/// Quality gate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
    Error,
}

/// Quality gate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub name: String,
    pub status: GateStatus,
    pub score: f64,
    pub threshold: f64,
    pub duration_ms: u64,
    pub details: serde_json::Value,
    pub errors: Vec<String>,
}

/// Quality thresholds by risk tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub lint_errors_max: u32,
    pub type_errors_max: u32,
    pub test_failure_max: u32,
    pub coverage_min: f64,
    pub mutation_score_min: f64,
    pub caws_violations_max: u32,
}

/// Quality gate trait
#[async_trait]
pub trait QualityGate: Send + Sync {
    /// Get the gate name
    fn name(&self) -> &str;

    /// Check if this gate should run for the given artifacts
    fn should_run(&self, artifacts: &ExecutionArtifacts) -> bool;

    /// Execute the quality gate
    async fn execute(
        &self,
        artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<QualityGateResult, QualityGateError>;
}

/// CAWS compliance gate
pub struct CawsComplianceGate {
    validator: Arc<dyn crate::caws_runtime::CawsRuntimeValidator>,
}

impl CawsComplianceGate {
    pub fn new(validator: Arc<dyn crate::caws_runtime::CawsRuntimeValidator>) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl QualityGate for CawsComplianceGate {
    fn name(&self) -> &str {
        "caws_compliance"
    }

    fn should_run(&self, _artifacts: &ExecutionArtifacts) -> bool {
        true // Always run CAWS compliance checks
    }

    async fn execute(
        &self,
        artifacts: &ExecutionArtifacts,
        _working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<QualityGateResult, QualityGateError> {
        let start_time = std::time::Instant::now();

        // Create a mock task descriptor for validation
        let task_desc = crate::caws_runtime::TaskDescriptor {
            task_id: format!("quality-gate-{}", artifacts.task_id),
            scope_in: vec![".".to_string()], // Assume current directory scope
            risk_tier: 2, // Default to high risk tier
            acceptance: None,
            metadata: Some(serde_json::json!({
                "quality_gate": true,
                "artifacts_id": artifacts.id,
            })),
        };

        // Create diff stats from artifacts
        let diff_stats = crate::caws_runtime::DiffStats {
            files_changed: artifacts.code_changes.len() as u32,
            lines_changed: artifacts.code_changes.iter()
                .map(|c| c.lines_added + c.lines_removed)
                .sum(),
            touched_paths: artifacts.code_changes.iter()
                .map(|c| c.file_path.clone())
                .collect(),
        };

        // Run CAWS validation with timeout
        let validation_result = timeout(
            Duration::from_secs(30),
            self.validator.validate(
                &crate::caws_runtime::WorkingSpec {
                    risk_tier: 2,
                    scope_in: vec![".".to_string()],
                    change_budget_max_files: 50,
                    change_budget_max_loc: 1000,
                },
                &task_desc,
                &diff_stats,
                &[], // no patches
                &[], // no language hints
                artifacts.test_results.total > 0, // tests added
                true, // assume deterministic
                vec![], // no waivers
            )
        ).await.map_err(|_| QualityGateError::Timeout)??;

        let duration = start_time.elapsed().as_millis() as u64;
        let violations_count = validation_result.violations.len() as u32;

        let (status, score) = if violations_count == 0 {
            (GateStatus::Passed, 1.0)
        } else if violations_count <= thresholds.caws_violations_max {
            (GateStatus::Warning, 0.7)
        } else {
            (GateStatus::Failed, 0.0)
        };

        let details = serde_json::json!({
            "violations_count": violations_count,
            "max_violations": thresholds.caws_violations_max,
            "violations": validation_result.violations.iter()
                .map(|v| serde_json::json!({
                    "code": v.code,
                    "message": v.message,
                    "remediation": v.remediation
                }))
                .collect::<Vec<_>>()
        });

        Ok(QualityGateResult {
            name: self.name().to_string(),
            status,
            score,
            threshold: thresholds.caws_violations_max as f64,
            duration_ms: duration,
            details,
            errors: vec![], // CAWS validator provides structured violations
        })
    }
}

/// Linting gate
pub struct LintingGate {
    linter_command: Vec<String>,
    config_file: Option<String>,
}

impl LintingGate {
    pub fn new(linter_command: Vec<String>, config_file: Option<String>) -> Self {
        Self {
            linter_command,
            config_file,
        }
    }

    /// Create a JavaScript/TypeScript linting gate
    pub fn eslint() -> Self {
        Self::new(
            vec!["npx".to_string(), "eslint".to_string(), "--format=json".to_string(), ".".to_string()],
            Some(".eslintrc.js".to_string()),
        )
    }

    /// Create a Rust linting gate
    pub fn clippy() -> Self {
        Self::new(
            vec!["cargo".to_string(), "clippy".to_string(), "--message-format=json".to_string()],
            None,
        )
    }
}

#[async_trait]
impl QualityGate for LintingGate {
    fn name(&self) -> &str {
        "linting"
    }

    fn should_run(&self, artifacts: &ExecutionArtifacts) -> bool {
        // Run if there are code changes
        !artifacts.code_changes.is_empty()
    }

    async fn execute(
        &self,
        artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<QualityGateResult, QualityGateError> {
        let start_time = std::time::Instant::now();

        // Check if config file exists
        if let Some(config_file) = &self.config_file {
            let config_path = working_dir.join(config_file);
            if !config_path.exists() {
                return Ok(QualityGateResult {
                    name: self.name().to_string(),
                    status: GateStatus::Skipped,
                    score: 0.5,
                    threshold: thresholds.lint_errors_max as f64,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    details: serde_json::json!({"reason": "config file not found"}),
                    errors: vec![format!("Config file {} not found", config_file)],
                });
            }
        }

        // Run linter command
        let output = timeout(
            Duration::from_secs(60),
            Command::new(&self.linter_command[0])
                .args(&self.linter_command[1..])
                .current_dir(working_dir)
                .output()
        ).await.map_err(|_| QualityGateError::Timeout)??;

        let duration = start_time.elapsed().as_millis() as u64;

        if let Some(code) = output.status.code() {
            if code != 0 && !output.stderr.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Ok(QualityGateResult {
                    name: self.name().to_string(),
                    status: GateStatus::Error,
                    score: 0.0,
                    threshold: thresholds.lint_errors_max as f64,
                    duration_ms: duration,
                    details: serde_json::json!({"exit_code": code}),
                    errors: vec![stderr.to_string()],
                });
            }
        }

        // Parse output to count errors/warnings
        let stdout = String::from_utf8_lossy(&output.stdout);
        let (error_count, warning_count) = self.parse_lint_output(&stdout)?;

        let (status, score) = if error_count == 0 && warning_count == 0 {
            (GateStatus::Passed, 1.0)
        } else if error_count <= thresholds.lint_errors_max {
            (GateStatus::Warning, 0.8)
        } else {
            (GateStatus::Failed, 0.0)
        };

        let details = serde_json::json!({
            "errors": error_count,
            "warnings": warning_count,
            "max_errors": thresholds.lint_errors_max
        });

        Ok(QualityGateResult {
            name: self.name().to_string(),
            status,
            score,
            threshold: thresholds.lint_errors_max as f64,
            duration_ms: duration,
            details,
            errors: vec![],
        })
    }
}

impl LintingGate {
    /// Parse lint output to extract error and warning counts
    fn parse_lint_output(&self, output: &str) -> Result<(u32, u32), QualityGateError> {
        // Simple parsing - in practice this would be more sophisticated
        // based on the specific linter output format
        let mut errors = 0;
        let mut warnings = 0;

        for line in output.lines() {
            if line.to_lowercase().contains("error") {
                errors += 1;
            } else if line.to_lowercase().contains("warning") {
                warnings += 1;
            }
        }

        Ok((errors, warnings))
    }
}

/// Testing gate
pub struct TestingGate {
    test_command: Vec<String>,
    timeout_seconds: u64,
}

impl TestingGate {
    pub fn new(test_command: Vec<String>, timeout_seconds: u64) -> Self {
        Self {
            test_command,
            timeout_seconds,
        }
    }

    /// Create a JavaScript/TypeScript testing gate
    pub fn jest() -> Self {
        Self::new(
            vec!["npm".to_string(), "test".to_string()],
            300, // 5 minutes
        )
    }

    /// Create a Rust testing gate
    pub fn cargo_test() -> Self {
        Self::new(
            vec!["cargo".to_string(), "test".to_string()],
            600, // 10 minutes
        )
    }
}

#[async_trait]
impl QualityGate for TestingGate {
    fn name(&self) -> &str {
        "testing"
    }

    fn should_run(&self, artifacts: &ExecutionArtifacts) -> bool {
        // Run if there are test results or code changes
        artifacts.test_results.total > 0 || !artifacts.code_changes.is_empty()
    }

    async fn execute(
        &self,
        artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<QualityGateResult, QualityGateError> {
        let start_time = std::time::Instant::now();

        // Run test command
        let output = timeout(
            Duration::from_secs(self.timeout_seconds),
            Command::new(&self.test_command[0])
                .args(&self.test_command[1..])
                .current_dir(working_dir)
                .output()
        ).await.map_err(|_| QualityGateError::Timeout)??;

        let duration = start_time.elapsed().as_millis() as u64;

        // Parse test results
        let (passed, failed, total) = if !output.stdout.is_empty() {
            self.parse_test_output(&String::from_utf8_lossy(&output.stdout))?
        } else {
            // Use artifact data if tests already ran
            (artifacts.test_results.passed, artifacts.test_results.failed, artifacts.test_results.total)
        };

        let (status, score) = if failed == 0 && passed > 0 {
            (GateStatus::Passed, 1.0)
        } else if failed <= thresholds.test_failure_max {
            (GateStatus::Warning, 0.7)
        } else {
            (GateStatus::Failed, 0.0)
        };

        let details = serde_json::json!({
            "passed": passed,
            "failed": failed,
            "total": total,
            "max_failures": thresholds.test_failure_max
        });

        Ok(QualityGateResult {
            name: self.name().to_string(),
            status,
            score,
            threshold: thresholds.test_failure_max as f64,
            duration_ms: duration,
            details,
            errors: vec![],
        })
    }
}

impl TestingGate {
    /// Parse test output to extract pass/fail counts
    fn parse_test_output(&self, output: &str) -> Result<(u32, u32, u32), QualityGateError> {
        // Simple parsing - in practice this would be more sophisticated
        // based on the specific test framework output format
        let mut passed = 0;
        let mut failed = 0;

        for line in output.lines() {
            if line.to_lowercase().contains("pass") || line.to_lowercase().contains("ok") {
                passed += 1;
            } else if line.to_lowercase().contains("fail") || line.to_lowercase().contains("error") {
                failed += 1;
            }
        }

        let total = passed + failed;
        Ok((passed, failed, total))
    }
}

/// Coverage gate
pub struct CoverageGate {
    coverage_command: Vec<String>,
    coverage_file: String,
}

impl CoverageGate {
    pub fn new(coverage_command: Vec<String>, coverage_file: String) -> Self {
        Self {
            coverage_command,
            coverage_file,
        }
    }

    /// Create a JavaScript/TypeScript coverage gate
    pub fn istanbul() -> Self {
        Self::new(
            vec!["npx".to_string(), "nyc".to_string(), "report".to_string()],
            "coverage/coverage-summary.json".to_string(),
        )
    }

    /// Create a Rust coverage gate
    pub fn tarpaulin() -> Self {
        Self::new(
            vec!["cargo".to_string(), "tarpaulin".to_string(), "--out".to_string(), "Json".to_string()],
            "tarpaulin-report.json".to_string(),
        )
    }
}

#[async_trait]
impl QualityGate for CoverageGate {
    fn name(&self) -> &str {
        "coverage"
    }

    fn should_run(&self, artifacts: &ExecutionArtifacts) -> bool {
        // Run if there are test results or code changes
        artifacts.test_results.total > 0 || !artifacts.code_changes.is_empty()
    }

    async fn execute(
        &self,
        artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<QualityGateResult, QualityGateError> {
        let start_time = std::time::Instant::now();

        // Run coverage command
        let output = timeout(
            Duration::from_secs(120),
            Command::new(&self.coverage_command[0])
                .args(&self.coverage_command[1..])
                .current_dir(working_dir)
                .output()
        ).await.map_err(|_| QualityGateError::Timeout)??;

        let duration = start_time.elapsed().as_millis() as u64;

        // Try to read coverage file, fallback to artifact data
        let coverage_percentage = if let Ok(coverage_data) = tokio::fs::read_to_string(working_dir.join(&self.coverage_file)).await {
            self.parse_coverage_file(&coverage_data)?
        } else {
            artifacts.coverage.coverage_percentage
        };

        let (status, score) = if coverage_percentage >= thresholds.coverage_min {
            (GateStatus::Passed, coverage_percentage / 100.0)
        } else if coverage_percentage >= thresholds.coverage_min * 0.8 {
            (GateStatus::Warning, coverage_percentage / 100.0)
        } else {
            (GateStatus::Failed, coverage_percentage / 100.0)
        };

        let details = serde_json::json!({
            "coverage_percentage": coverage_percentage,
            "min_coverage": thresholds.coverage_min
        });

        Ok(QualityGateResult {
            name: self.name().to_string(),
            status,
            score,
            threshold: thresholds.coverage_min,
            duration_ms: duration,
            details,
            errors: vec![],
        })
    }
}

impl CoverageGate {
    /// Parse coverage file to extract percentage
    fn parse_coverage_file(&self, content: &str) -> Result<f64, QualityGateError> {
        // Simple JSON parsing - in practice this would be more robust
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(total) = json.get("total") {
                if let Some(lines) = total.get("lines") {
                    if let Some(pct) = lines.get("pct").and_then(|v| v.as_f64()) {
                        return Ok(pct);
                    }
                }
            }
        }

        // Fallback: extract percentage from text
        for line in content.lines() {
            if line.contains('%') {
                if let Some(pct_str) = line.split('%').next() {
                    if let Ok(pct) = pct_str.trim().parse::<f64>() {
                        return Ok(pct);
                    }
                }
            }
        }

        Err(QualityGateError::ParseError("Could not parse coverage percentage".to_string()))
    }
}

/// Mutation testing gate
pub struct MutationGate {
    mutation_command: Vec<String>,
    timeout_seconds: u64,
}

impl MutationGate {
    pub fn new(mutation_command: Vec<String>, timeout_seconds: u64) -> Self {
        Self {
            mutation_command,
            timeout_seconds,
        }
    }

    /// Create a Stryker mutation testing gate
    pub fn stryker() -> Self {
        Self::new(
            vec!["npx".to_string(), "stryker".to_string(), "run".to_string()],
            1800, // 30 minutes
        )
    }

    /// Create a Rust mutation testing gate
    pub fn cargo_mutants() -> Self {
        Self::new(
            vec!["cargo".to_string(), "mutants".to_string()],
            3600, // 1 hour
        )
    }
}

#[async_trait]
impl QualityGate for MutationGate {
    fn name(&self) -> &str {
        "mutation"
    }

    fn should_run(&self, artifacts: &ExecutionArtifacts) -> bool {
        // Run if there are code changes and tests
        !artifacts.code_changes.is_empty() && artifacts.test_results.total > 0
    }

    async fn execute(
        &self,
        artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<QualityGateResult, QualityGateError> {
        let start_time = std::time::Instant::now();

        // Run mutation testing command
        let output = timeout(
            Duration::from_secs(self.timeout_seconds),
            Command::new(&self.mutation_command[0])
                .args(&self.mutation_command[1..])
                .current_dir(working_dir)
                .output()
        ).await.map_err(|_| QualityGateError::Timeout)??;

        let duration = start_time.elapsed().as_millis() as u64;

        // Parse mutation score, fallback to artifact data
        let mutation_score = if !output.stdout.is_empty() {
            self.parse_mutation_output(&String::from_utf8_lossy(&output.stdout))?
        } else {
            artifacts.mutation.mutation_score
        };

        let (status, score) = if mutation_score >= thresholds.mutation_score_min {
            (GateStatus::Passed, mutation_score)
        } else if mutation_score >= thresholds.mutation_score_min * 0.8 {
            (GateStatus::Warning, mutation_score)
        } else {
            (GateStatus::Failed, mutation_score)
        };

        let details = serde_json::json!({
            "mutation_score": mutation_score,
            "min_score": thresholds.mutation_score_min
        });

        Ok(QualityGateResult {
            name: self.name().to_string(),
            status,
            score,
            threshold: thresholds.mutation_score_min,
            duration_ms: duration,
            details,
            errors: vec![],
        })
    }
}

impl MutationGate {
    /// Parse mutation testing output to extract score
    fn parse_mutation_output(&self, output: &str) -> Result<f64, QualityGateError> {
        // Simple parsing - look for percentage patterns
        for line in output.lines() {
            if line.to_lowercase().contains("score") || line.contains('%') {
                if let Some(pct_str) = line.split('%').next() {
                    if let Ok(pct) = pct_str.trim().parse::<f64>() {
                        return Ok(pct);
                    }
                }
            }
        }

        Err(QualityGateError::ParseError("Could not parse mutation score".to_string()))
    }
}

/// Type checking gate
pub struct TypeCheckGate {
    type_check_command: Vec<String>,
}

impl TypeCheckGate {
    pub fn new(type_check_command: Vec<String>) -> Self {
        Self { type_check_command }
    }

    /// Create a TypeScript type checking gate
    pub fn tsc() -> Self {
        Self::new(vec!["npx".to_string(), "tsc".to_string(), "--noEmit".to_string()])
    }

    /// Create a Rust type checking gate
    pub fn cargo_check() -> Self {
        Self::new(vec!["cargo".to_string(), "check".to_string()])
    }
}

#[async_trait]
impl QualityGate for TypeCheckGate {
    fn name(&self) -> &str {
        "type_check"
    }

    fn should_run(&self, artifacts: &ExecutionArtifacts) -> bool {
        // Run if there are code changes
        !artifacts.code_changes.is_empty()
    }

    async fn execute(
        &self,
        _artifacts: &ExecutionArtifacts,
        working_dir: &PathBuf,
        thresholds: &QualityThresholds,
    ) -> Result<QualityGateResult, QualityGateError> {
        let start_time = std::time::Instant::now();

        // Run type checking command
        let output = timeout(
            Duration::from_secs(120),
            Command::new(&self.type_check_command[0])
                .args(&self.type_check_command[1..])
                .current_dir(working_dir)
                .output()
        ).await.map_err(|_| QualityGateError::Timeout)??;

        let duration = start_time.elapsed().as_millis() as u64;

        let error_count = if output.status.success() {
            0
        } else {
            // Count error lines in output
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .filter(|line| line.to_lowercase().contains("error"))
                .count() as u32
        };

        let (status, score) = if error_count == 0 {
            (GateStatus::Passed, 1.0)
        } else if error_count <= thresholds.type_errors_max {
            (GateStatus::Warning, 0.8)
        } else {
            (GateStatus::Failed, 0.0)
        };

        let details = serde_json::json!({
            "errors": error_count,
            "max_errors": thresholds.type_errors_max
        });

        Ok(QualityGateResult {
            name: self.name().to_string(),
            status,
            score,
            threshold: thresholds.type_errors_max as f64,
            duration_ms: duration,
            details,
            errors: if error_count > 0 {
                vec![String::from_utf8_lossy(&output.stderr).to_string()]
            } else {
                vec![]
            },
        })
    }
}

pub type Result<T> = std::result::Result<T, QualityGateError>;

#[derive(Debug, thiserror::Error)]
pub enum QualityGateError {
    #[error("Quality gate execution timed out")]
    Timeout,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Command execution failed: {0}")]
    CommandError(String),

    #[error("Output parsing failed: {0}")]
    ParseError(String),

    #[error("Quality gate not supported")]
    UnsupportedGate,
}


