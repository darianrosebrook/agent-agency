//! Execution artifacts contract for comprehensive artifact tracking.
//!
//! Defines all artifacts produced during task execution with provenance,
//! test results, coverage data, and complete audit trails.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All artifacts produced during task execution with provenance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionArtifacts {
    /// Contract version for compatibility
    pub version: String,

    /// Task identifier
    pub task_id: Uuid,

    /// Working spec identifier
    pub working_spec_id: String,

    /// Execution iteration number
    pub iteration: u32,

    /// All code changes made during execution
    pub code_changes: CodeChanges,

    /// All test artifacts and execution results
    pub tests: TestArtifacts,

    /// Code coverage analysis results
    pub coverage: CoverageResults,

    /// Linting and static analysis results
    pub linting: LintingResults,

    /// Complete provenance and audit trail
    pub provenance: Provenance,

    /// Artifact storage and management metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ArtifactMetadata>,
}

impl Default for ExecutionArtifacts {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            task_id: Uuid::nil(),
            working_spec_id: String::new(),
            iteration: 0,
            code_changes: CodeChanges::default(),
            tests: TestArtifacts::default(),
            coverage: CoverageResults::default(),
            linting: LintingResults::default(),
            provenance: Provenance::default(),
            metadata: None,
        }
    }
}

/// All code changes made during execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodeChanges {
    /// Unified diffs for all code changes
    pub diffs: Vec<DiffArtifact>,

    /// Newly created files
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub new_files: Vec<NewFileArtifact>,

    /// Paths of deleted files
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub deleted_files: Vec<String>,

    /// Code change statistics
    pub statistics: CodeChangeStats,
}

impl Default for CodeChanges {
    fn default() -> Self {
        Self {
            diffs: Vec::new(),
            new_files: Vec::new(),
            deleted_files: Vec::new(),
            statistics: CodeChangeStats::default(),
        }
    }
}

/// Unified diff artifact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiffArtifact {
    /// File path
    pub file_path: String,

    /// Type of change
    pub change_type: ChangeType,

    /// Complete diff content
    pub diff_content: String,

    /// Lines added in this change
    pub lines_added: u32,

    /// Lines removed in this change
    pub lines_removed: u32,

    /// Parsed diff hunks
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hunks: Vec<DiffHunk>,
}

/// Type of code change
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Individual diff hunk
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiffHunk {
    /// Old file starting line
    pub old_start: u32,

    /// Old file line count
    pub old_lines: u32,

    /// New file starting line
    pub new_start: u32,

    /// New file line count
    pub new_lines: u32,

    /// Diff lines with context
    pub lines: Vec<String>,
}

/// Newly created file artifact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NewFileArtifact {
    /// File path
    pub path: String,

    /// File content
    pub content: String,

    /// File permissions (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
}

/// Code change statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodeChangeStats {
    /// Total files modified
    pub files_modified: u32,

    /// Total lines added
    pub lines_added: u32,

    /// Total lines removed
    pub lines_removed: u32,

    /// Total lines of code after changes
    pub total_loc: u32,
}

impl Default for CodeChangeStats {
    fn default() -> Self {
        Self {
            files_modified: 0,
            lines_added: 0,
            lines_removed: 0,
            total_loc: 0,
        }
    }
}

/// All test artifacts and execution results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestArtifacts {
    /// Unit test execution results
    pub unit_tests: TestSuiteResults,

    /// Integration test execution results
    pub integration_tests: TestSuiteResults,

    /// End-to-end test execution results
    pub e2e_tests: E2eTestResults,

    /// Test files created or modified
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub test_files: Vec<TestFileInfo>,
}

impl Default for TestArtifacts {
    fn default() -> Self {
        Self {
            unit_tests: TestSuiteResults::default(),
            integration_tests: TestSuiteResults::default(),
            e2e_tests: E2eTestResults::default(),
            test_files: Vec::new(),
        }
    }
}

