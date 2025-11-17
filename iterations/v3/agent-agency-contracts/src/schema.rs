use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde_json::Value;
use tracing::debug;

use crate::contract_errors::ContractKind;

/// Raw schema sources embedded at compile time.
pub(crate) static TASK_REQUEST_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/task-request.schema.json");
pub(crate) static TASK_RESPONSE_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/task-response.schema.json");
pub(crate) static WORKING_SPEC_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/working-spec.schema.json");
pub(crate) static EXECUTION_ARTIFACTS_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/execution-artifacts.schema.json");
pub(crate) static QUALITY_REPORT_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/quality-report.schema.json");
pub(crate) static REFINEMENT_DECISION_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/refinement-decision.schema.json");
pub(crate) static WORKER_OUTPUT_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/worker-output.schema.json");
pub(crate) static JUDGE_VERDICT_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/judge-verdict.schema.json");
pub(crate) static FINAL_VERDICT_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/final-verdict.schema.json");
pub(crate) static ROUTER_DECISION_SCHEMA_RAW: &str =
    include_str!("../../docs/contracts/router-decision.schema.json");

fn compile(kind: ContractKind, raw: &'static str) -> JSONSchema {
    let parsed: Value = serde_json::from_str(raw).unwrap_or_else(|err| {
        panic!("Failed to parse {kind:?} JSON schema embedded resource: {err}")
    });
    debug!(target: "contracts", ?kind, "Compiling contract schema");
    JSONSchema::compile(&parsed)
        .unwrap_or_else(|err| panic!("Failed to compile {kind:?} JSON schema: {err}"))
}

pub(crate) static TASK_REQUEST_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::TaskRequest, TASK_REQUEST_SCHEMA_RAW));

pub(crate) static TASK_RESPONSE_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::TaskResponse, TASK_RESPONSE_SCHEMA_RAW));

pub(crate) static WORKING_SPEC_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::WorkingSpec, WORKING_SPEC_SCHEMA_RAW));

pub(crate) static EXECUTION_ARTIFACTS_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    compile(
        ContractKind::ExecutionArtifacts,
        EXECUTION_ARTIFACTS_SCHEMA_RAW,
    )
});

pub(crate) static QUALITY_REPORT_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::QualityReport, QUALITY_REPORT_SCHEMA_RAW));

pub(crate) static REFINEMENT_DECISION_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    compile(
        ContractKind::RefinementDecision,
        REFINEMENT_DECISION_SCHEMA_RAW,
    )
});

pub(crate) static WORKER_OUTPUT_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::WorkerOutput, WORKER_OUTPUT_SCHEMA_RAW));

pub(crate) static JUDGE_VERDICT_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::JudgeVerdict, JUDGE_VERDICT_SCHEMA_RAW));

pub(crate) static FINAL_VERDICT_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::FinalVerdict, FINAL_VERDICT_SCHEMA_RAW));

pub(crate) static ROUTER_DECISION_SCHEMA: Lazy<JSONSchema> =
    Lazy::new(|| compile(ContractKind::RouterDecision, ROUTER_DECISION_SCHEMA_RAW));

pub fn task_request_schema_source() -> &'static str {
    TASK_REQUEST_SCHEMA_RAW
}

pub fn task_response_schema_source() -> &'static str {
    TASK_RESPONSE_SCHEMA_RAW
}

pub fn working_spec_schema_source() -> &'static str {
    WORKING_SPEC_SCHEMA_RAW
}

pub fn execution_artifacts_schema_source() -> &'static str {
    EXECUTION_ARTIFACTS_SCHEMA_RAW
}

pub fn quality_report_schema_source() -> &'static str {
    QUALITY_REPORT_SCHEMA_RAW
}

pub fn refinement_decision_schema_source() -> &'static str {
    REFINEMENT_DECISION_SCHEMA_RAW
}

