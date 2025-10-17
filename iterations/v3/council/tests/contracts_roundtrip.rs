use council::contracts::*;
use serde_json::{json, Value};

#[test]
fn worker_output_round_trip() {
    let wo = WorkerOutput {
        metadata: WorkerMetadata {
            task_id: "T-1".into(),
            risk_tier: 2,
            seeds: Seeds {
                time_seed: "2025-01-01T00:00:00Z".into(),
                uuid_seed: "0000".into(),
                random_seed: 42,
            },
        },
        artifacts: Artifacts {
            patches: vec![PatchArtifact {
                path: "a.txt".into(),
                diff: "+hi".into(),
            }],
            commands: vec![],
        },
        rationale: "did X then Y".into(),
        self_assessment: SelfAssessment {
            caws_checklist: CawsChecklist {
                within_scope: true,
                within_budget: true,
                tests_added: true,
                deterministic: true,
            },
            notes: "ok".into(),
        },
        claims: vec![Claim {
            id: "C1".into(),
            title: "Implements A1".into(),
            summary: Some("meets A1".into()),
        }],
        evidence_refs: vec![EvidenceLink {
            claim_id: "C1".into(),
            ref_: "tests#L1".into(),
            note: None,
        }],
        waivers: vec![],
    };
    let s = serde_json::to_string(&wo).unwrap();
    let back: WorkerOutput = serde_json::from_str(&s).unwrap();
    assert_eq!(back.metadata.task_id, "T-1");
}

#[test]
fn verdict_round_trip() {
    let jv = JudgeVerdict {
        judge_id: "jc".into(),
        version: "1".into(),
        verdict: VerdictSimple::Pass,
        reasons: vec!["meets CAWS".into()],
        evidence: vec![],
    };
    let s = serde_json::to_string(&jv).unwrap();
    let back: JudgeVerdict = serde_json::from_str(&s).unwrap();
    matches!(back.verdict, VerdictSimple::Pass);
}

#[test]
fn final_verdict_round_trip() {
    let fv = FinalVerdict {
        decision: FinalDecision::Accept,
        votes: vec![VoteEntry {
            judge_id: "jc".into(),
            weight: 0.4,
            verdict: VerdictSimple::Pass,
        }],
        dissent: String::new(),
        remediation: vec![],
        constitutional_refs: vec![],
        verification_summary: Some(VerificationSummary {
            claims_total: 1,
            claims_verified: 1,
            coverage_pct: 1.0,
        }),
    };
    let s = serde_json::to_string(&fv).unwrap();
    let back: FinalVerdict = serde_json::from_str(&s).unwrap();
    if let FinalDecision::Accept = back.decision {
    } else {
        panic!("bad decision");
    }
}
