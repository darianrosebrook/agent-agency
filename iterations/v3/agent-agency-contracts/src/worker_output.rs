use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ContractError, ContractKind, ValidationIssue};
use crate::schema::WORKER_OUTPUT_SCHEMA;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerOutputContract {
    pub metadata: WorkerMetadata,
    pub artifacts: WorkerArtifacts,
    pub rationale: String,
    #[serde(rename = "self_assessment")]
    pub self_assessment: WorkerSelfAssessment,
    #[serde(default)]
    pub waivers: Vec<WaiverContract>,
    #[serde(default)]
    pub claims: Vec<ClaimContract>,
    #[serde(rename = "evidence_refs", default)]
    pub evidence_refs: Vec<EvidenceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerMetadata {
    pub task_id: String,
    pub risk_tier: u8,
    pub seeds: WorkerSeeds,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerSeeds {
    pub time_seed: String,
    pub uuid_seed: String,
    pub random_seed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerArtifacts {
    pub patches: Vec<PatchArtifact>,
    #[serde(default)]
    pub commands: Vec<CommandArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchArtifact {
    pub path: String,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandArtifact {
    pub cmd: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerSelfAssessment {
    pub caws_checklist: CawsChecklist,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CawsChecklist {
    pub within_scope: bool,
    pub within_budget: bool,
    pub tests_added: bool,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WaiverContract {
    pub id: String,
    pub reason: String,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimContract {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceReference {
    pub claim_id: String,
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(default)]
    pub note: Option<String>,
}

impl WorkerOutputContract {
    pub fn validate(&self) -> Result<(), ContractError> {
        let value = serde_json::to_value(self)
            .map_err(|err| ContractError::serialization(ContractKind::WorkerOutput, err))?;
        validate_worker_output_value(&value)
    }

    pub fn try_from_value(value: Value) -> Result<Self, ContractError> {
        validate_worker_output_value(&value)?;
        serde_json::from_value(value)
            .map_err(|err| ContractError::serialization(ContractKind::WorkerOutput, err))
    }
}

pub fn validate_worker_output_value(value: &Value) -> Result<(), ContractError> {
    match WORKER_OUTPUT_SCHEMA.validate(value) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let issues: Vec<ValidationIssue> = errors
                .map(|error| ValidationIssue {
                    instance_path: error.instance_path.to_string(),
                    schema_path: error.schema_path.to_string(),
                    message: error.to_string(),
                })
                .collect();
            Err(ContractError::validation(ContractKind::WorkerOutput, issues))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_output_round_trip() {
        let contract = WorkerOutputContract {
            metadata: WorkerMetadata {
                task_id: "TASK-123".into(),
                risk_tier: 2,
                seeds: WorkerSeeds {
                    time_seed: "2025-01-01T00:00:00Z".into(),
                    uuid_seed: "00000000-0000-0000-0000-000000000000".into(),
                    random_seed: 42,
                },
            },
            artifacts: WorkerArtifacts {
                patches: vec![PatchArtifact {
                    path: "src/lib.rs".into(),
                    diff: "---".into(),
                }],
                commands: vec![CommandArtifact {
                    cmd: "cargo test".into(),
                    dry_run: false,
                }],
            },
            rationale: "Implemented feature".into(),
            self_assessment: WorkerSelfAssessment {
                caws_checklist: CawsChecklist {
                    within_scope: true,
                    within_budget: true,
                    tests_added: true,
                    deterministic: true,
                },
                notes: "All good".into(),
            },
            waivers: vec![WaiverContract {
                id: "W-1".into(),
                reason: "Experimental".into(),
                scope: Some("tests".into()),
            }],
            claims: vec![ClaimContract {
                id: "C-1".into(),
                title: "Claims coverage".into(),
                summary: Some("Ensured coverage".into()),
            }],
            evidence_refs: vec![EvidenceReference {
                claim_id: "C-1".into(),
                reference: "evidence://123".into(),
                note: Some("Validated".into()),
            }],
        };

        contract.validate().expect("validation");
        let json = serde_json::to_value(&contract).unwrap();
        assert!(validate_worker_output_value(&json).is_ok());
        let round_trip = WorkerOutputContract::try_from_value(json).unwrap();
        assert_eq!(round_trip, contract);
    }

    #[test]
    fn worker_output_validation_errors() {
        let bad = serde_json::json!({
            "metadata": {"task_id": "TASK-1", "risk_tier": 4},
            "artifacts": {"patches": []},
            "rationale": "",
            "self_assessment": {"caws_checklist": {"within_scope": true, "within_budget": true, "tests_added": true, "deterministic": true}, "notes": ""}
        });

        let err = validate_worker_output_value(&bad).expect_err("should fail");
        assert_eq!(err.kind(), ContractKind::WorkerOutput);
        assert!(!err.issues().is_empty());
    }
}
