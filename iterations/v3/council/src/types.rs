//! Core types for the Council system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a task evaluation session
pub type TaskId = Uuid;
pub type JudgeId = String;
pub type VerdictId = Uuid;

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
    Critical,    // Blocking violation
    Major,       // Significant issue
    Minor,       // Minor issue
    Warning,     // Best practice violation
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Automated,  // Tests, lints, etc.
    Manual,     // Human review
    Hybrid,     // Automated + manual
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
    pub content: String,
    pub files_modified: Vec<FileModification>,
    pub rationale: String,
    pub self_assessment: SelfAssessment,
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
pub enum FinalVerdict {
    Accepted {
        confidence: f32,
        summary: String,
    },
    Rejected {
        primary_reasons: Vec<String>,
        summary: String,
    },
    RequiresModification {
        required_changes: Vec<RequiredChange>,
        summary: String,
    },
    NeedsInvestigation {
        questions: Vec<String>,
        summary: String,
    },
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
    Support,    // Supporting the verdict
    Oppose,     // Opposing the verdict
    Neutral,    // Neutral or seeking clarification
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
