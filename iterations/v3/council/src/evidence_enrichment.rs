//! Evidence Enrichment for Council Evaluation
//! 
//! Integrates claim extraction pipeline with council evaluation process to provide
//! evidence-backed judge verdicts. Based on V2's verification engine patterns.

use crate::types::{JudgeVerdict, Evidence as CouncilEvidence, EvidenceSource as CouncilEvidenceSource};
use crate::models::TaskSpec;
use claim_extraction::{
    ClaimExtractionAndVerificationProcessor, 
    ProcessingContext, 
    AtomicClaim, 
    Evidence as ClaimEvidence,
    EvidenceType as ClaimEvidenceType,
    SourceType as ClaimSourceType,
};
use anyhow::Result;
use tracing::{info, debug, warn};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

/// Evidence enrichment coordinator for council evaluations
#[derive(Debug)]
pub struct EvidenceEnrichmentCoordinator {
    claim_processor: ClaimExtractionAndVerificationProcessor,
    evidence_cache: HashMap<String, Vec<CouncilEvidence>>,
}

impl EvidenceEnrichmentCoordinator {
    pub fn new() -> Self {
        Self {
            claim_processor: ClaimExtractionAndVerificationProcessor::new(),
            evidence_cache: HashMap::new(),
        }
    }

    /// Extract and enrich evidence for a task specification
    pub async fn enrich_task_evidence(
        &mut self,
        task_spec: &TaskSpec,
    ) -> Result<Vec<CouncilEvidence>> {
        info!("Starting evidence enrichment for task {}", task_spec.id);

        // Create processing context from task spec
        let context = self.create_processing_context(task_spec);

        // Extract claims from task description and worker output
        let mut all_evidence = Vec::new();

        // Extract claims from task description
        if let Ok(description_result) = self.claim_processor.process_sentence(&task_spec.description, &context).await {
            let description_evidence = self.convert_claim_evidence_to_council_evidence(
                &description_result.verification_evidence,
                "task_description",
            );
            all_evidence.extend(description_evidence);
        }

        // Extract claims from worker output if available
        let output_text = format!("{}", task_spec.worker_output.content);
        if let Ok(output_result) = self.claim_processor.process_sentence(&output_text, &context).await {
            let output_evidence = self.convert_claim_evidence_to_council_evidence(
                &output_result.verification_evidence,
                "worker_output",
            );
            all_evidence.extend(output_evidence);
        }

        // Extract claims from acceptance criteria
        for criterion in &task_spec.acceptance_criteria {
            if let Ok(criterion_result) = self.claim_processor.process_sentence(&criterion.description, &context).await {
                let criterion_evidence = self.convert_claim_evidence_to_council_evidence(
                    &criterion_result.verification_evidence,
                    &format!("acceptance_criterion_{}", criterion.id),
                );
                all_evidence.extend(criterion_evidence);
            }
        }

        // Cache evidence for this task
        self.evidence_cache.insert(task_spec.id.to_string(), all_evidence.clone());

        info!("Enriched {} evidence items for task {}", all_evidence.len(), task_spec.id);
        Ok(all_evidence)
    }

    /// Get cached evidence for a task
    pub fn get_cached_evidence(&self, task_id: &str) -> Option<&Vec<CouncilEvidence>> {
        self.evidence_cache.get(task_id)
    }

    /// Enhance a judge verdict with evidence-based reasoning
    pub async fn enhance_verdict_with_evidence(
        &self,
        verdict: &mut JudgeVerdict,
        task_id: &str,
        evidence: &[CouncilEvidence],
    ) -> Result<()> {
        debug!("Enhancing verdict with {} evidence items for task {}", evidence.len(), task_id);

        // Calculate evidence-based confidence adjustment
        let evidence_confidence = self.calculate_evidence_confidence(evidence);
        
        // Add evidence summary to verdict reasoning
        let evidence_summary = self.generate_evidence_summary(evidence);
        
        // Update verdict based on evidence strength
        match verdict {
            JudgeVerdict::Pass { reasoning, confidence, evidence: verdict_evidence } => {
                *reasoning = format!("{} Evidence: {}", reasoning, evidence_summary);
                *confidence = (*confidence * 0.7 + evidence_confidence * 0.3).min(1.0);
                verdict_evidence.extend(evidence.iter().cloned());
            }
            JudgeVerdict::Fail { reasoning, evidence: verdict_evidence, .. } => {
                *reasoning = format!("{} Evidence: {}", reasoning, evidence_summary);
                verdict_evidence.extend(evidence.iter().cloned());
            }
            JudgeVerdict::Uncertain { reasoning, evidence: verdict_evidence, .. } => {
                *reasoning = format!("{} Evidence: {}", reasoning, evidence_summary);
                verdict_evidence.extend(evidence.iter().cloned());
            }
        }

        Ok(())
    }

