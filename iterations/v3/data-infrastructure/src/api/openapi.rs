//! OpenAPI Documentation
//!
//! Generates OpenAPI 3.0 specification for the Agent Agency API.
//!
//! @author @darianrosebrook

use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::openapi::{PathItem, Paths, Response, Responses};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI specification for Agent Agency API
#[derive(OpenApi)]
#[openapi(
    paths(),
    components(schemas()),
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
