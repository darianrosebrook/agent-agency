//! Internal types for council orchestrator
//!
//! Data structures and types used by the council coordinator
//! for consensus building, queue management, and evaluation.

use crate::advanced_monitoring::SLOTracker;
use crate::authority::{ExpertAuthorityManager, ExpertQualification};
use crate::models::{EvidencePacket, ParticipantContribution, RiskTier, TaskSpec};
use crate::resilience::ResilienceManager;
use crate::types::{ConsensusResult, FinalVerdict, JudgeVerdict};
use crate::CouncilConfig;
use crate::{MultimodalEvidenceEnricher, ClaimWithMultimodalEvidence};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

/// Placeholder types for missing agent_agency_research dependency
#[derive(Debug, Clone)]
pub struct KnowledgeSeeker {
    // Placeholder - actual implementation would come from agent_agency_research
}

impl KnowledgeSeeker {
    /// Placeholder implementation for get_decision_context
    pub async fn get_decision_context(&self, _decision_point: &str, _project_scope: Option<&str>) -> anyhow::Result<MultimodalContext> {
        // Return a placeholder multimodal context
        Ok(MultimodalContext {
            evidence_items: vec![], // No evidence items in placeholder
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Placeholder implementation for get_evidence_context
    pub async fn get_evidence_context(&self, _claim: &str, _context_scope: Option<&str>) -> anyhow::Result<MultimodalContext> {
        // Return a placeholder multimodal context
        Ok(MultimodalContext {
            evidence_items: vec![], // No evidence items in placeholder
            metadata: std::collections::HashMap::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceItem {
    pub modality: String,
    pub confidence: f32,
    pub similarity_score: f32,
    pub is_global: bool,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct MultimodalContext {
    // Placeholder - actual implementation would come from agent_agency_research
    pub evidence_items: Vec<EvidenceItem>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Internal metrics for tracking coordinator performance
#[derive(Debug, Clone, Default)]
pub struct CoordinatorMetrics {
    pub total_evaluations: u64,
    pub successful_evaluations: u64,
    pub failed_evaluations: u64,
    pub total_evaluation_time_ms: u64,
    pub total_enrichment_time_ms: u64,
    pub total_judge_inference_time_ms: u64,
    pub total_debate_time_ms: u64,
    pub sla_violations: u64,
    pub judge_performance: HashMap<String, super::metrics::JudgePerformanceStats>,
    /// Queue tracking metrics for evaluation management
    pub queue_metrics: QueueMetrics,
}

/// Queue tracking metrics for evaluation management
#[derive(Debug, Clone, Default)]
pub struct QueueMetrics {
    /// Current queue depth (number of pending evaluations)
    pub current_depth: u64,
    /// Maximum queue depth reached
    pub max_depth: u64,
    /// Total tasks processed through queue
    pub total_processed: u64,
    /// Average processing time per task (ms)
    pub avg_processing_time_ms: u64,
    /// Queue processing rate (tasks per second)
    pub processing_rate: f64,
    /// Queue bottlenecks detected
    pub bottlenecks_detected: u64,
    /// Queue optimization events
    pub optimization_events: u64,
    /// Queue management operations
    pub management_operations: u64,
    /// Last queue depth update timestamp
    pub last_update: DateTime<Utc>,
}

/// Queue task status for tracking individual evaluation tasks
#[derive(Debug, Clone)]
pub enum QueueTaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Queue task information for tracking individual evaluation tasks
#[derive(Debug, Clone)]
pub struct QueueTask {
    pub task_id: Uuid,
    pub status: QueueTaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub priority: u8, // 1-10, higher is more urgent
    pub estimated_duration_ms: u64,
    pub actual_duration_ms: Option<u64>,
}

/// Queue analytics for performance analysis
#[derive(Debug, Clone)]
pub struct QueueAnalytics {
    /// Queue processing efficiency (0.0-1.0)
    pub efficiency: f64,
    /// Queue backlog trend (positive = growing, negative = shrinking)
    pub backlog_trend: f64,
    /// Average wait time for tasks (ms)
    pub avg_wait_time_ms: u64,
    /// Queue utilization percentage
    pub utilization_percentage: f64,
    /// Bottleneck identification results
    pub bottlenecks: Vec<String>,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

/// Queue tracker for managing evaluation task queue
#[derive(Debug, Clone)]
pub struct QueueTracker {
    /// Active queue tasks
    pub active_tasks: HashMap<Uuid, QueueTask>,
    /// Queue processing history for analytics
    pub processing_history: Vec<QueueProcessingEvent>,
    /// Queue performance metrics
    pub performance_metrics: QueuePerformanceMetrics,
    /// Queue configuration and limits
    pub config: QueueConfig,
}

/// Queue processing event for tracking task lifecycle
#[derive(Debug, Clone)]
pub struct QueueProcessingEvent {
    pub task_id: Uuid,
    pub event_type: QueueEventType,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub metadata: HashMap<String, String>,
}

/// Types of queue processing events
#[derive(Debug, Clone)]
pub enum QueueEventType {
    TaskEnqueued,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    QueueOptimized,
    BottleneckDetected,
    LoadBalanced,
}

/// Queue performance metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct QueuePerformanceMetrics {
    /// Total tasks processed
    pub total_processed: u64,
    /// Total tasks failed
    pub total_failed: u64,
    /// Average processing time (ms)
    pub avg_processing_time_ms: u64,
    /// Peak queue depth
    pub peak_depth: u64,
    /// Current queue depth
    pub current_depth: u64,
    /// Queue throughput (tasks/second)
    pub throughput: f64,
    /// Queue latency (ms)
    pub avg_latency_ms: u64,
    /// Queue error rate
    pub error_rate: f64,
}

/// Queue configuration and limits
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum queue depth
    pub max_depth: usize,
    /// Maximum concurrent tasks
    pub max_concurrent: usize,
    /// Task timeout (seconds)
    pub task_timeout_seconds: u64,
    /// Queue optimization interval (seconds)
    pub optimization_interval_seconds: u64,
    /// Enable automatic load balancing
    pub enable_load_balancing: bool,
    /// Bottleneck detection threshold
    pub bottleneck_threshold: f64,
}

/// Executor progress tracking
#[derive(Debug, Clone)]
pub struct ExecutorProgress {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub in_progress_tasks: usize,
    pub estimated_completion_time: Option<DateTime<Utc>>,
}

/// Executor status enumeration
#[derive(Debug, Clone)]
pub enum ExecutorStatus {
    Idle,
    Processing,
    Completed,
    Failed,
}

/// Noop provenance emitter for testing
#[derive(Debug, Clone)]
pub struct NoopEmitter;

impl NoopEmitter {
    pub fn new() -> Self {
        Self
    }
}