    /// Create processing context from task specification
    fn create_processing_context(&self, task_spec: &TaskSpec) -> ProcessingContext {
        ProcessingContext {
            task_id: task_spec.id,
            working_spec_id: task_spec.caws_spec.as_ref()
                .map(|spec| spec.working_spec_path.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            source_file: task_spec.scope.files_affected.first().cloned(),
            line_number: None,
            surrounding_context: task_spec.description.clone(),
            domain_hints: task_spec.scope.domains.clone(),
        }
    }

    /// Convert claim extraction evidence to council evidence format
    fn convert_claim_evidence_to_council_evidence(
        &self,
        claim_evidence: &[ClaimEvidence],
        source_context: &str,
    ) -> Vec<CouncilEvidence> {
        claim_evidence
            .iter()
            .map(|evidence| CouncilEvidence {
                source: self.convert_evidence_source(&evidence.source.source_type),
                content: evidence.content.clone(),
                relevance: evidence.confidence as f32,
                timestamp: evidence.timestamp,
            })
            .collect()
    }

    /// Convert claim extraction source type to council evidence source
    fn convert_evidence_source(&self, source_type: &ClaimSourceType) -> CouncilEvidenceSource {
        match source_type {
            ClaimSourceType::FileSystem => CouncilEvidenceSource::CodeAnalysis,
            ClaimSourceType::TestSuite => CouncilEvidenceSource::TestResults,
            ClaimSourceType::Database => CouncilEvidenceSource::HistoricalData,
            ClaimSourceType::Web => CouncilEvidenceSource::ResearchAgent,
            ClaimSourceType::ResearchAgent => CouncilEvidenceSource::ResearchAgent,
            ClaimSourceType::CouncilDecision => CouncilEvidenceSource::ExpertKnowledge,
        }
    }

    /// Calculate overall evidence confidence score
    fn calculate_evidence_confidence(&self, evidence: &[CouncilEvidence]) -> f32 {
        if evidence.is_empty() {
            return 0.0;
        }

        let total_relevance: f32 = evidence.iter().map(|e| e.relevance).sum();
        let avg_relevance = total_relevance / evidence.len() as f32;

        // Bonus for multiple evidence sources
        let source_diversity_bonus = if evidence.len() > 1 {
            0.1 * (evidence.len() as f32 - 1.0).min(3.0) / 3.0
        } else {
            0.0
        };

        // Bonus for recent evidence
        let now = Utc::now();
        let recent_evidence_count = evidence.iter()
            .filter(|e| (now - e.timestamp).num_hours() < 24)
            .count();
        let recency_bonus = if recent_evidence_count > 0 {
            0.05 * (recent_evidence_count as f32 / evidence.len() as f32)
        } else {
            0.0
        };

        (avg_relevance + source_diversity_bonus + recency_bonus).min(1.0)
    }

    /// Generate a summary of evidence for verdict reasoning
    fn generate_evidence_summary(&self, evidence: &[CouncilEvidence]) -> String {
        if evidence.is_empty() {
            return "No supporting evidence found".to_string();
        }

        let source_counts: HashMap<CouncilEvidenceSource, usize> = evidence
            .iter()
            .fold(HashMap::new(), |mut acc, e| {
                *acc.entry(e.source.clone()).or_insert(0) += 1;
                acc
            });

        let source_summary = source_counts
            .iter()
            .map(|(source, count)| format!("{}: {}", self.format_evidence_source(source), count))
            .collect::<Vec<_>>()
            .join(", ");

        let avg_relevance = evidence.iter().map(|e| e.relevance).sum::<f32>() / evidence.len() as f32;

        format!("{} sources ({}), avg relevance: {:.2}", 
                source_summary, evidence.len(), avg_relevance)
    }

    /// Format evidence source for human-readable output
    fn format_evidence_source(&self, source: &CouncilEvidenceSource) -> &'static str {
        match source {
            CouncilEvidenceSource::CodeAnalysis => "code analysis",
            CouncilEvidenceSource::TestResults => "test results",
            CouncilEvidenceSource::Documentation => "documentation",
            CouncilEvidenceSource::CAWSRules => "CAWS rules",
            CouncilEvidenceSource::HistoricalData => "historical data",
            CouncilEvidenceSource::ExpertKnowledge => "expert knowledge",
            CouncilEvidenceSource::ResearchAgent => "research agent",
        }
    }
}

impl Default for EvidenceEnrichmentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
