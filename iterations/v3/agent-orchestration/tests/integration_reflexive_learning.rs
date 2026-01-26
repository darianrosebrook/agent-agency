//! Integration tests for reflexive learning system
//!
//! Tests the complete reflexive learning loop including outcome processing,
//! routing adjustments, and continuous learning integration.
//!
//! Note: These tests require mock database operations since WorkerAssignmentStrategy
//! and ReflexiveLearner depend on real database connections. Full integration tests
//! should be run with a real database connection.

use std::collections::HashMap;
use uuid::Uuid;

use agent_orchestration::planning::curriculum_learning::{CurriculumConfig, CurriculumProfile, LearningOutcome};

/// Test curriculum configuration defaults
#[test]
fn test_curriculum_config_defaults() {
    let config = CurriculumConfig::default();
    
    assert!(config.enabled, "Curriculum learning should be enabled by default");
    assert_eq!(config.min_tasks_for_advancement, 5);
    assert!((config.advancement_success_threshold - 0.8).abs() < 0.01);
    assert!((config.regression_failure_threshold - 0.5).abs() < 0.01);
    assert!(config.enable_difficulty_adjustment);
    assert!((config.difficulty_adjustment_rate - 0.1).abs() < 0.01);
}

/// Test curriculum profile creation
#[test]
fn test_curriculum_profile_creation() {
    let agent_id = Uuid::new_v4();
    let profile = CurriculumProfile::new(agent_id);
    
    assert_eq!(profile.agent_id, agent_id);
    assert!(profile.skill_levels.is_empty());
    assert!(profile.completed_milestones.is_empty());
    assert!(profile.active_milestones.is_empty());
}

/// Test curriculum profile with data
#[test]
fn test_curriculum_profile_with_data() {
    let agent_id = Uuid::new_v4();
    let mut skill_levels = HashMap::new();
    skill_levels.insert("code_generation".to_string(), 3);
    skill_levels.insert("testing".to_string(), 2);
    skill_levels.insert("documentation".to_string(), 1);
    
    let profile = CurriculumProfile {
        agent_id,
        skill_levels,
        completed_milestones: vec!["milestone_1".to_string(), "milestone_2".to_string()],
        active_milestones: vec!["milestone_3".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    assert_eq!(profile.skill_levels.len(), 3);
    assert_eq!(profile.skill_levels.get("code_generation"), Some(&3));
    assert_eq!(profile.completed_milestones.len(), 2);
    assert_eq!(profile.active_milestones.len(), 1);
}

/// Test learning outcome creation
#[test]
fn test_learning_outcome_creation() {
    let agent_id = Uuid::new_v4();
    let outcome = LearningOutcome {
        agent_id,
        task_type: "code_generation".to_string(),
        success: true,
        quality_score: 0.85,
        execution_time_ms: 5000,
        timestamp: chrono::Utc::now(),
    };
    
    assert_eq!(outcome.agent_id, agent_id);
    assert_eq!(outcome.task_type, "code_generation");
    assert!(outcome.success);
    assert!((outcome.quality_score - 0.85).abs() < 0.01);
    assert_eq!(outcome.execution_time_ms, 5000);
}

/// Test learning outcome serialization
#[test]
fn test_learning_outcome_serialization() {
    let agent_id = Uuid::new_v4();
    let outcome = LearningOutcome {
        agent_id,
        task_type: "testing".to_string(),
        success: false,
        quality_score: 0.45,
        execution_time_ms: 10000,
        timestamp: chrono::Utc::now(),
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&outcome).expect("Should serialize");
    
    // Deserialize back
    let deserialized: LearningOutcome = serde_json::from_str(&json).expect("Should deserialize");
    
    assert_eq!(deserialized.agent_id, agent_id);
    assert_eq!(deserialized.task_type, "testing");
    assert!(!deserialized.success);
    assert!((deserialized.quality_score - 0.45).abs() < 0.01);
}

/// Test curriculum profile serialization
#[test]
fn test_curriculum_profile_serialization() {
    let agent_id = Uuid::new_v4();
    let mut skill_levels = HashMap::new();
    skill_levels.insert("refactoring".to_string(), 4);
    
    let profile = CurriculumProfile {
        agent_id,
        skill_levels,
        completed_milestones: vec!["m1".to_string()],
        active_milestones: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&profile).expect("Should serialize");
    
    // Deserialize back
    let deserialized: CurriculumProfile = serde_json::from_str(&json).expect("Should deserialize");
    
    assert_eq!(deserialized.agent_id, agent_id);
    assert_eq!(deserialized.skill_levels.get("refactoring"), Some(&4));
    assert_eq!(deserialized.completed_milestones.len(), 1);
}

// Note: Full integration tests for ReflexiveLearner and WorkerAssignmentStrategy
// require database setup. These tests validate the type definitions and basic
// functionality that doesn't require database connections.
//
// To run full integration tests:
// 1. Start PostgreSQL with test database
// 2. Run migrations: cargo sqlx migrate run
// 3. Run tests with database: DATABASE_URL=... cargo test --features full