/// Test suite execution results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestSuiteResults {
    /// Total tests in suite
    pub total: u32,

    /// Tests passed
    pub passed: u32,

    /// Tests failed
    pub failed: u32,

    /// Tests skipped
    pub skipped: u32,

    /// Total execution time in milliseconds
    pub duration_ms: u64,

    /// Individual test results
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub results: Vec<TestResult>,
}

impl Default for TestSuiteResults {
    fn default() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            results: Vec::new(),
        }
    }
}

/// Individual test result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestResult {
    /// Test name
    pub name: String,

    /// Test execution status
    pub status: TestStatus,

    /// Execution time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Components tested together (for integration tests)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components_tested: Vec<String>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Number of assertions executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertions: Option<u32>,
}

/// Test execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

/// End-to-end test execution results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct E2eTestResults {
    /// Total E2E tests
    pub total: u32,

    /// Tests passed
    pub passed: u32,

    /// Tests failed
    pub failed: u32,

    /// Tests skipped
    pub skipped: u32,

    /// Total execution time in milliseconds
    pub duration_ms: u64,

    /// E2E scenario results
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub scenarios: Vec<E2eScenarioResult>,
}

impl Default for E2eTestResults {
    fn default() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            scenarios: Vec::new(),
        }
    }
}

/// Individual E2E scenario result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct E2eScenarioResult {
    /// Scenario name
    pub name: String,

    /// User journey description
    pub user_journey: String,

    /// Execution status
    pub status: TestStatus,

    /// Execution time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Screenshots captured (file paths)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub screenshots: Vec<String>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Test file information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestFileInfo {
    /// Test file path
    pub path: String,

    /// Test file type
    pub r#type: TestFileType,

    /// Whether this is a new or modified file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TestFileStatus>,
}

/// Type of test file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestFileType {
    Unit,
    Integration,
    E2e,
    Contract,
}

/// Test file status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestFileStatus {
    New,
    Modified,
    Existing,
}

/// Code coverage analysis results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CoverageResults {
    /// Percentage of lines covered by tests (0.0-1.0)
    pub line_coverage: f64,

    /// Percentage of branches covered by tests (0.0-1.0)
    pub branch_coverage: f64,

    /// Percentage of functions covered by tests (0.0-1.0)
    pub function_coverage: f64,

    /// Mutation testing score (0.0-1.0)
    pub mutation_score: f64,

    /// Path to coverage report file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_report_path: Option<String>,

    /// Uncovered line information
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub uncovered_lines: Vec<UncoveredLines>,

    /// Uncovered branch information
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub uncovered_branches: Vec<UncoveredBranch>,
}

impl Default for CoverageResults {
    fn default() -> Self {
        Self {
            line_coverage: 0.0,
            branch_coverage: 0.0,
            function_coverage: 0.0,
            mutation_score: 0.0,
            coverage_report_path: None,
            uncovered_lines: Vec::new(),
            uncovered_branches: Vec::new(),
        }
    }
}

/// Uncovered lines in a file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UncoveredLines {
    /// File path
    pub file: String,

    /// Line numbers not covered
    pub lines: Vec<u32>,
}

/// Uncovered branch in code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UncoveredBranch {
    /// File path
    pub file: String,

    /// Line number with uncovered branch
    pub line: u32,

    /// Branch conditions not covered
    pub conditions: Vec<String>,
}

/// Linting and static analysis results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LintingResults {
    /// Total number of issues found
    pub total_issues: u32,

    /// Number of error-level issues
    pub errors: u32,

    /// Number of warning-level issues
    pub warnings: u32,

    /// Number of info-level issues
    pub info: u32,

    /// Issues grouped by file
    pub issues_by_file: std::collections::HashMap<String, Vec<LintingIssue>>,

    /// Linter version used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter_version: Option<String>,

    /// Configuration used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_used: Option<String>,
}

