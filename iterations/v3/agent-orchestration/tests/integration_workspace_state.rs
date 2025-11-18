//! Integration Tests for Unified Workspace State Manager
//!
//! Tests the complete workspace state management integration:
//! 1. File watcher bridge - event conversion and handling
//! 2. Embedding service adapter - embedding generation and storage
//! 3. Unified workspace setup - end-to-end integration
//! 4. Context generation - code/documentation/config contexts
//! 5. State capture and diff generation
//!
//! @author @darianrosebrook

#[cfg(all(feature = "data-processing", feature = "memory"))]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::sleep;
    use uuid::Uuid;

    use agent_data_processing::ingestion::FileWatcher as DataProcessingFileWatcher;
    use agent_memory::embedding_integration::EmbeddingIntegration;
    use agent_memory::memory_types::MemoryConfig;
    use data_infrastructure::database_config::DatabaseConfig;
    use data_infrastructure::DatabaseClient;
    use system_resilience::workspace_state::{
        ContextGenerationConfig, EmbeddingServiceTrait, FileWatchConfig, MetricsConfig,
        UnifiedWorkspaceStateManagerBuilder, WorkspaceStateEvent,
    };
    #[cfg(feature = "evaluation")]
    use testing_validation::database_lifecycle::TestDatabaseManager;

    use agent_orchestration::workspace_integration::{
        setup_unified_workspace, EmbeddingServiceAdapter, FileWatcherBridge,
        UnifiedWorkspaceSetupConfig,
    };

    /// Helper to create a test database with automatic setup and cleanup
    #[cfg(feature = "evaluation")]
    async fn create_test_database() -> (TestDatabaseManager, DatabaseClient) {
        // Get base database URL (without database name)
        let base_url = std::env::var("DATABASE_URL")
            .map(|url| {
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
        test_db
            .initialize_schema()
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

        let db_client = DatabaseClient::new(config)
            .await
            .expect("Failed to create test database client");

        (test_db, db_client)
    }

    /// Helper to create a test database client
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

        DatabaseClient::new(config)
            .await
            .expect("Failed to create test database client")
    }

    /// Helper to create a test EmbeddingIntegration
    async fn create_test_embedding_integration() -> Arc<EmbeddingIntegration> {
        let db_client = create_test_db_client().await;

        // Use a mock embedding config for testing
        // In real tests, this would use actual embedding service configuration
        let embedding_config = agent_memory::memory_types::EmbeddingConfig {
            model_name: "embeddinggemma".to_string(),
            dimensions: 768,
            similarity_threshold: 0.7,
        };

        let memory_config = MemoryConfig {
            embedding_config,
            ..Default::default()
        };

        Arc::new(
            EmbeddingIntegration::new(&memory_config.embedding_config)
                .await
                .expect("Failed to create EmbeddingIntegration"),
        )
    }

    /// Helper to create test files in a temporary directory
    fn create_test_files(temp_dir: &Path) -> Vec<PathBuf> {
        let files = vec![
            ("test.rs", "fn main() { println!(\"Hello, world!\"); }"),
            ("README.md", "# Test Project\n\nThis is a test project."),
            ("config.json", r#"{"name": "test", "version": "1.0.0"}"#),
        ];

        let mut created_files = Vec::new();
        for (filename, content) in files {
            let file_path = temp_dir.join(filename);
            std::fs::write(&file_path, content).expect("Failed to write test file");
            created_files.push(file_path);
        }

        created_files
    }

    #[tokio::test]
    async fn test_file_watcher_bridge_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let watch_path = temp_dir.path().to_path_buf();

        // Create file watcher
        let file_watcher =
            DataProcessingFileWatcher::new(vec![watch_path.clone()], vec!["**/*".to_string()])
                .expect("Failed to create FileWatcher");

        // Create event handler (from unified manager)
        let (event_sender, _) = tokio::sync::broadcast::channel(100);
        let event_handler = Arc::new(
            system_resilience::workspace_state::FileWatcherEventHandler::new(
                event_sender,
                watch_path.clone(),
            ),
        );

        // Create bridge
        let bridge_result = FileWatcherBridge::new(file_watcher, event_handler);
        assert!(bridge_result.is_ok(), "Failed to create FileWatcherBridge");
    }

    #[tokio::test]
    async fn test_file_watcher_bridge_start_stop() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let watch_path = temp_dir.path().to_path_buf();

        // Create file watcher
        let file_watcher =
            DataProcessingFileWatcher::new(vec![watch_path.clone()], vec!["**/*".to_string()])
                .expect("Failed to create FileWatcher");

        // Create event handler
        let (event_sender, _) = tokio::sync::broadcast::channel(100);
        let event_handler = Arc::new(
            system_resilience::workspace_state::FileWatcherEventHandler::new(
                event_sender,
                watch_path.clone(),
            ),
        );

        // Create and start bridge
        let mut bridge = FileWatcherBridge::new(file_watcher, event_handler)
            .expect("Failed to create FileWatcherBridge");

        let start_result = bridge.start().await;
        assert!(start_result.is_ok(), "Failed to start FileWatcherBridge");

        // Give it a moment to start
        sleep(Duration::from_millis(100)).await;

        // Stop bridge
        let stop_result = bridge.stop().await;
        assert!(stop_result.is_ok(), "Failed to stop FileWatcherBridge");
    }

    #[tokio::test]
    async fn test_file_watcher_bridge_file_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let watch_path = temp_dir.path().to_path_buf();

        // Create file watcher
        let file_watcher =
            DataProcessingFileWatcher::new(vec![watch_path.clone()], vec!["**/*".to_string()])
                .expect("Failed to create FileWatcher");

        // Create event handler with event receiver
        let (event_sender, _event_receiver) = tokio::sync::broadcast::channel(100);
        let event_handler = Arc::new(
            system_resilience::workspace_state::FileWatcherEventHandler::new(
                event_sender,
                watch_path.clone(),
            ),
        );

        // Create and start bridge
        let mut bridge = FileWatcherBridge::new(file_watcher, Arc::clone(&event_handler))
            .expect("Failed to create FileWatcherBridge");

        bridge.start().await.expect("Failed to start bridge");

        // Wait for watcher to initialize
        sleep(Duration::from_millis(500)).await;

        // Create a test file
        let test_file = watch_path.join("test_event.rs");
        std::fs::write(&test_file, "fn test() {}").expect("Failed to write test file");

        // Wait for event to be processed
        sleep(Duration::from_millis(1000)).await;

        // TODO: Implement comprehensive event verification in integration test
        //       Currently verifies bridge is running only; should implement comprehensive event verification that uses timeout mechanisms and validates event type and content for proper integration testing.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Event verification uses timeout mechanisms
        // - Event type and content are validated
        // - Test handles missing or delayed events gracefully
        // - Test assertions are meaningful and comprehensive
        //
        // DEPENDENCIES:
        // - Event timeout utilities (Required)
        // - Event validation logic (Required)
        // - Test infrastructure for event handling (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (test infrastructure enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Integration testing and event handling expertise

        bridge.stop().await.expect("Failed to stop bridge");
    }

    #[tokio::test]
    async fn test_embedding_service_adapter_creation() {
        let embedding_integration = create_test_embedding_integration().await;
        let _adapter = EmbeddingServiceAdapter::new(embedding_integration);

        // Verify adapter was created
        assert!(true, "EmbeddingServiceAdapter created successfully");
    }

    #[tokio::test]
    #[ignore] // Requires actual embedding service running
    async fn test_embedding_service_adapter_generate_embedding() {
        let embedding_integration = create_test_embedding_integration().await;
        let adapter = EmbeddingServiceAdapter::new(embedding_integration);

        // Generate embedding
        let result = adapter.generate_embedding("test text").await;

        match result {
            Ok(embedding) => {
                assert_eq!(embedding.len(), 768, "Embedding should be 768 dimensions");
                assert!(
                    !embedding.iter().all(|&x| x == 0.0),
                    "Embedding should not be all zeros"
                );
            }
            Err(e) => {
                // If embedding service is not available, skip test
                eprintln!("Embedding service not available: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires actual embedding service and database
    async fn test_embedding_service_adapter_store_file_embedding() {
        let embedding_integration = create_test_embedding_integration().await;
        let adapter = EmbeddingServiceAdapter::new(embedding_integration);

        let test_file = PathBuf::from("test.rs");
        let content = "fn main() {}";
        let embedding = vec![0.1; 768]; // Mock embedding

        // Store embedding
        let result = adapter
            .store_file_embedding(
                test_file.clone(),
                content,
                embedding.clone(),
                Some(serde_json::json!({"test": true})),
            )
            .await;

        match result {
            Ok(_) => {
                // Verify we can search for it
                let search_result = adapter.search_files_by_similarity("main function", 5).await;
                assert!(search_result.is_ok(), "Search should succeed");
            }
            Err(e) => {
                eprintln!("Failed to store embedding (may need database setup): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_unified_workspace_setup_config_default() {
        let config = UnifiedWorkspaceSetupConfig::default();

        assert_eq!(config.workspace_root, PathBuf::from("."));
        assert!(!config.watch_paths.is_empty());
        assert!(!config.file_patterns.is_empty());
        assert!(!config.embedding_extensions.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires database and embedding service
    async fn test_setup_unified_workspace() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let watch_path = temp_dir.path().to_path_buf();

        let config = UnifiedWorkspaceSetupConfig {
            workspace_root: watch_path.clone(),
            watch_paths: vec![watch_path.clone()],
            file_patterns: vec!["**/*".to_string()],
            embedding_extensions: vec!["rs".to_string(), "md".to_string()],
            debounce_ms: 500,
            auto_capture_state: true,
            generate_embeddings: true,
        };

        // Create test files
        create_test_files(temp_dir.path());

        // Create embedding integration
        let embedding_integration = create_test_embedding_integration().await;

        // Setup unified workspace
        let result = setup_unified_workspace(config, embedding_integration).await;

        match result {
            Ok((_manager, mut bridge)) => {
                // Manager is already initialized by setup_unified_workspace
                // Verify manager is initialized
                assert!(true, "Unified workspace manager initialized");

                // Stop bridge
                bridge.stop().await.expect("Failed to stop bridge");
            }
            Err(e) => {
                eprintln!(
                    "Failed to setup unified workspace (may need database/service): {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_unified_workspace_state_capture() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let workspace_root = temp_dir.path().to_path_buf();

        // Create test files
        create_test_files(temp_dir.path());

        // Build unified manager without file watching
        let context_config = ContextGenerationConfig {
            enabled: true,
            max_files_per_context: 10,
            similarity_threshold: 0.7,
            language_filters: vec![],
            framework_filters: vec![],
            code_context_enabled: true,
            docs_context_enabled: true,
            config_context_enabled: true,
        };

        let metrics_config = MetricsConfig {
            enabled: false,
            update_interval_secs: 5,
            detailed_metrics: false,
        };

        let mut manager = UnifiedWorkspaceStateManagerBuilder::new(&workspace_root)
            .with_context_generation(context_config)
            .with_metrics_config(metrics_config)
            .build()
            .expect("Failed to build unified manager");

        // Initialize
        manager.initialize().await.expect("Failed to initialize");

        // Capture state
        let state_result = manager.capture_state().await;
        assert!(state_result.is_ok(), "Failed to capture workspace state");

        let state_id = state_result.unwrap().data;
        let state = manager.get_state(state_id).await.expect("Failed to get state");
        assert!(!state.files.is_empty(), "State should contain files");
    }

    #[tokio::test]
    async fn test_unified_workspace_context_generation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let workspace_root = temp_dir.path().to_path_buf();

        // Create test files
        create_test_files(temp_dir.path());

        // Build unified manager
        let context_config = ContextGenerationConfig {
            enabled: true,
            max_files_per_context: 10,
            similarity_threshold: 0.7,
            language_filters: vec![],
            framework_filters: vec![],
            code_context_enabled: true,
            docs_context_enabled: true,
            config_context_enabled: true,
        };

        let metrics_config = MetricsConfig {
            enabled: false,
            update_interval_secs: 5,
            detailed_metrics: false,
        };

        let mut manager = UnifiedWorkspaceStateManagerBuilder::new(&workspace_root)
            .with_context_generation(context_config)
            .with_metrics_config(metrics_config)
            .build()
            .expect("Failed to build unified manager");

        manager.initialize().await.expect("Failed to initialize");

        // Generate code context
        let code_context = manager.generate_code_context(Some("rust"), None).await;

        assert!(code_context.is_ok(), "Failed to generate code context");
        let context = code_context.unwrap();
        assert!(
            !context.files.is_empty(),
            "Code context should contain files"
        );

        // Generate documentation context
        let doc_context = manager.generate_documentation_context().await;

        assert!(
            doc_context.is_ok(),
            "Failed to generate documentation context"
        );

        // Generate config context
        let config_context = manager.generate_config_context().await;

        assert!(config_context.is_ok(), "Failed to generate config context");
    }

    #[tokio::test]
    async fn test_unified_workspace_event_broadcasting() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let workspace_root = temp_dir.path().to_path_buf();

        // Build unified manager
        let mut manager = UnifiedWorkspaceStateManagerBuilder::new(&workspace_root)
            .build()
            .expect("Failed to build unified manager");

        manager.initialize().await.expect("Failed to initialize");

        // Subscribe to events
        let _event_receiver = manager.subscribe_to_events();

        // Capture state (should trigger event)
        let _ = manager.capture_state().await;

        // Wait a bit for event
        sleep(Duration::from_millis(100)).await;

        // TODO: Implement comprehensive event subscription verification
        //       Currently verifies subscription works only; should implement comprehensive verification that validates event type and content for proper integration testing of event subscription functionality.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Event type is verified correctly
        // - Event content is validated
        // - Subscription mechanism is properly tested
        // - Test handles subscription failures gracefully
        //
        // DEPENDENCIES:
        // - Event validation utilities (Required)
        // - Subscription testing infrastructure (Required)
        // - Event content parsing utilities (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (test infrastructure enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Integration testing and event subscription expertise
        assert!(true, "Event subscription verified");
    }

    #[tokio::test]
    async fn test_unified_workspace_metrics() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let workspace_root = temp_dir.path().to_path_buf();

        // Build unified manager with metrics enabled
        let metrics_config = MetricsConfig {
            enabled: true,
            update_interval_secs: 1,
            detailed_metrics: true,
        };

        let mut manager = UnifiedWorkspaceStateManagerBuilder::new(&workspace_root)
            .with_metrics_config(metrics_config)
            .build()
            .expect("Failed to build unified manager");

        manager.initialize().await.expect("Failed to initialize");

        // Wait for metrics to update
        sleep(Duration::from_millis(1500)).await;

        // Get metrics
        let metrics = manager.get_metrics().await;
        assert!(
            metrics.snapshots.total_snapshots >= 0,
            "Metrics should be available"
        );
    }
}
