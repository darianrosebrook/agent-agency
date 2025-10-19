//! @darianrosebrook
//! Job scheduler with concurrency caps for ingestion pipeline

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, info, warn};
use parking_lot::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobType {
    VideoIngest,
    SlidesIngest,
    DiagramIngest,
    CaptionsIngest,
    VisionOcr,
    AsrTranscription,
    EntityExtraction,
    VisualCaptioning,
    Embedding,
}

impl JobType {
    /// Get concurrency cap for this job type
    pub fn concurrency_cap(&self) -> usize {
        match self {
            JobType::VideoIngest => 2,
            JobType::SlidesIngest => 3,
            JobType::DiagramIngest => 3,
            JobType::CaptionsIngest => 5,
            JobType::VisionOcr => 2,
            JobType::AsrTranscription => 1,      // ASR is expensive
            JobType::EntityExtraction => 4,
            JobType::VisualCaptioning => 1,      // Expensive model inference
            JobType::Embedding => 2,
        }
    }

    /// Get timeout in milliseconds
    pub fn timeout_ms(&self) -> u64 {
        match self {
            JobType::VideoIngest => 300_000,      // 5 minutes
            JobType::SlidesIngest => 60_000,       // 1 minute
            JobType::DiagramIngest => 30_000,      // 30 seconds
            JobType::CaptionsIngest => 15_000,     // 15 seconds
            JobType::VisionOcr => 10_000,          // 10 seconds
            JobType::AsrTranscription => 120_000,  // 2 minutes
            JobType::EntityExtraction => 30_000,   // 30 seconds
            JobType::VisualCaptioning => 30_000,   // 30 seconds
            JobType::Embedding => 60_000,          // 1 minute
        }
    }
}

#[derive(Debug, Clone)]
pub struct IngestionJob {
    pub id: Uuid,
    pub job_type: JobType,
    pub content_hash: String,
    pub priority: u8,
}

struct JobStats {
    active_count: usize,
    queued_count: usize,
    total_completed: u64,
    total_failed: u64,
}

/// Job scheduler with concurrency governance
pub struct JobScheduler {
    stats: Arc<Mutex<HashMap<JobType, JobStats>>>,
    queue_size_limit: usize,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub fn new(queue_size_limit: usize) -> Self {
        let mut stats = HashMap::new();
        for job_type in [
            JobType::VideoIngest,
            JobType::SlidesIngest,
            JobType::DiagramIngest,
            JobType::CaptionsIngest,
            JobType::VisionOcr,
            JobType::AsrTranscription,
            JobType::EntityExtraction,
            JobType::VisualCaptioning,
            JobType::Embedding,
        ] {
            stats.insert(
                job_type,
                JobStats {
                    active_count: 0,
                    queued_count: 0,
                    total_completed: 0,
                    total_failed: 0,
                },
            );
        }

        info!("Job scheduler initialized with queue limit: {}", queue_size_limit);

        Self {
            stats: Arc::new(Mutex::new(stats)),
            queue_size_limit,
        }
    }

    /// Try to acquire a slot for a job
    pub fn try_acquire(&self, job_type: JobType) -> Result<bool> {
        let mut stats = self.stats.lock();

        if let Some(stat) = stats.get_mut(&job_type) {
            let cap = job_type.concurrency_cap();

            if stat.active_count < cap {
                stat.active_count += 1;
                debug!(
                    "Job slot acquired for {:?} ({}/{})",
                    job_type, stat.active_count, cap
                );
                Ok(true)
            } else {
                if stat.queued_count >= self.queue_size_limit {
                    warn!(
                        "Queue full for {:?}: {} queued (limit: {})",
                        job_type, stat.queued_count, self.queue_size_limit
                    );
                    return Ok(false);
                }
                stat.queued_count += 1;
                debug!(
                    "Job queued for {:?} ({} queued)",
                    job_type, stat.queued_count
                );
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Release a job slot (call when job completes or fails)
    pub fn release(&self, job_type: JobType, success: bool) {
        let mut stats = self.stats.lock();

        if let Some(stat) = stats.get_mut(&job_type) {
            stat.active_count = stat.active_count.saturating_sub(1);
            if stat.queued_count > 0 {
                stat.queued_count -= 1;
            }

            if success {
                stat.total_completed += 1;
            } else {
                stat.total_failed += 1;
            }

            debug!(
                "Job released for {:?}: active={}, queued={}",
                job_type, stat.active_count, stat.queued_count
            );
        }
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> HashMap<JobType, (usize, usize, u64, u64)> {
        let stats = self.stats.lock();
        stats
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    (v.active_count, v.queued_count, v.total_completed, v.total_failed),
                )
            })
            .collect()
    }

    /// Get total active jobs across all types
    pub fn active_count(&self) -> usize {
        self.stats.lock().values().map(|s| s.active_count).sum()
    }

    /// Get total queued jobs across all types
    pub fn queued_count(&self) -> usize {
        self.stats.lock().values().map(|s| s.queued_count).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_type_concurrency_caps() {
        assert_eq!(JobType::VideoIngest.concurrency_cap(), 2);
        assert_eq!(JobType::AsrTranscription.concurrency_cap(), 1);
        assert_eq!(JobType::VisualCaptioning.concurrency_cap(), 1);
    }

    #[test]
    fn test_job_scheduler_acquire_and_release() {
        let scheduler = JobScheduler::new(10);

        // Should acquire first slot
        assert!(scheduler.try_acquire(JobType::VideoIngest).unwrap());
        assert_eq!(scheduler.active_count(), 1);

        // Should acquire second slot (cap is 2)
        assert!(scheduler.try_acquire(JobType::VideoIngest).unwrap());
        assert_eq!(scheduler.active_count(), 2);

        // Should queue third (cap exceeded)
        assert!(!scheduler.try_acquire(JobType::VideoIngest).unwrap());
        assert_eq!(scheduler.queued_count(), 1);

        // Release one
        scheduler.release(JobType::VideoIngest, true);
        assert_eq!(scheduler.active_count(), 1);
        assert_eq!(scheduler.queued_count(), 0);
    }

    #[test]
    fn test_job_scheduler_different_types() {
        let scheduler = JobScheduler::new(10);

        // Different job types have independent caps
        assert!(scheduler.try_acquire(JobType::AsrTranscription).unwrap());
        assert!(!scheduler.try_acquire(JobType::AsrTranscription).unwrap()); // ASR cap is 1

        assert!(scheduler.try_acquire(JobType::VideoIngest).unwrap());
        assert_eq!(scheduler.active_count(), 2); // 1 ASR + 1 Video
    }

    #[test]
    fn test_job_scheduler_queue_limit() {
        let scheduler = JobScheduler::new(2); // Small queue

        // Fill cap
        assert!(scheduler.try_acquire(JobType::VideoIngest).unwrap());
        assert!(scheduler.try_acquire(JobType::VideoIngest).unwrap());

        // Queue up to limit
        assert!(!scheduler.try_acquire(JobType::VideoIngest).unwrap());
        assert!(!scheduler.try_acquire(JobType::VideoIngest).unwrap());

        // Exceed limit
        assert!(!scheduler.try_acquire(JobType::VideoIngest).unwrap());
    }
}

