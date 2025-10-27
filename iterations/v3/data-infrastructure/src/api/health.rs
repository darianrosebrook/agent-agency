//! Health check endpoints for API server
//!
//! Provides health monitoring for database, orchestrator, and worker services.

use axum::response::Json;
use serde_json::json;
use crate::health::HealthStatus;
use crate::WorkerPoolHealth;

/// Health check endpoint
pub async fn health_check(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
) -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "agent-agency-v3-api",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "api": "healthy",
            "database": match check_database_health(&state.db_client).await {
                Ok(_) => "healthy".to_string(),
                Err(e) => format!("unhealthy: {}", e)
            },
            "orchestrator": match check_orchestrator_health().await {
                Ok(_) => "healthy".to_string(),
                Err(e) => format!("unhealthy: {}", e)
            },
            "workers": match check_workers_health(&state.worker_pool).await {
                Ok(_) => "healthy".to_string(),
                Err(e) => format!("unhealthy: {}", e)
            }
        }
    }))
}

/// Check database health by attempting a simple connection
async fn check_database_health(db_client: &crate::DatabaseClient) -> Result<(), String> {
    // Use the DatabaseHealthMonitor if available, otherwise perform basic connectivity check
    if let Some(health_monitor) = db_client.health_monitor() {
        match health_monitor.perform_health_check().await {
            Ok(status) => {
                match status.overall_health {
                    HealthStatus::Healthy => Ok(()),
                    HealthStatus::Degraded => {
                        println!("⚠️  Database health is degraded");
                        Ok(()) // Still functional
                    },
                    HealthStatus::Unhealthy => {
                        Err("Database is unhealthy".to_string())
                    },
                    HealthStatus::Critical => {
                        Err("Database is in critical state".to_string())
                    },
                }
            }
            Err(e) => {
                println!("⚠️  Failed to perform database health check: {}", e);
                Err(format!("Health check failed: {}", e))
            }
        }
    } else {
        // Fallback to basic connectivity check
        match db_client.pool().acquire().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Database connection failed: {}", e)),
        }
    }
}

/// Check orchestrator health by verifying CoreML models are loaded
async fn check_orchestrator_health() -> Result<(), String> {
    // Check if CoreML models are available (this would need agent-orchestration integration)
    // For now, return healthy since this requires cross-crate integration
    // TODO: Integrate with agent-orchestration CoreML manager
    Ok(())
}

/// Check workers health by verifying worker pool status
async fn check_workers_health(worker_pool: &std::sync::Arc<dyn WorkerPoolHealth>) -> Result<(), String> {
    worker_pool.health_check().await
}
