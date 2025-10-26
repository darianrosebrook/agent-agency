//! Quality gate checks that require global analysis

use crate::config::{QualityGateConfig, QualityViolation, Severity};
use regex::Regex;
use std::collections::HashMap;

/// Analyze duplicate struct names across all files
pub fn check_duplicate_names(files: &HashMap<String, String>, config: &QualityGateConfig) -> Vec<QualityViolation> {
    let mut violations = Vec::new();
    let mut name_counts: HashMap<String, Vec<String>> = HashMap::new();

    let struct_regex = Regex::new(r"(?:pub\s+)?(?:struct|enum)\s+(\w+)").unwrap();

    for (file_path, content) in files {
        // Skip excluded files
        if should_exclude_file(file_path, config) {
            continue;
        }

        for line in content.lines() {
            if let Some(captures) = struct_regex.captures(line) {
                let name = captures.get(1).unwrap().as_str().to_string();
                name_counts.entry(name).or_insert_with(Vec::new).push(file_path.clone());
            }
        }
    }

    for (name, files_with_name) in name_counts {
        let count = files_with_name.len();
        if count > config.max_duplicate_names {
            violations.push(QualityViolation {
                rule: "duplicate-names".to_string(),
                severity: Severity::Warning,
                file: files_with_name.join(", "),
                line: None,
                column: None,
                message: format!("Name '{}' is duplicated {} times across {} files", name, count, files_with_name.len()),
                suggestion: Some("Consider using a common type from agent-agency-common-types or renaming for clarity".to_string()),
                details: Some({
                    let mut details = HashMap::new();
                    details.insert("name".to_string(), name.clone().into());
                    details.insert("count".to_string(), count.into());
                    details.insert("files".to_string(), files_with_name.clone().into());
                    details
                }),
            });
        }
    }

    violations
}

/// Check for architectural violations
pub fn check_architecture_violations(files: &HashMap<String, String>, _config: &QualityGateConfig) -> Vec<QualityViolation> {
    let mut violations = Vec::new();

    // Check for direct database access in API handlers
    let db_access_regex = Regex::new(r"sqlx|diesel|rusqlite").unwrap();

    for (file_path, content) in files {
        if file_path.contains("api") || file_path.contains("handler") {
            if db_access_regex.is_match(content) {
                violations.push(QualityViolation {
                    rule: "architecture-violation".to_string(),
                    severity: Severity::Error,
                    file: file_path.clone(),
                    line: None,
                    column: None,
                    message: "API handlers should not access database directly".to_string(),
                    suggestion: Some("Use repository pattern or service layer for data access".to_string()),
                    details: Some({
                        let mut details = HashMap::new();
                        details.insert("violation_type".to_string(), "direct_db_access".into());
                        details.insert("layer".to_string(), "api".into());
                        details
                    }),
                });
            }
        }
    }

    violations
}

/// Check for security violations
pub fn check_security_violations(files: &HashMap<String, String>, _config: &QualityGateConfig) -> Vec<QualityViolation> {
    let mut violations = Vec::new();

    // Check for hardcoded secrets
    let secret_regex = Regex::new(r#"(?i)(password|secret|key|token)\s*[:=]\s*["'][^"']+["']"#).unwrap();
    // Check for unsafe code blocks
    let unsafe_regex = Regex::new(r"unsafe\s*\{").unwrap();

    for (file_path, content) in files {
        // Check for hardcoded secrets
        if secret_regex.is_match(content) {
            violations.push(QualityViolation {
                rule: "security-violation".to_string(),
                severity: Severity::Error,
                file: file_path.clone(),
                line: None,
                column: None,
                message: "Potential hardcoded secret detected".to_string(),
                suggestion: Some("Use environment variables or secure credential storage".to_string()),
                details: Some({
                    let mut details = HashMap::new();
                    details.insert("violation_type".to_string(), "hardcoded_secret".into());
                    details
                }),
            });
        }

        // Check for unsafe blocks
        if unsafe_regex.is_match(content) {
            violations.push(QualityViolation {
                rule: "security-violation".to_string(),
                severity: Severity::Warning,
                file: file_path.clone(),
                line: None,
                column: None,
                message: "Unsafe code block detected".to_string(),
                suggestion: Some("Review unsafe block for security implications".to_string()),
                details: Some({
                    let mut details = HashMap::new();
                    details.insert("violation_type".to_string(), "unsafe_code".into());
                    details
                }),
            });
        }
    }

    violations
}

/// Check for dependency violations
pub fn check_dependency_violations(files: &HashMap<String, String>, _config: &QualityGateConfig) -> Vec<QualityViolation> {
    let mut violations = Vec::new();

    for (file_path, content) in files {
        // Check for direct HTTP calls in business logic
        if file_path.contains("src/") && !file_path.contains("api") && !file_path.contains("client") {
            if content.contains("reqwest::") || content.contains("hyper::") {
                violations.push(QualityViolation {
                    rule: "dependency-violation".to_string(),
                    severity: Severity::Warning,
                    file: file_path.clone(),
                    line: None,
                    column: None,
                    message: "Business logic should not make direct HTTP calls".to_string(),
                    suggestion: Some("Use a dedicated HTTP client service or repository".to_string()),
                    details: Some({
                        let mut details = HashMap::new();
                        details.insert("violation_type".to_string(), "direct_http".into());
                        details.insert("layer".to_string(), "business_logic".into());
                        details
                    }),
                });
            }
        }
    }

    violations
}

/// Helper function to check if a file should be excluded
fn should_exclude_file(file_path: &str, config: &QualityGateConfig) -> bool {
    // Check directory exclusions
    for dir in &config.exclude_dirs {
        if file_path.starts_with(dir) {
            return true;
        }
    }

    // Check pattern exclusions
    for pattern in &config.exclude_patterns {
        if file_path.contains(pattern) {
            return true;
        }
    }

    false
}
