//! Integration Tests for Task State Persistence
//!
//! Tests database-backed task state persistence including:
//! 1. State save and load operations
//! 2. Resumable task detection
//! 3. Checkpoint creation and listing
//! 4. State deletion and cleanup
//! 5. Error handling and edge cases
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use anyhow::Result;

use agent_agency_contracts::WorkingSpec;
use agent_orchestration::orchestration::task_state_persistence::{
    DatabaseTaskStatePersistence, TaskStatePersistence, TaskExecutionState, ExecutionStateStatus,
};
use data_infrastructure::simple_client::DatabaseClient;
use data_infrastructure::database_config::DatabaseConfig;
#[cfg(feature = "evaluation")]
use testing_validation::database_lifecycle::TestDatabaseManager;

/// Helper to create a test database with automatic setup and cleanup
#[cfg(feature = "evaluation")]
async fn create_test_database() -> (TestDatabaseManager, DatabaseClient) {
    // Get base database URL (without database name)
    // Extract base connection from DATABASE_URL or use default
    let base_url = std::env::var("DATABASE_URL")
        .map(|url| {
            // Extract base connection (everything before the last /)
            if let Some(last_slash) = url.rfind('/') {
                url[..last_slash].to_string()
            } else {
                url
            }
        })
        .unwrap_or_else(|_| "postgresql://postgres@localhost:5432".to_string());
    
    let admin_url = format!("{}/postgres", base_url);
    
    // Create isolated test database
    let test_db = TestDatabaseManager::new(&admin_url, None)
        .await
        .expect("Failed to create test database");
    
    // Initialize schema (applies all migrations)
    test_db.initialize_schema()
        .await
        .expect("Failed to initialize test database schema");
    
    // Create database client for the test database
    let config = DatabaseConfig {
        database_url: test_db.database_url(),
        pool_max: Some(5),
        connection_timeout: Some(30),
        query_timeout: Some(60),
        ..Default::default()
    };
    
    let db_client = DatabaseClient::new(config).await
        .expect("Failed to create test database client");
    
    (test_db, db_client)
}

/// Helper to create a test database client (legacy - for backward compatibility)
#[cfg(feature = "evaluation")]
async fn create_test_db_client() -> DatabaseClient {
    let (_, client) = create_test_database().await;
    client
}

/// Legacy helper without evaluation feature
#[cfg(not(feature = "evaluation"))]
async fn create_test_db_client() -> DatabaseClient {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost:5432/agent_agency_test".to_string());
    
    let config = DatabaseConfig {
        database_url: database_url.clone(),
        pool_max: Some(5),
        connection_timeout: Some(30),
        query_timeout: Some(60),
        ..Default::default()
    };
    
    DatabaseClient::new(config).await
        .expect("Failed to create test database client")
}

