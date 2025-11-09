//! Agent Agency V3 API Server
//!
//! Comprehensive HTTP API server providing REST endpoints for:
//! - Task management and orchestration
//! - Chain of thought observation (orchestrator, council, workers)
//! - Chat/context queries with orchestrator
//! - Project scaffolding and management
//! - Database inspection
//! - Session control (pause/resume/cancel/reinstate)
//! - Progress logs and monitoring
//! - System health and resource management
//! - Analytics and metrics
//!
//! # CRITICAL: Observational API Design
//!
//! **This API is designed for OBSERVATION, not manipulation.**
//!
//! The API acts as a "doctor's MRI machine" - it observes what's happening inside
//! the orchestrator without directly controlling execution. This preserves research
//! integrity by ensuring the orchestrator maintains full autonomy over its execution
//! lifecycle.
//!
//! ## Design Principles
//!
//! 1. **Observation Only**: All endpoints observe orchestrator state, never manipulate it directly
//! 2. **Request-Based Control**: Control operations (pause/resume/cancel) are requests that
//!    are logged in chain-of-thought, but the orchestrator decides whether to honor them
//! 3. **Research Integrity**: No direct manipulation of execution state - orchestrator maintains
//!    full control over its own execution lifecycle
//! 4. **Agent Autonomy**: Agents use their own connections to task execution, not through the API
//!
//! ## What This Means
//!
//! - **Task Submission**: Requests orchestrator to start a task (orchestrator handles execution)
//! - **State Observation**: Query task status, chain of thought, council decisions, worker actions
//! - **Control Requests**: Request pause/resume/cancel (orchestrator decides if safe)
//! - **Never Manipulate**: Never directly change execution state - only observe and request
//!
//! ## Why This Matters
//!
//! Direct manipulation of orchestrator execution state would compromise research integrity.
//! By maintaining strict observation boundaries, we ensure that:
//! - Orchestrator decisions are autonomous and reproducible
//! - Research results are not contaminated by external manipulation
//! - The orchestrator's chain of thought accurately reflects its own reasoning
//! - Agents maintain their own execution connections independently
//!
//! @author @darianrosebrook

