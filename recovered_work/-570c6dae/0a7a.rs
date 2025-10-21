//! Token evaluator for checking design system compliance

use std::collections::HashMap;
use regex::Regex;
use async_trait::async_trait;

use super::{Evaluator, EvalContext, EvalCriterion, EvaluationError};
use crate::types::{Artifact, TaskType, ArtifactType};

/// Token evaluator configuration
#[derive(Debug, Clone)]
pub struct TokenEvaluatorConfig {
    pub allowed_colors: Vec<String>,
    pub allowed_spacing: Vec<String>,
    pub allowed_font_sizes: Vec<String>,
    pub required_tokens: Vec<String>,
    pub banned_patterns: Vec<String>,
}

impl Default for TokenEvaluatorConfig {
    fn default() -> Self {
        Self {
            allowed_colors: vec![
                "#000000".to_string(),
                "#ffffff".to_string(),
                "#0066cc".to_string(),
                "#ff0000".to_string(),
            ],
            allowed_spacing: vec![
                "4px".to_string(),
                "8px".to_string(),
                "16px".to_string(),
                "24px".to_string(),
            ],
            allowed_font_sizes: vec![
                "12px".to_string(),
                "14px".to_string(),
                "16px".to_string(),
                "18px".to_string(),
                "24px".to_string(),
            ],
            required_tokens: vec![
                "color-primary".to_string(),
                "spacing-medium".to_string(),
            ],
            banned_patterns: vec![
                r"#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})".to_string(), // Hex colors
                r"\d+px".to_string(), // Magic pixel values
            ],
        }
    }
}

/// Token evaluator for design system compliance
pub struct TokenEvaluator {
    config: TokenEvaluatorConfig,
}

impl TokenEvaluator {
    /// Create a new token evaluator
    pub fn new() -> Self {
        Self {
            config: TokenEvaluatorConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: TokenEvaluatorConfig) -> Self {
        Self { config }
    }

    /// Check for hardcoded colors
    fn evaluate_color_compliance(&self, content: &str) -> EvalCriterion {
        let mut violations = Vec::new();

        // Check for hex colors not in allowed list
        let hex_regex = Regex::new(r"#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})").unwrap();
        for capture in hex_regex.find_iter(content) {
            let color = capture.as_str();
            if !self.config.allowed_colors.contains(&color.to_string()) {
                violations.push(format!("Hardcoded color: {}", color));
            }
        }

        // Check for RGB/RGBA colors
        let rgb_regex = Regex::new(r"rgb(a?)\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*(?:,\s*[\d.]+\s*)?\)").unwrap();
        for capture in rgb_regex.find_iter(content) {
            violations.push(format!("Hardcoded RGB color: {}", capture.as_str()));
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "colors-compliant".to_string(),
            description: "Only allowed colors are used".to_string(),
            weight: 0.25,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("No hardcoded colors found".to_string())
            } else {
                Some(format!("Violations: {}", violations.join("; ")))
            },
        }
    }

