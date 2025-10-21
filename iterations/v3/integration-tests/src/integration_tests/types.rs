//! Shared types for integration tests
//!
//! This module contains common data structures used across integration tests.

use std::collections::HashMap;

/// Test result with detailed metrics
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub duration: std::time::Duration,
    pub success: bool,
    pub error_message: Option<String>,
    pub metrics: HashMap<String, f64>,
}
