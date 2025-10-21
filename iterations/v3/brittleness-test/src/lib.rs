//! Test our brittleness fixes in isolation

use std::fs;
use std::path::Path;

/// SHA256 computation that handles binary files (brittleness fix #1)
pub fn compute_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    if !path.exists() {
        return Ok(compute_sha256_bytes(&[])); // Empty file SHA256
    }

    // Read as raw bytes to handle binary files, symlinks, and invalid UTF-8
    let content = fs::read(path)?;
    Ok(compute_sha256_bytes(&content))
}

/// Compute SHA256 hash of content bytes
fn compute_sha256_bytes(content: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Budget limits for file changes
#[derive(Debug, Clone)]
pub struct BudgetLimits {
    pub max_files: usize,
    pub max_loc: usize,
}

/// Current budget state
#[derive(Debug, Clone)]
pub struct BudgetState {
    pub files_used: usize,
    pub loc_used: i64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Budget checker that calculates LOC deltas correctly (brittleness fix #2)
#[derive(Debug, Clone)]
pub struct BudgetChecker {
    limits: BudgetLimits,
    current: BudgetState,
}

impl BudgetChecker {
    pub fn new(max_files: usize, max_loc: usize) -> Self {
        Self {
            limits: BudgetLimits { max_files, max_loc },
            current: BudgetState {
                files_used: 0,
                loc_used: 0,
                last_updated: chrono::Utc::now(),
            },
        }
    }

    pub fn limits(&self) -> &BudgetLimits {
        &self.limits
    }

    /// Calculate projected state after applying changes
    pub fn projected_state(&self, changes: &[FileChange]) -> BudgetState {
        let mut projected_files = self.current.files_used;
        let mut projected_loc = self.current.loc_used;

        for change in changes {
            match change {
                FileChange::Create { content } => {
                    projected_files += 1;
                    projected_loc += content.lines().count() as i64;
                }
                FileChange::Modify { old_content, new_content } => {
                    // Calculate LOC delta (brittleness fix: don't add full new content)
                    let old_lines = old_content.lines().count() as i64;
                    let new_lines = new_content.lines().count() as i64;
                    projected_loc += new_lines - old_lines; // Actual delta
                }
                FileChange::Delete { .. } => {
                    // Conservative: don't reduce LOC count
                    projected_files = projected_files.saturating_sub(1);
                }
            }
        }

        BudgetState {
            files_used: projected_files,
            loc_used: projected_loc,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// File change operations
#[derive(Debug, Clone)]
pub enum FileChange {
    Create { content: String },
    Modify { old_content: String, new_content: String },
    Delete { old_content: String },
}

/// Diff hunk for parsing
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<String>,
}

/// Diff applier with robust validation (brittleness fix #3)
pub struct DiffApplier;

impl DiffApplier {
    pub fn new() -> Self {
        Self
    }

    /// Parse unified diff with comprehensive validation
    pub fn parse_unified_diff(&self, diff_content: &str) -> Result<Vec<DiffHunk>, DiffError> {
        // Basic validation: ensure we have content
        if diff_content.trim().is_empty() {
            return Err(DiffError::Validation("Empty diff content".to_string()));
        }

        let mut hunks = Vec::new();
        let lines: Vec<&str> = diff_content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            // Look for hunk header: @@ -old_start,old_count +new_start,new_count @@
            if lines[i].starts_with("@@") && lines[i].ends_with("@@") {
                let hunk = self.parse_hunk(&lines, &mut i)?;
                hunks.push(hunk);
            } else if lines[i].trim().is_empty() {
                // Skip empty lines
                i += 1;
            } else if lines[i].starts_with("diff --git") ||
                      lines[i].starts_with("index ") ||
                      lines[i].starts_with("---") ||
                      lines[i].starts_with("+++") {
                // Skip standard diff metadata lines
                i += 1;
            } else {
                // Unexpected line - could be corrupted diff
                return Err(DiffError::Validation(
                    format!("Unexpected line in diff at position {}: {}", i, lines[i])
                ));
            }
        }

        // Validate that we found at least one hunk
        if hunks.is_empty() {
            return Err(DiffError::Validation("No valid hunks found in diff".to_string()));
        }

        Ok(hunks)
    }

    /// Parse a single hunk with robust validation
    fn parse_hunk(&self, lines: &[&str], i: &mut usize) -> Result<DiffHunk, DiffError> {
        let header = lines[*i];
        let header_parts: Vec<&str> = header.split_whitespace().collect();

        if header_parts.len() != 3 || !header_parts[2].ends_with("@@") {
            return Err(DiffError::Validation(format!("Invalid hunk header: {}", header)));
        }

        let ranges = header_parts[1];
        if !ranges.starts_with('-') {
            return Err(DiffError::Validation(format!("Invalid range format, expected '-' prefix: {}", ranges)));
        }

        let range_parts: Vec<&str> = ranges[1..].split(',').collect();
        if range_parts.len() != 2 {
            return Err(DiffError::Validation(format!("Invalid hunk ranges format: {}", ranges)));
        }

        let old_range = range_parts[0];
        let old_start: usize = old_range.parse().map_err(|_| {
            DiffError::Validation(format!("Invalid old start line number: {}", old_range))
        })?;

        let old_count: usize = range_parts[1].parse().map_err(|_| {
            DiffError::Validation(format!("Invalid old line count: {}", range_parts[1]))
        })?;

        // Parse new ranges
        let new_ranges = header_parts[2].trim_end_matches("@@");
        if !new_ranges.starts_with('+') {
            return Err(DiffError::Validation(format!("Invalid new range format, expected '+' prefix: {}", new_ranges)));
        }

        let new_range_parts: Vec<&str> = new_ranges[1..].split(',').collect();
        if new_range_parts.len() != 2 {
            return Err(DiffError::Validation(format!("Invalid new hunk ranges: {}", new_ranges)));
        }

        let _: usize = new_range_parts[0].parse().map_err(|_| {
            DiffError::Validation(format!("Invalid new start line number: {}", new_range_parts[0]))
        })?;
        let _: usize = new_range_parts[1].parse().map_err(|_| {
            DiffError::Validation(format!("Invalid new line count: {}", new_range_parts[1]))
        })?;

        // Skip to hunk content
        *i += 1;

        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();

        while *i < lines.len() && !lines[*i].starts_with("@@") {
            let line = lines[*i];
            if line.starts_with(' ') || line.starts_with('-') {
                old_lines.push(line[1..].to_string());
            }
            if line.starts_with(' ') || line.starts_with('+') {
                new_lines.push(line[1..].to_string());
            }
            *i += 1;
        }

        Ok(DiffHunk {
            old_start,
            old_lines: old_lines.len(),
            new_start: old_start, // Simplified for test
            new_lines: new_lines.len(),
            lines: [old_lines, new_lines].concat(),
        })
    }
}

/// Diff parsing errors
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error("Validation error: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Test SHA256 computation handles binary files
    #[test]
    fn test_sha256_binary_files() {
        let temp_dir = tempdir().unwrap();
        let binary_file = temp_dir.path().join("binary.dat");

        // Write binary data that would break UTF-8 parsing
        let binary_data = vec![0, 159, 146, 150, 255, 0, 1, 2, 3];
        fs::write(&binary_file, &binary_data).unwrap();

        let result = compute_file_sha256(&binary_file);
        assert!(result.is_ok(), "SHA256 should handle binary files");
        assert_eq!(result.unwrap().len(), 64, "Should produce 64-char SHA256 hash");
    }

    /// Test budget checker LOC calculation accuracy
    #[test]
    fn test_budget_loc_calculation_accuracy() {
        let checker = BudgetChecker::new(10, 1000);

        // Test file modification calculates delta correctly
        let changes = vec![FileChange::Modify {
            old_content: "line1\nline2\nline3\n".to_string(), // 3 lines
            new_content: "line1\nmodified\nline3\nline4\n".to_string(), // 4 lines
        }];

        let projected = checker.projected_state(&changes);
        assert_eq!(projected.files_used, 0, "No new files created");
        assert_eq!(projected.loc_used, 1, "Should count +1 LOC delta (4-3)");
    }

    /// Test diff validation prevents crashes on malformed input
    #[test]
    fn test_diff_validation_robustness() {
        let applier = DiffApplier::new();

        // Test empty diff
        let result = applier.parse_unified_diff("");
        assert!(result.is_err(), "Empty diff should be rejected");

        // Test invalid header
        let invalid = "@@ invalid header @@\n+content";
        let result = applier.parse_unified_diff(invalid);
        assert!(result.is_err(), "Invalid header should be rejected");

        // Test valid diff
        let valid = "@@ -1,3 +1,4 @@\n context\n-context2\n+change1\n+change2";
        let result = applier.parse_unified_diff(valid);
        assert!(result.is_ok(), "Valid diff should be accepted");
        assert_eq!(result.unwrap().len(), 1, "Should parse one hunk");
    }
}
