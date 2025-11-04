//! Core types for evidence collection module

use schemars::JsonSchema;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Evidence collection result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceCollectionResult {
    pub evidence: Vec<EvidenceItem>,
    pub total_items: usize,
    #[schemars(with = "String")]
    pub collection_time: DateTime<Utc>,
    pub confidence: f64,
}

/// Individual evidence item
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceItem {
    pub id: String,
    pub evidence_type: EvidenceType,
    pub source: EvidenceSource,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub confidence: f64,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
}

/// Types of evidence
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EvidenceType {
    CodeAnalysis,
    TestExecution,
    Documentation,
    Performance,
    Security,
    Constitutional,
    Other(String),
}

/// Source of evidence
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EvidenceSource {
    StaticAnalysis,
    RuntimeExecution,
    DocumentationReview,
    PerformanceTest,
    SecurityScan,
    ConstitutionalCheck,
    Other(String),
}

/// Evidence collector configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceCollectorConfig {
    pub enable_code_analysis: bool,
    pub enable_test_execution: bool,
    pub enable_documentation: bool,
    pub enable_performance: bool,
    pub enable_security: bool,
    pub enable_constitutional: bool,
    pub max_evidence_items: usize,
    pub confidence_threshold: f64,
    pub timeout_seconds: u64,
}

impl Default for EvidenceCollectorConfig {
    fn default() -> Self {
        Self {
            enable_code_analysis: true,
            enable_test_execution: true,
            enable_documentation: true,
            enable_performance: true,
            enable_security: true,
            enable_constitutional: true,
            max_evidence_items: 1000,
            confidence_threshold: 0.7,
            timeout_seconds: 300,
        }
    }
}

/// Evidence analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceAnalysisResult {
    pub analysis_type: AnalysisType,
    pub findings: Vec<Finding>,
    pub summary: String,
    pub confidence: f64,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
}

/// Types of analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AnalysisType {
    CodeQuality,
    TestCoverage,
    DocumentationCompleteness,
    PerformanceMetrics,
    SecurityVulnerabilities,
    ConstitutionalCompliance,
    Other(String),
}

/// Individual finding
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: Option<String>,
    pub confidence: f64,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Evidence filter configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceFilterConfig {
    pub min_confidence: f64,
    pub max_items: usize,
    pub severity_filter: Option<Severity>,
    pub type_filter: Option<EvidenceType>,
    pub source_filter: Option<EvidenceSource>,
}

impl Default for EvidenceFilterConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_items: 100,
            severity_filter: None,
            type_filter: None,
            source_filter: None,
        }
    }
}
