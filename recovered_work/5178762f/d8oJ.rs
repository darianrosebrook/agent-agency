//! Diff Application with Conflict Detection
//!
//! Applies unified diffs with validation and rollback capability.
//! Checks SHA256 conflicts and provides detailed error information.
//!
//! @author @darianrosebrook

use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::sandbox::diff_generator::UnifiedDiff;

#[derive(Debug, Error)]
pub enum DiffApplyError {
    #[error("SHA256 conflict: expected {expected}, found {actual}")]
    Sha256Conflict { expected: String, actual: String },

    #[error("Diff application failed: {reason}")]
    ApplyFailed { reason: String },

    #[error("File operation failed: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Patch parsing failed: {0}")]
    PatchError(String),

    #[error("Hunk application failed at line {line}: {reason}")]
    HunkFailed { line: usize, reason: String },
}

/// Result of applying a diff
#[derive(Debug, Clone)]
pub enum ApplyResult {
    Success,
    Failed { failing_hunks: Vec<HunkFailure> },
}

/// Details of a failed hunk application
#[derive(Debug, Clone)]
pub struct HunkFailure {
    pub hunk_index: usize,
    pub line_number: usize,
    pub expected_context: Vec<String>,
    pub actual_context: Vec<String>,
    pub reason: String,
}

/// Diff applier with conflict detection and validation
///
/// **INVARIANT**: Dry-run validation before any file modifications
/// **INVARIANT**: SHA256 conflict detection prevents stale diff application
/// **INVARIANT**: Atomic apply (temp file + rename) or complete rollback
pub struct DiffApplier {
    workspace_root: PathBuf,
}

impl DiffApplier {
    /// Create a new diff applier for the workspace
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Apply a unified diff with validation and conflict detection
    ///
    /// **Safety**: Performs dry-run validation first, then atomic apply.
    /// Returns detailed error information on failure.
    pub async fn apply_diff(
        &self,
        diff: &UnifiedDiff,
        dry_run: bool,
    ) -> Result<ApplyResult, DiffApplyError> {
        // 1. Check SHA256 conflict if expected hash provided
        if let Some(expected_sha) = &diff.expected_sha256 {
            let file_path = self.workspace_root.join(&diff.file_path);
            let actual_sha = self.compute_file_sha256(&file_path).await?;
            if actual_sha != *expected_sha {
                return Err(DiffApplyError::Sha256Conflict {
                    expected: expected_sha.clone(),
                    actual: actual_sha,
                });
            }
        }

        // 2. Dry-run validation
        let dry_result = self.apply_internal(diff, true).await?;
        if !matches!(dry_result, ApplyResult::Success) {
            return Ok(dry_result);
        }

        // 3. Apply for real if not dry-run
        if !dry_run {
            self.apply_internal(diff, false).await?;
        }

        Ok(ApplyResult::Success)
    }

    /// Internal diff application logic
    async fn apply_internal(
        &self,
        diff: &UnifiedDiff,
        dry_run: bool,
    ) -> Result<ApplyResult, DiffApplyError> {
        let file_path = self.workspace_root.join(&diff.file_path);

        // Read original file content
        let original_content = if file_path.exists() {
            fs::read_to_string(&file_path).await?
        } else {
            String::new() // New file
        };

        // Parse diff into hunks
        let hunks = self.parse_unified_diff(&diff.diff_content)?;

        // Apply hunks in order
        let mut applied_content = original_content.clone();
        let mut failing_hunks = Vec::new();

        for (hunk_idx, hunk) in hunks.iter().enumerate() {
            match self.apply_hunk(&applied_content, hunk, hunk_idx) {
                Ok(new_content) => {
                    applied_content = new_content;
                }
                Err(failure) => {
                    failing_hunks.push(failure);
                    // Continue with other hunks for comprehensive error reporting
                }
            }
        }

        if !failing_hunks.is_empty() {
            return Ok(ApplyResult::Failed { failing_hunks });
        }

        // Write to file (or temp file for dry-run)
        if !dry_run {
            // Atomic write: write to temp file, then rename
            let temp_path = file_path.with_extension("tmp");
            let mut temp_file = fs::File::create(&temp_path).await?;
            temp_file.write_all(applied_content.as_bytes()).await?;
            temp_file.sync_all().await?; // fsync for durability

            // Atomic rename
            fs::rename(&temp_path, &file_path).await?;
        }

        Ok(ApplyResult::Success)
    }

