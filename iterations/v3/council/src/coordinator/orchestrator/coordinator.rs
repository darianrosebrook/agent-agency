//! Main consensus coordinator implementation
//!
//! Central orchestrator for council operations, integrating
//! evaluation, queue management, metrics, and expert authority.

use super::types::{ConsensusCoordinator as CoordinatorStruct, KnowledgeSeeker, NoopEmitter, CoordinatorMetrics, QueueTracker};
use super::metrics::MetricsManager;
use super::queue::QueueManager;
use super::evaluation::EvaluationOrchestrator;
use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use crate::models::{EvidencePacket, ParticipantContribution, RiskTier, TaskSpec};
use crate::resilience::ResilienceManager;
use crate::council_types::{ConsensusResult, FinalVerdict, JudgeVerdict};
use crate::authority::{ExpertAuthorityManager, ExpertQualification, OverrideRequest};
use crate::{MultimodalEvidenceEnricher, ClaimWithMultimodalEvidence};
use crate::CouncilConfig;
use crate::advanced_monitoring::SLOTracker;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Provenance emitter trait for tracking operations
#[async_trait::async_trait]
pub trait ProvenanceEmitter: Send + Sync {
    async fn emit(&self, event: ProvenanceEvent) -> Result<()>;
}

/// Provenance event for tracking
#[derive(Debug, Clone)]
pub struct ProvenanceEvent {
    pub event_type: String,
    pub task_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
impl ProvenanceEmitter for NoopEmitter {
    async fn emit(&self, _event: ProvenanceEvent) -> Result<()> {
        Ok(())
    }
}

/// Main consensus coordinator for council operations
pub struct ConsensusCoordinator {
    inner: CoordinatorStruct,
    metrics_manager: MetricsManager,
    queue_manager: QueueManager,
    evaluation_orchestrator: EvaluationOrchestrator,
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator
    pub fn new(
        config: CouncilConfig,
        evidence_enrichment: EvidenceEnrichmentCoordinator,
        resilience_manager: Arc<ResilienceManager>,
        multimodal_evidence_enricher: MultimodalEvidenceEnricher,
        slo_tracker: Arc<SLOTracker>,
    ) -> Self {
        let emitter: Arc<dyn ProvenanceEmitter> = Arc::new(NoopEmitter::new());
        let knowledge_seeker = Some(Arc::new(KnowledgeSeeker));

        // Initialize managers
        let metrics_manager = MetricsManager::new();
        let queue_manager = QueueManager::new(metrics_manager.queue_tracker());

        let evaluation_orchestrator = EvaluationOrchestrator::new(
            evidence_enrichment.clone(),
            resilience_manager.clone(),
            multimodal_evidence_enricher.clone(),
            knowledge_seeker.clone(),
            metrics_manager.clone(),
        );

        let expert_authority_manager = Arc::new(RwLock::new(ExpertAuthorityManager::new()));

        let inner = CoordinatorStruct {
            config,
            emitter,
            evidence_enrichment,
            resilience_manager,
            metrics: metrics_manager.metrics(),
            multimodal_evidence_enricher,
            knowledge_seeker,
            queue_tracker: metrics_manager.queue_tracker(),
            expert_authority_manager,
            db_client: None, // Would be set if database is available
            slo_tracker,
        };

        Self {
            inner,
            metrics_manager,
            queue_manager,
            evaluation_orchestrator,
        }
    }

    /// Evaluate a task through the complete council process
    pub async fn evaluate_task(&self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        self.evaluation_orchestrator.evaluate_task(task_spec).await
    }

    /// Enqueue a task for processing
    pub async fn enqueue_task(&self, task_spec: TaskSpec) -> Result<Uuid> {
        use super::types::QueueTask;
        use super::types::QueueTaskStatus;

        let queue_task = QueueTask {
            task_id: task_spec.id,
            status: QueueTaskStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            priority: 5, // Default priority
            estimated_duration_ms: 30000, // 30 seconds
            actual_duration_ms: None,
        };

        self.queue_manager.enqueue_task(queue_task).await?;
        Ok(task_spec.id)
    }

