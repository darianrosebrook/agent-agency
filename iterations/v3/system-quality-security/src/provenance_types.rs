//! Types for provenance service

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Provenance record for a CAWS verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub id: Uuid,
    pub verdict_id: Uuid,
    pub task_id: Uuid,
    pub decision: VerdictDecision,
    pub consensus_score: f32,
    pub judge_verdicts: HashMap<String, JudgeVerdictProvenance>,
    pub caws_compliance: CawsComplianceProvenance,
    pub claim_verification: Option<ClaimVerificationProvenance>,
    pub git_commit_hash: Option<String>,
    pub git_trailer: String,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Verdict decision types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerdictDecision {
    Accept {
        confidence: f32,
        summary: String,
    },
    Reject {
        primary_reasons: Vec<String>,
        summary: String,
    },
    RequireModification {
        required_changes: Vec<RequiredChange>,
        summary: String,
    },
    NeedInvestigation {
        questions: Vec<String>,
        summary: String,
    },
}

/// Required changes for modification verdicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredChange {
    pub priority: Priority,
    pub description: String,
    pub rationale: String,
    pub estimated_effort: Option<String>,
}

/// Priority levels for changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Judge verdict provenance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeVerdictProvenance {
    pub judge_id: String,
    pub verdict: String,
    pub confidence: f32,
    pub reasoning: String,
    pub evidence_count: u32,
    pub evaluation_time_ms: u64,
    pub model_version: String,
    pub optimization_target: String,
}

/// CAWS compliance provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsComplianceProvenance {
    pub is_compliant: bool,
    pub compliance_score: f32,
    pub violations: Vec<ViolationProvenance>,
    pub waivers_used: Vec<WaiverProvenance>,
    pub budget_adherence: BudgetAdherence,
}

/// Violation provenance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationProvenance {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub location: Option<String>,
    pub constitutional_ref: Option<String>,
    pub suggestion: Option<String>,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ViolationSeverity {
    Critical,
    Major,
    Minor,
    Warning,
}

/// Waiver provenance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverProvenance {
    pub id: String,
    pub reason: String,
    pub justification: String,
    pub time_bounded: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
}

/// Budget adherence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAdherence {
    pub max_files: u32,
    pub actual_files: u32,
    pub max_loc: u32,
    pub actual_loc: u32,
    pub max_time_minutes: Option<u32>,
    pub actual_time_minutes: Option<u32>,
    pub within_budget: bool,
}

/// Claim verification provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimVerificationProvenance {
    pub claims_total: u32,
    pub claims_verified: u32,
    pub claims_unverified: u32,
    pub verification_quality: f32,
    pub evidence_items: Vec<EvidenceItemProvenance>,
    pub ambiguities_resolved: u32,
    pub extraction_time_ms: u64,
    pub verification_time_ms: u64,
}

/// Evidence item provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItemProvenance {
    pub id: Uuid,
    pub claim_id: Option<String>,
    pub source: EvidenceSource,
    pub quality_score: f32,
    pub verification_status: VerificationStatus,
    pub collection_time_ms: u64,
}

/// Evidence source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    WebSearch,
    CodeAnalysis,
    Documentation,
    VectorDatabase,
    ExpertKnowledge,
    HistoricalData,
    ResearchAgent,
}

/// Verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    InsufficientEvidence,
    Pending,
    Failed,
}

/// Provenance query filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceQuery {
    pub task_id: Option<Uuid>,
    pub verdict_id: Option<Uuid>,
    pub decision_type: Option<VerdictDecisionType>,
    pub time_range: Option<TimeRange>,
    pub judge_id: Option<String>,
    pub compliance_status: Option<ComplianceStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    // Additional filter fields
    pub entity_types: Option<Vec<String>>,
    pub entity_ids: Option<Vec<Uuid>>,
    pub activity_types: Option<Vec<String>>,
    pub activity_ids: Option<Vec<Uuid>>,
    pub agent_types: Option<Vec<String>>,
    pub agent_ids: Option<Vec<String>>,
    pub custom_filters: Option<Vec<ProvenanceFilter>>,
}

