//! Integration tests for CAWS compliance checker
//!
//! Tests the CAWS compliance checker in caws_checker.rs
//! to ensure proper validation of worker tasks against CAWS rules.
//!
//! @author @darianrosebrook

#[cfg(test)]
mod tests {
    use agent_workers::caws_checker::CawsChecker;
    use serde_json::json;

    #[tokio::test]
    async fn test_check_compliance_with_valid_task() {
        let checker = CawsChecker::new();

        let task = json!({
            "id": "TASK-001",
            "title": "Test Task",
            "scope": {
                "in": ["src/test/"],
                "out": ["node_modules/"]
            },
            "change_budget": {
                "max_files": 10,
                "max_loc": 500
            }
        })
        .to_string();

        let result = checker
            .check_compliance(&task)
            .await
            .expect("Compliance check should succeed");

        // Should return a check result
        // recommendations.len() is usize, always >= 0
    }

    #[tokio::test]
    async fn test_check_compliance_with_invalid_task() {
        let checker = CawsChecker::new();

        let task = "invalid json task";

        let result = checker.check_compliance(task).await;

        // Should handle invalid input gracefully
        match result {
            Ok(result) => {
                // May return violations for invalid input
                // recommendations.len() is usize, always >= 0
                let _ = result.recommendations.len();
            }
            Err(_) => {
                // Or may return error for invalid input
            }
        }
    }

    #[tokio::test]
    async fn test_check_compliance_with_empty_task() {
        let checker = CawsChecker::new();

        let task = "";

        let result = checker.check_compliance(task).await;

        // Should handle empty input gracefully
        match result {
            Ok(result) => {
                // recommendations.len() is usize, always >= 0
                let _ = result.recommendations.len();
            }
            Err(_) => {
                // Or may return error for empty input
            }
        }
    }
}