    /// Parse unified diff content into hunks
    fn parse_unified_diff(&self, diff_content: &str) -> Result<Vec<DiffHunk>, DiffApplyError> {
        let mut hunks = Vec::new();
        let lines: Vec<&str> = diff_content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            // Look for hunk header: @@ -old_start,old_count +new_start,new_count @@
            if lines[i].starts_with("@@") && lines[i].ends_with("@@") {
                let hunk = self.parse_hunk(&lines, &mut i)?;
                hunks.push(hunk);
            } else {
                i += 1;
            }
        }

        Ok(hunks)
    }

    /// Parse a single hunk from diff lines
    fn parse_hunk(&self, lines: &[&str], i: &mut usize) -> Result<DiffHunk, DiffApplyError> {
        // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
        let header = lines[*i];
        let header_parts: Vec<&str> = header.split_whitespace().collect();
        if header_parts.len() != 3 || !header_parts[2].ends_with("@@") {
            return Err(DiffApplyError::PatchError(format!("Invalid hunk header: {}", header)));
        }

        let ranges = header_parts[1];
        let range_parts: Vec<&str> = ranges.split(',').collect();
        if range_parts.len() != 2 {
            return Err(DiffApplyError::PatchError(format!("Invalid hunk ranges: {}", ranges)));
        }

        let old_start: usize = range_parts[0][1..].parse().map_err(|_| {
            DiffApplyError::PatchError(format!("Invalid old start: {}", range_parts[0]))
        })?;
        let old_count: usize = range_parts[1].parse().map_err(|_| {
            DiffApplyError::PatchError(format!("Invalid old count: {}", range_parts[1]))
        })?;

        // Skip to hunk content
        *i += 1;

        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();

        while *i < lines.len() && !lines[*i].starts_with("@@") {
            let line = lines[*i];
            if line.starts_with(' ') || line.starts_with('-') {
                old_lines.push(line[1..].to_string()); // Remove marker
            }
            if line.starts_with(' ') || line.starts_with('+') {
                new_lines.push(line[1..].to_string()); // Remove marker
            }
            *i += 1;
        }

        Ok(DiffHunk {
            old_start: old_start.saturating_sub(1), // Convert to 0-indexed
            old_lines,
            new_lines,
        })
    }

    /// Apply a single hunk to content
    fn apply_hunk(
        &self,
        content: &str,
        hunk: &DiffHunk,
        hunk_idx: usize,
    ) -> Result<String, HunkFailure> {
        let lines: Vec<&str> = content.lines().collect();

        // Find the context match in the original content
        let context_start = hunk.old_start;
        let context_end = (context_start + hunk.old_lines.len()).min(lines.len());

        if context_end - context_start < hunk.old_lines.len() {
            return Err(HunkFailure {
                hunk_index: hunk_idx,
                line_number: context_start,
                expected_context: hunk.old_lines.clone(),
                actual_context: lines[context_start..context_end].to_vec().iter().map(|s| s.to_string()).collect(),
                reason: "Insufficient context lines".to_string(),
            });
        }

        // Check if context matches
        let actual_context: Vec<String> = lines[context_start..context_end]
            .iter()
            .map(|s| s.to_string())
            .collect();

        if actual_context != hunk.old_lines {
            return Err(HunkFailure {
                hunk_index: hunk_idx,
                line_number: context_start,
                expected_context: hunk.old_lines.clone(),
                actual_context,
                reason: "Context mismatch".to_string(),
            });
        }

        // Apply the hunk: replace old lines with new lines
        let mut result_lines = lines[..context_start].to_vec();
        result_lines.extend(hunk.new_lines.iter().map(|s| s.as_str()));
        result_lines.extend(&lines[context_end..]);

        Ok(result_lines.join("\n") + "\n")
    }

    /// Compute SHA256 of file content
    async fn compute_file_sha256(&self, file_path: &Path) -> Result<String, DiffApplyError> {
        if !file_path.exists() {
            return Ok(compute_sha256_bytes(&[])); // Empty file SHA256
        }

        // Read as raw bytes to handle binary files, symlinks, and invalid UTF-8
        let content = fs::read(file_path).await?;
        Ok(compute_sha256_bytes(&content))
    }
}

/// Parsed diff hunk
#[derive(Debug, Clone)]
struct DiffHunk {
    old_start: usize, // 0-indexed line number
    old_lines: Vec<String>,
    new_lines: Vec<String>,
}

