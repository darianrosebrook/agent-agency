//! Test requirements management

use super::types::*;

/// Test requirements manager
#[derive(Debug)]
pub struct RequirementsManager;

impl RequirementsManager {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_requirements(&self, test_spec: &TestSpecification) -> Result<(), String> {
        // Validate test specification requirements
        if test_spec.test_id.is_empty() {
            return Err("Test ID cannot be empty".to_string());
        }

        if test_spec.inputs.is_empty() {
            return Err("Test must have at least one input".to_string());
        }

        Ok(())
    }

    pub fn extract_requirements(&self, test_spec: &TestSpecification) -> Vec<String> {
        let mut requirements = Vec::new();

        for input in &test_spec.inputs {
            if input.required {
                requirements.push(format!("{} is required", input.name));
            }
        }

        if test_spec.execution_context.timeout_seconds > 0 {
            requirements.push(format!("Must complete within {} seconds",
                test_spec.execution_context.timeout_seconds));
        }

        requirements
    }
}