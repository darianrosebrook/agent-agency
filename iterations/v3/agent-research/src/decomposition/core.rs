//! Core decomposition functionality

use crate::extraction_types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Stage 3: Decomposition into atomic claims
#[derive(Debug)]
pub struct DecompositionStage {
    claim_extractor: Arc<RwLock<ClaimExtractor>>,
    context_bracket_adder: Arc<RwLock<ContextBracketAdder>>,
}

impl DecompositionStage {
    pub fn new() -> Self {
        Self {
            claim_extractor: Arc::new(RwLock::new(ClaimExtractor::new())),
            context_bracket_adder: Arc::new(RwLock::new(ContextBracketAdder::new())),
        }
    }

    /// Process a sentence through decomposition (ported from V2)
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DecompositionResult> {
        debug!("Starting decomposition for: {}", sentence);

        // Extract atomic claims using V2 compound sentence decomposition
        let atomic_claims = self.extract_atomic_claims(sentence, context).await?;

        let decomposition_confidence = self.calculate_decomposition_confidence(&atomic_claims);

        Ok(DecompositionResult {
            atomic_claims,
            decomposition_confidence,
        })
    }

    /// Extract atomic claims from a disambiguated sentence (ported from V2)
    pub async fn extract_atomic_claims(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<AtomicClaim>> {
        let extractor = self.claim_extractor.read().await;
        extractor.extract_atomic_claims(sentence, context).await
    }

    /// Calculate overall decomposition confidence
    pub fn calculate_decomposition_confidence(&self, claims: &[AtomicClaim]) -> f32 {
        if claims.is_empty() {
            return 0.0;
        }

        let total_confidence: f32 = claims.iter().map(|c| c.confidence_score).sum();
        total_confidence / claims.len() as f32
    }
}

/// Configuration for decomposition processing
#[derive(Debug, Clone)]
pub struct DecompositionConfig {
    /// Maximum number of atomic claims to extract
    pub max_atomic_claims: usize,
    /// Minimum confidence threshold for claims
    pub min_confidence_threshold: f32,
    /// Whether to enable contextual bracketing
    pub enable_contextual_bracketing: bool,
    /// Maximum depth for compound sentence decomposition
    pub max_decomposition_depth: usize,
}

impl Default for DecompositionConfig {
    fn default() -> Self {
        Self {
            max_atomic_claims: 10,
            min_confidence_threshold: 0.6,
            enable_contextual_bracketing: true,
            max_decomposition_depth: 5,
        }
    }
}

/// Result of decomposition processing
#[derive(Debug, Clone)]
pub struct DecompositionResult {
    /// Extracted atomic claims
    pub atomic_claims: Vec<AtomicClaim>,
    /// Overall confidence in the decomposition
    pub decomposition_confidence: f32,
}

impl DecompositionResult {
    /// Check if decomposition was successful
    pub fn is_successful(&self, threshold: f32) -> bool {
        !self.atomic_claims.is_empty() && self.decomposition_confidence >= threshold
    }

    /// Get claims above confidence threshold
    pub fn high_confidence_claims(&self, threshold: f32) -> Vec<&AtomicClaim> {
        self.atomic_claims.iter()
            .filter(|c| c.confidence_score >= threshold)
            .collect()
    }
}

/// Forward declarations for types that will be implemented in other modules
#[derive(Debug)]
pub struct ClaimExtractor;

impl ClaimExtractor {
    pub fn new() -> Self {
        Self
    }

    pub async fn extract_atomic_claims(
        &self,
        _sentence: &str,
        _context: &ProcessingContext,
    ) -> Result<Vec<AtomicClaim>> {
        // TODO: Implement claim extraction logic
        Ok(Vec::new())
    }
}

#[derive(Debug)]
pub struct ContextBracketAdder;

impl ContextBracketAdder {
    pub fn new() -> Self {
        Self
    }

    pub async fn add_contextual_brackets(
        &self,
        _claims: &[AtomicClaim],
        _context: &ProcessingContext,
    ) -> Result<Vec<AtomicClaim>> {
        // TODO: Implement contextual bracketing logic
        Ok(Vec::new())
    }
}
