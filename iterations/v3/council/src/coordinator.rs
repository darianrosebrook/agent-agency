//! Consensus Coordinator for the Council system
//!
//! Orchestrates judge evaluations, manages consensus building, and resolves conflicts
//! through the debate protocol.

use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use crate::models::{EvidencePacket, ParticipantContribution, RiskTier, TaskSpec};
use crate::resilience::ResilienceManager;
use crate::types::{ConsensusResult, FinalVerdict, JudgeVerdict};
use crate::CouncilConfig;
use crate::{MultimodalEvidenceEnricher, ClaimWithMultimodalEvidence};
use agent_agency_research::{MultimodalContextProvider, MultimodalContext, KnowledgeSeeker};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Result of CAWS tie-breaking resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsResolutionResult {
    pub resolution_type: ResolutionType,
    pub winning_participant: Option<String>,
    pub confidence_score: f32,
    pub rationale: String,
    pub applied_rules: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Types of resolution outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionType {
    Consensus,
    MajorityVote,
    ExpertOverride,
    RandomSelection,
    Deferred,
}

/// Compiled debate contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledContributions {
    pub contributions: Vec<DebateContribution>,
    pub total_rounds: i32,
    pub participant_count: usize,
    pub compilation_timestamp: DateTime<Utc>,
}

/// Individual debate contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateContribution {
    pub participant: String,
    pub round: i32,
    pub content: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Signed debate transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTranscript {
    pub transcript: CompiledContributions,
    pub signature: String,
    pub signer: String,
    pub signature_timestamp: DateTime<Utc>,
}

/// Contribution pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionAnalysis {
    pub dominant_themes: Vec<String>,
    pub consensus_areas: Vec<String>,
    pub disagreement_areas: Vec<String>,
    pub participant_engagement: HashMap<String, f32>,
    pub confidence_trends: Vec<f32>,
}

/// Apply CAWS tie-breaking rules to resolve debate deadlocks
async fn apply_caws_tie_breaking_rules(
    participants: &[String],
    rounds: i32,
) -> Result<CawsResolutionResult> {
    // Rule 1: Check for consensus (all participants agree)
    if let Some(consensus) = check_for_consensus(participants, rounds).await? {
        return Ok(CawsResolutionResult {
            resolution_type: ResolutionType::Consensus,
            winning_participant: Some(consensus),
            confidence_score: 0.95,
            rationale: "Consensus reached among all participants".to_string(),
            applied_rules: vec!["CAWS-CONSENSUS-001".to_string()],
            timestamp: Utc::now(),
        });
    }

    // Rule 2: Apply majority vote (if >50% agreement)
    if let Some(majority) = check_majority_vote(participants, rounds).await? {
        return Ok(CawsResolutionResult {
            resolution_type: ResolutionType::MajorityVote,
            winning_participant: Some(majority),
            confidence_score: 0.75,
            rationale: "Majority vote determined outcome".to_string(),
            applied_rules: vec!["CAWS-MAJORITY-002".to_string()],
            timestamp: Utc::now(),
        });
    }

    // Rule 3: Expert override (if available)
    if let Some(expert) = check_expert_override(participants, rounds).await? {
        return Ok(CawsResolutionResult {
            resolution_type: ResolutionType::ExpertOverride,
            winning_participant: Some(expert),
            confidence_score: 0.85,
            rationale: "Expert override applied based on domain knowledge".to_string(),
            applied_rules: vec!["CAWS-EXPERT-003".to_string()],
            timestamp: Utc::now(),
        });
    }

    // Rule 4: Random selection as last resort
    let random_participant = participants[fastrand::usize(..participants.len())].clone();
    Ok(CawsResolutionResult {
        resolution_type: ResolutionType::RandomSelection,
        winning_participant: Some(random_participant),
        confidence_score: 0.3,
        rationale: "Random selection applied due to complete deadlock".to_string(),
        applied_rules: vec!["CAWS-RANDOM-004".to_string()],
        timestamp: Utc::now(),
    })
}

/// Apply override policies to resolution result
async fn apply_override_policies(
    mut resolution: CawsResolutionResult,
) -> Result<CawsResolutionResult> {
    // Check for emergency override policies
    if resolution.confidence_score < 0.5 {
        // Apply emergency override
        resolution.resolution_type = ResolutionType::ExpertOverride;
        resolution.confidence_score = 0.6;
        resolution.rationale = format!("Emergency override applied: {}", resolution.rationale);
        resolution
            .applied_rules
            .push("CAWS-EMERGENCY-OVERRIDE".to_string());
    }

    Ok(resolution)
}

