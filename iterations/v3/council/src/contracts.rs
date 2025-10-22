//! Contract definitions for council operations
//!
//! Defines the interfaces and contracts between council components,
//! ensuring type safety and clear boundaries between services.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Contract for judge evaluation services
#[async_trait]
pub trait JudgeEvaluator: Send + Sync {
    /// Evaluate a task and return a verdict
    async fn evaluate(&self, task_spec: &TaskSpec) -> Result<JudgeEvaluation>;
    
    /// Get judge configuration and capabilities
    fn get_judge_spec(&self) -> JudgeSpec;
    
    /// Check if judge is available for evaluation
    async fn is_available(&self) -> Result<bool>;
    
    /// Get performance metrics for this judge
    async fn get_metrics(&self) -> Result<JudgeMetrics>;
}

/// Contract for debate protocol services
#[async_trait]
pub trait DebateService: Send + Sync {
    /// Start a debate session for conflicting verdicts
    async fn start_debate(
        &self,
        task_id: TaskId,
        conflicting_judges: Vec<JudgeId>,
        individual_verdicts: HashMap<JudgeId, JudgeVerdict>,
    ) -> Result<DebateSession>;
    
    /// Get active debate session
    async fn get_debate_session(&self, session_id: Uuid) -> Result<Option<DebateSession>>;
    
    /// Get all active debates
    async fn get_active_debates(&self) -> Result<Vec<DebateSession>>;
}

/// Contract for verdict storage services
#[async_trait]
pub trait VerdictStorage: Send + Sync {
    /// Store a consensus result
    async fn store_consensus(
        &self,
        consensus_result: ConsensusResult,
        debate_session: Option<DebateSession>,
    ) -> Result<VerdictId>;
    
    /// Retrieve a verdict by ID
    async fn get_verdict(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>>;
    
    /// Get verdicts for a specific task
    async fn get_verdicts_for_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>>;
    
    /// Get verdicts within a time range
    async fn get_verdicts_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<VerdictRecord>>;
    
    /// Delete a verdict (for testing/cleanup)
    async fn delete_verdict(&self, verdict_id: VerdictId) -> Result<()>;
    
    /// Get storage statistics
    async fn get_stats(&self) -> Result<VerdictStoreStats>;
}

/// Contract for research agent services
#[async_trait]
pub trait ResearchAgent: Send + Sync {
    /// Research a topic and return findings
    async fn research_topic(&self, topic: String, context: Vec<String>) -> Result<Vec<ResearchFinding>>;
    
    /// Get research agent capabilities
    fn get_capabilities(&self) -> ResearchCapabilities;
    
    /// Check if research agent is available
    async fn is_available(&self) -> Result<bool>;
}

/// Research agent capabilities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchCapabilities {
    pub web_search: bool,
    pub code_analysis: bool,
    pub documentation_search: bool,
    pub vector_search: bool,
    pub max_context_length: u32,
    pub supported_formats: Vec<String>,
}

/// Contract for consensus coordination services
#[async_trait]
pub trait ConsensusService: Send + Sync {
    /// Evaluate a task with the council
    async fn evaluate_task(&self, task_spec: TaskSpec) -> Result<ConsensusResult>;
    
    /// Evaluate a task with validation artifacts and research evidence
    async fn evaluate_task_with_artifacts(
        &self,
        task_spec: TaskSpec,
        validation_result: CawsValidationResult,
        research_bundle: Option<ResearchEvidenceBundle>,
    ) -> Result<ConsensusResult>;
    
    /// Get current council metrics
    async fn get_metrics(&self) -> Result<CouncilMetrics>;
    
    /// Check council health
    async fn health_check(&self) -> Result<bool>;
}

