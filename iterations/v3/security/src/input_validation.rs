//! Input validation utilities for external data

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashSet;

/// Maximum allowed length for various input types
pub const MAX_STRING_LENGTH: usize = 10_000;
pub const MAX_IDENTIFIER_LENGTH: usize = 100;
pub const MAX_URL_LENGTH: usize = 2048;
pub const MAX_EMAIL_LENGTH: usize = 254;

/// File upload limits
pub const MAX_FILE_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_FILENAME_LENGTH: usize = 255;

/// API payload limits
pub const MAX_JSON_PAYLOAD_SIZE: usize = 1024 * 1024; // 1MB
pub const MAX_FORM_DATA_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_QUERY_PARAM_LENGTH: usize = 1000;
pub const MAX_HEADER_VALUE_LENGTH: usize = 8192;

/// Content type validation
pub const ALLOWED_IMAGE_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];
pub const ALLOWED_DOCUMENT_TYPES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "text/plain",
    "text/csv",
];

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
    static ref SAFE_PATH_PATTERN: Regex = Regex::new(r#"^[^<>"'|?*\[\]{}\\,;:%$#]*$"#).unwrap();
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

/// Validate file upload data
pub fn validate_file_upload(
    filename: &str,
    content_type: &str,
    size_bytes: usize,
    allowed_types: &[&str],
) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        errors: Vec::new(),
        sanitized_value: Some(filename.to_string()),
    };

    // Filename validation
    if filename.len() > MAX_FILENAME_LENGTH {
        result.errors.push(format!(
            "Filename exceeds maximum length of {} characters",
            MAX_FILENAME_LENGTH
        ));
        result.is_valid = false;
    }

    // Check for dangerous filename patterns
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        result.errors.push("Filename contains dangerous path traversal patterns".to_string());
        result.is_valid = false;
    }

    // Check for script injection in filename
    if filename.contains('<') || filename.contains('>') || filename.contains('"') ||
       filename.contains('\'') || filename.contains('|') {
        result.errors.push("Filename contains potentially dangerous characters".to_string());
        result.is_valid = false;
    }

    // File size validation
    if size_bytes > MAX_FILE_SIZE_BYTES {
        result.errors.push(format!(
            "File size {} bytes exceeds maximum allowed size of {} bytes",
            size_bytes, MAX_FILE_SIZE_BYTES
        ));
        result.is_valid = false;
    }

    // Content type validation
    if !allowed_types.is_empty() && !allowed_types.contains(&content_type) {
        result.errors.push(format!(
            "Content type '{}' is not allowed. Allowed types: {:?}",
            content_type, allowed_types
        ));
        result.is_valid = false;
    }

    // MIME type format validation
    if !content_type.contains('/') || content_type.len() > 100 {
        result.errors.push("Invalid content type format".to_string());
        result.is_valid = false;
    }

    result
}

/// Validate API payload size and structure
pub fn validate_api_payload(payload: &str, content_type: &str) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        errors: Vec::new(),
        sanitized_value: Some(payload.to_string()),
    };

    // Size validation based on content type
    let max_size = match content_type {
        "application/json" => MAX_JSON_PAYLOAD_SIZE,
        "application/x-www-form-urlencoded" | "multipart/form-data" => MAX_FORM_DATA_SIZE,
        _ => MAX_STRING_LENGTH,
    };

    if payload.len() > max_size {
        result.errors.push(format!(
            "Payload size {} bytes exceeds maximum allowed size of {} bytes for content type {}",
            payload.len(), max_size, content_type
        ));
        result.is_valid = false;
    }

    // JSON structure validation
    if content_type == "application/json" {
        if let Err(e) = serde_json::from_str::<serde_json::Value>(payload) {
            result.errors.push(format!("Invalid JSON structure: {}", e));
            result.is_valid = false;
        }

        // Additional JSON security checks
        if payload.contains('\0') {
            result.errors.push("JSON payload contains null bytes".to_string());
            result.is_valid = false;
        }

        // Check for extremely nested structures (potential DoS)
        let nesting_depth = count_json_nesting(payload);
        if nesting_depth > 10 {
            result.errors.push("JSON payload has excessive nesting depth".to_string());
            result.is_valid = false;
        }
    }

    result
}

/// Validate query parameters
pub fn validate_query_params(params: &[(String, String)]) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        errors: Vec::new(),
        sanitized_value: None,
    };

    for (key, value) in params {
        // Key validation
        if key.len() > 100 {
            result.errors.push(format!("Query parameter key '{}' is too long", key));
            result.is_valid = false;
        }

        if !IDENTIFIER_PATTERN.is_match(key) {
            result.errors.push(format!("Query parameter key '{}' contains invalid characters", key));
            result.is_valid = false;
        }

        // Value validation
        if value.len() > MAX_QUERY_PARAM_LENGTH {
            result.errors.push(format!("Query parameter '{}' value is too long", key));
            result.is_valid = false;
        }

        // Check for injection patterns
        if value.contains('<') || value.contains('>') || value.contains('"') ||
           value.contains('\'') || value.contains('|') || value.contains(';') {
            result.errors.push(format!("Query parameter '{}' contains potentially dangerous characters", key));
            result.is_valid = false;
        }
    }

    result
}

