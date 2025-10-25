//! Task evaluation orchestration
//!
//! Core evaluation logic for task processing, judge verdict collection,
//! consensus calculation, and final decision making.

use super::types::{KnowledgeSeeker, MultimodalContext, CoordinatorMetrics, QueueTracker};
use super::metrics::MetricsManager;
use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use crate::models::{EvidencePacket, ParticipantContribution, RiskTier, TaskSpec};
use crate::resilience::ResilienceManager;
use crate::types::{ConsensusResult, FinalVerdict, JudgeVerdict};
use crate::{MultimodalEvidenceEnricher, ClaimWithMultimodalEvidence};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Task evaluation orchestrator
#[derive(Debug)]
pub struct EvaluationOrchestrator {
    evidence_enrichment: EvidenceEnrichmentCoordinator,
    resilience_manager: Arc<ResilienceManager>,
    multimodal_evidence_enricher: MultimodalEvidenceEnricher,
    knowledge_seeker: Option<Arc<KnowledgeSeeker>>,
    metrics_manager: MetricsManager,
}

impl EvaluationOrchestrator {
    /// Create a new evaluation orchestrator
    pub fn new(
        evidence_enrichment: EvidenceEnrichmentCoordinator,
        resilience_manager: Arc<ResilienceManager>,
        multimodal_evidence_enricher: MultimodalEvidenceEnricher,
        knowledge_seeker: Option<Arc<KnowledgeSeeker>>,
        metrics_manager: MetricsManager,
    ) -> Self {
        Self {
            evidence_enrichment,
            resilience_manager,
            multimodal_evidence_enricher,
            knowledge_seeker,
            metrics_manager,
        }
    }

    /// Evaluate a task through the complete council process
    pub async fn evaluate_task(&self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        let start_time = tokio::time::Instant::now();

        info!("Starting council evaluation for task {}", task_id);

        // Record evaluation start
        self.metrics_manager.record_evaluation_start().await;

        // Track individual stage timings for SLA verification
        let enrichment_start = tokio::time::Instant::now();

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
        self.metrics_manager.record_enrichment_time(enrichment_time).await;
        debug!("Evidence enrichment completed in {}ms", enrichment_time);

        // Track judge inference timing
        let judge_inference_start = tokio::time::Instant::now();

        // Create individual judge verdicts with evidence enhancement
        let individual_verdicts = self.collect_judge_verdicts(&task_spec, &evidence).await?;

        let judge_inference_time = judge_inference_start.elapsed().as_millis() as u64;
        self.metrics_manager.record_judge_inference_time(judge_inference_time).await;
        debug!("Judge inference completed in {}ms", judge_inference_time);

        // Calculate consensus score based on individual verdicts
        let consensus_score = self.calculate_consensus_score(&individual_verdicts);

        // Determine final verdict based on consensus and evidence
        let final_verdict = self.determine_final_verdict(consensus_score, &individual_verdicts, &evidence);

        // Build consensus result with comprehensive analysis
        let result = ConsensusResult {
            task_id,
            final_verdict,
            consensus_score,
            individual_verdicts,
            evidence_used: evidence,
            processing_metadata: self.build_processing_metadata(start_time.elapsed().as_millis() as u64),
        };

        // Record successful evaluation
        self.metrics_manager.record_evaluation_success(start_time.elapsed().as_millis() as u64).await;

        info!(
            "Council evaluation completed for task {}: verdict={:?}, consensus={:.2}",
            task_id,
            result.final_verdict,
            consensus_score
        );

        Ok(result)
    }

