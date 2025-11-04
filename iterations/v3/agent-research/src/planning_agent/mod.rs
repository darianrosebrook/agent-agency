//! Planning Agent for Agent Agency V3
//!
//! The Planning Agent is responsible for transforming natural language task requests
//! into validated, executable Working Specifications. It integrates CAWS validation
//! to ensure specifications meet quality and safety requirements before execution.

pub mod planning_errors;
pub mod planner;
pub mod types;
pub mod planning_caws_integration;
pub mod validation_pipeline;
pub mod refinement_engine;
pub mod validation;
pub mod spec_generation;

pub use planning_errors::{PlanningError, PlanningResult};
pub use planner::PlanningAgent;
pub use types::{
    PlanningConfig, PlanningRequest, PlanningResponse, PlanningMetadata,
    ValidationResults, RefinementRecord, RiskAssessment, ValidationStatus, ValidationIssue, IssueSeverity,
};
pub use planning_caws_integration::{CawsValidator, ValidationContext, ValidationOptions};
pub use validation_pipeline::{ValidationPipeline, ValidationStage};
pub use refinement_engine::{RefinementEngine, RefinementSuggestion};
