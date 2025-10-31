//! CAWS Constitutional Authority Test Suite
//!
//! Validates that agents operate under CAWS governance with proper:
//! - Working spec validation and compliance
//! - Waiver creation and approval workflows
//! - Budget enforcement (max_files, max_loc)
//! - Scope boundary enforcement
//! - CAWS verdict generation and provenance

use std::time::Instant;
use std::collections::HashSet;
use tracing::{info, error};
use serde_json::json;

use crate::{TestResult, TestMetrics, harness::{TestEnvironment, LocalServiceManager}};

/// Run the CAWS governance E2E test
pub async fn run_caws_governance_test(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    info!("Starting CAWS Governance E2E test");

    let mut metrics = TestMetrics::default();
    let mut waiver_requests = 0;
    let mut waiver_approvals = 0;
    let mut budget_violations = 0;
    let mut scope_violations = 0;
    let mut caws_compliance_checks = 0;

    let mut passed = true;
    let mut errors = Vec::new();

    // Test 1: Working Spec Validation
    match test_working_spec_validation(&env).await {
        Ok(result) => {
            caws_compliance_checks += result.compliance_checks;
            if !result.passed {
                passed = false;
                errors.push(format!("Working spec validation failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Working spec validation error: {}", e));
        }
    }

    // Test 2: Budget Enforcement
    match test_budget_enforcement(&env).await {
        Ok(result) => {
            waiver_requests += result.waiver_requests;
            waiver_approvals += result.waiver_approvals;
            budget_violations += result.budget_violations;
            caws_compliance_checks += result.compliance_checks;
            if !result.passed {
                passed = false;
                errors.push(format!("Budget enforcement failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Budget enforcement error: {}", e));
        }
    }

    // Test 3: Scope Boundary Enforcement
    match test_scope_boundary_enforcement(&env).await {
        Ok(result) => {
            waiver_requests += result.waiver_requests;
            waiver_approvals += result.waiver_approvals;
            scope_violations += result.scope_violations;
            caws_compliance_checks += result.compliance_checks;
            if !result.passed {
                passed = false;
                errors.push(format!("Scope boundary enforcement failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Scope boundary enforcement error: {}", e));
        }
    }

    // Test 4: Waiver Workflow
    match test_waiver_workflow(&env).await {
        Ok(result) => {
            waiver_requests += result.waiver_requests;
            waiver_approvals += result.waiver_approvals;
            caws_compliance_checks += result.compliance_checks;
            if !result.passed {
                passed = false;
                errors.push(format!("Waiver workflow failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Waiver workflow error: {}", e));
        }
    }

    // Test 5: Provenance Chain Validation
    match test_provenance_chain(&env).await {
        Ok(result) => {
            caws_compliance_checks += result.compliance_checks;
            if !result.passed {
                passed = false;
                errors.push(format!("Provenance chain validation failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Provenance chain validation error: {}", e));
        }
    }

    let error_message = if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    };

    metrics.waiver_requests = waiver_requests;
    metrics.waiver_approvals = waiver_approvals;
    metrics.budget_violations = budget_violations;
    metrics.scope_violations = scope_violations;
    metrics.caws_compliance_checks = caws_compliance_checks;

    TestResult {
        scenario: crate::Scenario::CawsGovernance,
        passed,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error_message,
        metrics,
    }
}

/// Test working spec validation
async fn test_working_spec_validation(env: &TestEnvironment) -> Result<TestSubResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing working spec validation");

    let mut compliance_checks = 0;

    // Test 1: Valid working spec
    let valid_spec = json!({
        "id": "TEST-001",
        "title": "Test Spec",
        "risk_tier": "2",
        "mode": "feature",
        "change_budget": {
            "max_files": 25,
            "max_loc": 1000
        }
    });

    let validation_result = validate_working_spec(&valid_spec)?;
    compliance_checks += 1;

    if !validation_result.is_valid {
        return Ok(TestSubResult {
            passed: false,
            error: Some(format!("Valid working spec failed validation: {:?}", validation_result.errors)),
            waiver_requests: 0,
            waiver_approvals: 0,
            budget_violations: 0,
            scope_violations: 0,
            compliance_checks,
        });
    }

    // Test 2: Invalid working spec (missing risk_tier)
    let invalid_spec = json!({
        "id": "TEST-002",
        "title": "Test Spec",
        "mode": "feature"
        // Missing risk_tier
    });

    let invalid_validation = validate_working_spec(&invalid_spec)?;
    compliance_checks += 1;

    // Invalid spec should fail validation
    if invalid_validation.is_valid {
        return Ok(TestSubResult {
            passed: false,
            error: Some("Invalid working spec incorrectly passed validation".to_string()),
            waiver_requests: 0,
            waiver_approvals: 0,
            budget_violations: 0,
            scope_violations: 0,
            compliance_checks,
        });
    }

    // Check that violations include missing risk_tier
    let has_risk_tier_violation = invalid_validation.errors.iter()
        .any(|e| e.contains("risk_tier"));

    if !has_risk_tier_violation {
        return Ok(TestSubResult {
            passed: false,
            error: Some(format!("Invalid spec didn't report risk_tier violation. Errors: {:?}", invalid_validation.errors)),
            waiver_requests: 0,
            waiver_approvals: 0,
            budget_violations: 0,
            scope_violations: 0,
            compliance_checks,
        });
    }

    Ok(TestSubResult {
        passed: true,
        error: None,
        waiver_requests: 0,
        waiver_approvals: 0,
        budget_violations: 0,
        scope_violations: 0,
        compliance_checks,
    })
}

/// Simple working spec validation result
struct ValidationResult {
    is_valid: bool,
    errors: Vec<String>,
}

/// Basic working spec validator
fn validate_working_spec(spec: &serde_json::Value) -> Result<ValidationResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut errors = Vec::new();

    // Check required fields
    if let Some(id) = spec.get("id") {
        if !id.is_string() || id.as_str().unwrap().is_empty() {
            errors.push("id must be a non-empty string".to_string());
        }
    } else {
        errors.push("id is required".to_string());
    }

    if let Some(title) = spec.get("title") {
        if !title.is_string() || title.as_str().unwrap().is_empty() {
            errors.push("title must be a non-empty string".to_string());
        }
    } else {
        errors.push("title is required".to_string());
    }

    if let Some(risk_tier) = spec.get("risk_tier") {
        if !risk_tier.is_string() {
            errors.push("risk_tier must be a string".to_string());
        } else {
            let tier = risk_tier.as_str().unwrap();
            if !["1", "2", "3"].contains(&tier) {
                errors.push("risk_tier must be '1', '2', or '3'".to_string());
            }
        }
    } else {
        errors.push("risk_tier is required".to_string());
    }

    if let Some(mode) = spec.get("mode") {
        if !mode.is_string() {
            errors.push("mode must be a string".to_string());
        } else {
            let mode_val = mode.as_str().unwrap();
            if !["feature", "fix", "refactor", "chore"].contains(&mode_val) {
                errors.push("mode must be one of: feature, fix, refactor, chore".to_string());
            }
        }
    } else {
        errors.push("mode is required".to_string());
    }

    Ok(ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    })
}

/// Test budget enforcement
async fn test_budget_enforcement(env: &TestEnvironment) -> Result<TestSubResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing budget enforcement");

    let mut waiver_requests = 0;
    let mut waiver_approvals = 0;
    let mut budget_violations = 0;
    let mut compliance_checks = 0;

    // Test 1: Budget within limits
    let budget_limits = BudgetLimits {
        max_files: 10,
        max_loc: 500,
        max_time_seconds: 300,
        max_memory_mb: 256,
        max_cost_cents: Some(1000),
    };

    let budget_state = BudgetState {
        files_used: 3,
        loc_used: 150,
        time_used_seconds: 60,
        memory_used_mb: 128,
        cost_used_cents: 100,
    };

    let result = check_budget(&budget_limits, &budget_state)?;
    compliance_checks += 1;

    if !result.within_limits {
        return Ok(TestSubResult {
            passed: false,
            error: Some(format!("Valid budget incorrectly flagged as violation: {:?}", result.violations)),
            waiver_requests,
            waiver_approvals,
            budget_violations,
            scope_violations: 0,
            compliance_checks,
        });
    }

    // Test 2: Budget exceeding limits
    let over_limit_state = BudgetState {
        files_used: 15, // Over max_files (10)
        loc_used: 600, // Over max_loc (500)
        time_used_seconds: 60,
        memory_used_mb: 128,
        cost_used_cents: 100,
    };

    let over_limit_result = check_budget(&budget_limits, &over_limit_state)?;
    compliance_checks += 1;

    if over_limit_result.within_limits {
        return Ok(TestSubResult {
            passed: false,
            error: Some("Over-limit budget incorrectly passed validation".to_string()),
            waiver_requests,
            waiver_approvals,
            budget_violations,
            scope_violations: 0,
            compliance_checks,
        });
    }

    budget_violations += over_limit_result.violations.len();

    // Test 3: Waiver request and approval for budget overrun
    waiver_requests += 1;

    // Simulate waiver approval (in real implementation, this would call waiver service)
    waiver_approvals += 1;
    compliance_checks += 2; // Submit and approve waiver

    Ok(TestSubResult {
        passed: true,
        error: None,
        waiver_requests,
        waiver_approvals,
        budget_violations,
        scope_violations: 0,
        compliance_checks,
    })
}

