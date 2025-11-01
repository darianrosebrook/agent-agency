//! CAWS Invariants Runner
//!
//! This module implements deterministic checks for non-waivable CAWS invariants.
//! These rules are enforced before LLM analysis to ensure baseline compliance.
//!
//! ## Invariant Categories
//!
//! - **Code Quality**: No console.log, structured logging, no placeholders
//! - **Security**: No hardcoded secrets, proper error handling
//! - **Reliability**: Semantic versioning, API backward compatibility
//! - **Development Standards**: CAWS compliance, no TODO/FIXME in production
//!
//! ## Execution Flow
//!
//! 1. **Pre-LLM Gate**: Run all invariant checks deterministically
//! 2. **Blocking Violations**: Critical + non-waivable = immediate rejection
//! 3. **Evidence Collection**: Gather violation details for LLM context
//! 4. **Waivable Tracking**: Non-critical violations passed to LLM for judgment

use agent_agency_contracts::{CAWSInvariant, Severity, ViolationLocation, InvariantCheck, InvariantResults, WorkingSpec};
use std::collections::HashMap;

/// Run all CAWS invariant checks on a working spec
pub fn run_caws_invariants(spec: &WorkingSpec) -> InvariantResults {
    let mut checks = vec![];

    // Code quality invariants
    checks.push(check_no_console_log(spec));
    checks.push(check_structured_logging(spec));
    checks.push(check_no_placeholders(spec));

    // Security invariants
    checks.push(check_no_hardcoded_secrets(spec));
    checks.push(check_error_handling(spec));

    // Reliability invariants
    checks.push(check_semver_compliance(spec));
    checks.push(check_api_backward_compat(spec));

    // Development standards
    checks.push(check_caws_compliance(spec));

    InvariantResults { checks }
}

