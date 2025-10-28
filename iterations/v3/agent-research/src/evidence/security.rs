//! Security scanning evidence collection

use super::types::*;
use crate::extraction_types::{AtomicClaim, Evidence, EvidenceType, EvidenceSource, ProcessingContext};
use crate::evidence::evidence_types::EvidenceCollectorConfig;
use anyhow::Result;

/// Security collector
#[derive(Debug)]
pub struct SecurityCollector {
    config: EvidenceCollectorConfig,
}

impl SecurityCollector {
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
            evidence_type: EvidenceType::SecurityScan,
            content: "Security scanning evidence collection not yet implemented".to_string(),
            source: EvidenceSource::CodeSearch {
                location: "security".to_string(),
                authority: "security-scan".to_string(),
                freshness: chrono::Utc::now(),
            },
            confidence: 0.8,
            relevance: 0.9,
            timestamp: chrono::Utc::now(),
        }])
    }
}
