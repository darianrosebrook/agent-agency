//! Performance measurement evidence collection

use super::types::*;
use crate::extraction_types::{AtomicClaim, Evidence, EvidenceType, EvidenceSource, ProcessingContext};
use crate::evidence::evidence_types::EvidenceCollectorConfig;
use anyhow::Result;

/// Performance collector

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceCollectorr {
    config: EvidenceCollectorConfig,
}

impl PerformanceCollector {
    pub fn new() -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
        }
    }

    pub fn with_config(config: EvidenceCollectorConfig) -> Self {
        Self { config }
    }

    pub async fn collect_evidence(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        Ok(vec![Evidence {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::PerformanceMetrics,
            content: "Performance measurement evidence collection not yet implemented".to_string(),
            source: EvidenceSource::CodeSearch {
                location: "performance".to_string(),
                authority: "performance-measurement".to_string(),
                freshness: chrono::Utc::now(),
            },
            confidence: 0.7,
            relevance: 0.8,
            timestamp: chrono::Utc::now(),
        }])
    }
}
