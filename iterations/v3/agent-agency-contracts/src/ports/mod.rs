//! Ports for hexagonal architecture
//!
//! Ports define service boundaries and enable dependency injection.
//! Implementations live in consuming crates to break circular dependencies.

pub mod planning_engine;
pub mod memory_system;
pub mod council_coordinator;
pub mod research_evidence;
pub mod tool_chain;
pub mod data_processing;

