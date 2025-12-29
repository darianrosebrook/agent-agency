//! Integration tests for reflexive learning system
//!
//! Tests the complete reflexive learning loop including outcome processing,
//! routing adjustments, and continuous learning integration.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_orchestration::planning::reflexive_learner::{ReflexiveLearner, LearningConfig, RoutingAdjustment};
use agent_orchestration::planning::worker_assignment::WorkerAssignmentStrategy;

/// Test complete reflexive learning loop
#[tokio::test]
async fn test_reflexive_learning_loop() -> Result<()> {
    // Create mock worker assignment strategy
    let worker_assignment_strategy = Arc::new(MockWorkerAssignmentStrategy::new());

    // Create reflexive learner
    let config = LearningConfig {
        min_outcomes_for_adjustment: 3,
        learning_rate: 0.1,
        outcome_decay_factor: 0.95,
        max_history_size: 100,
        enable_auto_adjustments: true,
    };

    let reflexive_learner = Arc::new(ReflexiveLearner::new(
        worker_assignment_strategy.clone(),
        config,
    ));

    // Test outcome processing and learning
    test_outcome_processing(&reflexive_learner).await?;

    // Test routing adjustments generation
    test_routing_adjustments(&reflexive_learner).await?;

    // Test continuous learning loop
    test_continuous_learning(&reflexive_learner).await?;

    // Test performance pattern recognition
    test_pattern_recognition(&reflexive_learner).await?;

    Ok(())
}

/// Test outcome processing and storage
async fn test_outcome_processing(reflexive_learner: &ReflexiveLearner) -> Result<()> {
    let worker_id = Uuid::new_v4();

    // Process multiple outcomes to build learning history
    for i in 0..5 {
        let artifacts = create_test_artifacts(i, true, 0.7 + (i as f64 * 0.05));
        let milestone = create_test_milestone(&format!("Test milestone {}", i), "code_generation");

        let adjustments = reflexive_learner.process_outcome(&artifacts, &milestone, worker_id).await?;
        assert!(!adjustments.is_empty(), "Should generate adjustments for learning");
    }

    // Check that outcomes were recorded
    let history = reflexive_learner.get_outcome_history().await?;
    assert_eq!(history.len(), 5, "Should have recorded all 5 outcomes");

    // Verify outcome details
    for (i, outcome) in history.iter().enumerate() {
        assert_eq!(outcome.worker_id, worker_id);
        assert_eq!(outcome.success, true);
        assert!((outcome.quality_score - (0.7 + (i as f64 * 0.05))).abs() < 0.01);
    }

    Ok(())
}

/// Test routing adjustments generation
async fn test_routing_adjustments(reflexive_learner: &ReflexiveLearner) -> Result<()> {
    let worker_id = Uuid::new_v4();

    // Create varied performance history
    let test_cases = vec![
        (true, 0.9, "Should generate positive adjustments for excellent performance"),
        (true, 0.6, "Should generate moderate adjustments for good performance"),
        (false, 0.3, "Should generate corrective adjustments for poor performance"),
        (true, 0.95, "Should generate strong positive adjustments for outstanding performance"),
    ];

    for (i, (success, quality, description)) in test_cases.iter().enumerate() {
        let artifacts = create_test_artifacts(i as i32 + 10, *success, *quality);
        let milestone = create_test_milestone(&format!("Adjustment test {}", i), "testing");

        let adjustments = reflexive_learner.process_outcome(&artifacts, &milestone, worker_id).await?;

        assert!(!adjustments.is_empty(), "{}", description);

        // Verify adjustment types
        let has_worker_adjustment = adjustments.iter().any(|adj| adj.adjustment_type.contains("worker"));
        assert!(has_worker_adjustment, "Should include worker-specific adjustments");
    }

    Ok(())
}

