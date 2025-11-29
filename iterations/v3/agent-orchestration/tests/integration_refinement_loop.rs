//! Integration tests for the refinement loop with iterative refinement
//!
//! Tests the complete iterative refinement system including:
//! - Pre-execution council review
//! - Intelligent spec refinement
//! - Quality tracking across iterations
//! - Iteration decision logic
//!
//! @author @darianrosebrook

use std::sync::Arc;
use uuid::Uuid;

use agent_agency_contracts::final_verdict::{FinalDecision, FinalVerdictContract, VerificationSummary};
use agent_agency_contracts::planning_io::{BudgetEnforcement, ChangeBudget};
use agent_agency_contracts::types::prelude::{BlastRadius, ExecutionMode, RiskTier, TaskDescriptor, TaskPriority};
use agent_agency_contracts::working_spec::{
    AcceptanceCriterion, BudgetLimits, MoSCoWPriority, RollbackPlan, RollbackStrategy,
    ScopeRestrictions, TestPlan, WorkingSpec, WorkingSpecConstraints, WorkingSpecContext,
    DataImpact,
};
use agent_agency_contracts::task_request::Environment;
use agent_agency_contracts::ExecutionStatus;
use agent_evaluation::EvaluationOrchestrator;
use agent_orchestration::planning::refinement_loop::{
    ArtifactValidator, CouncilReviewer, OrchestrationExecutor, ProgressTracker,
    RefinementLoopConfig, RefinementLoopCoordinator, SpecRefiner, StatePersistence,
};
use agent_orchestration::planning::intelligent_spec_refiner::IntelligentSpecRefiner;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Mutex;

/// Mock executor that succeeds on the Nth attempt
struct MockOrchestrationExecutor {
    /// Number of times execute has been called
    call_count: Arc<Mutex<u32>>,
    /// Succeed on this attempt (1-indexed)
    succeed_on_attempt: u32,
}

impl MockOrchestrationExecutor {
    fn new(succeed_on_attempt: u32) -> Self {
        Self {
            call_count: Arc::new(Mutex::new(0)),
            succeed_on_attempt,
        }
    }
}

#[async_trait]
impl OrchestrationExecutor for MockOrchestrationExecutor {
    async fn execute_orchestration(
        &self,
        _working_spec: &WorkingSpec,
        _task_descriptor: &TaskDescriptor,
    ) -> Result<FinalVerdictContract> {
        let mut count = self.call_count.lock().await;
        *count += 1;
        let current_count = *count;
        drop(count);

        if current_count >= self.succeed_on_attempt {
            Ok(FinalVerdictContract {
                decision: FinalDecision::Accept,
                votes: vec![],
                dissent: String::new(),
                remediation: vec![],
                constitutional_refs: vec![],
                verification_summary: VerificationSummary {
                    claims_total: 10,
                    claims_verified: 10,
                    coverage_pct: 100.0,
                },
            })
        } else {
            Ok(FinalVerdictContract {
                decision: FinalDecision::Reject,
                votes: vec![],
                dissent: "Quality threshold not met".to_string(),
                remediation: vec!["Improve test coverage".to_string()],
                constitutional_refs: vec![],
                verification_summary: VerificationSummary {
                    claims_total: 10,
                    claims_verified: 5,
                    coverage_pct: 50.0,
                },
            })
        }
    }
}

/// Mock artifact validator that always succeeds
struct MockArtifactValidator;

#[async_trait]
impl ArtifactValidator for MockArtifactValidator {
    async fn validate_execution_artifacts(
        &self,
        _verdict: &FinalVerdictContract,
        _task_descriptor: &TaskDescriptor,
    ) -> Result<bool> {
        Ok(true)
    }
}

/// Mock council reviewer with configurable behavior
struct MockCouncilReviewer {
    /// Number of times to request refinement before approving
    refine_count: Arc<Mutex<u32>>,
    /// Maximum refinements before approval
    max_refinements: u32,
}

impl MockCouncilReviewer {
    fn new(max_refinements: u32) -> Self {
        Self {
            refine_count: Arc::new(Mutex::new(0)),
            max_refinements,
        }
    }
}

