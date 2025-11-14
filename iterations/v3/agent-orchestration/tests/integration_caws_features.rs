//! Integration Tests for CAWS Multi-Spec and Complexity Mode Features
//!
//! Tests the complete CAWS integration updates:
//! 1. Multi-spec resolution priority system
//! 2. Feature-specific spec loading
//! 3. Complexity mode detection and quality requirements
//! 4. Mode-aware evidence gate creation
//! 5. Quality gates execution with mode parameter
//!
//! @author @darianrosebrook

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use agent_agency_contracts::planning_io::ExecutionPlan as ContractExecutionPlan;
use agent_agency_contracts::WorkingSpec;

use agent_orchestration::planning::caws_complexity_mode::{
    CawsComplexityMode, QualityRequirements,
};
use agent_orchestration::planning::caws_integration::CawsPlanBridge;
use agent_orchestration::planning::caws_spec_resolver::{CawsSpecResolver, SpecInfo};

/// Helper to create a test CAWS directory structure
fn setup_test_caws_dir(temp_dir: &TempDir) -> PathBuf {
    let caws_dir = temp_dir.path().join(".caws");
    fs::create_dir_all(&caws_dir).unwrap();
    caws_dir
}

/// Helper to create a test working spec YAML content
fn create_test_spec_yaml(id: &str, title: &str, risk_tier: u8) -> String {
    format!(
        r#"version: "1.0"
id: "{}"
title: "{}"
description: "Test specification"
risk_tier: {}
acceptance_criteria:
  - id: "A1"
    given: "User is logged out"
    when: "User submits valid credentials"
    then: "User is logged in and redirected to dashboard"
scope:
  - allowed_paths: ["src/"]
    blocked_paths: ["node_modules/"]
file_changes: []
coverage_targets:
  line_coverage: 0.85
  branch_coverage: 0.80
  mutation_score: 0.60
"#,
        id, title, risk_tier
    )
}

#[tokio::test]
async fn test_multi_spec_resolution_priority() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Create feature-specific spec
    let feature_spec_path = specs_dir.join("user-auth.yaml");
    fs::write(
        &feature_spec_path,
        create_test_spec_yaml("user-auth", "User Authentication", 1),
    )?;

    // Create legacy spec
    let legacy_spec_path = caws_dir.join("working-spec.yaml");
    fs::write(
        &legacy_spec_path,
        create_test_spec_yaml("legacy", "Legacy Spec", 2),
    )?;

    let resolver = CawsSpecResolver::new(temp_dir.path())?;

    // Priority 1: Feature-specific spec via spec_id
    let resolved = resolver.resolve_spec(Some("user-auth"), None)?;
    assert_eq!(resolved, feature_spec_path);

    // Priority 2: Explicit path
    let resolved = resolver.resolve_spec(None, Some(&legacy_spec_path))?;
    assert_eq!(resolved, legacy_spec_path);

    // Priority 4: Legacy fallback (when no spec_id and no explicit path)
    // Should warn about multiple specs but use legacy
    let resolved = resolver.resolve_spec(None, None)?;
    assert_eq!(resolved, legacy_spec_path);

    Ok(())
}

#[tokio::test]
async fn test_auto_detect_single_spec() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Create single feature spec
    let feature_spec_path = specs_dir.join("feature-1.yaml");
    fs::write(
        &feature_spec_path,
        create_test_spec_yaml("feature-1", "Feature 1", 2),
    )?;

    let resolver = CawsSpecResolver::new(temp_dir.path())?;

    // Should auto-detect single spec
    let resolved = resolver.resolve_spec(None, None)?;
    assert_eq!(resolved, feature_spec_path);

    Ok(())
}

