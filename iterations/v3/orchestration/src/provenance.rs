use agent_agency_database::DatabaseClient;
use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Provenance emitter for orchestration tracking
pub struct OrchestrationProvenanceEmitter {
    /// In-memory event storage
    events: Arc<RwLock<HashMap<String, Vec<ProvenanceEvent>>>>,
    /// Active orchestration sessions
    active_sessions: Arc<RwLock<HashMap<String, OrchestrationSession>>>,
    /// Database client for persistence
    database_client: Option<Arc<DatabaseClient>>,
}

#[derive(Debug, Clone)]
struct ProvenanceEvent {
    event_id: Uuid,
    event_type: String,
    payload: Value,
    timestamp: chrono::DateTime<chrono::Utc>,
    task_id: Option<String>,
}

#[derive(Debug, Clone)]
struct OrchestrationSession {
    task_id: String,
    scope: Vec<String>,
    deterministic: bool,
    start_time: chrono::DateTime<chrono::Utc>,
    validation_passed: Option<bool>,
    exit_status: Option<String>,
}

impl OrchestrationProvenanceEmitter {
    pub fn new() -> Self {
        Self::with_database_client(None)
    }

    pub fn with_database_client(database_client: Option<Arc<DatabaseClient>>) -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            database_client,
        }
    }

    pub async fn emit_event(&self, event_type: &str, payload: Value) -> Result<()> {
        debug!("Emitting orchestration event: {}", event_type);

        // Validate event type
        if event_type.trim().is_empty() {
            return Err(anyhow::anyhow!("Event type cannot be empty"));
        }

        // Extract task_id from payload if present
        let task_id = payload
            .get("task_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let event = ProvenanceEvent {
            event_id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            payload: payload.clone(),
            timestamp: chrono::Utc::now(),
            task_id: task_id.clone(),
        };

        // Store event
        let mut events = self.events.write().await;
        let task_events = events
            .entry(task_id.unwrap_or_else(|| "global".to_string()))
            .or_insert_with(Vec::new);
        task_events.push(event.clone());

        // Keep only recent events (last 1000 per task)
        if task_events.len() > 1000 {
            task_events.remove(0);
        }

        debug!(
            "Successfully emitted event: {} for task: {:?}",
            event_type, event.task_id
        );

        Ok(())
    }

    pub fn orchestrate_enter(&self, task_id: &str, scope: &[String], deterministic: bool) {
        debug!("Starting orchestration session for task: {}", task_id);

        // Validate inputs
        if task_id.trim().is_empty() {
            warn!("Empty task_id provided to orchestrate_enter");
            return;
        }

        if scope.is_empty() {
            warn!("Empty scope provided for task: {}", task_id);
        }

        // 1. Asynchronous session creation with thread-safe operations
        let session_id = format!("session_{}", Uuid::new_v4());
        let session = OrchestrationSession {
            task_id: task_id.to_string(),
            scope: scope.to_vec(),
            deterministic,
            start_time: chrono::Utc::now(),
            validation_passed: None,
            exit_status: None,
        };

        // 2. Thread-safe session storage with concurrent access handling
        {
            let mut active_sessions = self.active_sessions.write().await;
            active_sessions.insert(session_id.clone(), session.clone());
            debug!(
                "Created orchestration session {} for task {}",
                session_id, task_id
            );
        }

        // 3. Async database persistence if available
        if let Some(db_client) = &self.database_client {
            if let Err(e) = self
                .persist_session_async(&session_id, &session, db_client)
                .await
            {
                warn!(
                    "Failed to persist session {} to database: {}",
                    session_id, e
                );
                // Continue with in-memory session despite database error
            }
        }

        // 4. Session event logging for observability
        let event = ProvenanceEvent {
            event_id: Uuid::new_v4(),
            event_type: "session_created".to_string(),
            task_id: task_id.to_string(),
            timestamp: chrono::Utc::now(),
            payload: serde_json::json!({
                "session_id": session_id,
                "scope": scope,
                "deterministic": deterministic
            }),
        };

        // Store event in memory
        {
            let mut events = self.events.write().await;
            let task_events = events.entry(task_id.to_string()).or_insert_with(Vec::new);
            task_events.push(event.clone());
        }

        // Persist event to database if available
        if let Some(db_client) = &self.database_client {
            if let Err(e) = self.persist_event_async(&event, db_client).await {
                warn!("Failed to persist event to database: {}", e);
            }
        }

        info!(
            "Created orchestration session {} for task {}",
            session_id, task_id
        );
    }

    pub fn validation_result(&self, task_id: &str, passed: bool) {
        debug!(
            "Recording validation result for task {}: {}",
            task_id, passed
        );

        if task_id.trim().is_empty() {
            warn!("Empty task_id provided to validation_result");
            return;
        }

        // 1. Async validation tracking with thread-safe updates
        // Update session validation status
        {
            let mut active_sessions = self.active_sessions.write().await;
            if let Some(session) = active_sessions.values_mut().find(|s| s.task_id == task_id) {
                session.validation_passed = Some(passed);
                debug!(
                    "Updated validation status for session with task {}",
                    task_id
                );
            }
        }

        // 2. Persist validation result to database if available
        if let Some(db_client) = &self.database_client {
            if let Err(e) = self
                .update_session_validation_async(task_id, passed, db_client)
                .await
            {
                warn!("Failed to persist validation update to database: {}", e);
            }
        }

        // 3. Emit validation event for observability
        let event = ProvenanceEvent {
            event_id: Uuid::new_v4(),
            event_type: if passed {
                "validation_passed"
            } else {
                "validation_failed"
            }
            .to_string(),
            task_id: task_id.to_string(),
            timestamp: chrono::Utc::now(),
            payload: serde_json::json!({
                "passed": passed
            }),
        };

        // Store event in memory
        {
            let mut events = self.events.write().await;
            let task_events = events.entry(task_id.to_string()).or_insert_with(Vec::new);
            task_events.push(event.clone());
        }

        // Persist event to database if available
        if let Some(db_client) = &self.database_client {
            if let Err(e) = self.persist_event_async(&event, db_client).await {
                warn!("Failed to persist validation event to database: {}", e);
            }
        }
        if passed {
            info!("Validation passed for task: {}", task_id);
        } else {
            warn!("Validation failed for task: {}", task_id);
        }
    }

    pub fn orchestrate_exit(&self, task_id: &str, status: &str) {
        debug!(
            "Recording orchestration exit for task {}: {}",
            task_id, status
        );

        if task_id.trim().is_empty() {
            warn!("Empty task_id provided to orchestrate_exit");
            return;
        }

        // Validate status
        if status.trim().is_empty() {
            warn!("Empty status provided for task: {}", task_id);
            return;
        }

        // 1. Session completion tracking with duration calculation
        let exit_time = chrono::Utc::now();
        let mut session_duration = None;

        // Update session with completion status
        {
            let mut active_sessions = self.active_sessions.write().await;
            if let Some(session) = active_sessions.values_mut().find(|s| s.task_id == task_id) {
                session.exit_status = Some(status.to_string());

                // 2. Calculate session duration
                session_duration = Some(exit_time.signed_duration_since(session.start_time));
                debug!(
                    "Session for task {} completed with duration: {:.2}s",
                    task_id,
                    session_duration.unwrap().num_seconds() as f64
                );
            }
        }

        // 3. Persist session completion to database if available
        if let Some(db_client) = &self.database_client {
            if let Err(e) = self
                .complete_session_async(task_id, status, exit_time, session_duration, db_client)
                .await
            {
                warn!("Failed to persist session completion to database: {}", e);
            }
        }

        // 4. Emit session completion event
        let event = ProvenanceEvent {
            event_id: Uuid::new_v4(),
            event_type: "session_completed".to_string(),
            task_id: task_id.to_string(),
            timestamp: exit_time,
            payload: serde_json::json!({
                "exit_status": status,
                "duration_seconds": session_duration.map(|d| d.num_seconds()),
                "completed_at": exit_time.to_rfc3339()
            }),
        };

        // Store event in memory
        {
            let mut events = self.events.write().await;
            let task_events = events.entry(task_id.to_string()).or_insert_with(Vec::new);
            task_events.push(event.clone());
        }

        // Persist event to database if available
        if let Some(db_client) = &self.database_client {
            if let Err(e) = self.persist_event_async(&event, db_client).await {
                warn!("Failed to persist completion event to database: {}", e);
            }
        }

        info!(
            "Orchestration completed for task {} with status: {} (duration: {:.2}s)",
            task_id,
            status,
            session_duration
                .map(|d| d.num_seconds() as f64)
                .unwrap_or(0.0)
        );
    }

    /// Get events for a specific task
    pub async fn get_task_events(&self, task_id: &str) -> Result<Vec<ProvenanceEvent>> {
        let events = self.events.read().await;
        Ok(events.get(task_id).cloned().unwrap_or_default())
    }

    /// Get all stored events
    pub async fn get_all_events(&self) -> Result<HashMap<String, Vec<ProvenanceEvent>>> {
        let events = self.events.read().await;
        Ok(events.clone())
    }

    /// Get event count for a task
    pub async fn get_event_count(&self, task_id: &str) -> Result<usize> {
        let events = self.events.read().await;
        Ok(events.get(task_id).map(|e| e.len()).unwrap_or(0))
    }

    /// Clear events for a task (for testing/cleanup)
    pub async fn clear_task_events(&self, task_id: &str) -> Result<()> {
        let mut events = self.events.write().await;
        events.remove(task_id);
        debug!("Cleared events for task: {}", task_id);
        Ok(())
    }

    /// Persist session to database asynchronously
    async fn persist_session_async(
        &self,
        session_id: &str,
        session: &OrchestrationSession,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<()> {
        // Insert session into database
        let query = r#"
            INSERT INTO orchestration_sessions (
                session_id, task_id, scope, deterministic,
                start_time, validation_passed, exit_status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        sqlx::query(query)
            .bind(session_id)
            .bind(&session.task_id)
            .bind(&session.scope)
            .bind(session.deterministic)
            .bind(session.start_time)
            .bind(session.validation_passed)
            .bind(&session.exit_status)
            .execute(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to persist session: {}", e))?;

        debug!("Persisted session {} to database", session_id);
        Ok(())
    }

    /// Persist event to database asynchronously
    async fn persist_event_async(
        &self,
        event: &ProvenanceEvent,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<()> {
        // Insert event into database
        let query = r#"
            INSERT INTO orchestration_events (
                event_id, event_type, task_id, timestamp, payload
            ) VALUES ($1, $2, $3, $4, $5)
        "#;

        sqlx::query(query)
            .bind(event.event_id)
            .bind(&event.event_type)
            .bind(&event.task_id)
            .bind(event.timestamp)
            .bind(&event.payload)
            .execute(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to persist event: {}", e))?;

        debug!("Persisted event {} to database", event.event_id);
        Ok(())
    }

    /// Update session validation status in database asynchronously
    async fn update_session_validation_async(
        &self,
        task_id: &str,
        passed: bool,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<()> {
        // Update session validation status in database
        let query = r#"
            UPDATE orchestration_sessions
            SET validation_passed = $1
            WHERE task_id = $2
        "#;

        sqlx::query(query)
            .bind(passed)
            .bind(task_id)
            .execute(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update session validation: {}", e))?;

        debug!("Updated validation status for task {} in database", task_id);
        Ok(())
    }

    /// Complete session in database asynchronously
    async fn complete_session_async(
        &self,
        task_id: &str,
        exit_status: &str,
        exit_time: chrono::DateTime<chrono::Utc>,
        duration: Option<chrono::Duration>,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<()> {
        // Update session with completion data
        let duration_seconds = duration.map(|d| d.num_seconds() as f64);

        let query = r#"
            UPDATE orchestration_sessions
            SET exit_status = $1
            WHERE task_id = $2
        "#;

        sqlx::query(query)
            .bind(exit_status)
            .bind(task_id)
            .execute(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to complete session: {}", e))?;

        // Insert session completion metrics
        if let Some(duration_secs) = duration_seconds {
            let metrics_query = r#"
                INSERT INTO orchestration_metrics (
                    task_id, metric_type, metric_value, recorded_at
                ) VALUES ($1, $2, $3, $4)
            "#;

            sqlx::query(metrics_query)
                .bind(task_id)
                .bind("session_duration_seconds")
                .bind(duration_secs)
                .bind(exit_time)
                .execute(db_client.pool())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to record session metrics: {}", e))?;
        }

        debug!("Completed session for task {} in database", task_id);
        Ok(())
    }
}