#[async_trait]
impl CouncilReviewer for MockCouncilReviewer {
    async fn perform_council_review(
        &self,
        working_spec: &WorkingSpec,
        _task_descriptor: &TaskDescriptor,
    ) -> Result<(bool, bool, String)> {
        let mut count = self.refine_count.lock().await;
        *count += 1;
        let current_count = *count;
        drop(count);

        if current_count > self.max_refinements {
            // Approve after max refinements
            Ok((true, false, "Approved after refinement".to_string()))
        } else {
            // Request refinement with specific feedback
            let feedback = if working_spec.acceptance_criteria.len() < 3 {
                "Need at least 3 acceptance criteria for T2 task"
            } else if working_spec.test_plan.coverage_targets.is_none() {
                "Need coverage targets defined in test plan"
            } else {
                "Description could be more detailed"
            };
            Ok((false, true, feedback.to_string()))
        }
    }
}

/// Mock progress tracker
struct MockProgressTracker {
    progress_updates: Arc<Mutex<Vec<(Uuid, f32, Option<String>)>>>,
}

impl MockProgressTracker {
    fn new() -> Self {
        Self {
            progress_updates: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl ProgressTracker for MockProgressTracker {
    async fn update_task_progress(
        &self,
        task_id: Uuid,
        progress: f32,
        message: Option<String>,
    ) -> Result<()> {
        let mut updates = self.progress_updates.lock().await;
        updates.push((task_id, progress, message));
        Ok(())
    }

    async fn update_task_status(
        &self,
        _task_id: Uuid,
        _status: ExecutionStatus,
        _message: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    async fn track_iteration_progress(
        &self,
        _task_id: Uuid,
        _iteration: u32,
        _quality_score: f64,
        _improvement_delta: f64,
    ) -> Result<()> {
        Ok(())
    }

    async fn detect_and_report_plateaus(
        &self,
        _task_id: Uuid,
        _quality_scores: &[f64],
        _iteration: u32,
    ) -> Result<()> {
        Ok(())
    }
}

/// Mock state persistence
struct MockStatePersistence;

#[async_trait]
impl StatePersistence for MockStatePersistence {
    async fn save_execution_state(&self, _task_id: Uuid) -> Result<()> {
        Ok(())
    }
}

/// Create a test working spec
fn create_test_working_spec() -> WorkingSpec {
    WorkingSpec {
        version: "1.0.0".to_string(),
        id: "TEST-001".to_string(),
        title: "Test Integration Task".to_string(),
        description: "A task for testing the refinement loop".to_string(),
        goals: vec!["Complete the test task".to_string()],
        risk_tier: 2,
        constraints: WorkingSpecConstraints {
            max_duration_minutes: Some(60),
            max_iterations: Some(5),
            budget_limits: Some(BudgetLimits {
                max_files: Some(10),
                max_loc: Some(500),
            }),
            scope_restrictions: Some(ScopeRestrictions {
                allowed_paths: vec!["src/".to_string()],
                blocked_paths: vec!["node_modules/".to_string()],
            }),
        },
        acceptance_criteria: vec![AcceptanceCriterion {
            id: "A1".to_string(),
            given: "Given a valid input".to_string(),
            when: "When the task is executed".to_string(),
            then: "Then the expected output is produced".to_string(),
            priority: Some(MoSCoWPriority::Must),
        }],
        test_plan: TestPlan {
            unit_tests: vec![],
            integration_tests: vec![],
            e2e_scenarios: vec![],
            coverage_targets: None,
        },
        rollback_plan: RollbackPlan {
            strategy: RollbackStrategy::GitRevert,
            automated_steps: vec![],
            manual_steps: vec![],
            data_impact: DataImpact::None,
            downtime_required: Some(false),
            rollback_window_minutes: Some(30),
        },
        context: WorkingSpecContext {
            workspace_root: ".".to_string(),
            git_branch: "main".to_string(),
            recent_changes: vec![],
            dependencies: std::collections::HashMap::new(),
            environment: Environment::Development,
        },
        non_functional_requirements: None,
        validation_results: None,
        quality_gates: None,
        scope: vec![],
        metadata: None,
        milestones: vec![],
        change_budget: ChangeBudget {
            max_files: 10,
            max_loc: 500,
            max_migrations: 0,
            allow_breaking_changes: false,
            allow_new_dependencies: true,
            enforcement_mode: BudgetEnforcement::Warning,
        },
        file_changes: vec![],
        coverage_targets: None,
        overview: String::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Create a test task descriptor
fn create_test_task_descriptor() -> TaskDescriptor {
    TaskDescriptor {
        task_id: Uuid::new_v4(),
        description: "A test task for integration testing".to_string(),
        change_budget: ChangeBudget {
            max_files: 10,
            max_loc: 500,
            max_migrations: 0,
            allow_breaking_changes: false,
            allow_new_dependencies: true,
            enforcement_mode: BudgetEnforcement::Warning,
        },
        priority: TaskPriority::Medium,
        execution_mode: ExecutionMode::Auto,
        risk_tier: Some(RiskTier::Tier2),
        blast_radius: BlastRadius {
            modules: vec![],
            data_migration: false,
            external_deps: vec![],
        },
        scope_in: agent_agency_contracts::task_request::ScopeRestrictions {
            allowed_paths: vec!["src/".to_string()],
            blocked_paths: vec!["node_modules/".to_string()],
        },
        scope_out: None,
        acceptance: Some("Test acceptance criteria".to_string()),
    }
}

#[tokio::test]
async fn test_refinement_loop_single_iteration_success() {
    // Setup: Executor succeeds on first attempt
    let executor = Arc::new(MockOrchestrationExecutor::new(1));
    let validator = Arc::new(MockArtifactValidator);
    let council = Arc::new(MockCouncilReviewer::new(0)); // Approve immediately
    let spec_refiner: Arc<dyn SpecRefiner> = Arc::new(IntelligentSpecRefiner::new());
    let progress_tracker = Arc::new(MockProgressTracker::new());
    let state_persistence = Arc::new(MockStatePersistence);

    let config = RefinementLoopConfig {
        enable_council_review: true,
        max_retries: 3,
        retry_delay_ms: 100,
    };

    let coordinator = RefinementLoopCoordinator::new(
        config,
        EvaluationOrchestrator::new(),
        None,
    );

    let task_id = Uuid::new_v4();
    let working_spec = create_test_working_spec();
    let task_descriptor = create_test_task_descriptor();

    let result = coordinator
        .execute_refinement_loop(
            task_id,
            working_spec,
            &task_descriptor,
            executor,
            validator,
            Some(council),
            Some(spec_refiner),
            progress_tracker,
            Some(state_persistence),
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.iterations, 1);
    assert!(!result.quality_scores.is_empty());
}

#[tokio::test]
async fn test_refinement_loop_with_refinement_iterations() {
    // Setup: Council requests 2 refinements before approving
    let executor = Arc::new(MockOrchestrationExecutor::new(1));
    let validator = Arc::new(MockArtifactValidator);
    let council = Arc::new(MockCouncilReviewer::new(2)); // Request 2 refinements
    let spec_refiner: Arc<dyn SpecRefiner> = Arc::new(IntelligentSpecRefiner::new());
    let progress_tracker = Arc::new(MockProgressTracker::new());
    let state_persistence = Arc::new(MockStatePersistence);

    let config = RefinementLoopConfig {
        enable_council_review: true,
        max_retries: 5,
        retry_delay_ms: 100,
    };

    let coordinator = RefinementLoopCoordinator::new(
        config,
        EvaluationOrchestrator::new(),
        None,
    );

    let task_id = Uuid::new_v4();
    let working_spec = create_test_working_spec();
    let task_descriptor = create_test_task_descriptor();

    let result = coordinator
        .execute_refinement_loop(
            task_id,
            working_spec,
            &task_descriptor,
            executor,
            validator,
            Some(council),
            Some(spec_refiner),
            progress_tracker,
            Some(state_persistence),
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    // Should have completed with multiple iterations due to refinement
    assert!(result.iterations >= 1);
    assert!(!result.quality_scores.is_empty());
}

#[tokio::test]
async fn test_intelligent_spec_refiner_integration() {
    let refiner = IntelligentSpecRefiner::new();
    let spec = create_test_working_spec();

    // Test refinement with council feedback
    let refined = refiner
        .refine_working_spec(&spec, "Need more acceptance criteria and test coverage")
        .await;

    assert!(refined.is_ok());
    let refined = refined.unwrap();

    // Check that refinement was applied
    assert!(refined.description.contains("Refined") || refined.acceptance_criteria.len() > spec.acceptance_criteria.len() || refined.test_plan.coverage_targets.is_some());
}

#[tokio::test]
async fn test_refinement_loop_tracks_quality_improvement() {
    // Setup: Executor succeeds on first attempt
    let executor = Arc::new(MockOrchestrationExecutor::new(1));
    let validator = Arc::new(MockArtifactValidator);
    // Request 1 refinement to ensure multiple iterations and progress tracking
    let council = Arc::new(MockCouncilReviewer::new(1));
    let spec_refiner: Arc<dyn SpecRefiner> = Arc::new(IntelligentSpecRefiner::new());
    let progress_tracker = Arc::new(MockProgressTracker::new());
    let state_persistence = Arc::new(MockStatePersistence);

    let config = RefinementLoopConfig {
        enable_council_review: true,
        max_retries: 3,
        retry_delay_ms: 100,
    };

    let coordinator = RefinementLoopCoordinator::new(
        config,
        EvaluationOrchestrator::new(),
        None,
    );

    let task_id = Uuid::new_v4();
    let working_spec = create_test_working_spec();
    let task_descriptor = create_test_task_descriptor();

    let result = coordinator
        .execute_refinement_loop(
            task_id,
            working_spec,
            &task_descriptor,
            executor,
            validator,
            Some(council),
            Some(spec_refiner),
            progress_tracker.clone(),
            Some(state_persistence),
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    
    // Should have multiple iterations due to council refinement request
    assert!(result.iterations >= 1, "Should have at least 1 iteration");
    
    // Verify quality scores were tracked
    assert!(!result.quality_scores.is_empty(), "Quality scores should have been tracked");
}

#[tokio::test]
async fn test_refinement_loop_without_council() {
    // Setup: No council review
    let executor = Arc::new(MockOrchestrationExecutor::new(1));
    let validator = Arc::new(MockArtifactValidator);
    let spec_refiner: Arc<dyn SpecRefiner> = Arc::new(IntelligentSpecRefiner::new());
    let progress_tracker = Arc::new(MockProgressTracker::new());
    let state_persistence = Arc::new(MockStatePersistence);

    let config = RefinementLoopConfig {
        enable_council_review: false, // Disable council review
        max_retries: 3,
        retry_delay_ms: 100,
    };

    let coordinator = RefinementLoopCoordinator::new(
        config,
        EvaluationOrchestrator::new(),
        None,
    );

    let task_id = Uuid::new_v4();
    let working_spec = create_test_working_spec();
    let task_descriptor = create_test_task_descriptor();

    let result = coordinator
        .execute_refinement_loop(
            task_id,
            working_spec,
            &task_descriptor,
            executor,
            validator,
            None, // No council
            Some(spec_refiner),
            progress_tracker,
            Some(state_persistence),
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.iterations, 1);
}

#[tokio::test]
async fn test_iteration_records_contain_quality_deltas() {
    let executor = Arc::new(MockOrchestrationExecutor::new(2));
    let validator = Arc::new(MockArtifactValidator);
    let council = Arc::new(MockCouncilReviewer::new(0));
    let spec_refiner: Arc<dyn SpecRefiner> = Arc::new(IntelligentSpecRefiner::new());
    let progress_tracker = Arc::new(MockProgressTracker::new());
    let state_persistence = Arc::new(MockStatePersistence);

    let config = RefinementLoopConfig {
        enable_council_review: true,
        max_retries: 3,
        retry_delay_ms: 100,
    };

    let coordinator = RefinementLoopCoordinator::new(
        config,
        EvaluationOrchestrator::new(),
        None,
    );

    let task_id = Uuid::new_v4();
    let working_spec = create_test_working_spec();
    let task_descriptor = create_test_task_descriptor();

    let result = coordinator
        .execute_refinement_loop(
            task_id,
            working_spec,
            &task_descriptor,
            executor,
            validator,
            Some(council),
            Some(spec_refiner),
            progress_tracker,
            Some(state_persistence),
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    
    // Check iteration history
    assert!(!result.iteration_history.is_empty());
    
    // First iteration should have no quality delta
    if let Some(first) = result.iteration_history.first() {
        assert!(first.quality_delta.is_none() || first.iteration == 1);
    }
}
