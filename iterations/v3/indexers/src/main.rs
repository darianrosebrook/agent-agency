//! @darianrosebrook
//! Indexers module CLI for testing BM25, HNSW, and job scheduling

use indexers::*;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Indexers CLI initialized");

    // Create job scheduler
    let scheduler = JobScheduler::new(50);
    info!("Job scheduler created with queue limit: 50");

    // Try to acquire jobs
    for job_type in [
        JobType::VideoIngest,
        JobType::AsrTranscription,
        JobType::VisualCaptioning,
    ] {
        let acquired = scheduler.try_acquire(job_type)?;
        info!("Job {:?} acquired: {}", job_type, acquired);
    }

    info!("Active jobs: {}", scheduler.active_count());
    info!("Queued jobs: {}", scheduler.queued_count());

    // Release jobs
    scheduler.release(JobType::VideoIngest, true);
    scheduler.release(JobType::AsrTranscription, true);
    scheduler.release(JobType::VisualCaptioning, false);

    info!("Jobs released");
    info!("Remaining active: {}", scheduler.active_count());

    Ok(())
}
