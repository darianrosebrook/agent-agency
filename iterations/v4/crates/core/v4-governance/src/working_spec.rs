//! CAWS Working Spec Types
//!
//! Rust types for CAWS working specifications, matching the JSON schema
//! at `.caws/schemas/working-spec.schema.json`. Provides deserialization
//! from YAML, validation, budget checking, and scope boundary enforcement.

use std::collections::HashMap;
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─── Core Types ───────────────────────────────────────────────────────────────

/// A CAWS working specification — the blueprint for a governed change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    /// Unique identifier (pattern: `^(PROJ|FEAT|FIX|ARCH)-\d{4}$`)
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Risk tier (1=Critical, 2=Standard, 3=LowRisk)
    pub risk_tier: u8,

    /// Change mode
    pub mode: SpecMode,

    /// Change budget limits
    #[serde(default)]
    pub change_budget: Option<ChangeBudget>,

    /// Impact analysis
    #[serde(default)]
    pub blast_radius: Option<BlastRadius>,

    /// Rollback time target
    #[serde(default)]
    pub operational_rollback_slo: Option<String>,

    /// File scope boundaries
    #[serde(default)]
    pub scope: Option<Scope>,

    /// Waiver IDs applying to this spec
    #[serde(default)]
    pub waiver_ids: Vec<String>,

    /// Rules that must never be violated
    #[serde(default)]
    pub invariants: Vec<String>,

    /// Given/When/Then acceptance criteria
    #[serde(default)]
    pub acceptance: Vec<AcceptanceCriterion>,

    /// Non-functional requirements
    #[serde(default)]
    pub non_functional: Option<NonFunctional>,

    /// Observability configuration
    #[serde(default)]
    pub observability: Option<Observability>,

    /// Rollback steps
    #[serde(default)]
    pub rollback: Vec<String>,

    /// Contract definitions
    #[serde(default)]
    pub contracts: Vec<Contract>,

    /// Experimental mode flag
    #[serde(default)]
    pub experiment_mode: Option<bool>,

    /// Time limit for experimental features (hours)
    #[serde(default)]
    pub timeboxed_hours: Option<u32>,

    /// AI confidence assessment
    #[serde(default)]
    pub ai_assessment: Option<AiAssessment>,

    /// Human override for special cases
    #[serde(default)]
    pub human_override: Option<HumanOverride>,
}

/// Change mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecMode {
    Feature,
    Refactor,
    Fix,
    Doc,
    Chore,
}

/// Change budget — atomic size limits per PR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeBudget {
    /// Maximum files changeable
    pub max_files: u32,
    /// Maximum lines of code
    pub max_loc: u32,
}

/// Blast radius — impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    /// Modules affected by the change
    pub modules: Vec<String>,
    /// Whether data migration is required
    #[serde(default)]
    pub data_migration: bool,
}

/// Scope boundaries — which files may be touched
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Allowed file patterns (glob)
    #[serde(rename = "in")]
    pub include: Vec<String>,
    /// Denied file patterns (glob, takes precedence)
    #[serde(rename = "out")]
    pub exclude: Vec<String>,
}

/// Acceptance criterion in Given/When/Then format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// Criterion ID (e.g., "A1")
    pub id: String,
    /// Precondition
    pub given: String,
    /// Action
    pub when: String,
    /// Expected outcome
    pub then: String,
    /// Current status
    #[serde(default)]
    pub status: CriterionStatus,
}

/// Status of an acceptance criterion
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CriterionStatus {
    #[default]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    Completed,
    Failed,
}

/// Risk tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskTier {
    /// Tier 1: 90% coverage, 70% mutation, manual review required
    Critical,
    /// Tier 2: 80% coverage, 70% mutation, optional review
    Standard,
    /// Tier 3: 70% coverage, 30% mutation
    LowRisk,
}

impl RiskTier {
    /// Parse from the integer representation used in YAML
    pub fn from_level(level: u8) -> Option<Self> {
        match level {
            1 => Some(Self::Critical),
            2 => Some(Self::Standard),
            3 => Some(Self::LowRisk),
            _ => None,
        }
    }

    /// Coverage threshold for this tier
    pub fn coverage_threshold(&self) -> f64 {
        match self {
            Self::Critical => 0.90,
            Self::Standard => 0.80,
            Self::LowRisk => 0.70,
        }
    }

    /// Mutation score threshold for this tier
    pub fn mutation_threshold(&self) -> f64 {
        match self {
            Self::Critical => 0.70,
            Self::Standard => 0.70,
            Self::LowRisk => 0.30,
        }
    }

