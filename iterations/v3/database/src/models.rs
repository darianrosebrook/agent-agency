//! Database models and types for Agent Agency V3

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Judge model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Judge {
    pub id: Uuid,
    pub name: String,
    pub model_name: String,
    pub endpoint: String,
    pub weight: f32,
    pub timeout_ms: i32,
    pub optimization_target: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Worker model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub worker_type: String,
    pub specialty: Option<String>,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: serde_json::Value,
    pub performance_history: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub risk_tier: String,
    pub scope: serde_json::Value,
    pub acceptance_criteria: serde_json::Value,
    pub context: serde_json::Value,
    pub caws_spec: Option<serde_json::Value>,
    pub status: String,
    pub assigned_worker_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Task execution model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskExecution {
    pub id: Uuid,
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub execution_started_at: DateTime<Utc>,
    pub execution_completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub status: String,
    pub worker_output: serde_json::Value,
    pub self_assessment: serde_json::Value,
    pub metadata: serde_json::Value,
    pub error_message: Option<String>,
    pub tokens_used: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub execution_metadata: Option<serde_json::Value>,
    pub result_data: Option<serde_json::Value>,
}

/// Council verdict model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CouncilVerdict {
    pub id: Uuid,
    pub task_id: Uuid,
    pub verdict_id: Uuid,
    pub consensus_score: f32,
    pub final_verdict: serde_json::Value,
    pub individual_verdicts: serde_json::Value,
    pub debate_rounds: i32,
    pub evaluation_time_ms: i32,
    pub created_at: DateTime<Utc>,
    pub contract: serde_json::Value,
    pub updated_at: Option<DateTime<Utc>>,
    pub verdict_details: Option<serde_json::Value>,
}

