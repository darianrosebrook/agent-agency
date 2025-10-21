//! CAWS (Coding-Agent Working Standard) integration
//!
//! Provides budget checking, council approval, and waiver management
//! for enforcing CAWS policies during autonomous file editing.

pub mod budget_checker;
pub mod council_approval;
pub mod waiver_generator;

pub use budget_checker::{BudgetChecker, BudgetState, BudgetLimits, BudgetError};
pub use council_approval::{CouncilApprovalWorkflow, BudgetOverrunPlea, PleaEvidence, CouncilDecision};
pub use waiver_generator::{WaiverGenerator, Waiver};
