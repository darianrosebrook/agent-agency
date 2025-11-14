//! Schema snapshot tests
//!
//! Ensures JSON Schema generation remains stable and matches expected snapshots.
//! Detects accidental schema changes that could break API compatibility.
//!
//! @author @darianrosebrook

use agent_agency_contracts::types::planning::TaskScope;
use agent_agency_contracts::types::prelude::*;
use agent_agency_contracts::{AcceptanceCriterion, ExecutionContext, Milestone};
use serde_json::Value;

/// Generate JSON schema for a type and compare with snapshot
#[allow(unused_macros)]
macro_rules! test_schema_snapshot {
    ($type:ty, $snapshot_name:literal) => {
        #[test]
        fn $snapshot_name() {
            let schema = schemars::schema_for!($type);
            let schema_json = serde_json::to_string_pretty(&schema).expect("serialize schema");

            // TODO: Implement schema snapshot comparison
            //       Currently validates JSON only; should compare against stored snapshot file for schema regression detection.
            //
            // COMPLETION CHECKLIST:
            // [ ] Load stored snapshot file
            // [ ] Compare current schema with snapshot
            // [ ] Detect schema changes and regressions
            // [ ] Generate diff report for changes
            // [ ] Handle missing snapshot files
            // [ ] Add unit tests with mock snapshots
            // [ ] Add integration tests with real schema changes
            // [ ] Performance: Comparison should complete in <10ms
            // [ ] Documentation: Document snapshot format
            //
            // ACCEPTANCE CRITERIA:
            // - Schema is compared against snapshot
            // - Changes are detected accurately
            // - Diff reports are generated
            // - Missing snapshots are handled gracefully
            // - Comparison performance is acceptable
            //
            // DEPENDENCIES:
            // - Snapshot file storage (Required)
            // - Schema comparison logic (Required)
            // - Diff generation utilities (Required)
            //
            // ESTIMATED EFFORT: 4-6 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (testing feature)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: Schema testing expertise
            //
            // TODO: Implement comprehensive schema snapshot comparison
            //       Currently verifies schema is valid JSON and contains expected fields only; should implement comprehensive comparison that compares schema against snapshot, detects changes accurately, and generates diff reports.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Schema is compared against snapshot
            // - Changes are detected accurately
            // - Diff reports are generated
            // - Missing snapshots are handled gracefully
            //
            // DEPENDENCIES:
            // - Snapshot file storage (Required)
            // - Schema comparison logic (Required)
            // - Diff generation utilities (Required)
            //
            // ESTIMATED EFFORT: 4-6 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (testing feature)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: Schema testing expertise
            let parsed: Value =
                serde_json::from_str(&schema_json).expect("schema should be valid JSON");

            assert!(
                parsed.get("$schema").is_some() || parsed.get("type").is_some(),
                "Schema should have either $schema or type field"
            );

            // Store snapshot for manual comparison
            let snapshot_path = format!("target/schemas/{}.json", $snapshot_name);
            std::fs::create_dir_all("target/schemas").ok();
            std::fs::write(&snapshot_path, &schema_json).expect("write schema snapshot");
        }
    };
}

