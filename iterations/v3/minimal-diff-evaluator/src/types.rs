use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Minimal diff evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEvaluationResult {
    /// Evaluation ID
    pub id: Uuid,
    /// Overall surgical change score (0.0 to 1.0, higher is better)
    pub surgical_change_score: f64,
    /// Change complexity score (0.0 to 1.0, higher is more complex)
    pub change_complexity_score: f64,
    /// Change impact score (0.0 to 1.0, higher is more impactful)
    pub change_impact_score: f64,
    /// Language-specific analysis results
    pub language_analysis: LanguageAnalysisResult,
    /// Change classification
    pub change_classification: ChangeClassification,
    /// Impact analysis
    pub impact_analysis: ImpactAnalysis,
    /// Recommendations for improvement
    pub recommendations: Vec<Recommendation>,
    /// Evaluation metadata
    pub metadata: EvaluationMetadata,
}

/// Language-specific analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysisResult {
    /// Programming language detected
    pub language: ProgrammingLanguage,
    /// AST-based change analysis
    pub ast_changes: Vec<ASTChange>,
    /// Code quality metrics
    pub quality_metrics: QualityMetrics,
    /// Complexity metrics
    pub complexity_metrics: ComplexityMetrics,
    /// Language-specific violations
    pub violations: Vec<LanguageViolation>,
    /// Language-specific warnings
    pub warnings: Vec<LanguageWarning>,
}

/// Programming language types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProgrammingLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Java,
    Cpp,
    C,
    Go,
    Swift,
    Kotlin,
    Scala,
    Haskell,
    OCaml,
    FSharp,
    Clojure,
    Elixir,
    Erlang,
    Ruby,
    PHP,
    Perl,
    Lua,
    R,
    Julia,
    Zig,
    Nim,
    Dart,
    Unknown,
}

/// AST change representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTChange {
    /// Change ID
    pub id: Uuid,
    /// Change type
    pub change_type: ASTChangeType,
    /// Node type affected
    pub node_type: String,
    /// Node location
    pub location: SourceLocation,
    /// Change description
    pub description: String,
    /// Impact level
    pub impact_level: ImpactLevel,
    /// Dependencies affected
    pub dependencies: Vec<String>,
}

/// AST change types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ASTChangeType {
    /// Function signature change
    FunctionSignature,
    /// Function body change
    FunctionBody,
    /// Class definition change
    ClassDefinition,
    /// Interface change
    InterfaceChange,
    /// Type definition change
    TypeDefinition,
    /// Import/export change
    ImportExport,
    /// Constant change
    ConstantChange,
    /// Variable change
    VariableChange,
    /// Comment change
    CommentChange,
    /// Documentation change
    DocumentationChange,
    /// Configuration change
    ConfigurationChange,
    /// Test change
    TestChange,
    /// Other change
    Other,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file_path: String,
    /// Start line number
    pub start_line: u32,
    /// End line number
    pub end_line: u32,
    /// Start column number
    pub start_column: u32,
    /// End column number
    pub end_column: u32,
    /// Byte offset start
    pub start_byte: usize,
    /// Byte offset end
    pub end_byte: usize,
}

/// Impact level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactLevel {
    /// No impact
    None,
    /// Low impact
    Low,
    /// Medium impact
    Medium,
    /// High impact
    High,
    /// Critical impact
    Critical,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Cyclomatic complexity
    pub cyclomatic_complexity: u32,
    /// Cognitive complexity
    pub cognitive_complexity: u32,
    /// Lines of code
    pub lines_of_code: u32,
    /// Comment density
    pub comment_density: f64,
    /// Test coverage (if available)
    pub test_coverage: Option<f64>,
    /// Code duplication percentage
    pub duplication_percentage: f64,
}

/// Complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Structural complexity
    pub structural_complexity: f64,
    /// Logical complexity
    pub logical_complexity: f64,
    /// Dependency complexity
    pub dependency_complexity: f64,
    /// Overall complexity score
    pub overall_complexity: f64,
}

/// Language violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageViolation {
    /// Violation ID
    pub id: Uuid,
    /// Rule violated
    pub rule: String,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Description
    pub description: String,
    /// Location
    pub location: Option<SourceLocation>,
    /// Suggestion for fix
    pub suggestion: Option<String>,
}

/// Violation severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

/// Language warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageWarning {
    /// Warning ID
    pub id: Uuid,
    /// Rule that triggered warning
    pub rule: String,
    /// Description
    pub description: String,
    /// Location
    pub location: Option<SourceLocation>,
    /// Suggestion
    pub suggestion: Option<String>,
}

/// Change classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeClassification {
    /// Primary change type
    pub primary_type: ChangeType,
    /// Secondary change types
    pub secondary_types: Vec<ChangeType>,
    /// Change category
    pub category: ChangeCategory,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Confidence score
    pub confidence: f64,
}

/// Change type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangeType {
    /// Bug fix
    BugFix,
    /// Feature addition
    FeatureAddition,
    /// Refactoring
    Refactoring,
    /// Performance improvement
    PerformanceImprovement,
    /// Security fix
    SecurityFix,
    /// Documentation update
    DocumentationUpdate,
    /// Configuration change
    ConfigurationChange,
    /// Test addition
    TestAddition,
    /// Test modification
    TestModification,
    /// Dependency update
    DependencyUpdate,
    /// Code style change
    CodeStyleChange,
    /// Other change
    Other,
}

