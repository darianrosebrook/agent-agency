//! Filesystem traversal and content helpers
//!
//! This module provides utilities for walking directories and reading content.

use walkdir::WalkDir;
use std::path::Path;

/// Filesystem utilities
pub struct FsUtils;

impl FsUtils {
    /// Walk directory and collect files
    pub fn walk_directory<P: AsRef<Path>>(path: P, extensions: &[&str]) -> Vec<std::path::PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                if let Some(ext) = e.path().extension() {
                    extensions.iter().any(|&allowed| ext == allowed)
                } else {
                    false
                }
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Read file content safely
    pub fn read_file_content<P: AsRef<Path>>(path: P) -> Result<String> {
        std::fs::read_to_string(path)
    }

    /// Search for keyword in file content
    pub fn search_in_file<P: AsRef<Path>>(path: P, keyword: &str) -> Result<Vec<(usize, String)>> {
        let content = Self::read_file_content(path)?;
        let mut results = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&keyword.to_lowercase()) {
                results.push((line_num + 1, line.to_string()));
            }
        }

        Ok(results)
    }
}