#[tokio::test]
async fn test_multi_agent_context_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Create multiple specs
    fs::write(
        specs_dir.join("feature-1.yaml"),
        create_test_spec_yaml("feature-1", "Feature 1", 2),
    )?;
    fs::write(
        specs_dir.join("feature-2.yaml"),
        create_test_spec_yaml("feature-2", "Feature 2", 2),
    )?;

    let resolver = CawsSpecResolver::new(temp_dir.path())?;

    // Should detect multi-agent context
    assert!(resolver.is_multi_agent_context());

    // List specs
    let specs = resolver.list_specs()?;
    assert_eq!(specs.len(), 2);
    assert!(specs.iter().any(|s| s.id == "feature-1"));
    assert!(specs.iter().any(|s| s.id == "feature-2"));

    Ok(())
}

#[tokio::test]
async fn test_complexity_mode_detection_from_mode_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);

    // Test Simple mode
    fs::write(caws_dir.join("mode"), "simple")?;
    let mode = CawsComplexityMode::detect(temp_dir.path())?;
    assert_eq!(mode, CawsComplexityMode::Simple);

    // Test Standard mode
    fs::write(caws_dir.join("mode"), "standard")?;
    let mode = CawsComplexityMode::detect(temp_dir.path())?;
    assert_eq!(mode, CawsComplexityMode::Standard);

    // Test Enterprise mode
    fs::write(caws_dir.join("mode"), "enterprise")?;
    let mode = CawsComplexityMode::detect(temp_dir.path())?;
    assert_eq!(mode, CawsComplexityMode::Enterprise);

    Ok(())
}

#[tokio::test]
async fn test_complexity_mode_detection_from_config_yaml() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);

    // Create config.yaml
    fs::write(caws_dir.join("config.yaml"), "mode: simple\n")?;
    let mode = CawsComplexityMode::detect(temp_dir.path())?;
    assert_eq!(mode, CawsComplexityMode::Simple);

    // Update to enterprise
    fs::write(caws_dir.join("config.yaml"), "mode: enterprise\n")?;
    let mode = CawsComplexityMode::detect(temp_dir.path())?;
    assert_eq!(mode, CawsComplexityMode::Enterprise);

    Ok(())
}

#[tokio::test]
async fn test_complexity_mode_defaults_to_standard() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // No config files
    let mode = CawsComplexityMode::detect(temp_dir.path())?;
    assert_eq!(mode, CawsComplexityMode::Standard);

    Ok(())
}

#[tokio::test]
async fn test_quality_requirements_by_mode_and_tier() -> Result<()> {
    // Test Simple mode
    let simple_mode = CawsComplexityMode::Simple;
    let reqs_t1 = simple_mode.quality_requirements(1);
    let reqs_t2 = simple_mode.quality_requirements(2);
    let reqs_t3 = simple_mode.quality_requirements(3);

    assert_eq!(reqs_t1.line_coverage, 0.70);
    assert_eq!(reqs_t1.mutation_score, 0.30);
    assert_eq!(reqs_t2.line_coverage, 0.70 * 0.95);
    assert_eq!(reqs_t3.line_coverage, 0.70 * 0.90);
    assert!(!reqs_t1.contracts_required);

    // Test Standard mode
    let standard_mode = CawsComplexityMode::Standard;
    let reqs_t1 = standard_mode.quality_requirements(1);
    let reqs_t2 = standard_mode.quality_requirements(2);

    assert_eq!(reqs_t1.line_coverage, 0.80);
    assert_eq!(reqs_t1.mutation_score, 0.50);
    assert_eq!(reqs_t2.line_coverage, 0.80 * 0.95);
    assert!(reqs_t1.contracts_required);

    // Test Enterprise mode
    let enterprise_mode = CawsComplexityMode::Enterprise;
    let reqs_t1 = enterprise_mode.quality_requirements(1);
    let reqs_t2 = enterprise_mode.quality_requirements(2);

    assert_eq!(reqs_t1.line_coverage, 0.90);
    assert_eq!(reqs_t1.mutation_score, 0.70);
    assert_eq!(reqs_t2.line_coverage, 0.90 * 0.95);
    assert!(reqs_t1.contracts_required);
    assert!(reqs_t1.manual_review_required);

    Ok(())
}

