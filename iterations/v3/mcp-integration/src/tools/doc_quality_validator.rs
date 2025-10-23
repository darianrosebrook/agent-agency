//! Documentation Quality Validator Tool
//!
//! Provides documentation quality validation as an MCP tool for autonomous agents.
//! Integrates with the V3 Rust architecture to provide documentation quality
//! validation capabilities to AI models and agents.

use crate::types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};
use uuid::Uuid;

/// Documentation quality validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocQualityResult {
    pub validation_id: String,
    pub quality_score: f64,
    pub issues: Vec<QualityIssue>,
    pub metrics: QualityMetrics,
    pub recommendations: Vec<String>,
}

/// Quality issue found in documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub severity: QualitySeverity,
    pub rule_id: String,
    pub message: String,
    pub line_number: u32,
    pub suggested_fix: String,
}

/// Quality severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualitySeverity {
    Error,
    Warning,
    Info,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub superiority_claims: u32,
    pub unfounded_achievements: u32,
    pub marketing_language: u32,
    pub temporal_docs: u32,
    pub emoji_usage: u32,
}

/// Documentation quality validator tool
#[derive(Debug)]
pub struct DocQualityValidator {
    tool_id: Uuid,
    linter_path: String,
}

impl DocQualityValidator {
    /// Create a new documentation quality validator
    pub fn new() -> Self {
        Self {
            tool_id: Uuid::new_v4(),
            linter_path: "scripts/doc-quality-linter.py".to_string(),
        }
    }

