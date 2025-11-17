//! Unified validation types for cross-crate validation operations
//!
//! This module provides shared types for validation issues, severity levels,
//! and validation results used across planning, system configuration, and research.
//!
//! @author @darianrosebrook

#[cfg(feature = "serde")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Validation severity levels - unified across all validation contexts
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidationSeverity {
    /// Critical issue preventing execution
    Critical,
    /// High-priority issue requiring attention
    High,
    /// Error issue (maps to Critical/High)
    Error,
    /// Medium-priority issue
    Medium,
    /// Warning issue (maps to Medium)
    Warning,
    /// Low-priority issue
    Low,
    /// Informational note
    Info,
}

impl ValidationSeverity {
    /// Convert from simple IssueSeverity (Info, Warning, Error) to ValidationSeverity
    pub fn from_simple(severity: SimpleIssueSeverity) -> Self {
        match severity {
            SimpleIssueSeverity::Error => ValidationSeverity::Error,
            SimpleIssueSeverity::Warning => ValidationSeverity::Warning,
            SimpleIssueSeverity::Info => ValidationSeverity::Info,
        }
    }

    /// Convert to simple IssueSeverity for backward compatibility
    pub fn to_simple(self) -> SimpleIssueSeverity {
        match self {
            ValidationSeverity::Critical | ValidationSeverity::High | ValidationSeverity::Error => {
                SimpleIssueSeverity::Error
            }
            ValidationSeverity::Warning | ValidationSeverity::Medium => {
                SimpleIssueSeverity::Warning
            }
            ValidationSeverity::Low | ValidationSeverity::Info => SimpleIssueSeverity::Info,
        }
    }
}

/// Simple issue severity for backward compatibility
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimpleIssueSeverity {
    /// Informational issue
    Info,
    /// Warning issue
    Warning,
    /// Error issue
    Error,
}

/// Validation category - supports both enum and string for flexibility
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationCategory {
    /// Enum-based category (preferred)
    Enum(ValidationCategoryEnum),
    /// String-based category (for backward compatibility)
    String(String),
}

impl ValidationCategory {
    /// Create from string
    pub fn from_string(s: String) -> Self {
        // Try to match known categories
        match s.as_str() {
            "dependency" => ValidationCategory::Enum(ValidationCategoryEnum::Dependency),
            "scope" => ValidationCategory::Enum(ValidationCategoryEnum::Scope),
            "resource" => ValidationCategory::Enum(ValidationCategoryEnum::Resource),
            "quality" => ValidationCategory::Enum(ValidationCategoryEnum::Quality),
            "evidence" => ValidationCategory::Enum(ValidationCategoryEnum::Evidence),
            "council" => ValidationCategory::Enum(ValidationCategoryEnum::Council),
            "performance" => ValidationCategory::Enum(ValidationCategoryEnum::Performance),
            "security" => ValidationCategory::Enum(ValidationCategoryEnum::Security),
            _ => ValidationCategory::String(s),
        }
    }

    /// Get as string
    pub fn as_str(&self) -> &str {
        match self {
            ValidationCategory::Enum(e) => e.as_str(),
            ValidationCategory::String(s) => s.as_str(),
        }
    }
}

/// Enum-based validation categories
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationCategoryEnum {
    /// Dependency-related issues
    Dependency,
    /// Scope boundary violations
    Scope,
    /// Resource constraint issues
    Resource,
    /// Quality gate violations
    Quality,
    /// Evidence requirement issues
    Evidence,
    /// Council compliance issues
    Council,
    /// Performance constraint issues
    Performance,
    /// Security requirement violations
    Security,
}

impl ValidationCategoryEnum {
    pub fn as_str(&self) -> &str {
        match self {
            ValidationCategoryEnum::Dependency => "dependency",
            ValidationCategoryEnum::Scope => "scope",
            ValidationCategoryEnum::Resource => "resource",
            ValidationCategoryEnum::Quality => "quality",
            ValidationCategoryEnum::Evidence => "evidence",
            ValidationCategoryEnum::Council => "council",
            ValidationCategoryEnum::Performance => "performance",
            ValidationCategoryEnum::Security => "security",
        }
    }
}

impl std::fmt::Display for ValidationCategoryEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for ValidationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Unified validation issue type
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Issue severity
    pub severity: ValidationSeverity,
    /// Issue category (enum or string)
    pub category: ValidationCategory,
    /// Human-readable description
    pub description: String,
    /// Affected milestone or component (optional)
    pub affected_component: Option<String>,
    /// Suggested fix (optional)
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue with enum category
    pub fn new(
        severity: ValidationSeverity,
        category: ValidationCategoryEnum,
        description: String,
    ) -> Self {
        Self {
            severity,
            category: ValidationCategory::Enum(category),
            description,
            affected_component: None,
            suggestion: None,
        }
    }

    /// Create a new validation issue with string category (for backward compatibility)
    pub fn with_string_category(
        severity: ValidationSeverity,
        category: String,
        description: String,
    ) -> Self {
        Self {
            severity,
            category: ValidationCategory::from_string(category),
            description,
            affected_component: None,
            suggestion: None,
        }
    }

    /// Create from simple IssueSeverity and string category (for backward compatibility)
    pub fn from_simple(
        severity: SimpleIssueSeverity,
        category: String,
        description: String,
    ) -> Self {
        Self {
            severity: ValidationSeverity::from_simple(severity),
            category: ValidationCategory::from_string(category),
            description,
            affected_component: None,
            suggestion: None,
        }
    }
}

