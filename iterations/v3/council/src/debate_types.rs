//! Core types for the debate protocol system

use crate::council_types::{JudgeId, TaskId, VerdictId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Judge assignment decision for uncertain verdicts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JudgeAssignment {
    Supporting,
    Opposing,
    Neutral,
}

/// Research coordination result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCoordination {
    pub sources: Vec<ResearchSource>,
    pub findings: Vec<ResearchFinding>,
    pub coordination_quality: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Research integration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchIntegration {
    pub enhanced_arguments: HashMap<JudgeId, DebateArgument>,
    pub integrated_findings: Vec<ResearchFinding>,
    pub integration_quality: f32,
    pub validation_status: ValidationStatus,
}

/// Research validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchValidation {
    pub validated_findings: Vec<ResearchFinding>,
    pub validation_metrics: ValidationMetrics,
    pub overall_confidence: f32,
    pub validation_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Research source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSource {
    pub source_id: String,
    pub source_type: ResearchSourceType,
    pub url: Option<String>,
    pub title: String,
    pub credibility_score: f32,
    pub relevance_score: f32,
}

/// Types of research sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchSourceType {
    Academic,
    Industry,
    News,
    Documentation,
    ExpertOpinion,
    Experimental,
}

/// Research finding from coordinated research
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchFinding {
    pub finding_id: String,
    pub source_id: String,
    pub content: String,
    pub confidence_score: f32,
    pub supporting_evidence: Vec<String>,
    pub contradicting_evidence: Vec<String>,
    pub relevance_to_debate: f32,
}

/// Validation status for research findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    PartiallyValid,
    Invalid,
    NeedsFurtherResearch,
}

/// Validation metrics for research validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    pub consistency_score: f32,
    pub evidence_strength: f32,
    pub peer_review_score: f32,
    pub recency_score: f32,
}

/// Debate argument presented by a judge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateArgument {
    pub argument_id: String,
    pub judge_id: JudgeId,
    pub position: ArgumentPosition,
    pub content: String,
    pub strength_score: f32,
    pub evidence: Vec<EvidenceItem>,
    pub counter_arguments: Vec<String>,
    pub confidence_score: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Position taken in the debate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentPosition {
    Support,
    Oppose,
    Neutral,
    Clarification,
}

/// Evidence item supporting an argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_type: EvidenceType,
    pub content: String,
    pub source: String,
    pub credibility: f32,
    pub relevance: f32,
}

/// Types of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    Factual,
    Statistical,
    ExpertOpinion,
    Research,
    Historical,
    Logical,
}

/// Debate round containing arguments from all judges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRound {
    pub round_number: u32,
    pub arguments: HashMap<JudgeId, DebateArgument>,
    pub consensus_check: ConsensusCheck,
    pub research_integration: Option<ResearchIntegration>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Consensus check result for a debate round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusCheck {
    pub consensus_reached: bool,
    pub consensus_strength: f32,
    pub dissenting_judges: Vec<JudgeId>,
    pub convergence_metrics: ConvergenceMetrics,
}

/// Metrics measuring convergence in debate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceMetrics {
    pub position_stability: f32,
    pub argument_consistency: f32,
    pub evidence_overlap: f32,
    pub confidence_trend: ConfidenceTrend,
}

/// Confidence trend over debate rounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceTrend {
    Increasing,
    Stable,
    Decreasing,
    Fluctuating,
}

/// Debate session tracking the entire debate process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateSession {
    pub session_id: Uuid,
    pub task_id: TaskId,
    pub conflicting_judges: Vec<JudgeId>,
    pub rounds: Vec<DebateRound>,
    pub final_consensus: Option<DebateConsensus>,
    pub status: DebateStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Final consensus reached in debate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateConsensus {
    pub consensus_type: ConsensusType,
    pub winning_position: Option<ArgumentPosition>,
    pub confidence_score: f32,
    pub rationale: String,
    pub supporting_evidence: Vec<String>,
}

/// Types of consensus outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusType {
    Unanimous,
    Majority,
    Compromise,
    Deadlock,
    Timeout,
}

/// Status of a debate session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebateStatus {
    Active,
    ConsensusReached,
    Deadlock,
    Timeout,
    Cancelled,
}

/// Debate configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateConfig {
    pub max_rounds: u32,
    pub consensus_threshold: f32,
    pub timeout_seconds: u64,
    pub min_participants: usize,
    pub max_participants: usize,
    pub argument_quality_threshold: f32,
    pub research_enabled: bool,
    pub research_timeout_seconds: u64,
}

impl Default for DebateConfig {
    fn default() -> Self {
        Self {
            max_rounds: 5,
            consensus_threshold: 0.7,
            timeout_seconds: 300,
            min_participants: 2,
            max_participants: 10,
            argument_quality_threshold: 0.6,
            research_enabled: true,
            research_timeout_seconds: 60,
        }
    }
}

/// Debate protocol result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateResult {
    pub session: DebateSession,
    pub execution_time_ms: u64,
    pub total_rounds: u32,
    pub research_performed: bool,
    pub final_verdict: Option<super::types::JudgeVerdict>,
}

/// Argument generation trait
#[async_trait::async_trait]
pub trait ArgumentGenerator: Send + Sync {
    async fn generate_argument(
        &self,
        judge_id: &JudgeId,
        position: ArgumentPosition,
        round_number: u32,
        context: &DebateContext,
    ) -> Result<DebateArgument>;
}

/// Research agent trait for gathering external information
#[async_trait::async_trait]
pub trait ResearchAgent: Send + Sync {
    async fn coordinate_research(&self, query: &str, debate_context: &DebateContext) -> Result<ResearchCoordination>;
    async fn validate_research(&self, findings: &[ResearchFinding]) -> Result<ResearchValidation>;
}

/// Context information for debate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateContext {
    pub task_id: TaskId,
    pub task_description: String,
    pub current_round: u32,
    pub previous_arguments: Vec<DebateArgument>,
    pub research_findings: Vec<ResearchFinding>,
    pub time_remaining_seconds: u64,
}

/// Debate metrics for monitoring and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateMetrics {
    pub total_sessions: u64,
    pub successful_consensus_rate: f32,
    pub average_rounds_per_session: f32,
    pub average_session_duration_ms: u64,
    pub research_utilization_rate: f32,
    pub argument_quality_average: f32,
}

/// Debate protocol error types
#[derive(Debug, thiserror::Error)]
pub enum DebateError {
    #[error("Insufficient participants for debate: need {min}, got {actual}")]
    InsufficientParticipants { min: usize, actual: usize },

    #[error("Debate timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("Maximum rounds ({max}) exceeded")]
    MaxRoundsExceeded { max: u32 },

    #[error("Argument generation failed: {reason}")]
    ArgumentGenerationFailed { reason: String },

    #[error("Research coordination failed: {reason}")]
    ResearchFailed { reason: String },

    #[error("Invalid debate configuration: {field}")]
    InvalidConfiguration { field: String },
}

pub type DebateResultType<T> = Result<T, DebateError>;
