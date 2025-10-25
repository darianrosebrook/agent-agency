//! Common validation abstractions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common trait for all validators
pub trait Validator<T> {
    type Error;

    fn validate(&self, value: &T) -> Result<(), Vec<Self::Error>>;
}

/// Common trait for validation rules
pub trait ValidationRule<T> {
    type Error;

    fn check(&self, value: &T) -> Result<(), Self::Error>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

/// Generic validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} (field: {})", self.code, self.message, self.field)
    }
}

impl std::error::Error for ValidationError {}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Validation result with severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub field: String,
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Comprehensive validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub is_valid: bool,
    pub total_issues: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub issues: Vec<ValidationIssue>,
    pub score: f32, // 0.0 to 1.0
    pub validated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ValidationSummary {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            total_issues: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            issues: Vec::new(),
            score: 1.0,
            validated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.total_issues += 1;
        match issue.severity {
            ValidationSeverity::Error => self.error_count += 1,
            ValidationSeverity::Warning => self.warning_count += 1,
            ValidationSeverity::Info => self.info_count += 1,
        }

        if issue.severity == ValidationSeverity::Error {
            self.is_valid = false;
        }

        // Adjust score based on severity
        let penalty = match issue.severity {
            ValidationSeverity::Error => 0.3,
            ValidationSeverity::Warning => 0.1,
            ValidationSeverity::Info => 0.05,
        };
        self.score = (self.score - penalty).max(0.0);

        self.issues.push(issue);
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }

    pub fn get_errors(&self) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|issue| issue.severity == ValidationSeverity::Error)
            .collect()
    }

    pub fn get_warnings(&self) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|issue| issue.severity == ValidationSeverity::Warning)
            .collect()
    }
}

/// Rule-based validator
pub struct RuleBasedValidator<T> {
    rules: Vec<Box<dyn ValidationRule<T, Error = ValidationError>>>,
}

impl<T> RuleBasedValidator<T> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn add_rule<R>(mut self, rule: R) -> Self
    where
        R: ValidationRule<T, Error = ValidationError> + 'static,
    {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn validate(&self, value: &T) -> ValidationSummary {
        let mut summary = ValidationSummary::new();

        for rule in &self.rules {
            match rule.check(value) {
                Ok(()) => {}
                Err(error) => {
                    let issue = ValidationIssue {
                        severity: ValidationSeverity::Error,
                        field: error.field,
                        code: error.code,
                        message: error.message,
                        suggestion: None,
                        details: error.details,
                    };
                    summary.add_issue(issue);
                }
            }
        }

        summary.validated_at = chrono::Utc::now();
        summary
    }
}

impl<T> Default for RuleBasedValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}