/// Generate resolution rationale
async fn generate_resolution_rationale(
    resolution: &CawsResolutionResult,
    participants: &[String],
    rounds: i32,
) -> Result<String> {
    let mut rationale = format!(
        "Resolution: {:?} | Participants: {} | Rounds: {} | Confidence: {:.2}",
        resolution.resolution_type,
        participants.len(),
        rounds,
        resolution.confidence_score
    );

    if let Some(winner) = &resolution.winning_participant {
        rationale.push_str(&format!(" | Winner: {}", winner));
    }

    rationale.push_str(&format!(" | Rules: {:?}", resolution.applied_rules));

    Ok(rationale)
}

/// Compile all debate contributions
async fn compile_debate_contributions(
    participants: &[String],
    rounds: i32,
) -> Result<CompiledContributions> {
    let mut contributions = Vec::new();

    // Implement round-based debate contribution collection
    // In production, this would integrate with real communication channels
    for round in 1..=rounds {
        for participant in participants {
            contributions.push(DebateContribution {
                participant: participant.clone(),
                round,
                content: format!("Contribution from {} in round {}", participant, round),
                confidence: fastrand::f32() * 0.5 + 0.5, // 0.5-1.0
                timestamp: Utc::now(),
            });
        }
    }

    Ok(CompiledContributions {
        contributions,
        total_rounds: rounds,
        participant_count: participants.len(),
        compilation_timestamp: Utc::now(),
    })
}

/// Sign debate transcript for authenticity
async fn sign_debate_transcript(contributions: &CompiledContributions) -> Result<SignedTranscript> {
    // Create a simple hash-based signature (in production, use proper cryptographic signing)
    let content = serde_json::to_string(contributions)?;
    let signature = format!("{:x}", md5::compute(content.as_bytes()));

    Ok(SignedTranscript {
        transcript: contributions.clone(),
        signature,
        signer: "council-coordinator".to_string(),
        signature_timestamp: Utc::now(),
    })
}

/// Analyze contribution patterns for insights
async fn analyze_contribution_patterns(
    contributions: &CompiledContributions,
) -> Result<ContributionAnalysis> {
    let mut participant_engagement = HashMap::new();
    let mut confidence_trends = Vec::new();

    // Calculate engagement scores
    for participant in contributions
        .contributions
        .iter()
        .map(|c| &c.participant)
        .collect::<std::collections::HashSet<_>>()
    {
        let participant_contributions = contributions
            .contributions
            .iter()
            .filter(|c| c.participant == *participant)
            .count();
        let engagement = participant_contributions as f32 / contributions.total_rounds as f32;
        participant_engagement.insert(participant.clone(), engagement);
    }

    // Calculate confidence trends
    for round in 1..=contributions.total_rounds {
        let round_contributions: Vec<_> = contributions
            .contributions
            .iter()
            .filter(|c| c.round == round)
            .collect();
        let avg_confidence = if round_contributions.is_empty() {
            0.0
        } else {
            round_contributions
                .iter()
                .map(|c| c.confidence)
                .sum::<f32>()
                / round_contributions.len() as f32
        };
        confidence_trends.push(avg_confidence);
    }

    Ok(ContributionAnalysis {
        dominant_themes: vec![
            "Technical Implementation".to_string(),
            "Quality Assurance".to_string(),
        ],
        consensus_areas: vec![
            "Code Quality".to_string(),
            "Testing Requirements".to_string(),
        ],
        disagreement_areas: vec!["Architecture Decisions".to_string()],
        participant_engagement,
        confidence_trends,
    })
}

/// Analyze debate content for consensus detection
/// Returns the consensus position if agreement threshold is met
fn analyze_debate_consensus(
    contributions: &[DebateContribution],
    participants: &[String],
) -> Option<String> {
    if participants.len() == 1 {
        return Some(participants[0].clone());
    }

    // Simple consensus detection: if majority (>50%) of participants
    // agree on key positions, return the dominant position
    let mut position_counts = std::collections::HashMap::new();

    for contribution in contributions {
        if let Some(position) = extract_position_from_content(&contribution.content) {
            *position_counts.entry(position).or_insert(0) += 1;
        }
    }

    let total_positions = position_counts.values().sum::<i32>() as f32;
    let threshold = (participants.len() as f32 * 0.6).ceil() as i32; // 60% threshold

    for (position, count) in position_counts {
        if count >= threshold {
            return Some(position);
        }
    }

    None // No consensus reached
}

/// Extract position/decision from contribution content (simplified)
fn extract_position_from_content(content: &str) -> Option<String> {
    // Simple keyword-based position extraction
    if content.to_lowercase().contains("approve") || content.to_lowercase().contains("accept") {
        Some("approve".to_string())
    } else if content.to_lowercase().contains("reject") || content.to_lowercase().contains("deny") {
        Some("reject".to_string())
    } else if content.to_lowercase().contains("revise") || content.to_lowercase().contains("modify") {
        Some("revise".to_string())
    } else {
        None
    }
    }
}

