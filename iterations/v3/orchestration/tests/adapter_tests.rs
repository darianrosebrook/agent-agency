use orchestration::adapter::build_short_circuit_verdict;
use orchestration::caws_runtime::*;

#[test]
fn builds_reject_with_refs_and_remediation() {
    let v = ValidationResult {
        task_id: "T-1".into(),
        snapshot: ComplianceSnapshot {
            within_scope: false,
            within_budget: false,
            tests_added: false,
            deterministic: false,
        },
        violations: vec![
            Violation {
                code: ViolationCode::OutOfScope,
                message: "outside".into(),
                remediation: Some("restrict scope".into()),
            },
            Violation {
                code: ViolationCode::BudgetExceeded,
                message: "budget".into(),
                remediation: None,
            },
        ],
        waivers: vec![],
        validated_at: chrono::Utc::now(),
    };
    let fv = build_short_circuit_verdict(&v).expect("should reject");
    assert!(matches!(
        fv.decision,
        council::contracts::FinalDecision::Reject
    ));
    assert!(fv.remediation.len() >= 2);
    assert!(fv.constitutional_refs.iter().any(|r| r == "CAWS:Scope"));
    assert!(fv.constitutional_refs.iter().any(|r| r == "CAWS:Budget"));
}
