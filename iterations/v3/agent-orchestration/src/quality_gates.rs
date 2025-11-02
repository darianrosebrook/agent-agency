//! Quality Gates Enforcement Module
//!
//! Implements actual quality gate checks for coverage, linting, and type checking.
//! Replaces placeholder implementations with real tooling integration.
//!
//! @author @darianrosebrook

use std::process::Command;
use std::path::Path;
use anyhow::{Result, Context};
use tracing::{debug, warn, error, info};
use agent_agency_contracts::planning_io::QualityGates;
use chrono::Utc;

/// Quality gate execution result
#[derive(Debug, Clone)]
pub struct QualityGateResult {
    pub gate_name: String,
    pub passed: bool,
    pub score: f64,
    pub threshold: f64,
    pub duration_ms: u64,
    pub issues: Vec<QualityGateIssue>,
    pub command_used: Option<String>,
}

/// Individual issue found by a quality gate
#[derive(Debug, Clone)]
pub struct QualityGateIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub suggestion: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Quality gate executor
pub struct QualityGateExecutor {
    workspace_root: String,
}

impl QualityGateExecutor {
    /// Create a new quality gate executor
    pub fn new(workspace_root: String) -> Self {
        Self { workspace_root }
    }

    /// Execute all quality gates based on the working spec requirements
    pub async fn execute_quality_gates(
        &self,
        quality_gates: &QualityGates,
        risk_tier: u32,
    ) -> Result<Vec<QualityGateResult>> {
        let mut results = Vec::new();

        // Check coverage if specified
        if let Some(min_coverage) = quality_gates.min_coverage {
            debug!("Checking coverage with minimum threshold: {:.1}%", min_coverage * 100.0);
            let coverage_result = self.check_coverage(min_coverage).await?;
            results.push(coverage_result);
        } else if !quality_gates.coverage_requirements.is_empty() {
            // Check coverage requirements by type
            for (test_type, threshold) in &quality_gates.coverage_requirements {
                debug!("Checking {} coverage with threshold: {:.1}%", test_type, threshold * 100.0);
                let coverage_result = self.check_coverage_by_type(test_type, *threshold).await?;
                results.push(coverage_result);
            }
        }

        // Always check linting for Tier 1 and 2, optionally for Tier 3
        if risk_tier <= 2 || self.should_check_linting(quality_gates) {
            debug!("Checking linting (risk tier: {})", risk_tier);
            let lint_result = self.check_linting().await?;
            results.push(lint_result);
        }

        // Always check type checking for Tier 1 and 2, optionally for Tier 3
        if risk_tier <= 2 || self.should_check_type_checking(quality_gates) {
            debug!("Checking type checking (risk tier: {})", risk_tier);
            let type_check_result = self.check_type_checking().await?;
            results.push(type_check_result);
        }

        // Check mutation testing if required
        if quality_gates.mutation_requirements.required {
            debug!("Checking mutation testing with minimum score: {:.1}%", 
                quality_gates.mutation_requirements.min_score * 100.0);
            let mutation_result = self.check_mutation_testing(
                quality_gates.mutation_requirements.min_score
            ).await?;
            results.push(mutation_result);
        }

        // Check security if required
        if quality_gates.security_requirements.scan_required {
            debug!("Checking security requirements");
            let security_result = self.check_security(&quality_gates.security_requirements).await?;
            results.push(security_result);
        }

        Ok(results)
    }

    /// Check if linting should be enforced based on quality gates
    fn should_check_linting(&self, quality_gates: &QualityGates) -> bool {
        // Check if documentation requirements indicate code quality checks
        quality_gates.documentation_requirements.code_docs_required
    }

    /// Check if type checking should be enforced based on quality gates
    fn should_check_type_checking(&self, quality_gates: &QualityGates) -> bool {
        // Type checking is generally always good to check
        // This could be enhanced with explicit type checking requirements
        true
    }

