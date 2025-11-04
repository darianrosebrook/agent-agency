//! Documentation review evidence collection

use super::types::*;
use crate::extraction_types::{AtomicClaim, Evidence, EvidenceType, EvidenceSource, ProcessingContext};
use crate::evidence::evidence_types::EvidenceCollectorConfig;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

/// Documentation collector

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationCollector {
    config: EvidenceCollectorConfig,
}

impl DocumentationCollector {
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
            evidence_type: EvidenceType::Documentation,
            content: "Documentation review evidence collection not yet implemented".to_string(),
            source: EvidenceSource::CodeSearch {
                location: "docs".to_string(),
                authority: "documentation-review".to_string(),
                freshness: chrono::Utc::now(),
            },
            confidence: 0.6,
            relevance: 0.7,
            timestamp: chrono::Utc::now(),
        }])
    }
}
