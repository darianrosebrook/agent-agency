//! Integration Tests for V3 Agent Agency System
//!
//! This crate provides comprehensive integration testing for all V3 components,
//! including cross-component communication, end-to-end workflows, and performance benchmarks.
//!
//! This module has been refactored into submodules for better organization.
//! See the `integration_tests/` subdirectory for the implementation details.

pub mod fixtures;
pub mod helpers;
pub mod mocks;
pub mod test_utils;

pub mod autonomous_pipeline_test;
pub mod claim_extraction_tests;
pub mod council_tests;
pub mod multimodal_rag_e2e_tests;
pub mod multimodal_rag_integration_test;
pub mod performance_benchmarks;

// Re-export everything from the integration_tests module
pub use integration_tests::*;

// Re-export existing modules
pub use fixtures::*;
pub use helpers::*;
pub use mocks::*;
pub use test_utils::*;

pub use multimodal_rag_e2e_tests::{MultimodalRagE2eTests, PerformanceMetrics};

// Declare the integration_tests module
pub mod integration_tests;