/// Test continuous learning loop functionality
async fn test_continuous_learning(reflexive_learner: &ReflexiveLearner) -> Result<()> {
    // Start continuous learning loop
    reflexive_learner.start_continuous_learning().await?;

    // Allow some time for learning loop to process
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Verify learning loop is running
    assert!(reflexive_learner.is_learning_loop_active().await?, "Learning loop should be active");

    // Stop learning loop
    reflexive_learner.stop_continuous_learning().await?;

    // Verify learning loop stopped
    assert!(!reflexive_learner.is_learning_loop_active().await?, "Learning loop should be stopped");

    Ok(())
}

/// Test performance pattern recognition
async fn test_pattern_recognition(reflexive_learner: &ReflexiveLearner) -> Result<()> {
    let worker_id = Uuid::new_v4();

    // Create specific performance patterns
    let patterns = vec![
        // Pattern 1: Improving performance over time
        vec![0.5, 0.6, 0.7, 0.8, 0.9],
        // Pattern 2: Consistent high performance
        vec![0.9, 0.9, 0.9, 0.9, 0.9],
        // Pattern 3: Declining performance
        vec![0.9, 0.8, 0.7, 0.6, 0.5],
        // Pattern 4: Erratic performance
        vec![0.5, 0.9, 0.4, 0.95, 0.6],
    ];

    for (pattern_idx, quality_scores) in patterns.iter().enumerate() {
        let pattern_worker_id = Uuid::new_v4();

        // Feed pattern data
        for (i, &quality) in quality_scores.iter().enumerate() {
            let artifacts = create_test_artifacts(
                (pattern_idx * 10 + i) as i32,
                quality > 0.5, // Success if quality > 0.5
                quality
            );
            let milestone = create_test_milestone(
                &format!("Pattern {} step {}", pattern_idx, i),
                "analysis"
            );

            reflexive_learner.process_outcome(&artifacts, &milestone, pattern_worker_id).await?;
        }

        // Test that system recognizes different patterns
        let recent_outcomes = reflexive_learner.get_recent_outcomes(pattern_worker_id, 5).await?;
        assert_eq!(recent_outcomes.len(), 5, "Should have 5 recent outcomes");

        // Verify pattern is captured in learning data
        let avg_quality = recent_outcomes.iter().map(|o| o.quality_score).sum::<f64>() / recent_outcomes.len() as f64;
        let expected_avg = quality_scores.iter().sum::<f64>() / quality_scores.len() as f64;
        assert!((avg_quality - expected_avg).abs() < 0.01, "Pattern should be accurately captured");
    }

    Ok(())
}

/// Test learning integration with curriculum system
#[tokio::test]
async fn test_curriculum_integration() -> Result<()> {
    // This test would require a full database setup
    // For now, test the integration interfaces

    let worker_assignment_strategy = Arc::new(MockWorkerAssignmentStrategy::new());

    // Create curriculum learning engine (mock for this test)
    let curriculum_config = agent_orchestration::planning::curriculum_learning::CurriculumConfig::default();

    // Create reflexive learner with curriculum integration
    let reflexive_learner = ReflexiveLearner::with_curriculum_engine(
        worker_assignment_strategy.clone(),
        // Mock curriculum engine for testing
        MockCurriculumEngine::new(),
        LearningConfig::default(),
    );

    // Test that curriculum integration works
    let artifacts = create_test_artifacts(100, true, 0.85);
    let milestone = create_test_milestone("Curriculum integration test", "code_generation");
    let worker_id = Uuid::new_v4();

    let adjustments = reflexive_learner.process_outcome(&artifacts, &milestone, worker_id).await?;
    assert!(!adjustments.is_empty(), "Should generate adjustments with curriculum integration");

    Ok(())
}

