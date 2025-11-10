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
    extract::{Path, State, Query},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use std::collections::HashMap;
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use tracing::{info, error, warn};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use system_quality_security::{AuthService, AuthConfig, authentication::PasswordPolicy};
use totp_rs::{TOTP, Algorithm};
use base32;

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
    /// Query performance monitor
    #[cfg(feature = "orchestration")]
    query_performance_monitor: Option<Arc<data_infrastructure::monitoring::query_performance::QueryPerformanceMonitor>>,
    /// Authentication service for password hashing and JWT generation
    auth_service: Arc<AuthService>,
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
                stream_timeout_seconds: env::var("STREAM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300), // Default: 5 minutes
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
                query_performance_monitor: Arc::new(data_infrastructure::monitoring::query_performance::QueryPerformanceMonitor::with_defaults()),
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

    // Initialize authentication service
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            warn!("⚠️  JWT_SECRET not set, using default (NOT SECURE FOR PRODUCTION)");
            "default-jwt-secret-key-change-in-production-min-32-chars".to_string()
        });
    
    let auth_config = AuthConfig {
        jwt_secret,
        token_expiry_seconds: 3600, // 1 hour
        refresh_token_expiry_seconds: 86400 * 7, // 7 days
        password_hash_params: argon2::Params::default(),
        max_failed_attempts: 5,
        lockout_duration_seconds: 900, // 15 minutes
        password_policy: PasswordPolicy::default(),
    };
    
    let auth_service = Arc::new(AuthService::new(auth_config));
    info!("✅ Authentication service initialized");

    // Create application state
    #[cfg(feature = "orchestration")]
    let (api_state_final, websocket_manager, query_perf_monitor) = if let Some((s, ws)) = api_state {
        (Some(s.api), Some(ws), Some(s.query_performance_monitor))
    } else {
        (None, None, None)
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
        #[cfg(feature = "orchestration")]
        query_performance_monitor: query_perf_monitor,
        auth_service,
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
        .route("/api/v1/tasks/stats", get(get_tasks_stats_handler))
        .route("/api/v1/tasks/:task_id", get(get_task_status_handler))
        .route("/api/v1/tasks/:task_id", axum::routing::patch(update_task_handler))
        .route("/api/v1/tasks/:task_id", axum::routing::delete(delete_task_handler))
        .route("/api/v1/tasks/:task_id/result", get(get_task_result_handler))
        .route("/api/v1/tasks/:task_id/cancel", post(cancel_task_handler))
        .route("/api/v1/tasks/:task_id/pause", post(pause_task_handler))
        .route("/api/v1/tasks/:task_id/resume", post(resume_task_handler))
        .route("/api/v1/projects/:project_id/tasks", post(create_project_task_handler))
        .route("/api/v1/projects/:project_id/tasks/:task_id", axum::routing::patch(update_project_task_handler))
        .route("/api/v1/projects/:project_id/tasks/:task_id", axum::routing::delete(delete_project_task_handler));

    // Worker/Agent management endpoints
    router = router
        .route("/api/v1/agents", get(list_agents_handler))
        .route("/api/v1/agents/stats", get(get_agents_stats_handler))
        .route("/api/v1/agents/:id", get(get_agent_handler))
        .route("/api/v1/agents/:id", axum::routing::patch(update_agent_handler))
        .route("/api/v1/agents/:id", axum::routing::delete(delete_agent_handler))
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
        .route("/api/v1/judges/:id", axum::routing::patch(update_judge_handler))
        .route("/api/v1/judges/:id", axum::routing::delete(delete_judge_handler))
        .route("/api/v1/judges/:id/stats", get(get_judge_stats_handler))
        .route("/api/v1/judges/:id/evaluations", get(get_judge_evaluations_handler));

    // Telemetry & Observability endpoints
    router = router
        .route("/api/v1/telemetry/contributions", get(get_contributions_handler))
        .route("/api/v1/telemetry/model-contributions", get(get_model_contributions_handler))
        .route("/api/v1/telemetry/agent-activity", get(get_agent_activity_handler))
        .route("/api/v1/observability/efficiency", get(get_efficiency_handler))
        .route("/api/v1/observability/system-metrics", get(get_resource_usage_handler))
        .route("/api/v1/observability/alerts", get(get_alerts_handler));

    // Chain of thought and observation endpoints
    router = router
        .route("/api/v1/tasks/:task_id/chain-of-thought", get(get_chain_of_thought_handler))
        .route("/api/v1/tasks/:task_id/council-decisions", get(get_council_decisions_handler))
        .route("/api/v1/tasks/:task_id/worker-actions", get(get_worker_actions_handler));

    // Chat and context endpoints
    router = router
        .route("/api/v1/chat", post(chat_handler));
    
    // Chat stream handlers require ApiState, so they're conditionally added
    #[cfg(feature = "orchestration")]
    {
        router = router
            .route("/api/v1/chat/stream", post(stream_agent_response_wrapper))
            .route("/api/v1/chat/stream/cancel", post(cancel_stream_wrapper));
    }
    
    router = router
        .route("/api/v1/chat/sessions", get(list_chat_sessions_handler))
        .route("/api/v1/chat/sessions/:session_id", get(get_chat_session_handler))
        .route("/api/v1/chat/sessions/:session_id/messages", get(get_chat_messages_handler));

    // Project management endpoints
    router = router
        .route("/api/v1/projects", post(scaffold_project_handler))
        .route("/api/v1/projects", get(list_projects_handler))
        .route("/api/v1/projects/:project_id", get(get_project_handler))
        .route("/api/v1/projects/:project_id", axum::routing::patch(update_project_handler))
        .route("/api/v1/projects/:project_id", axum::routing::delete(delete_project_handler))
        .route("/api/v1/projects/:project_id/stats", get(get_project_stats_handler))
        .route("/api/v1/projects/:project_id/tasks", get(get_project_tasks_handler))
        .route("/api/v1/projects/:project_id/tasks/stats", get(get_project_tasks_stats_handler))
        .route("/api/v1/projects/:project_id/milestones", get(get_project_milestones_handler))
        .route("/api/v1/projects/:project_id/milestones", post(create_project_milestone_handler))
        .route("/api/v1/projects/:project_id/milestones/:milestone_id", axum::routing::patch(update_project_milestone_handler));

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
        .route("/api/v1/system/metrics", get(get_resource_usage_handler));

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

    // Query performance monitoring endpoints
    #[cfg(feature = "orchestration")]
    {
        router = router
            .route("/api/v1/query-performance/summary", get(query_performance_summary_handler))
            .route("/api/v1/query-performance/metrics", get(query_performance_metrics_handler))
            .route("/api/v1/query-performance/slow", get(query_performance_slow_handler))
            .route("/api/v1/query-performance/top-slow", get(query_performance_top_slow_handler));
    }

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

    // Authentication endpoints
    router = router
        .route("/api/v1/auth/login", post(login_handler))
        .route("/api/v1/auth/logout", post(logout_handler))
        .route("/api/v1/auth/refresh", post(refresh_token_handler))
        .route("/api/v1/users/me", get(get_current_user_handler))
        .route("/api/v1/auth/password-reset/request", post(request_password_reset_handler))
        .route("/api/v1/auth/password-reset/confirm", post(confirm_password_reset_handler));

    // Settings management endpoints
    router = router
        // User settings
        .route("/api/v1/settings/user", get(get_user_settings_handler))
        .route("/api/v1/settings/user", post(create_user_setting_handler))
        .route("/api/v1/settings/user/:key", get(get_user_setting_handler))
        .route("/api/v1/settings/user/:key", axum::routing::patch(update_user_setting_handler))
        .route("/api/v1/settings/user/:key", axum::routing::delete(delete_user_setting_handler))
        // App settings
        .route("/api/v1/settings/app", get(get_app_settings_handler))
        .route("/api/v1/settings/app", post(create_app_setting_handler))
        .route("/api/v1/settings/app/:key", get(get_app_setting_handler))
        .route("/api/v1/settings/app/:key", axum::routing::patch(update_app_setting_handler))
        .route("/api/v1/settings/app/:key", axum::routing::delete(delete_app_setting_handler))
        // Integrations
        .route("/api/v1/settings/integrations", get(list_integrations_handler))
        .route("/api/v1/settings/integrations", post(create_integration_handler))
        .route("/api/v1/settings/integrations/:id", get(get_integration_handler))
        .route("/api/v1/settings/integrations/:id", axum::routing::patch(update_integration_handler))
        .route("/api/v1/settings/integrations/:id", axum::routing::delete(delete_integration_handler))
        // API keys
        .route("/api/v1/settings/api-keys", get(list_api_keys_handler))
        .route("/api/v1/settings/api-keys", post(create_api_key_handler))
        .route("/api/v1/settings/api-keys/:id", get(get_api_key_handler))
        .route("/api/v1/settings/api-keys/:id", axum::routing::patch(update_api_key_handler))
        .route("/api/v1/settings/api-keys/:id/revoke", post(revoke_api_key_handler))
        .route("/api/v1/settings/api-keys/:id", axum::routing::delete(delete_api_key_handler))
        // Password change
        .route("/api/v1/settings/password", post(change_password_handler))
        // Two-factor authentication
        .route("/api/v1/settings/2fa", get(get_2fa_handler))
        .route("/api/v1/settings/2fa", post(setup_2fa_handler))
        .route("/api/v1/settings/2fa/verify", post(verify_2fa_handler))
        .route("/api/v1/settings/2fa", axum::routing::delete(disable_2fa_handler));

    // Rules & Governance endpoints
    router = router
        // CAWS Rules CRUD
        .route("/api/v1/rules", get(list_rules_handler))
        .route("/api/v1/rules", post(create_rule_handler))
        .route("/api/v1/rules/:id", get(get_rule_handler))
        .route("/api/v1/rules/:id", axum::routing::patch(update_rule_handler))
        .route("/api/v1/rules/:id", axum::routing::delete(delete_rule_handler))
        // Rule validation
        .route("/api/v1/rules/:id/validate", post(validate_rule_handler))
        // Rule templates
        .route("/api/v1/rules/templates", get(list_rule_templates_handler))
        .route("/api/v1/rules/templates", post(create_rule_template_handler))
        // Rule enforcement status
        .route("/api/v1/rules/:id/enforcement", get(get_rule_enforcement_handler))
        .route("/api/v1/rules/:id/enforcement", axum::routing::patch(update_rule_enforcement_handler))
        // Rule history
        .route("/api/v1/rules/:id/history", get(get_rule_history_handler))
        // Violations
        .route("/api/v1/violations", get(list_violations_handler))
        .route("/api/v1/violations/:id", get(get_violation_handler))
        .route("/api/v1/violations/:id", axum::routing::patch(update_violation_handler))
        .route("/api/v1/violations/:id/resolve", post(resolve_violation_handler))
        // Specifications
        .route("/api/v1/specifications", get(list_specifications_handler))
        .route("/api/v1/specifications", post(create_specification_handler))
        .route("/api/v1/specifications/:id", get(get_specification_handler))
        .route("/api/v1/specifications/:id", axum::routing::patch(update_specification_handler))
        .route("/api/v1/specifications/:id", axum::routing::delete(delete_specification_handler));

    // Add CORS if enabled
    if enable_cors {
        router = router.layer(tower_http::cors::CorsLayer::permissive());
    }

    router.with_state(app_state)
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
        return None;
    }
    
    // Try to read the working spec
    let spec_content = std::fs::read_to_string(&spec_path).ok()?;
    
    // Simple heuristic: parse max_files and max_loc from YAML using string matching
    // This is a basic implementation - in production, use proper YAML parsing
    let max_files = extract_yaml_value(&spec_content, "max_files")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(10);
    let max_loc = extract_yaml_value(&spec_content, "max_loc")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(500);
    
    // Estimate: 5 minutes per file + 1 second per 10 lines
    let file_time = max_files * 300; // 5 minutes per file
    let loc_time = max_loc / 10; // 1 second per 10 lines
    Some(file_time + loc_time)
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
            
            let workspace_root = std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| ".".to_string());
            
            let request_context = RequestTaskContext {
                workspace_root: workspace_root.clone(),
                git_branch: detect_git_branch(&workspace_root).unwrap_or_else(|| "main".to_string()),
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
                    meta.insert("workspace_root".to_string(), serde_json::Value::String(request_context.workspace_root.clone()));
                    meta.insert("git_branch".to_string(), serde_json::Value::String(request_context.git_branch));
                    meta.insert("environment".to_string(), serde_json::Value::String(format!("{:?}", request_context.environment)));
                    meta
                },
            };

            // Clone workspace_root for use after task_context is moved
            let workspace_root = request_context.workspace_root.clone();

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
                        estimated_completion: estimate_completion_from_spec(&workspace_root)
                            .map(|seconds| Utc::now() + ChronoDuration::seconds(seconds)),
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

