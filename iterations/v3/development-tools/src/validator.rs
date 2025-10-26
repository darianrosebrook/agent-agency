//! CAWS Validator Core
//!
//! Consolidated validation logic extracted from orchestration, workers, and MCP integration.

use crate::policy::{CawsPolicy, ViolationSeverity, RuleCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub task_id: String,
    pub violations: Vec<Violation>,
    pub compliance_score: f32,
    pub passed: bool,
    pub validated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub category: RuleCategory,
    pub message: String,
    pub remediation: Option<String>,
    pub location: Option<ViolationLocation>,
}

/// Location of violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationLocation {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// Validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub task_id: String,
    pub risk_tier: String,
    pub working_spec: serde_json::Value,
    pub diff_stats: DiffStats,
    pub test_results: Option<TestResults>,
    pub security_scan: Option<SecurityScanResults>,
}

/// Diff statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub files_modified: Vec<String>,
}

/// Test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub coverage_percentage: Option<f32>,
}

/// Security scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResults {
    pub vulnerabilities_found: u32,
    pub critical_vulnerabilities: u32,
    pub high_vulnerabilities: u32,
}

/// Main CAWS validator
#[derive(Debug)]
pub struct CawsValidator {
    policy: CawsPolicy,
}

impl CawsValidator {
    pub fn new(policy: CawsPolicy) -> Self {
        Self { policy }
    }

    /// Validate against CAWS policies
    pub async fn validate(&self, context: ValidationContext) -> ValidationResult {
        let mut violations = Vec::new();

        // Budget validation
        violations.extend(self.validate_budget(&context));

        // Risk tier validation
        violations.extend(self.validate_risk_tier(&context));

        // Quality gates
        violations.extend(self.validate_quality_gates(&context));

        // Security validation
        violations.extend(self.validate_security(&context));

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(&violations);
        let passed = violations.iter().all(|v| v.severity != ViolationSeverity::Critical)
            && violations.iter().filter(|v| v.severity == ViolationSeverity::Error).count() == 0;

        ValidationResult {
            task_id: context.task_id,
            violations,
            compliance_score,
            passed,
            validated_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    fn validate_budget(&self, context: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        if let Some(limits) = self.policy.budget_limits.get(&context.risk_tier) {
            if context.diff_stats.files_changed > limits.max_files {
                violations.push(Violation {
                    rule_id: "budget-files".to_string(),
                    severity: ViolationSeverity::Error,
                    category: RuleCategory::Budget,
                    message: format!("Files changed ({}) exceeds budget limit ({})",
                                   context.diff_stats.files_changed, limits.max_files),
                    remediation: Some("Split changes into smaller PRs or request budget increase".to_string()),
                    location: None,
                });
            }

            let total_loc = context.diff_stats.lines_added + context.diff_stats.lines_deleted;
            if total_loc > limits.max_loc {
                violations.push(Violation {
                    rule_id: "budget-loc".to_string(),
                    severity: ViolationSeverity::Error,
                    category: RuleCategory::Budget,
                    message: format!("Lines of code changed ({}) exceeds budget limit ({})",
                                   total_loc, limits.max_loc),
                    remediation: Some("Refactor changes to be more focused".to_string()),
                    location: None,
                });
            }
        }

        violations
    }

    fn validate_risk_tier(&self, context: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        if let Some(tier_config) = self.policy.risk_tiers.get(&context.risk_tier) {
            // Check mandatory checks are performed
            for check in &tier_config.mandatory_checks {
                match check.as_str() {
                    "tests" => {
                        if let Some(test_results) = &context.test_results {
                            if test_results.failed_tests > 0 {
                                violations.push(Violation {
                                    rule_id: "risk-tier-tests".to_string(),
                                    severity: ViolationSeverity::Error,
                                    category: RuleCategory::Quality,
                                    message: format!("Risk tier {} requires passing tests, but {} tests failed",
                                                   context.risk_tier, test_results.failed_tests),
                                    remediation: Some("Fix failing tests".to_string()),
                                    location: None,
                                });
                            }
                        }
                    }
                    "security" => {
                        if let Some(security) = &context.security_scan {
                            if security.critical_vulnerabilities > 0 {
                                violations.push(Violation {
                                    rule_id: "risk-tier-security".to_string(),
                                    severity: ViolationSeverity::Critical,
                                    category: RuleCategory::Security,
                                    message: format!("Risk tier {} prohibits critical security vulnerabilities",
                                                   context.risk_tier),
                                    remediation: Some("Fix critical security vulnerabilities".to_string()),
                                    location: None,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        violations
    }

    fn validate_quality_gates(&self, context: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Test coverage requirements
        if let Some(test_results) = &context.test_results {
            if let Some(coverage) = test_results.coverage_percentage {
                match context.risk_tier.as_str() {
                    "high" => {
                        if coverage < 0.8 {
                            violations.push(Violation {
                                rule_id: "quality-coverage-high".to_string(),
                                severity: ViolationSeverity::Error,
                                category: RuleCategory::Quality,
                                message: format!("High risk tier requires 80%+ coverage, got {:.1}%", coverage * 100.0),
                                remediation: Some("Add more tests to achieve required coverage".to_string()),
                                location: None,
                            });
                        }
                    }
                    "medium" => {
                        if coverage < 0.7 {
                            violations.push(Violation {
                                rule_id: "quality-coverage-medium".to_string(),
                                severity: ViolationSeverity::Warning,
                                category: RuleCategory::Quality,
                                message: format!("Medium risk tier recommends 70%+ coverage, got {:.1}%", coverage * 100.0),
                                remediation: Some("Consider adding more tests".to_string()),
                                location: None,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        violations
    }

    fn validate_security(&self, context: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        if let Some(security) = &context.security_scan {
            if security.vulnerabilities_found > 0 {
                let severity = if security.critical_vulnerabilities > 0 {
                    ViolationSeverity::Critical
                } else if security.high_vulnerabilities > 0 {
                    ViolationSeverity::Error
                } else {
                    ViolationSeverity::Warning
                };

                violations.push(Violation {
                    rule_id: "security-scan".to_string(),
                    severity,
                    category: RuleCategory::Security,
                    message: format!("Found {} security vulnerabilities ({} critical, {} high)",
                                   security.vulnerabilities_found,
                                   security.critical_vulnerabilities,
                                   security.high_vulnerabilities),
                    remediation: Some("Address security vulnerabilities before proceeding".to_string()),
                    location: None,
                });
            }
        }

        violations
    }

    fn calculate_compliance_score(&self, violations: &[Violation]) -> f32 {
        if violations.is_empty() {
            return 1.0;
        }

        // Weight violations by severity
        let total_weight: f32 = violations.iter().map(|v| {
            match v.severity {
                ViolationSeverity::Critical => 1.0,
                ViolationSeverity::Error => 0.8,
                ViolationSeverity::Warning => 0.4,
                ViolationSeverity::Info => 0.1,
            }
        }).sum();

        // Perfect score minus weighted violations
        (1.0 - total_weight).max(0.0)
    }
}

/// Async validation trait for integration
#[async_trait]
pub trait AsyncValidator: Send + Sync {
    async fn validate_async(&self, context: ValidationContext) -> ValidationResult;
}

#[async_trait]
impl AsyncValidator for CawsValidator {
    async fn validate_async(&self, context: ValidationContext) -> ValidationResult {
        self.validate(context).await
    }
}
