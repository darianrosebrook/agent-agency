use anyhow::{Context, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProvenanceEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Minimal in-memory provenance emitter used by the orchestrator.
#[derive(Default, Clone)]
pub struct OrchestrationProvenanceEmitter {
    events: Arc<RwLock<HashMap<String, Vec<ProvenanceEvent>>>>,
}

impl OrchestrationProvenanceEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn emit_event(&self, task_id: &str, event_type: &str, payload: Value) -> Result<()> {
        if event_type.trim().is_empty() {
            anyhow::bail!("event type cannot be empty");
        }
        let mut guard = self.events.write().await;
        let entry = guard.entry(task_id.to_string()).or_insert_with(Vec::new);
        entry.push(ProvenanceEvent {
            event_id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
        });
        if entry.len() > 500 {
            entry.remove(0);
        }
        Ok(())
    }

    pub async fn orchestrate_enter(
        &self,
        task_id: &str,
        scope: &[String],
        deterministic: bool,
    ) -> Result<()> {
        if task_id.trim().is_empty() {
            warn!(
                target = "provenance",
                "attempted to start orchestration with empty task id"
            );
            return Ok(());
        }
        debug!(
            target = "provenance",
            task_id, "starting orchestration session"
        );
        self.emit_event(
            task_id,
            "session_created",
            serde_json::json!({
                "scope": scope,
                "deterministic": deterministic,
            }),
        )
        .await
    }

    pub async fn validation_result(&self, task_id: &str, passed: bool) -> Result<()> {
        self.emit_event(
            task_id,
            "validation_result",
            serde_json::json!({ "passed": passed }),
        )
        .await
    }

    pub async fn orchestrate_exit(&self, task_id: &str, status: &str) -> Result<()> {
        info!(
            target = "provenance",
            task_id, status, "orchestration completed"
        );
        self.emit_event(
            task_id,
            "session_completed",
            serde_json::json!({ "status": status }),
        )
        .await
    }

    #[cfg(test)]
    pub async fn events_for_task(&self, task_id: &str) -> Vec<ProvenanceEvent> {
        self.events
            .read()
            .await
            .get(task_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// High-level provenance events emitted across orchestration components.
#[derive(Debug, Clone)]
pub enum ProvEvent {
    OrchestrateEnter {
        task_id: String,
        scope_in: Vec<String>,
        deterministic: bool,
    },
    OrchestrateExit {
        task_id: String,
        outcome: String,
    },
    ValidationResult {
        task_id: String,
        short_circuit: bool,
    },
    JudgeVerdict {
        task_uuid: Uuid,
        judge: String,
        weight: f32,
        decision: String,
        score: f32,
    },
    FinalVerdict {
        task_uuid: Uuid,
        summary: String,
    },
}

#[async_trait::async_trait]
pub trait ProvenanceBackend: Send + Sync {
    async fn record_event(&self, event: ProvEvent) -> Result<()>;
}

/// Simple backend that stores provenance events in memory (primarily for tests).
#[derive(Default)]
pub struct InMemoryBackend(pub Mutex<Vec<ProvEvent>>);

#[async_trait::async_trait]
impl ProvenanceBackend for InMemoryBackend {
    async fn record_event(&self, event: ProvEvent) -> Result<()> {
        self.0.lock().push(event);
        Ok(())
    }
}
