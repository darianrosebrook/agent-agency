//! End-to-End Integration Tests
//!
//! These tests verify the full pipeline from task request to execution authorization,
//! integrating all V4 crates: Core, Reasoning, and Infrastructure layers.

use v4_arbiter::{Arbiter, DecisionMaker, DenialReason, GateChecker, TaskRouter, VerificationCertificate};
use v4_council::{
    aggregate, AggregationConfig, ChangeType, CodeChange, Council, JudgeEvidence, TestEvidence,
};
use v4_governance::gates::{CAWSEvaluator, CAWSMetrics};
use v4_memory::{EdgeType, MemorySystem};
use v4_observability::init_with_version;
use v4_storage::{Repository, TaskStatus};
use v4_symbolic::{
    DefaultReasoner, ExecutionPlanBuilder, OperatorGraph, OperatorNode,
    RuleContext, RuleEngine, SymbolicReasoner,
};
use v4_types::council::{CouncilVerdict, JudgeType};
use v4_types::operators::SeekOp;
use v4_types::task::{
    AcceptanceCriterion, Environment, TaskConstraints, TaskPriority, TaskRequest, TaskSpec,
};
use v4_types::verdict::Fingerprints;
use v4_types::OperatorType;

// ============================================================================
// Test Fixtures
// ============================================================================

fn make_task_request(id: &str, title: &str) -> TaskRequest {
    TaskRequest {
        id: id.to_string(),
        title: title.to_string(),
        description: format!("Description for {}", title),
        constraints: TaskConstraints::default(),
        priority: TaskPriority::Normal,
        environment: Environment::Development,
        metadata: None,
    }
}

fn make_task_spec(id: &str, risk_tier: u8) -> TaskSpec {
    TaskSpec {
        id: id.to_string(),
        version: "1.0".to_string(),
        title: "Test task".to_string(),
        description: "A test task for council review".to_string(),
        goals: vec!["Complete the test".to_string()],
        risk_tier,
        constraints: TaskConstraints::default(),
        acceptance_criteria: vec![AcceptanceCriterion {
            id: "AC-1".to_string(),
            given: "A test environment".to_string(),
            when: "Tests are run".to_string(),
            then: "All tests pass".to_string(),
        }],
        created_at: chrono::Utc::now(),
    }
}

fn make_good_evidence(task_spec: TaskSpec) -> JudgeEvidence {
    let mut evidence = JudgeEvidence::new(task_spec);
    evidence.proposed_operators = vec!["Seek::ReadFile".to_string()];
    evidence.code_changes = vec![CodeChange {
        path: "src/lib.rs".to_string(),
        change_type: ChangeType::Modified,
        lines_added: 50,
        lines_removed: 10,
        description: "Added new feature".to_string(),
    }];
    evidence.test_results = Some(TestEvidence {
        total_tests: 100,
        passed: 100,
        failed: 0,
        skipped: 0,
        coverage: Some(85.0),
        failed_tests: vec![],
    });
    evidence
}

fn make_fingerprints() -> Fingerprints {
    Fingerprints {
        dataset_sha256: Some("sha256:dataset".to_string()),
        model_sha256: Some("sha256:model".to_string()),
        tokenizer_sha256: Some("sha256:tokenizer".to_string()),
        tool_registry_sha256: Some("sha256:tools".to_string()),
        invariant_set_sha256: Some("sha256:invariants".to_string()),
    }
}

// ============================================================================
// Full Pipeline Tests
// ============================================================================

/// Test the complete pipeline: Task → Symbolic → Council → Arbiter → Authorization
#[tokio::test]
async fn test_full_pipeline_happy_path() {
    // Setup observability
    let obs = init_with_version("1.0.0-test");
    obs.register_standard_metrics();
    obs.health.register_simple("test", || true);

    // Create task
    let task = make_task_request("pipeline-1", "Read configuration files");

    // 1. SYMBOLIC REASONING
    let reasoner = DefaultReasoner::new();
    let proposal = reasoner.propose_operators(&task).await.unwrap();

    // Verify proposal has provenance (INV-CORE-05)
    assert!(proposal.provenance.is_some());
    assert!(!proposal.operators.is_empty());

    // Validate proposal
    let validation = reasoner.validate_proposal(&proposal).await;
    assert!(validation.valid, "Proposal should be valid");

    // Build execution plan
    let plan = ExecutionPlanBuilder::new(&task.id)
        .add_operators(proposal.operators.clone())
        .with_max_iterations(100)
        .build()
        .unwrap();

    // Verify termination guarantee (INV-CORE-07)
    assert!(plan.max_iterations <= 100);

    // 2. COUNCIL REVIEW
    let council = Council::new();
    let evidence = make_good_evidence(make_task_spec(&task.id, 1));
    let verdict = council.quick_review(&evidence).await.unwrap();

    // Low risk + good tests should pass
    assert!(!verdict.vetoed, "Should not be vetoed");
    assert!(verdict.scores.aggregate >= 0.7, "Should have good score");

    // 3. ARBITER DECISION
    let arbiter = Arbiter::new();
    let authorization = arbiter.evaluate(task.clone()).await.unwrap();

    // Verify authorization
    assert!(authorization.is_authorized(), "Should be authorized");

    // Verify certificate (INV-CORE-10)
    let cert = authorization.certificate().unwrap();
    assert!(cert.verify(), "Certificate should verify");

    // Track metrics
    let task_counter = obs.metrics.counter("tasks_completed", "");
    task_counter.inc();
    assert_eq!(task_counter.get(), 1);
}

