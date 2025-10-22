//! Quality validators implementation

use crate::types::*;
use crate::error::*;
use async_trait::async_trait;

/// Compilation validator - ensures code compiles
pub struct CompilationValidator;

#[async_trait]
impl super::gates::QualityValidatorTrait for CompilationValidator {
    async fn validate(&self, context: &ValidationContext) -> ValidationResult<ValidationResult> {
        // Run cargo check on the package
        let output = tokio::process::Command::new("cargo")
            .args(&["check", "--package", &context.package_name])
            .current_dir(&context.workspace_root)
            .output()
            .await
            .map_err(|e| ValidationError::ExternalToolFailure {
                tool_name: "cargo".to_string(),
                message: e.to_string(),
            })?;

        if output.status.success() {
            Ok(ValidationResult::Pass {
                score: 1.0,
                details: "Compilation successful".to_string(),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_count = count_compilation_errors(&stderr);

            let score = if error_count == 0 {
                1.0
            } else {
                (10.0 - error_count.min(10) as f32) / 10.0
            };

            Ok(ValidationResult::Fail {
                score,
                details: format!("{} compilation errors found", error_count),
                suggestions: extract_compilation_suggestions(&stderr),
            })
        }
    }
}

/// Test validator - ensures tests pass and meet coverage
pub struct TestValidator {
    min_coverage: f32,
}

impl TestValidator {
    pub fn new(min_coverage: f32) -> Self {
        Self { min_coverage }
    }
}

#[async_trait]
impl super::gates::QualityValidatorTrait for TestValidator {
    async fn validate(&self, context: &ValidationContext) -> ValidationResult<ValidationResult> {
        // Run tests
        let test_output = tokio::process::Command::new("cargo")
            .args(&["test", "--package", &context.package_name])
            .current_dir(&context.workspace_root)
            .output()
            .await
            .map_err(|e| ValidationError::ExternalToolFailure {
                tool_name: "cargo-test".to_string(),
                message: e.to_string(),
            })?;

        if !test_output.status.success() {
            return Ok(Ok(ValidationResult::Fail {
                score: 0.0,
                details: "Tests failed".to_string(),
                suggestions: vec![
                    "Run 'cargo test' to see detailed failures".to_string(),
                    "Check test output for specific error messages".to_string(),
                ],
            }));
        }

        // Try to get coverage (if cargo-tarpaulin is available)
        match get_test_coverage(&context.workspace_root, &context.package_name).await {
            Ok(coverage) => {
                if coverage >= self.min_coverage {
                    Ok(ValidationResult::Pass {
                        score: 1.0,
                        details: format!("Tests passed with {:.1}% coverage", coverage * 100.0),
                    })
                } else {
                    Ok(ValidationResult::Warning {
                        score: coverage / self.min_coverage,
                        details: format!("Tests passed but coverage {:.1}% below required {:.1}%",
                                       coverage * 100.0, self.min_coverage * 100.0),
                        suggestions: vec![
                            "Add more unit tests".to_string(),
                            "Improve branch coverage in complex functions".to_string(),
                        ],
                    })
                }
            }
            Err(_) => {
                // Coverage tool not available, just check that tests pass
                Ok(ValidationResult::Warning {
                    score: 0.7, // Partial score when coverage can't be measured
                    details: "Tests passed but coverage measurement unavailable".to_string(),
                    suggestions: vec![
                        "Install cargo-tarpaulin for coverage measurement".to_string(),
                        "Run 'cargo tarpaulin --out Html' for detailed coverage report".to_string(),
                    ],
                })
            }
        }
    }
}

/// Linting validator - ensures code quality standards
pub struct LintValidator;

#[async_trait]
impl super::gates::QualityValidatorTrait for LintValidator {
    async fn validate(&self, context: &ValidationContext) -> ValidationResult<ValidationResult> {
        // Run clippy
        let output = tokio::process::Command::new("cargo")
            .args(&["clippy", "--package", &context.package_name, "--", "-D", "warnings"])
            .current_dir(&context.workspace_root)
            .output()
            .await
            .map_err(|e| ValidationError::ExternalToolFailure {
                tool_name: "clippy".to_string(),
                message: e.to_string(),
            })?;

        if output.status.success() {
            Ok(ValidationResult::Pass {
                score: 1.0,
                details: "No linting warnings or errors".to_string(),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let warning_count = count_lint_warnings(&stderr);

            let score = if warning_count == 0 {
                1.0
            } else {
                (10.0 - warning_count.min(10) as f32) / 10.0
            };

            Ok(ValidationResult::Warning {
                score,
                details: format!("{} linting warnings found", warning_count),
                suggestions: vec![
                    "Run 'cargo clippy' to see detailed warnings".to_string(),
                    "Fix warnings or add #[allow(...)] attributes where appropriate".to_string(),
                ],
            })
        }
    }
}

/// Security validator - ensures security best practices
pub struct SecurityValidator;

#[async_trait]
impl super::gates::QualityValidatorTrait for SecurityValidator {
    async fn validate(&self, context: &ValidationContext) -> ValidationResult<ValidationResult> {
        // Run cargo audit if available
        match tokio::process::Command::new("cargo")
            .args(&["audit"])
            .current_dir(&context.workspace_root)
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                Ok(ValidationResult::Pass {
                    score: 1.0,
                    details: "No security vulnerabilities found".to_string(),
                })
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let vuln_count = count_security_vulnerabilities(&stderr);

                let score = if vuln_count == 0 {
                    1.0
                } else {
                    (5.0 - vuln_count.min(5) as f32) / 5.0
                };

                    Ok(ValidationResult::Fail {
                    score,
                    details: format!("{} security vulnerabilities found", vuln_count),
                    suggestions: vec![
                        "Run 'cargo audit' to see detailed vulnerability report".to_string(),
                        "Update vulnerable dependencies".to_string(),
                        "Review security advisories at https://rustsec.org/advisories".to_string(),
                    ],
                })
            }
            Err(_) => {
                // cargo audit not available, do basic checks
                Ok(ValidationResult::Warning {
                    score: 0.5,
                    details: "Security audit tool not available".to_string(),
                    suggestions: vec![
                        "Install cargo-audit: cargo install cargo-audit".to_string(),
                        "Run regular security audits manually".to_string(),
                    ],
                })
            }
        }
    }
}

/// Performance validator - ensures performance requirements
pub struct PerformanceValidator {
    max_response_time_ms: u64,
}

impl PerformanceValidator {
    pub fn new(max_response_time_ms: u64) -> Self {
        Self { max_response_time_ms }
    }
}

#[async_trait]
impl super::gates::QualityValidatorTrait for PerformanceValidator {
    async fn validate(&self, context: &ValidationContext) -> ValidationResult<ValidationResult> {
        // Check if execution time is within bounds
        let execution_time_ms = context.execution_time.as_millis() as u64;

        if execution_time_ms <= self.max_response_time_ms {
            Ok(ValidationResult::Pass {
                score: 1.0,
                details: format!("Execution completed in {}ms (within {}ms limit)",
                               execution_time_ms, self.max_response_time_ms),
            })
        } else {
            let overrun_percent = (execution_time_ms as f32 / self.max_response_time_ms as f32) - 1.0;
            let score = (1.0 - overrun_percent.min(1.0)).max(0.0);

            Ok(ValidationResult::Warning {
                score,
                details: format!("Execution took {}ms ({}% over {}ms limit)",
                               execution_time_ms, (overrun_percent * 100.0) as u32, self.max_response_time_ms),
                suggestions: vec![
                    "Optimize performance-critical code paths".to_string(),
                    "Consider parallelization for CPU-intensive operations".to_string(),
                    "Review algorithm complexity".to_string(),
                ],
            })
        }
    }
}

/// Documentation validator - ensures documentation quality
pub struct DocumentationValidator;

#[async_trait]
impl super::gates::QualityValidatorTrait for DocumentationValidator {
    async fn validate(&self, context: &ValidationContext) -> ValidationResult<ValidationResult> {
        // Run cargo doc to check documentation
        let output = tokio::process::Command::new("cargo")
            .args(&["doc", "--package", &context.package_name, "--no-deps"])
            .current_dir(&context.workspace_root)
            .output()
            .await
            .map_err(|e| ValidationError::ExternalToolFailure {
                tool_name: "cargo-doc".to_string(),
                message: e.to_string(),
            })?;

        if output.status.success() {
            // Could check for undocumented items here
            Ok(ValidationResult::Pass {
                score: 1.0,
                details: "Documentation generated successfully".to_string(),
            })
        } else {
            Ok(ValidationResult::Warning {
                score: 0.8,
                details: "Documentation generation had issues".to_string(),
                suggestions: vec![
                    "Run 'cargo doc' to see documentation issues".to_string(),
                    "Add missing documentation for public APIs".to_string(),
                ],
            })
        }
    }
}

/// Helper functions for parsing tool outputs

fn count_compilation_errors(stderr: &str) -> usize {
    stderr.lines()
        .filter(|line| line.contains("error[E") || line.contains("error:"))
        .count()
}

fn extract_compilation_suggestions(stderr: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Extract common error patterns and suggest fixes
    if stderr.contains("E0063") {
        suggestions.push("Fix missing struct fields".to_string());
    }
    if stderr.contains("E0277") {
        suggestions.push("Add required trait bounds".to_string());
    }
    if stderr.contains("E0308") {
        suggestions.push("Fix mismatched types".to_string());
    }

    if suggestions.is_empty() {
        suggestions.push("Run 'cargo check' for detailed error messages".to_string());
    }

    suggestions
}

async fn get_test_coverage(workspace_root: &std::path::Path, package_name: &str) -> Result<f32> {
    // Try to run tarpaulin for coverage
    let output = tokio::process::Command::new("cargo")
        .args(&["tarpaulin", "--packages", package_name, "--out", "Json"])
        .current_dir(workspace_root)
        .output()
        .await?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON output to extract coverage percentage
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(coverage) = json.get("coverage").and_then(|c| c.as_f64()) {
                return Ok((coverage / 100.0) as f32);
            }
        }
    }

    Err(anyhow::anyhow!("Could not determine test coverage"))
}

fn count_lint_warnings(stderr: &str) -> usize {
    stderr.lines()
        .filter(|line| line.contains("warning:") || line.contains("clippy::"))
        .count()
}

fn count_security_vulnerabilities(stderr: &str) -> usize {
    stderr.lines()
        .filter(|line| line.contains("vulnerability") || line.contains("advisory"))
        .count()
}
