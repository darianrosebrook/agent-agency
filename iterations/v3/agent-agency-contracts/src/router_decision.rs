use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ContractError, ContractKind, ValidationIssue};
use crate::schema::ROUTER_DECISION_SCHEMA;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouterDecisionContract {
    pub task_id: String,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Assignment {
    pub worker_type: WorkerType,
    pub model: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerType {
    Generalist,
    Specialist,
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
            Err(ContractError::validation(ContractKind::RouterDecision, issues))
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
}
