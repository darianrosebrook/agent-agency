//! Stage 3: Atomic Claim Decomposition
//! 
//! Breaks down sentences into atomic, verifiable claims and adds
//! contextual brackets for proper scope. Based on V2 decomposition logic.

use crate::types::*;
use anyhow::Result;
use tracing::{info, warn, debug};

/// Stage 3: Decomposition into atomic claims
#[derive(Debug)]
pub struct DecompositionStage {
    claim_extractor: ClaimExtractor,
    context_bracket_adder: ContextBracketAdder,
}

impl DecompositionStage {
    pub fn new() -> Self {
        Self {
            claim_extractor: ClaimExtractor::new(),
            context_bracket_adder: ContextBracketAdder::new(),
        }
    }

    /// Process a sentence through decomposition
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DecompositionResult> {
        debug!("Starting decomposition for: {}", sentence);

        // Extract atomic claims
        let atomic_claims = self.extract_atomic_claims(sentence, context).await?;
        
        // Add contextual brackets to each claim
        let mut enhanced_claims = Vec::new();
        for mut claim in atomic_claims {
            let implied_context = self.build_implied_context(context);
            self.add_contextual_brackets(&mut claim, &implied_context).await?;
            enhanced_claims.push(claim);
        }

        let decomposition_confidence = self.calculate_decomposition_confidence(&enhanced_claims);

        Ok(DecompositionResult {
            atomic_claims: enhanced_claims,
            decomposition_confidence,
        })
    }

    /// Extract atomic claims from a disambiguated sentence
    pub async fn extract_atomic_claims(
        &self,
        disambiguated_sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        // Extract different types of claims
        claims.extend(self.claim_extractor.extract_factual_claims(disambiguated_sentence)?);
        claims.extend(self.claim_extractor.extract_procedural_claims(disambiguated_sentence)?);
        claims.extend(self.claim_extractor.extract_technical_claims(disambiguated_sentence, context)?);
        claims.extend(self.claim_extractor.extract_constitutional_claims(disambiguated_sentence)?);

        Ok(claims)
    }

    /// Add contextual brackets to claims for proper scope
    pub async fn add_contextual_brackets(
        &self,
        claim: &mut AtomicClaim,
        implied_context: &ImpliedContext,
    ) -> Result<()> {
        // Add domain context brackets
        for domain in &implied_context.domain_context {
            claim.contextual_brackets.push(format!("[domain: {}]", domain));
        }

        // Add scope context brackets
        claim.contextual_brackets.push(format!(
            "[scope: {}]",
            implied_context.scope_context.component_boundaries.join(", ")
        ));

        // Add verification context brackets
        for method in &implied_context.verification_context.verification_methods {
            claim.contextual_brackets.push(format!("[verification: {:?}]", method));
        }

        // Add temporal context if available
        if let Some(temporal) = &implied_context.temporal_context {
            claim.contextual_brackets.push(format!("[timeframe: {}]", temporal.timeframe));
        }

        Ok(())
    }

    /// Build implied context from processing context
    fn build_implied_context(&self, context: &ProcessingContext) -> ImpliedContext {
        ImpliedContext {
            domain_context: context.domain_hints.clone(),
            temporal_context: Some(TemporalContext {
                timeframe: "current".to_string(),
                version_context: None,
                change_context: None,
            }),
            scope_context: ScopeContext {
                component_boundaries: vec![context.working_spec_id.clone()],
                data_boundaries: vec!["working_spec".to_string()],
                system_boundaries: vec!["agent_agency".to_string()],
            },
            verification_context: VerificationContext {
                verification_methods: vec![
                    VerificationMethod::CodeAnalysis,
                    VerificationMethod::TestExecution,
                    VerificationMethod::ConstitutionalCheck,
                ],
                evidence_sources: vec![
                    SourceType::FileSystem,
                    SourceType::TestSuite,
                    SourceType::Database,
                ],
                confidence_thresholds: vec![
                    ConfidenceThreshold {
                        evidence_type: EvidenceType::CodeAnalysis,
                        minimum_confidence: 0.8,
                        weight: 0.4,
                    },
                    ConfidenceThreshold {
                        evidence_type: EvidenceType::TestResults,
                        minimum_confidence: 0.9,
                        weight: 0.6,
                    },
                ],
            },
        }
    }

    /// Calculate confidence in decomposition quality
    fn calculate_decomposition_confidence(&self, claims: &[AtomicClaim]) -> f64 {
        if claims.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = claims.iter().map(|claim| claim.confidence).sum();
        let average_confidence = total_confidence / claims.len() as f64;
        
        // Boost confidence for claims with contextual brackets
        let contextual_boost = claims.iter()
            .filter(|claim| !claim.contextual_brackets.is_empty())
            .count() as f64 / claims.len() as f64 * 0.2;

        (average_confidence + contextual_boost).min(1.0)
    }
}

