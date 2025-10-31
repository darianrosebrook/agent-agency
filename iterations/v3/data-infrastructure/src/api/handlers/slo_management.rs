//! SLO Management API handlers
//! 
//! This module contains all API handlers related to Service Level Objective (SLO)
//! management, including definition, measurement, tracking, and alerting.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json;
use tracing::{info, error};
use uuid::Uuid;

use crate::api::ApiState;

/// Acknowledge an SLO alert
pub async fn acknowledge_slo_alert(
    State(state): State<ApiState>,
    Path(alert_id): Path<String>,
    Json(ack_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let alert_uuid = uuid::Uuid::parse_str(&alert_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let acknowledged_by = ack_data.get("acknowledged_by")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let acknowledgment_notes = ack_data.get("acknowledgment_notes")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match state.api.db_client.acknowledge_slo_alert(&alert_uuid, acknowledged_by, acknowledgment_notes).await {
        Ok(()) => {
            info!("Acknowledged SLO alert: {}", alert_id);
            Ok(Json(serde_json::json!({
                "id": alert_id,
                "status": "acknowledged",
                "acknowledged_by": acknowledged_by,
                "acknowledged_at": chrono::Utc::now(),
                "acknowledgment_notes": acknowledgment_notes,
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to acknowledge SLO alert {}: {}", alert_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// List all SLOs
pub async fn list_slos(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.api.db_client.list_slos().await {
        Ok(slos) => {
            let slo_list: Vec<serde_json::Value> = slos.into_iter().map(|slo| {
                serde_json::json!({
                    "id": slo["id"],
                    "name": slo["name"],
                    "description": slo["description"],
                    "target_value": slo["target_value"],
                    "current_value": slo["current_value"],
                    "status": slo["status"],
                    "created_at": slo["created_at"],
                    "updated_at": slo["updated_at"]
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "slos": slo_list,
                "total": slo_list.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list SLOs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get SLO status and current metrics
pub async fn get_slo_status(
    State(state): State<ApiState>,
    Path(slo_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let slo_uuid = uuid::Uuid::parse_str(&slo_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.get_slo_status(&slo_uuid).await {
        Ok(Some(slo_status)) => {
            Ok(Json(serde_json::json!({
                "slo_id": slo_id,
                "name": slo_status["name"],
                "target_value": slo_status["target_value"],
                "current_value": slo_status["current_value"],
                "status": slo_status["status"],
                "last_measured": slo_status["updated_at"],
                "measurement_window": "1h", // Default value
                "alert_threshold": 0.95, // Default value
                "is_alerting": slo_status["status"] == "alerting",
                "trend": "stable", // Default value
                "metadata": serde_json::json!({}),
                "status": "success"
            })))
        }
        Ok(None) => {
            error!("SLO not found: {}", slo_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to get SLO status for {}: {}", slo_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Get SLO measurements over time
pub async fn get_slo_measurements(
    State(state): State<ApiState>,
    Path(slo_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let slo_uuid = uuid::Uuid::parse_str(&slo_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.get_slo_measurements(&slo_uuid).await {
        Ok(measurements) => {
            let measurement_list: Vec<serde_json::Value> = measurements.into_iter().map(|measurement| {
                serde_json::json!({
                    "id": measurement["id"],
                    "slo_id": measurement["slo_id"],
                    "value": measurement["value"],
                    "timestamp": measurement["timestamp"],
                    "metadata": measurement["metadata"]
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "slo_id": slo_id,
                "measurements": measurement_list,
                "total_measurements": measurement_list.len(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to get SLO measurements for {}: {}", slo_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// List SLO alerts
pub async fn list_slo_alerts(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.api.db_client.list_slo_alerts().await {
        Ok(alerts) => {
            let alert_list: Vec<serde_json::Value> = alerts.into_iter().map(|alert| {
                serde_json::json!({
                    "id": alert["id"],
                    "slo_id": alert["slo_id"],
                    "alert_type": alert["alert_type"],
                    "severity": alert["severity"],
                    "message": alert["message"],
                    "status": alert["status"],
                    "created_at": alert["created_at"],
                    "acknowledged_at": alert["acknowledged_at"],
                    "acknowledged_by": None::<String>, // Not available in current structure
                    "metadata": serde_json::json!({})
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "alerts": alert_list,
                "total": alert_list.len(),
                "active_alerts": alert_list.iter().filter(|a| a.get("status").and_then(|s| s.as_str()) == Some("active")).count(),
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list SLO alerts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
