//! File Operations Tool - First-class file editing with safety guards
//!
//! Provides structured, deterministic file operations for autonomous agents
//! with allow-list enforcement, budget controls, and atomic rollback capabilities.

pub mod git_workspace;
pub mod temp_workspace;

use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

/// Unique identifier for a changeset operation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeSetId(pub String);

/// A single file patch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    /// Relative path to the file
    pub path: String,
    /// Individual hunks that make up this patch
    pub hunks: Vec<Hunk>,
    /// Expected SHA256 hash of file before applying patch (optional validation)
    pub expected_prev_sha256: Option<String>,
}

/// A single hunk within a patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    /// Line number where old content starts
    pub old_start: u32,
    /// Number of lines in old content
    pub old_lines: u32,
    /// Line number where new content starts
    pub new_start: u32,
    /// Number of lines in new content
    pub new_lines: u32,
    /// The actual content lines (with +/- prefixes for diff format)
    pub lines: String,
}

/// A complete changeset containing multiple patches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    /// All patches to apply atomically
    pub patches: Vec<Patch>,
}

/// Allow-list for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowList {
    /// Glob patterns for allowed paths (e.g., ["src/**/*.rs", "tests/**/*.rs"])
    pub globs: Vec<String>,
}

/// Budget constraints for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budgets {
    /// Maximum number of files that can be modified
    pub max_files: usize,
    /// Maximum lines of code that can be changed (across all files)
    pub max_loc: usize,
}

/// Types of budget violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    TooManyFiles,
    TooManyLines,
    BlockedPath,
}

/// Severity levels for violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,      // Minor exceedance, auto-approvable
    Medium,   // Significant exceedance, requires review
    High,     // Major exceedance, requires senior approval
    Critical, // Extreme exceedance, blocked
}

/// Individual budget violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub violation_type: ViolationType,
    pub actual_value: usize,
    pub budget_limit: usize,
    pub severity: ViolationSeverity,
    pub description: String,
}

/// Waiver request for budget exceedances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverRequest {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub changeset_fingerprint: String,
    pub budget_violations: Vec<BudgetViolation>,
    pub justification_required: bool,
    pub risk_assessment: RiskLevel,
    pub auto_approved: bool,
    pub approved_by: Option<String>,
    pub approval_timestamp: Option<DateTime<Utc>>,
    pub justification: Option<String>,
}

/// Risk assessment for waiver requests
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,      // Safe to auto-approve
    Medium,   // Requires justification
    High,     // Requires explicit approval
    Critical, // Blocked regardless of approval
}

/// Workspace abstraction for safe file operations
#[async_trait::async_trait]
pub trait Workspace: Send + Sync {
    /// Get the root path of this workspace
    fn root(&self) -> &Path;

