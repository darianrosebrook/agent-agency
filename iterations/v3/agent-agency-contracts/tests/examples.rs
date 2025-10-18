use agent_agency_contracts::{
    final_verdict_schema_source, judge_verdict_schema_source, router_decision_schema_source,
    validate_final_verdict_value, validate_judge_verdict_value, validate_router_decision_value,
    validate_worker_output_value, worker_output_schema_source,
};
use serde_json::Value;
use std::path::PathBuf;

fn load_example(name: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has parent")
        .join("docs/contracts/examples")
        .join(name);
    let data = std::fs::read_to_string(&root)
        .unwrap_or_else(|err| panic!("failed to read {}: {}", root.display(), err));
    serde_json::from_str(&data).expect("example json parses")
}

#[test]
fn worker_output_examples_match_schema() {
    let value = load_example("worker-output.json");
    validate_worker_output_value(&value).expect("worker output example matches schema");

    let schema = worker_output_schema_source();
    assert!(schema.contains("worker-output.schema.json"));
}

#[test]
fn judge_verdict_examples_match_schema() {
    let value = load_example("judge-verdict.json");
    validate_judge_verdict_value(&value).expect("judge verdict example matches schema");

    let schema = judge_verdict_schema_source();
    assert!(schema.contains("judge-verdict.schema.json"));
}

#[test]
fn final_verdict_examples_match_schema() {
    let value = load_example("final-verdict.json");
    validate_final_verdict_value(&value).expect("final verdict example matches schema");

    let partial = load_example("final-verdict-partial.json");
    validate_final_verdict_value(&partial).expect("partial final verdict example matches schema");

    let schema = final_verdict_schema_source();
    assert!(schema.contains("final-verdict.schema.json"));
}

#[test]
fn router_decision_examples_match_schema() {
    let value = load_example("router-decision.json");
    validate_router_decision_value(&value).expect("router decision example matches schema");

    let schema = router_decision_schema_source();
    assert!(schema.contains("router-decision.schema.json"));
}