    /// Whether manual review is required
    pub fn requires_manual_review(&self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Whether contracts are required
    pub fn requires_contracts(&self) -> bool {
        matches!(self, Self::Critical | Self::Standard)
    }
}

/// Non-functional requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonFunctional {
    /// Accessibility requirements
    #[serde(default)]
    pub a11y: Vec<String>,
    /// Performance budgets
    #[serde(default)]
    pub perf: Option<HashMap<String, serde_json::Value>>,
    /// Security requirements
    #[serde(default)]
    pub security: Vec<String>,
    /// Reliability requirements
    #[serde(default)]
    pub reliability: Vec<String>,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observability {
    /// Log patterns
    #[serde(default)]
    pub logs: Vec<String>,
    /// Metric names
    #[serde(default)]
    pub metrics: Vec<String>,
    /// Trace span names
    #[serde(default)]
    pub traces: Vec<String>,
}

/// Contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Contract type (openapi, graphql, proto, pact, database, websocket)
    #[serde(rename = "type")]
    pub contract_type: String,
    /// Path to contract file
    #[serde(default)]
    pub path: Option<String>,
    /// API operations covered
    #[serde(default)]
    pub operations: Vec<String>,
    /// Database tables covered
    #[serde(default)]
    pub tables: Vec<String>,
    /// WebSocket events covered
    #[serde(default)]
    pub events: Vec<String>,
}

/// AI confidence assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAssessment {
    /// Confidence level (1-10)
    pub confidence_level: u8,
    /// Areas of uncertainty
    #[serde(default)]
    pub uncertainty_areas: Vec<String>,
    /// Whether pairing is recommended
    #[serde(default)]
    pub recommended_pairing: bool,
}

/// Human override for special cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanOverride {
    /// Who approved the override
    pub approved_by: String,
    /// Why it was overridden
    pub reason: String,
    /// Which requirements were waived
    #[serde(default)]
    pub waived_requirements: Vec<String>,
    /// When the override expires
    #[serde(default)]
    pub expiry_date: Option<String>,
}

// ─── Parsing ──────────────────────────────────────────────────────────────────

impl WorkingSpec {
    /// Load a working spec from a YAML file
    pub fn from_yaml_file(path: &Path) -> Result<Self, SpecError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SpecError::Io(e.to_string()))?;
        Self::from_yaml_str(&content)
    }

    /// Parse a working spec from a YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<Self, SpecError> {
        serde_yaml::from_str(yaml).map_err(|e| SpecError::Parse(e.to_string()))
    }

    /// Get the parsed risk tier
    pub fn risk_tier(&self) -> Option<RiskTier> {
        RiskTier::from_level(self.risk_tier)
    }
}

// ─── Validation ───────────────────────────────────────────────────────────────

/// Validation error for a working spec
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Field that failed validation
    pub field: String,
    /// Description of the issue
    pub message: String,
}

impl WorkingSpec {
    /// Validate the spec against CAWS rules. Returns a list of issues (empty = valid).
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // risk_tier must be 1, 2, or 3
        if RiskTier::from_level(self.risk_tier).is_none() {
            issues.push(ValidationIssue {
                field: "risk_tier".to_string(),
                message: format!("Invalid risk tier: {}. Must be 1, 2, or 3.", self.risk_tier),
            });
        }

        // scope.in must be non-empty if scope is present
        if let Some(scope) = &self.scope {
            if scope.include.is_empty() {
                issues.push(ValidationIssue {
                    field: "scope.in".to_string(),
                    message: "scope.in must contain at least one path pattern".to_string(),
                });
            }
        }

        // At least 1 invariant required
        if self.invariants.is_empty() {
            issues.push(ValidationIssue {
                field: "invariants".to_string(),
                message: "At least one invariant is required".to_string(),
            });
        }

        // At least 1 acceptance criterion required
        if self.acceptance.is_empty() {
            issues.push(ValidationIssue {
                field: "acceptance".to_string(),
                message: "At least one acceptance criterion is required".to_string(),
            });
        }

        // id must be non-empty
        if self.id.is_empty() {
            issues.push(ValidationIssue {
                field: "id".to_string(),
                message: "Spec id must not be empty".to_string(),
            });
        }

        // title length check
        if self.title.len() < 10 {
            issues.push(ValidationIssue {
                field: "title".to_string(),
                message: "Title must be at least 10 characters".to_string(),
            });
        }