impl Default for LintingResults {
    fn default() -> Self {
        Self {
            total_issues: 0,
            errors: 0,
            warnings: 0,
            info: 0,
            issues_by_file: std::collections::HashMap::new(),
            linter_version: None,
            config_used: None,
        }
    }
}

/// Individual linting issue
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LintingIssue {
    /// Line number
    pub line: u32,

    /// Column number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,

    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue code
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// Suggested fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Complete provenance and audit trail
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Provenance {
    /// Unique execution identifier
    pub execution_id: Uuid,

    /// Worker that performed the execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<String>,

    /// Worker version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_version: Option<String>,

    /// When execution started
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// When execution completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Total execution duration in milliseconds
    pub duration_ms: u64,

    /// Execution environment details
    pub environment: ExecutionEnvironment,

    /// Git information at execution time
    pub git_info: GitInfo,

    /// Deterministic seeds used
    pub seeds_used: ExecutionSeeds,

    /// Complete audit trail of execution steps
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub audit_trail: Vec<AuditEvent>,
}

impl Default for Provenance {
    fn default() -> Self {
        Self {
            execution_id: Uuid::nil(),
            worker_id: None,
            worker_version: None,
            started_at: chrono::Utc::now(),
            completed_at: None,
            duration_ms: 0,
            environment: ExecutionEnvironment::default(),
            git_info: GitInfo::default(),
            seeds_used: ExecutionSeeds::default(),
            audit_trail: Vec::new(),
        }
    }
}

/// Execution environment details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionEnvironment {
    /// Operating system
    pub os: String,

    /// CPU architecture
    pub architecture: String,

    /// Rust compiler version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,

    /// Runtime dependencies and versions
    pub dependencies: std::collections::HashMap<String, String>,
}

impl Default for ExecutionEnvironment {
    fn default() -> Self {
        Self {
            os: "unknown".to_string(),
            architecture: "unknown".to_string(),
            rust_version: None,
            dependencies: std::collections::HashMap::new(),
        }
    }
}

/// Git repository information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GitInfo {
    /// Current commit hash
    pub commit_hash: String,

    /// Current branch
    pub branch: String,

    /// Whether working directory has uncommitted changes
    pub dirty: bool,

    /// Uncommitted changes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub uncommitted_changes: Vec<String>,
}

impl Default for GitInfo {
    fn default() -> Self {
        Self {
            commit_hash: "unknown".to_string(),
            branch: "unknown".to_string(),
            dirty: false,
            uncommitted_changes: Vec::new(),
        }
    }
}

/// Deterministic seeds used for reproducible execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionSeeds {
    /// Time seed for deterministic timestamps
    pub time_seed: String,

    /// UUID seed for deterministic ID generation
    pub uuid_seed: String,

    /// Random seed for deterministic random operations
    pub random_seed: i64,
}

/// Individual audit event in the execution trail
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuditEvent {
    /// When the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Event type or description
    pub event: String,

    /// Additional event details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Artifact storage and management metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ArtifactMetadata {
    /// Whether artifacts were compressed for storage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_applied: Option<bool>,

    /// Storage location identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_location: Option<String>,

    /// Retention policy identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_policy: Option<String>,

    /// Categorization tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

/// Validate an execution artifacts value against the JSON schema
pub fn validate_execution_artifacts_value(value: &serde_json::Value) -> Result<(), crate::error::ContractError> {
    use crate::error::{ContractError, ContractKind};
    use crate::schema::EXECUTION_ARTIFACTS_SCHEMA;

    EXECUTION_ARTIFACTS_SCHEMA.validate(value).map_err(|errors| {
        let issues = errors
            .into_iter()
            .map(|error| crate::error::ValidationIssue {
                instance_path: error.instance_path.to_string(),
                schema_path: error.schema_path.to_string(),
                message: error.to_string(),
            })
            .collect();
        ContractError::validation(ContractKind::ExecutionArtifacts, issues)
    })
}

// TODO: Add proper Default implementations after fixing struct field mismatches