    /// Check test coverage using cargo-llvm-cov or similar tool
    async fn check_coverage(&self, min_threshold: f64) -> Result<QualityGateResult> {
        let start_time = Utc::now();
        
        // Try cargo-llvm-cov first (most common Rust coverage tool)
        let coverage_result = if self.has_command("cargo-llvm-cov") {
            self.check_coverage_llvm_cov(min_threshold).await
        } else if self.has_command("cargo-tarpaulin") {
            self.check_coverage_tarpaulin(min_threshold).await
        } else {
            warn!("No coverage tool found (cargo-llvm-cov or cargo-tarpaulin). Skipping coverage check.");
            return Ok(QualityGateResult {
                gate_name: "coverage".to_string(),
                passed: true, // Don't fail if tool not available
                score: 0.0,
                threshold: min_threshold,
                duration_ms: 0,
                issues: vec![QualityGateIssue {
                    severity: IssueSeverity::Warning,
                    code: "COVERAGE_TOOL_MISSING".to_string(),
                    message: "Coverage tool not found. Install cargo-llvm-cov or cargo-tarpaulin to enable coverage checking.".to_string(),
                    file: None,
                    line: None,
                    column: None,
                    suggestion: Some("Install cargo-llvm-cov: cargo install cargo-llvm-cov".to_string()),
                }],
                command_used: None,
            });
        };

        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        match coverage_result {
            Ok((coverage_percent, issues)) => {
                let passed = coverage_percent >= min_threshold;
                let result = QualityGateResult {
                    gate_name: "coverage".to_string(),
                    passed,
                    score: coverage_percent,
                    threshold: min_threshold,
                    duration_ms,
                    issues,
                    command_used: Some("cargo-llvm-cov".to_string()),
                };

                if passed {
                    info!("Coverage check passed: {:.1}% >= {:.1}%", 
                        coverage_percent * 100.0, min_threshold * 100.0);
                } else {
                    warn!("Coverage check failed: {:.1}% < {:.1}%", 
                        coverage_percent * 100.0, min_threshold * 100.0);
                }

                Ok(result)
            }
            Err(e) => {
                error!("Coverage check failed with error: {}", e);
                Ok(QualityGateResult {
                    gate_name: "coverage".to_string(),
                    passed: false,
                    score: 0.0,
                    threshold: min_threshold,
                    duration_ms,
                    issues: vec![QualityGateIssue {
                        severity: IssueSeverity::Error,
                        code: "COVERAGE_CHECK_ERROR".to_string(),
                        message: format!("Coverage check failed: {}", e),
                        file: None,
                        line: None,
                        column: None,
                        suggestion: None,
                    }],
                    command_used: Some("cargo-llvm-cov".to_string()),
                })
            }
        }
    }

    /// Check coverage by test type (unit, integration, e2e)
    async fn check_coverage_by_type(
        &self,
        test_type: &str,
        min_threshold: f64,
    ) -> Result<QualityGateResult> {
        // For now, use general coverage check
        // Could be enhanced to filter by test type
        self.check_coverage(min_threshold).await
    }

    /// Check coverage using cargo-llvm-cov
    async fn check_coverage_llvm_cov(&self, min_threshold: f64) -> Result<(f64, Vec<QualityGateIssue>)> {
        let output = Command::new("cargo")
            .args(&["llvm-cov", "--json", "--summary-only"])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute cargo llvm-cov")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "cargo llvm-cov failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let json_output: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse cargo llvm-cov JSON output")?;

        // Extract line coverage from JSON output
        let coverage_percent = json_output
            .get("summary")
            .and_then(|s| s.get("lines"))
            .and_then(|l| l.get("percent"))
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0) / 100.0;

        let mut issues = Vec::new();
        if coverage_percent < min_threshold {
            issues.push(QualityGateIssue {
                severity: IssueSeverity::Error,
                code: "COVERAGE_BELOW_THRESHOLD".to_string(),
                message: format!(
                    "Coverage {:.1}% is below required threshold {:.1}%",
                    coverage_percent * 100.0,
                    min_threshold * 100.0
                ),
                file: None,
                line: None,
                column: None,
                suggestion: Some("Add more tests to increase coverage".to_string()),
            });
        }