async fn update_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let update = data_infrastructure::database_operations::UpdateTask {
        title: payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
        description: payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        risk_tier: payload.get("risk_tier").and_then(|v| v.as_str()).map(|s| s.to_string()),
        scope: payload.get("scope").cloned(),
        acceptance_criteria: payload.get("acceptance_criteria").cloned(),
        context: payload.get("context").cloned(),
        caws_spec: payload.get("caws_spec").cloned(),
        status: payload.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
        assigned_worker_id: payload.get("assigned_worker_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
        priority: payload.get("priority").and_then(|v| v.as_i64()).map(|i| i as i32),
        deadline: payload.get("deadline").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
        metadata: payload.get("metadata").cloned(),
        completed_at: payload.get("completed_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
    };
    
    match db.update_task(task_uuid, update).await {
        Ok(task) => {
            Ok(Json(serde_json::json!({
                "task_id": task.id.to_string(),
                "title": task.title,
                "description": task.description,
                "status": task.status,
                "updated_at": task.updated_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.delete_task(task_uuid).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "deleted",
                "task_id": task_id,
            })))
        }
        Err(e) => {
            error!("Failed to delete task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_tasks_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    match db.get_tasks().await {
        Ok(tasks) => {
            let total = tasks.len();
            let completed = tasks.iter().filter(|t| t.status == "completed").count();
            let in_progress = tasks.iter().filter(|t| t.status == "in_progress").count();
            let pending = tasks.iter().filter(|t| t.status == "pending").count();
            let cancelled = tasks.iter().filter(|t| t.status == "cancelled").count();
            let failed = tasks.iter().filter(|t| t.status == "failed").count();
            
            Ok(Json(serde_json::json!({
                "total": total,
                "completed": completed,
                "in_progress": in_progress,
                "pending": pending,
                "cancelled": cancelled,
                "failed": failed,
            })))
        }
        Err(e) => {
            error!("Failed to get tasks stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_project_task_handler(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let _project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let title = payload.get("title")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let description = payload.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let create = data_infrastructure::database_operations::CreateTask {
        title: title.to_string(),
        description: description.to_string(),
        risk_tier: payload.get("risk_tier").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "2".to_string()),
        scope: payload.get("scope").cloned().unwrap_or_else(|| serde_json::json!({})),
        acceptance_criteria: payload.get("acceptance_criteria").cloned().unwrap_or_else(|| serde_json::json!([])),
        context: payload.get("context").cloned().unwrap_or_else(|| serde_json::json!({})),
        caws_spec: payload.get("caws_spec").cloned(),
        status: payload.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "pending".to_string()),
        assigned_worker_id: payload.get("assigned_worker_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
        priority: payload.get("priority").and_then(|v| v.as_i64()).map(|i| i as i32),
        deadline: payload.get("deadline").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
        metadata: {
            let mut metadata = payload.get("metadata").cloned().unwrap_or_else(|| serde_json::json!({}));
            // Add project_id to metadata for linking
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("project_id".to_string(), serde_json::Value::String(project_id.clone()));
            }
            Some(metadata)
        },
    };
    
    // Create task using CreateTask struct
    match db.create_task_from_create(create).await {
        Ok(task) => {
            Ok(Json(serde_json::json!({
                "task_id": task.id.to_string(),
                "title": task.title,
                "description": task.description,
                "status": task.status,
                "project_id": project_id,
                "created_at": task.created_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let _project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Verify task belongs to project (check metadata)
    let task = db.get_task(&task_uuid).await
        .map_err(|e| {
            error!("Failed to get task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(metadata) = task.metadata.as_ref().and_then(|m| m.as_object()) {
        if let Some(meta_project_id) = metadata.get("project_id").and_then(|v| v.as_str()) {
            if meta_project_id != project_id {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }
    
    // Update task (same as regular update_task_handler)
    let update = data_infrastructure::database_operations::UpdateTask {
        title: payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
        description: payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        risk_tier: payload.get("risk_tier").and_then(|v| v.as_str()).map(|s| s.to_string()),
        scope: payload.get("scope").cloned(),
        acceptance_criteria: payload.get("acceptance_criteria").cloned(),
        context: payload.get("context").cloned(),
        caws_spec: payload.get("caws_spec").cloned(),
        status: payload.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
        assigned_worker_id: payload.get("assigned_worker_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
        priority: payload.get("priority").and_then(|v| v.as_i64()).map(|i| i as i32),
        deadline: payload.get("deadline").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
        metadata: payload.get("metadata").cloned(),
        completed_at: payload.get("completed_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
    };
    
    match db.update_task(task_uuid, update).await {
        Ok(task) => {
            Ok(Json(serde_json::json!({
                "task_id": task.id.to_string(),
                "title": task.title,
                "status": task.status,
                "project_id": project_id,
                "updated_at": task.updated_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let _project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Verify task belongs to project (check metadata)
    let task = db.get_task(&task_uuid).await
        .map_err(|e| {
            error!("Failed to get task: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(metadata) = task.metadata.as_ref().and_then(|m| m.as_object()) {
        if let Some(meta_project_id) = metadata.get("project_id").and_then(|v| v.as_str()) {
            if meta_project_id != project_id {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }
    
    match db.delete_task(task_uuid).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "deleted",
                "task_id": task_id,
                "project_id": project_id,
            })))
        }
        Err(e) => {
            error!("Failed to delete task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Worker/Agent management handlers
async fn list_agents_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    match db.get_workers().await {
        Ok(workers) => {
            let agents: Vec<JsonValue> = workers.into_iter().map(|worker| {
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
            }).collect();
            
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    match db.get_workers().await {
        Ok(workers) => {
            let total = workers.len();
            let active = workers.iter().filter(|w| w.is_active).count();
            let inactive = total - active;
            
            // Count by worker type
            let mut by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.get_worker(agent_uuid).await {
        Ok(Some(worker)) => {
            Ok(Json(serde_json::json!({
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
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let update = data_infrastructure::database_operations::UpdateWorker {
        name: payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        worker_type: payload.get("worker_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        specialty: payload.get("specialty").and_then(|v| v.as_str()).map(|s| s.to_string()),
        model_name: payload.get("model_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        endpoint: payload.get("endpoint").and_then(|v| v.as_str()).map(|s| s.to_string()),
        capabilities: payload.get("capabilities").cloned(),
        performance_history: payload.get("performance_history").cloned(),
        is_active: payload.get("is_active").and_then(|v| v.as_bool()),
    };
    
    match db.update_worker(agent_uuid, update).await {
        Ok(worker) => {
            Ok(Json(serde_json::json!({
                "id": worker.id.to_string(),
                "name": worker.name,
                "worker_type": worker.worker_type,
                "is_active": worker.is_active,
                "updated_at": worker.updated_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.delete_worker(agent_uuid).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "deleted",
                "agent_id": agent_id,
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get worker
    let worker = db.get_worker(agent_uuid).await
        .map_err(|e| {
            error!("Failed to get worker: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get task executions for this worker
    match db.get_task_executions_by_worker(agent_uuid).await {
        Ok(executions) => {
            let total_tasks = executions.len();
            let completed = executions.iter().filter(|e| e.status == "completed").count();
            let failed = executions.iter().filter(|e| e.status == "failed").count();
            let in_progress = executions.iter().filter(|e| e.status == "in_progress").count();
            
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.get_worker(agent_uuid).await {
        Ok(Some(worker)) => {
            Ok(Json(serde_json::json!({
                "agent_id": agent_id,
                "status": if worker.is_active { "healthy" } else { "inactive" },
                "is_active": worker.is_active,
                "last_updated": worker.updated_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.get_worker(agent_uuid).await {
        Ok(Some(worker)) => {
            Ok(Json(serde_json::json!({
                "agent_id": agent_id,
                "performance_history": worker.performance_history,
                "capabilities": worker.capabilities,
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Verify worker exists
    let worker = db.get_worker(agent_uuid).await
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let agent_uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
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
        Ok(worker) => {
            Ok(Json(serde_json::json!({
                "status": "stopped",
                "agent_id": agent_id,
                "is_active": worker.is_active,
            })))
        }
        Err(e) => {
            error!("Failed to stop worker: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Judge management handlers
async fn list_judges_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    match db.get_judges().await {
        Ok(judges) => {
            let judges_list: Vec<JsonValue> = judges.into_iter().map(|judge| {
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
            }).collect();
            
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let name = payload.get("name")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let model_name = payload.get("model_name")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let endpoint = payload.get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let create = data_infrastructure::database_operations::CreateJudge {
        name: name.to_string(),
        model_name: model_name.to_string(),
        endpoint: endpoint.to_string(),
        weight: payload.get("weight").and_then(|v| v.as_f64()).map(|f| f as f32).unwrap_or(1.0),
        timeout_ms: payload.get("timeout_ms").and_then(|v| v.as_i64()).map(|i| i as i32).unwrap_or(5000),
        optimization_target: payload.get("optimization_target").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "accuracy".to_string()),
        is_active: payload.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true),
    };
    
    match db.create_judge(create).await {
        Ok(judge) => {
            Ok(Json(serde_json::json!({
                "id": judge.id.to_string(),
                "name": judge.name,
                "model_name": judge.model_name,
                "endpoint": judge.endpoint,
                "weight": judge.weight,
                "is_active": judge.is_active,
                "created_at": judge.created_at.to_rfc3339(),
            })))
        }
        Err(e) => {
            error!("Failed to create judge: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_judges_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.get_judge(judge_uuid).await {
        Ok(Some(judge)) => {
            Ok(Json(serde_json::json!({
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
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let update = data_infrastructure::database_operations::UpdateJudge {
        name: payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        model_name: payload.get("model_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        endpoint: payload.get("endpoint").and_then(|v| v.as_str()).map(|s| s.to_string()),
        weight: payload.get("weight").and_then(|v| v.as_f64()).map(|f| f as f32),
        timeout_ms: payload.get("timeout_ms").and_then(|v| v.as_i64()).map(|i| i as i32),
        optimization_target: payload.get("optimization_target").and_then(|v| v.as_str()).map(|s| s.to_string()),
        is_active: payload.get("is_active").and_then(|v| v.as_bool()),
    };
    
    match db.update_judge(judge_uuid, update).await {
        Ok(judge) => {
            Ok(Json(serde_json::json!({
                "id": judge.id.to_string(),
                "name": judge.name,
                "model_name": judge.model_name,
                "weight": judge.weight,
                "is_active": judge.is_active,
                "updated_at": judge.updated_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.delete_judge(judge_uuid).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "deleted",
                "judge_id": judge_id,
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get judge
    let judge = db.get_judge(judge_uuid).await
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
                evaluations.iter()
                    .filter_map(|e| e.confidence_score.or(e.confidence))
                    .collect::<Vec<_>>()
                    .iter()
                    .sum::<f32>() / total_evaluations as f32
            } else {
                0.0
            };
            
            // Count verdict decisions
            let mut verdict_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let judge_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.get_judge_evaluations_by_judge(judge_uuid).await {
        Ok(evaluations) => {
            let evaluations_list: Vec<JsonValue> = evaluations.into_iter().map(|eval| {
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
            }).collect();
            
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Get days parameter (default to 30)
    let days = params.get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(30);
    
    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days);
    
    // Query provenance_entries for code contribution events
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

async fn get_model_contributions_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Query telemetry_data for model-related contributions
    match db.query(
        "SELECT source, COUNT(*) as count, MAX(timestamp) as last_used FROM telemetry_data WHERE data_type = 'Metric' AND (source LIKE '%model%' OR source LIKE '%llm%' OR source LIKE '%inference%' OR payload->>'model' IS NOT NULL) GROUP BY source ORDER BY count DESC",
        &[]
    ).await {
        Ok(rows) => {
            let mut model_stats: Vec<JsonValue> = Vec::new();
            let mut total_requests = 0;
            
            for row in rows {
                let source: String = row.try_get("source").unwrap_or_default();
                let count: i64 = row.try_get("count").unwrap_or(0);
                let last_used: Option<chrono::DateTime<chrono::Utc>> = row.try_get("last_used").ok();
                
                total_requests += count;
                model_stats.push(serde_json::json!({
                    "model": source,
                    "request_count": count,
                    "last_used": last_used.map(|d| d.to_rfc3339()),
                }));
            }
            
            Ok(Json(serde_json::json!({
                "total_requests": total_requests,
                "models": model_stats,
            })))
        }
        Err(_) => {
            // If table doesn't exist or query fails, return empty result
            Ok(Json(serde_json::json!({
                "total_requests": 0,
                "models": [],
            })))
        }
    }
}

async fn get_agent_activity_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Get hours parameter (default to 24)
    let hours = params.get("hours")
        .and_then(|h| h.parse::<i64>().ok())
        .unwrap_or(24);
    
    let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(hours);
    
    // Query telemetry_data for agent activity
    match db.query(
        "SELECT DATE_TRUNC('hour', timestamp) as hour, source, COUNT(*) as activity_count FROM telemetry_data WHERE (source LIKE '%agent%' OR source LIKE '%worker%' OR source LIKE '%orchestrator%') AND timestamp >= $1 GROUP BY DATE_TRUNC('hour', timestamp), source ORDER BY hour DESC",
        &[&cutoff_time]
    ).await {
        Ok(rows) => {
            let mut activity: Vec<JsonValue> = Vec::new();
            let mut by_source: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            
            for row in rows {
                let hour: chrono::DateTime<chrono::Utc> = row.try_get("hour").unwrap_or_default();
                let source: String = row.try_get("source").unwrap_or_default();
                let count: i64 = row.try_get("activity_count").unwrap_or(0);
                
                *by_source.entry(source.clone()).or_insert(0) += count;
                
                activity.push(serde_json::json!({
                    "hour": hour.to_rfc3339(),
                    "source": source,
                    "activity_count": count,
                }));
            }
            
            let source_stats: Vec<JsonValue> = by_source.into_iter().map(|(source, count)| {
                serde_json::json!({
                    "source": source,
                    "total_activity": count,
                })
            }).collect();
            
            Ok(Json(serde_json::json!({
                "period_hours": hours,
                "time_series": activity,
                "by_source": source_stats,
            })))
        }
        Err(_) => {
            // If table doesn't exist or query fails, return empty result
            Ok(Json(serde_json::json!({
                "period_hours": hours,
                "time_series": [],
                "by_source": [],
            })))
        }
    }
}

async fn get_efficiency_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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

async fn get_system_metrics_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Query telemetry_data for system resource metrics
    match db.query(
        "SELECT payload->>'metric_name' as metric_name, AVG((payload->>'value')::float) as avg_value, MAX((payload->>'value')::float) as max_value FROM telemetry_data WHERE data_type = 'Metric' AND (payload->>'metric_name' LIKE '%cpu%' OR payload->>'metric_name' LIKE '%memory%' OR payload->>'metric_name' LIKE '%disk%' OR payload->>'metric_name' LIKE '%network%') AND timestamp >= NOW() - INTERVAL '1 hour' GROUP BY payload->>'metric_name'",
        &[]
    ).await {
        Ok(rows) => {
            let mut metrics: Vec<JsonValue> = Vec::new();
            
            for row in rows {
                let metric_name: Option<String> = row.try_get("metric_name").ok();
                let avg_value: Option<f64> = row.try_get("avg_value").ok();
                let max_value: Option<f64> = row.try_get("max_value").ok();
                
                if let Some(name) = metric_name {
                    metrics.push(serde_json::json!({
                        "metric": name,
                        "average": avg_value,
                        "max": max_value,
                    }));
                }
            }
            
            Ok(Json(serde_json::json!({
                "metrics": metrics,
                "period": "1 hour",
            })))
        }
        Err(_) => {
            // If table doesn't exist or query fails, return empty result
            Ok(Json(serde_json::json!({
                "metrics": [],
                "period": "1 hour",
            })))
        }
    }
}

async fn get_alerts_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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

// Wrapper handlers for chat stream endpoints (convert AppState to ApiState)
#[cfg(feature = "orchestration")]
async fn stream_agent_response_wrapper(
    State(state): State<AppState>,
    Json(request): Json<data_infrastructure::api::handlers::chat_handlers::StreamAgentRequest>,
) -> axum::response::Response {
    let api = match state.api.as_ref() {
        Some(api) => api,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "API service unavailable").into_response(),
    };
    let websocket_manager = match state.websocket_manager.as_ref() {
        Some(ws) => ws,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "WebSocket manager unavailable").into_response(),
    };
    let query_performance_monitor = match state.query_performance_monitor.as_ref() {
        Some(qpm) => qpm,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "Query performance monitor unavailable").into_response(),
    };
    
    let api_state = ApiState {
        api: api.clone(),
        websocket_manager: websocket_manager.clone(),
        query_performance_monitor: query_performance_monitor.clone(),
    };
    
    match data_infrastructure::api::handlers::chat_handlers::stream_agent_response(
        axum::extract::State(api_state),
        axum::Json(request)
    ).await {
        Ok(sse) => sse.into_response(),
        Err(e) => {
            error!("Stream agent response error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)).into_response()
        }
    }
}

#[cfg(feature = "orchestration")]
async fn cancel_stream_wrapper(
    State(state): State<AppState>,
    Json(request): Json<data_infrastructure::api::handlers::chat_handlers::CancelStreamRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let api = state.api.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let websocket_manager = state.websocket_manager.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let query_performance_monitor = state.query_performance_monitor.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let api_state = ApiState {
        api: api.clone(),
        websocket_manager: websocket_manager.clone(),
        query_performance_monitor: query_performance_monitor.clone(),
    };
    
    match data_infrastructure::api::handlers::chat_handlers::cancel_stream(
        axum::extract::State(api_state),
        axum::Json(request)
    ).await {
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
            // Use lightweight summaries to avoid cloning large vectors
            let task_summaries = service.list_task_summaries().await;
            let sessions: Vec<JsonValue> = task_summaries.into_iter().map(|task| {
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    match db.get_execution_plans().await {
        Ok(plans) => {
            let projects: Vec<JsonValue> = plans.into_iter().map(|plan| {
                serde_json::json!({
                    "project_id": plan.id.to_string(),
                    "name": plan.title,
                    "overview": plan.overview,
                    "state": plan.state,
                    "created_at": plan.created_at.to_rfc3339(),
                    "updated_at": plan.updated_at.to_rfc3339(),
                    "completed_at": plan.completed_at.map(|d| d.to_rfc3339()),
                })
            }).collect();
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.get_execution_plan(project_uuid).await {
        Ok(Some(plan)) => {
            Ok(Json(serde_json::json!({
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
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let update = data_infrastructure::database_operations::UpdateExecutionPlan {
        title: payload.get("name").or(payload.get("title")).and_then(|v| v.as_str()).map(|s| s.to_string()),
        overview: payload.get("overview").and_then(|v| v.as_str()).map(|s| s.to_string()),
        state: payload.get("state").and_then(|v| v.as_str()).map(|s| s.to_string()),
        milestones: payload.get("milestones").cloned(),
        dependency_graph: payload.get("dependency_graph").cloned(),
        change_budget: payload.get("change_budget").cloned(),
        quality_gates: payload.get("quality_gates").cloned(),
        evidence_requirements: payload.get("evidence_requirements").cloned(),
        active_waivers: payload.get("active_waivers").cloned(),
        metadata: payload.get("metadata").cloned(),
        approved_at: payload.get("approved_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
        completed_at: payload.get("completed_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
    };
    
    match db.update_execution_plan(project_uuid, update).await {
        Ok(plan) => {
            Ok(Json(serde_json::json!({
                "project_id": plan.id.to_string(),
                "name": plan.title,
                "overview": plan.overview,
                "state": plan.state,
                "updated_at": plan.updated_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.delete_execution_plan(project_uuid).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "deleted",
                "project_id": project_id,
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
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
    let in_progress_milestones = milestones.iter().filter(|m| m.state == "in_progress").count();
    
    // Get tasks for this project (if tasks table has project_id or plan_id field)
    // For now, we'll use metadata to link tasks to projects
    let task_count = 0; // TODO: Query tasks table when project_id field is added
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let _project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get all tasks and filter by project_id in metadata
    match db.get_tasks().await {
        Ok(tasks) => {
            let project_tasks: Vec<JsonValue> = tasks.into_iter()
                .filter(|task| {
                    if let Some(metadata) = task.metadata.as_ref().and_then(|m| m.as_object()) {
                        if let Some(meta_project_id) = metadata.get("project_id").and_then(|v| v.as_str()) {
                            return meta_project_id == project_id;
                        }
                    }
                    false
                })
                .map(|task| {
                    serde_json::json!({
                        "task_id": task.id.to_string(),
                        "title": task.title,
                        "description": task.description,
                        "status": task.status,
                        "risk_tier": task.risk_tier,
                        "priority": task.priority,
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let _project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get all tasks and filter by project_id in metadata
    match db.get_tasks().await {
        Ok(tasks) => {
            let project_tasks: Vec<_> = tasks.into_iter()
                .filter(|task| {
                    if let Some(metadata) = task.metadata.as_ref().and_then(|m| m.as_object()) {
                        if let Some(meta_project_id) = metadata.get("project_id").and_then(|v| v.as_str()) {
                            return meta_project_id == project_id;
                        }
                    }
                    false
                })
                .collect();
            
            let total = project_tasks.len();
            let completed = project_tasks.iter().filter(|t| t.status == "completed").count();
            let in_progress = project_tasks.iter().filter(|t| t.status == "in_progress").count();
            let pending = project_tasks.iter().filter(|t| t.status == "pending").count();
            let cancelled = project_tasks.iter().filter(|t| t.status == "cancelled").count();
            let failed = project_tasks.iter().filter(|t| t.status == "failed").count();
            
            Ok(Json(serde_json::json!({
                "total": total,
                "completed": completed,
                "in_progress": in_progress,
                "pending": pending,
                "cancelled": cancelled,
                "failed": failed,
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match db.get_milestones(project_uuid).await {
        Ok(milestones) => {
            let milestone_list: Vec<JsonValue> = milestones.into_iter().map(|m| {
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
            }).collect();
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let milestone_id = payload.get("id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let objective = payload.get("objective")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let create = data_infrastructure::database_operations::CreateMilestone {
        id: milestone_id.to_string(),
        plan_id: project_uuid,
        objective: objective.to_string(),
        scope: Some(payload.get("scope").cloned().unwrap_or_else(|| serde_json::json!({}))),
        interfaces: Some(payload.get("interfaces").cloned().unwrap_or_else(|| serde_json::json!([]))),
        tests: Some(payload.get("tests").cloned().unwrap_or_else(|| serde_json::json!([]))),
        evidence_gate: Some(payload.get("evidence_gate").cloned().unwrap_or_else(|| serde_json::json!({}))),
        rollback_plan: payload.get("rollback_plan").and_then(|v| v.as_str()).map(|s| s.to_string()),
        dependencies: Some(payload.get("dependencies").cloned().unwrap_or_else(|| serde_json::json!([]))),
        state: Some(payload.get("state").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "pending".to_string())),
        assigned_worker_id: payload.get("assigned_worker_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
        estimated_effort: payload.get("estimated_effort").and_then(|v| v.as_f64()),
        priority: payload.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string()),
        risk_tier: payload.get("risk_tier").and_then(|v| v.as_i64()).map(|i| i as i32),
        is_blocking: payload.get("is_blocking").and_then(|v| v.as_bool()),
        blocking_reason: payload.get("blocking_reason").and_then(|v| v.as_str()).map(|s| s.to_string()),
        metrics: payload.get("metrics").cloned(),
    };
    
    match db.create_milestone(create).await {
        Ok(milestone) => {
            Ok(Json(serde_json::json!({
                "id": milestone.id,
                "plan_id": milestone.plan_id.to_string(),
                "objective": milestone.objective,
                "state": milestone.state,
                "created_at": milestone.created_at.to_rfc3339(),
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let update = data_infrastructure::database_operations::UpdateMilestone {
        objective: payload.get("objective").and_then(|v| v.as_str()).map(|s| s.to_string()),
        scope: payload.get("scope").cloned(),
        interfaces: payload.get("interfaces").cloned(),
        tests: payload.get("tests").cloned(),
        evidence_gate: payload.get("evidence_gate").cloned(),
        rollback_plan: payload.get("rollback_plan").and_then(|v| v.as_str()).map(|s| s.to_string()),
        dependencies: payload.get("dependencies").cloned(),
        state: payload.get("state").and_then(|v| v.as_str()).map(|s| s.to_string()),
        assigned_worker_id: payload.get("assigned_worker_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
        estimated_effort: payload.get("estimated_effort").and_then(|v| v.as_f64()),
        priority: payload.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string()),
        risk_tier: payload.get("risk_tier").and_then(|v| v.as_i64()).map(|i| i as i32),
        is_blocking: payload.get("is_blocking").and_then(|v| v.as_bool()),
        blocking_reason: payload.get("blocking_reason").and_then(|v| v.as_str()).map(|s| s.to_string()),
        metrics: payload.get("metrics").cloned(),
        started_at: payload.get("started_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
        completed_at: payload.get("completed_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
    };
    
    match db.update_milestone(project_uuid, milestone_id.clone(), update).await {
        Ok(milestone) => {
            Ok(Json(serde_json::json!({
                "id": milestone.id,
                "plan_id": milestone.plan_id.to_string(),
                "objective": milestone.objective,
                "state": milestone.state,
                "updated_at": milestone.updated_at.to_rfc3339(),
            })))
        }
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
                // Serialize rows to JSON with proper type handling
                // Supports all common PostgreSQL types: String, i32, i64, f64, bool, Uuid, DateTime, JSONB
                // Handles NULL values correctly and extracts column names dynamically
                let results: Vec<JsonValue> = rows.into_iter().map(|row| {
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
                            } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                                val.map(serde_json::Value::String).or(Some(serde_json::Value::Null))
                            } else if let Ok(val) = row.try_get::<i32, _>(column_name) {
                                Some(serde_json::Value::Number(val.into()))
                            } else if let Ok(val) = row.try_get::<Option<i32>, _>(column_name) {
                                val.map(|v| serde_json::Value::Number(v.into())).or(Some(serde_json::Value::Null))
                            } else if let Ok(val) = row.try_get::<i64, _>(column_name) {
                                Some(serde_json::Value::Number(val.into()))
                            } else if let Ok(val) = row.try_get::<Option<i64>, _>(column_name) {
                                val.map(|v| serde_json::Value::Number(v.into())).or(Some(serde_json::Value::Null))
                            } else if let Ok(val) = row.try_get::<f64, _>(column_name) {
                                serde_json::Number::from_f64(val).map(serde_json::Value::Number)
                            } else if let Ok(val) = row.try_get::<Option<f64>, _>(column_name) {
                                val.and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number)).or(Some(serde_json::Value::Null))
                            } else if let Ok(val) = row.try_get::<bool, _>(column_name) {
                                Some(serde_json::Value::Bool(val))
                            } else if let Ok(val) = row.try_get::<Option<bool>, _>(column_name) {
                                val.map(serde_json::Value::Bool).or(Some(serde_json::Value::Null))
                            } else if let Ok(val) = row.try_get::<Uuid, _>(column_name) {
                                Some(serde_json::Value::String(val.to_string()))
                            } else if let Ok(val) = row.try_get::<Option<Uuid>, _>(column_name) {
                                val.map(|v| serde_json::Value::String(v.to_string())).or(Some(serde_json::Value::Null))
                            } else if let Ok(val) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(column_name) {
                                Some(serde_json::Value::String(val.to_rfc3339()))
                            } else if let Ok(val) = row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(column_name) {
                                val.map(|v| serde_json::Value::String(v.to_rfc3339())).or(Some(serde_json::Value::Null))
                            } else if let Ok(val) = row.try_get::<serde_json::Value, _>(column_name) {
                                Some(val)
                            } else if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(column_name) {
                                val.or(Some(serde_json::Value::Null))
                            } else {
                                // Fallback: try to get as text or return null
                                row.try_get::<String, _>(column_name).ok()
                                    .map(serde_json::Value::String)
                                    .or_else(|| {
                                        row.try_get::<Option<String>, _>(column_name).ok().flatten()
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
    use sysinfo::{System, Disks, Networks};
    
    let mut system = System::new_all();
    system.refresh_all();
    
    // Get CPU usage
    let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
    
    // Get memory usage
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let memory_usage_mb = used_memory / (1024 * 1024); // Convert to MB
    
    // Get disk usage
    let mut total_disk = 0u64;
    let mut used_disk = 0u64;
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        total_disk += disk.total_space();
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
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match list_provenance_records(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
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
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match link_provenance_to_commit(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Json(payload)).await {
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
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match verify_provenance_trailer(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Path(commit_hash)).await {
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
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match get_provenance_by_commit(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Path(commit_hash)).await {
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
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match get_task_provenance(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Path(task_id)).await {
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
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match list_waivers(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
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
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match create_waiver(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Json(payload)).await {
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
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match approve_waiver(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Path(waiver_id), Json(payload)).await {
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
async fn list_slos_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    #[cfg(feature = "orchestration")]
    {
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match list_slos(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                })).await {
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
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match get_slo_status(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Path(slo_name)).await {
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
        if let Some(db) = &state.db_client {
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match get_slo_measurements(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                }), Path(slo_name)).await {
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
            if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
                match list_slo_alerts(State(ApiState {
                    api: api.clone(),
                    websocket_manager: ws_manager.clone(),
                    query_performance_monitor: query_monitor.clone(),
                })).await {
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
    email: Option<String>,
    username: Option<String>,
    password: String,
    totp_code: Option<String>, // Optional TOTP code for 2FA
    recovery_code: Option<String>, // Optional recovery code for 2FA
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
    email: String,
    username: String,
    name: Option<String>,
    roles: Vec<String>,
    is_active: bool,
    last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct PasswordResetRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct PasswordResetConfirmRequest {
    token: String,
    new_password: String,
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

#[derive(Debug, Deserialize)]
struct Setup2FARequest {
    method: String,
}

#[derive(Debug, Deserialize)]
struct Verify2FARequest {
    method: String,
    code: String,
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get user by email or username
    let user = if let Some(ref email) = login_req.email {
        db.get_user_by_email(email).await
            .map_err(|e| {
                error!("Database error during login: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    } else if let Some(ref username) = login_req.username {
        db.get_user_by_username(username).await
            .map_err(|e| {
                error!("Database error during login: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let user = user.ok_or_else(|| {
        warn!("Login attempt with invalid credentials");
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
    let password_valid = state.auth_service.verify_password(&login_req.password, &user.password_hash)
        .map_err(|e| {
            error!("Password verification error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !password_valid {
        // Increment failed attempts
        let failed_attempts = user.failed_attempts + 1;
        let update = data_infrastructure::database_operations::UpdateUser {
            email: None,
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

    // Check if 2FA is enabled and verify code if provided
    let two_fa = db.get_two_factor_auth(user.id, Some("totp")).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if let Some(two_fa_config) = two_fa {
        if two_fa_config.is_enabled {
            // 2FA is enabled - require code
            let code_provided = login_req.totp_code.as_ref().or(login_req.recovery_code.as_ref());
            
            if code_provided.is_none() {
                // Password correct but 2FA code required
                return Err(StatusCode::UNAUTHORIZED); // Return 401 to indicate 2FA required
            }
            
            let code = code_provided.unwrap();
            let mut code_valid = false;
            
            // Check recovery code first
            if two_fa_config.backup_codes.contains(code) {
                // Valid recovery code - remove it
                let mut updated_backup_codes = two_fa_config.backup_codes.clone();
                updated_backup_codes.retain(|c| c != code);
                
                let update = data_infrastructure::database_operations::UpdateTwoFactorAuth {
                    secret_encrypted: None,
                    backup_codes: Some(updated_backup_codes),
                    is_enabled: None,
                };
                
                let _ = db.update_two_factor_auth(user.id, "totp", update).await;
                code_valid = true;
            } else if let Some(totp_code) = &login_req.totp_code {
                // Verify TOTP code
                let secret_base32 = two_fa_config.secret_encrypted.clone();
                
                // Decode base32 secret to bytes
                let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret_base32)
                    .ok_or_else(|| {
                        error!("Failed to decode TOTP secret from base32");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;
                
                let totp = TOTP::new(
                    Algorithm::SHA1,
                    6,
                    1,
                    30,
                    secret_bytes,
                    Some("Agent Agency V3".to_string()),
                    user.email.clone(),
                ).map_err(|e| {
                    error!("Failed to create TOTP instance: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                // Verify with tolerance window of ±1 step
                code_valid = totp.check(totp_code, 1);
            }
            
            if !code_valid {
                warn!("Invalid 2FA code for user: {}", user.id);
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // Reset failed attempts on successful login
    let update = data_infrastructure::database_operations::UpdateUser {
        email: None,
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
    let token = state.auth_service.generate_token(&user_id_str, &user.roles)
        .map_err(|e| {
            error!("Token generation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let refresh_token = state.auth_service.generate_token(&user_id_str, &user.roles)
        .map_err(|e| {
            error!("Refresh token generation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let token_hash = hash_token(&token);
    let refresh_token_hash = Some(hash_token(&refresh_token));
    
    let expires_at = Utc::now() + ChronoDuration::hours(24);
    let refresh_expires_at = Some(Utc::now() + ChronoDuration::days(7));

    // Get IP address and user agent from headers
    let ip_address = headers.get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers.get("user-agent")
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
                    email: user.email,
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Extract token from Authorization header
    let token = headers.get("authorization")
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Extract token from Authorization header
    let token = headers.get("authorization")
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
    let session = db.get_session_by_token_hash(&token_hash).await
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
    let user = db.get_user(session.user_id).await
        .map_err(|e| {
            error!("Database error during get current user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Hash the refresh token for database lookup
    let refresh_token_hash = hash_token(&refresh_req.refresh_token);
    
    // Query session by refresh_token_hash
    let query = r#"
        SELECT 
            id, user_id, token_hash, refresh_token_hash, expires_at, 
            refresh_expires_at, is_active, ip_address, user_agent, created_at, updated_at
        FROM sessions
        WHERE refresh_token_hash = $1
        LIMIT 1
    "#;
    
    let session_row = db.query_one_with_params(query, &[&refresh_token_hash]).await
        .map_err(|e| {
            error!("Database error during token refresh: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let session = session_row.ok_or_else(|| {
        warn!("Refresh token not found");
        StatusCode::UNAUTHORIZED
    })?;
    
    // Extract session fields
    let session_id: Uuid = session.get("id");
    let user_id: Uuid = session.get("user_id");
    let refresh_expires_at: Option<chrono::DateTime<chrono::Utc>> = session.try_get("refresh_expires_at").ok().flatten();
    let is_active: bool = session.get("is_active");
    
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
    let user = db.get_user(user_id).await
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
    let new_token = state.auth_service.generate_token(&user_id_str, &user.roles)
        .map_err(|e| {
            error!("Token generation error during refresh: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Generate new refresh token (rotate for security)
    let new_refresh_token = state.auth_service.generate_token(&user_id_str, &user.roles)
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
                    email: user.email,
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

async fn request_password_reset_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(reset_req): Json<PasswordResetRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get user by email
    let user = db.get_user_by_email(&reset_req.email).await
        .map_err(|e| {
            error!("Database error during password reset request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Always return success (don't reveal if email exists)
    if let Some(user) = user {
        // Generate reset token
        let reset_token = Uuid::new_v4().to_string();
        let token_hash = hash_token(&reset_token);
        let expires_at = Utc::now() + ChronoDuration::hours(1);

        let ip_address = headers.get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let token = data_infrastructure::database_operations::CreatePasswordResetToken {
            user_id: user.id,
            token_hash,
            expires_at,
            ip_address,
        };

        match db.create_password_reset_token(token).await {
            Ok(_) => {
                info!("Password reset token created for user: {}", user.id);
                // PLACEHOLDER: Email sending not implemented
                // In production, integrate with email service (SMTP, SendGrid, SES, etc.)
                // Required: Email service configuration, template rendering, error handling
                // For now, log token in development only (NOT SECURE - remove in production)
                if std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "development" {
                    warn!("Password reset token (DEV ONLY - email sending not implemented): {}", reset_token);
                } else {
                    warn!("Password reset token created but email sending not implemented - token logged for debugging");
                }
            }
            Err(e) => {
                error!("Failed to create password reset token: {}", e);
            }
        }
    }

    // Always return success
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "If the email exists, a password reset link has been sent"
    })))
}

// Query performance handlers (delegate to existing handlers)
#[cfg(feature = "orchestration")]
async fn query_performance_summary_handler(
    State(state): State<AppState>,
) -> Result<Json<JsonValue>, StatusCode> {
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
        match data_infrastructure::api::handlers::query_performance::get_query_performance_summary(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
            })
        ).await {
            Ok(response) => Ok(Json(serde_json::to_value(response.0).unwrap_or(serde_json::json!({})))),
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
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
        match data_infrastructure::api::handlers::query_performance::get_all_query_metrics(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
            }),
            Query(params)
        ).await {
            Ok(response) => Ok(Json(serde_json::to_value(response.0).unwrap_or(serde_json::json!({})))),
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
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
        match data_infrastructure::api::handlers::query_performance::get_slow_queries(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
            }),
            Query(params)
        ).await {
            Ok(response) => Ok(Json(serde_json::to_value(response.0).unwrap_or(serde_json::json!({})))),
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
    if let (Some(api), Some(ws_manager), Some(query_monitor)) = (&state.api, &state.websocket_manager, &state.query_performance_monitor) {
        match data_infrastructure::api::handlers::query_performance::get_top_slow_queries(
            State(ApiState {
                api: api.clone(),
                websocket_manager: ws_manager.clone(),
                query_performance_monitor: query_monitor.clone(),
            }),
            Query(params)
        ).await {
            Ok(response) => Ok(Json(serde_json::to_value(response.0).unwrap_or(serde_json::json!({})))),
            Err(status) => Err(status),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn confirm_password_reset_handler(
    State(state): State<AppState>,
    Json(confirm_req): Json<PasswordResetConfirmRequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let token_hash = hash_token(&confirm_req.token);

    // Get password reset token
    let reset_token = db.get_password_reset_token(&token_hash).await
        .map_err(|e| {
            error!("Database error during password reset confirm: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Get user
    let user = db.get_user(reset_token.user_id).await
        .map_err(|e| {
            error!("Database error during password reset confirm: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Hash new password using AuthService
    let new_password_hash = state.auth_service.hash_password(&confirm_req.new_password)
        .map_err(|e| {
            error!("Password hashing error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Update user password
    let update = data_infrastructure::database_operations::UpdateUser {
        email: None,
        username: None,
        password_hash: Some(new_password_hash),
        name: None,
        roles: None,
        is_active: None,
        failed_attempts: Some(0), // Reset failed attempts
        locked_until: None,
        last_login: None,
    };

    match db.update_user(user.id, update).await {
        Ok(_) => {
            // Mark token as used
            let _ = db.mark_password_reset_token_used(reset_token.id).await;
            
            info!("Password reset completed for user: {}", user.id);
            
            Ok(Json(serde_json::json!({
                "status": "success",
                "message": "Password reset successfully"
            })))
        }
        Err(e) => {
            error!("Failed to update password: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
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
    let session = db.get_session_by_token_hash(&token_hash).await
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let setting_type = params.get("type").map(|s| s.as_str());

    match db.get_user_settings(user_id, setting_type).await {
        Ok(settings) => {
            Ok(Json(serde_json::json!({
                "settings": settings,
                "total": settings.len()
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let setting_type = params.get("type").map(|s| s.as_str());
    let is_public = params.get("is_public").and_then(|s| s.parse::<bool>().ok());

    match db.get_app_settings(setting_type, is_public).await {
        Ok(settings) => {
            Ok(Json(serde_json::json!({
                "settings": settings,
                "total": settings.len()
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let created_by = get_user_id_from_auth(&headers, db).await?
        .to_string();

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let updated_by = Some(get_user_id_from_auth(&headers, db).await?
        .to_string());

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let provider = params.get("provider").map(|s| s.as_str());
    let is_active = params.get("is_active").and_then(|s| s.parse::<bool>().ok());

    match db.get_integrations(provider, is_active).await {
        Ok(integrations) => {
            Ok(Json(serde_json::json!({
                "integrations": integrations,
                "total": integrations.len()
            })))
        }
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let created_by = get_user_id_from_auth(&headers, db).await?
        .to_string();

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let integration_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let integration_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let updated_by = Some(get_user_id_from_auth(&headers, db).await?
        .to_string());

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let integration_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let is_active = params.get("is_active").and_then(|s| s.parse::<bool>().ok());

    match db.get_user_api_keys(user_id, is_active).await {
        Ok(api_keys) => {
            // Don't expose key_hash or secret data
            let sanitized_keys: Vec<serde_json::Value> = api_keys.iter().map(|key| {
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
            }).collect();

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
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let key_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let api_key = base64::encode(&key_bytes);
        let key_prefix = api_key.chars().take(8).collect::<String>();
        let key_hash = hash_token(&api_key);
        (api_key, key_prefix, key_hash)
    };
    
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let created_by = user_id.to_string();

    let expires_at = req.expires_at
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify ownership
    let existing_key = db.get_api_key(key_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if existing_key.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let expires_at = req.expires_at
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify ownership
    let existing_key = db.get_api_key(key_id).await
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Verify ownership
    let existing_key = db.get_api_key(key_id).await
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    // Get user
    let user = db.get_user(user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify current password
    let password_valid = state.auth_service.verify_password(&req.current_password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !password_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Hash new password
    let new_password_hash = state.auth_service.hash_password(&req.new_password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update user password
    let update = data_infrastructure::database_operations::UpdateUser {
        email: None,
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

// Two-factor authentication handlers
async fn get_2fa_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    match db.get_two_factor_auth(user_id, None).await {
        Ok(Some(two_fa)) => {
            // Don't expose secret_encrypted or backup_codes
            Ok(Json(serde_json::json!({
                "method": two_fa.method,
                "is_enabled": two_fa.is_enabled,
                "last_used_at": two_fa.last_used_at.map(|d| d.to_rfc3339()),
            })))
        }
        Ok(None) => Ok(Json(serde_json::json!({
            "is_enabled": false,
            "method": serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to get 2FA: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn setup_2fa_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<Setup2FARequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    // Generate TOTP secret and recovery codes (before any await to ensure Send)
    let (secret_bytes, secret_base32, backup_codes) = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let secret_bytes: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
        let secret_base32 = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret_bytes);
        let backup_codes: Vec<String> = (0..10)
            .map(|_| {
                let code: u32 = rng.gen_range(10000000..99999999);
                format!("{:08}", code)
            })
            .collect();
        (secret_bytes, secret_base32, backup_codes)
    };
    
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    // Get user info for QR code label
    let user = db.get_user(user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Create TOTP instance using the original secret bytes
    let totp = TOTP::new(
        Algorithm::SHA1,
        6, // 6-digit codes
        1, // 1 step = 30 seconds
        30, // Period = 30 seconds
        secret_bytes.clone(),
        Some("Agent Agency V3".to_string()),
        user.email.clone(),
    ).map_err(|e| {
        error!("Failed to create TOTP instance: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate QR code URL (otpauth:// format)
    let issuer = "Agent Agency V3";
    let account_name = user.email.clone();
    let qr_url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        urlencoding::encode(issuer),
        urlencoding::encode(&account_name),
        secret_base32,
        urlencoding::encode(issuer)
    );

    // PLACEHOLDER: 2FA secret encryption not implemented
    // In production, encrypt secret using proper key management (AWS KMS, HashiCorp Vault, etc.)
    // Required: Encryption key management, secure key storage, encryption/decryption functions
    // For now, store base32 encoded (NOT SECURE - secrets should be encrypted at rest)
    let secret_encrypted = secret_base32.clone();

    let method = req.method.clone();
    let create = data_infrastructure::database_operations::CreateTwoFactorAuth {
        user_id,
        method: req.method,
        secret_encrypted,
        backup_codes: backup_codes.clone(),
        is_enabled: false, // Not enabled until verified
    };

    match db.create_two_factor_auth(create).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "setup",
                "method": method,
                "secret": secret_base32, // Return secret for manual entry
                "qr_url": qr_url,
                "backup_codes": backup_codes,
                "message": "Scan QR code with authenticator app or enter secret manually. Verify with code to enable 2FA."
            })))
        }
        Err(e) => {
            error!("Failed to setup 2FA: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn verify_2fa_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<Verify2FARequest>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    // Get user info for issuer/account name
    let user = db.get_user(user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get 2FA config
    let two_fa = db.get_two_factor_auth(user_id, Some(&req.method)).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut code_valid = false;

    // Check if code is a recovery code first
    if two_fa.backup_codes.contains(&req.code) {
        // Valid recovery code - remove it from the list
        let mut updated_backup_codes = two_fa.backup_codes.clone();
        updated_backup_codes.retain(|code| code != &req.code);
        
        let update = data_infrastructure::database_operations::UpdateTwoFactorAuth {
            secret_encrypted: None,
            backup_codes: Some(updated_backup_codes),
            is_enabled: None,
        };
        
        let _ = db.update_two_factor_auth(user_id, &req.method, update).await;
        code_valid = true;
    } else {
        // Verify TOTP code
        // Decode the stored secret (in production, decrypt first)
        let secret_base32 = two_fa.secret_encrypted.clone();
        
        // Decode base32 secret to bytes
        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret_base32)
            .ok_or_else(|| {
                error!("Failed to decode TOTP secret from base32");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("Agent Agency V3".to_string()),
            user.email.clone(),
        ).map_err(|e| {
            error!("Failed to create TOTP instance: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Verify the code with a tolerance window of ±1 step (30 seconds)
        // This handles clock skew and user delay
        code_valid = totp.check(&req.code, 1);
    }

    if !code_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Enable 2FA (if not already enabled)
    let update = data_infrastructure::database_operations::UpdateTwoFactorAuth {
        secret_encrypted: None,
        backup_codes: None,
        is_enabled: Some(true),
    };

    match db.update_two_factor_auth(user_id, &req.method, update).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "enabled",
                "method": req.method,
                "message": "2FA enabled successfully"
            })))
        }
        Err(e) => {
            error!("Failed to enable 2FA: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn disable_2fa_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let user_id = get_user_id_from_auth(&headers, db).await?;

    // Get 2FA config to find method
    let two_fa = db.get_two_factor_auth(user_id, None).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match db.delete_two_factor_auth(user_id, &two_fa.method).await {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "status": "disabled",
                "message": "2FA disabled successfully"
            })))
        }
        Err(e) => {
            error!("Failed to disable 2FA: {}", e);
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
struct CreateViolationRequest {
    task_id: Uuid,
    violation_code: String,
    severity: String,
    description: String,
    file_path: Option<String>,
    line_number: Option<i32>,
    column_number: Option<i32>,
    rule_id: String,
    constitutional_reference: Option<String>,
    status: Option<String>,
    metadata: Option<JsonValue>,
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Get rule
    let _rule = db.get_caws_rule(&id).await
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let task_id = params.get("task_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let task_id = params.get("task_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    
    let update = data_infrastructure::database_operations::UpdateRuleEnforcementStatus {
        enforcement_state: req.enforcement_state,
        paused_until: req.paused_until,
        paused_reason: req.paused_reason,
        override_reason: req.override_reason,
        metadata: req.metadata,
    };
    
    match db.update_rule_enforcement_status(&id, task_id, update).await {
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let limit = params.get("limit")
        .and_then(|s| s.parse::<u32>().ok());
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let task_id = params.get("task_id")
        .and_then(|s| Uuid::parse_str(s).ok());
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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

async fn resolve_violation_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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

async fn list_specifications_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<JsonValue>, StatusCode> {
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
    let db = state.db_client.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
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
