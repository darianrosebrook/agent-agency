//! Evidence filtering and ranking logic

use super::types::*;
use crate::types::{AtomicClaim, Evidence, EvidenceType};
use std::collections::HashMap;

/// Evidence filter and ranking engine
#[derive(Debug)]
pub struct EvidenceFilter {
    config: EvidenceCollectorConfig,
}

impl EvidenceFilter {
    pub fn new() -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
        }
    }

    pub fn with_config(config: EvidenceCollectorConfig) -> Self {
        Self { config }
    }

    /// Filter and rank evidence based on quality and relevance
    pub fn filter_and_rank_evidence(&self, evidence: Vec<Evidence>, claim: &AtomicClaim) -> Vec<Evidence> {
        // Filter out low-quality evidence
        let filtered: Vec<Evidence> = evidence.into_iter()
            .filter(|e| {
                e.confidence >= self.config.min_credibility_threshold &&
                e.relevance >= self.config.min_relevance_threshold
            })
            .take(self.config.max_evidence_per_claim)
            .collect();

        // Rank evidence by quality score
        let mut ranked = filtered;
        ranked.sort_by(|a, b| {
            let score_a = self.calculate_evidence_score(a);
            let score_b = self.calculate_evidence_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        ranked
    }

    /// Calculate overall quality score for evidence
    fn calculate_evidence_score(&self, evidence: &Evidence) -> f64 {
        // Simple scoring based on confidence and relevance
        evidence.confidence * 0.6 + evidence.relevance * 0.4
    }
}