    /// Process the next queued task
    pub async fn process_next_task(&self) -> Result<Option<ConsensusResult>> {
        if let Some(task_id) = self.queue_manager.dequeue_task().await {
            // Would need to retrieve the actual TaskSpec from storage
            // For now, return None as placeholder
            warn!("Task processing not fully implemented - missing TaskSpec retrieval for {}", task_id);
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// Get current queue statistics
    pub async fn get_queue_stats(&self) -> super::queue::QueueStats {
        self.queue_manager.get_queue_stats().await
    }

    /// Get coordinator metrics
    pub async fn get_metrics(&self) -> CoordinatorMetrics {
        self.metrics_manager.get_metrics_snapshot().await
    }

    /// Register an expert authority
    pub async fn register_expert(&self, qualification: ExpertQualification) -> Result<()> {
        let mut manager = self.inner.expert_authority_manager.write().await;
        manager.register_expert(qualification).await
    }

    /// Submit an override request
    pub async fn submit_override_request(&self, request: OverrideRequest) -> Result<Uuid> {
        let mut manager = self.inner.expert_authority_manager.write().await;
        manager.submit_override_request(request).await
    }

    /// Approve an override request
    pub async fn approve_override_request(&self, request_id: Uuid, approver_id: &str) -> Result<()> {
        let mut manager = self.inner.expert_authority_manager.write().await;
        manager.approve_override_request(request_id, approver_id).await
    }

    /// Check if participant has override authority
    pub async fn has_override_authority(&self, participant_id: &str, required_level: &crate::authority::ExpertAuthorityLevel) -> bool {
        let manager = self.inner.expert_authority_manager.read().await;
        manager.has_authority(participant_id, required_level).await
    }

    /// Get active override requests
    pub async fn get_active_overrides(&self) -> Vec<OverrideRequest> {
        let manager = self.inner.expert_authority_manager.read().await;
        manager.get_active_overrides().await
    }

    /// Get override audit trail
    pub async fn get_override_audit_trail(&self, override_id: Option<Uuid>) -> Vec<crate::authority::OverrideAuditEntry> {
        let manager = self.inner.expert_authority_manager.read().await;
        manager.get_audit_trail(override_id).await
    }

    /// Cleanup expired overrides
    pub async fn cleanup_expired_overrides(&self) -> Vec<Uuid> {
        let mut manager = self.inner.expert_authority_manager.write().await;
        manager.cleanup_expired_overrides().await
    }

    /// Get health status of the coordinator
    pub async fn health_status(&self) -> CoordinatorHealth {
        let queue_stats = self.get_queue_stats().await;
        let metrics = self.get_metrics().await;

        let overall_health = if metrics.total_evaluations > 0 {
            let success_rate = metrics.successful_evaluations as f64 / metrics.total_evaluations as f64;
            if success_rate > 0.95 {
                HealthStatus::Healthy
            } else if success_rate > 0.8 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Unhealthy
            }
        } else {
            HealthStatus::Unknown
        };

        CoordinatorHealth {
            overall_status: overall_health,
            queue_depth: queue_stats.pending_tasks,
            active_tasks: queue_stats.active_tasks,
            total_evaluations: metrics.total_evaluations,
            success_rate: if metrics.total_evaluations > 0 {
                metrics.successful_evaluations as f64 / metrics.total_evaluations as f64
            } else {
                0.0
            },
        }
    }
}

/// Coordinator health status
#[derive(Debug, Clone)]
pub struct CoordinatorHealth {
    pub overall_status: HealthStatus,
    pub queue_depth: usize,
    pub active_tasks: usize,
    pub total_evaluations: u64,
    pub success_rate: f64,
}

/// Health status levels
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}