/// Change category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeCategory {
    /// Functional change
    Functional,
    /// Non-functional change
    NonFunctional,
    /// Cosmetic change
    Cosmetic,
    /// Infrastructure change
    Infrastructure,
    /// Test change
    Test,
    /// Documentation change
    Documentation,
}

/// Risk level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    /// Very low risk
    VeryLow,
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Very high risk
    VeryHigh,
}

/// Impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Files affected
    pub files_affected: u32,
    /// Functions affected
    pub functions_affected: u32,
    /// Classes affected
    pub classes_affected: u32,
    /// Interfaces affected
    pub interfaces_affected: u32,
    /// Dependencies affected
    pub dependencies_affected: u32,
    /// Test files affected
    pub test_files_affected: u32,
    /// Documentation files affected
    pub documentation_files_affected: u32,
    /// Configuration files affected
    pub configuration_files_affected: u32,
    /// Impact score (0.0 to 1.0)
    pub impact_score: f64,
    /// Blast radius (files that might be affected)
    pub blast_radius: u32,
}

/// Recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation ID
    pub id: Uuid,
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Priority level
    pub priority: PriorityLevel,
    /// Description
    pub description: String,
    /// Action required
    pub action: String,
    /// Expected benefit
    pub expected_benefit: String,
    /// Implementation effort
    pub implementation_effort: EffortLevel,
}

/// Recommendation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationType {
    /// Reduce complexity
    ReduceComplexity,
    /// Improve test coverage
    ImproveTestCoverage,
    /// Add documentation
    AddDocumentation,
    /// Refactor code
    RefactorCode,
    /// Optimize performance
    OptimizePerformance,
    /// Fix security issues
    FixSecurityIssues,
    /// Improve maintainability
    ImproveMaintainability,
    /// Reduce dependencies
    ReduceDependencies,
    /// Other recommendation
    Other,
}

/// Priority level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLevel {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Effort level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffortLevel {
    /// Minimal effort
    Minimal,
    /// Low effort
    Low,
    /// Medium effort
    Medium,
    /// High effort
    High,
    /// Very high effort
    VeryHigh,
}

/// Evaluation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetadata {
    /// Evaluation timestamp
    pub timestamp: DateTime<Utc>,
    /// Evaluation duration (milliseconds)
    pub duration_ms: u64,
    /// Files analyzed
    pub files_analyzed: u32,
    /// Lines analyzed
    pub lines_analyzed: u32,
    /// AST nodes analyzed
    pub ast_nodes_analyzed: u32,
    /// Language support version
    pub language_support_version: String,
    /// Evaluation tool version
    pub tool_version: String,
}

/// Diff evaluation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEvaluationConfig {
    /// Enable AST-based analysis
    pub enable_ast_analysis: bool,
    /// Enable impact analysis
    pub enable_impact_analysis: bool,
    /// Enable language-specific analysis
    pub enable_language_analysis: bool,
    /// Maximum file size for analysis (bytes)
    pub max_file_size: u64,
    /// Maximum analysis time (seconds)
    pub max_analysis_time: u64,
    /// Language-specific configurations
    pub language_configs: HashMap<ProgrammingLanguage, LanguageConfig>,
    /// Quality thresholds
    pub quality_thresholds: QualityThresholds,
}

/// Language-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Enable language-specific analysis
    pub enabled: bool,
    /// Custom rules for this language
    pub custom_rules: Vec<String>,
    /// Complexity thresholds
    pub complexity_thresholds: ComplexityThresholds,
    /// Quality thresholds
    pub quality_thresholds: QualityThresholds,
}

/// Complexity thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityThresholds {
    /// Maximum cyclomatic complexity
    pub max_cyclomatic_complexity: u32,
    /// Maximum cognitive complexity
    pub max_cognitive_complexity: u32,
    /// Maximum lines per function
    pub max_lines_per_function: u32,
    /// Maximum parameters per function
    pub max_parameters_per_function: u32,
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
}

/// Quality thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum test coverage percentage
    pub min_test_coverage: f64,
    /// Maximum code duplication percentage
    pub max_code_duplication: f64,
    /// Minimum comment density
    pub min_comment_density: f64,
    /// Maximum file size (lines)
    pub max_file_size_lines: u32,
    /// Maximum function size (lines)
    pub max_function_size_lines: u32,
}

/// Diff evaluation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEvaluationStats {
    /// Total evaluations performed
    pub total_evaluations: u64,
    /// Average surgical change score
    pub avg_surgical_change_score: f64,
    /// Average change complexity score
    pub avg_change_complexity_score: f64,
    /// Average change impact score
    pub avg_change_impact_score: f64,
    /// Evaluations by language
    pub evaluations_by_language: HashMap<ProgrammingLanguage, u64>,
    /// Evaluations by change type
    pub evaluations_by_change_type: HashMap<ChangeType, u64>,
    /// Evaluations by risk level
    pub evaluations_by_risk_level: HashMap<RiskLevel, u64>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}
