use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ContractError, ContractKind, ValidationIssue};
use crate::schema::FINAL_VERDICT_SCHEMA;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinalVerdictContract {
    pub decision: FinalDecision,
    pub votes: Vec<VoteEntry>,
    pub dissent: String,
    #[serde(default)]
    pub remediation: Vec<String>,
    #[serde(default)]
    pub constitutional_refs: Vec<String>,
    pub verification_summary: VerificationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalDecision {
    Accept,
    Reject,
    Modify,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoteEntry {
    pub judge_id: String,
    pub weight: f32,
    pub verdict: VoteVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoteVerdict {
    Pass,
    Fail,
    Uncertain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationSummary {
    pub claims_total: u32,
    pub claims_verified: u32,
    pub coverage_pct: f32,
}

impl FinalVerdictContract {
    pub fn validate(&self) -> Result<(), ContractError> {
        let value = serde_json::to_value(self)
            .map_err(|err| ContractError::serialization(ContractKind::FinalVerdict, err))?;
        validate_final_verdict_value(&value)
    }

    pub fn try_from_value(value: Value) -> Result<Self, ContractError> {
        validate_final_verdict_value(&value)?;
        serde_json::from_value(value)
            .map_err(|err| ContractError::serialization(ContractKind::FinalVerdict, err))
    }
}

pub fn validate_final_verdict_value(value: &Value) -> Result<(), ContractError> {
    match FINAL_VERDICT_SCHEMA.validate(value) {
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
                ContractKind::FinalVerdict,
                issues,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_verdict_validation() {
        let contract = FinalVerdictContract {
            decision: FinalDecision::Accept,
            votes: vec![VoteEntry {
                judge_id: "tech".into(),
                weight: 0.4,
                verdict: VoteVerdict::Pass,
            }],
            dissent: String::new(),
            remediation: vec![],
            constitutional_refs: vec![],
            verification_summary: VerificationSummary {
                claims_total: 4,
                claims_verified: 4,
                coverage_pct: 1.0,
            },
        };

        contract.validate().expect("valid");
        let json = serde_json::to_value(&contract).unwrap();
        assert!(validate_final_verdict_value(&json).is_ok());
    }
}
