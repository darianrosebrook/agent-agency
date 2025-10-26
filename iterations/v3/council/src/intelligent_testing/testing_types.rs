//! Core types for intelligent edge case testing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Input type enumeration for test parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

/// Test input specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    pub name: String,
    pub input_type: InputType,
    pub required: bool,
    pub description: String,
}

/// Edge case test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCaseTestResult {
    pub test_id: Uuid,
    pub test_name: String,
    pub passed: bool,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub coverage_improvement: f64,
    pub edge_case_coverage: f64,
    pub generation_confidence: f64,
}

/// Edge case test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCaseReport {
    pub report_id: String,
    pub test_results: Vec<EdgeCaseTestResult>,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub coverage_score: f64,
}

/// Edge case test specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCaseTest {
    pub test_id: Uuid,
    pub test_name: String,
    pub test_type: TestType,
    pub test_scenario: TestScenario,
    pub edge_case_type: EdgeCaseType,
    pub risk_level: RiskLevel,
    pub expected_behavior: String,
    pub generation_reason: String,
    pub confidence_score: f64,
}

/// Test case specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub test_id: Uuid,
    pub test_name: String,
    pub test_type: String,
    pub test_scenario: String,
    pub expected_outcome: String,
    pub test_data: HashMap<String, String>,
    pub priority: u32,
}

/// Intelligent Edge Case Testing System
#[derive(Debug)]
pub struct IntelligentEdgeCaseTesting {
    pub dynamic_test_generator: std::sync::Arc<super::generation::DynamicTestGenerator>,
    pub edge_case_analyzer: std::sync::Arc<super::analysis::EdgeCaseAnalyzer>,
    pub test_optimizer: std::sync::Arc<super::optimization::TestOptimizer>,
    pub coverage_analyzer: std::sync::Arc<super::performance::CoverageAnalyzer>,
    pub test_history: std::sync::Arc<tokio::sync::RwLock<HashMap<String, TestHistory>>>,
}

/// Intelligent test insights from edge case analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentTestInsights {
    pub dynamic_tests: DynamicTestResults,
    pub edge_case_analysis: EdgeCaseAnalysis,
    pub test_optimization: TestOptimization,
    pub coverage_analysis: CoverageAnalysis,
}

/// Dynamic test generation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTestResults {
    pub generated_tests: Vec<GeneratedTest>,
    pub test_coverage_improvement: f64,
    pub edge_case_coverage: f64,
    pub generation_confidence: f64,
    pub test_effectiveness_score: f64,
}

/// Generated test with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTest {
    pub test_id: Uuid,
    pub test_name: String,
    pub test_type: TestType,
    pub test_scenario: TestScenario,
    pub expected_outcome: ExpectedOutcome,
    pub edge_case_type: EdgeCaseType,
    pub generation_reason: String,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TestType {
    Unit,
    Integration,
    EdgeCase,
    Boundary,
    Performance,
    Security,
}

/// Test scenario enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TestScenario {
    HappyPath,
    EdgeCase,
    ErrorCondition,
    BoundaryValue,
    InvalidInput,
    Concurrency,
    Performance,
    Security,
}

/// Edge case type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCaseType {
    BoundaryValue,
    InvalidInput,
    NullUndefined,
    LargeData,
    SpecialCharacters,
    Concurrency,
    RaceCondition,
    MemoryLeak,
    PerformanceDegradation,
}

/// Risk level for edge cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Expected outcome enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectedOutcome {
    Success,
    Error,
    Exception,
    Timeout,
    PerformanceDegradation,
}

/// Test data enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestData {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<TestData>),
    Object(HashMap<String, TestData>),
}

/// Test data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataWithMetadata {
    pub data: TestData,
    pub data_type: DataType,
    pub generation_method: String,
    pub edge_case_reason: Option<String>,
}

/// Data type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Primitive,
    Complex,
    Collection,
    Custom,
}


// Result types for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCaseAnalysis {
    pub identified_edge_cases: Vec<String>,
    pub risk_assessment: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOptimization {
    pub optimized_tests: Vec<GeneratedTest>,
    pub efficiency_improvements: HashMap<String, f64>,
    pub redundancy_reduction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    pub current_coverage: f64,
    pub gaps_identified: Vec<String>,
    pub improvement_suggestions: Vec<String>,
}

// Test history and execution types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHistory {
    pub test_id: String,
    pub executions: Vec<TestExecution>,
    pub success_rate: f64,
    pub average_execution_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution {
    pub execution_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub result: TestResult,
    pub execution_time_ms: u64,
    pub coverage_achieved: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestResult {
    Passed,
    Failed(String),
    Skipped(String),
    Error(String),
}

// Execution context types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub test_suite: String,
    pub environment: String,
    pub configuration: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub max_concurrency: u32,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            test_suite: "default".to_string(),
            environment: "development".to_string(),
            configuration: HashMap::new(),
            timeout_seconds: 300,
            max_concurrency: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub memory_mb: u64,
    pub cpu_cores: u32,
    pub disk_space_mb: u64,
    pub network_bandwidth_mbps: u32,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            memory_mb: 1024,
            cpu_cores: 2,
            disk_space_mb: 1000,
            network_bandwidth_mbps: 100,
        }
    }
}

// Test specification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSpecification {
    pub test_id: String,
    pub description: String,
    pub inputs: Vec<TestInput>,
    pub expected_outputs: Vec<String>,
    pub execution_context: ExecutionContext,
    pub resource_requirements: ResourceRequirements,
    pub priority: u32,
    pub tags: Vec<String>,
}