use crate::types::*;

pub struct ClaimExtractionProcessor;

impl ClaimExtractionProcessor {
    pub fn new() -> Self { Self }
    pub async fn run(&self, _input: &str, _ctx: &ProcessingContext) -> Result<ClaimExtractionResult, ClaimExtractionError> {
        // Stub: return empty extraction result
        Ok(ClaimExtractionResult {
            original_sentence: _input.to_string(),
            disambiguated_sentence: _input.to_string(),
            atomic_claims: Vec::new(),
            verification_evidence: Vec::new(),
            processing_metadata: ProcessingMetadata::default(),
        })
    }
}