    /// Check for hardcoded spacing values
    fn evaluate_spacing_compliance(&self, content: &str) -> EvalCriterion {
        let mut violations = Vec::new();

        // Find pixel values that aren't in allowed spacing
        let px_regex = Regex::new(r"\b(\d+)px\b").unwrap();
        for capture in px_regex.captures_iter(content) {
            if let Some(px_match) = capture.get(1) {
                let px_value = format!("{}px", px_match.as_str());
                if !self.config.allowed_spacing.contains(&px_value) {
                    violations.push(format!("Hardcoded spacing: {}", px_value));
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "spacing-compliant".to_string(),
            description: "Only allowed spacing values are used".to_string(),
            weight: 0.25,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("No hardcoded spacing found".to_string())
            } else {
                Some(format!("Violations: {}", violations.join("; ")))
            },
        }
    }

    /// Check for hardcoded font sizes
    fn evaluate_font_size_compliance(&self, content: &str) -> EvalCriterion {
        let mut violations = Vec::new();

        // Find font-size declarations
        let font_size_regex = Regex::new(r"font-size:\s*([^;]+)").unwrap();
        for capture in font_size_regex.captures_iter(content) {
            if let Some(size_match) = capture.get(1) {
                let size_value = size_match.as_str().trim();
                if !self.config.allowed_font_sizes.iter().any(|allowed| size_value.contains(allowed)) {
                    violations.push(format!("Hardcoded font-size: {}", size_value));
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "font-sizes-compliant".to_string(),
            description: "Only allowed font sizes are used".to_string(),
            weight: 0.25,
            passed,
            score,
            notes: if violations.is_empty() {
                Some("No hardcoded font sizes found".to_string())
            } else {
                Some(format!("Violations: {}", violations.join("; ")))
            },
        }
    }

    /// Check for required token usage
    fn evaluate_required_tokens(&self, content: &str) -> EvalCriterion {
        let mut missing_tokens = Vec::new();

        for token in &self.config.required_tokens {
            if !content.contains(token) {
                missing_tokens.push(token.clone());
            }
        }

        let passed = missing_tokens.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "required-tokens-present".to_string(),
            description: "All required design tokens are used".to_string(),
            weight: 0.25,
            passed,
            score,
            notes: if missing_tokens.is_empty() {
                Some("All required tokens present".to_string())
            } else {
                Some(format!("Missing tokens: {}", missing_tokens.join(", ")))
            },
        }
    }

    /// Check for banned patterns
    fn evaluate_banned_patterns(&self, content: &str) -> EvalCriterion {
        let mut violations = Vec::new();

        for pattern in &self.config.banned_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for capture in regex.find_iter(content) {
                    violations.push(format!("Banned pattern '{}': {}", pattern, capture.as_str()));
                }
            }
        }

        let passed = violations.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "no-banned-patterns".to_string(),
            description: "No banned patterns found".to_string(),
            weight: 0.0, // Additional check, doesn't affect score much
            passed,
            score,
            notes: if violations.is_empty() {
                Some("No banned patterns detected".to_string())
            } else {
                Some(format!("Found: {}", violations.join("; ")))
            },
        }
    }

    /// Extract content that might contain design tokens (CSS, styles, etc.)
    fn extract_design_content(&self, artifacts: &[Artifact]) -> String {
        let mut design_content = String::new();

        for artifact in artifacts {
            // Check file extension or content for design-related files
            let is_design_file = artifact.file_path.ends_with(".css")
                || artifact.file_path.ends_with(".scss")
                || artifact.file_path.ends_with(".tsx")
                || artifact.file_path.contains("style")
                || artifact.file_path.contains("theme")
                || artifact.content.contains("color:")
                || artifact.content.contains("font-size:")
                || artifact.content.contains("margin:")
                || artifact.content.contains("padding:");

            if is_design_file || matches!(artifact.artifact_type, ArtifactType::Code) {
                design_content.push_str(&artifact.content);
                design_content.push('\n');
            }
        }

        design_content
    }
}

#[async_trait]
impl Evaluator for TokenEvaluator {
    async fn evaluate(&self, artifacts: &[Artifact], _context: &EvalContext) -> Result<Vec<EvalCriterion>, EvaluationError> {
        let design_content = self.extract_design_content(artifacts);

        if design_content.trim().is_empty() {
            return Ok(vec![EvalCriterion {
                id: "design-content-present".to_string(),
                description: "Design content is present for token evaluation".to_string(),
                weight: 1.0,
                passed: false,
                score: 0.0,
                notes: Some("No design-related content found in artifacts".to_string()),
            }]);
        }

        let mut criteria = Vec::new();
        criteria.push(self.evaluate_color_compliance(&design_content));
        criteria.push(self.evaluate_spacing_compliance(&design_content));
        criteria.push(self.evaluate_font_size_compliance(&design_content));
        criteria.push(self.evaluate_required_tokens(&design_content));
        criteria.push(self.evaluate_banned_patterns(&design_content));

        Ok(criteria)
    }

    fn applies_to(&self, task_type: &TaskType) -> bool {
        matches!(task_type, TaskType::CodeGeneration | TaskType::DesignTokenApplication | TaskType::CodeFix)
    }

    fn evaluator_type(&self) -> &'static str {
        "token"
    }
}
