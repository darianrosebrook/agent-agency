//! Type definitions for shared data structures
//!
//! This module contains all the data transfer objects (DTOs) that are shared
//! between multiple crates in the workspace. These types define the boundaries
//! between different domains and ensure type safety across crate boundaries.
//!
//! @author @darianrosebrook

pub mod planning;
pub mod execution;
pub mod data;
pub mod council;
pub mod memory;
pub mod research;
pub mod tool_chain;
pub mod data_processing;
pub mod prelude;

