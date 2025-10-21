//! Multimodal Job Scheduler for Workers
//!
//! Handles scheduling and coordination of multimodal processing jobs
//! with concurrency caps, backpressure handling, and performance monitoring.

use anyhow::Result;
use chrono::{DateTime, Utc};
// TODO: Fix indexers imports when available
// use indexers::{
//     database::{IndexingJob, IndexingJobStatus},
//     types::IndexerConfig,
// };
use std::collections::HashMap;
use std::sync::Arc;
use std::default::Default;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::types::*;
use agent_agency_observability::{
    AgentPerformanceMetrics, AgentPerformanceTracker, AgentTelemetryCollector, AgentType,
};

/// Multimodal job scheduling configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultimodalSchedulerConfig {
    /// Maximum concurrent multimodal jobs
    pub max_concurrent_jobs: usize,
    /// Job timeout in milliseconds
    pub job_timeout_ms: u64,
    /// Backpressure threshold (queue depth)
    pub backpressure_threshold: usize,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Performance monitoring enabled
    pub performance_monitoring: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
}

/// Retry configuration for failed jobs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Recovery timeout in milliseconds
    pub recovery_timeout_ms: u64,
    /// Half-open max requests
    pub half_open_max_requests: u32,
}

/// Multimodal job with enhanced metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultimodalJob {
    pub id: Uuid,
    pub job_type: MultimodalJobType,
    pub priority: JobPriority,
    pub content: MultimodalContent,
    pub metadata: JobMetadata,
    pub status: MultimodalJobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub error_message: Option<String>,
    pub performance_metrics: Option<JobPerformanceMetrics>,
}

/// Types of multimodal jobs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MultimodalJobType {
    /// Video processing (ingestion, enrichment, indexing)
    VideoProcessing,
    /// Image processing (Vision analysis, captioning)
    ImageProcessing,
    /// Audio processing (ASR, transcription)
    AudioProcessing,
    /// Text processing (entity extraction, indexing)
    TextProcessing,
    /// Cross-modal validation
    CrossModalValidation,
    /// Embedding generation
    EmbeddingGeneration,
    /// Search indexing
    SearchIndexing,
}

