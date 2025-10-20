//! File path guard to enforce allow-list restrictions

use std::path::{Path, PathBuf};

/// Guards against unauthorized file access
pub struct FileGuard {
    allowed_paths: Vec<PathBuf>,
    allowed_patterns: Vec<String>,
}

impl FileGuard {
    /// Create a new file guard with allowed paths
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        let allowed_patterns = allowed_paths.iter()
            .filter_map(|p| p.to_str())
            .map(|s| s.to_string())
            .collect();

        Self {
            allowed_paths,
            allowed_patterns,
        }
    }

    /// Check if a path is allowed
    pub fn is_allowed(&self, path: &Path) -> bool {
        // Normalize the path
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Check against allowed paths
        for allowed in &self.allowed_paths {
            let canonical_allowed = match allowed.canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Check if the path is within the allowed directory
            if canonical_path.starts_with(&canonical_allowed) {
                return true;
            }

            // Check exact file match
            if canonical_path == canonical_allowed {
                return true;
            }
        }

        false
    }

    /// Validate a path and return error if not allowed
    pub fn validate_path(&self, path: &str) -> Result<(), FileGuardError> {
        let path_buf = PathBuf::from(path);
        if !self.is_allowed(&path_buf) {
            return Err(FileGuardError::PathNotAllowed(path.to_string()));
        }
        Ok(())
    }

    /// Get allowed paths
    pub fn allowed_paths(&self) -> &[PathBuf] {
        &self.allowed_paths
    }

    /// Add an allowed path
    pub fn add_allowed_path(&mut self, path: PathBuf) {
        if let Some(path_str) = path.to_str() {
            self.allowed_patterns.push(path_str.to_string());
        }
        self.allowed_paths.push(path);
    }

    /// Remove an allowed path
    pub fn remove_allowed_path(&mut self, path: &Path) {
        self.allowed_paths.retain(|p| p != path);
        self.allowed_patterns.retain(|p| p != path.to_str().unwrap_or(""));
    }
}

/// Errors from file guard operations
#[derive(Debug, thiserror::Error)]
pub enum FileGuardError {
    #[error("Path not allowed: {0}")]
    PathNotAllowed(String),
}
