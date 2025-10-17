use std::fs;

#[test]
fn worker_output_conforms_to_schema() {
    let output = serde_json::json!({
        "content": "Implemented feature X",
        "files_modified": [
            {"path":"src/main.rs","operation":"Modify","content":null,"diff":"+fn x(){}","size_bytes":12}
        ],
        "rationale": "Meets acceptance criteria",
        "self_assessment": {"caws_compliance":0.95,"quality_score":0.9,"confidence":0.85,"concerns":[],"improvements":[],"estimated_effort":null},
        "metadata": {}
    });

    let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("contracts")
        .join("worker-output.schema.json");
    let schema_str = fs::read_to_string(&schema_path).expect("read worker-output.schema.json");
    let schema_json: serde_json::Value = serde_json::from_str(&schema_str).expect("schema json");
    let compiled = jsonschema::JSONSchema::compile(&schema_json).expect("compile schema");
    let validation_result = compiled.validate(&output);
    if let Err(errors) = validation_result {
        for e in errors {
            eprintln!("Schema error: {} at {}", e, e.instance_path);
        }
        panic!("WorkerOutput JSON does not conform to schema");
    }
}