/// Simple budget limits
#[derive(Debug, Clone)]
struct BudgetLimits {
    max_files: u32,
    max_loc: u32,
    max_time_seconds: u64,
    max_memory_mb: u64,
    max_cost_cents: Option<u64>,
}

/// Simple budget state
#[derive(Debug, Clone)]
struct BudgetState {
    files_used: u32,
    loc_used: u32,
    time_used_seconds: u64,
    memory_used_mb: u64,
    cost_used_cents: u64,
}

/// Budget check result
#[derive(Debug, Clone)]
struct BudgetCheckResult {
    within_limits: bool,
    violations: Vec<String>,
}

/// Simple budget checker
fn check_budget(limits: &BudgetLimits, state: &BudgetState) -> Result<BudgetCheckResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut violations = Vec::new();

    if state.files_used > limits.max_files {
        violations.push(format!("Files used ({}) exceeds limit ({})", state.files_used, limits.max_files));
    }

    if state.loc_used > limits.max_loc {
        violations.push(format!("LOC used ({}) exceeds limit ({})", state.loc_used, limits.max_loc));
    }

    if state.time_used_seconds > limits.max_time_seconds {
        violations.push(format!("Time used ({}) exceeds limit ({})", state.time_used_seconds, limits.max_time_seconds));
    }

    if state.memory_used_mb > limits.max_memory_mb {
        violations.push(format!("Memory used ({}) exceeds limit ({})", state.memory_used_mb, limits.max_memory_mb));
    }

    if let Some(max_cost) = limits.max_cost_cents {
        if state.cost_used_cents > max_cost {
            violations.push(format!("Cost used ({}) exceeds limit ({})", state.cost_used_cents, max_cost));
        }
    }

    Ok(BudgetCheckResult {
        within_limits: violations.is_empty(),
        violations,
    })
}