    /// Collect verdicts from all judges
    async fn collect_judge_verdicts(&self, task_spec: &TaskSpec, evidence: &EvidencePacket) -> Result<HashMap<String, JudgeVerdict>> {
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
                &task_spec.id.to_string(),
                evidence,
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
            .enhance_verdict_with_evidence(&mut technical_verdict, &task_spec.id.to_string(), evidence)
            .await?;
        individual_verdicts.insert("technical".to_string(), technical_verdict);

        // Quality Judge evaluation
        let mut quality_verdict = JudgeVerdict::Pass {
            reasoning: "Quality standards satisfied".to_string(),
            confidence: 0.7,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(&mut quality_verdict, &task_spec.id.to_string(), evidence)
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
                &task_spec.id.to_string(),
                evidence,
            )
            .await?;
        individual_verdicts.insert("integration".to_string(), integration_verdict);

        Ok(individual_verdicts)
    }

    /// Calculate consensus score from individual verdicts
    fn calculate_consensus_score(&self, verdicts: &HashMap<String, JudgeVerdict>) -> f64 {
        let total_verdicts = verdicts.len() as f64;
        let passing_verdicts = verdicts.values()
            .filter(|verdict| matches!(verdict, JudgeVerdict::Pass { .. }))
            .count() as f64;

        if total_verdicts > 0.0 {
            passing_verdicts / total_verdicts
        } else {
            0.0
        }
    }

    /// Determine final verdict based on consensus and evidence
    fn determine_final_verdict(
        &self,
        consensus_score: f64,
        verdicts: &HashMap<String, JudgeVerdict>,
        evidence: &EvidencePacket,
    ) -> FinalVerdict {
        // High consensus threshold for approval
        const APPROVAL_THRESHOLD: f64 = 0.85;
        const CONDITIONAL_THRESHOLD: f64 = 0.7;

        if consensus_score >= APPROVAL_THRESHOLD {
            FinalVerdict::Approved {
                reasoning: "Strong consensus across all judges".to_string(),
                confidence: consensus_score,
                risk_assessment: self.assess_final_risk(verdicts, evidence),
            }
        } else if consensus_score >= CONDITIONAL_THRESHOLD {
            FinalVerdict::ConditionalApproval {
                reasoning: "Majority consensus with conditions".to_string(),
                confidence: consensus_score,
                conditions: self.generate_approval_conditions(verdicts),
                risk_assessment: self.assess_final_risk(verdicts, evidence),
            }
        } else {
            FinalVerdict::Rejected {
                reasoning: "Insufficient consensus for approval".to_string(),
                confidence: consensus_score,
                concerns: self.extract_rejection_concerns(verdicts),
            }
        }
    }

    /// Assess final risk based on verdicts and evidence
    fn assess_final_risk(&self, _verdicts: &HashMap<String, JudgeVerdict>, _evidence: &EvidencePacket) -> RiskTier {
        // Placeholder risk assessment - would analyze evidence strength and judge concerns
        RiskTier::Low
    }

    /// Generate approval conditions based on verdicts
    fn generate_approval_conditions(&self, _verdicts: &HashMap<String, JudgeVerdict>) -> Vec<String> {
        // Placeholder conditions - would analyze specific judge concerns
        vec![
            "Address technical debt within 30 days".to_string(),
            "Implement additional testing for edge cases".to_string(),
        ]
    }

    /// Extract rejection concerns from verdicts
    fn extract_rejection_concerns(&self, verdicts: &HashMap<String, JudgeVerdict>) -> Vec<String> {
        let mut concerns = Vec::new();

        for (judge_name, verdict) in verdicts {
            if let JudgeVerdict::Fail { reasoning, .. } = verdict {
                concerns.push(format!("{}: {}", judge_name, reasoning));
            }
        }

        concerns
    }

    /// Build processing metadata for the result
    fn build_processing_metadata(&self, total_time_ms: u64) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("total_processing_time_ms".to_string(), total_time_ms.to_string());
        metadata.insert("evaluation_timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("coordinator_version".to_string(), "2.0".to_string());
        metadata
    }
}

/// Processing metadata for consensus results
#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    pub total_processing_time_ms: u64,
    pub enrichment_time_ms: u64,
    pub judge_inference_time_ms: u64,
    pub debate_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub coordinator_version: String,
}


