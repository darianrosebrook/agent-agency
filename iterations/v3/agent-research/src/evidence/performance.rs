//! Performance measurement evidence collection

use super::common::{helpers, CollectorCtx, EvidenceCollector};
use super::types::*;
use crate::evidence::evidence_types::EvidenceCollectorConfig;
use crate::extraction_types::{AtomicClaim, Evidence, EvidenceType, ProcessingContext};
use anyhow::Result;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Performance collector
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceCollector {
    config: EvidenceCollectorConfig,
}

#[async_trait]
impl EvidenceCollector for PerformanceCollector {
    type Input = AtomicClaim;
    type Output = Vec<Evidence>;

    fn name(&self) -> &'static str {
        "performance"
    }

    fn config(&self) -> &EvidenceCollectorConfig {
        &self.config
    }

    async fn collect(
        &self,
        claim: &AtomicClaim,
        ctx: &CollectorCtx,
    ) -> Result<Vec<Evidence>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        // Check timeout
        if ctx.should_timeout() {
            return Err(Box::from("Performance collection timed out"));
        }

        let evidence = helpers::create_evidence_base(
            claim.id,
            EvidenceType::PerformanceMetrics,
            "Performance measurement evidence collection not yet implemented".to_string(),
            0.7,
            0.8,
        );

        // Update source with performance-specific information
        let evidence = Evidence {
            source: crate::extraction_types::EvidenceSource::CodeSearch {
                location: "performance".to_string(),
                authority: "performance-measurement".to_string(),
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

impl PerformanceCollector {
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
        use crate::evidence::common::{CollectorCtx, EvidenceCollector as _};
        let ctx = CollectorCtx::new(self.config.clone(), context.clone());
        <Self as EvidenceCollector>::run(self, claim, &ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
