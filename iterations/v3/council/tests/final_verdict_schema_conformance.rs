use serde_json::json;
use std::fs;

#[test]
fn final_verdict_conforms_to_schema() {
    // Arrange: build a plausible FinalVerdict JSON using known fields from docs
    let verdict = json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "task_id": "00000000-0000-0000-0000-000000000000",
        "consensus": {
            "score": 0.82,
            "rounds": 2,
            "dissent_rate": 0.1
        },
        "judges": [
            {
                "name": "spec-judge",
                "weight": 0.5,
                "verdict": {
                    "decision": "accept",
                    "score": 0.84,
                    "citations": ["doc://spec#A1"],
                    "notes": "Meets acceptance criteria"
                }
            },
            {
                "name": "risk-judge",
                "weight": 0.5,
                "verdict": {
                    "decision": "accept",
                    "score": 0.80,
                    "citations": [],
                    "notes": "Risk acceptable"
                }
            }
        ],
        "artifacts": {
            "provenance_id": "00000000-0000-0000-0000-00000000000a"
        },
        "timestamp": "2025-01-01T00:00:00Z"
    });

    // Load schema
    let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("contracts")
        .join("final-verdict.schema.json");
    let schema_str =
        fs::read_to_string(&schema_path).expect("failed to read final-verdict.schema.json");
    let schema_json: serde_json::Value =
        serde_json::from_str(&schema_str).expect("invalid schema JSON");

    // Validate using jsonschema crate
    let compiled =
        jsonschema::JSONSchema::compile(&schema_json).expect("failed to compile JSON Schema");
    if let Err(errors) = compiled.validate(&verdict) {
        for e in errors {
            eprintln!("Schema error: {} at {}", e, e.instance_path);
        }
        panic!("FinalVerdict JSON does not conform to schema");
    }
}
