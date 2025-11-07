//! Query Management API handlers
//! 
//! This module contains all API handlers related to query management,
//! including saved queries and query execution.

use serde_json;
use sqlx::Row;


/// List saved queries
pub async fn list_saved_queries(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    let limit = params.get("limit").and_then(|l| l.parse::<i64>().ok()).unwrap_or(50);
    let offset = params.get("offset").and_then(|o| o.parse::<i64>().ok()).unwrap_or(0);
    let user_id = params.get("user_id");

    // Build query with optional user filter
    let _limit_ref: &(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync) = &limit;
    let _offset_ref: &(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync) = &offset;

    let (query, params_vec): (String, Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>>) = if let Some(user_id) = user_id {
        let user_id_str = user_id.as_str().to_string();
        (r#"
            SELECT id, name, description, query_sql, parameters, created_by, created_at, updated_at, is_public
            FROM saved_queries
            WHERE (created_by = $1 OR is_public = true)
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
        "#.to_string(), vec![Box::new(user_id_str), Box::new(limit), Box::new(offset)])
    } else {
        (r#"
            SELECT id, name, description, query_sql, parameters, created_by, created_at, updated_at, is_public
            FROM saved_queries
            WHERE is_public = true
            ORDER BY updated_at DESC
            LIMIT $1 OFFSET $2
        "#.to_string(), vec![Box::new(limit), Box::new(offset)])
    };

    // Convert Vec<Box<dyn ...>> to slice of references
    let params_refs: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)> = params_vec.iter().map(|b| b.as_ref()).collect();

    match state.db_client.query(&query, &params_refs).await {
        Ok(rows) => {
            let queries: Vec<serde_json::Value> = rows.into_iter()
                .map(|row| {
                    serde_json::json!({
                        "id": row.try_get::<String, _>("id").unwrap_or_default(),
                        "name": row.try_get::<String, _>("name").unwrap_or_default(),
                        "description": row.try_get::<Option<String>, _>("description").unwrap_or_default(),
                        "query_sql": row.try_get::<String, _>("query_sql").unwrap_or_default(),
                        "parameters": row.try_get::<Option<String>, _>("parameters").unwrap_or_default(),
                        "created_by": row.try_get::<String, _>("created_by").unwrap_or_default(),
                        "created_at": row.try_get::<String, _>("created_at").unwrap_or_default(),
                        "updated_at": row.try_get::<String, _>("updated_at").unwrap_or_default(),
                        "is_public": row.try_get::<bool, _>("is_public").unwrap_or(false)
                    })
                })
                .collect();

            Ok(axum::Json(serde_json::json!({
                "queries": queries,
                "total": queries.len(),
                "limit": limit,
                "offset": offset,
                "status": "success"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to list saved queries: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Save a query
pub async fn save_query(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
    axum::extract::Json(query_data): axum::extract::Json<serde_json::Value>
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    // Validate required fields
    let name = query_data.get("name")
        .and_then(|n| n.as_str())
        .ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    let query_sql = query_data.get("query_sql")
        .and_then(|q| q.as_str())
        .ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    let created_by = query_data.get("created_by")
        .and_then(|c| c.as_str())
        .unwrap_or("unknown");

    let description = query_data.get("description").and_then(|d| d.as_str());
    let parameters = query_data.get("parameters").and_then(|p| p.as_str());
    let is_public = query_data.get("is_public").and_then(|p| p.as_bool()).unwrap_or(false);

    // Validate query SQL (basic check)
    if query_sql.trim().is_empty() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Generate query ID
    let query_id = uuid::Uuid::new_v4().to_string();

    // Insert query into database
    let insert_query = r#"
        INSERT INTO saved_queries (id, name, description, query_sql, parameters, created_by, is_public, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
    "#;

    match state.db_client.execute(insert_query, &[&query_id, &name, &description, &query_sql, &parameters, &created_by, &is_public]).await {
        Ok(_) => {
            // TODO: Log the query save operation using log_audit_event

            Ok(axum::Json(serde_json::json!({
                "message": "Query saved successfully",
                "query_id": query_id,
                "name": name,
                "saved_at": chrono::Utc::now(),
                "status": "success"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to save query: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a saved query
pub async fn delete_saved_query(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
    axum::extract::Path(query_id): axum::extract::Path<String>
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    // Validate query ID format
    if let Err(_) = uuid::Uuid::parse_str(&query_id) {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Delete query from database
    let delete_query = r#"
        DELETE FROM saved_queries
        WHERE id = $1
    "#;

    match state.db_client.execute(delete_query, &[&query_id]).await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // TODO: Log the query deletion using log_audit_event

                Ok(axum::Json(serde_json::json!({
                    "message": "Query deleted successfully",
                    "query_id": query_id,
                    "deleted_at": chrono::Utc::now(),
                    "status": "success"
                })))
            } else {
                Err(axum::http::StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete query {}: {}", query_id, e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
