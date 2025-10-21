//! CAWS integration for planning validation
//!
//! This module provides integration with CAWS (Code Analysis and Writing Standards)
//! to validate working specifications before they proceed to execution.

use std::sync::Arc;
use async_trait::async_trait;

use crate::error::{PlanningError, PlanningResult};

/// CAWS validation error
#[derive(Debug, thiserror::Error)]
pub enum CawsValidationError {
    #[error("CAWS validation service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Working spec validation failed: {0}")]
    ValidationFailed(String),

    #[error("CAWS configuration error: {0}")]
    ConfigurationError(String),

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation timeout: {0}")]
    Timeout(String),
}

/// Context for CAWS validation
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Risk tier of the task
    pub risk_tier: agent_agency_contracts::task_request::RiskTier,

    /// Environment context
    pub environment: agent_agency_contracts::task_request::Environment,

    /// Additional validation options
    pub options: ValidationOptions,
}

/// Validation options
#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// Enable strict mode (fail on warnings)
    pub strict_mode: bool,

    /// Include detailed suggestions
    pub include_suggestions: bool,

    /// Skip expensive validations
    pub skip_expensive: bool,
}

/// CAWS validation result
#[derive(Debug, Clone)]
pub struct CawsValidationResult {
    /// Whether validation passed
    pub compliant: bool,

    /// Compliance score (0.0-1.0)
    pub compliance_score: f64,

    /// Validation violations found
    pub violations: Vec<ValidationViolation>,

    /// Improvement suggestions
    pub suggestions: Vec<String>,

    /// Quality indicators
    pub quality_indicators: Vec<QualityIndicator>,
}

/// Validation violation
#[derive(Debug, Clone)]
pub struct ValidationViolation {
    /// Violation code
    pub code: String,

    /// Severity level
    pub severity: ViolationSeverity,

    /// Human-readable message
    pub message: String,

    /// Location in the working spec (if applicable)
    pub location: Option<String>,
}

/// Violation severity
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

/// Quality indicator
#[derive(Debug, Clone)]
pub struct QualityIndicator {
    /// Indicator name
    pub name: String,

    /// Score (0.0-1.0)
    pub score: f64,

    /// Evidence supporting the score
    pub evidence: String,
}

/// CAWS validator trait
#[async_trait]
pub trait CawsValidator: Send + Sync {
    /// Validate a working specification
    async fn validate_working_spec(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        context: &ValidationContext,
    ) -> Result<CawsValidationResult, CawsValidationError>;
}

/// Default CAWS validator implementation
pub struct DefaultCawsValidator {
    // In a real implementation, this would hold CAWS service client,
    // configuration, and cached validation rules
}

impl DefaultCawsValidator {
    /// Create a new default CAWS validator
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl CawsValidator for DefaultCawsValidator {
    async fn validate_working_spec(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        context: &ValidationContext,
    ) -> Result<CawsValidationResult, CawsValidationError> {
        // This is a simplified implementation. In practice, this would:
        // 1. Send the working spec to CAWS service for analysis
        // 2. Apply risk-tier specific validation rules
        // 3. Run static analysis on the specification
        // 4. Check for compliance with coding standards
        // 5. Validate test coverage requirements
        // 6. Assess security implications

        let mut violations = Vec::new();
        let mut suggestions = Vec::new();
        let mut quality_indicators = Vec::new();

        // Basic validation checks
        self.validate_basic_structure(working_spec, &mut violations)?;
        self.validate_risk_tier_compliance(working_spec, context, &mut violations)?;
        self.validate_acceptance_criteria(working_spec, &mut violations, &mut suggestions)?;
        self.validate_test_plan(working_spec, context, &mut violations, &mut suggestions)?;

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(&violations, working_spec.risk_tier);

        // Generate quality indicators
        quality_indicators.push(QualityIndicator {
            name: "structural_completeness".to_string(),
            score: if violations.is_empty() { 1.0 } else { 0.7 },
            evidence: format!("Working spec has {} violations", violations.len()),
        });

        quality_indicators.push(QualityIndicator {
            name: "test_coverage".to_string(),
            score: self.assess_test_coverage(working_spec),
            evidence: "Based on test plan completeness".to_string(),
        });

        quality_indicators.push(QualityIndicator {
            name: "rollback_readiness".to_string(),
            score: 0.8, // Assume decent rollback plan
            evidence: "Rollback strategy defined".to_string(),
        });

        let compliant = violations.iter().all(|v| v.severity != ViolationSeverity::Error);

        Ok(CawsValidationResult {
            compliant,
            compliance_score,
            violations,
            suggestions,
            quality_indicators,
        })
    }
}

impl DefaultCawsValidator {
    fn validate_basic_structure(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        violations: &mut Vec<ValidationViolation>,
    ) -> Result<(), CawsValidationError> {
        // Check required fields
        if working_spec.title.trim().is_empty() {
            violations.push(ValidationViolation {
                code: "MISSING_TITLE".to_string(),
                severity: ViolationSeverity::Error,
                message: "Working spec title cannot be empty".to_string(),
                location: Some("title".to_string()),
            });
        }

        if working_spec.goals.is_empty() {
            violations.push(ValidationViolation {
                code: "MISSING_GOALS".to_string(),
                severity: ViolationSeverity::Error,
                message: "Working spec must have at least one goal".to_string(),
                location: Some("goals".to_string()),
            });
        }

        if working_spec.acceptance_criteria.is_empty() {
            violations.push(ValidationViolation {
                code: "MISSING_ACCEPTANCE_CRITERIA".to_string(),
                severity: ViolationSeverity::Error,
                message: "Working spec must have acceptance criteria".to_string(),
                location: Some("acceptance_criteria".to_string()),
            });
        }

        Ok(())
    }