/// Validate HTTP headers
pub fn validate_http_headers(headers: &[(String, String)]) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        errors: Vec::new(),
        sanitized_value: None,
    };

    for (key, value) in headers {
        // Header name validation
        if key.len() > 100 {
            result.errors.push(format!("Header name '{}' is too long", key));
            result.is_valid = false;
        }

        // RFC 7230 header name validation (token characters)
        if !key.chars().all(|c| c.is_ascii() && (c.is_alphanumeric() || c == '-' || c == '_')) {
            result.errors.push(format!("Header name '{}' contains invalid characters", key));
            result.is_valid = false;
        }

        // Header value validation
        if value.len() > MAX_HEADER_VALUE_LENGTH {
            result.errors.push(format!("Header '{}' value is too long", key));
            result.is_valid = false;
        }

        // Check for header injection
        if value.contains('\r') || value.contains('\n') {
            result.errors.push(format!("Header '{}' contains CRLF characters", key));
            result.is_valid = false;
        }

        // Check for control characters
        if value.chars().any(|c| c.is_control() && c != '\t') {
            result.errors.push(format!("Header '{}' contains control characters", key));
            result.is_valid = false;
        }
    }

    result
}

/// Count JSON nesting depth to prevent DoS attacks
fn count_json_nesting(json_str: &str) -> usize {
    let mut max_depth = 0;
    let mut current_depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for c in json_str.chars() {
        match c {
            '"' if !escaped => in_string = !in_string,
            '{' | '[' if !in_string => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            '}' | ']' if !in_string => {
                current_depth = current_depth.saturating_sub(1);
            }
            '\\' if in_string => escaped = !escaped,
            _ => escaped = false,
        }
    }

    max_depth
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

    #[test]
    fn test_file_upload_validation() {
        // Valid file upload
        let result = validate_file_upload(
            "document.pdf",
            "application/pdf",
            1024 * 1024, // 1MB
            ALLOWED_DOCUMENT_TYPES,
        );
        assert!(result.is_valid);

        // Invalid filename (path traversal)
        let result = validate_file_upload(
            "../../../etc/passwd",
            "text/plain",
            100,
            ALLOWED_DOCUMENT_TYPES,
        );
        assert!(!result.is_valid);

        // Invalid file size
        let result = validate_file_upload(
            "large_file.pdf",
            "application/pdf",
            MAX_FILE_SIZE_BYTES + 1,
            ALLOWED_DOCUMENT_TYPES,
        );
        assert!(!result.is_valid);

        // Invalid content type
        let result = validate_file_upload(
            "script.exe",
            "application/x-msdownload",
            100,
            ALLOWED_DOCUMENT_TYPES,
        );
        assert!(!result.is_valid);
    }

    #[test]
    fn test_api_payload_validation() {
        // Valid JSON payload
        let result = validate_api_payload(
            r#"{"key": "value", "number": 42}"#,
            "application/json"
        );
        assert!(result.is_valid);

        // Invalid JSON
        let result = validate_api_payload(
            r#"{"invalid": json}"#,
            "application/json"
        );
        assert!(!result.is_valid);

        // Oversized payload
        let large_payload = "x".repeat(MAX_JSON_PAYLOAD_SIZE + 1);
        let result = validate_api_payload(&large_payload, "application/json");
        assert!(!result.is_valid);

        // JSON with null bytes (security issue)
        let result = validate_api_payload(
            "{\"key\": \"value\\0bad\"}",
            "application/json"
        );
        assert!(!result.is_valid);
    }

    #[test]
    fn test_query_param_validation() {
        // Valid parameters
        let params = vec![
            ("user_id".to_string(), "123".to_string()),
            ("search".to_string(), "query".to_string()),
        ];
        let result = validate_query_params(&params);
        assert!(result.is_valid);

        // Invalid key characters
        let params = vec![
            ("user-id".to_string(), "123".to_string()), // hyphens not allowed in our pattern
        ];
        let result = validate_query_params(&params);
        assert!(!result.is_valid);

        // Oversized value
        let large_value = "x".repeat(MAX_QUERY_PARAM_LENGTH + 1);
        let params = vec![
            ("param".to_string(), large_value),
        ];
        let result = validate_query_params(&params);
        assert!(!result.is_valid);

        // Dangerous characters
        let params = vec![
            ("param".to_string(), "value<script>".to_string()),
        ];
        let result = validate_query_params(&params);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_http_header_validation() {
        // Valid headers
        let headers = vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("authorization".to_string(), "Bearer token".to_string()),
        ];
        let result = validate_http_headers(&headers);
        assert!(result.is_valid);

        // Invalid header name
        let headers = vec![
            ("content type".to_string(), "application/json".to_string()), // spaces not allowed
        ];
        let result = validate_http_headers(&headers);
        assert!(!result.is_valid);

        // Oversized header value
        let large_value = "x".repeat(MAX_HEADER_VALUE_LENGTH + 1);
        let headers = vec![
            ("x-custom".to_string(), large_value),
        ];
        let result = validate_http_headers(&headers);
        assert!(!result.is_valid);

        // CRLF injection
        let headers = vec![
            ("x-header".to_string(), "value\r\ninjected".to_string()),
        ];
        let result = validate_http_headers(&headers);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_json_nesting_depth() {
        // Shallow nesting
        let json = r#"{"a": {"b": {"c": "value"}}}"#;
        let result = validate_api_payload(json, "application/json");
        assert!(result.is_valid);

        // Deep nesting (DoS protection)
        let deep_json = (0..12).fold("x".to_string(), |acc, _| format!(r#"{{"nested": {}}}"#, acc));
        let result = validate_api_payload(&deep_json, "application/json");
        assert!(!result.is_valid);
    }
}
