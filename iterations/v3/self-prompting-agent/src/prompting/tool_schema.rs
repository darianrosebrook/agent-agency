//! Tool Call Schema & Validation
//!
//! Defines the JSON schema for tool calls and validation logic.
//! Ensures deterministic, parseable outputs from model providers.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolSchemaError {
    #[error("Schema validation failed: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Invalid JSON format: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid path: {path} - {reason}")]
    InvalidPath { path: PathBuf, reason: String },
}

/// Kind of file change operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// Create a new file
    Create,
    /// Modify an existing file (requires expected_sha256)
    Modify,
    /// Delete an existing file
    Delete,
}

/// Individual file change in a patch action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path to modify (must be in allow-list)
    pub path: PathBuf,

    /// Type of change
    pub kind: ChangeKind,

    /// Expected SHA256 of target file (modify/delete only)
    /// Must match current file content before applying change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_sha256: Option<String>,

    /// Full new content (create/modify only)
    /// For delete operations, this field is ignored
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Complete patch action tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchAction {
    /// Tool type identifier (must be "patch")
    #[serde(rename = "type")]
    pub action_type: String,

    /// List of file changes to apply atomically
    pub changes: Vec<FileChange>,

    /// Human-readable rationale for changes
    pub rationale: String,
}

/// Validator for tool call schema compliance
pub struct ToolCallValidator {
    allow_list: Vec<PathBuf>,
}

impl ToolCallValidator {
    pub fn new(allow_list: Vec<PathBuf>) -> Self {
        Self { allow_list }
    }

    /// Validate a tool call against schema and allow-list
    pub fn validate(&self, json_str: &str) -> Result<PatchAction, ToolSchemaError> {
        // Parse JSON first
        let action: PatchAction = serde_json::from_str(json_str)?;

        // Validate action type
        if action.action_type != "patch" {
            return Err(ToolSchemaError::ValidationError {
                field: "type".to_string(),
                message: format!("Expected 'patch', got '{}'", action.action_type),
            });
        }

        // Validate changes
        if action.changes.is_empty() {
            return Err(ToolSchemaError::ValidationError {
                field: "changes".to_string(),
                message: "At least one change required".to_string(),
            });
        }

        for (idx, change) in action.changes.iter().enumerate() {
            self.validate_file_change(change, idx)?;
        }

        Ok(action)
    }