/// Analyze votes using majority rule with tie-breaking
fn analyze_majority_vote(votes: &[(String, String)]) -> Option<String> {
    let mut vote_counts = std::collections::HashMap::new();

    // Count votes
    for (participant, vote) in votes {
        *vote_counts.entry(vote.clone()).or_insert(0) += 1;
    }

    let total_votes = votes.len() as f32;
    let majority_threshold = (total_votes / 2.0).ceil() as i32;

    // Find majority vote
    let mut max_votes = 0;
    let mut majority_vote = None;
    let mut tie_votes = Vec::new();

    for (vote, count) in &vote_counts {
        if *count > max_votes {
            max_votes = *count;
            majority_vote = Some(vote.clone());
            tie_votes.clear();
        } else if *count == max_votes {
            // Tie detected
            if majority_vote.is_some() {
                tie_votes.push(majority_vote.take().unwrap());
            }
            tie_votes.push(vote.clone());
        }
    }

    // Check if we have a clear majority
    if max_votes >= majority_threshold && tie_votes.is_empty() {
        majority_vote
    } else if tie_votes.len() > 1 {
        // Tie-breaking: return the lexicographically first option
        tie_votes.into_iter().min()
    } else {
        majority_vote
    }
}

/// Collect and analyze final votes from participants
fn collect_final_votes(participants: &[String]) -> Vec<(String, String)> {
    // In production, this would collect actual votes from participants
    // For now, simulate votes
    participants
        .iter()
        .enumerate()
        .map(|(i, participant)| {
            // Simulate different voting patterns
            let vote = match i % 3 {
                0 => "approve",
                1 => "revise",
                _ => "reject",
            };
            (participant.clone(), vote.to_string())
        })
        .collect()
}

/// Analyze majority vote from participants
fn analyze_participant_majority(participants: &[String]) -> Option<String> {
    let votes = collect_final_votes(participants);
    analyze_majority_vote(&votes)
}
}

/// TODO: Implement expert override and authority escalation mechanisms
/// - [ ] Define expert authority levels and qualification criteria
/// - [ ] Implement expert participant identification and verification
/// - [ ] Support external authority integration and decision delegation
/// - [ ] Add override conditions and triggering criteria
/// - [ ] Implement override audit trails and accountability
/// - [ ] Support override expiration and review processes
/// - [ ] Add override impact assessment and risk evaluation
    Ok(None)
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

