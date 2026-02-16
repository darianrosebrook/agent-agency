//! V4 Governance
//!
//! This crate implements governance modes and CAWS gate enforcement.
//! It provides the policy layer that determines what actions are allowed
//! and under what conditions.
//!
//! ## Design Principles
//!
//! 1. **Fail-closed** - Unknown state = denied
//! 2. **Neural advisory, symbolic authoritative** - LLMs propose, rules decide
//! 3. **Hard thresholds** - No exceptions to critical gates
//! 4. **Auditable** - Every decision is logged with reasoning

pub mod gates;
pub mod modes;
pub mod policy;
pub mod waivers;
pub mod working_spec;

pub use gates::{CAWSEvaluator, GateEvaluation, GateVerdict};
pub use modes::{GovernanceMode, ModeTransition};
pub use policy::{ActionRequest, Policy, PolicyDecision, PolicyViolation};
pub use waivers::{Waiver, WaiverError, WaiverGate, WaiverSet, WaiverStatus};
pub use working_spec::{
    AcceptanceCriterion, BlastRadius, BudgetChecker, BudgetViolation, ChangeBudget,
    CriterionStatus, RiskTier, ScopeEnforcer, SpecMode, WorkingSpec,
};
