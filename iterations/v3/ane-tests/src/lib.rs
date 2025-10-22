//! Standalone ANE tests and benchmarks library
//!
//! This crate provides isolated testing and benchmarking of ANE components
//! without dependencies on the main candle backend.

pub mod errors;
pub mod compat;
pub mod resource_pool;
pub mod models;
pub mod infer;
pub mod metrics;
pub mod manager;
