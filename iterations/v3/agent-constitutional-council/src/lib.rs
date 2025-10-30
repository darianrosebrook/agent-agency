//! Agent Constitutional Council
//!
//! This crate provides the core constitutional governance system for Agent Agency,
//! implementing four specialized AI judges that provide real-time oversight and
//! decision-making for autonomous agent operations.
//!
//! The council operates with hybrid reasoning: deterministic CAWS invariant checks
//! combined with LLM-based analysis for gray-zone decisions. All judges implement
//! the same hybrid pattern for consistent governance.
//!
//! ## Features
//!
//! - **Hybrid Constitutionalism**: Deterministic CAWS gates + LLM reasoning
//! - **Four Specialized Judges**: Constitutional, Technical, Quality, Integration
//! - **Engine Agnostic**: Generic over JudgeEngine trait (no direct CoreML dependency)
//! - **Structured IO**: JSON schema-validated verdicts and prompts
//! - **Performance Aware**: Token limits, caching, and SLA enforcement
//!
//! ## Architecture
//!
//! ```text
//! [agent-orchestration] → CouncilCoordinator<CoreMLEngine>
//!       ↓
//! [agent-constitutional-council] ← depends only on contracts
//!       ↓
//! [agent-agency-contracts] ← JudgeEngine trait, structured IO
//!       ↓
//! [engine-coreml] → system-acceleration → CoreML inference
//! ```
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod council;
pub mod judges;
pub mod invariants;
pub mod metrics;

pub use council::{CouncilCoordinator, ReviewContext};
pub use judges::{Judges, Judge, ConstitutionalJudge, TechnicalAuditor, QualityEvaluator, IntegrationValidator};
pub use invariants::run_caws_invariants;
pub use metrics::CouncilMetrics;

/// Council decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDecision {
    /// Overall decision label
    pub label: agent_agency_contracts::VerdictLabel,

    /// Confidence score (0.0-1.0)
    pub score: f32,

    /// Decision rationale
    pub rationale: String,

    /// All judge verdicts
    pub judge_verdicts: Vec<agent_agency_contracts::JudgeVerdict>,

    /// Consensus violations (if any)
    pub consensus_violations: Vec<String>,

    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Error types for council operations
#[derive(thiserror::Error, Debug)]
pub enum CouncilError {
    #[error("Engine error: {0}")]
    Engine(#[from] agent_agency_contracts::EngineError),

    #[error("Judge error: {0}")]
    Judge(String),

    #[error("Consensus failure: {0}")]
    Consensus(String),

    #[error("Invariant violation: {0}")]
    Invariant(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, CouncilError>;

/// Council result type for operations
pub type CouncilResult<T> = Result<T>;

