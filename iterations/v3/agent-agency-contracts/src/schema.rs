use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde_json::Value;
use tracing::debug;

use crate::error::ContractKind;

/// Raw schema sources embedded at compile time.
pub(crate) static WORKER_OUTPUT_SCHEMA_RAW: &str = include_str!("../../docs/contracts/worker-output.schema.json");
pub(crate) static JUDGE_VERDICT_SCHEMA_RAW: &str = include_str!("../../docs/contracts/judge-verdict.schema.json");
pub(crate) static FINAL_VERDICT_SCHEMA_RAW: &str = include_str!("../../docs/contracts/final-verdict.schema.json");
pub(crate) static ROUTER_DECISION_SCHEMA_RAW: &str = include_str!("../../docs/contracts/router-decision.schema.json");

fn compile(kind: ContractKind, raw: &'static str) -> JSONSchema {
    let parsed: Value = serde_json::from_str(raw).unwrap_or_else(|err| {
        panic!(
            "Failed to parse {kind:?} JSON schema embedded resource: {err}"
        )
    });
    debug!(target: "contracts", ?kind, "Compiling contract schema");
    JSONSchema::compile(&parsed).unwrap_or_else(|err| {
        panic!("Failed to compile {kind:?} JSON schema: {err}")
    })
}

pub(crate) static WORKER_OUTPUT_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    compile(ContractKind::WorkerOutput, WORKER_OUTPUT_SCHEMA_RAW)
});

pub(crate) static JUDGE_VERDICT_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    compile(ContractKind::JudgeVerdict, JUDGE_VERDICT_SCHEMA_RAW)
});

pub(crate) static FINAL_VERDICT_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    compile(ContractKind::FinalVerdict, FINAL_VERDICT_SCHEMA_RAW)
});

pub(crate) static ROUTER_DECISION_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    compile(ContractKind::RouterDecision, ROUTER_DECISION_SCHEMA_RAW)
});
