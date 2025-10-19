//! Multimodal ingestors for V3 RAG system
//! 
//! Ingests videos, slides, diagrams, and captions from local filesystem,
//! normalizes to canonical data model, and feeds to embedding/indexing pipeline.

pub mod video_ingestor;
pub mod slides_ingestor;
pub mod diagrams_ingestor;
pub mod captions_ingestor;
pub mod file_watcher;
pub mod types;

pub use video_ingestor::VideoIngestor;
pub use slides_ingestor::SlidesIngestor;
pub use diagrams_ingestor::DiagramsIngestor;
pub use captions_ingestor::CaptionsIngestor;
pub use file_watcher::FileWatcher;
pub use types::*;

use std::path::Path;

/// Common trait for all ingestors
pub trait Ingestor: Send + Sync {
    /// Ingest content from given path
    async fn ingest(
        &self,
        path: &Path,
        project_scope: Option<&str>,
    ) -> anyhow::Result<IngestResult>;

    /// Get supported file extensions
    fn supported_extensions(&self) -> &[&str];
}
