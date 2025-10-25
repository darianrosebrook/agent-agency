//! Common validation patterns and utilities

use crate::types::{ValidationResult, ConfigValidationResult, ConfigError};
use regex::Regex;
use std::collections::HashSet;

/// Common validation functions
pub struct Validators;

impl Validators {
    /// Validate email format
    pub fn validate_email(email: &str) -> ValidationResult {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if email.is_empty() {
            return ValidationResult::failure(vec!["Email cannot be empty".to_string()]);
        }

        if !email_regex.is_match(email) {
            return ValidationResult::failure(vec!["Invalid email format".to_string()]);
        }

        if email.len() > 254 {
            return ValidationResult::failure(vec!["Email too long (max 254 characters)".to_string()]);
        }

        ValidationResult::success()
    }

    /// Validate UUID format
    pub fn validate_uuid(uuid: &str) -> ValidationResult {
        let uuid_regex = Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
        ).unwrap();

        if uuid.is_empty() {
            return ValidationResult::failure(vec!["UUID cannot be empty".to_string()]);
        }

        if !uuid_regex.is_match(uuid) {
            return ValidationResult::failure(vec!["Invalid UUID format".to_string()]);
        }

        ValidationResult::success()
    }

    /// Validate URL format
    pub fn validate_url(url: &str) -> ValidationResult {
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();

        if url.is_empty() {
            return ValidationResult::failure(vec!["URL cannot be empty".to_string()]);
        }

        if !url_regex.is_match(url) {
            return ValidationResult::failure(vec!["Invalid URL format".to_string()]);
        }

        if url.len() > 2048 {
            return ValidationResult::failure(vec!["URL too long (max 2048 characters)".to_string()]);
        }

        ValidationResult::success()
    }

    /// Validate string length
    pub fn validate_string_length(value: &str, min: usize, max: usize, field_name: &str) -> ValidationResult {
        let mut errors = Vec::new();

        if value.len() < min {
            errors.push(format!("{} too short (minimum {} characters)", field_name, min));
        }

        if value.len() > max {
            errors.push(format!("{} too long (maximum {} characters)", field_name, max));
        }

        if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(errors)
        }
    }

    /// Validate numeric range
    pub fn validate_numeric_range<T: PartialOrd + std::fmt::Display>(
        value: T,
        min: T,
        max: T,
        field_name: &str
    ) -> ValidationResult {
        let mut errors = Vec::new();

        if value < min {
            errors.push(format!("{} too small (minimum {})", field_name, min));
        }

        if value > max {
            errors.push(format!("{} too large (maximum {})", field_name, max));
        }

        if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(errors)
        }
    }

    /// Validate required field
    pub fn validate_required<T>(value: &Option<T>, field_name: &str) -> ValidationResult {
        match value {
            Some(_) => ValidationResult::success(),
            None => ValidationResult::failure(vec![format!("{} is required", field_name)]),
        }
    }

    /// Validate collection not empty
    pub fn validate_not_empty<T>(collection: &[T], field_name: &str) -> ValidationResult {
        if collection.is_empty() {
            ValidationResult::failure(vec![format!("{} cannot be empty", field_name)])
        } else {
            ValidationResult::success()
        }
    }

    /// Validate unique values in collection
    pub fn validate_unique<T: std::hash::Hash + Eq>(collection: &[T], field_name: &str) -> ValidationResult {
        let mut seen = HashSet::new();
        for item in collection {
            if !seen.insert(item) {
                return ValidationResult::failure(vec![format!("{} contains duplicate values", field_name)]);
            }
        }
        ValidationResult::success()
    }

    /// Combine multiple validation results
    pub fn combine_results(results: Vec<ValidationResult>) -> ValidationResult {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut is_valid = true;

        for result in results {
            if !result.is_valid {
                is_valid = false;
            }
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }

        ValidationResult {
            is_valid,
            errors: all_errors,
            warnings: all_warnings,
            validated_at: chrono::Utc::now(),
        }
    }
}

/// Configuration validator helper
pub struct ConfigValidators;

impl ConfigValidators {
    /// Validate database configuration
    pub fn validate_database_config(host: &str, port: u16, database: &str, username: &str) -> ConfigValidationResult {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Host validation
        if host.is_empty() {
            errors.push(ConfigError {
                field: "host".to_string(),
                message: "Database host cannot be empty".to_string(),
                suggestion: Some("Set DATABASE_HOST environment variable".to_string()),
            });
        }

        // Port validation
        if port == 0 {
            errors.push(ConfigError {
                field: "port".to_string(),
                message: "Database port cannot be zero".to_string(),
                suggestion: Some("Set DATABASE_PORT environment variable".to_string()),
            });
        }

        // Database name validation
        if database.is_empty() {
            errors.push(ConfigError {
                field: "database".to_string(),
                message: "Database name cannot be empty".to_string(),
                suggestion: Some("Set DATABASE_NAME environment variable".to_string()),
            });
        }

        // Username validation
        if username.is_empty() {
            errors.push(ConfigError {
                field: "username".to_string(),
                message: "Database username cannot be empty".to_string(),
                suggestion: Some("Set DATABASE_USER environment variable".to_string()),
            });
        }

        ConfigValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Validate API configuration
    pub fn validate_api_config(host: &str, port: u16, cors_origins: &[String]) -> ConfigValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Host validation
        if host.is_empty() {
            errors.push(ConfigError {
                field: "host".to_string(),
                message: "API host cannot be empty".to_string(),
                suggestion: Some("Set API_HOST environment variable".to_string()),
            });
        }

        // Port validation
        if port == 0 {
            errors.push(ConfigError {
                field: "port".to_string(),
                message: "API port cannot be zero".to_string(),
                suggestion: Some("Set API_PORT environment variable".to_string()),
            });
        }

        // CORS validation
        if cors_origins.is_empty() {
            warnings.push("No CORS origins configured - API may be inaccessible from browsers".to_string());
        }

        ConfigValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}