/// Extracts atomic claims from text
#[derive(Debug)]
struct ClaimExtractor {
    factual_patterns: Vec<regex::Regex>,
    procedural_patterns: Vec<regex::Regex>,
    technical_patterns: Vec<regex::Regex>,
    constitutional_patterns: Vec<regex::Regex>,
}

impl ClaimExtractor {
    fn new() -> Self {
        Self {
            factual_patterns: vec![
                regex::Regex::new(r"\b(is|are|was|were|has|have|had)\s+([^.!?]+)").unwrap(),
                regex::Regex::new(r"\b(contains|includes|excludes|equals|matches|differs)\s+([^.!?]+)").unwrap(),
            ],
            procedural_patterns: vec![
                regex::Regex::new(r"\b(should|must|can|cannot|will|shall)\s+([^.!?]+)").unwrap(),
                regex::Regex::new(r"\b(processes|handles|manages|creates|updates|deletes)\s+([^.!?]+)").unwrap(),
            ],
            technical_patterns: vec![
                regex::Regex::new(r"\b(function|method|class|interface|type)\s+([^.!?]+)").unwrap(),
                regex::Regex::new(r"\b(implements|extends|inherits|overrides|calls|returns)\s+([^.!?]+)").unwrap(),
            ],
            constitutional_patterns: vec![
                regex::Regex::new(r"\b(CAWS|constitutional|compliance|validation)\s+([^.!?]+)").unwrap(),
                regex::Regex::new(r"\b(working spec|risk tier|change budget)\s+([^.!?]+)").unwrap(),
            ],
        }
    }

    fn extract_factual_claims(&self, sentence: &str) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();
        
        for pattern in &self.factual_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Factual,
                    verifiability: VerifiabilityLevel::DirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: "unknown".to_string(),
                        component_boundaries: vec![],
                        data_impact: DataImpact::ReadOnly,
                    },
                    confidence: 0.8,
                    contextual_brackets: vec![],
                });
            }
        }
        
        Ok(claims)
    }

    fn extract_procedural_claims(&self, sentence: &str) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();
        
        for pattern in &self.procedural_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Procedural,
                    verifiability: VerifiabilityLevel::IndirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: "unknown".to_string(),
                        component_boundaries: vec![],
                        data_impact: DataImpact::Write,
                    },
                    confidence: 0.7,
                    contextual_brackets: vec![],
                });
            }
        }
        
        Ok(claims)
    }

    fn extract_technical_claims(&self, sentence: &str, context: &ProcessingContext) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();
        
        for pattern in &self.technical_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Technical,
                    verifiability: VerifiabilityLevel::DirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: context.working_spec_id.clone(),
                        component_boundaries: context.domain_hints.clone(),
                        data_impact: DataImpact::ReadOnly,
                    },
                    confidence: 0.9,
                    contextual_brackets: vec![],
                });
            }
        }
        
        Ok(claims)
    }

    fn extract_constitutional_claims(&self, sentence: &str) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();
        
        for pattern in &self.constitutional_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Constitutional,
                    verifiability: VerifiabilityLevel::DirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: "caws".to_string(),
                        component_boundaries: vec!["constitutional".to_string()],
                        data_impact: DataImpact::Critical,
                    },
                    confidence: 0.95,
                    contextual_brackets: vec![],
                });
            }
        }
        
        Ok(claims)
    }
}

/// Adds contextual brackets to claims
#[derive(Debug)]
struct ContextBracketAdder {
    // TODO: Add context bracket logic
}

impl ContextBracketAdder {
    fn new() -> Self {
        Self {}
    }
}

/// Context that is implied but not explicitly stated
#[derive(Debug, Clone)]
pub struct ImpliedContext {
    pub domain_context: Vec<String>,
    pub temporal_context: Option<TemporalContext>,
    pub scope_context: ScopeContext,
    pub verification_context: VerificationContext,
}

#[derive(Debug, Clone)]
pub struct TemporalContext {
    pub timeframe: String,
    pub version_context: Option<String>,
    pub change_context: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScopeContext {
    pub component_boundaries: Vec<String>,
    pub data_boundaries: Vec<String>,
    pub system_boundaries: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VerificationContext {
    pub verification_methods: Vec<VerificationMethod>,
    pub evidence_sources: Vec<SourceType>,
    pub confidence_thresholds: Vec<ConfidenceThreshold>,
}

#[derive(Debug, Clone)]
pub struct ConfidenceThreshold {
    pub evidence_type: EvidenceType,
    pub minimum_confidence: f64,
    pub weight: f64,
}
