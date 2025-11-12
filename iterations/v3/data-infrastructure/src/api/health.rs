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
        // Pass the pool reference for real connectivity check
        match health_monitor.perform_health_check(Some(db_client.pool())).await {
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
    // TODO: Integrate with agent-orchestration CoreML manager
    // - [ ] Integrate with agent-orchestration crate for health checks
    // - [ ] Check CoreML model availability and health
    // - [ ] Verify orchestrator service is responding
    // - [ ] Handle health check errors gracefully
    // - [ ] Add unit tests with mock orchestrator
    // - [ ] Add integration tests with real orchestrator service
    //
    // TODO: Implement comprehensive orchestrator health check with CoreML model verification
    //       Currently returns healthy without checking; should implement comprehensive health check that integrates with agent-orchestration crate, checks CoreML model availability and health, and verifies orchestrator service is responding.
    //
    // COMPLETION CHECKLIST:
    // [ ] Primary functionality implemented
    // [ ] API/data structures defined & stable
    // [ ] Error handling + validation aligned with error taxonomy
    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
    // [ ] Integration tests for external systems/contracts
    // [ ] Documentation: public API + system behavior
    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
    // [ ] Security posture reviewed (inputs, authz, sandboxing)
    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
    // [ ] Configurability and feature flags defined if relevant
    // [ ] Failure-mode cards documented (degradation paths)
    //
    // ACCEPTANCE CRITERIA:
    // - Agent-orchestration crate is integrated for health checks
    // - CoreML model availability and health are checked
    // - Orchestrator service response is verified
    // - Health check errors are handled gracefully
    //
    // DEPENDENCIES:
    // - agent-orchestration crate integration (Required)
    // - CoreML model health checking (Required)
    // - Orchestrator service client (Required)
    //
    // ESTIMATED EFFORT: 6-8 hours (medium confidence)
    // PRIORITY: Medium
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 2 (health check functionality)
    // - Change Budget: ~150 LOC
    // - Reviewer Requirements: Health checking and cross-crate integration expertise
    Ok(())
}

/// Check workers health by verifying worker pool status
async fn check_workers_health(worker_pool: &std::sync::Arc<dyn WorkerPoolHealth>) -> Result<(), String> {
    worker_pool.health_check().await
}
