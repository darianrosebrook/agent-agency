//! Core types for the Council system

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a task evaluation session
pub type TaskId = Uuid;
pub type JudgeId = String;
pub type VerdictId = Uuid;

/// Task type for evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    CodeReview,
    TestExecution,
    Build,
}

/// Judge specialization score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializationScore {
    pub domain: String,
    pub score: f32,
    pub confidence: f32,
}

/// Historical judge performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalJudgeData {
    pub judge_id: String,
    pub task_type: String,
    pub accuracy_score: f32,
    pub consistency_score: f32,
    pub speed_score: f32,
    pub total_tasks: u32,
    pub recent_trend: String,
}

/// Risk tier for CAWS compliance evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTier {
    Tier1, // Critical - Auth, billing, migrations
    Tier2, // Standard - Features, APIs, data writes
    Tier3, // Low risk - UI, internal tools
}

/// Judge evaluation verdict
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JudgeVerdict {
    Pass {
        confidence: f32,
        reasoning: String,
        evidence: Vec<Evidence>,
    },
    Fail {
        violations: Vec<Violation>,
        reasoning: String,
        evidence: Vec<Evidence>,
    },
    Uncertain {
        concerns: Vec<Concern>,
        reasoning: String,
        evidence: Vec<Evidence>,
        recommendation: Recommendation,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical, // Blocking violation
    Major,    // Significant issue
    Minor,    // Minor issue
    Warning,  // Best practice violation
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concern {
    pub area: String,
    pub description: String,
    pub impact: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Recommendation {
    Accept,      // Accept despite concerns
    Reject,      // Reject due to concerns
    Modify,      // Request modifications
    Investigate, // Need more information
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub source: EvidenceSource,
    pub content: String,
    pub relevance: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceSource {
    CodeAnalysis,
    TestResults,
    Documentation,
    CAWSRules,
    HistoricalData,
    ExpertKnowledge,
    ResearchAgent,
}

/// Complete evaluation result from a judge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeEvaluation {
    pub judge_id: JudgeId,
    pub task_id: TaskId,
    pub verdict: JudgeVerdict,
    pub evaluation_time_ms: u64,
    pub tokens_used: Option<u32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Task specification for council evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub risk_tier: RiskTier,
    pub scope: TaskScope,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub context: TaskContext,
    pub worker_output: WorkerOutput,
    pub caws_spec: Option<CawsSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    pub files_affected: Vec<String>,
    pub max_files: Option<u32>,
    pub max_loc: Option<u32>,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
    pub verification_method: VerificationMethod,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    Automated, // Tests, lints, etc.
    Manual,    // Human review
    Hybrid,    // Automated + manual
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub workspace_root: String,
    pub git_branch: String,
    pub recent_changes: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerOutput {
    pub worker_id: Uuid,
    pub task_id: TaskId,
    pub content: String,
    pub files_modified: Vec<FileModification>,
    pub rationale: String,
    pub self_assessment: SelfAssessment,
    pub response_time_ms: Option<u64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModification {
    pub path: String,
    pub operation: FileOperation,
    pub content: Option<String>,
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    Create,
    Modify,
    Delete,
    Move { from: String, to: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAssessment {
    pub caws_compliance: f32,
    pub quality_score: f32,
    pub confidence: f32,
    pub concerns: Vec<String>,
    pub improvements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsSpec {
    pub working_spec_path: String,
    pub risk_tier: RiskTier,
    pub budgets: CawsBudgets,
    pub waivers: Vec<CawsWaiver>,
    pub quality_gates: Vec<QualityGate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsBudgets {
    pub max_files: u32,
    pub max_loc: u32,
    pub max_time_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsWaiver {
    pub id: String,
    pub reason: String,
    pub justification: String,
    pub time_bounded: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub name: String,
    pub required: bool,
    pub threshold: Option<f32>,
    pub description: String,
}

/// Consensus result from the council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub task_id: TaskId,
    pub verdict_id: VerdictId,
    pub final_verdict: FinalVerdict,
    pub individual_verdicts: HashMap<JudgeId, JudgeVerdict>,
    pub consensus_score: f32,
    pub debate_rounds: u32,
    pub evaluation_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalVerdict {
    pub decision: String,
    pub confidence: f32,
    pub summary: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredChange {
    pub priority: Priority,
    pub description: String,
    pub rationale: String,
    pub estimated_effort: Option<String>,
}

/// Debate session for resolving conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateSession {
    pub session_id: Uuid,
    pub task_id: TaskId,
    pub conflicting_judges: Vec<JudgeId>,
    pub rounds: Vec<DebateRound>,
    pub final_consensus: Option<ConsensusResult>,
    pub status: DebateStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebateStatus {
    Active,
    Resolved,
    Timeout,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRound {
    pub round_number: u32,
    pub arguments: HashMap<JudgeId, DebateArgument>,
    pub evidence_requests: Vec<EvidenceRequest>,
    pub research_input: Option<ResearchInput>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateArgument {
    pub judge_id: JudgeId,
    pub position: ArgumentPosition,
    pub reasoning: String,
    pub evidence_cited: Vec<Evidence>,
    pub counter_arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentPosition {
    Support, // Supporting the verdict
    Oppose,  // Opposing the verdict
    Neutral, // Neutral or seeking clarification
}

impl std::fmt::Display for ArgumentPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgumentPosition::Support => write!(f, "Support"),
            ArgumentPosition::Oppose => write!(f, "Oppose"),
            ArgumentPosition::Neutral => write!(f, "Neutral"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequest {
    pub requesting_judge: JudgeId,
    pub requested_from: EvidenceSource,
    pub question: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchInput {
    pub research_agent_id: String,
    pub findings: Vec<ResearchFinding>,
    pub confidence: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchFinding {
    pub topic: String,
    pub finding: String,
    pub relevance: f32,
    pub sources: Vec<String>,
}

/// Performance metrics for council operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_evaluations: u64,
    pub consensus_rate: f32,
    pub average_evaluation_time_ms: f64,
    pub judge_performance: HashMap<JudgeId, JudgeMetrics>,
    pub debate_sessions: u64,
    pub debate_resolution_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeMetrics {
    pub total_evaluations: u64,
    pub average_time_ms: f64,
    pub accuracy_rate: f32,
    pub reliability_score: f32,
    pub last_evaluation: Option<chrono::DateTime<chrono::Utc>>,
}

impl JudgeVerdict {
    /// Get the confidence score for this verdict
    pub fn confidence(&self) -> f32 {
        match self {
            JudgeVerdict::Pass { confidence, .. } => *confidence,
            JudgeVerdict::Fail { .. } => 1.0, // Fail verdicts are always confident
            JudgeVerdict::Uncertain { .. } => 0.5, // Uncertain verdicts have medium confidence
        }
    }

    /// Check if this verdict indicates acceptance
    pub fn is_accepting(&self) -> bool {
        matches!(self, JudgeVerdict::Pass { .. })
    }

    /// Get the primary reasoning for this verdict
    pub fn reasoning(&self) -> &str {
        match self {
            JudgeVerdict::Pass { reasoning, .. } => reasoning,
            JudgeVerdict::Fail { reasoning, .. } => reasoning,
            JudgeVerdict::Uncertain { reasoning, .. } => reasoning,
        }
    }
}

// ============================================================================
// Claim Extraction and Verification Types (V2 Integration)
// ============================================================================

/// Conversation context for claim extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub conversation_id: String,
    pub tenant_id: String,
    pub previous_messages: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Ambiguity analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityAnalysis {
    pub referential_ambiguities: Vec<String>,
    pub structural_ambiguities: Vec<String>,
    pub temporal_ambiguities: Vec<String>,
    pub resolution_confidence: f64,
}

/// Disambiguation result from stage 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationResult {
    pub original_text: String,
    pub resolved_text: String,
    pub resolved_ambiguities: Vec<ResolutionAttempt>,
    pub unresolved_ambiguities: Vec<UnresolvableAmbiguity>,
    pub resolution_confidence: f64,
    pub timestamp: String,
}

/// Resolution attempt for an ambiguity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionAttempt {
    pub ambiguity_type: String,
    pub original_text: String,
    pub resolved_text: String,
    pub confidence: f64,
    pub resolution_method: String,
    pub timestamp: String,
}

/// Unresolvable ambiguity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvableAmbiguity {
    pub ambiguity_type: String,
    pub ambiguous_text: String,
    pub reason: String,
    pub timestamp: String,
}

/// Verifiable content result from stage 2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableContentResult {
    pub verifiable_segments: Vec<String>,
    pub non_verifiable_segments: Vec<String>,
    pub qualification_confidence: f64,
    pub timestamp: String,
}

/// Atomic claim from stage 3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicClaim {
    pub id: String,
    pub statement: String,
    pub confidence: f64,
    pub source_context: String,
    pub verification_requirements: Vec<VerificationCriteria>,
}

/// Verification criteria for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCriteria {
    pub criterion_type: String,
    pub description: String,
    pub required_evidence: Vec<String>,
    pub priority: Priority,
}

/// Extracted claim with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedClaim {
    pub id: String,
    pub statement: String,
    pub confidence: f64,
    pub source_context: String,
    pub verification_requirements: Vec<VerificationCriteria>,
}

/// Evidence manifest for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceManifest {
    pub evidence: Vec<EvidenceItem>,
    pub quality: Option<f64>,
    pub caws_compliant: bool,
    pub timestamp: String,
}

/// Evidence item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub source: String,
    pub content: String,
    pub relevance: f64,
    pub credibility: f64,
}

/// Verification result from stage 4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub status: VerificationStatus,
    pub evidence_quality: f64,
    pub caws_compliance: bool,
    pub verification_trail: Vec<VerificationStep>,
}

/// Verification status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    InsufficientEvidence,
}

/// Verification step in the trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    pub step_type: String,
    pub description: String,
    pub outcome: String,
    pub timestamp: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Scope validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeValidation {
    pub within_scope: bool,
    pub violations: Vec<String>,
    pub waiver_required: bool,
    pub waiver_justification: Option<String>,
}

/// Working spec for CAWS compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    pub id: String,
    pub title: String,
    pub scope: Option<WorkingSpecScope>,
    pub risk_tier: RiskTier,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

/// Working spec scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpecScope {
    pub r#in: Option<Vec<String>>,
    pub out: Option<Vec<String>>,
}

/// Claim-based evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimBasedEvaluation {
    pub id: String,
    pub timestamp: String,
    pub disambiguation_result: DisambiguationResult,
    pub qualification_result: VerifiableContentResult,
    pub decomposition_result: ClaimDecompositionResult,
    pub verification_results: Vec<VerificationResult>,
    pub overall_confidence: f64,
    pub caws_compliance: bool,
}

/// Claim decomposition result from stage 3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimDecompositionResult {
    pub atomic_claims: Vec<AtomicClaim>,
    pub decomposition_confidence: f64,
    pub timestamp: String,
}

/// Learning update for pattern improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningUpdate {
    pub pattern_id: String,
    pub update_type: String,
    pub success: bool,
    pub feedback: String,
    pub timestamp: String,
}

/// Pattern update for learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUpdate {
    pub pattern_id: String,
    pub pattern_type: String,
    pub pattern_data: serde_json::Value,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_updated: String,
}

/// Arbitration decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationDecision {
    pub decision_id: String,
    pub claim_id: String,
    pub decision: String,
    pub confidence: f64,
    pub reasoning: String,
    pub timestamp: String,
}