        // Tier 1 and 2 require contracts
        if let Some(tier) = self.risk_tier() {
            if tier.requires_contracts() && self.contracts.is_empty() {
                issues.push(ValidationIssue {
                    field: "contracts".to_string(),
                    message: format!(
                        "Risk tier {} requires at least one contract definition",
                        self.risk_tier
                    ),
                });
            }
        }

        issues
    }

    /// Returns true if the spec passes validation
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

// ─── Budget Checker ───────────────────────────────────────────────────────────

/// Budget violation details
#[derive(Debug, Clone, Error)]
pub enum BudgetViolation {
    /// More files changed than allowed
    #[error("Files exceeded: {actual} changed, limit is {limit}")]
    FilesExceeded {
        limit: u32,
        actual: u32,
    },
    /// More LOC changed than allowed
    #[error("LOC exceeded: {actual} changed, limit is {limit}")]
    LocExceeded {
        limit: u32,
        actual: u32,
    },
    /// No budget defined in the spec
    #[error("No change budget defined in working spec")]
    NoBudget,
}

/// Checks whether a change set fits within a working spec's budget
pub struct BudgetChecker;

impl BudgetChecker {
    /// Check a change against the spec's budget
    pub fn check(
        spec: &WorkingSpec,
        files_changed: u32,
        loc_changed: u32,
    ) -> Result<(), BudgetViolation> {
        let budget = spec.change_budget.as_ref().ok_or(BudgetViolation::NoBudget)?;

        if files_changed > budget.max_files {
            return Err(BudgetViolation::FilesExceeded {
                limit: budget.max_files,
                actual: files_changed,
            });
        }

        if loc_changed > budget.max_loc {
            return Err(BudgetViolation::LocExceeded {
                limit: budget.max_loc,
                actual: loc_changed,
            });
        }

        Ok(())
    }

    /// Check budget with waiver support. Active, non-expired waivers for the
    /// `ChangeBudget` gate skip the budget check entirely.
    pub fn check_with_waivers<W>(
        spec: &WorkingSpec,
        files_changed: u32,
        loc_changed: u32,
        waivers: &[W],
    ) -> Result<(), BudgetViolation>
    where
        W: WaiverCheck,
    {
        // If any active budget waiver exists, skip the check
        let has_active_waiver = waivers
            .iter()
            .any(|w| w.is_budget_waiver() && w.is_active_now());

        if has_active_waiver {
            return Ok(());
        }

        Self::check(spec, files_changed, loc_changed)
    }
}

/// Trait for checking waiver status (allows decoupling from concrete Waiver type)
pub trait WaiverCheck {
    /// Whether this waiver applies to the change_budget gate
    fn is_budget_waiver(&self) -> bool;
    /// Whether this waiver is currently active (not expired, not revoked)
    fn is_active_now(&self) -> bool;
}

// ─── Scope Enforcer ───────────────────────────────────────────────────────────

/// Scope enforcement errors
#[derive(Debug, Clone, Error)]
pub enum ScopeError {
    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(String),
    #[error("No scope defined in working spec")]
    NoScope,
}

/// Enforces file scope boundaries from a working spec
pub struct ScopeEnforcer {
    include_set: GlobSet,
    exclude_set: GlobSet,
}

impl ScopeEnforcer {
    /// Build a scope enforcer from a working spec's scope section
    pub fn from_spec(spec: &WorkingSpec) -> Result<Self, ScopeError> {
        let scope = spec.scope.as_ref().ok_or(ScopeError::NoScope)?;
        Self::from_scope(scope)
    }

    /// Build from a Scope directly
    pub fn from_scope(scope: &Scope) -> Result<Self, ScopeError> {
        let include_set = build_globset(&scope.include)?;
        let exclude_set = build_globset(&scope.exclude)?;
        Ok(Self { include_set, exclude_set })
    }

    /// Check whether a path is in scope.
    ///
    /// Rules:
    /// - If path matches `scope.out`, return `false` (out takes precedence)
    /// - If path matches `scope.in`, return `true`
    /// - If path matches neither, return `false` (default deny)
    pub fn is_in_scope(&self, path: &str) -> bool {
        // Out takes precedence
        if self.exclude_set.is_match(path) {
            return false;
        }
        self.include_set.is_match(path)
    }
}

fn build_globset(patterns: &[String]) -> Result<GlobSet, ScopeError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        // Allow patterns with or without leading glob
        let glob = Glob::new(pattern)
            .or_else(|_| Glob::new(&format!("**/{pattern}")))
            .map_err(|e| ScopeError::InvalidPattern(format!("{pattern}: {e}")))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| ScopeError::InvalidPattern(e.to_string()))
}

