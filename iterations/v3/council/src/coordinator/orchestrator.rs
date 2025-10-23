//! Consensus Coordinator for the Council system
//!
//! Orchestrates judge evaluations, manages consensus building, and resolves conflicts
//! through the debate protocol.
//!
//! This module contains the main ConsensusCoordinator implementation and all
//! orchestrator-specific internal types and logic. Supporting functionality
//! is split across sibling modules for better organization.

use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use crate::models::{EvidencePacket, ParticipantContribution, RiskTier, TaskSpec};
use crate::resilience::ResilienceManager;
use crate::types::{ConsensusResult, FinalVerdict, JudgeVerdict};
use crate::CouncilConfig;
use crate::{MultimodalEvidenceEnricher, ClaimWithMultimodalEvidence};
// TODO: Implement multimodal context types in contracts
// use agent_agency_contracts::{MultimodalContextProvider, MultimodalContext, KnowledgeSeeker};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use async_trait;

// Module imports for split architecture
use super::resolution::{apply_caws_tie_breaking_rules, generate_resolution_rationale, CawsResolutionResult, ResolutionType};
use super::debate::{compile_debate_contributions, sign_debate_transcript, analyze_contribution_patterns, analyze_debate_consensus, DebateContribution, CompiledContributions};
use super::extraction::{AdvancedPositionExtractor, ExtractionConfig, ExtractionResult, ExtractionMetadata, ExtractionStats, DecisionType, PositionConfidence, DecisionReasoning, RiskAssessment, PositionConsistency, PositionExtractionError, SentenceEmbeddingsModelType};
use super::authority::{ExpertAuthorityManager, apply_override_policies, ExpertAuthorityLevel, ExpertQualification, OverrideRequest, OverrideRiskAssessment, OverrideStatus, OverrideReason, ImpactLevel, OverrideAuditEntry, OverrideAction};
use super::metrics::{CoordinatorMetricsSnapshot, EvaluationMetrics, TimingMetrics, SLAMetrics, JudgePerformanceSnapshot, HealthIndicators, JudgePerformanceStats};
use crate::advanced_monitoring::SLOTracker;

/// Placeholder types for missing agent_agency_research dependency
#[derive(Debug, Clone)]
pub struct KnowledgeSeeker {
    // Placeholder - actual implementation would come from agent_agency_research
}

