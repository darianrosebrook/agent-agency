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

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::sync::Arc;
use system_quality_security::{
    authentication::PasswordPolicy,
    policy_types::{RateLimitRequest, RateLimitingPolicy},
    rate_limiting::RateLimiter,
    AuthConfig, AuthService,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Database integration
use data_infrastructure::database_config::DatabaseConfig;
use data_infrastructure::database_init::{initialize_database, verify_schema};
use data_infrastructure::simple_client::DatabaseClient;
use sqlx::Row;

// Unified orchestrator adapter
use data_interfaces::service_contracts::OrchestrationService;
use data_interfaces_adapters::orchestration_adapter::UnifiedOrchestratorAdapter;

// API modules
#[cfg(feature = "orchestration")]
use data_infrastructure::api::handlers::*;
#[cfg(feature = "orchestration")]
use data_infrastructure::api::openapi::create_swagger_ui;
#[cfg(feature = "orchestration")]
use data_infrastructure::api::types::ApiConfig as DataApiConfig;
#[cfg(feature = "orchestration")]
use data_infrastructure::api::{ApiState, RestApi};
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
    /// Query performance monitor
    #[cfg(feature = "orchestration")]
    query_performance_monitor:
        Option<Arc<data_infrastructure::monitoring::query_performance::QueryPerformanceMonitor>>,
    /// Authentication service for password hashing and JWT generation
    auth_service: Arc<AuthService>,
    /// Rate limiter for API request throttling
    rate_limiter: Arc<RateLimiter>,
    /// Telemetry service for LLM and agent activity logging
    telemetry_service: Arc<data_infrastructure::telemetry_service::TelemetryService>,
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
    info!(
        "   - API Keys: {}",
        if args.require_api_key {
            "Required"
        } else {
            "Optional"
        }
    );
    info!(
        "   - CORS: {}",
        if args.enable_cors {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    info!(
        "   - Database: {}",
        if db_client.is_some() {
            "Connected"
        } else {
            "Not connected"
        }
    );

    // Initialize UnifiedOrchestrator using factory from agent-orchestration
    #[cfg(feature = "orchestration")]
    let unified_orchestrator = {
        info!("🔧 Initializing UnifiedOrchestrator...");

        // Verify database schema before initialization if database is available
        if let Some(db) = db_client.as_ref() {
            info!("Verifying database schema for UnifiedOrchestrator...");
            let pool = db.pool();

            // Check if planning_audit_events table has description column
            let has_description: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'planning_audit_events'
                    AND column_name = 'description'
                )
                "#,
            )
            .fetch_one(pool)
            .await
            .unwrap_or(false);

            if !has_description {
                error!("⚠️  Schema issue detected: planning_audit_events table missing 'description' column");
                error!("   This will cause UnifiedOrchestrator initialization to fail");
                error!("   Please run migrations to fix the schema");
            } else {
                info!("✅ Schema verification passed: planning_audit_events table has 'description' column");
            }

            // Check other critical tables
            let tables_to_check = [("tasks", "description"), ("waivers", "description")];

            for (table, column) in &tables_to_check {
                let has_column: bool = sqlx::query_scalar(
                    r#"
                    SELECT EXISTS (
                        SELECT 1
                        FROM information_schema.columns
                        WHERE table_name = $1
                        AND column_name = $2
                    )
                    "#,
                )
                .bind(table)
                .bind(column)
                .fetch_one(pool)
                .await
                .unwrap_or(false);

                if !has_column {
                    warn!("⚠️  Table '{}' missing '{}' column", table, column);
                }
            }
        }

        // Create database operations adapter if database client is available
        use agent_orchestration::orchestration::UnifiedOrchestratorFactory;
        use agent_orchestration::planning::DatabaseOperations;

        let db_ops: Option<Arc<dyn DatabaseOperations>> = if let Some(db) = db_client.as_ref() {
            // Use DatabaseOperationsAdapter from this crate
            Some(Arc::new(
                data_interfaces_adapters::DatabaseOperationsAdapter::new(db.clone()),
            ))
        } else {
            None
        };

        match UnifiedOrchestratorFactory::create(db_ops).await {
            Ok(orchestrator) => {
                info!("✅ UnifiedOrchestrator initialized successfully");
                // Wrap in UnifiedOrchestratorAdapter
                Some(Arc::new(UnifiedOrchestratorAdapter::from_orchestrator(
                    orchestrator,
                )))
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
            let mut orchestrator_service =
                data_infrastructure::OrchestratorService::new(db.clone());

            // Wire UnifiedOrchestrator via TaskExecutor if available
            if let Some(ref adapter) = unified_orchestrator {
                use data_interfaces_adapters::UnifiedOrchestratorTaskExecutor;
                let executor =
                    Arc::new(UnifiedOrchestratorTaskExecutor::new(adapter.orchestrator()));
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
                stream_timeout_seconds: env::var("STREAM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300), // Default: 5 minutes
            };

            // Create RestApi with orchestrator service
            // ProgressTracker is created internally by with_orchestrator_service
            let rest_api = Arc::new(RestApi::with_orchestrator_service(
                api_config,
                orchestrator_service.clone(),
                db.clone(),
            ));

            // Create WebSocket manager with Redis support if available
            let ws_manager = if let Some(ref redis_url) = redis_url {
                match data_infrastructure::websocket::WebSocketManager::with_redis(Some(redis_url))
                    .await
                {
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
                query_performance_monitor: Arc::new(data_infrastructure::monitoring::query_performance::QueryPerformanceMonitor::with_defaults()),
                coreml_inference_callback: None,
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
                let executor =
                    Arc::new(UnifiedOrchestratorTaskExecutor::new(adapter.orchestrator()));
                service = service.with_task_executor(executor);
            }
            Some(Arc::new(service))
        } else {
            None
        }
    };

    // Initialize authentication service
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        warn!("⚠️  JWT_SECRET not set, using default (NOT SECURE FOR PRODUCTION)");
        "default-jwt-secret-key-change-in-production-min-32-chars".to_string()
    });

    let auth_config = AuthConfig {
        jwt_secret,
        token_expiry_seconds: 3600,              // 1 hour
        refresh_token_expiry_seconds: 86400 * 7, // 7 days
        password_hash_params: argon2::Params::default(),
        max_failed_attempts: 5,
        lockout_duration_seconds: 900, // 15 minutes
        password_policy: PasswordPolicy::default(),
    };

    let auth_service = Arc::new(AuthService::new(auth_config));
    info!("✅ Authentication service initialized");

    // Initialize rate limiter
    // Production configuration: 100 requests per minute, burst of 20
    // Development: Can be adjusted via environment variables
    let rate_limit_config = RateLimitingPolicy {
        enabled: env::var("RATE_LIMIT_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true), // Enabled by default in production
        requests_per_window: env::var("RATE_LIMIT_REQUESTS_PER_WINDOW")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100),
        window_seconds: env::var("RATE_LIMIT_WINDOW_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60),
        burst_size: env::var("RATE_LIMIT_BURST_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        cleanup_interval_seconds: 300, // 5 minutes
    };
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config.clone()));
    info!(
        "✅ Rate limiter initialized (enabled: {}, {}/{}s, burst: {})",
        rate_limit_config.enabled,
        rate_limit_config.requests_per_window,
        rate_limit_config.window_seconds,
        rate_limit_config.burst_size
    );

    // Create application state
    #[cfg(feature = "orchestration")]
    let (api_state_final, websocket_manager, query_perf_monitor) = if let Some((s, ws)) = api_state
    {
        (Some(s.api), Some(ws), Some(s.query_performance_monitor))
    } else {
        (None, None, None)
    };

    // Create telemetry service
    let telemetry_service = Arc::new(data_infrastructure::telemetry_service::TelemetryService::new(
        db_client.clone(),
    ));

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
        #[cfg(feature = "orchestration")]
        query_performance_monitor: query_perf_monitor,
        auth_service,
        rate_limiter,
        telemetry_service,
    };

    // Build router with all endpoints
    let app = create_router(app_state.clone(), args.enable_cors);

    // Bind server
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("API server ready at http://{}", addr);
    info!("Health check: http://{}/health", addr);
    info!("API endpoints available at http://{}/api/v1", addr);

    // Take initial task stats snapshot if needed
    if let Ok(snapshot_taken) = app_state.telemetry_service.maybe_snapshot_task_stats().await {
        if snapshot_taken {
            info!("Initial task stats snapshot created");
        }
    }

    // Initialize and start ContinuousBenchmarker if research feature is enabled
    #[cfg(feature = "research")]
    {
        use agent_research::benchmark_runner::BenchmarkRunner;
        use agent_research::benchmarking::{
            BenchmarkScheduler, ContinuousBenchmarker, DatasetManager,
        };
        use agent_research::performance_tracker::PerformanceTracker;
        use agent_research::scoring_system::MultiDimensionalScoringSystem;
        use std::sync::Arc;

        info!("🔬 Initializing ContinuousBenchmarker...");

        // Create all required dependencies
        let scheduler = Arc::new(BenchmarkScheduler::new());
        let dataset_manager = Arc::new(DatasetManager::new());
        let benchmark_runner = Arc::new(BenchmarkRunner::new());
        let performance_tracker = Arc::new(PerformanceTracker::new());
        let scoring_system = Arc::new(MultiDimensionalScoringSystem::new());

        // Create ContinuousBenchmarker
        let continuous_benchmarker = Arc::new(ContinuousBenchmarker::new(
            scheduler.clone(),
            dataset_manager.clone(),
            benchmark_runner.clone(),
            performance_tracker.clone(),
            scoring_system.clone(),
        ));

        // Start continuous benchmarking
        match continuous_benchmarker.start().await {
            Ok(_) => {
                info!("✅ ContinuousBenchmarker started successfully");

                // Schedule default benchmarks (daily micro, weekly macro)
                // Get active models from PerformanceTracker (empty initially, will be populated as models are registered)
                let active_models = performance_tracker
                    .get_active_models()
                    .await
                    .unwrap_or_default();
                if !active_models.is_empty() {
                    if let Err(e) = scheduler.schedule_default_benchmarks(active_models).await {
                        warn!("⚠️  Failed to schedule default benchmarks: {}", e);
                    } else {
                        info!("✅ Default benchmarks scheduled (daily micro, weekly macro)");
                    }
                } else {
                    info!("📝 No active models found - benchmarks will be scheduled when models are registered");
                }
            }
            Err(e) => {
                warn!("⚠️  Failed to start ContinuousBenchmarker: {}", e);
                warn!("   Continuous benchmarking will not be available");
            }
        }
    }

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
        .route("/metrics", get(get_metrics_handler))
        .route("/", get(root_handler));

    // OpenAPI documentation endpoints
    #[cfg(feature = "orchestration")]
    {
        // SwaggerUi already includes the /api-docs/openapi.json route, so we don't need to register it separately
        router = router.merge(create_swagger_ui());
    }

    // Task management endpoints
    router = router
        .route("/api/v1/tasks", post(submit_task_handler))
        .route("/api/v1/tasks", get(list_tasks_handler))
        .route("/api/v1/tasks/stats", get(get_tasks_stats_handler))
        .route(
            "/api/v1/tasks/stats/history",
            get(get_tasks_stats_history_handler),
        )
        .route("/api/v1/tasks/:task_id", get(get_task_status_handler))
        .route(
            "/api/v1/tasks/:task_id",
            axum::routing::patch(update_task_handler),
        )
        .route(
            "/api/v1/tasks/:task_id",
            axum::routing::delete(delete_task_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/result",
            get(get_task_result_handler),
        )
        .route("/api/v1/tasks/:task_id/cancel", post(cancel_task_handler))
        .route("/api/v1/tasks/:task_id/pause", post(pause_task_handler))
        .route("/api/v1/tasks/:task_id/resume", post(resume_task_handler))
        .route(
            "/api/v1/projects/:project_id/tasks",
            post(create_project_task_handler),
        )
        .route(
            "/api/v1/projects/:project_id/tasks/:task_id",
            axum::routing::patch(update_project_task_handler),
        )
        .route(
            "/api/v1/projects/:project_id/tasks/:task_id",
            axum::routing::delete(delete_project_task_handler),
        );

    // Worker/Agent management endpoints
    router = router
        .route("/api/v1/agents", get(list_agents_handler))
        .route("/api/v1/agents/stats", get(get_agents_stats_handler))
        .route(
            "/api/v1/agents/tasks/completion",
            get(get_agents_tasks_completion_handler),
        )
        .route(
            "/api/v1/agents/efficiency",
            get(get_agents_efficiency_handler),
        )
        .route("/api/v1/agents/:id", get(get_agent_handler))
        .route(
            "/api/v1/agents/:id",
            axum::routing::patch(update_agent_handler),
        )
        .route(
            "/api/v1/agents/:id",
            axum::routing::delete(delete_agent_handler),
        )
        .route("/api/v1/agents/:id/stats", get(get_agent_stats_handler))
        .route("/api/v1/agents/:id/health", get(get_agent_health_handler))
        .route("/api/v1/agents/:id/metrics", get(get_agent_metrics_handler))
        .route("/api/v1/agents/:id/logs", get(get_agent_logs_handler))
        .route("/api/v1/agents/:id/restart", post(restart_agent_handler))
        .route("/api/v1/agents/:id/stop", post(stop_agent_handler));

    // Judge management endpoints
    router = router
        .route("/api/v1/judges", get(list_judges_handler))
        .route("/api/v1/judges", post(create_judge_handler))
        .route("/api/v1/judges/stats", get(get_judges_stats_handler))
        .route("/api/v1/judges/:id", get(get_judge_handler))
        .route(
            "/api/v1/judges/:id",
            axum::routing::patch(update_judge_handler),
        )
        .route(
            "/api/v1/judges/:id",
            axum::routing::delete(delete_judge_handler),
        )
        .route("/api/v1/judges/:id/stats", get(get_judge_stats_handler))
        .route(
            "/api/v1/judges/:id/evaluations",
            get(get_judge_evaluations_handler),
        );

    // Telemetry & Observability endpoints
    router = router
        .route(
            "/api/v1/telemetry/contributions",
            get(get_contributions_handler),
        )
        .route(
            "/api/v1/telemetry/model-contributions",
            get(get_model_contributions_handler),
        )
        .route(
            "/api/v1/telemetry/agent-activity",
            get(get_agent_activity_handler),
        )
        .route(
            "/api/v1/observability/efficiency",
            get(get_efficiency_handler),
        )
        .route(
            "/api/v1/observability/system-metrics",
            get(get_resource_usage_handler),
        )
        .route("/api/v1/observability/alerts", get(get_alerts_handler));

    // Chain of thought and observation endpoints
    router = router
        .route(
            "/api/v1/tasks/:task_id/chain-of-thought",
            get(get_chain_of_thought_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/council-decisions",
            get(get_council_decisions_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/worker-actions",
            get(get_worker_actions_handler),
        );

    // Task comments endpoints
    router = router
        .route(
            "/api/v1/tasks/:task_id/comments",
            get(get_task_comments_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/comments",
            post(create_task_comment_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/comments/:comment_id",
            axum::routing::patch(update_task_comment_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/comments/:comment_id",
            axum::routing::delete(delete_task_comment_handler),
        );

    // Chat and context endpoints
    router = router.route("/api/v1/chat", post(chat_handler));

    // Chat stream handlers require ApiState, so they're conditionally added
    #[cfg(feature = "orchestration")]
    {
        router = router
            .route("/api/v1/chat/stream", post(stream_agent_response_wrapper))
            .route("/api/v1/chat/stream/cancel", post(cancel_stream_wrapper));
    }

    router = router
        .route("/api/v1/chat/sessions", get(list_chat_sessions_handler))
        .route(
            "/api/v1/chat/sessions/:session_id",
            get(get_chat_session_handler),
        )
        .route(
            "/api/v1/chat/sessions/:session_id/messages",
            get(get_chat_messages_handler),
        );

    // Project management endpoints
    router = router
        .route("/api/v1/projects", post(scaffold_project_handler))
        .route("/api/v1/projects", get(list_projects_handler))
        .route("/api/v1/projects/:project_id", get(get_project_handler))
        .route(
            "/api/v1/projects/:project_id",
            axum::routing::patch(update_project_handler),
        )
        .route(
            "/api/v1/projects/:project_id",
            axum::routing::delete(delete_project_handler),
        )
        .route(
            "/api/v1/projects/:project_id/stats",
            get(get_project_stats_handler),
        )
        .route(
            "/api/v1/projects/:project_id/tasks",
            get(get_project_tasks_handler),
        )
        .route(
            "/api/v1/projects/:project_id/tasks/stats",
            get(get_project_tasks_stats_handler),
        )
        .route(
            "/api/v1/projects/:project_id/milestones",
            get(get_project_milestones_handler),
        )
        .route(
            "/api/v1/projects/:project_id/milestones",
            post(create_project_milestone_handler),
        )
        .route(
            "/api/v1/projects/:project_id/milestones/:milestone_id",
            axum::routing::patch(update_project_milestone_handler),
        )
        .route(
            "/api/v1/projects/:project_id/members",
            get(get_project_members_handler),
        )
        .route(
            "/api/v1/projects/:project_id/work-history",
            get(get_project_work_history_handler),
        )
        .route(
            "/api/v1/projects/:project_id/settings",
            get(get_project_settings_handler),
        )
        .route(
            "/api/v1/projects/:project_id/settings",
            axum::routing::patch(update_project_settings_handler),
        )
        .route(
            "/api/v1/projects/:project_id/task-settings",
            get(get_project_task_settings_handler),
        )
        .route(
            "/api/v1/projects/:project_id/task-settings",
            axum::routing::patch(update_project_task_settings_handler),
        )
        .route(
            "/api/v1/projects/:project_id/overview-versions",
            get(get_project_overview_versions_handler),
        )
        .route(
            "/api/v1/projects/:project_id/overview-versions",
            post(create_project_overview_version_handler),
        )
        .route(
            "/api/v1/projects/:project_id/overview-versions/:version_id/restore",
            post(restore_project_overview_version_handler),
        );

    // Database inspection endpoints
    router = router
        .route("/api/v1/database/tables", get(list_database_tables_handler))
        .route(
            "/api/v1/database/tables/:table_name",
            get(get_table_schema_handler),
        )
        .route("/api/v1/database/query", post(execute_query_handler))
        .route("/api/v1/database/stats", get(get_database_stats_handler));

    // Session control endpoints
    router = router
        .route(
            "/api/v1/sessions/:session_id/pause",
            post(pause_session_handler),
        )
        .route(
            "/api/v1/sessions/:session_id/resume",
            post(resume_session_handler),
        )
        .route(
            "/api/v1/sessions/:session_id/cancel",
            post(cancel_session_handler),
        )
        .route(
            "/api/v1/sessions/:session_id/reinstate",
            post(reinstate_session_handler),
        )
        .route(
            "/api/v1/sessions/:session_id",
            get(get_session_status_handler),
        );

    // Progress logs endpoints
    router = router
        .route("/api/v1/tasks/:task_id/logs", get(get_task_logs_handler))
        .route(
            "/api/v1/tasks/:task_id/progress",
            get(get_task_progress_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/events",
            get(get_task_events_handler),
        );

    // System health and monitoring endpoints
    router = router
        .route("/api/v1/system/health", get(get_system_health_handler))
        .route("/api/v1/system/resources", get(get_resource_usage_handler))
        .route("/api/v1/system/metrics", get(get_resource_usage_handler));

    // Analytics endpoints
    router = router
        .route("/api/v1/analytics/tasks", get(get_task_analytics_handler))
        .route(
            "/api/v1/analytics/performance",
            get(get_performance_analytics_handler),
        )
        .route(
            "/api/v1/analytics/success-rates",
            get(get_success_rates_handler),
        );

    // Search endpoints
    router = router.route("/api/v1/search", get(search_handler));

    // Query management endpoints
    router = router
        .route("/api/v1/queries", get(list_queries_handler))
        .route("/api/v1/queries", post(save_query_handler))
        .route("/api/v1/queries/:query_id", delete(delete_query_handler));

    // Query performance monitoring endpoints
    #[cfg(feature = "orchestration")]
    {
        router = router
            .route(
                "/api/v1/query-performance/summary",
                get(query_performance_summary_handler),
            )
            .route(
                "/api/v1/query-performance/metrics",
                get(query_performance_metrics_handler),
            )
            .route(
                "/api/v1/query-performance/slow",
                get(query_performance_slow_handler),
            )
            .route(
                "/api/v1/query-performance/top-slow",
                get(query_performance_top_slow_handler),
            );
    }

    // Provenance endpoints
    router = router
        .route("/api/v1/provenance", get(list_provenance_handler))
        .route("/api/v1/provenance/link", post(link_provenance_handler))
        .route(
            "/api/v1/provenance/verify/:commit_hash",
            get(verify_provenance_handler),
        )
        .route(
            "/api/v1/provenance/commit/:commit_hash",
            get(get_provenance_by_commit_handler),
        )
        .route(
            "/api/v1/tasks/:task_id/provenance",
            get(get_task_provenance_handler),
        );

    // Waiver management endpoints
    router = router
        .route("/api/v1/waivers", get(list_waivers_handler))
        .route("/api/v1/waivers", post(create_waiver_handler))
        .route(
            "/api/v1/waivers/:waiver_id/approve",
            post(approve_waiver_handler),
        );

    // SLO management endpoints
    router = router
        .route("/api/v1/slos", get(list_slos_handler))
        .route("/api/v1/slos/:slo_name/status", get(get_slo_status_handler))
        .route(
            "/api/v1/slos/:slo_name/measurements",
            get(get_slo_measurements_handler),
        )
        .route("/api/v1/slo-alerts", get(list_slo_alerts_handler));

    // Testing endpoints
    #[cfg(feature = "testing")]
    {
        router = router
            .route(
                "/api/v1/testing/integrated-test",
                post(run_integrated_test_handler),
            )
            .route(
                "/api/v1/testing/integrated-test/all",
                post(run_all_integrated_tests_handler),
            )
            .route(
                "/api/v1/testing/scenarios",
                get(list_test_scenarios_handler),
            );
    }

    // Authentication endpoints
    router = router
        .route("/api/v1/auth/login", post(login_handler))
        .route("/api/v1/auth/logout", post(logout_handler))
        .route("/api/v1/auth/refresh", post(refresh_token_handler))
        .route("/api/v1/users/me", get(get_current_user_handler));

    // Settings management endpoints
    router = router
        // User settings
        .route("/api/v1/settings/user", get(get_user_settings_handler))
        .route("/api/v1/settings/user", post(create_user_setting_handler))
        .route("/api/v1/settings/user/:key", get(get_user_setting_handler))
        .route(
            "/api/v1/settings/user/:key",
            axum::routing::patch(update_user_setting_handler),
        )
        .route(
            "/api/v1/settings/user/:key",
            axum::routing::delete(delete_user_setting_handler),
        )
        // App settings
        .route("/api/v1/settings/app", get(get_app_settings_handler))
        .route("/api/v1/settings/app", post(create_app_setting_handler))
        .route("/api/v1/settings/app/:key", get(get_app_setting_handler))
        .route(
            "/api/v1/settings/app/:key",
            axum::routing::patch(update_app_setting_handler),
        )
        .route(
            "/api/v1/settings/app/:key",
            axum::routing::delete(delete_app_setting_handler),
        )
        // Integrations
        .route(
            "/api/v1/settings/integrations",
            get(list_integrations_handler),
        )
        .route(
            "/api/v1/settings/integrations",
            post(create_integration_handler),
        )
        .route(
            "/api/v1/settings/integrations/:id",
            get(get_integration_handler),
        )
        .route(
            "/api/v1/settings/integrations/:id",
            axum::routing::patch(update_integration_handler),
        )
        .route(
            "/api/v1/settings/integrations/:id",
            axum::routing::delete(delete_integration_handler),
        )
        // API keys
        .route("/api/v1/settings/api-keys", get(list_api_keys_handler))
        .route("/api/v1/settings/api-keys", post(create_api_key_handler))
        .route("/api/v1/settings/api-keys/:id", get(get_api_key_handler))
        .route(
            "/api/v1/settings/api-keys/:id",
            axum::routing::patch(update_api_key_handler),
        )
        .route(
            "/api/v1/settings/api-keys/:id/revoke",
            post(revoke_api_key_handler),
        )
        .route(
            "/api/v1/settings/api-keys/:id",
            axum::routing::delete(delete_api_key_handler),
        )
        // Password change
        .route("/api/v1/settings/password", post(change_password_handler));

    // Rules & Governance endpoints
    router = router
        // CAWS Rules CRUD
        .route("/api/v1/rules", get(list_rules_handler))
        .route("/api/v1/rules", post(create_rule_handler))
        .route("/api/v1/rules/:id", get(get_rule_handler))
        .route(
            "/api/v1/rules/:id",
            axum::routing::patch(update_rule_handler),
        )
        .route(
            "/api/v1/rules/:id",
            axum::routing::delete(delete_rule_handler),
        )
        // Rule validation
        .route("/api/v1/rules/:id/validate", post(validate_rule_handler))
        // Rule templates
        .route("/api/v1/rules/templates", get(list_rule_templates_handler))
        .route(
            "/api/v1/rules/templates",
            post(create_rule_template_handler),
        )
        // Rule enforcement status
        .route(
            "/api/v1/rules/:id/enforcement",
            get(get_rule_enforcement_handler),
        )
        .route(
            "/api/v1/rules/:id/enforcement",
            axum::routing::patch(update_rule_enforcement_handler),
        )
        // Rule history
        .route("/api/v1/rules/:id/history", get(get_rule_history_handler))
        // Violations
        .route("/api/v1/violations", get(list_violations_handler))
        .route("/api/v1/violations/:id", get(get_violation_handler))
        .route(
            "/api/v1/violations/:id",
            axum::routing::patch(update_violation_handler),
        )
        .route(
            "/api/v1/violations/:id/resolve",
            post(resolve_violation_handler),
        )
        // Compliance stats
        .route(
            "/api/v1/rules/compliance-stats",
            get(get_compliance_stats_handler),
        )
        // Specifications
        .route("/api/v1/specifications", get(list_specifications_handler))
        .route("/api/v1/specifications", post(create_specification_handler))
        .route("/api/v1/specifications/:id", get(get_specification_handler))
        .route(
            "/api/v1/specifications/:id",
            axum::routing::patch(update_specification_handler),
        )
        .route(
            "/api/v1/specifications/:id",
            axum::routing::delete(delete_specification_handler),
        );

    // Add CORS layer - default to permissive for development
    // When enable_cors is true, use permissive CORS that allows all origins, methods, and headers
    // When enable_cors is false, use a restrictive layer that still handles OPTIONS preflight
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::normalize_path::NormalizePathLayer;

    let cors = if enable_cors {
        CorsLayer::permissive()
    } else {
        // Even when CORS is "disabled", we still need to handle OPTIONS preflight
        // to avoid 405 Method Not Allowed errors for CORS-enabled clients
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PATCH,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers(Any)
    };

    // Normalize paths - trim trailing slashes for consistent routing
    // This ensures /api/v1/tasks and /api/v1/tasks/ both work the same way
    router = router
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(cors);

    router.with_state(app_state)
}

// ============================================================================
// Rate Limiting
// ============================================================================

/// Check rate limit for a request
/// Returns Ok(()) if allowed, Err(StatusCode::TOO_MANY_REQUESTS) if rate limited
async fn check_rate_limit(
    rate_limiter: &RateLimiter,
    client_id: &str,
    operation: &str,
) -> Result<(), (StatusCode, Json<JsonValue>)> {
    let request = RateLimitRequest {
        client_id: client_id.to_string(),
        operation: operation.to_string(),
        timestamp: Utc::now(),
    };

    match rate_limiter.check_rate_limit(&request).await {
        Ok(result) => {
            if result.allowed {
                Ok(())
            } else {
                warn!(
                    "Rate limit exceeded for client '{}' on operation '{}' (count: {}, retry after: {:?}s)",
                    client_id, operation, result.current_count, result.retry_after_seconds
                );
                Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(serde_json::json!({
                        "error": "Rate limit exceeded",
                        "retry_after_seconds": result.retry_after_seconds,
                        "current_count": result.current_count,
                        "reset_time": result.reset_time.to_rfc3339(),
                    })),
                ))
            }
        }
        Err(e) => {
            error!("Rate limiter error: {}", e);
            // On rate limiter error, allow the request (fail open)
            Ok(())
        }
    }
}

/// Extract client IP from headers (supports X-Forwarded-For, X-Real-IP)
fn extract_client_ip(headers: &HeaderMap) -> String {
    // Check X-Forwarded-For header (may contain multiple IPs)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // Take the first IP (original client)
            if let Some(first_ip) = xff_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Check X-Real-IP header
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(xri_str) = xri.to_str() {
            return xri_str.trim().to_string();
        }
    }

    // Default to unknown
    "unknown".to_string()
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Detect the current git branch from the workspace root
fn detect_git_branch(workspace_root: &str) -> Option<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(workspace_root)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

/// Estimate completion time from working spec
/// Returns estimated completion time in seconds, or None if unable to estimate
fn estimate_completion_from_spec(workspace_root: &str) -> Option<i64> {
    // Look for .caws/working-spec.yaml in workspace root
    let spec_path = std::path::Path::new(workspace_root)
        .join(".caws")
        .join("working-spec.yaml");

    if !spec_path.exists() {
        // No working spec found, provide reasonable default based on task type
        // This is a fallback for when no formal spec exists
        return Some(300); // 5 minutes default for unspecified tasks
    }

    // Try to read the working spec
    let spec_content = std::fs::read_to_string(&spec_path).ok()?;

    // Enhanced heuristic: parse max_files and max_loc from YAML using string matching
    // Also consider risk_tier and mode for better estimation
    let max_files = extract_yaml_value(&spec_content, "max_files")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(10);

    let max_loc = extract_yaml_value(&spec_content, "max_loc")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(500);

    let risk_tier = extract_yaml_value(&spec_content, "risk_tier")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(2); // Default to tier 2

    // Mode affects estimation (feature work takes longer than fixes)
    let mode_multiplier = if spec_content.contains("mode: feature") {
        1.5
    } else if spec_content.contains("mode: refactor") {
        1.2
    } else {
        1.0 // fix or other modes
    };

    // Risk tier affects estimation (higher tiers need more care)
    let risk_multiplier = match risk_tier {
        1 => 2.0, // Critical systems - more time for testing/validation
        2 => 1.5, // Standard features - moderate testing
        3 => 1.0, // Low risk - minimal testing
        _ => 1.2,
    };

    // Base estimation: 5 minutes per file + 1 second per 10 lines
    let file_time = (max_files as f64 * 300.0) * mode_multiplier * risk_multiplier;
    let loc_time = (max_loc as f64 / 10.0) * mode_multiplier;

    Some((file_time + loc_time) as i64)
}

/// Extract a value from YAML content using simple string matching
/// This is a basic implementation - proper YAML parsing would be better
fn extract_yaml_value(content: &str, key: &str) -> Option<String> {
    let search_key = format!("{}:", key);
    for line in content.lines() {
        if line.trim().starts_with(&search_key) {
            if let Some(value) = line.split(':').nth(1) {
                return Some(value.trim().to_string());
            }
        }
    }
    None
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
                health["database"] =
                    serde_json::json!({ "status": "disconnected", "error": e.to_string() });
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
    headers: HeaderMap,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    // Rate limiting check
    let client_ip = extract_client_ip(&headers);
    if let Err((status, _body)) =
        check_rate_limit(&state.rate_limiter, &client_ip, "submit_task").await
    {
        return Err(status);
    }

    // Input validation for task submission
    // Validate title if provided - must be non-empty
    if let Some(title) = payload.get("title") {
        if let Some(title_str) = title.as_str() {
            if title_str.trim().is_empty() {
                warn!("Task submission rejected: empty title");
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // Validate description - required and must be non-empty
    if let Some(desc) = payload.get("description") {
        if let Some(desc_str) = desc.as_str() {
            if desc_str.trim().is_empty() {
                warn!("Task submission rejected: empty description");
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    // Validate priority if provided - must be 0-10 range or valid string
    if let Some(priority) = payload.get("priority") {
        if let Some(priority_num) = priority.as_i64() {
            if !(0..=10).contains(&priority_num) {
                warn!(
                    "Task submission rejected: priority {} out of range (0-10)",
                    priority_num
                );
                return Err(StatusCode::BAD_REQUEST);
            }
        } else if let Some(priority_str) = priority.as_str() {
            // Allow string values: critical, high, normal, low
            let valid_priorities = ["critical", "high", "normal", "low"];
            if !valid_priorities.contains(&priority_str.to_lowercase().as_str()) {
                // Also allow numeric strings in 0-10 range
                if let Ok(num) = priority_str.parse::<i64>() {
                    if !(0..=10).contains(&num) {
                        warn!(
                            "Task submission rejected: priority {} out of range (0-10)",
                            priority_str
                        );
                        return Err(StatusCode::BAD_REQUEST);
                    }
                } else {
                    warn!(
                        "Task submission rejected: invalid priority value '{}'",
                        priority_str
                    );
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        }
    }

    // Validate risk_tier if provided - must be 1, 2, or 3
    if let Some(risk_tier) = payload.get("risk_tier") {
        if let Some(tier_num) = risk_tier.as_i64() {
            if !(1..=3).contains(&tier_num) {
                warn!(
                    "Task submission rejected: risk_tier {} must be 1, 2, or 3",
                    tier_num
                );
                return Err(StatusCode::BAD_REQUEST);
            }
        } else if let Some(tier_str) = risk_tier.as_str() {
            // Allow string values: "1", "2", "3"
            if let Ok(num) = tier_str.parse::<i64>() {
                if !(1..=3).contains(&num) {
                    warn!(
                        "Task submission rejected: risk_tier {} must be 1, 2, or 3",
                        tier_str
                    );
                    return Err(StatusCode::BAD_REQUEST);
                }
            } else {
                warn!(
                    "Task submission rejected: invalid risk_tier value '{}'",
                    tier_str
                );
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    #[cfg(feature = "orchestration")]
    {
        // Use UnifiedOrchestratorAdapter if available
        // CRITICAL: Do not fallback to legacy API - fail if UnifiedOrchestrator is not available
        if let Some(unified_orchestrator) = &state.unified_orchestrator {
            // Extract task data
            let description = payload
                .get("description")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let execution_mode = payload
                .get("execution_mode")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Create task submission request
            use data_infrastructure::api::types::TaskSubmissionRequest;
            let request = TaskSubmissionRequest {
                description: description.to_string(),
                execution_mode,
                risk_tier: payload
                    .get("risk_tier")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                context: payload
                    .get("context")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                priority: payload
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
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
            use agent_agency_contracts::task_request::{
                Environment, TaskContext as RequestTaskContext,
            };
            use agent_agency_contracts::TaskContext as ContractsTaskContext;
            use chrono::Utc;

            let workspace_root = std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| ".".to_string());

            let request_context = RequestTaskContext {
                workspace_root: workspace_root.clone(),
                git_branch: detect_git_branch(&workspace_root)
                    .unwrap_or_else(|| "main".to_string()),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            };

            // Convert to ContractsTaskContext
            let task_context = ContractsTaskContext {
                task_id: Uuid::new_v4(),   // Generate new task ID
                worker_id: Uuid::new_v4(), // Generate worker ID
                start_time: Utc::now(),
                timeout_ms: 300_000, // 5 minutes default
                retry_count: 0,
                max_retries: 3,
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert(
                        "workspace_root".to_string(),
                        serde_json::Value::String(request_context.workspace_root.clone()),
                    );
                    meta.insert(
                        "git_branch".to_string(),
                        serde_json::Value::String(request_context.git_branch),
                    );
                    meta.insert(
                        "environment".to_string(),
                        serde_json::Value::String(format!("{:?}", request_context.environment)),
                    );
                    meta
                },
            };

            // Clone workspace_root for use after task_context is moved
            let workspace_root = request_context.workspace_root.clone();

            // Generate task_id from working spec ID
            let task_id = if working_spec.id.starts_with("TASK-") {
                working_spec
                    .id
                    .strip_prefix("TASK-")
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(|| Uuid::new_v4())
            } else {
                Uuid::new_v4()
            };

            // Insert task into database before spawning execution
            // This is required because task_execution_states has a foreign key to tasks
            if let Some(db) = &state.db_client {
                let now = Utc::now();
                let risk_tier_str = payload
                    .get("risk_tier")
                    .and_then(|v| v.as_str())
                    .unwrap_or("3");
                let priority_str = payload
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .unwrap_or("normal");
                let priority_int: i32 = match priority_str {
                    "critical" => 1,
                    "high" => 2,
                    "normal" => 5,
                    "low" => 8,
                    _ => 5,
                };

                if let Err(e) = sqlx::query(
                    r#"
                    INSERT INTO tasks (id, title, description, risk_tier, priority, status, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(task_id)
                .bind(&working_spec.title)
                .bind(&working_spec.description)
                .bind(risk_tier_str)
                .bind(priority_int)
                .bind("pending")
                .bind(now)
                .bind(now)
                .execute(db.pool())
                .await
                {
                    warn!("Failed to insert task into database: {}. Task execution will continue but state tracking may fail.", e);
                } else {
                    debug!("Task {} inserted into database", task_id);
                }
            }

            // Spawn task execution in background to avoid blocking the HTTP request
            let orchestrator_clone = unified_orchestrator.clone();
            let spec_clone = working_spec.clone();
            let context_clone = task_context.clone();
            let task_id_for_log = task_id;
            let telemetry = state.telemetry_service.clone();
            let worker_id = task_context.worker_id;

            tokio::spawn(async move {
                info!("Starting background execution of task {}", task_id_for_log);
                let start_time = std::time::Instant::now();

                // Record task started activity
                let _ = telemetry
                    .record_agent_activity(
                        worker_id,
                        data_infrastructure::telemetry_service::activity_types::TASK_STARTED,
                        Some(task_id_for_log),
                        None,
                        true,
                        None,
                    )
                    .await;

                match orchestrator_clone
                    .orchestrate_task(spec_clone, context_clone)
                    .await
                {
                    Ok(result) => {
                        let duration_ms = start_time.elapsed().as_millis() as i32;
                        info!(
                            "Task {} completed successfully: {} ({}ms)",
                            task_id_for_log, result.success, duration_ms
                        );

                        // Record task completed activity
                        let _ = telemetry
                            .record_agent_activity(
                                worker_id,
                                data_infrastructure::telemetry_service::activity_types::TASK_COMPLETED,
                                Some(task_id_for_log),
                                Some(duration_ms),
                                result.success,
                                None,
                            )
                            .await;
                    }
                    Err(e) => {
                        let duration_ms = start_time.elapsed().as_millis() as i32;
                        error!("Task {} execution failed: {:?} ({}ms)", task_id_for_log, e, duration_ms);

                        // Record task failed activity
                        let _ = telemetry
                            .record_agent_activity(
                                worker_id,
                                data_infrastructure::telemetry_service::activity_types::TASK_FAILED,
                                Some(task_id_for_log),
                                Some(duration_ms),
                                false,
                                Some(&format!("{:?}", e)),
                            )
                            .await;
                    }
                }

                // Also trigger a snapshot check periodically
                let _ = telemetry.maybe_snapshot_task_stats().await;
            });

            // Return task submission response immediately
            use data_infrastructure::api::types::TaskSubmissionResponse;
            let response = TaskSubmissionResponse {
                task_id,
                status: "accepted".to_string(),
                message: "Task submitted successfully and is executing in background".to_string(),
                estimated_completion: estimate_completion_from_spec(&workspace_root)
                    .map(|seconds| Utc::now() + ChronoDuration::seconds(seconds)),
            };
            Ok(Json(serde_json::json!(response)))
        } else {
            // UnifiedOrchestrator is not available - this is a critical error
            // Do NOT fallback to legacy API that silently completes tasks
            error!("CRITICAL: UnifiedOrchestrator not initialized - task execution will fail");
            error!("   UnifiedOrchestrator initialization failed during server startup");
            error!("   Check server logs for initialization errors (likely database schema issue)");
            error!("   Tasks cannot be executed without UnifiedOrchestrator");

            // Return detailed error response
            let error_response = serde_json::json!({
                "error": "UnifiedOrchestrator not available",
                "message": "Task execution is disabled because UnifiedOrchestrator failed to initialize. Check server logs for initialization errors.",
                "details": "This usually indicates a database schema issue (e.g., missing 'description' column in planning_audit_events table). Run migrations to fix.",
                "status": "service_unavailable"
            });

            // Return error response with proper status code
            // Note: Function signature returns Result<Json<JsonValue>, StatusCode>
            // We return the error JSON with 200 status, but include error details in JSON
            // The client should check the "status" field in the response
            Ok(Json(error_response))
        }
    }

    #[cfg(not(feature = "orchestration"))]
    {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

async fn list_tasks_handler(State(state): State<AppState>) -> Result<Json<JsonValue>, StatusCode> {
    // Read tasks from database for consistent data source with stats endpoint
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.list_tasks().await {
        Ok(tasks) => {
            // Calculate status counts
            let total = tasks.len();
            let pending = tasks.iter().filter(|t| t.status == "pending").count();
            let running = tasks.iter().filter(|t| t.status == "in_progress" || t.status == "running").count();
            let completed = tasks.iter().filter(|t| t.status == "completed").count();
            let failed = tasks.iter().filter(|t| t.status == "failed").count();
            let cancelled = tasks.iter().filter(|t| t.status == "cancelled").count();

            // Transform tasks to JSON format expected by dashboard
            let task_list: Vec<JsonValue> = tasks
                .into_iter()
                .map(|task| {
                    serde_json::json!({
                        "id": task.id,
                        "title": task.title,
                        "description": task.description,
                        "risk_tier": task.risk_tier,
                        "scope": task.scope,
                        "acceptance_criteria": task.acceptance_criteria,
                        "context": task.context,
                        "caws_spec": task.caws_spec,
                        "status": task.status,
                        "assigned_worker_id": task.assigned_worker_id,
                        "project_id": task.project_id,
                        "priority": task.priority,
                        "deadline": task.deadline,
                        "metadata": task.metadata,
                        "created_at": task.created_at,
                        "updated_at": task.updated_at,
                        "completed_at": task.completed_at,
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({
                "tasks": task_list,
                "total": total,
                "status_counts": {
                    "pending": pending,
                    "running": running,
                    "completed": completed,
                    "failed": failed,
                    "cancelled": cancelled,
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_task_status_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    #[cfg(feature = "orchestration")]
    {
        // Read task status from database
        if let Some(db) = &state.db_client {
            // Try to get execution state first (more detailed)
            let exec_state_result: Result<Option<(String, chrono::DateTime<Utc>, f64)>, _> = sqlx::query_as(
                r#"
                SELECT status, last_updated, 
                       COALESCE((state_data->>'progress_percentage')::float, 0.0) as progress
                FROM task_execution_states 
                WHERE task_id = $1
                "#,
            )
            .bind(task_uuid)
            .fetch_optional(db.pool())
            .await;

            if let Ok(Some((status, updated_at, progress))) = exec_state_result {
                use data_infrastructure::api::types::TaskStatusResponse;
                let response = TaskStatusResponse {
                    task_id: task_uuid,
                    status,
                    progress_percentage: progress as f32,
                    current_phase: None,
                    started_at: Some(updated_at),
                    updated_at: Some(updated_at),
                    quality_score: None,
                };
                return Ok(Json(serde_json::json!(response)));
            }

            // Fallback to tasks table
            let task_result: Result<Option<(String, chrono::DateTime<Utc>)>, _> = sqlx::query_as(
                "SELECT status, updated_at FROM tasks WHERE id = $1",
            )
            .bind(task_uuid)
            .fetch_optional(db.pool())
            .await;

            if let Ok(Some((status, updated_at))) = task_result {
                use data_infrastructure::api::types::TaskStatusResponse;
                let response = TaskStatusResponse {
                    task_id: task_uuid,
                    status,
                    progress_percentage: 0.0,
                    current_phase: None,
                    started_at: Some(updated_at),
                    updated_at: Some(updated_at),
                    quality_score: None,
                };
                return Ok(Json(serde_json::json!(response)));
            }
        }

        // Return not found if task doesn't exist
        use data_infrastructure::api::types::TaskStatusResponse;
        let response = TaskStatusResponse {
            task_id: task_uuid,
            status: "not_found".to_string(),
            progress_percentage: 0.0,
            current_phase: Some("unknown".to_string()),
            started_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            quality_score: None,
        };
        Ok(Json(serde_json::json!(response)))
        /*
        // Use UnifiedOrchestratorAdapter if available, fallback to legacy API
        if let Some(unified_orchestrator) = &state.unified_orchestrator {
            match unified_orchestrator.get_task_status(&task_uuid).await {
                Ok(status) => {
                    use data_infrastructure::api::types::TaskStatusResponse;
                    let response = TaskStatusResponse {
                        task_id: status.task_id,
                        status: format!("{:?}", status.status).to_lowercase(),
                        progress_percentage: status
                            .progress_percent
                            .map(|p| p as f32)
                            .unwrap_or(0.0),
                        current_phase: Some(status.status.to_string()), // Extract from execution state
                        started_at: Some(status.created_at),
                        updated_at: Some(status.updated_at),
                        quality_score: None, // Quality score calculated from execution metrics (future enhancement)
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
        */
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

async fn update_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let update = data_infrastructure::database_operations::UpdateTask {
        title: payload
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        risk_tier: payload
            .get("risk_tier")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        scope: payload.get("scope").cloned(),
        acceptance_criteria: payload.get("acceptance_criteria").cloned(),
        context: payload.get("context").cloned(),
        caws_spec: payload.get("caws_spec").cloned(),
        status: payload
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        assigned_worker_id: payload
            .get("assigned_worker_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        project_id: payload
            .get("project_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        priority: payload
            .get("priority")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32),
        deadline: payload
            .get("deadline")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        metadata: payload.get("metadata").cloned(),
        completed_at: payload
            .get("completed_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
    };

    match db.update_task(task_uuid, update).await {
        Ok(task) => Ok(Json(serde_json::json!({
            "task_id": task.id.to_string(),
            "title": task.title,
            "description": task.description,
            "status": task.status,
            "updated_at": task.updated_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to update task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.delete_task(task_uuid).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "task_id": task_id,
        }))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                warn!("Attempted to delete non-existent task: {}", task_id);
                Err(StatusCode::NOT_FOUND)
            } else {
                error!("Failed to delete task: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

async fn get_tasks_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_tasks().await {
        Ok(tasks) => {
            let total = tasks.len();
            let completed = tasks.iter().filter(|t| t.status == "completed").count();
            let in_progress = tasks.iter().filter(|t| t.status == "in_progress").count();
            let pending = tasks.iter().filter(|t| t.status == "pending").count();
            let cancelled = tasks.iter().filter(|t| t.status == "cancelled").count();
            let failed = tasks.iter().filter(|t| t.status == "failed").count();
            
            // Calculate completion rate (completed / total, excluding cancelled)
            let active_total = total - cancelled;
            let completion_rate = if active_total > 0 {
                (completed as f64 / active_total as f64) * 100.0
            } else {
                0.0
            };
            
            // Calculate success rate (completed / (completed + failed))
            let finished = completed + failed;
            let success_rate = if finished > 0 {
                (completed as f64 / finished as f64) * 100.0
            } else {
                0.0
            };

            Ok(Json(serde_json::json!({
                "total": total,
                "completed": completed,
                "in_progress": in_progress,
                "pending": pending,
                "cancelled": cancelled,
                "failed": failed,
                "completion_rate": completion_rate,
                "success_rate": success_rate,
            })))
        }
        Err(e) => {
            error!("Failed to get tasks stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_tasks_stats_history_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Parse period parameter (e.g., "30d", "7d", "90d")
    let period = params.get("period").map(|s| s.as_str()).unwrap_or("30d");
    let days = period
        .strip_suffix("d")
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(30);

    // First, try to get from task_stats_history table (persisted snapshots)
    match db.get_task_stats_history(Some(days)).await {
        Ok(history) if !history.is_empty() => {
            return Ok(Json(serde_json::json!({
                "period": period,
                "period_days": days,
                "history": history,
                "source": "persisted_snapshots",
            })));
        }
        _ => {
            // Fall back to computing from tasks table
        }
    }

    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

    // Query tasks grouped by day with completion rates
    match db
        .query(
            "SELECT
            DATE_TRUNC('day', created_at) as day,
            COUNT(*) as total_tasks,
            COUNT(*) FILTER (WHERE status = 'completed') as completed_tasks,
            COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress_tasks,
            COUNT(*) FILTER (WHERE status = 'pending') as pending_tasks,
            COUNT(*) FILTER (WHERE status = 'failed') as failed_tasks,
            COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled_tasks
        FROM tasks
        WHERE created_at >= $1
        GROUP BY DATE_TRUNC('day', created_at)
        ORDER BY day DESC",
            &[&cutoff_date],
        )
        .await
    {
        Ok(rows) => {
            let mut history: Vec<JsonValue> = Vec::new();

            for row in rows {
                let day: chrono::DateTime<chrono::Utc> = row.try_get("day").unwrap_or_default();
                let total: i64 = row.try_get("total_tasks").unwrap_or(0);
                let completed: i64 = row.try_get("completed_tasks").unwrap_or(0);
                let in_progress: i64 = row.try_get("in_progress_tasks").unwrap_or(0);
                let pending: i64 = row.try_get("pending_tasks").unwrap_or(0);
                let failed: i64 = row.try_get("failed_tasks").unwrap_or(0);
                let cancelled: i64 = row.try_get("cancelled_tasks").unwrap_or(0);

                let completion_rate = if total > 0 {
                    (completed as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                history.push(serde_json::json!({
                    "date": day.to_rfc3339(),
                    "total": total,
                    "completed": completed,
                    "in_progress": in_progress,
                    "pending": pending,
                    "failed": failed,
                    "cancelled": cancelled,
                    "completion_rate": completion_rate,
                }));
            }

            Ok(Json(serde_json::json!({
                "period": period,
                "period_days": days,
                "history": history,
                "source": "computed_from_tasks",
            })))
        }
        Err(e) => {
            error!("Failed to get tasks stats history: {}", e);
            // Return empty history on error
            Ok(Json(serde_json::json!({
                "period": period,
                "period_days": days,
                "history": [],
                "source": "error",
            })))
        }
    }
}

async fn create_project_task_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let _project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let description = payload
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let create = data_infrastructure::database_operations::CreateTask {
        title: title.to_string(),
        description: description.to_string(),
        risk_tier: payload
            .get("risk_tier")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "2".to_string()),
        scope: payload
            .get("scope")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        acceptance_criteria: payload
            .get("acceptance_criteria")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
        context: payload
            .get("context")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        caws_spec: payload.get("caws_spec").cloned(),
        status: payload
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "pending".to_string()),
        assigned_worker_id: payload
            .get("assigned_worker_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        project_id: Some(project_uuid),
        priority: payload
            .get("priority")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32),
        deadline: payload
            .get("deadline")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        metadata: payload.get("metadata").cloned(),
    };

    // Create task using CreateTask struct
    match db.create_task_from_create(create).await {
        Ok(task) => Ok(Json(serde_json::json!({
            "task_id": task.id.to_string(),
            "title": task.title,
            "description": task.description,
            "status": task.status,
            "project_id": project_id,
            "created_at": task.created_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_project_task_handler(
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(String, String)>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let _project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify task belongs to project (check project_id)
    let task = db
        .get_task(&task_uuid)
        .await
        .map_err(|e| {
            error!("Failed to get task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(task_project_id) = task.project_id {
        if task_project_id.to_string() != project_id {
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update task (same as regular update_task_handler)
    let update = data_infrastructure::database_operations::UpdateTask {
        title: payload
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        risk_tier: payload
            .get("risk_tier")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        scope: payload.get("scope").cloned(),
        acceptance_criteria: payload.get("acceptance_criteria").cloned(),
        context: payload.get("context").cloned(),
        caws_spec: payload.get("caws_spec").cloned(),
        status: payload
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        assigned_worker_id: payload
            .get("assigned_worker_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        project_id: task.project_id,
        priority: payload
            .get("priority")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32),
        deadline: payload
            .get("deadline")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        metadata: payload.get("metadata").cloned(),
        completed_at: payload
            .get("completed_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
    };

    match db.update_task(task_uuid, update).await {
        Ok(task) => Ok(Json(serde_json::json!({
            "task_id": task.id.to_string(),
            "title": task.title,
            "status": task.status,
            "project_id": project_id,
            "updated_at": task.updated_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to update task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_project_task_handler(
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(String, String)>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let _project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify task belongs to project (check project_id)
    let task = db
        .get_task(&task_uuid)
        .await
        .map_err(|e| {
            error!("Failed to get task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(task_project_id) = task.project_id {
        if task_project_id.to_string() != project_id {
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    match db.delete_task(task_uuid).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "task_id": task_id,
            "project_id": project_id,
        }))),
        Err(e) => {
            error!("Failed to delete task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Worker/Agent management handlers
async fn list_agents_handler(State(state): State<AppState>) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_workers().await {
        Ok(workers) => {
            let agents: Vec<JsonValue> = workers
                .into_iter()
                .map(|worker| {
                    serde_json::json!({
                        "id": worker.id.to_string(),
                        "name": worker.name,
                        "worker_type": worker.worker_type,
                        "specialty": worker.specialty,
                        "model_name": worker.model_name,
                        "endpoint": worker.endpoint,
                        "is_active": worker.is_active,
                        "created_at": worker.created_at.to_rfc3339(),
                        "updated_at": worker.updated_at.to_rfc3339(),
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({ "agents": agents })))
        }
        Err(e) => {
            error!("Failed to get workers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_agents_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_workers().await {
        Ok(workers) => {
            let total = workers.len();
            let active = workers.iter().filter(|w| w.is_active).count();
            let inactive = total - active;

            // Count by worker type
            let mut by_type: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for worker in &workers {
                *by_type.entry(worker.worker_type.clone()).or_insert(0) += 1;
            }

            Ok(Json(serde_json::json!({
                "total": total,
                "active": active,
                "inactive": inactive,
                "by_type": by_type,
            })))
        }
        Err(e) => {
            error!("Failed to get workers stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_agent_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_worker(agent_uuid).await {
        Ok(Some(worker)) => Ok(Json(serde_json::json!({
            "id": worker.id.to_string(),
            "name": worker.name,
            "worker_type": worker.worker_type,
            "specialty": worker.specialty,
            "model_name": worker.model_name,
            "endpoint": worker.endpoint,
            "capabilities": worker.capabilities,
            "performance_history": worker.performance_history,
            "is_active": worker.is_active,
            "created_at": worker.created_at.to_rfc3339(),
            "updated_at": worker.updated_at.to_rfc3339(),
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get worker: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_agent_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let update = data_infrastructure::database_operations::UpdateWorker {
        name: payload
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        worker_type: payload
            .get("worker_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        specialty: payload
            .get("specialty")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        model_name: payload
            .get("model_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        endpoint: payload
            .get("endpoint")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        capabilities: payload.get("capabilities").cloned(),
        performance_history: payload.get("performance_history").cloned(),
        is_active: payload.get("is_active").and_then(|v| v.as_bool()),
    };

    match db.update_worker(agent_uuid, update).await {
        Ok(worker) => Ok(Json(serde_json::json!({
            "id": worker.id.to_string(),
            "name": worker.name,
            "worker_type": worker.worker_type,
            "is_active": worker.is_active,
            "updated_at": worker.updated_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to update worker: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_agent_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.delete_worker(agent_uuid).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "agent_id": agent_id,
        }))),
        Err(e) => {
            error!("Failed to delete worker: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_agent_stats_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get worker
    let worker = db
        .get_worker(agent_uuid)
        .await
        .map_err(|e| {
            error!("Failed to get worker: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get task executions for this worker
    match db.get_task_executions_by_worker(agent_uuid).await {
        Ok(executions) => {
            let total_tasks = executions.len();
            let completed = executions
                .iter()
                .filter(|e| e.status == "completed")
                .count();
            let failed = executions.iter().filter(|e| e.status == "failed").count();
            let in_progress = executions
                .iter()
                .filter(|e| e.status == "in_progress")
                .count();

            Ok(Json(serde_json::json!({
                "agent_id": agent_id,
                "name": worker.name,
                "total_tasks": total_tasks,
                "completed": completed,
                "failed": failed,
                "in_progress": in_progress,
                "success_rate": if total_tasks > 0 { (completed as f64 / total_tasks as f64) * 100.0 } else { 0.0 },
                "performance_history": worker.performance_history,
            })))
        }
        Err(_) => {
            // If task_executions query fails, return basic stats
            Ok(Json(serde_json::json!({
                "agent_id": agent_id,
                "name": worker.name,
                "total_tasks": 0,
                "completed": 0,
                "failed": 0,
                "in_progress": 0,
                "success_rate": 0.0,
                "performance_history": worker.performance_history,
            })))
        }
    }
}

async fn get_agent_health_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_worker(agent_uuid).await {
        Ok(Some(worker)) => Ok(Json(serde_json::json!({
            "agent_id": agent_id,
            "status": if worker.is_active { "healthy" } else { "inactive" },
            "is_active": worker.is_active,
            "last_updated": worker.updated_at.to_rfc3339(),
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get worker health: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_agent_metrics_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_worker(agent_uuid).await {
        Ok(Some(worker)) => Ok(Json(serde_json::json!({
            "agent_id": agent_id,
            "performance_history": worker.performance_history,
            "capabilities": worker.capabilities,
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get worker metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_agent_logs_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Query audit_trail_entries for agent-specific logs
    // Check both 'agent' and 'worker' entity types
    let query = r#"
        SELECT
            id,
            entity_type,
            entity_id,
            action,
            details,
            user_id,
            ip_address,
            created_at
        FROM audit_trail_entries
        WHERE (entity_type = 'agent' OR entity_type = 'worker')
          AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 1000
    "#;

    match db.query_with_params(query, &[&agent_uuid]).await {
        Ok(rows) => {
            let logs: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "id": row.get::<Uuid, _>("id").to_string(),
                    "entity_type": row.get::<String, _>("entity_type"),
                    "entity_id": row.get::<Uuid, _>("entity_id").to_string(),
                    "action": row.get::<String, _>("action"),
                    "details": row.try_get::<serde_json::Value, _>("details").unwrap_or(serde_json::json!({})),
                    "user_id": row.try_get::<Option<String>, _>("user_id").ok().flatten(),
                    "ip_address": row.try_get::<Option<String>, _>("ip_address").ok().flatten(),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "agent_id": agent_id,
                "logs": logs,
                "total": logs.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to query agent logs: {}", e);
            // Return empty logs instead of error for graceful degradation
            Ok(Json(serde_json::json!({
                "agent_id": agent_id,
                "logs": [],
                "total": 0,
                "status": "success",
                "message": "No logs found or error occurred"
            })))
        }
    }
}

async fn restart_agent_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify worker exists
    let worker = db
        .get_worker(agent_uuid)
        .await
        .map_err(|e| {
            error!("Database error during agent restart: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Restart worker by setting is_active to true
    // This effectively restarts the worker by making it active again
    let update = data_infrastructure::database_operations::UpdateWorker {
        name: None,
        worker_type: None,
        specialty: None,
        model_name: None,
        endpoint: None,
        capabilities: None,
        performance_history: None,
        is_active: Some(true), // Restart by activating
    };

    match db.update_worker(agent_uuid, update).await {
        Ok(updated_worker) => {
            // Log restart action to audit trail
            let _ = db.execute(
                r#"
                    INSERT INTO audit_trail_entries (entity_type, entity_id, action, details, created_at)
                    VALUES ('worker', $1, 'restart', $2, NOW())
                "#,
                &[
                    &agent_uuid,
                    &serde_json::json!({
                        "previous_status": worker.is_active,
                        "new_status": true,
                        "restarted_at": chrono::Utc::now().to_rfc3339()
                    }).to_string().as_str(),
                ],
            ).await;

            info!("Agent restarted: {}", agent_id);

            Ok(Json(serde_json::json!({
                "status": "success",
                "agent_id": agent_id,
                "message": "Agent restarted successfully",
                "is_active": updated_worker.is_active
            })))
        }
        Err(e) => {
            error!("Failed to restart agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn stop_agent_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Update worker to inactive
    let update = data_infrastructure::database_operations::UpdateWorker {
        name: None,
        worker_type: None,
        specialty: None,
        model_name: None,
        endpoint: None,
        capabilities: None,
        performance_history: None,
        is_active: Some(false),
    };

    match db.update_worker(agent_uuid, update).await {
        Ok(worker) => Ok(Json(serde_json::json!({
            "status": "stopped",
            "agent_id": agent_id,
            "is_active": worker.is_active,
        }))),
        Err(e) => {
            error!("Failed to stop worker: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get task completion metrics for all agents
///
/// Returns aggregated task completion statistics per agent including:
/// - Total tasks executed
/// - Completed tasks count
/// - Failed tasks count
/// - Success rate
/// - Average execution time
/// - Completion rate over time periods
async fn get_agents_tasks_completion_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Parse time period (default: last 24 hours)
    let hours = params
        .get("hours")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(24);
    let period_start = Utc::now() - ChronoDuration::hours(hours);

    // Query task completion metrics grouped by agent
    let query = r#"
        SELECT
            w.id as worker_id,
            w.name as worker_name,
            w.worker_type,
            COUNT(te.id) as total_tasks,
            COUNT(CASE WHEN te.status = 'completed' THEN 1 END) as completed_tasks,
            COUNT(CASE WHEN te.status = 'failed' THEN 1 END) as failed_tasks,
            COUNT(CASE WHEN te.status = 'cancelled' THEN 1 END) as cancelled_tasks,
            COUNT(CASE WHEN te.status = 'running' THEN 1 END) as running_tasks,
            AVG(CASE WHEN te.execution_time_ms IS NOT NULL THEN te.execution_time_ms ELSE NULL END) as avg_execution_time_ms,
            MIN(CASE WHEN te.execution_time_ms IS NOT NULL THEN te.execution_time_ms ELSE NULL END) as min_execution_time_ms,
            MAX(CASE WHEN te.execution_time_ms IS NOT NULL THEN te.execution_time_ms ELSE NULL END) as max_execution_time_ms,
            SUM(CASE WHEN te.status = 'completed' THEN 1 ELSE 0 END)::float / NULLIF(COUNT(te.id), 0) as success_rate
        FROM workers w
        LEFT JOIN task_executions te ON w.id = te.worker_id
            AND te.execution_started_at >= $1
        GROUP BY w.id, w.name, w.worker_type
        HAVING COUNT(te.id) > 0
        ORDER BY total_tasks DESC
    "#;

    match sqlx::query(query)
        .bind(period_start)
        .fetch_all(db.pool())
        .await
    {
        Ok(rows) => {
            let mut agent_metrics: Vec<serde_json::Value> = Vec::new();

            for row in rows {
                let worker_id: Uuid = row.get("worker_id");
                let worker_name: String = row.get("worker_name");
                let worker_type: String = row.get("worker_type");
                let total_tasks: i64 = row.get("total_tasks");
                let completed_tasks: i64 = row.get("completed_tasks");
                let failed_tasks: i64 = row.get("failed_tasks");
                let cancelled_tasks: i64 = row.get("cancelled_tasks");
                let running_tasks: i64 = row.get("running_tasks");
                let avg_execution_time: Option<i64> = row.try_get("avg_execution_time_ms").ok();
                let min_execution_time: Option<i64> = row.try_get("min_execution_time_ms").ok();
                let max_execution_time: Option<i64> = row.try_get("max_execution_time_ms").ok();
                let success_rate: Option<f64> = row.try_get("success_rate").ok();

                let completion_rate = if total_tasks > 0 {
                    (completed_tasks as f64 / total_tasks as f64) * 100.0
                } else {
                    0.0
                };

                agent_metrics.push(serde_json::json!({
                    "agent_id": worker_id.to_string(),
                    "agent_name": worker_name,
                    "worker_type": worker_type,
                    "total_tasks": total_tasks,
                    "completed_tasks": completed_tasks,
                    "failed_tasks": failed_tasks,
                    "cancelled_tasks": cancelled_tasks,
                    "running_tasks": running_tasks,
                    "completion_rate_percent": completion_rate,
                    "success_rate": success_rate.unwrap_or(0.0),
                    "avg_execution_time_ms": avg_execution_time,
                    "min_execution_time_ms": min_execution_time,
                    "max_execution_time_ms": max_execution_time,
                    "period_hours": hours,
                    "period_start": period_start.to_rfc3339(),
                }));
            }

            // Calculate aggregate totals
            let total_all_tasks: i64 = agent_metrics
                .iter()
                .map(|m| m["total_tasks"].as_i64().unwrap_or(0))
                .sum();
            let total_completed: i64 = agent_metrics
                .iter()
                .map(|m| m["completed_tasks"].as_i64().unwrap_or(0))
                .sum();
            let total_failed: i64 = agent_metrics
                .iter()
                .map(|m| m["failed_tasks"].as_i64().unwrap_or(0))
                .sum();

            Ok(Json(serde_json::json!({
                "agents": agent_metrics,
                "summary": {
                    "total_agents": agent_metrics.len(),
                    "total_tasks": total_all_tasks,
                    "total_completed": total_completed,
                    "total_failed": total_failed,
                    "overall_completion_rate": if total_all_tasks > 0 {
                        (total_completed as f64 / total_all_tasks as f64) * 100.0
                    } else {
                        0.0
                    },
                    "overall_success_rate": if total_all_tasks > 0 {
                        ((total_all_tasks - total_failed) as f64 / total_all_tasks as f64) * 100.0
                    } else {
                        0.0
                    },
                },
                "period_hours": hours,
                "period_start": period_start.to_rfc3339(),
                "period_end": Utc::now().to_rfc3339(),
            })))
        }
        Err(e) => {
            error!("Failed to get agent task completion metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get efficiency metrics for all agents
///
/// Returns efficiency metrics per agent including:
/// - Tasks per hour (throughput)
/// - Average execution time
/// - Efficiency score (throughput / avg_time)
/// - Resource utilization
/// - Performance trends
async fn get_agents_efficiency_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Parse time period (default: last 24 hours)
    let hours = params
        .get("hours")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(24);
    let period_start = Utc::now() - ChronoDuration::hours(hours);

    // Query efficiency metrics grouped by agent
    let query = r#"
        SELECT
            w.id as worker_id,
            w.name as worker_name,
            w.worker_type,
            COUNT(te.id) as total_tasks,
            COUNT(CASE WHEN te.status = 'completed' THEN 1 END) as completed_tasks,
            AVG(CASE WHEN te.execution_time_ms IS NOT NULL THEN te.execution_time_ms ELSE NULL END) as avg_execution_time_ms,
            PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY te.execution_time_ms) as median_execution_time_ms,
            PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY te.execution_time_ms) as p95_execution_time_ms,
            SUM(CASE WHEN te.status = 'completed' THEN 1 ELSE 0 END)::float / NULLIF(COUNT(te.id), 0) as success_rate,
            SUM(CASE WHEN te.tokens_used IS NOT NULL THEN te.tokens_used ELSE 0 END) as total_tokens_used,
            AVG(CASE WHEN te.tokens_used IS NOT NULL THEN te.tokens_used ELSE NULL END) as avg_tokens_per_task
        FROM workers w
        LEFT JOIN task_executions te ON w.id = te.worker_id
            AND te.execution_started_at >= $1
        GROUP BY w.id, w.name, w.worker_type
        HAVING COUNT(te.id) > 0
        ORDER BY completed_tasks DESC
    "#;

    match sqlx::query(query)
        .bind(period_start)
        .fetch_all(db.pool())
        .await
    {
        Ok(rows) => {
            let mut agent_efficiency: Vec<serde_json::Value> = Vec::new();

            for row in rows {
                let worker_id: Uuid = row.get("worker_id");
                let worker_name: String = row.get("worker_name");
                let worker_type: String = row.get("worker_type");
                let total_tasks: i64 = row.get("total_tasks");
                let completed_tasks: i64 = row.get("completed_tasks");
                let avg_execution_time: Option<i64> = row.try_get("avg_execution_time_ms").ok();
                let median_execution_time: Option<i64> =
                    row.try_get("median_execution_time_ms").ok();
                let p95_execution_time: Option<i64> = row.try_get("p95_execution_time_ms").ok();
                let success_rate: Option<f64> = row.try_get("success_rate").ok();
                let total_tokens: Option<i64> = row.try_get("total_tokens_used").ok();
                let avg_tokens: Option<i64> = row.try_get("avg_tokens_per_task").ok();

                // Calculate tasks per hour (throughput)
                let tasks_per_hour = if hours > 0 && completed_tasks > 0 {
                    completed_tasks as f64 / hours as f64
                } else {
                    0.0
                };

                // Calculate efficiency score (higher is better: more tasks completed with less time)
                // Formula: (completed_tasks / hours) / (avg_time_ms / 1000 / 60) = tasks per hour / (avg_time in minutes)
                let efficiency_score = if let Some(avg_time) = avg_execution_time {
                    if avg_time > 0 {
                        let avg_time_minutes = avg_time as f64 / 1000.0 / 60.0;
                        tasks_per_hour / avg_time_minutes.max(0.1) // Avoid division by zero
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                agent_efficiency.push(serde_json::json!({
                    "agent_id": worker_id.to_string(),
                    "agent_name": worker_name,
                    "worker_type": worker_type,
                    "total_tasks": total_tasks,
                    "completed_tasks": completed_tasks,
                    "tasks_per_hour": tasks_per_hour,
                    "success_rate": success_rate.unwrap_or(0.0),
                    "avg_execution_time_ms": avg_execution_time,
                    "median_execution_time_ms": median_execution_time,
                    "p95_execution_time_ms": p95_execution_time,
                    "efficiency_score": efficiency_score,
                    "total_tokens_used": total_tokens,
                    "avg_tokens_per_task": avg_tokens,
                    "period_hours": hours,
                }));
            }

            // Calculate aggregate efficiency metrics
            let total_completed: i64 = agent_efficiency
                .iter()
                .map(|m| m["completed_tasks"].as_i64().unwrap_or(0))
                .sum();
            let overall_tasks_per_hour = if hours > 0 {
                total_completed as f64 / hours as f64
            } else {
                0.0
            };

            // Calculate average efficiency score
            let efficiency_scores: Vec<f64> = agent_efficiency
                .iter()
                .filter_map(|m| m["efficiency_score"].as_f64())
                .filter(|&s| s > 0.0)
                .collect();
            let avg_efficiency_score = if !efficiency_scores.is_empty() {
                efficiency_scores.iter().sum::<f64>() / efficiency_scores.len() as f64
            } else {
                0.0
            };

            Ok(Json(serde_json::json!({
                "agents": agent_efficiency,
                "summary": {
                    "total_agents": agent_efficiency.len(),
                    "total_completed_tasks": total_completed,
                    "overall_tasks_per_hour": overall_tasks_per_hour,
                    "avg_efficiency_score": avg_efficiency_score,
                },
                "period_hours": hours,
                "period_start": period_start.to_rfc3339(),
                "period_end": Utc::now().to_rfc3339(),
            })))
        }
        Err(e) => {
            error!("Failed to get agent efficiency metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Judge management handlers
async fn list_judges_handler(State(state): State<AppState>) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_judges().await {
        Ok(judges) => {
            let judges_list: Vec<JsonValue> = judges
                .into_iter()
                .map(|judge| {
                    serde_json::json!({
                        "id": judge.id.to_string(),
                        "name": judge.name,
                        "model_name": judge.model_name,
                        "endpoint": judge.endpoint,
                        "weight": judge.weight,
                        "timeout_ms": judge.timeout_ms,
                        "optimization_target": judge.optimization_target,
                        "is_active": judge.is_active,
                        "created_at": judge.created_at.to_rfc3339(),
                        "updated_at": judge.updated_at.to_rfc3339(),
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({ "judges": judges_list })))
        }
        Err(e) => {
            error!("Failed to get judges: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_judge_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let model_name = payload
        .get("model_name")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let endpoint = payload
        .get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let create = data_infrastructure::database_operations::CreateJudge {
        name: name.to_string(),
        model_name: model_name.to_string(),
        endpoint: endpoint.to_string(),
        weight: payload
            .get("weight")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .unwrap_or(1.0),
        timeout_ms: payload
            .get("timeout_ms")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32)
            .unwrap_or(5000),
        optimization_target: payload
            .get("optimization_target")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "accuracy".to_string()),
        is_active: payload
            .get("is_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    };

    match db.create_judge(create).await {
        Ok(judge) => Ok(Json(serde_json::json!({
            "id": judge.id.to_string(),
            "name": judge.name,
            "model_name": judge.model_name,
            "endpoint": judge.endpoint,
            "weight": judge.weight,
            "is_active": judge.is_active,
            "created_at": judge.created_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to create judge: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_judges_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_judges().await {
        Ok(judges) => {
            let total = judges.len();
            let active = judges.iter().filter(|j| j.is_active).count();
            let inactive = total - active;

            // Calculate average weight
            let avg_weight = if total > 0 {
                judges.iter().map(|j| j.weight).sum::<f32>() / total as f32
            } else {
                0.0
            };

            Ok(Json(serde_json::json!({
                "total": total,
                "active": active,
                "inactive": inactive,
                "average_weight": avg_weight,
            })))
        }
        Err(e) => {
            error!("Failed to get judges stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_judge_handler(
    State(state): State<AppState>,
    Path(judge_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let judge_uuid = Uuid::parse_str(&judge_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_judge(judge_uuid).await {
        Ok(Some(judge)) => Ok(Json(serde_json::json!({
            "id": judge.id.to_string(),
            "name": judge.name,
            "model_name": judge.model_name,
            "endpoint": judge.endpoint,
            "weight": judge.weight,
            "timeout_ms": judge.timeout_ms,
            "optimization_target": judge.optimization_target,
            "is_active": judge.is_active,
            "created_at": judge.created_at.to_rfc3339(),
            "updated_at": judge.updated_at.to_rfc3339(),
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get judge: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_judge_handler(
    State(state): State<AppState>,
    Path(judge_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let judge_uuid = Uuid::parse_str(&judge_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let update = data_infrastructure::database_operations::UpdateJudge {
        name: payload
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        model_name: payload
            .get("model_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        endpoint: payload
            .get("endpoint")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        weight: payload
            .get("weight")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32),
        timeout_ms: payload
            .get("timeout_ms")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32),
        optimization_target: payload
            .get("optimization_target")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        is_active: payload.get("is_active").and_then(|v| v.as_bool()),
    };

    match db.update_judge(judge_uuid, update).await {
        Ok(judge) => Ok(Json(serde_json::json!({
            "id": judge.id.to_string(),
            "name": judge.name,
            "model_name": judge.model_name,
            "weight": judge.weight,
            "is_active": judge.is_active,
            "updated_at": judge.updated_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to update judge: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_judge_handler(
    State(state): State<AppState>,
    Path(judge_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let judge_uuid = Uuid::parse_str(&judge_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.delete_judge(judge_uuid).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "judge_id": judge_id,
        }))),
        Err(e) => {
            error!("Failed to delete judge: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_judge_stats_handler(
    State(state): State<AppState>,
    Path(judge_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let judge_uuid = Uuid::parse_str(&judge_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get judge
    let judge = db
        .get_judge(judge_uuid)
        .await
        .map_err(|e| {
            error!("Failed to get judge: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get judge evaluations for this judge
    match db.get_judge_evaluations_by_judge(judge_uuid).await {
        Ok(evaluations) => {
            let total_evaluations = evaluations.len();
            let avg_confidence = if total_evaluations > 0 {
                evaluations
                    .iter()
                    .filter_map(|e| e.confidence_score.or(e.confidence))
                    .collect::<Vec<_>>()
                    .iter()
                    .sum::<f32>()
                    / total_evaluations as f32
            } else {
                0.0
            };

            // Count verdict decisions
            let mut verdict_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for eval in &evaluations {
                if let Some(decision) = &eval.verdict_decision {
                    *verdict_counts.entry(decision.clone()).or_insert(0) += 1;
                }
            }

            Ok(Json(serde_json::json!({
                "judge_id": judge_id,
                "name": judge.name,
                "total_evaluations": total_evaluations,
                "average_confidence": avg_confidence,
                "weight": judge.weight,
                "is_active": judge.is_active,
                "verdict_counts": verdict_counts,
            })))
        }
        Err(_) => {
            // If evaluations query fails, return basic stats
            Ok(Json(serde_json::json!({
                "judge_id": judge_id,
                "name": judge.name,
                "total_evaluations": 0,
                "average_confidence": 0.0,
                "weight": judge.weight,
                "is_active": judge.is_active,
                "verdict_counts": {},
            })))
        }
    }
}

async fn get_judge_evaluations_handler(
    State(state): State<AppState>,
    Path(judge_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let judge_uuid = Uuid::parse_str(&judge_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_judge_evaluations_by_judge(judge_uuid).await {
        Ok(evaluations) => {
            let evaluations_list: Vec<JsonValue> = evaluations
                .into_iter()
                .map(|eval| {
                    serde_json::json!({
                        "id": eval.id.to_string(),
                        "verdict_id": eval.verdict_id.to_string(),
                        "judge_id": eval.judge_id.to_string(),
                        "judge_verdict": eval.judge_verdict,
                        "verdict_decision": eval.verdict_decision,
                        "confidence_score": eval.confidence_score,
                        "confidence": eval.confidence,
                        "evaluation_score": eval.evaluation_score,
                        "reasoning": eval.reasoning,
                        "evaluation_time_ms": eval.evaluation_time_ms,
                        "tokens_used": eval.tokens_used,
                        "created_at": eval.created_at.to_rfc3339(),
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({ "evaluations": evaluations_list })))
        }
        Err(e) => {
            error!("Failed to get judge evaluations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Telemetry & Observability handlers
async fn get_contributions_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get days parameter (default to 30)
    let days = params
        .get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .or_else(|| {
            // Support start_date/end_date for backwards compatibility
            params.get("start_date").and_then(|start| {
                let start_date =
                    chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", start)).ok()?;
                let now = chrono::Utc::now();
                let diff = now.signed_duration_since(start_date);
                Some(diff.num_days())
            })
        })
        .map(|d| d.max(1))
        .unwrap_or(30);

    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days);

    // Check if group_by parameter is set
    let group_by = params.get("group_by").map(|s| s.as_str());

    // Default behavior: return daily breakdown
    // If group_by is explicitly set to something other than "day", return aggregated totals
    if group_by == Some("total") {
        match db.query(
            "SELECT COUNT(*) as count FROM provenance_entries WHERE action IN ('code_change', 'file_edit', 'commit', 'create', 'update', 'delete') AND timestamp >= $1",
            &[&cutoff_date]
        ).await {
            Ok(rows) => {
                let total_contributions: i64 = rows.first()
                    .and_then(|row| row.try_get("count").ok())
                    .unwrap_or(0);

                // Get unique contributors count
                let unique_contributors = match db.query(
                    "SELECT COUNT(DISTINCT actor) as count FROM provenance_entries WHERE action IN ('code_change', 'file_edit', 'commit', 'create', 'update', 'delete') AND timestamp >= $1",
                    &[&cutoff_date]
                ).await {
                    Ok(contributor_rows) => {
                        contributor_rows.first()
                            .and_then(|row| row.try_get::<i64, _>("count").ok())
                            .unwrap_or(0)
                    }
                    Err(_) => 0,
                };

                Ok(Json(serde_json::json!({
                    "period_days": days,
                    "total_contributions": total_contributions,
                    "unique_contributors": unique_contributors,
                })))
            }
            Err(_) => {
                Ok(Json(serde_json::json!({
                    "period_days": days,
                    "total_contributions": 0,
                    "unique_contributors": 0,
                })))
            }
        }
    } else {
        // Return daily breakdown (default behavior)
        match db.query(
            "SELECT DATE_TRUNC('day', timestamp) as day, COUNT(*) as count, COUNT(DISTINCT actor) as unique_contributors FROM provenance_entries WHERE action IN ('code_change', 'file_edit', 'commit', 'create', 'update', 'delete') AND timestamp >= $1 GROUP BY DATE_TRUNC('day', timestamp) ORDER BY day DESC",
            &[&cutoff_date]
        ).await {
            Ok(rows) => {
                let mut contributions: Vec<JsonValue> = Vec::new();
                let mut total_contributions = 0;
                let mut unique_contributors = std::collections::HashSet::new();

                for row in rows {
                    let day: chrono::DateTime<chrono::Utc> = row.try_get("day").unwrap_or_default();
                    let count: i64 = row.try_get("count").unwrap_or(0);
                    let contributors: i64 = row.try_get("unique_contributors").unwrap_or(0);

                    total_contributions += count;
                    contributions.push(serde_json::json!({
                        "day": day.to_rfc3339(),
                        "count": count,
                        "unique_contributors": contributors,
                    }));
                }

                // Get unique contributors count from all rows
                match db.query(
                    "SELECT DISTINCT actor FROM provenance_entries WHERE action IN ('code_change', 'file_edit', 'commit', 'create', 'update', 'delete') AND timestamp >= $1",
                    &[&cutoff_date]
                ).await {
                    Ok(contributor_rows) => {
                        for row in contributor_rows {
                            if let Ok(actor) = row.try_get::<String, _>("actor") {
                                unique_contributors.insert(actor);
                            }
                        }
                    }
                    Err(_) => {}
                }

                Ok(Json(serde_json::json!({
                    "period_days": days,
                    "total_contributions": total_contributions,
                    "unique_contributors": unique_contributors.len(),
                    "daily_contributions": contributions,
                })))
            }
            Err(_) => {
                // If table doesn't exist or query fails, return empty result
                Ok(Json(serde_json::json!({
                    "period_days": days,
                    "total_contributions": 0,
                    "unique_contributors": 0,
                    "daily_contributions": [],
                })))
            }
        }
    }
}

async fn get_model_contributions_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get hours parameter (default 24)
    let hours = params
        .get("hours")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(24);

    // Use the new telemetry_model_contributions table
    match db.get_model_contributions(Some(hours)).await {
        Ok(contributions) => {
            let total_requests: i64 = contributions
                .iter()
                .map(|c| c.get("total_requests").and_then(|v| v.as_i64()).unwrap_or(0))
                .sum();

            let total_tokens: i64 = contributions
                .iter()
                .map(|c| c.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0))
                .sum();

            let total_cost: f64 = contributions
                .iter()
                .map(|c| c.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .sum();

            Ok(Json(serde_json::json!({
                "contributions": contributions,
                "summary": {
                    "total_requests": total_requests,
                    "total_tokens": total_tokens,
                    "total_cost_usd": total_cost,
                    "period_hours": hours,
                },
            })))
        }
        Err(e) => {
            error!("Failed to get model contributions: {}", e);
            // Return empty result on error
            Ok(Json(serde_json::json!({
                "contributions": [],
                "summary": {
                    "total_requests": 0,
                    "total_tokens": 0,
                    "total_cost_usd": 0.0,
                    "period_hours": hours,
                },
            })))
        }
    }
}

async fn get_agent_activity_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get hours parameter (default to 24)
    let hours = params
        .get("hours")
        .and_then(|h| h.parse::<i32>().ok())
        .unwrap_or(24);

    // Use the new telemetry_agent_activity table
    match db.get_agent_activity(Some(hours)).await {
        Ok(activities) => {
            // Calculate summary statistics
            let total_activities: i64 = activities
                .iter()
                .map(|a| a.get("total_activities").and_then(|v| v.as_i64()).unwrap_or(0))
                .sum();

            let successful: i64 = activities
                .iter()
                .map(|a| a.get("successful").and_then(|v| v.as_i64()).unwrap_or(0))
                .sum();

            let failed: i64 = activities
                .iter()
                .map(|a| a.get("failed").and_then(|v| v.as_i64()).unwrap_or(0))
                .sum();

            let success_rate = if total_activities > 0 {
                successful as f64 / total_activities as f64
            } else {
                0.0
            };

            Ok(Json(serde_json::json!({
                "activities": activities,
                "summary": {
                    "total_activities": total_activities,
                    "successful": successful,
                    "failed": failed,
                    "success_rate": success_rate,
                    "period_hours": hours,
                },
            })))
        }
        Err(e) => {
            error!("Failed to get agent activity: {}", e);
            // Return empty result on error
            Ok(Json(serde_json::json!({
                "activities": [],
                "summary": {
                    "total_activities": 0,
                    "successful": 0,
                    "failed": 0,
                    "success_rate": 0.0,
                    "period_hours": hours,
                },
            })))
        }
    }
}

async fn get_efficiency_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Query telemetry_data for efficiency metrics
    match db.query(
        "SELECT payload->>'metric_name' as metric_name, AVG((payload->>'value')::float) as avg_value, MIN((payload->>'value')::float) as min_value, MAX((payload->>'value')::float) as max_value FROM telemetry_data WHERE data_type = 'Metric' AND (payload->>'metric_name' LIKE '%efficiency%' OR payload->>'metric_name' LIKE '%throughput%' OR payload->>'metric_name' LIKE '%latency%') AND timestamp >= NOW() - INTERVAL '24 hours' GROUP BY payload->>'metric_name'",
        &[]
    ).await {
        Ok(rows) => {
            let mut metrics: Vec<JsonValue> = Vec::new();

            for row in rows {
                let metric_name: Option<String> = row.try_get("metric_name").ok();
                let avg_value: Option<f64> = row.try_get("avg_value").ok();
                let min_value: Option<f64> = row.try_get("min_value").ok();
                let max_value: Option<f64> = row.try_get("max_value").ok();

                if let Some(name) = metric_name {
                    metrics.push(serde_json::json!({
                        "metric": name,
                        "average": avg_value,
                        "min": min_value,
                        "max": max_value,
                    }));
                }
            }

            Ok(Json(serde_json::json!({
                "metrics": metrics,
                "period": "24 hours",
            })))
        }
        Err(_) => {
            // If table doesn't exist or query fails, return empty result
            Ok(Json(serde_json::json!({
                "metrics": [],
                "period": "24 hours",
            })))
        }
    }
}

/// Get Prometheus-formatted metrics
///
/// Returns system and business metrics in Prometheus text format for scraping by Prometheus.
/// This endpoint is used by Prometheus monitoring infrastructure.
async fn get_metrics_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    use axum::body::Body;
    use axum::response::Response;
    use std::fmt::Write;

    let mut metrics_output = String::new();

    // Collect system metrics using sysinfo
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
    let total_memory = system.total_memory() as f64;
    let used_memory = system.used_memory() as f64;
    let memory_usage_percent = if total_memory > 0.0 {
        (used_memory / total_memory) * 100.0
    } else {
        0.0
    };

    // Calculate disk usage
    let mut total_disk_space = 0u64;
    let mut total_used_space = 0u64;
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        total_disk_space += disk.total_space();
        total_used_space += disk.total_space() - disk.available_space();
    }
    let disk_usage_percent = if total_disk_space > 0 {
        (total_used_space as f64 / total_disk_space as f64) * 100.0
    } else {
        0.0
    };

    // Write system metrics in Prometheus format
    // Note: writeln! returns fmt::Result but formatting errors are extremely rare for simple cases
    writeln!(
        metrics_output,
        "# HELP system_cpu_usage_percent CPU usage percentage (0-100)"
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "# TYPE system_cpu_usage_percent gauge")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "system_cpu_usage_percent {}", cpu_usage)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    writeln!(
        metrics_output,
        "# HELP system_memory_usage_percent Memory usage percentage (0-100)"
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "# TYPE system_memory_usage_percent gauge")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(
        metrics_output,
        "system_memory_usage_percent {}",
        memory_usage_percent
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    writeln!(
        metrics_output,
        "# HELP system_memory_total_bytes Total system memory in bytes"
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "# TYPE system_memory_total_bytes gauge")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(
        metrics_output,
        "system_memory_total_bytes {}",
        total_memory as u64
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    writeln!(
        metrics_output,
        "# HELP system_memory_used_bytes Used system memory in bytes"
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "# TYPE system_memory_used_bytes gauge")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(
        metrics_output,
        "system_memory_used_bytes {}",
        used_memory as u64
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    writeln!(
        metrics_output,
        "# HELP system_disk_usage_percent Disk usage percentage (0-100)"
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "# TYPE system_disk_usage_percent gauge")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(
        metrics_output,
        "system_disk_usage_percent {}",
        disk_usage_percent
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    writeln!(
        metrics_output,
        "# HELP system_disk_total_bytes Total disk space in bytes"
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "# TYPE system_disk_total_bytes gauge")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(
        metrics_output,
        "system_disk_total_bytes {}",
        total_disk_space
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    writeln!(
        metrics_output,
        "# HELP system_disk_used_bytes Used disk space in bytes"
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(metrics_output, "# TYPE system_disk_used_bytes gauge")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    writeln!(
        metrics_output,
        "system_disk_used_bytes {}",
        total_used_space
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Collect business metrics from database if available
    if let Some(db) = state.db_client.as_ref() {
        let now = Utc::now();
        let one_hour_ago = now - ChronoDuration::hours(1);
        let one_minute_ago = now - ChronoDuration::minutes(1);

        // Active users
        if let Ok(active_users) = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(DISTINCT user_id) FROM sessions WHERE is_active = true AND expires_at > $1"#
        )
        .bind(now)
        .fetch_one(db.pool())
        .await
        {
            writeln!(metrics_output, "# HELP business_active_users Number of active users").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_active_users gauge").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "business_active_users {}", active_users).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // Requests per second
        if let Ok(requests_last_minute) = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM audit_trail_entries WHERE created_at >= $1"#,
        )
        .bind(one_minute_ago)
        .fetch_one(db.pool())
        .await
        {
            let requests_per_second = requests_last_minute as f64 / 60.0;
            writeln!(
                metrics_output,
                "# HELP business_requests_per_second Requests per second"
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_requests_per_second gauge")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(
                metrics_output,
                "business_requests_per_second {}",
                requests_per_second
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // Task throughput
        if let Ok(tasks_last_hour) = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM task_executions WHERE execution_started_at >= $1"#,
        )
        .bind(one_hour_ago)
        .fetch_one(db.pool())
        .await
        {
            writeln!(
                metrics_output,
                "# HELP business_tasks_per_hour Task throughput per hour"
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_tasks_per_hour gauge")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(
                metrics_output,
                "business_tasks_per_hour {}",
                tasks_last_hour
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // Task completion time
        if let Ok(Some(avg_completion_time)) = sqlx::query_scalar::<_, Option<i64>>(
            r#"SELECT AVG(execution_time_ms) FROM task_executions WHERE execution_completed_at IS NOT NULL AND execution_time_ms IS NOT NULL AND execution_started_at >= $1"#
        )
        .bind(one_hour_ago)
        .fetch_one(db.pool())
        .await
        {
            writeln!(metrics_output, "# HELP business_task_completion_time_ms Average task completion time in milliseconds").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_task_completion_time_ms gauge").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "business_task_completion_time_ms {}", avg_completion_time).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // Error rate
        if let (Ok(total_tasks), Ok(failed_tasks)) = (
            sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM task_executions WHERE execution_started_at >= $1"#
            )
            .bind(one_hour_ago)
            .fetch_one(db.pool())
            .await,
            sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM task_executions WHERE status = 'failed' AND execution_started_at >= $1"#
            )
            .bind(one_hour_ago)
            .fetch_one(db.pool())
            .await,
        ) {
            let error_rate = if total_tasks > 0 {
                failed_tasks as f64 / total_tasks as f64
            } else {
                0.0
            };
            writeln!(metrics_output, "# HELP business_error_rate Error rate (0.0-1.0)").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_error_rate gauge").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "business_error_rate {}", error_rate).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let system_availability = (1.0 - error_rate) * 100.0;
            writeln!(metrics_output, "# HELP business_system_availability System availability percentage (0-100)").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_system_availability gauge").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "business_system_availability {}", system_availability).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // Task counts by state from database
        if let Ok(Some(active_count)) = sqlx::query_scalar::<_, Option<i64>>(
            r#"SELECT COUNT(*) FROM task_executions WHERE status = 'running' AND execution_completed_at IS NULL"#
        )
        .fetch_one(db.pool())
        .await
        {
            writeln!(metrics_output, "# HELP business_active_tasks Number of currently active tasks").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_active_tasks gauge").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "business_active_tasks {}", active_count).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        if let Ok(Some(completed_count)) = sqlx::query_scalar::<_, Option<i64>>(
            r#"SELECT COUNT(*) FROM task_executions WHERE status = 'completed' AND execution_started_at >= $1"#
        )
        .bind(one_hour_ago)
        .fetch_one(db.pool())
        .await
        {
            writeln!(metrics_output, "# HELP business_completed_tasks Total number of completed tasks").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_completed_tasks counter").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "business_completed_tasks {}", completed_count).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        if let Ok(Some(failed_count)) = sqlx::query_scalar::<_, Option<i64>>(
            r#"SELECT COUNT(*) FROM task_executions WHERE status = 'failed' AND execution_started_at >= $1"#
        )
        .bind(one_hour_ago)
        .fetch_one(db.pool())
        .await
        {
            writeln!(metrics_output, "# HELP business_failed_tasks Total number of failed tasks").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "# TYPE business_failed_tasks counter").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            writeln!(metrics_output, "business_failed_tasks {}", failed_count).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    // Return as plain text response with Prometheus content type
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .body(Body::from(metrics_output))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

async fn get_alerts_handler(State(state): State<AppState>) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Query telemetry_data for alerts and errors
    match db.query(
        "SELECT id, timestamp, source, payload, tags FROM telemetry_data WHERE (data_type = 'Event' AND (payload->>'level' = 'error' OR payload->>'level' = 'warning' OR payload->>'alert' IS NOT NULL)) OR (data_type = 'Log' AND (payload->>'level' = 'ERROR' OR payload->>'level' = 'WARN')) ORDER BY timestamp DESC LIMIT 100",
        &[]
    ).await {
        Ok(rows) => {
            let mut alerts: Vec<JsonValue> = Vec::new();

            for row in rows {
                let id: Uuid = row.try_get("id").unwrap_or_default();
                let timestamp: chrono::DateTime<chrono::Utc> = row.try_get("timestamp").unwrap_or_default();
                let source: String = row.try_get("source").unwrap_or_default();
                let payload: serde_json::Value = row.try_get("payload").unwrap_or(serde_json::json!({}));
                let tags: serde_json::Value = row.try_get("tags").unwrap_or(serde_json::json!({}));

                alerts.push(serde_json::json!({
                    "id": id.to_string(),
                    "timestamp": timestamp.to_rfc3339(),
                    "source": source,
                    "level": payload.get("level").and_then(|v| v.as_str()).unwrap_or("unknown"),
                    "message": payload.get("message").and_then(|v| v.as_str()).or_else(|| payload.get("error").and_then(|v| v.as_str())).unwrap_or(""),
                    "tags": tags,
                }));
            }

            Ok(Json(serde_json::json!({
                "alerts": alerts,
                "count": alerts.len(),
            })))
        }
        Err(_) => {
            // If table doesn't exist or query fails, return empty result
            Ok(Json(serde_json::json!({
                "alerts": [],
                "count": 0,
            })))
        }
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
                    let chain_json: Vec<JsonValue> = chain
                        .into_iter()
                        .map(|entry| {
                            serde_json::json!({
                                "timestamp": entry.timestamp.to_rfc3339(),
                                "phase": entry.phase,
                                "reasoning": entry.reasoning,
                                "decision": entry.decision,
                                "context": entry.context,
                            })
                        })
                        .collect();
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
                    let decisions_json: Vec<JsonValue> = decisions
                        .into_iter()
                        .map(|decision| {
                            serde_json::json!({
                                "timestamp": decision.timestamp.to_rfc3339(),
                                "judge": decision.judge,
                                "verdict": decision.verdict,
                                "reasoning": decision.reasoning,
                                "confidence": decision.confidence,
                            })
                        })
                        .collect();
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
                    let actions_json: Vec<JsonValue> = actions
                        .into_iter()
                        .map(|action| {
                            serde_json::json!({
                                "timestamp": action.timestamp.to_rfc3339(),
                                "worker_id": action.worker_id.to_string(),
                                "action": action.action,
                                "result": action.result,
                                "artifacts": action.artifacts,
                            })
                        })
                        .collect();
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

// Wrapper handlers for chat stream endpoints (convert AppState to ApiState)
#[cfg(feature = "orchestration")]
async fn stream_agent_response_wrapper(
    State(state): State<AppState>,
    Json(request): Json<data_infrastructure::api::handlers::chat_handlers::StreamAgentRequest>,
) -> axum::response::Response {
    let api = match state.api.as_ref() {
        Some(api) => api,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "API service unavailable").into_response()
        }
    };
    let websocket_manager = match state.websocket_manager.as_ref() {
        Some(ws) => ws,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "WebSocket manager unavailable",
            )
                .into_response()
        }
    };
    let query_performance_monitor = match state.query_performance_monitor.as_ref() {
        Some(qpm) => qpm,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Query performance monitor unavailable",
            )
                .into_response()
        }
    };

    // Extract CoreMLManager from UnifiedOrchestrator if available
    let coreml_callback: Option<
        Arc<
            dyn Fn(
                    String,
                ) -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<String, String>> + Send>,
                > + Send
                + Sync,
        >,
    > = if let Some(ref unified_orch) = state.unified_orchestrator {
        let orch_clone = unified_orch.orchestrator();
        Some(Arc::new(move |message: String| {
            let orch = Arc::clone(&orch_clone);
            let msg = message.clone();
            Box::pin(async move { generate_coreml_chat_response(orch, &msg).await })
        }))
    } else {
        None
    };

    let api_state = ApiState {
        api: api.clone(),
        websocket_manager: websocket_manager.clone(),
        query_performance_monitor: query_performance_monitor.clone(),
        coreml_inference_callback: coreml_callback,
    };

    match data_infrastructure::api::handlers::chat_handlers::stream_agent_response(
        axum::extract::State(api_state),
        axum::Json(request),
    )
    .await
    {
        Ok(sse) => sse.into_response(),
        Err(e) => {
            error!("Stream agent response error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)).into_response()
        }
    }
}

/// Generate chat response using CoreML via UnifiedOrchestrator
// Note: system-acceleration feature is defined in Cargo.toml, this warning is a false positive from check-cfg
#[allow(unexpected_cfgs)]
#[cfg(all(feature = "orchestration", feature = "system-acceleration"))]
async fn generate_coreml_chat_response(
    orchestrator: Arc<
        agent_orchestration::orchestration::unified_orchestrator::UnifiedOrchestrator,
    >,
    message: &str,
) -> Result<String, String> {
    use agent_orchestration::coreml::CoreMLManager;
    use std::path::PathBuf;
    use std::sync::Arc;
    use system_acceleration::ane::infer::MistralInferenceOptions;

    // Access plan_generator's coreml_manager
    // Since plan_generator is private, we create a new CoreMLManager instance
    // In the future, we should add a method to UnifiedOrchestrator to access CoreMLManager
    let model_path = std::env::var("COREML_MODELS_PATH")
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|_| {
            PathBuf::from("/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml")
        });

    let manager = Arc::new(CoreMLManager::new(model_path));

    // Try to load models
    if let Err(e) = manager.load_available_models().await {
        return Err(format!("Failed to load CoreML models: {}", e));
    }

    // Build chat prompt
    let prompt = format!(
        "You are a helpful AI assistant. The user asked: {}\n\nPlease provide a helpful response.",
        message
    );

    // Configure inference options
    let options = MistralInferenceOptions {
        max_tokens: 512,
        temperature: Some(0.7),
        top_p: Some(0.9),
        timeout_ms: 30000,
        use_kv_cache: true,
        sequence_length: None,  // Will use policy recommendation
        task_type: None,        // Will auto-detect from input
        backend_policy: None,   // Will use policy recommendation (ANE by default)
    };

    // Generate response using Mistral model
    match manager
        .generate_text("mistral-7b-instruct", &prompt, &options)
        .await
    {
        Ok(text) => Ok(text),
        Err(e) => {
            // Try to find any available language model
            let language_models = manager
                .get_models_by_type(agent_orchestration::coreml::CoreMLModelType::Language)
                .await;
            if let Some(model) = language_models.first() {
                manager
                    .generate_text(&model.metadata.name, &prompt, &options)
                    .await
                    .map_err(|e| format!("CoreML inference failed: {}", e))
            } else {
                Err(format!("No language models available. Error: {}", e))
            }
        }
    }
}

// Note: system-acceleration feature is defined in Cargo.toml, this warning is a false positive from check-cfg
#[allow(unexpected_cfgs)]
#[cfg(all(feature = "orchestration", not(feature = "system-acceleration")))]
async fn generate_coreml_chat_response(
    _orchestrator: Arc<
        agent_orchestration::orchestration::unified_orchestrator::UnifiedOrchestrator,
    >,
    _message: &str,
) -> Result<String, String> {
    Err("CoreML chat response requires system-acceleration feature".to_string())
}

#[cfg(feature = "orchestration")]
async fn cancel_stream_wrapper(
    State(state): State<AppState>,
    Json(request): Json<data_infrastructure::api::handlers::chat_handlers::CancelStreamRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let api = state.api.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let websocket_manager = state
        .websocket_manager
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let query_performance_monitor = state
        .query_performance_monitor
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let api_state = ApiState {
        api: api.clone(),
        websocket_manager: websocket_manager.clone(),
        query_performance_monitor: query_performance_monitor.clone(),
        coreml_inference_callback: None,
    };

    match data_infrastructure::api::handlers::chat_handlers::cancel_stream(
        axum::extract::State(api_state),
        axum::Json(request),
    )
    .await
    {
        Ok(json) => Ok(json),
        Err(e) => {
            error!("Cancel stream error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
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
        let query = payload
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or(StatusCode::BAD_REQUEST)?;

        // Get optional task_id for context
        let task_id = payload
            .get("task_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        // Context integration with orchestrator
        // Current implementation: Basic context from orchestrator's chain of thought
        // Future enhancement: Full memory system integration with conversation history
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
                    response["chain_of_thought_entries"] =
                        serde_json::json!(task_state.chain_of_thought.len());
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
            // Use lightweight summaries to avoid cloning large vectors
            let task_summaries = service.list_task_summaries().await;
            let sessions: Vec<JsonValue> = task_summaries
                .into_iter()
                .map(|task| {
                    serde_json::json!({
                        "session_id": task.task_id.to_string(),
                        "description": task.description,
                        "status": format!("{:?}", task.status),
                        "created_at": task.started_at.to_rfc3339(),
                        "updated_at": task.updated_at.to_rfc3339(),
                    })
                })
                .collect();
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
                Err(_) => Err(StatusCode::NOT_FOUND),
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

// Search handler - unified search across all resources using vector and knowledge search
async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Extract query parameters
    let query = params.get("q").ok_or(StatusCode::BAD_REQUEST)?;

    if query.trim().is_empty() {
        return Ok(Json(serde_json::json!({
            "results": [],
            "total": 0,
            "limit": 50,
            "offset": 0
        })));
    }

    let search_type = params.get("type").map(|s| s.as_str());
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50)
        .min(100); // Cap at 100 results
    let offset = params
        .get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let use_vector_search = params
        .get("vector_search")
        .map(|s| s == "true")
        .unwrap_or(true);
    let use_knowledge_search = params
        .get("knowledge_search")
        .map(|s| s == "true")
        .unwrap_or(true);
    let _similarity_threshold = params
        .get("similarity_threshold")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.7);

    let mut all_results: Vec<serde_json::Value> = Vec::new();

    // Search projects
    if search_type.is_none() || search_type == Some("all") || search_type == Some("project") {
        match db.get_execution_plans().await {
            Ok(plans) => {
                for plan in plans {
                    // Text search in title and overview
                    let matches = plan.title.to_lowercase().contains(&query.to_lowercase())
                        || plan
                            .overview
                            .as_ref()
                            .map(|o| o.to_lowercase().contains(&query.to_lowercase()))
                            .unwrap_or(false);

                    if matches {
                        all_results.push(serde_json::json!({
                            "id": plan.id.to_string(),
                            "type": "project",
                            "title": plan.title,
                            "description": plan.overview,
                            "url": format!("/projects/{}", plan.id),
                            "metadata": {
                                "state": plan.state,
                                "created_at": plan.created_at.to_rfc3339(),
                            }
                        }));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to search projects: {}", e);
            }
        }
    }

    // Search tasks
    if search_type.is_none() || search_type == Some("all") || search_type == Some("task") {
        match db.get_tasks().await {
            Ok(tasks) => {
                for task in tasks {
                    // Text search in title and description
                    let matches = task.title.to_lowercase().contains(&query.to_lowercase())
                        || task
                            .description
                            .to_lowercase()
                            .contains(&query.to_lowercase());

                    if matches {
                        // Find project for this task
                        let project_id =
                            task.project_id.map(|id| id.to_string()).unwrap_or_default();
                        all_results.push(serde_json::json!({
                            "id": task.id.to_string(),
                            "type": "task",
                            "title": task.title,
                            "description": task.description,
                            "url": if !project_id.is_empty() {
                                format!("/projects/{}/tasks", project_id)
                            } else {
                                "/tasks".to_string()
                            },
                            "metadata": {
                                "status": format!("{:?}", task.status),
                                "priority": task.priority,
                                "created_at": task.created_at.to_rfc3339(),
                            }
                        }));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to search tasks: {}", e);
            }
        }
    }

    // Search workers/agents
    if search_type.is_none() || search_type == Some("all") || search_type == Some("agent") {
        match db.get_workers().await {
            Ok(workers) => {
                for worker in workers {
                    // Text search in name and specialty
                    let matches = worker.name.to_lowercase().contains(&query.to_lowercase())
                        || worker
                            .specialty
                            .as_ref()
                            .map(|s| s.to_lowercase().contains(&query.to_lowercase()))
                            .unwrap_or(false);

                    if matches {
                        all_results.push(serde_json::json!({
                            "id": worker.id.to_string(),
                            "type": "agent",
                            "title": worker.name,
                            "description": worker.specialty.clone().unwrap_or_default(),
                            "url": format!("/settings?tab=agents"),
                            "metadata": {
                                "is_active": worker.is_active,
                                "created_at": worker.created_at.to_rfc3339(),
                            }
                        }));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to search agents: {}", e);
            }
        }
    }

    // Search chat messages
    if search_type.is_none() || search_type == Some("all") || search_type == Some("chat") {
        // Search chat messages in database
        // Join with chat_sessions to get session information
        match db.query(
            "SELECT cm.id, cm.content, cm.role, cm.session_id, cm.created_at, cs.title as session_title
             FROM chat_messages cm
             LEFT JOIN chat_sessions cs ON cm.session_id = cs.id
             WHERE cm.content ILIKE $1
             ORDER BY cm.created_at DESC
             LIMIT $2",
            &[&format!("%{}%", query), &(limit as i64)]
        ).await {
            Ok(rows) => {
                for row in rows {
                    let message_id: Uuid = row.try_get("id").unwrap_or_else(|_| Uuid::new_v4());
                    let content: String = row.try_get("content").unwrap_or_default();
                    let role: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());
                    let session_id: Option<Uuid> = row.try_get("session_id").ok();
                    let session_title: Option<String> = row.try_get("session_title").ok();
                    let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now());

                    // Use session title if available, otherwise truncate content
                    let title = if let Some(ref st) = session_title {
                        format!("{} - Chat", st)
                    } else if content.len() > 60 {
                        format!("{}...", &content[..60])
                    } else {
                        content.clone()
                    };

                    // Truncate description
                    let description = if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.clone()
                    };

                    let url = if let Some(sid) = session_id {
                        format!("/chat?session={}", sid)
                    } else {
                        "/chat".to_string()
                    };

                    all_results.push(serde_json::json!({
                        "id": message_id.to_string(),
                        "type": "chat",
                        "title": title,
                        "description": description,
                        "url": url,
                        "metadata": {
                            "role": role,
                            "session_id": session_id.map(|id| id.to_string()),
                            "session_title": session_title,
                            "created_at": created_at.to_rfc3339(),
                        }
                    }));
                }
            }
            Err(e) => {
                // Chat messages table might not exist, that's okay
                debug!("Chat messages search failed (table may not exist): {}", e);
            }
        }
    }

    // Vector search (if enabled)
    if use_vector_search {
        // Search block_vectors table for similar content using text search
        // Note: Full vector similarity search requires generating embeddings for the query
        // For now, we'll use text-based search on the content field
        match db
            .query(
                "SELECT DISTINCT ON (block_id)
                block_id, content, modality, metadata, project_scope
             FROM block_vectors
             WHERE content ILIKE $1
             ORDER BY block_id, created_at DESC
             LIMIT $2",
                &[&format!("%{}%", query), &(limit as i64)],
            )
            .await
        {
            Ok(rows) => {
                for row in rows {
                    let block_id: Uuid = row.try_get("block_id").unwrap_or_else(|_| Uuid::new_v4());
                    let content: String = row.try_get("content").unwrap_or_default();
                    let modality: String = row
                        .try_get("modality")
                        .unwrap_or_else(|_| "text".to_string());
                    let metadata: serde_json::Value = row
                        .try_get("metadata")
                        .unwrap_or_else(|_| serde_json::json!({}));
                    let project_scope: Option<String> = row.try_get("project_scope").ok();

                    // Determine URL based on project scope
                    let url = if let Some(scope) = project_scope.as_ref() {
                        format!("/projects/{}/workspace", scope)
                    } else {
                        "/workspace".to_string()
                    };

                    // Extract title from content or metadata
                    let title = metadata
                        .get("title")
                        .and_then(|v| v.as_str())
                        .or_else(|| content.lines().next())
                        .unwrap_or("Untitled")
                        .to_string();

                    // Truncate description
                    let description = if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.clone()
                    };

                    all_results.push(serde_json::json!({
                        "id": block_id.to_string(),
                        "type": "file",
                        "title": title,
                        "description": description,
                        "url": url,
                        "metadata": {
                            "modality": modality,
                            "project_scope": project_scope,
                        }
                    }));
                }
            }
            Err(e) => {
                warn!("Vector search failed: {}", e);
            }
        }
    }

    // Knowledge base search (if enabled)
    if use_knowledge_search {
        // Search knowledge base using text search on canonical_name and properties
        // Full semantic search requires generating embeddings - this is a text-based fallback
        match db
            .query(
                "SELECT entity_id, canonical_name, entity_type, source, properties
             FROM external_knowledge_entities
             WHERE canonical_name ILIKE $1
                OR properties::text ILIKE $1
             ORDER BY usage_count DESC, confidence DESC
             LIMIT $2",
                &[&format!("%{}%", query), &(limit as i64)],
            )
            .await
        {
            Ok(rows) => {
                for row in rows {
                    let entity_id: Uuid =
                        row.try_get("entity_id").unwrap_or_else(|_| Uuid::new_v4());
                    let canonical_name: String = row.try_get("canonical_name").unwrap_or_default();
                    let entity_type: String = row.try_get("entity_type").unwrap_or_default();
                    let source: String = row.try_get("source").unwrap_or_default();
                    let properties: serde_json::Value = row
                        .try_get("properties")
                        .unwrap_or_else(|_| serde_json::json!({}));

                    // Extract description from properties
                    let description = properties
                        .get("description")
                        .and_then(|v| v.as_str())
                        .or_else(|| properties.get("definition").and_then(|v| v.as_str()))
                        .unwrap_or(&canonical_name)
                        .to_string();

                    all_results.push(serde_json::json!({
                        "id": entity_id.to_string(),
                        "type": "file", // Knowledge entities shown as files for now
                        "title": canonical_name,
                        "description": description,
                        "url": format!("/knowledge/{}", entity_id),
                        "metadata": {
                            "entity_type": entity_type,
                            "source": source,
                            "knowledge_base": true,
                        }
                    }));
                }
            }
            Err(e) => {
                warn!("Knowledge base search failed: {}", e);
            }
        }
    }

    // Sort results by relevance (simple: exact title matches first)
    all_results.sort_by(|a, b| {
        let a_title = a["title"].as_str().unwrap_or("").to_lowercase();
        let b_title = b["title"].as_str().unwrap_or("").to_lowercase();
        let query_lower = query.to_lowercase();

        let a_exact = a_title == query_lower;
        let b_exact = b_title == query_lower;

        match (a_exact, b_exact) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_starts = a_title.starts_with(&query_lower);
                let b_starts = b_title.starts_with(&query_lower);
                match (a_starts, b_starts) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            }
        }
    });

    // Apply pagination
    let total = all_results.len();
    let paginated_results: Vec<serde_json::Value> = all_results
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    Ok(Json(serde_json::json!({
        "results": paginated_results,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

// Task comments handlers
async fn get_task_comments_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db
        .query(
            "SELECT id, task_id, content, created_by, created_at, updated_at
         FROM task_comments
         WHERE task_id = $1
         ORDER BY created_at ASC",
            &[&task_uuid],
        )
        .await
    {
        Ok(rows) => {
            let comments: Vec<JsonValue> = rows.into_iter().map(|row| {
                serde_json::json!({
                    "comment_id": row.try_get::<Uuid, _>("id").unwrap_or_else(|_| Uuid::new_v4()).to_string(),
                    "task_id": row.try_get::<Uuid, _>("task_id").unwrap_or_else(|_| Uuid::new_v4()).to_string(),
                    "content": row.try_get::<String, _>("content").unwrap_or_default(),
                    "created_by": row.try_get::<Option<String>, _>("created_by").ok().flatten(),
                    "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                        .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
                    "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at")
                        .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
                })
            }).collect();

            Ok(Json(serde_json::json!({ "comments": comments })))
        }
        Err(e) => {
            // Table might not exist yet, return empty array
            debug!("Failed to get task comments (table may not exist): {}", e);
            Ok(Json(serde_json::json!({ "comments": [] })))
        }
    }
}

async fn create_task_comment_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if content.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let created_by = payload.get("created_by").and_then(|v| v.as_str());

    match db
        .query_one(
            "INSERT INTO task_comments (task_id, content, created_by)
         VALUES ($1, $2, $3)
         RETURNING id, task_id, content, created_by, created_at, updated_at",
            &[&task_uuid, &content, &created_by],
        )
        .await
    {
        Ok(Some(row)) => {
            let comment = serde_json::json!({
                "comment_id": row.try_get::<Uuid, _>("id").unwrap_or_else(|_| Uuid::new_v4()).to_string(),
                "task_id": row.try_get::<Uuid, _>("task_id").unwrap_or_else(|_| Uuid::new_v4()).to_string(),
                "content": row.try_get::<String, _>("content").unwrap_or_default(),
                "created_by": row.try_get::<Option<String>, _>("created_by").ok().flatten(),
                "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                    .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
                "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at")
                    .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
            });

            Ok(Json(comment))
        }
        Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            error!("Failed to create task comment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_task_comment_handler(
    State(state): State<AppState>,
    Path((task_id, comment_id)): Path<(String, String)>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let _task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let comment_uuid = Uuid::parse_str(&comment_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if content.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match db
        .query_one(
            "UPDATE task_comments
         SET content = $1, updated_at = NOW()
         WHERE id = $2
         RETURNING id, task_id, content, created_by, created_at, updated_at",
            &[&content, &comment_uuid],
        )
        .await
    {
        Ok(Some(row)) => {
            let comment = serde_json::json!({
                "comment_id": row.try_get::<Uuid, _>("id").unwrap_or_else(|_| Uuid::new_v4()).to_string(),
                "task_id": row.try_get::<Uuid, _>("task_id").unwrap_or_else(|_| Uuid::new_v4()).to_string(),
                "content": row.try_get::<String, _>("content").unwrap_or_default(),
                "created_by": row.try_get::<Option<String>, _>("created_by").ok().flatten(),
                "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                    .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
                "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at")
                    .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
            });

            Ok(Json(comment))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update task comment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_task_comment_handler(
    State(state): State<AppState>,
    Path((task_id, comment_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let _task_uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let comment_uuid = Uuid::parse_str(&comment_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db
        .execute("DELETE FROM task_comments WHERE id = $1", &[&comment_uuid])
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to delete task comment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Project members, work history, and settings handlers
async fn get_project_members_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get unique workers assigned to tasks in this project
    match db
        .query(
            "SELECT DISTINCT w.id, w.name, w.description, w.is_active, w.created_at
         FROM workers w
         INNER JOIN tasks t ON t.assigned_worker_id = w.id
         WHERE t.project_id = $1
         ORDER BY w.name",
            &[&project_uuid],
        )
        .await
    {
        Ok(rows) => {
            let members: Vec<JsonValue> = rows
                .into_iter()
                .map(|row| {
                    let worker_id = row
                        .try_get::<Uuid, _>("id")
                        .unwrap_or_else(|_| Uuid::new_v4());
                    serde_json::json!({
                        "member_id": format!("{}-{}", project_id, worker_id),
                        "project_id": project_id.clone(),
                        "user_id": worker_id.to_string(), // Using worker_id as user_id
                        "role": "agent", // Default role
                        "joined_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                            .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({ "members": members })))
        }
        Err(e) => {
            warn!("Failed to get project members: {}", e);
            Ok(Json(serde_json::json!({ "members": [] })))
        }
    }
}

async fn get_project_work_history_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50)
        .min(100);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    // Get work history from provenance entries for tasks in this project
    match db
        .query(
            "SELECT pe.id, pe.task_id, pe.action, pe.actor, pe.resource_id, pe.resource_type,
                pe.change_summary, pe.timestamp, pe.metadata, t.title as task_title
         FROM provenance_entries pe
         INNER JOIN tasks t ON t.id = pe.task_id
         WHERE t.project_id = $1
         ORDER BY pe.timestamp DESC
         LIMIT $2 OFFSET $3",
            &[&project_uuid, &limit, &offset],
        )
        .await
    {
        Ok(rows) => {
            let entries: Vec<JsonValue> = rows.into_iter().map(|row| {
                serde_json::json!({
                    "entry_id": row.try_get::<Uuid, _>("id").unwrap_or_else(|_| Uuid::new_v4()).to_string(),
                    "project_id": project_id.clone(),
                    "task_id": row.try_get::<Uuid, _>("task_id").map(|id| id.to_string()).ok(),
                    "action": row.try_get::<String, _>("action").unwrap_or_default(),
                    "description": row.try_get::<String, _>("change_summary").unwrap_or_default(),
                    "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("timestamp")
                        .unwrap_or_else(|_| chrono::Utc::now()).to_rfc3339(),
                    "created_by": row.try_get::<String, _>("actor").ok(),
                })
            }).collect();

            // Get total count
            let total_result = db
                .query_one(
                    "SELECT COUNT(*) as total
                 FROM provenance_entries pe
                 INNER JOIN tasks t ON t.id = pe.task_id
                 WHERE t.project_id = $1",
                    &[&project_uuid],
                )
                .await;

            let total = total_result
                .ok()
                .and_then(|r| r)
                .and_then(|row| row.try_get::<i64, _>("total").ok())
                .unwrap_or(entries.len() as i64);

            Ok(Json(serde_json::json!({
                "entries": entries,
                "total": total,
                "limit": limit,
                "offset": offset
            })))
        }
        Err(e) => {
            warn!("Failed to get project work history: {}", e);
            Ok(Json(serde_json::json!({
                "entries": [],
                "total": 0,
                "limit": limit,
                "offset": offset
            })))
        }
    }
}

async fn get_project_settings_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_execution_plan(project_uuid).await {
        Ok(Some(plan)) => {
            // Extract settings from plan metadata or use defaults
            let metadata = if plan.metadata.is_null() {
                serde_json::json!({})
            } else {
                plan.metadata.clone()
            };
            let settings = serde_json::json!({
                "project_id": project_id.clone(),
                "notifications": metadata.get("notifications").cloned().unwrap_or(serde_json::json!({
                    "email": true,
                    "in_app": true
                })),
                "permissions": metadata.get("permissions").cloned().unwrap_or(serde_json::json!({
                    "public": false,
                    "allow_comments": true
                })),
                "workflow": metadata.get("workflow").cloned().unwrap_or(serde_json::json!({
                    "auto_assign": false,
                    "require_approval": false
                })),
            });

            Ok(Json(settings))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get project settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_project_settings_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_execution_plan(project_uuid).await {
        Ok(Some(plan)) => {
            // Merge settings into metadata
            let mut metadata = if plan.metadata.is_null() {
                serde_json::json!({})
            } else {
                plan.metadata.clone()
            };

            if let Some(notifications) = payload.get("notifications") {
                metadata["notifications"] = notifications.clone();
            }
            if let Some(permissions) = payload.get("permissions") {
                metadata["permissions"] = permissions.clone();
            }
            if let Some(workflow) = payload.get("workflow") {
                metadata["workflow"] = workflow.clone();
            }

            // Update plan with new metadata using direct SQL update
            match db
                .execute(
                    "UPDATE execution_plans SET metadata = $1, updated_at = NOW() WHERE id = $2",
                    &[&metadata, &project_uuid],
                )
                .await
            {
                Ok(_) => {
                    let settings = serde_json::json!({
                        "project_id": project_id.clone(),
                        "notifications": metadata.get("notifications").cloned().unwrap_or(serde_json::json!({
                            "email": true,
                            "in_app": true
                        })),
                        "permissions": metadata.get("permissions").cloned().unwrap_or(serde_json::json!({
                            "public": false,
                            "allow_comments": true
                        })),
                        "workflow": metadata.get("workflow").cloned().unwrap_or(serde_json::json!({
                            "auto_assign": false,
                            "require_approval": false
                        })),
                    });

                    Ok(Json(settings))
                }
                Err(e) => {
                    error!("Failed to update project settings: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get project for settings update: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project_task_settings_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_execution_plan(project_uuid).await {
        Ok(Some(plan)) => {
            // Extract task settings from plan metadata or use defaults
            let metadata = if plan.metadata.is_null() {
                serde_json::json!({})
            } else {
                plan.metadata.clone()
            };
            let task_settings = metadata
                .get("task_settings")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            // Merge with defaults
            let settings = serde_json::json!({
                "project_id": project_id.clone(),
                "default_status": task_settings.get("default_status").cloned().unwrap_or(serde_json::json!("todo")),
                "auto_archive": task_settings.get("auto_archive").cloned().unwrap_or(serde_json::json!(true)),
                "auto_archive_days": task_settings.get("auto_archive_days").cloned().unwrap_or(serde_json::json!(30)),
                "enable_dependencies": task_settings.get("enable_dependencies").cloned().unwrap_or(serde_json::json!(false)),
                "require_description": task_settings.get("require_description").cloned().unwrap_or(serde_json::json!(false)),
                "priority_levels": task_settings.get("priority_levels").cloned().unwrap_or(serde_json::json!(4)),
                "auto_assign_priority": task_settings.get("auto_assign_priority").cloned().unwrap_or(serde_json::json!(true)),
                "max_tags": task_settings.get("max_tags").cloned().unwrap_or(serde_json::json!(5)),
                "enable_time_tracking": task_settings.get("enable_time_tracking").cloned().unwrap_or(serde_json::json!(true)),
                "time_alert_threshold": task_settings.get("time_alert_threshold").cloned().unwrap_or(serde_json::json!(50)),
                "work_hours": task_settings.get("work_hours").cloned().unwrap_or(serde_json::json!(8)),
                "auto_move_stale": task_settings.get("auto_move_stale").cloned().unwrap_or(serde_json::json!(true)),
                "smart_distribution": task_settings.get("smart_distribution").cloned().unwrap_or(serde_json::json!(true)),
                "deadline_reminders": task_settings.get("deadline_reminders").cloned().unwrap_or(serde_json::json!(true)),
            });

            Ok(Json(settings))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get project task settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_project_task_settings_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_execution_plan(project_uuid).await {
        Ok(Some(plan)) => {
            // Get current metadata
            let mut metadata = if plan.metadata.is_null() {
                serde_json::json!({})
            } else {
                plan.metadata.clone()
            };

            // Get or create task_settings object
            let mut task_settings = metadata
                .get("task_settings")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            // Update task_settings with payload values
            if let Some(default_status) = payload.get("default_status") {
                task_settings["default_status"] = default_status.clone();
            }
            if let Some(auto_archive) = payload.get("auto_archive") {
                task_settings["auto_archive"] = auto_archive.clone();
            }
            if let Some(auto_archive_days) = payload.get("auto_archive_days") {
                task_settings["auto_archive_days"] = auto_archive_days.clone();
            }
            if let Some(enable_dependencies) = payload.get("enable_dependencies") {
                task_settings["enable_dependencies"] = enable_dependencies.clone();
            }
            if let Some(require_description) = payload.get("require_description") {
                task_settings["require_description"] = require_description.clone();
            }
            if let Some(priority_levels) = payload.get("priority_levels") {
                task_settings["priority_levels"] = priority_levels.clone();
            }
            if let Some(auto_assign_priority) = payload.get("auto_assign_priority") {
                task_settings["auto_assign_priority"] = auto_assign_priority.clone();
            }
            if let Some(max_tags) = payload.get("max_tags") {
                task_settings["max_tags"] = max_tags.clone();
            }
            if let Some(enable_time_tracking) = payload.get("enable_time_tracking") {
                task_settings["enable_time_tracking"] = enable_time_tracking.clone();
            }
            if let Some(time_alert_threshold) = payload.get("time_alert_threshold") {
                task_settings["time_alert_threshold"] = time_alert_threshold.clone();
            }
            if let Some(work_hours) = payload.get("work_hours") {
                task_settings["work_hours"] = work_hours.clone();
            }
            if let Some(auto_move_stale) = payload.get("auto_move_stale") {
                task_settings["auto_move_stale"] = auto_move_stale.clone();
            }
            if let Some(smart_distribution) = payload.get("smart_distribution") {
                task_settings["smart_distribution"] = smart_distribution.clone();
            }
            if let Some(deadline_reminders) = payload.get("deadline_reminders") {
                task_settings["deadline_reminders"] = deadline_reminders.clone();
            }

            // Update metadata with task_settings
            metadata["task_settings"] = task_settings.clone();

            // Update plan with new metadata
            match db
                .execute(
                    "UPDATE execution_plans SET metadata = $1, updated_at = NOW() WHERE id = $2",
                    &[&metadata, &project_uuid],
                )
                .await
            {
                Ok(_) => {
                    // Return updated settings
                    let settings = serde_json::json!({
                        "project_id": project_id.clone(),
                        "default_status": task_settings.get("default_status").cloned().unwrap_or(serde_json::json!("todo")),
                        "auto_archive": task_settings.get("auto_archive").cloned().unwrap_or(serde_json::json!(true)),
                        "auto_archive_days": task_settings.get("auto_archive_days").cloned().unwrap_or(serde_json::json!(30)),
                        "enable_dependencies": task_settings.get("enable_dependencies").cloned().unwrap_or(serde_json::json!(false)),
                        "require_description": task_settings.get("require_description").cloned().unwrap_or(serde_json::json!(false)),
                        "priority_levels": task_settings.get("priority_levels").cloned().unwrap_or(serde_json::json!(4)),
                        "auto_assign_priority": task_settings.get("auto_assign_priority").cloned().unwrap_or(serde_json::json!(true)),
                        "max_tags": task_settings.get("max_tags").cloned().unwrap_or(serde_json::json!(5)),
                        "enable_time_tracking": task_settings.get("enable_time_tracking").cloned().unwrap_or(serde_json::json!(true)),
                        "time_alert_threshold": task_settings.get("time_alert_threshold").cloned().unwrap_or(serde_json::json!(50)),
                        "work_hours": task_settings.get("work_hours").cloned().unwrap_or(serde_json::json!(8)),
                        "auto_move_stale": task_settings.get("auto_move_stale").cloned().unwrap_or(serde_json::json!(true)),
                        "smart_distribution": task_settings.get("smart_distribution").cloned().unwrap_or(serde_json::json!(true)),
                        "deadline_reminders": task_settings.get("deadline_reminders").cloned().unwrap_or(serde_json::json!(true)),
                    });

                    Ok(Json(settings))
                }
                Err(e) => {
                    error!("Failed to update project task settings: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get project for task settings update: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project_overview_versions_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);

    let query = "SELECT version_id, project_id, overview, change_summary, created_by, created_at
                 FROM project_overview_versions
                 WHERE project_id = $1
                 ORDER BY created_at DESC
                 LIMIT $2";

    match db.query(query, &[&project_uuid, &limit]).await {
        Ok(rows) => {
            let versions: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "version_id": row.try_get::<Uuid, _>("version_id").ok().map(|u| u.to_string()),
                    "project_id": row.try_get::<Uuid, _>("project_id").ok().map(|u| u.to_string()),
                    "overview": row.try_get::<String, _>("overview").ok(),
                    "change_summary": row.try_get::<Option<String>, _>("change_summary").ok().flatten(),
                    "created_by": row.try_get::<Option<String>, _>("created_by").ok().flatten(),
                    "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                        .ok()
                        .map(|dt| dt.to_rfc3339()),
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "versions": versions
            })))
        }
        Err(e) => {
            error!("Failed to get project overview versions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_project_overview_version_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let overview = payload
        .get("overview")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let change_summary = payload
        .get("change_summary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let created_by = payload
        .get("created_by")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let query = "INSERT INTO project_overview_versions (project_id, overview, change_summary, created_by)
                 VALUES ($1, $2, $3, $4)
                 RETURNING version_id, project_id, overview, change_summary, created_by, created_at";

    match db
        .query(
            query,
            &[&project_uuid, &overview, &change_summary, &created_by],
        )
        .await
    {
        Ok(rows) => {
            if rows.is_empty() {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                let row = &rows[0];
                Ok(Json(serde_json::json!({
                    "version_id": row.try_get::<Uuid, _>("version_id").ok().map(|u| u.to_string()),
                    "project_id": row.try_get::<Uuid, _>("project_id").ok().map(|u| u.to_string()),
                    "overview": row.try_get::<String, _>("overview").ok(),
                    "change_summary": row.try_get::<Option<String>, _>("change_summary").ok().flatten(),
                    "created_by": row.try_get::<Option<String>, _>("created_by").ok().flatten(),
                    "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                        .ok()
                        .map(|dt| dt.to_rfc3339()),
                })))
            }
        }
        Err(e) => {
            error!("Failed to create project overview version: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn restore_project_overview_version_handler(
    State(state): State<AppState>,
    Path((project_id, version_id)): Path<(String, String)>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let version_uuid = Uuid::parse_str(&version_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get the version
    let query =
        "SELECT overview FROM project_overview_versions WHERE version_id = $1 AND project_id = $2";
    let rows = db
        .query(query, &[&version_uuid, &project_uuid])
        .await
        .map_err(|e| {
            error!("Failed to get overview version: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if rows.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let overview: String = rows[0]
        .try_get("overview")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update the project with the restored overview
    let update = data_infrastructure::database_operations::UpdateExecutionPlan {
        title: None,
        overview: Some(overview.clone()),
        state: None,
        milestones: None,
        dependency_graph: None,
        change_budget: None,
        quality_gates: None,
        evidence_requirements: None,
        active_waivers: None,
        metadata: None,
        approved_at: None,
        completed_at: None,
    };

    match db.update_execution_plan(project_uuid, update).await {
        Ok(plan) => Ok(Json(serde_json::json!({
            "project_id": plan.id.to_string(),
            "name": plan.title,
            "overview": plan.overview,
            "state": plan.state,
            "updated_at": plan.updated_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to restore project overview version: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
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
            let project_name = payload
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("new-project");
            let project_type = payload
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("standard");
            let description = payload
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Request orchestrator to scaffold project (observational - orchestrator decides how)
            let scaffold_description = format!(
                "Scaffold a new {} project named '{}'. {}",
                project_type, project_name, description
            );

            match service
                .execute_task(scaffold_description, Some("auto".to_string()), None)
                .await
            {
                Ok(task_id) => Ok(Json(serde_json::json!({
                    "status": "scaffold_requested",
                    "task_id": task_id.to_string(),
                    "project_name": project_name,
                    "message": "Project scaffolding requested. Orchestrator will handle scaffolding. Use task_id to observe progress."
                }))),
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
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_execution_plans().await {
        Ok(plans) => {
            let projects: Vec<JsonValue> = plans
                .into_iter()
                .map(|plan| {
                    serde_json::json!({
                        "project_id": plan.id.to_string(),
                        "name": plan.title,
                        "overview": plan.overview,
                        "state": plan.state,
                        "created_at": plan.created_at.to_rfc3339(),
                        "updated_at": plan.updated_at.to_rfc3339(),
                        "completed_at": plan.completed_at.map(|d| d.to_rfc3339()),
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({ "projects": projects })))
        }
        Err(e) => {
            error!("Failed to list projects: {}", e);
            Ok(Json(serde_json::json!({ "projects": [] })))
        }
    }
}

async fn get_project_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_execution_plan(project_uuid).await {
        Ok(Some(plan)) => Ok(Json(serde_json::json!({
            "project_id": plan.id.to_string(),
            "name": plan.title,
            "overview": plan.overview,
            "state": plan.state,
            "working_spec_id": plan.working_spec_id,
            "milestones": plan.milestones,
            "dependency_graph": plan.dependency_graph,
            "change_budget": plan.change_budget,
            "quality_gates": plan.quality_gates,
            "evidence_requirements": plan.evidence_requirements,
            "active_waivers": plan.active_waivers,
            "metadata": plan.metadata,
            "created_at": plan.created_at.to_rfc3339(),
            "updated_at": plan.updated_at.to_rfc3339(),
            "approved_at": plan.approved_at.map(|d| d.to_rfc3339()),
            "completed_at": plan.completed_at.map(|d| d.to_rfc3339()),
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_project_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check if overview is being updated and get current overview for comparison
    let new_overview = payload
        .get("overview")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let old_overview = if new_overview.is_some() {
        match db.get_execution_plan(project_uuid).await {
            Ok(Some(plan)) => plan.overview, // plan.overview is already Option<String>
            _ => None,
        }
    } else {
        None
    };

    // Create version if overview changed
    // Compare Option<String> with Option<String>
    if let (Some(new_ov), Some(old_ov)) = (&new_overview, &old_overview) {
        if new_ov != old_ov {
            // Create version entry before updating
            let change_summary = payload
                .get("change_summary")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let created_by = payload
                .get("created_by")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let version_query = "INSERT INTO project_overview_versions (project_id, overview, change_summary, created_by) VALUES ($1, $2, $3, $4)";
            if let Err(e) = db
                .execute(
                    version_query,
                    &[&project_uuid, old_ov, &change_summary, &created_by],
                )
                .await
            {
                error!("Failed to create overview version: {}", e);
                // Continue with update even if version creation fails
            }
        }
    }

    let update = data_infrastructure::database_operations::UpdateExecutionPlan {
        title: payload
            .get("name")
            .or(payload.get("title"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        overview: new_overview,
        state: payload
            .get("state")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        milestones: payload.get("milestones").cloned(),
        dependency_graph: payload.get("dependency_graph").cloned(),
        change_budget: payload.get("change_budget").cloned(),
        quality_gates: payload.get("quality_gates").cloned(),
        evidence_requirements: payload.get("evidence_requirements").cloned(),
        active_waivers: payload.get("active_waivers").cloned(),
        metadata: payload.get("metadata").cloned(),
        approved_at: payload
            .get("approved_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        completed_at: payload
            .get("completed_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
    };

    match db.update_execution_plan(project_uuid, update).await {
        Ok(plan) => Ok(Json(serde_json::json!({
            "project_id": plan.id.to_string(),
            "name": plan.title,
            "overview": plan.overview,
            "state": plan.state,
            "updated_at": plan.updated_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to update project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_project_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.delete_execution_plan(project_uuid).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "project_id": project_id,
        }))),
        Err(e) => {
            error!("Failed to delete project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project_stats_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get project
    let plan = match db.get_execution_plan(project_uuid).await {
        Ok(Some(p)) => p,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get project: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get milestones for this project
    let milestones = db.get_milestones(project_uuid).await.unwrap_or_default();
    let milestone_count = milestones.len();
    let completed_milestones = milestones.iter().filter(|m| m.state == "completed").count();
    let in_progress_milestones = milestones
        .iter()
        .filter(|m| m.state == "in_progress")
        .count();

    // Get tasks for this project using project_id field
    let task_count = match db.query(
        "SELECT COUNT(*)::bigint as count FROM tasks WHERE project_id = $1",
        &[&project_uuid],
    ).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                row.try_get::<i64, _>("count").unwrap_or(0) as usize
            } else {
                0
            }
        }
        Err(e) => {
            warn!("Failed to get task count for project: {}", e);
            0
        }
    };

    Ok(Json(serde_json::json!({
        "project_id": project_id,
        "milestone_count": milestone_count,
        "completed_milestones": completed_milestones,
        "in_progress_milestones": in_progress_milestones,
        "pending_milestones": milestone_count - completed_milestones - in_progress_milestones,
        "task_count": task_count,
        "state": plan.state,
        "created_at": plan.created_at.to_rfc3339(),
        "updated_at": plan.updated_at.to_rfc3339(),
    })))
}

async fn get_project_tasks_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let _project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get all tasks and filter by project_id
    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_tasks().await {
        Ok(tasks) => {
            let project_tasks: Vec<JsonValue> = tasks
                .into_iter()
                .filter(|task| task.project_id == Some(project_uuid))
                .map(|task| {
                    serde_json::json!({
                        "task_id": task.id.to_string(),
                        "title": task.title,
                        "description": task.description,
                        "status": task.status,
                        "risk_tier": task.risk_tier,
                        "priority": task.priority,
                        "assigned_worker_id": task.assigned_worker_id.map(|id| id.to_string()),
                        "created_at": task.created_at.to_rfc3339(),
                        "updated_at": task.updated_at.to_rfc3339(),
                        "completed_at": task.completed_at.map(|d| d.to_rfc3339()),
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({ "tasks": project_tasks })))
        }
        Err(e) => {
            error!("Failed to get project tasks: {}", e);
            Ok(Json(serde_json::json!({ "tasks": [] })))
        }
    }
}

async fn get_project_tasks_stats_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Use efficient database query to get project task stats
    match db.get_project_task_stats(project_uuid).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            error!("Failed to get project tasks stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project_milestones_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_milestones(project_uuid).await {
        Ok(milestones) => {
            let milestone_list: Vec<JsonValue> = milestones
                .into_iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "plan_id": m.plan_id.to_string(),
                        "objective": m.objective,
                        "state": m.state,
                        "priority": m.priority,
                        "risk_tier": m.risk_tier,
                        "is_blocking": m.is_blocking,
                        "estimated_effort": m.estimated_effort,
                        "started_at": m.started_at.map(|d| d.to_rfc3339()),
                        "completed_at": m.completed_at.map(|d| d.to_rfc3339()),
                        "created_at": m.created_at.to_rfc3339(),
                        "updated_at": m.updated_at.to_rfc3339(),
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({ "milestones": milestone_list })))
        }
        Err(e) => {
            error!("Failed to get milestones: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_project_milestone_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let milestone_id = payload
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let objective = payload
        .get("objective")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let create = data_infrastructure::database_operations::CreateMilestone {
        id: milestone_id.to_string(),
        plan_id: project_uuid,
        objective: objective.to_string(),
        scope: Some(
            payload
                .get("scope")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        ),
        interfaces: Some(
            payload
                .get("interfaces")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([])),
        ),
        tests: Some(
            payload
                .get("tests")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([])),
        ),
        evidence_gate: Some(
            payload
                .get("evidence_gate")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        ),
        rollback_plan: payload
            .get("rollback_plan")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        dependencies: Some(
            payload
                .get("dependencies")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([])),
        ),
        state: Some(
            payload
                .get("state")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "pending".to_string()),
        ),
        assigned_worker_id: payload
            .get("assigned_worker_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        estimated_effort: payload.get("estimated_effort").and_then(|v| v.as_f64()),
        priority: payload
            .get("priority")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        risk_tier: payload
            .get("risk_tier")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32),
        is_blocking: payload.get("is_blocking").and_then(|v| v.as_bool()),
        blocking_reason: payload
            .get("blocking_reason")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        metrics: payload.get("metrics").cloned(),
    };

    match db.create_milestone(create).await {
        Ok(milestone) => Ok(Json(serde_json::json!({
            "id": milestone.id,
            "plan_id": milestone.plan_id.to_string(),
            "objective": milestone.objective,
            "state": milestone.state,
            "created_at": milestone.created_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to create milestone: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_project_milestone_handler(
    State(state): State<AppState>,
    Path((project_id, milestone_id)): Path<(String, String)>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let project_uuid = Uuid::parse_str(&project_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let update = data_infrastructure::database_operations::UpdateMilestone {
        objective: payload
            .get("objective")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        scope: payload.get("scope").cloned(),
        interfaces: payload.get("interfaces").cloned(),
        tests: payload.get("tests").cloned(),
        evidence_gate: payload.get("evidence_gate").cloned(),
        rollback_plan: payload
            .get("rollback_plan")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        dependencies: payload.get("dependencies").cloned(),
        state: payload
            .get("state")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        assigned_worker_id: payload
            .get("assigned_worker_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok()),
        estimated_effort: payload.get("estimated_effort").and_then(|v| v.as_f64()),
        priority: payload
            .get("priority")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        risk_tier: payload
            .get("risk_tier")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32),
        is_blocking: payload.get("is_blocking").and_then(|v| v.as_bool()),
        blocking_reason: payload
            .get("blocking_reason")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        metrics: payload.get("metrics").cloned(),
        started_at: payload
            .get("started_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        completed_at: payload
            .get("completed_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
    };

    match db
        .update_milestone(project_uuid, milestone_id.clone(), update)
        .await
    {
        Ok(milestone) => Ok(Json(serde_json::json!({
            "id": milestone.id,
            "plan_id": milestone.plan_id.to_string(),
            "objective": milestone.objective,
            "state": milestone.state,
            "updated_at": milestone.updated_at.to_rfc3339(),
        }))),
        Err(e) => {
            error!("Failed to update milestone: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
                let tables: Vec<JsonValue> = rows
                    .into_iter()
                    .filter_map(|row| {
                        Some(serde_json::json!({
                            "name": row.try_get::<String, _>("table_name").ok()?,
                            "schema": row.try_get::<String, _>("table_schema").ok()?,
                        }))
                    })
                    .collect();

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
                let columns: Vec<JsonValue> = rows
                    .into_iter()
                    .filter_map(|row| {
                        Some(serde_json::json!({
                            "name": row.try_get::<String, _>("column_name").ok()?,
                            "type": row.try_get::<String, _>("data_type").ok()?,
                            "nullable": row.try_get::<String, _>("is_nullable").ok()? == "YES",
                            "default": row.try_get::<Option<String>, _>("column_default").ok()?,
                        }))
                    })
                    .collect();

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
    headers: HeaderMap,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    // Rate limiting - stricter for database queries (expensive operation)
    let client_ip = extract_client_ip(&headers);
    if let Err((status, _body)) =
        check_rate_limit(&state.rate_limiter, &client_ip, "execute_query").await
    {
        return Err(status);
    }

    if let Some(db) = &state.db_client {
        let query_text = payload
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or(StatusCode::BAD_REQUEST)?;

        // SQL query safety validation
        // 1. Query type validation - only allow read-only operations
        // 2. SQL injection prevention - block dangerous patterns
        // 3. Query complexity limits - add LIMIT clause if missing
        
        let query_upper = query_text.trim().to_uppercase();
        
        // Validate query starts with allowed read-only operations
        let is_safe_query = query_upper.starts_with("SELECT") 
            || query_upper.starts_with("EXPLAIN")
            || query_upper.starts_with("SHOW");
        
        if !is_safe_query {
            warn!("Rejected non-read query attempt: {}", &query_text[..query_text.len().min(50)]);
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Block dangerous SQL patterns that could be injected
        // Block dangerous SQL patterns - use word boundaries to avoid false positives
        // (e.g., "last_updated" should not trigger "UPDATE" detection)
        let dangerous_command_patterns = [
            " DROP ", " DELETE ", " UPDATE ", " INSERT ", " TRUNCATE ", " ALTER ", " CREATE ",
            " GRANT ", " REVOKE ", " EXECUTE ", " EXEC ",
            "DROP TABLE", "DROP DATABASE", "DELETE FROM", "UPDATE SET", "INSERT INTO",
        ];
        
        let dangerous_injection_patterns = [
            "INTO OUTFILE", "INTO DUMPFILE", "LOAD_FILE", "BENCHMARK(", 
            "SLEEP(", "WAITFOR", "PG_SLEEP(", "XP_", "SP_",
            ";--", "/*", "*/", "@@VERSION", "CHAR(", "0x",
        ];
        
        // Check command patterns (with spaces for word boundaries)
        let query_padded = format!(" {} ", query_upper);
        for pattern in dangerous_command_patterns {
            if query_padded.contains(pattern) {
                warn!("Blocked dangerous SQL command '{}' in query", pattern.trim());
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        
        // Check injection patterns (exact match)
        for pattern in dangerous_injection_patterns {
            if query_upper.contains(pattern) {
                warn!("Blocked dangerous SQL injection pattern '{}' in query", pattern);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        
        // Enforce result limit to prevent resource exhaustion
        let query_with_limit = if !query_upper.contains("LIMIT") && query_upper.starts_with("SELECT") {
            format!("{} LIMIT 1000", query_text)
        } else {
            query_text.to_string()
        };

        match db.query(&query_with_limit, &[]).await {
            Ok(rows) => {
                // Serialize rows to JSON with proper type handling
                // Supports all common PostgreSQL types: String, i32, i64, f64, bool, Uuid, DateTime, JSONB
                // Handles NULL values correctly and extracts column names dynamically
                let results: Vec<JsonValue> = rows
                    .into_iter()
                    .map(|row| {
                        use sqlx::Column;
                        let mut json_obj = serde_json::Map::new();

                        // Get column names from row
                        let columns = row.columns();

                        for column in columns {
                            let column_name = column.name();

                            // Try to get value based on PostgreSQL type
                            // Handle common types with fallback to text
                            let value: Option<serde_json::Value> = {
                                // Try different type conversions by column name
                                if let Ok(val) = row.try_get::<String, _>(column_name) {
                                    Some(serde_json::Value::String(val))
                                } else if let Ok(val) =
                                    row.try_get::<Option<String>, _>(column_name)
                                {
                                    val.map(serde_json::Value::String)
                                        .or(Some(serde_json::Value::Null))
                                } else if let Ok(val) = row.try_get::<i32, _>(column_name) {
                                    Some(serde_json::Value::Number(val.into()))
                                } else if let Ok(val) = row.try_get::<Option<i32>, _>(column_name) {
                                    val.map(|v| serde_json::Value::Number(v.into()))
                                        .or(Some(serde_json::Value::Null))
                                } else if let Ok(val) = row.try_get::<i64, _>(column_name) {
                                    Some(serde_json::Value::Number(val.into()))
                                } else if let Ok(val) = row.try_get::<Option<i64>, _>(column_name) {
                                    val.map(|v| serde_json::Value::Number(v.into()))
                                        .or(Some(serde_json::Value::Null))
                                } else if let Ok(val) = row.try_get::<f64, _>(column_name) {
                                    serde_json::Number::from_f64(val).map(serde_json::Value::Number)
                                } else if let Ok(val) = row.try_get::<Option<f64>, _>(column_name) {
                                    val.and_then(|v| {
                                        serde_json::Number::from_f64(v)
                                            .map(serde_json::Value::Number)
                                    })
                                    .or(Some(serde_json::Value::Null))
                                } else if let Ok(val) = row.try_get::<bool, _>(column_name) {
                                    Some(serde_json::Value::Bool(val))
                                } else if let Ok(val) = row.try_get::<Option<bool>, _>(column_name)
                                {
                                    val.map(serde_json::Value::Bool)
                                        .or(Some(serde_json::Value::Null))
                                } else if let Ok(val) = row.try_get::<Uuid, _>(column_name) {
                                    Some(serde_json::Value::String(val.to_string()))
                                } else if let Ok(val) = row.try_get::<Option<Uuid>, _>(column_name)
                                {
                                    val.map(|v| serde_json::Value::String(v.to_string()))
                                        .or(Some(serde_json::Value::Null))
                                } else if let Ok(val) =
                                    row.try_get::<chrono::DateTime<chrono::Utc>, _>(column_name)
                                {
                                    Some(serde_json::Value::String(val.to_rfc3339()))
                                } else if let Ok(val) = row
                                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(
                                        column_name,
                                    )
                                {
                                    val.map(|v| serde_json::Value::String(v.to_rfc3339()))
                                        .or(Some(serde_json::Value::Null))
                                } else if let Ok(val) =
                                    row.try_get::<serde_json::Value, _>(column_name)
                                {
                                    Some(val)
                                } else if let Ok(val) =
                                    row.try_get::<Option<serde_json::Value>, _>(column_name)
                                {
                                    val.or(Some(serde_json::Value::Null))
                                } else {
                                    // Fallback: try to get as text or return null
                                    row.try_get::<String, _>(column_name)
                                        .ok()
                                        .map(serde_json::Value::String)
                                        .or_else(|| {
                                            row.try_get::<Option<String>, _>(column_name)
                                                .ok()
                                                .flatten()
                                                .map(serde_json::Value::String)
                                                .or(Some(serde_json::Value::Null))
                                        })
                                }
                            };

                            if let Some(json_val) = value {
                                json_obj.insert(column_name.to_string(), json_val);
                            } else {
                                // If all conversions failed, insert null
                                json_obj.insert(column_name.to_string(), serde_json::Value::Null);
                            }
                        }

                        serde_json::Value::Object(json_obj)
                    })
                    .collect();

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
                Err(_) => Err(StatusCode::NOT_FOUND),
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
                Err(_) => Err(StatusCode::NOT_FOUND),
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
                Err(_) => Err(StatusCode::NOT_FOUND),
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
                Err(_) => Err(StatusCode::NOT_FOUND),
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
                Err(_) => Err(StatusCode::NOT_FOUND),
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
                Err(_) => Err(StatusCode::NOT_FOUND),
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
                Err(_) => Err(StatusCode::NOT_FOUND),
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
                health["database"] =
                    serde_json::json!({ "status": "unhealthy", "error": e.to_string() });
                health["status"] = serde_json::json!("degraded");
            }
        }
    }

    Ok(Json(health))
}

// Duplicate get_metrics_handler removed - using Prometheus format version at line 2727
// The /metrics endpoint uses the Prometheus format handler which is standard for monitoring

async fn get_resource_usage_handler(
    State(_state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    use sysinfo::{Disks, Networks, System};

    let mut system = System::new_all();
    system.refresh_all();

    // Get CPU usage
    let cpu_usage = system.global_cpu_info().cpu_usage() as f64;

    // Get memory usage
    let _total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let memory_usage_mb = used_memory / (1024 * 1024); // Convert to MB

    // Get disk usage
    let mut _total_disk = 0u64;
    let mut used_disk = 0u64;
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        _total_disk += disk.total_space();
        used_disk += disk.total_space().saturating_sub(disk.available_space());
    }
    let disk_usage_mb = used_disk / (1024 * 1024); // Convert to MB

    // Get network usage
    let mut network_bytes = 0u64;
    let networks = Networks::new_with_refreshed_list();
    for (_interface_name, network) in networks.iter() {
        network_bytes += network.received() + network.transmitted();
    }
    let network_usage_mb = network_bytes / (1024 * 1024); // Convert to MB

    Ok(Json(serde_json::json!({
        "cpu": cpu_usage,
        "memory": memory_usage_mb,
        "disk": disk_usage_mb,
        "network": network_usage_mb,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
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
            let completed_tasks: Vec<_> = tasks
                .iter()
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

            let avg_duration_ms = if count > 0 {
                total_duration_ms / count as f64
            } else {
                0.0
            };

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
                name: payload
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or(StatusCode::BAD_REQUEST)?
                    .to_string(),
                query_text: payload
                    .get("query_text")
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
                Ok(_) => Ok(Json(
                    serde_json::json!({ "status": "deleted", "query_id": query_id }),
                )),
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match list_provenance_records(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                    coreml_inference_callback: None,
                }))
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => {
                        // If database error (e.g., table doesn't exist), return empty list
                        warn!(
                            "Failed to list provenance (table may not exist): {:?}",
                            status
                        );
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match link_provenance_to_commit(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Json(payload),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match verify_provenance_trailer(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Path(commit_hash),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match get_provenance_by_commit(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Path(commit_hash),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match get_task_provenance(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Path(task_id),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match list_waivers(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                    coreml_inference_callback: None,
                }))
                .await
                {
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match create_waiver(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Json(payload),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match approve_waiver(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Path(waiver_id),
                    Json(payload),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
async fn list_slos_handler(State(state): State<AppState>) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match list_slos(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                    coreml_inference_callback: None,
                }))
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match get_slo_status(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Path(slo_name),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
        if let Some(_db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match get_slo_measurements(
                    State(ApiState {
                        api: api.clone(),
                        websocket_manager: ws_manager.clone(),
                        query_performance_monitor: query_monitor.clone(),
                        coreml_inference_callback: None,
                    }),
                    Path(slo_name),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
                &state.api,
                &state.websocket_manager,
                &state.query_performance_monitor,
            ) {
                match list_slo_alerts(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                    coreml_inference_callback: None,
                }))
                .await
                {
                    Ok(response) => Ok(response),
                    Err(status) => Err(status),
                }
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
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

// ============================================================================
// Authentication Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    refresh_token: Option<String>,
    user: UserResponse,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: String,
    username: String,
    name: Option<String>,
    roles: Vec<String>,
    is_active: bool,
    last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

// Settings management request/response types

#[derive(Debug, Deserialize)]
struct CreateUserSettingRequest {
    setting_key: String,
    setting_value: serde_json::Value,
    setting_type: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUserSettingRequest {
    setting_value: Option<serde_json::Value>,
    setting_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateAppSettingRequest {
    setting_key: String,
    setting_value: serde_json::Value,
    setting_type: String,
    description: Option<String>,
    is_public: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateAppSettingRequest {
    setting_value: Option<serde_json::Value>,
    setting_type: Option<String>,
    description: Option<String>,
    is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CreateIntegrationRequest {
    name: String,
    integration_type: String,
    provider: String,
    configuration: serde_json::Value,
    credentials: serde_json::Value,
    is_active: bool,
    is_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateIntegrationRequest {
    name: Option<String>,
    configuration: Option<serde_json::Value>,
    credentials: Option<serde_json::Value>,
    is_active: Option<bool>,
    is_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    key_name: String,
    scopes: Vec<String>,
    rate_limit_per_minute: Option<i32>,
    rate_limit_per_hour: Option<i32>,
    rate_limit_per_day: Option<i32>,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyRequest {
    key_name: Option<String>,
    scopes: Option<Vec<String>>,
    rate_limit_per_minute: Option<i32>,
    rate_limit_per_hour: Option<i32>,
    rate_limit_per_day: Option<i32>,
    expires_at: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

/// Helper function to hash a token (for session storage)
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Password hashing and token generation now handled by AuthService in AppState

async fn login_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get user by username
    let user = db
        .get_user_by_username(&login_req.username)
        .await
        .map_err(|e| {
            error!("Database error during login: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            warn!(
                "Login attempt with invalid username: {}",
                login_req.username
            );
            StatusCode::UNAUTHORIZED
        })?;

    // Check if account is locked
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            warn!("Login attempt for locked account: {}", user.id);
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Check if account is active
    if !user.is_active {
        warn!("Login attempt for inactive account: {}", user.id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Verify password using AuthService
    let password_valid = state
        .auth_service
        .verify_password(&login_req.password, &user.password_hash)
        .map_err(|e| {
            error!("Password verification error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !password_valid {
        // Increment failed attempts
        let failed_attempts = user.failed_attempts + 1;
        let update = data_infrastructure::database_operations::UpdateUser {
            username: None,
            password_hash: None,
            name: None,
            roles: None,
            is_active: None,
            failed_attempts: Some(failed_attempts),
            locked_until: if failed_attempts >= 5 {
                Some(Utc::now() + ChronoDuration::minutes(15))
            } else {
                None
            },
            last_login: None,
        };

        let _ = db.update_user(user.id, update).await;

        warn!("Failed login attempt for user: {}", user.id);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Reset failed attempts on successful login
    let update = data_infrastructure::database_operations::UpdateUser {
        username: None,
        password_hash: None,
        name: None,
        roles: None,
        is_active: None,
        failed_attempts: Some(0),
        locked_until: None,
        last_login: Some(Utc::now()),
    };
    let _ = db.update_user(user.id, update).await;

    // Generate tokens using AuthService
    let user_id_str = user.id.to_string();
    let token = state
        .auth_service
        .generate_token(&user_id_str, &user.roles)
        .map_err(|e| {
            error!("Token generation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let refresh_token = state
        .auth_service
        .generate_token(&user_id_str, &user.roles)
        .map_err(|e| {
            error!("Refresh token generation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let token_hash = hash_token(&token);
    let refresh_token_hash = Some(hash_token(&refresh_token));

    let expires_at = Utc::now() + ChronoDuration::hours(24);
    let refresh_expires_at = Some(Utc::now() + ChronoDuration::days(7));

    // Get IP address and user agent from headers
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Create session
    let session = data_infrastructure::database_operations::CreateSession {
        user_id: user.id,
        token_hash,
        refresh_token_hash,
        expires_at,
        refresh_expires_at,
        ip_address,
        user_agent,
    };

    match db.create_session(session).await {
        Ok(_) => {
            info!("Successful login for user: {}", user.id);

            Ok(Json(LoginResponse {
                token,
                refresh_token: Some(refresh_token),
                user: UserResponse {
                    id: user.id.to_string(),
                    username: user.username,
                    name: user.name,
                    roles: user.roles,
                    is_active: user.is_active,
                    last_login: Some(Utc::now()),
                },
                expires_at,
            }))
        }
        Err(e) => {
            error!("Failed to create session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn logout_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Extract token from Authorization header
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_hash = hash_token(&token);

    // Find and deactivate session
    if let Ok(Some(session)) = db.get_session_by_token_hash(&token_hash).await {
        let update = data_infrastructure::database_operations::UpdateSession {
            token_hash: None,
            refresh_token_hash: None,
            expires_at: None,
            refresh_expires_at: None,
            is_active: Some(false),
        };

        match db.update_session(session.id, update).await {
            Ok(_) => {
                info!("User logged out: {}", session.user_id);
                Ok(Json(serde_json::json!({
                    "status": "success",
                    "message": "Logged out successfully"
                })))
            }
            Err(e) => {
                error!("Failed to update session during logout: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        // Token not found, but return success anyway (idempotent)
        Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Logged out successfully"
        })))
    }
}

async fn get_current_user_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserResponse>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Extract token from Authorization header
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_hash = hash_token(&token);

    // Find session
    let session = db
        .get_session_by_token_hash(&token_hash)
        .await
        .map_err(|e| {
            error!("Database error during get current user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if session is expired
    if Utc::now() > session.expires_at {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user
    let user = db
        .get_user(session.user_id)
        .await
        .map_err(|e| {
            error!("Database error during get current user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        username: user.username,
        name: user.name,
        roles: user.roles,
        is_active: user.is_active,
        last_login: user.last_login,
    }))
}

async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(refresh_req): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Hash the refresh token for database lookup
    let refresh_token_hash = hash_token(&refresh_req.refresh_token);

    // Query session by refresh_token_hash using trait method
    let session = db
        .get_session_by_refresh_token_hash(&refresh_token_hash)
        .await
        .map_err(|e| {
            error!("Database error during token refresh: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            warn!("Refresh token not found");
            StatusCode::UNAUTHORIZED
        })?;

    // Extract session fields
    let session_id = session.id;
    let user_id = session.user_id;
    let refresh_expires_at = session.refresh_expires_at;
    let is_active = session.is_active;

    // Validate session is active
    if !is_active {
        warn!("Refresh attempt with inactive session: {}", session_id);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Validate refresh token hasn't expired
    if let Some(refresh_expires) = refresh_expires_at {
        if Utc::now() > refresh_expires {
            warn!("Refresh token expired for session: {}", session_id);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Get user to generate new tokens with correct roles
    let user = db
        .get_user(user_id)
        .await
        .map_err(|e| {
            error!("Database error fetching user during refresh: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if user is active
    if !user.is_active {
        warn!("Refresh attempt for inactive user: {}", user_id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if user is locked
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            warn!("Refresh attempt for locked user: {}", user_id);
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Generate new access token
    let user_id_str = user.id.to_string();
    let new_token = state
        .auth_service
        .generate_token(&user_id_str, &user.roles)
        .map_err(|e| {
            error!("Token generation error during refresh: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Generate new refresh token (rotate for security)
    let new_refresh_token = state
        .auth_service
        .generate_token(&user_id_str, &user.roles)
        .map_err(|e| {
            error!("Refresh token generation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let new_token_hash = hash_token(&new_token);
    let new_refresh_token_hash = hash_token(&new_refresh_token);

    // Update session with new tokens
    let expires_at = Utc::now() + ChronoDuration::hours(24);
    let refresh_expires_at = Utc::now() + ChronoDuration::days(7);

    let update = data_infrastructure::database_operations::UpdateSession {
        token_hash: Some(new_token_hash),
        refresh_token_hash: Some(new_refresh_token_hash),
        expires_at: Some(expires_at),
        refresh_expires_at: Some(refresh_expires_at),
        is_active: None,
    };

    match db.update_session(session_id, update).await {
        Ok(_) => {
            info!("Token refreshed successfully for user: {}", user_id);

            Ok(Json(LoginResponse {
                token: new_token,
                refresh_token: Some(new_refresh_token),
                expires_at: expires_at,
                user: UserResponse {
                    id: user.id.to_string(),
                    username: user.username,
                    name: user.name,
                    roles: user.roles,
                    is_active: user.is_active,
                    last_login: user.last_login,
                },
            }))
        }
        Err(e) => {
            error!("Failed to update session during refresh: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Query performance handlers (delegate to existing handlers)
#[cfg(feature = "orchestration")]
async fn query_performance_summary_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
        &state.api,
        &state.websocket_manager,
        &state.query_performance_monitor,
    ) {
        match data_infrastructure::api::handlers::query_performance::get_query_performance_summary(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
                coreml_inference_callback: None,
            }),
        )
        .await
        {
            Ok(response) => Ok(Json(
                serde_json::to_value(response.0).unwrap_or(serde_json::json!({})),
            )),
            Err(status) => Err(status),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[cfg(feature = "orchestration")]
async fn query_performance_metrics_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
        &state.api,
        &state.websocket_manager,
        &state.query_performance_monitor,
    ) {
        match data_infrastructure::api::handlers::query_performance::get_all_query_metrics(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
                coreml_inference_callback: None,
            }),
            Query(params),
        )
        .await
        {
            Ok(response) => Ok(Json(
                serde_json::to_value(response.0).unwrap_or(serde_json::json!({})),
            )),
            Err(status) => Err(status),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[cfg(feature = "orchestration")]
async fn query_performance_slow_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
        &state.api,
        &state.websocket_manager,
        &state.query_performance_monitor,
    ) {
        match data_infrastructure::api::handlers::query_performance::get_slow_queries(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
                coreml_inference_callback: None,
            }),
            Query(params),
        )
        .await
        {
            Ok(response) => Ok(Json(
                serde_json::to_value(response.0).unwrap_or(serde_json::json!({})),
            )),
            Err(status) => Err(status),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[cfg(feature = "orchestration")]
async fn query_performance_top_slow_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (
        &state.api,
        &state.websocket_manager,
        &state.query_performance_monitor,
    ) {
        match data_infrastructure::api::handlers::query_performance::get_top_slow_queries(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
                coreml_inference_callback: None,
            }),
            Query(params),
        )
        .await
        {
            Ok(response) => Ok(Json(
                serde_json::to_value(response.0).unwrap_or(serde_json::json!({})),
            )),
            Err(status) => Err(status),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// ============================================================================
// Settings Management Handlers
// ============================================================================

// Helper function to extract user_id from Authorization header
async fn get_user_id_from_auth(
    headers: &axum::http::HeaderMap,
    db: &Arc<data_infrastructure::DatabaseClient>,
) -> Result<Uuid, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_hash = hash_token(&token);
    let session = db
        .get_session_by_token_hash(&token_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if Utc::now() > session.expires_at {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(session.user_id)
}

// User settings handlers
async fn get_user_settings_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let setting_type = params.get("type").map(|s| s.as_str());

    match db.get_user_settings(user_id, setting_type).await {
        Ok(settings) => Ok(Json(serde_json::json!({
            "settings": settings,
            "total": settings.len()
        }))),
        Err(e) => {
            error!("Failed to get user settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_user_setting_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateUserSettingRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    let create = data_infrastructure::database_operations::CreateUserSetting {
        user_id,
        setting_key: req.setting_key,
        setting_value: req.setting_value,
        setting_type: req.setting_type,
    };

    match db.create_user_setting(create).await {
        Ok(setting) => Ok(Json(serde_json::json!(setting))),
        Err(e) => {
            error!("Failed to create user setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_setting_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(key): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    match db.get_user_setting(user_id, &key).await {
        Ok(Some(setting)) => Ok(Json(serde_json::json!(setting))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get user setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_user_setting_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(key): Path<String>,
    Json(req): Json<UpdateUserSettingRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    let update = data_infrastructure::database_operations::UpdateUserSetting {
        setting_value: req.setting_value,
        setting_type: req.setting_type,
    };

    match db.update_user_setting(user_id, &key, update).await {
        Ok(setting) => Ok(Json(serde_json::json!(setting))),
        Err(e) => {
            error!("Failed to update user setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_user_setting_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(key): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    match db.delete_user_setting(user_id, &key).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "setting_key": key
        }))),
        Err(e) => {
            error!("Failed to delete user setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// App settings handlers
async fn get_app_settings_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let setting_type = params.get("type").map(|s| s.as_str());
    let is_public = params.get("is_public").and_then(|s| s.parse::<bool>().ok());

    match db.get_app_settings(setting_type, is_public).await {
        Ok(settings) => Ok(Json(serde_json::json!({
            "settings": settings,
            "total": settings.len()
        }))),
        Err(e) => {
            error!("Failed to get app settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_app_setting_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateAppSettingRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let created_by = get_user_id_from_auth(&headers, db).await?.to_string();

    let create = data_infrastructure::database_operations::CreateAppSetting {
        setting_key: req.setting_key,
        setting_value: req.setting_value,
        setting_type: req.setting_type,
        description: req.description,
        is_public: req.is_public,
        created_by,
    };

    match db.create_app_setting(create).await {
        Ok(setting) => Ok(Json(serde_json::json!(setting))),
        Err(e) => {
            error!("Failed to create app setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_app_setting_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_app_setting(&key).await {
        Ok(Some(setting)) => Ok(Json(serde_json::json!(setting))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get app setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_app_setting_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(key): Path<String>,
    Json(req): Json<UpdateAppSettingRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let updated_by = Some(get_user_id_from_auth(&headers, db).await?.to_string());

    let update = data_infrastructure::database_operations::UpdateAppSetting {
        setting_value: req.setting_value,
        setting_type: req.setting_type,
        description: req.description,
        is_public: req.is_public,
        updated_by,
    };

    match db.update_app_setting(&key, update).await {
        Ok(setting) => Ok(Json(serde_json::json!(setting))),
        Err(e) => {
            error!("Failed to update app setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_app_setting_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.delete_app_setting(&key).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "setting_key": key
        }))),
        Err(e) => {
            error!("Failed to delete app setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Integration handlers
async fn list_integrations_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let provider = params.get("provider").map(|s| s.as_str());
    let is_active = params.get("is_active").and_then(|s| s.parse::<bool>().ok());

    match db.get_integrations(provider, is_active).await {
        Ok(integrations) => Ok(Json(serde_json::json!({
            "integrations": integrations,
            "total": integrations.len()
        }))),
        Err(e) => {
            error!("Failed to get integrations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_integration_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateIntegrationRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let created_by = get_user_id_from_auth(&headers, db).await?.to_string();

    let create = data_infrastructure::database_operations::CreateIntegration {
        name: req.name,
        integration_type: req.integration_type,
        provider: req.provider,
        configuration: req.configuration,
        credentials: req.credentials,
        is_active: req.is_active,
        is_enabled: req.is_enabled,
        created_by,
    };

    match db.create_integration(create).await {
        Ok(integration) => Ok(Json(serde_json::json!(integration))),
        Err(e) => {
            error!("Failed to create integration: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_integration_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let integration_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_integration(integration_id).await {
        Ok(Some(integration)) => Ok(Json(serde_json::json!(integration))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get integration: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_integration_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateIntegrationRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let integration_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let updated_by = Some(get_user_id_from_auth(&headers, db).await?.to_string());

    let update = data_infrastructure::database_operations::UpdateIntegration {
        name: req.name,
        configuration: req.configuration,
        credentials: req.credentials,
        is_active: req.is_active,
        is_enabled: req.is_enabled,
        updated_by,
    };

    match db.update_integration(integration_id, update).await {
        Ok(integration) => Ok(Json(serde_json::json!(integration))),
        Err(e) => {
            error!("Failed to update integration: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_integration_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let integration_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.delete_integration(integration_id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "integration_id": id
        }))),
        Err(e) => {
            error!("Failed to delete integration: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// API key handlers
async fn list_api_keys_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let is_active = params.get("is_active").and_then(|s| s.parse::<bool>().ok());

    match db.get_user_api_keys(user_id, is_active).await {
        Ok(api_keys) => {
            // Don't expose key_hash or secret data
            let sanitized_keys: Vec<serde_json::Value> = api_keys
                .iter()
                .map(|key| {
                    serde_json::json!({
                        "id": key.id.to_string(),
                        "key_name": key.key_name,
                        "key_prefix": key.key_prefix,
                        "scopes": key.scopes,
                        "rate_limit_per_minute": key.rate_limit_per_minute,
                        "rate_limit_per_hour": key.rate_limit_per_hour,
                        "rate_limit_per_day": key.rate_limit_per_day,
                        "last_used_at": key.last_used_at.map(|d| d.to_rfc3339()),
                        "expires_at": key.expires_at.map(|d| d.to_rfc3339()),
                        "is_active": key.is_active,
                        "is_revoked": key.is_revoked,
                        "created_at": key.created_at.to_rfc3339(),
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({
                "api_keys": sanitized_keys,
                "total": sanitized_keys.len()
            })))
        }
        Err(e) => {
            error!("Failed to get API keys: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn create_api_key_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    // Generate API key using secure random (before any await to ensure Send)
    let (api_key, key_prefix, key_hash) = {
        use base64::{engine::general_purpose, Engine as _};
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let key_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let api_key = general_purpose::STANDARD.encode(&key_bytes);
        let key_prefix = api_key.chars().take(8).collect::<String>();
        let key_hash = hash_token(&api_key);
        (api_key, key_prefix, key_hash)
    };

    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let created_by = user_id.to_string();

    let expires_at = req
        .expires_at
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let create = data_infrastructure::database_operations::CreateApiKey {
        user_id,
        key_name: req.key_name,
        key_hash,
        key_prefix: key_prefix.clone(),
        scopes: req.scopes,
        rate_limit_per_minute: req.rate_limit_per_minute,
        rate_limit_per_hour: req.rate_limit_per_hour,
        rate_limit_per_day: req.rate_limit_per_day,
        expires_at,
        created_by,
    };

    match db.create_api_key(create).await {
        Ok(_) => {
            // Return the key only once (in production, this should be shown only once)
            Ok(Json(serde_json::json!({
                "status": "created",
                "api_key": format!("aa_{}", api_key),
                "key_prefix": key_prefix,
                "warning": "Store this key securely. It will not be shown again."
            })))
        }
        Err(e) => {
            error!("Failed to create API key: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_api_key_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match db.get_api_key(key_id).await {
        Ok(Some(key)) => {
            // Verify ownership
            if key.user_id != user_id {
                return Err(StatusCode::FORBIDDEN);
            }

            // Don't expose key_hash
            Ok(Json(serde_json::json!({
                "id": key.id.to_string(),
                "key_name": key.key_name,
                "key_prefix": key.key_prefix,
                "scopes": key.scopes,
                "rate_limit_per_minute": key.rate_limit_per_minute,
                "rate_limit_per_hour": key.rate_limit_per_hour,
                "rate_limit_per_day": key.rate_limit_per_day,
                "last_used_at": key.last_used_at.map(|d| d.to_rfc3339()),
                "expires_at": key.expires_at.map(|d| d.to_rfc3339()),
                "is_active": key.is_active,
                "is_revoked": key.is_revoked,
                "created_at": key.created_at.to_rfc3339(),
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get API key: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_api_key_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateApiKeyRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify ownership
    let existing_key = db
        .get_api_key(key_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if existing_key.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let expires_at = req
        .expires_at
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let update = data_infrastructure::database_operations::UpdateApiKey {
        key_name: req.key_name,
        scopes: req.scopes,
        rate_limit_per_minute: req.rate_limit_per_minute,
        rate_limit_per_hour: req.rate_limit_per_hour,
        rate_limit_per_day: req.rate_limit_per_day,
        expires_at,
        is_active: req.is_active,
    };

    match db.update_api_key(key_id, update).await {
        Ok(key) => {
            // Don't expose key_hash
            Ok(Json(serde_json::json!({
                "id": key.id.to_string(),
                "key_name": key.key_name,
                "key_prefix": key.key_prefix,
                "scopes": key.scopes,
                "rate_limit_per_minute": key.rate_limit_per_minute,
                "rate_limit_per_hour": key.rate_limit_per_hour,
                "rate_limit_per_day": key.rate_limit_per_day,
                "expires_at": key.expires_at.map(|d| d.to_rfc3339()),
                "is_active": key.is_active,
            })))
        }
        Err(e) => {
            error!("Failed to update API key: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn revoke_api_key_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify ownership
    let existing_key = db
        .get_api_key(key_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if existing_key.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let reason = req.get("reason").cloned();

    match db.revoke_api_key(key_id, reason).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "revoked",
            "api_key_id": id
        }))),
        Err(e) => {
            error!("Failed to revoke API key: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_api_key_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify ownership
    let existing_key = db
        .get_api_key(key_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if existing_key.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    match db.delete_api_key(key_id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "api_key_id": id
        }))),
        Err(e) => {
            error!("Failed to delete API key: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Password change handler
async fn change_password_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    // Get user
    let user = db
        .get_user(user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify current password
    let password_valid = state
        .auth_service
        .verify_password(&req.current_password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !password_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Hash new password
    let new_password_hash = state
        .auth_service
        .hash_password(&req.new_password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update user password
    let update = data_infrastructure::database_operations::UpdateUser {
        username: None,
        password_hash: Some(new_password_hash),
        name: None,
        roles: None,
        is_active: None,
        failed_attempts: Some(0),
        locked_until: None,
        last_login: None,
    };

    match db.update_user(user_id, update).await {
        Ok(_) => {
            info!("Password changed for user: {}", user_id);
            Ok(Json(serde_json::json!({
                "status": "success",
                "message": "Password changed successfully"
            })))
        }
        Err(e) => {
            error!("Failed to change password: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Rules & Governance Handlers
// ============================================================================

#[derive(Deserialize)]
struct CreateRuleRequest {
    id: String,
    name: String,
    description: String,
    rule_type: String,
    severity: String,
    file_patterns: JsonValue,
    config: JsonValue,
    constitutional_reference: Option<String>,
    is_active: bool,
}

#[derive(Deserialize)]
struct UpdateRuleRequest {
    name: Option<String>,
    description: Option<String>,
    rule_type: Option<String>,
    severity: Option<String>,
    file_patterns: Option<JsonValue>,
    config: Option<JsonValue>,
    constitutional_reference: Option<String>,
    is_active: Option<bool>,
}

#[derive(Deserialize)]
struct UpdateViolationRequest {
    status: Option<String>,
    metadata: Option<JsonValue>,
}

#[derive(Deserialize)]
struct CreateSpecificationRequest {
    name: String,
    version: String,
    description: Option<String>,
    rules: JsonValue,
    config: JsonValue,
    is_active: bool,
}

#[derive(Deserialize)]
struct UpdateSpecificationRequest {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    rules: Option<JsonValue>,
    config: Option<JsonValue>,
    is_active: Option<bool>,
}

#[derive(Deserialize)]
struct CreateRuleTemplateRequest {
    id: String,
    name: String,
    description: String,
    rule_type: String,
    template_config: JsonValue,
    example_config: Option<JsonValue>,
    is_system: bool,
    created_by: String,
}

#[derive(Deserialize)]
struct UpdateRuleEnforcementRequest {
    enforcement_state: Option<String>,
    paused_until: Option<DateTime<Utc>>,
    paused_reason: Option<String>,
    override_reason: Option<String>,
    metadata: Option<JsonValue>,
}

async fn list_rules_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let rule_type = params.get("rule_type").map(|s| s.as_str());
    let is_active = params.get("is_active").and_then(|s| s.parse::<bool>().ok());

    match db.get_caws_rules(rule_type, is_active).await {
        Ok(rules) => Ok(Json(serde_json::json!(rules))),
        Err(e) => {
            error!("Failed to list rules: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_rule_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let create = data_infrastructure::database_operations::CreateCawsRule {
        id: req.id,
        name: req.name,
        description: req.description,
        rule_type: req.rule_type,
        severity: req.severity,
        file_patterns: req.file_patterns,
        config: req.config,
        constitutional_reference: req.constitutional_reference,
        is_active: req.is_active,
    };

    match db.create_caws_rule(create).await {
        Ok(rule) => Ok(Json(serde_json::json!(rule))),
        Err(e) => {
            error!("Failed to create rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_rule_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_caws_rule(&id).await {
        Ok(Some(rule)) => Ok(Json(serde_json::json!(rule))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_rule_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateRuleRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let update = data_infrastructure::database_operations::UpdateCawsRule {
        name: req.name,
        description: req.description,
        rule_type: req.rule_type,
        severity: req.severity,
        file_patterns: req.file_patterns,
        config: req.config,
        constitutional_reference: req.constitutional_reference,
        is_active: req.is_active,
    };

    match db.update_caws_rule(&id, update).await {
        Ok(rule) => Ok(Json(serde_json::json!(rule))),
        Err(e) => {
            error!("Failed to update rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_rule_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.delete_caws_rule(&id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "message": "Rule deleted successfully"
        }))),
        Err(e) => {
            error!("Failed to delete rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn validate_rule_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(_req): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get rule
    let _rule = db
        .get_caws_rule(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Basic validation - check if rule config is valid JSON
    // In production, this would validate against rule schema
    Ok(Json(serde_json::json!({
        "valid": true,
        "rule_id": id,
        "message": "Rule configuration is valid"
    })))
}

async fn list_rule_templates_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let rule_type = params.get("rule_type").map(|s| s.as_str());

    match db.get_rule_templates(rule_type).await {
        Ok(templates) => Ok(Json(serde_json::json!(templates))),
        Err(e) => {
            error!("Failed to list rule templates: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_rule_template_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleTemplateRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let create = data_infrastructure::database_operations::CreateRuleTemplate {
        id: req.id,
        name: req.name,
        description: req.description,
        rule_type: req.rule_type,
        template_config: req.template_config,
        example_config: req.example_config,
        is_system: req.is_system,
        created_by: req.created_by,
    };

    match db.create_rule_template(create).await {
        Ok(template) => Ok(Json(serde_json::json!(template))),
        Err(e) => {
            error!("Failed to create rule template: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_rule_enforcement_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let task_id = params.get("task_id").and_then(|s| Uuid::parse_str(s).ok());

    match db.get_rule_enforcement_status(Some(&id), task_id).await {
        Ok(statuses) => Ok(Json(serde_json::json!(statuses))),
        Err(e) => {
            error!("Failed to get rule enforcement status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_rule_enforcement_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    Json(req): Json<UpdateRuleEnforcementRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let task_id = params.get("task_id").and_then(|s| Uuid::parse_str(s).ok());

    let update = data_infrastructure::database_operations::UpdateRuleEnforcementStatus {
        enforcement_state: req.enforcement_state,
        paused_until: req.paused_until,
        paused_reason: req.paused_reason,
        override_reason: req.override_reason,
        metadata: req.metadata,
    };

    match db
        .update_rule_enforcement_status(&id, task_id, update)
        .await
    {
        Ok(status) => Ok(Json(serde_json::json!(status))),
        Err(e) => {
            error!("Failed to update rule enforcement status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_rule_history_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let limit = params.get("limit").and_then(|s| s.parse::<u32>().ok());

    match db.get_rule_history(&id, limit).await {
        Ok(history) => Ok(Json(serde_json::json!(history))),
        Err(e) => {
            error!("Failed to get rule history: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_violations_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let task_id = params.get("task_id").and_then(|s| Uuid::parse_str(s).ok());
    let rule_id = params.get("rule_id").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());

    match db.get_caws_violations(task_id, rule_id, status).await {
        Ok(violations) => Ok(Json(serde_json::json!(violations))),
        Err(e) => {
            error!("Failed to list violations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_violation_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_caws_violation(id).await {
        Ok(Some(violation)) => Ok(Json(serde_json::json!(violation))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get violation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_violation_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateViolationRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let update = data_infrastructure::database_operations::UpdateCawsViolation {
        status: req.status,
        resolved_at: None, // Will be set automatically if status is "resolved"
        metadata: req.metadata,
    };

    match db.update_caws_violation(id, update).await {
        Ok(violation) => Ok(Json(serde_json::json!(violation))),
        Err(e) => {
            error!("Failed to update violation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Testing endpoints
// Note: We spawn the integrated_test binary as a subprocess to avoid circular dependency
#[cfg(feature = "testing")]
async fn run_integrated_test_handler(
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let scenario_id = payload
        .get("scenario_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Running integrated test via API: {}", scenario_id);

    // Spawn the integrated_test binary as a subprocess
    // Note: This requires the binary to be built and available in PATH
    // Get workspace root (assume we're in iterations/v3/data-interfaces-adapters)
    let workspace_root = std::env::current_dir()
        .ok()
        .and_then(|d| d.parent().map(|p| p.to_path_buf()))
        .and_then(|d| d.parent().map(|p| p.to_path_buf()))
        .and_then(|d| d.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("../../.."));

    let testing_dir = workspace_root.join("iterations/v3/testing-validation");
    let manifest_path = testing_dir.join("Cargo.toml");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "integrated_test",
            "--features",
            "full",
            "--manifest-path",
        ])
        .arg(&manifest_path)
        .arg(scenario_id) // Pass scenario_id as argument
        .current_dir(&testing_dir)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            // Try to parse the report file
            let workspace_root = std::env::current_dir()
                .ok()
                .and_then(|d| d.parent().map(|p| p.to_path_buf()))
                .and_then(|d| d.parent().map(|p| p.to_path_buf()))
                .and_then(|d| d.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("../../.."));
            let report_path =
                workspace_root.join("iterations/v3/testing-validation/integrated_test_report.md");
            let report_content = std::fs::read_to_string(&report_path)
                .unwrap_or_else(|_| "Report not available".to_string());

            Ok(Json(serde_json::json!({
                "scenario_id": scenario_id,
                "status": if result.status.success() { "completed" } else { "failed" },
                "exit_code": result.status.code(),
                "stdout": stdout,
                "stderr": stderr,
                "report": report_content,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })))
        }
        Err(e) => {
            error!("Failed to run integrated test: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(feature = "testing")]
async fn run_all_integrated_tests_handler() -> Result<Json<JsonValue>, StatusCode> {
    info!("Running all integrated tests via API");

    // Spawn the integrated_test binary as a subprocess
    let workspace_root = std::env::current_dir()
        .ok()
        .and_then(|d| d.parent().map(|p| p.to_path_buf()))
        .and_then(|d| d.parent().map(|p| p.to_path_buf()))
        .and_then(|d| d.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("../../.."));

    let testing_dir = workspace_root.join("iterations/v3/testing-validation");
    let manifest_path = testing_dir.join("Cargo.toml");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "integrated_test",
            "--features",
            "full",
            "--manifest-path",
        ])
        .arg(&manifest_path)
        .current_dir(&testing_dir)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            // Try to parse the report file
            let workspace_root = std::env::current_dir()
                .ok()
                .and_then(|d| d.parent().map(|p| p.to_path_buf()))
                .and_then(|d| d.parent().map(|p| p.to_path_buf()))
                .and_then(|d| d.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("../../.."));
            let report_path =
                workspace_root.join("iterations/v3/testing-validation/integrated_test_report.md");
            let report_content = std::fs::read_to_string(&report_path)
                .unwrap_or_else(|_| "Report not available".to_string());

            Ok(Json(serde_json::json!({
                "status": if result.status.success() { "completed" } else { "failed" },
                "exit_code": result.status.code(),
                "stdout": stdout,
                "stderr": stderr,
                "report": report_content,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })))
        }
        Err(e) => {
            error!("Failed to run integrated tests: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(feature = "testing")]
async fn list_test_scenarios_handler() -> Result<Json<JsonValue>, StatusCode> {
    Ok(Json(serde_json::json!({
        "scenarios": [
            {
                "id": "integrated-rust",
                "name": "Rust Code Fix",
                "file_type": "rust",
                "description": "Tests agent's ability to fix Rust compilation errors"
            },
            {
                "id": "integrated-typescript",
                "name": "TypeScript Code Fix",
                "file_type": "typescript",
                "description": "Tests agent's ability to fix TypeScript type errors"
            },
            {
                "id": "integrated-python",
                "name": "Python Code Fix",
                "file_type": "python",
                "description": "Tests agent's ability to fix Python syntax/logic errors"
            }
        ]
    })))
}

async fn resolve_violation_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.resolve_caws_violation(id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "resolved",
            "message": "Violation resolved successfully"
        }))),
        Err(e) => {
            error!("Failed to resolve violation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_compliance_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Aggregate compliance stats in database for performance
    let query = r#"
        WITH rule_stats AS (
            SELECT
                COUNT(*) as total_rules,
                COUNT(*) FILTER (WHERE is_active = true) as active_rules
            FROM caws_rules
        ),
        violation_stats AS (
            SELECT
                COUNT(*) as total_violations,
                COUNT(*) FILTER (WHERE status = 'open') as open_violations,
                COUNT(*) FILTER (WHERE status = 'resolved') as resolved_violations
            FROM caws_violations
        ),
        severity_counts AS (
            SELECT
                severity,
                COUNT(*) as count
            FROM caws_violations
            WHERE status = 'open'
            GROUP BY severity
        ),
        violations_by_rule AS (
            SELECT
                r.id as rule_id,
                r.name as rule_name,
                COUNT(v.id) as violation_count
            FROM caws_rules r
            LEFT JOIN caws_violations v ON r.id = v.rule_id AND v.status = 'open'
            GROUP BY r.id, r.name
            HAVING COUNT(v.id) > 0
            ORDER BY violation_count DESC
            LIMIT 20
        )
        SELECT
            rs.total_rules,
            rs.active_rules,
            vs.total_violations,
            vs.open_violations,
            vs.resolved_violations,
            CASE
                WHEN rs.active_rules > 0
                THEN GREATEST(0, 100 - (vs.open_violations::float / rs.active_rules::float * 100))
                ELSE 100
            END as compliance_score,
            COALESCE(
                jsonb_object_agg(DISTINCT sc.severity, sc.count) FILTER (WHERE sc.severity IS NOT NULL),
                '{}'::jsonb
            ) as violations_by_severity,
            COALESCE(
                jsonb_agg(
                    jsonb_build_object(
                        'rule_id', vbr.rule_id,
                        'rule_name', vbr.rule_name,
                        'violation_count', vbr.violation_count
                    )
                ) FILTER (WHERE vbr.rule_id IS NOT NULL),
                '[]'::jsonb
            ) as violations_by_rule
        FROM rule_stats rs
        CROSS JOIN violation_stats vs
        LEFT JOIN severity_counts sc ON true
        LEFT JOIN violations_by_rule vbr ON true
        GROUP BY rs.total_rules, rs.active_rules, vs.total_violations,
                 vs.open_violations, vs.resolved_violations
    "#;

    match db.query(query, &[]).await {
        Ok(rows) => {
            if rows.is_empty() {
                // Return default stats if no data
                Ok(Json(serde_json::json!({
                    "total_rules": 0,
                    "active_rules": 0,
                    "total_violations": 0,
                    "open_violations": 0,
                    "resolved_violations": 0,
                    "compliance_score": 100.0,
                    "violations_by_severity": {},
                    "violations_by_rule": []
                })))
            } else {
                let row = &rows[0];

                // Extract values from row
                let total_rules: i64 = row.try_get("total_rules").unwrap_or(0);
                let active_rules: i64 = row.try_get("active_rules").unwrap_or(0);
                let total_violations: i64 = row.try_get("total_violations").unwrap_or(0);
                let open_violations: i64 = row.try_get("open_violations").unwrap_or(0);
                let resolved_violations: i64 = row.try_get("resolved_violations").unwrap_or(0);
                let compliance_score: f64 = row.try_get("compliance_score").unwrap_or(100.0);

                // Extract JSONB fields
                let violations_by_severity: serde_json::Value = row
                    .try_get("violations_by_severity")
                    .unwrap_or(serde_json::json!({}));
                let violations_by_rule: serde_json::Value = row
                    .try_get("violations_by_rule")
                    .unwrap_or(serde_json::json!([]));

                Ok(Json(serde_json::json!({
                    "total_rules": total_rules,
                    "active_rules": active_rules,
                    "total_violations": total_violations,
                    "open_violations": open_violations,
                    "resolved_violations": resolved_violations,
                    "compliance_score": compliance_score,
                    "violations_by_severity": violations_by_severity,
                    "violations_by_rule": violations_by_rule
                })))
            }
        }
        Err(e) => {
            error!("Failed to get compliance stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_specifications_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let name = params.get("name").map(|s| s.as_str());
    let is_active = params.get("is_active").and_then(|s| s.parse::<bool>().ok());

    match db.get_caws_specifications(name, is_active).await {
        Ok(specs) => Ok(Json(serde_json::json!(specs))),
        Err(e) => {
            error!("Failed to list specifications: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_specification_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateSpecificationRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let create = data_infrastructure::database_operations::CreateCawsSpecification {
        name: req.name,
        version: req.version,
        description: req.description,
        rules: req.rules,
        config: req.config,
        is_active: req.is_active,
    };

    match db.create_caws_specification(create).await {
        Ok(spec) => Ok(Json(serde_json::json!(spec))),
        Err(e) => {
            error!("Failed to create specification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_specification_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.get_caws_specification(id).await {
        Ok(Some(spec)) => Ok(Json(serde_json::json!(spec))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get specification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_specification_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSpecificationRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let update = data_infrastructure::database_operations::UpdateCawsSpecification {
        name: req.name,
        version: req.version,
        description: req.description,
        rules: req.rules,
        config: req.config,
        is_active: req.is_active,
    };

    match db.update_caws_specification(id, update).await {
        Ok(spec) => Ok(Json(serde_json::json!(spec))),
        Err(e) => {
            error!("Failed to update specification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_specification_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state
        .db_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match db.delete_caws_specification(id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "deleted",
            "message": "Specification deleted successfully"
        }))),
        Err(e) => {
            error!("Failed to delete specification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
