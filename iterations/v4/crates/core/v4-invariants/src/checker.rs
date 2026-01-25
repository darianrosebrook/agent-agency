//! Invariant Checker
//!
//! The main entry point for checking invariants.
//! Combines Core and CAWS invariants into a unified checker.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::CoreInvariant;
use crate::Severity;

/// Result of running invariant checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantResult {
    /// Whether all invariants passed
    pub passed: bool,
    /// List of violations
    pub violations: Vec<InvariantViolation>,
    /// Timestamp of check
    #[serde(with = "chrono::serde::ts_seconds")]
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

impl InvariantResult {
    /// Create a passing result
    pub fn pass() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
            checked_at: chrono::Utc::now(),
        }
    }

    /// Create a failing result
    pub fn fail(violations: Vec<InvariantViolation>) -> Self {
        Self {
            passed: false,
            violations,
            checked_at: chrono::Utc::now(),
        }
    }

    /// Get blocking violations (Critical or Error severity)
    pub fn blocking_violations(&self) -> Vec<&InvariantViolation> {
        self.violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Critical | Severity::Error))
            .collect()
    }

    /// Check if there are any critical violations
    pub fn has_critical(&self) -> bool {
        self.violations
            .iter()
            .any(|v| matches!(v.severity, Severity::Critical))
    }

    /// Merge with another result
    pub fn merge(mut self, other: Self) -> Self {
        self.passed = self.passed && other.passed;
        self.violations.extend(other.violations);
        self
    }
}

/// A single invariant violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantViolation {
    /// ID of the violated invariant
    pub invariant_id: String,
    /// Human-readable message
    pub message: String,
    /// Severity of the violation
    pub severity: Severity,
    /// Location context (file path, line number, etc.)
    pub location: Option<String>,
}

/// Invariant checker that runs all checks
pub struct InvariantChecker {
    /// Whether to fail on first violation
    fail_fast: bool,
    /// Content hash for audit trail
    content_hash: Option<String>,
}

impl InvariantChecker {
    /// Create a new checker
    pub fn new() -> Self {
        Self {
            fail_fast: false,
            content_hash: None,
        }
    }

    /// Enable fail-fast mode
    pub fn fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }

    /// Check content and compute hash for audit trail
    pub fn with_content_hash(mut self, content: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        self.content_hash = Some(hex::encode(hasher.finalize()));
        self
    }

    /// Get the content hash (for audit trail)
    pub fn content_hash(&self) -> Option<&str> {
        self.content_hash.as_deref()
    }

    /// Check a structured LLM response (INV-CORE-01)
    pub fn check_structured_response(&self, response: &str) -> InvariantResult {
        // Try to parse as JSON
        match serde_json::from_str::<serde_json::Value>(response) {
            Ok(_) => InvariantResult::pass(),
            Err(_) => {
                // Check if it looks like free-form text (multiple sentences, paragraphs)
                let has_paragraphs = response.contains("\n\n");
                let sentence_count = response.matches(". ").count();

                if has_paragraphs || sentence_count > 3 {
                    InvariantResult::fail(vec![InvariantViolation {
                        invariant_id: CoreInvariant::NoFreeFormCoT.id().to_string(),
                        message: "LLM response appears to be free-form text, not structured data"
                            .to_string(),
                        severity: Severity::Critical,
                        location: None,
                    }])
                } else {
                    InvariantResult::pass()
                }
            }
        }
    }

    /// Check that state is explicit (INV-CORE-02)
    pub fn check_explicit_state(&self, state_keys: &[&str], required: &[&str]) -> InvariantResult {
        let missing: Vec<_> = required
            .iter()
            .filter(|r| !state_keys.contains(r))
            .collect();

        if missing.is_empty() {
            InvariantResult::pass()
        } else {
            InvariantResult::fail(vec![InvariantViolation {
                invariant_id: CoreInvariant::ExplicitStateOnly.id().to_string(),
                message: format!("Missing required state keys: {:?}", missing),
                severity: Severity::Critical,
                location: None,
            }])
        }
    }

    /// Check bounded memory (INV-CORE-03)
    pub fn check_bounded_memory(&self, memory_size: usize, max_size: usize) -> InvariantResult {
        if memory_size <= max_size {
            InvariantResult::pass()
        } else {
            InvariantResult::fail(vec![InvariantViolation {
                invariant_id: CoreInvariant::BoundedMemory.id().to_string(),
                message: format!(
                    "Memory size {} exceeds maximum {}",
                    memory_size, max_size
                ),
                severity: Severity::Error,
                location: None,
            }])
        }
    }

    /// Check termination guarantee (INV-CORE-07)
    pub fn check_iteration_bound(
        &self,
        current: u32,
        max: u32,
    ) -> InvariantResult {
        if current < max {
            InvariantResult::pass()
        } else {
            InvariantResult::fail(vec![InvariantViolation {
                invariant_id: CoreInvariant::TerminationGuarantee.id().to_string(),
                message: format!("Iteration {} reached maximum {}", current, max),
                severity: Severity::Error,
                location: None,
            }])
        }
    }

    /// Check that routing decision is logged (INV-CORE-08)
    pub fn check_routing_logged(&self, has_reasoning: bool) -> InvariantResult {
        if has_reasoning {
            InvariantResult::pass()
        } else {
            InvariantResult::fail(vec![InvariantViolation {
                invariant_id: CoreInvariant::NoHiddenRouters.id().to_string(),
                message: "Routing decision missing reasoning".to_string(),
                severity: Severity::Error,
                location: None,
            }])
        }
    }

    /// Check crypto audit trail (INV-CORE-10)
    pub fn check_event_hash(&self, event_hash: Option<&str>) -> InvariantResult {
        match event_hash {
            Some(hash) if hash.len() == 64 => InvariantResult::pass(), // SHA-256 hex
            Some(hash) => InvariantResult::fail(vec![InvariantViolation {
                invariant_id: CoreInvariant::CryptoAuditTrail.id().to_string(),
                message: format!("Invalid hash length: {} (expected 64)", hash.len()),
                severity: Severity::Error,
                location: None,
            }]),
            None => InvariantResult::fail(vec![InvariantViolation {
                invariant_id: CoreInvariant::CryptoAuditTrail.id().to_string(),
                message: "Event missing content hash".to_string(),
                severity: Severity::Error,
                location: None,
            }]),
        }
    }
}

