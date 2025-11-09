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

        // TODO: Implement comprehensive active scenario tracking
        //       Currently just ensures directory exists; should implement comprehensive tracking that stores active scenario state properly despite async context limitations for complete scenario management.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Active scenario is tracked properly
        // - Tracking works in async context
        // - Scenario state is persisted and retrievable
        // - Tracking handles concurrent scenarios gracefully
        //
        // DEPENDENCIES:
        // - Async-safe state tracking mechanism (Required)
        // - Scenario state persistence (Required)
        // - Concurrent scenario management (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (evaluation infrastructure enhancement)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Async state management and scenario tracking expertise
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

    /// Scaffold comprehensive broken test files for a scenario
    ///
    /// Creates all three comprehensive broken files (broken-rust.rs, broken-types.ts, broken-python.py)
    /// with multiple intentional errors for testing orchestration capabilities.
    ///
    /// Returns a vector of paths to the created files.
    pub async fn scaffold_comprehensive_broken_files(
        &self,
        scenario_id: &str,
    ) -> Result<Vec<PathBuf>, String> {
        // Ensure scenario directory exists
        self.setup_scenario(scenario_id).await?;

        let mut created_files = Vec::new();

        // Create broken-rust.rs with comprehensive Rust errors
        let broken_rust_content = r#"// Intentionally broken Rust file for arbiter testing
// This file contains multiple compilation errors that the arbiter should fix

use std::collections::HashMap;

// Missing trait derives
#[derive(Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Duplicate struct definition (should be removed)
#[derive(Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Type mismatch - should be u32, not String
let user_id: String = 123;

// Missing import
let result = fetch_user_data(user_id);

// Unused variable
let unused_var = "this should be removed or prefixed with underscore";

// Function with wrong return type
fn calculate_total(items: Vec<u32>) -> String {
    items.iter().sum()
}

// Missing error handling
fn risky_operation() -> Result<serde_json::Value, serde_json::Error> {
    let data = serde_json::from_str("invalid json")?;
    Ok(data)
}

// Inconsistent naming convention
let user_name = "john"; // Should be user_name (snake_case is correct in Rust)
let user_age = 25; // This is correct

// Missing type annotation
let config = HashMap::new();
config.insert("api_url", "https://api.example.com");
config.insert("timeout", "5000");
config.insert("retries", "3");

// TODO comment that should be addressed
// TODO: Implement proper error handling for API calls

// PLACEHOLDER: This is a placeholder that needs implementation
fn placeholder_function() {
    // PLACEHOLDER: Add actual implementation
    todo!("Implement this function");
}

// MOCK DATA: This should be replaced with real data
const MOCK_USERS: &[User] = &[
    User {
        id: "1".to_string(),
        name: "John".to_string(),
        email: "john@example.com".to_string(),
        created_at: chrono::Utc::now(),
    },
    User {
        id: "2".to_string(),
        name: "Jane".to_string(),
        email: "jane@example.com".to_string(),
        created_at: chrono::Utc::now(),
    },
];

// Missing trait implementation
impl User {
    pub fn new(id: String, name: String, email: String) -> Self {
        Self {
            id,
            name,
            email,
            created_at: chrono::Utc::now(),
        }
    }
}

// Missing Display trait for custom error
#[derive(Debug)]
pub enum UserError {
    NotFound,
    InvalidEmail,
    DuplicateId,
}

// Missing field in struct
#[derive(Debug)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    // Missing: pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn main() {
    println!("Hello, broken Rust world!");
}
"#;

        let rust_path = self.create_test_file(scenario_id, "broken-rust.rs", broken_rust_content).await?;
        created_files.push(rust_path);

        // Create broken-types.ts with comprehensive TypeScript errors
        let broken_types_content = r#"// Intentionally broken TypeScript file for arbiter testing
// This file contains multiple compilation errors that the arbiter should fix

interface User {
  id: string;
  name: string;
  email: string;
  // Missing required field: createdAt
}

// Duplicate interface definition (should be removed)
interface User {
  id: string;
  name: string;
  email: string;
  createdAt: Date;
}

// Type mismatch - should be number, not string
const userId: string = 123;

// Missing import
const result = fetchUserData(userId);

// Unused variable
const unusedVar = "this should be removed or prefixed with underscore";

// Function with wrong return type
function calculateTotal(items: number[]): string {
  return items.reduce((sum, item) => sum + item, 0);
}

// Missing error handling
function riskyOperation() {
  const data = JSON.parse(null); // This will throw
  return data;
}

// Inconsistent naming convention
const user_name = "john"; // Should be userName
const userAge = 25; // This is correct

// Missing type annotation
const config = {
  apiUrl: "https://api.example.com",
  timeout: 5000,
  retries: 3
};

// TODO comment that should be addressed
// TODO: Implement proper error handling for API calls

// PLACEHOLDER: This is a placeholder that needs implementation
function placeholderFunction() {
  // PLACEHOLDER: Add actual implementation
}

