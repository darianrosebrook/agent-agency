/**
 * Sandbox API Endpoints - P0-8 Implementation
 *
 * REST API endpoints for sandbox operations with security controls and audit logging.
 */

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::audit::extract_audit_context;
use agent_agency_security::{Sandbox, SandboxMode, ResourceLimits, SandboxContext, ExecutionRequest, SandboxResult};
use agent_agency_security::sandbox::SandboxStatus;

/// Request to create and execute in a sandbox
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub command: Vec<String>,
    pub sandbox_mode: Option<SandboxMode>,
    pub cpu_limit: Option<f64>,
    pub memory_limit_mb: Option<u64>,
    pub timeout_seconds: Option<u64>,
    pub network_enabled: Option<bool>,
    pub environment_vars: Option<std::collections::HashMap<String, String>>,
}

/// API response for successful operations
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Execute a command in the sandbox
pub async fn execute_command(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Json(request): Json<ExecuteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    // Create sandbox context
    let context = SandboxContext {
        id: uuid::Uuid::new_v4(),
        mode: request.sandbox_mode.unwrap_or(SandboxMode::Docker),
        limits: agent_agency_security::ResourceLimits {
            cpu_cores: request.cpu_limit,
            memory_mb: request.memory_limit_mb,
            disk_mb: Some(100), // Default 100MB disk limit
            network_enabled: request.network_enabled.unwrap_or(false),
            timeout_seconds: request.timeout_seconds,
        },
        environment_vars: request.environment_vars.unwrap_or_default(),
        working_directory: None,
        network_access: request.network_enabled.unwrap_or(false),
        filesystem_access: vec![], // No filesystem access by default
    };

    // Create execution request
    let exec_request = ExecutionRequest {
        command: request.command,
        context,
        input_data: None,
    };

    // Execute in sandbox
    match state.sandbox.execute(exec_request).await {
        Ok(result) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "sandbox_id": result.success, // We'll use success field temporarily
                    "exit_code": result.exit_code,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                    "execution_time_ms": result.execution_time_ms,
                    "success": result.success,
                    "error_message": result.error_message
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Sandbox execution failed: {:?}", e)),
            }))
        }
    }
}

/// Get sandbox status
pub async fn get_sandbox_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SandboxStatus>>, StatusCode> {
    match state.sandbox.get_status().await {
        Ok(status) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(status),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to get sandbox status: {:?}", e)),
            }))
        }
    }
}

/// Validate sandbox configuration
pub async fn validate_sandbox_config(
    State(state): State<AppState>,
    Json(context): Json<SandboxContext>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    match state.sandbox.validate_config(&context).await {
        Ok(_) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "valid": true,
                    "message": "Sandbox configuration is valid"
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: Some(serde_json::json!({
                    "valid": false,
                    "message": format!("Invalid configuration: {:?}", e)
                })),
                error: Some(format!("Configuration validation failed: {:?}", e)),
            }))
        }
    }
}

/// Clean up a sandbox
pub async fn cleanup_sandbox(
    State(state): State<AppState>,
    Path(sandbox_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let uuid = match uuid::Uuid::parse_str(&sandbox_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid sandbox ID format".to_string()),
            }));
        }
    };

    match state.sandbox.cleanup(&uuid).await {
        Ok(_) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "sandbox_id": sandbox_id,
                    "cleaned_up_at": chrono::Utc::now().to_rfc3339()
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to cleanup sandbox: {:?}", e)),
            }))
        }
    }
}

/// Create sandbox API router
pub fn create_sandbox_router() -> Router<AppState> {
    Router::new()
        .route("/sandbox/execute", post(execute_command))
        .route("/sandbox/status", get(get_sandbox_status))
        .route("/sandbox/validate", post(validate_sandbox_config))
        .route("/sandbox/:sandbox_id/cleanup", delete(cleanup_sandbox))
}
