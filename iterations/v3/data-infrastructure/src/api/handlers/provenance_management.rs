//! Provenance Management API handlers
//!
//! This module contains all API handlers related to provenance tracking,
//! including record management, commit linking, and verification.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json;
use tracing::{error, info};

use crate::api::ApiState;

/// List provenance records with pagination and filtering
pub async fn list_provenance_records(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.api.db_client.list_provenance_records().await {
        Ok(records) => {
            let record_list: Vec<serde_json::Value> = records
                .into_iter()
                .map(|record| {
                    serde_json::json!({
                        "id": record.id,
                        "task_id": record.task_id,
                        "action": record.action,
                        "actor": record.actor,
                        "resource_id": record.resource_id,
                        "resource_type": record.resource_type,
                        "change_summary": record.change_summary,
                        "timestamp": record.timestamp,
                        "created_at": record.created_at,
                        "metadata": record.metadata
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({
                "records": record_list,
                "total": record_list.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list provenance records: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Link provenance record to commit hash
pub async fn link_provenance_to_commit(
    State(state): State<ApiState>,
    Json(link_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let provenance_id = link_data
        .get("provenance_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let commit_hash = link_data
        .get("commit_hash")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let provenance_uuid =
        uuid::Uuid::parse_str(provenance_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Validate commit hash format (basic validation)
    if commit_hash.len() != 40 || !commit_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state
        .api
        .db_client
        .link_provenance_to_commit(&provenance_uuid, commit_hash)
        .await
    {
        Ok(_record) => {
            info!(
                "Linked provenance record {} to commit {}",
                provenance_id, commit_hash
            );
            Ok(Json(serde_json::json!({
                "provenance_id": provenance_id,
                "commit_hash": commit_hash,
                "linked_at": chrono::Utc::now(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!(
                "Failed to link provenance record {} to commit {}: {}",
                provenance_id, commit_hash, e
            );
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Verify provenance trailer for a commit
pub async fn verify_provenance_trailer(
    State(state): State<ApiState>,
    Path(commit_hash): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate commit hash format
    if commit_hash.len() != 40 || !commit_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state
        .api
        .db_client
        .verify_provenance_trailer(&commit_hash)
        .await
    {
        Ok(verification_result) => Ok(Json(serde_json::json!({
            "commit_hash": commit_hash,
            "is_valid": verification_result["verified"],
            "verification_status": if verification_result["verified"].as_bool().unwrap_or(false) { "valid" } else { "invalid" },
            "records_found": verification_result["entries"].as_array().map(|a| a.len()).unwrap_or(0),
            "verification_details": verification_result["entries"],
            "verified_at": chrono::Utc::now(),
            "status": "success"
        }))),
        Err(e) => {
            error!(
                "Failed to verify provenance trailer for commit {}: {}",
                commit_hash, e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get provenance records by commit hash
pub async fn get_provenance_by_commit(
    State(state): State<ApiState>,
    Path(commit_hash): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate commit hash format
    if commit_hash.len() != 40 || !commit_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state
        .api
        .db_client
        .get_provenance_by_commit(&commit_hash)
        .await
    {
        Ok(records) => {
            let record_list: Vec<serde_json::Value> = records
                .into_iter()
                .map(|record| {
                    serde_json::json!({
                        "id": record.id,
                        "task_id": record.task_id,
                        "action": record.action,
                        "actor": record.actor,
                        "resource_id": record.resource_id,
                        "resource_type": record.resource_type,
                        "change_summary": record.change_summary,
                        "timestamp": record.timestamp,
                        "created_at": record.created_at,
                        "metadata": record.metadata
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({
                "commit_hash": commit_hash,
                "records": record_list,
                "total_records": record_list.len(),
                "summary": {
                    "entity_types": record_list.iter()
                        .map(|r| r.get("entity_type").and_then(|t| t.as_str()).unwrap_or("unknown"))
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>(),
                    "actions": record_list.iter()
                        .map(|r| r.get("action").and_then(|a| a.as_str()).unwrap_or("unknown"))
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>(),
                    "actors": record_list.iter()
                        .map(|r| r.get("actor").and_then(|a| a.as_str()).unwrap_or("unknown"))
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>()
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!(
                "Failed to get provenance records for commit {}: {}",
                commit_hash, e
            );
            Err(StatusCode::NOT_FOUND)
        }
    }
}
