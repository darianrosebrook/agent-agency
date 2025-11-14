//! Basic security validation tests for system-quality-security crate
//!
//! Tests core input validation functions that exist in the crate

use system_quality_security::*;

/// Test basic input validation functionality that exists
#[cfg(test)]
mod basic_input_validation_tests {
    use super::*;

    #[test]
    fn test_validate_string_input_valid() {
        let result = validate_string_input("hello world", "test_field", 100);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.sanitized_value.unwrap(), "hello world");
    }

    #[test]
    fn test_validate_string_input_too_long() {
        let long_input = "a".repeat(200);
        let result = validate_string_input(&long_input, "test_field", 100);

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("exceeds maximum length"));
    }

    #[test]
    fn test_validate_string_input_with_null_bytes() {
        let input_with_null = "hello\x00world";
        let result = validate_string_input(input_with_null, "test_field", 100);

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("contains null bytes"));
        assert_eq!(result.sanitized_value.unwrap(), "helloworld");
    }

    #[test]
    fn test_validate_identifier_valid() {
        let result = validate_identifier("valid_identifier_123", "test_field");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_identifier_starts_with_number() {
        let result = validate_identifier("123invalid", "test_field");

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("must start with a letter"));
    }

    #[test]
    fn test_validate_email_valid() {
        let result = validate_email("user@example.com", "email_field");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_email_invalid_format() {
        let result = validate_email("invalid-email", "email_field");

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("invalid email format"));
    }

    #[test]
    fn test_validate_url_valid() {
        let result = validate_url("https://example.com/path", "url_field", true);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_url_invalid_protocol() {
        let result = validate_url("ftp://example.com", "url_field", true);

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("must use HTTP or HTTPS"));
    }

    #[test]
    fn test_validate_json_input_valid() {
        let json = r#"{"key": "value", "number": 42}"#;
        let result = validate_json_input(json, "json_field", 10);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_json_input_malformed() {
        let malformed_json = r#"{"key": "unclosed string"#;
        let result = validate_json_input(malformed_json, "json_field", 10);

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("invalid JSON"));
    }

    #[test]
    fn test_validate_file_path_valid() {
        let result = validate_file_path("/path/to/file.txt", "file_path");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_file_path_invalid() {
        let result = validate_file_path("/path/<invalid>chars", "file_path");

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("contains invalid characters"));
    }

    #[test]
    fn test_validate_numeric_valid() {
        let result = validate_numeric("42", "number_field", 0i32, 100i32);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_numeric_out_of_range() {
        let result = validate_numeric("150", "number_field", 0i32, 100i32);

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("out of range"));
    }

    #[test]
    fn test_validate_sql_safe_valid() {
        let result = validate_sql_safe("SELECT * FROM users WHERE id = ?", "sql_field");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_sql_safe_dangerous() {
        let result = validate_sql_safe("SELECT * FROM users; DROP TABLE users;", "sql_field");

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("dangerous SQL"));
    }
}

/// Test file upload validation
#[cfg(test)]
mod file_upload_tests {
    use super::*;

    #[test]
    fn test_validate_file_upload_valid() {
        let allowed_types = &["application/pdf", "text/plain"];
        let result = validate_file_upload(
            "document.pdf",
            "application/pdf",
            1024 * 1024,
            allowed_types,
        );

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_file_upload_invalid_type() {
        let allowed_types = &["application/pdf"];
        let result = validate_file_upload(
            "script.exe",
            "application/x-msdownload",
            1024,
            allowed_types,
        );

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("file type not allowed"));
    }

    #[test]
    fn test_validate_file_upload_too_large() {
        let allowed_types = &["application/pdf"];
        let result = validate_file_upload(
            "large.pdf",
            "application/pdf",
            MAX_FILE_SIZE_BYTES + 1,
            allowed_types,
        );

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("file too large"));
    }
}

/// Test API validation functions
#[cfg(test)]
mod api_validation_tests {
    use super::*;

    #[test]
    fn test_validate_api_payload_valid() {
        let payload = r#"{"action": "update", "id": 123}"#;
        let result = validate_api_payload(payload, "application/json");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_api_payload_wrong_content_type() {
        let payload = r#"{"action": "update"}"#;
        let result = validate_api_payload(payload, "text/plain");

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("expected JSON"));
    }

    #[test]
    fn test_validate_query_params_valid() {
        let params = vec![
            ("action".to_string(), "list".to_string()),
            ("limit".to_string(), "10".to_string()),
        ];

        let result = validate_query_params(&params);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_query_params_too_long() {
        let long_param = "a".repeat(MAX_QUERY_PARAM_LENGTH + 1);
        let params = vec![("action".to_string(), long_param)];

        let result = validate_query_params(&params);

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("parameter too long"));
    }

    #[test]
    fn test_validate_http_headers_valid() {
        let headers = vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("authorization".to_string(), "Bearer token123".to_string()),
        ];

        let result = validate_http_headers(&headers);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_http_headers_too_long() {
        let long_value = "a".repeat(MAX_HEADER_VALUE_LENGTH + 1);
        let headers = vec![("x-custom".to_string(), long_value)];

        let result = validate_http_headers(&headers);

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("header value too long"));
    }
}

/// Test batch validation functionality
#[cfg(test)]
mod batch_validation_tests {
    use super::*;

