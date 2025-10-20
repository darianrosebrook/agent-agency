//! Planning Agent for Agent Agency V3
//!
//! The Planning Agent is responsible for transforming natural language task requests
//! into validated, executable Working Specifications. It integrates CAWS validation
//! to ensure specifications meet quality and safety requirements before execution.

pub mod error;
pub mod planner;
pub mod caws_integration;
pub mod validation_pipeline;
pub mod refinement_engine;

pub use error::{PlanningError, PlanningResult};
pub use planner::{PlanningAgent, PlanningConfig, PlanningRequest, PlanningResponse};
pub use caws_integration::{CawsValidator, ValidationContext};
pub use validation_pipeline::{ValidationPipeline, ValidationStage};
pub use refinement_engine::{RefinementEngine, RefinementSuggestion};