// NEW: Research evidence bundle for claim verification
#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchEvidenceBundle {
    pub query_id: uuid::Uuid,
    pub evidence_items: Vec<EvidenceItem>,
    pub synthesis_quality: f32,
    pub cross_references: Vec<CrossReference>,
    pub collection_time_ms: u64,
    pub claim_coverage_pct: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: uuid::Uuid,
    pub claim_id: Option<String>,
    pub content: String,
    pub source: KnowledgeSource,
    pub quality_score: f32,
    pub verification_status: VerificationStatus,
    pub relevance_score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossReference {
    pub source_id: uuid::Uuid,
    pub target_id: uuid::Uuid,
    pub relationship: CrossReferenceType,
    pub strength: f32,
    pub context: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CrossReferenceType {
    Supports,
    Contradicts,
    BuildsUpon,
    References,
    Similar,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KnowledgeSource {
    WebSearch,
    CodeAnalysis,
    Documentation,
    VectorDatabase,
    ExpertKnowledge,
    HistoricalData,
}

/// Contract for task routing services
#[async_trait]
pub trait TaskRouter: Send + Sync {
    /// Route a task to appropriate workers
    async fn route_task(&self, task_spec: &TaskSpec) -> Result<Vec<WorkerAssignment>>;
    
    /// Get available workers for task type
    async fn get_available_workers(&self, task_type: &str) -> Result<Vec<Worker>>;
    
    /// Update worker performance metrics
    async fn update_worker_metrics(&self, worker_id: Uuid, metrics: WorkerMetrics) -> Result<()>;
}

/// Worker assignment for task routing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerAssignment {
    pub worker_id: Uuid,
    pub worker_name: String,
    pub capability_score: f32,
    pub estimated_time_ms: u64,
    pub reasoning: String,
}

/// Worker metrics for performance tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerMetrics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time_ms: f64,
    pub caws_compliance_rate: f32,
    pub quality_score: f32,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Contract for CAWS compliance services
#[async_trait]
pub trait CawsComplianceService: Send + Sync {
    /// Validate task against CAWS requirements
    async fn validate_task(&self, task_spec: &TaskSpec) -> Result<CawsValidationResult>;
    
    /// Check worker output for CAWS compliance
    async fn validate_worker_output(&self, output: &WorkerOutput, task_spec: &TaskSpec) -> Result<CawsValidationResult>;
    
    /// Get CAWS rule violations
    async fn get_violations(&self, task_id: TaskId) -> Result<Vec<Violation>>;
    
    /// Check waiver validity
    async fn validate_waiver(&self, waiver: &CawsWaiver) -> Result<bool>;
}

/// CAWS validation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CawsValidationResult {
    pub is_compliant: bool,
    pub compliance_score: f32,
    pub violations: Vec<Violation>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

/// Contract for model inference services
#[async_trait]
pub trait ModelInferenceService: Send + Sync {
    /// Run inference on a model with given input
    async fn run_inference(&self, model_name: &str, input: &str, options: InferenceOptions) -> Result<InferenceResult>;
    
    /// Check if model is available
    async fn is_model_available(&self, model_name: &str) -> Result<bool>;
    
    /// Get model information
    async fn get_model_info(&self, model_name: &str) -> Result<Option<ModelInfo>>;
    
    /// Get available models
    async fn get_available_models(&self) -> Result<Vec<ModelInfo>>;
}

/// Model inference options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InferenceOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub timeout_ms: Option<u64>,
    pub optimization_target: Option<OptimizationTarget>,
}

// ==== Shared contract types matching JSON Schemas ====

