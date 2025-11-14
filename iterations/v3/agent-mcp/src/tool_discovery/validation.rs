//! Tool validation functionality

use crate::mcp_types::*;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Tool validation result
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the tool is valid
    pub is_valid: bool,
    /// Validation errors found
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation score (0.0-1.0)
    pub score: f32,
}

/// Tool validator trait
pub trait ToolValidator {
    /// Validate a tool
    async fn validate_tool(&self, tool: &MCPTool) -> Result<ValidationResult>;

    /// Get validator name
    fn name(&self) -> &str;
}

/// Basic tool validator implementation
pub struct BasicToolValidator {
    timeout: Duration,
}

impl BasicToolValidator {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl ToolValidator for BasicToolValidator {
    async fn validate_tool(&self, tool: &MCPTool) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Basic validation checks
        if tool.name.is_empty() {
            errors.push("Tool name is empty".to_string());
        }

        if tool.description.is_empty() {
            warnings.push("Tool description is empty".to_string());
        }

        if tool.parameters.is_empty() && tool.output_schema.is_null() {
            warnings.push("Tool has no input or output schema".to_string());
        }

        let score = if errors.is_empty() {
            if warnings.is_empty() {
                1.0
            } else {
                0.8
            }
        } else {
            0.0
        };

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            score,
        })
    }

    fn name(&self) -> &str {
        "basic"
    }
}

/// Schema validator for tool schemas
pub struct SchemaValidator;

impl SchemaValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_input_schema(&self, schema: &serde_json::Value) -> Vec<String> {
        let mut errors = Vec::new();

        // Basic JSON schema validation
        if !schema.is_object() {
            errors.push("Input schema must be an object".to_string());
            return errors;
        }

        // Check for required fields
        let obj = schema.as_object().unwrap();
        if !obj.contains_key("type") {
            errors.push("Input schema missing 'type' field".to_string());
        }

        errors
    }

    pub fn validate_output_schema(&self, schema: &serde_json::Value) -> Vec<String> {
        let mut errors = Vec::new();

        // Similar validation for output schema
        if !schema.is_object() {
            errors.push("Output schema must be an object".to_string());
            return errors;
        }

        let obj = schema.as_object().unwrap();
        if !obj.contains_key("type") {
            errors.push("Output schema missing 'type' field".to_string());
        }

        errors
    }
}
