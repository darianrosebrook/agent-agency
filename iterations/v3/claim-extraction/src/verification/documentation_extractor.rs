//! Documentation parsing and doc-derived claims
//!
//! This module handles parsing of documentation and extracting claims from docs.

use regex::Regex;
use std::collections::HashMap;
use crate::verification::types::*;

/// Documentation claim extractor
pub struct DocumentationExtractor;

impl DocumentationExtractor {
    /// Extract claims from documentation output
    pub async fn extract_documentation_claims(&self, doc_output: &DocumentationOutput, style_guide: &DocumentationStandards) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        // Parse documentation structure
        let doc_structure = self.parse_documentation_structure(doc_output)?;

        // Extract API documentation claims
        for api_doc in &doc_structure.api_references {
            if let Some(api_claim) = self.extract_api_documentation_claim(api_doc, style_guide)? {
                claims.push(api_claim);
            }
        }

        // Extract usage example claims
        for example in &doc_structure.examples {
            if let Some(example_claim) = self.extract_usage_example_claim(example, style_guide)? {
                claims.push(example_claim);
            }
        }

        // Extract architectural claims from sections
        for section in &doc_structure.sections {
            if let Some(section_claim) = self.extract_section_claim(section, style_guide)? {
                claims.push(section_claim);
            }
        }

        Ok(claims)
    }

    /// Check documentation consistency and completeness
    pub async fn check_documentation_consistency(&self, doc_output: &DocumentationOutput, style_guide: &DocumentationStandards) -> Result<DocumentationConsistency> {
        let mut issues = Vec::new();
        let mut score = 1.0;

        // Check required sections exist
        for required_section in &style_guide.required_sections {
            if !self.has_section(&doc_output.content, required_section) {
                issues.push(format!("Missing required section: {}", required_section));
                score -= 0.2;
            }
        }

        // Check style guide compliance
        for (rule, expected) in &style_guide.style_guide {
            if !self.check_style_rule(&doc_output.content, rule, expected) {
                issues.push(format!("Style violation: {} should be {}", rule, expected));
                score -= 0.1;
            }
        }

        // Check API documentation completeness
        let api_completeness = self.check_api_documentation_completeness(doc_output)?;
        score *= api_completeness;

        if api_completeness < 0.8 {
            issues.push("API documentation incomplete".to_string());
        }

        Ok(DocumentationConsistency {
            overall_score: score.max(0.0),
            issues,
            sections_present: style_guide.required_sections.iter()
                .map(|s| (s.clone(), self.has_section(&doc_output.content, s)))
                .collect(),
            api_completeness,
        })
    }

    /// Parse documentation structure
    pub fn parse_documentation_structure(&self, doc_output: &DocumentationOutput) -> Result<DocumentationStructure> {
        let mut sections = Vec::new();
        let mut examples = Vec::new();
        let mut api_references = Vec::new();

        // Extract sections (markdown headers)
        let section_re = Regex::new(r"^#{1,6}\s+(.+)$")?;
        for line in doc_output.content.lines() {
            if let Some(captures) = section_re.captures(line) {
                if let Some(section) = captures.get(1) {
                    sections.push(section.as_str().to_string());
                }
            }
        }

        // Extract code examples
        let example_re = Regex::new(r"```(?:\w+)?\n(.*?)\n```")?;
        for capture in example_re.captures_iter(&doc_output.content) {
            if let Some(code) = capture.get(1) {
                examples.push(UsageExample {
                    description: "Code example".to_string(),
                    code: code.as_str().to_string(),
                    language: "unknown".to_string(),
                });
            }
        }

        // Extract API references
        let api_re = Regex::new(r"(?:GET|POST|PUT|DELETE|PATCH)\s+(/\S+)")?;
        for capture in api_re.captures_iter(&doc_output.content) {
            if let Some(endpoint) = capture.get(1) {
                api_references.push(endpoint.as_str().to_string());
            }
        }

        Ok(DocumentationStructure {
            sections,
            examples,
            api_references,
        })
    }

    /// Check if section exists in documentation
    fn has_section(&self, content: &str, title: &str) -> bool {
        content.lines().any(|l| {
            let line = l.trim_start();
            line.starts_with('#') && line.to_lowercase().contains(&title.to_lowercase())
        })
    }

    /// Check style guide rule compliance
    fn check_style_rule(&self, content: &str, rule: &str, expected: &str) -> bool {
        match rule {
            "max_line_length" => {
                if let Ok(max_len) = expected.parse::<usize>() {
                    content.lines().all(|line| line.len() <= max_len)
                } else {
                    true
                }
            }
            "consistent_headers" => {
                // Check if headers are consistently formatted
                let header_lines: Vec<_> = content.lines()
                    .filter(|l| l.trim_start().starts_with('#'))
                    .collect();
                if header_lines.len() <= 1 {
                    true
                } else {
                    // Check if all headers follow the same style
                    let first_style = header_lines[0].chars().take_while(|c| *c == '#').count();
                    header_lines.iter().all(|line| {
                        line.chars().take_while(|c| *c == '#').count() == first_style
                    })
                }
            }
            _ => true, // Unknown rules pass
        }
    }

    /// Check API documentation completeness
    fn check_api_documentation_completeness(&self, doc_output: &DocumentationOutput) -> Result<f64> {
        let mut completeness = 0.0;
        let mut checks = 0;

        // Check for parameter documentation
        if doc_output.content.contains("Parameters:") || doc_output.content.contains("Params:") {
            completeness += 0.3;
        }
        checks += 1;

        // Check for response documentation
        if doc_output.content.contains("Returns:") || doc_output.content.contains("Response:") {
            completeness += 0.3;
        }
        checks += 1;

        // Check for error documentation
        if doc_output.content.contains("Errors:") || doc_output.content.contains("Throws:") {
            completeness += 0.2;
        }
        checks += 1;

        // Check for examples
        if doc_output.content.contains("```") {
            completeness += 0.2;
        }
        checks += 1;

        Ok(if checks > 0 { completeness / checks as f64 } else { 0.0 })
    }

    /// Extract API documentation claim
    fn extract_api_documentation_claim(&self, api_ref: &str, _style_guide: &DocumentationStandards) -> Result<Option<AtomicClaim>> {
        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4().to_string(),
            claim_text: format!("API endpoint {} is documented", api_ref),
            claim_type: crate::ClaimType::Functional,
            confidence: 0.9,
            source: "documentation".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }))
    }

    /// Extract usage example claim
    fn extract_usage_example_claim(&self, example: &UsageExample, _style_guide: &DocumentationStandards) -> Result<Option<AtomicClaim>> {
        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4().to_string(),
            claim_text: format!("Usage example provided: {}", example.description),
            claim_type: crate::ClaimType::Functional,
            confidence: 0.8,
            source: "documentation".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }))
    }

    /// Extract section claim
    fn extract_section_claim(&self, section: &str, _style_guide: &DocumentationStandards) -> Result<Option<AtomicClaim>> {
        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4().to_string(),
            claim_text: format!("Documentation section '{}' exists", section),
            claim_type: crate::ClaimType::Informational,
            confidence: 0.95,
            source: "documentation".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }))
    }
}

/// Documentation consistency check result
pub struct DocumentationConsistency {
    pub overall_score: f64,
    pub issues: Vec<String>,
    pub sections_present: HashMap<String, bool>,
    pub api_completeness: f64,
}