/// Helper to create a test TaskExecutionState
fn create_test_state(task_id: Uuid, status: ExecutionStateStatus) -> TaskExecutionState {
    TaskExecutionState {
        task_id,
        working_spec: WorkingSpec {
            version: "1.0".to_string(),
            id: format!("test-spec-{}", task_id),
            title: "Test Spec".to_string(),
            description: "Test description".to_string(),
            goals: vec![],
            risk_tier: 2,
            constraints: agent_agency_contracts::WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            acceptance_criteria: vec![],
            test_plan: agent_agency_contracts::TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: agent_agency_contracts::RollbackPlan::default(),
            context: agent_agency_contracts::WorkingSpecContext {
                workspace_root: "/tmp".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: HashMap::new(),
                environment: agent_agency_contracts::task_request::Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![],
            metadata: None,
            milestones: vec![],
            change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                max_files: 10,
                max_loc: 100,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            file_changes: vec![],
            coverage_targets: None,
            overview: "Test overview".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        execution_plan: None,
        artifacts: vec![],
        current_iteration: 1,
        quality_scores: vec![0.8],
        current_phase: "execution".to_string(),
        progress_percentage: 50.0,
        status,
        created_at: Utc::now(),
        last_updated: Utc::now(),
        checkpoint_at: None,
        error: None,
        metadata: HashMap::new(),
    }
}

/// Helper to create a test task in the database
async fn create_test_task(db_client: &DatabaseClient, task_id: Uuid) -> Result<()> {
    // Use sqlx directly on the pool for proper parameter binding
    // The DatabaseClient::execute method doesn't support parameterized queries with trait objects
    sqlx::query(
        r#"
        INSERT INTO tasks (id, title, description, priority, status)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(task_id)
    .bind("Test Task")
    .bind("Test task for state persistence")
    .bind(5i32)
    .bind("pending")
    .execute(db_client.pool())
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_save_and_load() {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
    
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_id = Uuid::new_v4();
    
    // Create test task in database
    create_test_task(&db_client, task_id).await.unwrap();
    
    // Create and save state
    let state = create_test_state(task_id, ExecutionStateStatus::Running);
    persistence.save_state(&state).await.unwrap();
    
    // Load state
    let loaded = persistence.load_state(task_id).await.unwrap();
    assert!(loaded.is_some());
    let loaded_state = loaded.unwrap();
    
    // Verify state matches
    assert_eq!(loaded_state.task_id, task_id);
    assert_eq!(loaded_state.status, ExecutionStateStatus::Running);
    assert_eq!(loaded_state.current_iteration, 1);
    assert_eq!(loaded_state.progress_percentage, 50.0);
    
    // Cleanup
    persistence.delete_state(task_id).await.unwrap();
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_list_resumable_tasks() {
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_id_1 = Uuid::new_v4();
    let task_id_2 = Uuid::new_v4();
    let task_id_3 = Uuid::new_v4();
    
    // Create test tasks
    create_test_task(&db_client, task_id_1).await.unwrap();
    create_test_task(&db_client, task_id_2).await.unwrap();
    create_test_task(&db_client, task_id_3).await.unwrap();
    
    // Create states with different statuses
    let state_running = create_test_state(task_id_1, ExecutionStateStatus::Running);
    let state_paused = create_test_state(task_id_2, ExecutionStateStatus::Paused);
    let state_completed = create_test_state(task_id_3, ExecutionStateStatus::Completed);
    
    persistence.save_state(&state_running).await.unwrap();
    persistence.save_state(&state_paused).await.unwrap();
    persistence.save_state(&state_completed).await.unwrap();
    
    // List resumable tasks
    let resumable = persistence.list_resumable_tasks().await.unwrap();
    
    // Should include running and paused, but not completed
    assert!(resumable.contains(&task_id_1));
    assert!(resumable.contains(&task_id_2));
    assert!(!resumable.contains(&task_id_3));
    
    // Cleanup
    persistence.delete_state(task_id_1).await.unwrap();
    persistence.delete_state(task_id_2).await.unwrap();
    persistence.delete_state(task_id_3).await.unwrap();
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_has_resumable_state() {
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_id_running = Uuid::new_v4();
    let task_id_completed = Uuid::new_v4();
    let task_id_nonexistent = Uuid::new_v4();
    
    // Create test tasks
    create_test_task(&db_client, task_id_running).await.unwrap();
    create_test_task(&db_client, task_id_completed).await.unwrap();
    
    // Create states
    let state_running = create_test_state(task_id_running, ExecutionStateStatus::Running);
    let state_completed = create_test_state(task_id_completed, ExecutionStateStatus::Completed);
    
    persistence.save_state(&state_running).await.unwrap();
    persistence.save_state(&state_completed).await.unwrap();
    
    // Check resumable state
    assert!(persistence.has_resumable_state(task_id_running).await.unwrap());
    assert!(!persistence.has_resumable_state(task_id_completed).await.unwrap());
    assert!(!persistence.has_resumable_state(task_id_nonexistent).await.unwrap());
    
    // Cleanup
    persistence.delete_state(task_id_running).await.unwrap();
    persistence.delete_state(task_id_completed).await.unwrap();
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_checkpoints() {
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_id = Uuid::new_v4();
    
    // Create test task
    create_test_task(&db_client, task_id).await.unwrap();
    
    // Create initial state
    let mut state = create_test_state(task_id, ExecutionStateStatus::Running);
    persistence.save_state(&state).await.unwrap();
    
    // Create first checkpoint
    persistence.create_checkpoint(task_id, &state).await.unwrap();
    
    // Update state
    state.current_iteration = 2;
    state.progress_percentage = 75.0;
    persistence.save_state(&state).await.unwrap();
    
    // Create second checkpoint
    persistence.create_checkpoint(task_id, &state).await.unwrap();
    
    // List checkpoints
    let checkpoints = persistence.list_checkpoints(task_id).await.unwrap();
    assert_eq!(checkpoints.len(), 2);
    
    // Verify checkpoints are ordered DESC (most recent first)
    assert!(checkpoints[0] >= checkpoints[1]);
    
    // Cleanup
    persistence.delete_state(task_id).await.unwrap();
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_delete_state() {
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_id = Uuid::new_v4();
    
    // Create test task
    create_test_task(&db_client, task_id).await.unwrap();
    
    // Create and save state
    let state = create_test_state(task_id, ExecutionStateStatus::Running);
    persistence.save_state(&state).await.unwrap();
    
    // Create checkpoint
    persistence.create_checkpoint(task_id, &state).await.unwrap();
    
    // Verify state exists
    assert!(persistence.load_state(task_id).await.unwrap().is_some());
    assert_eq!(persistence.list_checkpoints(task_id).await.unwrap().len(), 1);
    
    // Delete state
    persistence.delete_state(task_id).await.unwrap();
    
    // Verify state is deleted
    assert!(persistence.load_state(task_id).await.unwrap().is_none());
    assert_eq!(persistence.list_checkpoints(task_id).await.unwrap().len(), 0);
    assert!(!persistence.has_resumable_state(task_id).await.unwrap());
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_update_state() {
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_id = Uuid::new_v4();
    
    // Create test task
    create_test_task(&db_client, task_id).await.unwrap();
    
    // Create initial state
    let mut state = create_test_state(task_id, ExecutionStateStatus::Running);
    persistence.save_state(&state).await.unwrap();
    
    // Update state
    state.current_iteration = 3;
    state.progress_percentage = 90.0;
    state.status = ExecutionStateStatus::Paused;
    persistence.save_state(&state).await.unwrap();
    
    // Load and verify updates
    let loaded = persistence.load_state(task_id).await.unwrap().unwrap();
    assert_eq!(loaded.current_iteration, 3);
    assert_eq!(loaded.progress_percentage, 90.0);
    assert_eq!(loaded.status, ExecutionStateStatus::Paused);
    
    // Cleanup
    persistence.delete_state(task_id).await.unwrap();
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_crashed_state_resumable() {
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_id = Uuid::new_v4();
    
    // Create test task
    create_test_task(&db_client, task_id).await.unwrap();
    
    // Create crashed state
    let state = create_test_state(task_id, ExecutionStateStatus::Crashed);
    persistence.save_state(&state).await.unwrap();
    
    // Verify crashed state is resumable
    assert!(persistence.has_resumable_state(task_id).await.unwrap());
    let resumable = persistence.list_resumable_tasks().await.unwrap();
    assert!(resumable.contains(&task_id));
    
    // Cleanup
    persistence.delete_state(task_id).await.unwrap();
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored flag
async fn test_database_persistence_multiple_tasks() {
    // Create isolated test database with automatic migrations
    let (test_db, db_client) = create_test_database().await;
    let db_client = Arc::new(db_client);
    let persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let task_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    
    // Create test tasks and states
    for task_id in &task_ids {
        create_test_task(&db_client, *task_id).await.unwrap();
        let state = create_test_state(*task_id, ExecutionStateStatus::Running);
        persistence.save_state(&state).await.unwrap();
    }
    
    // List resumable tasks
    let resumable = persistence.list_resumable_tasks().await.unwrap();
    assert_eq!(resumable.len(), 5);
    
    // Verify all tasks are in resumable list
    for task_id in &task_ids {
        assert!(resumable.contains(task_id));
        assert!(persistence.has_resumable_state(*task_id).await.unwrap());
    }
    
    // Cleanup
    for task_id in &task_ids {
        persistence.delete_state(*task_id).await.unwrap();
    }
    
    // Drop test database
    test_db.drop_database().await.unwrap();
}
