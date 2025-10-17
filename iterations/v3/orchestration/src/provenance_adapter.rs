use crate::provenance::{ProvEvent, ProvenanceBackend};
use anyhow::Result;

/// Adapter that forwards orchestration provenance events to the provenance service/client.
/// Replace the internals with calls into `v3/provenance` crate APIs when available.
#[derive(Clone)]
pub struct ProvenanceServiceAdapter<P: ProvenanceClient + Send + Sync + 'static> {
    client: P,
}

impl<P: ProvenanceClient + Send + Sync + 'static> ProvenanceServiceAdapter<P> {
    pub fn new(client: P) -> Self { Self { client } }
}

#[async_trait::async_trait]
impl<P: ProvenanceClient + Send + Sync + 'static> ProvenanceBackend for ProvenanceServiceAdapter<P> {
    async fn record_event(&self, event: ProvEvent) -> Result<()> {
        match event {
            ProvEvent::OrchestrateEnter { task_id, scope_in, deterministic } => {
                self.client.orchestrate_enter(&task_id, &scope_in, deterministic).await
            }
            ProvEvent::OrchestrateExit { task_id, outcome } => {
                self.client.orchestrate_exit(&task_id, &outcome).await
            }
            ProvEvent::ValidationResult { task_id, short_circuit } => {
                self.client.validation_result(&task_id, short_circuit).await
            }
            ProvEvent::JudgeVerdict { task_uuid, judge, weight, decision, score } => {
                self.client.judge_verdict(task_uuid, &judge, weight, &decision, score).await
            }
            ProvEvent::FinalVerdict { task_uuid, summary } => {
                self.client.final_verdict(task_uuid, &summary).await
            }
        }
    }
}

/// Minimal client trait to be implemented by the provenance subsystem
#[async_trait::async_trait]
pub trait ProvenanceClient {
    async fn orchestrate_enter(&self, task_id: &str, scope_in: &[String], deterministic: bool) -> Result<()>;
    async fn orchestrate_exit(&self, task_id: &str, outcome: &str) -> Result<()>;
    async fn validation_result(&self, task_id: &str, short_circuit: bool) -> Result<()>;
    async fn judge_verdict(&self, task_uuid: uuid::Uuid, judge: &str, weight: f32, decision: &str, score: f32) -> Result<()>;
    async fn final_verdict(&self, task_uuid: uuid::Uuid, summary: &str) -> Result<()>;
}