/// Verdict decision type filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerdictDecisionType {
    Accept,
    Reject,
    RequireModification,
    NeedInvestigation,
}

/// Time range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Compliance status filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PartialCompliance,
}

/// Provenance chain for audit trails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub chain_id: Uuid,
    pub entries: Vec<ProvenanceRecord>,
    pub integrity_verified: bool,
    pub chain_length: u32,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Provenance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceStats {
    pub total_records: u64,
    pub total_verdicts: u64,
    pub acceptance_rate: f32,
    pub average_consensus_score: f32,
    pub average_compliance_score: f32,
    pub average_verification_quality: f32,
    pub most_active_judge: String,
    pub common_violations: Vec<ViolationStats>,
    pub time_range: TimeRange,
}

/// Violation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationStats {
    pub rule: String,
    pub count: u64,
    pub severity_distribution: HashMap<ViolationSeverity, u64>,
    pub average_resolution_time_ms: f64,
}

/// Provenance export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceExport {
    pub export_id: Uuid,
    pub format: ExportFormat,
    pub records: Vec<ProvenanceRecord>,
    pub metadata: ExportMetadata,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

/// Export formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Tarball,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub total_records: u64,
    pub time_range: TimeRange,
    pub filters_applied: Vec<String>,
    pub export_reason: String,
    pub recipient: Option<String>,
}

/// Provenance integrity check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckResult {
    pub is_valid: bool,
    pub issues: Vec<IntegrityIssue>,
    pub checked_records: u64,
    pub checked_at: DateTime<Utc>,
}

/// Integrity issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityIssue {
    pub record_id: Uuid,
    pub issue_type: IntegrityIssueType,
    pub description: String,
    pub severity: IntegritySeverity,
}

/// Integrity issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrityIssueType {
    SignatureInvalid,
    GitCommitMissing,
    GitTrailerCorrupted,
    TimestampInconsistent,
    ChainBroken,
    DataCorrupted,
}

/// Integrity issue severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegritySeverity {
    Critical,
    Major,
    Minor,
    Warning,
}

impl ProvenanceRecord {
    /// Create a new provenance record
    pub fn new(
        verdict_id: Uuid,
        task_id: Uuid,
        decision: VerdictDecision,
        consensus_score: f32,
        judge_verdicts: HashMap<String, JudgeVerdictProvenance>,
        caws_compliance: CawsComplianceProvenance,
        claim_verification: Option<ClaimVerificationProvenance>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            verdict_id,
            task_id,
            decision,
            consensus_score,
            judge_verdicts,
            caws_compliance,
            claim_verification,
            git_commit_hash: None,
            git_trailer: format!("CAWS-VERDICT-ID: {}", verdict_id),
            signature: String::new(), // Will be set by signer
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Get the decision summary
    pub fn decision_summary(&self) -> &str {
        match &self.decision {
            VerdictDecision::Accept { summary, .. } => summary,
            VerdictDecision::Reject { summary, .. } => summary,
            VerdictDecision::RequireModification { summary, .. } => summary,
            VerdictDecision::NeedInvestigation { summary, .. } => summary,
        }
    }

    /// Check if the verdict was accepted
    pub fn is_accepted(&self) -> bool {
        matches!(self.decision, VerdictDecision::Accept { .. })
    }

    /// Get the number of judge verdicts
    pub fn judge_count(&self) -> usize {
        self.judge_verdicts.len()
    }

    /// Calculate overall quality score
    pub fn overall_quality_score(&self) -> f32 {
        let mut scores = vec![self.consensus_score, self.caws_compliance.compliance_score];

        if let Some(ref claim_verification) = self.claim_verification {
            scores.push(claim_verification.verification_quality);
        }

        scores.iter().sum::<f32>() / scores.len() as f32
    }
}

impl VerdictDecision {
    /// Get the decision type as a string
    pub fn decision_type(&self) -> &'static str {
        match self {
            VerdictDecision::Accept { .. } => "accept",
            VerdictDecision::Reject { .. } => "reject",
            VerdictDecision::RequireModification { .. } => "require_modification",
            VerdictDecision::NeedInvestigation { .. } => "need_investigation",
        }
    }