/// Test pipeline rejection on high-risk task
#[tokio::test]
async fn test_pipeline_high_risk_task() {
    let task = make_task_request("high-risk-1", "Deploy to production");

    // Symbolic reasoning
    let reasoner = DefaultReasoner::new();
    let _proposal = reasoner.propose_operators(&task).await.unwrap();

    // Council review with high risk tier
    let council = Council::new();
    let evidence = JudgeEvidence::new(make_task_spec(&task.id, 5));
    let verdict = council.quick_review(&evidence).await.unwrap();

    // High risk should get lower score
    assert!(
        verdict.scores.constitutional < 0.95,
        "High risk should lower constitutional score"
    );
}

/// Test fail-closed behavior (INV-CORE-09)
#[tokio::test]
async fn test_fail_closed_on_missing_certificate() {
    let decision_maker = DecisionMaker::new();

    // Create passing verdict
    let results = vec![
        v4_council::JudgeEvaluationResult {
            judge_id: "const".to_string(),
            judge_type: JudgeType::Constitutional,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Good".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
        v4_council::JudgeEvaluationResult {
            judge_id: "tech".to_string(),
            judge_type: JudgeType::Technical,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Good".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
        v4_council::JudgeEvaluationResult {
            judge_id: "qual".to_string(),
            judge_type: JudgeType::Quality,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Good".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
    ];

    let config = AggregationConfig::default();
    let verdict = aggregate(&results, &config).unwrap();

    // Pass all gates
    let checker = GateChecker::new();
    let gate_result = checker.check_default();

    // But NO certificate provided - should fail closed
    let auth = decision_maker.decide(&verdict, &gate_result, None, None);

    assert!(!auth.authorized, "Should NOT be authorized without certificate");
    assert!(
        matches!(auth.denial_reason, Some(DenialReason::CertificateFailure)),
        "Denial reason should be CertificateFailure"
    );
}

// ============================================================================
// Invariant Enforcement Tests
// ============================================================================

/// Test INV-CORE-04: Deterministic operator selection
#[test]
fn test_deterministic_selection() {
    let mut engine = RuleEngine::with_defaults();

    let ctx = RuleContext {
        task_title: Some("search for patterns".to_string()),
        task_description: Some("find all occurrences in codebase".to_string()),
        environment: Some(Environment::Development),
        ..Default::default()
    };

    // Same context should produce same hash
    let result1 = engine.select(&ctx);
    let result2 = engine.select(&ctx);

    assert_eq!(
        result1.context_hash, result2.context_hash,
        "Same context must produce same hash"
    );
    assert_eq!(
        result1.operators.len(),
        result2.operators.len(),
        "Same context must select same operators"
    );
}

/// Test INV-CORE-05: Provenance required
#[tokio::test]
async fn test_provenance_required() {
    let task = make_task_request("prov-test", "Test provenance");
    let reasoner = DefaultReasoner::new();

    let proposal = reasoner.propose_operators(&task).await.unwrap();

    // Every proposal must have provenance
    assert!(
        proposal.provenance.is_some(),
        "Proposal must have provenance chain"
    );

    let provenance = proposal.provenance.unwrap();
    assert!(!provenance.entries().is_empty(), "Provenance must have entries");
    assert!(
        provenance.verify_integrity().is_ok(),
        "Provenance must have valid integrity"
    );
}

/// Test INV-CORE-07: Termination guarantee via bounded iterations
#[test]
fn test_termination_guarantee() {
    let mut graph = OperatorGraph::new();

    let op = OperatorType::Seek(SeekOp::ReadFile {
        path: "test.rs".to_string(),
    });

    // Add nodes
    graph.add_node(OperatorNode::new("a", op.clone())).unwrap();
    graph.add_node(OperatorNode::new("b", op.clone())).unwrap();
    graph.add_node(OperatorNode::new("c", op)).unwrap();

    // Create cycle
    graph.add_edge("a", "b").unwrap();
    graph.add_edge("b", "c").unwrap();
    graph.add_edge("c", "a").unwrap();

    // Cycle detection must work
    let cycle = graph.detect_cycle();
    assert!(cycle.is_some(), "Must detect cycle for termination guarantee");

    // Validation should fail with cycle error
    let result = graph.validate();
    assert!(result.is_err(), "Graph with cycle should fail validation");
}

/// Test INV-CORE-08: No hidden routers (logging requirement)
#[test]
fn test_routing_has_reasoning() {
    let router = TaskRouter::new();
    let task = make_task_request("route-test", "Test routing");

    let decision = router.route(&task, 1);

    // Must have reasoning
    assert!(
        !decision.reasoning.primary_reason.is_empty(),
        "Routing must have primary reason"
    );
    assert!(!decision.factors.is_empty(), "Routing must have factors");

    // Factors must have names
    for factor in &decision.factors {
        assert!(
            !factor.name.is_empty(),
            "Each factor must have a name"
        );
    }
}

/// Test INV-CORE-10: Crypto audit trail
#[test]
fn test_certificate_chain() {
    let cert = VerificationCertificate::builder()
        .task_hash("sha256:task123")
        .proposal_hash("sha256:proposal456")
        .verdict_hash("sha256:verdict789")
        .gate_hash("sha256:gate012")
        .issuer("v4-arbiter")
        .build()
        .unwrap();

    // Certificate must verify
    assert!(cert.verify(), "Certificate must verify");

    // Certificate must not be expired
    assert!(!cert.is_expired(), "Fresh certificate must not be expired");

    // Certificate must have all required hashes
    let summary = cert.summary();
    assert!(!summary.certificate_id.is_empty());
    assert!(!summary.certificate_hash.is_empty());
}

// ============================================================================
// Storage + Memory Integration Tests
// ============================================================================

/// Test event sourcing with task lifecycle
#[test]
fn test_event_sourced_task_lifecycle() {
    let repo = Repository::in_memory().unwrap();
    let task_id = "lifecycle-1";

    // Create task
    repo.create_task(task_id, "Implement feature", "Add new functionality", 2)
        .unwrap();

    // Record invariant check
    repo.record_invariant_check("INV-CORE-04", task_id, true, None)
        .unwrap();

    // Record council verdict
    repo.record_council_verdict(task_id, 0.95, 0.90, 0.88, 0.91, true)
        .unwrap();

    // Spawn worker
    repo.spawn_worker("worker-1", task_id, "local").unwrap();

    // Start task
    repo.start_task(task_id, "worker-1").unwrap();

    let task = repo.get_task(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);

    // Complete task
    repo.complete_task(task_id, "sha256:result").unwrap();

    let task = repo.get_task(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Completed);

    // Verify integrity
    assert!(repo.verify_all().unwrap(), "All events must verify");

    // Check event count
    let events = repo.events_for_task(task_id).unwrap();
    assert_eq!(events.len(), 3); // Created, Started, Completed
}

/// Test memory system with graph expansion
#[test]
fn test_memory_graph_expansion() {
    let memory = MemorySystem::new();

    // Store related concepts
    memory
        .store("rust-ownership", "Rust ownership model", "concept")
        .unwrap();
    memory
        .store("rust-borrowing", "Rust borrowing rules", "concept")
        .unwrap();
    memory
        .store("rust-lifetimes", "Rust lifetimes", "concept")
        .unwrap();

    // Add relationships
    memory
        .store_with_relations(
            "rust-memory-safety",
            "Rust memory safety through ownership",
            "concept",
            &[
                ("rust-ownership".to_string(), EdgeType::PartOf),
                ("rust-borrowing".to_string(), EdgeType::PartOf),
                ("rust-lifetimes".to_string(), EdgeType::RelatedTo),
            ],
        )
        .unwrap();

    // Recall should find related items
    let result = memory.recall("memory safety", 2, 10);
    assert!(!result.items.is_empty());

    // Provenance should be tracked
    assert!(!result.provenance.chain_hash.is_empty());
}

/// Test memory decay with success path retention
#[test]
fn test_memory_success_path_retention() {
    let memory = MemorySystem::new();

    // Store item
    memory
        .store("important", "Critical information", "data")
        .unwrap();

    // Record multiple uses (moves to SuccessPath)
    for _ in 0..5 {
        memory.record_use("important").unwrap();
    }

    // Item should retain high weight
    let item = memory.get("important").unwrap();
    assert!(item.weight > 0.9, "Frequently used items should have high weight");
}

// ============================================================================
// Observability Integration Tests
// ============================================================================

/// Test observability context with all components
#[test]
fn test_observability_full_context() {
    let obs = init_with_version("2.0.0");
    obs.register_standard_metrics();

    // Register health checks
    obs.health.register_simple("database", || true);
    obs.health.register_simple("cache", || true);

    // Track metrics
    let requests = obs.metrics.counter("requests_total", "");
    let duration = obs.metrics.histogram("request_duration_seconds", "");

    requests.inc();
    requests.inc();
    duration.observe(0.15);
    duration.observe(0.25);

    assert_eq!(requests.get(), 2);
    let hist = duration.get();
    assert_eq!(hist.count, 2);

    // Check health
    let report = obs.check_health();
    assert!(report.is_healthy());
    assert_eq!(report.checks.len(), 2);
}

/// Test traced span lifecycle
#[test]
fn test_traced_span_lifecycle() {
    let obs = init_with_version("1.0.0");

    {
        let mut span = obs.start_span("test_operation");
        span.set_attribute("task_id", "task-123");
        span.add_event("started");
        span.add_event("processing");
        span.set_ok();
        // Automatically recorded on drop
    }

    let traces = obs.traces.recent_traces(10);
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].span_count(), 1);
    assert!(!traces[0].has_errors());
}

/// Test traced span with error
#[test]
fn test_traced_span_error() {
    let obs = init_with_version("1.0.0");

    {
        let mut span = obs.start_span("failing_operation");
        span.set_error("Connection timeout");
    }

    let error_traces = obs.traces.error_traces(10);
    assert_eq!(error_traces.len(), 1);
}

// ============================================================================
// Cross-Layer Integration Tests
// ============================================================================

/// Test full workflow with storage, reasoning, and observability
#[tokio::test]
async fn test_cross_layer_integration() {
    // Setup
    let obs = init_with_version("1.0.0");
    let repo = Repository::in_memory().unwrap();
    let task_id = "cross-layer-1";

    // 1. Create task in storage
    repo.create_task(task_id, "Cross-layer test", "Test all layers", 1)
        .unwrap();

    // 2. Track with observability
    let task_counter = obs.metrics.counter("tasks_created", "");
    task_counter.inc();

    let mut span = obs.start_span("task_processing");
    span.set_attribute("task_id", task_id);

    // 3. Symbolic reasoning
    let task = make_task_request(task_id, "Cross-layer test");
    let reasoner = DefaultReasoner::new();
    let proposal = reasoner.propose_operators(&task).await.unwrap();

    span.add_event("operators_proposed");

    // 4. Record invariant check
    repo.record_invariant_check("INV-CORE-05", task_id, proposal.provenance.is_some(), None)
        .unwrap();

    // 5. Council review
    let council = Council::new();
    let evidence = make_good_evidence(make_task_spec(task_id, 1));
    let verdict = council.quick_review(&evidence).await.unwrap();

    span.add_event("council_reviewed");

    // 6. Record verdict
    repo.record_council_verdict(
        task_id,
        verdict.scores.constitutional,
        verdict.scores.technical,
        verdict.scores.quality,
        verdict.scores.aggregate,
        !verdict.vetoed,
    )
    .unwrap();

    // 7. Gate check
    let checker = GateChecker::new();
    let gate_result = checker.check_default();

    span.add_event("gates_checked");

    // 8. Record gate result
    repo.record_gate_result(
        "integration-fingerprint",
        task_id,
        if gate_result.can_proceed { 1.0 } else { 0.0 },
        0.5,
    )
    .unwrap();

    // 9. Full arbiter evaluation
    let arbiter = Arbiter::new();
    let auth = arbiter.evaluate(task).await.unwrap();

    if auth.is_authorized() {
        span.set_ok();

        // Start and complete task
        repo.spawn_worker("worker-1", task_id, "local").unwrap();
        repo.start_task(task_id, "worker-1").unwrap();
        repo.complete_task(task_id, "sha256:result").unwrap();

        obs.metrics.counter("tasks_completed", "").inc();
    } else {
        span.set_error("Authorization denied");
        repo.fail_task(task_id, "Authorization denied").unwrap();
    }

    drop(span); // Record the span

    // Verify final state
    let task_proj = repo.get_task(task_id).unwrap();
    let counts = repo.counts();

    // Verify traces recorded
    let traces = obs.traces.recent_traces(10);
    assert!(!traces.is_empty());

    // Verify metrics
    assert_eq!(task_counter.get(), 1);

    // Verify task was created
    assert!(counts.total_tasks >= 1);

    // Verify events were recorded
    let event_count = repo.event_count().unwrap();
    assert!(event_count > 0, "Events should be recorded");
}

/// Test memory integration with task context
#[test]
fn test_memory_task_context() {
    let memory = MemorySystem::new();

    // Store task-related context
    memory
        .store(
            "task-pattern-1",
            "Reading configuration files using Seek operators",
            "pattern",
        )
        .unwrap();
    memory
        .store(
            "task-pattern-2",
            "Writing output files using Modify operators",
            "pattern",
        )
        .unwrap();
    memory
        .store(
            "operator-seek",
            "Seek operators for read-only operations",
            "operator",
        )
        .unwrap();

    // Link patterns to operators
    memory
        .store_with_relations(
            "best-practice-1",
            "Use Seek for reading, Modify for writing",
            "best-practice",
            &[
                ("task-pattern-1".to_string(), EdgeType::RelatedTo),
                ("task-pattern-2".to_string(), EdgeType::RelatedTo),
                ("operator-seek".to_string(), EdgeType::References),
            ],
        )
        .unwrap();

    // Recall best practices for reading tasks
    let result = memory.recall("reading configuration", 2, 10);
    assert!(!result.items.is_empty());

    // Check provenance
    assert!(!result.provenance.entries.is_empty());
}

// ============================================================================
// CAWS Gate Tests
// ============================================================================

/// Test all CAWS gates pass with perfect metrics
#[test]
fn test_caws_gates_perfect_metrics() {
    let evaluator = CAWSEvaluator::new();
    let metrics = CAWSMetrics::perfect();
    let fingerprints = make_fingerprints();

    let verdict = evaluator.evaluate(&metrics, fingerprints);

    assert!(verdict.all_passed, "Perfect metrics should pass all gates");
}

/// Test CAWS gates fail with poor metrics
#[test]
fn test_caws_gates_poor_metrics() {
    let evaluator = CAWSEvaluator::new();
    let metrics = CAWSMetrics::failing();
    let fingerprints = make_fingerprints();

    let verdict = evaluator.evaluate(&metrics, fingerprints);

    assert!(!verdict.all_passed, "Failing metrics should fail gates");
    assert!(!verdict.failed_gates().is_empty());
}

// ============================================================================
// Veto Logic Tests
// ============================================================================

/// Test council veto on low score
#[test]
fn test_council_veto_on_low_score() {
    let results = vec![
        v4_council::JudgeEvaluationResult {
            judge_id: "const".to_string(),
            judge_type: JudgeType::Constitutional,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Good".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
        v4_council::JudgeEvaluationResult {
            judge_id: "tech".to_string(),
            judge_type: JudgeType::Technical,
            score: 0.40, // Below veto threshold
            confidence: 0.9,
            reasoning: "Major issues".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
        v4_council::JudgeEvaluationResult {
            judge_id: "qual".to_string(),
            judge_type: JudgeType::Quality,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Good".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
    ];

    let config = AggregationConfig::default();
    let verdict = aggregate(&results, &config).unwrap();

    assert!(verdict.vetoed, "Should be vetoed due to low technical score");
    assert!(
        verdict
            .vetoing_judges
            .iter()
            .any(|j| *j == JudgeType::Technical),
        "Technical judge should be in vetoing list"
    );
}

/// Test council approval threshold
#[test]
fn test_council_approval_threshold() {
    // All judges give 0.95
    let results = vec![
        v4_council::JudgeEvaluationResult {
            judge_id: "const".to_string(),
            judge_type: JudgeType::Constitutional,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Excellent".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
        v4_council::JudgeEvaluationResult {
            judge_id: "tech".to_string(),
            judge_type: JudgeType::Technical,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Excellent".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
        v4_council::JudgeEvaluationResult {
            judge_id: "qual".to_string(),
            judge_type: JudgeType::Quality,
            score: 0.95,
            confidence: 0.9,
            reasoning: "Excellent".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        },
    ];

    let config = AggregationConfig::default();
    let verdict = aggregate(&results, &config).unwrap();

    assert!(!verdict.vetoed);
    assert!(verdict.scores.aggregate >= 0.90);
    assert!(matches!(
        verdict.verdict,
        CouncilVerdict::Approved { .. }
    ));
}