// MOCK DATA: This should be replaced with real data
const mockUsers = [
  { id: "1", name: "John", email: "john@example.com" },
  { id: "2", name: "Jane", email: "jane@example.com" }
];

export { User, calculateTotal, riskyOperation, config, mockUsers };
"#;

        let types_path = self.create_test_file(scenario_id, "broken-types.ts", broken_types_content).await?;
        created_files.push(types_path);

        // Create broken-python.py with comprehensive Python errors
        let broken_python_content = r#"# Intentionally broken Python file for arbiter testing
# This file contains multiple errors that the arbiter should fix

import json
import requests
from typing import Dict, List, Optional
from datetime import datetime

# Missing import
result = fetch_user_data(user_id)

# Type annotation issues
def calculate_total(items: List[int]) -> str:  # Should return int, not str
    return sum(items)

# Missing error handling
def risky_operation():
    data = json.loads("invalid json")  # This will raise JSONDecodeError
    return data

# Inconsistent naming convention
user_name = "john"  # Should be user_name (snake_case is correct in Python)
userAge = 25  # Should be user_age

# Unused variable
unused_var = "this should be removed or prefixed with underscore"

# Missing type annotations
config = {
    "api_url": "https://api.example.com",
    "timeout": 5000,
    "retries": 3
}

# TODO comment that should be addressed
# TODO: Implement proper error handling for API calls

# PLACEHOLDER: This is a placeholder that needs implementation
def placeholder_function():
    # PLACEHOLDER: Add actual implementation
    pass

# MOCK DATA: This should be replaced with real data
mock_users = [
    {"id": "1", "name": "John", "email": "john@example.com"},
    {"id": "2", "name": "Jane", "email": "jane@example.com"}
]

# Missing docstring
class User:
    def __init__(self, user_id: str, name: str, email: str):
        self.id = user_id
        self.name = name
        self.email = email
        self.created_at = datetime.now()

    def to_dict(self):
        return {
            "id": self.id,
            "name": self.name,
            "email": self.email,
            "created_at": self.created_at.isoformat()
        }

# Missing error handling in class method
def get_user_by_id(user_id: str) -> Optional[User]:
    # This should have proper error handling
    response = requests.get(f"https://api.example.com/users/{user_id}")
    data = response.json()  # This can fail
    return User(data["id"], data["name"], data["email"])

# Missing type hints
def process_users(users):
    results = []
    for user in users:
        if user["email"].endswith("@example.com"):
            results.append(user)
    return results

# Indentation error (intentional)
def broken_indentation():
print("This has wrong indentation")

# Missing return statement
def function_without_return(x: int) -> int:
    x * 2  # Should be: return x * 2

# Unreachable code
def unreachable_code():
    return "first return"
    return "second return"  # This will never be reached

if __name__ == "__main__":
    print("Hello, broken Python world!")
"#;

        let python_path = self.create_test_file(scenario_id, "broken-python.py", broken_python_content).await?;
        created_files.push(python_path);

        Ok(created_files)
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

    #[tokio::test]
    async fn test_scaffold_comprehensive_broken_files() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());
        
        let scenario_id = "test-scenario-comprehensive";
        
        // Scaffold comprehensive broken files
        let result = manager.scaffold_comprehensive_broken_files(scenario_id).await;
        assert!(result.is_ok());
        
        let created_files = result.unwrap();
        assert_eq!(created_files.len(), 3);
        
        // Verify all three files exist
        let rust_path = manager.get_scenario_dir(scenario_id).join("broken-rust.rs");
        let types_path = manager.get_scenario_dir(scenario_id).join("broken-types.ts");
        let python_path = manager.get_scenario_dir(scenario_id).join("broken-python.py");
        
        assert!(rust_path.exists(), "broken-rust.rs should exist");
        assert!(types_path.exists(), "broken-types.ts should exist");
        assert!(python_path.exists(), "broken-python.py should exist");
        
        // Verify Rust file content
        let rust_content = fs::read_to_string(&rust_path).unwrap();
        assert!(rust_content.contains("Duplicate struct definition"));
        assert!(rust_content.contains("Type mismatch"));
        assert!(rust_content.contains("TODO:"));
        assert!(rust_content.contains("PLACEHOLDER:"));
        assert!(rust_content.contains("MOCK DATA:"));
        
        // Verify TypeScript file content
        let types_content = fs::read_to_string(&types_path).unwrap();
        assert!(types_content.contains("Duplicate interface definition"));
        assert!(types_content.contains("Type mismatch"));
        assert!(types_content.contains("TODO:"));
        assert!(types_content.contains("PLACEHOLDER:"));
        assert!(types_content.contains("MOCK DATA:"));
        
        // Verify Python file content
        let python_content = fs::read_to_string(&python_path).unwrap();
        assert!(python_content.contains("Missing import"));
        assert!(python_content.contains("TODO:"));
        assert!(python_content.contains("PLACEHOLDER:"));
        assert!(python_content.contains("MOCK DATA:"));
        assert!(python_content.contains("broken_indentation"));
    }
}