/// Validation result with detailed feedback
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct ValidationResult<T = ValidationIssue> {
    /// Whether validation passed
    pub valid: bool,
    /// Validation score (0.0-1.0)
    pub score: f64,
    /// Detailed validation issues
    pub issues: Vec<T>,
    /// Validation warnings (non-blocking)
    pub warnings: Vec<String>,
    /// Suggested improvements
    pub suggestions: Vec<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl<T> ValidationResult<T> {
    /// Create a new validation result
    pub fn new(valid: bool, score: f64, issues: Vec<T>) -> Self {
        Self {
            valid,
            score,
            issues,
            warnings: Vec::new(),
            suggestions: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a passing result
    pub fn pass() -> Self
    where
        T: Default,
    {
        Self::new(true, 1.0, Vec::new())
    }

    /// Create a failing result with issues
    pub fn fail(issues: Vec<T>) -> Self {
        Self::new(false, 0.0, issues)
    }
}

impl ValidationResult<ValidationIssue> {
    /// Check if there are any critical or error issues
    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|issue| {
            matches!(
                issue.severity,
                ValidationSeverity::Critical | ValidationSeverity::High | ValidationSeverity::Error
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_severity_conversion() {
        assert_eq!(
            ValidationSeverity::from_simple(SimpleIssueSeverity::Error),
            ValidationSeverity::Error
        );
        assert_eq!(
            ValidationSeverity::from_simple(SimpleIssueSeverity::Warning),
            ValidationSeverity::Warning
        );
        assert_eq!(
            ValidationSeverity::from_simple(SimpleIssueSeverity::Info),
            ValidationSeverity::Info
        );
    }

    #[test]
    fn validation_category_from_string() {
        let cat = ValidationCategory::from_string("dependency".to_string());
        assert!(matches!(
            cat,
            ValidationCategory::Enum(ValidationCategoryEnum::Dependency)
        ));

        let cat = ValidationCategory::from_string("custom_category".to_string());
        assert!(matches!(cat, ValidationCategory::String(_)));
    }

    #[test]
    fn validation_category_from_string_all_enum_variants() {
        // Test all 8 match arms
        assert!(matches!(
            ValidationCategory::from_string("dependency".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Dependency)
        ));
        assert!(matches!(
            ValidationCategory::from_string("scope".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Scope)
        ));
        assert!(matches!(
            ValidationCategory::from_string("resource".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Resource)
        ));
        assert!(matches!(
            ValidationCategory::from_string("quality".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Quality)
        ));
        assert!(matches!(
            ValidationCategory::from_string("evidence".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Evidence)
        ));
        assert!(matches!(
            ValidationCategory::from_string("council".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Council)
        ));
        assert!(matches!(
            ValidationCategory::from_string("performance".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Performance)
        ));
        assert!(matches!(
            ValidationCategory::from_string("security".to_string()),
            ValidationCategory::Enum(ValidationCategoryEnum::Security)
        ));
        // Test custom string (catch-all)
        match ValidationCategory::from_string("custom".to_string()) {
            ValidationCategory::String(s) => assert_eq!(s, "custom"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn validation_category_enum_as_str_all_variants() {
        // Test all 8 match arms
        assert_eq!(
            ValidationCategoryEnum::Dependency.as_str(),
            "dependency"
        );
        assert_eq!(ValidationCategoryEnum::Scope.as_str(), "scope");
        assert_eq!(ValidationCategoryEnum::Resource.as_str(), "resource");
        assert_eq!(ValidationCategoryEnum::Quality.as_str(), "quality");
        assert_eq!(ValidationCategoryEnum::Evidence.as_str(), "evidence");
        assert_eq!(ValidationCategoryEnum::Council.as_str(), "council");
        assert_eq!(
            ValidationCategoryEnum::Performance.as_str(),
            "performance"
        );
        assert_eq!(ValidationCategoryEnum::Security.as_str(), "security");
    }

    #[test]
    fn validation_category_as_str() {
        // Test Enum variant
        let cat = ValidationCategory::Enum(ValidationCategoryEnum::Dependency);
        assert_eq!(cat.as_str(), "dependency");

        // Test String variant
        let cat = ValidationCategory::String("custom".to_string());
        assert_eq!(cat.as_str(), "custom");
    }

    #[test]
    fn validation_issue_creation() {
        let issue = ValidationIssue::new(
            ValidationSeverity::Error,
            ValidationCategoryEnum::Quality,
            "Test issue".to_string(),
        );
        assert_eq!(issue.description, "Test issue");
        assert!(matches!(
            issue.category,
            ValidationCategory::Enum(ValidationCategoryEnum::Quality)
        ));
    }

    #[test]
    fn validation_result_has_critical_issues_with_critical() {
        let result = ValidationResult {
            valid: false,
            score: 0.5,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::Critical,
                ValidationCategoryEnum::Security,
                "Critical security issue".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(result.has_critical_issues());
    }

    #[test]
    fn validation_result_has_critical_issues_with_high() {
        let result = ValidationResult {
            valid: false,
            score: 0.6,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::High,
                ValidationCategoryEnum::Quality,
                "High severity issue".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(result.has_critical_issues());
    }

    #[test]
    fn validation_result_has_critical_issues_with_error() {
        let result = ValidationResult {
            valid: false,
            score: 0.7,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::Error,
                ValidationCategoryEnum::Scope,
                "Error severity issue".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(result.has_critical_issues());
    }

    #[test]
    fn validation_result_has_critical_issues_without_critical() {
        let result = ValidationResult {
            valid: true,
            score: 0.9,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::Warning,
                ValidationCategoryEnum::Performance,
                "Warning only".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(!result.has_critical_issues());
    }

    #[test]
    fn validation_result_has_critical_issues_empty() {
        let result = ValidationResult {
            valid: true,
            score: 1.0,
            issues: vec![],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(!result.has_critical_issues());
    }

    #[test]
    fn validation_result_has_critical_issues_boolean_mutation_detection() {
        // Test that has_critical_issues() actually checks severity, not just returns true/false
        // This catches mutations that return hardcoded true or false
        
        // Case 1: Has critical - MUST return true
        let with_critical = ValidationResult {
            valid: false,
            score: 0.5,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::Critical,
                ValidationCategoryEnum::Security,
                "Critical".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(with_critical.has_critical_issues(), "Must return true for Critical severity");
        
        // Case 2: Has high - MUST return true
        let with_high = ValidationResult {
            valid: false,
            score: 0.6,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::High,
                ValidationCategoryEnum::Quality,
                "High".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(with_high.has_critical_issues(), "Must return true for High severity");
        
        // Case 3: Has error - MUST return true
        let with_error = ValidationResult {
            valid: false,
            score: 0.7,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::Error,
                ValidationCategoryEnum::Scope,
                "Error".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(with_error.has_critical_issues(), "Must return true for Error severity");
        
        // Case 4: Only warning - MUST return false
        let only_warning = ValidationResult {
            valid: true,
            score: 0.9,
            issues: vec![ValidationIssue::new(
                ValidationSeverity::Warning,
                ValidationCategoryEnum::Performance,
                "Warning".to_string(),
            )],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(!only_warning.has_critical_issues(), "Must return false for Warning severity");
        
        // Case 5: Empty - MUST return false
        let empty = ValidationResult {
            valid: true,
            score: 1.0,
            issues: vec![],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(!empty.has_critical_issues(), "Must return false for empty issues");
        
        // Case 6: Mixed severities - MUST return true if any critical/high/error
        let mixed = ValidationResult {
            valid: false,
            score: 0.5,
            issues: vec![
                ValidationIssue::new(ValidationSeverity::Warning, ValidationCategoryEnum::Performance, "Warning".to_string()),
                ValidationIssue::new(ValidationSeverity::Critical, ValidationCategoryEnum::Security, "Critical".to_string()),
            ],
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };
        assert!(mixed.has_critical_issues(), "Must return true when Critical present even with Warning");
    }

    #[test]
    fn validation_category_enum_display() {
        assert_eq!(ValidationCategoryEnum::Dependency.to_string(), "dependency");
        assert_eq!(ValidationCategoryEnum::Scope.to_string(), "scope");
        assert_eq!(ValidationCategoryEnum::Resource.to_string(), "resource");
        assert_eq!(ValidationCategoryEnum::Quality.to_string(), "quality");
        assert_eq!(ValidationCategoryEnum::Evidence.to_string(), "evidence");
        assert_eq!(ValidationCategoryEnum::Council.to_string(), "council");
        assert_eq!(
            ValidationCategoryEnum::Performance.to_string(),
            "performance"
        );
        assert_eq!(ValidationCategoryEnum::Security.to_string(), "security");
    }

    #[test]
    fn validation_category_display() {
        let cat_enum = ValidationCategory::Enum(ValidationCategoryEnum::Dependency);
        assert_eq!(cat_enum.to_string(), "dependency");

        let cat_string = ValidationCategory::String("custom".to_string());
        assert_eq!(cat_string.to_string(), "custom");
    }
}