// ─── Errors ───────────────────────────────────────────────────────────────────

/// Errors when loading or processing a working spec
#[derive(Debug, Error)]
pub enum SpecError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("YAML parse error: {0}")]
    Parse(String),
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_SPEC_YAML: &str = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 2
mode: feature
change_budget:
  max_files: 10
  max_loc: 500
scope:
  in:
    - "src/**"
  out:
    - "vendor/**"
invariants:
  - "Must not break existing tests"
acceptance:
  - id: "A1"
    given: "Code compiles"
    when: "Tests are run"
    then: "All tests pass"
contracts:
  - type: "openapi"
    path: "api.yaml"
"#;

    const FULL_SPEC_YAML: &str = r#"
id: "AGENT-001"
title: "P0 Blockers: Truthful System Loop with Auditability and Control"
risk_tier: 1
mode: feature
change_budget:
  max_files: 40
  max_loc: 2500
blast_radius:
  modules: ["orchestration", "api-server", "workers"]
  data_migration: true
operational_rollback_slo: "10m"
scope:
  in:
    - "iterations/v3/api-server/src/main.rs"
    - "iterations/v3/workers/src/executor.rs"
  out:
    - "iterations/v3/apple-silicon/"
    - "iterations/v3/enrichers/"
invariants:
  - "No simulations in production"
  - "All task operations must be auditable"
acceptance:
  - id: "A1"
    given: "Task is created and executed"
    when: "GET /tasks/:id is called"
    then: "Response includes non-empty events[] array"
  - id: "A2"
    given: "Task execution triggers worker call"
    when: "Worker receives HTTP request"
    then: "Request contains mapped CawsSpec"
non_functional:
  a11y: ["keyboard-navigation"]
  security:
    - "No plaintext secrets"
observability:
  logs:
    - "task.operation.*"
  metrics:
    - "api_request_duration"
  traces:
    - "task_execution_flow"
rollback:
  - "Drop newly created tables"
contracts:
  - type: "openapi"
    path: "docs/api/tasks.yaml"
    operations:
      - "getTaskEvents"
      - "cancelTask"
  - type: "database"
    tables:
      - "audit_logs"
      - "task_events"
