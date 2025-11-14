//! Evaluation Framework Module
//!
//! This module provides comprehensive evaluation capabilities for agent orchestration,
//! including scenario-based testing, multi-dimensional metrics, and operator-quality reporting.
//!
//! @author @darianrosebrook

pub mod contracts;
pub mod determinism;
pub mod framework;
pub mod metrics;
#[cfg(any(test, feature = "evaluation"))]
pub mod playground;
pub mod query;
pub mod reporters;
pub mod scenario_runner;
pub mod sinks;
pub mod trace;

#[cfg(test)]
mod integration_test;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod success_criteria;

pub use framework::EvaluationReport;
pub use framework::{AgentEvaluation, EvaluationDimensions, EvaluationEngine, EvaluationScenario};
pub use scenario_runner::run_scenario;
