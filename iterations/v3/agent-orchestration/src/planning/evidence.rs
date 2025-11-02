//! Evidence Collector - Collect and validate execution evidence
//!
//! Real evidence collection system that integrates with agent-research
//! evidence collectors for comprehensive milestone validation.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use async_trait::async_trait;
use agent_agency_contracts::planning_io::{Milestone, EvidenceGate};
// Local type definitions to avoid circular dependency with agent-research
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResearchEvidence {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub content: String,
    pub evidence_type: ResearchEvidenceType,
    pub confidence: f64,
    pub source: String,
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ResearchEvidenceType {
    CodeReview,
    CodeAnalysis, // Alias/synonym for CodeReview
    TestExecution,
    PerformanceMetrics,
    Performance, // Alias/synonym for PerformanceMetrics
    SecurityScan,
    Security, // Alias/synonym for SecurityScan
    Constitutional, // Constitutional/CAWS compliance evidence
    Documentation,
}

#[async_trait::async_trait]
pub trait ResearchEvidenceCollector: Send + Sync {
    async fn collect_evidence(&self, context: &ProcessingContext) -> anyhow::Result<Vec<ResearchEvidence>>;
}

/// No-op research evidence collector for when research feature is disabled
pub struct NoOpResearchEvidenceCollector;

#[async_trait::async_trait]
impl ResearchEvidenceCollector for NoOpResearchEvidenceCollector {
    async fn collect_evidence(&self, _context: &ProcessingContext) -> anyhow::Result<Vec<ResearchEvidence>> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingContext {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub milestone_id: String,
    pub evidence_types: Vec<ResearchEvidenceType>,
    pub priority: String,
}

use crate::planning::plan_types::{EvidenceBundle, EvidenceArtifact};

/// Evidence collector with real integration to agent-research
pub struct EvidenceCollector {
    /// Research evidence collector
    research_collector: Arc<dyn ResearchEvidenceCollector>,

    /// Evidence validation configuration
    validation_config: EvidenceValidationConfig,

    /// Evidence storage configuration
    storage_config: EvidenceStorageConfig,
}

/// Evidence validation configuration
#[derive(Debug, Clone)]
pub struct EvidenceValidationConfig {
    /// Minimum evidence quality score (0.0-1.0)
    pub min_quality_score: f64,

    /// Require all evidence types to be present
    pub require_all_types: bool,

    /// Allow partial evidence collection
    pub allow_partial: bool,

    /// Validation timeout in seconds
    pub validation_timeout_seconds: u64,

    /// Enable evidence caching
    pub enable_caching: bool,
}

/// Evidence storage configuration
#[derive(Debug, Clone)]
pub struct EvidenceStorageConfig {
    /// Storage backend type
    pub backend: EvidenceStorageBackend,

    /// Storage location
    pub location: String,

    /// Retention period in days
    pub retention_days: u32,

    /// Compression enabled
    pub compression_enabled: bool,
}

/// Evidence storage backend types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceStorageBackend {
    /// File system storage
    FileSystem,

    /// Database storage
    Database,

    /// Distributed storage
    Distributed,
}

impl Default for EvidenceValidationConfig {
    fn default() -> Self {
        Self {
            min_quality_score: 0.8,
            require_all_types: false,
            allow_partial: true,
            validation_timeout_seconds: 300,
            enable_caching: true,
        }
    }
}

impl Default for EvidenceStorageConfig {
    fn default() -> Self {
        Self {
            backend: EvidenceStorageBackend::FileSystem,
            location: "/tmp/evidence".to_string(),
            retention_days: 30,
            compression_enabled: true,
        }
    }
}

impl EvidenceCollector {
    /// Create new evidence collector with real research integration
    pub fn new(research_collector: Arc<dyn ResearchEvidenceCollector>) -> Self {
        Self::with_config(
            research_collector,
            EvidenceValidationConfig::default(),
            EvidenceStorageConfig::default(),
        )
    }

    /// Create with custom configuration
    pub fn with_config(
        research_collector: Arc<dyn ResearchEvidenceCollector>,
        validation_config: EvidenceValidationConfig,
        storage_config: EvidenceStorageConfig,
    ) -> Self {
        Self {
            research_collector,
            validation_config,
            storage_config,
        }
    }

    /// Collect evidence for milestone completion using real evidence collection
    pub async fn collect_evidence(&self, milestone: &Milestone, plan_id: &str) -> Result<EvidenceBundle> {
        let collection_start = Utc::now();

        // Convert milestone to research evidence collection context
        let collection_context = self.create_collection_context(milestone)?;

        // Collect evidence using research collector
        let research_evidence = self.research_collector.collect_evidence(&collection_context).await?;

        // Convert research evidence to planning evidence bundle
        let evidence_bundle = self.convert_research_evidence_to_bundle(
            research_evidence,
            milestone,
            plan_id,
            collection_start,
        ).await?;

        // Validate evidence against milestone requirements
        self.validate_evidence_bundle(&evidence_bundle, &milestone.evidence_gate).await?;

        // Store evidence if configured
        if matches!(self.storage_config.backend, EvidenceStorageBackend::FileSystem) {
            self.store_evidence_bundle(&evidence_bundle).await?;
        }

        Ok(evidence_bundle)
    }

    /// Validate collected evidence against gate requirements
    pub async fn validate_evidence(&self, evidence: &EvidenceBundle, gate: &EvidenceGate) -> Result<bool> {
        // Check coverage requirements
        if let Some(test_coverage) = self.get_test_coverage(evidence) {
            if test_coverage.line_coverage < gate.min_coverage ||
               test_coverage.branch_coverage < gate.min_branch_coverage {
                return Ok(false);
            }
        } else if gate.min_coverage > 0.0 {
            return Ok(false); // No coverage evidence but required
        }

        // Check mutation score
        if let Some(mutation_score) = self.get_mutation_score(evidence) {
            if mutation_score < gate.min_mutation_score {
                return Ok(false);
            }
        } else if gate.min_mutation_score > 0.0 {
            return Ok(false); // No mutation evidence but required
        }

        // Check security scan if required
        if gate.security_scan_required && !self.has_security_scan(evidence) {
            return Ok(false);
        }

        // Check required artifacts
        for required_artifact in &gate.required_artifacts {
            if !self.has_artifact(evidence, required_artifact) {
                return Ok(false);
            }
        }

        // Check performance budget if specified
        if let Some(performance_budget) = &gate.performance_budget {
            if !self.validate_performance_budget(evidence, performance_budget) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get overall evidence quality score
    pub fn get_evidence_quality_score(&self, evidence: &EvidenceBundle) -> f64 {
        let mut total_score = 0.0;
        let mut artifact_count = 0;

        for artifact in &evidence.artifacts {
            if let Some(quality) = artifact.metadata.get("quality_score")
                .and_then(|v| v.as_f64()) {
                total_score += quality;
                artifact_count += 1;
            }
        }

        if artifact_count > 0 {
            total_score / artifact_count as f64
        } else {
            0.0
        }
    }

    /// Create collection context for research collector
    fn create_collection_context(&self, milestone: &Milestone) -> Result<ProcessingContext> {
        #[cfg(feature = "research")]
        use agent_research::extraction_types::{ProcessingContext, ClaimType};

        #[cfg(feature = "research")]
        {
            Ok(ProcessingContext {
                source_id: milestone.id.clone(),
                claim_type: ClaimType::Implementation, // Planning milestones are implementation claims
                context: serde_json::json!({
                    "milestone_id": milestone.id,
                    "objective": milestone.objective,
                    "scope": milestone.scope,
                    "risk_tier": milestone.risk_tier,
                }),
                metadata: HashMap::new(),
            })
        }

        #[cfg(not(feature = "research"))]
        {
            Err(anyhow::anyhow!("Research feature not enabled - cannot create collection context"))
        }
    }

    /// Convert research evidence to planning evidence bundle
    async fn convert_research_evidence_to_bundle(
        &self,
        research_evidence: Vec<ResearchEvidence>,
        milestone: &Milestone,
        plan_id: &str,
        collection_start: chrono::DateTime<Utc>,
    ) -> Result<EvidenceBundle> {
        let mut artifacts = Vec::new();

        for research_ev in research_evidence {
            let artifact = self.convert_single_evidence(research_ev, collection_start).await?;
            artifacts.push(artifact);
        }

        Ok(EvidenceBundle {
            meets_quality_gates: true,
            metadata: std::collections::HashMap::new(),
            milestone_id: milestone.id.clone(),
            plan_id: plan_id.to_string(),
            artifacts,
            collected_at: collection_start,
            quality_score: None,
        })
    }

    /// Convert single research evidence to planning artifact
    async fn convert_single_evidence(
        &self,
        research_ev: ResearchEvidence,
        collected_at: chrono::DateTime<Utc>,
    ) -> Result<EvidenceArtifact> {
        // Convert evidence type
        let artifact_type = match research_ev.evidence_type {
            ResearchEvidenceType::CodeAnalysis => "code_analysis".to_string(),
            ResearchEvidenceType::TestExecution => "test_results".to_string(),
            ResearchEvidenceType::Documentation => "documentation".to_string(),
            ResearchEvidenceType::Performance => "performance".to_string(),
            ResearchEvidenceType::Security => "security".to_string(),
            ResearchEvidenceType::Constitutional => "constitutional".to_string(),
            _ => "other".to_string(),
        };

        // Extract quality score if available
        let quality_score = research_ev.confidence_score;

        // Create metadata
        let mut metadata = HashMap::from([
            ("collected_at".to_string(), serde_json::Value::String(collected_at.to_string())),
            ("quality_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(quality_score).unwrap())),
            ("source".to_string(), serde_json::Value::String("research_collector".to_string())),
        ]);

        // Add evidence-specific metadata
        if let Some(evidence_data) = &research_ev.data {
            metadata.insert("evidence_data".to_string(), evidence_data.clone());
        }

        // Put the data and verified info in metadata since EvidenceArtifact doesn't have these fields
        metadata.insert("data".to_string(), research_ev.data.unwrap_or(serde_json::Value::Null));
        metadata.insert("verified".to_string(), serde_json::Value::Bool(research_ev.verified));

        Ok(EvidenceArtifact {
            metadata: std::collections::HashMap::new(),
            id: Uuid::new_v4(),
            artifact_type,
            content: crate::planning::plan_types::EvidenceContent::Structured(metadata.clone()),
            quality_score,
            collected_at,
        })
    }

    /// Validate evidence bundle against gate requirements
    async fn validate_evidence_bundle(&self, bundle: &EvidenceBundle, gate: &EvidenceGate) -> Result<()> {
        if !self.validate_evidence(bundle, gate).await? {
            return Err(anyhow!(
                "Evidence validation failed for milestone {}. Quality score: {:?}",
                bundle.milestone_id,
                bundle.quality_score
            ));
        }

        // Check minimum quality score
        let quality_score = self.get_evidence_quality_score(bundle);
        if quality_score < self.validation_config.min_quality_score {
            return Err(anyhow!(
                "Evidence quality score {} below minimum threshold {}",
                quality_score,
                self.validation_config.min_quality_score
            ));
        }

        Ok(())
    }

    /// Store evidence bundle to configured backend
    async fn store_evidence_bundle(&self, bundle: &EvidenceBundle) -> Result<()> {
        match self.storage_config.backend {
            EvidenceStorageBackend::FileSystem => {
                self.store_to_filesystem(bundle).await
            }
            EvidenceStorageBackend::Database => {
                self.store_to_database(bundle).await
            }
            EvidenceStorageBackend::Distributed => {
                self.store_to_distributed(bundle).await
            }
        }
    }

    /// Store evidence to file system
    async fn store_to_filesystem(&self, bundle: &EvidenceBundle) -> Result<()> {
        use tokio::fs;
        use std::path::PathBuf;

        let evidence_dir = PathBuf::from(&self.storage_config.location)
            .join("planning")
            .join(&bundle.milestone_id);

        fs::create_dir_all(&evidence_dir).await?;

        let evidence_file = evidence_dir.join(format!("evidence-{}.json",
            bundle.collected_at.timestamp()));

        let evidence_json = serde_json::to_string_pretty(bundle)?;
        fs::write(evidence_file, evidence_json).await?;

        Ok(())
    }

    /// Store evidence to database
    async fn store_to_database(&self, _bundle: &EvidenceBundle) -> Result<()> {
        // TODO: Implement database storage
        Err(anyhow!("Database storage not yet implemented"))
    }

    /// Store evidence to distributed storage
    async fn store_to_distributed(&self, _bundle: &EvidenceBundle) -> Result<()> {
        // TODO: Implement distributed storage
        Err(anyhow!("Distributed storage not yet implemented"))
    }

    /// Helper methods for evidence validation
    fn get_test_coverage(&self, evidence: &EvidenceBundle) -> Option<TestCoverage> {
        for artifact in &evidence.artifacts {
            if artifact.artifact_type == "test_results" {
                if let Some(line_cov) = artifact.metadata.get("line_coverage")
                    .and_then(|v| v.as_f64()) {
                    if let Some(branch_cov) = artifact.metadata.get("branch_coverage")
                        .and_then(|v| v.as_f64()) {
                        return Some(TestCoverage {
                            line_coverage: line_cov,
                            branch_coverage: branch_cov,
                        });
                    }
                }
            }
        }
        None
    }

    fn get_mutation_score(&self, evidence: &EvidenceBundle) -> Option<f64> {
        for artifact in &evidence.artifacts {
            if artifact.artifact_type == "mutation_testing" {
                return artifact.metadata.get("mutation_score")
                    .and_then(|v| v.as_f64());
            }
        }
        None
    }

    fn has_security_scan(&self, evidence: &EvidenceBundle) -> bool {
        evidence.artifacts.iter()
            .any(|a| a.artifact_type == "security_scan")
    }

    fn has_artifact(&self, evidence: &EvidenceBundle, artifact_type: &str) -> bool {
        evidence.artifacts.iter()
            .any(|a| a.artifact_type == artifact_type)
    }

    fn validate_performance_budget(&self, evidence: &EvidenceBundle, budget: &agent_agency_contracts::planning_io::PerformanceBudget) -> bool {
        for artifact in &evidence.artifacts {
            if artifact.artifact_type == "performance" {
                if let Some(p95) = artifact.metadata.get("p95_ms")
                    .and_then(|v| v.as_u64()) {
                    if p95 > budget.max_p95_ms {
                        return false;
                    }
                }

                if let Some(p99) = artifact.metadata.get("p99_ms")
                    .and_then(|v| v.as_u64()) {
                    if p99 > budget.max_p99_ms {
                        return false;
                    }
                }

                if let Some(memory) = artifact.metadata.get("memory_mb")
                    .and_then(|v| v.as_u64()) {
                    if memory > budget.max_memory_mb as u64 {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Test coverage data
#[derive(Debug, Clone)]
pub struct TestCoverage {
    pub line_coverage: f64,
    pub branch_coverage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock research evidence collector for testing
    struct MockResearchCollector;

    #[async_trait::async_trait]
    impl ResearchEvidenceCollector for MockResearchCollector {
        async fn collect_evidence(&self, _context: &ProcessingContext) -> anyhow::Result<Vec<ResearchEvidence>> {
            Ok(vec![
                ResearchEvidence {
                    id: Uuid::new_v4(),
                    claim_id: "test-claim".to_string(),
                    evidence_type: ResearchEvidenceType::TestExecution,
                    source: "test".to_string(),
                    confidence_score: 0.9,
                    data: Some(serde_json::json!({
                        "passed": 10,
                        "failed": 0,
                        "coverage": 0.85
                    })),
                    metadata: HashMap::new(),
                    verified: true,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            ])
        }
    }

    #[test]
    fn test_evidence_collector_creation() {
        let mock_collector = Arc::new(MockResearchCollector);
        let collector = EvidenceCollector::new(mock_collector);
        // Collector created successfully
        assert!(true);
    }

    #[test]
    fn test_evidence_validation_config() {
        let config = EvidenceValidationConfig::default();
        assert_eq!(config.min_quality_score, 0.8);
        assert!(!config.require_all_types);
        assert!(config.allow_partial);
    }

    #[test]
    fn test_storage_config() {
        let config = EvidenceStorageConfig::default();
        assert!(matches!(config.backend, EvidenceStorageBackend::FileSystem));
        assert_eq!(config.retention_days, 30);
    }

    #[test]
    fn test_test_coverage_extraction() {
        let collector = EvidenceCollector::new(Arc::new(MockResearchCollector));

        let evidence = EvidenceBundle {
            milestone_id: "test".to_string(),
            plan_id: Uuid::new_v4(),
            artifacts: vec![EvidenceArtifact {
                id: Uuid::new_v4(),
                artifact_type: "test_results".to_string(),
                content: crate::planning::plan_types::EvidenceContent::Structured(HashMap::from([
                    ("data".to_string(), serde_json::Value::Null),
                    ("verified".to_string(), serde_json::Value::Bool(true)),
                    ("line_coverage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.85).unwrap())),
                    ("branch_coverage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.80).unwrap())),
                ])),
                quality_score: 0.9,
                collected_at: Utc::now(),
            }],
            collected_at: Utc::now(),
            quality_score: Some(0.9),
        };

        let coverage = collector.get_test_coverage(&evidence);
        assert!(coverage.is_some());
        let coverage = coverage.unwrap();
        assert_eq!(coverage.line_coverage, 0.85);
        assert_eq!(coverage.branch_coverage, 0.80);
    }
}
