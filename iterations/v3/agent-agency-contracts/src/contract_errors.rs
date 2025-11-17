use schemars::JsonSchema;
use std::fmt;

/// Contract artifact categories handled by the interoperability layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum ContractKind {
    TaskRequest,
    TaskResponse,
    WorkingSpec,
    ExecutionArtifacts,
    QualityReport,
    RefinementDecision,
    WorkerOutput,
    JudgeVerdict,
    FinalVerdict,
    RouterDecision,
}

impl fmt::Display for ContractKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ContractKind::TaskRequest => "task-request",
            ContractKind::TaskResponse => "task-response",
            ContractKind::WorkingSpec => "working-spec",
            ContractKind::ExecutionArtifacts => "execution-artifacts",
            ContractKind::QualityReport => "quality-report",
            ContractKind::RefinementDecision => "refinement-decision",
            ContractKind::WorkerOutput => "worker-output",
            ContractKind::JudgeVerdict => "judge-verdict",
            ContractKind::FinalVerdict => "final-verdict",
            ContractKind::RouterDecision => "router-decision",
        };
        write!(f, "{}", label)
    }
}

/// Detailed validation failure emitted by the contract validator.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct ValidationIssue {
    pub instance_path: String,
    pub schema_path: String,
    pub message: String,
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (instance: {}, schema: {})",
            self.message, self.instance_path, self.schema_path
        )
    }
}

/// Errors raised by the interoperability contract layer.
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("{kind} contract validation failed")]
    Validation {
        kind: ContractKind,
        issues: Vec<ValidationIssue>,
    },
    #[error("{kind} contract (de)serialization error: {source}")]
    Serialization {
        kind: ContractKind,
        #[source]
        source: serde_json::Error,
    },
}

impl ContractError {
    pub fn validation(kind: ContractKind, issues: Vec<ValidationIssue>) -> Self {
        Self::Validation { kind, issues }
    }

    pub fn serialization(kind: ContractKind, source: serde_json::Error) -> Self {
        Self::Serialization { kind, source }
    }

    pub fn kind(&self) -> ContractKind {
        match self {
            ContractError::Validation { kind, .. } => *kind,
            ContractError::Serialization { kind, .. } => *kind,
        }
    }

    pub fn issues(&self) -> &[ValidationIssue] {
        match self {
            ContractError::Validation { issues, .. } => issues,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_kind_display_all_variants() {
        assert_eq!(ContractKind::TaskRequest.to_string(), "task-request");
        assert_eq!(ContractKind::TaskResponse.to_string(), "task-response");
        assert_eq!(ContractKind::WorkingSpec.to_string(), "working-spec");
        assert_eq!(
            ContractKind::ExecutionArtifacts.to_string(),
            "execution-artifacts"
        );
        assert_eq!(ContractKind::QualityReport.to_string(), "quality-report");
        assert_eq!(
            ContractKind::RefinementDecision.to_string(),
            "refinement-decision"
        );
        assert_eq!(ContractKind::WorkerOutput.to_string(), "worker-output");
        assert_eq!(ContractKind::JudgeVerdict.to_string(), "judge-verdict");
        assert_eq!(ContractKind::FinalVerdict.to_string(), "final-verdict");
        assert_eq!(ContractKind::RouterDecision.to_string(), "router-decision");
    }

    #[test]
    fn validation_issue_display_format() {
        let issue = ValidationIssue {
            instance_path: "/path/to/field".to_string(),
            schema_path: "/properties/field".to_string(),
            message: "Invalid type".to_string(),
        };

        let formatted = issue.to_string();
        assert!(formatted.contains("Invalid type"));
        assert!(formatted.contains("/path/to/field"));
        assert!(formatted.contains("/properties/field"));
        assert!(formatted.contains("instance:"));
        assert!(formatted.contains("schema:"));
    }

    #[test]
    fn validation_issue_display_empty_fields() {
        let issue = ValidationIssue {
            instance_path: String::new(),
            schema_path: String::new(),
            message: String::new(),
        };

        let formatted = issue.to_string();
        assert_eq!(formatted, " (instance: , schema: )");
    }

    #[test]
    fn validation_issue_display_special_characters() {
        let issue = ValidationIssue {
            instance_path: "/path/with/special-chars".to_string(),
            schema_path: "/properties/field[0]".to_string(),
            message: "Error: \"quoted\" message".to_string(),
        };

        let formatted = issue.to_string();
        assert!(formatted.contains("Error: \"quoted\" message"));
        assert!(formatted.contains("/path/with/special-chars"));
        assert!(formatted.contains("/properties/field[0]"));
    }
}
