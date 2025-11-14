//! Contextual bracket extraction and application

use crate::extraction_types::*;
use anyhow::Result;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Context bracket adder for adding contextual scope to claims

#[derive(Debug)]
pub struct ContextBracketAdder {
    temporal_patterns: Vec<Regex>,
    scope_patterns: Vec<Regex>,
    condition_patterns: Vec<Regex>,
}

impl ContextBracketAdder {
    pub fn new() -> Self {
        Self {
            temporal_patterns: Self::build_temporal_patterns(),
            scope_patterns: Self::build_scope_patterns(),
            condition_patterns: Self::build_condition_patterns(),
        }
    }

    /// Extract contextual brackets from a claim
    pub async fn extract_contextual_brackets(
        &self,
        claim_text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<String>> {
        debug!("Extracting contextual brackets for: {}", claim_text);

        let mut brackets = Vec::new();

        // Extract temporal brackets
        if let Some(temporal) = self.extract_temporal_bracket(claim_text) {
            brackets.push(format!("temporal:{}", temporal));
        }

        // Extract scope brackets
        if let Some(scope) = self.extract_scope_bracket(claim_text, context) {
            brackets.push(format!("scope:{}", scope));
        }

        // Extract conditional brackets
        if let Some(condition) = self.extract_conditional_bracket(claim_text) {
            brackets.push(format!("condition:{}", condition));
        }

        // Extract domain-specific brackets
        let domain_brackets = self.extract_domain_brackets(claim_text, context);
        brackets.extend(domain_brackets);

        Ok(brackets)
    }

    /// Apply contextual brackets to a statement
    pub fn apply_contextual_brackets(&self, statement: &str, brackets: &[String]) -> String {
        if brackets.is_empty() {
            statement.to_string()
        } else {
            format!("[{}] {}", brackets.join(", "), statement)
        }
    }

    /// Derive verification requirements from brackets
    pub fn derive_verification_requirements(
        &self,
        statement: &str,
        brackets: &[String],
    ) -> VerificationRequirements {
        let mut requirements = VerificationRequirements {
            test_types: Vec::new(),
            data_sources: Vec::new(),
            temporal_constraints: None,
            scope_constraints: Vec::new(),
            confidence_threshold: 0.8,
        };

        for bracket in brackets {
            if bracket.starts_with("temporal:") {
                requirements.temporal_constraints = Some(bracket[9..].to_string());
                requirements
                    .test_types
                    .push("temporal_verification".to_string());
            } else if bracket.starts_with("scope:") {
                requirements
                    .scope_constraints
                    .push(bracket[6..].to_string());
            } else if bracket.starts_with("condition:") {
                requirements
                    .test_types
                    .push("conditional_verification".to_string());
            }
        }

        // Add basic verification requirements
        if statement.contains("performance")
            || statement.contains("speed")
            || statement.contains("time")
        {
            requirements.test_types.push("performance_test".to_string());
        }

        if statement.contains("security")
            || statement.contains("safe")
            || statement.contains("protect")
        {
            requirements.test_types.push("security_test".to_string());
        }

        requirements
    }

    /// Extract temporal bracket from text
    fn extract_temporal_bracket(&self, text: &str) -> Option<String> {
        for pattern in &self.temporal_patterns {
            if let Some(captures) = pattern.captures(text) {
                if let Some(temporal) = captures.get(1) {
                    return Some(temporal.as_str().to_string());
                }
            }
        }
        None
    }

    /// Extract scope bracket from text
    fn extract_scope_bracket(&self, text: &str, context: &ProcessingContext) -> Option<String> {
        // Check for explicit scope indicators
        for pattern in &self.scope_patterns {
            if let Some(captures) = pattern.captures(text) {
                if let Some(scope) = captures.get(1) {
                    return Some(scope.as_str().to_string());
                }
            }
        }

        // Infer scope from context
        if text.contains("system") || text.contains("application") {
            Some("system".to_string())
        } else if text.contains("user") || text.contains("interface") {
            Some("user_interface".to_string())
        } else if text.contains("data") || text.contains("database") {
            Some("data_layer".to_string())
        } else {
            Some(context.working_spec_id.to_string())
        }
    }

    /// Extract conditional bracket from text
    fn extract_conditional_bracket(&self, text: &str) -> Option<String> {
        for pattern in &self.condition_patterns {
            if let Some(captures) = pattern.captures(text) {
                if let Some(condition) = captures.get(1) {
                    return Some(condition.as_str().to_string());
                }
            }
        }
        None
    }

    /// Extract domain-specific brackets
    fn extract_domain_brackets(&self, text: &str, context: &ProcessingContext) -> Vec<String> {
        let mut brackets = Vec::new();

        // Performance-related brackets
        if text.contains("performance") || text.contains("latency") || text.contains("throughput") {
            brackets.push("domain:performance".to_string());
        }

        // Security-related brackets
        if text.contains("security") || text.contains("auth") || text.contains("encrypt") {
            brackets.push("domain:security".to_string());
        }

        // Reliability brackets
        if text.contains("reliability") || text.contains("availability") || text.contains("fault") {
            brackets.push("domain:reliability".to_string());
        }

        // Scalability brackets
        if text.contains("scale") || text.contains("capacity") || text.contains("load") {
            brackets.push("domain:scalability".to_string());
        }

        brackets
    }

    /// Build temporal pattern regexes
    fn build_temporal_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"(during|after|before|when|while|since|until)\s+([^,.]+)").unwrap(),
            Regex::new(r"(always|never|sometimes|often|rarely)").unwrap(),
            Regex::new(r"(\d+(?:\.\d+)?\s*(?:second|minute|hour|day|week|month|year)s?)").unwrap(),
        ]
    }

    /// Build scope pattern regexes
    fn build_scope_patterns() -> Vec<Regex> {
        vec![
            Regex::new(
                r"(?:in|within|for)\s+(?:the\s+)?([^,.]+(?:system|component|module|service))",
            )
            .unwrap(),
            Regex::new(r"(?:across|throughout)\s+(?:the\s+)?([^,.]+)").unwrap(),
            Regex::new(r"(?:only|exclusively)\s+(?:in|for|within)\s+([^,.]+)").unwrap(),
        ]
    }

    /// Build condition pattern regexes
    fn build_condition_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"(?:if|when|unless|provided that)\s+([^,.]+)").unwrap(),
            Regex::new(r"(?:assuming|given|supposing)\s+(?:that\s+)?([^,.]+)").unwrap(),
            Regex::new(r"(?:in case|in the event)\s+(?:of\s+)?([^,.]+)").unwrap(),
        ]
    }
}