    fn validate_file_change(&self, change: &FileChange, idx: usize) -> Result<(), ToolSchemaError> {
        let field_prefix = format!("changes[{}]", idx);

        // Validate path is in allow-list
        if !self.is_path_allowed(&change.path) {
            return Err(ToolSchemaError::InvalidPath {
                path: change.path.clone(),
                reason: "Path not in allow-list".to_string(),
            });
        }

        // Validate path is safe (no .., absolute, etc.)
        if !self.is_path_safe(&change.path) {
            return Err(ToolSchemaError::InvalidPath {
                path: change.path.clone(),
                reason: "Path contains unsafe components (.. or absolute)".to_string(),
            });
        }

        // Validate change-specific requirements
        match change.kind {
            ChangeKind::Create => {
                if change.expected_sha256.is_some() {
                    return Err(ToolSchemaError::ValidationError {
                        field: format!("{}.expected_sha256", field_prefix),
                        message: "expected_sha256 not allowed for create operations".to_string(),
                    });
                }
                if change.content.is_none() {
                    return Err(ToolSchemaError::ValidationError {
                        field: format!("{}.content", field_prefix),
                        message: "content required for create operations".to_string(),
                    });
                }
            }
            ChangeKind::Modify => {
                if change.expected_sha256.is_none() {
                    return Err(ToolSchemaError::ValidationError {
                        field: format!("{}.expected_sha256", field_prefix),
                        message: "expected_sha256 required for modify operations".to_string(),
                    });
                }
                if change.content.is_none() {
                    return Err(ToolSchemaError::ValidationError {
                        field: format!("{}.content", field_prefix),
                        message: "content required for modify operations".to_string(),
                    });
                }
            }
            ChangeKind::Delete => {
                if change.expected_sha256.is_none() {
                    return Err(ToolSchemaError::ValidationError {
                        field: format!("{}.expected_sha256", field_prefix),
                        message: "expected_sha256 required for delete operations".to_string(),
                    });
                }
                if change.content.is_some() {
                    return Err(ToolSchemaError::ValidationError {
                        field: format!("{}.content", field_prefix),
                        message: "content not allowed for delete operations".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn is_path_allowed(&self, path: &PathBuf) -> bool {
        self.allow_list.iter().any(|allowed| {
            path.starts_with(allowed) || path == allowed
        })
    }

    fn is_path_safe(&self, path: &PathBuf) -> bool {
        // No absolute paths
        if path.is_absolute() {
            return false;
        }

        // No .. components
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return false;
        }

        // No hidden files (starting with .)
        if path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
            return false;
        }

        true
    }
}

impl PatchAction {
    /// Get the JSON schema for this tool
    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["type", "changes", "rationale"],
            "properties": {
                "type": {
                    "type": "string",
                    "const": "patch",
                    "description": "Tool action type"
                },
                "changes": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["path", "kind"],
                        "oneOf": [
                            {
                                "properties": {
                                    "kind": { "const": "create" },
                                    "path": { "type": "string", "description": "File path to create" },
                                    "content": { "type": "string", "description": "Full file content" }
                                },
                                "required": ["content"]
                            },
                            {
                                "properties": {
                                    "kind": { "const": "modify" },
                                    "path": { "type": "string", "description": "File path to modify" },
                                    "expected_sha256": { "type": "string", "description": "SHA256 of current content" },
                                    "content": { "type": "string", "description": "Full new file content" }
                                },
                                "required": ["expected_sha256", "content"]
                            },
                            {
                                "properties": {
                                    "kind": { "const": "delete" },
                                    "path": { "type": "string", "description": "File path to delete" },
                                    "expected_sha256": { "type": "string", "description": "SHA256 of current content" }
                                },
                                "required": ["expected_sha256"]
                            }
                        ]
                    }
                },
                "rationale": {
                    "type": "string",
                    "description": "Explanation of why these changes are needed"
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_create_action() {
        let validator = ToolCallValidator::new(vec![PathBuf::from("src/")]);

        let json = r#"{
            "type": "patch",
            "changes": [
                {
                    "path": "src/main.rs",
                    "kind": "create",
                    "content": "fn main() { println!(\"Hello!\"); }"
                }
            ],
            "rationale": "Add main function"
        }"#;

        let result = validator.validate(json);
        assert!(result.is_ok());

        let action = result.unwrap();
        assert_eq!(action.changes.len(), 1);
        assert_eq!(action.changes[0].kind, ChangeKind::Create);
    }

    #[test]
    fn test_valid_modify_action() {
        let validator = ToolCallValidator::new(vec![PathBuf::from("src/")]);

        let json = r#"{
            "type": "patch",
            "changes": [
                {
                    "path": "src/main.rs",
                    "kind": "modify",
                    "expected_sha256": "abc123",
                    "content": "fn main() { println!(\"Updated!\"); }"
                }
            ],
            "rationale": "Update greeting"
        }"#;

        let result = validator.validate(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_path_out_of_scope() {
        let validator = ToolCallValidator::new(vec![PathBuf::from("src/")]);

        let json = r#"{
            "type": "patch",
            "changes": [
                {
                    "path": "other/file.rs",
                    "kind": "create",
                    "content": "fn main() {}"
                }
            ],
            "rationale": "Create file"
        }"#;

        let result = validator.validate(json);
        assert!(matches!(result, Err(ToolSchemaError::InvalidPath { .. })));
    }

    #[test]
    fn test_invalid_path_unsafe() {
        let validator = ToolCallValidator::new(vec![PathBuf::from("src/")]);

        let json = r#"{
            "type": "patch",
            "changes": [
                {
                    "path": "../escape.rs",
                    "kind": "create",
                    "content": "fn main() {}"
                }
            ],
            "rationale": "Create file"
        }"#;

        let result = validator.validate(json);
        assert!(matches!(result, Err(ToolSchemaError::InvalidPath { .. })));
    }

    #[test]
    fn test_invalid_missing_content_for_create() {
        let validator = ToolCallValidator::new(vec![PathBuf::from("src/")]);

        let json = r#"{
            "type": "patch",
            "changes": [
                {
                    "path": "src/main.rs",
                    "kind": "create"
                }
            ],
            "rationale": "Create file"
        }"#;

        let result = validator.validate(json);
        assert!(matches!(result, Err(ToolSchemaError::ValidationError { .. })));
    }

    #[test]
    fn test_invalid_wrong_type() {
        let validator = ToolCallValidator::new(vec![PathBuf::from("src/")]);

        let json = r#"{
            "type": "invalid",
            "changes": [],
            "rationale": "Test"
        }"#;

        let result = validator.validate(json);
        assert!(matches!(result, Err(ToolSchemaError::ValidationError { .. })));
    }
}