/// Helper function to create test milestones
fn create_test_milestone(description: &str, task_type: &str) -> agent_agency_contracts::planning_io::Milestone {
    use agent_agency_contracts::planning_io::{Milestone, MilestoneState};

    Milestone {
        id: format!("test_milestone_{}", Uuid::new_v4().simple()),
        objective: description.to_string(),
        description: description.to_string(),
        dependencies: vec![],
        estimated_duration_minutes: 30,
        state: MilestoneState::Completed,
        assigned_workers: vec![Uuid::new_v4()],
        artifacts: vec![],
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("task_type".to_string(), serde_json::Value::String(task_type.to_string()));
            meta.insert("complexity".to_string(), serde_json::Value::String("medium".to_string()));
            meta
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Helper function to create test execution artifacts
fn create_test_artifacts(task_id_offset: i32, success: bool, quality_score: f64) -> ExecutionArtifacts {
    ExecutionArtifacts {
        task_id: format!("test_task_{}", task_id_offset),
        milestone_id: format!("test_milestone_{}", task_id_offset),
        execution_start: chrono::Utc::now(),
        execution_end: chrono::Utc::now(),
        success,
        output: Some("Test execution output".to_string()),
        error_message: if success { None } else { Some("Test error".to_string()) },
        metrics: {
            let mut metrics = std::collections::HashMap::new();
            metrics.insert("execution_time_ms".to_string(), serde_json::Value::Number(5000.into()));
            metrics.insert("quality_score".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(quality_score).unwrap()
            ));
            metrics.insert("complexity_score".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(0.7).unwrap()
            ));
            metrics
        },
        artifacts: vec![],
    }
}

/// Mock worker assignment strategy for testing
struct MockWorkerAssignmentStrategy {
    assignments: Arc<RwLock<std::collections::HashMap<String, Uuid>>>,
}

impl MockWorkerAssignmentStrategy {
    fn new() -> Self {
        Self {
            assignments: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl WorkerAssignmentStrategy for MockWorkerAssignmentStrategy {
    async fn assign_worker(&self, milestone: &agent_agency_contracts::planning_io::Milestone) -> Result<Uuid> {
        if let Some(&worker_id) = milestone.assigned_workers.first() {
            Ok(worker_id)
        } else {
            let worker_id = Uuid::new_v4();
            Ok(worker_id)
        }
    }

    async fn get_worker_performance(&self, _worker_id: Uuid) -> Result<f64> {
        Ok(0.8)
    }

    async fn update_worker_performance(&self, _worker_id: Uuid, _performance: f64) -> Result<()> {
        Ok(())
    }
}

/// Mock curriculum engine for testing
struct MockCurriculumEngine;

impl MockCurriculumEngine {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl agent_orchestration::planning::curriculum_learning::CurriculumLearningEngine for MockCurriculumEngine {
    async fn initialize_agent_curriculum(&self, _agent_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn get_agent_skill_level(&self, _agent_id: Uuid, _task_type: &str) -> Result<u32> {
        Ok(2) // Mock intermediate skill level
    }

    async fn record_milestone_completion(
        &self,
        _agent_id: Uuid,
        _milestone_id: &str,
        _quality_score: f64,
        _execution_time_ms: u64,
        _timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        Ok(())
    }

    async fn record_learning_history(
        &self,
        _agent_id: Uuid,
        _task_type: &str,
        _success: bool,
        _quality_score: f64,
        _execution_time_ms: u64,
        _timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        Ok(())
    }

    async fn check_milestone_prerequisites(&self, _agent_id: Uuid, _milestone_id: &str) -> Result<bool> {
        Ok(true)
    }

    async fn get_agent_learning_history(&self, _agent_id: Uuid, _limit: Option<i64>) -> Result<Vec<agent_orchestration::planning::curriculum_learning::LearningOutcome>> {
        Ok(vec![])
    }

    async fn get_agent_curriculum_profile(&self, _agent_id: Uuid) -> Result<agent_orchestration::planning::curriculum_learning::CurriculumProfile> {
        Ok(agent_orchestration::planning::curriculum_learning::CurriculumProfile {
            agent_id: Uuid::new_v4(),
            skill_levels: std::collections::HashMap::new(),
            completed_milestones: vec![],
            active_milestones: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn calculate_difficulty_adjustment(&self, _agent_id: Uuid, _milestone: &agent_agency_contracts::planning_io::Milestone, _current_skill: u32) -> Result<f64> {
        Ok(1.0)
    }
}






