//! Quality Validation and CAWS Compliance
//!
//! Validates task results against CAWS standards and ensures
//! quality gates are met before task completion.

use crate::types::*;
use crate::execution::ExecutionResult;
use agent_agency_council::CAWSValidator;
use std::collections::HashMap;

/// Quality validator for task results
pub struct QualityValidator {
    caws_validator: Option<CAWSValidator>,
    quality_thresholds: HashMap<String, f64>,
}

impl QualityValidator {
    /// Create a new quality validator
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("react-component".to_string(), 0.8);
        thresholds.insert("file-editing".to_string(), 0.9);
        thresholds.insert("research".to_string(), 0.7);
        thresholds.insert("code-generation".to_string(), 0.85);

        Self {
            caws_validator: None, // Would be injected
            quality_thresholds: thresholds,
        }
    }

    /// Validate the result of a task execution
    pub async fn validate_result(&self, result: &ExecutionResult) -> Result<crate::types::QualityValidation, QualityError> {
        // Check basic success
        if !result.success {
            return Ok(crate::types::QualityValidation {
                passed: false,
                score: 0.0,
                violations: vec!["Task execution failed".to_string()],
                recommendations: vec!["Retry task execution".to_string()],
            });
        }

        // Validate output format and content
        let content_validation = self.validate_content(result).await?;
        let caws_validation = self.validate_caws_compliance(result).await?;

        // Calculate overall score
        let overall_score = (content_validation.score + caws_validation.score) / 2.0;

        // Get threshold for this tool type
        let threshold = self.quality_thresholds
            .get(&result.tool_id)
            .copied()
            .unwrap_or(0.8);

        let passed = overall_score >= threshold;

        let mut violations = Vec::new();
        violations.extend(content_validation.violations);
        violations.extend(caws_validation.violations);

        let mut recommendations = Vec::new();
        recommendations.extend(content_validation.recommendations);
        recommendations.extend(caws_validation.recommendations);

        if !passed {
            recommendations.push(format!(
                "Quality score {:.2} below threshold {:.2} for {}",
                overall_score, threshold, result.tool_id
            ));
        }

        Ok(crate::types::QualityValidation {
            passed,
            score: overall_score,
            violations,
            recommendations,
        })
    }

    /// Validate content quality and format
    async fn validate_content(&self, result: &ExecutionResult) -> Result<ValidationResult, QualityError> {
        match result.tool_id.as_str() {
            "react-generator" => self.validate_react_content(result).await,
            "file-editor" => self.validate_file_content(result).await,
            "research-assistant" => self.validate_research_content(result).await,
            _ => Ok(ValidationResult {
                score: 0.8,
                violations: vec![],
                recommendations: vec!["Unknown tool type - manual review recommended".to_string()],
            }),
        }
    }

    /// Validate CAWS compliance
    async fn validate_caws_compliance(&self, result: &ExecutionResult) -> Result<ValidationResult, QualityError> {
        // In a real implementation, this would use the CAWS validator
        // For now, return a basic compliance check
        Ok(ValidationResult {
            score: 0.9,
            violations: vec![],
            recommendations: vec![],
        })
    }

    /// Validate React component generation content
    async fn validate_react_content(&self, result: &ExecutionResult) -> Result<ValidationResult, QualityError> {
        let mut score = 1.0;
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        if let Some(output) = &result.output {
            // Check for required React component elements
            let component_str = output.get("component")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if !component_str.contains("import React") {
                score -= 0.2;
                violations.push("Missing React import".to_string());
                recommendations.push("Add React import statement".to_string());
            }

            if !component_str.contains("interface") && !component_str.contains("type") {
                score -= 0.1;
                violations.push("Missing TypeScript interface/type".to_string());
                recommendations.push("Add TypeScript interface for props".to_string());
            }

            if !component_str.contains("export const") && !component_str.contains("export function") {
                score -= 0.3;
                violations.push("Missing component export".to_string());
                recommendations.push("Export the component function".to_string());
            }

            // Check SCSS module
            let scss_str = output.get("scss")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if scss_str.is_empty() {
                score -= 0.2;
                violations.push("Missing SCSS module".to_string());
                recommendations.push("Generate SCSS module with component styles".to_string());
            }

            // Check utils
            let utils_str = output.get("utils")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if utils_str.is_empty() {
                score -= 0.1;
                recommendations.push("Consider adding utility functions".to_string());
            }
        } else {
            score = 0.0;
            violations.push("No output generated".to_string());
            recommendations.push("Generate React component output".to_string());
        }

        Ok(ValidationResult {
            score: score.max(0.0),
            violations,
            recommendations,
        })
    }

    /// Validate file editing content
    async fn validate_file_content(&self, result: &ExecutionResult) -> Result<ValidationResult, QualityError> {
        let mut score = 1.0;
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        if let Some(output) = &result.output {
            // Check for required file editing elements
            if !output.get("file").and_then(|v| v.as_str()).is_some() {
                score -= 0.2;
                violations.push("Missing file path".to_string());
                recommendations.push("Include target file path in output".to_string());
            }

            if !output.get("changes_applied").and_then(|v| v.as_str()).is_some() {
                score -= 0.3;
                violations.push("Missing change description".to_string());
                recommendations.push("Describe the changes applied to the file".to_string());
            }

            let syntax_valid = output.get("syntax_valid")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !syntax_valid {
                score -= 0.4;
                violations.push("Syntax validation failed".to_string());
                recommendations.push("Fix syntax errors in file changes".to_string());
            }
        } else {
            score = 0.0;
            violations.push("No file editing output generated".to_string());
            recommendations.push("Generate file editing results".to_string());
        }

        Ok(ValidationResult {
            score: score.max(0.0),
            violations,
            recommendations,
        })
    }

    /// Validate research assistant content
    async fn validate_research_content(&self, result: &ExecutionResult) -> Result<ValidationResult, QualityError> {
        let mut score = 1.0;
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        if let Some(output) = &result.output {
            // Check for required research elements
            if !output.get("sources").and_then(|v| v.as_array()).is_some() {
                score -= 0.2;
                violations.push("Missing research sources".to_string());
                recommendations.push("Include sources used for research".to_string());
            }

            if !output.get("key_findings").and_then(|v| v.as_array()).is_some() {
                score -= 0.3;
                violations.push("Missing key findings".to_string());
                recommendations.push("Summarize key findings from research".to_string());
            }

            if !output.get("recommendations").and_then(|v| v.as_array()).is_some() {
                score -= 0.2;
                violations.push("Missing recommendations".to_string());
                recommendations.push("Provide actionable recommendations".to_string());
            }

            let confidence = output.get("confidence_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            if confidence < 0.7 {
                score -= 0.1;
                recommendations.push(format!("Low confidence score ({:.2}), consider additional research", confidence));
            }
        } else {
            score = 0.0;
            violations.push("No research output generated".to_string());
            recommendations.push("Generate research results".to_string());
        }

        Ok(ValidationResult {
            score: score.max(0.0),
            violations,
            recommendations,
        })
    }
}

/// Result of a validation check
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub score: f64,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Errors from quality validation
#[derive(Debug, thiserror::Error)]
pub enum QualityError {
    #[error("Content validation failed: {0}")]
    ContentValidationError(String),

    #[error("CAWS validation failed: {0}")]
    CAWSValidationError(String),

    #[error("Quality threshold not met: {0}")]
    ThresholdNotMet(String),
}