    /// Apply a changeset with validation and safety checks
    async fn apply(
        &self,
        changeset: &ChangeSet,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> Result<ChangeSetId>;

    /// Revert a previously applied changeset
    async fn revert(&self, changeset_id: &ChangeSetId) -> Result<()>;

    /// Promote workspace changes to the source location (if applicable)
    async fn promote(&self) -> Result<()>;
}

/// Errors that can occur during file operations
#[derive(Error, Debug)]
pub enum FileOpsError {
    #[error("File operation blocked: {0}")]
    Blocked(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),
}

pub type Result<T> = std::result::Result<T, FileOpsError>;

/// Validate that a changeset complies with allow-list and budgets
pub fn validate_changeset(
    changeset: &ChangeSet,
    allowlist: &AllowList,
    budgets: &Budgets,
) -> Result<()> {
    // Check file count budget
    if changeset.patches.len() > budgets.max_files {
        return Err(FileOpsError::BudgetExceeded(
            format!("Too many files: {} > {}", changeset.patches.len(), budgets.max_files)
        ));
    }

    // Check allow-list compliance
    for patch in &changeset.patches {
        if !is_path_allowed(&patch.path, allowlist) {
            return Err(FileOpsError::Blocked(
                format!("Path not allowed: {}", patch.path)
            ));
        }
    }

    // Check total LOC budget
    let total_loc: usize = changeset.patches.iter()
        .map(|p| p.hunks.iter().map(|h| h.lines.lines().count()).sum::<usize>())
        .sum();

    if total_loc > budgets.max_loc {
        return Err(FileOpsError::BudgetExceeded(
            format!("Too many lines changed: {} > {}", total_loc, budgets.max_loc)
        ));
    }

    Ok(())
}

/// Validate changeset and generate waiver request if violations found
pub fn validate_changeset_with_waiver(
    changeset: &ChangeSet,
    allowlist: &AllowList,
    budgets: &Budgets,
) -> std::result::Result<(), WaiverRequest> {
    let violations = analyze_budget_violations(changeset, allowlist, budgets);

    if violations.is_empty() {
        Ok(())
    } else {
        Err(generate_waiver_request(changeset, violations))
    }
}

/// Analyze budget violations without failing
pub fn analyze_budget_violations(
    changeset: &ChangeSet,
    allowlist: &AllowList,
    budgets: &Budgets,
) -> Vec<BudgetViolation> {
    let mut violations = Vec::new();

    // Check file count budget
    let file_count = changeset.patches.len();
    if file_count > budgets.max_files {
        let severity = calculate_severity(file_count, budgets.max_files);
        violations.push(BudgetViolation {
            violation_type: ViolationType::TooManyFiles,
            actual_value: file_count,
            budget_limit: budgets.max_files,
            severity,
            description: format!("Too many files: {} > {}", file_count, budgets.max_files),
        });
    }

    // Check allow-list compliance
    for patch in &changeset.patches {
        if !is_path_allowed(&patch.path, allowlist) {
            violations.push(BudgetViolation {
                violation_type: ViolationType::BlockedPath,
                actual_value: 1, // One blocked path
                budget_limit: 0, // No blocked paths allowed
                severity: ViolationSeverity::Critical, // Path violations are always critical
                description: format!("Path not allowed: {}", patch.path),
            });
        }
    }

    // Check total LOC budget
    let total_loc: usize = changeset.patches.iter()
        .map(|p| p.hunks.iter().map(|h| h.lines.lines().count()).sum::<usize>())
        .sum();

    if total_loc > budgets.max_loc {
        let severity = calculate_severity(total_loc, budgets.max_loc);
        violations.push(BudgetViolation {
            violation_type: ViolationType::TooManyLines,
            actual_value: total_loc,
            budget_limit: budgets.max_loc,
            severity,
            description: format!("Too many lines changed: {} > {}", total_loc, budgets.max_loc),
        });
    }

    violations
}

/// Generate waiver request for budget violations
pub fn generate_waiver_request(
    changeset: &ChangeSet,
    violations: Vec<BudgetViolation>,
) -> WaiverRequest {
    use uuid::Uuid;

    let id = Uuid::new_v4().to_string();
    let timestamp = Utc::now();

    // Generate changeset fingerprint for tracking
    let changeset_fingerprint = generate_changeset_fingerprint(changeset);

    // Assess overall risk level
    let risk_assessment = assess_overall_risk(&violations);

    // Determine if justification is required
    let justification_required = matches!(risk_assessment, RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical);

    // Auto-approve low-risk violations
    let auto_approved = matches!(risk_assessment, RiskLevel::Low);

    WaiverRequest {
        id,
        timestamp,
        changeset_fingerprint,
        budget_violations: violations,
        justification_required,
        risk_assessment,
        auto_approved,
        approved_by: None,
        approval_timestamp: None,
        justification: None,
    }
}

/// Generate fingerprint for changeset tracking
fn generate_changeset_fingerprint(changeset: &ChangeSet) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    changeset.patches.len().hash(&mut hasher);

    for patch in &changeset.patches {
        patch.path.hash(&mut hasher);
        patch.hunks.len().hash(&mut hasher);
    }

    format!("{:x}", hasher.finish())
}

/// Calculate violation severity based on exceedance ratio
fn calculate_severity(actual: usize, limit: usize) -> ViolationSeverity {
    if limit == 0 {
        return ViolationSeverity::Critical;
    }

    let ratio = actual as f64 / limit as f64;

    if ratio <= 1.2 {
        ViolationSeverity::Low
    } else if ratio <= 2.0 {
        ViolationSeverity::Medium
    } else if ratio <= 5.0 {
        ViolationSeverity::High
    } else {
        ViolationSeverity::Critical
    }
}