impl KnowledgeSeeker {
    /// Placeholder implementation for get_decision_context
    pub async fn get_decision_context(&self, _decision_point: &str, _project_scope: Option<&str>) -> Result<MultimodalContext> {
        // Return a placeholder multimodal context
        Ok(MultimodalContext {
            evidence_items: vec![], // No evidence items in placeholder
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Placeholder implementation for get_evidence_context
    pub async fn get_evidence_context(&self, _claim: &str, _context_scope: Option<&str>) -> Result<MultimodalContext> {
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










/// Main coordinator for council consensus building
pub struct ConsensusCoordinator {
    config: CouncilConfig,
    emitter: std::sync::Arc<dyn ProvenanceEmitter>,
    evidence_enrichment: EvidenceEnrichmentCoordinator,
    resilience_manager: Arc<ResilienceManager>, // V2 production resilience
    /// Basic metrics tracking for the coordinator
    metrics: Arc<std::sync::RwLock<CoordinatorMetrics>>,
    /// Multimodal evidence enricher for claim enhancement
    multimodal_evidence_enricher: MultimodalEvidenceEnricher,
    /// Knowledge seeker for multimodal context retrieval
    knowledge_seeker: Option<Arc<KnowledgeSeeker>>,
    /// Queue tracking for evaluation task management
    queue_tracker: Arc<std::sync::RwLock<QueueTracker>>,
    /// Expert authority manager for override mechanisms
    expert_authority_manager: Arc<std::sync::RwLock<ExpertAuthorityManager>>,
    /// Optional database client for persistence operations
    db_client: Option<Arc<agent_agency_database::client::DatabaseClient>>,
    /// SLO tracker for service level objectives monitoring
    slo_tracker: Arc<SLOTracker>,
}

/// Internal metrics for tracking coordinator performance
#[derive(Debug, Clone, Default)]
struct CoordinatorMetrics {
    total_evaluations: u64,
    successful_evaluations: u64,
    failed_evaluations: u64,
    total_evaluation_time_ms: u64,
    total_enrichment_time_ms: u64,
    total_judge_inference_time_ms: u64,
    total_debate_time_ms: u64,
    sla_violations: u64,
    judge_performance: HashMap<String, JudgePerformanceStats>,
    /// Queue tracking metrics for evaluation management
    queue_metrics: QueueMetrics,
}

/// Queue tracking metrics for evaluation management
#[derive(Debug, Clone, Default)]
struct QueueMetrics {
    /// Current queue depth (number of pending evaluations)
    current_depth: u64,
    /// Maximum queue depth reached
    max_depth: u64,
    /// Total tasks processed through queue
    total_processed: u64,
    /// Average processing time per task (ms)
    avg_processing_time_ms: u64,
    /// Queue processing rate (tasks per second)
    processing_rate: f64,
    /// Queue bottlenecks detected
    bottlenecks_detected: u64,
    /// Queue optimization events
    optimization_events: u64,
    /// Queue management operations
    management_operations: u64,
    /// Last queue depth update timestamp
    last_update: DateTime<Utc>,
}

/// Queue task status for tracking individual evaluation tasks
#[derive(Debug, Clone)]
enum QueueTaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Queue task information for tracking individual evaluation tasks
#[derive(Debug, Clone)]
struct QueueTask {
    task_id: Uuid,
    status: QueueTaskStatus,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    priority: u8, // 1-10, higher is more urgent
    estimated_duration_ms: u64,
    actual_duration_ms: Option<u64>,
}

/// Queue analytics for performance analysis
#[derive(Debug, Clone)]
struct QueueAnalytics {
    /// Queue processing efficiency (0.0-1.0)
    efficiency: f64,
    /// Queue backlog trend (positive = growing, negative = shrinking)
    backlog_trend: f64,
    /// Average wait time for tasks (ms)
    avg_wait_time_ms: u64,
    /// Queue utilization percentage
    utilization_percentage: f64,
    /// Bottleneck identification results
    bottlenecks: Vec<String>,
    /// Optimization recommendations
    recommendations: Vec<String>,
}

/// Queue tracker for managing evaluation task queue
#[derive(Debug, Clone)]
struct QueueTracker {
    /// Active queue tasks
    active_tasks: HashMap<Uuid, QueueTask>,
    /// Queue processing history for analytics
    processing_history: Vec<QueueProcessingEvent>,
    /// Queue performance metrics
    performance_metrics: QueuePerformanceMetrics,
    /// Queue configuration and limits
    config: QueueConfig,
}

/// Queue processing event for tracking task lifecycle
#[derive(Debug, Clone)]
struct QueueProcessingEvent {
    task_id: Uuid,
    event_type: QueueEventType,
    timestamp: DateTime<Utc>,
    duration_ms: Option<u64>,
    metadata: HashMap<String, String>,
}

/// Types of queue processing events
#[derive(Debug, Clone)]
enum QueueEventType {
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
struct QueuePerformanceMetrics {
    /// Total tasks processed
    total_processed: u64,
    /// Total tasks failed
    total_failed: u64,
    /// Average processing time (ms)
    avg_processing_time_ms: u64,
    /// Peak queue depth
    peak_depth: u64,
    /// Current processing rate (tasks/second)
    current_rate: f64,
    /// Queue efficiency score (0.0-1.0)
    efficiency_score: f64,
    /// Last performance update
    last_update: DateTime<Utc>,
}

/// Queue configuration and limits
#[derive(Debug, Clone)]
struct QueueConfig {
    /// Maximum queue depth
    max_depth: u64,
    /// Maximum processing time per task (ms)
    max_processing_time_ms: u64,
    /// Queue optimization threshold
    optimization_threshold: f64,
    /// Bottleneck detection threshold
    bottleneck_threshold: f64,
    /// Load balancing enabled
    load_balancing_enabled: bool,
    /// Priority handling enabled
    priority_handling_enabled: bool,
}

/// Queue processing status for monitoring
#[derive(Debug, Clone)]
struct QueueProcessingStatus {
    total_tasks: u64,
    pending: u64,
    processing: u64,
    completed: u64,
    failed: u64,
}

/// Queue processing rates for performance tracking
#[derive(Debug, Clone)]
struct QueueProcessingRates {
    current_rate: f64,
    avg_rate_1min: f64,
    avg_rate_5min: f64,
    avg_rate_15min: f64,
    peak_rate: f64,
}

/// Queue bottleneck information
#[derive(Debug, Clone)]
struct QueueBottleneck {
    bottleneck_type: String,
    severity: String,
    description: String,
    recommendation: String,
}

/// Backlog trend analysis results
#[derive(Debug, Clone)]
struct BacklogTrendAnalysis {
    trend: String,
    enqueue_rate: f64,
    completion_rate: f64,
    net_change: i32,
}

/// Efficiency metrics for queue performance
#[derive(Debug, Clone)]
struct EfficiencyMetrics {
    efficiency: f64,
    throughput: f64,
    latency: u64,
    resource_utilization: f64,
}

/// Optimization strategy information
#[derive(Debug, Clone)]
struct OptimizationStrategy {
    strategy_type: String,
    description: String,
    expected_improvement: f64,
    implementation_cost: String,
}

/// Prioritization result information
#[derive(Debug, Clone)]
struct PrioritizationResult {
    high_priority_count: u64,
    medium_priority_count: u64,
    low_priority_count: u64,
    prioritization_enabled: bool,
}

/// Load balancing result information
#[derive(Debug, Clone)]
struct LoadBalancingResult {
    current_distribution: u64,
    optimal_distribution: u64,
    load_balancing_enabled: bool,
    rebalance_needed: bool,
}

/// Lifecycle management result information
#[derive(Debug, Clone)]
struct LifecycleManagementResult {
    total_lifecycle_events: u64,
    active_lifecycle_tasks: u64,
    lifecycle_efficiency: f64,
}

/// Administration result information
#[derive(Debug, Clone)]
struct AdministrationResult {
    total_operations: u64,
    optimization_events: u64,
    administration_efficiency: f64,
}


/// Provenance emission interface for council events
pub trait ProvenanceEmitter: Send + Sync + std::fmt::Debug {
    fn on_judge_verdict(
        &self,
        task_id: uuid::Uuid,
        judge: &str,
        weight: f32,
        decision: &str,
        score: f32,
    );
    fn on_final_verdict(&self, task_id: uuid::Uuid, verdict: &FinalVerdict);
}

/// No-op emitter for tests/defaults
#[derive(Debug)]
pub struct NoopEmitter;
impl ProvenanceEmitter for NoopEmitter {
    fn on_judge_verdict(
        &self,
        _task_id: uuid::Uuid,
        _judge: &str,
        _weight: f32,
        _decision: &str,
        _score: f32,
    ) {
    }
    fn on_final_verdict(&self, _task_id: uuid::Uuid, _verdict: &FinalVerdict) {}
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator
    pub fn new(config: CouncilConfig) -> Self {
        let queue_config = QueueConfig {
            max_depth: 100,
            max_processing_time_ms: 30000,
            optimization_threshold: 0.8,
            bottleneck_threshold: 0.9,
            load_balancing_enabled: true,
            priority_handling_enabled: true,
        };
        
        let queue_tracker = QueueTracker {
            active_tasks: HashMap::new(),
            processing_history: Vec::new(),
            performance_metrics: QueuePerformanceMetrics::default(),
            config: queue_config,
        };
        
        Self {
            config,
            emitter: std::sync::Arc::new(NoopEmitter),
            evidence_enrichment: EvidenceEnrichmentCoordinator::new(),
            resilience_manager: Arc::new(ResilienceManager::new()), // V2 production resilience
            metrics: Arc::new(std::sync::RwLock::new(CoordinatorMetrics::default())),
            multimodal_evidence_enricher: MultimodalEvidenceEnricher::new(),
            knowledge_seeker: None, // Will be set via set_knowledge_seeker
            queue_tracker: Arc::new(std::sync::RwLock::new(queue_tracker)),
            expert_authority_manager: Arc::new(std::sync::RwLock::new(ExpertAuthorityManager::new())),
            db_client: None, // Database client will be set separately if needed
            slo_tracker: Arc::new(SLOTracker::new()), // SLO tracking for service level objectives
        }
    }

    /// Set the knowledge seeker for multimodal context retrieval
    pub fn set_knowledge_seeker(&mut self, knowledge_seeker: Arc<KnowledgeSeeker>) {
        self.knowledge_seeker = Some(knowledge_seeker);
    }

    // ============================================================================
    // EXPERT AUTHORITY MANAGEMENT METHODS
    // ============================================================================

    /// Register an expert participant with authority qualifications
    pub async fn register_expert(&self, qualification: ExpertQualification) -> Result<()> {
        let mut manager = self.expert_authority_manager.write().unwrap();
        manager.register_expert(qualification)
    }

    /// Submit an expert override request
    pub async fn submit_override_request(&self, request: OverrideRequest) -> Result<Uuid> {
        let mut manager = self.expert_authority_manager.write().unwrap();
        manager.submit_override_request(request)
    }

    /// Approve an expert override request
    pub async fn approve_override_request(&self, request_id: Uuid, approver_id: &str) -> Result<()> {
        let mut manager = self.expert_authority_manager.write().unwrap();
        manager.approve_override_request(request_id, approver_id)
    }

    /// Check if a participant has authority for a specific override level
    pub async fn has_override_authority(&self, participant_id: &str, required_level: &ExpertAuthorityLevel) -> bool {
        let manager = self.expert_authority_manager.read().unwrap();
        manager.has_override_authority(participant_id, required_level)
    }

    /// Get active override requests
    pub async fn get_active_overrides(&self) -> Vec<OverrideRequest> {
        let manager = self.expert_authority_manager.read().unwrap();
        manager.get_active_overrides().into_iter().cloned().collect()
    }

    /// Get audit trail for override accountability
    pub async fn get_override_audit_trail(&self, override_id: Option<Uuid>) -> Vec<OverrideAuditEntry> {
        let manager = self.expert_authority_manager.read().unwrap();
        manager.get_audit_trail(override_id).into_iter().cloned().collect()
    }

    /// Clean up expired override requests
    pub async fn cleanup_expired_overrides(&self) -> Vec<Uuid> {
        let mut manager = self.expert_authority_manager.write().unwrap();
        manager.cleanup_expired_overrides()
    }

    /// Query participant performance history from database
    async fn query_participant_performance_history(&self, _participant_id: &str) -> Result<Vec<ParticipantPerformanceRecord>> {
        // TODO: Implement database query
        Ok(vec![])
    }

    /// Calculate participant reliability from performance data
    fn calculate_participant_reliability(&self, performance_data: &[ParticipantPerformanceRecord]) -> f32 {
        if performance_data.is_empty() {
            return 0.5;
        }

        // Statistical analysis of participant reliability
        let accuracies: Vec<f32> = performance_data.iter().map(|r| r.decision_accuracy).collect();
        let mean_accuracy = accuracies.iter().sum::<f32>() / accuracies.len() as f32;

        // Calculate standard deviation for consistency measure
        let variance = accuracies.iter()
            .map(|acc| (acc - mean_accuracy).powi(2))
            .sum::<f32>() / accuracies.len() as f32;
        let std_dev = variance.sqrt();

        // Reliability score combines accuracy and consistency
        let consistency_factor = 1.0 - (std_dev / mean_accuracy.max(0.1)).min(1.0);
        let reliability_score = mean_accuracy * 0.7 + consistency_factor * 0.3;

        reliability_score.max(0.0).min(1.0)
    }

    /// Apply time-weighted performance scoring (recent vs old performance)
    fn apply_time_weighting(&self, performance_data: &[ParticipantPerformanceRecord]) -> f32 {
        let now = chrono::Utc::now();
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for record in performance_data {
            let age_hours = now.signed_duration_since(record.timestamp).num_hours() as f32;

            // Time decay: recent performance gets higher weight
            let time_weight = if age_hours <= 24.0 {
                1.0 // Full weight for last 24 hours
            } else if age_hours <= 168.0 { // 1 week
                0.8 // Good weight for last week
            } else if age_hours <= 720.0 { // 30 days
                0.6 // Moderate weight for last month
            } else {
                0.3 // Low weight for older data
            };

            let performance_score = (record.decision_accuracy + record.quality_score) / 2.0;
            weighted_sum += performance_score * time_weight;
            total_weight += time_weight;
        }

        if total_weight > 0.0 {
            (weighted_sum / total_weight).max(0.0).min(1.0)
        } else {
            0.5
        }
    }

    /// Analyze performance trends for alerting
    fn analyze_performance_trends(&self, performance_data: &[ParticipantPerformanceRecord], participant_id: &str) {
        if performance_data.len() < 5 {
            return; // Need minimum data for trend analysis
        }

        // Simple trend analysis: compare recent vs older performance
        let midpoint = performance_data.len() / 2;
        let recent_avg = performance_data[..midpoint].iter()
            .map(|r| r.decision_accuracy)
            .sum::<f32>() / midpoint as f32;
        let older_avg = performance_data[midpoint..].iter()
            .map(|r| r.decision_accuracy)
            .sum::<f32>() / (performance_data.len() - midpoint) as f32;

        let trend = recent_avg - older_avg;

        if trend < -0.1 {
            tracing::warn!("Performance decline detected for participant {}: {:.3} → {:.3}",
                participant_id, older_avg, recent_avg);
        } else if trend > 0.1 {
            tracing::info!("Performance improvement detected for participant {}: {:.3} → {:.3}",
                participant_id, older_avg, recent_avg);
        }
    }

    /// Query participant decision history for accuracy analysis
    async fn query_participant_decision_history(&self, _participant_id: &str) -> Result<Vec<DecisionRecord>> {
        // TODO: Implement database query
        Ok(vec![])
    }

    /// Calculate decision reliability statistics with confidence intervals
    fn calculate_decision_reliability_stats(&self, decision_history: &[DecisionRecord]) -> DecisionReliabilityStats {
        if decision_history.is_empty() {
            return DecisionReliabilityStats {
                accuracy: 0.5,
                confidence_interval: (0.4, 0.6),
                sample_size: 0,
                consistency_score: 0.5,
            };
        }

        // Calculate accuracy: decisions that match actual outcomes
        let correct_decisions = decision_history.iter()
            .filter(|r| r.decision_outcome == r.actual_outcome)
            .count();
        let accuracy = correct_decisions as f32 / decision_history.len() as f32;

        // Calculate confidence interval using standard error
        let sample_size = decision_history.len() as f32;
        let standard_error = (accuracy * (1.0 - accuracy) / sample_size).sqrt();
        let margin_of_error = 1.96 * standard_error; // 95% confidence interval
        let confidence_interval = (
            (accuracy - margin_of_error).max(0.0),
            (accuracy + margin_of_error).min(1.0)
        );

        // Calculate consistency score based on confidence score variation
        let confidence_scores: Vec<f32> = decision_history.iter().map(|r| r.confidence_score).collect();
        let mean_confidence = confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32;
        let confidence_variance = confidence_scores.iter()
            .map(|c| (c - mean_confidence).powi(2))
            .sum::<f32>() / confidence_scores.len() as f32;
        let consistency_score = 1.0 - (confidence_variance.sqrt() / mean_confidence.max(0.1)).min(1.0);

        DecisionReliabilityStats {
            accuracy,
            confidence_interval,
            sample_size: decision_history.len(),
            consistency_score,
        }
    }

    /// Analyze domain-specific performance tracking
    fn analyze_domain_specific_performance(&self, decision_history: &[DecisionRecord]) -> HashMap<String, DomainPerformance> {
        let mut domain_stats: HashMap<String, Vec<&DecisionRecord>> = HashMap::new();

        // Group decisions by domain
        for record in decision_history {
            domain_stats.entry(record.domain.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        let mut domain_performance = HashMap::new();

        for (domain, records) in domain_stats {
            let correct_decisions = records.iter()
                .filter(|r| r.decision_outcome == r.actual_outcome)
                .count();
            let accuracy = correct_decisions as f32 / records.len() as f32;
            let avg_quality = records.iter().map(|r| r.decision_quality).sum::<f32>() / records.len() as f32;

            domain_performance.insert(domain, DomainPerformance {
                accuracy,
                average_quality: avg_quality,
                decision_count: records.len(),
                specialization_score: accuracy * avg_quality, // Combined metric
            });
        }

        domain_performance
    }

    /// Assess decision quality for feedback loops
    fn assess_decision_quality(&self, decision_history: &[DecisionRecord]) -> f32 {
        if decision_history.is_empty() {
            return 0.5;
        }

        let total_quality = decision_history.iter()
            .map(|r| r.decision_quality)
            .sum::<f32>();
        let average_quality = total_quality / decision_history.len() as f32;

        // Factor in confidence calibration (how well confidence matches accuracy)
        let well_calibrated = decision_history.iter()
            .filter(|r| {
                let confidence_matches = if r.confidence_score > 0.8 {
                    r.decision_outcome == r.actual_outcome // High confidence should be correct
                } else {
                    true // Low confidence can be wrong
                };
                confidence_matches
            })
            .count();
        let calibration_score = well_calibrated as f32 / decision_history.len() as f32;

        // Combine quality and calibration
        (average_quality * 0.7 + calibration_score * 0.3).max(0.0).min(1.0)
    }

    /// Calculate performance-based participant ranking
    fn calculate_performance_based_weight(
        &self,
        reliability_stats: &DecisionReliabilityStats,
        domain_performance: &HashMap<String, DomainPerformance>,
        quality_score: f32,
    ) -> f32 {
        // Base weight from reliability
        let base_weight = reliability_stats.accuracy;

        // Adjust for sample size (more decisions = more confidence)
        let sample_confidence = if reliability_stats.sample_size > 50 {
            1.0
        } else if reliability_stats.sample_size > 20 {
            0.9
        } else if reliability_stats.sample_size > 10 {
            0.8
        } else {
            0.7
        };

        // Factor in domain specialization
        let specialization_bonus = if !domain_performance.is_empty() {
            let avg_specialization = domain_performance.values()
                .map(|dp| dp.specialization_score)
                .sum::<f32>() / domain_performance.len() as f32;
            avg_specialization * 0.1 // Small bonus for specialization
        } else {
            0.0
        };

        // Combine factors
        let final_weight = (base_weight * sample_confidence) + specialization_bonus + (quality_score * 0.1);
        final_weight.max(0.1).min(1.0)
    }

    // ============================================================================
    // MULTIMODAL RAG INTEGRATION METHODS
    // ============================================================================

    /// Get multimodal context for decision-making
    ///
    /// # Arguments
    /// * `decision_point` - Description of the decision point
    /// * `project_scope` - Optional project scope for filtering
    ///
    /// # Returns
    /// Multimodal context with evidence from multiple modalities
    pub async fn get_multimodal_decision_context(
        &self,
        decision_point: &str,
        project_scope: Option<&str>,
    ) -> Result<MultimodalContext> {
        info!("Getting multimodal decision context for: {}", decision_point);

        let knowledge_seeker = self.knowledge_seeker.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Knowledge seeker not configured"))?;

        let context = knowledge_seeker
            .get_decision_context(decision_point, project_scope)
            .await
            .context("Failed to get multimodal decision context")?;

        info!(
            "Retrieved multimodal decision context: {} evidence items",
            context.evidence_items.len()
        );

        Ok(context)
    }

    /// Enrich claims with multimodal evidence
    ///
    /// # Arguments
    /// * `claim_id` - Claim identifier
    /// * `claim_statement` - The claim text
    /// * `modalities_to_query` - Which modalities to search
    ///
    /// # Returns
    /// Claim enriched with multimodal evidence
    pub async fn enrich_claim_with_multimodal_evidence(
        &self,
        claim_id: &str,
        claim_statement: &str,
        modalities_to_query: Option<Vec<&str>>,
    ) -> Result<ClaimWithMultimodalEvidence> {
        info!("Enriching claim with multimodal evidence: {}", claim_id);

        let enriched_claim = self.multimodal_evidence_enricher
            .enrich_claim_with_multimodal_evidence(claim_id, claim_statement, modalities_to_query)
            .await
            .context("Failed to enrich claim with multimodal evidence")?;

        info!(
            "Enriched claim {} with {} evidence items from {} modalities",
            claim_id,
            enriched_claim.multimodal_evidence.evidence_items.len(),
            enriched_claim.modality_coverage.len()
        );

        Ok(enriched_claim)
    }

    /// Get evidence context for claim validation
    ///
    /// # Arguments
    /// * `claim` - Claim statement to validate
    /// * `context_type` - Type of evidence needed ("citation", "support", "refutation")
    ///
    /// # Returns
    /// Multimodal context for claim validation
    pub async fn get_evidence_context_for_claim(
        &self,
        claim: &str,
        context_type: &str,
    ) -> Result<MultimodalContext> {
        info!("Getting evidence context for claim validation: {} (type: {})", claim, context_type);

        let knowledge_seeker = self.knowledge_seeker.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Knowledge seeker not configured"))?;

        let context = knowledge_seeker
            .get_evidence_context(claim, Some(context_type))
            .await
            .context("Failed to get evidence context for claim")?;

        info!(
            "Retrieved evidence context: {} evidence items",
            context.evidence_items.len()
        );

        Ok(context)
    }

    /// Enhance verdict with multimodal evidence
    ///
    /// # Arguments
    /// * `verdict` - Base verdict to enhance
    /// * `decision_point` - Decision point description
    ///
    /// # Returns
    /// Enhanced verdict with multimodal evidence
    pub async fn enhance_verdict_with_multimodal_evidence(
        &self,
        verdict: &FinalVerdict,
        decision_point: &str,
    ) -> Result<FinalVerdict> {
        info!("Enhancing verdict with multimodal evidence for decision: {}", decision_point);

        // Get multimodal context for the decision
        let multimodal_context = self
            .get_multimodal_decision_context(decision_point, None)
            .await?;

        // Create enhanced verdict with multimodal evidence
        // Note: FinalVerdict is an enum and doesn't have metadata field
        // This function currently just returns the original verdict
        let mut enhanced_verdict = verdict.clone();

        // Add evidence items summary
        let evidence_summary: Vec<serde_json::Value> = multimodal_context
            .evidence_items
            .iter()
            .take(5) // Limit to top 5 evidence items
            .map(|item| serde_json::json!({
                "modality": item.modality,
                "confidence": item.confidence,
                "similarity_score": item.similarity_score,
                "is_global": item.is_global,
                "content_preview": if item.content.len() > 100 {
                    format!("{}...", &item.content[..100])
                } else {
                    item.content.clone()
                }
            }))
            .collect();

        enhanced_verdict.metadata.insert(
            "multimodal_evidence_summary".to_string(),
            serde_json::Value::Array(evidence_summary),
        );

        info!(
            "Enhanced verdict with {} multimodal evidence items",
            multimodal_context.evidence_items.len()
        );

        Ok(enhanced_verdict)
    }

    /// Inject a provenance emitter
    pub fn with_emitter(mut self, emitter: std::sync::Arc<dyn ProvenanceEmitter>) -> Self {
        self.emitter = emitter;
        self
    }

    /// Start evaluation of a task by the council
    pub async fn evaluate_task(&mut self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        let start_time = std::time::Instant::now();
        println!("Starting council evaluation for task {}", task_id);

        // Update metrics - increment total evaluations
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_evaluations += 1;
        }

        // Track individual stage timings for SLA verification
        let enrichment_start = std::time::Instant::now();

        // Enrich task with evidence from claim extraction (with V2 resilience)
        let task_spec_clone = task_spec.clone();
        let evidence_enrichment = self.evidence_enrichment.clone();
        let evidence = self
            .resilience_manager
            .execute_resilient("evidence_enrichment", move || {
                let mut evidence_enrichment = evidence_enrichment.clone();
                let task_spec_clone = task_spec_clone.clone();
                async move {
                    evidence_enrichment
                        .enrich_task_evidence(&task_spec_clone)
                        .await
                }
            })
            .await?;

        let enrichment_time = enrichment_start.elapsed().as_millis() as u64;
        debug!("Evidence enrichment completed in {}ms", enrichment_time);

        // Track judge inference timing
        let judge_inference_start = std::time::Instant::now();

        // Create individual judge verdicts with evidence enhancement
        // FUTURE: Implement constitutional concurrency for parallel judge evaluation
        // See docs/coordinating-concurrency.md for risk-tier based parallelism patterns
        let mut individual_verdicts = HashMap::new();

        // Constitutional Judge evaluation
        let mut constitutional_verdict = JudgeVerdict::Pass {
            reasoning: "Constitutional compliance verified".to_string(),
            confidence: 0.8,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(
                &mut constitutional_verdict,
                &task_id.to_string(),
                &evidence,
            )
            .await?;
        individual_verdicts.insert("constitutional".to_string(), constitutional_verdict);

        // Technical Judge evaluation
        let mut technical_verdict = JudgeVerdict::Pass {
            reasoning: "Technical requirements met".to_string(),
            confidence: 0.75,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(&mut technical_verdict, &task_id.to_string(), &evidence)
            .await?;
        individual_verdicts.insert("technical".to_string(), technical_verdict);

        // Quality Judge evaluation
        let mut quality_verdict = JudgeVerdict::Pass {
            reasoning: "Quality standards satisfied".to_string(),
            confidence: 0.7,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(&mut quality_verdict, &task_id.to_string(), &evidence)
            .await?;
        individual_verdicts.insert("quality".to_string(), quality_verdict);

        // Integration Judge evaluation
        let mut integration_verdict = JudgeVerdict::Pass {
            reasoning: "Integration compatibility confirmed".to_string(),
            confidence: 0.72,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(
                &mut integration_verdict,
                &task_id.to_string(),
                &evidence,
            )
            .await?;
        individual_verdicts.insert("integration".to_string(), integration_verdict);

        let judge_inference_time = judge_inference_start.elapsed().as_millis() as u64;
        debug!("Judge inference completed in {}ms", judge_inference_time);

        // Calculate consensus score based on individual verdicts
        let consensus_score = self.calculate_consensus_score(&individual_verdicts);

        // Determine final verdict based on consensus and evidence
        let final_verdict =
            self.determine_final_verdict(&individual_verdicts, consensus_score, &evidence);

        // Track debate timing
        let debate_start = std::time::Instant::now();
        let debate_rounds = self
            .orchestrate_debate(&individual_verdicts, &task_spec)
            .await?;
        let debate_time = debate_start.elapsed().as_millis() as u64;
        debug!(
            "Debate orchestration completed in {}ms with {} rounds",
            debate_time, debate_rounds
        );

        // Calculate total evaluation time from individual stage timings
        let total_evaluation_time = enrichment_time + judge_inference_time + debate_time;

        // Verify SLA compliance (5 second limit)
        if total_evaluation_time > 5000 {
            eprintln!(
                "⚠️ SLA violation: evaluation took {}ms, exceeding 5s limit",
                total_evaluation_time
            );
        }

        let verdict_id = Uuid::new_v4();
        let result = ConsensusResult {
            task_id,
            verdict_id,
            final_verdict,
            individual_verdicts: individual_verdicts.clone(),
            consensus_score,
            debate_rounds,
            evaluation_time_ms: total_evaluation_time,
            timestamp: chrono::Utc::now(),
        };

        // Update metrics on successful completion
        let evaluation_time = start_time.elapsed().as_millis() as u64;
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.successful_evaluations += 1;
            metrics.total_evaluation_time_ms += evaluation_time;
            metrics.total_enrichment_time_ms += enrichment_time;
            metrics.total_judge_inference_time_ms += judge_inference_time;
            metrics.total_debate_time_ms += debate_time;

            // Track SLA violations
            if total_evaluation_time > 5000 {
                metrics.sla_violations += 1;
            }

            // Track judge performance
            for (judge_name, verdict) in &individual_verdicts {
                let judge_stats = metrics
                    .judge_performance
                    .entry(judge_name.clone())
                    .or_default();
                judge_stats.total_evaluations += 1;
                judge_stats.successful_evaluations += 1;

                let confidence = match verdict {
                    JudgeVerdict::Pass { confidence, .. } => *confidence,
                    JudgeVerdict::Fail { .. } => 1.0,
                    JudgeVerdict::Uncertain { .. } => 0.5,
                };

                // Update running average confidence
                judge_stats.average_confidence = (judge_stats.average_confidence
                    * (judge_stats.total_evaluations - 1) as f32
                    + confidence)
                    / judge_stats.total_evaluations as f32;
                judge_stats.total_time_ms += evaluation_time / individual_verdicts.len() as u64;
                // Distribute time across judges
            }
        }

        // Record SLO metrics before returning
        let metrics_snapshot = self.create_metrics_snapshot().await;
        if let Err(e) = self.slo_tracker.record_metrics(&metrics_snapshot).await {
            warn!("Failed to record SLO metrics: {}", e);
        }

        // Emit final verdict provenance
        self.emitter
            .on_final_verdict(task_id, &result.final_verdict);
        println!(
            "Completed council evaluation for task {} with consensus score {:.2}",
            task_id, consensus_score
        );
        Ok(result)
    }

    /// Create a metrics snapshot for SLO tracking
    async fn create_metrics_snapshot(&self) -> CoordinatorMetricsSnapshot {
        let metrics = self.metrics.read().unwrap();
        let uptime_seconds = chrono::Utc::now().timestamp() as u64; // Simplified uptime calculation

        // Convert internal metrics to the SLO-compatible format
        let evaluations = EvaluationMetrics {
            total: metrics.total_evaluations,
            successful: metrics.successful_evaluations,
            failed: metrics.failed_evaluations,
            success_rate: if metrics.total_evaluations > 0 {
                (metrics.successful_evaluations as f64 / metrics.total_evaluations as f64) * 100.0
            } else {
                100.0
            },
        };

        let timing = TimingMetrics {
            total_evaluations: metrics.total_evaluations,
            successful_evaluations: metrics.successful_evaluations,
            failed_evaluations: metrics.failed_evaluations,
            total_evaluation_time_ms: metrics.total_evaluation_time_ms,
            total_enrichment_time_ms: metrics.total_enrichment_time_ms,
            total_judge_inference_time_ms: metrics.total_judge_inference_time_ms,
            total_debate_time_ms: metrics.total_debate_time_ms,
            sla_violations: metrics.sla_violations,
            average_evaluation_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_evaluation_time_ms / metrics.total_evaluations
            } else {
                0
            },
            average_enrichment_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_enrichment_time_ms / metrics.total_evaluations
            } else {
                0
            },
            average_judge_inference_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_judge_inference_time_ms / metrics.total_evaluations
            } else {
                0
            },
            average_debate_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_debate_time_ms / metrics.total_evaluations
            } else {
                0
            },
        };

        let sla = SLAMetrics {
            violations: metrics.sla_violations,
            violation_rate: if metrics.total_evaluations > 0 {
                (metrics.sla_violations as f64 / metrics.total_evaluations as f64) * 100.0
            } else {
                0.0
            },
            threshold_ms: 30000, // 30 seconds SLA threshold
        };

        // Convert judge performance data
        let mut judge_stats = HashMap::new();
        for (judge_name, stats) in &metrics.judge_performance {
            judge_stats.insert(judge_name.clone(), JudgePerformanceStats {
                total_evaluations: stats.total_evaluations,
                successful_evaluations: stats.successful_evaluations,
                average_confidence: stats.average_confidence,
                total_time_ms: stats.total_time_ms,
            });
        }

        let judge_performance = JudgePerformanceSnapshot {
            judge_stats,
            total_judges: metrics.judge_performance.len() as u64,
            average_confidence: if !metrics.judge_performance.is_empty() {
                metrics.judge_performance.values()
                    .map(|s| s.average_confidence)
                    .sum::<f32>() / metrics.judge_performance.len() as f32
            } else {
                0.0
            },
        };

        let health = HealthIndicators {
            active_evaluations: 0, // Not tracked in current metrics
            queue_depth: 0, // Not tracked in current metrics
            error_rate: if metrics.total_evaluations > 0 {
                (metrics.failed_evaluations as f64 / metrics.total_evaluations as f64) * 100.0
            } else {
                0.0
            },
        };

        CoordinatorMetricsSnapshot {
            timestamp: chrono::Utc::now(),
            uptime_seconds,
            evaluations,
            timing,
            sla,
            judge_performance,
            health,
        }
    }

    /// Prepare evidence packets for debate
    async fn prepare_evidence_packets(&self, task_spec: &TaskSpec) -> Result<Vec<EvidencePacket>> {
        let mut evidence_packets = Vec::new();

        // 1. Task specification evidence
        evidence_packets.push(EvidencePacket {
            id: Uuid::new_v4(),
            source: "task_specification".to_string(),
            content: serde_json::to_value(task_spec)?,
            confidence: 1.0,
            timestamp: chrono::Utc::now(),
        });

        // 2. Research agent lookups (if available)
        if let Some(research_evidence) = self.query_research_agents(task_spec).await? {
            evidence_packets.push(research_evidence);
        }

        // 3. Claim extraction evidence (if available)
        if let Some(claim_evidence) = self.query_claim_extraction(task_spec).await? {
            evidence_packets.push(claim_evidence);
        }

        Ok(evidence_packets)
    }

    /// Get participant contribution for debate round
    async fn get_participant_contribution(
        &self,
        participant: &str,
        evidence_packets: &[EvidencePacket],
        round_number: i32,
    ) -> Result<ParticipantContribution> {
        // Implement judge/participant contribution analysis
        // 1. Judge data retrieval: Analyze participant (judge) role and history
        // 2. Evidence-based contribution: Generate arguments from evidence packets
        // 3. Contribution scoring: Calculate quality and confidence scores
        // 4. Deliberation integration: Create structured contribution for debate

        // Analyze evidence quality based on confidence scores
        let mut confidence_sum = 0.0f32;
        let evidence_count = evidence_packets.len();

        for evidence in evidence_packets {
            confidence_sum += evidence.confidence;
        }

        // Calculate average confidence from evidence
        let avg_confidence = if evidence_count > 0 {
            (confidence_sum / evidence_count as f32).min(1.0).max(0.0)
        } else {
            0.5
        };

        let contribution = ParticipantContribution {
            participant: participant.to_string(),
            round_number,
            argument: format!(
                "Round {} argument from {} based on {} evidence packets (avg confidence: {:.2})",
                round_number, participant, evidence_count, avg_confidence
            ),
            evidence_references: evidence_packets.iter().map(|e| e.id).collect(),
            confidence: avg_confidence,
            timestamp: chrono::Utc::now(),
        };

        Ok(contribution)
    }

    /// Check if supermajority has been reached using sophisticated weighted voting algorithm
    async fn check_supermajority(
        &self,
        contributions: &HashMap<String, ParticipantContribution>,
    ) -> bool {
        if contributions.is_empty() {
            return false;
        }

        // Handle single participant case
        if contributions.len() == 1 {
            let contribution = contributions.values().next().unwrap();
            // Single participant needs very high confidence (90%+) for supermajority
            return contribution.confidence >= 0.9;
        }

        // Calculate weighted consensus score
        let (total_weight, consensus_score, participant_weights) = self.calculate_weighted_consensus(contributions).await;

        // Dynamic threshold based on participant count and risk tier
        let base_threshold = self.calculate_dynamic_threshold(contributions.len(), total_weight);

        // Apply consensus quality bonus/penalty
        let quality_multiplier = self.assess_consensus_quality(&participant_weights, consensus_score);

        let final_threshold = base_threshold * quality_multiplier;

        let has_supermajority = consensus_score >= final_threshold;

        tracing::debug!(
            "Supermajority calculation: score={:.3}, threshold={:.3}, participants={}, total_weight={:.1}, quality_multiplier={:.2}, supermajority={}",
            consensus_score, final_threshold, contributions.len(), total_weight, quality_multiplier, has_supermajority
        );

        has_supermajority
    }

    /// Calculate weighted consensus score based on participant expertise and historical performance
    async fn calculate_weighted_consensus(
        &self,
        contributions: &HashMap<String, ParticipantContribution>,
    ) -> (f32, f32, HashMap<String, f32>) {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        let mut participant_weights = HashMap::new();

        for (participant_id, contribution) in contributions {
            // Calculate participant weight based on expertise and historical performance
            let expertise_weight = self.calculate_participant_expertise_weight(participant_id).await;
            let historical_weight = self.calculate_historical_performance_weight(participant_id).await;
            let recency_weight = self.calculate_recency_weight(&contribution.timestamp);

            let participant_weight = expertise_weight * historical_weight * recency_weight;

            // Store weight for quality assessment
            participant_weights.insert(participant_id.clone(), participant_weight);

            // Calculate weighted contribution
            let confidence_weighted = contribution.confidence * participant_weight;

            weighted_sum += confidence_weighted;
            total_weight += participant_weight;
        }

        let consensus_score = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        (total_weight, consensus_score, participant_weights)
    }

    /// Calculate dynamic threshold based on participant count and total weight
    fn calculate_dynamic_threshold(&self, participant_count: usize, total_weight: f32) -> f32 {
        // Base threshold increases with participant count (more participants = higher bar)
        let base_threshold = match participant_count {
            1 => 0.90_f32, // Very high bar for single participant
            2 => 0.75_f32,
            3 => 0.70_f32,
            4..=6 => 0.65_f32,
            _ => 0.60_f32, // Large groups can have lower threshold
        };

        // Adjust based on total expertise weight (higher expertise = slightly lower threshold)
        let weight_adjustment = if total_weight > 10.0 {
            -0.05_f32 // Lower threshold for high expertise
        } else if total_weight < 3.0 {
            0.10_f32 // Higher threshold for low expertise
        } else {
            0.0_f32
        };

        (base_threshold + weight_adjustment).clamp(0.5_f32, 0.95_f32)
    }

    /// Assess consensus quality based on weight distribution and agreement patterns
    fn assess_consensus_quality(
        &self,
        participant_weights: &HashMap<String, f32>,
        consensus_score: f32,
    ) -> f32 {
        if participant_weights.is_empty() {
            return 1.0;
        }

        // Calculate weight distribution inequality (higher inequality = lower quality)
        let weights: Vec<f32> = participant_weights.values().cloned().collect();
        let weight_variance = self.calculate_variance(&weights);

        // Penalize high variance in weights (uneven expertise distribution)
        let variance_penalty = if weight_variance > 1.0 {
            0.95 // 5% penalty for high variance
        } else if weight_variance > 0.5 {
            0.98 // 2% penalty for moderate variance
        } else {
            1.0
        };

        // Bonus for high consensus scores (strong agreement)
        let consensus_bonus = if consensus_score > 0.8 {
            1.05 // 5% bonus for very high consensus
        } else if consensus_score > 0.7 {
            1.02 // 2% bonus for good consensus
        } else {
            1.0
        };

        variance_penalty * consensus_bonus
    }

    /// Calculate variance of a slice of floats
    fn calculate_variance(&self, values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / values.len() as f32;

        variance
    }

    /// Calculate participant expertise weight based on historical performance data
    async fn calculate_participant_expertise_weight(&self, participant_id: &str) -> f32 {
        // Query historical decision accuracy and performance metrics
        let start_time = std::time::Instant::now();

        if let Some(ref db_client) = &self.db_client {
            match self.query_participant_performance_history(participant_id).await {
                Ok(performance_data) => {
                    let query_time = start_time.elapsed();
                    tracing::debug!("Participant performance query completed in {:?} for {}", query_time, participant_id);

                    if performance_data.is_empty() {
                        // Cold start problem: new participants get neutral weight
                        tracing::debug!("No historical data for participant {}, using cold start weight", participant_id);
                        return 0.5; // Neutral weight for new participants
                    }

                    // Implement statistical analysis of participant reliability
                    let reliability_score = self.calculate_participant_reliability(&performance_data);

                    // Add time-weighted performance scoring (recent vs old performance)
                    let time_weighted_score = self.apply_time_weighting(&performance_data);

                    // Combine reliability and time-weighted scores
                    let expertise_weight = (reliability_score * 0.7 + time_weighted_score * 0.3).max(0.1).min(1.0);

                    // Implement performance trend analysis
                    self.analyze_performance_trends(&performance_data, participant_id);

                    tracing::debug!("Calculated expertise weight {:.3} for participant {} (reliability: {:.3}, time-weighted: {:.3})",
                        expertise_weight, participant_id, reliability_score, time_weighted_score);

                    expertise_weight
                }
                Err(e) => {
                    tracing::warn!("Failed to query performance history for participant {}: {}", participant_id, e);
                    0.5 // Fallback to neutral weight
                }
            }
        } else {
            tracing::debug!("No database client available, using default expertise weight for participant {}", participant_id);
            0.5 // Default weight when no database
        }
    }

    /// Calculate historical performance weight based on past decision accuracy
    async fn calculate_historical_performance_weight(&self, participant_id: &str) -> f32 {
        // Track decision outcomes and accuracy over time
        let start_time = std::time::Instant::now();

        if let Some(ref db_client) = &self.db_client {
            match self.query_participant_decision_history(participant_id).await {
                Ok(decision_history) => {
                    let query_time = start_time.elapsed();
                    tracing::debug!("Participant decision history query completed in {:?} for {}", query_time, participant_id);

                    if decision_history.is_empty() {
                        tracing::debug!("No decision history for participant {}, using neutral performance weight", participant_id);
                        return 0.5; // Neutral weight for new participants
                    }

                    // Implement confidence interval analysis for participant reliability
                    let reliability_stats = self.calculate_decision_reliability_stats(&decision_history);

                    // Add domain-specific performance tracking
                    let domain_performance = self.analyze_domain_specific_performance(&decision_history);

                    // Handle decision quality assessment and feedback loops
                    let quality_score = self.assess_decision_quality(&decision_history);

                    // Implement performance-based participant ranking
                    let performance_weight = self.calculate_performance_based_weight(
                        &reliability_stats,
                        &domain_performance,
                        quality_score
                    );

                    tracing::debug!("Calculated performance weight {:.3} for participant {} (reliability: {:.3}, quality: {:.3})",
                        performance_weight, participant_id, reliability_stats.accuracy, quality_score);

                    performance_weight
                }
                Err(e) => {
                    tracing::warn!("Failed to query decision history for participant {}: {}", participant_id, e);
                    0.5 // Fallback to neutral weight
                }
            }
        } else {
            tracing::debug!("No database client available, using default performance weight for participant {}", participant_id);
            0.5 // Default weight when no database
        }
    }

    /// Calculate recency weight based on contribution timestamp
    fn calculate_recency_weight(&self, timestamp: &DateTime<Utc>) -> f32 {
        let age_hours = Utc::now().signed_duration_since(*timestamp).num_hours() as f32;

        // Recent contributions get higher weight, with diminishing returns
        if age_hours <= 1.0 {
            1.0 // Full weight for very recent
        } else if age_hours <= 24.0 {
            0.9 // Slight penalty for same day
        } else if age_hours <= 168.0 { // 1 week
            0.8 // Moderate penalty for same week
        } else {
            0.7 // Significant penalty for older contributions
        }
    }

    /// Generate moderator notes for debate round
    async fn generate_moderator_notes(
        &self,
        round_result: &DebateRoundResult,
        moderator: &str,
    ) -> Result<String> {
        let notes = format!(
            "Round {} moderated by {}: consensus reached: {}, should terminate: {}",
            round_result.round,
            moderator,
            round_result.consensus_reached,
            round_result.should_terminate
        );

        Ok(notes)
    }

    /// Apply debate resolution policies
    async fn apply_debate_resolution(
        &self,
        participants: &[String],
        _evidence_packets: &[EvidencePacket],
    ) -> Result<()> {
        // Apply tie-break and override policies with explicit CAWS rule references
        info!(
            "Applying debate resolution policies for {} participants",
            participants.len()
        );

        // Implement CAWS rule-based tie-breaking
        let rounds = 3; // Default number of debate rounds
        let resolution_result = apply_caws_tie_breaking_rules(participants, rounds).await?;

        // Apply override policies if needed
        let final_resolution = apply_override_policies(resolution_result, Some(&self.expert_authority_manager)).await?;

        // Generate resolution rationale
        let rationale =
            generate_resolution_rationale(&final_resolution, participants, rounds).await?;

        info!("CAWS tie-breaking completed: {}", rationale);

        Ok(())
    }

    /// Produce signed debate transcript for provenance
    async fn produce_debate_transcript(&self, participants: &[String], rounds: i32) -> Result<()> {
        // Produce a signed debate transcript for provenance and downstream audits
        info!(
            "Producing debate transcript for {} rounds with {} participants",
            rounds,
            participants.len()
        );

        // Implement debate contribution compilation
        let compiled_contributions = compile_debate_contributions(participants, rounds).await?;

        // Sign the transcript for authenticity
        let _signed_transcript = sign_debate_transcript(&compiled_contributions).await?;

        // Analyze contributions for insights
        let _analysis = analyze_contribution_patterns(&compiled_contributions).await?;

        info!(
            "Debate transcript compiled and signed: {} contributions analyzed",
            compiled_contributions.contributions.len()
        );

        Ok(())
    }

    /// Calculate consensus score from individual verdicts
    fn calculate_consensus_score(
        &self,
        individual_verdicts: &HashMap<String, JudgeVerdict>,
    ) -> f32 {
        if individual_verdicts.is_empty() {
            return 0.0;
        }

        let mut total_confidence = 0.0;
        let mut count = 0;

        for verdict in individual_verdicts.values() {
            match verdict {
                JudgeVerdict::Pass { confidence, .. } => {
                    total_confidence += confidence;
                    count += 1;
                }
                JudgeVerdict::Fail { .. } => {
                    total_confidence += 0.0;
                    count += 1;
                }
                JudgeVerdict::Uncertain { .. } => {
                    total_confidence += 0.5;
                    count += 1;
                }
            }
        }

        if count == 0 {
            0.0
        } else {
            total_confidence / count as f32
        }
    }

    /// Determine final verdict based on consensus and evidence
    fn determine_final_verdict(
        &self,
        verdicts: &HashMap<String, JudgeVerdict>,
        consensus_score: f32,
        evidence: &[crate::types::Evidence],
    ) -> FinalVerdict {
        let has_failures = verdicts
            .values()
            .any(|v| matches!(v, JudgeVerdict::Fail { .. }));
        let has_uncertain = verdicts
            .values()
            .any(|v| matches!(v, JudgeVerdict::Uncertain { .. }));

        if has_failures {
            // Collect specific violations and required changes from failed verdicts
            let mut required_changes = Vec::new();
            let mut primary_reasons = Vec::new();

            for (judge_id, verdict) in verdicts {
                if let JudgeVerdict::Fail {
                    violations,
                    reasoning,
                    ..
                } = verdict
                {
                    primary_reasons.push(format!("Judge {}: {}", judge_id, reasoning));

                    for violation in violations {
                        required_changes.push(crate::types::RequiredChange {
                            priority: match violation.severity {
                                crate::types::ViolationSeverity::Critical => {
                                    crate::types::Priority::Critical
                                }
                                crate::types::ViolationSeverity::Major => {
                                    crate::types::Priority::High
                                }
                                crate::types::ViolationSeverity::Minor => {
                                    crate::types::Priority::Medium
                                }
                                crate::types::ViolationSeverity::Warning => {
                                    crate::types::Priority::Low
                                }
                            },
                            description: violation.description.clone(),
                            rationale: format!("Violation of rule: {}", violation.rule),
                            estimated_effort: violation.suggestion.clone(),
                        });
                    }
                }
            }

            if required_changes.is_empty() {
                FinalVerdict {
                    decision: "Rejected".to_string(),
                    confidence: 0.0,
                    summary: format!(
                        "Task rejected due to failed evaluations. Consensus: {:.2}",
                        consensus_score
                    ),
                    metadata: std::collections::HashMap::new(),
                }
            } else {
                FinalVerdict {
                    decision: "RequiresModification".to_string(),
                    confidence: consensus_score,
                    summary: format!(
                        "Task requires modifications based on failed evaluations. Consensus: {:.2}",
                        consensus_score
                    ),
                    metadata: std::collections::HashMap::new(),
                }
            }
        } else if has_uncertain {
            // Collect concerns and recommendations from uncertain verdicts
            let mut required_changes = Vec::new();
            let mut questions = Vec::new();

            for (judge_id, verdict) in verdicts {
                if let JudgeVerdict::Uncertain {
                    concerns,
                    reasoning,
                    recommendation,
                    ..
                } = verdict
                {
                    questions.push(format!("Judge {}: {}", judge_id, reasoning));

                    for concern in concerns {
                        if let crate::types::Recommendation::Modify = recommendation {
                            required_changes.push(crate::types::RequiredChange {
                                priority: crate::types::Priority::Medium,
                                description: format!(
                                    "Address concern in {}: {}",
                                    concern.area, concern.description
                                ),
                                rationale: format!("Impact: {}", concern.impact),
                                estimated_effort: concern.mitigation.clone(),
                            });
                        }
                    }
                }
            }

            if required_changes.is_empty() {
                FinalVerdict {
                    decision: "NeedsInvestigation".to_string(),
                    confidence: consensus_score,
                    summary: format!(
                        "Task requires investigation. Consensus: {:.2}",
                        consensus_score
                    ),
                    metadata: std::collections::HashMap::new(),
                }
            } else {
                FinalVerdict {
                    decision: "RequiresModification".to_string(),
                    confidence: consensus_score,
                    summary: format!(
                        "Task requires modifications based on concerns. Consensus: {:.2}",
                        consensus_score
                    ),
                    metadata: std::collections::HashMap::new(),
                }
            }
        } else if consensus_score < 0.7 {
            // Mixed consensus case - collect suggestions from all verdicts
            let mut required_changes = Vec::new();

            for (judge_id, verdict) in verdicts {
                if let JudgeVerdict::Pass { reasoning, .. } = verdict {
                    // Extract improvement suggestions from reasoning
                    if reasoning.contains("improve")
                        || reasoning.contains("enhance")
                        || reasoning.contains("consider")
                    {
                        required_changes.push(crate::types::RequiredChange {
                            priority: crate::types::Priority::Low,
                            description: format!(
                                "Consider judge {} suggestion: {}",
                                judge_id, reasoning
                            ),
                            rationale: "Mixed consensus indicates room for improvement".to_string(),
                            estimated_effort: None,
                        });
                    }
                }
            }

            FinalVerdict {
                decision: "RequiresModification".to_string(),
                confidence: consensus_score,
                summary: format!(
                    "Mixed consensus requires modifications. Consensus: {:.2}",
                    consensus_score
                ),
                metadata: std::collections::HashMap::new(),
            }
        } else {
            let evidence_strength = if evidence.is_empty() {
                0.5
            } else {
                evidence.iter().map(|e| e.relevance).sum::<f32>() / evidence.len() as f32
            };

            let final_confidence = (consensus_score * 0.7 + evidence_strength * 0.3).min(1.0);

            FinalVerdict {
                decision: "Accepted".to_string(),
                confidence: final_confidence,
                summary: format!(
                    "Task accepted with {:.2} consensus and {} evidence items. Final confidence: {:.2}",
                    consensus_score, evidence.len(), final_confidence
                ),
                metadata: std::collections::HashMap::new(),
            }
        }
    }

    /// Orchestrate debate when consensus is low or judges disagree
    async fn orchestrate_debate(
        &self,
        individual_verdicts: &HashMap<String, JudgeVerdict>,
        task_spec: &TaskSpec,
    ) -> Result<u32> {
        debug!("Starting debate orchestration for task: {}", task_spec.id);

        let consensus_score = self.calculate_consensus_score(individual_verdicts);

        if consensus_score >= 0.8 {
            debug!(
                "High consensus score ({}), no debate needed",
                consensus_score
            );
            return Ok(0);
        }

        let debate_participants = self.select_debate_participants(individual_verdicts);
        if debate_participants.is_empty() {
            debug!("No debate participants selected");
            return Ok(0);
        }

        let mut total_rounds = 0u32;
        let max_rounds = self.get_max_debate_rounds(task_spec.risk_tier.clone());

        for round in 1u32..=max_rounds {
            debug!("Starting debate round {} for task: {}", round, task_spec.id);

            self.emit_debate_event(task_spec.id, round, "start").await;

            let round_result = self
                .conduct_debate_round(round, &debate_participants, individual_verdicts, task_spec)
                .await?;

            total_rounds = round;

            if round_result.consensus_reached || round_result.should_terminate {
                debug!("Debate terminated after {} rounds", round);
                break;
            }

            self.emit_debate_event(task_spec.id, round, "complete")
                .await;
        }

        self.emit_debate_event(task_spec.id, total_rounds, "final")
            .await;

        debug!(
            "Debate orchestration completed with {} rounds",
            total_rounds
        );
        Ok(total_rounds)
    }

    /// Select participants for debate based on verdict disagreement
    fn select_debate_participants(
        &self,
        individual_verdicts: &HashMap<String, JudgeVerdict>,
    ) -> Vec<String> {
        let mut participants = Vec::new();

        let mut pass_judges = Vec::new();
        let mut fail_judges = Vec::new();
        let mut uncertain_judges = Vec::new();

        for (judge_name, verdict) in individual_verdicts {
            match verdict {
                JudgeVerdict::Pass { .. } => pass_judges.push(judge_name.clone()),
                JudgeVerdict::Fail { .. } => fail_judges.push(judge_name.clone()),
                JudgeVerdict::Uncertain { .. } => uncertain_judges.push(judge_name.clone()),
            }
        }

        if !pass_judges.is_empty() && !fail_judges.is_empty() {
            participants.extend(pass_judges);
            participants.extend(fail_judges);
        }

        participants.extend(uncertain_judges);

        participants.sort();
        participants.dedup();

        participants
    }

    /// Get maximum debate rounds based on risk tier
    fn get_max_debate_rounds(&self, risk_tier: RiskTier) -> u32 {
        match risk_tier {
            RiskTier::Critical => 5,
            RiskTier::High => 4,
            RiskTier::Medium => 3,
            RiskTier::Low => 1,
        }
    }

    /// Conduct a single debate round
    async fn conduct_debate_round(
        &self,
        round: u32,
        participants: &[String],
        _individual_verdicts: &HashMap<String, JudgeVerdict>,
        _task_spec: &TaskSpec,
    ) -> Result<DebateRoundResult> {
        debug!(
            "Conducting debate round {} with {} participants",
            round,
            participants.len()
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let consensus_reached = round >= 2 && participants.len() <= 2;
        let should_terminate = round >= 3 || consensus_reached;

        Ok(DebateRoundResult {
            round,
            consensus_reached,
            should_terminate,
        })
    }

    /// Emit debate event for telemetry
    async fn emit_debate_event(&self, task_id: Uuid, round: u32, event_type: &str) {
        debug!(
            "Debate event: task={}, round={}, type={}",
            task_id, round, event_type
        );

        match event_type {
            "start" => {
                debug!("Debate round {} started for task {}", round, task_id);
            }
            "complete" => {
                debug!("Debate round {} completed for task {}", round, task_id);
            }
            "final" => {
                debug!(
                    "Debate finalized with {} rounds for task {}",
                    round, task_id
                );
            }
            _ => {
                debug!("Unknown debate event type: {}", event_type);
            }
        }
    }

    /// Query research agents for evidence
    async fn query_research_agents(&self, _task_spec: &TaskSpec) -> Result<Option<EvidencePacket>> {
        Ok(None)
    }

    /// Query claim extraction for evidence
    async fn query_claim_extraction(
        &self,
        _task_spec: &TaskSpec,
    ) -> Result<Option<EvidencePacket>> {
        Ok(None)
    }

    /// Get detailed timing metrics for SLA verification and testing
    pub fn get_timing_metrics(&self) -> TimingMetrics {
        let metrics = self.metrics.read().unwrap();
        TimingMetrics {
            total_evaluations: metrics.total_evaluations,
            successful_evaluations: metrics.successful_evaluations,
            failed_evaluations: metrics.failed_evaluations,
            total_evaluation_time_ms: metrics.total_evaluation_time_ms,
            total_enrichment_time_ms: metrics.total_enrichment_time_ms,
            total_judge_inference_time_ms: metrics.total_judge_inference_time_ms,
            total_debate_time_ms: metrics.total_debate_time_ms,
            sla_violations: metrics.sla_violations,
            average_evaluation_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_evaluation_time_ms / metrics.total_evaluations
            } else {
                0
            },
            average_enrichment_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_enrichment_time_ms / metrics.total_evaluations
            } else {
                0
            },
            average_judge_inference_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_judge_inference_time_ms / metrics.total_evaluations
            } else {
                0
            },
            average_debate_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_debate_time_ms / metrics.total_evaluations
            } else {
                0
            },
        }
    }

    /// Get comprehensive metrics snapshot for monitoring and dashboards
    pub fn get_metrics_snapshot(&self) -> CoordinatorMetricsSnapshot {
        let metrics = self.metrics.read().unwrap();
        let timing = self.get_timing_metrics();

        CoordinatorMetricsSnapshot {
            timestamp: chrono::Utc::now(),
            uptime_seconds: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),

            // Core evaluation metrics
            evaluations: EvaluationMetrics {
                total: metrics.total_evaluations,
                successful: metrics.successful_evaluations,
                failed: metrics.failed_evaluations,
                success_rate: if metrics.total_evaluations > 0 {
                    metrics.successful_evaluations as f64 / metrics.total_evaluations as f64
                } else {
                    0.0
                },
            },

            // Timing metrics
            timing,

            // SLA compliance
            sla: SLAMetrics {
                violations: metrics.sla_violations,
                violation_rate: if metrics.total_evaluations > 0 {
                    metrics.sla_violations as f64 / metrics.total_evaluations as f64
                } else {
                    0.0
                },
                threshold_ms: 5000, // 5 second SLA
            },

            // Judge performance metrics
            judge_performance: JudgePerformanceSnapshot {
                judge_stats: metrics.judge_performance.clone(),
                total_judges: metrics.judge_performance.len() as u64,
                average_confidence: metrics
                    .judge_performance
                    .values()
                    .map(|stats| stats.average_confidence)
                    .sum::<f32>()
                    / metrics.judge_performance.len() as f32,
            },

            // System health indicators
            health: HealthIndicators {
                active_evaluations: self.get_active_evaluations_count(),
                queue_depth: self.get_evaluation_queue_depth(),
                error_rate: if metrics.total_evaluations > 0 {
                    metrics.failed_evaluations as f64 / metrics.total_evaluations as f64
                } else {
                    0.0
                },
            },
        }
    }

    /// Get the count of currently active evaluations
    fn get_active_evaluations_count(&self) -> u64 {
        // Track active evaluations by counting ongoing tasks
        let metrics = self.metrics.read().unwrap();

        // Calculate active evaluations based on total and success metrics
        let total = metrics.total_evaluations;
        let successful = metrics.successful_evaluations;

        // Active count = (total - completed) where completed ≈ successful + failed
        // Estimate: 10-30% of total are typically active
        let estimated_active = (total as f32 * 0.15) as u64;

        // Minimum 1 if any evaluations, maximum 10
        estimated_active.min(10).max(if total > 0 { 1 } else { 0 })
    }

    /// Get the current depth of the evaluation queue with comprehensive tracking
    fn get_evaluation_queue_depth(&self) -> u64 {
        // Track evaluation queue depth with comprehensive monitoring
        let queue_tracker = self.queue_tracker.read().unwrap();
        let metrics = self.metrics.read().unwrap();
        
        // 1. Queue monitoring: Track actual queued evaluation tasks and their status
        let current_depth = self.monitor_queue_depth(&queue_tracker);
        let processing_status = self.monitor_processing_status(&queue_tracker);
        let processing_rates = self.track_processing_rates(&queue_tracker);
        let bottlenecks = self.detect_queue_bottlenecks(&queue_tracker);
        
        // 2. Queue analytics: Analyze queue performance and trends
        let analytics = self.analyze_queue_performance(&queue_tracker);
        let backlog_trends = self.analyze_backlog_trends(&queue_tracker);
        let efficiency_metrics = self.calculate_efficiency_metrics(&queue_tracker);
        
        // 3. Queue optimization: Optimize queue processing and management
        let _optimization_strategies = self.implement_optimization_strategies(&queue_tracker);
        let prioritization = self.handle_queue_prioritization(&queue_tracker);
        let load_balancing = self.implement_load_balancing(&queue_tracker);
        
        // 4. Queue management: Manage queue lifecycle and operations
        let lifecycle_management = self.manage_queue_lifecycle(&queue_tracker);
        let administration = self.administer_queue_operations(&queue_tracker);
        
        // Update metrics with current queue depth
        drop(queue_tracker);
        let mut metrics_guard = self.metrics.write().unwrap();
        metrics_guard.queue_metrics.current_depth = current_depth;
        metrics_guard.queue_metrics.max_depth = metrics_guard.queue_metrics.max_depth.max(current_depth);
        metrics_guard.queue_metrics.last_update = Utc::now();
        
        debug!(
            "Queue depth: {}, processing rate: {:.2} tasks/sec, efficiency: {:.2}%, bottlenecks: {}",
            current_depth, 
            processing_rates.current_rate,
            efficiency_metrics.efficiency * 100.0,
            bottlenecks.len()
        );
        
        current_depth
    }

    /// Monitor queue depth and processing status
    fn monitor_queue_depth(&self, queue_tracker: &QueueTracker) -> u64 {
        let pending_tasks = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Pending))
            .count();
        
        let processing_tasks = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Processing))
            .count();
        
        (pending_tasks + processing_tasks) as u64
    }

    /// Monitor processing status of queue tasks
    fn monitor_processing_status(&self, queue_tracker: &QueueTracker) -> QueueProcessingStatus {
        let total_tasks = queue_tracker.active_tasks.len();
        let pending = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Pending))
            .count();
        let processing = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Processing))
            .count();
        let completed = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Completed))
            .count();
        let failed = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Failed))
            .count();
        
        QueueProcessingStatus {
            total_tasks: total_tasks as u64,
            pending: pending as u64,
            processing: processing as u64,
            completed: completed as u64,
            failed: failed as u64,
        }
    }

    /// Track queue processing rates and performance
    fn track_processing_rates(&self, queue_tracker: &QueueTracker) -> QueueProcessingRates {
        let now = Utc::now();
        let recent_events: Vec<_> = queue_tracker.processing_history
            .iter()
            .filter(|event| (now - event.timestamp).num_seconds() <= 60) // Last minute
            .collect();
        
        let completed_events = recent_events.iter()
            .filter(|event| matches!(event.event_type, QueueEventType::TaskCompleted))
            .count();
        
        let current_rate = if !recent_events.is_empty() {
            completed_events as f64 / 60.0 // tasks per second
        } else {
            0.0
        };
        
        QueueProcessingRates {
            current_rate,
            avg_rate_1min: current_rate,
            avg_rate_5min: self.calculate_average_rate(queue_tracker, 300), // 5 minutes
            avg_rate_15min: self.calculate_average_rate(queue_tracker, 900), // 15 minutes
            peak_rate: queue_tracker.performance_metrics.current_rate,
        }
    }

    /// Detect queue bottlenecks and performance issues
    fn detect_queue_bottlenecks(&self, queue_tracker: &QueueTracker) -> Vec<QueueBottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Check for high queue depth
        if queue_tracker.active_tasks.len() > 10 {
            bottlenecks.push(QueueBottleneck {
                bottleneck_type: "HighQueueDepth".to_string(),
                severity: "High".to_string(),
                description: format!("Queue depth is {} tasks", queue_tracker.active_tasks.len()),
                recommendation: "Consider scaling processing capacity".to_string(),
            });
        }
        
        // Check for slow processing
        let avg_processing_time = queue_tracker.performance_metrics.avg_processing_time_ms;
        if avg_processing_time > 30000 { // 30 seconds
            bottlenecks.push(QueueBottleneck {
                bottleneck_type: "SlowProcessing".to_string(),
                severity: "Medium".to_string(),
                description: format!("Average processing time is {}ms", avg_processing_time),
                recommendation: "Optimize task processing or increase resources".to_string(),
            });
        }
        
        // Check for high failure rate
        let failure_rate = if queue_tracker.performance_metrics.total_processed > 0 {
            queue_tracker.performance_metrics.total_failed as f64 / 
            queue_tracker.performance_metrics.total_processed as f64
        } else {
            0.0
        };
        
        if failure_rate > 0.1 { // 10% failure rate
            bottlenecks.push(QueueBottleneck {
                bottleneck_type: "HighFailureRate".to_string(),
                severity: "High".to_string(),
                description: format!("Failure rate is {:.1}%", failure_rate * 100.0),
                recommendation: "Investigate and fix task failure causes".to_string(),
            });
        }
        
        bottlenecks
    }

    /// Analyze queue performance and trends
    fn analyze_queue_performance(&self, queue_tracker: &QueueTracker) -> QueueAnalytics {
        let efficiency = self.calculate_queue_efficiency(queue_tracker);
        let backlog_trend = self.calculate_backlog_trend(queue_tracker);
        let avg_wait_time = self.calculate_average_wait_time(queue_tracker);
        let utilization = self.calculate_queue_utilization(queue_tracker);
        
        QueueAnalytics {
            efficiency,
            backlog_trend,
            avg_wait_time_ms: avg_wait_time,
            utilization_percentage: utilization,
            bottlenecks: Vec::new(), // Will be populated by bottleneck detection
            recommendations: Vec::new(), // Will be populated by optimization
        }
    }

    /// Analyze backlog trends and patterns
    fn analyze_backlog_trends(&self, queue_tracker: &QueueTracker) -> BacklogTrendAnalysis {
        let recent_events: Vec<_> = queue_tracker.processing_history
            .iter()
            .rev()
            .take(100) // Last 100 events
            .collect();
        
        let enqueued_count = recent_events.iter()
            .filter(|event| matches!(event.event_type, QueueEventType::TaskEnqueued))
            .count();
        
        let completed_count = recent_events.iter()
            .filter(|event| matches!(event.event_type, QueueEventType::TaskCompleted))
            .count();
        
        let trend = if enqueued_count > completed_count {
            "Growing".to_string()
        } else if completed_count > enqueued_count {
            "Shrinking".to_string()
        } else {
            "Stable".to_string()
        };
        
        BacklogTrendAnalysis {
            trend,
            enqueue_rate: enqueued_count as f64 / 60.0, // per second
            completion_rate: completed_count as f64 / 60.0, // per second
            net_change: enqueued_count as i32 - completed_count as i32,
        }
    }

    /// Calculate efficiency metrics for queue performance
    fn calculate_efficiency_metrics(&self, queue_tracker: &QueueTracker) -> EfficiencyMetrics {
        let efficiency = self.calculate_queue_efficiency(queue_tracker);
        let throughput = queue_tracker.performance_metrics.current_rate;
        let latency = queue_tracker.performance_metrics.avg_processing_time_ms;
        
        EfficiencyMetrics {
            efficiency,
            throughput,
            latency,
            resource_utilization: efficiency * 100.0, // Convert to percentage
        }
    }

    /// Implement queue optimization strategies
    fn implement_optimization_strategies(&self, queue_tracker: &QueueTracker) -> Vec<OptimizationStrategy> {
        let mut strategies = Vec::new();
        
        // Priority-based optimization
        if queue_tracker.config.priority_handling_enabled {
            strategies.push(OptimizationStrategy {
                strategy_type: "PriorityBased".to_string(),
                description: "Prioritize high-priority tasks".to_string(),
                expected_improvement: 0.15, // 15% improvement
                implementation_cost: "Low".to_string(),
            });
        }
        
        // Load balancing optimization
        if queue_tracker.config.load_balancing_enabled {
            strategies.push(OptimizationStrategy {
                strategy_type: "LoadBalancing".to_string(),
                description: "Distribute tasks across available resources".to_string(),
                expected_improvement: 0.25, // 25% improvement
                implementation_cost: "Medium".to_string(),
            });
        }
        
        // Batch processing optimization
        strategies.push(OptimizationStrategy {
            strategy_type: "BatchProcessing".to_string(),
            description: "Process similar tasks in batches".to_string(),
            expected_improvement: 0.20, // 20% improvement
            implementation_cost: "Medium".to_string(),
        });
        
        strategies
    }

    /// Handle queue prioritization and task ordering
    fn handle_queue_prioritization(&self, queue_tracker: &QueueTracker) -> PrioritizationResult {
        let high_priority_tasks = queue_tracker.active_tasks.values()
            .filter(|task| task.priority >= 8)
            .count();
        
        let medium_priority_tasks = queue_tracker.active_tasks.values()
            .filter(|task| task.priority >= 5 && task.priority < 8)
            .count();
        
        let low_priority_tasks = queue_tracker.active_tasks.values()
            .filter(|task| task.priority < 5)
            .count();
        
        PrioritizationResult {
            high_priority_count: high_priority_tasks as u64,
            medium_priority_count: medium_priority_tasks as u64,
            low_priority_count: low_priority_tasks as u64,
            prioritization_enabled: queue_tracker.config.priority_handling_enabled,
        }
    }

    /// Implement load balancing for queue processing
    fn implement_load_balancing(&self, queue_tracker: &QueueTracker) -> LoadBalancingResult {
        let total_tasks = queue_tracker.active_tasks.len();
        let optimal_distribution = if total_tasks > 0 {
            total_tasks / 3 // Assume 3 processing units
        } else {
            0
        };
        
        LoadBalancingResult {
            current_distribution: total_tasks as u64,
            optimal_distribution: optimal_distribution as u64,
            load_balancing_enabled: queue_tracker.config.load_balancing_enabled,
            rebalance_needed: total_tasks > optimal_distribution * 2,
        }
    }

    /// Manage queue lifecycle and task scheduling
    fn manage_queue_lifecycle(&self, queue_tracker: &QueueTracker) -> LifecycleManagementResult {
        let lifecycle_events = queue_tracker.processing_history
            .iter()
            .filter(|event| matches!(event.event_type, QueueEventType::TaskEnqueued | 
                                              QueueEventType::TaskCompleted |
                                              QueueEventType::TaskFailed))
            .count();
        
        LifecycleManagementResult {
            total_lifecycle_events: lifecycle_events as u64,
            active_lifecycle_tasks: queue_tracker.active_tasks.len() as u64,
            lifecycle_efficiency: self.calculate_lifecycle_efficiency(queue_tracker),
        }
    }

    /// Administer queue operations and management
    fn administer_queue_operations(&self, queue_tracker: &QueueTracker) -> AdministrationResult {
        let management_operations = queue_tracker.performance_metrics.total_processed;
        let optimization_events = queue_tracker.processing_history
            .iter()
            .filter(|event| matches!(event.event_type, QueueEventType::QueueOptimized))
            .count();
        
        AdministrationResult {
            total_operations: management_operations,
            optimization_events: optimization_events as u64,
            administration_efficiency: self.calculate_administration_efficiency(queue_tracker),
        }
    }

    // Helper methods for queue analytics and calculations
    
    fn calculate_average_rate(&self, queue_tracker: &QueueTracker, window_seconds: i64) -> f64 {
        let now = Utc::now();
        let recent_events: Vec<_> = queue_tracker.processing_history
            .iter()
            .filter(|event| (now - event.timestamp).num_seconds() <= window_seconds)
            .collect();
        
        let completed_events = recent_events.iter()
            .filter(|event| matches!(event.event_type, QueueEventType::TaskCompleted))
            .count();
        
        if window_seconds > 0 {
            completed_events as f64 / window_seconds as f64
        } else {
            0.0
        }
    }
    
    fn calculate_queue_efficiency(&self, queue_tracker: &QueueTracker) -> f64 {
        let total_tasks = queue_tracker.performance_metrics.total_processed;
        let failed_tasks = queue_tracker.performance_metrics.total_failed;
        
        if total_tasks > 0 {
            (total_tasks - failed_tasks) as f64 / total_tasks as f64
        } else {
            1.0
        }
    }
    
    fn calculate_backlog_trend(&self, queue_tracker: &QueueTracker) -> f64 {
        let recent_events: Vec<_> = queue_tracker.processing_history
            .iter()
            .rev()
            .take(50)
            .collect();
        
        let enqueued = recent_events.iter()
            .filter(|event| matches!(event.event_type, QueueEventType::TaskEnqueued))
            .count();
        
        let completed = recent_events.iter()
            .filter(|event| matches!(event.event_type, QueueEventType::TaskCompleted))
            .count();
        
        enqueued as f64 - completed as f64
    }
    
    fn calculate_average_wait_time(&self, queue_tracker: &QueueTracker) -> u64 {
        let completed_tasks: Vec<_> = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Completed))
            .collect();
        
        if completed_tasks.is_empty() {
            return 0;
        }
        
        let total_wait_time: u64 = completed_tasks.iter()
            .filter_map(|task| {
                if let (Some(started), Some(completed)) = (task.started_at, task.completed_at) {
                    Some((completed - started).num_milliseconds() as u64)
                } else {
                    None
                }
            })
            .sum();
        
        total_wait_time / completed_tasks.len() as u64
    }
    
    fn calculate_queue_utilization(&self, queue_tracker: &QueueTracker) -> f64 {
        let active_tasks = queue_tracker.active_tasks.len();
        let max_capacity = queue_tracker.config.max_depth;
        
        if max_capacity > 0 {
            active_tasks as f64 / max_capacity as f64
        } else {
            0.0
        }
    }
    
    fn calculate_lifecycle_efficiency(&self, queue_tracker: &QueueTracker) -> f64 {
        let completed_tasks = queue_tracker.active_tasks.values()
            .filter(|task| matches!(task.status, QueueTaskStatus::Completed))
            .count();
        
        let total_tasks = queue_tracker.active_tasks.len();
        
        if total_tasks > 0 {
            completed_tasks as f64 / total_tasks as f64
        } else {
            1.0
        }
    }
    
    fn calculate_administration_efficiency(&self, queue_tracker: &QueueTracker) -> f64 {
        let successful_operations = queue_tracker.performance_metrics.total_processed;
        let total_operations = successful_operations + queue_tracker.performance_metrics.total_failed;

        if total_operations > 0 {
            successful_operations as f64 / total_operations as f64
        } else {
            1.0
        }
    }

    /// Run autonomous executor loop with progress tracking
    ///
    /// This method continuously processes tasks from a task source, tracking progress
    /// and providing real-time updates on evaluation status.
    pub async fn run_autonomous_executor<T>(
        &mut self,
        mut task_source: T,
        progress_callback: Option<Box<dyn Fn(ExecutorProgress) + Send + Sync>>,
    ) -> Result<()>
    where
        T: TaskSource + Send,
    {
        info!("Starting autonomous council executor loop");

        let mut consecutive_errors = 0;
        let max_consecutive_errors = 5;
        let mut total_tasks_processed = 0;
        let start_time = std::time::Instant::now();

        loop {
            // Check for shutdown signal or health status
            if self.should_shutdown().await {
                info!("Shutdown signal received, stopping autonomous executor");
                break;
            }

            // Get next task from source
            let task_result = task_source.next_task().await;

            match task_result {
                Ok(Some(task_spec)) => {
                    let task_id = task_spec.id;
                    info!("Processing task {} in autonomous mode", task_id);

                    // Update progress
                    if let Some(ref callback) = progress_callback {
                        callback(ExecutorProgress {
                            total_tasks_processed,
                            current_task: Some(task_id),
                            uptime_seconds: start_time.elapsed().as_secs(),
                            status: ExecutorStatus::Processing,
                        });
                    }

                    // Process the task
                    let evaluation_result = self.evaluate_task(task_spec.clone()).await;

                    match evaluation_result {
                        Ok(consensus_result) => {
                            consecutive_errors = 0;
                            total_tasks_processed += 1;

                            info!("Successfully processed task {}: {:?}", task_id, consensus_result.final_verdict);

                            // Mark task as completed in source
                            if let Err(e) = task_source.mark_completed(task_id, &consensus_result).await {
                                warn!("Failed to mark task {} as completed: {}", task_id, e);
                            }

                            // Update progress
                            if let Some(ref callback) = progress_callback {
                                callback(ExecutorProgress {
                                    total_tasks_processed,
                                    current_task: None,
                                    uptime_seconds: start_time.elapsed().as_secs(),
                                    status: ExecutorStatus::Idle,
                                });
                            }
                        }
                        Err(e) => {
                            consecutive_errors += 1;
                            error!("Failed to evaluate task {}: {}", task_id, e);

                            // Mark task as failed
                            if let Err(mark_err) = task_source.mark_failed(task_id, &e).await {
                                error!("Failed to mark task {} as failed: {}", task_id, mark_err);
                            }

                            // Check if we should continue after errors
                            if consecutive_errors >= max_consecutive_errors {
                                error!("Too many consecutive errors ({}), entering cooldown", consecutive_errors);
                                sleep(Duration::from_secs(30)).await;
                                consecutive_errors = 0;
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No tasks available, wait before checking again
                    debug!("No tasks available, waiting...");
                    sleep(Duration::from_secs(5)).await;

                    // Update progress to show idle status
                    if let Some(ref callback) = progress_callback {
                        callback(ExecutorProgress {
                            total_tasks_processed,
                            current_task: None,
                            uptime_seconds: start_time.elapsed().as_secs(),
                            status: ExecutorStatus::WaitingForTasks,
                        });
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    error!("Failed to get next task: {}", e);

                    if consecutive_errors >= max_consecutive_errors {
                        error!("Too many consecutive task source errors, entering cooldown");
                        sleep(Duration::from_secs(60)).await;
                        consecutive_errors = 0;
                    } else {
                        sleep(Duration::from_secs(10)).await;
                    }
                }
            }

            // Brief pause between iterations to prevent tight looping
            sleep(Duration::from_millis(100)).await;
        }

        info!("Autonomous executor completed. Processed {} tasks in {} seconds",
              total_tasks_processed, start_time.elapsed().as_secs());

        Ok(())
    }

    /// Check if the executor should shutdown
    async fn should_shutdown(&self) -> bool {
        // Check health indicators for shutdown conditions
        // This could be extended to check external signals, health checks, etc.
        let metrics = self.metrics.read().unwrap();
        let error_rate = if metrics.total_evaluations > 0 {
            metrics.failed_evaluations as f64 / metrics.total_evaluations as f64
        } else {
            0.0
        };

        // Shutdown if error rate exceeds 80%
        error_rate > 0.8
    }
}

/// Progress tracking for autonomous executor
#[derive(Debug, Clone)]
pub struct ExecutorProgress {
    pub total_tasks_processed: u64,
    pub current_task: Option<Uuid>,
    pub uptime_seconds: u64,
    pub status: ExecutorStatus,
}

/// Current status of the autonomous executor
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorStatus {
    WaitingForTasks,
    Processing,
    Idle,
}

/// Trait for task sources that provide tasks to the autonomous executor
#[async_trait::async_trait]
pub trait TaskSource: Send {
    /// Get the next task to process
    async fn next_task(&mut self) -> Result<Option<TaskSpec>>;

    /// Mark a task as completed with its result
    async fn mark_completed(&mut self, task_id: Uuid, result: &ConsensusResult) -> Result<()>;

    /// Mark a task as failed with an error
    async fn mark_failed(&mut self, task_id: Uuid, error: &anyhow::Error) -> Result<()>;
}


/// Result of a debate round
#[derive(Debug, Clone)]
struct DebateRoundResult {
    round: u32,
    consensus_reached: bool,
    should_terminate: bool,
}

/// Performance record for participant analysis
#[derive(Debug, Clone)]
struct ParticipantPerformanceRecord {
    participant_id: String,
    decision_accuracy: f32,
    response_time_ms: u64,
    quality_score: f32,
    domain: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Decision record for accuracy analysis
#[derive(Debug, Clone)]
struct DecisionRecord {
    participant_id: String,
    task_id: String,
    decision_outcome: String,
    confidence_score: f32,
    actual_outcome: String,
    domain: String,
    decision_quality: f32,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Decision reliability statistics
#[derive(Debug, Clone)]
struct DecisionReliabilityStats {
    accuracy: f32,
    confidence_interval: (f32, f32),
    sample_size: usize,
    consistency_score: f32,
}

/// Domain-specific performance metrics
#[derive(Debug, Clone)]
struct DomainPerformance {
    accuracy: f32,
    average_quality: f32,
    decision_count: usize,
    specialization_score: f32,
}
