//! CAWS Invariant Rules
//!
//! Defines the non-waivable rules that judges must enforce before
//! any LLM reasoning. These are the hard boundaries that cannot
//! be overridden by AI judgment.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::working_spec::WorkingSpec;

/// CAWS invariant rules that are never waivable
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum CAWSInvariant {
    /// No console.log in production code
    NoConsoleDotLog,

    /// No TODO/FIXME/PLACEHOLDER in production code
    NoPlaceholderCode,

    /// Must use structured logging (tracing)
    RequireStructuredLogging,

    /// No hardcoded credentials or secrets
    NoHardcodedSecrets,

    /// All fallible operations must have error handling
    RequireErrorHandling,

    /// Breaking changes require major version bump
    SemanticVersioning,

    /// No breaking API changes without version strategy
    APIBackwardCompat,

    /// Must follow CAWS development standards
    CAWSCompliance,
}

/// Result of running invariant checks
#[derive(Debug, Clone, JsonSchema)]
pub struct InvariantResults {
    /// Individual check results
    pub checks: Vec<InvariantCheck>,
}

/// Individual invariant check result
#[derive(Debug, Clone, JsonSchema)]
pub struct InvariantCheck {
    /// The invariant that was checked
    pub invariant: CAWSInvariant,

    /// Whether the check passed
    pub passed: bool,

    /// Violations found (if any)
    pub violations: Vec<ViolationLocation>,
}

/// Location of a violation in the working spec
#[derive(Debug, Clone, JsonSchema)]
pub struct ViolationLocation {
    /// Rule that was violated
    pub rule_id: String,

    /// Human-readable description
    pub description: String,

    /// Location context (line, file, etc.)
    pub context: String,