/// Assess overall risk level from violations
fn assess_overall_risk(violations: &[BudgetViolation]) -> RiskLevel {
    let max_severity = violations.iter()
        .map(|v| &v.severity)
        .max_by_key(|s| match s {
            ViolationSeverity::Low => 1,
            ViolationSeverity::Medium => 2,
            ViolationSeverity::High => 3,
            ViolationSeverity::Critical => 4,
        })
        .unwrap_or(&ViolationSeverity::Low);

    // Check for path violations (always critical)
    let has_path_violation = violations.iter()
        .any(|v| v.violation_type == ViolationType::BlockedPath);

    if has_path_violation {
        RiskLevel::Critical
    } else {
        match max_severity {
            ViolationSeverity::Low => RiskLevel::Low,
            ViolationSeverity::Medium => RiskLevel::Medium,
            ViolationSeverity::High => RiskLevel::High,
            ViolationSeverity::Critical => RiskLevel::Critical,
        }
    }
}

/// Apply waiver to bypass budget enforcement
pub fn apply_waiver(wr: &mut WaiverRequest, approver: &str, justification: Option<String>) -> std::result::Result<(), String> {
    if wr.approved_by.is_some() {
        return Err("Waiver request already approved".to_string());
    }

    // Validate justification if required
    if wr.justification_required && justification.is_none() {
        return Err("Justification required for this waiver".to_string());
    }

    wr.approved_by = Some(approver.to_string());
    wr.approval_timestamp = Some(Utc::now());
    wr.justification = justification;

    Ok(())
}

/// Check if a path is allowed by the allow-list
fn is_path_allowed(path: &str, allowlist: &AllowList) -> bool {
    // Simple glob matching - in production, use a proper glob library
    for glob in &allowlist.globs {
        if matches_glob_simple(path, glob) {
            println!("Path '{}' allowed by glob '{}'", path, glob);
            return true;
        } else {
            println!("Path '{}' NOT allowed by glob '{}'", path, glob);
        }
    }
    false
}

/// Simple glob matching (replace with proper glob library in production)
fn matches_glob_simple(path: &str, glob: &str) -> bool {
    // Basic implementation for common patterns
    if glob == "**/*.rs" {
        println!("  **/*.rs pattern: {} ends with .rs = {}", path, path.ends_with(".rs"));
        path.ends_with(".rs")
    } else if glob == "**/*" {
        println!("  **/* pattern: allowing everything");
        true // Allow everything
    } else if glob.contains("**") {
        let parts: Vec<&str> = glob.split("**").collect();
        println!("  ** pattern: glob='{}', parts={:?}", glob, parts);
        if parts.len() >= 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            println!("  Checking prefix='{}', suffix='{}' against path='{}'", prefix, suffix, path);

            // Check prefix match
            if !prefix.is_empty() && !path.starts_with(prefix) {
                println!("  Prefix check failed: '{}' does not start with '{}'", path, prefix);
                return false;
            }

            // Check suffix match - handle glob patterns
            if !suffix.is_empty() {
                if suffix.starts_with("*.") {
                    // Handle patterns like "*.rs" -> ends with ".rs"
                    let ext = &suffix[1..]; // Remove the "*"
                    println!("  Suffix glob pattern: checking if '{}' ends with '{}'", path, ext);
                    if !path.ends_with(ext) {
                        println!("  Suffix check failed: '{}' does not end with '{}'", path, ext);
                        return false;
                    }
                } else if !path.ends_with(suffix) {
                    println!("  Suffix literal check failed: '{}' does not end with '{}'", path, suffix);
                    return false;
                }
            }

            println!("  All checks passed for '{}'", path);
            true
        } else {
            println!("  Invalid ** pattern");
            false
        }
    } else {
        println!("  Exact match check: '{}' == '{}' = {}", path, glob, path == glob);
        path == glob
    }
}

/// Factory for creating appropriate workspace based on project type
pub struct WorkspaceFactory;

impl WorkspaceFactory {
    /// Create a workspace for the given project path, auto-detecting Git vs non-Git
    pub async fn from_path(project_path: &Path, task_id: &str) -> Result<Box<dyn Workspace>> {
        if Self::is_git_repository(project_path) {
            let workspace = git_workspace::GitWorktreeWorkspace::new(project_path, task_id).await?;
            Ok(Box::new(workspace))
        } else {
            let workspace = temp_workspace::TempMirrorWorkspace::new(project_path, task_id).await?;
            Ok(Box::new(workspace))
        }
    }

