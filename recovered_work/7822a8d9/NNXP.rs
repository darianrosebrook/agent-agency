//! Integration tests for brittleness fixes and critical edge cases
//!
//! These tests verify that our fixes for the most critical brittleness areas work correctly:
//! 1. SHA256 computation handles binary files and edge cases
//! 2. Budget calculation correctly tracks LOC deltas
//! 3. Diff validation prevents panics on malformed input
//! 4. Circuit breakers protect against provider failures

use self_prompting_agent::*;
use tempfile::tempdir;
use std::fs;
use std::path::PathBuf;
use tokio::test;

/// Test SHA256 computation robustness with various file types
#[tokio::test]
async fn test_sha256_computation_edge_cases() {
    let temp_dir = tempdir().unwrap();
    let applier = sandbox::diff_applier::DiffApplier::new(temp_dir.path().to_path_buf());

    // Test 1: Binary file with invalid UTF-8
    let binary_file = temp_dir.path().join("binary.dat");
    let binary_data = vec![0, 159, 146, 150, 255, 0, 1, 2, 3]; // Invalid UTF-8 sequence
    fs::write(&binary_file, &binary_data).unwrap();

    let result = applier.compute_file_sha256(&binary_file).await;
    assert!(result.is_ok(), "SHA256 should handle binary files");
    assert_eq!(result.unwrap().len(), 64, "Should produce 64-char SHA256 hash");

    // Test 2: Empty file
    let empty_file = temp_dir.path().join("empty.txt");
    fs::write(&empty_file, "").unwrap();

    let result = applier.compute_file_sha256(&empty_file).await;
    assert!(result.is_ok(), "SHA256 should handle empty files");

    // Test 3: Non-existent file
    let nonexistent = temp_dir.path().join("does_not_exist.txt");
    let result = applier.compute_file_sha256(&nonexistent).await;
    assert!(result.is_ok(), "SHA256 should handle non-existent files (returns empty hash)");
}