"#;

    // ── Deserialization ────────────────────────────────────────────────────

    #[test]
    fn test_parse_minimal_spec() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        assert_eq!(spec.id, "FEAT-0001");
        assert_eq!(spec.risk_tier, 2);
        assert_eq!(spec.mode, SpecMode::Feature);
        assert_eq!(spec.invariants.len(), 1);
        assert_eq!(spec.acceptance.len(), 1);
        assert_eq!(spec.acceptance[0].id, "A1");
        assert_eq!(spec.acceptance[0].status, CriterionStatus::Pending);
    }

    #[test]
    fn test_parse_full_spec() {
        let spec = WorkingSpec::from_yaml_str(FULL_SPEC_YAML).unwrap();
        assert_eq!(spec.id, "AGENT-001");
        assert_eq!(spec.risk_tier, 1);
        assert_eq!(spec.invariants.len(), 2);
        assert_eq!(spec.acceptance.len(), 2);
        assert_eq!(spec.contracts.len(), 2);

        let blast = spec.blast_radius.as_ref().unwrap();
        assert_eq!(blast.modules.len(), 3);
        assert!(blast.data_migration);

        let budget = spec.change_budget.as_ref().unwrap();
        assert_eq!(budget.max_files, 40);
        assert_eq!(budget.max_loc, 2500);

        let scope = spec.scope.as_ref().unwrap();
        assert_eq!(scope.include.len(), 2);
        assert_eq!(scope.exclude.len(), 2);

        assert_eq!(spec.rollback.len(), 1);
    }

    #[test]
    fn test_parse_all_modes() {
        for (yaml_mode, expected) in [
            ("feature", SpecMode::Feature),
            ("refactor", SpecMode::Refactor),
            ("fix", SpecMode::Fix),
            ("doc", SpecMode::Doc),
            ("chore", SpecMode::Chore),
        ] {
            let yaml = format!(
                r#"
id: "TEST-0001"
title: "Test spec for mode parsing"
risk_tier: 3
mode: {yaml_mode}
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#
            );
            let spec = WorkingSpec::from_yaml_str(&yaml).unwrap();
            assert_eq!(spec.mode, expected, "Failed for mode: {yaml_mode}");
        }
    }

    #[test]
    fn test_parse_criterion_statuses() {
        let yaml = r#"
id: "TEST-0001"
title: "Test spec for statuses"
risk_tier: 3
mode: fix
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
    status: pending
  - id: "A2"
    given: "x"
    when: "y"
    then: "z"
    status: in_progress
  - id: "A3"
    given: "x"
    when: "y"
    then: "z"
    status: completed
  - id: "A4"
    given: "x"
    when: "y"
    then: "z"
    status: failed
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        assert_eq!(spec.acceptance[0].status, CriterionStatus::Pending);
        assert_eq!(spec.acceptance[1].status, CriterionStatus::InProgress);
        assert_eq!(spec.acceptance[2].status, CriterionStatus::Completed);
        assert_eq!(spec.acceptance[3].status, CriterionStatus::Failed);
    }

    // ── Risk Tier ──────────────────────────────────────────────────────────

    #[test]
    fn test_risk_tier_from_level() {
        assert_eq!(RiskTier::from_level(1), Some(RiskTier::Critical));
        assert_eq!(RiskTier::from_level(2), Some(RiskTier::Standard));
        assert_eq!(RiskTier::from_level(3), Some(RiskTier::LowRisk));
        assert_eq!(RiskTier::from_level(0), None);
        assert_eq!(RiskTier::from_level(4), None);
    }

    #[test]
    fn test_risk_tier_thresholds() {
        assert!((RiskTier::Critical.coverage_threshold() - 0.90).abs() < f64::EPSILON);
        assert!((RiskTier::Standard.coverage_threshold() - 0.80).abs() < f64::EPSILON);
        assert!((RiskTier::LowRisk.coverage_threshold() - 0.70).abs() < f64::EPSILON);

        assert!(RiskTier::Critical.requires_manual_review());
        assert!(!RiskTier::Standard.requires_manual_review());

        assert!(RiskTier::Critical.requires_contracts());
        assert!(RiskTier::Standard.requires_contracts());
        assert!(!RiskTier::LowRisk.requires_contracts());
    }

    // ── Validation ─────────────────────────────────────────────────────────

    #[test]
    fn test_validate_valid_spec() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let issues = spec.validate();
        assert!(issues.is_empty(), "Expected no issues, got: {issues:?}");
    }

    #[test]
    fn test_validate_empty_invariants() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 3
mode: feature
invariants: []
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let issues = spec.validate();
        assert!(issues.iter().any(|i| i.field == "invariants"));
    }

    #[test]
    fn test_validate_empty_acceptance() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 3
mode: feature
invariants: ["rule"]
acceptance: []
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let issues = spec.validate();
        assert!(issues.iter().any(|i| i.field == "acceptance"));
    }

    #[test]
    fn test_validate_invalid_risk_tier() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 5
mode: feature
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let issues = spec.validate();
        assert!(issues.iter().any(|i| i.field == "risk_tier"));
    }

    #[test]
    fn test_validate_empty_scope_in() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 3
mode: feature
scope:
  in: []
  out: ["vendor/**"]
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let issues = spec.validate();
        assert!(issues.iter().any(|i| i.field == "scope.in"));
    }

    #[test]
    fn test_validate_tier1_requires_contracts() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 1
mode: feature
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
contracts: []
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let issues = spec.validate();
        assert!(issues.iter().any(|i| i.field == "contracts"));
    }

    #[test]
    fn test_validate_short_title() {
        let yaml = r#"
id: "FEAT-0001"
title: "Short"
risk_tier: 3
mode: feature
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let issues = spec.validate();
        assert!(issues.iter().any(|i| i.field == "title"));
    }

    #[test]
    fn test_is_valid_shorthand() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        assert!(spec.is_valid());
    }

    // ── Budget Checker ─────────────────────────────────────────────────────

    #[test]
    fn test_budget_check_within_limits() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        assert!(BudgetChecker::check(&spec, 5, 200).is_ok());
    }

    #[test]
    fn test_budget_check_at_exact_limit() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        // max_files=10, max_loc=500
        assert!(BudgetChecker::check(&spec, 10, 500).is_ok());
    }

    #[test]
    fn test_budget_check_files_exceeded() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let result = BudgetChecker::check(&spec, 15, 200);
        assert!(result.is_err());
        match result.unwrap_err() {
            BudgetViolation::FilesExceeded { limit, actual } => {
                assert_eq!(limit, 10);
                assert_eq!(actual, 15);
            }
            other => panic!("Expected FilesExceeded, got: {other:?}"),
        }
    }

    #[test]
    fn test_budget_check_loc_exceeded() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let result = BudgetChecker::check(&spec, 5, 600);
        assert!(result.is_err());
        match result.unwrap_err() {
            BudgetViolation::LocExceeded { limit, actual } => {
                assert_eq!(limit, 500);
                assert_eq!(actual, 600);
            }
            other => panic!("Expected LocExceeded, got: {other:?}"),
        }
    }

    #[test]
    fn test_budget_check_no_budget() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 3
