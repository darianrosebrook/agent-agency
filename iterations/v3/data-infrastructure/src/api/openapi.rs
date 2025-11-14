//! OpenAPI Documentation
//!
//! Generates OpenAPI 3.0 specification for the Agent Agency API.
//!
//! @author @darianrosebrook

use crate::api::api_errors::ErrorResponse;
use crate::api::handlers::auth_handlers::{LoginRequest, LoginResponse, RefreshTokenRequest, UserResponse};
use crate::api::openapi_paths;
use crate::api::types::{TaskResultResponse, TaskStatusResponse, TaskSubmissionRequest, TaskSubmissionResponse};
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
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "System health and status endpoints"),
        (name = "Tasks", description = "Task management and execution endpoints"),
        (name = "Chat", description = "Chat sessions and messaging endpoints"),
        (name = "Authentication", description = "User authentication and authorization"),
        (name = "Provenance", description = "Code provenance and audit tracking"),
        (name = "System", description = "System monitoring and metrics"),
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
    SwaggerUi::new("/swagger-ui/{_:.*}")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .config(utoipa_swagger_ui::Config::new(["/api-docs/openapi.json"]))
}

/// Handler to serve OpenAPI JSON spec
pub async fn get_openapi_spec() -> axum::response::Json<utoipa::openapi::OpenApi> {
    axum::response::Json(ApiDoc::openapi())
}
