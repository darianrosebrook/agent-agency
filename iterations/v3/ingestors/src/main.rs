//! @darianrosebrook
//! Ingestors module CLI for testing multimodal RAG ingestion

use ingestors::*;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Ingestors CLI initialized");

    // Example: Create ingestors
    let _video_ingestor = VideoIngestor::new(None, None);
    let _slides_ingestor = SlidesIngestor::new();
    let _diagrams_ingestor = DiagramsIngestor::new();
    let _captions_ingestor = CaptionsIngestor::new();
    let _file_watcher = FileWatcher::new(None);

    info!("All ingestors ready");

    Ok(())
}
