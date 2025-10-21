//! Unified diff application for file modifications

use std::path::{Path, PathBuf};
use tokio::fs;
use diffy::Patch;

use crate::types::UnifiedDiff;

/// Applies unified diffs to files safely
pub struct DiffApplier;

impl DiffApplier {
    /// Create a new diff applier
    pub fn new() -> Self {
        Self
    }

    /// Apply a unified diff to the workspace
    pub async fn apply_diff(
        &self,
        diff: &UnifiedDiff,
        workspace_root: &Path,
    ) -> Result<Vec<String>, DiffApplyError> {
        let file_path = workspace_root.join(&diff.file_path);

        // Read the current file content
        let current_content = match fs::read_to_string(&file_path).await {
            Ok(content) => content,
            Err(_) => String::new(), // File doesn't exist, treat as empty
        };

        // Create the diff string
        let diff_text = self.diff_to_string(diff);

        // Apply the patch
        let patch = Patch::from_str(&diff_text)
            .map_err(|e| DiffApplyError::InvalidDiff(e.to_string()))?;

        let new_content = diffy::apply(&current_content, &patch)
            .map_err(|e| DiffApplyError::ApplyFailed(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| DiffApplyError::IoError(e))?;
        }

        // Write the new content
        fs::write(&file_path, &new_content).await
            .map_err(|e| DiffApplyError::IoError(e))?;

        Ok(vec![diff.file_path.clone()])
    }

    /// Convert UnifiedDiff to diff string format
    fn diff_to_string(&self, diff: &UnifiedDiff) -> String {
        let mut result = format!("--- a/{}\n+++ b/{}\n", diff.file_path, diff.file_path);

        for hunk in &diff.hunks {
            result.push_str(&format!("@@ -{},{} +{},{} @@\n",
                hunk.old_start, hunk.old_lines,
                hunk.new_start, hunk.new_lines));

            for line in &hunk.lines {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Validate a diff before applying
    pub fn validate_diff(&self, diff: &UnifiedDiff) -> Result<(), DiffApplyError> {
        if diff.file_path.is_empty() {
            return Err(DiffApplyError::InvalidDiff("Empty file path".to_string()));
        }

        if diff.hunks.is_empty() {
            return Err(DiffApplyError::InvalidDiff("No hunks in diff".to_string()));
        }

        // Check for potentially dangerous patterns
        for hunk in &diff.hunks {
            for line in &hunk.lines {
                // Prevent deletion of entire files
                if line.starts_with('-') && line.len() > 1000 {
                    return Err(DiffApplyError::DangerousDiff("Large deletion detected".to_string()));
                }
            }
        }

        Ok(())
    }
}

/// Errors during diff application
#[derive(Debug, thiserror::Error)]
pub enum DiffApplyError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid diff: {0}")]
    InvalidDiff(String),

    #[error("Failed to apply diff: {0}")]
    ApplyFailed(String),

    #[error("Dangerous diff detected: {0}")]
    DangerousDiff(String),
}
