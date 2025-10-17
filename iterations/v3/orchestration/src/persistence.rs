use anyhow::Result;
use council::contracts as api;

/// Placeholder trait for verdict persistence
#[async_trait::async_trait]
pub trait VerdictWriter: Send + Sync {
    async fn persist_verdict(&self, task_id: &str, verdict: &api::FinalVerdict) -> Result<()>;
    async fn persist_waivers(&self, task_id: &str, waivers: &[api::Waiver]) -> Result<()>;
}

/// In-memory stub implementation; replace with DB client (Postgres) later.
pub struct InMemoryWriter;

#[async_trait::async_trait]
impl VerdictWriter for InMemoryWriter {
    async fn persist_verdict(&self, _task_id: &str, _verdict: &api::FinalVerdict) -> Result<()> { Ok(()) }
    async fn persist_waivers(&self, _task_id: &str, _waivers: &[api::Waiver]) -> Result<()> { Ok(()) }
}

