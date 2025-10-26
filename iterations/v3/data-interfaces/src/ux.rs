//! User Experience Module
//!
//! UX utilities for consistent user interaction patterns.

/// Format a success message
pub fn format_success(message: &str) -> String {
    format!("✅ {}", message)
}

/// Format an error message
pub fn format_error(message: &str) -> String {
    format!("❌ {}", message)
}

/// Format an info message
pub fn format_info(message: &str) -> String {
    format!("ℹ️ {}", message)
}
