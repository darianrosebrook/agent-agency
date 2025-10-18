use std::fmt;

/// Contract artifact categories handled by the interoperability layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContractKind {
    WorkerOutput,
    JudgeVerdict,
    FinalVerdict,
    RouterDecision,
}

impl fmt::Display for ContractKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ContractKind::WorkerOutput => "worker-output",
            ContractKind::JudgeVerdict => "judge-verdict",
            ContractKind::FinalVerdict => "final-verdict",
            ContractKind::RouterDecision => "router-decision",
        };
        write!(f, "{}", label)
    }
}

/// Detailed validation failure emitted by the contract validator.
#[derive(Debug, Clone, PartialEq, Eq)]
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