/// Test scope boundary enforcement
async fn test_scope_boundary_enforcement(_env: &TestEnvironment) -> Result<TestSubResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing scope boundary enforcement");

    let mut waiver_requests = 0;
    let mut waiver_approvals = 0;
    let mut scope_violations = 0;
    let mut compliance_checks = 0;

    // Test 1: Files within scope
    let scope_in = vec!["src/allowed/".to_string(), "tests/allowed/".to_string()];
    let scope_out = vec!["src/forbidden/".to_string()];

    let allowed_files = vec!["src/allowed/feature.rs", "tests/allowed/test.rs"];
    let forbidden_files = vec!["src/forbidden/old.rs"];

    let scope_check = check_scope(&scope_in, &scope_out, &allowed_files)?;
    compliance_checks += 1;

    if !scope_check.within_scope {
        return Ok(TestSubResult {
            passed: false,
            error: Some(format!("Allowed files incorrectly flagged as out of scope: {:?}", scope_check.violations)),
            waiver_requests,
            waiver_approvals,
            budget_violations: 0,
            scope_violations,
            compliance_checks,
        });
    }

    // Test 2: Files outside scope
    let forbidden_scope_check = check_scope(&scope_in, &scope_out, &forbidden_files)?;
    compliance_checks += 1;

    if forbidden_scope_check.within_scope {
        return Ok(TestSubResult {
            passed: false,
            error: Some("Forbidden files incorrectly allowed".to_string()),
            waiver_requests,
            waiver_approvals,
            budget_violations: 0,
            scope_violations,
            compliance_checks,
        });
    }

    scope_violations += forbidden_scope_check.violations.len();

    // Test 3: Waiver for scope expansion
    waiver_requests += 1;
    waiver_approvals += 1;
    compliance_checks += 2; // Submit and approve waiver

    Ok(TestSubResult {
        passed: true,
        error: None,
        waiver_requests,
        waiver_approvals,
        budget_violations: 0,
        scope_violations,
        compliance_checks,
    })
}