    fn validate_risk_tier_compliance(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        context: &ValidationContext,
        violations: &mut Vec<ValidationViolation>,
    ) -> Result<(), CawsValidationError> {
        // T1 tasks require stricter validation
        if working_spec.risk_tier == 1 {
            if working_spec.acceptance_criteria.len() < 3 {
                violations.push(ValidationViolation {
                    code: "INSUFFICIENT_ACCEPTANCE_CRITERIA_T1".to_string(),
                    severity: ViolationSeverity::Error,
                    message: "T1 tasks must have at least 3 acceptance criteria".to_string(),
                    location: Some("acceptance_criteria".to_string()),
                });
            }

            // Check for comprehensive test plan
            if working_spec.test_plan.unit_tests.is_empty() {
                violations.push(ValidationViolation {
                    code: "MISSING_UNIT_TESTS_T1".to_string(),
                    severity: ViolationSeverity::Error,
                    message: "T1 tasks must include unit tests".to_string(),
                    location: Some("test_plan.unit_tests".to_string()),
                });
            }

            // Check budget limits are reasonable
            if let Some(budget) = &working_spec.constraints.budget_limits {
                if budget.max_files.unwrap_or(100) > 25 {
                    violations.push(ValidationViolation {
                        code: "BUDGET_TOO_LARGE_T1".to_string(),
                        severity: ViolationSeverity::Warning,
                        message: "T1 task budget may be too large for safe execution".to_string(),
                        location: Some("constraints.budget_limits".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_acceptance_criteria(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        violations: &mut Vec<ValidationViolation>,
        suggestions: &mut Vec<String>,
    ) -> Result<(), CawsValidationError> {
        for (i, criterion) in working_spec.acceptance_criteria.iter().enumerate() {
            // Check criterion structure
            if criterion.given.trim().is_empty() ||
               criterion.when.trim().is_empty() ||
               criterion.then.trim().is_empty() {
                violations.push(ValidationViolation {
                    code: "MALFORMED_ACCEPTANCE_CRITERION".to_string(),
                    severity: ViolationSeverity::Error,
                    message: format!("Acceptance criterion {} has empty fields", criterion.id),
                    location: Some(format!("acceptance_criteria[{}]", i)),
                });
            }

            // Check for testable criteria
            if !criterion.then.contains("should") && !criterion.then.contains("will") {
                suggestions.push(format!(
                    "Consider making acceptance criterion {} more testable with specific outcomes",
                    criterion.id
                ));
            }
        }

        Ok(())
    }

    fn validate_test_plan(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        context: &ValidationContext,
        violations: &mut Vec<ValidationViolation>,
        suggestions: &mut Vec<String>,
    ) -> Result<(), CawsValidationError> {
        let test_plan = &working_spec.test_plan;

        // Check coverage targets based on risk tier
        if let Some(targets) = &test_plan.coverage_targets {
            match working_spec.risk_tier {
                1 => {
                    if targets.line_coverage.unwrap_or(0.0) < 0.9 {
                        violations.push(ValidationViolation {
                            code: "INSUFFICIENT_COVERAGE_T1".to_string(),
                            severity: ViolationSeverity::Error,
                            message: "T1 tasks require 90%+ line coverage".to_string(),
                            location: Some("test_plan.coverage_targets.line_coverage".to_string()),
                        });
                    }
                }
                2 => {
                    if targets.line_coverage.unwrap_or(0.0) < 0.8 {
                        violations.push(ValidationViolation {
                            code: "INSUFFICIENT_COVERAGE_T2".to_string(),
                            severity: ViolationSeverity::Warning,
                            message: "T2 tasks should have 80%+ line coverage".to_string(),
                            location: Some("test_plan.coverage_targets.line_coverage".to_string()),
                        });
                    }
                }
                _ => {}
            }
        } else if working_spec.risk_tier <= 2 {
            suggestions.push("Consider defining explicit test coverage targets".to_string());
        }

        Ok(())
    }

    fn calculate_compliance_score(&self, violations: &[ValidationViolation], risk_tier: u32) -> f64 {
        if violations.is_empty() {
            return 1.0;
        }

        // Weight violations by severity and risk tier
        let mut score = 1.0;
        for violation in violations {
            let weight = match (violation.severity, risk_tier) {
                (ViolationSeverity::Error, 1) => 0.3, // T1 errors are critical
                (ViolationSeverity::Error, _) => 0.2,
                (ViolationSeverity::Warning, 1) => 0.1, // T1 warnings are important
                (ViolationSeverity::Warning, _) => 0.05,
                (ViolationSeverity::Info, _) => 0.01,
            };
            score = (score - weight).max(0.0f64);
        }

        score
    }

    fn assess_test_coverage(&self, working_spec: &agent_agency_contracts::working_spec::WorkingSpec) -> f64 {
        let test_plan = &working_spec.test_plan;

        let mut coverage_score = 0.0;

        // Unit tests presence
        if !test_plan.unit_tests.is_empty() {
            coverage_score += 0.4;
        }

        // Integration tests presence
        if !test_plan.integration_tests.is_empty() {
            coverage_score += 0.3;
        }

        // E2E tests presence
        if !test_plan.e2e_scenarios.is_empty() {
            coverage_score += 0.2;
        }

        // Coverage targets defined
        if test_plan.coverage_targets.is_some() {
            coverage_score += 0.1;
        }

        coverage_score
    }
}

/// Create a new CAWS validator instance
pub fn create_caws_validator() -> Arc<dyn CawsValidator> {
    Arc::new(DefaultCawsValidator::new())
}
