//! Deterministic Diff Generation
//!
//! Generates unified diffs from original content vs proposed content.
//! Uses fixed formatting to ensure byte-identical outputs for same inputs.
//!
//! @author @darianrosebrook

use std::path::Path;
use thiserror::Error;
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Error)]
pub enum DiffError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Diff generation failed: {0}")]
    GenerationError(String),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

/// Unified diff representation with deterministic formatting
#[derive(Debug, Clone, PartialEq)]
pub struct UnifiedDiff {
    pub file_path: std::path::PathBuf,
    pub diff_content: String,
    pub sha256: String,
    pub expected_sha256: Option<String>, // For modify/delete operations
}

/// Deterministic diff generator
///
/// **INVARIANT**: Same inputs produce byte-identical diff content
/// **INVARIANT**: Uses fixed formatting options (no timestamps, stable headers)
/// **INVARIANT**: LF line endings regardless of platform
pub struct DiffGenerator {
    context_lines: usize,
}

impl Default for DiffGenerator {
    fn default() -> Self {
        Self {
            context_lines: 3, // Standard unified diff context
        }
    }
}

impl DiffGenerator {
    /// Create a new diff generator with custom context lines
    pub fn new(context_lines: usize) -> Self {
        Self { context_lines }
    }

    /// Generate unified diff from original vs modified content
    ///
    /// **Determinism**: Uses fixed formatting to ensure identical outputs
    /// for identical inputs across runs and platforms.
    pub fn generate_diff(
        &self,
        original: Option<&str>,
        modified: &str,
        file_path: &Path,
    ) -> Result<UnifiedDiff, DiffError> {
        // Validate file path
        if file_path.is_absolute() {
            return Err(DiffError::InvalidPath("Absolute paths not allowed".to_string()));
        }

        if file_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(DiffError::InvalidPath("Parent directory (..) not allowed".to_string()));
        }

        // Normalize line endings to LF for cross-platform consistency
        let original_normalized = original.map(|s| normalize_line_endings(s));
        let modified_normalized = normalize_line_endings(modified);

        // Generate diff using similar crate with deterministic settings
        let diff = TextDiff::from_lines(
            original_normalized.as_deref().unwrap_or(""),
            &modified_normalized,
        );

        // Format as unified diff with stable headers
        let diff_content = self.format_unified_diff(&diff, file_path)?;

        // Compute SHA256 of the diff content for integrity
        let sha256 = compute_sha256(&diff_content);

        Ok(UnifiedDiff {
            file_path: file_path.to_path_buf(),
            diff_content,
            sha256,
            expected_sha256: original.map(|_| compute_sha256(&original_normalized.unwrap())),
        })
    }

    /// Format diff as unified diff with deterministic headers
    fn format_unified_diff(
        &self,
        diff: &TextDiff,
        file_path: &Path,
    ) -> Result<String, DiffError> {
        let mut output = String::new();

        // Unified diff header (deterministic, no timestamps)
        let old_file = format!("a/{}", file_path.display());
        let new_file = format!("b/{}", file_path.display());

        output.push_str(&format!("--- {}\n", old_file));
        output.push_str(&format!("+++ {}\n", new_file));

        // Process hunks
        let mut current_hunk_start = None;
        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    // Context line
                    if current_hunk_start.is_none() {
                        // Start a new hunk if we have enough context
                        if old_lines.len() >= self.context_lines {
                            current_hunk_start = Some(change.old_index().unwrap_or(0));
                        }
                    }

                    if current_hunk_start.is_some() {
                        old_lines.push(change.to_string());
                        new_lines.push(change.to_string());
                    }
                }
                ChangeTag::Delete => {
                    // Start hunk if not already started
                    if current_hunk_start.is_none() {
                        current_hunk_start = Some(change.old_index().unwrap_or(0).saturating_sub(old_lines.len()));
                    }
                    old_lines.push(change.to_string());
                }
                ChangeTag::Insert => {
                    // Start hunk if not already started
                    if current_hunk_start.is_none() {
                        current_hunk_start = Some(change.old_index().unwrap_or(0).saturating_sub(old_lines.len()));
                    }
                    new_lines.push(change.to_string());
                }
            }

            // Check if we should flush the hunk
            let should_flush = {
                let has_changes = old_lines.len() != new_lines.len() ||
                                old_lines.iter().zip(&new_lines).any(|(o, n)| o != n);
                let context_full = old_lines.len() >= self.context_lines * 2;

                has_changes && (context_full || change.tag() != ChangeTag::Equal)
            };

            if should_flush && current_hunk_start.is_some() {
                self.flush_hunk(&mut output, current_hunk_start.unwrap(), &old_lines, &new_lines)?;
                current_hunk_start = None;
                old_lines.clear();
                new_lines.clear();
            }
        }

        // Flush any remaining hunk
        if current_hunk_start.is_some() && (!old_lines.is_empty() || !new_lines.is_empty()) {
            self.flush_hunk(&mut output, current_hunk_start.unwrap(), &old_lines, &new_lines)?;
        }

        Ok(output)
    }

    /// Flush a hunk to the output
    fn flush_hunk(
        &self,
        output: &mut String,
        hunk_start: usize,
        old_lines: &[String],
        new_lines: &[String],
    ) -> Result<(), DiffError> {
        // Count non-context lines for hunk header
        let old_count = old_lines.len();
        let new_count = new_lines.len();

        // Hunk header: @@ -old_start,old_count +new_start,new_count @@
        let old_start = hunk_start + 1; // 1-indexed
        let new_start = hunk_start + 1; // 1-indexed for equal lines

        output.push_str(&format!("@@ -{},{} +{},{} @@\n",
                                old_start, old_count, new_start, new_count));

        // Output lines with unified diff markers
        for line in old_lines {
            if line.starts_with('-') || line.starts_with('+') {
                output.push_str(line);
            } else {
                output.push_str(&format!(" {}", line));
            }
        }

        for line in new_lines {
            if line.starts_with('-') || line.starts_with('+') {
                output.push_str(line);
            } else {
                output.push_str(&format!("+{}", line));
            }
        }

        Ok(())
    }
}