    /// Get the MCP tool definition
    pub fn get_tool_definition(&self) -> MCPTool {
        MCPTool {
            id: self.tool_id,
            name: "doc_quality_validator".to_string(),
            description: "Validates documentation quality against engineering standards and prevents problematic content".to_string(),
            version: "1.0.0".to_string(),
            author: "Agent Agency V3".to_string(),
            tool_type: ToolType::Documentation,
            capabilities: vec![
                ToolCapability::TextProcessing,
                ToolCapability::FileRead,
                ToolCapability::FileSystemAccess,
            ],
            parameters: ToolParameters {
                required: vec![
                    ParameterDefinition {
                        name: "content".to_string(),
                        parameter_type: ParameterType::String,
                        description: "Documentation content to validate".to_string(),
                        default_value: None,
                        validation_rules: vec![],
                    },
                    ParameterDefinition {
                        name: "content_type".to_string(),
                        parameter_type: ParameterType::String,
                        description: "Type of documentation content".to_string(),
                        default_value: Some(serde_json::Value::String("markdown".to_string())),
                        validation_rules: vec![],
                    },
                ],
                optional: vec![
                    ParameterDefinition {
                        name: "file_path".to_string(),
                        parameter_type: ParameterType::String,
                        description: "Path to the documentation file (optional)".to_string(),
                        default_value: None,
                        validation_rules: vec![],
                    },
                    ParameterDefinition {
                        name: "validation_level".to_string(),
                        parameter_type: ParameterType::String,
                        description: "Validation strictness level".to_string(),
                        default_value: Some(serde_json::Value::String("moderate".to_string())),
                        validation_rules: vec![],
                    },
                    ParameterDefinition {
                        name: "include_suggestions".to_string(),
                        parameter_type: ParameterType::Boolean,
                        description: "Include suggested fixes for issues".to_string(),
                        default_value: Some(serde_json::Value::Bool(true)),
                        validation_rules: vec![],
                    },
                ],
                constraints: vec![],
            },
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "validation_id": {"type": "string"},
                    "quality_score": {"type": "number", "minimum": 0, "maximum": 1},
                    "issues": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "severity": {"type": "string", "enum": ["error", "warning", "info"]},
                                "rule_id": {"type": "string"},
                                "message": {"type": "string"},
                                "line_number": {"type": "integer"},
                                "suggested_fix": {"type": "string"}
                            }
                        }
                    },
                    "metrics": {
                        "type": "object",
                        "properties": {
                            "superiority_claims": {"type": "integer"},
                            "unfounded_achievements": {"type": "integer"},
                            "marketing_language": {"type": "integer"},
                            "temporal_docs": {"type": "integer"},
                            "emoji_usage": {"type": "integer"}
                        }
                    },
                    "recommendations": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                }
            }),
            manifest: ToolManifest {
                name: "doc_quality_validator".to_string(),
                version: "1.0.0".to_string(),
                description: "Documentation quality validation tool".to_string(),
                author: "Agent Agency V3".to_string(),
                tool_type: ToolType::Documentation,
                entry_point: "validate".to_string(),
                dependencies: vec![],
                capabilities: vec![
                    ToolCapability::FileRead,
                    ToolCapability::FileSystemAccess,
                ],
                parameters: ToolParameters {
                    required: vec![],
                    optional: vec![],
                    constraints: vec![],
                },
                output_schema: serde_json::json!({
                    "type": "object"
                }),
                endpoint: Some("/tools/doc_quality_validator".to_string()),
                caws_compliance: None,
                metadata: HashMap::new(),
            },
            caws_compliance: CawsComplianceStatus::Compliant,
            registration_time: Utc::now(),
            last_updated: Utc::now(),
            usage_count: 0,
            metadata: HashMap::new(),
            endpoint: "/tools/doc_quality_validator".to_string(),
        }
    }

    /// Validate documentation quality
    pub async fn validate_quality(
        &self,
        content: &str,
        content_type: &str,
        file_path: Option<&str>,
        validation_level: &str,
        include_suggestions: bool,
    ) -> Result<DocQualityResult> {
        info!(
            content_length = content.len(),
            content_type = content_type,
            file_path = ?file_path,
            validation_level = validation_level,
            "Starting documentation quality validation"
        );

        // Create temporary file for content
        let temp_file = tempfile::NamedTempFile::with_suffix(&format!(".{}", content_type))
            .context("Failed to create temporary file")?;
        
        tokio::fs::write(temp_file.path(), content)
            .await
            .context("Failed to write content to temporary file")?;

        // Build command
        let mut cmd = Command::new("python3");
        cmd.arg(&self.linter_path)
            .arg("--path")
            .arg(temp_file.path())
            .arg("--format")
            .arg("json");

        if validation_level != "moderate" {
            cmd.arg("--validation-level").arg(validation_level);
        }

        if include_suggestions {
            cmd.arg("--include-suggestions");
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Execute the linter
        let output = cmd.output().await.context("Failed to execute documentation quality linter")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                exit_code = output.status.code(),
                stderr = %stderr,
                "Documentation quality linter failed"
            );
            
            return Ok(DocQualityResult {
                validation_id: format!("val_{}", Uuid::new_v4()),
                quality_score: 0.0,
                issues: vec![QualityIssue {
                    severity: QualitySeverity::Error,
                    rule_id: "LINTER_ERROR".to_string(),
                    message: format!("Linter failed: {}", stderr),
                    line_number: 0,
                    suggested_fix: "Fix linter errors before validation".to_string(),
                }],
                metrics: QualityMetrics {
                    superiority_claims: 0,
                    unfounded_achievements: 0,
                    marketing_language: 0,
                    temporal_docs: 0,
                    emoji_usage: 0,
                },
                recommendations: vec!["Fix linter errors before validation".to_string()],
            });
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let linter_output: serde_json::Value = serde_json::from_str(&stdout)
            .context("Failed to parse linter JSON output")?;

        // Extract issues
        let issues = self.parse_issues(&linter_output)?;
        
        // Calculate quality score
        let quality_score = self.calculate_quality_score(&issues, validation_level);
        
        // Generate metrics
        let metrics = self.generate_metrics(&issues);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&issues, quality_score);

        let result = DocQualityResult {
            validation_id: format!("val_{}", Uuid::new_v4()),
            quality_score,
            issues,
            metrics,
            recommendations,
        };

        info!(
            validation_id = %result.validation_id,
            quality_score = result.quality_score,
            issues_count = result.issues.len(),
            "Documentation quality validation completed"
        );

        Ok(result)
    }

    /// Parse issues from linter output
    fn parse_issues(&self, linter_output: &serde_json::Value) -> Result<Vec<QualityIssue>> {
        let mut issues = Vec::new();

        if let Some(issues_array) = linter_output.get("issues").and_then(|v| v.as_array()) {
            for issue_value in issues_array {
                if let Ok(issue) = serde_json::from_value::<QualityIssue>(issue_value.clone()) {
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    /// Calculate quality score based on issues and validation level
    fn calculate_quality_score(&self, issues: &[QualityIssue], validation_level: &str) -> f64 {
        if issues.is_empty() {
            return 1.0;
        }

        // Count issues by severity
        let error_count = issues.iter().filter(|i| i.severity == QualitySeverity::Error).count();
        let warning_count = issues.iter().filter(|i| i.severity == QualitySeverity::Warning).count();
        let info_count = issues.iter().filter(|i| i.severity == QualitySeverity::Info).count();

        // Calculate base score
        let total_issues = issues.len();
        let base_score = (1.0 - (total_issues as f64 * 0.1)).max(0.0);

        // Apply severity penalties
        let error_penalty = error_count as f64 * 0.2;
        let warning_penalty = warning_count as f64 * 0.1;
        let info_penalty = info_count as f64 * 0.05;

        // Apply validation level multiplier
        let level_multiplier = match validation_level {
            "strict" => 1.0,
            "moderate" => 0.8,
            "lenient" => 0.6,
            _ => 0.8,
        };

        let final_score = (base_score - error_penalty - warning_penalty - info_penalty).max(0.0);
        (final_score * level_multiplier).min(1.0)
    }

    /// Generate quality metrics from issues
    fn generate_metrics(&self, issues: &[QualityIssue]) -> QualityMetrics {
        let mut metrics = QualityMetrics {
            superiority_claims: 0,
            unfounded_achievements: 0,
            marketing_language: 0,
            temporal_docs: 0,
            emoji_usage: 0,
        };

        for issue in issues {
            match issue.rule_id.as_str() {
                "SUPERIORITY_CLAIM" => metrics.superiority_claims += 1,
                "UNFOUNDED_ACHIEVEMENT" => metrics.unfounded_achievements += 1,
                "MARKETING_LANGUAGE" => metrics.marketing_language += 1,
                "TEMPORAL_DOC" => metrics.temporal_docs += 1,
                "EMOJI_USAGE" => metrics.emoji_usage += 1,
                _ => {}
            }
        }

        metrics
    }

    /// Generate recommendations based on issues and quality score
    fn generate_recommendations(&self, issues: &[QualityIssue], quality_score: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if quality_score < 0.5 {
            recommendations.push("Documentation quality is very low. Consider a complete rewrite focusing on engineering-grade content.".to_string());
        } else if quality_score < 0.8 {
            recommendations.push("Documentation quality needs improvement. Address the identified issues.".to_string());
        }

        // Add specific recommendations based on issue types
        let issue_types: std::collections::HashSet<String> = issues
            .iter()
            .map(|i| i.rule_id.clone())
            .collect();

        if issue_types.contains("SUPERIORITY_CLAIM") {
            recommendations.push("Remove superiority claims and marketing language. Focus on technical capabilities.".to_string());
        }

        if issue_types.contains("UNFOUNDED_ACHIEVEMENT") {
            recommendations.push("Verify all achievement claims with evidence or use more accurate language.".to_string());
        }

        if issue_types.contains("TEMPORAL_DOC") {
            recommendations.push("Move temporal documentation to appropriate archive directories.".to_string());
        }

        if issue_types.contains("EMOJI_USAGE") {
            recommendations.push("Remove emojis or use only approved emojis (⚠️, ✅, 🚫).".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Documentation quality is good. Continue maintaining engineering-grade standards.".to_string());
        }

        recommendations
    }
}

impl Default for DocQualityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_quality() {
        let validator = DocQualityValidator::new();
        
        let content = "# My Project\n\nThis is a revolutionary breakthrough in AI technology!";
        let result = validator.validate_quality(
            content,
            "markdown",
            None,
            "moderate",
            true,
        ).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.quality_score < 1.0); // Should detect the superiority claim
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_get_tool_definition() {
        let validator = DocQualityValidator::new();
        let tool = validator.get_tool_definition();
        
        assert_eq!(tool.name, "doc_quality_validator");
        assert_eq!(tool.tool_type, ToolType::Documentation);
        assert!(!tool.parameters.required.is_empty());
    }
}