#[tokio::test]
async fn test_bridge_loads_feature_spec() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Create feature spec
    fs::write(
        specs_dir.join("user-auth.yaml"),
        create_test_spec_yaml("user-auth", "User Authentication", 1),
    )?;

    // Set complexity mode
    fs::write(caws_dir.join("mode"), "standard")?;

    let bridge = CawsPlanBridge::with_project_root(temp_dir.path())?;

    // Load feature spec
    let spec = bridge.load_spec(Some("user-auth"), None)?;
    assert_eq!(spec.id, "user-auth");
    assert_eq!(spec.title, "User Authentication");
    assert_eq!(spec.risk_tier, 1);

    Ok(())
}

#[tokio::test]
async fn test_bridge_creates_mode_aware_evidence_gates() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Test with Simple mode
    fs::write(caws_dir.join("mode"), "simple")?;

    // Create spec with Tier 2
    fs::write(
        specs_dir.join("test-spec.yaml"),
        create_test_spec_yaml("test-spec", "Test Spec", 2),
    )?;

    let bridge = CawsPlanBridge::with_project_root(temp_dir.path())?;
    assert_eq!(bridge.complexity_mode(), CawsComplexityMode::Simple);

    // Access evidence gate creation through spec_to_plan conversion
    let spec = bridge.load_spec(Some("test-spec"), None)?;
    let plan = bridge.spec_to_plan(spec)?;
    let gate = &plan.milestones[0].evidence_gate;
    let requirements = CawsComplexityMode::Simple.quality_requirements(2);
    assert_eq!(gate.min_coverage, requirements.line_coverage);
    assert_eq!(gate.min_mutation_score, requirements.mutation_score);

    // Test with Enterprise mode
    fs::write(caws_dir.join("mode"), "enterprise")?;

    // Create spec with Tier 1
    fs::write(
        specs_dir.join("test-spec-2.yaml"),
        create_test_spec_yaml("test-spec-2", "Test Spec 2", 1),
    )?;

    let bridge = CawsPlanBridge::with_project_root(temp_dir.path())?;
    assert_eq!(bridge.complexity_mode(), CawsComplexityMode::Enterprise);

    // Access evidence gate creation through spec_to_plan conversion
    let spec = bridge.load_spec(Some("test-spec-2"), None)?;
    let plan = bridge.spec_to_plan(spec)?;
    let gate = &plan.milestones[0].evidence_gate;
    let requirements = CawsComplexityMode::Enterprise.quality_requirements(1);
    assert_eq!(gate.min_coverage, requirements.line_coverage);
    assert_eq!(gate.min_mutation_score, requirements.mutation_score);
    assert!(gate.security_scan_required); // Enterprise + Tier 1 requires security scan

    Ok(())
}

#[tokio::test]
async fn test_bridge_validates_with_mode_aware_requirements() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Set Enterprise mode (high requirements)
    fs::write(caws_dir.join("mode"), "enterprise")?;

    // Create spec with insufficient coverage for Enterprise mode
    let spec_yaml = format!(
        r#"version: "1.0"
id: "test-spec"
title: "Test Spec"
description: "Test"
risk_tier: 2
acceptance_criteria:
  - id: "A1"
    given: "Test"
    when: "Test"
    then: "Test"
scope:
  - allowed_paths: ["src/"]
    blocked_paths: []
file_changes: []
coverage_targets:
  line_coverage: 0.75
  branch_coverage: 0.70
  mutation_score: 0.40
"#
    );

    fs::write(specs_dir.join("test-spec.yaml"), spec_yaml)?;

    let bridge = CawsPlanBridge::with_project_root(temp_dir.path())?;
    let spec = bridge.load_spec(Some("test-spec"), None)?;

    // Should fail validation - Enterprise mode + Tier 2 requires 0.80 * 0.95 = 0.76 coverage
    // but spec only has 0.75
    assert!(bridge.validate_risk_tier_constraints(&spec).is_err());

    // Update spec to meet requirements
    let spec_yaml = format!(
        r#"version: "1.0"
id: "test-spec"
title: "Test Spec"
description: "Test"
risk_tier: 2
acceptance_criteria:
  - id: "A1"
    given: "Test"
    when: "Test"
    then: "Test"
scope:
  - allowed_paths: ["src/"]
    blocked_paths: []
file_changes: []
coverage_targets:
  line_coverage: 0.80
  branch_coverage: 0.75
  mutation_score: 0.50
"#
    );

    fs::write(specs_dir.join("test-spec.yaml"), spec_yaml)?;
    let spec = bridge.load_spec(Some("test-spec"), None)?;

    // Should pass validation
    assert!(bridge.validate_working_spec(&spec).is_ok());

    Ok(())
}

