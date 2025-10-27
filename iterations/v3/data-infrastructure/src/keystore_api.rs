/**
 * Keystore API Endpoints - P0-8 Implementation
 *
 * REST API endpoints for keystore operations with proper authentication and audit logging.
 */

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

use crate::AppState;
use crate::audit::extract_audit_context;
use system_quality_security::{Keystore, KeyType, KeyPermission, KeystoreResult, KeyMetadata};

/// Request to store a new key
#[derive(Debug, Deserialize)]
pub struct StoreKeyRequest {
    pub name: String,
    pub key_type: KeyType,
    pub value: String, // Base64 encoded
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub permissions: Vec<KeyPermission>,
    pub expires_at: Option<String>, // ISO 8601 datetime
}

/// Request to update a key
#[derive(Debug, Deserialize)]
pub struct UpdateKeyRequest {
    pub value: Option<String>, // Base64 encoded
    pub permissions: Option<Vec<KeyPermission>>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub expires_at: Option<String>, // ISO 8601 datetime
}

/// Query parameters for listing keys
#[derive(Debug, Deserialize)]
pub struct ListKeysQuery {
    pub owner: Option<String>,
    pub key_type: Option<KeyType>,
    pub tags: Option<String>, // Comma-separated
}

/// API response for successful operations
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Store a new key
pub async fn store_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Json(request): Json<StoreKeyRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    // Decode the base64 value
    let key_bytes = match general_purpose::STANDARD.decode(&request.value) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid base64 encoding".to_string()),
            }));
        }
    };

    // Parse expires_at if provided
    let expires_at = if let Some(expires_str) = &request.expires_at {
        match chrono::DateTime::parse_from_rfc3339(expires_str) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return Ok(Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid expires_at format".to_string()),
                }));
            }
        }
    } else {
        None
    };

    // Store the key
    match state.keystore.store_key(
        &request.name,
        request.key_type,
        &key_bytes,
        &audit_context.user_id.unwrap_or_else(|| "anonymous".to_string()),
        request.permissions,
        request.description.as_deref(),
        request.tags,
        expires_at,
    ).await {
        Ok(key_id) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "key_id": key_id.to_string(),
                    "name": request.name,
                    "created_at": chrono::Utc::now().to_rfc3339()
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to store key: {:?}", e)),
            }))
        }
    }
}

/// Retrieve a key
pub async fn get_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(key_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    let uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid key ID format".to_string()),
            }));
        }
    };

    match state.keystore.get_key(&uuid, &audit_context.user_id.unwrap_or_else(|| "anonymous".to_string())).await {
        Ok(key_bytes) => {
            let encoded_value = general_purpose::STANDARD.encode(&key_bytes);
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "key_id": key_id,
                    "value": encoded_value
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to retrieve key: {:?}", e)),
            }))
        }
    }
}

/// Update an existing key
pub async fn update_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(key_id): Path<String>,
    Json(request): Json<UpdateKeyRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    let uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid key ID format".to_string()),
            }));
        }
    };

    // Decode value if provided
    let key_bytes = if let Some(ref value) = request.value {
        match general_purpose::STANDARD.decode(value) {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                return Ok(Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid base64 encoding".to_string()),
                }));
            }
        }
    } else {
        None
    };

    // Parse expires_at if provided
    let expires_at = if let Some(expires_str) = &request.expires_at {
        match chrono::DateTime::parse_from_rfc3339(expires_str) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return Ok(Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid expires_at format".to_string()),
                }));
            }
        }
    } else {
        None
    };

    match state.keystore.update_key(
        &uuid,
        key_bytes.as_deref(),
        request.permissions,
        request.description.as_deref(),
        request.tags,
        expires_at,
        &audit_context.user_id.unwrap_or_else(|| "anonymous".to_string()),
    ).await {
        Ok(_) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "key_id": key_id,
                    "updated_at": chrono::Utc::now().to_rfc3339()
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to update key: {:?}", e)),
            }))
        }
    }
}

/// Delete a key
pub async fn delete_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(key_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    let uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid key ID format".to_string()),
            }));
        }
    };

    match state.keystore.delete_key(&uuid, &audit_context.user_id.unwrap_or_else(|| "anonymous".to_string())).await {
        Ok(_) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "key_id": key_id,
                    "deleted_at": chrono::Utc::now().to_rfc3339()
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to delete key: {:?}", e)),
            }))
        }
    }
}

/// List keys with filtering
pub async fn list_keys(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Query(query): Query<ListKeysQuery>,
) -> Result<Json<ApiResponse<Vec<KeyMetadata>>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    // Parse tags if provided
    let tags: Option<Vec<String>> = query.tags.as_ref()
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    match state.keystore.list_keys(
        query.owner.as_deref(),
        query.key_type.as_ref(),
        tags.as_deref(),
        &audit_context.user_id.unwrap_or_else(|| "anonymous".to_string()),
    ).await {
        Ok(keys) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(keys),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to list keys: {:?}", e)),
            }))
        }
    }
}

/// Get key metadata
pub async fn get_key_metadata(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(key_id): Path<String>,
) -> Result<Json<ApiResponse<KeyMetadata>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    let uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid key ID format".to_string()),
            }));
        }
    };

    match state.keystore.get_key_metadata(&uuid, &audit_context.user_id.unwrap_or_else(|| "anonymous".to_string())).await {
        Ok(metadata) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(metadata),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to get key metadata: {:?}", e)),
            }))
        }
    }
}

/// Rotate a key
pub async fn rotate_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(key_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let audit_context = extract_audit_context(&headers, Some(addr));

    let uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid key ID format".to_string()),
            }));
        }
    };

    match state.keystore.rotate_key(&uuid, &audit_context.user_id.unwrap_or_else(|| "anonymous".to_string())).await {
        Ok(new_key_id) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "old_key_id": key_id,
                    "new_key_id": new_key_id.to_string(),
                    "rotated_at": chrono::Utc::now().to_rfc3339()
                })),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to rotate key: {:?}", e)),
            }))
        }
    }
}

/// Create keystore API router
pub fn create_keystore_router() -> Router<AppState> {
    Router::new()
        .route("/keys", post(store_key))
        .route("/keys", get(list_keys))
        .route("/keys/:key_id", get(get_key))
        .route("/keys/:key_id", put(update_key))
        .route("/keys/:key_id", delete(delete_key))
        .route("/keys/:key_id/metadata", get(get_key_metadata))
        .route("/keys/:key_id/rotate", post(rotate_key))
}