#[derive(Debug, Serialize, Deserialize)]
pub struct Seeds {
    pub time_seed: String,
    pub uuid_seed: String,
    pub random_seed: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerMetadata {
    pub task_id: String,
    pub risk_tier: u8,
    pub seeds: Seeds,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchArtifact {
    pub path: String,
    pub diff: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandArtifact {
    pub cmd: String,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifacts {
    pub patches: Vec<PatchArtifact>,
    #[serde(default)]
    pub commands: Vec<CommandArtifact>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CawsChecklist {
    pub within_scope: bool,
    pub within_budget: bool,
    pub tests_added: bool,
    pub deterministic: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelfAssessment {
    pub caws_checklist: CawsChecklist,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Waiver {
    pub id: String,
    pub reason: String,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerOutput {
    pub metadata: WorkerMetadata,
    pub artifacts: Artifacts,
    pub rationale: String,
    pub self_assessment: SelfAssessment,
    #[serde(default)]
    pub waivers: Vec<Waiver>,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub evidence_refs: Vec<EvidenceLink>,
    
    // NEW: Claim verification support
    #[serde(default)]
    pub claim_references: Option<Vec<ClaimReference>>,
    #[serde(default)]
    pub evidence_references: Option<Vec<EvidenceReference>>,
    #[serde(default)]
    pub verification_artifacts: Option<VerificationArtifacts>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum VerdictSimple { Pass, Fail, Uncertain }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType { Research, StaticCheck, Test }

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub r#type: EvidenceType,
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JudgeVerdict {
    pub judge_id: String,
    pub version: String,
    pub verdict: VerdictSimple,
    pub reasons: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FinalDecision { Accept, Reject, Modify }

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteEntry {
    pub judge_id: String,
    pub weight: f32,
    pub verdict: VerdictSimple,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinalVerdict {
    pub decision: FinalDecision,
    pub votes: Vec<VoteEntry>,
    pub dissent: String,
    #[serde(default)]
    pub remediation: Vec<String>,
    #[serde(default)]
    pub constitutional_refs: Vec<String>,
    #[serde(default)]
    pub verification_summary: Option<VerificationSummary>,
    pub metadata: Option<serde_json::Value>,
    
    // NEW: Orchestration metrics for comprehensive tracking
    #[serde(default)]
    pub orchestration_metrics: OrchestrationMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ThermalStatus {
    Normal,
    Warning,
    Throttling,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceLink {
    pub claim_id: String,
    pub ref_: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub claims_total: u32,
    pub claims_verified: u32,
    pub coverage_pct: f32,
    
    // NEW: Enhanced verification metrics
    pub claim_coverage_pct: f32,
    pub evidence_quality_avg: f32,
    pub factual_consistency_score: f32,
}

// NEW: Claim verification support structures
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimReference {
    pub id: String,
    pub statement: String,
    pub confidence: f32,
    pub source_context: String,
    pub verification_requirements: Vec<VerificationCriteria>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceReference {
    pub claim_id: String,
    pub evidence_type: EvidenceType,
    pub source: String,
    pub quality_score: f32,
    pub verification_status: VerificationStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationArtifacts {
    pub claims_extracted: u32,
    pub claims_verified: u32,
    pub verification_quality: f32,
    pub ambiguities_resolved: u32,
    pub extraction_time_ms: u64,
    pub verification_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationCriteria {
    pub criterion_type: String,
    pub description: String,
    pub required: bool,
    pub verification_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    InsufficientEvidence,
    Pending,
    Failed,
}

// NEW: Orchestration metrics for comprehensive tracking
#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrationMetrics {
    pub debate_rounds: u32,
    pub consensus_score: f32,
    pub dissent_rate: f32,
    pub claim_coverage_pct: f32,
    pub context_reuse_pct: f32,
    pub evaluation_time_ms: u64,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub thermal_status: ThermalStatus,
    pub ane_utilization: Option<f32>,
}

/// Model inference result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InferenceResult {
    pub output: String,
    pub tokens_used: u32,
    pub inference_time_ms: u64,
    pub confidence: Option<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Model information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub size_gb: f32,
    pub optimization_targets: Vec<OptimizationTarget>,
    pub capabilities: Vec<String>,
    pub max_context_length: u32,
    pub is_available: bool,
}

/// Contract for performance monitoring services
#[async_trait]
pub trait PerformanceMonitor: Send + Sync {
    /// Record a performance metric
    async fn record_metric(&self, metric: PerformanceMetric) -> Result<()>;
    
    /// Get performance metrics for an entity
    async fn get_metrics(&self, entity_type: &str, entity_id: Uuid) -> Result<Vec<PerformanceMetric>>;
    
    /// Get system performance summary
    async fn get_system_performance(&self) -> Result<SystemPerformanceSummary>;
    
    /// Check if system is performing within targets
    async fn check_performance_targets(&self) -> Result<PerformanceTargetStatus>;
}

/// System performance summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemPerformanceSummary {
    pub total_evaluations: u64,
    pub average_evaluation_time_ms: f64,
    pub consensus_rate: f32,
    pub system_uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub active_debates: u32,
    pub active_tasks: u32,
}

/// Performance target status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceTargetStatus {
    pub evaluation_time_target_ms: u64,
    pub current_average_ms: f64,
    pub is_within_target: bool,
    pub memory_usage_target_mb: u64,
    pub current_memory_mb: u64,
    pub memory_within_target: bool,
    pub thermal_target_c: u32,
    pub current_thermal_c: Option<u32>,
    pub thermal_within_target: bool,
}

/// Contract for audit trail services
#[async_trait]
pub trait AuditTrailService: Send + Sync {
    /// Record an audit trail entry
    async fn record_entry(&self, entry: CreateAuditTrailEntry) -> Result<()>;
    
    /// Get audit trail for an entity
    async fn get_audit_trail(&self, entity_type: &str, entity_id: Uuid) -> Result<Vec<AuditTrailEntry>>;
    
    /// Search audit trail by action
    async fn search_by_action(&self, action: &str) -> Result<Vec<AuditTrailEntry>>;
    
    /// Get audit trail within time range
    async fn get_audit_trail_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<AuditTrailEntry>>;
}

/// Contract for configuration services
#[async_trait]
pub trait ConfigurationService: Send + Sync {
    /// Get configuration value
    async fn get_config<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned;
    
    /// Set configuration value
    async fn set_config<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize;
    
    /// Get all configuration
    async fn get_all_config(&self) -> Result<HashMap<String, serde_json::Value>>;
    
    /// Reload configuration from source
    async fn reload_config(&self) -> Result<()>;
}

/// Service registry for dependency injection
#[derive(Debug)]
pub struct ServiceRegistry {
    judge_evaluators: HashMap<String, Box<dyn JudgeEvaluator>>,
    debate_service: Option<Box<dyn DebateService>>,
    verdict_storage: Option<Box<dyn VerdictStorage>>,
    research_agent: Option<Box<dyn ResearchAgent>>,
    consensus_service: Option<Box<dyn ConsensusService>>,
    task_router: Option<Box<dyn TaskRouter>>,
    caws_compliance: Option<Box<dyn CawsComplianceService>>,
    model_inference: Option<Box<dyn ModelInferenceService>>,
    performance_monitor: Option<Box<dyn PerformanceMonitor>>,
    audit_trail: Option<Box<dyn AuditTrailService>>,
    configuration: Option<Box<dyn ConfigurationService>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            judge_evaluators: HashMap::new(),
            debate_service: None,
            verdict_storage: None,
            research_agent: None,
            consensus_service: None,
            task_router: None,
            caws_compliance: None,
            model_inference: None,
            performance_monitor: None,
            audit_trail: None,
            configuration: None,
        }
    }

    /// Register a judge evaluator
    pub fn register_judge_evaluator(&mut self, name: String, evaluator: Box<dyn JudgeEvaluator>) {
        self.judge_evaluators.insert(name, evaluator);
    }

    /// Get a judge evaluator
    pub fn get_judge_evaluator(&self, name: &str) -> Option<&dyn JudgeEvaluator> {
        self.judge_evaluators.get(name).map(|e| e.as_ref())
    }

    /// Register debate service
    pub fn register_debate_service(&mut self, service: Box<dyn DebateService>) {
        self.debate_service = Some(service);
    }

    /// Get debate service
    pub fn get_debate_service(&self) -> Option<&dyn DebateService> {
        self.debate_service.as_ref().map(|s| s.as_ref())
    }

    /// Register verdict storage
    pub fn register_verdict_storage(&mut self, storage: Box<dyn VerdictStorage>) {
        self.verdict_storage = Some(storage);
    }

    /// Get verdict storage
    pub fn get_verdict_storage(&self) -> Option<&dyn VerdictStorage> {
        self.verdict_storage.as_ref().map(|s| s.as_ref())
    }

    /// Register research agent
    pub fn register_research_agent(&mut self, agent: Box<dyn ResearchAgent>) {
        self.research_agent = Some(agent);
    }

    /// Get research agent
    pub fn get_research_agent(&self) -> Option<&dyn ResearchAgent> {
        self.research_agent.as_ref().map(|a| a.as_ref())
    }

    /// Register consensus service
    pub fn register_consensus_service(&mut self, service: Box<dyn ConsensusService>) {
        self.consensus_service = Some(service);
    }

    /// Get consensus service
    pub fn get_consensus_service(&self) -> Option<&dyn ConsensusService> {
        self.consensus_service.as_ref().map(|s| s.as_ref())
    }

    /// Register task router
    pub fn register_task_router(&mut self, router: Box<dyn TaskRouter>) {
        self.task_router = Some(router);
    }

    /// Get task router
    pub fn get_task_router(&self) -> Option<&dyn TaskRouter> {
        self.task_router.as_ref().map(|r| r.as_ref())
    }

    /// Register CAWS compliance service
    pub fn register_caws_compliance(&mut self, service: Box<dyn CawsComplianceService>) {
        self.caws_compliance = Some(service);
    }

    /// Get CAWS compliance service
    pub fn get_caws_compliance(&self) -> Option<&dyn CawsComplianceService> {
        self.caws_compliance.as_ref().map(|s| s.as_ref())
    }

    /// Register model inference service
    pub fn register_model_inference(&mut self, service: Box<dyn ModelInferenceService>) {
        self.model_inference = Some(service);
    }

    /// Get model inference service
    pub fn get_model_inference(&self) -> Option<&dyn ModelInferenceService> {
        self.model_inference.as_ref().map(|s| s.as_ref())
    }

    /// Register performance monitor
    pub fn register_performance_monitor(&mut self, monitor: Box<dyn PerformanceMonitor>) {
        self.performance_monitor = Some(monitor);
    }

    /// Get performance monitor
    pub fn get_performance_monitor(&self) -> Option<&dyn PerformanceMonitor> {
        self.performance_monitor.as_ref().map(|m| m.as_ref())
    }

    /// Register audit trail service
    pub fn register_audit_trail(&mut self, service: Box<dyn AuditTrailService>) {
        self.audit_trail = Some(service);
    }

    /// Get audit trail service
    pub fn get_audit_trail(&self) -> Option<&dyn AuditTrailService> {
        self.audit_trail.as_ref().map(|s| s.as_ref())
    }

    /// Register configuration service
    pub fn register_configuration(&mut self, service: Box<dyn ConfigurationService>) {
        self.configuration = Some(service);
    }

    /// Get configuration service
    pub fn get_configuration(&self) -> Option<&dyn ConfigurationService> {
        self.configuration.as_ref().map(|s| s.as_ref())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_registry_creation() {
        let registry = ServiceRegistry::new();
        assert!(registry.judge_evaluators.is_empty());
        assert!(registry.debate_service.is_none());
        assert!(registry.verdict_storage.is_none());
    }

    #[test]
    fn test_inference_options_default() {
        let options = InferenceOptions {
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            timeout_ms: Some(5000),
            optimization_target: Some(OptimizationTarget::CPU),
        };

        assert_eq!(options.max_tokens, Some(1000));
        assert_eq!(options.temperature, Some(0.7));
    }

    #[test]
    fn test_worker_assignment_creation() {
        let assignment = WorkerAssignment {
            worker_id: Uuid::new_v4(),
            worker_name: "test-worker".to_string(),
            capability_score: 0.85,
            estimated_time_ms: 2000,
            reasoning: "Best match for task requirements".to_string(),
        };

        assert_eq!(assignment.worker_name, "test-worker");
        assert_eq!(assignment.capability_score, 0.85);
    }
}
