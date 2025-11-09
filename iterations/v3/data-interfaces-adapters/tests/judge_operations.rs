//! Integration tests for judge database operations
//!
//! Tests the judge database operations implemented in database_operations_adapter.rs
//! to ensure proper integration with the database and type mapping.
//!
//! @author @darianrosebrook

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;
    use data_infrastructure::DatabaseClient;
    use data_infrastructure::DatabaseConfig;
    use agent_orchestration::planning::data_infrastructure_types::{
        CreateJudge, CreateJudgeEvaluation, Judge, JudgeEvaluation
    };
    use data_interfaces_adapters::database_operations::DatabaseOperationsAdapter;

    /// Helper to create a test database client
    async fn create_test_db_client() -> Arc<DatabaseClient> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost:5432/agent_agency_test".to_string());
        
        let config = DatabaseConfig {
            connection_string: database_url,
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
        };
        
        Arc::new(
            DatabaseClient::new(config).await
                .expect("Failed to create test database client")
        )
    }

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_create_and_get_judge() {
        let db_client = create_test_db_client().await;
        let adapter = DatabaseOperationsAdapter::new(db_client.clone());

        let create_judge = CreateJudge {
            id: Uuid::new_v4(),
            name: "Test Judge".to_string(),
            judge_type: "quality".to_string(),
            configuration: serde_json::json!({
                "model_name": "test-model",
                "endpoint": "http://localhost:8000",
                "weight": 1.0,
                "timeout_ms": 5000,
                "optimization_target": "accuracy",
                "is_active": true
            }),
        };

        let judge = adapter.create_judge(create_judge.clone()).await
            .expect("Failed to create judge");

        assert_eq!(judge.name, create_judge.name);
        assert!(judge.id != Uuid::nil());

        let judges = adapter.get_judges().await
            .expect("Failed to get judges");

        assert!(judges.iter().any(|j| j.id == judge.id));
    }

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_create_and_get_judge_evaluation() {
        let db_client = create_test_db_client().await;
        let adapter = DatabaseOperationsAdapter::new(db_client.clone());

        // First create a judge
        let create_judge = CreateJudge {
            id: Uuid::new_v4(),
            name: "Test Judge".to_string(),
            judge_type: "quality".to_string(),
            configuration: serde_json::json!({
                "model_name": "test-model",
                "endpoint": "http://localhost:8000",
                "weight": 1.0,
                "timeout_ms": 5000,
                "optimization_target": "accuracy",
                "is_active": true
            }),
        };

        let judge = adapter.create_judge(create_judge).await
            .expect("Failed to create judge");

        let task_id = Uuid::new_v4();
        let create_evaluation = CreateJudgeEvaluation {
            judge_id: judge.id,
            task_id,
            evaluation: serde_json::json!({
                "verdict": "approve",
                "confidence": 0.95,
                "reasoning": "Test evaluation"
            }),
            score: 0.95,
        };

        let evaluation = adapter.create_judge_evaluation(create_evaluation.clone()).await
            .expect("Failed to create judge evaluation");

        assert_eq!(evaluation.judge_id, judge.id);
        assert_eq!(evaluation.task_id, task_id);

        let evaluations = adapter.get_judge_evaluations(task_id).await
            .expect("Failed to get judge evaluations");

        assert!(evaluations.iter().any(|e| e.id == evaluation.id));
    }

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_get_judge_evaluations_returns_empty_for_nonexistent_task() {
        let db_client = create_test_db_client().await;
        let adapter = DatabaseOperationsAdapter::new(db_client.clone());

        let nonexistent_task_id = Uuid::new_v4();
        let evaluations = adapter.get_judge_evaluations(nonexistent_task_id).await
            .expect("Failed to get judge evaluations");

        assert!(evaluations.is_empty());
    }
}