    /// Severity level
    pub severity: Severity,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum Severity {
    /// Minor issues, suggestions
    Info,

    /// Notable concerns requiring attention
    Low,

    /// Significant problems affecting quality
    Medium,

    /// Critical issues blocking execution
    High,

    /// Catastrophic failures requiring immediate rejection
    Critical,
}

impl InvariantResults {
    /// Check if any blocking failures were found
    pub fn blocking_failure(&self) -> Option<ViolationLocation> {
        for check in &self.checks {
            if !check.passed {
                // Find the most severe violation
                if let Some(violation) = check.violations.iter()
                    .find(|v| matches!(v.severity, Severity::Critical | Severity::High)) {
                    return Some(violation.clone());
                }
            }
        }
        None
    }
}

/// Run all CAWS invariant checks on a working specification
pub fn run_caws_invariants(spec: &WorkingSpec) -> InvariantResults {
    let mut checks = vec![];

    checks.push(check_no_console_log(spec));
    checks.push(check_no_placeholders(spec));
    checks.push(check_structured_logging(spec));
    checks.push(check_no_hardcoded_secrets(spec));
    checks.push(check_error_handling(spec));
    checks.push(check_semver_compliance(spec));
    checks.push(check_api_backward_compat(spec));
    checks.push(check_caws_compliance(spec));

    InvariantResults { checks }
}

/// Check: No console.log in production code
fn check_no_console_log(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    let violations = if desc_lower.contains("console.log") || desc_lower.contains("console.") {
        vec![ViolationLocation {
            rule_id: "CAWS-LOG-001".to_string(),
            description: "Use of console.log detected - must use structured logging".to_string(),
            context: "Working specification description".to_string(),
            severity: Severity::High,
        }]
    } else {
        vec![]
    };

    InvariantCheck {
        invariant: CAWSInvariant::NoConsoleDotLog,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: No TODO/FIXME/PLACEHOLDER in production code
fn check_no_placeholders(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    let violations = if desc_lower.contains("todo") ||
                      desc_lower.contains("fixme") ||
                      desc_lower.contains("placeholder") ||
                      desc_lower.contains("stub") {
        vec![ViolationLocation {
            rule_id: "CAWS-CODE-001".to_string(),
            description: "TODO/FIXME/PLACEHOLDER detected - incomplete implementation".to_string(),
            context: "Working specification description".to_string(),
            severity: Severity::Critical,
        }]
    } else {
        vec![]
    };

    InvariantCheck {
        invariant: CAWSInvariant::NoPlaceholderCode,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: Structured logging required
fn check_structured_logging(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    let violations = if !desc_lower.contains("tracing") &&
                       !desc_lower.contains("structured logging") &&
                       !desc_lower.contains("log::") {
        vec![ViolationLocation {
            rule_id: "CAWS-LOG-002".to_string(),
            description: "No structured logging mentioned - must use tracing crate".to_string(),
            context: "Working specification description".to_string(),
            severity: Severity::Medium,
        }]
    } else {
        vec![]
    };

    InvariantCheck {
        invariant: CAWSInvariant::RequireStructuredLogging,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: No hardcoded secrets
fn check_no_hardcoded_secrets(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    let violations = if desc_lower.contains("hardcoded") &&
                      (desc_lower.contains("password") ||
                       desc_lower.contains("secret") ||
                       desc_lower.contains("key") ||
                       desc_lower.contains("token")) {
        vec![ViolationLocation {
            rule_id: "CAWS-SEC-001".to_string(),
            description: "Hardcoded secrets detected - use environment variables".to_string(),
            context: "Working specification description".to_string(),
            severity: Severity::Critical,
        }]
    } else {
        vec![]
    };

    InvariantCheck {
        invariant: CAWSInvariant::NoHardcodedSecrets,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: Semantic versioning compliance
fn check_semver_compliance(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    let violations = if desc_lower.contains("breaking change") ||
                      desc_lower.contains("breaking api") ||
                      desc_lower.contains("incompatible") {
        // Check if version bump is mentioned
        if !desc_lower.contains("major version") &&
           !desc_lower.contains("version bump") &&
           !desc_lower.contains("semver") {
            vec![ViolationLocation {
                rule_id: "CAWS-API-001".to_string(),
                description: "Breaking change without version strategy mentioned".to_string(),
                context: "Working specification description".to_string(),
                severity: Severity::High,
            }]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    InvariantCheck {
        invariant: CAWSInvariant::SemanticVersioning,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: Error handling required
fn check_error_handling(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    let violations = if (desc_lower.contains("network") ||
                        desc_lower.contains("file") ||
                        desc_lower.contains("database") ||
                        desc_lower.contains("api call") ||
                        desc_lower.contains("external service")) &&
                       !desc_lower.contains("error handling") &&
                       !desc_lower.contains("try/catch") &&
                       !desc_lower.contains("result") &&
                       !desc_lower.contains("option") &&
                       !desc_lower.contains("?") {
        vec![ViolationLocation {
            rule_id: "CAWS-ERR-001".to_string(),
            description: "Fallible operations without error handling mentioned".to_string(),
            context: "Working specification description".to_string(),
            severity: Severity::Medium,
        }]
    } else {
        vec![]
    };

    InvariantCheck {
        invariant: CAWSInvariant::RequireErrorHandling,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: API backward compatibility
fn check_api_backward_compat(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    let violations = if (desc_lower.contains("remove") ||
                        desc_lower.contains("delete") ||
                        desc_lower.contains("change signature")) &&
                       (desc_lower.contains("api") ||
                        desc_lower.contains("endpoint") ||
                        desc_lower.contains("function")) {
        // Check if backward compatibility is addressed
        if !desc_lower.contains("backward compatible") &&
           !desc_lower.contains("versioned") &&
           !desc_lower.contains("deprecated") {
            vec![ViolationLocation {
                rule_id: "CAWS-API-002".to_string(),
                description: "API change without backward compatibility strategy".to_string(),
                context: "Working specification description".to_string(),
                severity: Severity::High,
            }]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    InvariantCheck {
        invariant: CAWSInvariant::APIBackwardCompat,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: CAWS development standards compliance
fn check_caws_compliance(spec: &WorkingSpec) -> InvariantCheck {
    let desc_lower = spec.description.to_lowercase();

    // Check for CAWS-specific development practices
    let caws_indicators = [
        "test-driven",
        "invariant",
        "working spec",
        "acceptance criteria",
        "risk tier",
        "deterministic",
        "structured logging",
        "error handling",
    ];

    let caws_score = caws_indicators.iter()
        .filter(|indicator| desc_lower.contains(*indicator))
        .count();

    let violations = if caws_score >= 4 {
        vec![]
    } else {
        vec![ViolationLocation {
            rule_id: "CAWS-STD-001".to_string(),
            description: format!("Low CAWS standards compliance ({} of {} indicators)", caws_score, caws_indicators.len()),
            context: "Working specification description".to_string(),
            severity: Severity::Low,
        }]
    };

    InvariantCheck {
        invariant: CAWSInvariant::CAWSCompliance,
        passed: violations.is_empty(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use crate::working_spec::{
        WorkingSpec, WorkingSpecMetadata, WorkingSpecConstraints, AcceptanceCriterion,
        TestPlan, RollbackPlan, RollbackStrategy, DataImpact, WorkingSpecContext,
        ScopeRestrictions,
    };
    use crate::planning_io::{ChangeBudget, EnforcementMode};
    use crate::task_request::Environment;

    fn create_test_spec(description: &str) -> WorkingSpec {
        WorkingSpec {
            id: uuid::Uuid::new_v4().to_string(),
            version: "1.0".to_string(),
            title: "Test Spec".to_string(),
            description: description.to_string(),
            goals: vec!["Test goal".to_string()],
            risk_tier: 2,
            constraints: WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            milestones: vec![],
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "A1".to_string(),
                given: "User is logged in".to_string(),
                when: "User clicks button".to_string(),
                then: "Action is performed".to_string(),
                priority: None,
            }],
            test_plan: TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: RollbackPlan {
                strategy: RollbackStrategy::GitRevert,
                automated_steps: vec![],
                manual_steps: vec![],
                data_impact: DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(5),
            },
            context: WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![ScopeRestrictions {
                allowed_paths: vec!["src/".to_string()],
                blocked_paths: vec!["node_modules/".to_string()],
            }],
            change_budget: ChangeBudget {
                max_files: 10,
                max_loc: 1000,
                max_migrations: 3,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: EnforcementMode::Strict,
            },
            file_changes: vec![],
            coverage_targets: None,
            overview: String::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: Some(WorkingSpecMetadata {
                created_by: "test".to_string(),
                last_modified: chrono::Utc::now(),
                tags: vec![],
            }),
        }
    }

    #[test]
    fn test_no_console_log_pass() {
        let spec = create_test_spec("Implement logging with tracing crate");
        let results = run_caws_invariants(&spec);
        let check = results.checks.iter().find(|c| matches!(c.invariant, CAWSInvariant::NoConsoleDotLog)).unwrap();
        assert!(check.passed);
    }

    #[test]
    fn test_no_console_log_fail() {
        let spec = create_test_spec("Use console.log for debugging");
        let results = run_caws_invariants(&spec);
        let check = results.checks.iter().find(|c| matches!(c.invariant, CAWSInvariant::NoConsoleDotLog)).unwrap();
        assert!(!check.passed);
        assert_eq!(check.violations.len(), 1);
        assert_eq!(check.violations[0].severity, Severity::High);
    }

    #[test]
    fn test_no_placeholders_fail() {
        let spec = create_test_spec("TODO: implement authentication");
        let results = run_caws_invariants(&spec);
        let check = results.checks.iter().find(|c| matches!(c.invariant, CAWSInvariant::NoPlaceholderCode)).unwrap();
        assert!(!check.passed);
        assert_eq!(check.violations[0].severity, Severity::Critical);
    }

    #[test]
    fn test_blocking_failure_detection() {
        let spec = create_test_spec("TODO: implement authentication with console.log");
        let results = run_caws_invariants(&spec);
        let blocking = results.blocking_failure();
        assert!(blocking.is_some());
        assert_eq!(blocking.unwrap().severity, Severity::Critical);
    }
}