#[tokio::test]
async fn test_spec_to_plan_with_complexity_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Set Standard mode
    fs::write(caws_dir.join("mode"), "standard")?;

    // Create spec
    fs::write(
        specs_dir.join("test-feature.yaml"),
        create_test_spec_yaml("test-feature", "Test Feature", 2),
    )?;

    let bridge = CawsPlanBridge::with_project_root(temp_dir.path())?;
    let spec = bridge.load_spec(Some("test-feature"), None)?;

    // Convert to plan
    let plan = bridge.spec_to_plan(spec)?;

    // Verify plan has correct quality gates based on mode
    assert_eq!(plan.quality_gates.coverage_requirements.len(), 2); // unit and integration
    assert!(plan.quality_gates.mutation_requirements.required);

    // Verify evidence gates use mode-aware requirements
    for milestone in &plan.milestones {
        let requirements = bridge
            .complexity_mode()
            .quality_requirements(milestone.risk_tier as u8);
        assert_eq!(
            milestone.evidence_gate.min_coverage,
            requirements.line_coverage
        );
        assert_eq!(
            milestone.evidence_gate.min_mutation_score,
            requirements.mutation_score
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_legacy_spec_fallback() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);

    // Create only legacy spec (no specs directory)
    fs::write(
        caws_dir.join("working-spec.yaml"),
        create_test_spec_yaml("legacy", "Legacy Spec", 2),
    )?;

    let bridge = CawsPlanBridge::with_project_root(temp_dir.path())?;

    // Should load legacy spec
    let spec = bridge.load_spec(None, None)?;
    assert_eq!(spec.id, "legacy");

    // Deprecated method should also work
    #[allow(deprecated)]
    let spec_legacy = bridge.load_legacy_spec()?;
    assert_eq!(spec_legacy.id, "legacy");

    Ok(())
}

#[tokio::test]
async fn test_spec_resolver_warns_on_multi_agent_legacy_use() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let caws_dir = setup_test_caws_dir(&temp_dir);
    let specs_dir = caws_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    // Create multiple specs
    fs::write(
        specs_dir.join("feature-1.yaml"),
        create_test_spec_yaml("feature-1", "Feature 1", 2),
    )?;
    fs::write(
        specs_dir.join("feature-2.yaml"),
        create_test_spec_yaml("feature-2", "Feature 2", 2),
    )?;

    // Also create legacy spec
    fs::write(
        caws_dir.join("working-spec.yaml"),
        create_test_spec_yaml("legacy", "Legacy Spec", 2),
    )?;

    let resolver = CawsSpecResolver::new(temp_dir.path())?;

    // Should detect multi-agent context
    assert!(resolver.is_multi_agent_context());

    // Resolving without spec_id should fall back to legacy (with warning in logs)
    let resolved = resolver.resolve_spec(None, None)?;
    assert_eq!(resolved, caws_dir.join("working-spec.yaml"));

    Ok(())
}
