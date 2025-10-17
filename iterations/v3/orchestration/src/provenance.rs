use council::contracts::FinalVerdict;
use council::coordinator::ProvenanceEmitter;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct OrchestrationProvenanceEmitter {
    backend: Arc<dyn ProvenanceBackend>,
}

impl OrchestrationProvenanceEmitter {
    pub fn new(backend: Arc<dyn ProvenanceBackend>) -> Self { Self { backend } }

    pub fn orchestrate_enter(&self, task_id: &str, scope_in: &[String], deterministic: bool) {
        let _ = self.backend.record_event(ProvEvent::OrchestrateEnter {
            task_id: task_id.to_string(),
            scope_in: scope_in.to_vec(),
            deterministic,
        });
    }

    pub fn orchestrate_exit(&self, task_id: &str, outcome: &str) {
        let _ = self.backend.record_event(ProvEvent::OrchestrateExit {
            task_id: task_id.to_string(),
            outcome: outcome.to_string(),
        });
    }

    pub fn validation_result(&self, task_id: &str, short_circuit: bool) {
        let _ = self.backend.record_event(ProvEvent::ValidationResult {
            task_id: task_id.to_string(),
            short_circuit,
        });
    }
}

impl ProvenanceEmitter for OrchestrationProvenanceEmitter {
    fn on_judge_verdict(&self, task_uuid: uuid::Uuid, judge: &str, weight: f32, decision: &str, score: f32) {
        let _ = self.backend.record_event(ProvEvent::JudgeVerdict {
            task_uuid,
            judge: judge.to_string(),
            weight,
            decision: decision.to_string(),
            score,
        });
    }

    fn on_final_verdict(&self, task_uuid: uuid::Uuid, verdict: &FinalVerdict) {
        let summary = match verdict {
            FinalVerdict::Accepted { summary, .. } => summary.clone(),
            FinalVerdict::Rejected { reason, .. } => reason.clone(),
            _ => String::new(),
        };
        let _ = self.backend.record_event(ProvEvent::FinalVerdict {
            task_uuid,
            summary,
        });
    }
}

#[derive(Debug)]
pub enum ProvEvent {
    OrchestrateEnter { task_id: String, scope_in: Vec<String>, deterministic: bool },
    OrchestrateExit { task_id: String, outcome: String },
    ValidationResult { task_id: String, short_circuit: bool },
    JudgeVerdict { task_uuid: uuid::Uuid, judge: String, weight: f32, decision: String, score: f32 },
    FinalVerdict { task_uuid: uuid::Uuid, summary: String },
}

#[async_trait::async_trait]
pub trait ProvenanceBackend: Send + Sync {
    async fn record_event(&self, event: ProvEvent) -> anyhow::Result<()>;
}

/// In-memory backend for tests
pub struct InMemoryBackend(pub parking_lot::Mutex<Vec<ProvEvent>>);

#[async_trait::async_trait]
impl ProvenanceBackend for InMemoryBackend {
    async fn record_event(&self, event: ProvEvent) -> anyhow::Result<()> {
        self.0.lock().push(event);
        Ok(())
    }
}