/// Job priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum JobPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Multimodal content for processing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultimodalContent {
    pub modality: String,
    pub content_type: String,
    pub file_path: Option<String>,
    pub content_data: Option<Vec<u8>>,
    pub text_content: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Job metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobMetadata {
    pub project_scope: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub source: String,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Multimodal job status
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MultimodalJobStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

/// Job performance metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobPerformanceMetrics {
    pub processing_time_ms: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f32,
    pub throughput_items_per_sec: f32,
    pub error_rate: f32,
    pub quality_score: f32,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Multimodal job scheduler with concurrency control and backpressure
pub struct MultimodalJobScheduler {
    config: MultimodalSchedulerConfig,
    job_queue: Arc<RwLock<Vec<MultimodalJob>>>,
    active_jobs: Arc<RwLock<HashMap<Uuid, MultimodalJob>>>,
    completed_jobs: Arc<RwLock<HashMap<Uuid, MultimodalJob>>>,
    concurrency_semaphore: Arc<Semaphore>,
    circuit_breaker_state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    performance_tracker: Arc<AgentPerformanceTracker>,
    job_sender: mpsc::UnboundedSender<MultimodalJob>,
    job_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<MultimodalJob>>>>,
}

impl MultimodalJobScheduler {
    /// Create a new multimodal job scheduler
    pub fn new(config: MultimodalSchedulerConfig) -> Self {
        let (job_sender, job_receiver) = mpsc::unbounded_channel();
        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));

        let performance_tracker = Arc::new(AgentPerformanceTracker::new(
            AgentType::GeneralistWorker,
            "multimodal-scheduler".to_string(),
            Arc::new(AgentTelemetryCollector::new(Default::default())),
        ));

        Self {
            config,
            job_queue: Arc::new(RwLock::new(Vec::new())),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            completed_jobs: Arc::new(RwLock::new(HashMap::new())),
            concurrency_semaphore,
            circuit_breaker_state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            performance_tracker,
            job_sender,
            job_receiver: Arc::new(RwLock::new(Some(job_receiver))),
        }
    }

    /// Initialize the scheduler
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing multimodal job scheduler");

        // Start job processing loop
        self.start_job_processing_loop().await?;

        // Start performance monitoring if enabled
        if self.config.performance_monitoring {
            self.start_performance_monitoring().await?;
        }

        info!("Multimodal job scheduler initialized");
        Ok(())
    }

    /// Schedule a multimodal job
    pub async fn schedule_job(&self, job: MultimodalJob) -> Result<()> {
        // Check circuit breaker state
        if !self.can_accept_jobs().await {
            return Err(anyhow::anyhow!("Circuit breaker is open, cannot accept new jobs"));
        }

        // Check backpressure
        if self.is_under_backpressure().await {
            warn!("Scheduler under backpressure, queuing job {}", job.id);
        }

        // Add to queue
        let mut queue = self.job_queue.write().await;
        queue.push(job.clone());
        queue.sort_by(|a, b| b.priority.cmp(&a.priority)); // Higher priority first

        info!("Scheduled multimodal job: {} (type: {:?}, priority: {:?})", 
              job.id, job.job_type, job.priority);

        // Send to processing loop
        if let Err(e) = self.job_sender.send(job) {
            error!("Failed to send job to processing loop: {}", e);
        }

        Ok(())
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Option<MultimodalJobStatus> {
        // Check active jobs first
        if let Some(job) = self.active_jobs.read().await.get(&job_id) {
            return Some(job.status.clone());
        }

        // Check completed jobs
        if let Some(job) = self.completed_jobs.read().await.get(&job_id) {
            return Some(job.status.clone());
        }

        // Check queue
        let queue = self.job_queue.read().await;
        if let Some(job) = queue.iter().find(|j| j.id == job_id) {
            return Some(job.status.clone());
        }

        None
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        let queue = self.job_queue.read().await;
        let active = self.active_jobs.read().await;
        let completed = self.completed_jobs.read().await;

        let total_jobs = queue.len() + active.len() + completed.len();
        let pending_jobs = queue.len();
        let running_jobs = active.len();
        let completed_jobs_count = completed.len();

        let success_rate = if completed_jobs_count > 0 {
            let successful = completed.values()
                .filter(|job| matches!(job.status, MultimodalJobStatus::Completed))
                .count();
            successful as f32 / completed_jobs_count as f32
        } else {
            0.0
        };

        SchedulerStats {
            total_jobs,
            pending_jobs,
            running_jobs,
            completed_jobs: completed_jobs_count,
            success_rate,
            circuit_breaker_state: self.circuit_breaker_state.read().await.clone(),
            available_capacity: self.concurrency_semaphore.available_permits(),
            max_concurrent_jobs: self.config.max_concurrent_jobs,
        }
    }

    /// Cancel a job
    pub async fn cancel_job(&self, job_id: Uuid) -> Result<()> {
        // Check if job is in queue
        let mut queue = self.job_queue.write().await;
        if let Some(pos) = queue.iter().position(|j| j.id == job_id) {
            let mut job = queue.remove(pos);
            job.status = MultimodalJobStatus::Cancelled;
            job.updated_at = Utc::now();
            
            let mut completed = self.completed_jobs.write().await;
            completed.insert(job_id, job);
            
            info!("Cancelled queued job: {}", job_id);
            return Ok(());
        }

        // Check if job is active
        let mut active = self.active_jobs.write().await;
        if let Some(mut job) = active.remove(&job_id) {
            job.status = MultimodalJobStatus::Cancelled;
            job.updated_at = Utc::now();
            
            let mut completed = self.completed_jobs.write().await;
            completed.insert(job_id, job);
            
            info!("Cancelled active job: {}", job_id);
            return Ok(());
        }

        Err(anyhow::anyhow!("Job not found: {}", job_id))
    }

    /// Start job processing loop
    async fn start_job_processing_loop(&mut self) -> Result<()> {
        let receiver = self.job_receiver.write().await.take();
        if let Some(mut receiver) = receiver {
            let scheduler = Arc::new(self.clone());
            
            tokio::spawn(async move {
                while let Some(job) = receiver.recv().await {
                    let scheduler_clone = scheduler.clone();
                    tokio::spawn(async move {
                        if let Err(e) = scheduler_clone.process_job(job).await {
                            error!("Failed to process job: {}", e);
                        }
                    });
                }
            });
        }

        Ok(())
    }

    /// Process a single job
    async fn process_job(&self, mut job: MultimodalJob) -> Result<()> {
        // Acquire semaphore permit
        let _permit = self.concurrency_semaphore.acquire().await
            .map_err(|e| anyhow::anyhow!("Failed to acquire semaphore: {}", e))?;

        // Update job status
        job.status = MultimodalJobStatus::Running;
        job.started_at = Some(Utc::now());
        job.updated_at = Utc::now();

        // Move to active jobs
        {
            let mut active = self.active_jobs.write().await;
            active.insert(job.id, job.clone());
        }

        info!("Started processing multimodal job: {} (type: {:?})", job.id, job.job_type);

        // Process the job based on type
        let start_time = std::time::Instant::now();
        let result = match job.job_type {
            MultimodalJobType::VideoProcessing => self.process_video_job(&job).await,
            MultimodalJobType::ImageProcessing => self.process_image_job(&job).await,
            MultimodalJobType::AudioProcessing => self.process_audio_job(&job).await,
            MultimodalJobType::TextProcessing => self.process_text_job(&job).await,
            MultimodalJobType::CrossModalValidation => self.process_cross_modal_job(&job).await,
            MultimodalJobType::EmbeddingGeneration => self.process_embedding_job(&job).await,
            MultimodalJobType::SearchIndexing => self.process_indexing_job(&job).await,
        };

        let processing_time = start_time.elapsed();

        // Update job with result
        let mut active = self.active_jobs.write().await;
        if let Some(active_job) = active.remove(&job.id) {
            let mut completed_job = active_job;
            
            match result {
                Ok(metrics) => {
                    completed_job.status = MultimodalJobStatus::Completed;
                    completed_job.performance_metrics = Some(metrics);
                    info!("Completed multimodal job: {} in {}ms", 
                          completed_job.id, processing_time.as_millis());
                }
                Err(e) => {
                    completed_job.status = MultimodalJobStatus::Failed;
                    completed_job.error_message = Some(e.to_string());
                    error!("Failed multimodal job: {} - {}", completed_job.id, e);
                    
                    // Handle retry logic
                    if completed_job.retry_count < self.config.retry_config.max_retries {
                        completed_job.status = MultimodalJobStatus::Retrying;
                        completed_job.retry_count += 1;
                        
                        // Schedule retry with exponential backoff
                        let delay = self.calculate_retry_delay(completed_job.retry_count);
                        let scheduler = self.clone();
                        let job_clone = completed_job.clone();
                        
                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            if let Err(e) = scheduler.schedule_job(job_clone).await {
                                error!("Failed to reschedule job: {}", e);
                            }
                        });
                    }
                }
            }
            
            completed_job.completed_at = Some(Utc::now());
            completed_job.updated_at = Utc::now();
            
            let mut completed = self.completed_jobs.write().await;
            completed.insert(completed_job.id, completed_job);
        }

        // Update circuit breaker
        self.update_circuit_breaker(result.is_err()).await;

        Ok(())
    }

    /// Process video job (placeholder implementation)
    async fn process_video_job(&self, job: &MultimodalJob) -> Result<JobPerformanceMetrics> {
        /// TODO: Implement actual video processing pipeline
        /// - [ ] Integrate video codec support (H.264, H.265, VP9, AV1)
        /// - [ ] Implement video frame extraction and sampling strategies
        /// - [ ] Add video metadata extraction (duration, resolution, bitrate)
        /// - [ ] Support video preprocessing (stabilization, quality enhancement)
        /// - [ ] Implement video segmentation and scene detection
        /// - [ ] Add video content analysis and feature extraction
        /// - [ ] Support different video formats and container types
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        Ok(JobPerformanceMetrics {
            processing_time_ms: 1000,
            memory_usage_mb: 256.0,
            cpu_usage_percent: 75.0,
            throughput_items_per_sec: 1.0,
            error_rate: 0.0,
            quality_score: 0.95,
        })
    }

    /// Process image job
    async fn process_image_job(&self, job: &MultimodalJob) -> Result<JobPerformanceMetrics> {
        // Simulate image processing
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        Ok(JobPerformanceMetrics {
            processing_time_ms: 500,
            memory_usage_mb: 128.0,
            cpu_usage_percent: 60.0,
            throughput_items_per_sec: 2.0,
            error_rate: 0.0,
            quality_score: 0.92,
        })
    }

    /// Process audio job
    async fn process_audio_job(&self, job: &MultimodalJob) -> Result<JobPerformanceMetrics> {
        // Simulate audio processing
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
        
        Ok(JobPerformanceMetrics {
            processing_time_ms: 800,
            memory_usage_mb: 192.0,
            cpu_usage_percent: 70.0,
            throughput_items_per_sec: 1.25,
            error_rate: 0.0,
            quality_score: 0.88,
        })
    }

    /// Process text job
    async fn process_text_job(&self, job: &MultimodalJob) -> Result<JobPerformanceMetrics> {
        // Simulate text processing
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        Ok(JobPerformanceMetrics {
            processing_time_ms: 200,
            memory_usage_mb: 64.0,
            cpu_usage_percent: 40.0,
            throughput_items_per_sec: 5.0,
            error_rate: 0.0,
            quality_score: 0.96,
        })
    }

    /// Process multimodal fusion job (placeholder implementation)
    async fn process_multimodal_fusion_job(&self, job: &MultimodalJob) -> Result<JobPerformanceMetrics> {
        /// TODO: Implement cross-modal validation and consistency checking
        /// - [ ] Validate consistency between different modality representations
        /// - [ ] Implement cross-modal alignment and synchronization
        /// - [ ] Add multimodal fusion validation and quality assessment
        /// - [ ] Support temporal alignment across modalities (audio/video sync)
        /// - [ ] Implement cross-modal anomaly detection and correction
        /// - [ ] Add confidence scoring for cross-modal relationships
        /// - [ ] Support multimodal data integrity and corruption detection
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        Ok(JobPerformanceMetrics {
            processing_time_ms: 300,
            memory_usage_mb: 96.0,
            cpu_usage_percent: 50.0,
            throughput_items_per_sec: 3.33,
            error_rate: 0.0,
            quality_score: 0.94,
        })
    }

    /// Process embedding generation job
    async fn process_embedding_job(&self, job: &MultimodalJob) -> Result<JobPerformanceMetrics> {
        // Simulate embedding generation
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        
        Ok(JobPerformanceMetrics {
            processing_time_ms: 400,
            memory_usage_mb: 160.0,
            cpu_usage_percent: 65.0,
            throughput_items_per_sec: 2.5,
            error_rate: 0.0,
            quality_score: 0.91,
        })
    }

    /// Process search indexing job
    async fn process_indexing_job(&self, job: &MultimodalJob) -> Result<JobPerformanceMetrics> {
        // Simulate search indexing
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        
        Ok(JobPerformanceMetrics {
            processing_time_ms: 600,
            memory_usage_mb: 224.0,
            cpu_usage_percent: 80.0,
            throughput_items_per_sec: 1.67,
            error_rate: 0.0,
            quality_score: 0.93,
        })
    }

    /// Check if scheduler can accept new jobs
    async fn can_accept_jobs(&self) -> bool {
        let state = self.circuit_breaker_state.read().await;
        matches!(*state, CircuitBreakerState::Closed | CircuitBreakerState::HalfOpen)
    }

    /// Check if scheduler is under backpressure
    async fn is_under_backpressure(&self) -> bool {
        let queue = self.job_queue.read().await;
        queue.len() >= self.config.backpressure_threshold
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, retry_count: u32) -> u64 {
        let delay = self.config.retry_config.initial_delay_ms as f64
            * self.config.retry_config.backoff_multiplier.powi(retry_count as i32);
        
        delay.min(self.config.retry_config.max_delay_ms as f64) as u64
    }

    /// Update circuit breaker state
    async fn update_circuit_breaker(&self, failed: bool) {
        let mut state = self.circuit_breaker_state.write().await;
        let mut failure_count = self.failure_count.write().await;
        let mut last_failure_time = self.last_failure_time.write().await;

        if failed {
            *failure_count += 1;
            *last_failure_time = Some(Utc::now());

            if *failure_count >= self.config.circuit_breaker_config.failure_threshold {
                *state = CircuitBreakerState::Open;
                warn!("Circuit breaker opened due to {} failures", *failure_count);
            }
        } else {
            // Reset failure count on success
            *failure_count = 0;
        }

        // Check if circuit should transition from Open to HalfOpen
        if matches!(*state, CircuitBreakerState::Open) {
            if let Some(last_failure) = *last_failure_time {
                let recovery_timeout = chrono::Duration::milliseconds(
                    self.config.circuit_breaker_config.recovery_timeout_ms as i64
                );
                
                if Utc::now() - last_failure > recovery_timeout {
                    *state = CircuitBreakerState::HalfOpen;
                    info!("Circuit breaker transitioned to half-open state");
                }
            }
        }
    }

    /// Start performance monitoring
    async fn start_performance_monitoring(&self) -> Result<()> {
        let tracker = self.performance_tracker.clone();
        let scheduler = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let stats = scheduler.get_stats().await;
                let metrics = AgentPerformanceMetrics {
                    total_tasks: stats.total_jobs as u64,
                    completed_tasks: stats.completed_jobs as u64,
                    failed_tasks: stats.total_jobs - stats.completed_jobs,
                    average_execution_time_ms: 0.0, // Would be calculated from actual jobs
                    success_rate: stats.success_rate,
                    throughput_per_second: 0.0, // Would be calculated from actual jobs
                    memory_usage_mb: 0.0, // Would be tracked from actual jobs
                    cpu_usage_percent: 0.0, // Would be tracked from actual jobs
                };

                tracker.record_metrics(metrics).await;
            }
        });

        Ok(())
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedulerStats {
    pub total_jobs: usize,
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
    pub success_rate: f32,
    #[serde(skip)]
    pub circuit_breaker_state: CircuitBreakerState,
    pub available_capacity: usize,
    pub max_concurrent_jobs: usize,
}

