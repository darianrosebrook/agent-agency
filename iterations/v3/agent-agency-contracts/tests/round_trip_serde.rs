//! Round-trip serialization/deserialization tests
//!
//! Ensures all contract types can be serialized to JSON and deserialized back
//! to the same values, validating forward compatibility and serde correctness.
//!
//! @author @darianrosebrook

use agent_agency_contracts::types::council::{CouncilVerdict, FinalDecision};
use agent_agency_contracts::types::data::{ContentType, ProcessedContent, ProcessingId};
use agent_agency_contracts::types::planning::TaskScope;
use agent_agency_contracts::types::prelude::*;
use agent_agency_contracts::{
    AcceptanceCriterion, EvidenceGate, ExecutionContext, InterfaceContract, Milestone,
    MilestoneMetrics, MilestonePriority, MilestoneScope, MilestoneState, MoSCoWPriority,
    TestRequirement,
};
use chrono::Utc;
use serde_json;
use std::collections::HashMap;
use uuid::Uuid;

/// Helper macro to test round-trip serialization
#[allow(unused_macros)]
macro_rules! test_round_trip {
    ($name:ident, $value:expr) => {
        #[test]
        fn $name() {
            let original = $value;
            let json = serde_json::to_string(&original).expect("serialization should succeed");
            let deserialized: $name =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(original, deserialized, "round-trip should preserve values");
        }
    };
}

#[test]
fn test_task_priority_round_trip() {
    let priorities = vec![
        TaskPriority::Low,
        TaskPriority::Normal,
        TaskPriority::Medium,
        TaskPriority::High,
        TaskPriority::Urgent,
        TaskPriority::Critical,
    ];

    for priority in priorities {
        let json = serde_json::to_string(&priority).expect("serialize TaskPriority");
        let deserialized: TaskPriority =
            serde_json::from_str(&json).expect("deserialize TaskPriority");
        assert_eq!(priority, deserialized);
    }
}

#[test]
fn test_execution_mode_round_trip() {
    let modes = vec![
        ExecutionMode::DryRun,
        ExecutionMode::Auto,
        ExecutionMode::Strict,
    ];

    for mode in modes {
        let json = serde_json::to_string(&mode).expect("serialize ExecutionMode");
        let deserialized: ExecutionMode =
            serde_json::from_str(&json).expect("deserialize ExecutionMode");
        assert_eq!(mode, deserialized);
    }
}

#[test]
fn test_risk_tier_round_trip() {
    let tiers = vec![RiskTier::Tier1, RiskTier::Tier2, RiskTier::Tier3];

    for tier in tiers {
        let json = serde_json::to_string(&tier).expect("serialize RiskTier");
        let deserialized: RiskTier = serde_json::from_str(&json).expect("deserialize RiskTier");
        assert_eq!(tier, deserialized);
    }
}

#[test]
fn test_blast_radius_round_trip() {
    let blast_radius = BlastRadius {
        modules: vec!["auth".to_string(), "api".to_string()],
        data_migration: true,
        external_deps: vec!["postgres".to_string()],
    };

    let json = serde_json::to_string(&blast_radius).expect("serialize BlastRadius");
    let deserialized: BlastRadius = serde_json::from_str(&json).expect("deserialize BlastRadius");
    assert_eq!(blast_radius.modules, deserialized.modules);
    assert_eq!(blast_radius.data_migration, deserialized.data_migration);
    assert_eq!(blast_radius.external_deps, deserialized.external_deps);
}

#[test]
fn test_task_scope_round_trip() {
    let scope = TaskScope {
        in_scope: vec!["src/auth/".to_string(), "tests/auth/".to_string()],
        out_scope: vec!["node_modules/".to_string()],
    };

    let json = serde_json::to_string(&scope).expect("serialize TaskScope");
    let deserialized: TaskScope = serde_json::from_str(&json).expect("deserialize TaskScope");
    assert_eq!(scope.in_scope, deserialized.in_scope);
    assert_eq!(scope.out_scope, deserialized.out_scope);
}

#[test]
fn test_execution_context_round_trip() {
    let ctx = ExecutionContext {
        session_id: Uuid::new_v4(),
        planning_engine: "test-engine".to_string(),
        engine_version: "1.0.0".to_string(),
        planning_metadata: vec![
            ("key1".to_string(), serde_json::json!("value1")),
            ("key2".to_string(), serde_json::json!({"nested": true})),
        ]
        .into_iter()
        .collect(),
    };

    let json = serde_json::to_string(&ctx).expect("serialize ExecutionContext");
    let deserialized: ExecutionContext =
        serde_json::from_str(&json).expect("deserialize ExecutionContext");
    assert_eq!(ctx.session_id, deserialized.session_id);
    assert_eq!(ctx.planning_engine, deserialized.planning_engine);
    assert_eq!(ctx.engine_version, deserialized.engine_version);
}

#[test]
fn test_milestone_round_trip() {
    let milestone = Milestone {
        id: "M1".to_string(),
        objective: "Test objective".to_string(),
        scope: MilestoneScope {
            files: vec!["src/".to_string()],
            directories: vec!["tests/".to_string()],
            included_paths: vec!["src/".to_string()],
            excluded_paths: vec!["tests/".to_string()],
            will_modify: false,
            allowed_operations: vec!["read".to_string(), "write".to_string()],
            parallelism: None,
            resource_requirements: HashMap::new(),
        },
        interfaces: vec![],
        tests: vec![],
        evidence_gate: EvidenceGate {
            min_coverage: 0.8,
            min_branch_coverage: 0.9,
            min_mutation_score: 0.7,
            security_scan_required: false,
            performance_budget: None,
            required_artifacts: vec![],
            custom_validations: vec![],
        },
        quality_gates: vec!["coverage".to_string()],
        dependencies: vec!["M0".to_string()],
        estimated_duration: Some(60),
        rollback_plan: "Revert changes".to_string(),
        state: MilestoneState::Pending,
        assigned_workers: vec![],
        estimated_effort: 1.0,
        priority: MilestonePriority::Normal,
        risk_tier: 2,
        is_blocking: false,
        blocking_reason: None,
        metrics: None,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&milestone).expect("serialize Milestone");
    let deserialized: Milestone = serde_json::from_str(&json).expect("deserialize Milestone");
    assert_eq!(milestone.id, deserialized.id);
    assert_eq!(milestone.objective, deserialized.objective);
    assert_eq!(milestone.dependencies, deserialized.dependencies);
}