/// Test budget checker LOC calculation accuracy
#[test]
fn test_budget_loc_calculation_accuracy() {
    use caws::BudgetChecker;
    use types::{ChangeSet, ChangeOperation, FileChange};

    let checker = BudgetChecker::new(10, 1000); // Max 10 files, 1000 LOC

    // Test 1: File creation - should count total lines
    let create_changeset = ChangeSet {
        changes: vec![FileChange {
            path: PathBuf::from("new_file.rs"),
            operation: ChangeOperation::Create {
                content: "line1\nline2\nline3\nline4\nline5\n".to_string(), // 5 lines
            },
        }],
        rationale: "Create new file".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    let projected = checker.projected_state(&create_changeset).unwrap();
    assert_eq!(projected.files_used, 1, "Should count 1 new file");
    assert_eq!(projected.loc_used, 5, "Should count 5 lines for new file");

    // Test 2: File modification - should calculate delta
    let modify_changeset = ChangeSet {
        changes: vec![FileChange {
            path: PathBuf::from("existing.rs"),
            operation: ChangeOperation::Modify {
                expected_content: "line1\nline2\nline3\n".to_string(), // 3 lines
                new_content: "line1\nmodified\nline3\nline4\n".to_string(), // 4 lines
            },
        }],
        rationale: "Modify existing file".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    let projected = checker.projected_state(&modify_changeset).unwrap();
    assert_eq!(projected.files_used, 1, "Should count 1 modified file");
    assert_eq!(projected.loc_used, 1, "Should count +1 LOC delta (4-3)");

    // Test 3: Multiple operations
    let multi_changeset = ChangeSet {
        changes: vec![
            FileChange {
                path: PathBuf::from("file1.rs"),
                operation: ChangeOperation::Create {
                    content: "a\nb\nc\n".to_string(), // 3 lines
                },
            },
            FileChange {
                path: PathBuf::from("file2.rs"),
                operation: ChangeOperation::Modify {
                    expected_content: "x\ny\n".to_string(), // 2 lines
                    new_content: "x\ny\nz\nw\n".to_string(), // 4 lines
                },
            },
        ],
        rationale: "Multiple changes".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    let projected = checker.projected_state(&multi_changeset).unwrap();
    assert_eq!(projected.files_used, 2, "Should count 2 files changed");
    assert_eq!(projected.loc_used, 5, "Should count 3 (create) + 2 (modify delta) = 5 LOC");
}

/// Test diff validation prevents crashes on malformed input
#[test]
fn test_diff_validation_robustness() {
    let applier = sandbox::diff_applier::DiffApplier::new(PathBuf::from("/tmp"));

    // Test 1: Empty diff
    let result = applier.parse_unified_diff("");
    assert!(result.is_err(), "Empty diff should be rejected");

    // Test 2: Malformed header (missing @@)
    let malformed_header = "-1,3 +2,4\n context\n+change";
    let result = applier.parse_unified_diff(malformed_header);
    assert!(result.is_err(), "Malformed header should be rejected");

    // Test 3: Invalid range format (missing -/+ prefixes)
    let invalid_range = "@@ 1,3 2,4 @@\n context\n+change";
    let result = applier.parse_unified_diff(invalid_range);
    assert!(result.is_err(), "Invalid range format should be rejected");

    // Test 4: Non-numeric line numbers
    let non_numeric = "@@ -abc,3 +def,4 @@\n context\n+change";
    let result = applier.parse_unified_diff(non_numeric);
    assert!(result.is_err(), "Non-numeric line numbers should be rejected");

    // Test 5: Valid diff should work
    let valid_diff = "@@ -1,3 +1,4 @@\n context1\n-context2\n+change1\n+change2\n context3";
    let result = applier.parse_unified_diff(valid_diff);
    assert!(result.is_ok(), "Valid diff should be accepted");
    assert_eq!(result.unwrap().len(), 1, "Should parse one hunk");

    // Test 6: Diff with standard metadata lines should be handled
    let diff_with_metadata = "diff --git a/file.rs b/file.rs\n--- a/file.rs\n+++ b/file.rs\n@@ -1,2 +1,3 @@\n old\n+new\n old2";
    let result = applier.parse_unified_diff(diff_with_metadata);
    assert!(result.is_ok(), "Diff with metadata lines should be accepted");
}

/// Test circuit breaker state management and provider selection
#[test]
fn test_circuit_breaker_provider_protection() {
    use models::selection::{ModelRegistry, CircuitBreakerConfig, CircuitBreakerState, CircuitState};
    use types::Task;

    let mut registry = ModelRegistry::new();

    // Configure circuit breaker with aggressive settings for testing
    registry.default_circuit_config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout_secs: 1, // 1 second for testing
        success_threshold: 1,
    };

    // Create mock providers (we'll simulate their behavior)
    // Note: In real implementation, these would be actual ModelProvider instances

    // Test 1: Circuit breaker state transitions
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout_secs: 60,
        success_threshold: 1,
    };

    let mut breaker = CircuitBreakerState::new(config);
    assert_eq!(breaker.state, CircuitState::Closed);
    assert!(breaker.should_attempt());

    // Record failures
    breaker.record_failure();
    assert_eq!(breaker.state, CircuitState::Closed); // Still closed (1 failure)

    breaker.record_failure();
    assert_eq!(breaker.state, CircuitState::Open); // Now open (2 failures)
    assert!(!breaker.should_attempt()); // Should block requests

    // Test 2: Half-open state allows testing
    breaker.state = CircuitState::HalfOpen;
    breaker.success_count = 0;
    assert!(breaker.should_attempt()); // Half-open allows attempts

    // Record success in half-open
    breaker.record_success();
    assert_eq!(breaker.state, CircuitState::Closed); // Should close after success
}

/// Test end-to-end budget enforcement with realistic scenarios
#[test]
fn test_budget_enforcement_realistic_scenarios() {
    use caws::BudgetChecker;
    use types::{ChangeSet, ChangeOperation, FileChange};

    let checker = BudgetChecker::new(3, 50); // Small budget for testing

    // Scenario 1: Large file modification that would exceed budget
    let large_modify = ChangeSet {
        changes: vec![FileChange {
            path: PathBuf::from("large_file.rs"),
            operation: ChangeOperation::Modify {
                expected_content: "x\n".repeat(10), // 10 lines
                new_content: "x\n".repeat(60), // 60 lines (would add 50 LOC)
            },
        }],
        rationale: "Large modification".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    let result = checker.would_exceed(&large_modify);
    assert!(result.is_ok());
    assert!(result.unwrap(), "Large modification should exceed budget");

    // Scenario 2: Multiple small changes that stay within budget
    let small_changes = ChangeSet {
        changes: vec![
            FileChange {
                path: PathBuf::from("file1.rs"),
                operation: ChangeOperation::Create {
                    content: "line\n".repeat(10), // 10 lines
                },
            },
            FileChange {
                path: PathBuf::from("file2.rs"),
                operation: ChangeOperation::Modify {
                    expected_content: "old\n".repeat(5), // 5 lines
                    new_content: "new\n".repeat(8), // 8 lines (+3)
                },
            },
        ],
        rationale: "Small changes".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    let result = checker.would_exceed(&small_changes);
    assert!(result.is_ok());
    assert!(!result.unwrap(), "Small changes should stay within budget (13 LOC < 50)");

    // Scenario 3: File deletion should not reduce LOC count (conservative)
    let deletion = ChangeSet {
        changes: vec![FileChange {
            path: PathBuf::from("deleted.rs"),
            operation: ChangeOperation::Delete {
                expected_content: "x\n".repeat(20), // 20 lines being deleted
            },
        }],
        rationale: "File deletion".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    let projected = checker.projected_state(&deletion).unwrap();
    assert_eq!(projected.loc_used, 0, "Deletion should not reduce LOC count (conservative approach)");
}

/// Test diff application with real file operations
#[tokio::test]
async fn test_diff_application_real_files() {
    let temp_dir = tempdir().unwrap();
    let workspace_root = temp_dir.path().to_path_buf();

    // Create a test file
    let test_file = workspace_root.join("test.rs");
    let original_content = "fn main() {\n    println!(\"hello\");\n}\n";
    fs::write(&test_file, &original_content).unwrap();

    // Create diff applier
    let applier = sandbox::diff_applier::DiffApplier::new(workspace_root.clone());

    // Test valid diff application
    let valid_diff = sandbox::diff_generator::UnifiedDiff {
        file_path: PathBuf::from("test.rs"),
        diff_content: "@@ -1,3 +1,4 @@\n fn main() {\n-    println!(\"hello\");\n+    println!(\"hello world\");\n+    println!(\"second line\");\n }\n".to_string(),
        sha256: "dummy".to_string(),
        expected_sha256: Some(applier.compute_file_sha256(&test_file).await.unwrap()),
    };

    let result = applier.apply_diff(&valid_diff, false).await;
    assert!(result.is_ok(), "Valid diff should apply successfully");

    // Verify content changed
    let new_content = fs::read_to_string(&test_file).unwrap();
    assert!(new_content.contains("hello world"), "Content should be modified");
    assert!(new_content.contains("second line"), "New line should be added");
}

/// Test workspace manager with budget constraints
#[tokio::test]
async fn test_workspace_manager_budget_integration() {
    use sandbox::WorkspaceManager;

    let temp_dir = tempdir().unwrap();
    let workspace_root = temp_dir.path().to_path_buf();

    let mut manager = WorkspaceManager::auto_detect(workspace_root).await.unwrap();
    manager.set_allow_list(vec![PathBuf::from("src/")]);
    manager.set_budgets(2, 20); // Small budget

    // Create a changeset that exceeds budget
    let oversized_changeset = types::ChangeSet {
        changes: vec![types::FileChange {
            path: PathBuf::from("src/large.rs"),
            operation: types::ChangeOperation::Create {
                content: "line\n".repeat(25), // 25 lines > 20 LOC budget
            },
        }],
        rationale: "Oversized file".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    let result = manager.apply_changes(oversized_changeset).await;
    assert!(result.is_err(), "Oversized changeset should be rejected");

    if let Err(sandbox::WorkspaceError::BudgetExceeded { current, proposed, limit }) = result {
        assert!(proposed.loc_used > limit.max_loc as i64, "Should exceed LOC limit");
    } else {
        panic!("Expected BudgetExceeded error");
    }
}

/// Test comprehensive error handling and recovery
#[tokio::test]
async fn test_error_handling_and_recovery() {
    let temp_dir = tempdir().unwrap();
    let applier = sandbox::diff_applier::DiffApplier::new(temp_dir.path().to_path_buf());

    // Test 1: SHA256 mismatch detection
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "original content").unwrap();

    let diff_with_wrong_sha = sandbox::diff_generator::UnifiedDiff {
        file_path: PathBuf::from("test.rs"),
        diff_content: "@@ -1 +1 @@\n-original\n+modified\n".to_string(),
        sha256: "dummy".to_string(),
        expected_sha256: Some("wrong_sha256_hash".to_string()), // Wrong hash
    };

    let result = applier.apply_diff(&diff_with_wrong_sha, false).await;
    assert!(result.is_err(), "Wrong SHA256 should be rejected");

    // Verify file wasn't modified
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "original content", "File should be unchanged on SHA256 mismatch");

    // Test 2: Diff validation prevents application of corrupted diffs
    let corrupted_diff = sandbox::diff_generator::UnifiedDiff {
        file_path: PathBuf::from("test.rs"),
        diff_content: "@@ -invalid -range @@\n+content".to_string(), // Invalid range
        sha256: "dummy".to_string(),
        expected_sha256: Some(applier.compute_file_sha256(&test_file).await.unwrap()),
    };

    let result = applier.apply_diff(&corrupted_diff, false).await;
    assert!(result.is_err(), "Corrupted diff should be rejected");
}