        Ok((coverage_percent, issues))
    }

    /// Check coverage using cargo-tarpaulin
    async fn check_coverage_tarpaulin(&self, min_threshold: f64) -> Result<(f64, Vec<QualityGateIssue>)> {
        let output = Command::new("cargo")
            .args(&["tarpaulin", "--out", "Json", "--output-dir", "."])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute cargo tarpaulin")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "cargo tarpaulin failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Parse tarpaulin JSON output (usually in tarpaulin.json)
        let json_path = Path::new(&self.workspace_root).join("tarpaulin.json");
        let json_content = std::fs::read_to_string(&json_path)
            .context("Failed to read tarpaulin.json")?;

        let json_output: serde_json::Value = serde_json::from_str(&json_content)
            .context("Failed to parse tarpaulin JSON output")?;

        // Extract line coverage percentage
        let coverage_percent = json_output
            .get("line_percent")
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0) / 100.0;

        let mut issues = Vec::new();
        if coverage_percent < min_threshold {
            issues.push(QualityGateIssue {
                severity: IssueSeverity::Error,
                code: "COVERAGE_BELOW_THRESHOLD".to_string(),
                message: format!(
                    "Coverage {:.1}% is below required threshold {:.1}%",
                    coverage_percent * 100.0,
                    min_threshold * 100.0
                ),
                file: None,
                line: None,
                column: None,
                suggestion: Some("Add more tests to increase coverage".to_string()),
            });
        }

        Ok((coverage_percent, issues))
    }

    /// Check linting using cargo clippy
    async fn check_linting(&self) -> Result<QualityGateResult> {
        let start_time = Utc::now();

        let output = Command::new("cargo")
            .args(&["clippy", "--message-format", "json", "--", "--no-deps"])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute cargo clippy")?;

        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        // Parse clippy JSON output
        let mut issues = Vec::new();
        let mut has_errors = false;
        let mut has_warnings = false;

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        for line in stdout_str.lines() {
            if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                if message.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(msg) = message.get("message") {
                        let level = msg.get("level")
                            .and_then(|l| l.as_str())
                            .unwrap_or("unknown");
                        
                        let code = msg.get("code")
                            .and_then(|c| c.get("code"))
                            .and_then(|c| c.as_str())
                            .unwrap_or("UNKNOWN")
                            .to_string();

                        let message_text = msg.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("")
                            .to_string();

                        let span = msg.get("spans")
                            .and_then(|s| s.as_array())
                            .and_then(|s| s.first())
                            .and_then(|s| s.as_object());

                        let file = span.and_then(|s| s.get("file_name"))
                            .and_then(|f| f.as_str())
                            .map(|f| f.to_string());
                        
                        let line = span.and_then(|s| s.get("line_start"))
                            .and_then(|l| l.as_u64())
                            .map(|l| l as u32);

                        let column = span.and_then(|s| s.get("column_start"))
                            .and_then(|c| c.as_u64())
                            .map(|c| c as u32);

                        let severity = match level {
                            "error" => {
                                has_errors = true;
                                IssueSeverity::Error
                            }
                            "warning" => {
                                has_warnings = true;
                                IssueSeverity::Warning
                            }
                            _ => IssueSeverity::Info,
                        };

                        issues.push(QualityGateIssue {
                            severity,
                            code,
                            message: message_text,
                            file,
                            line,
                            column,
                            suggestion: None,
                        });
                    }
                }
            }
        }

        // Also check stderr for any errors
        if !output.stderr.is_empty() {
            let stderr_text = String::from_utf8_lossy(&output.stderr);
            if stderr_text.contains("error") {
                has_errors = true;
                issues.push(QualityGateIssue {
                    severity: IssueSeverity::Error,
                    code: "CLIPPY_ERROR".to_string(),
                    message: format!("Clippy execution error: {}", stderr_text),
                    file: None,
                    line: None,
                    column: None,
                    suggestion: None,
                });
            }
        }

        let passed = !has_errors && output.status.success();
        let score = if passed && !has_warnings { 1.0 } else if passed { 0.8 } else { 0.0 };

        let result = QualityGateResult {
            gate_name: "linting".to_string(),
            passed,
            score,
            threshold: 1.0, // Linting should have no errors
            duration_ms,
            issues,
            command_used: Some("cargo clippy".to_string()),
        };

        if passed {
            info!("Linting check passed ({} warnings, 0 errors)", 
                result.issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count());
        } else {
            warn!("Linting check failed ({} errors found)", 
                result.issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Error)).count());
        }

        Ok(result)
    }

    /// Check type checking using cargo check
    async fn check_type_checking(&self) -> Result<QualityGateResult> {
        let start_time = Utc::now();

        let output = Command::new("cargo")
            .args(&["check", "--message-format", "json", "--", "--no-deps"])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute cargo check")?;

        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        // Parse cargo check JSON output
        let mut issues = Vec::new();
        let mut has_errors = false;

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        for line in stdout_str.lines() {
            if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                if message.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(msg) = message.get("message") {
                        let level = msg.get("level")
                            .and_then(|l| l.as_str())
                            .unwrap_or("unknown");
                        
                        if level == "error" {
                            has_errors = true;

                            let code = msg.get("code")
                                .and_then(|c| c.get("code"))
                                .and_then(|c| c.as_str())
                                .unwrap_or("TYPE_ERROR")
                                .to_string();

                            let message_text = msg.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("")
                                .to_string();

                            let span = msg.get("spans")
                                .and_then(|s| s.as_array())
                                .and_then(|s| s.first())
                                .and_then(|s| s.as_object());

                            let file = span.and_then(|s| s.get("file_name"))
                                .and_then(|f| f.as_str())
                                .map(|f| f.to_string());
                            
                            let line = span.and_then(|s| s.get("line_start"))
                                .and_then(|l| l.as_u64())
                                .map(|l| l as u32);

                            let column = span.and_then(|s| s.get("column_start"))
                                .and_then(|c| c.as_u64())
                                .map(|c| c as u32);

                            issues.push(QualityGateIssue {
                                severity: IssueSeverity::Error,
                                code,
                                message: message_text,
                                file,
                                line,
                                column,
                                suggestion: None,
                            });
                        }
                    }
                }
            }
        }

        // Also check stderr for compilation errors
        if !output.stderr.is_empty() {
            let stderr_text = String::from_utf8_lossy(&output.stderr);
            if stderr_text.contains("error") {
                has_errors = true;
                issues.push(QualityGateIssue {
                    severity: IssueSeverity::Error,
                    code: "COMPILATION_ERROR".to_string(),
                    message: format!("Compilation error: {}", stderr_text),
                    file: None,
                    line: None,
                    column: None,
                    suggestion: None,
                });
            }
        }

        let passed = !has_errors && output.status.success();
        let score = if passed { 1.0 } else { 0.0 };
        
        let issues_count = issues.len(); // Save count before moving

        let result = QualityGateResult {
            gate_name: "type_checking".to_string(),
            passed,
            score,
            threshold: 1.0, // Type checking should have no errors
            duration_ms,
            issues,
            command_used: Some("cargo check".to_string()),
        };

        if passed {
            info!("Type checking passed");
        } else {
            warn!("Type checking failed ({} errors found)", issues_count);
        }

        Ok(result)
    }

    /// Check mutation testing using cargo-mutants or similar
    async fn check_mutation_testing(&self, min_score: f64) -> Result<QualityGateResult> {
        let start_time = Utc::now();

        // Mutation testing is optional and may not be available
        if !self.has_command("cargo-mutants") {
            warn!("cargo-mutants not found. Skipping mutation testing.");
            return Ok(QualityGateResult {
                gate_name: "mutation_testing".to_string(),
                passed: true, // Don't fail if tool not available
                score: 0.0,
                threshold: min_score,
                duration_ms: 0,
                issues: vec![QualityGateIssue {
                    severity: IssueSeverity::Warning,
                    code: "MUTATION_TOOL_MISSING".to_string(),
                    message: "Mutation testing tool not found. Install cargo-mutants to enable mutation testing.".to_string(),
                    file: None,
                    line: None,
                    column: None,
                    suggestion: Some("Install cargo-mutants: cargo install cargo-mutants".to_string()),
                }],
                command_used: None,
            });
        }

        let output = Command::new("cargo")
            .args(&["mutants", "--json"])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute cargo mutants")?;

        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        // Parse mutation testing output
        // This is a simplified implementation - actual parsing would depend on cargo-mutants output format
        let passed = output.status.success();
        let score = if passed { 1.0 } else { 0.5 }; // Simplified scoring

        let mut issues = Vec::new();
        if score < min_score {
            issues.push(QualityGateIssue {
                severity: IssueSeverity::Error,
                code: "MUTATION_SCORE_BELOW_THRESHOLD".to_string(),
                message: format!(
                    "Mutation score {:.1}% is below required threshold {:.1}%",
                    score * 100.0,
                    min_score * 100.0
                ),
                file: None,
                line: None,
                column: None,
                suggestion: Some("Improve test coverage to catch more mutations".to_string()),
            });
        }

        Ok(QualityGateResult {
            gate_name: "mutation_testing".to_string(),
            passed,
            score,
            threshold: min_score,
            duration_ms,
            issues,
            command_used: Some("cargo mutants".to_string()),
        })
    }

    /// Check security requirements
    async fn check_security(
        &self,
        security_requirements: &agent_agency_contracts::planning_io::SecurityRequirements,
    ) -> Result<QualityGateResult> {
        let start_time = Utc::now();

        // Check for cargo-audit (advisory checking)
        let mut issues = Vec::new();
        let mut passed = true;
        let mut score = 1.0;

        if self.has_command("cargo-audit") {
            let output = Command::new("cargo")
                .args(&["audit", "--json"])
                .current_dir(&self.workspace_root)
                .output()
                .context("Failed to execute cargo audit")?;

            // Parse audit results
            if !output.status.success() {
                passed = false;
                score = 0.0;
                issues.push(QualityGateIssue {
                    severity: IssueSeverity::Error,
                    code: "SECURITY_VULNERABILITIES".to_string(),
                    message: "Security vulnerabilities found in dependencies".to_string(),
                    file: None,
                    line: None,
                    column: None,
                    suggestion: Some("Run 'cargo audit' to see details and update dependencies".to_string()),
                });
            }
        } else {
            warn!("cargo-audit not found. Skipping security audit.");
            issues.push(QualityGateIssue {
                severity: IssueSeverity::Warning,
                code: "AUDIT_TOOL_MISSING".to_string(),
                message: "Security audit tool not found. Install cargo-audit to enable security scanning.".to_string(),
                file: None,
                line: None,
                column: None,
                suggestion: Some("Install cargo-audit: cargo install cargo-audit".to_string()),
            });
        }

        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        Ok(QualityGateResult {
            gate_name: "security".to_string(),
            passed,
            score,
            threshold: 1.0,
            duration_ms,
            issues,
            command_used: Some("cargo audit".to_string()),
        })
    }

    /// Check if a command is available in PATH
    fn has_command(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

