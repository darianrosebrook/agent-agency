//! Test fixture for mutation testing scenario
//!
//! Provides JSON schema validation specification and test cases
//! for implementing and testing a schema validator with mutation testing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON Schema validation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    #[serde(rename = "$schema")]
    pub schema_version: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub properties: Option<HashMap<String, JsonSchemaProperty>>,
    pub required: Option<Vec<String>>,
    pub items: Option<Box<JsonSchema>>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    pub enum_values: Option<Vec<serde_json::Value>>,
}

/// Schema property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaProperty {
    #[serde(rename = "type")]
    pub property_type: Option<String>,
    pub description: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    pub required: Option<bool>,
}

/// Test case for schema validation
#[derive(Debug, Clone)]
pub struct ValidationTestCase {
    pub name: String,
    pub input: serde_json::Value,
    pub expected_valid: bool,
    pub description: String,
}

/// Get the user profile schema specification
pub fn get_user_profile_schema() -> JsonSchema {
    JsonSchema {
        schema_version: Some("https://json-schema.org/draft/2020-12/schema".to_string()),
        title: Some("User Profile".to_string()),
        description: Some("Schema for user profile validation".to_string()),
        schema_type: Some("object".to_string()),
        properties: Some({
            let mut props = HashMap::new();

            // Name property
            props.insert("name".to_string(), JsonSchemaProperty {
                property_type: Some("string".to_string()),
                description: Some("User's full name".to_string()),
                minimum: None,
                maximum: None,
                min_length: Some(2),
                max_length: Some(100),
                pattern: Some(r"^[a-zA-Z\s]+$".to_string()),
                format: None,
                required: None,
            });

            // Age property
            props.insert("age".to_string(), JsonSchemaProperty {
                property_type: Some("integer".to_string()),
                description: Some("User's age in years".to_string()),
                minimum: Some(0.0),
                maximum: Some(150.0),
                min_length: None,
                max_length: None,
                pattern: None,
                format: None,
                required: None,
            });

            // Email property
            props.insert("email".to_string(), JsonSchemaProperty {
                property_type: Some("string".to_string()),
                description: Some("User's email address".to_string()),
                minimum: None,
                maximum: None,
                min_length: Some(5),
                max_length: Some(254),
                pattern: Some(r"^[^@]+@[^@]+\.[^@]+$".to_string()),
                format: Some("email".to_string()),
                required: None,
            });

            // Tags property (array of strings)
            props.insert("tags".to_string(), JsonSchemaProperty {
                property_type: Some("array".to_string()),
                description: Some("User tags/keywords".to_string()),
                minimum: None,
                maximum: None,
                min_length: None,
                max_length: None,
                pattern: None,
                format: None,
                required: None,
            });

            props
        }),
        required: Some(vec!["name".to_string(), "email".to_string()]),
        items: None,
        minimum: None,
        maximum: None,
        min_length: None,
        max_length: None,
        pattern: None,
        format: None,
        enum_values: None,
    }
}

