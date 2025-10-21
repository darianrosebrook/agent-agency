use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationCode {
    OutOfScope,
    BudgetExceeded,
    MissingTests,
    NonDeterministic,
    DisallowedTool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub code: ViolationCode,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceSnapshot {
    pub within_scope: bool,
    pub within_budget: bool,
    pub tests_added: bool,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverRef {
    pub id: String,
    pub reason: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub task_id: String,
    pub snapshot: ComplianceSnapshot,
    pub violations: Vec<Violation>,
    pub waivers: Vec<WaiverRef>,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    pub risk_tier: u8,
    pub scope_in: Vec<String>,
    pub change_budget_max_files: u32,
    pub change_budget_max_loc: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Strict,
    Auto,
    DryRun,
}

pub struct TaskDescriptor {
    pub task_id: String,
    pub scope_in: Vec<String>,
    pub risk_tier: u8,
    pub execution_mode: ExecutionMode,
    pub acceptance: Option<Vec<String>>, // acceptance criteria text
    pub metadata: Option<std::collections::BTreeMap<String, String>>, // arbitrary metadata
}

#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("invalid spec: {0}")]
    InvalidSpec(String),
}

#[async_trait::async_trait]
pub trait CawsRuntimeValidator: Send + Sync {
    async fn validate(
        &self,
        spec: &WorkingSpec,
        desc: &TaskDescriptor,
        diff_stats: &DiffStats,
        patches: &[String],
        language_hints: &[String],
        tests_added: bool,
        deterministic: bool,
        waivers: Vec<WaiverRef>,
    ) -> Result<ValidationResult, ValidatorError>;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: u32,
    pub lines_changed: u32,
    pub touched_paths: Vec<String>,
}

pub struct DefaultValidator;
impl DefaultValidator {
    fn mde(&self) -> impl MinimalDiffEvaluator {
        NoopMde
    }
}

// Minimal Diff Evaluator (stub interface)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MdeFinding {
    pub loc_added: u32,
    pub loc_removed: u32,
    pub ast_change_units: u32,
    pub risky_patterns: Vec<String>,
    pub suggested_split: bool,
}

#[async_trait::async_trait]
pub trait MinimalDiffEvaluator: Send + Sync {
    async fn analyze(
        &self,
        patches: &[String],
        language_hints: &[String],
    ) -> Result<MdeFinding, ValidatorError>;
}

pub struct NoopMde;

#[async_trait::async_trait]
impl MinimalDiffEvaluator for NoopMde {
    async fn analyze(
        &self,
        _patches: &[String],
        _language_hints: &[String],
    ) -> Result<MdeFinding, ValidatorError> {
        Ok(MdeFinding::default())
    }
}

#[async_trait::async_trait]
impl CawsRuntimeValidator for DefaultValidator {
    async fn validate(
        &self,
        spec: &WorkingSpec,
        desc: &TaskDescriptor,
        diff_stats: &DiffStats,
        patches: &[String],
        language_hints: &[String],
        tests_added: bool,
        deterministic: bool,
        waivers: Vec<WaiverRef>,
    ) -> Result<ValidationResult, ValidatorError> {
        if spec.risk_tier < 1 || spec.risk_tier > 3 {
            return Err(ValidatorError::InvalidSpec("risk_tier".into()));
        }
        let within_budget = diff_stats.files_changed <= spec.change_budget_max_files
            && diff_stats.lines_changed <= spec.change_budget_max_loc;
        let within_scope = diff_stats
            .touched_paths
            .iter()
            .all(|p| desc.scope_in.iter().any(|s| p.starts_with(s)));
        let snapshot = ComplianceSnapshot {
            within_scope,
            within_budget,
            tests_added,
            deterministic,
        };

        let mut violations = Vec::new();
        // Invoke Minimal Diff Evaluator (stub) for future AST-aware checks
        let mde = self
            .mde()
            .analyze(patches, language_hints)
            .await
            .unwrap_or_default();
        // If suggested_split or excessive ast_change_units, hint remediation via budget violation context
        if mde.suggested_split || mde.ast_change_units > 1000 {
            violations.push(Violation {
                code: ViolationCode::BudgetExceeded,
                message: "Large or complex diff detected".into(),
                remediation: Some("Split changes and reduce AST-level churn".into()),
            });
        }
        if !within_scope {
            violations.push(Violation {
                code: ViolationCode::OutOfScope,
                message: "Touched file outside scope".into(),
                remediation: Some("Restrict changes to scope.in or update working spec".into()),
            });
        }
        if !within_budget {
            violations.push(Violation {
                code: ViolationCode::BudgetExceeded,
                message: "Change budget exceeded".into(),
                remediation: Some("Split PR or request budget waiver".into()),
            });
        }
        if !tests_added {
            violations.push(Violation {
                code: ViolationCode::MissingTests,
                message: "No tests added".into(),
                remediation: Some("Add failing test first per CAWS".into()),
            });
        }
        if !deterministic {
            violations.push(Violation {
                code: ViolationCode::NonDeterministic,
                message: "Non-deterministic code detected".into(),
                remediation: Some("Inject time/uuid/random".into()),
            });
        }

        Ok(ValidationResult {
            task_id: desc.task_id.clone(),
            snapshot,
            violations,
            waivers,
            validated_at: Utc::now(),
        })
    }
}
