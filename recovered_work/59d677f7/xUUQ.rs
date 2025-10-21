//! File Operations Tool - First-class file editing with safety guards
//!
//! Provides structured, deterministic file operations for autonomous agents
//! with allow-list enforcement, budget controls, and atomic rollback capabilities.

use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// Check if a path is allowed by the allow-list
fn is_path_allowed(path: &str, allowlist: &AllowList) -> bool {
    // Simple glob matching - in production, use a proper glob library
    for glob in &allowlist.globs {
        if matches_glob_simple(path, glob) {
            return true;
        }
    }
    false
}

/// Simple glob matching (replace with proper glob library in production)
fn matches_glob_simple(path: &str, glob: &str) -> bool {
    // Very basic implementation - expand this with proper glob matching
    if glob.contains("**") {
        let prefix = glob.split("**").next().unwrap_or("");
        let suffix = glob.split("**").nth(1).unwrap_or("");
        path.starts_with(prefix) && path.ends_with(suffix)
    } else {
        path == glob
    }
}

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
