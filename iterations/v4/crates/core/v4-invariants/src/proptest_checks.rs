//! Property-Based Tests for Invariants
//!
//! Uses proptest to verify invariant properties hold across a wide
//! range of inputs. This catches edge cases that unit tests miss.

#[cfg(test)]
mod tests {
    use crate::caws::run_caws_checks;
    use crate::checker::{compute_hash, InvariantChecker};
    use crate::core::CoreInvariant;
    use proptest::prelude::*;

    // ========== Hash Invariant Properties ==========

    proptest! {
        /// Hash output is always 64 characters (SHA-256 hex)
        #[test]
        fn hash_length_is_always_64(content in ".*") {
            let hash = compute_hash(&content);
            prop_assert_eq!(hash.len(), 64);
        }

        /// Same input always produces same hash (deterministic)
        #[test]
        fn hash_is_deterministic(content in ".*") {
            let hash1 = compute_hash(&content);
            let hash2 = compute_hash(&content);
            prop_assert_eq!(hash1, hash2);
        }

        /// Different inputs produce different hashes (collision resistance)
        /// Note: This is probabilistic - collisions are theoretically possible
        /// but astronomically unlikely for SHA-256
        #[test]
        fn different_inputs_different_hashes(a in ".+", b in ".+") {
            prop_assume!(a != b);
            let hash_a = compute_hash(&a);
            let hash_b = compute_hash(&b);
            prop_assert_ne!(hash_a, hash_b);
        }

        /// Hash contains only hex characters
        #[test]
        fn hash_is_valid_hex(content in ".*") {
            let hash = compute_hash(&content);
            prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    // ========== CAWS Check Properties ==========

    proptest! {
        /// Clean content (no keywords) should always pass
        #[test]
        fn clean_content_passes(
            content in "[a-zA-Z0-9 ]{0,100}"
                .prop_filter("no bad keywords",
                    |s| !s.to_lowercase().contains("console")
                        && !s.to_lowercase().contains("todo")
                        && !s.to_lowercase().contains("fixme")
                        && !s.to_lowercase().contains("placeholder")
                        && !s.to_lowercase().contains("stub")
                        && !s.to_lowercase().contains("unimplemented")
                        && !s.to_lowercase().contains("password")
                        && !s.to_lowercase().contains("secret")
                        && !s.to_lowercase().contains("api_key")
                        && !s.to_lowercase().contains("token")
                        && !s.to_lowercase().contains("credential")
                        && !s.to_lowercase().contains("hardcoded")
                )
        ) {
            let result = run_caws_checks(&content);
            prop_assert!(result.passed, "Clean content should pass: {:?}", result.violations);
        }

        /// Content with console.log should always fail
        #[test]
        fn console_log_always_fails(prefix in "[a-zA-Z0-9 ]{0,20}", suffix in "[a-zA-Z0-9 ]{0,20}") {
            let content = format!("{}console.log('test'){}", prefix, suffix);
            let result = run_caws_checks(&content);
            prop_assert!(!result.passed, "console.log should fail");
            prop_assert!(result.violations.iter().any(|v| v.invariant_id == "CAWS-LOG-001"));
        }

        /// Content with placeholder keywords should always fail
        #[test]
        fn placeholder_always_fails(
            prefix in "[a-zA-Z0-9 ]{0,20}",
            suffix in "[a-zA-Z0-9 ]{0,20}",
            keyword in prop_oneof!["TODO", "FIXME", "PLACEHOLDER", "STUB"]
        ) {
            let content = format!("{}{}: implement this{}", prefix, keyword, suffix);
            let result = run_caws_checks(&content);
            prop_assert!(!result.passed, "Placeholder {} should fail", keyword);
        }

        /// Result always has a timestamp
        #[test]
        fn result_always_has_timestamp(content in ".*") {
            let result = run_caws_checks(&content);
            // Timestamp should be recent (within last minute)
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(result.checked_at);
            prop_assert!(diff.num_seconds() < 60);
        }
    }

    // ========== Structured Response Properties ==========

    proptest! {
        /// Valid JSON always passes structured response check
        #[test]
        fn valid_json_passes(
            key in "[a-z]{1,10}",
            value in "[a-zA-Z0-9]{1,20}"
        ) {
            let checker = InvariantChecker::new();
            let json = format!(r#"{{"{}": "{}"}}"#, key, value);
            let result = checker.check_structured_response(&json);
            prop_assert!(result.passed, "Valid JSON should pass: {}", json);
        }

        /// Deeply nested JSON still passes
        #[test]
        fn nested_json_passes(depth in 1usize..5) {
            let checker = InvariantChecker::new();
            let mut json = r#"{"value": "leaf"}"#.to_string();
            for _ in 0..depth {
                json = format!(r#"{{"nested": {}}}"#, json);
            }
            let result = checker.check_structured_response(&json);
            prop_assert!(result.passed, "Nested JSON should pass: {}", json);
        }

        /// JSON arrays pass
        #[test]
        fn json_array_passes(len in 0usize..10) {
            let checker = InvariantChecker::new();
            let items: Vec<String> = (0..len).map(|i| format!("\"item{}\"", i)).collect();
            let json = format!("[{}]", items.join(", "));
            let result = checker.check_structured_response(&json);
            prop_assert!(result.passed, "JSON array should pass: {}", json);
        }
    }

    // ========== Bounded Memory Properties ==========

    proptest! {
        /// Memory within bounds always passes
        #[test]
        fn memory_within_bounds_passes(size in 0usize..1000, max in 1000usize..10000) {
            let checker = InvariantChecker::new();
            let result = checker.check_bounded_memory(size, max);
            prop_assert!(result.passed, "Size {} within max {} should pass", size, max);
        }

        /// Memory exceeding bounds always fails
        #[test]
        fn memory_exceeding_bounds_fails(size in 1001usize..10000, max in 0usize..1000) {
            prop_assume!(size > max);
            let checker = InvariantChecker::new();
            let result = checker.check_bounded_memory(size, max);
            prop_assert!(!result.passed, "Size {} exceeding max {} should fail", size, max);
        }

        /// Boundary condition: size == max passes
        #[test]
        fn memory_at_boundary_passes(max in 1usize..10000) {
            let checker = InvariantChecker::new();
            let result = checker.check_bounded_memory(max, max);
            prop_assert!(result.passed, "Size {} at max {} should pass", max, max);
        }
    }

    // ========== Iteration Bound Properties ==========

    proptest! {
        /// Iteration within bounds always passes
        #[test]
        fn iteration_within_bounds_passes(current in 0u32..100, max in 100u32..1000) {
            let checker = InvariantChecker::new();
            let result = checker.check_iteration_bound(current, max);
            prop_assert!(result.passed, "Iteration {} within max {} should pass", current, max);
        }

        /// Iteration at max always fails (must be strictly less than)
        #[test]
        fn iteration_at_max_fails(max in 1u32..1000) {
            let checker = InvariantChecker::new();
            let result = checker.check_iteration_bound(max, max);
            prop_assert!(!result.passed, "Iteration {} at max {} should fail", max, max);
        }

        /// Iteration exceeding max always fails
        #[test]
        fn iteration_exceeding_max_fails(max in 1u32..100, excess in 1u32..100) {
            let checker = InvariantChecker::new();
            let current = max + excess;
            let result = checker.check_iteration_bound(current, max);
            prop_assert!(!result.passed, "Iteration {} exceeding max {} should fail", current, max);
        }
    }

    // ========== Explicit State Properties ==========

    proptest! {
        /// All required keys present always passes
        #[test]
        fn all_required_present_passes(
            extra_keys in prop::collection::vec("[a-z]{1,10}", 0..5)
        ) {
            let checker = InvariantChecker::new();
            let required = ["task_id", "status"];
            let mut state_keys: Vec<&str> = required.to_vec();
            for key in &extra_keys {
                state_keys.push(key);
            }
            let result = checker.check_explicit_state(&state_keys, &required);
            prop_assert!(result.passed, "All required present should pass");
        }

        /// Empty required set always passes
        #[test]
        fn empty_required_always_passes(
            state_keys in prop::collection::vec("[a-z]{1,10}", 0..10)
        ) {
            let checker = InvariantChecker::new();
            let state_refs: Vec<&str> = state_keys.iter().map(|s| s.as_str()).collect();
            let result = checker.check_explicit_state(&state_refs, &[]);
            prop_assert!(result.passed, "Empty required should always pass");
        }
    }

    // ========== Hash Validation Properties ==========

    proptest! {
        /// Valid 64-char hex hash always passes
        #[test]
        fn valid_hash_passes(hash in "[a-f0-9]{64}") {
            let checker = InvariantChecker::new();
            let result = checker.check_event_hash(Some(&hash));
            prop_assert!(result.passed, "Valid hash should pass: {}", hash);
        }

        /// Invalid length hash always fails
        #[test]
        fn invalid_length_hash_fails(hash in "[a-f0-9]{1,63}|[a-f0-9]{65,100}") {
            let checker = InvariantChecker::new();
            let result = checker.check_event_hash(Some(&hash));
            prop_assert!(!result.passed, "Invalid length hash should fail: {}", hash);
        }

        /// Missing hash always fails
        #[test]
        fn missing_hash_fails(_dummy in 0..1) {
            let checker = InvariantChecker::new();
            let result = checker.check_event_hash(None);
            prop_assert!(!result.passed, "Missing hash should fail");
        }
    }

    // ========== Core Invariant Properties ==========

    #[test]
    fn all_invariants_have_unique_ids() {
        let ids: Vec<_> = CoreInvariant::all().iter().map(|i| i.id().as_str()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len(), "All invariant IDs must be unique");
    }

    #[test]
    fn all_invariants_have_descriptions() {
        for inv in CoreInvariant::all() {
            let desc = inv.description();
            assert!(!desc.is_empty(), "{:?} should have description", inv);
        }
    }

    #[test]
    fn critical_invariants_are_subset() {
        let all_invariants = CoreInvariant::all();
        let critical: Vec<_> = all_invariants
            .iter()
            .filter(|i| i.is_critical())
            .collect();

        // At least some should be critical
        assert!(!critical.is_empty(), "Should have critical invariants");

        // Not all should be critical
        assert!(
            critical.len() < all_invariants.len(),
            "Not all invariants should be critical"
        );
    }

    // ========== Result Merge Properties ==========

    proptest! {
        /// Merging with pass preserves original pass/fail
        #[test]
        fn merge_with_pass_preserves(original_passed: bool) {
            let original = if original_passed {
                crate::checker::InvariantResult::pass()
            } else {
                crate::checker::InvariantResult::fail(vec![
                    crate::checker::InvariantViolation {
                        invariant_id: "TEST-001".to_string(),
                        message: "test".to_string(),
                        severity: crate::Severity::Error,
                        location: None,
                    }
                ])
            };

            let pass = crate::checker::InvariantResult::pass();
            let merged = original.merge(pass);

            prop_assert_eq!(merged.passed, original_passed);
        }

        /// Merging with fail always results in fail
        #[test]
        fn merge_with_fail_always_fails(original_passed: bool) {
            let original = if original_passed {
                crate::checker::InvariantResult::pass()
            } else {
                crate::checker::InvariantResult::fail(vec![])
            };

            let fail = crate::checker::InvariantResult::fail(vec![
                crate::checker::InvariantViolation {
                    invariant_id: "TEST-001".to_string(),
                    message: "test".to_string(),
                    severity: crate::Severity::Error,
                    location: None,
                }
            ]);

            let merged = original.merge(fail);
            prop_assert!(!merged.passed, "Merge with fail should always fail");
        }
    }
}
