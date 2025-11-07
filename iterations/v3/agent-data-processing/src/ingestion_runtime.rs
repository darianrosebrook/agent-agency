//! Runtime for handling ingestion commands with queue, workers, and deduplication

use std::{collections::HashSet, path::PathBuf, sync::Arc};

use tokio::sync::{broadcast, Mutex};
use moka::future::Cache;
use tracing::{info, error};

use crate::{
    data_processing_types::*,
    ingestion::DefaultIngestionStage,
    pipeline::PipelineStage,
    DataProcessingResult,
};

/// Commands for the ingestion runtime
#[derive(Debug, Clone)]
pub enum IngestionCmd {
    FileUpsert { path: PathBuf },
    FileRemove { path: PathBuf },
}

/// Hooks that upper layers can provide
pub type OutputHook = Arc<dyn Fn(ProcessingOutput) -> futures::future::BoxFuture<'static, ()> + Send + Sync>;
pub type RemovalHook = Arc<dyn Fn(PathBuf) -> futures::future::BoxFuture<'static, ()> + Send + Sync>;

/// Runtime for handling ingestion operations with bounded queue and workers
pub struct IngestionRuntime {
    tx: broadcast::Sender<IngestionCmd>,
}

impl IngestionRuntime {
    /// Get a sender for queuing ingestion commands
    pub fn sender(&self) -> broadcast::Sender<IngestionCmd> {
        self.tx.clone()
    }
}

/// Builder for configuring the ingestion runtime
pub struct IngestionRuntimeBuilder {
    concurrency: usize,
    queue_capacity: usize,
    output_hook: Option<OutputHook>,
    removal_hook: Option<RemovalHook>,
}

impl Default for IngestionRuntimeBuilder {
    fn default() -> Self {
        Self {
            concurrency: 4,
            queue_capacity: 256,
            output_hook: None,
            removal_hook: None,
        }
    }
}

impl IngestionRuntimeBuilder {
    /// Set the number of worker tasks
    pub fn concurrency(mut self, n: usize) -> Self {
        self.concurrency = n;
        self
    }

    /// Set the maximum queue capacity
    pub fn queue_capacity(mut self, n: usize) -> Self {
        self.queue_capacity = n;
        self
    }

    /// Set the hook for processing successful ingestion outputs
    pub fn output_hook<F, Fut>(mut self, f: F) -> Self
    where F: Fn(ProcessingOutput) -> Fut + Send + Sync + 'static,
          Fut: std::future::Future<Output=()> + Send + 'static {
        self.output_hook = Some(Arc::new(move |o| Box::pin(f(o))));
        self
    }

    /// Set the hook for handling file removal cleanup
    pub fn removal_hook<F, Fut>(mut self, f: F) -> Self
    where F: Fn(PathBuf) -> Fut + Send + Sync + 'static,
          Fut: std::future::Future<Output=()> + Send + 'static {
        self.removal_hook = Some(Arc::new(move |p| Box::pin(f(p))));
        self
    }

    /// Build the ingestion runtime with configured parameters
    pub async fn build(self) -> DataProcessingResult<IngestionRuntime> {
        let (tx, _) = broadcast::channel::<IngestionCmd>(self.queue_capacity);

        let stage = Arc::new(DefaultIngestionStage::new().await?);

        // Recent-op cache for idempotency (per (path,mtime,len))
        let recent = Cache::builder()
            .max_capacity(50_000)
            .time_to_live(std::time::Duration::from_secs(600))
            .build();

        let output_hook = self.output_hook.unwrap_or_else(|| Arc::new(|_| Box::pin(async {})));
        let removal_hook = self.removal_hook.unwrap_or_else(|| Arc::new(|_| Box::pin(async {})));

        // Basic dedupe of enqueued paths during bursts
        let enqueued: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));

        // Spawn workers, each subscribing to the broadcast channel
        for _ in 0..self.concurrency {
            let stage = stage.clone();
            let output_hook = output_hook.clone();
            let removal_hook = removal_hook.clone();
            let recent = recent.clone();
            let enqueued = enqueued.clone();
            let mut rx = tx.subscribe();

            tokio::spawn(async move {
                while let Ok(cmd) = rx.recv().await {
                    match cmd {
                        IngestionCmd::FileUpsert { path } => {
                            // Mark dequeued
                            { enqueued.lock().await.remove(&path); }

                            match super::ingestion_util::data_input_from_path(path.clone()).await {
                                Ok(input) => {
                                    let ident = super::ingestion_util::op_identity(&path).await.ok();
                                    if let Some(id) = ident {
                                        if recent.get(&id).await.is_some() {
                                            info!("skip duplicate op {}", id);
                                            continue;
                                        }
                                        recent.insert(id.clone(), ()).await;
                                    }

                                    match stage.process(input).await {
                                        Ok(output) => (output_hook)(output).await,
                                        Err(e) => error!("ingest failed for {:?}: {}", path, e),
                                    }
                                }
                                Err(e) => error!("failed to build DataInput for {:?}: {}", path, e),
                            }
                        }
                        IngestionCmd::FileRemove { path } => {
                            { enqueued.lock().await.remove(&path); }
                            (removal_hook)(path).await;
                        }
                    }
                }
            });
        }

        Ok(IngestionRuntime { tx })
    }
}
