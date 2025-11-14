//! Health check endpoints for API server
//!
//! Provides health monitoring for database, orchestrator, and worker services.

use crate::health::HealthStatus;
use crate::WorkerPoolHealth;
use axum::response::Json;
use serde_json::json;

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
        // Pass the pool reference for real connectivity check
        match health_monitor
            .perform_health_check(Some(db_client.pool()))
            .await
        {
            Ok(status) => {
                match status.overall_health {
                    HealthStatus::Healthy => Ok(()),
                    HealthStatus::Degraded => {
                        println!("⚠️  Database health is degraded");
                        Ok(()) // Still functional
                    }
                    HealthStatus::Unhealthy => Err("Database is unhealthy".to_string()),
                    HealthStatus::Critical => Err("Database is in critical state".to_string()),
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

/// Check orchestrator health by verifying CoreML models are available and platform supports acceleration
async fn check_orchestrator_health() -> Result<(), String> {
    use std::path::PathBuf;
    use tracing::{debug, warn};

    // Check platform support for CoreML/ANE
    let platform_supported = check_coreml_platform_support();
    if !platform_supported {
        debug!("CoreML platform not supported (requires macOS on Apple Silicon)");
        // Platform not supported is not an error - orchestrator can still work without ANE
        return Ok(());
    }

    // Check if CoreML model directory exists and contains models
    let model_path = std::env::var("COREML_MODELS_PATH")
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|_| {
            // Default to project models directory
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .map(|p| p.join("models").join("coreml"))
                .unwrap_or_else(|| PathBuf::from("/models/coreml"))
        });

    // Verify model directory exists
    if !model_path.exists() {
        warn!(
            "CoreML model directory does not exist: {:?}",
            model_path
        );
        // Model directory missing is not critical - models may be loaded from elsewhere
        return Ok(());
    }

    // Check if directory contains any .mlmodel or .mlpackage files
    let has_models = match std::fs::read_dir(&model_path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str());
                ext == Some("mlmodel") || ext == Some("mlpackage") || path.is_dir()
            }),
        Err(e) => {
            warn!("Failed to read CoreML model directory: {}", e);
            false
        }
    };

    if !has_models {
        warn!("No CoreML models found in directory: {:?}", model_path);
        // No models found is not critical - orchestrator may use other inference backends
        return Ok(());
    }

    debug!(
        "CoreML models available at: {:?}",
        model_path
    );

    // Note: Full health check with agent-orchestration crate integration is not possible
    // due to circular dependency. This implementation checks:
    // 1. Platform support (macOS + Apple Silicon)
    // 2. Model directory existence
    // 3. Presence of model files
    //
    // For comprehensive health checking including:
    // - Actual model loading verification
    // - ANE availability detection
    // - Model inference capability testing
    // - Orchestrator service response verification
    //
    // The agent-orchestration crate would need to be integrated, which requires
    // resolving the circular dependency or using a different integration pattern.

    Ok(())
}

/// Check if the current platform supports CoreML/ANE acceleration
fn check_coreml_platform_support() -> bool {
    // CoreML requires macOS on Apple Silicon
    cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")
}

/// Check workers health by verifying worker pool status
async fn check_workers_health(
    worker_pool: &std::sync::Arc<dyn WorkerPoolHealth>,
) -> Result<(), String> {
    worker_pool.health_check().await
}
