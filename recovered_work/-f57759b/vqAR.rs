//! Input validation utilities for external data

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashSet;

/// Maximum allowed length for various input types
pub const MAX_STRING_LENGTH: usize = 10_000;
pub const MAX_IDENTIFIER_LENGTH: usize = 100;
pub const MAX_URL_LENGTH: usize = 2048;
pub const MAX_EMAIL_LENGTH: usize = 254;

/// Input validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub sanitized_value: Option<String>,
}

/// Common validation patterns
lazy_static::lazy_static! {
    static ref IDENTIFIER_PATTERN: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    static ref EMAIL_PATTERN: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref URL_PATTERN: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    static ref ALPHANUMERIC_PATTERN: Regex = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
    static ref SAFE_PATH_PATTERN: Regex = Regex::new(r"^[^<>\"'|?*\[\]{}\\,;:%$#]*$").unwrap();
}

/// Validate and sanitize a string input
pub fn validate_string_input(input: &str, field_name: &str, max_length: usize) -> ValidationResult {
    let mut errors = Vec::new();
    let mut sanitized = input.to_string();

    // Length validation
    if input.len() > max_length {
        errors.push(format!("{} exceeds maximum length of {} characters", field_name, max_length));
    }

    // Null byte check
    if input.contains('\0') {
        errors.push(format!("{} contains null bytes", field_name));
        sanitized = input.replace('\0', "");
    }

    // Control character check (except common whitespace)
    if input.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        errors.push(format!("{} contains control characters", field_name));
        sanitized = input.chars().filter(|c| !c.is_control() || c.is_whitespace()).collect();
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        sanitized_value: Some(sanitized),
    }
}

/// Validate an identifier (variable name, function name, etc.)
pub fn validate_identifier(input: &str, field_name: &str) -> ValidationResult {
    let mut result = validate_string_input(input, field_name, MAX_IDENTIFIER_LENGTH);

    if !IDENTIFIER_PATTERN.is_match(input) {
        result.errors.push(format!("{} must start with a letter or underscore and contain only alphanumeric characters and underscores", field_name));
        result.is_valid = false;
    }

    // Reserved word check
    let reserved_words = ["null", "undefined", "true", "false", "NaN"];
    if reserved_words.contains(&input.to_lowercase().as_str()) {
        result.errors.push(format!("{} cannot be a reserved word", field_name));
        result.is_valid = false;
    }

    result
}

/// Validate an email address
pub fn validate_email(input: &str, field_name: &str) -> ValidationResult {
    let mut result = validate_string_input(input, field_name, MAX_EMAIL_LENGTH);

    if !EMAIL_PATTERN.is_match(input) {
        result.errors.push(format!("{} is not a valid email address", field_name));
        result.is_valid = false;
    }

    result
}

/// Validate a URL
pub fn validate_url(input: &str, field_name: &str, allow_http: bool) -> ValidationResult {
    let mut result = validate_string_input(input, field_name, MAX_URL_LENGTH);

    let url_pattern = if allow_http {
        &*URL_PATTERN
    } else {
        // HTTPS only pattern
        lazy_static::lazy_static! {
            static ref HTTPS_ONLY_PATTERN: Regex = Regex::new(r"^https://[^\s/$.?#].[^\s]*$").unwrap();
        }
        &*HTTPS_ONLY_PATTERN
    };

    if !url_pattern.is_match(input) {
        let protocol_req = if allow_http { "http or https" } else { "https" };
        result.errors.push(format!("{} must be a valid {} URL", field_name, protocol_req));
        result.is_valid = false;
    }

    result
}

/// Validate alphanumeric input only
pub fn validate_alphanumeric(input: &str, field_name: &str, max_length: usize) -> ValidationResult {
    let mut result = validate_string_input(input, field_name, max_length);

    if !ALPHANUMERIC_PATTERN.is_match(input) {
        result.errors.push(format!("{} must contain only alphanumeric characters", field_name));
        result.is_valid = false;
    }

    result
}

/// Validate file path for safety
pub fn validate_file_path(input: &str, field_name: &str) -> ValidationResult {
    let mut result = validate_string_input(input, field_name, 500);

    // Check for directory traversal
    if input.contains("..") || input.contains("../") || input.contains("..\\") {
        result.errors.push(format!("{} contains directory traversal sequences", field_name));
        result.is_valid = false;
    }

    // Check for dangerous characters
    if !SAFE_PATH_PATTERN.is_match(input) {
        result.errors.push(format!("{} contains unsafe characters", field_name));
        result.is_valid = false;
    }

    // Check for absolute paths that might be dangerous
    if input.starts_with('/') || input.starts_with('\\') || input.contains(":\\") {
        result.errors.push(format!("{} cannot contain absolute paths", field_name));
        result.is_valid = false;
    }

    result
}

