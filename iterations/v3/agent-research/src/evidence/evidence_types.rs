//! Core types for evidence collection

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Code metrics for analysis
#[derive(Debug)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub function_count: usize,
}

/// Test timing data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTimingData {
    pub test_name: String,
    pub duration_ms: f64,
    pub setup_time_ms: Option<f64>,
    pub teardown_time_ms: Option<f64>,
    pub timestamp: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteTimingData {
    pub suite_name: String,
    pub tests: Vec<TestTimingData>,
    pub total_duration_ms: f64,
    pub timestamp: String,
}

#[derive(Debug)]
pub struct TestTimingAnalysis {
    pub test_count: usize,
    pub average_time_ms: f64,
    pub p95_time_ms: f64,
    pub regressions_detected: usize,
    pub slowest_test: Option<String>,
}

/// Verification methods enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationMethod {
    CodeAnalysis,
    TestExecution,
    DocumentationReview,
    PerformanceMeasurement,
    SecurityScan,
    ConstitutionalCheck,
    Measurement,
    LogicalAnalysis,
    ProcessAnalysis,
}

/// Evidence collection configuration
#[derive(Debug, Clone)]
pub struct EvidenceCollectorConfig {
    pub min_relevance_threshold: f64,
    pub min_credibility_threshold: f64,
    pub max_evidence_per_claim: usize,
    pub enable_cross_reference: bool,
    pub enable_source_validation: bool,
}

impl Default for EvidenceCollectorConfig {
    fn default() -> Self {
        Self {
            min_relevance_threshold: 0.5,
            min_credibility_threshold: 0.6,
            max_evidence_per_claim: 5,
            enable_cross_reference: true,
            enable_source_validation: true,
        }
    }
}
