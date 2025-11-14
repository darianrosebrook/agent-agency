//! Ports for hexagonal architecture
//!
//! Ports define service boundaries and enable dependency injection.
//! Implementations live in consuming crates to break circular dependencies.

pub mod council_coordinator;
pub mod data_processing;
pub mod memory_system;
pub mod planning_engine;
pub mod research_evidence;
pub mod tool_chain;
