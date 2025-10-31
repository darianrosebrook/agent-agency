//! System and Monitoring API handlers
//! 
//! This module contains all API handlers related to system monitoring,
//! metrics, dashboard data, and proxy operations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json;
use tracing::{info, error};
use uuid::Uuid;

use crate::api::ApiState;

/// Get task provenance
pub async fn get_task_provenance(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_uuid = uuid::Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.get_task_provenance(&task_uuid).await {
        Ok(provenance_records) => {
            let provenance_list: Vec<serde_json::Value> = provenance_records.into_iter().map(|record| {
                serde_json::json!({
                    "id": record.id,
                    "task_id": record.task_id,
                    "action": record.action,
                    "timestamp": record.timestamp,
                    "actor": record.actor,
                    "resource_id": record.resource_id,
                    "resource_type": record.resource_type,
                    "change_summary": record.change_summary,
                    "metadata": record.metadata
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "task_id": task_id,
                "provenance": provenance_list,
                "total_records": provenance_list.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get task provenance for {}: {}", task_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Proxy handler for forwarding requests
pub async fn proxy_handler(
    State(_state): State<ApiState>,
    Json(request_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let target_url = request_data.get("url")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let method = request_data.get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET");
    
    let headers = request_data.get("headers")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<std::collections::HashMap<String, String>>()
        })
        .unwrap_or_default();
    
    let body = request_data.get("body");

    // Validate URL for security (prevent SSRF attacks)
    let url = reqwest::Url::parse(target_url)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Only allow HTTP/HTTPS protocols
    if url.scheme() != "http" && url.scheme() != "https" {
        error!("Invalid URL scheme: {}", url.scheme());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            error!("Failed to create HTTP client: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Build request based on method
    let mut request_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => {
            error!("Unsupported HTTP method: {}", method);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Add headers
    for (key, value) in headers {
        request_builder = request_builder.header(&key, value);
    }

    // Add body if present
    if let Some(body_val) = body {
        if let Some(body_str) = body_val.as_str() {
            request_builder = request_builder.body(body_str.to_string());
        } else {
            request_builder = request_builder.json(body_val);
        }
    }

    // Execute request
    match request_builder.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            
            // Read response headers
            let response_headers: serde_json::Value = response.headers()
                .iter()
                .filter_map(|(k, v)| {
                    v.to_str().ok().map(|val| (k.to_string(), serde_json::Value::String(val.to_string())))
                })
                .collect();

            // Read response body
            let body_text = response.text().await.unwrap_or_default();
            let body_json: serde_json::Value = serde_json::from_str(&body_text)
                .unwrap_or_else(|_| serde_json::Value::String(body_text));

            Ok(Json(serde_json::json!({
                "status_code": status_code,
                "headers": response_headers,
                "body": body_json,
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Proxy request failed: {}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

/// Get system metrics
pub async fn get_metrics(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.api.db_client.get_system_metrics().await {
        Ok(metrics) => {
            Ok(Json(serde_json::json!({
                "timestamp": chrono::Utc::now(),
                "metrics": {
                    "cpu_usage": 0.0, // Default value
                    "memory_usage": 0.0, // Default value
                    "disk_usage": 0.0, // Default value
                    "network_io": 0.0, // Default value
                    "active_connections": 0, // Default value
                    "request_rate": 0.0, // Default value
                    "error_rate": 0.0, // Default value
                    "response_time_p95": 0.0, // Default value
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get system metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get dashboard data
pub async fn get_dashboard_data(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.api.db_client.get_dashboard_data().await {
        Ok(dashboard_data) => {
            Ok(Json(serde_json::json!({
                "timestamp": chrono::Utc::now(),
                "dashboard": {
                    "system_health": "healthy", // Default value
                    "active_tasks": dashboard_data["metrics"]["active_tasks"],
                    "completed_tasks": dashboard_data["metrics"]["total_tasks"],
                    "failed_tasks": 0, // Default value
                    "active_workers": 1, // Default value
                    "queue_depth": 0, // Default value
                    "throughput": 0.0, // Default value
                    "error_rate": 0.0, // Default value
                    "recent_activity": dashboard_data["recent_tasks"],
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get dashboard data: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get diff summary
pub async fn get_diff_summary(
    State(_state): State<ApiState>,
    Json(diff_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let old_content = diff_data.get("old_content")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let new_content = diff_data.get("new_content")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let context = diff_data.get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // DEPENDENCY: AI service for diff summary generation not yet available
    // When integrated, this should:
    // 1. Send old_content, new_content, and context to AI service
    // 2. Request structured analysis including:
    //    - Summary of changes
    //    - Categorized change list (additions, deletions, modifications)
    //    - Impact assessment (breaking changes, performance, security)
    //    - Recommendations for review or testing
    //    - Confidence score for the analysis
    // 3. Return structured response with all analysis components
    //
    // Real implementation requires:
    // - AI service client (e.g., OpenAI, Anthropic, or local LLM service)
    // - Prompt engineering for diff analysis
    // - Structured output parsing
    // - Error handling and fallback mechanisms
    //
    // For now, return error indicating dependency is not available
    error!("Diff summary generation requested but AI service not available. Context: {}", context.chars().take(100).collect::<String>());
    
    Err(StatusCode::NOT_IMPLEMENTED)
}
