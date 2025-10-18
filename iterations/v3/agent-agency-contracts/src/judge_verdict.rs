use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ContractError, ContractKind, ValidationIssue};
use crate::schema::JUDGE_VERDICT_SCHEMA;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JudgeVerdictContract {
    pub judge_id: String,
    pub version: String,
    pub verdict: JudgeDecision,
    pub reasons: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JudgeDecision {
    Pass,
    Fail,
    Uncertain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceItem {
    #[serde(rename = "type")]
    pub kind: EvidenceType,
    pub r#ref: String,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Research,
    StaticCheck,
    Test,
}

impl JudgeVerdictContract {
    pub fn validate(&self) -> Result<(), ContractError> {
        let value = serde_json::to_value(self)
            .map_err(|err| ContractError::serialization(ContractKind::JudgeVerdict, err))?;
        validate_judge_verdict_value(&value)
    }

    pub fn try_from_value(value: Value) -> Result<Self, ContractError> {
        validate_judge_verdict_value(&value)?;
        serde_json::from_value(value)
            .map_err(|err| ContractError::serialization(ContractKind::JudgeVerdict, err))
    }
}

pub fn validate_judge_verdict_value(value: &Value) -> Result<(), ContractError> {
    match JUDGE_VERDICT_SCHEMA.validate(value) {
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
                ContractKind::JudgeVerdict,
                issues,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn judge_verdict_validation() {
        let verdict = JudgeVerdictContract {
            judge_id: "tech".into(),
            version: "1.0".into(),
            verdict: JudgeDecision::Pass,
            reasons: vec!["All checks passed".into()],
            evidence: vec![EvidenceItem {
                kind: EvidenceType::Test,
                r#ref: "tests::unit".into(),
                summary: Some("Unit tests green".into()),
            }],
        };

        verdict.validate().expect("valid");
        let json = serde_json::to_value(&verdict).unwrap();
        assert!(validate_judge_verdict_value(&json).is_ok());
    }

    #[test]
    fn judge_verdict_invalid() {
        let bad = serde_json::json!({"judge_id": "tech"});
        let err = validate_judge_verdict_value(&bad).expect_err("should fail");
        assert_eq!(err.kind(), ContractKind::JudgeVerdict);
        assert!(!err.issues().is_empty());
    }
}