/// Judge evaluation model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JudgeEvaluation {
    pub id: Uuid,
    pub verdict_id: Uuid,
    pub judge_id: Uuid,
    pub judge_verdict: serde_json::Value,
    pub evaluation_time_ms: i32,
    pub tokens_used: Option<i32>,
    pub confidence: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub evaluation_score: Option<f32>,
    pub confidence_score: Option<f32>,
    pub reasoning: Option<String>,
    pub evidence_used: Option<serde_json::Value>,
    pub evaluation_metadata: Option<serde_json::Value>,
    pub verdict_decision: Option<String>,
    pub risk_assessment: Option<serde_json::Value>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Debate session model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DebateSession {
    pub id: Uuid,
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub conflicting_judges: serde_json::Value,
    pub rounds: serde_json::Value,
    pub status: String,
    pub final_consensus: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Knowledge entry model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KnowledgeEntry {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub source: String,
    pub source_url: Option<String>,
    pub relevance_score: f32,
    pub tags: serde_json::Value,
    pub embedding: Option<Vec<f32>>, // pgvector as Vec<f32>
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub embedding_vector: Option<Vec<f32>>,
    pub access_level: Option<String>,
    pub version: Option<String>,
    pub parent_id: Option<Uuid>,
}

/// Performance metric model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceMetric {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_unit: Option<String>,
    pub metadata: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

/// CAWS compliance model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CawsCompliance {
    pub id: Uuid,
    pub task_id: Uuid,
    pub verdict_id: Option<Uuid>,
    pub compliance_score: f32,
    pub violations: serde_json::Value,
    pub waivers: serde_json::Value,
    pub budget_adherence: serde_json::Value,
    pub quality_gates: serde_json::Value,
    pub provenance_trail: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Audit trail entry model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditTrailEntry {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub details: serde_json::Value,
    pub user_id: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub created_at: DateTime<Utc>,
}

/// Source integrity record model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceIntegrityRecord {
    pub id: Uuid,
    pub source_id: String,
    pub source_type: String,
    pub content_hash: String,
    pub content_size: i64,
    pub hash_algorithm: String,
    pub integrity_status: String,
    pub tampering_indicators: serde_json::Value,
    pub verification_metadata: serde_json::Value,
    pub first_seen_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub verification_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Source integrity verification model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceIntegrityVerification {
    pub id: Uuid,
    pub source_integrity_id: Uuid,
    pub verification_type: String,
    pub verification_result: String,
    pub calculated_hash: String,
    pub stored_hash: String,
    pub hash_match: bool,
    pub tampering_detected: bool,
    pub verification_details: serde_json::Value,
    pub verified_by: Option<String>,
    pub verification_duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Source integrity alert model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceIntegrityAlert {
    pub id: Uuid,
    pub source_integrity_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub alert_message: String,
    pub alert_data: serde_json::Value,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input types for creating new records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJudge {
    pub name: String,
    pub model_name: String,
    pub endpoint: String,
    pub weight: f32,
    pub timeout_ms: i32,
    pub optimization_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorker {
    pub name: String,
    pub worker_type: String,
    pub specialty: Option<String>,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: String,
    pub risk_tier: String,
    pub scope: serde_json::Value,
    pub acceptance_criteria: serde_json::Value,
    pub context: serde_json::Value,
    pub caws_spec: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskExecution {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub worker_output: serde_json::Value,
    pub self_assessment: serde_json::Value,
    pub metadata: serde_json::Value,
    pub tokens_used: Option<i32>,
    pub execution_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCouncilVerdict {
    pub task_id: Uuid,
    pub verdict_id: Uuid,
    pub consensus_score: f32,
    pub final_verdict: serde_json::Value,
    pub individual_verdicts: serde_json::Value,
    pub debate_rounds: i32,
    pub evaluation_time_ms: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJudgeEvaluation {
    pub verdict_id: Uuid,
    pub judge_id: Uuid,
    pub judge_verdict: serde_json::Value,
    pub evaluation_time_ms: i32,
    pub tokens_used: Option<i32>,
    pub confidence: Option<f32>,
    pub evaluation_score: Option<f32>,
    pub confidence_score: Option<f32>,
    pub reasoning: Option<String>,
    pub evidence_used: Option<serde_json::Value>,
    pub evaluation_metadata: Option<serde_json::Value>,
    pub verdict_decision: Option<String>,
    pub risk_assessment: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeEntry {
    pub title: String,
    pub content: String,
    pub source: String,
    pub source_url: Option<String>,
    pub relevance_score: f32,
    pub tags: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub content_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceIntegrityRecord {
    pub source_id: String,
    pub source_type: String,
    pub content_hash: String,
    pub content_size: i64,
    pub hash_algorithm: String,
    pub integrity_status: String,
    pub tampering_indicators: serde_json::Value,
    pub verification_metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceIntegrityVerification {
    pub source_integrity_id: Uuid,
    pub verification_type: String,
    pub verification_result: String,
    pub calculated_hash: String,
    pub stored_hash: String,
    pub hash_match: bool,
    pub tampering_detected: bool,
    pub verification_details: serde_json::Value,
    pub verified_by: Option<String>,
    pub verification_duration_ms: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceIntegrityAlert {
    pub source_integrity_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub alert_message: String,
    pub alert_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerformanceMetric {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_unit: Option<String>,
    pub metadata: serde_json::Value,
    pub metric_type: Option<String>,
    pub component: Option<String>,
    pub task_id: Option<Uuid>,
    pub execution_id: Option<Uuid>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCawsCompliance {
    pub task_id: Uuid,
    pub verdict_id: Option<Uuid>,
    pub compliance_score: f32,
    pub violations: serde_json::Value,
    pub waivers: serde_json::Value,
    pub budget_adherence: serde_json::Value,
    pub quality_gates: serde_json::Value,
    pub provenance_trail: serde_json::Value,
    pub compliance_status: Option<String>,
    pub recommendations: Option<serde_json::Value>,
    pub audit_timestamp: Option<DateTime<Utc>>,
    pub compliance_metadata: Option<serde_json::Value>,
    pub audit_details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditTrailEntry {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub details: serde_json::Value,
    pub user_id: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Update types for modifying existing records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJudge {
    pub name: Option<String>,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub weight: Option<f32>,
    pub timeout_ms: Option<i32>,
    pub optimization_target: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorker {
    pub name: Option<String>,
    pub worker_type: Option<String>,
    pub specialty: Option<String>,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub performance_history: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub status: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub risk_tier: Option<String>,
    pub scope: Option<serde_json::Value>,
    pub acceptance_criteria: Option<serde_json::Value>,
    pub context: Option<serde_json::Value>,
    pub caws_spec: Option<serde_json::Value>,
    pub status: Option<String>,
    pub assigned_worker_id: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskExecution {
    pub execution_completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub status: Option<String>,
    pub worker_output: Option<serde_json::Value>,
    pub self_assessment: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub tokens_used: Option<i32>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_data: Option<serde_json::Value>,
    pub execution_metadata: Option<serde_json::Value>,
}

/// Query filters and pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            limit: None,
            offset: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilters {
    pub status: Option<String>,
    pub risk_tier: Option<String>,
    pub assigned_worker_id: Option<Uuid>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictFilters {
    pub task_id: Option<Uuid>,
    pub consensus_score_min: Option<f32>,
    pub consensus_score_max: Option<f32>,
    pub min_consensus_score: Option<f32>,
    pub max_consensus_score: Option<f32>,
    pub min_debate_rounds: Option<i32>,
    pub max_debate_rounds: Option<i32>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFilters {
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub relevance_score_min: Option<f32>,
    pub content_type: Option<String>,
    pub access_level: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// Statistics and analytics types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilMetrics {
    pub date: DateTime<Utc>,
    pub total_verdicts: i64,
    pub avg_consensus_score: Option<f64>,
    pub avg_debate_rounds: Option<f64>,
    pub accepted_count: i64,
    pub rejected_count: i64,
    pub modification_required_count: i64,
    pub avg_evaluation_time_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgePerformance {
    pub judge_id: Uuid,
    pub judge_name: String,
    pub model_name: String,
    pub total_evaluations: i64,
    pub avg_evaluation_time_ms: Option<f64>,
    pub avg_confidence: Option<f64>,
    pub avg_evaluation_score: Option<f32>,
    pub avg_confidence_score: Option<f32>,
    pub pass_count: i64,
    pub fail_count: i64,
    pub uncertain_count: i64,
    pub approved_count: Option<i32>,
    pub rejected_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformance {
    pub worker_id: Uuid,
    pub worker_name: String,
    pub worker_type: String,
    pub specialty: Option<String>,
    pub total_executions: i64,
    pub avg_execution_time_ms: Option<f64>,
    pub completed_count: i64,
    pub failed_count: i64,
    pub avg_tokens_used: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionSummary {
    pub task_id: Uuid,
    pub title: String,
    pub status: String,
    pub risk_tier: String,
    pub total_executions: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub running_count: i64,
    pub avg_execution_time_ms: Option<f64>,
    pub first_execution: Option<DateTime<Utc>>,
    pub last_completion: Option<DateTime<Utc>>,
    pub executions: serde_json::Value,
    pub verdicts: serde_json::Value,
    pub compliance: serde_json::Value,
}

/// CAWS violation model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CawsViolation {
    pub id: Uuid,
    pub task_id: Uuid,
    pub violation_code: String,
    pub severity: String,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub column_number: Option<i32>,
    pub rule_id: String,
    pub constitutional_reference: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// CAWS rule model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CawsRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: String,
    pub severity: String,
    pub file_patterns: serde_json::Value,
    pub config: serde_json::Value,
    pub constitutional_reference: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CAWS specification model from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CawsSpecification {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub specification: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Knowledge source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeSource {
    Wikidata,
    WordNet,
}

impl KnowledgeSource {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            KnowledgeSource::Wikidata => "wikidata",
            KnowledgeSource::WordNet => "wordnet",
        }
    }
}

/// External knowledge entity from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalKnowledgeEntity {
    pub id: Option<Uuid>,
    pub source: KnowledgeSource,
    pub entity_key: String,
    pub canonical_name: String,
    pub lang: Option<String>,
    pub entity_type: Option<String>,
    pub properties: serde_json::Value,
    pub confidence: f64,
    pub usage_count: i32,
    pub usage_decay: Option<f64>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub dump_version: Option<String>,
    pub toolchain: Option<String>,
    pub license: Option<String>,
}

/// Knowledge relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelationship {
    pub id: Option<Uuid>,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub relationship_type: String,
    pub confidence: f64,
    pub metadata: Option<serde_json::Value>,
}

/// Knowledge base statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub source: String,
    pub total_entities: i64,
    pub total_vectors: i64,
    pub total_relationships: i64,
    pub avg_confidence: f64,
    pub avg_usage_count: f64,
    pub last_updated: DateTime<Utc>,
}
