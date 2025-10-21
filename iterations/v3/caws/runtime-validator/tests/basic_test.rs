//! Basic tests for CAWS runtime validator

use caws_runtime_validator::{
    CawsValidator, CawsPolicy,
    validator::{ValidationContext, DiffStats},
    policy::PolicyValidator,
};

#[tokio::test]
async fn test_basic_validation() {
    let policy = CawsPolicy::default();
    let validator = CawsValidator::new(policy);

    let context = ValidationContext {
        task_id: "test-task".to_string(),
        risk_tier: "medium".to_string(),
        working_spec: serde_json::json!({"test": "spec"}),
        diff_stats: DiffStats {
            files_changed: 5,
            lines_added: 100,
            lines_deleted: 20,
            files_modified: vec!["src/main.rs".to_string()],
        },
        test_results: None,
        security_scan: None,
    };

    let result = validator.validate(context).await;

    assert_eq!(result.task_id, "test-task");
    assert!(result.passed); // Should pass with default policy
    assert!(result.compliance_score > 0.0);
}

#[test]
fn test_policy_validation() {
    let mut policy = CawsPolicy::default();
    policy.risk_tiers.get_mut("low").unwrap().level = 0; // Invalid level

    let result = PolicyValidator::validate_policy(&policy);
    assert!(result.is_err());
}
