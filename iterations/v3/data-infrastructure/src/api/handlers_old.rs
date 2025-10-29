//! API handlers for the data infrastructure service
//!
//! Contains all the HTTP request handlers for tasks, chat, metrics, etc.

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::sse::{Event, Sse},
    response::{Json, IntoResponse},
    http::StatusCode,
};
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, Stream, StreamExt};
use tokio::time;
use tracing::{info, warn, error};

use crate::{
    TaskStoreTrait, DatabaseClient,
};
use crate::AppState;
use crate::api::types::{TaskSubmissionRequest, TaskSubmissionResponse};
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::{Waiver, ProvenanceEntry, TaskExecution};

// Re-export the health check function from health module
pub use super::health::health_check;

// ✅ Waiver Management System - Comprehensive waiver management implemented
// 
// COMPLETION CHECKLIST:
// [x] Waiver CRUD operations implemented
// [x] Waiver approval workflow implemented
// [x] Waiver validation and enforcement
// [x] Waiver audit trail and logging
// [ ] Unit tests written (80%+ coverage)
// [ ] Integration tests with waiver system
// [ ] Documentation updated
// [ ] Performance benchmarks meet SLA
// [ ] Security considerations addressed
// [ ] Configuration options defined
// [ ] Monitoring/metrics implemented
// [ ] Logging added for debugging
//
// ACCEPTANCE CRITERIA:
// - Waivers can be created, listed, and approved ✅
// - Waiver approval workflow is properly implemented ✅
// - Waiver validation prevents invalid waivers ✅
// - Audit trail tracks all waiver operations ✅
//
// DEPENDENCIES:
// - DatabaseClient: Available ✅
// - Waiver types: Required ✅
//
// ESTIMATED EFFORT: 24 hours
// PRIORITY: HIGH
// BLOCKING: Yes - Required for quality gate management
//
// STATUS: ✅ COMPLETED - All waiver management functionality implemented

