//! Telemetry Service for LLM Request and Agent Activity Logging
//!
//! Provides a centralized service for recording telemetry data including:
//! - LLM request metrics (tokens, response times, costs)
//! - Agent activity events (task execution, completions, failures)
//! - Daily task statistics snapshots
//!
//! @author @darianrosebrook

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Telemetry service for recording LLM and agent activity metrics
#[derive(Clone)]
pub struct TelemetryService {
    db_client: Option<Arc<crate::simple_client::DatabaseClient>>,
    last_snapshot_check: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl TelemetryService {
    /// Create a new telemetry service
    pub fn new(db_client: Option<Arc<crate::simple_client::DatabaseClient>>) -> Self {
        Self {
            db_client,
            last_snapshot_check: Arc::new(RwLock::new(None)),
        }
    }

    /// Record an LLM request for telemetry
    ///
    /// # Arguments
    /// * `model_name` - Name of the LLM model (e.g., "gpt-4", "claude-3-opus")
    /// * `provider` - Provider name (e.g., "openai", "anthropic")
    /// * `task_id` - Optional task ID this request is associated with
    /// * `agent_id` - Optional agent ID that made the request
    /// * `prompt_tokens` - Number of tokens in the prompt
    /// * `completion_tokens` - Number of tokens in the completion
    /// * `response_time_ms` - Response time in milliseconds
    /// * `success` - Whether the request was successful
    /// * `error_message` - Optional error message if request failed
    /// * `cost_usd` - Optional cost in USD
    pub async fn record_llm_request(
        &self,
        model_name: &str,
        provider: &str,
        task_id: Option<Uuid>,
        agent_id: Option<Uuid>,
        prompt_tokens: i32,
        completion_tokens: i32,
        response_time_ms: Option<i32>,
        success: bool,
        error_message: Option<&str>,
        cost_usd: Option<f64>,
    ) -> Result<Option<Uuid>> {
        let db = match &self.db_client {
            Some(db) => db,
            None => {
                tracing::debug!("Telemetry service: No database client, skipping LLM request log");
                return Ok(None);
            }
        };

        match db
            .record_llm_request(
                model_name,
                provider,
                task_id,
                agent_id,
                prompt_tokens,
                completion_tokens,
                response_time_ms,
                success,
                error_message,
                cost_usd,
                None, // metadata
            )
            .await
        {
            Ok(id) => {
                tracing::debug!(
                    "Recorded LLM request: model={}, tokens={}, success={}",
                    model_name,
                    prompt_tokens + completion_tokens,
                    success
                );
                Ok(Some(id))
            }
            Err(e) => {
                tracing::warn!("Failed to record LLM request: {}", e);
                // Don't fail the main operation if telemetry fails
                Ok(None)
            }
        }
    }

    /// Record agent activity for telemetry
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent
    /// * `activity_type` - Type of activity (e.g., "task_started", "task_completed")
    /// * `task_id` - Optional task ID this activity is associated with
    /// * `duration_ms` - Optional duration in milliseconds
    /// * `success` - Whether the activity was successful
    /// * `error_message` - Optional error message if activity failed
    pub async fn record_agent_activity(
        &self,
        agent_id: Uuid,
        activity_type: &str,
        task_id: Option<Uuid>,
        duration_ms: Option<i32>,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<Option<Uuid>> {
        let db = match &self.db_client {
            Some(db) => db,
            None => {
                tracing::debug!(
                    "Telemetry service: No database client, skipping agent activity log"
                );
                return Ok(None);
            }
        };

        match db
            .record_agent_activity(
                agent_id,
                activity_type,
                task_id,
                duration_ms,
                success,
                error_message,
                None, // metadata
            )
            .await
        {
            Ok(id) => {
                tracing::debug!(
                    "Recorded agent activity: agent={}, type={}, success={}",
                    agent_id,
                    activity_type,
                    success
                );
                Ok(Some(id))
            }
            Err(e) => {
                tracing::warn!("Failed to record agent activity: {}", e);
                // Don't fail the main operation if telemetry fails
                Ok(None)
            }
        }
    }

    /// Check and trigger daily task stats snapshot if needed
    ///
    /// This method should be called periodically (e.g., on each API request or in a background task)
    /// to ensure daily snapshots are taken.
    pub async fn maybe_snapshot_task_stats(&self) -> Result<bool> {
        let db = match &self.db_client {
            Some(db) => db,
            None => return Ok(false),
        };

        // Check if we've already checked recently (within the last hour)
        let mut last_check = self.last_snapshot_check.write().await;
        let now = chrono::Utc::now();

        if let Some(last) = *last_check {
            if now - last < chrono::Duration::hours(1) {
                return Ok(false);
            }
        }

        // Update last check time
        *last_check = Some(now);
        drop(last_check);

        // Check if a snapshot exists for today
        match db.has_snapshot_today().await {
            Ok(true) => {
                tracing::debug!("Task stats snapshot already exists for today");
                Ok(false)
            }
            Ok(false) => {
                // Take a snapshot
                match db.snapshot_task_stats().await {
                    Ok(()) => {
                        tracing::info!("Created daily task stats snapshot");
                        Ok(true)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create task stats snapshot: {}", e);
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to check for existing snapshot: {}", e);
                Ok(false)
            }
        }
    }

    /// Get model contributions for the specified time period
    pub async fn get_model_contributions(
        &self,
        hours: Option<i32>,
    ) -> Result<Vec<serde_json::Value>> {
        let db = match &self.db_client {
            Some(db) => db,
            None => return Ok(Vec::new()),
        };

        db.get_model_contributions(hours).await
    }

    /// Get agent activity for the specified time period
    pub async fn get_agent_activity(&self, hours: Option<i32>) -> Result<Vec<serde_json::Value>> {
        let db = match &self.db_client {
            Some(db) => db,
            None => return Ok(Vec::new()),
        };

        db.get_agent_activity(hours).await
    }

    /// Get task stats history for the specified time period
    pub async fn get_task_stats_history(
        &self,
        days: Option<i32>,
    ) -> Result<Vec<serde_json::Value>> {
        let db = match &self.db_client {
            Some(db) => db,
            None => return Ok(Vec::new()),
        };

        db.get_task_stats_history(days).await
    }
}

/// Activity types for agent telemetry
pub mod activity_types {
    pub const TASK_STARTED: &str = "task_started";
    pub const TASK_COMPLETED: &str = "task_completed";
    pub const TASK_FAILED: &str = "task_failed";
    pub const TASK_CANCELLED: &str = "task_cancelled";
    pub const TASK_PAUSED: &str = "task_paused";
    pub const TASK_RESUMED: &str = "task_resumed";
    pub const INFERENCE_STARTED: &str = "inference_started";
    pub const INFERENCE_COMPLETED: &str = "inference_completed";
    pub const INFERENCE_FAILED: &str = "inference_failed";
    pub const COUNCIL_EVALUATION: &str = "council_evaluation";
    pub const DEBATE_ROUND: &str = "debate_round";
    pub const WORKER_PLEADING: &str = "worker_pleading";
}

/// LLM providers for telemetry
pub mod providers {
    pub const OPENAI: &str = "openai";
    pub const ANTHROPIC: &str = "anthropic";
    pub const COREML: &str = "coreml";
    pub const LOCAL: &str = "local";
    pub const UNKNOWN: &str = "unknown";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_service_without_db() {
        let service = TelemetryService::new(None);

        // Should not fail without database
        let result = service
            .record_llm_request("gpt-4", "openai", None, None, 100, 50, Some(500), true, None, None)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let result = service
            .record_agent_activity(Uuid::new_v4(), "task_started", None, None, true, None)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

