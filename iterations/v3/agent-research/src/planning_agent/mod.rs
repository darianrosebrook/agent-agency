//! Planning Agent for Agent Agency V3
//!
//! The Planning Agent is responsible for transforming natural language task requests
//! into validated, executable Working Specifications. It integrates CAWS validation
//! to ensure specifications meet quality and safety requirements before execution.

pub mod planner;
pub mod planning_caws_integration;
pub mod planning_errors;
pub mod refinement_engine;
pub mod spec_generation;
pub mod types;
pub mod validation;
pub mod validation_pipeline;

pub use planner::PlanningAgent;
pub use planning_caws_integration::{CawsValidator, ValidationContext, ValidationOptions};
pub use planning_errors::{PlanningError, PlanningResult};
pub use refinement_engine::{RefinementEngine, RefinementSuggestion};
pub use types::{
    IssueSeverity, PlanningConfig, PlanningMetadata, PlanningRequest, PlanningResponse,
    RefinementRecord, RiskAssessment, ValidationIssue, ValidationResults, ValidationStatus,
};
pub use validation_pipeline::{ValidationPipeline, ValidationStage};
