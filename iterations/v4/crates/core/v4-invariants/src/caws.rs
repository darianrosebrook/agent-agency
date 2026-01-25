//! CAWS Invariants
//!
//! Coding-Agent Working Spec invariants ported from V3 with enhancements.
//! These are the non-waivable rules that must be enforced.

use serde::{Deserialize, Serialize};

use crate::checker::{InvariantResult, InvariantViolation};
use crate::Severity;

/// CAWS invariant rules that are never waivable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl CAWSInvariant {
    /// Get the invariant ID
    pub fn id(&self) -> &'static str {
        match self {
            Self::NoConsoleDotLog => "CAWS-LOG-001",
            Self::NoPlaceholderCode => "CAWS-CODE-001",
            Self::RequireStructuredLogging => "CAWS-LOG-002",
            Self::NoHardcodedSecrets => "CAWS-SEC-001",
            Self::RequireErrorHandling => "CAWS-ERR-001",
            Self::SemanticVersioning => "CAWS-API-001",
            Self::APIBackwardCompat => "CAWS-API-002",
            Self::CAWSCompliance => "CAWS-STD-001",
        }
    }

    /// Get severity of violations
    pub fn severity(&self) -> Severity {
        match self {
            Self::NoPlaceholderCode => Severity::Critical,
            Self::NoHardcodedSecrets => Severity::Critical,
            Self::NoConsoleDotLog => Severity::Error,
            Self::SemanticVersioning => Severity::Error,
            Self::APIBackwardCompat => Severity::Error,
            Self::RequireErrorHandling => Severity::Warning,
            Self::RequireStructuredLogging => Severity::Warning,
            Self::CAWSCompliance => Severity::Info,
        }
    }

    /// Get all CAWS invariants
    pub fn all() -> [Self; 8] {
        [
            Self::NoConsoleDotLog,
            Self::NoPlaceholderCode,
            Self::RequireStructuredLogging,
            Self::NoHardcodedSecrets,
            Self::RequireErrorHandling,
            Self::SemanticVersioning,
            Self::APIBackwardCompat,
            Self::CAWSCompliance,
        ]
    }
}

// InvariantId is defined in core.rs

/// Check content for CAWS invariant violations
pub fn run_caws_checks(content: &str) -> InvariantResult {
    let mut violations = Vec::new();
    let content_lower = content.to_lowercase();

    // Check: No console.log
    if content_lower.contains("console.log") || content_lower.contains("console.") {
        violations.push(InvariantViolation {
            invariant_id: CAWSInvariant::NoConsoleDotLog.id().to_string(),
            message: "Use of console.log detected - must use structured logging".to_string(),
            severity: Severity::Error,
            location: None,
        });
    }

    // Check: No placeholders
    let placeholder_patterns = ["todo", "fixme", "placeholder", "stub", "unimplemented"];
    for pattern in placeholder_patterns {
        if content_lower.contains(pattern) {
            violations.push(InvariantViolation {
                invariant_id: CAWSInvariant::NoPlaceholderCode.id().to_string(),
                message: format!("{} detected - incomplete implementation", pattern.to_uppercase()),
                severity: Severity::Critical,
                location: None,
            });
            break; // One violation is enough
        }
    }

    // Check: No hardcoded secrets
    let secret_keywords = ["password", "secret", "api_key", "apikey", "token", "credential"];
    let hardcoded_patterns = ["hardcoded", "hardcode", "hard-coded", "hard_coded"];

    for secret in secret_keywords {
        for hardcoded in hardcoded_patterns {
            if content_lower.contains(secret) && content_lower.contains(hardcoded) {
                violations.push(InvariantViolation {
                    invariant_id: CAWSInvariant::NoHardcodedSecrets.id().to_string(),
                    message: format!("Hardcoded {} detected - use environment variables", secret),
                    severity: Severity::Critical,
                    location: None,
                });
            }
        }
    }

    InvariantResult {
        passed: violations.is_empty(),
        violations,
        checked_at: chrono::Utc::now(),
    }
}

/// Check a file path against blocked patterns
pub fn check_blocked_paths(path: &str, blocked: &[String]) -> Option<InvariantViolation> {
    for pattern in blocked {
        // Simple glob matching (could use glob crate for more complex patterns)
        let pattern_clean = pattern.trim_matches('*');
        if path.contains(pattern_clean) {
            return Some(InvariantViolation {
                invariant_id: "CAWS-PATH-001".to_string(),
                message: format!("Path {} is blocked by pattern {}", path, pattern),
                severity: Severity::Error,
                location: Some(path.to_string()),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_console_log() {
        let result = run_caws_checks("console.log('debug')");
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn test_no_placeholders() {
        let result = run_caws_checks("TODO: implement this");
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.severity == Severity::Critical));
    }

    #[test]
    fn test_clean_content() {
        let result = run_caws_checks("fn main() { println!(\"Hello\"); }");
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_hardcoded_secrets() {
        let result = run_caws_checks("Using hardcoded password for testing");
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.invariant_id == "CAWS-SEC-001"));
    }

    #[test]
    fn test_blocked_paths() {
        let blocked = vec!["**/node_modules/**".to_string(), "**/.git/**".to_string()];

        let violation = check_blocked_paths("src/node_modules/package/index.js", &blocked);
        assert!(violation.is_some());

        let no_violation = check_blocked_paths("src/components/Button.tsx", &blocked);
        assert!(no_violation.is_none());
    }
}
