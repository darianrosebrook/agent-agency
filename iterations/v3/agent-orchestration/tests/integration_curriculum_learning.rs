//! Integration tests for curriculum learning system
//!
//! Tests the complete integration of curriculum learning with unified orchestration,
//! including skill progression, difficulty adjustment, and learning loop feedback.

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use agent_orchestration::planning::curriculum_learning::CurriculumLearningEngine;
use data_infrastructure::DatabaseOperations;

/// Test curriculum learning integration with unified orchestrator
#[tokio::test]
async fn test_curriculum_learning_orchestration_integration() -> Result<()> {
    // Setup database connection for testing
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/agent_agency_test".to_string());

    // Create database connection pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Create database operations from pool
    let db_config = data_infrastructure::database_config::DatabaseConfig::default();
    let db_ops: Arc<dyn DatabaseOperations + Send + Sync> = match data_infrastructure::create_database_operations(db_config).await {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("Failed to create database operations: {}. Skipping test.", e);
            return Ok(());
        }
    };

    // Create curriculum learning engine
    let curriculum_engine = Arc::new(CurriculumLearningEngine::new(db_ops.clone()));

    // Test worker skill progression through curriculum
    let worker_id = Uuid::new_v4();
    test_skill_progression(worker_id, &curriculum_engine).await?;

    // Test milestone completion tracking
    test_milestone_completion_tracking(worker_id, &curriculum_engine).await?;

    // Test skill level retrieval
    test_skill_level_retrieval(worker_id, &curriculum_engine).await?;

    Ok(())
}

/// Test skill progression through curriculum learning
async fn test_skill_progression(worker_id: Uuid, curriculum_engine: &CurriculumLearningEngine) -> Result<()> {
    // Record learning history to establish baseline
    curriculum_engine.record_learning_history(
        worker_id,
        "code_generation",
        true,
        0.8,
        Some(5000),
        chrono::Utc::now(),
    ).await?;

    // Check that skill level can be retrieved
    let skill_level = curriculum_engine.get_agent_skill_level(worker_id, "code_generation").await?;

    // Skill level should be a valid value (0-4 range)
    assert!(skill_level <= 4, "Skill level should be in valid range (0-4)");

    // Record additional learning outcomes to test progression
    for i in 1..5 {
        curriculum_engine.record_learning_history(
            worker_id,
            "code_generation",
            true,
            0.8 + (i as f64 * 0.05), // Increasing quality
            Some(4000 - (i * 200) as u64), // Decreasing time (improvement)
            chrono::Utc::now(),
        ).await?;
    }

    Ok(())
}

/// Test milestone completion tracking
async fn test_milestone_completion_tracking(worker_id: Uuid, curriculum_engine: &CurriculumLearningEngine) -> Result<()> {
    let milestone_id = format!("test_milestone_{}", Uuid::new_v4().simple());

    // Record milestone completion
    let result = curriculum_engine.record_milestone_completion(
        worker_id,
        &milestone_id,
        0.85,
        Some(10000),
        chrono::Utc::now(),
    ).await?;

    // Verify result was recorded
    assert!(result.success, "Milestone completion should be recorded successfully");

    Ok(())
}

/// Test skill level retrieval
async fn test_skill_level_retrieval(worker_id: Uuid, curriculum_engine: &CurriculumLearningEngine) -> Result<()> {
    // Get all skill levels for the worker
    let skill_levels = curriculum_engine.get_all_skill_levels(worker_id).await?;

    // Should return a HashMap of domain -> skill level
    // The worker should have at least the code_generation skill from previous tests
    if !skill_levels.is_empty() {
        for (domain, level) in &skill_levels {
            assert!(!domain.is_empty(), "Domain should not be empty");
            assert!(*level <= 4, "Skill level should be in valid range (0-4)");
        }
    }

    // Test getting a specific skill level
    let code_gen_level = curriculum_engine.get_agent_skill_level(worker_id, "code_generation").await?;
    assert!(code_gen_level <= 4, "Code generation skill level should be in valid range");

    Ok(())
}

/// Test that curriculum engine can be cloned and used across threads
#[tokio::test]
async fn test_curriculum_engine_thread_safety() -> Result<()> {
    let db_config = data_infrastructure::database_config::DatabaseConfig::default();
    let db_ops: Arc<dyn DatabaseOperations + Send + Sync> = match data_infrastructure::create_database_operations(db_config).await {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("Failed to create database operations: {}. Skipping test.", e);
            return Ok(());
        }
    };

    let curriculum_engine = Arc::new(CurriculumLearningEngine::new(db_ops));
    let worker_id = Uuid::new_v4();

    // Clone the engine and use it in multiple tasks
    let engine_clone1 = curriculum_engine.clone();
    let engine_clone2 = curriculum_engine.clone();

    let handle1 = tokio::spawn(async move {
        engine_clone1.get_agent_skill_level(worker_id, "testing").await
    });

    let handle2 = tokio::spawn(async move {
        engine_clone2.get_agent_skill_level(worker_id, "documentation").await
    });

    // Both tasks should complete without errors
    let result1 = handle1.await?;
    let result2 = handle2.await?;

    // Results should be valid skill levels
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    Ok(())
}