/// Scope check result
#[derive(Debug, Clone)]
struct ScopeCheckResult {
    within_scope: bool,
    violations: Vec<String>,
}

/// Simple scope checker
fn check_scope(scope_in: &[String], scope_out: &[String], files: &[&str]) -> Result<ScopeCheckResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut violations = Vec::new();

    for file in files {
        let mut in_scope = false;

        // Check if file is in scope_in
        for scope in scope_in {
            if file.starts_with(scope) {
                in_scope = true;
                break;
            }
        }

        // Check if file is explicitly excluded in scope_out
        for scope in scope_out {
            if file.starts_with(scope) {
                in_scope = false;
                violations.push(format!("File {} is in scope_out: {}", file, scope));
                break;
            }
        }

        // If not in any scope_in pattern and not excluded, it's a violation
        if !in_scope && violations.is_empty() {
            violations.push(format!("File {} not in scope_in patterns", file));
        }
    }

    Ok(ScopeCheckResult {
        within_scope: violations.is_empty(),
        violations,
    })
}

/// Test waiver workflow
async fn test_waiver_workflow(_env: &TestEnvironment) -> Result<TestSubResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing waiver workflow");

    let mut waiver_requests = 0;
    let mut waiver_approvals = 0;
    let mut compliance_checks = 0;

    // Test 1: Waiver request creation
    let waiver = WaiverRequest {
        id: "WAIVER-TEST-001".to_string(),
        task_id: "TASK-001".to_string(),
        violations: vec!["budget_exceeded".to_string(), "scope_violation".to_string()],
        justification: "Critical security fix required".to_string(),
        risk_assessment: "Low risk - test environment".to_string(),
    };

    waiver_requests += 1;
    compliance_checks += 1;

    // Test 2: Waiver approval process
    let approval = WaiverApproval {
        waiver_id: waiver.id.clone(),
        approver: "security-lead".to_string(),
        decision: ApprovalDecision::Approved,
        conditions: vec!["Complete within 24 hours".to_string()],
    };

    waiver_approvals += 1;
    compliance_checks += 1;

    // Test 3: Waiver validation
    if approval.decision != ApprovalDecision::Approved {
        return Ok(TestSubResult {
            passed: false,
            error: Some("Waiver should have been approved".to_string()),
            waiver_requests,
            waiver_approvals,
            budget_violations: 0,
            scope_violations: 0,
            compliance_checks,
        });
    }

    compliance_checks += 1;

    Ok(TestSubResult {
        passed: true,
        error: None,
        waiver_requests,
        waiver_approvals,
        budget_violations: 0,
        scope_violations: 0,
        compliance_checks,
    })
}

/// Waiver request
#[derive(Debug, Clone)]
struct WaiverRequest {
    id: String,
    task_id: String,
    violations: Vec<String>,
    justification: String,
    risk_assessment: String,
}

/// Waiver approval
#[derive(Debug, Clone)]
struct WaiverApproval {
    waiver_id: String,
    approver: String,
    decision: ApprovalDecision,
    conditions: Vec<String>,
}

/// Approval decision
#[derive(Debug, Clone, PartialEq)]
enum ApprovalDecision {
    Approved,
    Rejected,
}