#[test]
fn test_acceptance_criterion_round_trip() {
    let criterion = AcceptanceCriterion {
        id: "A1".to_string(),
        given: "User is logged in".to_string(),
        when: "User submits form".to_string(),
        then: "Form is validated".to_string(),
        priority: Some(MoSCoWPriority::Must),
    };

    let json = serde_json::to_string(&criterion).expect("serialize AcceptanceCriterion");
    let deserialized: AcceptanceCriterion =
        serde_json::from_str(&json).expect("deserialize AcceptanceCriterion");
    assert_eq!(criterion.id, deserialized.id);
    assert_eq!(criterion.given, deserialized.given);
    assert_eq!(criterion.when, deserialized.when);
    assert_eq!(criterion.then, deserialized.then);
}

#[test]
fn test_processed_content_round_trip() {
    let content = ProcessedContent {
        id: ProcessingId(Uuid::new_v4()),
        content_type: ContentType::Text,
        metadata: vec![
            ("size".to_string(), serde_json::json!(1024)),
            ("encoding".to_string(), serde_json::json!("utf-8")),
        ]
        .into_iter()
        .collect(),
    };

    let json = serde_json::to_string(&content).expect("serialize ProcessedContent");
    let deserialized: ProcessedContent =
        serde_json::from_str(&json).expect("deserialize ProcessedContent");
    assert_eq!(content.id, deserialized.id);
    assert_eq!(content.content_type, deserialized.content_type);
}

#[test]
fn test_council_verdict_round_trip() {
    let verdicts = vec![
        CouncilVerdict::Approved,
        CouncilVerdict::ConditionalApproval,
        CouncilVerdict::Rejected,
    ];

    for verdict in verdicts {
        let json = serde_json::to_string(&verdict).expect("serialize CouncilVerdict");
        let deserialized: CouncilVerdict =
            serde_json::from_str(&json).expect("deserialize CouncilVerdict");
        assert_eq!(verdict, deserialized);
    }
}

#[test]
fn test_final_decision_round_trip() {
    let decision = FinalDecision {
        id: "decision-1".to_string(),
        verdict: CouncilVerdict::Approved,
        reasoning: "All checks passed".to_string(),
        requirements: vec!["test".to_string(), "coverage".to_string()],
        participants: vec!["judge1".to_string(), "judge2".to_string()],
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&decision).expect("serialize FinalDecision");
    let deserialized: FinalDecision =
        serde_json::from_str(&json).expect("deserialize FinalDecision");
    assert_eq!(decision.id, deserialized.id);
    assert_eq!(decision.verdict, deserialized.verdict);
    assert_eq!(decision.reasoning, deserialized.reasoning);
    assert_eq!(decision.requirements, deserialized.requirements);
}

#[test]
fn test_partial_serde_with_optionals() {
    // Test that optional fields work correctly with serde defaults
    let minimal_milestone = Milestone {
        id: "M1".to_string(),
        objective: "Test".to_string(),
        scope: MilestoneScope {
            files: vec![],
            directories: vec![],
            included_paths: vec![],
            excluded_paths: vec![],
            will_modify: false,
            allowed_operations: vec![],
            parallelism: None,
            resource_requirements: HashMap::new(),
        },
        interfaces: vec![],
        tests: vec![],
        evidence_gate: EvidenceGate {
            min_coverage: 0.0,
            min_branch_coverage: 0.0,
            min_mutation_score: 0.0,
            security_scan_required: false,
            performance_budget: None,
            required_artifacts: vec![],
            custom_validations: vec![],
        },
        quality_gates: vec![],
        dependencies: vec![],
        estimated_duration: None,
        rollback_plan: "".to_string(),
        state: MilestoneState::Pending,
        assigned_workers: vec![],
        estimated_effort: 0.0,
        priority: MilestonePriority::Normal,
        risk_tier: 1,
        is_blocking: false,
        blocking_reason: None,
        metrics: None,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&minimal_milestone).expect("serialize minimal Milestone");
    let deserialized: Milestone =
        serde_json::from_str(&json).expect("deserialize minimal Milestone");
    assert_eq!(minimal_milestone.id, deserialized.id);
    assert_eq!(
        minimal_milestone.estimated_duration,
        deserialized.estimated_duration
    );
}

#[test]
fn test_uuid_serialization_compatibility() {
    // Test that UUIDs serialize/deserialize correctly with schemars
    let task_id = Uuid::new_v4();
    let ctx = ExecutionContext {
        session_id: task_id,
        planning_engine: "test".to_string(),
        engine_version: "1.0.0".to_string(),
        planning_metadata: std::collections::HashMap::new(),
    };

    let json = serde_json::to_string(&ctx).expect("serialize ExecutionContext with UUID");
    // UUID should serialize as a string
    assert!(json.contains(&task_id.to_string()));

    let deserialized: ExecutionContext =
        serde_json::from_str(&json).expect("deserialize ExecutionContext with UUID");
    assert_eq!(ctx.session_id, deserialized.session_id);
}
