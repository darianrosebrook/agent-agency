//! Input sanitization utilities for cleaning external data

use regex::Regex;

/// Sanitize HTML content by escaping dangerous tags
pub fn sanitize_html(input: &str) -> String {
    lazy_static::lazy_static! {
        static ref HTML_PATTERN: Regex = Regex::new(r"<[^>]*>").unwrap();
    }

    HTML_PATTERN.replace_all(input, "").to_string()
}

/// Sanitize SQL input by escaping quotes (basic protection)
pub fn sanitize_sql(input: &str) -> String {
    input
        .replace("'", "''")
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
}

/// Sanitize filename by removing dangerous characters
pub fn sanitize_filename(input: &str) -> String {
    lazy_static::lazy_static! {
        static ref FILENAME_PATTERN: Regex = Regex::new(r"[<>:\|?*\[\]{}\/\\]").unwrap();
    }

    let sanitized = FILENAME_PATTERN.replace_all(input, "_");

    // Ensure it doesn't start or end with dots/spaces
    let trimmed = sanitized.trim_matches(|c: char| c == '.' || c.is_whitespace());

    // Limit length
    if trimmed.len() > 255 {
        format!("{}.txt", &trimmed[..251])
    } else if trimmed.is_empty() {
        "unnamed.txt".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Sanitize URL by removing dangerous protocols and characters
pub fn sanitize_url(input: &str) -> String {
    let mut sanitized = input.to_string();

    // Remove dangerous protocols
    let dangerous_protocols = ["javascript:", "vbscript:", "data:", "file:"];
    for protocol in &dangerous_protocols {
        if sanitized.to_lowercase().starts_with(protocol) {
            return "#".to_string(); // Safe fallback
        }
    }

    // Remove control characters
    sanitized = sanitized.chars()
        .filter(|c| !c.is_control())
        .collect();

    sanitized
}

/// Sanitize JSON string values by escaping control characters
pub fn sanitize_json_string(input: &str) -> String {
    input
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
        .chars()
        .filter(|c| c.is_ascii() || c as u32 <= 0xFFFF) // Filter out non-BMP characters
        .collect()
}

/// Sanitize log messages by removing sensitive patterns
pub fn sanitize_log_message(input: &str, sensitive_patterns: &[&str]) -> String {
    let mut sanitized = input.to_string();

    for pattern in sensitive_patterns {
        // Simple pattern replacement - replace sensitive content with [REDACTED]
        if sanitized.to_lowercase().contains(&pattern.to_lowercase()) {
            sanitized = sanitized.replace(pattern, "[REDACTED]");
        }
    }

    sanitized
}

/// Sanitize user input for logging (remove newlines, limit length)
pub fn sanitize_for_logging(input: &str, max_length: usize) -> String {
    let sanitized = input
        .replace("\n", " ")
        .replace("\r", " ")
        .replace("\t", " ");

    if sanitized.len() > max_length {
        format!("{}...", &sanitized[..max_length.saturating_sub(3)])
    } else {
        sanitized
    }
}

/// Sanitize command-line arguments by escaping shell metacharacters
pub fn sanitize_shell_arg(input: &str) -> String {
    lazy_static::lazy_static! {
        static ref SHELL_META_PATTERN: Regex = Regex::new(r"[\$`\(\)\[\]\{\}\*\?\+\^\|]").unwrap();
    }

    let escaped = SHELL_META_PATTERN.replace_all(input, "\\$0");

    // Quote if it contains spaces or special chars
    if input.contains(' ') || input.contains('"') || input.contains('\'') {
        format!("'{}'", escaped)
    } else {
        escaped.to_string()
    }
}

/// Comprehensive input sanitization for API requests
pub fn sanitize_api_input(input: &serde_json::Value) -> serde_json::Value {
    match input {
        serde_json::Value::String(s) => {
            // Sanitize string content
            let sanitized = sanitize_json_string(s);
            serde_json::Value::String(sanitized)
        }
        serde_json::Value::Object(obj) => {
            let mut sanitized_obj = serde_json::Map::new();
            for (key, value) in obj {
                // Sanitize keys (remove dangerous characters)
                let safe_key = key.chars()
                    .filter(|c| c.is_alphanumeric() || c == '_' || c == '-')
                    .collect::<String>();
                sanitized_obj.insert(safe_key, sanitize_api_input(value));
            }
            serde_json::Value::Object(sanitized_obj)
        }
        serde_json::Value::Array(arr) => {
            let sanitized_arr = arr.iter()
                .map(|v| sanitize_api_input(v))
                .collect::<Vec<_>>();
            serde_json::Value::Array(sanitized_arr)
        }
        other => other.clone(), // Numbers, booleans, null are safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_sanitization() {
        assert_eq!(sanitize_html("<script>alert('xss')</script>"), "alert('xss')");
        assert_eq!(sanitize_html("normal text"), "normal text");
        assert_eq!(sanitize_html("<b>bold</b> text"), "bold text");
    }

    #[test]
    fn test_filename_sanitization() {
        assert_eq!(sanitize_filename("safe_file.txt"), "safe_file.txt");
        assert_eq!(sanitize_filename("file<>:|?*.txt"), "file_____.txt");
        assert_eq!(sanitize_filename(""), "unnamed.txt");
        assert_eq!(sanitize_filename(&"a".repeat(300)), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.txt");
    }

    #[test]
    fn test_url_sanitization() {
        assert_eq!(sanitize_url("https://example.com"), "https://example.com");
        assert_eq!(sanitize_url("javascript:alert('xss')"), "#");
        assert_eq!(sanitize_url("vbscript:msgbox('xss')"), "#");
        assert_eq!(sanitize_url("http://example.com\n"), "http://example.com");
    }

    #[test]
    fn test_json_string_sanitization() {
        assert_eq!(sanitize_json_string("normal"), "normal");
        assert_eq!(sanitize_json_string("quote\"here"), "quote\\\"here");
        assert_eq!(sanitize_json_string("backslash\\here"), "backslash\\\\here");
        assert_eq!(sanitize_json_string("new\nline"), "new\\nline");
    }

    #[test]
    fn test_log_message_sanitization() {
        let patterns = &["password", "secret", "token"];
        assert_eq!(
            sanitize_log_message("user=john password=secret123", patterns),
            "user=john [REDACTED]=[REDACTED]"
        );
        assert_eq!(
            sanitize_log_message("token=abc123 normal=text", patterns),
            "[REDACTED]=abc123 normal=text"
        );
    }

    #[test]
    fn test_shell_arg_sanitization() {
        assert_eq!(sanitize_shell_arg("simple"), "simple");
        assert_eq!(sanitize_shell_arg("has spaces"), "'has spaces'");
        assert_eq!(sanitize_shell_arg("$HOME"), "'\\$HOME'");
        assert_eq!(sanitize_shell_arg("normal_arg"), "normal_arg");
    }

    #[test]
    fn test_api_input_sanitization() {
        let input = serde_json::json!({
            "name": "test<script>alert('xss')</script>",
            "values": ["safe", "also<script>bad</script>safe"],
            "nested": {
                "key<script>": "value"
            }
        });

        let sanitized = sanitize_api_input(&input);

        if let serde_json::Value::Object(obj) = sanitized {
            assert_eq!(obj["name"], "testalert('xss')");
            assert_eq!(obj["values"][0], "safe");
            assert_eq!(obj["values"][1], "alsobadsafe");
            assert_eq!(obj["nested"]["key"], "value");
        } else {
            panic!("Expected object");
        }
    }
}
