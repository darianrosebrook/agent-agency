use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, udiff::UnifiedDiff};
use std::collections::HashMap;

use crate::types::{Digest, PayloadKind};
use crate::types::Digest as SourceDigest;

/// Unified diff with explicit lineage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineagedDiff {
    /// Base commit ID that this diff is relative to
    pub base_commit: String,
    /// Digest of the base content
    pub base_digest: Digest,
    /// Digest of the after content
    pub after_digest: Digest,
    /// The actual unified diff content
    pub diff_content: Vec<u8>,
    /// Metadata about the diff
    pub metadata: DiffMetadata,
}

/// Metadata about a unified diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetadata {
    /// Number of lines added
    pub lines_added: usize,
    /// Number of lines removed
    pub lines_removed: usize,
    /// Number of lines modified
    pub lines_modified: usize,
    /// Number of hunks in the diff
    pub hunks_count: usize,
    /// Whether the diff is empty (no changes)
    pub is_empty: bool,
    /// File path being diffed
    pub file_path: String,
    /// Timestamp when diff was created
    pub created_at: u64,
}

/// Diff generator for creating unified diffs with lineage
pub struct DiffGenerator {
    /// Configuration for diff generation
    config: DiffConfig,
}

/// Configuration for diff generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    /// Number of context lines to include around changes
    pub context_lines: usize,
    /// Maximum diff size before falling back to full content
    pub max_diff_size: usize,
    /// Whether to include file headers in diff
    pub include_file_headers: bool,
    /// Whether to normalize line endings before diffing
    pub normalize_line_endings: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            context_lines: 3,
            max_diff_size: 1024 * 1024, // 1MB
            include_file_headers: true,
            normalize_line_endings: true,
        }
    }
}

impl DiffGenerator {
    /// Create a new diff generator with default configuration
    pub fn new() -> Self {
        Self {
            config: DiffConfig::default(),
        }
    }

    /// Create a new diff generator with custom configuration
    pub fn with_config(config: DiffConfig) -> Self {
        Self { config }
    }

    /// Generate a unified diff between two content versions
    pub fn generate_diff(
        &self,
        base_content: &[u8],
        after_content: &[u8],
        base_commit: &str,
        base_digest: Digest,
        after_digest: Digest,
        file_path: &str,
    ) -> Result<LineagedDiff> {
        // Normalize line endings if configured
        let (base_normalized, after_normalized) = if self.config.normalize_line_endings {
            (
                self.normalize_line_endings(base_content),
                self.normalize_line_endings(after_content),
            )
        } else {
            (base_content.to_vec(), after_content.to_vec())
        };

        // Generate the unified diff
        let diff_content = self.create_unified_diff(&base_normalized, &after_normalized, file_path)?;

        // Check if diff is too large
        if diff_content.len() > self.config.max_diff_size {
            return Err(anyhow!(
                "Diff size {} exceeds maximum allowed size {}",
                diff_content.len(),
                self.config.max_diff_size
            ));
        }

        // Analyze the diff to extract metadata
        let metadata = self.analyze_diff(&diff_content, file_path)?;

        Ok(LineagedDiff {
            base_commit: base_commit.to_string(),
            base_digest,
            after_digest,
            diff_content,
            metadata,
        })
    }

    /// Create a unified diff between two content versions
    fn create_unified_diff(
        &self,
        base_content: &[u8],
        after_content: &[u8],
        file_path: &str,
    ) -> Result<Vec<u8>> {
        // Convert bytes to strings for diffing
        let base_str = String::from_utf8_lossy(base_content);
        let after_str = String::from_utf8_lossy(after_content);

        // Create unified diff
        let diff = UnifiedDiff::from_strings(
            &base_str,
            &after_str,
            self.config.context_lines,
        );

        // Convert to bytes
        let mut diff_bytes = Vec::new();
        
        if self.config.include_file_headers {
            // Add file header
            diff_bytes.extend_from_slice(b"--- a/");
            diff_bytes.extend_from_slice(file_path.as_bytes());
            diff_bytes.extend_from_slice(b"\n+++ b/");
            diff_bytes.extend_from_slice(file_path.as_bytes());
            diff_bytes.extend_from_slice(b"\n");
        }

        // Add diff hunks
        for hunk in diff.hunks() {
            diff_bytes.extend_from_slice(hunk.header().as_bytes());
            diff_bytes.extend_from_slice(b"\n");
            
            for op in hunk.ops() {
                let line = match op.tag() {
                    ChangeTag::Equal => format!(" {}", op.value()),
                    ChangeTag::Delete => format!("-{}", op.value()),
                    ChangeTag::Insert => format!("+{}", op.value()),
                };
                diff_bytes.extend_from_slice(line.as_bytes());
            }
        }

        Ok(diff_bytes)
    }