// ============================================================================
// Trait Definitions for Claim Extraction System
// ============================================================================

use async_trait::async_trait;

/// Main claim extraction and verification processor trait
#[async_trait]
pub trait ClaimExtractionAndVerificationProcessor {
    async fn process_claim_extraction_and_verification(
        &self,
        worker_output: serde_json::Value,
        task_context: serde_json::Value,
        conversation_context: Option<ConversationContext>,
    ) -> Result<ClaimBasedEvaluation, Box<dyn std::error::Error + Send + Sync>>;
}

/// Ambiguity handler trait
#[async_trait]
pub trait AmbiguityHandler {
    async fn disambiguation_stage(
        &self,
        text: &str,
        context: &ConversationContext,
    ) -> Result<DisambiguationResult, Box<dyn std::error::Error + Send + Sync>>;

    async fn resolve_referential_ambiguities(
        &self,
        text: &str,
        patterns: &[regex::Regex],
        context: &ConversationContext,
    ) -> (Vec<ResolutionAttempt>, Vec<UnresolvableAmbiguity>, String);

    async fn resolve_structural_ambiguities(
        &self,
        text: &str,
        patterns: &[regex::Regex],
        context: &ConversationContext,
    ) -> (Vec<ResolutionAttempt>, Vec<UnresolvableAmbiguity>, String);

