//! Claim Extraction Module
//!
//! Extracts atomic claims from content that can be verified
//! through fact-checking and source validation.

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::evidence_types::*;

/// Extraction pattern for different content types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractionPattern {
    /// Type of pattern
    pub pattern_type: PatternType,
    /// Keyword indicators for this pattern
    pub indicators: Vec<String>,
    /// Rules for decomposing content into claims
    pub decomposition_rules: Vec<DecompositionRule>,
}

/// Types of extraction patterns
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PatternType {
    /// Code-related claims
    Code,
    /// Documentation claims
    Documentation,
    /// Research claims
    Research,
    /// General claims
    General,
}

/// Rules for decomposing content into atomic claims
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum DecompositionRule {
    /// Split by logical operators (and, or, but)
    SplitByLogicalOperators,
    /// Extract function specifications
    ExtractFunctionSpecifications,
    /// Isolate performance claims
    IsolatePerformanceClaims,
    /// Split by requirements
    SplitByRequirements,
    /// Extract compliance statements
    ExtractComplianceStatements,
    /// Isolate functional requirements
    IsolateFunctionalRequirements,
    /// Extract research findings
    ExtractResearchFindings,
    /// Isolate methodology claims
    IsolateMethodologyClaims,
    /// Split by hypothesis
    SplitByHypothesis,
}

/// Claim extractor for breaking down complex statements into verifiable claims
#[derive(Debug)]
pub struct ClaimExtractor {
    /// Extraction patterns for different content types
    extraction_patterns: HashMap<String, ExtractionPattern>,
}

impl ClaimExtractor {
    /// Create a new claim extractor
    pub async fn new() -> Result<Self> {
        let mut patterns = HashMap::new();

        // Code-related claims
        patterns.insert(
            "code".to_string(),
            ExtractionPattern {
                pattern_type: PatternType::Code,
                indicators: vec![
                    "function".to_string(),
                    "class".to_string(),
                    "method".to_string(),
                    "variable".to_string(),
                    "algorithm".to_string(),
                ],
                decomposition_rules: vec![
                    DecompositionRule::SplitByLogicalOperators,
                    DecompositionRule::ExtractFunctionSpecifications,
                    DecompositionRule::IsolatePerformanceClaims,
                ],
            },
        );

        // Documentation claims
        patterns.insert(
            "documentation".to_string(),
            ExtractionPattern {
                pattern_type: PatternType::Documentation,
                indicators: vec![
                    "must".to_string(),
                    "should".to_string(),
                    "requires".to_string(),
                    "specification".to_string(),
                    "requirement".to_string(),
                ],
                decomposition_rules: vec![
                    DecompositionRule::SplitByRequirements,
                    DecompositionRule::ExtractComplianceStatements,
                    DecompositionRule::IsolateFunctionalRequirements,
                ],
            },
        );

        // Research claims
        patterns.insert(
            "research".to_string(),
            ExtractionPattern {
                pattern_type: PatternType::Research,
                indicators: vec![
                    "study".to_string(),
                    "research".to_string(),
                    "evidence".to_string(),
                    "finding".to_string(),
                    "conclusion".to_string(),
                ],
                decomposition_rules: vec![
                    DecompositionRule::ExtractResearchFindings,
                    DecompositionRule::IsolateMethodologyClaims,
                    DecompositionRule::SplitByHypothesis,
                ],
            },
        );

        Ok(Self {
            extraction_patterns: patterns,
        })
    }

    /// Extract atomic claims from content
    pub async fn extract_claims(
        &self,
        content: &str,
        content_type: &str,
        context: &ProcessingContext,
    ) -> Result<ClaimExtractionResult> {
        info!("Extracting claims from {} content", content_type);

        let pattern = self.extraction_patterns.get(content_type).ok_or_else(|| {
            anyhow::anyhow!("No extraction pattern for content type: {}", content_type)
        })?;

        // Phase 1: Contextual disambiguation
        let disambiguated = self.disambiguate_context(content, context).await?;

        // Phase 2: Verifiable content qualification
        let qualified = self
            .qualify_verifiable_content(&disambiguated, pattern)
            .await?;

        // Phase 3: Atomic claim decomposition
        let claims = self.decompose_atomic_claims(&qualified, pattern).await?;

        // Phase 4: CAWS-compliant verification preparation
        let _verification_requirements = self
            .prepare_verification_requirements(&claims, context)
            .await?;

        let entity_count = claims.iter().map(|c| c.entities.len()).sum();

        Ok(ClaimExtractionResult {
            claims,
            metadata: ExtractionMetadata {
                processing_time_ms: 0, // Would be measured in real implementation
                entity_count,
                confidence_score: 0.8, // Default confidence
                content_length: content.len(),
            },
        })
    }

    /// Disambiguate content context
    async fn disambiguate_context(
        &self,
        content: &str,
        _context: &ProcessingContext,
    ) -> Result<String> {
        // Basic implementation - in real implementation this would use NLP
        Ok(content.to_string())
    }

    /// Qualify verifiable content
    async fn qualify_verifiable_content(
        &self,
        content: &str,
        pattern: &ExtractionPattern,
    ) -> Result<String> {
        // Check if content contains indicators for this pattern
        let has_indicators = pattern
            .indicators
            .iter()
            .any(|indicator| content.to_lowercase().contains(&indicator.to_lowercase()));

        if !has_indicators {
            return Ok(content.to_string());
        }

        // Filter content to verifiable parts
        Ok(content.to_string())
    }

    /// Decompose content into atomic claims
    async fn decompose_atomic_claims(
        &self,
        content: &str,
        pattern: &ExtractionPattern,
    ) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        // Simple sentence-based decomposition
        let sentences: Vec<&str> = content
            .split(|c| c == '.' || c == '!' || c == '?')
            .collect();

        for (i, sentence) in sentences.iter().enumerate() {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            // Check if sentence contains claim indicators
            let has_indicators = pattern
                .indicators
                .iter()
                .any(|indicator| sentence.to_lowercase().contains(&indicator.to_lowercase()));

            if has_indicators {
                let claim = AtomicClaim {
                    id: format!("claim_{}", i),
                    text: sentence.to_string(),
                    claim_type: match pattern.pattern_type {
                        PatternType::Code => ClaimType::Factual,
                        PatternType::Documentation => ClaimType::Definitional,
                        PatternType::Research => ClaimType::Factual,
                        PatternType::General => ClaimType::Factual,
                    },
                    entities: vec![], // Would be extracted by NER
                    confidence: 0.7,
                    positions: vec![], // Would be calculated
                    evidence: vec![],  // Would be gathered
                };

                claims.push(claim);
            }
        }

        Ok(claims)
    }

    /// Prepare verification requirements
    async fn prepare_verification_requirements(
        &self,
        claims: &[AtomicClaim],
        _context: &ProcessingContext,
    ) -> Result<Vec<String>> {
        let requirements = claims
            .iter()
            .map(|claim| format!("Verify: {}", claim.text))
            .collect();

        Ok(requirements)
    }
}
