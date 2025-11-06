//! Constitutional checking evidence collection

use super::common::{EvidenceCollector, CollectorCtx, helpers};
use super::types::*;
use crate::extraction_types::{AtomicClaim, Evidence, EvidenceType, ProcessingContext};
use crate::evidence::evidence_types::EvidenceCollectorConfig;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

/// Constitutional collector
#[derive(Debug, Serialize, Deserialize)]
pub struct ConstitutionalCollector {
    config: EvidenceCollectorConfig,
}

#[async_trait::async_trait]
impl EvidenceCollector for ConstitutionalCollector {
    type Input = AtomicClaim;
    type Output = Vec<Evidence>;

    fn name(&self) -> &'static str { "constitutional" }

    fn config(&self) -> &EvidenceCollectorConfig {
        &self.config
    }

    async fn collect(&self, claim: &AtomicClaim, ctx: &CollectorCtx) -> Result<Vec<Evidence>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        // Check timeout
        if ctx.should_timeout() {
            return Err(Box::from("Constitutional collection timed out"));
        }

        let evidence = helpers::create_evidence_base(
            claim.id,
            EvidenceType::ConstitutionalReference,
            "Constitutional checking evidence collection not yet implemented".to_string(),
            0.9,
            0.95,
        );

        // Update source with constitutional-specific information
        let evidence = Evidence {
            source: crate::extraction_types::EvidenceSource::CodeSearch {
                location: "constitution".to_string(),
                authority: "constitutional-check".to_string(),
                freshness: chrono::Utc::now(),
            },
            ..evidence
        };

        // Validate evidence quality using min_relevance_threshold as proxy for confidence threshold
        if evidence.confidence < ctx.config.min_relevance_threshold {
            return Err(Box::from("Evidence confidence below threshold"));
        }

        Ok(vec![evidence])
    }
}

impl ConstitutionalCollector {
    pub fn new() -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
        }
    }

    pub fn with_config(config: EvidenceCollectorConfig) -> Self {
        Self { config }
    }

    /// Legacy method for backward compatibility
    pub async fn collect_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        use crate::evidence::common::{EvidenceCollector as _, CollectorCtx};
        let ctx = CollectorCtx::new(self.config.clone(), context.clone());
        <Self as EvidenceCollector>::run(self, claim, &ctx).await.map_err(|e| anyhow::anyhow!("{}", e))
    }
}