    /// Analyze a diff to extract metadata
    fn analyze_diff(&self, diff_content: &[u8], file_path: &str) -> Result<DiffMetadata> {
        let diff_str = String::from_utf8_lossy(diff_content);
        let mut lines_added = 0;
        let mut lines_removed = 0;
        let mut lines_modified = 0;
        let mut hunks_count = 0;
        let mut in_hunk = false;

        for line in diff_str.lines() {
            if line.starts_with("@@") {
                hunks_count += 1;
                in_hunk = true;
                continue;
            }

            if in_hunk {
                if line.starts_with('+') && !line.starts_with("+++") {
                    lines_added += 1;
                } else if line.starts_with('-') && !line.starts_with("---") {
                    lines_removed += 1;
                } else if line.starts_with(' ') {
                    // Context line
                    continue;
                } else if line.starts_with("@@") {
                    // New hunk
                    continue;
                } else {
                    // End of hunk
                    in_hunk = false;
                }
            }
        }

        // Calculate modified lines (approximation)
        lines_modified = lines_added.min(lines_removed);

        let is_empty = lines_added == 0 && lines_removed == 0;

        Ok(DiffMetadata {
            lines_added,
            lines_removed,
            lines_modified,
            hunks_count,
            is_empty,
            file_path: file_path.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Normalize line endings to LF
    fn normalize_line_endings(&self, content: &[u8]) -> Vec<u8> {
        let mut normalized = Vec::new();
        let mut i = 0;

        while i < content.len() {
            if i + 1 < content.len() && content[i] == b'\r' && content[i + 1] == b'\n' {
                // CRLF -> LF
                normalized.push(b'\n');
                i += 2;
            } else if content[i] == b'\r' {
                // CR -> LF
                normalized.push(b'\n');
                i += 1;
            } else {
                normalized.push(content[i]);
                i += 1;
            }
        }

        normalized
    }

    /// Apply a unified diff to base content to reconstruct the after content
    pub fn apply_diff(&self, base_content: &[u8], diff: &LineagedDiff) -> Result<Vec<u8>> {
        let base_str = String::from_utf8_lossy(base_content);
        let diff_str = String::from_utf8_lossy(&diff.diff_content);

        // Parse the diff and apply changes
        let mut result_lines = Vec::new();
        let base_lines: Vec<&str> = base_str.lines().collect();
        let diff_lines: Vec<&str> = diff_str.lines().collect();

        let mut base_index = 0;
        let mut diff_index = 0;

        // Skip file headers if present
        while diff_index < diff_lines.len() && diff_lines[diff_index].starts_with("---") {
            diff_index += 1;
        }
        while diff_index < diff_lines.len() && diff_lines[diff_index].starts_with("+++") {
            diff_index += 1;
        }

        while diff_index < diff_lines.len() {
            let line = diff_lines[diff_index];
            
            if line.starts_with("@@") {
                // Parse hunk header
                let hunk_info = self.parse_hunk_header(line)?;
                diff_index += 1;

                // Process hunk
                let mut hunk_base_line = hunk_info.base_start;
                let mut hunk_after_line = hunk_info.after_start;

                while diff_index < diff_lines.len() && !diff_lines[diff_index].starts_with("@@") {
                    let diff_line = diff_lines[diff_index];
                    
                    match diff_line.chars().next() {
                        Some(' ') => {
                            // Context line - copy from base
                            if hunk_base_line <= base_lines.len() {
                                result_lines.push(base_lines[hunk_base_line - 1]);
                            }
                            hunk_base_line += 1;
                            hunk_after_line += 1;
                        }
                        Some('-') => {
                            // Deleted line - skip in base
                            hunk_base_line += 1;
                        }
                        Some('+') => {
                            // Added line - add to result
                            let added_line = &diff_line[1..];
                            result_lines.push(added_line);
                            hunk_after_line += 1;
                        }
                        _ => {
                            // End of hunk or invalid line
                            break;
                        }
                    }
                    diff_index += 1;
                }
            } else {
                diff_index += 1;
            }
        }

        // Add remaining base lines
        while base_index < base_lines.len() {
            result_lines.push(base_lines[base_index]);
            base_index += 1;
        }

        Ok(result_lines.join("\n").into_bytes())
    }

    /// Parse a hunk header to extract line numbers
    fn parse_hunk_header(&self, header: &str) -> Result<HunkInfo> {
        // Parse header like "@@ -1,3 +1,4 @@"
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow!("Invalid hunk header: {}", header));
        }

        let base_part = parts[1]; // -1,3
        let after_part = parts[2]; // +1,4

        let base_info = self.parse_line_range(base_part)?;
        let after_info = self.parse_line_range(after_part)?;

        Ok(HunkInfo {
            base_start: base_info.0,
            base_count: base_info.1,
            after_start: after_info.0,
            after_count: after_info.1,
        })
    }

    /// Parse a line range like "1,3" or "1"
    fn parse_line_range(&self, range: &str) -> Result<(usize, usize)> {
        let range = range.trim_start_matches('-').trim_start_matches('+');
        let parts: Vec<&str> = range.split(',').collect();
        
        let start = parts[0].parse::<usize>()?;
        let count = if parts.len() > 1 {
            parts[1].parse::<usize>()?
        } else {
            1
        };

        Ok((start, count))
    }
}

/// Information about a diff hunk
#[derive(Debug, Clone)]
struct HunkInfo {
    base_start: usize,
    base_count: usize,
    after_start: usize,
    after_count: usize,
}

/// Diff storage for managing unified diffs
pub struct DiffStore {
    /// Storage for diff objects
    diffs: HashMap<Digest, LineagedDiff>,
}

impl DiffStore {
    /// Create a new diff store
    pub fn new() -> Self {
        Self {
            diffs: HashMap::new(),
        }
    }