/// Validate numeric input within bounds
pub fn validate_numeric<T>(input: &str, field_name: &str, min: T, max: T) -> ValidationResult
where
    T: PartialOrd + std::str::FromStr + std::fmt::Display + Copy,
{
    let mut result = validate_string_input(input, field_name, 50);

    match input.parse::<T>() {
        Ok(value) => {
            if value < min {
                result.errors.push(format!("{} must be at least {}", field_name, min));
                result.is_valid = false;
            } else if value > max {
                result.errors.push(format!("{} must be at most {}", field_name, max));
                result.is_valid = false;
            }
        }
        Err(_) => {
            result.errors.push(format!("{} is not a valid number", field_name));
            result.is_valid = false;
        }
    }

    result
}

/// Validate JSON input for safety
pub fn validate_json_input(input: &str, field_name: &str, max_depth: usize) -> ValidationResult {
    let mut result = validate_string_input(input, field_name, MAX_STRING_LENGTH);

    // Parse JSON to check validity and depth
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(value) => {
            if json_depth(&value) > max_depth {
                result.errors.push(format!("{} JSON exceeds maximum depth of {}", field_name, max_depth));
                result.is_valid = false;
            }
        }
        Err(e) => {
            result.errors.push(format!("{} contains invalid JSON: {}", field_name, e));
            result.is_valid = false;
        }
    }

    result
}

/// Calculate JSON nesting depth
fn json_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(arr) => {
            1 + arr.iter().map(json_depth).max().unwrap_or(0)
        }
        serde_json::Value::Object(obj) => {
            1 + obj.values().map(json_depth).max().unwrap_or(0)
        }
        _ => 1,
    }
}

/// TODO: Replace basic SQL injection check with comprehensive security validation
/// Requirements for completion:
/// - [ ] Implement comprehensive SQL injection detection using multiple techniques
/// - [ ] Add support for different SQL dialects and injection patterns
/// - [ ] Implement proper input sanitization and parameterized queries
/// - [ ] Add support for advanced attack pattern detection
/// - [ ] Implement proper error handling for validation failures
/// - [ ] Add support for security validation performance optimization
/// - [ ] Implement proper memory management for security validation
/// - [ ] Add support for security validation monitoring and alerting
/// - [ ] Implement proper cleanup of security validation resources
/// - [ ] Add support for security validation result reporting and logging
pub fn validate_sql_safe(input: &str, field_name: &str) -> ValidationResult {
    let mut result = validate_string_input(input, field_name, 1000);

    let dangerous_patterns = [
        "DROP", "DELETE", "UPDATE", "INSERT", "ALTER", "CREATE",
        "--", "/*", "*/", "xp_", "sp_", "exec", "union", "select",
        "1=1", "1=0", "script", "javascript:", "vbscript:",
    ];

    let input_upper = input.to_uppercase();
    for pattern in &dangerous_patterns {
        if input_upper.contains(&pattern.to_uppercase()) {
            result.errors.push(format!("{} contains potentially dangerous SQL pattern: {}", field_name, pattern));
            result.is_valid = false;
            break;
        }
    }

    result
}

/// Validate environment variable name format
pub fn validate_env_var_name(name: &str, field_name: &str) -> ValidationResult {
    let mut result = validate_string_input(name, field_name, 100);

    // Environment variable names should be uppercase, alphanumeric with underscores
    lazy_static::lazy_static! {
        static ref ENV_VAR_PATTERN: Regex = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();
    }

    if !ENV_VAR_PATTERN.is_match(name) {
        result.errors.push(format!("{} is not a valid environment variable name (should be uppercase, start with letter, contain only alphanumeric and underscores)", field_name));
        result.is_valid = false;
    }

    result
}