impl Default for MultimodalSchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 10,
            job_timeout_ms: 30000,
            backpressure_threshold: 100,
            retry_config: RetryConfig {
                max_retries: 3,
                initial_delay_ms: 1000,
                backoff_multiplier: 2.0,
                max_delay_ms: 30000,
            },
            performance_monitoring: true,
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 5,
                recovery_timeout_ms: 60000,
                half_open_max_requests: 3,
            },
        }
    }
}

impl Clone for MultimodalJobScheduler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            job_queue: self.job_queue.clone(),
            active_jobs: self.active_jobs.clone(),
            completed_jobs: self.completed_jobs.clone(),
            concurrency_semaphore: self.concurrency_semaphore.clone(),
            circuit_breaker_state: self.circuit_breaker_state.clone(),
            failure_count: self.failure_count.clone(),
            last_failure_time: self.last_failure_time.clone(),
            performance_tracker: self.performance_tracker.clone(),
            job_sender: self.job_sender.clone(),
            job_receiver: self.job_receiver.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = MultimodalSchedulerConfig::default();
        let scheduler = MultimodalJobScheduler::new(config);
        
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_jobs, 0);
        assert_eq!(stats.max_concurrent_jobs, 10);
    }

    #[tokio::test]
    async fn test_job_scheduling() {
        let config = MultimodalSchedulerConfig::default();
        let mut scheduler = MultimodalJobScheduler::new(config);
        scheduler.initialize().await.unwrap();

        let job = MultimodalJob {
            id: Uuid::new_v4(),
            job_type: MultimodalJobType::TextProcessing,
            priority: JobPriority::Normal,
            content: MultimodalContent {
                modality: "text".to_string(),
                content_type: "plain".to_string(),
                file_path: None,
                content_data: None,
                text_content: Some("Test content".to_string()),
                metadata: HashMap::new(),
            },
            metadata: JobMetadata {
                project_scope: None,
                user_id: None,
                session_id: None,
                source: "test".to_string(),
                tags: vec![],
                custom_fields: HashMap::new(),
            },
            status: MultimodalJobStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            retry_count: 0,
            error_message: None,
            performance_metrics: None,
        };

        scheduler.schedule_job(job).await.unwrap();
        
        // Give some time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let stats = scheduler.get_stats().await;
        assert!(stats.total_jobs > 0);
    }
}
