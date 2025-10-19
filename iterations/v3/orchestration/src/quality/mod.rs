//! Quality Gates System
//!
//! First-line quality enforcement with CAWS compliance, linting, testing,
//! coverage, and mutation analysis for autonomous task execution.

pub mod gates;
pub mod orchestrator;
pub mod satisficing;

pub use gates::{QualityGate, GateStatus, QualityGateResult, QualityThresholds};
pub use orchestrator::{QualityGateOrchestrator, QualityGateOrchestratorConfig, QualityReport};
pub use satisficing::{SatisficingEvaluator, SatisficingConfig, SatisficingResult};