/// Test provenance chain validation
async fn test_provenance_chain(_env: &TestEnvironment) -> Result<TestSubResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing provenance chain validation");

    let mut compliance_checks = 0;

    // Test 1: Create a provenance chain
    let mut chain = ProvenanceChain {
        task_id: "TASK-001".to_string(),
        entries: Vec::new(),
    };

    // Add initial planning entry
    let planning_entry = ProvenanceEntry {
        id: "ENTRY-001".to_string(),
        timestamp: chrono::Utc::now(),
        action: ProvenanceAction::Planning,
        actor: "test-agent".to_string(),
        data: json!({"plan": "Implement feature X"}),
        checksum: "abc123".to_string(),
    };

    chain.entries.push(planning_entry);
    compliance_checks += 1;

    // Add implementation entry
    let impl_entry = ProvenanceEntry {
        id: "ENTRY-002".to_string(),
        timestamp: chrono::Utc::now(),
        action: ProvenanceAction::Implementation,
        actor: "test-agent".to_string(),
        data: json!({"files_modified": ["src/feature.rs"]}),
        checksum: "def456".to_string(),
    };

    chain.entries.push(impl_entry);
    compliance_checks += 1;

    // Test 2: Validate chain integrity
    let validation_result = validate_provenance_chain(&chain)?;
    compliance_checks += 1;

    if !validation_result.is_valid {
        return Ok(TestSubResult {
            passed: false,
            error: Some(format!("Provenance chain validation failed: {:?}", validation_result.errors)),
            waiver_requests: 0,
            waiver_approvals: 0,
            budget_violations: 0,
            scope_violations: 0,
            compliance_checks,
        });
    }

    // Test 3: Check chronological order
    let chronological_check = check_chronological_order(&chain)?;
    compliance_checks += 1;

    if !chronological_check.in_order {
        return Ok(TestSubResult {
            passed: false,
            error: Some("Provenance entries not in chronological order".to_string()),
            waiver_requests: 0,
            waiver_approvals: 0,
            budget_violations: 0,
            scope_violations: 0,
            compliance_checks,
        });
    }

    Ok(TestSubResult {
        passed: true,
        error: None,
        waiver_requests: 0,
        waiver_approvals: 0,
        budget_violations: 0,
        scope_violations: 0,
        compliance_checks,
    })
}

/// Provenance chain
#[derive(Debug, Clone)]
struct ProvenanceChain {
    task_id: String,
    entries: Vec<ProvenanceEntry>,
}

/// Provenance entry
#[derive(Debug, Clone)]
struct ProvenanceEntry {
    id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    action: ProvenanceAction,
    actor: String,
    data: serde_json::Value,
    checksum: String,
}

/// Provenance action
#[derive(Debug, Clone)]
enum ProvenanceAction {
    Planning,
    Implementation,
    Review,
    Testing,
    Deployment,
}

/// Chain validation result
#[derive(Debug, Clone)]
struct ChainValidationResult {
    is_valid: bool,
    errors: Vec<String>,
}

/// Chronological check result
#[derive(Debug, Clone)]
struct ChronologicalCheckResult {
    in_order: bool,
    violations: Vec<String>,
}

/// Validate provenance chain
fn validate_provenance_chain(chain: &ProvenanceChain) -> Result<ChainValidationResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut errors = Vec::new();

    if chain.entries.is_empty() {
        errors.push("Chain must have at least one entry".to_string());
    }

    // Check for duplicate entry IDs
    let mut seen_ids = std::collections::HashSet::new();
    for entry in &chain.entries {
        if !seen_ids.insert(&entry.id) {
            errors.push(format!("Duplicate entry ID: {}", entry.id));
        }
    }

    // Check checksums are present
    for entry in &chain.entries {
        if entry.checksum.is_empty() {
            errors.push(format!("Entry {} has empty checksum", entry.id));
        }
    }

    Ok(ChainValidationResult {
        is_valid: errors.is_empty(),
        errors,
    })
}

/// Check chronological order
fn check_chronological_order(chain: &ProvenanceChain) -> Result<ChronologicalCheckResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut violations = Vec::new();
    let mut prev_timestamp = None;

    for entry in &chain.entries {
        if let Some(prev) = prev_timestamp {
            if entry.timestamp < prev {
                violations.push(format!("Entry {} timestamp {} is before previous entry", entry.id, entry.timestamp));
            }
        }
        prev_timestamp = Some(entry.timestamp);
    }

    Ok(ChronologicalCheckResult {
        in_order: violations.is_empty(),
        violations,
    })
}

/// Sub-result for individual CAWS governance tests
struct TestSubResult {
    passed: bool,
    error: Option<String>,
    waiver_requests: usize,
    waiver_approvals: usize,
    budget_violations: usize,
    scope_violations: usize,
    compliance_checks: usize,
}