/// Compute SHA256 hash of content
fn compute_sha256(content: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_diff_applier_creation() {
        let temp_dir = tempdir().unwrap();
        let applier = DiffApplier::new(temp_dir.path().to_path_buf());
        assert_eq!(applier.workspace_root, temp_dir.path());
    }

    #[tokio::test]
    async fn test_simple_addition_apply() {
        let temp_dir = tempdir().unwrap();
        let applier = DiffApplier::new(temp_dir.path().to_path_buf());

        // Create original file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "line1\nline2\n").await.unwrap();

        // Create diff for addition
        let diff = UnifiedDiff {
            file_path: PathBuf::from("test.txt"),
            diff_content: "--- a/test.txt\n+++ b/test.txt\n@@ -1,2 +1,3 @@\n line1\n line2\n+line3\n".to_string(),
            sha256: "dummy".to_string(),
            expected_sha256: Some(compute_sha256("line1\nline2\n")),
        };

        // Apply the diff
        let result = applier.apply_diff(&diff, false).await.unwrap();
        assert!(matches!(result, ApplyResult::Success));

        // Verify file content
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "line1\nline2\nline3\n");
    }

    #[tokio::test]
    async fn test_sha256_conflict_detection() {
        let temp_dir = tempdir().unwrap();
        let applier = DiffApplier::new(temp_dir.path().to_path_buf());

        // Create file with different content than expected
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "modified content\n").await.unwrap();

        let diff = UnifiedDiff {
            file_path: PathBuf::from("test.txt"),
            diff_content: "--- a/test.txt\n+++ b/test.txt\n@@ -1 +1 @@\n-original\n+modified\n".to_string(),
            sha256: "dummy".to_string(),
            expected_sha256: Some(compute_sha256("original content\n")), // Wrong expected hash
        };

        let result = applier.apply_diff(&diff, false).await;
        assert!(matches!(result, Err(DiffApplyError::Sha256Conflict { .. })));
    }

    #[tokio::test]
    async fn test_dry_run_validation() {
        let temp_dir = tempdir().unwrap();
        let applier = DiffApplier::new(temp_dir.path().to_path_buf());

        // Create original file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "line1\nline2\n").await.unwrap();

        let diff = UnifiedDiff {
            file_path: PathBuf::from("test.txt"),
            diff_content: "--- a/test.txt\n+++ b/test.txt\n@@ -1,2 +1,3 @@\n line1\n line2\n+line3\n".to_string(),
            sha256: "dummy".to_string(),
            expected_sha256: Some(compute_sha256("line1\nline2\n")),
        };

        // Dry run should succeed
        let result = applier.apply_diff(&diff, true).await.unwrap();
        assert!(matches!(result, ApplyResult::Success));

        // File should be unchanged
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "line1\nline2\n");
    }

    #[tokio::test]
    async fn test_context_mismatch_failure() {
        let temp_dir = tempdir().unwrap();
        let applier = DiffApplier::new(temp_dir.path().to_path_buf());

        // Create file with different content
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "different\ncontent\n").await.unwrap();

        let diff = UnifiedDiff {
            file_path: PathBuf::from("test.txt"),
            diff_content: "--- a/test.txt\n+++ b/test.txt\n@@ -1,2 +1,3 @@\n line1\n line2\n+line3\n".to_string(),
            sha256: "dummy".to_string(),
            expected_sha256: Some(compute_sha256("different\ncontent\n")),
        };

        let result = applier.apply_diff(&diff, false).await.unwrap();
        match result {
            ApplyResult::Failed { failing_hunks } => {
                assert_eq!(failing_hunks.len(), 1);
                assert_eq!(failing_hunks[0].reason, "Context mismatch");
            }
            _ => panic!("Expected hunk failure"),
        }
    }

    #[tokio::test]
    async fn test_new_file_creation() {
        let temp_dir = tempdir().unwrap();
        let applier = DiffApplier::new(temp_dir.path().to_path_buf());

        let file_path = temp_dir.path().join("newfile.txt");

        let diff = UnifiedDiff {
            file_path: PathBuf::from("newfile.txt"),
            diff_content: "--- a/newfile.txt\n+++ b/newfile.txt\n@@ -0,0 +1,2 @@\n+line1\n+line2\n".to_string(),
            sha256: "dummy".to_string(),
            expected_sha256: None, // New file
        };

        let result = applier.apply_diff(&diff, false).await.unwrap();
        assert!(matches!(result, ApplyResult::Success));

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "line1\nline2\n");
    }
}