use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescriptor {
    pub task_id: String,
    pub scope_in: Vec<String>,
    pub risk_tier: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidatorError {
    #[error("invalid spec: {0}")] InvalidSpec(String),
}

#[async_trait::async_trait]
pub trait CawsRuntimeValidator: Send + Sync {
    async fn validate(&self, spec: &WorkingSpec, desc: &TaskDescriptor, diff_stats: &DiffStats, tests_added: bool, deterministic: bool, waivers: Vec<WaiverRef>) -> Result<ValidationResult, ValidatorError>;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: u32,
    pub lines_changed: u32,
    pub touched_paths: Vec<String>,
}

pub struct DefaultValidator;

#[async_trait::async_trait]
impl CawsRuntimeValidator for DefaultValidator {
    async fn validate(&self, spec: &WorkingSpec, desc: &TaskDescriptor, diff_stats: &DiffStats, tests_added: bool, deterministic: bool, waivers: Vec<WaiverRef>) -> Result<ValidationResult, ValidatorError> {
        if spec.risk_tier < 1 || spec.risk_tier > 3 { return Err(ValidatorError::InvalidSpec("risk_tier".into())); }
        let within_budget = diff_stats.files_changed <= spec.change_budget_max_files && diff_stats.lines_changed <= spec.change_budget_max_loc;
        let within_scope = diff_stats.touched_paths.iter().all(|p| desc.scope_in.iter().any(|s| p.starts_with(s)));
        let snapshot = ComplianceSnapshot { within_scope, within_budget, tests_added, deterministic };

        let mut violations = Vec::new();
        if !within_scope { violations.push(Violation { code: ViolationCode::OutOfScope, message: "Touched file outside scope".into(), remediation: Some("Restrict changes to scope.in or update working spec".into())}); }
        if !within_budget { violations.push(Violation { code: ViolationCode::BudgetExceeded, message: "Change budget exceeded".into(), remediation: Some("Split PR or request budget waiver".into())}); }
        if !tests_added { violations.push(Violation { code: ViolationCode::MissingTests, message: "No tests added".into(), remediation: Some("Add failing test first per CAWS".into())}); }
        if !deterministic { violations.push(Violation { code: ViolationCode::NonDeterministic, message: "Non-deterministic code detected".into(), remediation: Some("Inject time/uuid/random".into())}); }

        Ok(ValidationResult { task_id: desc.task_id.clone(), snapshot, violations, waivers, validated_at: Utc::now() })
    }
}

