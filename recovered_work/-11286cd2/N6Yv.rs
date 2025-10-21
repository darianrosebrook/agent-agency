//! End-to-End Test Scenarios for Autonomous Task Execution
//!
//! Comprehensive integration tests covering the complete autonomous execution pipeline
//! from task submission through planning, execution, quality assurance, and refinement.

pub mod scenarios;
pub mod harness;
pub mod assertions;
pub mod fixtures;

pub use scenarios::{E2eTestScenarios, TestScenario};
pub use harness::{E2eTestHarness, TestEnvironment};
pub use assertions::E2eAssertions;
pub use fixtures::TestFixtures;