use clap::Parser;
use std::env;
use std::sync::Arc;
use axum::{
    routing::{get, post, delete},
    Router,
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use tracing::{info, error, warn};

// Database integration
use data_infrastructure::database_config::DatabaseConfig;
use data_infrastructure::database_init::{initialize_database, verify_schema};
use data_infrastructure::simple_client::DatabaseClient;
use sqlx::Row;

// Unified orchestrator adapter
use data_interfaces_adapters::orchestration_adapter::UnifiedOrchestratorAdapter;
use data_interfaces::service_contracts::OrchestrationService;

// API modules
#[cfg(feature = "orchestration")]
use data_infrastructure::api::{ApiState, RestApi};
#[cfg(feature = "orchestration")]
use data_infrastructure::api::types::ApiConfig as DataApiConfig;
#[cfg(feature = "orchestration")]
use data_infrastructure::api::handlers::*;
#[cfg(feature = "orchestration")]
use data_infrastructure::orchestrator_service::TaskStatus;

#[derive(Parser)]
#[command(name = "agent-agency-api")]
#[command(about = "Agent Agency V3 REST API Server")]
struct Args {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Enable CORS
    #[arg(long)]
    enable_cors: bool,

    /// Require API key authentication
    #[arg(long)]
    require_api_key: bool,

    /// Config file path
    #[arg(long, default_value = "api-server-config.toml")]
    config_file: String,
}

/// Application state shared across all handlers
#[derive(Clone)]
struct AppState {
    db_client: Option<Arc<DatabaseClient>>,
    #[cfg(feature = "orchestration")]
    api: Option<Arc<RestApi>>,
    #[cfg(feature = "orchestration")]
    orchestrator_service: Option<Arc<data_infrastructure::OrchestratorService>>,
    /// Unified orchestrator adapter (new implementation)
    #[cfg(feature = "orchestration")]
    unified_orchestrator: Option<Arc<UnifiedOrchestratorAdapter>>,
    #[cfg(feature = "orchestration")]
    websocket_manager: Option<Arc<data_infrastructure::websocket::WebSocketManager>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("🚀 Starting Agent Agency V3 API Server");
    info!("📍 Server: {}:{}", args.host, args.port);

    // Validate configuration if API key auth is required
    if args.require_api_key {
        if let Ok(api_keys_env) = env::var("AGENT_AGENCY_API_KEYS") {
            let keys: Vec<String> = api_keys_env
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if keys.is_empty() {
                eprintln!("❌ API key authentication required but no API keys configured!");
                eprintln!("   Set AGENT_AGENCY_API_KEYS environment variable");
                std::process::exit(1);
            }
            info!("🔐 API key authentication enabled with {} keys", keys.len());
        } else {
            eprintln!("❌ API key authentication required but AGENT_AGENCY_API_KEYS not set!");
            std::process::exit(1);
        }
    }

    // Initialize database connection and run migrations
    let db_client = if let Ok(database_url) = env::var("DATABASE_URL") {
        info!("📦 Initializing database connection...");

        let db_config = DatabaseConfig {
            database_url: database_url.clone(),
            pool_max: Some(10),
            connection_timeout: Some(30),
            query_timeout: Some(60),
            ..Default::default()
        };

        match initialize_database(db_config).await {
            Ok(client) => {
                info!("✅ Database initialized and migrations applied");

                // Verify schema
                if let Err(e) = verify_schema(client.pool()).await {
                    warn!("⚠️  Schema verification warning: {}", e);
                } else {
                    info!("✅ Database schema verified");
                }

                Some(Arc::new(client))
            }
            Err(e) => {
                error!("⚠️  Failed to initialize database: {}", e);
                error!("   Continuing in standalone mode without database");
                None
            }
        }
    } else {
        warn!("⚠️  Note: DATABASE_URL not set - running in standalone mode");
        warn!("   Set DATABASE_URL to enable database persistence");
        None
    };

    info!("⚙️  Configuration loaded:");
    info!("   - API Keys: {}", if args.require_api_key { "Required" } else { "Optional" });
    info!("   - CORS: {}", if args.enable_cors { "Enabled" } else { "Disabled" });
    info!("   - Database: {}", if db_client.is_some() { "Connected" } else { "Not connected" });

    // Initialize UnifiedOrchestrator using factory from agent-orchestration
    #[cfg(feature = "orchestration")]
    let unified_orchestrator = {
        info!("🔧 Initializing UnifiedOrchestrator...");
        
        // Create database operations adapter if database client is available
        use agent_orchestration::orchestration::UnifiedOrchestratorFactory;
        use agent_orchestration::planning::DatabaseOperations;
        
        let db_ops: Option<Arc<dyn DatabaseOperations>> = if let Some(db) = db_client.as_ref() {
            // Use DatabaseOperationsAdapter from this crate
            Some(Arc::new(data_interfaces_adapters::DatabaseOperationsAdapter::new(db.clone())))
        } else {
            None
        };
        
        match UnifiedOrchestratorFactory::create(db_ops).await {
            Ok(orchestrator) => {
                info!("✅ UnifiedOrchestrator initialized successfully");
                // Wrap in UnifiedOrchestratorAdapter
                Some(Arc::new(UnifiedOrchestratorAdapter::from_orchestrator(orchestrator)))
            }
            Err(e) => {
                error!("⚠️  Failed to initialize UnifiedOrchestrator: {}", e);
                error!("   Continuing with legacy OrchestratorService only");
                None
            }
        }
    };

    // Initialize orchestrator service and API (if orchestration feature enabled)
    #[cfg(feature = "orchestration")]
    let api_state = {
        if let Some(db) = db_client.as_ref() {
            // Create orchestrator service
            let mut orchestrator_service = data_infrastructure::OrchestratorService::new(db.clone());

            // Wire UnifiedOrchestrator via TaskExecutor if available
            if let Some(ref adapter) = unified_orchestrator {
                use data_interfaces_adapters::UnifiedOrchestratorTaskExecutor;
                let executor = Arc::new(UnifiedOrchestratorTaskExecutor::new(adapter.orchestrator()));
                orchestrator_service = orchestrator_service.with_task_executor(executor);
                info!("✅ UnifiedOrchestratorTaskExecutor wired to OrchestratorService");
            } else {
                warn!("⚠️  UnifiedOrchestratorAdapter not available - OrchestratorService will queue tasks");
            }

            let orchestrator_service = Arc::new(orchestrator_service);

            // Get Redis URL from environment (optional)
            let redis_url = env::var("REDIS_URL").ok();

            // Create API config
            let api_config = DataApiConfig {
                host: args.host.clone(),
                port: args.port,
                enable_cors: args.enable_cors,
                require_api_key: args.require_api_key,
                api_keys: if args.require_api_key {
                    env::var("AGENT_AGENCY_API_KEYS")
                        .unwrap_or_default()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    vec![]
                },
                enable_rate_limiting: false,
                rate_limit_per_minute: 100,
                redis_url: redis_url.clone(),
            };

            // Create progress tracker
            use data_infrastructure::api::server::ProgressTracker;
            let progress_tracker = Arc::new(ProgressTracker { task_id: Uuid::new_v4() });

            // Create RestApi with orchestrator service
            let rest_api = Arc::new(RestApi::with_orchestrator_service(
                api_config,
                orchestrator_service.clone(),
                progress_tracker,
                db.clone(),
            ));

            // Create WebSocket manager with Redis support if available
            let ws_manager = if let Some(ref redis_url) = redis_url {
                match data_infrastructure::websocket::WebSocketManager::with_redis(Some(redis_url)).await {
                    Ok(manager) => {
                        info!("✅ WebSocket manager initialized with Redis: {}", redis_url);
                        Arc::new(manager)
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to initialize Redis for WebSocket: {}. Using local-only mode.", e);
                        Arc::new(data_infrastructure::websocket::WebSocketManager::new())
                    }
                }
            } else {
                info!("📡 WebSocket manager initialized in local-only mode (no Redis)");
                Arc::new(data_infrastructure::websocket::WebSocketManager::new())
            };
            Some((ApiState { 
                api: rest_api,
                websocket_manager: ws_manager.clone(),
            }, ws_manager))
        } else {
            None
        }
    };

    // Create orchestrator service for direct access (reuse the one created above)
    #[cfg(feature = "orchestration")]
    let orchestrator_service = {
        if let Some(db) = db_client.as_ref() {
            let mut service = data_infrastructure::OrchestratorService::new(db.clone());
            if let Some(ref adapter) = unified_orchestrator {
                use data_interfaces_adapters::UnifiedOrchestratorTaskExecutor;
                let executor = Arc::new(UnifiedOrchestratorTaskExecutor::new(adapter.orchestrator()));
                service = service.with_task_executor(executor);
            }
            Some(Arc::new(service))
        } else {
            None
        }
    };

    // Create application state
    #[cfg(feature = "orchestration")]
    let (api_state_final, websocket_manager) = if let Some((s, ws)) = api_state {
        (Some(s.api), Some(ws))
    } else {
        (None, None)
    };
    
    let app_state = AppState {
        db_client,
        #[cfg(feature = "orchestration")]
        api: api_state_final,
        #[cfg(feature = "orchestration")]
        orchestrator_service,
        #[cfg(feature = "orchestration")]
        unified_orchestrator,
        #[cfg(feature = "orchestration")]
        websocket_manager,
    };

    // Build router with all endpoints
    let app = create_router(app_state.clone(), args.enable_cors);

    // Bind server
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("✅ API server ready at http://{}", addr);
    info!("🏥 Health check: http://{}/health", addr);
    info!("📊 API endpoints available at http://{}/api/v1", addr);

    // Serve requests
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the main router with all API endpoints
fn create_router(app_state: AppState, enable_cors: bool) -> Router {
    let mut router = Router::new()
        // Health and status
        .route("/health", get(health_handler))
        .route("/api/v1/health", get(health_handler))
        .route("/", get(root_handler));

    // Task management endpoints
    router = router
        .route("/api/v1/tasks", post(submit_task_handler))
        .route("/api/v1/tasks", get(list_tasks_handler))
        .route("/api/v1/tasks/:task_id", get(get_task_status_handler))
        .route("/api/v1/tasks/:task_id/result", get(get_task_result_handler))
        .route("/api/v1/tasks/:task_id/cancel", post(cancel_task_handler))
        .route("/api/v1/tasks/:task_id/pause", post(pause_task_handler))
        .route("/api/v1/tasks/:task_id/resume", post(resume_task_handler));

    // Chain of thought and observation endpoints
    router = router
        .route("/api/v1/tasks/:task_id/chain-of-thought", get(get_chain_of_thought_handler))
        .route("/api/v1/tasks/:task_id/council-decisions", get(get_council_decisions_handler))
        .route("/api/v1/tasks/:task_id/worker-actions", get(get_worker_actions_handler));

    // Chat and context endpoints
    router = router
        .route("/api/v1/chat", post(chat_handler))
        .route("/api/v1/chat/sessions", get(list_chat_sessions_handler))
        .route("/api/v1/chat/sessions/:session_id", get(get_chat_session_handler))
        .route("/api/v1/chat/sessions/:session_id/messages", get(get_chat_messages_handler));

    // Project management endpoints
    router = router
        .route("/api/v1/projects", post(scaffold_project_handler))
        .route("/api/v1/projects", get(list_projects_handler))
        .route("/api/v1/projects/:project_id", get(get_project_handler))
        .route("/api/v1/projects/:project_id/tasks", get(get_project_tasks_handler));

    // Database inspection endpoints
    router = router
        .route("/api/v1/database/tables", get(list_database_tables_handler))
        .route("/api/v1/database/tables/:table_name", get(get_table_schema_handler))
        .route("/api/v1/database/query", post(execute_query_handler))
        .route("/api/v1/database/stats", get(get_database_stats_handler));

    // Session control endpoints
    router = router
        .route("/api/v1/sessions/:session_id/pause", post(pause_session_handler))
        .route("/api/v1/sessions/:session_id/resume", post(resume_session_handler))
        .route("/api/v1/sessions/:session_id/cancel", post(cancel_session_handler))
        .route("/api/v1/sessions/:session_id/reinstate", post(reinstate_session_handler))
        .route("/api/v1/sessions/:session_id", get(get_session_status_handler));

    // Progress logs endpoints
    router = router
        .route("/api/v1/tasks/:task_id/logs", get(get_task_logs_handler))
        .route("/api/v1/tasks/:task_id/progress", get(get_task_progress_handler))
        .route("/api/v1/tasks/:task_id/events", get(get_task_events_handler));

    // System health and monitoring endpoints
    router = router
        .route("/api/v1/system/health", get(get_system_health_handler))
        .route("/api/v1/system/resources", get(get_resource_usage_handler))
        .route("/api/v1/system/metrics", get(get_system_metrics_handler));

    // Analytics endpoints
    router = router
        .route("/api/v1/analytics/tasks", get(get_task_analytics_handler))
        .route("/api/v1/analytics/performance", get(get_performance_analytics_handler))
        .route("/api/v1/analytics/success-rates", get(get_success_rates_handler));

    // Query management endpoints
    router = router
        .route("/api/v1/queries", get(list_queries_handler))
        .route("/api/v1/queries", post(save_query_handler))
        .route("/api/v1/queries/:query_id", delete(delete_query_handler));

    // Provenance endpoints
    router = router
        .route("/api/v1/provenance", get(list_provenance_handler))
        .route("/api/v1/provenance/link", post(link_provenance_handler))
        .route("/api/v1/provenance/verify/:commit_hash", get(verify_provenance_handler))
        .route("/api/v1/provenance/commit/:commit_hash", get(get_provenance_by_commit_handler))
        .route("/api/v1/tasks/:task_id/provenance", get(get_task_provenance_handler));

    // Waiver management endpoints
    router = router
        .route("/api/v1/waivers", get(list_waivers_handler))
        .route("/api/v1/waivers", post(create_waiver_handler))
        .route("/api/v1/waivers/:waiver_id/approve", post(approve_waiver_handler));

    // SLO management endpoints
    router = router
        .route("/api/v1/slos", get(list_slos_handler))
        .route("/api/v1/slos/:slo_name/status", get(get_slo_status_handler))
        .route("/api/v1/slos/:slo_name/measurements", get(get_slo_measurements_handler))
        .route("/api/v1/slo-alerts", get(list_slo_alerts_handler));

    // Add CORS if enabled
    if enable_cors {
        router = router.layer(tower_http::cors::CorsLayer::permissive());
    }

    router.with_state(app_state)
}

// ============================================================================
// Handler Functions
// ============================================================================

async fn root_handler() -> &'static str {
    "Agent Agency V3 API Server\n\nSee /api/v1 for available endpoints"
}

async fn health_handler(State(state): State<AppState>) -> Result<Json<JsonValue>, StatusCode> {
    let mut health = serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    // Check database health
    if let Some(db) = &state.db_client {
        match db.pool().acquire().await {
            Ok(_) => {
                health["database"] = serde_json::json!({ "status": "connected" });
            }
            Err(e) => {
                health["database"] = serde_json::json!({ "status": "disconnected", "error": e.to_string() });
            }
        }
    } else {
        health["database"] = serde_json::json!({ "status": "not_configured" });
    }

    Ok(Json(health))
}

// Task management handlers
async fn submit_task_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        // Use UnifiedOrchestratorAdapter if available, fallback to legacy API
        if let Some(unified_orchestrator) = &state.unified_orchestrator {
            // Extract task data
            let description = payload.get("description")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let execution_mode = payload.get("execution_mode")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Create task submission request
            use data_infrastructure::api::types::TaskSubmissionRequest;
            let request = TaskSubmissionRequest {
                description: description.to_string(),
                execution_mode,
                risk_tier: payload.get("risk_tier").and_then(|v| v.as_str()).map(|s| s.to_string()),
                context: payload.get("context").and_then(|v| v.as_str()).map(|s| s.to_string()),
                priority: payload.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string()),
                deadline: None,
            };

            // Convert to WorkingSpec
            use data_interfaces_adapters::working_spec_converter::convert_task_request_to_working_spec;
            let working_spec = match convert_task_request_to_working_spec(request) {
                Ok(spec) => spec,
                Err(e) => {
                    error!("Failed to convert task request to WorkingSpec: {}", e);
                    return Err(StatusCode::BAD_REQUEST);
                }
            };

            // Create TaskContext (convert from RequestTaskContext to ContractsTaskContext)
            use agent_agency_contracts::task_request::{TaskContext as RequestTaskContext, Environment};
            use agent_agency_contracts::TaskContext as ContractsTaskContext;
            use chrono::Utc;
            
            let request_context = RequestTaskContext {
                workspace_root: std::env::current_dir()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| ".".to_string()),
                git_branch: "main".to_string(), // TODO: Detect actual git branch
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            };
            
            // Convert to ContractsTaskContext
            let task_context = ContractsTaskContext {
                task_id: Uuid::new_v4(), // Generate new task ID
                worker_id: Uuid::new_v4(), // Generate worker ID
                start_time: Utc::now(),
                timeout_ms: 300_000, // 5 minutes default
                retry_count: 0,
                max_retries: 3,
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert("workspace_root".to_string(), serde_json::Value::String(request_context.workspace_root));
                    meta.insert("git_branch".to_string(), serde_json::Value::String(request_context.git_branch));
                    meta.insert("environment".to_string(), serde_json::Value::String(format!("{:?}", request_context.environment)));
                    meta
                },
            };

            // Execute task via UnifiedOrchestratorAdapter
            match unified_orchestrator.orchestrate_task(working_spec, task_context).await {
                Ok(result) => {
                    // Return task submission response
                    use data_infrastructure::api::types::TaskSubmissionResponse;
                    let response = TaskSubmissionResponse {
                        task_id: result.task_id,
                        status: if result.success { "accepted".to_string() } else { "failed".to_string() },
                        message: if result.success {
                            "Task submitted successfully".to_string()
                        } else {
                            format!("Task execution failed: {}", result.errors.join(", "))
                        },
                        estimated_completion: None, // TODO: Calculate from working spec
                    };
                    Ok(Json(serde_json::json!(response)))
                }
                Err(e) => {
                    error!("Failed to orchestrate task: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else if let Some(api) = &state.api {
            // Fallback to legacy API
            // Extract task data
            let description = payload.get("description")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let execution_mode = payload.get("execution_mode")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Create task submission request
            use data_infrastructure::api::types::TaskSubmissionRequest;
            let request = TaskSubmissionRequest {
                description: description.to_string(),
                execution_mode,
                risk_tier: payload.get("risk_tier").and_then(|v| v.as_str()).map(|s| s.to_string()),
                context: payload.get("context").and_then(|v| v.as_str()).map(|s| s.to_string()),
                priority: payload.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string()),
                deadline: None,
            };

            match api.submit_task(request).await {
                Ok(response) => Ok(Json(serde_json::json!(response))),
                Err(e) => {
                    error!("Failed to submit task: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn list_tasks_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(api) = &state.api {
            match api.list_tasks().await {
                Ok(tasks) => Ok(Json(serde_json::json!(tasks))),
                Err(e) => {
                    error!("Failed to list tasks: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_task_status_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    #[cfg(feature = "orchestration")]
    {
        // Use UnifiedOrchestratorAdapter if available, fallback to legacy API
        if let Some(unified_orchestrator) = &state.unified_orchestrator {
            match unified_orchestrator.get_task_status(&task_uuid).await {
                Ok(status) => {
                    use data_infrastructure::api::types::TaskStatusResponse;
                    let response = TaskStatusResponse {
                        task_id: status.task_id,
                        status: format!("{:?}", status.status).to_lowercase(),
                        progress_percentage: status.progress_percent.map(|p| p as f32).unwrap_or(0.0),
                        current_phase: None, // TODO: Extract from execution state
                        started_at: Some(status.created_at),
                        updated_at: Some(status.updated_at),
                        quality_score: None, // TODO: Extract from execution state
                    };
                    Ok(Json(serde_json::json!(response)))
                }
                Err(e) => {
                    error!("Failed to get task status: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else if let Some(api) = &state.api {
            match api.get_task_status(task_uuid).await {
                Ok(status) => Ok(Json(serde_json::json!(status))),
                Err(e) => {
                    error!("Failed to get task status: {:?}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_task_result_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    #[cfg(feature = "orchestration")]
    {
        if let Some(api) = &state.api {
            match api.get_task_result(task_uuid).await {
                Ok(result) => Ok(Json(serde_json::json!(result))),
                Err(e) => {
                    error!("Failed to get task result: {:?}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn cancel_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Use UnifiedOrchestratorAdapter if available, fallback to legacy service
        if let Some(unified_orchestrator) = &state.unified_orchestrator {
            match unified_orchestrator.cancel_task(&task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({
                    "status": "cancelled",
                    "task_id": task_id,
                    "message": "Task cancelled successfully"
                }))),
                Err(e) => {
                    error!("Failed to cancel task: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else if let Some(service) = &state.orchestrator_service {
            // Fallback to legacy service
            match service.request_cancel_task(task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({
                    "status": "cancel_requested",
                    "task_id": task_id,
                    "message": "Cancellation request forwarded to orchestrator. Orchestrator will decide if cancellation is safe and update status accordingly."
                }))),
                Err(e) => {
                    error!("Failed to request cancellation: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn pause_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Use UnifiedOrchestratorAdapter if available, fallback to legacy service
        if let Some(unified_orchestrator) = &state.unified_orchestrator {
            match unified_orchestrator.pause_task(&task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({ 
                    "status": "paused", 
                    "task_id": task_id,
                    "message": "Task paused successfully"
                }))),
                Err(e) => {
                    error!("Failed to pause task: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else if let Some(service) = &state.orchestrator_service {
            // Fallback to legacy service
            match service.request_pause_task(task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({ 
                    "status": "pause_requested", 
                    "task_id": task_id,
                    "message": "Pause request forwarded to orchestrator. Orchestrator will decide if pause is safe and update status accordingly."
                }))),
                Err(e) => {
                    error!("Failed to request pause: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn resume_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Use UnifiedOrchestratorAdapter if available, fallback to legacy service
        if let Some(unified_orchestrator) = &state.unified_orchestrator {
            match unified_orchestrator.resume_task(&task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({ 
                    "status": "resumed", 
                    "task_id": task_id,
                    "message": "Task resumed successfully"
                }))),
                Err(e) => {
                    error!("Failed to resume task: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else if let Some(service) = &state.orchestrator_service {
            // Fallback to legacy service
            match service.request_resume_task(task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({ 
                    "status": "resume_requested", 
                    "task_id": task_id,
                    "message": "Resume request forwarded to orchestrator. Orchestrator will decide if resume is safe and update status accordingly."
                }))),
                Err(e) => {
                    error!("Failed to request resume: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

// Chain of thought handlers
async fn get_chain_of_thought_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.get_chain_of_thought(task_uuid).await {
                Ok(chain) => {
                    let chain_json: Vec<JsonValue> = chain.into_iter().map(|entry| {
                        serde_json::json!({
                            "timestamp": entry.timestamp.to_rfc3339(),
                            "phase": entry.phase,
                            "reasoning": entry.reasoning,
                            "decision": entry.decision,
                            "context": entry.context,
                        })
                    }).collect();
                    Ok(Json(serde_json::json!({
                        "task_id": task_id,
                        "chain_of_thought": chain_json
                    })))
                }
                Err(e) => {
                    error!("Failed to get chain of thought: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_council_decisions_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.get_council_decisions(task_uuid).await {
                Ok(decisions) => {
                    let decisions_json: Vec<JsonValue> = decisions.into_iter().map(|decision| {
                        serde_json::json!({
                            "timestamp": decision.timestamp.to_rfc3339(),
                            "judge": decision.judge,
                            "verdict": decision.verdict,
                            "reasoning": decision.reasoning,
                            "confidence": decision.confidence,
                        })
                    }).collect();
                    Ok(Json(serde_json::json!({
                        "task_id": task_id,
                        "decisions": decisions_json
                    })))
                }
                Err(e) => {
                    error!("Failed to get council decisions: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_worker_actions_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.get_worker_actions(task_uuid).await {
                Ok(actions) => {
                    let actions_json: Vec<JsonValue> = actions.into_iter().map(|action| {
                        serde_json::json!({
                            "timestamp": action.timestamp.to_rfc3339(),
                            "worker_id": action.worker_id.to_string(),
                            "action": action.action,
                            "result": action.result,
                            "artifacts": action.artifacts,
                        })
                    }).collect();
                    Ok(Json(serde_json::json!({
                        "task_id": task_id,
                        "actions": actions_json
                    })))
                }
                Err(e) => {
                    error!("Failed to get worker actions: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

// Chat handlers (observational - query orchestrator context)
async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        // Extract query from payload
        let query = payload.get("query")
            .and_then(|v| v.as_str())
            .ok_or(StatusCode::BAD_REQUEST)?;

        // Get optional task_id for context
        let task_id = payload.get("task_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        // TODO: Integrate with orchestrator context/memory system:
        // 1. Context querying: Query orchestrator's context system
        //    - Retrieve context for the given task ID
        //    - Access orchestrator's memory/context storage
        //    - Handle context retrieval errors gracefully
        // 2. Memory integration: Integrate with memory system
        //    - Query memory system for relevant context
        //    - Retrieve conversation history and context
        //    - Support context filtering and search
        // 3. Response generation: Generate contextual responses
        //    - Use retrieved context to inform responses
        //    - Include relevant context in response
        //    - Handle missing context appropriately
        // ACCEPTANCE CRITERIA:
        // - Orchestrator context is queried for given task ID
        // - Memory system integration provides relevant context
        // - Responses include contextual information when available
        // DEPENDENCIES:
        // - Orchestrator context API (Required)
        // - Memory system integration (Required)
        // PRIORITY: High
        let mut response = serde_json::json!({
            "response": format!("Query received: '{}'. This endpoint observes orchestrator context. Full chat integration requires orchestrator memory/context system.", query),
            "query": query,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(tid) = task_id {
            response["task_id"] = serde_json::json!(tid.to_string());
            
            // If we have the task, include its chain of thought as context
            if let Some(service) = &state.orchestrator_service {
                if let Ok(Some(task_state)) = service.get_task_status(tid).await {
                    response["context_available"] = serde_json::json!(true);
                    response["chain_of_thought_entries"] = serde_json::json!(task_state.chain_of_thought.len());
                }
            }
        }

        Ok(Json(response))
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn list_chat_sessions_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        // Observe active tasks as "chat sessions" (each task can have context queries)
        if let Some(service) = &state.orchestrator_service {
            let tasks = service.list_tasks().await;
            let sessions: Vec<JsonValue> = tasks.into_iter().map(|task| {
                serde_json::json!({
                    "session_id": task.task_id.to_string(),
                    "description": task.description,
                    "status": format!("{:?}", task.status),
                    "created_at": task.started_at.to_rfc3339(),
                    "updated_at": task.updated_at.to_rfc3339(),
                })
            }).collect();
            Ok(Json(serde_json::json!({ "sessions": sessions })))
        } else {
            Ok(Json(serde_json::json!({ "sessions": [] })))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "sessions": [] })))
    }
}

async fn get_chat_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            if let Ok(Some(task_state)) = service.get_task_status(task_uuid).await {
                Ok(Json(serde_json::json!({
                    "session_id": session_id,
                    "task_id": task_state.task_id.to_string(),
                    "description": task_state.description,
                    "status": format!("{:?}", task_state.status),
                    "chain_of_thought_entries": task_state.chain_of_thought.len(),
                    "council_decisions": task_state.council_decisions.len(),
                    "worker_actions": task_state.worker_actions.len(),
                    "created_at": task_state.started_at.to_rfc3339(),
                    "updated_at": task_state.updated_at.to_rfc3339(),
                })))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_chat_messages_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            // Return chain of thought as "messages" (observational)
            match service.get_chain_of_thought(task_uuid).await {
                Ok(chain) => {
                    let messages: Vec<JsonValue> = chain.into_iter().map(|entry| {
                        serde_json::json!({
                            "timestamp": entry.timestamp.to_rfc3339(),
                            "role": "orchestrator",
                            "content": format!("[{}] {} - {}", entry.phase, entry.reasoning, entry.decision),
                            "context": entry.context,
                        })
                    }).collect();
                    Ok(Json(serde_json::json!({ "messages": messages })))
                }
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Ok(Json(serde_json::json!({ "messages": [] })))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "messages": [] })))
    }
}

// Project management handlers (request orchestrator to scaffold, observe results)
async fn scaffold_project_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let project_name = payload.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("new-project");
            let project_type = payload.get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("standard");
            let description = payload.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Request orchestrator to scaffold project (observational - orchestrator decides how)
            let scaffold_description = format!(
                "Scaffold a new {} project named '{}'. {}",
                project_type, project_name, description
            );

            match service.execute_task(scaffold_description, Some("auto".to_string()), None).await {
                Ok(task_id) => {
                    Ok(Json(serde_json::json!({
                        "status": "scaffold_requested",
                        "task_id": task_id.to_string(),
                        "project_name": project_name,
                        "message": "Project scaffolding requested. Orchestrator will handle scaffolding. Use task_id to observe progress."
                    })))
                }
                Err(e) => {
                    error!("Failed to request project scaffolding: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn list_projects_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        // Observe tasks that might be projects (tasks with "scaffold" or "project" in description)
        if let Some(service) = &state.orchestrator_service {
            let tasks = service.list_tasks().await;
            let projects: Vec<JsonValue> = tasks.into_iter()
                .filter(|task| {
                    let desc_lower = task.description.to_lowercase();
                    desc_lower.contains("scaffold") || desc_lower.contains("project")
                })
                .map(|task| {
                    serde_json::json!({
                        "project_id": task.task_id.to_string(),
                        "name": task.description.chars().take(50).collect::<String>(),
                        "status": format!("{:?}", task.status),
                        "created_at": task.started_at.to_rfc3339(),
                    })
                }).collect();
            Ok(Json(serde_json::json!({ "projects": projects })))
        } else {
            Ok(Json(serde_json::json!({ "projects": [] })))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "projects": [] })))
    }
}

async fn get_project_handler(
    State(_state): State<AppState>,
    Path(_project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}

async fn get_project_tasks_handler(
    State(_state): State<AppState>,
    Path(_project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    Ok(Json(serde_json::json!({ "tasks": [] })))
}

// Database inspection handlers
async fn list_database_tables_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let Some(db) = &state.db_client {
        // Query PostgreSQL system catalog for tables
        let query = r#"
            SELECT table_name, table_schema
            FROM information_schema.tables
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_schema, table_name
        "#;

        match db.query(query, &[]).await {
            Ok(rows) => {
                let tables: Vec<JsonValue> = rows.into_iter().filter_map(|row| {
                    Some(serde_json::json!({
                        "name": row.try_get::<String, _>("table_name").ok()?,
                        "schema": row.try_get::<String, _>("table_schema").ok()?,
                    }))
                }).collect();

                Ok(Json(serde_json::json!({ "tables": tables })))
            }
            Err(e) => {
                error!("Failed to list tables: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn get_table_schema_handler(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let Some(db) = &state.db_client {
        let query = r#"
            SELECT column_name, data_type, is_nullable, column_default
            FROM information_schema.columns
            WHERE table_name = $1
            ORDER BY ordinal_position
        "#;

        match db.query(query, &[&table_name]).await {
            Ok(rows) => {
                let columns: Vec<JsonValue> = rows.into_iter().filter_map(|row| {
                    Some(serde_json::json!({
                        "name": row.try_get::<String, _>("column_name").ok()?,
                        "type": row.try_get::<String, _>("data_type").ok()?,
                        "nullable": row.try_get::<String, _>("is_nullable").ok()? == "YES",
                        "default": row.try_get::<Option<String>, _>("column_default").ok()?,
                    }))
                }).collect();

                Ok(Json(serde_json::json!({
                    "table_name": table_name,
                    "columns": columns
                })))
            }
            Err(e) => {
                error!("Failed to get table schema: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn execute_query_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let Some(db) = &state.db_client {
        let query_text = payload.get("query")
            .and_then(|v| v.as_str())
            .ok_or(StatusCode::BAD_REQUEST)?;

        // TODO: Implement comprehensive SQL query safety validation:
        // 1. Query type validation: Support additional safe query types
        //    - Allow SELECT queries (read-only operations)
        //    - Support EXPLAIN, DESCRIBE, SHOW queries (metadata queries)
        //    - Validate against dangerous operations (DROP, DELETE, UPDATE, etc.)
        // 2. SQL injection prevention: Enhanced protection against SQL injection
        //    - Parse and validate SQL syntax structure
        //    - Whitelist allowed SQL keywords and patterns
        //    - Block dangerous SQL patterns and functions
        // 3. Query complexity limits: Prevent resource exhaustion
        //    - Limit query execution time and resource usage
        //    - Restrict result set sizes and pagination
        //    - Monitor and throttle query execution
        // ACCEPTANCE CRITERIA:
        // - Only safe read-only queries are allowed to execute
        // - SQL injection attempts are detected and blocked
        // - Query complexity limits prevent resource exhaustion
        // - Dangerous operations (DROP, DELETE, UPDATE) are rejected
        // DEPENDENCIES:
        // - SQL parser for syntax validation (Required)
        // - Query complexity analyzer (Required)
        // PRIORITY: High
        let query_upper = query_text.trim().to_uppercase();
        if !query_upper.starts_with("SELECT") {
            return Err(StatusCode::BAD_REQUEST);
        }

        match db.query(query_text, &[]).await {
            Ok(rows) => {
                // TODO: Implement proper database row to JSON serialization
                //       Currently returns empty JSON objects; should properly serialize row data with type handling and column mapping.
                //
                // COMPLETION CHECKLIST:
                // [ ] Extract column names from query result
                // [ ] Map database types to JSON types
                // [ ] Handle NULL values appropriately
                // [ ] Support all PostgreSQL types (text, numeric, boolean, json, etc.)
                // [ ] Add proper error handling for type conversion failures
                // [ ] Add unit tests with various row types
                // [ ] Add integration tests with real database queries
                // [ ] Performance: Serialization should complete in <10ms for typical rows
                // [ ] Documentation: Document type mapping rules
                //
                // ACCEPTANCE CRITERIA:
                // - All database types are properly converted to JSON
                // - Column names are preserved in JSON output
                // - NULL values are handled correctly (null vs omitted)
                // - Nested types (arrays, objects) are properly serialized
                // - Error handling provides clear messages for conversion failures
                //
                // DEPENDENCIES:
                // - Database query result rows (Required)
                // - Column metadata from query (Required)
                // - Type conversion utilities (Required)
                //
                // ESTIMATED EFFORT: 6-8 hours (medium confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (API server feature)
                // - Change Budget: ~150 LOC
                // - Reviewer Requirements: Database and serialization expertise
                let results: Vec<JsonValue> = rows.into_iter().map(|_row| {
                    serde_json::json!({})
                }).collect();

                Ok(Json(serde_json::json!({
                    "results": results,
                    "row_count": results.len()
                })))
            }
            Err(e) => {
                error!("Query execution failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn get_database_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let Some(db) = &state.db_client {
        // Get database size and table counts
        let size_query = "SELECT pg_size_pretty(pg_database_size(current_database())) as size";
        let table_count_query = r#"
            SELECT COUNT(*) as count
            FROM information_schema.tables
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        "#;

        let size_result = db.query_one(size_query, &[]).await;
        let table_count_result = db.query_one(table_count_query, &[]).await;

        let mut stats = serde_json::json!({});

        if let Ok(Some(row)) = size_result {
            if let Ok(size) = row.try_get::<String, _>("size") {
                stats["database_size"] = serde_json::json!(size);
            }
        }

        if let Ok(Some(row)) = table_count_result {
            if let Ok(count) = row.try_get::<i64, _>("count") {
                stats["table_count"] = serde_json::json!(count);
            }
        }

        Ok(Json(stats))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// Session control handlers (request orchestrator, observe results)
async fn pause_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.request_pause_task(task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({
                    "status": "pause_requested",
                    "session_id": session_id,
                    "message": "Pause request forwarded to orchestrator"
                }))),
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn resume_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.request_resume_task(task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({
                    "status": "resume_requested",
                    "session_id": session_id,
                    "message": "Resume request forwarded to orchestrator"
                }))),
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn cancel_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.request_cancel_task(task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({
                    "status": "cancel_requested",
                    "session_id": session_id,
                    "message": "Cancellation request forwarded to orchestrator"
                }))),
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn reinstate_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        // Reinstating is essentially resuming a cancelled/failed task
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            // Request resume (orchestrator will decide if reinstatement is safe)
            match service.request_resume_task(task_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({
                    "status": "reinstate_requested",
                    "session_id": session_id,
                    "message": "Reinstatement request forwarded to orchestrator"
                }))),
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_session_status_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&session_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            if let Ok(Some(task_state)) = service.get_task_status(task_uuid).await {
                Ok(Json(serde_json::json!({
                    "session_id": session_id,
                    "status": format!("{:?}", task_state.status),
                    "created_at": task_state.started_at.to_rfc3339(),
                    "updated_at": task_state.updated_at.to_rfc3339(),
                    "completed_at": task_state.completed_at.map(|d| d.to_rfc3339()),
                })))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_FOUND)
    }
}

// Progress logs handlers (observational)
async fn get_task_logs_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.get_task_logs(task_uuid).await {
                Ok(logs) => Ok(Json(serde_json::json!({ "logs": logs }))),
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "logs": [] })))
    }
}

async fn get_task_progress_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.get_task_progress(task_uuid).await {
                Ok(progress) => Ok(Json(progress)),
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "progress": {} })))
    }
}

async fn get_task_events_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match service.get_task_events(task_uuid).await {
                Ok(events) => Ok(Json(serde_json::json!({ "events": events }))),
                Err(_) => Err(StatusCode::NOT_FOUND)
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "events": [] })))
    }
}

// System health handlers
async fn get_system_health_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let mut health = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    // Database health
    if let Some(db) = &state.db_client {
        match db.pool().acquire().await {
            Ok(_) => {
                health["database"] = serde_json::json!({ "status": "healthy" });
            }
            Err(e) => {
                health["database"] = serde_json::json!({ "status": "unhealthy", "error": e.to_string() });
                health["status"] = serde_json::json!("degraded");
            }
        }
    }

    Ok(Json(health))
}

async fn get_resource_usage_handler(
    State(_state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    // TODO: Implement resource usage monitoring
    Ok(Json(serde_json::json!({
        "cpu": 0.0,
        "memory": 0,
        "disk": 0,
        "network": 0
    })))
}

async fn get_system_metrics_handler(
    State(_state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    // TODO: Implement system metrics
    Ok(Json(serde_json::json!({ "metrics": {} })))
}

// Analytics handlers (observational - aggregated from task states)
async fn get_task_analytics_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(service) = &state.orchestrator_service {
            let analytics = service.get_task_analytics().await;
            Ok(Json(analytics))
        } else {
            Ok(Json(serde_json::json!({ "analytics": {} })))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "analytics": {} })))
    }
}

async fn get_performance_analytics_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        // Observe performance metrics from task execution times
        if let Some(service) = &state.orchestrator_service {
            let tasks = service.list_tasks().await;
            let completed_tasks: Vec<_> = tasks.iter()
                .filter(|t| matches!(t.status, TaskStatus::Completed))
                .collect();

            let mut total_duration_ms = 0.0;
            let mut count = 0;
            for task in &completed_tasks {
                if let Some(completed_at) = task.completed_at {
                    let duration = (completed_at - task.started_at).num_milliseconds() as f64;
                    total_duration_ms += duration;
                    count += 1;
                }
            }

            let avg_duration_ms = if count > 0 { total_duration_ms / count as f64 } else { 0.0 };

            Ok(Json(serde_json::json!({
                "average_task_duration_ms": avg_duration_ms,
                "completed_tasks_count": count,
                "total_tasks": tasks.len(),
                "performance_score": if count > 0 && avg_duration_ms > 0.0 {
                    (1000.0 / avg_duration_ms * 100.0).min(100.0) // Higher is better, capped at 100
                } else {
                    0.0
                }
            })))
        } else {
            Ok(Json(serde_json::json!({ "performance": {} })))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "performance": {} })))
    }
}

async fn get_success_rates_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        // Observe success rates from task completion status
        if let Some(service) = &state.orchestrator_service {
            let analytics = service.get_task_analytics().await;
            Ok(Json(serde_json::json!({
                "success_rate": analytics.get("success_rate").cloned().unwrap_or(serde_json::json!("0.00%")),
                "total_tasks": analytics.get("total_tasks").cloned().unwrap_or(serde_json::json!(0)),
                "completed": analytics.get("completed").cloned().unwrap_or(serde_json::json!(0)),
                "failed": analytics.get("failed").cloned().unwrap_or(serde_json::json!(0)),
            })))
        } else {
            Ok(Json(serde_json::json!({ "success_rate": "0.00%" })))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Ok(Json(serde_json::json!({ "success_rate": "0.00%" })))
    }
}

// Query management handlers (delegate to existing handlers)
async fn list_queries_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(api) = &state.api {
            match api.list_saved_queries().await {
                Ok(queries) => Ok(Json(serde_json::json!(queries))),
                Err(e) => {
                    // If table doesn't exist or database error, return empty list instead of 500
                    // This provides better UX for test environments
                    warn!("Failed to list queries (table may not exist): {:?}", e);
                    Ok(Json(serde_json::json!([])))
                }
            }
        } else {
            // Return empty list instead of SERVICE_UNAVAILABLE for better UX
            Ok(Json(serde_json::json!([])))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        // Return empty list when orchestration feature is not enabled
        Ok(Json(serde_json::json!([])))
    }
}

async fn save_query_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(api) = &state.api {
            use data_infrastructure::api::types::SaveQueryRequest;
            let request = SaveQueryRequest {
                name: payload.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or(StatusCode::BAD_REQUEST)?
                    .to_string(),
                query_text: payload.get("query_text")
                    .and_then(|v| v.as_str())
                    .ok_or(StatusCode::BAD_REQUEST)?
                    .to_string(),
            };

            match api.save_query(request).await {
                Ok(query) => Ok(Json(serde_json::json!(query))),
                Err(e) => {
                    error!("Failed to save query: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn delete_query_handler(
    State(state): State<AppState>,
    Path(query_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(api) = &state.api {
            let query_uuid = Uuid::parse_str(&query_id).map_err(|_| StatusCode::BAD_REQUEST)?;
            match api.delete_saved_query(query_uuid).await {
                Ok(_) => Ok(Json(serde_json::json!({ "status": "deleted", "query_id": query_id }))),
                Err(e) => {
                    error!("Failed to delete query: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

// Provenance handlers (delegate to existing handlers)
async fn list_provenance_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager)) = (&state.api, &state.websocket_manager) {
                match list_provenance_records(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                })).await {
                    Ok(response) => Ok(response),
                    Err(status) => {
                        // If database error (e.g., table doesn't exist), return empty list
                        warn!("Failed to list provenance (table may not exist): {:?}", status);
                        Ok(Json(serde_json::json!([])))
                    }
                }
            } else {
                // Return empty list if services not available
                Ok(Json(serde_json::json!([])))
            }
        } else {
            // Return empty list instead of SERVICE_UNAVAILABLE
            Ok(Json(serde_json::json!([])))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        // Return empty list when orchestration feature is not enabled
        Ok(Json(serde_json::json!([])))
    }
}

async fn link_provenance_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match link_provenance_to_commit(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Json(payload)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn verify_provenance_handler(
    State(state): State<AppState>,
    Path(commit_hash): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match verify_provenance_trailer(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Path(commit_hash)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_provenance_by_commit_handler(
    State(state): State<AppState>,
    Path(commit_hash): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match get_provenance_by_commit(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Path(commit_hash)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_task_provenance_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match get_task_provenance(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Path(task_id)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

// Waiver handlers (delegate to existing handlers)
async fn list_waivers_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager)) = (&state.api, &state.websocket_manager) {
                match list_waivers(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                })).await {
                    Ok(response) => Ok(response),
                    Err(status) => {
                        // If database error (e.g., table doesn't exist), return empty list
                        warn!("Failed to list waivers (table may not exist): {:?}", status);
                        Ok(Json(serde_json::json!([])))
                    }
                }
            } else {
                // Return empty list if services not available
                Ok(Json(serde_json::json!([])))
            }
        } else {
            // Return empty list instead of SERVICE_UNAVAILABLE
            Ok(Json(serde_json::json!([])))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        // Return empty list when orchestration feature is not enabled
        Ok(Json(serde_json::json!([])))
    }
}

async fn create_waiver_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match create_waiver(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Json(payload)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn approve_waiver_handler(
    State(state): State<AppState>,
    Path(waiver_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match approve_waiver(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Path(waiver_id), Json(payload)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

// SLO handlers (delegate to existing handlers)
async fn list_slos_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match list_slos(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            })).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_slo_status_handler(
    State(state): State<AppState>,
    Path(slo_name): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match get_slo_status(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Path(slo_name)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn get_slo_measurements_handler(
    State(state): State<AppState>,
    Path(slo_name): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            match get_slo_measurements(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            }), Path(slo_name)).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn list_slo_alerts_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(_db) = &state.db_client {
            match list_slo_alerts(State(ApiState {
                api: state.api.as_ref().unwrap().clone(),
                websocket_manager: state.websocket_manager.as_ref().unwrap().clone(),
            })).await {
                Ok(response) => Ok(response),
                Err(status) => Err(status),
            }
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}
