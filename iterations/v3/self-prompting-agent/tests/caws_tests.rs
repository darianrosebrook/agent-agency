//! Tests for CAWS integration functionality

use self_prompting_agent::caws::*;
use self_prompting_agent::types::SelfPromptingAgentError;

#[tokio::test]
async fn test_caws_integration_creation_with_path() {
    let path = Some("/path/to/working/spec.yaml".to_string());
    let integration = CawsIntegration::new(path.clone());

    // Test that integration is created - we can't access the private field directly
    // but we can test the functionality
    let result = integration.validate_task("test task").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_caws_integration_creation_without_path() {
    let integration = CawsIntegration::new(None);

    let result = integration.validate_task("test task").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_validate_task_success() {
    let integration = CawsIntegration::new(None);

    let result = integration.validate_task("Implement user authentication").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_validate_task_empty_description() {
    let integration = CawsIntegration::new(None);

    let result = integration.validate_task("").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        SelfPromptingAgentError::Validation(msg) => {
            assert_eq!(msg, "Task description cannot be empty");
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_validate_task_whitespace_only() {
    let integration = CawsIntegration::new(None);

    let result = integration.validate_task("   \n\t   ").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        SelfPromptingAgentError::Validation(msg) => {
            assert_eq!(msg, "Task description cannot be empty");
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_check_quality_gates() {
    let integration = CawsIntegration::new(None);

    let result = integration.check_quality_gates().await;
    assert!(result.is_ok());

    let gates = result.unwrap();
    assert_eq!(gates.len(), 3);
    assert!(gates.contains(&"Code compiles successfully".to_string()));
    assert!(gates.contains(&"Tests pass".to_string()));
    assert!(gates.contains(&"Documentation updated".to_string()));
}

#[tokio::test]
async fn test_record_provenance() {
    let integration = CawsIntegration::new(None);

    let result = integration.record_provenance("Code review completed").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_record_provenance_various_operations() {
    let integration = CawsIntegration::new(None);

    let operations = vec![
        "Code generation",
        "Test execution",
        "Documentation update",
        "Code review",
    ];

    for operation in operations {
        let result = integration.record_provenance(operation).await;
        assert!(result.is_ok(), "Failed to record provenance for: {}", operation);
    }
}

#[tokio::test]
async fn test_working_spec_validator_creation() {
    let validator = WorkingSpecValidator::new();
    // Test that validator is created successfully
    let result = validator.validate_spec("{}").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_spec_empty() {
    let validator = WorkingSpecValidator::new();

    let result = validator.validate_spec("").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_spec_json() {
    let validator = WorkingSpecValidator::new();

    let json_spec = r#"{
        "version": "1.0",
        "description": "Test working spec",
        "quality_gates": ["compile", "test", "docs"]
    }"#;

    let result = validator.validate_spec(json_spec).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_spec_yaml_like() {
    let validator = WorkingSpecValidator::new();

    let yaml_spec = r#"
version: "1.0"
description: Test working spec
quality_gates:
  - compile
  - test
  - docs
"#;

    let result = validator.validate_spec(yaml_spec).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_spec_complex() {
    let validator = WorkingSpecValidator::new();

    let complex_spec = r#"{
        "version": "2.0",
        "metadata": {
            "author": "Test Author",
            "created": "2024-01-01"
        },
        "requirements": [
            {
                "id": "REQ001",
                "description": "Must compile without errors",
                "type": "functional"
            }
        ],
        "quality_gates": [
            {
                "name": "compilation",
                "description": "Code must compile successfully",
                "required": true
            },
            {
                "name": "testing",
                "description": "All tests must pass",
                "required": true
            }
        ],
        "provenance": {
            "track_changes": true,
            "record_operations": true
        }
    }"#;

    let result = validator.validate_spec(complex_spec).await;
    assert!(result.is_ok());
}

// Debug implementations are not available for these structs
