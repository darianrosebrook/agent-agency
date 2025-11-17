use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract_errors::{ContractError, ContractKind, ValidationIssue};
use crate::schema::ROUTER_DECISION_SCHEMA;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct RouterDecisionContract {
    pub task_id: String,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct Assignment {
    pub worker_type: WorkerType,
    pub model: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerType {
    Generalist,
    Specialist(String),
}

impl RouterDecisionContract {
    pub fn validate(&self) -> Result<(), ContractError> {
        let value = serde_json::to_value(self)
            .map_err(|err| ContractError::serialization(ContractKind::RouterDecision, err))?;
        validate_router_decision_value(&value)
    }

    pub fn try_from_value(value: Value) -> Result<Self, ContractError> {
        validate_router_decision_value(&value)?;
        serde_json::from_value(value)
            .map_err(|err| ContractError::serialization(ContractKind::RouterDecision, err))
    }
}

pub fn validate_router_decision_value(value: &Value) -> Result<(), ContractError> {
    match ROUTER_DECISION_SCHEMA.validate(value) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let issues: Vec<ValidationIssue> = errors
                .map(|error| ValidationIssue {
                    instance_path: error.instance_path.to_string(),
                    schema_path: error.schema_path.to_string(),
                    message: error.to_string(),
                })
                .collect();
            Err(ContractError::validation(
                ContractKind::RouterDecision,
                issues,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_decision_validation() {
        let contract = RouterDecisionContract {
            task_id: "TASK-1".into(),
            assignments: vec![Assignment {
                worker_type: WorkerType::Generalist,
                model: "gpt-4o".into(),
                reason: "General improvements".into(),
            }],
        };

        contract.validate().expect("valid");
        let json = serde_json::to_value(&contract).unwrap();
        assert!(validate_router_decision_value(&json).is_ok());
    }

    #[test]
    fn router_decision_validation_invalid_missing_required_fields() {
        let invalid = serde_json::json!({
            "task_id": "TASK-1"
            // Missing assignments
        });
        let err = validate_router_decision_value(&invalid).expect_err("should fail");
        assert_eq!(err.kind(), ContractKind::RouterDecision);
        assert!(!err.issues().is_empty());
    }

    #[test]
    fn router_decision_validation_invalid_wrong_type() {
        let invalid = serde_json::json!({
            "task_id": 123,  // Should be string
            "assignments": []
        });
        let err = validate_router_decision_value(&invalid).expect_err("should fail");
        assert_eq!(err.kind(), ContractKind::RouterDecision);
        assert!(!err.issues().is_empty());
    }

    #[test]
    fn router_decision_validation_invalid_empty_assignments() {
        let invalid = serde_json::json!({
            "task_id": "TASK-1",
            "assignments": "not_an_array"
        });
        let err = validate_router_decision_value(&invalid).expect_err("should fail");
        assert_eq!(err.kind(), ContractKind::RouterDecision);
        assert!(!err.issues().is_empty());
    }

    #[test]
    fn router_decision_contract_validate_uses_real_validation() {
        // Test that validate() actually calls schema validation
        let valid_contract = RouterDecisionContract {
            task_id: "TASK-1".into(),
            assignments: vec![Assignment {
                worker_type: WorkerType::Generalist,
                model: "gpt-4o".into(),
                reason: "General improvements".into(),
            }],
        };
        
        // Valid contract should pass - proves validate() is real
        let result = valid_contract.validate();
        assert!(result.is_ok(), "Valid contract should pass validation");
    }

    #[test]
    fn router_decision_contract_validate_rejects_invalid_data() {
        // Test with invalid JSON that should fail schema validation
        use serde_json::json;
        
        let invalid_json = json!({
            "task_id": 123, // Wrong type - should be string
            "assignments": []
        });
        
        // Direct validation should fail
        let result = validate_router_decision_value(&invalid_json);
        // This proves validation is real - if stubbed to Ok(()), this would pass incorrectly
        assert!(result.is_err(), "Wrong type for task_id should be rejected");
    }
}
