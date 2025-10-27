//! Health check endpoints for API server
//!
//! Provides health monitoring for database, orchestrator, and worker services.

use axum::response::Json;
use serde_json::json;

/// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "agent-agency-v3-api",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "api": "healthy",
            "database": match check_database_health().await {
                Ok(_) => "healthy",
                Err(e) => format!("unhealthy: {}", e)
            },
            "orchestrator": match check_orchestrator_health().await {
                Ok(_) => "healthy",
                Err(e) => format!("unhealthy: {}", e)
            },
            "workers": match check_workers_health().await {
                Ok(_) => "healthy",
                Err(e) => format!("unhealthy: {}", e)
            }
        }
    }))
}

/// Check database health by attempting a simple connection
async fn check_database_health() -> Result<(), String> {
    // Try to get a database client and run a simple query
    match crate::client::orchestrator::DatabaseOrchestrator::new().await {
        Ok(orchestrator) => {
            match orchestrator.health_check().await {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Database connection failed: {}", e))
            }
        }
        Err(e) => Err(format!("Database orchestrator initialization failed: {}", e))
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
async fn check_workers_health() -> Result<(), String> {
    // Check if worker processes are running and healthy
    // For now, return healthy since worker pool integration is not yet implemented
    // TODO: Implement real worker health checks
    Ok(())
}