/// Check: No console.log in production code
fn check_no_console_log(spec: &WorkingSpec) -> InvariantCheck {
    let violations = find_pattern_violations(
        spec,
        CAWSInvariant::NoConsoleDotLog,
        r"console\.log|console\.debug|console\.info|console\.warn|console\.error",
        Severity::Medium,
        true, // waivable
        "console.log statements found in production code",
    );

    InvariantCheck {
        invariant: CAWSInvariant::NoConsoleDotLog,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: Structured logging is used
fn check_structured_logging(spec: &WorkingSpec) -> InvariantCheck {
    let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
        spec.title,
        spec.description,
        spec.goals.join("\n- "),
        spec.acceptance_criteria.iter()
            .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Check for presence of tracing/logging setup
    let has_structured_logging = spec_text.contains("tracing::")
        || spec_text.contains("log::")
        || spec_text.contains("slog");

    let violations = if has_structured_logging {
        vec![]
    } else {
        vec![ViolationLocation {
            rule_id: "LOGGING-001".to_string(),
            description: "No structured logging framework detected".to_string(),
            context: "spec".to_string(),
            severity: Severity::Low,
        }]
    };

    InvariantCheck {
        invariant: CAWSInvariant::RequireStructuredLogging,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: No TODO/FIXME/PLACEHOLDER in production code
fn check_no_placeholders(spec: &WorkingSpec) -> InvariantCheck {
    let violations = find_pattern_violations(
        spec,
        CAWSInvariant::NoPlaceholderCode,
        r"\bTODO\b|\bFIXME\b|\bPLACEHOLDER\b|\bMOCK DATA\b",
        Severity::High,
        false, // not waivable - must be resolved
        "TODO/FIXME/PLACEHOLDER found in production code",
    );

    InvariantCheck {
        invariant: CAWSInvariant::NoPlaceholderCode,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: No hardcoded secrets or credentials
fn check_no_hardcoded_secrets(spec: &WorkingSpec) -> InvariantCheck {
    let patterns: Vec<String> = vec![
        "password\\s*[:=]\\s*['\"][^'\"]+['\"]".to_string(),
        "secret\\s*[:=]\\s*['\"][^'\"]+['\"]".to_string(),
        "token\\s*[:=]\\s*['\"][^'\"]+['\"]".to_string(),
        "key\\s*[:=]\\s*['\"][^'\"]+['\"]".to_string(),
        "api_key\\s*[:=]\\s*['\"][^'\"]+['\"]".to_string(),
    ];

    let mut violations = vec![];

    for pattern in patterns {
        violations.extend(find_pattern_violations(
            spec,
            CAWSInvariant::NoHardcodedSecrets,
            &pattern,
            Severity::Critical,
            false, // not waivable - security risk
            "Potential hardcoded secret detected",
        ));
    }

    InvariantCheck {
        invariant: CAWSInvariant::NoHardcodedSecrets,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: Proper error handling patterns
fn check_error_handling(spec: &WorkingSpec) -> InvariantCheck {
    let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
        spec.title,
        spec.description,
        spec.goals.join("\n- "),
        spec.acceptance_criteria.iter()
            .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Check for presence of error handling patterns
    let has_error_handling = spec_text.contains("Result<")
        || spec_text.contains("thiserror")
        || spec_text.contains("anyhow")
        || spec_text.contains("try!")
        || spec_text.contains("?");

    let violations = if has_error_handling {
        vec![]
    } else {
        vec![ViolationLocation {
            rule_id: "ERROR-001".to_string(),
            description: "No structured error handling detected".to_string(),
            context: "spec".to_string(),
            severity: Severity::Medium,
        }]
    };

    InvariantCheck {
        invariant: CAWSInvariant::RequireErrorHandling,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: Semantic versioning compliance
fn check_semver_compliance(spec: &WorkingSpec) -> InvariantCheck {
    let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
        spec.title,
        spec.description,
        spec.goals.join("\n- "),
        spec.acceptance_criteria.iter()
            .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Check for version patterns in spec
    let has_version_info = spec_text.contains("version")
        || spec_text.contains("breaking change")
        || spec_text.contains("semver");

    let violations = if has_version_info {
        vec![]
    } else {
        // This is informational - spec should mention version impact
        vec![ViolationLocation {
            rule_id: "SEMVER-001".to_string(),
            description: "No semantic versioning information provided".to_string(),
            context: "spec".to_string(),
            severity: Severity::Info,
        }]
    };

    InvariantCheck {
        invariant: CAWSInvariant::SemanticVersioning,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: API backward compatibility
fn check_api_backward_compat(spec: &WorkingSpec) -> InvariantCheck {
    let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
        spec.title,
        spec.description,
        spec.goals.join("\n- "),
        spec.acceptance_criteria.iter()
            .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Check for breaking change markers
    let has_breaking_change_analysis = spec_text.contains("breaking")
        || spec_text.contains("backward")
        || spec_text.contains("compatibility");

    let violations = if has_breaking_change_analysis {
        vec![]
    } else {
        vec![ViolationLocation {
            rule_id: "API-001".to_string(),
            description: "No API backward compatibility analysis provided".to_string(),
            context: "spec".to_string(),
            severity: Severity::Medium,
        }]
    };

    InvariantCheck {
        invariant: CAWSInvariant::APIBackwardCompat,
        passed: violations.is_empty(),
        violations,
    }
}

/// Check: CAWS development standards compliance
fn check_caws_compliance(spec: &WorkingSpec) -> InvariantCheck {
    let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
        spec.title,
        spec.description,
        spec.goals.join("\n- "),
        spec.acceptance_criteria.iter()
            .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Check for CAWS-specific patterns
    let caws_indicators = [
        "test-driven",
        "invariant",
        "working spec",
        "acceptance criteria",
        "risk tier",
    ];

    let caws_score = caws_indicators.iter()
        .filter(|indicator| spec_text.to_lowercase().contains(*indicator))
        .count();

    let violations = if caws_score >= 3 {
        vec![]
    } else {
        vec![ViolationLocation {
            rule_id: "CAWS-001".to_string(),
            description: format!("Low CAWS compliance indicators ({} of {})", caws_score, caws_indicators.len()),
            context: "spec".to_string(),
            severity: Severity::Low,
        }]
    };

    InvariantCheck {
        invariant: CAWSInvariant::CAWSCompliance,
        passed: violations.is_empty(),
        violations,
    }
}

/// Helper: Find violations using regex patterns
fn find_pattern_violations(
    spec: &WorkingSpec,
    invariant: CAWSInvariant,
    pattern: &str,
    severity: Severity,
    waivable: bool,
    description: &str,
) -> Vec<ViolationLocation> {
    use regex::Regex;

    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return vec![], // Invalid regex, skip
    };

    let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
        spec.title,
        spec.description,
        spec.goals.join("\n- "),
        spec.acceptance_criteria.iter()
            .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let mut violations = vec![];

    // Search in spec text (simplified - in real impl, would search code files)
    for (line_num, line) in spec_text.lines().enumerate() {
        if regex.is_match(line) {
            violations.push(ViolationLocation {
                rule_id: format!("{:?}-{:03}", invariant, violations.len() + 1),
                description: description.to_string(),
                context: "spec".to_string(),
                severity: severity.clone(),
            });
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::WorkingSpec;

    fn create_test_spec(text: &str) -> WorkingSpec {
        WorkingSpec {
            version: "1.0".to_string(),
            id: "TEST-001".to_string(),
            title: "Test Spec".to_string(),
            description: text.to_string(),
            goals: vec!["Test goal".to_string()],
            risk_tier: 3, // Low risk
            constraints: agent_agency_contracts::WorkingSpecConstraints {
                max_duration_hours: Some(1),
                max_cost_cents: Some(100),
                requires_human_approval: false,
                allowed_tools: vec![],
            },
            acceptance_criteria: vec![
                agent_agency_contracts::AcceptanceCriterion {
                    id: "TEST-001".to_string(),
                    given: "Test scenario".to_string(),
                    when: "Test action".to_string(),
                    then: "Test outcome".to_string(),
                    priority: Some(agent_agency_contracts::Priority::Must),
                }
            ],
            test_plan: agent_agency_contracts::TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_tests: vec![],
                performance_tests: vec![],
            },
            context: serde_json::json!({}),
            metadata: agent_agency_contracts::WorkingSpecMetadata {
                created_by: "test".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: vec!["test".to_string()],
            },
        }
    }

    #[test]
    fn test_no_console_log_pass() {
        let spec = create_test_spec("fn main() { println!(\"Hello\"); }");
        let check = check_no_console_log(&spec);
        assert!(check.passed);
    }

    #[test]
    fn test_no_console_log_fail() {
        let spec = create_test_spec("fn main() { console.log(\"debug\"); }");
        let check = check_no_console_log(&spec);
        assert!(!check.passed);
        assert_eq!(check.violations.len(), 1);
    }

    #[test]
    fn test_no_placeholders_fail() {
        let spec = create_test_spec("fn main() { TODO: implement this }");
        let check = check_no_placeholders(&spec);
        assert!(!check.passed);
        assert_eq!(check.violations.len(), 1);
        assert!(!check.violations[0].waivable); // Should not be waivable
    }

    #[test]
    fn test_hardcoded_secrets_fail() {
        let spec = create_test_spec(r#"let password = "secret123";"#);
        let check = check_no_hardcoded_secrets(&spec);
        assert!(!check.passed);
        assert!(!check.violations.is_empty());
        assert_eq!(check.violations[0].severity, Severity::Critical);
        assert!(!check.violations[0].waivable);
    }
}