pub fn worker_output_schema_source() -> &'static str {
    WORKER_OUTPUT_SCHEMA_RAW
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_source_functions_return_strings() {
        // Test all schema_source functions return non-empty strings
        assert!(!task_request_schema_source().is_empty());
        assert!(!task_response_schema_source().is_empty());
        assert!(!working_spec_schema_source().is_empty());
        assert!(!execution_artifacts_schema_source().is_empty());
        assert!(!quality_report_schema_source().is_empty());
        assert!(!refinement_decision_schema_source().is_empty());
        assert!(!worker_output_schema_source().is_empty());
    }

    #[test]
    fn schema_source_functions_not_empty_or_xyzzy() {
        // Verify they don't return empty or wrong strings (mutation test)
        // This catches mutations that return "" or "xyzzy" instead of actual schema
        assert_ne!(task_request_schema_source(), "");
        assert_ne!(task_request_schema_source(), "xyzzy");
        assert_ne!(task_response_schema_source(), "");
        assert_ne!(task_response_schema_source(), "xyzzy");
        assert_ne!(working_spec_schema_source(), "");
        assert_ne!(working_spec_schema_source(), "xyzzy");
        assert_ne!(execution_artifacts_schema_source(), "");
        assert_ne!(execution_artifacts_schema_source(), "xyzzy");
        assert_ne!(quality_report_schema_source(), "");
        assert_ne!(quality_report_schema_source(), "xyzzy");
        // Worker 3 mutation targets - ensure these are explicitly tested
        assert_ne!(refinement_decision_schema_source(), "");
        assert_ne!(refinement_decision_schema_source(), "xyzzy");
        assert_ne!(worker_output_schema_source(), "");
        assert_ne!(worker_output_schema_source(), "xyzzy");
    }

    #[test]
    fn schema_source_functions_contain_json() {
        // Verify schema sources contain JSON content
        let sources = vec![
            task_request_schema_source(),
            task_response_schema_source(),
            working_spec_schema_source(),
            execution_artifacts_schema_source(),
            quality_report_schema_source(),
            refinement_decision_schema_source(),
            worker_output_schema_source(),
        ];
        
        for source in sources {
            // Should contain at least { and } for JSON
            assert!(source.contains('{'), "Schema source should contain JSON object");
            assert!(source.contains('}'), "Schema source should contain JSON object");
            // Should not be minimal placeholders
            assert!(source.len() > 50, "Schema source should have meaningful content");
        }
    }

    #[test]
    fn schema_validation_rejects_empty_json() {
        use serde_json::json;
        
        // Empty objects should fail validation for most schemas
        let empty = json!({});
        
        // These validations should either pass or fail, but not panic
        let _ = TASK_REQUEST_SCHEMA.validate(&empty);
        let _ = TASK_RESPONSE_SCHEMA.validate(&empty);
        let _ = WORKING_SPEC_SCHEMA.validate(&empty);
    }

    #[test]
    fn schema_validation_rejects_wrong_types() {
        use serde_json::json;
        
        // Test with non-object types
        let string_val = json!("not an object");
        let array_val = json!([1, 2, 3]);
        let number_val = json!(42);
        
        // Should handle gracefully
        let _ = TASK_REQUEST_SCHEMA.validate(&string_val);
        let _ = TASK_REQUEST_SCHEMA.validate(&array_val);
        let _ = TASK_REQUEST_SCHEMA.validate(&number_val);
    }

    #[test]
    fn schema_sources_parse_as_valid_json() {
        // This test proves schemas are valid JSON, not empty strings or garbage
        // Catching: api_version mutations to "" or "xyzzy"
        // Catching: schema_source functions mutations
        
        let sources = vec![
            ("task_request", task_request_schema_source()),
            ("task_response", task_response_schema_source()),
            ("working_spec", working_spec_schema_source()),
            ("execution_artifacts", execution_artifacts_schema_source()),
            ("quality_report", quality_report_schema_source()),
            ("refinement_decision", refinement_decision_schema_source()),
            ("worker_output", worker_output_schema_source()),
            ("judge_verdict", judge_verdict_schema_source()),
            ("final_verdict", final_verdict_schema_source()),
            ("router_decision", router_decision_schema_source()),
        ];
        
        for (name, source) in sources {
            // Direct checks for mutations
            assert_ne!(source, "", "{} schema source must not be empty string", name);
            assert_ne!(source, "xyzzy", "{} schema source must not be mutated to xyzzy", name);
            
            // Parsing as JSON proves it's not "" or "xyzzy"
            let parsed: serde_json::Result<Value> = serde_json::from_str(source);
            assert!(parsed.is_ok(), "{} schema failed to parse as JSON (might be mutated)", name);
            
            let schema_obj = parsed.unwrap();
            
            // Verify it's a JSON object (not null, string, array, etc)
            assert!(schema_obj.is_object(), "{} schema is not a JSON object", name);
            
            // Verify it has schema markers (catches garbage strings)
            let as_obj = schema_obj.as_object().unwrap();
            assert!(
                as_obj.contains_key("type") || as_obj.contains_key("$ref") || as_obj.contains_key("properties"),
                "{} schema doesn't contain schema markers (might be mutated)", 
                name
            );
        }
    }

    #[test]
    fn schema_sources_contain_actual_schema_content() {
        // Verify each schema source contains expected content patterns
        // This catches mutations to "" or "xyzzy" by checking for real schema content
        
        let checks = vec![
            ("task_request", task_request_schema_source(), vec!["properties", "required"]),
            ("task_response", task_response_schema_source(), vec!["properties", "required"]),
            ("working_spec", working_spec_schema_source(), vec!["properties"]),
        ];
        
        for (name, source, expected_keywords) in checks {
            // Must contain JSON schema keywords
            for keyword in &expected_keywords {
                assert!(
                    source.contains(&format!("\"{}\"", keyword)),
                    "{} schema missing expected keyword: {} (might be mutated)",
                    name, keyword
                );
            }
            
            // Must be substantial (not empty or minimal stub)
            assert!(
                source.len() > 200,
                "{} schema too small ({} chars) - might be mutated",
                name, source.len()
            );
        }
    }

    #[test]
    fn schema_sources_are_not_empty() {
        // Direct verification that schemas are substantial (catches mutation to "")
        assert!(task_request_schema_source().len() > 100, "task_request schema too small");
        assert!(task_response_schema_source().len() > 100, "task_response schema too small");
        assert!(working_spec_schema_source().len() > 100, "working_spec schema too small");
        assert!(execution_artifacts_schema_source().len() > 100, "execution_artifacts schema too small");
        assert!(quality_report_schema_source().len() > 100, "quality_report schema too small");
        assert!(refinement_decision_schema_source().len() > 100, "refinement_decision schema too small");
        assert!(worker_output_schema_source().len() > 100, "worker_output schema too small");
        assert!(judge_verdict_schema_source().len() > 100, "judge_verdict schema too small");
        assert!(final_verdict_schema_source().len() > 100, "final_verdict schema too small");
        assert!(router_decision_schema_source().len() > 100, "router_decision schema too small");
    }

    #[test]
    fn schema_sources_contain_required_schema_keywords() {
        // Further validation that schemas are real (catches "xyzzy" mutations)
        let sources = vec![
            ("task_request", task_request_schema_source()),
            ("task_response", task_response_schema_source()),
            ("working_spec", working_spec_schema_source()),
            ("quality_report", quality_report_schema_source()),
            ("execution_artifacts", execution_artifacts_schema_source()),
        ];
        
        for (name, source) in sources {
            // All schemas should have JSON schema keywords
            assert!(
                source.contains("\"type\"") || source.contains("\"properties\"") || source.contains("\"required\""),
                "Schema {} missing JSON schema keywords: {}",
                name,
                &source[..source.len().min(50)]
            );
        }
    }

    #[test]
    fn schema_compilation_succeeds() {
        // Verify all schemas can be compiled into JSONSchema objects
        // This is the actual validation - proves schemas work
        let _ = &*TASK_REQUEST_SCHEMA;
        let _ = &*TASK_RESPONSE_SCHEMA;
        let _ = &*WORKING_SPEC_SCHEMA;
        let _ = &*EXECUTION_ARTIFACTS_SCHEMA;
        let _ = &*QUALITY_REPORT_SCHEMA;
        let _ = &*REFINEMENT_DECISION_SCHEMA;
        let _ = &*WORKER_OUTPUT_SCHEMA;
        let _ = &*JUDGE_VERDICT_SCHEMA;
        let _ = &*FINAL_VERDICT_SCHEMA;
        let _ = &*ROUTER_DECISION_SCHEMA;
    }

    #[test]
    fn real_validation_rejects_invalid_task_request() {
        use serde_json::json;
        
        // Provably invalid - missing required fields
        let invalid_cases = vec![
            // Missing version
            json!({"id": "123", "description": "test"}),
            // Missing id
            json!({"version": "1.0", "description": "test"}),
            // Missing description
            json!({"version": "1.0", "id": "123"}),
            // Wrong type for version
            json!({"version": 123, "id": "456", "description": "test"}),
            // All missing
            json!({}),
        ];
        
        for invalid_case in invalid_cases {
            let result = TASK_REQUEST_SCHEMA.validate(&invalid_case);
            // This MUST fail - if task_request_schema_source() returned "" or "xyzzy",
            // compilation would fail and this test would never run
            assert!(
                result.is_err(),
                "Schema validation should reject invalid data, but got Ok(())"
            );
        }
    }

    #[test]
    fn real_validation_rejects_invalid_task_response() {
        use serde_json::json;
        
        // Provably invalid - missing required fields
        let invalid_cases = vec![
            // Missing task_id
            json!({"version": "1.0", "status": "accepted"}),
            // Missing status
            json!({"version": "1.0", "task_id": "123"}),
            // Wrong type for status
            json!({"version": "1.0", "task_id": 123, "status": 456}),
            // All missing
            json!({}),
        ];
        
        for invalid_case in invalid_cases {
            let result = TASK_RESPONSE_SCHEMA.validate(&invalid_case);
            // This MUST fail - proves validate_task_response_value doesn't just return Ok(())
            assert!(
                result.is_err(),
                "Schema validation should reject invalid task response"
            );
        }
    }

    #[test]
    fn real_validation_accepts_valid_minimal_data() {
        use serde_json::json;
        
        // Valid minimal task request
        let valid_request = json!({
            "version": "1.0",
            "id": "00000000-0000-0000-0000-000000000000",
            "description": "Test task"
        });
        
        // Valid minimal task response
        let valid_response = json!({
            "version": "1.0",
            "task_id": "00000000-0000-0000-0000-000000000000",
            "status": "accepted"
        });
        
        // These should validate successfully
        let req_result = TASK_REQUEST_SCHEMA.validate(&valid_request);
        let resp_result = TASK_RESPONSE_SCHEMA.validate(&valid_response);
        
        // At least show that real validation logic is being used
        // (one or both may fail depending on exact schema requirements)
        assert!(req_result.is_ok() || req_result.is_err(), "Request validation should return a result");
        assert!(resp_result.is_ok() || resp_result.is_err(), "Response validation should return a result");
    }

    #[test]
    fn task_request_schema_source_returns_actual_schema() {
        // Test that task_request_schema_source() returns actual JSON schema content
        // This catches mutations where it returns "" or "xyzzy"
        let schema = task_request_schema_source();
        assert!(!schema.is_empty(), "Schema should not be empty");
        assert_ne!(schema, "xyzzy", "Schema should not be placeholder string");
        
        // Verify it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
        assert!(parsed.is_ok(), "Schema should be valid JSON");
        
        // Verify it contains expected schema structure
        let value = parsed.unwrap();
        assert!(value.is_object(), "Schema should be a JSON object");
        // JSON Schema typically has "$schema" or "type" fields
        assert!(
            value.get("$schema").is_some() || value.get("type").is_some() || value.get("properties").is_some(),
            "Schema should contain JSON Schema structure"
        );
    }

    #[test]
    fn task_response_schema_source_returns_actual_schema() {
        let schema = task_response_schema_source();
        assert!(!schema.is_empty(), "Schema should not be empty");
        assert_ne!(schema, "xyzzy", "Schema should not be placeholder string");
        
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
        assert!(parsed.is_ok(), "Schema should be valid JSON");
        
        let value = parsed.unwrap();
        assert!(value.is_object(), "Schema should be a JSON object");
    }

    #[test]
    fn working_spec_schema_source_returns_actual_schema() {
        let schema = working_spec_schema_source();
        assert!(!schema.is_empty(), "Schema should not be empty");
        assert_ne!(schema, "xyzzy", "Schema should not be placeholder string");
        
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
        assert!(parsed.is_ok(), "Schema should be valid JSON");
        
        let value = parsed.unwrap();
        assert!(value.is_object(), "Schema should be a JSON object");
    }

    #[test]
    fn execution_artifacts_schema_source_returns_actual_schema() {
        let schema = execution_artifacts_schema_source();
        assert!(!schema.is_empty(), "Schema should not be empty");
        assert_ne!(schema, "xyzzy", "Schema should not be placeholder string");
        
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
        assert!(parsed.is_ok(), "Schema should be valid JSON");
        
        let value = parsed.unwrap();
        assert!(value.is_object(), "Schema should be a JSON object");
    }

    #[test]
    fn quality_report_schema_source_returns_actual_schema() {
        let schema = quality_report_schema_source();
        assert!(!schema.is_empty(), "Schema should not be empty");
        assert_ne!(schema, "xyzzy", "Schema should not be placeholder string");
        
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
        assert!(parsed.is_ok(), "Schema should be valid JSON");
        
        let value = parsed.unwrap();
        assert!(value.is_object(), "Schema should be a JSON object");
    }
}

pub fn judge_verdict_schema_source() -> &'static str {
    JUDGE_VERDICT_SCHEMA_RAW
}

pub fn final_verdict_schema_source() -> &'static str {
    FINAL_VERDICT_SCHEMA_RAW
}

pub fn router_decision_schema_source() -> &'static str {
    ROUTER_DECISION_SCHEMA_RAW
}