    /// Get the confidence score for the decision
    pub fn confidence(&self) -> f32 {
        match self {
            VerdictDecision::Accept { confidence, .. } => *confidence,
            VerdictDecision::Reject { .. } => 1.0, // Reject decisions are always confident
            VerdictDecision::RequireModification { .. } => 0.8, // High confidence in need for changes
            VerdictDecision::NeedInvestigation { .. } => 0.5,   // Medium confidence, need more info
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_provenance_record_creation() {
        let verdict_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let decision = VerdictDecision::Accept {
            confidence: 0.9,
            summary: "Task completed successfully".to_string(),
        };
        let consensus_score = 0.85;
        let judge_verdicts = HashMap::new();
        let caws_compliance = CawsComplianceProvenance {
            is_compliant: true,
            compliance_score: 0.95,
            violations: vec![],
            waivers_used: vec![],
            budget_adherence: BudgetAdherence {
                max_files: 10,
                actual_files: 8,
                max_loc: 1000,
                actual_loc: 750,
                max_time_minutes: Some(60),
                actual_time_minutes: Some(45),
                within_budget: true,
            },
        };

        let record = ProvenanceRecord::new(
            verdict_id,
            task_id,
            decision,
            consensus_score,
            judge_verdicts,
            caws_compliance,
            None,
        );

        assert_eq!(record.verdict_id, verdict_id);
        assert_eq!(record.task_id, task_id);
        assert_eq!(record.consensus_score, consensus_score);
        assert!(record.is_accepted());
        assert_eq!(record.judge_count(), 0);
    }

    #[test]
    fn test_verdict_decision_confidence() {
        let accept = VerdictDecision::Accept {
            confidence: 0.9,
            summary: "Accepted".to_string(),
        };
        let reject = VerdictDecision::Reject {
            primary_reasons: vec!["Failed tests".to_string()],
            summary: "Rejected".to_string(),
        };

        assert_eq!(accept.confidence(), 0.9);
        assert_eq!(reject.confidence(), 1.0);
        assert_eq!(accept.decision_type(), "accept");
        assert_eq!(reject.decision_type(), "reject");
    }

    #[test]
    fn test_overall_quality_score_calculation() {
        let record = ProvenanceRecord {
            id: Uuid::new_v4(),
            verdict_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            decision: VerdictDecision::Accept {
                confidence: 0.9,
                summary: "Test".to_string(),
            },
            consensus_score: 0.8,
            judge_verdicts: HashMap::new(),
            caws_compliance: CawsComplianceProvenance {
                is_compliant: true,
                compliance_score: 0.9,
                violations: vec![],
                waivers_used: vec![],
                budget_adherence: BudgetAdherence {
                    max_files: 10,
                    actual_files: 8,
                    max_loc: 1000,
                    actual_loc: 750,
                    max_time_minutes: Some(60),
                    actual_time_minutes: Some(45),
                    within_budget: true,
                },
            },
            claim_verification: Some(ClaimVerificationProvenance {
                claims_total: 5,
                claims_verified: 4,
                claims_unverified: 1,
                verification_quality: 0.85,
                evidence_items: vec![],
                ambiguities_resolved: 2,
                extraction_time_ms: 100,
                verification_time_ms: 200,
            }),
            git_commit_hash: None,
            git_trailer: "CAWS-VERDICT-ID: test".to_string(),
            signature: String::new(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        // Expected: (0.8 + 0.9 + 0.85) / 3 = 0.85
        assert_eq!(record.overall_quality_score(), 0.85);
    }
}

/// Types of provenance filters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterType {
    TimeRange,
    EntityType,
    EntityId,
    ActivityType,
    ActivityId,
    AgentType,
    AgentId,
    Custom,
}

/// Filter operators for provenance queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterOperator {
    Equals,
    In,
    Between,
}

/// Individual filter condition for provenance queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceFilter {
    pub filter_type: FilterType,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}