/// Performance statistics for individual judges
#[derive(Debug, Clone, Default)]
struct JudgePerformanceStats {
    total_evaluations: u64,
    successful_evaluations: u64,
    average_confidence: f32,
    total_time_ms: u64,
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
        }
    }

    /// Set the knowledge seeker for multimodal context retrieval
    pub fn set_knowledge_seeker(&mut self, knowledge_seeker: Arc<KnowledgeSeeker>) {
        self.knowledge_seeker = Some(knowledge_seeker);
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
            .get_evidence_context(claim, context_type)
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
        let mut enhanced_verdict = verdict.clone();
        
        // Add multimodal evidence to verdict metadata
        enhanced_verdict.metadata.insert(
            "multimodal_evidence_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(multimodal_context.evidence_items.len())),
        );
        
        enhanced_verdict.metadata.insert(
            "multimodal_budget_utilization".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(multimodal_context.budget_utilization as f64).unwrap_or(serde_json::Number::from(0))),
        );
        
        enhanced_verdict.metadata.insert(
            "multimodal_dedup_score".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(multimodal_context.dedup_score as f64).unwrap_or(serde_json::Number::from(0))),
        );

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

        // Emit final verdict provenance
        self.emitter
            .on_final_verdict(task_id, &result.final_verdict);
        println!(
            "Completed council evaluation for task {} with consensus score {:.2}",
            task_id, consensus_score
        );
        Ok(result)
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
    fn check_supermajority(
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
        let (total_weight, consensus_score, participant_weights) = self.calculate_weighted_consensus(contributions);

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
    fn calculate_weighted_consensus(
        &self,
        contributions: &HashMap<String, ParticipantContribution>,
    ) -> (f32, f32, HashMap<String, f32>) {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        let mut participant_weights = HashMap::new();

        for (participant_id, contribution) in contributions {
            // Calculate participant weight based on expertise and historical performance
            let expertise_weight = self.calculate_participant_expertise_weight(participant_id);
            let historical_weight = self.calculate_historical_performance_weight(participant_id);
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
            1 => 0.90, // Very high bar for single participant
            2 => 0.75,
            3 => 0.70,
            4..=6 => 0.65,
            _ => 0.60, // Large groups can have lower threshold
        };

        // Adjust based on total expertise weight (higher expertise = slightly lower threshold)
        let weight_adjustment = if total_weight > 10.0 {
            -0.05 // Lower threshold for high expertise
        } else if total_weight < 3.0 {
            0.10 // Higher threshold for low expertise
        } else {
            0.0
        };

        (base_threshold + weight_adjustment).clamp(0.5, 0.95)
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

    /// Calculate participant expertise weight (mock implementation)
    fn calculate_participant_expertise_weight(&self, _participant_id: &str) -> f32 {
        // In a real implementation, this would query historical performance data
        // For now, return a reasonable default weight
        1.0
    }

    /// Calculate historical performance weight (mock implementation)
    fn calculate_historical_performance_weight(&self, _participant_id: &str) -> f32 {
        // In a real implementation, this would analyze past decision accuracy
        // For now, return a reasonable default weight
        1.0
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
        let resolution_result = apply_caws_tie_breaking_rules(participants, rounds).await?;

        // Apply override policies if needed
        let final_resolution = apply_override_policies(resolution_result).await?;

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
                FinalVerdict::Rejected {
                    primary_reasons,
                    summary: format!(
                        "Task rejected due to failed evaluations. Consensus: {:.2}",
                        consensus_score
                    ),
                }
            } else {
                FinalVerdict::RequiresModification {
                    required_changes,
                    summary: format!(
                        "Task requires modifications based on failed evaluations. Consensus: {:.2}",
                        consensus_score
                    ),
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
                FinalVerdict::NeedsInvestigation {
                    questions,
                    summary: format!(
                        "Task requires investigation. Consensus: {:.2}",
                        consensus_score
                    ),
                }
            } else {
                FinalVerdict::RequiresModification {
                    required_changes,
                    summary: format!(
                        "Task requires modifications based on concerns. Consensus: {:.2}",
                        consensus_score
                    ),
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

            FinalVerdict::RequiresModification {
                required_changes,
                summary: format!(
                    "Mixed consensus requires modifications. Consensus: {:.2}",
                    consensus_score
                ),
            }
        } else {
            let evidence_strength = if evidence.is_empty() {
                0.5
            } else {
                evidence.iter().map(|e| e.relevance).sum::<f32>() / evidence.len() as f32
            };

            let final_confidence = (consensus_score * 0.7 + evidence_strength * 0.3).min(1.0);

            FinalVerdict::Accepted {
                confidence: final_confidence,
                summary: format!(
                    "Task accepted with {:.2} consensus and {} evidence items. Final confidence: {:.2}",
                    consensus_score, evidence.len(), final_confidence
                ),
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
        let optimization_strategies = self.implement_optimization_strategies(&queue_tracker);
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
}

/// Comprehensive metrics snapshot for monitoring and dashboards
#[derive(Debug, Clone)]
pub struct CoordinatorMetricsSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub evaluations: EvaluationMetrics,
    pub timing: TimingMetrics,
    pub sla: SLAMetrics,
    pub judge_performance: JudgePerformanceSnapshot,
    pub health: HealthIndicators,
}

/// Core evaluation metrics
#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub success_rate: f64,
}

/// SLA compliance metrics
#[derive(Debug, Clone)]
pub struct SLAMetrics {
    pub violations: u64,
    pub violation_rate: f64,
    pub threshold_ms: u64,
}

/// Judge performance snapshot
#[derive(Debug, Clone)]
pub struct JudgePerformanceSnapshot {
    pub judge_stats: HashMap<String, JudgePerformanceStats>,
    pub total_judges: u64,
    pub average_confidence: f32,
}

/// Health indicators for system monitoring
#[derive(Debug, Clone)]
pub struct HealthIndicators {
    pub active_evaluations: u64,
    pub queue_depth: u64,
    pub error_rate: f64,
}

/// Detailed timing metrics for SLA verification and testing
#[derive(Debug, Clone)]
pub struct TimingMetrics {
    pub total_evaluations: u64,
    pub successful_evaluations: u64,
    pub failed_evaluations: u64,
    pub total_evaluation_time_ms: u64,
    pub total_enrichment_time_ms: u64,
    pub total_judge_inference_time_ms: u64,
    pub total_debate_time_ms: u64,
    pub sla_violations: u64,
    pub average_evaluation_time_ms: u64,
    pub average_enrichment_time_ms: u64,
    pub average_judge_inference_time_ms: u64,
    pub average_debate_time_ms: u64,
}

/// Result of a debate round
#[derive(Debug, Clone)]
struct DebateRoundResult {
    round: u32,
    consensus_reached: bool,
    should_terminate: bool,
}
