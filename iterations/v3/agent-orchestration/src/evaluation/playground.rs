//! Playground Manager
//!
//! Manages test environments for scenario execution, including:
//! - Setting up playground directories with known issues
//! - Cleaning up after execution
//! - Managing scaffolded test files

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use uuid::Uuid;

/// Playground manager for test environments
pub struct PlaygroundManager {
    playground_root: PathBuf,
    active_scenarios: HashMap<String, PathBuf>,
}

impl PlaygroundManager {
    /// Create new playground manager with default root
    pub fn new() -> Self {
        Self::with_root(PathBuf::from(".playground"))
    }

    /// Create playground manager with custom root directory
    pub fn with_root(root: PathBuf) -> Self {
        Self {
            playground_root: root,
            active_scenarios: HashMap::new(),
        }
    }

    /// Set up playground environment for a scenario
    ///
    /// Creates a temporary directory with scenario-specific test files
    pub async fn setup_scenario(&self, scenario_id: &str) -> Result<(), String> {
        let scenario_dir = self.playground_root.join(scenario_id);
        
        // Create scenario directory
        fs::create_dir_all(&scenario_dir)
            .map_err(|e| format!("Failed to create playground directory: {}", e))?;

        // Store active scenario
        // Note: We can't mutate self in async context, so we'll track this differently
        // For now, just ensure directory exists
        
        Ok(())
    }

    /// Clean up playground environment after scenario execution
    pub async fn cleanup_scenario(&self, scenario_id: &str) -> Result<(), String> {
        let scenario_dir = self.playground_root.join(scenario_id);
        
        if scenario_dir.exists() {
            fs::remove_dir_all(&scenario_dir)
                .map_err(|e| format!("Failed to cleanup playground directory: {}", e))?;
        }
        
        Ok(())
    }

    /// Get playground directory for a scenario
    pub fn get_scenario_dir(&self, scenario_id: &str) -> PathBuf {
        self.playground_root.join(scenario_id)
    }

    /// Create a test file in the playground
    pub async fn create_test_file(
        &self,
        scenario_id: &str,
        filename: &str,
        content: &str,
    ) -> Result<PathBuf, String> {
        let scenario_dir = self.get_scenario_dir(scenario_id);
        let file_path = scenario_dir.join(filename);
        
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }
        
        fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write test file: {}", e))?;
        
        Ok(file_path)
    }

    /// Create a broken test file (with intentional errors)
    pub async fn create_broken_file(
        &self,
        scenario_id: &str,
        filename: &str,
        error_type: &str,
    ) -> Result<PathBuf, String> {
        let content = match error_type {
            "compilation" => {
                // Rust compilation error
                r#"fn main() {
    let x: i32 = "hello"; // Type mismatch
    println!("{}", x);
}
"#
            }
            "syntax" => {
                // Syntax error
                r#"fn main() {
    let x = 5
    println!("{}", x); // Missing semicolon
}
"#
            }
            "logic" => {
                // Logic error (compiles but wrong)
                r#"fn main() {
    let x = 5;
    let y = 10;
    let result = x - y; // Should be addition
    println!("Result: {}", result);
}
"#
            }
            _ => {
                return Err(format!("Unknown error type: {}", error_type));
            }
        };

        self.create_test_file(scenario_id, filename, content).await
    }

    /// List all files in a scenario playground
    pub fn list_scenario_files(&self, scenario_id: &str) -> Result<Vec<PathBuf>, String> {
        let scenario_dir = self.get_scenario_dir(scenario_id);
        
        if !scenario_dir.exists() {
            return Ok(vec![]);
        }

        let mut files = Vec::new();
        let entries = fs::read_dir(&scenario_dir)
            .map_err(|e| format!("Failed to read playground directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                // Recursively list files in subdirectories
                let subfiles = self.list_files_recursive(&path)?;
                files.extend(subfiles);
            }
        }

        Ok(files)
    }

    /// Recursively list files in a directory
    fn list_files_recursive(&self, dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                let subfiles = self.list_files_recursive(&path)?;
                files.extend(subfiles);
            }
        }

        Ok(files)
    }

    /// Clean up all playground directories
    pub async fn cleanup_all(&self) -> Result<(), String> {
        if self.playground_root.exists() {
            fs::remove_dir_all(&self.playground_root)
                .map_err(|e| format!("Failed to cleanup playground root: {}", e))?;
        }
        
        Ok(())
    }
}

impl Default for PlaygroundManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_playground_manager_creation() {
        let manager = PlaygroundManager::new();
        assert_eq!(manager.playground_root, PathBuf::from(".playground"));
    }

    #[tokio::test]
    async fn test_setup_and_cleanup_scenario() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());
        
        let scenario_id = "test-scenario-001";
        
        // Setup scenario
        let result = manager.setup_scenario(scenario_id).await;
        assert!(result.is_ok());
        
        // Verify directory exists
        let scenario_dir = manager.get_scenario_dir(scenario_id);
        assert!(scenario_dir.exists());
        
        // Cleanup scenario
        let result = manager.cleanup_scenario(scenario_id).await;
        assert!(result.is_ok());
        
        // Verify directory is removed
        assert!(!scenario_dir.exists());
    }

    #[tokio::test]
    async fn test_create_test_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());
        
        let scenario_id = "test-scenario-002";
        manager.setup_scenario(scenario_id).await.unwrap();
        
        let file_path = manager.create_test_file(
            scenario_id,
            "test.rs",
            "fn main() { println!(\"Hello\"); }",
        ).await;
        
        assert!(file_path.is_ok());
        let path = file_path.unwrap();
        assert!(path.exists());
        
        // Verify content
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Hello"));
    }

    #[tokio::test]
    async fn test_create_broken_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());
        
        let scenario_id = "test-scenario-003";
        manager.setup_scenario(scenario_id).await.unwrap();
        
        let file_path = manager.create_broken_file(scenario_id, "broken.rs", "compilation").await;
        assert!(file_path.is_ok());
        
        let path = file_path.unwrap();
        assert!(path.exists());
        
        // Verify content has error
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Type mismatch") || content.contains("\"hello\""));
    }
}
