//! Validation Module
//!
//! Input validation utilities for interfaces.

use crate::InterfaceError;

/// Validate a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), InterfaceError> {
    if value.trim().is_empty() {
        return Err(InterfaceError::ConfigurationError(
            format!("{} cannot be empty", field_name)
        ));
    }
    Ok(())
}

/// Validate a number is positive
pub fn validate_positive(value: i64, field_name: &str) -> Result<(), InterfaceError> {
    if value <= 0 {
        return Err(InterfaceError::ConfigurationError(
            format!("{} must be positive", field_name)
        ));
    }
    Ok(())
}