/// Verification requirements derived from contextual brackets

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequirements {
    /// Types of tests needed for verification
    pub test_types: Vec<String>,
    /// Data sources required for verification
    pub data_sources: Vec<String>,
    /// Temporal constraints for verification
    pub temporal_constraints: Option<String>,
    /// Scope constraints for verification
    pub scope_constraints: Vec<String>,
    /// Minimum confidence threshold required
    pub confidence_threshold: f32,
}

impl Default for VerificationRequirements {
    fn default() -> Self {
        Self {
            test_types: Vec::new(),
            data_sources: Vec::new(),
            temporal_constraints: None,
            scope_constraints: Vec::new(),
            confidence_threshold: 0.8,
        }
    }
}

/// Bracket validation utilities
pub struct BracketValidator;

impl BracketValidator {
    /// Validate that brackets are properly formed
    pub fn validate_brackets(brackets: &[String]) -> Result<()> {
        for bracket in brackets {
            if !bracket.contains(':') {
                return Err(anyhow::anyhow!("Invalid bracket format: {}", bracket));
            }

            let parts: Vec<&str> = bracket.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Bracket must have exactly one colon: {}",
                    bracket
                ));
            }

            let bracket_type = parts[0];
            let bracket_value = parts[1];

            if bracket_type.is_empty() || bracket_value.is_empty() {
                return Err(anyhow::anyhow!(
                    "Bracket type and value cannot be empty: {}",
                    bracket
                ));
            }
        }

        Ok(())
    }

    /// Merge duplicate brackets
    pub fn merge_brackets(brackets: Vec<String>) -> Vec<String> {
        let mut merged = HashMap::new();

        for bracket in brackets {
            let parts: Vec<&str> = bracket.split(':').collect();
            if parts.len() == 2 {
                let key = parts[0];
                let value = parts[1];

                merged
                    .entry(key.to_string())
                    .or_insert_with(Vec::new)
                    .push(value.to_string());
            }
        }

        merged
            .into_iter()
            .map(|(bracket_type, values)| {
                if values.len() == 1 {
                    format!("{}:{}", bracket_type, values[0])
                } else {
                    format!("{}:{{{}}}", bracket_type, values.join(", "))
                }
            })
            .collect()
    }
}