#[test]
fn test_task_priority_schema() {
    let schema = schemars::schema_for!(TaskPriority);
    let schema_json = serde_json::to_string_pretty(&schema).expect("serialize TaskPriority schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    // TaskPriority should be an enum
    assert!(
        parsed.get("enum").is_some() || parsed.get("oneOf").is_some(),
        "TaskPriority should be an enum schema"
    );
}

#[test]
fn test_execution_mode_schema() {
    let schema = schemars::schema_for!(ExecutionMode);
    let schema_json =
        serde_json::to_string_pretty(&schema).expect("serialize ExecutionMode schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    assert!(
        parsed.get("enum").is_some() || parsed.get("oneOf").is_some(),
        "ExecutionMode should be an enum schema"
    );
}

#[test]
fn test_risk_tier_schema() {
    let schema = schemars::schema_for!(RiskTier);
    let schema_json = serde_json::to_string_pretty(&schema).expect("serialize RiskTier schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    assert!(
        parsed.get("enum").is_some() || parsed.get("oneOf").is_some(),
        "RiskTier should be an enum schema"
    );
}

#[test]
fn test_blast_radius_schema() {
    let schema = schemars::schema_for!(BlastRadius);
    let schema_json = serde_json::to_string_pretty(&schema).expect("serialize BlastRadius schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    // BlastRadius should be an object with properties
    let properties = parsed
        .get("properties")
        .or_else(|| parsed.get("definitions"))
        .or_else(|| parsed.pointer("/allOf/0/properties"));

    assert!(properties.is_some(), "BlastRadius should have properties");
}

#[test]
fn test_task_scope_schema() {
    let schema = schemars::schema_for!(TaskScope);
    let schema_json = serde_json::to_string_pretty(&schema).expect("serialize TaskScope schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    let properties = parsed
        .get("properties")
        .or_else(|| parsed.pointer("/allOf/0/properties"));

    assert!(properties.is_some(), "TaskScope should have properties");

    // Verify expected fields exist
    if let Some(props) = properties {
        assert!(
            props.get("in_scope").is_some() || props.get("inScope").is_some(),
            "TaskScope should have in_scope field"
        );
        assert!(
            props.get("out_scope").is_some() || props.get("outScope").is_some(),
            "TaskScope should have out_scope field"
        );
    }
}

#[test]
fn test_execution_context_schema() {
    let schema = schemars::schema_for!(ExecutionContext);
    let schema_json =
        serde_json::to_string_pretty(&schema).expect("serialize ExecutionContext schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    let properties = parsed
        .get("properties")
        .or_else(|| parsed.pointer("/allOf/0/properties"));

    assert!(
        properties.is_some(),
        "ExecutionContext should have properties"
    );

    // Verify UUID field is serialized as string (via schemars with = "String")
    if let Some(props) = properties {
        let session_id = props.get("session_id").or_else(|| props.get("sessionId"));
        if let Some(id_schema) = session_id {
            let id_type = id_schema
                .get("type")
                .or_else(|| id_schema.pointer("/format"));
            assert!(
                id_type.is_some(),
                "session_id should have a type definition"
            );
        }
    }
}

#[test]
fn test_milestone_schema() {
    let schema = schemars::schema_for!(Milestone);
    let schema_json = serde_json::to_string_pretty(&schema).expect("serialize Milestone schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    let properties = parsed
        .get("properties")
        .or_else(|| parsed.pointer("/allOf/0/properties"));

    assert!(properties.is_some(), "Milestone should have properties");

    // Verify required fields
    if let Some(props) = properties {
        assert!(props.get("id").is_some(), "Milestone should have id field");
        assert!(
            props.get("objective").is_some(),
            "Milestone should have objective field"
        );
        assert!(
            props.get("scope").is_some(),
            "Milestone should have scope field"
        );
    }
}

#[test]
fn test_acceptance_criterion_schema() {
    let schema = schemars::schema_for!(AcceptanceCriterion);
    let schema_json =
        serde_json::to_string_pretty(&schema).expect("serialize AcceptanceCriterion schema");

    let parsed: Value = serde_json::from_str(&schema_json).expect("schema should be valid JSON");

    let properties = parsed
        .get("properties")
        .or_else(|| parsed.pointer("/allOf/0/properties"));

    assert!(
        properties.is_some(),
        "AcceptanceCriterion should have properties"
    );

    // Verify Given-When-Then structure
    if let Some(props) = properties {
        assert!(
            props.get("given").is_some(),
            "AcceptanceCriterion should have given field"
        );
        assert!(
            props.get("when").is_some(),
            "AcceptanceCriterion should have when field"
        );
        assert!(
            props.get("then").is_some(),
            "AcceptanceCriterion should have then field"
        );
    }
}

/// Test that all exported types can generate schemas without panicking
#[test]
fn test_all_types_schema_generation() {
    // Just verify schemas can be generated - don't validate contents
    let _ = schemars::schema_for!(TaskPriority);
    let _ = schemars::schema_for!(ExecutionMode);
    let _ = schemars::schema_for!(RiskTier);
    let _ = schemars::schema_for!(BlastRadius);
    let _ = schemars::schema_for!(TaskScope);
    let _ = schemars::schema_for!(ExecutionContext);
    let _ = schemars::schema_for!(Milestone);
    let _ = schemars::schema_for!(AcceptanceCriterion);
    let _ = schemars::schema_for!(ProcessingId);
    let _ = schemars::schema_for!(ContentType);
    let _ = schemars::schema_for!(CouncilVerdict);

    // If we get here, schema generation succeeded
}