    async fn resolve_temporal_ambiguities(
        &self,
        text: &str,
        patterns: &[regex::Regex],
        context: &ConversationContext,
    ) -> (Vec<ResolutionAttempt>, Vec<UnresolvableAmbiguity>, String);
}

/// Claim-based arbiter trait
#[async_trait]
pub trait ClaimBasedArbiter {
    async fn arbitrate_claims(
        &self,
        claims: Vec<AtomicClaim>,
        context: &ConversationContext,
    ) -> Result<Vec<ArbitrationDecision>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Claim learning system trait
#[async_trait]
pub trait ClaimLearningSystem {
    async fn update_patterns(
        &self,
        update: LearningUpdate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn get_pattern_performance(
        &self,
        pattern_id: &str,
    ) -> Result<PatternUpdate, Box<dyn std::error::Error + Send + Sync>>;
}

/// Trend type for resource usage analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendType {
    Increasing,
    Decreasing,
    Stable,
}

/// Resource usage trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTrend {
    pub resource_type: String,
    pub trend: TrendType,
    pub slope: f32,
    pub confidence: f32,
    pub time_window: i64,
}

/// Internal trend analysis structure
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub trend_type: TrendType,
    pub slope: f32,
    pub confidence: f32,
    pub time_window: i64,
}

