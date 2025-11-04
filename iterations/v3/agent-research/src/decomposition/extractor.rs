//! Atomic claim extraction functionality

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::extraction_types::*;
use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use tracing::debug;
use uuid::Uuid;

/// Atomic claim extractor implementation

#[derive(Debug)]
pub struct ClaimExtractor {
    #[serde(skip)]
    subject_verb_patterns: Vec<Regex>,
    #[serde(skip)]
    negation_patterns: Vec<Regex>,
}

impl ClaimExtractor {
    pub fn new() -> Self {
        Self {
            subject_verb_patterns: Self::build_subject_verb_patterns(),
            negation_patterns: Self::build_negation_patterns(),
        }
    }

    /// Extract atomic claims from a disambiguated sentence
    pub async fn extract_atomic_claims(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<AtomicClaim>> {
        debug!("Extracting atomic claims from: {}", sentence);

        let mut claims = Vec::new();
        let compound_sentences = self.split_compound_sentences(sentence);

        for (compound_index, compound) in compound_sentences.iter().enumerate() {
            let clauses = self.split_into_clauses(compound);

            for (clause_offset, clause) in clauses.iter().enumerate() {
                if let Some(claim) = self.process_clause(clause, context, compound_index, clause_offset).await? {
                    claims.push(claim);
                }
            }
        }

        debug!("Extracted {} atomic claims", claims.len());
        Ok(claims)
    }

    /// Split sentence into compound parts
    fn split_compound_sentences(&self, sentence: &str) -> Vec<String> {
        // Split on conjunctions and semicolons
        let conjunctions = ["and", "but", "or", "yet", "so", "for", "nor"];
        let mut parts = vec![sentence.to_string()];

        for conj in &conjunctions {
            parts = parts.into_iter()
                .flat_map(|part| {
                    if part.to_lowercase().contains(&format!(" {} ", conj)) {
                        part.split(&format!(" {} ", conj))
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                    } else {
                        vec![part]
                    }
                })
                .collect();
        }

        parts.into_iter()
            .flat_map(|part| {
                part.split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Split compound sentence into clauses
    fn split_into_clauses(&self, compound: &str) -> Vec<String> {
        // Split on relative pronouns and conjunctions
        let separators = ["that", "which", "who", "whom", "whose", "where", "when", "why", "how"];
        let mut clauses = vec![compound.to_string()];

        for sep in &separators {
            clauses = clauses.into_iter()
                .flat_map(|clause| {
                    if clause.to_lowercase().contains(&format!(" {} ", sep)) {
                        clause.split(&format!(" {} ", sep))
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                    } else {
                        vec![clause]
                    }
                })
                .collect();
        }

        clauses
    }

    /// Process a single clause into an atomic claim
    async fn process_clause(
        &self,
        clause: &str,
        context: &ProcessingContext,
        compound_index: usize,
        clause_offset: usize,
    ) -> Result<Option<AtomicClaim>> {
        let normalized_clause = self.normalize_clause(clause);

        if normalized_clause.len() < 8 {
            return Ok(None);
        }

        if !self.has_subject_verb_structure(&normalized_clause) {
            return Ok(None);
        }

        let claim_id = self.generate_claim_id(
            context.task_id,
            0, // sentence_index - simplified
            compound_index * 100 + clause_offset,
        );

        // TODO: Extract contextual brackets
        let contextual_brackets = Vec::new(); // Simplified

        // Apply contextual brackets to the statement
        let bracketed_statement =
            self.apply_contextual_brackets(&normalized_clause, &contextual_brackets);

        let confidence = self.calculate_claim_confidence(&normalized_clause);

        let claim = AtomicClaim {
            id: claim_id,
            claim_text: bracketed_statement,
            claim_type: self.infer_claim_type(&normalized_clause),
            verifiability: self.assess_verifiability(&normalized_clause),
            scope: ClaimScope {
                working_spec_id: context.working_spec_id.clone(),
                component_boundaries: vec!["system".to_string()], // Basic scope
                data_impact: DataImpact::None,
            },
            confidence: confidence.into(),
            evidence_links: Vec::new(),
            temporal_context: None,
            verification_status: VerificationStatus::Unverified,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(Some(claim))
    }

    /// Normalize a clause for processing
    fn normalize_clause(&self, clause: &str) -> String {
        // Basic normalization - remove extra whitespace, fix punctuation
        clause.trim()
            .replace(" ,", ",")
            .replace(" .", ".")
            .replace("  ", " ")
    }

    /// Check if clause has subject-verb structure
    fn has_subject_verb_structure(&self, clause: &str) -> bool {
        // Simple check for subject-verb pattern
        self.subject_verb_patterns.iter().any(|pattern| pattern.is_match(clause))
    }

    /// Generate a unique claim ID
    fn generate_claim_id(&self, task_id: Uuid, sentence_index: usize, clause_index: usize) -> Uuid {
        // Create deterministic UUID based on inputs
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        task_id.hash(&mut hasher);
        sentence_index.hash(&mut hasher);
        clause_index.hash(&mut hasher);

        let hash = hasher.finish();
        Uuid::from_u128(hash as u128)
    }

    /// Apply contextual brackets to statement
    fn apply_contextual_brackets(&self, statement: &str, brackets: &[String]) -> String {
        if brackets.is_empty() {
            statement.to_string()
        } else {
            format!("[{}] {}", brackets.join(", "), statement)
        }
    }

    /// Calculate claim confidence score
    fn calculate_claim_confidence(&self, clause: &str) -> f32 {
        let mut confidence: f32 = 0.5; // Base confidence

        // Increase confidence for longer, more specific claims
        if clause.len() > 20 {
            confidence += 0.1;
        }

        // Increase confidence for claims with quantifiable elements
        if clause.contains(|c: char| c.is_ascii_digit()) {
            confidence += 0.1;
        }

        // Decrease confidence for vague terms
        let vague_terms = ["maybe", "perhaps", "possibly", "might", "could"];
        for term in &vague_terms {
            if clause.to_lowercase().contains(term) {
                confidence -= 0.1;
            }
        }

        confidence.max(0.0).min(1.0)
    }

    /// Infer the type of claim
    fn infer_claim_type(&self, clause: &str) -> ClaimType {
        let clause_lower = clause.to_lowercase();

        if clause_lower.contains("must") || clause_lower.contains("should") || clause_lower.contains("required") {
            ClaimType::Requirement
        } else if clause_lower.contains("when") || clause_lower.contains("if") {
            ClaimType::Conditional
        } else if clause_lower.contains("causes") || clause_lower.contains("leads to") || clause_lower.contains("results in") {
            ClaimType::Causal
        } else {
            ClaimType::Factual
        }
    }

    /// Assess verifiability of the claim
    fn assess_verifiability(&self, clause: &str) -> VerifiabilityLevel {
        // Simple assessment based on content
        if clause.contains("test") || clause.contains("verify") || clause.contains("measure") {
            VerifiabilityLevel::DirectlyVerifiable
        } else if clause.contains(|c: char| c.is_ascii_digit()) {
            VerifiabilityLevel::IndirectlyVerifiable
        } else {
            VerifiabilityLevel::LowVerifiability
        }
    }

    /// Build regex patterns for subject-verb detection
    fn build_subject_verb_patterns() -> Vec<Regex> {
        vec![
            // Simple subject-verb pattern
            Regex::new(r"\b\w+\s+(is|are|was|were|has|have|had|does|do|did|can|could|will|would|should|may|might)\b").unwrap(),
            // Action verb pattern
            Regex::new(r"\b\w+\s+(creates?|reads?|writes?|deletes?|processes?|handles?|manages?|provides?|returns?|accepts?)\b").unwrap(),
        ]
    }

    /// Build negation patterns
    fn build_negation_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"\b(not|no|never|none|neither|nor)\b").unwrap(),
            Regex::new(r"\b(can't|cannot|won't|wouldn't|shouldn't|doesn't|don't|isn't|aren't)\b").unwrap(),
        ]
    }
}