mode: feature
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let result = BudgetChecker::check(&spec, 5, 200);
        assert!(matches!(result, Err(BudgetViolation::NoBudget)));
    }

    /// Simple test waiver for budget checking
    struct TestWaiver {
        is_budget: bool,
        is_active: bool,
    }

    impl WaiverCheck for TestWaiver {
        fn is_budget_waiver(&self) -> bool {
            self.is_budget
        }
        fn is_active_now(&self) -> bool {
            self.is_active
        }
    }

    #[test]
    fn test_budget_check_with_active_waiver() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let waivers = vec![TestWaiver { is_budget: true, is_active: true }];
        // Exceeds budget but waiver is active → OK
        assert!(BudgetChecker::check_with_waivers(&spec, 20, 1000, &waivers).is_ok());
    }

    #[test]
    fn test_budget_check_with_expired_waiver() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let waivers = vec![TestWaiver { is_budget: true, is_active: false }];
        // Exceeds budget and waiver is expired → violation
        assert!(BudgetChecker::check_with_waivers(&spec, 20, 1000, &waivers).is_err());
    }

    #[test]
    fn test_budget_check_with_wrong_gate_waiver() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let waivers = vec![TestWaiver { is_budget: false, is_active: true }];
        // Active waiver but for wrong gate → violation
        assert!(BudgetChecker::check_with_waivers(&spec, 20, 1000, &waivers).is_err());
    }

    // ── Scope Enforcer ─────────────────────────────────────────────────────

    #[test]
    fn test_scope_in_scope_path() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let enforcer = ScopeEnforcer::from_spec(&spec).unwrap();
        assert!(enforcer.is_in_scope("src/lib.rs"));
        assert!(enforcer.is_in_scope("src/deep/nested/file.rs"));
    }

    #[test]
    fn test_scope_out_takes_precedence() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 3
mode: feature
scope:
  in:
    - "**"
  out:
    - "vendor/**"
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let enforcer = ScopeEnforcer::from_spec(&spec).unwrap();
        // Everything is in scope.in, but vendor is excluded
        assert!(!enforcer.is_in_scope("vendor/lib.rs"));
        assert!(enforcer.is_in_scope("src/main.rs"));
    }

    #[test]
    fn test_scope_neither_in_nor_out() {
        let spec = WorkingSpec::from_yaml_str(MINIMAL_SPEC_YAML).unwrap();
        let enforcer = ScopeEnforcer::from_spec(&spec).unwrap();
        // "docs/README.md" matches neither src/** nor vendor/**
        assert!(!enforcer.is_in_scope("docs/README.md"));
    }

    #[test]
    fn test_scope_out_blocks_matching_in() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 3
mode: feature
scope:
  in:
    - "crates/**"
  out:
    - "crates/legacy/**"
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let enforcer = ScopeEnforcer::from_spec(&spec).unwrap();
        assert!(enforcer.is_in_scope("crates/core/lib.rs"));
        assert!(!enforcer.is_in_scope("crates/legacy/old.rs"));
    }

    #[test]
    fn test_scope_no_scope_defined() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec"
risk_tier: 3
mode: feature
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let result = ScopeEnforcer::from_spec(&spec);
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_with_specific_files() {
        let yaml = r#"
id: "FEAT-0001"
title: "A valid test feature spec with specific paths"
risk_tier: 3
mode: feature
scope:
  in:
    - "src/main.rs"
    - "src/lib.rs"
  out: []
invariants: ["rule"]
acceptance:
  - id: "A1"
    given: "x"
    when: "y"
    then: "z"
"#;
        let spec = WorkingSpec::from_yaml_str(yaml).unwrap();
        let enforcer = ScopeEnforcer::from_spec(&spec).unwrap();
        assert!(enforcer.is_in_scope("src/main.rs"));
        assert!(enforcer.is_in_scope("src/lib.rs"));
        assert!(!enforcer.is_in_scope("src/other.rs"));
    }
}