    /// Store a diff
    pub fn store_diff(&mut self, diff: LineagedDiff) -> Digest {
        let digest = self.compute_diff_digest(&diff);
        self.diffs.insert(digest, diff);
        digest
    }

    /// Retrieve a diff by digest
    pub fn get_diff(&self, digest: &Digest) -> Option<&LineagedDiff> {
        self.diffs.get(digest)
    }

    /// Check if a diff exists
    pub fn has_diff(&self, digest: &Digest) -> bool {
        self.diffs.contains_key(digest)
    }

    /// Remove a diff
    pub fn remove_diff(&mut self, digest: &Digest) -> Option<LineagedDiff> {
        self.diffs.remove(digest)
    }

    /// Compute digest for a diff
    fn compute_diff_digest(&self, diff: &LineagedDiff) -> Digest {
        // Use the after_digest as the diff digest since it represents the final state
        diff.after_digest
    }

    /// Get all stored diffs
    pub fn list_diffs(&self) -> Vec<&LineagedDiff> {
        self.diffs.values().collect()
    }

    /// Get diffs for a specific base commit
    pub fn get_diffs_for_commit(&self, base_commit: &str) -> Vec<&LineagedDiff> {
        self.diffs
            .values()
            .filter(|diff| diff.base_commit == base_commit)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_generation() {
        let generator = DiffGenerator::new();
        let base_content = b"line1\nline2\nline3\n";
        let after_content = b"line1\nline2_modified\nline3\nline4\n";
        
        let base_digest = Digest::from_bytes(&[1, 2, 3, 4]);
        let after_digest = Digest::from_bytes(&[5, 6, 7, 8]);

        let diff = generator
            .generate_diff(
                base_content,
                after_content,
                "commit123",
                base_digest,
                after_digest,
                "test.txt",
            )
            .unwrap();

        assert_eq!(diff.base_commit, "commit123");
        assert_eq!(diff.base_digest, base_digest);
        assert_eq!(diff.after_digest, after_digest);
        assert!(!diff.metadata.is_empty);
        assert!(diff.metadata.lines_added > 0);
    }

    #[test]
    fn test_diff_application() {
        let generator = DiffGenerator::new();
        let base_content = b"line1\nline2\nline3\n";
        let after_content = b"line1\nline2_modified\nline3\nline4\n";
        
        let base_digest = Digest::from_bytes(&[1, 2, 3, 4]);
        let after_digest = Digest::from_bytes(&[5, 6, 7, 8]);

        let diff = generator
            .generate_diff(
                base_content,
                after_content,
                "commit123",
                base_digest,
                after_digest,
                "test.txt",
            )
            .unwrap();

        let reconstructed = generator.apply_diff(base_content, &diff).unwrap();
        let reconstructed_str = String::from_utf8_lossy(&reconstructed);
        let expected_str = String::from_utf8_lossy(after_content);

        // Note: This test might fail due to line ending differences
        // In practice, you'd normalize both strings before comparison
        assert_eq!(reconstructed_str.trim(), expected_str.trim());
    }

    #[test]
    fn test_diff_store() {
        let mut store = DiffStore::new();
        let base_digest = Digest::from_bytes(&[1, 2, 3, 4]);
        let after_digest = Digest::from_bytes(&[5, 6, 7, 8]);

        let diff = LineagedDiff {
            base_commit: "commit123".to_string(),
            base_digest,
            after_digest,
            diff_content: b"--- a/test.txt\n+++ b/test.txt\n@@ -1,3 +1,4 @@\n line1\n-line2\n+line2_modified\n line3\n+line4\n".to_vec(),
            metadata: DiffMetadata {
                lines_added: 1,
                lines_removed: 1,
                lines_modified: 0,
                hunks_count: 1,
                is_empty: false,
                file_path: "test.txt".to_string(),
                created_at: 1234567890,
            },
        };

        let digest = store.store_diff(diff.clone());
        assert!(store.has_diff(&digest));
        assert_eq!(store.get_diff(&digest).unwrap().base_commit, "commit123");
    }
}