    /// Check if a path is a Git repository
    fn is_git_repository(path: &Path) -> bool {
        path.join(".git").exists()
    }
}

// Re-export workspace types for convenience
pub use git_workspace::GitWorktreeWorkspace;
pub use temp_workspace::TempMirrorWorkspace;

// Waiver system types are already public in the module

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_changeset_empty() {
        let changeset = ChangeSet { patches: vec![] };
        let allowlist = AllowList { globs: vec!["**/*.rs".to_string()] };
        let budgets = Budgets { max_files: 10, max_loc: 100 };

        assert!(validate_changeset(&changeset, &allowlist, &budgets).is_ok());
    }

    #[test]
    fn test_budget_violation_analysis() {
        let changeset = ChangeSet {
            patches: vec![
                Patch {
                    path: "src/main.rs".to_string(),
                    hunks: vec![Hunk {
                        old_start: 1,
                        old_lines: 0,
                        new_start: 1,
                        new_lines: 50,
                        lines: "+".repeat(50),
                    }],
                    expected_prev_sha256: None,
                },
                Patch {
                    path: "src/utils.rs".to_string(),
                    hunks: vec![Hunk {
                        old_start: 1,
                        old_lines: 0,
                        new_start: 1,
                        new_lines: 25,
                        lines: "+".repeat(25),
                    }],
                    expected_prev_sha256: None,
                },
            ],
        };

        let allowlist = AllowList {
            globs: vec!["src/**/*.rs".to_string()],
        };

        // Budget that allows 1 file, 30 LOC total
        let budgets = Budgets {
            max_files: 1,
            max_loc: 30,
        };

        let violations = analyze_budget_violations(&changeset, &allowlist, &budgets);

        assert_eq!(violations.len(), 2); // Too many files AND too many LOC

        // Check file count violation
        let file_violation = violations.iter().find(|v| v.violation_type == ViolationType::TooManyFiles).unwrap();
        assert_eq!(file_violation.actual_value, 2);
        assert_eq!(file_violation.budget_limit, 1);

        // Check LOC violation
        let loc_violation = violations.iter().find(|v| v.violation_type == ViolationType::TooManyLines).unwrap();
        assert_eq!(loc_violation.actual_value, 75); // 50 + 25
        assert_eq!(loc_violation.budget_limit, 30);
    }

    #[test]
    fn test_waiver_request_generation() {
        let changeset = ChangeSet {
            patches: vec![Patch {
                path: "src/main.rs".to_string(),
                hunks: vec![Hunk {
                    old_start: 1,
                    old_lines: 0,
                    new_start: 1,
                    new_lines: 100,
                    lines: "+".repeat(100),
                }],
                expected_prev_sha256: None,
            }],
        };

        let violations = vec![BudgetViolation {
            violation_type: ViolationType::TooManyLines,
            actual_value: 100,
            budget_limit: 50,
            severity: ViolationSeverity::Medium,
            description: "Too many lines changed".to_string(),
        }];

        let waiver = generate_waiver_request(&changeset, violations);

        assert!(!waiver.id.is_empty());
        assert_eq!(waiver.budget_violations.len(), 1);
        assert_eq!(waiver.risk_assessment, RiskLevel::Medium);
        assert!(waiver.justification_required);
        assert!(!waiver.auto_approved);
    }

    #[test]
    fn test_validate_changeset_with_waiver() {
        let changeset = ChangeSet {
            patches: vec![Patch {
                path: "src/main.rs".to_string(),
                hunks: vec![Hunk {
                    old_start: 1,
                    old_lines: 0,
                    new_start: 1,
                    new_lines: 100,
                    lines: "+".repeat(100),
                }],
                expected_prev_sha256: None,
            }],
        };

        let allowlist = AllowList {
            globs: vec!["src/**/*.rs".to_string()],
        };

        let budgets = Budgets {
            max_files: 10,
            max_loc: 50, // Will exceed this
        };

        let result = validate_changeset_with_waiver(&changeset, &allowlist, &budgets);

        // Should return waiver request due to LOC violation
        assert!(result.is_err());
        let waiver = result.unwrap_err();
        println!("Found {} violations:", waiver.budget_violations.len());
        for violation in &waiver.budget_violations {
            println!("  - {:?}: {}", violation.violation_type, violation.description);
        }
        assert_eq!(waiver.budget_violations.len(), 1);
        assert_eq!(waiver.budget_violations[0].violation_type, ViolationType::TooManyLines);
    }

    #[test]
    fn test_waiver_application() {
        let violations = vec![BudgetViolation {
            violation_type: ViolationType::TooManyLines,
            actual_value: 100,
            budget_limit: 50,
            severity: ViolationSeverity::Medium,
            description: "Too many lines".to_string(),
        }];

        let changeset = ChangeSet { patches: vec![] };
        let mut waiver = generate_waiver_request(&changeset, violations);

        // Apply waiver with justification
        let result = apply_waiver(&mut waiver, "test-user", Some("This change is necessary for feature X".to_string()));
        assert!(result.is_ok());

        assert_eq!(waiver.approved_by, Some("test-user".to_string()));
        assert!(waiver.approval_timestamp.is_some());
        assert_eq!(waiver.justification, Some("This change is necessary for feature X".to_string()));
    }

    #[test]
    fn test_critical_path_violation() {
        let changeset = ChangeSet {
            patches: vec![Patch {
                path: "blocked/config.yml".to_string(),
                hunks: vec![Hunk {
                    old_start: 1,
                    old_lines: 0,
                    new_start: 1,
                    new_lines: 1,
                    lines: "+key: value".to_string(),
                }],
                expected_prev_sha256: None,
            }],
        };

        let allowlist = AllowList {
            globs: vec!["src/**/*.rs".to_string()], // Doesn't allow config.yml
        };

        let budgets = Budgets {
            max_files: 10,
            max_loc: 1000,
        };

        let violations = analyze_budget_violations(&changeset, &allowlist, &budgets);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation_type, ViolationType::BlockedPath);
        assert_eq!(violations[0].severity, ViolationSeverity::Critical);

        let risk = assess_overall_risk(&violations);
        assert_eq!(risk, RiskLevel::Critical);
    }

    #[test]
    fn test_severity_calculation() {
        // Test low severity (minor exceedance)
        let severity = calculate_severity(60, 50); // 20% over
        assert_eq!(severity, ViolationSeverity::Low);

        // Test medium severity (100% over)
        let severity = calculate_severity(100, 50);
        assert_eq!(severity, ViolationSeverity::Medium);

        // Test high severity (300% over)
        let severity = calculate_severity(200, 50);
        assert_eq!(severity, ViolationSeverity::High);

        // Test critical severity (extreme)
        let severity = calculate_severity(300, 50);
        assert_eq!(severity, ViolationSeverity::Critical);
    }

    #[test]
    fn test_validate_changeset_budget_exceeded() {
        let patch = Patch {
            path: "src/main.rs".to_string(),
            hunks: vec![Hunk {
                old_start: 1,
                old_lines: 1,
                new_start: 1,
                new_lines: 1,
                lines: "+new line\n".to_string(),
            }],
            expected_prev_sha256: None,
        };
        let changeset = ChangeSet { patches: vec![patch] };
        let allowlist = AllowList { globs: vec!["**/*.rs".to_string()] };
        let budgets = Budgets { max_files: 0, max_loc: 100 }; // No files allowed

        assert!(matches!(
            validate_changeset(&changeset, &allowlist, &budgets),
            Err(FileOpsError::BudgetExceeded(_))
        ));
    }

    #[test]
    fn test_validate_changeset_path_blocked() {
        let patch = Patch {
            path: "blocked.txt".to_string(),
            hunks: vec![],
            expected_prev_sha256: None,
        };
        let changeset = ChangeSet { patches: vec![patch] };
        let allowlist = AllowList { globs: vec!["**/*.rs".to_string()] };
        let budgets = Budgets { max_files: 10, max_loc: 100 };

        assert!(matches!(
            validate_changeset(&changeset, &allowlist, &budgets),
            Err(FileOpsError::Blocked(_))
        ));
    }

    #[test]
    fn test_simple_glob_matching() {
        assert!(matches_glob_simple("src/main.rs", "**/*.rs"));
        assert!(matches_glob_simple("src/main.rs", "src/main.rs"));
        assert!(!matches_glob_simple("src/main.txt", "**/*.rs"));
    }
}
