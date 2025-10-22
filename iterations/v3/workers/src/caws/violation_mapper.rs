//! Violation code mapping and constitutional references for CAWS
//!
//! This module handles mapping of violations to constitutional references,
//! violation code generation, and compliance rule mapping.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maps violation codes to constitutional references
#[derive(Debug)]
pub struct ViolationCodeMapper {
    pub rule_mappings: HashMap<String, ConstitutionalReference>,
    pub severity_mappings: HashMap<String, ViolationSeverity>,
}

/// Constitutional reference for a violation
#[derive(Debug, Clone)]
pub struct ConstitutionalReference {
    pub article: String,
    pub section: String,
    pub clause: String,
    pub description: String,
    pub rationale: String,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl ViolationCodeMapper {
    /// Create a new violation code mapper
    pub fn new() -> Self {
        Self {
            rule_mappings: HashMap::new(),
            severity_mappings: HashMap::new(),
        }
    }

    /// Map a violation to its constitutional reference
    pub fn get_constitutional_reference(&self, violation_code: &str) -> Option<&ConstitutionalReference> {
        self.rule_mappings.get(violation_code)
    }

    /// Get severity for a violation code
    pub fn get_severity(&self, violation_code: &str) -> ViolationSeverity {
        self.severity_mappings.get(violation_code)
            .cloned()
            .unwrap_or(ViolationSeverity::Medium)
    }

    /// Generate a new violation code
    pub fn generate_violation_code(&self, rule_type: &str, language: &str) -> String {
        format!("{}_{}_{}", rule_type, language, uuid::Uuid::new_v4().to_string()[..8].to_string())
    }
}



