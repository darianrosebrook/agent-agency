//! Core types for intelligent edge case testing

use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestType {
    Unit,
    Integration,
    System,
    Performance,
    Security,
    EdgeCase,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCaseType {
    BoundaryValue,
    InvalidInput,
    NullEmpty,
    LargeData,
    SpecialCharacters,
    ConcurrentAccess,
    ResourceLimits,
    NetworkIssues,
    DataTypeMismatch,
    TimingIssues,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOutcome {
    Pass,
    Fail,
    Skip,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    MinValue,
    MaxValue,
    Length,
    Pattern,
    Required,
    Unique,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCaseFlag {
    Boundary,
    Invalid,
    Null,
    Empty,
    Large,
    SpecialChars,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestEnvironment {
    Development,
    Staging,
    Production,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Library,
    Service,
    Database,
    File,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    StateCheck,
    DataValidation,
    ResourceCheck,
    TimeCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutcomeType {
    Success,
    Failure,
    Partial,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionType {
    ExactMatch,
    RangeCheck,
    PatternMatch,
    PerformanceThreshold,
    ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    Exception,
    Timeout,
    Assertion,
    ResourceExhaustion,
    DataError,
}

/// Test scenario with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    pub scenario_name: String,
    pub input_data: HashMap<String, TestDataWithMetadata>,
    pub execution_context: ExecutionContext,
    pub preconditions: Vec<Precondition>,
    pub postconditions: Vec<Postcondition>,
}

/// Test data with metadata for intelligent processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataWithMetadata {
    pub data_type: DataType,
    pub value: serde_json::Value,
    pub constraints: Vec<Constraint>,
    pub edge_case_flags: Vec<EdgeCaseFlag>,
}

/// Data constraint specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub constraint_value: serde_json::Value,
    pub description: String,
}

/// Test execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub environment: TestEnvironment,
    pub dependencies: Vec<Dependency>,
    pub resources: ResourceRequirements,
    pub timeout_ms: u64,
}

/// External dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub dependency_name: String,
    pub dependency_type: DependencyType,
    pub version: String,
    pub required: bool,
}

/// Resource requirements for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub disk_space_mb: u64,
    pub network_bandwidth_mbps: u64,
}

/// Pre-execution condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precondition {
    pub condition_name: String,
    pub condition_type: ConditionType,
    pub condition_value: serde_json::Value,
    pub description: String,
}

/// Post-execution condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Postcondition {
    pub condition_name: String,
    pub condition_type: ConditionType,
    pub expected_value: serde_json::Value,
    pub description: String,
}

/// Expected test outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub outcome_type: OutcomeType,
    pub expected_result: serde_json::Value,
    pub success_criteria: Vec<SuccessCriterion>,
    pub failure_scenarios: Vec<FailureScenario>,
}

/// Success validation criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub criterion_name: String,
    pub criterion_type: CriterionType,
    pub expected_value: serde_json::Value,
    pub tolerance: Option<f64>,
}

/// Failure scenario specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureScenario {
    pub scenario_name: String,
    pub failure_type: FailureType,
    pub expected_error: String,
    pub error_code: Option<String>,
}

/// Test specification for component testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSpecification {
    pub spec_id: Uuid,
    pub component_name: String,
    pub test_requirements: Vec<TestRequirement>,
    pub edge_case_requirements: Vec<EdgeCaseRequirement>,
    pub performance_requirements: Vec<PerformanceRequirement>,
}

/// Test requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequirement {
    pub requirement_id: Uuid,
    pub requirement_type: String,
    pub description: String,
    pub priority: u32,
}

/// Edge case requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCaseRequirement {
    pub requirement_id: Uuid,
    pub edge_case_type: EdgeCaseType,
    pub description: String,
    pub test_coverage_required: f64,
}

/// Performance requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirement {
    pub requirement_id: Uuid,
    pub metric_name: String,
    pub target_value: f64,
    pub tolerance: f64,
}

/// Identified edge case from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedEdgeCase {
    pub edge_case_id: Uuid,
    pub edge_case_name: String,
    pub edge_case_type: EdgeCaseType,
    pub description: String,
    pub probability: f64,
    pub risk_level: RiskLevel,
    pub mitigation_strategy: Option<String>,
}

/// Edge case analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCaseAnalysis {
    pub identified_edge_cases: Vec<IdentifiedEdgeCase>,
    pub edge_case_coverage: f64,
    pub analysis_confidence: f64,
    pub risk_assessment: RiskAssessment,
    pub mitigation_strategies: Vec<MitigationStrategy>,
}

/// Risk assessment for edge cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_score: f64,
    pub risk_distribution: HashMap<RiskLevel, u32>,
    pub high_risk_areas: Vec<String>,
    pub risk_trends: Vec<RiskTrend>,
}

/// Risk trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTrend {
    pub trend_direction: TrendDirection,
    pub trend_magnitude: f64,
    pub trend_duration: u64,
    pub trend_confidence: f64,
}

/// Mitigation strategy for edge cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub strategy_name: String,
    pub strategy_type: StrategyType,
    pub effectiveness: f64,
    pub implementation_cost: f64,
    pub description: String,
}

/// Test optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOptimization {
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub efficiency_improvement: f64,
    pub redundancy_reduction: f64,
    pub optimization_confidence: f64,
    pub prioritized_tests: Vec<PrioritizedTest>,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_effort: ImplementationEffort,
    pub priority: Priority,
}

/// Prioritized test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedTest {
    pub test_id: Uuid,
    pub priority_score: f64,
    pub priority_reason: String,
    pub execution_order: u32,
    pub estimated_value: f64,
}

/// Coverage analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    pub overall_coverage: f64,
    pub coverage_breakdown: CoverageBreakdown,
    pub coverage_gaps: Vec<CoverageGap>,
    pub coverage_trends: Vec<CoverageTrend>,
    pub improvement_recommendations: Vec<CoverageRecommendation>,
}

/// Coverage breakdown by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageBreakdown {
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub edge_case_coverage: f64,
    pub integration_coverage: f64,
}

/// Coverage gap identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    pub gap_id: Uuid,
    pub gap_type: GapType,
    pub gap_description: String,
    pub gap_severity: GapSeverity,
    pub affected_components: Vec<String>,
}

/// Coverage trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrend {
    pub trend_direction: TrendDirection,
    pub trend_magnitude: f64,
    pub trend_duration: u64,
    pub trend_confidence: f64,
}

/// Coverage improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub expected_coverage_improvement: f64,
    pub implementation_effort: ImplementationEffort,
    pub priority: Priority,
}

/// Trend direction enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

/// Strategy type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    TestAddition,
    TestModification,
    TestRemoval,
    ProcessImprovement,
    ToolEnhancement,
}

/// Suggestion type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    AddTestCase,
    RemoveRedundantTest,
    OptimizeTestExecution,
    ImproveTestData,
    EnhanceAssertions,
}

/// Implementation effort enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Priority enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Gap type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapType {
    LineCoverage,
    BranchCoverage,
    EdgeCaseCoverage,
    IntegrationCoverage,
    PerformanceCoverage,
}

/// Gap severity enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Recommendation type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    AddUnitTests,
    AddIntegrationTests,
    AddEdgeCaseTests,
    ImproveTestQuality,
    OptimizeTestSuite,
}
