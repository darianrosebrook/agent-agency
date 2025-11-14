//! OpenAPI Documentation
//!
//! Generates OpenAPI 3.0 specification for the Agent Agency API.
//!
//! @author @darianrosebrook

use crate::api::api_errors::ErrorResponse;
use crate::api::api_types::{LinkProvenanceRequest, SaveQueryRequest, WaiverApprovalRequest, WaiverRequest};
use crate::api::handlers::auth_handlers::{LoginRequest, LoginResponse, RefreshTokenRequest, UserResponse};
use crate::api::openapi_paths;
use crate::api::types::{TaskStatusResponse, TaskSubmissionRequest, TaskSubmissionResponse};
use crate::chat_service::{ChatMessage, ChatSession};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI specification for Agent Agency API
#[derive(OpenApi)]
#[openapi(
    paths(
        openapi_paths::health_check_doc,
        openapi_paths::system_health_check_doc,
        openapi_paths::submit_task_doc,
        openapi_paths::list_tasks_doc,
        openapi_paths::get_task_status_doc,
        openapi_paths::get_task_result_doc,
        openapi_paths::cancel_task_doc,
        openapi_paths::pause_task_doc,
        openapi_paths::resume_task_doc,
        openapi_paths::get_chain_of_thought_doc,
        openapi_paths::get_council_decisions_doc,
        openapi_paths::get_worker_actions_doc,
        openapi_paths::list_chat_sessions_doc,
        openapi_paths::create_chat_session_doc,
        openapi_paths::get_chat_messages_doc,
        openapi_paths::send_chat_message_doc,
        openapi_paths::login_doc,
        openapi_paths::logout_doc,
        openapi_paths::refresh_token_doc,
        openapi_paths::get_current_user_doc,
        openapi_paths::list_provenance_doc,
        openapi_paths::get_provenance_by_commit_doc,
        openapi_paths::get_system_metrics_doc,
        openapi_paths::list_projects_doc,
        openapi_paths::get_project_doc,
        openapi_paths::get_project_tasks_doc,
        openapi_paths::list_database_tables_doc,
        openapi_paths::get_table_schema_doc,
        openapi_paths::execute_query_doc,
        openapi_paths::get_task_analytics_doc,
        openapi_paths::get_performance_analytics_doc,
        openapi_paths::get_success_rates_doc,
        openapi_paths::list_agents_doc,
        openapi_paths::get_agent_doc,
        openapi_paths::get_agent_stats_doc,
        openapi_paths::get_agent_health_doc,
        openapi_paths::get_agent_metrics_doc,
        openapi_paths::get_agent_logs_doc,
        openapi_paths::get_agents_stats_doc,
        openapi_paths::get_agents_tasks_completion_doc,
        openapi_paths::get_agents_efficiency_doc,
        openapi_paths::get_tasks_stats_doc,
        openapi_paths::get_tasks_stats_history_doc,
        openapi_paths::get_task_logs_doc,
        openapi_paths::get_task_progress_doc,
        openapi_paths::get_task_events_doc,
        openapi_paths::get_task_comments_doc,
        openapi_paths::create_task_comment_doc,
        openapi_paths::get_task_provenance_doc,
        openapi_paths::list_judges_doc,
        openapi_paths::get_judge_doc,
        openapi_paths::get_judges_stats_doc,
        openapi_paths::get_judge_stats_doc,
        openapi_paths::get_judge_evaluations_doc,
        openapi_paths::get_contributions_doc,
        openapi_paths::get_model_contributions_doc,
        openapi_paths::get_agent_activity_doc,
        openapi_paths::get_efficiency_doc,
        openapi_paths::get_observability_system_metrics_doc,
        openapi_paths::get_alerts_doc,
        openapi_paths::get_system_health_doc,
        openapi_paths::get_system_resources_doc,
        openapi_paths::get_session_status_doc,
        openapi_paths::pause_session_doc,
        openapi_paths::resume_session_doc,
        openapi_paths::cancel_session_doc,
        openapi_paths::get_chat_session_doc,
        openapi_paths::search_doc,
        openapi_paths::list_queries_doc,
        openapi_paths::save_query_doc,
        openapi_paths::delete_query_doc,
        openapi_paths::get_query_performance_summary_doc,
        openapi_paths::get_query_performance_metrics_doc,
        openapi_paths::get_slow_queries_doc,
        openapi_paths::get_top_slow_queries_doc,
        openapi_paths::link_provenance_doc,
        openapi_paths::verify_provenance_doc,
        openapi_paths::list_waivers_doc,
        openapi_paths::create_waiver_doc,
        openapi_paths::approve_waiver_doc,
        openapi_paths::list_slos_doc,
        openapi_paths::get_slo_status_doc,
        openapi_paths::get_slo_measurements_doc,
        openapi_paths::list_slo_alerts_doc,
    ),
    components(schemas(
        TaskSubmissionRequest,
        TaskSubmissionResponse,
        TaskStatusResponse,
        ChatSession,
        ChatMessage,
        LoginRequest,
        LoginResponse,
        RefreshTokenRequest,
        UserResponse,
        ErrorResponse,
        openapi_paths::CreateChatSessionRequest,
        openapi_paths::SendChatMessageRequest,
        openapi_paths::ExecuteQueryRequest,
        openapi_paths::CreateTaskCommentRequest,
        LinkProvenanceRequest,
        SaveQueryRequest,
        WaiverRequest,
        WaiverApprovalRequest,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "System health and status endpoints"),
        (name = "Tasks", description = "Task management and execution endpoints"),
        (name = "Chat", description = "Chat sessions and messaging endpoints"),
        (name = "Authentication", description = "User authentication and authorization"),
        (name = "Provenance", description = "Code provenance and audit tracking"),
        (name = "System", description = "System monitoring and metrics"),
        (name = "Judges", description = "Judge management and evaluation endpoints"),
        (name = "Telemetry", description = "Telemetry and contribution tracking"),
        (name = "Observability", description = "System observability and efficiency metrics"),
        (name = "Sessions", description = "Session control and management"),
        (name = "Search", description = "Search functionality"),
        (name = "Queries", description = "Query management and saved queries"),
        (name = "Query Performance", description = "Database query performance monitoring"),
        (name = "Waivers", description = "Quality gate waiver management"),
        (name = "SLOs", description = "Service level objective management"),
    ),
    info(
        title = "Agent Agency API",
        description = "REST API for Agent Agency - AI agent orchestration and management platform",
        version = "3.0.0",
        contact(
            name = "Agent Agency",
            email = "support@agent-agency.dev"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.agent-agency.dev", description = "Production server")
    )
)]
pub struct ApiDoc;

/// Security scheme for API key authentication
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
            components.add_security_scheme(
                "bearer",
                SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// Create Swagger UI router for interactive API documentation
pub fn create_swagger_ui() -> SwaggerUi {
    // Use a unique path to avoid conflicts with other routes
    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}

/// Handler to serve OpenAPI JSON spec
pub async fn get_openapi_spec() -> axum::response::Json<utoipa::openapi::OpenApi> {
    axum::response::Json(ApiDoc::openapi())
}