impl Default for InvariantChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute SHA-256 hash of content
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_response_json() {
        let checker = InvariantChecker::new();
        let result = checker.check_structured_response(r#"{"action": "read", "path": "file.rs"}"#);
        assert!(result.passed);
    }

    #[test]
    fn test_structured_response_freeform() {
        let checker = InvariantChecker::new();
        let freeform = "Let me think about this. First, I need to understand the problem. \
                        Then, I'll analyze the code. After that, I can propose a solution. \
                        Finally, we'll implement it.\n\nThis is a complex task.";
        let result = checker.check_structured_response(freeform);
        assert!(!result.passed);
        assert!(result.has_critical());
    }

    #[test]
    fn test_explicit_state() {
        let checker = InvariantChecker::new();

        let result = checker.check_explicit_state(
            &["task_id", "status", "progress"],
            &["task_id", "status"],
        );
        assert!(result.passed);

        let result = checker.check_explicit_state(&["task_id"], &["task_id", "status"]);
        assert!(!result.passed);
    }

    #[test]
    fn test_bounded_memory() {
        let checker = InvariantChecker::new();

        let result = checker.check_bounded_memory(100, 1000);
        assert!(result.passed);

        let result = checker.check_bounded_memory(1500, 1000);
        assert!(!result.passed);
    }

    #[test]
    fn test_iteration_bound() {
        let checker = InvariantChecker::new();

        let result = checker.check_iteration_bound(5, 100);
        assert!(result.passed);

        let result = checker.check_iteration_bound(100, 100);
        assert!(!result.passed);
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash("test content");
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_event_hash_validation() {
        let checker = InvariantChecker::new();

        // Valid hash
        let valid_hash = "a".repeat(64);
        let result = checker.check_event_hash(Some(&valid_hash));
        assert!(result.passed);

        // Invalid length
        let result = checker.check_event_hash(Some("tooshort"));
        assert!(!result.passed);

        // Missing
        let result = checker.check_event_hash(None);
        assert!(!result.passed);
    }

    #[test]
    fn test_result_merge() {
        let r1 = InvariantResult::pass();
        let r2 = InvariantResult::fail(vec![InvariantViolation {
            invariant_id: "TEST-001".to_string(),
            message: "Test violation".to_string(),
            severity: Severity::Error,
            location: None,
        }]);

        let merged = r1.merge(r2);
        assert!(!merged.passed);
        assert_eq!(merged.violations.len(), 1);
    }

    #[test]
    fn test_has_critical_true_when_critical_present() {
        let result = InvariantResult::fail(vec![InvariantViolation {
            invariant_id: "TEST-001".to_string(),
            message: "Critical issue".to_string(),
            severity: Severity::Critical,
            location: None,
        }]);
        assert!(result.has_critical());
    }

    #[test]
    fn test_has_critical_false_when_no_critical() {
        let result = InvariantResult::fail(vec![InvariantViolation {
            invariant_id: "TEST-001".to_string(),
            message: "Error issue".to_string(),
            severity: Severity::Error,
            location: None,
        }]);
        assert!(!result.has_critical());
    }

    #[test]
    fn test_blocking_violations_returns_correct_severities() {
        let result = InvariantResult::fail(vec![
            InvariantViolation {
                invariant_id: "TEST-001".to_string(),
                message: "Critical".to_string(),
                severity: Severity::Critical,
                location: None,
            },
            InvariantViolation {
                invariant_id: "TEST-002".to_string(),
                message: "Error".to_string(),
                severity: Severity::Error,
                location: None,
            },
            InvariantViolation {
                invariant_id: "TEST-003".to_string(),
                message: "Warning".to_string(),
                severity: Severity::Warning,
                location: None,
            },
            InvariantViolation {
                invariant_id: "TEST-004".to_string(),
                message: "Info".to_string(),
                severity: Severity::Info,
                location: None,
            },
        ]);

        let blocking = result.blocking_violations();
        assert_eq!(blocking.len(), 2);
        assert!(blocking.iter().all(|v| matches!(v.severity, Severity::Critical | Severity::Error)));
    }

    #[test]
    fn test_blocking_violations_empty_when_only_warnings() {
        let result = InvariantResult::fail(vec![InvariantViolation {
            invariant_id: "TEST-001".to_string(),
            message: "Warning".to_string(),
            severity: Severity::Warning,
            location: None,
        }]);

        assert!(result.blocking_violations().is_empty());
    }

    #[test]
    fn test_fail_fast_builder() {
        let checker = InvariantChecker::new().fail_fast();
        // Verify the builder pattern returns a modified Self, not default
        // We can test this indirectly by checking the content_hash method
        let checker_with_hash = InvariantChecker::new()
            .fail_fast()
            .with_content_hash("test");
        assert!(checker_with_hash.content_hash().is_some());
        // Also verify the hash is correct (not reset by fail_fast)
        assert_eq!(checker_with_hash.content_hash().unwrap().len(), 64);
    }

    #[test]
    fn test_fail_fast_preserves_content_hash() {
        // Test that fail_fast() doesn't reset the content_hash that was set before
        let checker = InvariantChecker::new()
            .with_content_hash("test content")
            .fail_fast();
        // If fail_fast returned Default::default(), content_hash would be None
        assert!(checker.content_hash().is_some());
    }

    #[test]
    fn test_fail_fast_chaining() {
        // Test that the builder chain order doesn't matter
        let checker1 = InvariantChecker::new()
            .fail_fast()
            .with_content_hash("data");
        let checker2 = InvariantChecker::new()
            .with_content_hash("data")
            .fail_fast();

        // Both should have the same hash
        assert_eq!(checker1.content_hash(), checker2.content_hash());
    }

    #[test]
    fn test_with_content_hash_returns_correct_hash() {
        let checker = InvariantChecker::new().with_content_hash("hello");
        let hash = checker.content_hash();
        assert!(hash.is_some());
        assert_eq!(hash.unwrap().len(), 64);
        // SHA-256 of "hello" is well-known
        assert_eq!(
            hash.unwrap(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_content_hash_none_when_not_set() {
        let checker = InvariantChecker::new();
        assert!(checker.content_hash().is_none());
    }

    #[test]
    fn test_check_structured_response_sentence_boundary() {
        let checker = InvariantChecker::new();

        // Exactly 3 sentences - should pass
        let three = "First. Second. Third. End";
        let result = checker.check_structured_response(three);
        assert!(result.passed);

        // More than 3 sentences - should fail
        let four = "First. Second. Third. Fourth. End";
        let result = checker.check_structured_response(four);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_structured_response_paragraphs_or_sentences() {
        let checker = InvariantChecker::new();

        // Has paragraphs but few sentences
        let paragraphs = "Line one.\n\nLine two.";
        let result = checker.check_structured_response(paragraphs);
        assert!(!result.passed);

        // Has many sentences but no paragraphs
        let sentences = "One. Two. Three. Four. Five.";
        let result = checker.check_structured_response(sentences);
        assert!(!result.passed);
    }
}