/// Resource usage prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePrediction {
    pub timestamp: i64,
    pub horizon_seconds: i64,
    pub predicted_usage: ResourceUsageMetrics,
    pub confidence: f32,
}

/// Resource usage metrics for learning and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub thermal_status: crate::learning::ThermalStatus,
    pub ane_utilization: Option<f32>,
    pub gpu_utilization: Option<f32>,
    pub energy_consumption: Option<f32>, // Joules
    // Additional fields expected by the code
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub io_bytes_per_sec: u64,
    pub network_bytes_per_sec: u64,
}

impl std::fmt::Display for RiskTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskTier::Tier1 => write!(f, "Tier1"),
            RiskTier::Tier2 => write!(f, "Tier2"),
            RiskTier::Tier3 => write!(f, "Tier3"),
        }
    }
}

impl TaskSpec {
    /// Check if this task requires unanimous approval
    pub fn requires_unanimous_approval(&self) -> bool {
        self.risk_tier == RiskTier::Tier1
    }

    /// Get the consensus threshold for this task
    pub fn consensus_threshold(&self) -> f32 {
        match self.risk_tier {
            RiskTier::Tier1 => 0.8, // 80% supermajority
            RiskTier::Tier2 => 0.6, // 60% majority
            RiskTier::Tier3 => 0.5, // 50% simple majority
        }
    }
}