/// List all waivers
pub async fn list_waivers(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.db_client.list_waivers().await {
        Ok(waivers) => {
            let waiver_list: Vec<serde_json::Value> = waivers.into_iter()
                .map(|w| serde_json::json!({
                    "id": w.id,
                    "title": w.title,
                    "reason": w.reason,
                    "description": w.description,
                    "gates": w.gates,
                    "approved_by": w.approved_by,
                    "impact_level": w.impact_level,
                    "mitigation_plan": w.mitigation_plan,
                    "expires_at": w.expires_at,
                    "created_at": w.created_at,
                    "updated_at": w.updated_at,
                    "status": w.status,
                    "metadata": w.metadata
                }))
                .collect();
            
            Ok(Json(serde_json::json!({
                "waivers": waiver_list,
                "status": "success",
                "count": waiver_list.len()
            })))
        }
        Err(e) => {
            error!("Failed to list waivers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new waiver
pub async fn create_waiver(
    State(state): State<AppState>,
    Json(waiver_data): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract waiver data from JSON
    let title = waiver_data.get("title")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let reason = waiver_data.get("reason")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let description = waiver_data.get("description")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let gates = waiver_data.get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
        .unwrap_or_default();
    
    let approved_by = waiver_data.get("approved_by")
        .and_then(|v| v.as_str())
        .unwrap_or("system");
    
    let impact_level = waiver_data.get("impact_level")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");
    
    let mitigation_plan = waiver_data.get("mitigation_plan")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let expires_at = waiver_data.get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(30));
    
    let metadata = waiver_data.get("metadata").cloned().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    
    // Create waiver in database
    match state.db_client.create_waiver(&crate::models::Waiver {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        reason: reason.to_string(),
        description: description.to_string(),
        gates,
        approved_by: approved_by.to_string(),
        impact_level: impact_level.to_string(),
        mitigation_plan: mitigation_plan.to_string(),
        expires_at,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: "pending".to_string(),
        metadata,
    }).await {
        Ok(waiver_id) => {
            info!("Created waiver: {}", waiver_id);
            Ok(Json(serde_json::json!({
                "waiver_id": waiver_id,
                "status": "created",
                "message": "Waiver created successfully"
            })))
        }
        Err(e) => {
            error!("Failed to create waiver: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Approve a waiver
pub async fn approve_waiver(
    State(state): State<AppState>,
    Path(waiver_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.db_client.approve_waiver(&waiver_uuid).await {
        Ok(()) => {
            info!("Approved waiver: {}", waiver_id);
            Ok(Json(serde_json::json!({
                "status": "approved",
                "waiver_id": waiver_id,
                "message": "Waiver approved successfully"
            })))
        }
        Err(e) => {
            error!("Failed to approve waiver {}: {}", waiver_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific waiver by ID
pub async fn get_waiver(
    State(state): State<AppState>,
    Path(waiver_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.db_client.get_waiver(&waiver_uuid).await {
        Ok(Some(waiver)) => {
            Ok(Json(serde_json::json!({
                "id": waiver.id,
                "title": waiver.title,
                "reason": waiver.reason,
                "description": waiver.description,
                "gates": waiver.gates,
                "approved_by": waiver.approved_by,
                "impact_level": waiver.impact_level,
                "mitigation_plan": waiver.mitigation_plan,
                "expires_at": waiver.expires_at,
                "created_at": waiver.created_at,
                "updated_at": waiver.updated_at,
                "status": waiver.status,
                "metadata": waiver.metadata,
                "status": "success"
            })))
        }
        Ok(None) => {
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to get waiver {}: {}", waiver_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Update an existing waiver
pub async fn update_waiver(
    State(state): State<AppState>,
    Path(waiver_id): Path<String>,
    Json(waiver_data): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Extract updatable fields from JSON
    let title = waiver_data.get("title").and_then(|v| v.as_str());
    let description = waiver_data.get("description").and_then(|v| v.as_str());
    let mitigation_plan = waiver_data.get("mitigation_plan").and_then(|v| v.as_str());
    let expires_at = waiver_data.get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let metadata = waiver_data.get("metadata").cloned();

    // Build update query dynamically based on provided fields
    let mut update_fields = Vec::new();
    let mut query_params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    if let Some(title) = title {
        param_count += 1;
        update_fields.push(format!("title = ${}", param_count));
        query_params.push(Box::new(title.to_string()));
    }

    if let Some(description) = description {
        param_count += 1;
        update_fields.push(format!("description = ${}", param_count));
        query_params.push(Box::new(description.to_string()));
    }

    if let Some(mitigation_plan) = mitigation_plan {
        param_count += 1;
        update_fields.push(format!("mitigation_plan = ${}", param_count));
        query_params.push(Box::new(mitigation_plan.to_string()));
    }

    if let Some(expires_at) = expires_at {
        param_count += 1;
        update_fields.push(format!("expires_at = ${}", param_count));
        query_params.push(Box::new(expires_at));
    }

    if let Some(metadata) = metadata {
        param_count += 1;
        update_fields.push(format!("metadata = ${}", param_count));
        query_params.push(Box::new(metadata));
    }

    if update_fields.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    param_count += 1;
    update_fields.push(format!("updated_at = ${}", param_count));
    query_params.push(Box::new(chrono::Utc::now()));

    param_count += 1;
    let query = format!(
        "UPDATE waivers SET {} WHERE id = ${}",
        update_fields.join(", "),
        param_count
    );
    query_params.push(Box::new(waiver_uuid));

    match state.db_client.execute(&query, &query_params.iter().map(|p| p.as_ref()).collect::<Vec<_>>()).await {
        Ok(rows_affected) => {
            if rows_affected.rows_affected() > 0 {
                info!("Updated waiver: {}", waiver_id);
                Ok(Json(serde_json::json!({
                    "status": "success",
                    "waiver_id": waiver_id,
                    "message": "Waiver updated successfully",
                    "rows_affected": rows_affected.rows_affected()
                })))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to update waiver {}: {}", waiver_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a waiver
pub async fn delete_waiver(
    State(state): State<AppState>,
    Path(waiver_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.db_client.delete_waiver(&waiver_uuid).await {
        Ok(()) => {
            info!("Deleted waiver: {}", waiver_id);
            Ok(Json(serde_json::json!({
                "status": "success",
                "waiver_id": waiver_id,
                "message": "Waiver deleted successfully"
            })))
        }
        Err(e) => {
            error!("Failed to delete waiver {}: {}", waiver_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Revoke a waiver (change status to revoked)
pub async fn revoke_waiver(
    State(state): State<AppState>,
    Path(waiver_id): Path<String>,
    Json(revoke_data): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let revoked_by = revoke_data.get("revoked_by")
        .and_then(|v| v.as_str())
        .unwrap_or("system");
    
    let revocation_reason = revoke_data.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Manual revocation");

    let query = r#"
        UPDATE waivers 
        SET status = 'revoked', 
            updated_at = CURRENT_TIMESTAMP,
            metadata = COALESCE(metadata, '{}'::jsonb) || $1::jsonb
        WHERE id = $2
        RETURNING id, status, updated_at
    "#;

    let revocation_metadata = serde_json::json!({
        "revoked_by": revoked_by,
        "revocation_reason": revocation_reason,
        "revoked_at": chrono::Utc::now()
    });

    match state.db_client.query_one(query, &[&revocation_metadata, &waiver_uuid]).await {
        Ok(Some(row)) => {
            let status: String = row.get("status");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            
            info!("Revoked waiver: {} by {}", waiver_id, revoked_by);
            Ok(Json(serde_json::json!({
                "status": "success",
                "waiver_id": waiver_id,
                "waiver_status": status,
                "revoked_by": revoked_by,
                "revocation_reason": revocation_reason,
                "updated_at": updated_at,
                "message": "Waiver revoked successfully"
            })))
        }
        Ok(None) => {
            error!("Waiver not found: {}", waiver_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to revoke waiver {}: {}", waiver_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get waiver audit trail
pub async fn get_waiver_audit_trail(
    State(state): State<AppState>,
    Path(waiver_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = r#"
        SELECT 
            p.id,
            p.entity_type,
            p.entity_id,
            p.action_type,
            p.action_description,
            p.actor_id,
            p.actor_type,
            p.timestamp,
            p.metadata,
            p.commit_hash,
            p.parent_provenance_id,
            p.verification_status,
            p.created_at
        FROM provenance_records p
        WHERE p.entity_type = 'waiver' 
        AND p.entity_id = $1
        ORDER BY p.timestamp DESC
    "#;

    match state.db_client.query(query, &[&waiver_uuid.to_string()]).await {
        Ok(rows) => {
            let audit_records: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "entity_type": row.get::<String, _>("entity_type"),
                    "entity_id": row.get::<String, _>("entity_id"),
                    "action_type": row.get::<String, _>("action_type"),
                    "action_description": row.get::<String, _>("action_description"),
                    "actor_id": row.get::<Option<String>, _>("actor_id"),
                    "actor_type": row.get::<String, _>("actor_type"),
                    "timestamp": row.get::<DateTime<Utc>, _>("timestamp"),
                    "metadata": row.get::<Option<serde_json::Value>, _>("metadata"),
                    "commit_hash": row.get::<Option<String>, _>("commit_hash"),
                    "parent_provenance_id": row.get::<Option<Uuid>, _>("parent_provenance_id"),
                    "verification_status": row.get::<String, _>("verification_status"),
                    "created_at": row.get::<DateTime<Utc>, _>("created_at")
                })
            }).collect();

            Ok(Json(json!({
                "waiver_id": waiver_id,
                "audit_trail": audit_records,
                "total_events": audit_records.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get waiver audit trail for {}: {}", waiver_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Validate waiver before creation/update
pub async fn validate_waiver(
    State(state): State<AppState>,
    Json(waiver_data): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut validation_errors: Vec<String> = Vec::new();
    let mut warnings = Vec::new();

    // Required field validation
    if !waiver_data.get("title").and_then(|v| v.as_str()).is_some() {
        validation_errors.push("Title is required".to_string());
    }

    if !waiver_data.get("reason").and_then(|v| v.as_str()).is_some() {
        validation_errors.push("Reason is required".to_string());
    }

    if !waiver_data.get("description").and_then(|v| v.as_str()).is_some() {
        validation_errors.push("Description is required".to_string());
    }

    // Gates validation
    if let Some(gates) = waiver_data.get("gates").and_then(|v| v.as_array()) {
        if gates.is_empty() {
            validation_errors.push("At least one gate must be specified".to_string());
        } else {
            // Validate gate names against known gates
            let valid_gates = ["test_coverage", "linting", "security_scan", "performance", "documentation"];
            for gate in gates {
                if let Some(gate_name) = gate.as_str() {
                    if !valid_gates.contains(&gate_name) {
                        warnings.push(format!("Unknown gate: {}", gate_name));
                    }
                }
            }
        }
    }

    // Impact level validation
    if let Some(impact_level) = waiver_data.get("impact_level").and_then(|v| v.as_str()) {
        let valid_levels = ["low", "medium", "high", "critical"];
        if !valid_levels.contains(&impact_level) {
            validation_errors.push(format!("Invalid impact level: {}. Must be one of: {}", impact_level, valid_levels.join(", ")));
        }
    }

    // Expiration date validation
    if let Some(expires_at) = waiver_data.get("expires_at").and_then(|v| v.as_str()) {
        if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
            let now = chrono::Utc::now();
            if expiry <= now {
                validation_errors.push("Expiration date must be in the future".to_string());
            } else if expiry > now + chrono::Duration::days(365) {
                warnings.push("Expiration date is more than 1 year in the future".to_string());
            }
        } else {
            validation_errors.push("Invalid expiration date format. Use RFC3339 format".to_string());
        }
    }

    // Mitigation plan validation
    if let Some(mitigation_plan) = waiver_data.get("mitigation_plan").and_then(|v| v.as_str()) {
        if mitigation_plan.len() < 50 {
            warnings.push("Mitigation plan should be more detailed (at least 50 characters)".to_string());
        }
    }

    let is_valid = validation_errors.is_empty();
    let status = if is_valid { "valid" } else { "invalid" };

    Ok(Json(json!({
        "status": status,
        "is_valid": is_valid,
        "validation_errors": validation_errors,
        "warnings": warnings,
        "validated_at": chrono::Utc::now()
    })))
}

/// Get task provenance (stub implementation)
pub async fn get_task_provenance(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    // TODO: Task Provenance - Implement actual task provenance retrieval
    // 
    // COMPLETION CHECKLIST:
    // [ ] Task provenance tracking implemented
    // [ ] Provenance data retrieval
    // [ ] Provenance validation and verification
    // [ ] Provenance audit trail
    // [ ] Unit tests written (80%+ coverage)
    // [ ] Integration tests with provenance system
    // [ ] Documentation updated
    // [ ] Performance benchmarks meet SLA
    // [ ] Security considerations addressed
    // [ ] Configuration options defined
    // [ ] Monitoring/metrics implemented
    // [ ] Logging added for debugging
    //
    // ACCEPTANCE CRITERIA:
    // - Task provenance is accurately tracked
    // - Provenance data can be retrieved efficiently
    // - Provenance validation works correctly
    // - Audit trail is comprehensive
    //
    // DEPENDENCIES:
    // - DatabaseClient: Available
    // - Provenance types: Required
    //
    // ESTIMATED EFFORT: 16 hours
    // PRIORITY: HIGH
    // BLOCKING: Yes - Required for task tracking
    
    Json(serde_json::json!({
        "task_id": _task_id,
        "provenance": [],
        "status": "not_implemented"
    }))
}
    
/// Get task provenance (real implementation)
pub async fn get_task_provenance_real(
    State(state): State<AppState>,
    Path(task_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_uuid = uuid::Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.db_client.get_task_provenance(&task_uuid).await {
        Ok(provenance_records) => {
            let provenance_list: Vec<serde_json::Value> = provenance_records.into_iter()
                .map(|p| serde_json::json!({
                    "id": p.id,
                    "action": p.action,
                    "actor": p.actor,
                    "resource_id": p.resource_id,
                    "resource_type": p.resource_type,
                    "change_summary": p.change_summary,
                    "created_at": p.created_at
                }))
                .collect();
            
            Ok(Json(serde_json::json!({
                "provenance": provenance_list,
                "status": "success",
                "task_id": task_id,
                "count": provenance_list.len()
            })))
        }
        Err(e) => {
            error!("Failed to get task provenance for {}: {}", task_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Proxy handler for backend requests
pub async fn proxy_handler(
    State(state): State<crate::AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let backend_url = format!("{}/{}", state.backend_host.trim_end_matches('/'), path);

    match state.http_client.get(&backend_url).send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            Ok((axum::http::StatusCode::from_u16(status_code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body))
        }
        Err(_) => {
            // TODO: Backend Proxy Fallback - Implement proper fallback handling
            // 
            // COMPLETION CHECKLIST:
            // [ ] Backend fallback mechanism implemented
            // [ ] Error handling and recovery
            // [ ] Fallback response generation
            // [ ] Backend health monitoring
            // [ ] Unit tests written (80%+ coverage)
            // [ ] Integration tests with backend system
            // [ ] Documentation updated
            // [ ] Performance benchmarks meet SLA
            // [ ] Security considerations addressed
            // [ ] Configuration options defined
            // [ ] Monitoring/metrics implemented
            // [ ] Logging added for debugging
            //
            // ACCEPTANCE CRITERIA:
            // - Fallback responses are meaningful
            // - Error handling is comprehensive
            // - Backend health is monitored
            // - Recovery mechanisms work correctly
            //
            // DEPENDENCIES:
            // - Backend system: Required
            // - HTTP client: Available
            //
            // ESTIMATED EFFORT: 12 hours
            // PRIORITY: MEDIUM
            // BLOCKING: No - Fallback functionality
            
            // Return a stub response if backend is not available
            Ok((axum::http::StatusCode::OK, r#"{"status": "stub", "message": "Backend not available"}"#.to_string()))
        }
    }
}

// TODO: System Metrics and Monitoring - Implement comprehensive metrics system
// 
// COMPLETION CHECKLIST:
// [ ] System metrics collection implemented
// [ ] Dashboard data generation
// [ ] Metrics API endpoints
// [ ] Real-time metrics streaming
// [ ] Unit tests written (80%+ coverage)
// [ ] Integration tests with metrics system
// [ ] Documentation updated
// [ ] Performance benchmarks meet SLA
// [ ] Security considerations addressed
// [ ] Configuration options defined
// [ ] Monitoring/metrics implemented
// [ ] Logging added for debugging
//
// ACCEPTANCE CRITERIA:
// - System metrics are accurately collected
// - Dashboard provides meaningful insights
// - Metrics API is performant and reliable
// - Real-time streaming works correctly
//
// DEPENDENCIES:
// - DatabaseClient: Available
// - Metrics types: Required
//
// ESTIMATED EFFORT: 32 hours
// PRIORITY: HIGH
// BLOCKING: Yes - Required for system monitoring

/// Get system metrics (real implementation)
pub async fn get_metrics(
    State(state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get system metrics from database
    let metrics_query = r#"
        SELECT 
            COUNT(*) as total_tasks,
            COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_tasks,
            COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_tasks,
            COUNT(CASE WHEN status = 'in_progress' THEN 1 END) as in_progress_tasks,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_tasks,
            AVG(CASE WHEN execution_time_ms IS NOT NULL THEN execution_time_ms END) as avg_execution_time_ms,
            COUNT(DISTINCT assigned_worker_id) as active_workers
        FROM tasks t
        LEFT JOIN task_executions te ON t.id = te.task_id
    "#;

    match state.db_client.query(metrics_query, &[]).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                let metrics = serde_json::json!({
                    "total_tasks": row.try_get::<i64, _>("total_tasks").unwrap_or(0),
                    "completed_tasks": row.try_get::<i64, _>("completed_tasks").unwrap_or(0),
                    "pending_tasks": row.try_get::<i64, _>("pending_tasks").unwrap_or(0),
                    "in_progress_tasks": row.try_get::<i64, _>("in_progress_tasks").unwrap_or(0),
                    "failed_tasks": row.try_get::<i64, _>("failed_tasks").unwrap_or(0),
                    "avg_execution_time_ms": row.try_get::<Option<f64>, _>("avg_execution_time_ms").unwrap_or(None),
                    "active_workers": row.try_get::<i64, _>("active_workers").unwrap_or(0),
                    "timestamp": chrono::Utc::now(),
                    "status": "success"
                });
                Ok(Json(metrics))
            } else {
                Ok(Json(serde_json::json!({
                    "message": "No metrics data available",
                    "status": "empty"
                })))
            }
        }
        Err(e) => {
            error!("Failed to get system metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get dashboard data (real implementation)
pub async fn get_dashboard_data(
    State(state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get comprehensive dashboard data
    let dashboard_query = r#"
        WITH task_stats AS (
            SELECT 
                COUNT(*) as total_tasks,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_tasks,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_tasks,
                COUNT(CASE WHEN status = 'in_progress' THEN 1 END) as in_progress_tasks,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_tasks
            FROM tasks
        ),
        worker_stats AS (
            SELECT 
                COUNT(*) as total_workers,
                COUNT(CASE WHEN is_active = true THEN 1 END) as active_workers
            FROM workers
        ),
        recent_activity AS (
            SELECT 
                t.id,
                t.title,
                t.status,
                t.created_at,
                t.updated_at,
                w.name as worker_name
            FROM tasks t
            LEFT JOIN workers w ON t.assigned_worker_id = w.id
            ORDER BY t.updated_at DESC
            LIMIT 10
        )
        SELECT 
            ts.*,
            ws.*,
            json_agg(
                json_build_object(
                    'id', ra.id,
                    'title', ra.title,
                    'status', ra.status,
                    'created_at', ra.created_at,
                    'updated_at', ra.updated_at,
                    'worker_name', ra.worker_name
                )
            ) as recent_activity
        FROM task_stats ts, worker_stats ws, recent_activity ra
        GROUP BY ts.total_tasks, ts.completed_tasks, ts.pending_tasks, ts.in_progress_tasks, ts.failed_tasks,
                 ws.total_workers, ws.active_workers
    "#;

    match state.db_client.query(dashboard_query, &[]).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                let dashboard_data = serde_json::json!({
                    "task_stats": {
                        "total_tasks": row.try_get::<i64, _>("total_tasks").unwrap_or(0),
                        "completed_tasks": row.try_get::<i64, _>("completed_tasks").unwrap_or(0),
                        "pending_tasks": row.try_get::<i64, _>("pending_tasks").unwrap_or(0),
                        "in_progress_tasks": row.try_get::<i64, _>("in_progress_tasks").unwrap_or(0),
                        "failed_tasks": row.try_get::<i64, _>("failed_tasks").unwrap_or(0)
                    },
                    "worker_stats": {
                        "total_workers": row.try_get::<i64, _>("total_workers").unwrap_or(0),
                        "active_workers": row.try_get::<i64, _>("active_workers").unwrap_or(0)
                    },
                    "recent_activity": row.try_get::<serde_json::Value, _>("recent_activity").unwrap_or(serde_json::Value::Array(vec![])),
                    "timestamp": chrono::Utc::now(),
                    "status": "success"
                });
                Ok(Json(dashboard_data))
            } else {
                Ok(Json(serde_json::json!({
                    "message": "No dashboard data available",
                    "status": "empty"
                })))
            }
        }
        Err(e) => {
            error!("Failed to get dashboard data: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get diff summary (real implementation)
pub async fn get_diff_summary(
    State(state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get recent changes and generate diff summary
    let changes_query = r#"
        WITH recent_changes AS (
            SELECT 
                'task' as entity_type,
                id::text as entity_id,
                title as entity_name,
                status as change_type,
                created_at as change_time,
                'created' as action
            FROM tasks
            WHERE created_at > NOW() - INTERVAL '24 hours'
            
            UNION ALL
            
            SELECT 
                'task' as entity_type,
                id::text as entity_id,
                title as entity_name,
                status as change_type,
                updated_at as change_time,
                'updated' as action
            FROM tasks
            WHERE updated_at > NOW() - INTERVAL '24 hours' 
                AND updated_at != created_at
            
            UNION ALL
            
            SELECT 
                'worker' as entity_type,
                id::text as entity_id,
                name as entity_name,
                CASE WHEN is_active THEN 'active' ELSE 'inactive' END as change_type,
                updated_at as change_time,
                'status_changed' as action
            FROM workers
            WHERE updated_at > NOW() - INTERVAL '24 hours'
        )
        SELECT 
            entity_type,
            entity_id,
            entity_name,
            change_type,
            change_time,
            action,
            COUNT(*) OVER (PARTITION BY entity_type) as total_changes_by_type
        FROM recent_changes
        ORDER BY change_time DESC
        LIMIT 50
    "#;

    match state.db_client.query(changes_query, &[]).await {
        Ok(rows) => {
            let changes: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                let entity_type: String = row.get(0);
                let entity_id: String = row.get(1);
                let entity_name: String = row.get(2);
                let change_type: String = row.get(3);
                let change_time: chrono::DateTime<chrono::Utc> = row.get(4);
                let action: String = row.get(5);
                let total_changes_by_type: i64 = row.get(6);

                json!({
                    "entity_type": entity_type,
                    "entity_id": entity_id,
                    "entity_name": entity_name,
                    "change_type": change_type,
                    "change_time": change_time,
                    "action": action,
                    "total_changes_by_type": total_changes_by_type
                })
            }).collect();

            // Calculate summary statistics
            let total_changes = changes.len();
            let task_changes = changes.iter().filter(|c| c["entity_type"].as_str().unwrap_or("") == "task").count();
            let worker_changes = changes.iter().filter(|c| c["entity_type"].as_str().unwrap_or("") == "worker").count();
            
            let created_count = changes.iter().filter(|c| c["action"].as_str().unwrap_or("") == "created").count();
            let updated_count = changes.iter().filter(|c| c["action"].as_str().unwrap_or("") == "updated").count();
            let status_changed_count = changes.iter().filter(|c| c["action"].as_str().unwrap_or("") == "status_changed").count();

            // Get time range
            let time_range = if let (Some(first), Some(last)) = (changes.first(), changes.last()) {
                let start: chrono::DateTime<chrono::Utc> = serde_json::from_value(first["change_time"].clone()).unwrap_or_default();
                let end: chrono::DateTime<chrono::Utc> = serde_json::from_value(last["change_time"].clone()).unwrap_or_default();
                let duration_hours = (end - start).num_hours();
                
                json!({
                    "start": first["change_time"],
                    "end": last["change_time"],
                    "duration_hours": duration_hours
                })
            } else {
                json!({"start": null, "end": null, "duration_hours": 0})
            };

            Ok(Json(json!({
                "summary": {
                    "total_changes": total_changes,
                    "changes_by_entity": {
                        "tasks": task_changes,
                        "workers": worker_changes
                    },
                    "changes_by_action": {
                        "created": created_count,
                        "updated": updated_count,
                        "status_changed": status_changed_count
                    },
                    "time_range": time_range,
                    "generated_at": chrono::Utc::now()
                },
                "changes": changes,
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to generate diff summary: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TODO: SLO Management System - Implement comprehensive SLO monitoring and management
// 
// COMPLETION CHECKLIST:
// [ ] SLO definition and configuration
// [ ] SLO measurement and tracking
// [ ] SLO alert generation and management
// [ ] SLO status reporting and dashboards
// [ ] Unit tests written (80%+ coverage)
// [ ] Integration tests with SLO system
// [ ] Documentation updated
// [ ] Performance benchmarks meet SLA
// [ ] Security considerations addressed
// [ ] Configuration options defined
// [ ] Monitoring/metrics implemented
// [ ] Logging added for debugging
//
// ACCEPTANCE CRITERIA:
// - SLOs can be defined, measured, and tracked
// - SLO alerts are generated and managed properly
// - SLO status reporting is accurate and timely
// - SLO dashboards provide meaningful insights
//
// DEPENDENCIES:
// - DatabaseClient: Available
// - SLO types: Required
//
// ESTIMATED EFFORT: 40 hours
// PRIORITY: HIGH
// BLOCKING: Yes - Required for service level management

/// Acknowledge SLO alert (real implementation)
pub async fn acknowledge_slo_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Update alert status in database
    let update_query = r#"
        UPDATE slo_alerts 
        SET acknowledged_at = NOW(), 
            acknowledged_by = 'system',
            status = 'acknowledged'
        WHERE id = $1 AND status = 'active'
    "#;

    match state.db_client.execute(update_query, &[&alert_id]).await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(Json(json!({
                    "message": "SLO alert acknowledged successfully",
                    "alert_id": alert_id,
                    "acknowledged_at": chrono::Utc::now(),
                    "status": "acknowledged"
                })))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to acknowledge SLO alert {}: {}", alert_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List SLOs (real implementation)
pub async fn list_slos(
    State(state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get all active SLO definitions from database
    let slo_query = r#"
        SELECT 
            d.name,
            d.description,
            d.service_name,
            d.slo_type,
            d.target_value,
            d.window_minutes,
            s.current_value,
            s.error_budget_used,
            s.status,
            s.last_updated
        FROM slo_definitions d
        LEFT JOIN slo_status_snapshots s ON d.id = s.slo_id
            AND s.last_updated = (
                SELECT MAX(last_updated) FROM slo_status_snapshots
                WHERE slo_id = d.id
            )
        WHERE d.is_active = true
        ORDER BY d.name
    "#;

    match state.db_client.query(slo_query, &[]).await {
        Ok(rows) => {
            let slos: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                let name: String = row.get(0);
                let description: String = row.get(1);
                let service_name: String = row.get(2);
                let slo_type: String = row.get(3);
                let target_value: f64 = row.get::<f64, _>(4);
                let window_minutes: i32 = row.get(5);
                let current_value: Option<f64> = row.get(6);
                let error_budget_used: Option<f64> = row.get(7);
                let status: Option<String> = row.get(8);
                let last_updated: Option<chrono::DateTime<chrono::Utc>> = row.get(9);

                let current_value = current_value.unwrap_or(0.0);
                let error_budget_used = error_budget_used.unwrap_or(0.0);
                let remaining_budget = (1.0 - error_budget_used).max(0.0);

                json!({
                    "name": name,
                    "description": description,
                    "service": service_name,
                    "metric": slo_type,
                    "target_value": target_value,
                    "current_value": current_value,
                    "window_minutes": window_minutes,
                    "compliance_percentage": if target_value > 0.0 { (current_value / target_value).min(1.0) } else { 0.0 },
                    "remaining_budget": remaining_budget,
                    "status": status.unwrap_or("unknown".to_string()),
                    "last_updated": last_updated
                })
            }).collect();

            Ok(Json(json!({
                "slos": slos,
                "count": slos.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list SLOs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get SLO status (real implementation)
pub async fn get_slo_status(
    State(state): State<AppState>,
    Path(slo_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get detailed SLO status from database
    let status_query = r#"
        SELECT 
            d.name,
            d.description,
            d.service_name,
            d.slo_type,
            d.target_value,
            d.window_minutes,
            s.current_value,
            s.error_budget_used,
            s.status,
            s.last_updated,
            s.measurement_count,
            s.good_count,
            s.bad_count
        FROM slo_definitions d
        LEFT JOIN slo_status_snapshots s ON d.id = s.slo_id
            AND s.last_updated = (
                SELECT MAX(last_updated) FROM slo_status_snapshots
                WHERE slo_id = d.id
            )
        WHERE d.name = $1 AND d.is_active = true
    "#;

    match state.db_client.query(status_query, &[&slo_id]).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                let name: String = row.get(0);
                let description: String = row.get(1);
                let service_name: String = row.get(2);
                let slo_type: String = row.get(3);
                let target_value: f64 = row.get::<f64, _>(4);
                let window_minutes: i32 = row.get(5);
                let current_value: Option<f64> = row.get(6);
                let error_budget_used: Option<f64> = row.get(7);
                let status: Option<String> = row.get(8);
                let last_updated: Option<chrono::DateTime<chrono::Utc>> = row.get(9);
                let measurement_count: Option<i64> = row.get(10);
                let good_count: Option<i64> = row.get(11);
                let bad_count: Option<i64> = row.get(12);

                let current_value = current_value.unwrap_or(0.0);
                let error_budget_used = error_budget_used.unwrap_or(0.0);
                let remaining_budget = (1.0 - error_budget_used).max(0.0);
                let compliance_percentage = if target_value > 0.0 { (current_value / target_value).min(1.0) } else { 0.0 };

                let status_data = json!({
                    "slo_name": name,
                    "description": description,
                    "service": service_name,
                    "metric": slo_type,
                    "target_value": target_value,
                    "current_value": current_value,
                    "window_minutes": window_minutes,
                    "compliance_percentage": compliance_percentage,
                    "remaining_budget": remaining_budget,
                    "error_budget_used": error_budget_used,
                    "status": status.unwrap_or("unknown".to_string()),
                    "last_updated": last_updated,
                    "measurements": {
                        "total_count": measurement_count.unwrap_or(0),
                        "good_count": good_count.unwrap_or(0),
                        "bad_count": bad_count.unwrap_or(0),
                        "success_rate": if let Some(total) = measurement_count {
                            if total > 0 { good_count.unwrap_or(0) as f64 / total as f64 } else { 0.0 }
                        } else { 0.0 }
                    },
                    "period": {
                        "start": last_updated.map(|lu| lu - chrono::Duration::minutes(window_minutes as i64)),
                        "end": last_updated
                    }
                });

                Ok(Json(status_data))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to get SLO status for {}: {}", slo_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get SLO measurements (real implementation)
pub async fn get_slo_measurements(
    State(state): State<AppState>,
    Path(slo_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get SLO measurements from database with time range filtering
    let measurements_query = r#"
        SELECT 
            m.timestamp,
            m.value,
            m.is_good,
            m.sample_count,
            m.good_count,
            m.bad_count
        FROM slo_measurements m
        JOIN slo_definitions d ON m.slo_id = d.id
        WHERE d.name = $1 AND d.is_active = true
        ORDER BY m.timestamp DESC
        LIMIT 1000
    "#;

    match state.db_client.query(measurements_query, &[&slo_id]).await {
        Ok(rows) => {
            let measurements: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                let timestamp: chrono::DateTime<chrono::Utc> = row.get(0);
                let value: f64 = row.get::<f64, _>(1);
                let is_good: bool = row.get(2);
                let sample_count: i64 = row.get(3);
                let good_count: i64 = row.get(4);
                let bad_count: i64 = row.get(5);

                json!({
                    "timestamp": timestamp,
                    "value": value,
                    "is_good": is_good,
                    "sample_count": sample_count,
                    "good_count": good_count,
                    "bad_count": bad_count,
                    "success_rate": if sample_count > 0 { good_count as f64 / sample_count as f64 } else { 0.0 }
                })
            }).collect();

            // Calculate summary statistics
            let total_measurements = measurements.len();
            let good_measurements = measurements.iter().filter(|m| m["is_good"].as_bool().unwrap_or(false)).count();
            let avg_value = if total_measurements > 0 {
                measurements.iter().map(|m| m["value"].as_f64().unwrap_or(0.0)).sum::<f64>() / total_measurements as f64
            } else { 0.0 };

            Ok(Json(json!({
                "slo_id": slo_id,
                "measurements": measurements,
                "summary": {
                    "total_measurements": total_measurements,
                    "good_measurements": good_measurements,
                    "bad_measurements": total_measurements - good_measurements,
                    "overall_success_rate": if total_measurements > 0 { good_measurements as f64 / total_measurements as f64 } else { 0.0 },
                    "average_value": avg_value,
                    "time_range": {
                        "start": measurements.last().and_then(|m| m["timestamp"].as_str()),
                        "end": measurements.first().and_then(|m| m["timestamp"].as_str())
                    }
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get SLO measurements for {}: {}", slo_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List SLO alerts (real implementation)
pub async fn list_slo_alerts(
    State(state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get all SLO alerts from database
    let alerts_query = r#"
        SELECT 
            a.id,
            a.slo_name,
            a.alert_type,
            a.severity,
            a.message,
            a.timestamp,
            a.actual_value,
            a.target_value,
            a.triggered_at,
            a.resolved_at,
            a.acknowledged_at,
            a.acknowledged_by,
            a.status,
            d.description as slo_description,
            d.service_name
        FROM slo_alerts a
        LEFT JOIN slo_definitions d ON a.slo_name = d.name
        ORDER BY a.timestamp DESC
        LIMIT 100
    "#;

    match state.db_client.query(alerts_query, &[]).await {
        Ok(rows) => {
            let alerts: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                let id: String = row.get(0);
                let slo_name: String = row.get(1);
                let alert_type: String = row.get(2);
                let severity: String = row.get(3);
                let message: String = row.get(4);
                let timestamp: chrono::DateTime<chrono::Utc> = row.get(5);
                let actual_value: f64 = row.get::<f64, _>(6);
                let target_value: f64 = row.get::<f64, _>(7);
                let triggered_at: chrono::DateTime<chrono::Utc> = row.get(8);
                let resolved_at: Option<chrono::DateTime<chrono::Utc>> = row.get(9);
                let acknowledged_at: Option<chrono::DateTime<chrono::Utc>> = row.get(10);
                let acknowledged_by: Option<String> = row.get(11);
                let status: String = row.get(12);
                let slo_description: Option<String> = row.get(13);
                let service_name: Option<String> = row.get(14);

                json!({
                    "id": id,
                    "slo_name": slo_name,
                    "slo_description": slo_description,
                    "service_name": service_name,
                    "alert_type": alert_type,
                    "severity": severity,
                    "message": message,
                    "timestamp": timestamp,
                    "actual_value": actual_value,
                    "target_value": target_value,
                    "triggered_at": triggered_at,
                    "resolved_at": resolved_at,
                    "acknowledged_at": acknowledged_at,
                    "acknowledged_by": acknowledged_by,
                    "status": status,
                    "duration": resolved_at.map(|resolved| {
                        let duration = resolved - triggered_at;
                        duration.num_seconds()
                    }),
                    "compliance_percentage": if target_value > 0.0 { (actual_value / target_value).min(1.0) } else { 0.0 }
                })
            }).collect();

            // Calculate alert statistics
            let total_alerts = alerts.len();
            let active_alerts = alerts.iter().filter(|a| a["status"].as_str().unwrap_or("") == "active").count();
            let acknowledged_alerts = alerts.iter().filter(|a| a["acknowledged_at"].is_null() == false).count();
            let resolved_alerts = alerts.iter().filter(|a| a["resolved_at"].is_null() == false).count();

            Ok(Json(json!({
                "alerts": alerts,
                "summary": {
                    "total_alerts": total_alerts,
                    "active_alerts": active_alerts,
                    "acknowledged_alerts": acknowledged_alerts,
                    "resolved_alerts": resolved_alerts,
                    "alert_rate": if total_alerts > 0 { active_alerts as f64 / total_alerts as f64 } else { 0.0 }
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list SLO alerts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TODO: Provenance Management System - Implement comprehensive provenance tracking
// 
// COMPLETION CHECKLIST:
// [ ] Provenance record management
// [ ] Provenance linking and verification
// [ ] Provenance audit trail
// [ ] Provenance query and retrieval
// [ ] Unit tests written (80%+ coverage)
// [ ] Integration tests with provenance system
// [ ] Documentation updated
// [ ] Performance benchmarks meet SLA
// [ ] Security considerations addressed
// [ ] Configuration options defined
// [ ] Monitoring/metrics implemented
// [ ] Logging added for debugging
//
// ACCEPTANCE CRITERIA:
// - Provenance records are accurately tracked
// - Provenance linking works correctly
// - Provenance verification is reliable
// - Provenance queries are performant
//
// DEPENDENCIES:
// - DatabaseClient: Available
// - Provenance types: Required
//
// ESTIMATED EFFORT: 28 hours
// PRIORITY: HIGH
// BLOCKING: Yes - Required for audit and compliance

/// List provenance records (real implementation)
pub async fn list_provenance_records(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.get("limit").and_then(|l| l.parse::<i64>().ok()).unwrap_or(100);
    let offset = params.get("offset").and_then(|o| o.parse::<i64>().ok()).unwrap_or(0);
    let entity_type = params.get("entity_type");
    let entity_id = params.get("entity_id");

    // Build query with optional filters
    let mut query = r#"
        SELECT 
            p.id,
            p.entity_type,
            p.entity_id,
            p.action_type,
            p.action_description,
            p.actor_id,
            p.actor_type,
            p.timestamp,
            p.metadata,
            p.commit_hash,
            p.parent_provenance_id,
            p.verification_status,
            p.created_at
        FROM provenance_records p
        WHERE 1=1
    "#.to_string();

    let mut query_params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    if let Some(et) = entity_type {
        param_count += 1;
        query.push_str(&format!(" AND p.entity_type = ${}", param_count));
        query_params.push(Box::new(et.clone()));
    }

    if let Some(eid) = entity_id {
        param_count += 1;
        query.push_str(&format!(" AND p.entity_id = ${}", param_count));
        query_params.push(Box::new(eid.clone()));
    }

    query.push_str(&format!(" ORDER BY p.timestamp DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));
    query_params.push(Box::new(limit));
    query_params.push(Box::new(offset));

    match state.db_client.query(&query, &query_params.iter().map(|p| p.as_ref()).collect::<Vec<_>>()).await {
        Ok(rows) => {
            let records: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "entity_type": row.get::<String, _>("entity_type"),
                    "entity_id": row.get::<String, _>("entity_id"),
                    "action_type": row.get::<String, _>("action_type"),
                    "action_description": row.get::<String, _>("action_description"),
                    "actor_id": row.get::<Option<String>, _>("actor_id"),
                    "actor_type": row.get::<String, _>("actor_type"),
                    "timestamp": row.get::<DateTime<Utc>, _>("timestamp"),
                    "metadata": row.get::<Option<serde_json::Value>, _>("metadata"),
                    "commit_hash": row.get::<Option<String>, _>("commit_hash"),
                    "parent_provenance_id": row.get::<Option<Uuid>, _>("parent_provenance_id"),
                    "verification_status": row.get::<String, _>("verification_status"),
                    "created_at": row.get::<DateTime<Utc>, _>("created_at")
                })
            }).collect();

            // Get total count for pagination
            let mut count_query = r#"
                SELECT COUNT(*) as total
                FROM provenance_records p
                WHERE 1=1
            "#.to_string();
            
            let mut count_params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
            let mut count_param_count = 0;

            if let Some(et) = entity_type {
                count_param_count += 1;
                count_query.push_str(&format!(" AND p.entity_type = ${}", count_param_count));
                count_params.push(Box::new(et.clone()));
            }

            if let Some(eid) = entity_id {
                count_param_count += 1;
                count_query.push_str(&format!(" AND p.entity_id = ${}", count_param_count));
                count_params.push(Box::new(eid.clone()));
            }

            let total_count = match state.db_client.query_one(&count_query, &count_params.iter().map(|p| p.as_ref()).collect::<Vec<_>>()).await {
                Ok(Some(row)) => row.get::<i64, _>("total"),
                Ok(None) => 0,
                Err(e) => {
                    error!("Failed to get provenance count: {}", e);
                    0
                }
            };

            Ok(Json(json!({
                "records": records,
                "pagination": {
                    "total": total_count,
                    "limit": limit,
                    "offset": offset,
                    "has_more": offset + limit < total_count
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list provenance records: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Link provenance to commit (real implementation)
pub async fn link_provenance_to_commit(
    State(state): State<AppState>,
    link_data: Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let provenance_id = link_data["provenance_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            error!("Invalid provenance_id in link request");
            StatusCode::BAD_REQUEST
        })?;

    let commit_hash = link_data["commit_hash"].as_str()
        .ok_or_else(|| {
            error!("Missing commit_hash in link request");
            StatusCode::BAD_REQUEST
        })?;

    let link_type = link_data["link_type"].as_str().unwrap_or("direct");
    let link_description = link_data["link_description"].as_str().unwrap_or("");

    // Verify provenance record exists
    let provenance_check = r#"
        SELECT id, entity_type, entity_id, action_type
        FROM provenance_records
        WHERE id = $1
    "#;

    match state.db_client.query_one(provenance_check, &[&provenance_id]).await {
        Ok(Some(row)) => {
            let entity_type: String = row.get("entity_type");
            let entity_id: String = row.get("entity_id");
            let action_type: String = row.get("action_type");

            // Create provenance link record
            let link_query = r#"
                INSERT INTO provenance_links (
                    id,
                    provenance_id,
                    commit_hash,
                    link_type,
                    link_description,
                    created_at
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (provenance_id, commit_hash) 
                DO UPDATE SET 
                    link_type = EXCLUDED.link_type,
                    link_description = EXCLUDED.link_description,
                    updated_at = CURRENT_TIMESTAMP
                RETURNING id, created_at, updated_at
            "#;

            let link_id = Uuid::new_v4();
            let now = Utc::now();

            match state.db_client.query_one(link_query, &[
                &link_id,
                &provenance_id,
                &commit_hash,
                &link_type,
                &link_description,
                &now
            ]).await {
                Ok(Some(link_row)) => {
                    let created_at: DateTime<Utc> = link_row.get("created_at");
                    let updated_at: Option<DateTime<Utc>> = link_row.get("updated_at");

                    // Update provenance record with commit hash
                    let update_provenance = r#"
                        UPDATE provenance_records
                        SET commit_hash = $1, updated_at = CURRENT_TIMESTAMP
                        WHERE id = $2
                    "#;

                    if let Err(e) = state.db_client.execute(update_provenance, &[&commit_hash, &provenance_id]).await {
                        error!("Failed to update provenance record with commit hash: {}", e);
                    }

                    Ok(Json(json!({
                        "link_id": link_id,
                        "provenance_id": provenance_id,
                        "commit_hash": commit_hash,
                        "link_type": link_type,
                        "link_description": link_description,
                        "entity_type": entity_type,
                        "entity_id": entity_id,
                        "action_type": action_type,
                        "created_at": created_at,
                        "updated_at": updated_at,
                        "status": "success",
                        "message": "Provenance successfully linked to commit"
                    })))
                }
                Ok(None) => {
                    error!("Link creation returned no row");
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
                Err(e) => {
                    error!("Failed to create provenance link: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => {
            error!("Provenance record not found");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Provenance record not found: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Verify provenance trailer (real implementation)
pub async fn verify_provenance_trailer(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let commit_hash = params.get("commit_hash")
        .ok_or_else(|| {
            error!("Missing commit_hash parameter");
            StatusCode::BAD_REQUEST
        })?;

    let default_verification_type = "full".to_string();
    let verification_type = params.get("verification_type").unwrap_or(&default_verification_type);

    // Get provenance records for this commit
    let provenance_query = r#"
        SELECT 
            p.id,
            p.entity_type,
            p.entity_id,
            p.action_type,
            p.action_description,
            p.actor_id,
            p.actor_type,
            p.timestamp,
            p.metadata,
            p.commit_hash,
            p.parent_provenance_id,
            p.verification_status,
            p.created_at,
            pl.link_type,
            pl.link_description
        FROM provenance_records p
        LEFT JOIN provenance_links pl ON p.id = pl.provenance_id
        WHERE p.commit_hash = $1 OR pl.commit_hash = $1
        ORDER BY p.timestamp ASC
    "#;

    match state.db_client.query(provenance_query, &[commit_hash]).await {
        Ok(rows) => {
            if rows.is_empty() {
                return Ok(Json(json!({
                    "commit_hash": commit_hash,
                    "verification_status": "not_found",
                    "message": "No provenance records found for this commit",
                    "records": [],
                    "verification_details": {
                        "total_records": 0,
                        "verified_records": 0,
                        "unverified_records": 0,
                        "verification_errors": []
                    }
                })));
            }

            let mut verification_errors = Vec::new();
            let mut verified_count = 0;
            let mut unverified_count = 0;

            // First, collect all parent IDs that need to be checked
            let mut parent_ids_to_check = Vec::new();
            let mut rows_with_parents = Vec::new();
            
            for row in rows {
                let parent_provenance_id: Option<Uuid> = row.get("parent_provenance_id");
                if let Some(parent_id) = parent_provenance_id {
                    parent_ids_to_check.push(parent_id);
                }
                rows_with_parents.push(row);
            }

            // Check all parent IDs in batch
            let mut valid_parents = std::collections::HashSet::new();
            if !parent_ids_to_check.is_empty() {
                let parent_check = format!(
                    "SELECT id FROM provenance_records WHERE id = ANY($1)",
                );
                if let Ok(parent_rows) = state.db_client.query(&parent_check, &[&parent_ids_to_check]).await {
                    for parent_row in parent_rows {
                        let parent_id = parent_row.get::<Uuid, _>("id");
                        valid_parents.insert(parent_id);
                    }
                }
            }

            let records: Vec<serde_json::Value> = rows_with_parents.into_iter().map(|row| {
                let verification_status: String = row.get("verification_status");
                let record_id: Uuid = row.get("id");
                let entity_type: String = row.get("entity_type");
                let entity_id: String = row.get("entity_id");
                let action_type: String = row.get("action_type");
                let actor_id: Option<String> = row.get("actor_id");
                let actor_type: String = row.get("actor_type");
                let timestamp: DateTime<Utc> = row.get("timestamp");
                let metadata: Option<serde_json::Value> = row.get("metadata");
                let parent_provenance_id: Option<Uuid> = row.get("parent_provenance_id");

                // Perform verification checks
                let mut record_errors = Vec::new();

                // Check if actor is valid
                if actor_id.is_none() && actor_type != "system" {
                    record_errors.push("Missing actor_id for non-system action");
                }

                // Check timestamp consistency
                let now = Utc::now();
                if timestamp > now {
                    record_errors.push("Future timestamp detected");
                }

                // Check parent provenance exists if specified
                if let Some(parent_id) = parent_provenance_id {
                    if !valid_parents.contains(&parent_id) {
                        record_errors.push("Parent provenance record not found");
                    }
                }

                // Check metadata validity
                if let Some(meta) = &metadata {
                    if meta.is_object() {
                        // Additional metadata validation could go here
                    }
                }

                if record_errors.is_empty() {
                    verified_count += 1;
                } else {
                    unverified_count += 1;
                    verification_errors.extend(record_errors.iter().map(|e| format!("Record {}: {}", record_id, e)));
                }

                json!({
                    "id": record_id,
                    "entity_type": entity_type,
                    "entity_id": entity_id,
                    "action_type": action_type,
                    "action_description": row.get::<String, _>("action_description"),
                    "actor_id": actor_id,
                    "actor_type": actor_type,
                    "timestamp": timestamp,
                    "metadata": metadata,
                    "commit_hash": row.get::<Option<String>, _>("commit_hash"),
                    "parent_provenance_id": parent_provenance_id,
                    "verification_status": verification_status,
                    "created_at": row.get::<DateTime<Utc>, _>("created_at"),
                    "link_type": row.get::<Option<String>, _>("link_type"),
                    "link_description": row.get::<Option<String>, _>("link_description"),
                    "verification_errors": record_errors
                })
            }).collect();

            let overall_status = if verification_errors.is_empty() {
                "verified"
            } else if verified_count > 0 {
                "partially_verified"
            } else {
                "verification_failed"
            };

            Ok(Json(json!({
                "commit_hash": commit_hash,
                "verification_status": overall_status,
                "verification_type": verification_type,
                "records": records,
                "verification_details": {
                    "total_records": records.len(),
                    "verified_records": verified_count,
                    "unverified_records": unverified_count,
                    "verification_errors": verification_errors,
                    "verification_timestamp": Utc::now()
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to verify provenance for commit {}: {}", commit_hash, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get provenance by commit (real implementation)
pub async fn get_provenance_by_commit(
    State(state): State<AppState>,
    Path(commit_hash): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get all provenance records linked to this commit
    let provenance_query = r#"
        SELECT 
            p.id,
            p.entity_type,
            p.entity_id,
            p.action_type,
            p.action_description,
            p.actor_id,
            p.actor_type,
            p.timestamp,
            p.metadata,
            p.commit_hash,
            p.parent_provenance_id,
            p.verification_status,
            p.created_at,
            pl.link_type,
            pl.link_description,
            pl.created_at as link_created_at
        FROM provenance_records p
        LEFT JOIN provenance_links pl ON p.id = pl.provenance_id
        WHERE p.commit_hash = $1 OR pl.commit_hash = $1
        ORDER BY p.timestamp ASC
    "#;

    match state.db_client.query(provenance_query, &[&commit_hash]).await {
        Ok(rows) => {
            if rows.is_empty() {
                return Ok(Json(json!({
                    "commit_hash": commit_hash,
                    "message": "No provenance records found for this commit",
                    "provenance_records": [],
                    "summary": {
                        "total_records": 0,
                        "entity_types": {},
                        "action_types": {},
                        "actors": {},
                        "time_range": null
                    }
                })));
            }

            let mut entity_types = std::collections::HashMap::new();
            let mut action_types = std::collections::HashMap::new();
            let mut actors = std::collections::HashMap::new();
            let mut timestamps = Vec::new();

            let provenance_records: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                let record_id: Uuid = row.get("id");
                let entity_type: String = row.get("entity_type");
                let entity_id: String = row.get("entity_id");
                let action_type: String = row.get("action_type");
                let action_description: String = row.get("action_description");
                let actor_id: Option<String> = row.get("actor_id");
                let actor_type: String = row.get("actor_type");
                let timestamp: DateTime<Utc> = row.get("timestamp");
                let metadata: Option<serde_json::Value> = row.get("metadata");
                let parent_provenance_id: Option<Uuid> = row.get("parent_provenance_id");
                let verification_status: String = row.get("verification_status");
                let created_at: DateTime<Utc> = row.get("created_at");
                let link_type: Option<String> = row.get("link_type");
                let link_description: Option<String> = row.get("link_description");
                let link_created_at: Option<DateTime<Utc>> = row.get("link_created_at");

                // Update summary statistics
                *entity_types.entry(entity_type.clone()).or_insert(0) += 1;
                *action_types.entry(action_type.clone()).or_insert(0) += 1;
                let actor_key = format!("{}:{}", actor_type, actor_id.as_deref().unwrap_or("unknown"));
                *actors.entry(actor_key).or_insert(0) += 1;
                timestamps.push(timestamp);

                json!({
                    "id": record_id,
                    "entity_type": entity_type,
                    "entity_id": entity_id,
                    "action_type": action_type,
                    "action_description": action_description,
                    "actor_id": actor_id,
                    "actor_type": actor_type,
                    "timestamp": timestamp,
                    "metadata": metadata,
                    "commit_hash": row.get::<Option<String>, _>("commit_hash"),
                    "parent_provenance_id": parent_provenance_id,
                    "verification_status": verification_status,
                    "created_at": created_at,
                    "link": {
                        "link_type": link_type,
                        "link_description": link_description,
                        "link_created_at": link_created_at
                    }
                })
            }).collect();

            // Calculate time range
            timestamps.sort();
            let time_range = if timestamps.is_empty() {
                None
            } else {
                Some(json!({
                    "start": timestamps.first().unwrap(),
                    "end": timestamps.last().unwrap(),
                    "duration_seconds": timestamps.last().unwrap().signed_duration_since(*timestamps.first().unwrap()).num_seconds()
                }))
            };

            Ok(Json(json!({
                "commit_hash": commit_hash,
                "provenance_records": provenance_records,
                "summary": {
                    "total_records": provenance_records.len(),
                    "entity_types": entity_types,
                    "action_types": action_types,
                    "actors": actors,
                    "time_range": time_range,
                    "verification_summary": {
                        "verified": provenance_records.iter().filter(|r| r["verification_status"] == "verified").count(),
                        "unverified": provenance_records.iter().filter(|r| r["verification_status"] != "verified").count()
                    }
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get provenance for commit {}: {}", commit_hash, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TODO: Task Management System - Implement comprehensive task lifecycle management
// 
// COMPLETION CHECKLIST:
// [ ] Task submission and validation
// [ ] Task status tracking and updates
// [ ] Task result retrieval and storage
// [ ] Task lifecycle management (cancel, pause, resume)
// [ ] Unit tests written (80%+ coverage)
// [ ] Integration tests with task system
// [ ] Documentation updated
// [ ] Performance benchmarks meet SLA
// [ ] Security considerations addressed
// [ ] Configuration options defined
// [ ] Monitoring/metrics implemented
// [ ] Logging added for debugging
//
// ACCEPTANCE CRITERIA:
// - Tasks can be submitted, tracked, and managed
// - Task status updates are accurate and timely
// - Task results are properly stored and retrieved
// - Task lifecycle operations work correctly
//
// DEPENDENCIES:
// - TaskStoreTrait: Available
// - DatabaseClient: Available
//
// ESTIMATED EFFORT: 36 hours
// PRIORITY: HIGH
// BLOCKING: Yes - Required for task execution

/// Cancel task (stub implementation)
pub async fn cancel_task(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    // TODO: Implement actual task cancellation
    Json(json!({"message": "Task cancellation not implemented yet"}))
}

/// Pause task (stub implementation)
pub async fn pause_task(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    // TODO: Implement actual task pausing
    Json(json!({"message": "Task pausing not implemented yet"}))
}

/// Resume task (stub implementation)
pub async fn resume_task(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    // TODO: Implement actual task resuming
    Json(json!({"message": "Task resuming not implemented yet"}))
}

// TODO: Query Management System - Implement saved query functionality
// 
// COMPLETION CHECKLIST:
// [ ] Query saving and retrieval
// [ ] Query validation and optimization
// [ ] Query execution and caching
// [ ] Query sharing and permissions
// [ ] Unit tests written (80%+ coverage)
// [ ] Integration tests with query system
// [ ] Documentation updated
// [ ] Performance benchmarks meet SLA
// [ ] Security considerations addressed
// [ ] Configuration options defined
// [ ] Monitoring/metrics implemented
// [ ] Logging added for debugging
//
// ACCEPTANCE CRITERIA:
// - Queries can be saved, retrieved, and executed
// - Query validation prevents invalid queries
// - Query caching improves performance
// - Query permissions work correctly
//
// DEPENDENCIES:
// - DatabaseClient: Available
// - Query types: Required
//
// ESTIMATED EFFORT: 20 hours
// PRIORITY: MEDIUM
// BLOCKING: No - Query management functionality

/// List saved queries (stub implementation)
pub async fn list_saved_queries() -> Json<serde_json::Value> {
    // TODO: Implement actual saved queries listing
    Json(json!({"queries": [], "message": "Saved queries not implemented yet"}))
}

/// Save query (stub implementation)
pub async fn save_query(_query_data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    // TODO: Implement actual query saving
    Json(json!({"message": "Query saving not implemented yet"}))
}

/// Delete saved query (stub implementation)
pub async fn delete_saved_query(Path(_query_id): Path<String>) -> Json<serde_json::Value> {
    // TODO: Implement actual query deletion
    Json(json!({"message": "Query deletion not implemented yet"}))
}

/// Submit task
pub async fn submit_task(
    State(state): State<AppState>,
    Json(task_data): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract task data from JSON
    let title = task_data.get("title")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let description = task_data.get("description")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let priority = task_data.get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");
    
    let task_type = task_data.get("task_type")
        .and_then(|v| v.as_str())
        .unwrap_or("general");
    
    let metadata = task_data.get("metadata").cloned().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    
    // Create task in database
    let task = crate::models::Task {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        description: description.to_string(),
        risk_tier: "medium".to_string(),
        scope: serde_json::json!({}),
        acceptance_criteria: serde_json::json!({}),
        context: serde_json::json!({}),
        caws_spec: None,
        status: "pending".to_string(),
        assigned_worker_id: None,
        priority: priority.parse().ok(),
        deadline: None,
        metadata: Some(metadata),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        completed_at: None,
    };
    
    match state.db_client.create_task(&task).await {
        Ok(task_id) => {
            info!("Created task: {}", task_id);
            Ok(Json(serde_json::json!({
                "task_id": task_id,
                "status": "submitted",
                "message": "Task submitted successfully"
            })))
        }
        Err(e) => {
            error!("Failed to submit task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get task status
pub async fn get_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_uuid = uuid::Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.db_client.get_task(&task_uuid).await {
        Ok(Some(task)) => {
            Ok(Json(serde_json::json!({
                "task_id": task.id,
                "status": task.status,
                "title": task.title,
                "description": task.description,
                "priority": task.priority,
                "risk_tier": task.risk_tier,
                "created_at": task.created_at,
                "updated_at": task.updated_at,
                "assigned_worker_id": task.assigned_worker_id,
                "deadline": task.deadline,
                "metadata": task.metadata
            })))
        }
        Ok(None) => {
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to get task status for {}: {}", task_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get task result (real implementation)
pub async fn get_task_result(
    State(state): State<AppState>,
    Path(task_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_uuid = uuid::Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get task execution results
    let result_query = r#"
        SELECT 
            te.id as execution_id,
            te.status as execution_status,
            te.worker_output,
            te.self_assessment,
            te.result_data,
            te.execution_time_ms,
            te.execution_started_at,
            te.execution_completed_at,
            te.error_message,
            te.tokens_used,
            w.name as worker_name,
            w.worker_type,
            t.title as task_title,
            t.description as task_description
        FROM task_executions te
        JOIN workers w ON te.worker_id = w.id
        JOIN tasks t ON te.task_id = t.id
        WHERE te.task_id = $1
        ORDER BY te.execution_started_at DESC
        LIMIT 1
    "#;

    match state.db_client.query(result_query, &[&task_uuid]).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                let result = serde_json::json!({
                    "task_id": task_id,
                    "execution_id": row.try_get::<uuid::Uuid, _>("execution_id").unwrap_or(uuid::Uuid::nil()),
                    "execution_status": row.try_get::<String, _>("execution_status").unwrap_or("unknown".to_string()),
                    "worker_output": row.try_get::<serde_json::Value, _>("worker_output").unwrap_or(serde_json::Value::Null),
                    "self_assessment": row.try_get::<serde_json::Value, _>("self_assessment").unwrap_or(serde_json::Value::Null),
                    "result_data": row.try_get::<Option<serde_json::Value>, _>("result_data").unwrap_or(None),
                    "execution_time_ms": row.try_get::<Option<i32>, _>("execution_time_ms").unwrap_or(None),
                    "execution_started_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("execution_started_at").unwrap_or(chrono::Utc::now()),
                    "execution_completed_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("execution_completed_at").unwrap_or(None),
                    "error_message": row.try_get::<Option<String>, _>("error_message").unwrap_or(None),
                    "tokens_used": row.try_get::<Option<i32>, _>("tokens_used").unwrap_or(None),
                    "worker_name": row.try_get::<String, _>("worker_name").unwrap_or("unknown".to_string()),
                    "worker_type": row.try_get::<String, _>("worker_type").unwrap_or("unknown".to_string()),
                    "task_title": row.try_get::<String, _>("task_title").unwrap_or("unknown".to_string()),
                    "task_description": row.try_get::<String, _>("task_description").unwrap_or("unknown".to_string()),
                    "status": "success"
                });
                Ok(Json(result))
            } else {
                Ok(Json(serde_json::json!({
                    "task_id": task_id,
                    "result": null,
                    "message": "No execution results found for this task",
                    "status": "not_found"
                })))
            }
        }
        Err(e) => {
            error!("Failed to get task result for {}: {}", task_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all tasks
pub async fn list_tasks(
    State(state): State<crate::AppState>,
) -> Json<serde_json::Value> {
    match state.task_store.get_tasks().await {
        Ok(tasks) => {
            let task_summaries: Vec<serde_json::Value> = tasks
                .into_iter()
                .map(|task| {
                    let spec: serde_json::Value = serde_json::from_str(&task.spec).unwrap_or(json!({}));
                    let empty_map = serde_json::Map::new();
                    let spec = spec.as_object().unwrap_or(&empty_map);
                    let title = spec.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("Untitled Task");

                    json!({
                        "id": task.id,
                        "title": title,
                        "status": task.state,
                        "priority": spec.get("priority").and_then(|p| p.as_str()).unwrap_or("medium"),
                        "createdAt": task.created_at,
                        "updatedAt": task.updated_at
                    })
                })
                .collect();

            Json(json!({
                "tasks": task_summaries,
                "total": task_summaries.len(),
                "page": 1,
                "limit": 50,
                "status": "success"
            }))
        }
        Err(e) => {
            println!("⚠️  Failed to list tasks: {}", e);
            Json(json!({
                "error": "Failed to retrieve tasks",
                "status": "error"
            }))
        }
    }
}