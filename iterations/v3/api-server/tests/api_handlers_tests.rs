#[cfg(test)]
mod tests {
    use agent_agency_api_server::*;
    use agent_agency_system_health_monitor::{SystemHealthMonitor, SystemHealthMonitorConfig};
    use agent_agency_api_server::alerts::AlertManager;
    use agent_agency_api_server::rate_limiter::RateLimiter;
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use axum::{Router, routing::{get, post}};
    use tower::ServiceExt;
    use serde_json::json;
    use std::sync::Arc;

    // Mock task store for testing
    struct MockTaskStore {
        tasks: Arc<std::sync::RwLock<Vec<PersistedTask>>>,
    }

    impl MockTaskStore {
        fn new(tasks: Arc<std::sync::RwLock<Vec<PersistedTask>>>) -> Self {
            Self { tasks }
        }
    }

    // Mock rate limiter for testing
    struct MockRateLimiter;

    impl MockRateLimiter {
        fn new() -> Self {
            Self
        }

        async fn check_rate_limit(&self, _client_ip: std::net::IpAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(()) // Always allow
        }
    }

    #[async_trait::async_trait]
    impl TaskStoreTrait for MockTaskStore {
        async fn create_task(&self, task: PersistedTask) -> anyhow::Result<()> {
            self.tasks.write().unwrap().push(task);
            Ok(())
        }

        async fn get_tasks(&self) -> anyhow::Result<Vec<PersistedTask>> {
            Ok(self.tasks.read().unwrap().clone())
        }

        async fn get_task(&self, task_id: String) -> anyhow::Result<Option<PersistedTask>> {
            Ok(self.tasks.read().unwrap().iter().find(|t| t.id == task_id).cloned())
        }

        async fn get_task_events(&self, _task_id: String) -> anyhow::Result<Vec<serde_json::Value>> {
            Ok(vec![json!({"event": "task_created", "timestamp": "2024-01-01T00:00:00Z"})])
        }
    }

    fn create_test_app_state() -> (AppState, Arc<std::sync::RwLock<Vec<PersistedTask>>>) {
        let tasks = Arc::new(std::sync::RwLock::new(Vec::new()));

        let mock_store = Arc::new(MockTaskStore::new(Arc::clone(&tasks)));

        let config = SystemHealthMonitorConfig::default();
        let health_monitor = Arc::new(SystemHealthMonitor::new(config));
        let alert_manager = Arc::new(AlertManager::new(None)); // No reliability monitor for testing
        let rate_limiter = Arc::new(RateLimiter::new()); // Simple rate limiter for testing

        let app_state = AppState {
            task_store: mock_store,
            health_monitor,
            alert_manager,
            rate_limiter,
        };

        (app_state, tasks)
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let app = Router::new().route("/health", get(health_check));

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(health_response["status"], "healthy");
        assert_eq!(health_response["service"], "agent-agency-v3-api");
        assert!(health_response["timestamp"].is_string());
        assert!(health_response["uptime_seconds"].is_number());
    }

    #[tokio::test]
    async fn test_list_tasks_empty() {
        let (app_state, _) = create_test_app_state();
        let app = Router::new()
            .route("/tasks", get(list_tasks))
            .with_state(app_state);

        let response = app
            .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let tasks_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(tasks_response["tasks"], json!([]));
        assert_eq!(tasks_response["total"], 0);
        assert_eq!(tasks_response["status"], "success");
    }

    #[tokio::test]
    async fn test_list_tasks_with_data() {
        let (app_state, tasks_store) = create_test_app_state();

        // Add a test task
        let task = PersistedTask {
            id: "test-task-1".to_string(),
            spec: json!({"description": "Test task", "priority": "high"}).to_string(),
            state: "pending".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            created_by: Some("test-user".to_string()),
            metadata: json!({"source": "test"}).to_string(),
        };
        tasks_store.write().unwrap().push(task);

        let app = Router::new()
            .route("/tasks", get(list_tasks))
            .with_state(app_state);

        let response = app
            .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let tasks_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(tasks_response["total"], 1);
        assert_eq!(tasks_response["status"], "success");

        let tasks = tasks_response["tasks"].as_array().unwrap();
        assert_eq!(tasks.len(), 1);

        let task_data = &tasks[0];
        assert_eq!(task_data["id"], "test-task-1");
        assert_eq!(task_data["title"], "Test task");
        assert_eq!(task_data["status"], "pending");
        assert_eq!(task_data["priority"], "high");
    }

    #[tokio::test]
    async fn test_get_task_found() {
        let (app_state, tasks_store) = create_test_app_state();

        // Add a test task
        let task = PersistedTask {
            id: "test-task-1".to_string(),
            spec: json!({"description": "Test task", "context": "Test context"}).to_string(),
            state: "completed".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            created_by: Some("test-user".to_string()),
            metadata: json!({"duration_ms": 1500}).to_string(),
        };
        tasks_store.write().unwrap().push(task);

        let app = Router::new()
            .route("/tasks/:task_id", get(get_task))
            .with_state(app_state);

        let response = app
            .oneshot(Request::builder().uri("/tasks/test-task-1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let task_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(task_response["task"]["id"], "test-task-1");
        assert_eq!(task_response["task"]["status"], "completed");
        assert_eq!(task_response["task"]["createdBy"], "test-user");
        assert!(task_response["events"].is_array());
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let (app_state, _) = create_test_app_state();
        let app = Router::new()
            .route("/tasks/:task_id", get(get_task))
            .with_state(app_state);

        let response = app
            .oneshot(Request::builder().uri("/tasks/non-existent").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let task_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(task_response["error"], "Task not found");
        assert_eq!(task_response["status"], "error");
    }

    // TODO: Add submit_task tests once ConnectInfo mocking is figured out

    #[tokio::test]
    async fn test_get_api_metrics() {
        let app = Router::new().route("/metrics", get(get_api_metrics));

        let response = app
            .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let metrics_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(metrics_response["status"], "ok");
        assert!(metrics_response["timestamp"].is_number());
        // The metrics are nested under "metrics"
        let metrics = &metrics_response["metrics"];
        assert!(metrics["uptime_seconds"].is_number());
        assert!(metrics["memory_usage_mb"].is_number());
    }

    #[tokio::test]
    async fn test_create_chat_session() {
        let app = Router::new().route("/chat/sessions", post(create_chat_session));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/chat/sessions")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let session_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(session_response["status"], "created");
        assert!(session_response["sessionId"].is_string());
    }

    #[tokio::test]
    async fn test_get_websocket_config() {
        let app = Router::new().route("/chat/sessions/:session_id/ws-config", get(get_websocket_config));

        let response = app
            .oneshot(Request::builder().uri("/chat/sessions/test-session/ws-config").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let config_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(config_response["session_id"], "test-session");
        assert!(config_response["backend_url"].is_string());
        assert!(config_response["protocols"].is_array());
    }

    #[tokio::test]
    async fn test_stub_endpoints() {
        let app = Router::new()
            .route("/waivers", get(list_waivers))
            .route("/waivers", post(create_waiver))
            .route("/waivers/:waiver_id/approve", post(approve_waiver))
            .route("/tasks/:task_id/provenance", get(get_task_provenance));

        // Test list waivers
        let response = app.clone()
            .oneshot(Request::builder().uri("/waivers").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test create waiver
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/waivers")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({}).to_string()))
                    .unwrap()
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test approve waiver
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/waivers/test-waiver/approve")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test task provenance
        let response = app
            .oneshot(Request::builder().uri("/tasks/test-task/provenance").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