    #[test]
    fn test_validate_batch_all_valid() {
        let inputs = vec![
            ("hello", "field1", ValidationType::Identifier),
            ("user@example.com", "field2", ValidationType::Email),
            ("https://example.com", "field3", ValidationType::Url(true)),
        ];

        let results = validate_batch(inputs);

        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_valid);
        }
    }

    #[test]
    fn test_validate_batch_some_invalid() {
        let inputs = vec![
            ("valid_name", "field1", ValidationType::Identifier),
            ("invalid-email", "field2", ValidationType::Email),
            ("123invalid", "field3", ValidationType::Identifier),
        ];

        let results = validate_batch(inputs);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_valid); // valid identifier
        assert!(!results[1].is_valid); // invalid email
        assert!(!results[2].is_valid); // invalid identifier
    }
}

/// Test environment variable validation
#[cfg(test)]
mod env_var_tests {
    use super::*;

    #[test]
    fn test_validate_env_var_name_valid() {
        let result = validate_env_var_name("DATABASE_URL", "env_var");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_env_var_name_invalid_chars() {
        let result = validate_env_var_name("INVALID-VAR", "env_var");

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("invalid characters"));
    }

    #[test]
    fn test_validate_env_var_name_starts_with_number() {
        let result = validate_env_var_name("1INVALID", "env_var");

        assert!(!result.is_valid);
        assert!(result.errors.len() == 1);
        assert!(result.errors[0].contains("must start with letter"));
    }
}

/// Test security constants and configuration
#[cfg(test)]
mod security_constants_tests {
    use super::*;

    #[test]
    fn test_security_constants_values() {
        // Test that security constants are reasonable and defined
        assert!(MAX_STRING_LENGTH > 1000);
        assert!(MAX_IDENTIFIER_LENGTH > 50);
        assert!(MAX_URL_LENGTH > 100);
        assert!(MAX_EMAIL_LENGTH > 50);
        assert!(MAX_FILE_SIZE_BYTES > 1024 * 1024); // At least 1MB
        assert!(MAX_JSON_PAYLOAD_SIZE > 1000);
        assert!(MAX_FORM_DATA_SIZE > 1024 * 1024); // At least 1MB
        assert!(MAX_QUERY_PARAM_LENGTH > 100);
        assert!(MAX_HEADER_VALUE_LENGTH > 1000);
    }

    #[test]
    fn test_allowed_content_types() {
        // Test that allowed content type arrays are properly defined
        assert!(ALLOWED_IMAGE_TYPES.contains(&"image/jpeg"));
        assert!(ALLOWED_IMAGE_TYPES.contains(&"image/png"));
        assert!(ALLOWED_DOCUMENT_TYPES.contains(&"application/pdf"));
        assert!(ALLOWED_DOCUMENT_TYPES.contains(&"text/plain"));
    }

    #[test]
    fn test_validation_type_enum() {
        // Test that ValidationType enum variants exist with proper constructors
        let types = vec![
            ValidationType::String(100),
            ValidationType::Identifier,
            ValidationType::Email,
            ValidationType::Url(true),
            ValidationType::Alphanumeric(50),
            ValidationType::FilePath,
            ValidationType::Numeric(0, 100),
            ValidationType::Json(10),
            ValidationType::SqlSafe,
        ];

        assert_eq!(types.len(), 9); // Should have all expected variants
    }
}

/// Integration test for comprehensive validation pipeline
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_comprehensive_validation_pipeline() {
        // Test a complete validation pipeline for a user registration scenario

        // 1. Username validation
        let username_result = validate_identifier("validuser123", "username");
        assert!(username_result.is_valid);

        // 2. Email validation
        let email_result = validate_email("user@example.com", "email");
        assert!(email_result.is_valid);

        // 3. Password strength (basic length check via string validation)
        let password_result = validate_string_input("strongpassword", "password", 100);
        assert!(password_result.is_valid);

        // 4. Environment variable validation (for config)
        let env_result = validate_env_var_name("APP_ENV", "environment");
        assert!(env_result.is_valid);

        // 5. URL validation (for profile picture)
        let url_result = validate_url("https://example.com/avatar.jpg", "avatar_url", true);
        assert!(url_result.is_valid);

        // All validations should pass for a valid user registration
        assert!(username_result.is_valid);
        assert!(email_result.is_valid);
        assert!(password_result.is_valid);
        assert!(env_result.is_valid);
        assert!(url_result.is_valid);
    }

    #[test]
    fn test_validation_error_messages_helpful() {
        // Test that error messages are descriptive and helpful for debugging

        // Test various invalid inputs and check error messages
        let invalid_username = validate_identifier("123invalid", "username");
        assert!(!invalid_username.is_valid);
        assert!(invalid_username.errors[0].contains("username"));
        assert!(invalid_username.errors[0].contains("start with a letter"));

        let invalid_email = validate_email("not-an-email", "user_email");
        assert!(!invalid_email.is_valid);
        assert!(invalid_email.errors[0].contains("user_email"));
        assert!(invalid_email.errors[0].contains("invalid email format"));

        let too_long_input = validate_string_input(&"a".repeat(200), "description", 100);
        assert!(!too_long_input.is_valid);
        assert!(too_long_input.errors[0].contains("description"));
        assert!(too_long_input.errors[0].contains("maximum length"));
        assert!(too_long_input.errors[0].contains("100"));
    }
}
