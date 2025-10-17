use crate::caws_runtime::{ValidationResult, ViolationCode};
use agent_agency_council::types::*;

fn code_to_ref(code: &ViolationCode) -> &'static str {
    match code {
        ViolationCode::OutOfScope => "CAWS:Scope",
        ViolationCode::BudgetExceeded => "CAWS:Budget",
        ViolationCode::MissingTests => "CAWS:Tests",
        ViolationCode::NonDeterministic => "CAWS:Determinism",
        ViolationCode::DisallowedTool => "CAWS:Tools",
    }
}

/// Build a short-circuit FinalVerdict when CAWS-critical violations occur.
/// Returns None if no hard-fail violations are present.
pub fn build_short_circuit_verdict(v: &ValidationResult) -> Option<api::FinalVerdict> {
    let mut remediation: Vec<String> = Vec::new();
    let mut refs: Vec<String> = Vec::new();
    let mut hard_fail = false;

    for viol in &v.violations {
        // All current violations are treated as hard-fail unless waived in upstream logic.
        hard_fail = true;
        remediation.push(viol.remediation.clone().unwrap_or_else(|| viol.message.clone()));
        refs.push(code_to_ref(&viol.code).to_string());
    }

    if hard_fail {
        Some(api::FinalVerdict {
            decision: api::FinalDecision::Reject,
            votes: vec![],
            dissent: "CAWS runtime validation failed".to_string(),
            remediation,
            constitutional_refs: refs,
        })
    } else {
        None
    }
}

