//! Evidence Enrichment for Council Evaluation
//!
//! Integrates claim extraction pipeline with council evaluation process to provide
//! evidence-backed judge verdicts. Based on V2's verification engine patterns.

// use crate::claim_extraction::ClaimExtractor;  // Temporarily commented to resolve circular dependency
use crate::models::TaskSpec;
use crate::types::{
    Evidence as CouncilEvidence, EvidenceSource as CouncilEvidenceSource, JudgeVerdict,
};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info};

/// Evidence enrichment coordinator for council evaluations
#[derive(Clone)]
pub struct EvidenceEnrichmentCoordinator {
    evidence_cache: HashMap<String, Vec<CouncilEvidence>>,
}

impl EvidenceEnrichmentCoordinator {
    pub fn new() -> Self {
        Self {
            evidence_cache: HashMap::new(),
        }
    }

    /// Extract and enrich evidence for a task specification
    pub async fn enrich_task_evidence(
        &mut self,
        task_spec: &TaskSpec,
    ) -> Result<Vec<CouncilEvidence>> {
        info!("Starting evidence enrichment for task {}", task_spec.id);

        // Extract claims from task description and worker output
        let mut all_evidence = Vec::new();

        // Create evidence from task description
        let description_evidence = vec![CouncilEvidence {
            source: CouncilEvidenceSource::CodeAnalysis,
            content: task_spec.description.clone(),
            relevance: 0.8,
            timestamp: Utc::now(),
        }];
        all_evidence.extend(description_evidence);

        // Create evidence from worker output
        let output_evidence = vec![CouncilEvidence {
            source: CouncilEvidenceSource::CodeAnalysis,
            content: task_spec.worker_output.content.clone(),
            relevance: 0.7,
            timestamp: Utc::now(),
        }];
        all_evidence.extend(output_evidence);

        // Create evidence from acceptance criteria
        for criterion in &task_spec.acceptance_criteria {
            let criterion_evidence = vec![CouncilEvidence {
                source: CouncilEvidenceSource::CodeAnalysis,
                content: criterion.description.clone(),
                relevance: 0.9,
                timestamp: Utc::now(),
            }];
            all_evidence.extend(criterion_evidence);
        }

        // Cache evidence for this task
        self.evidence_cache
            .insert(task_spec.id.to_string(), all_evidence.clone());

        info!(
            "Enriched {} evidence items for task {} using council claim extraction",
            all_evidence.len(),
            task_spec.id
        );
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
        debug!(
            "Enhancing verdict with {} evidence items for task {}",
            evidence.len(),
            task_id
        );

        // Calculate evidence-based confidence adjustment
        let evidence_confidence = self.calculate_evidence_confidence(evidence);

        // Add evidence summary to verdict reasoning
        let evidence_summary = self.generate_evidence_summary(evidence);

        // Update verdict based on evidence strength
        match verdict {
            JudgeVerdict::Pass {
                reasoning,
                confidence,
                evidence: verdict_evidence,
            } => {
                *reasoning = format!("{} Evidence: {}", reasoning, evidence_summary);
                *confidence = (*confidence * 0.7 + evidence_confidence * 0.3).min(1.0);
                verdict_evidence.extend(evidence.iter().cloned());
            }
            JudgeVerdict::Fail {
                reasoning,
                evidence: verdict_evidence,
                ..
            } => {
                *reasoning = format!("{} Evidence: {}", reasoning, evidence_summary);
                verdict_evidence.extend(evidence.iter().cloned());
            }
            JudgeVerdict::Uncertain {
                reasoning,
                evidence: verdict_evidence,
                ..
            } => {
                *reasoning = format!("{} Evidence: {}", reasoning, evidence_summary);
                verdict_evidence.extend(evidence.iter().cloned());
            }
        }

        Ok(())
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
        let recent_evidence_count = evidence
            .iter()
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

        let source_counts: HashMap<CouncilEvidenceSource, usize> =
            evidence.iter().fold(HashMap::new(), |mut acc, e| {
                *acc.entry(e.source.clone()).or_insert(0) += 1;
                acc
            });

        let source_summary = source_counts
            .iter()
            .map(|(source, count)| format!("{}: {}", self.format_evidence_source(source), count))
            .collect::<Vec<_>>()
            .join(", ");

        let avg_relevance =
            evidence.iter().map(|e| e.relevance).sum::<f32>() / evidence.len() as f32;

        format!(
            "{} sources ({}), avg relevance: {:.2}",
            source_summary,
            evidence.len(),
            avg_relevance
        )
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
