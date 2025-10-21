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


// Moved from caws_checker.rs: ViolationCodeMapper struct
#[derive(Debug)]
pub struct ViolationCodeMapper {
    // Maps violation codes to constitutional sections
    code_mappings: HashMap<String, ConstitutionalReference>,
}



// Moved from caws_checker.rs: ConstitutionalReference struct
#[derive(Debug, Clone)]
pub struct ConstitutionalReference {
    pub section: String,
    pub subsection: String,
    pub description: String,
    pub severity: ViolationSeverity,
}



// Moved from caws_checker.rs: ViolationCodeMapper impl block
impl ViolationCodeMapper {
    pub fn new() -> Self {
        let mut code_mappings = HashMap::new();

        // Add constitutional references for common violations
        code_mappings.insert(
            "CHANGE_SIZE_LIMIT".to_string(),
            ConstitutionalReference {
                section: "Change Management".to_string(),
                subsection: "Size Limits".to_string(),
                description: "Changes must be surgical and focused".to_string(),
                severity: ViolationSeverity::High,
            },
        );

        code_mappings.insert(
            "SURGICAL_CHANGE_REQUIREMENT".to_string(),
            ConstitutionalReference {
                section: "Change Management".to_string(),
                subsection: "Surgical Changes".to_string(),
                description: "Changes should be precise and minimal".to_string(),
                severity: ViolationSeverity::Medium,
            },
        );

        Self { code_mappings }
    }
}

