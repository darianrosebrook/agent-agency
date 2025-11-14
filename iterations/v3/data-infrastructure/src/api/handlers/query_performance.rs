//! Query Performance Monitoring API handlers
//!
//! Provides endpoints for query performance metrics, slow query alerts, and performance dashboards.
//!
//! @author @darianrosebrook

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde_json;
use std::collections::HashMap;

use crate::api::ApiState;
use crate::monitoring::query_performance::PerformanceSummary;

/// Get query performance summary
///
/// Returns overall performance statistics including total queries, slow query rate, and averages.
pub async fn get_query_performance_summary(
    State(state): State<ApiState>,
) -> Result<Json<PerformanceSummary>, StatusCode> {
    let summary = state
        .query_performance_monitor
        .get_performance_summary()
        .await;
    Ok(Json(summary))
}

/// Get all query metrics
///
/// Returns detailed metrics for all tracked queries.
/// Supports pagination via query parameters.
pub async fn get_all_query_metrics(
    State(state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = state.query_performance_monitor.get_all_metrics().await;
    let total = metrics.len();

    Ok(Json(serde_json::json!({
        "metrics": metrics,
        "total": total,
        "status": "success"
    })))
}

/// Get slow queries
///
/// Returns recent slow query alerts.
/// Query parameters:
/// - `limit`: Maximum number of alerts to return (default: 100)
pub async fn get_slow_queries(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.get("limit").and_then(|s| s.parse().ok());

    let slow_queries = state
        .query_performance_monitor
        .get_slow_queries(limit)
        .await;
    let total = slow_queries.len();

    Ok(Json(serde_json::json!({
        "slow_queries": slow_queries,
        "total": total,
        "limit": limit.unwrap_or(slow_queries.len()),
        "status": "success"
    })))
}

/// Get top slow queries
///
/// Returns queries sorted by average execution time (descending).
/// Query parameters:
/// - `limit`: Maximum number of queries to return (default: 20)
pub async fn get_top_slow_queries(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let queries = state
        .query_performance_monitor
        .get_top_slow_queries(limit)
        .await;
    let total = queries.len();

    Ok(Json(serde_json::json!({
        "queries": queries,
        "total": total,
        "limit": limit,
        "status": "success"
    })))
}
