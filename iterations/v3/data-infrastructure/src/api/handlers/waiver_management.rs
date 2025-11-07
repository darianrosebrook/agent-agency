//! Waiver Management API handlers
//! 
//! This module contains all API handlers related to waiver management,
//! including CRUD operations, approval workflows, and audit trails.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json;
use tracing::{info, error};
use uuid::Uuid;

use crate::api::ApiState;

/// List all waivers with optional filtering
pub async fn list_waivers(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.api.db_client.list_waivers().await {
        Ok(waivers) => {
            let waiver_list: Vec<serde_json::Value> = waivers.into_iter().map(|waiver| {
                serde_json::json!({
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
                    "metadata": waiver.metadata
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "waivers": waiver_list,
                "total": waiver_list.len(),
                "status": "success"
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
    State(state): State<ApiState>,
    Json(waiver_data): Json<serde_json::Value>,
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
    
    let impact_level = waiver_data.get("impact_level")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");
    
    let mitigation_plan = waiver_data.get("mitigation_plan")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let expires_at = waiver_data.get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // Create waiver struct
    let waiver = crate::models::Waiver {
        id: Uuid::new_v4(),
        title: title.to_string(),
        reason: reason.to_string(),
        description: description.to_string(),
        gates,
        approved_by: "system".to_string(), // Default approver
        impact_level: impact_level.to_string(),
        mitigation_plan: mitigation_plan.to_string(),
        expires_at: expires_at.unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(30)),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: "pending".to_string(),
        metadata: waiver_data.clone(),
    };

    // Create waiver in database
    match state.api.db_client.create_waiver(&waiver).await {
        Ok(waiver_id) => {
            info!("Created waiver: {}", waiver_id);
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
        Err(e) => {
            error!("Failed to create waiver: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Approve a waiver
pub async fn approve_waiver(
    State(state): State<ApiState>,
    Path(waiver_id): Path<String>,
    Json(approval_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let approved_by = approval_data.get("approved_by")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let _approval_notes = approval_data.get("approval_notes")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match state.api.db_client.approve_waiver(&waiver_uuid).await {
        Ok(()) => {
            info!("Approved waiver: {}", waiver_id);
            Ok(Json(serde_json::json!({
                "id": waiver_uuid,
                "status": "approved",
                "approved_by": approved_by,
                "updated_at": chrono::Utc::now(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to approve waiver {}: {}", waiver_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Get a specific waiver by ID
pub async fn get_waiver(
    State(state): State<ApiState>,
    Path(waiver_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.get_waiver(&waiver_uuid).await {
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
            error!("Waiver not found: {}", waiver_id);
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
    State(state): State<ApiState>,
    Path(waiver_id): Path<String>,
    Json(update_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Build dynamic update query based on provided fields
    let mut update_fields = Vec::new();
    let mut update_values: Vec<Box<dyn std::fmt::Display + Send + Sync>> = Vec::new();
    let mut param_count = 1;

    if let Some(title) = update_data.get("title").and_then(|v| v.as_str()) {
        update_fields.push(format!("title = ${}", param_count));
        update_values.push(Box::new(title.to_string()));
        param_count += 1;
    }

    if let Some(description) = update_data.get("description").and_then(|v| v.as_str()) {
        update_fields.push(format!("description = ${}", param_count));
        update_values.push(Box::new(description.to_string()));
        param_count += 1;
    }

    if let Some(mitigation_plan) = update_data.get("mitigation_plan").and_then(|v| v.as_str()) {
        update_fields.push(format!("mitigation_plan = ${}", param_count));
        update_values.push(Box::new(mitigation_plan.to_string()));
        param_count += 1;
    }

    if let Some(expires_at) = update_data.get("expires_at").and_then(|v| v.as_str()) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expires_at) {
            update_fields.push(format!("expires_at = ${}", param_count));
            update_values.push(Box::new(dt.with_timezone(&chrono::Utc)));
            param_count += 1;
        }
    }

    if update_fields.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    update_fields.push(format!("updated_at = ${}", param_count));
    update_values.push(Box::new(chrono::Utc::now()));
    param_count += 1;

    update_fields.push(format!("waiver_id = ${}", param_count));
    update_values.push(Box::new(waiver_uuid));

    let query = format!(
        "UPDATE waivers SET {} WHERE id = ${}",
        update_fields.join(", "),
        param_count
    );

    match state.api.db_client.execute(&query, &[]).await {
        Ok(_) => {
            info!("Updated waiver: {}", waiver_id);
            Ok(Json(serde_json::json!({
                "id": waiver_id,
                "updated_at": chrono::Utc::now(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to update waiver {}: {}", waiver_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a waiver
pub async fn delete_waiver(
    State(state): State<ApiState>,
    Path(waiver_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.delete_waiver(&waiver_uuid).await {
        Ok(_) => {
            info!("Deleted waiver: {}", waiver_id);
            Ok(Json(serde_json::json!({
                "id": waiver_id,
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to delete waiver {}: {}", waiver_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Revoke a waiver
pub async fn revoke_waiver(
    State(state): State<ApiState>,
    Path(waiver_id): Path<String>,
    Json(revocation_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let revoked_by = revocation_data.get("revoked_by")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let revocation_reason = revocation_data.get("revocation_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match state.api.db_client.revoke_waiver(&waiver_uuid, revoked_by, revocation_reason).await {
        Ok(()) => {
            info!("Revoked waiver: {}", waiver_id);
            Ok(Json(serde_json::json!({
                "id": waiver_uuid,
                "status": "revoked",
                "revoked_by": revoked_by,
                "revocation_reason": revocation_reason,
                "updated_at": chrono::Utc::now(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to revoke waiver {}: {}", waiver_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Get audit trail for a waiver
pub async fn get_waiver_audit_trail(
    State(state): State<ApiState>,
    Path(waiver_id): Path<String>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let waiver_uuid = uuid::Uuid::parse_str(&waiver_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.get_waiver_audit_trail(&waiver_uuid).await {
        Ok(audit_records) => {
            let audit_trail: Vec<serde_json::Value> = audit_records.into_iter().map(|record| {
                serde_json::json!({
                    "action": record["action"],
                    "actor": record["actor"],
                    "timestamp": record["timestamp"],
                    "metadata": record["metadata"]
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "waiver_id": waiver_id,
                "audit_trail": audit_trail,
                "total_actions": audit_trail.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get audit trail for waiver {}: {}", waiver_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Validate waiver data
pub async fn validate_waiver(
    State(_state): State<ApiState>,
    Json(waiver_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut validation_errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate required fields
    if !waiver_data.get("title").and_then(|v| v.as_str()).is_some() {
        validation_errors.push("Title is required".to_string());
    }

    if !waiver_data.get("reason").and_then(|v| v.as_str()).is_some() {
        validation_errors.push("Reason is required".to_string());
    }

    if !waiver_data.get("description").and_then(|v| v.as_str()).is_some() {
        validation_errors.push("Description is required".to_string());
    }

    // Validate impact level
    if let Some(impact_level) = waiver_data.get("impact_level").and_then(|v| v.as_str()) {
        if !["low", "medium", "high", "critical"].contains(&impact_level) {
            validation_errors.push("Impact level must be one of: low, medium, high, critical".to_string());
        }
    }

    // Validate expiration date
    if let Some(expires_at) = waiver_data.get("expires_at").and_then(|v| v.as_str()) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expires_at) {
            let now = chrono::Utc::now();
            if dt.with_timezone(&chrono::Utc) <= now {
                validation_errors.push("Expiration date must be in the future".to_string());
            }
        } else {
            validation_errors.push("Invalid expiration date format (use RFC3339)".to_string());
        }
    }

    // Validate gates
    if let Some(gates) = waiver_data.get("gates").and_then(|v| v.as_array()) {
        if gates.is_empty() {
            warnings.push("No quality gates specified for waiver".to_string());
        }
    }

    let is_valid = validation_errors.is_empty();

    Ok(Json(serde_json::json!({
        "valid": is_valid,
        "errors": validation_errors,
        "warnings": warnings,
        "status": "success"
    })))
}