/// Comprehensive input validation for API requests
pub fn validate_api_input(input: &serde_json::Value, context: &str) -> Result<()> {
    match input {
        serde_json::Value::String(s) => {
            let result = validate_string_input(s, context, MAX_STRING_LENGTH);
            if !result.is_valid {
                return Err(anyhow!("Input validation failed: {}", result.errors.join(", ")));
            }
        }
        serde_json::Value::Object(obj) => {
            // Validate each field in the object
            for (key, value) in obj {
                validate_api_input(value, &format!("{}.{}", context, key))?;
            }
        }
        serde_json::Value::Array(arr) => {
            // Validate each element in the array
            for (i, value) in arr.iter().enumerate() {
                validate_api_input(value, &format!("{}[{}]", context, i))?;
            }
        }
        _ => {} // Other JSON types (numbers, booleans, null) are generally safe
    }

    Ok(())
}

/// Batch validation for multiple inputs
pub fn validate_batch(inputs: Vec<(&str, &str, ValidationType)>) -> Vec<ValidationResult> {
    inputs.into_iter().map(|(input, field_name, validation_type)| {
        match validation_type {
            ValidationType::String(max_len) => validate_string_input(input, field_name, max_len),
            ValidationType::Identifier => validate_identifier(input, field_name),
            ValidationType::Email => validate_email(input, field_name),
            ValidationType::Url(allow_http) => validate_url(input, field_name, allow_http),
            ValidationType::Alphanumeric(max_len) => validate_alphanumeric(input, field_name, max_len),
            ValidationType::FilePath => validate_file_path(input, field_name),
            ValidationType::Numeric(min, max) => validate_numeric(input, field_name, min, max),
            ValidationType::Json(max_depth) => validate_json_input(input, field_name, max_depth),
            ValidationType::SqlSafe => validate_sql_safe(input, field_name),
        }
    }).collect()
}

/// Types of validation available
#[derive(Debug, Clone)]
pub enum ValidationType {
    String(usize),              // max_length
    Identifier,
    Email,
    Url(bool),                  // allow_http
    Alphanumeric(usize),        // max_length
    FilePath,
    Numeric(i64, i64),          // min, max
    Json(usize),                // max_depth
    SqlSafe,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_validation() {
        // Valid input
        let result = validate_string_input("hello world", "test_field", 100);
        assert!(result.is_valid);
        assert_eq!(result.sanitized_value.unwrap(), "hello world");

        // Too long
        let result = validate_string_input(&"x".repeat(200), "test_field", 100);
        assert!(!result.is_valid);
        assert!(result.errors[0].contains("exceeds maximum length"));
    }

    #[test]
    fn test_identifier_validation() {
        // Valid identifiers
        assert!(validate_identifier("valid_name", "field").is_valid);
        assert!(validate_identifier("_private", "field").is_valid);
        assert!(validate_identifier("MyClass123", "field").is_valid);

        // Invalid identifiers
        assert!(!validate_identifier("123invalid", "field").is_valid);
        assert!(!validate_identifier("invalid-name", "field").is_valid);
        assert!(!validate_identifier("null", "field").is_valid);
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("user@example.com", "email").is_valid);
        assert!(!validate_email("invalid-email", "email").is_valid);
        assert!(!validate_email("user@", "email").is_valid);
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("https://example.com", "url", false).is_valid);
        assert!(validate_url("http://example.com", "url", true).is_valid);
        assert!(!validate_url("http://example.com", "url", false).is_valid);
        assert!(!validate_url("not-a-url", "url", true).is_valid);
    }

    #[test]
    fn test_file_path_validation() {
        assert!(validate_file_path("safe/path/file.txt", "path").is_valid);
        assert!(!validate_file_path("../dangerous", "path").is_valid);
        assert!(!validate_file_path("/absolute/path", "path").is_valid);
        assert!(!validate_file_path("file<script>.txt", "path").is_valid);
    }

    #[test]
    fn test_json_validation() {
        assert!(validate_json_input(r#"{"key": "value"}"#, "json", 10).is_valid);
        assert!(validate_json_input(r#"{"nested": {"deep": "value"}}"#, "json", 10).is_valid);
        assert!(!validate_json_input(r#"{"nested": {"deep": "value"}}"#, "json", 1).is_valid);
        assert!(!validate_json_input(r#"invalid json"#, "json", 10).is_valid);
    }

    #[test]
    fn test_sql_injection_protection() {
        assert!(validate_sql_safe("normal input", "field").is_valid);
        assert!(!validate_sql_safe("DROP TABLE users", "field").is_valid);
        assert!(!validate_sql_safe("SELECT * FROM users WHERE 1=1", "field").is_valid);
    }
}
