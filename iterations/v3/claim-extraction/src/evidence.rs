use crate::types::*;

#[derive(Debug, Clone, Default)]
pub struct EvidenceCollector;

impl EvidenceCollector {
    pub fn new() -> Self { Self }
    pub async fn collect_for_claim(&self, _claim: &AtomicClaim, _ctx: &ProcessingContext) -> Vec<EvidenceItem> {
        Vec::new()
    }
}