/// Normalize line endings to LF for cross-platform consistency
fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
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
    use std::path::PathBuf;

    #[test]
    fn test_diff_generator_creation() {
        let generator = DiffGenerator::default();
        assert_eq!(generator.context_lines, 3);

        let generator = DiffGenerator::new(5);
        assert_eq!(generator.context_lines, 5);
    }

    #[test]
    fn test_simple_addition_diff() {
        let generator = DiffGenerator::default();
        let original = "line1\nline2\n";
        let modified = "line1\nline2\nline3\n";

        let diff = generator.generate_diff(
            Some(original),
            modified,
            Path::new("test.txt"),
        ).unwrap();

        // Should contain addition marker
        assert!(diff.diff_content.contains("+line3"));
        assert!(diff.diff_content.contains("@@"));
        assert!(diff.diff_content.contains("--- a/test.txt"));
        assert!(diff.diff_content.contains("+++ b/test.txt"));
    }

    #[test]
    fn test_simple_deletion_diff() {
        let generator = DiffGenerator::default();
        let original = "line1\nline2\nline3\n";
        let modified = "line1\nline3\n";

        let diff = generator.generate_diff(
            Some(original),
            modified,
            Path::new("test.txt"),
        ).unwrap();

        // Should contain deletion marker
        assert!(diff.diff_content.contains("-line2"));
    }

    #[test]
    fn test_new_file_diff() {
        let generator = DiffGenerator::default();
        let content = "line1\nline2\nline3\n";

        let diff = generator.generate_diff(
            None,
            content,
            Path::new("newfile.txt"),
        ).unwrap();

        // New file should show all lines as additions
        assert!(diff.diff_content.contains("+line1"));
        assert!(diff.diff_content.contains("+line2"));
        assert!(diff.diff_content.contains("+line3"));
    }

    #[test]
    fn test_deterministic_output() {
        let generator = DiffGenerator::default();
        let original = "line1\nline2\n";
        let modified = "line1\nline2\nline3\n";

        let diff1 = generator.generate_diff(
            Some(original),
            modified,
            Path::new("test.txt"),
        ).unwrap();

        let diff2 = generator.generate_diff(
            Some(original),
            modified,
            Path::new("test.txt"),
        ).unwrap();

        // Same inputs should produce identical diff content
        assert_eq!(diff1.diff_content, diff2.diff_content);
        assert_eq!(diff1.sha256, diff2.sha256);
    }

    #[test]
    fn test_invalid_path_absolute() {
        let generator = DiffGenerator::default();

        let result = generator.generate_diff(
            Some("content"),
            "modified",
            Path::new("/absolute/path"),
        );

        assert!(matches!(result, Err(DiffError::InvalidPath(_))));
    }

    #[test]
    fn test_invalid_path_parent_dir() {
        let generator = DiffGenerator::default();

        let result = generator.generate_diff(
            Some("content"),
            "modified",
            Path::new("../escape.txt"),
        );

        assert!(matches!(result, Err(DiffError::InvalidPath(_))));
    }

    #[test]
    fn test_line_ending_normalization() {
        let generator = DiffGenerator::default();
        let original = "line1\r\nline2\r\n"; // CRLF
        let modified = "line1\nline2\nline3\n"; // LF

        let diff = generator.generate_diff(
            Some(original),
            modified,
            Path::new("test.txt"),
        ).unwrap();

        // Should normalize both to LF and show the addition
        assert!(diff.diff_content.contains("+line3"));
    }

    #[test]
    fn test_sha256_computation() {
        let generator = DiffGenerator::default();
        let original = "unchanged\n";
        let modified = "unchanged\nmodified\n";

        let diff = generator.generate_diff(
            Some(original),
            modified,
            Path::new("test.txt"),
        ).unwrap();

        // SHA256 should be computed
        assert!(!diff.sha256.is_empty());
        assert_eq!(diff.sha256.len(), 64); // SHA256 hex length

        // Expected SHA256 should be computed for original
        assert!(diff.expected_sha256.is_some());
    }
}
