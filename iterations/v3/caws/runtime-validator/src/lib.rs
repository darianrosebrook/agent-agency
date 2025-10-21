//! CAWS Runtime Validator
//!
//! Standalone CAWS (Coding-Agent Working Standard) service that provides:
//! - Policy validation and enforcement
//! - Budget checking and waiver management
//! - Runtime compliance verification
//! - Integration with MCP tools and orchestration

pub mod policy;
pub mod validator;
pub mod budget;
pub mod waiver;
pub mod integration;

pub use policy::{CawsPolicy, PolicyValidator};
pub use validator::{CawsValidator, ValidationResult, Violation};
pub use budget::{BudgetChecker, BudgetLimits, BudgetState};
pub use waiver::{WaiverGenerator, WaiverManager};
pub use integration::{McpIntegration, OrchestrationIntegration};