/// Get comprehensive test cases for the schema
pub fn get_schema_test_cases() -> Vec<ValidationTestCase> {
    vec![
        // Valid cases
        ValidationTestCase {
            name: "valid_complete_profile".to_string(),
            input: serde_json::json!({
                "name": "John Doe",
                "age": 30,
                "email": "john@example.com",
                "tags": ["developer", "rust"]
            }),
            expected_valid: true,
            description: "Complete valid user profile".to_string(),
        },

        ValidationTestCase {
            name: "valid_minimal_profile".to_string(),
            input: serde_json::json!({
                "name": "Jane",
                "email": "jane@test.com"
            }),
            expected_valid: true,
            description: "Minimal valid profile with required fields only".to_string(),
        },

        ValidationTestCase {
            name: "valid_with_age_zero".to_string(),
            input: serde_json::json!({
                "name": "Baby",
                "age": 0,
                "email": "baby@example.com"
            }),
            expected_valid: true,
            description: "Valid profile with minimum age".to_string(),
        },

        // Invalid cases
        ValidationTestCase {
            name: "missing_required_name".to_string(),
            input: serde_json::json!({
                "age": 25,
                "email": "test@example.com"
            }),
            expected_valid: false,
            description: "Missing required name field".to_string(),
        },

        ValidationTestCase {
            name: "missing_required_email".to_string(),
            input: serde_json::json!({
                "name": "John",
                "age": 25
            }),
            expected_valid: false,
            description: "Missing required email field".to_string(),
        },

        ValidationTestCase {
            name: "name_too_short".to_string(),
            input: serde_json::json!({
                "name": "A",
                "email": "test@example.com"
            }),
            expected_valid: false,
            description: "Name shorter than minimum length".to_string(),
        },

        ValidationTestCase {
            name: "name_too_long".to_string(),
            input: serde_json::json!({
                "name": "A".repeat(101),
                "email": "test@example.com"
            }),
            expected_valid: false,
            description: "Name longer than maximum length".to_string(),
        },

        ValidationTestCase {
            name: "name_invalid_characters".to_string(),
            input: serde_json::json!({
                "name": "John123",
                "email": "john@example.com"
            }),
            expected_valid: false,
            description: "Name contains invalid characters (numbers)".to_string(),
        },

        ValidationTestCase {
            name: "age_negative".to_string(),
            input: serde_json::json!({
                "name": "John",
                "age": -5,
                "email": "john@example.com"
            }),
            expected_valid: false,
            description: "Negative age value".to_string(),
        },

        ValidationTestCase {
            name: "age_too_high".to_string(),
            input: serde_json::json!({
                "name": "John",
                "age": 200,
                "email": "john@example.com"
            }),
            expected_valid: false,
            description: "Age exceeds maximum allowed".to_string(),
        },

        ValidationTestCase {
            name: "email_invalid_format".to_string(),
            input: serde_json::json!({
                "name": "John",
                "email": "invalid-email"
            }),
            expected_valid: false,
            description: "Invalid email format".to_string(),
        },

        ValidationTestCase {
            name: "email_too_short".to_string(),
            input: serde_json::json!({
                "name": "John",
                "email": "a@b.c"
            }),
            expected_valid: false,
            description: "Email shorter than minimum length".to_string(),
        },

        ValidationTestCase {
            name: "email_too_long".to_string(),
            input: serde_json::json!({
                "name": "John",
                "email": format!("{}@example.com", "a".repeat(250))
            }),
            expected_valid: false,
            description: "Email longer than maximum length".to_string(),
        },

        // Edge cases
        ValidationTestCase {
            name: "age_not_integer".to_string(),
            input: serde_json::json!({
                "name": "John",
                "age": 25.5,
                "email": "john@example.com"
            }),
            expected_valid: false,
            description: "Age is not an integer".to_string(),
        },

        ValidationTestCase {
            name: "tags_wrong_type".to_string(),
            input: serde_json::json!({
                "name": "John",
                "email": "john@example.com",
                "tags": "not an array"
            }),
            expected_valid: false,
            description: "Tags field is not an array".to_string(),
        },
    ]
}

/// Expected implementation structure
pub fn get_expected_validator_structure() -> String {
    r#"
pub struct JsonSchemaValidator {
    // Schema validation logic
}

impl JsonSchemaValidator {
    pub fn new(schema: JsonSchema) -> Self {
        // Initialize validator with schema
    }

    pub fn validate(&self, data: &serde_json::Value) -> Result<(), ValidationError> {
        // Validate data against schema
    }

    fn validate_object(&self, data: &serde_json::Map<String, serde_json::Value>) -> Result<(), ValidationError> {
        // Validate object properties
    }

    fn validate_property(&self, property_name: &str, value: &serde_json::Value, property_schema: &JsonSchemaProperty) -> Result<(), ValidationError> {
        // Validate individual property
    }

    fn validate_string(&self, value: &str, property_schema: &JsonSchemaProperty) -> Result<(), ValidationError> {
        // Validate string constraints
    }

    fn validate_number(&self, value: &serde_json::Value, property_schema: &JsonSchemaProperty) -> Result<(), ValidationError> {
        // Validate number constraints
    }

    fn validate_array(&self, value: &serde_json::Value, property_schema: &JsonSchemaProperty) -> Result<(), ValidationError> {
        // Validate array constraints
    }
}

#[derive(Debug)]
pub enum ValidationError {
    MissingRequiredField(String),
    InvalidType { field: String, expected: String, actual: String },
    StringTooShort { field: String, min_length: usize, actual_length: usize },
    StringTooLong { field: String, max_length: usize, actual_length: usize },
    PatternMismatch { field: String, pattern: String },
    NumberBelowMinimum { field: String, minimum: f64, actual: f64 },
    NumberAboveMaximum { field: String, maximum: f64, actual: f64 },
    InvalidFormat { field: String, format: String },
}
"#.to_string()
}

/// Mutation testing configuration
pub fn get_mutation_config() -> serde_json::Value {
    serde_json::json!({
        "threshold": 0.90,
        "operators": [
            "negate_conditionals",
            "invert_negations",
            "return_constant",
            "replace_arithmetic",
            "replace_logical",
            "swap_arguments"
        ],
        "exclude_patterns": [
            "test_*",
            "*_test.rs"
        ]
    })
}


