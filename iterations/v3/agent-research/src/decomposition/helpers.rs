//! Helper functions and utilities for claim decomposition

use crate::extraction_types::*;
use anyhow::Result;

/// Confidence calculation utilities
pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    /// Calculate confidence based on claim characteristics
    pub fn calculate_claim_confidence(claim_text: &str) -> f32 {
        let mut confidence = 0.5; // Base confidence

        // Increase confidence for longer, more specific claims
        if claim_text.len() > 20 {
            confidence += 0.1;
        }

        // Increase confidence for claims with quantifiable elements
        if claim_text.chars().any(|c| c.is_ascii_digit()) {
            confidence += 0.1;
        }

        // Increase confidence for claims with specific technical terms
        let technical_terms = [
            "function", "method", "class", "struct", "enum", "trait",
            "database", "query", "table", "index", "cache", "memory",
            "performance", "latency", "throughput", "efficiency",
            "security", "authentication", "authorization", "encryption",
        ];

        for term in &technical_terms {
            if claim_text.to_lowercase().contains(term) {
                confidence += 0.05;
                break; // Only count once per term type
            }
        }

        // Decrease confidence for vague terms
        let vague_terms = ["maybe", "perhaps", "possibly", "might", "could", "should"];
        for term in &vague_terms {
            if claim_text.to_lowercase().contains(term) {
                confidence -= 0.1;
                break;
            }
        }

        // Decrease confidence for negation
        let negation_terms = ["not", "no", "never", "none", "cannot", "can't"];
        for term in &negation_terms {
            if claim_text.to_lowercase().contains(term) {
                confidence -= 0.05;
                break;
            }
        }

        confidence.max(0.0).min(1.0)
    }

    /// Calculate decomposition confidence for a set of claims
    pub fn calculate_decomposition_confidence(claims: &[AtomicClaim]) -> f32 {
        if claims.is_empty() {
            return 0.0;
        }

        let total_confidence: f32 = claims.iter().map(|c| c.confidence as f32).sum();
        let avg_confidence = total_confidence / claims.len() as f32;

        // Apply quality penalties
        let mut quality_penalty = 0.0;

        // Penalty for too few claims
        if claims.len() < 2 {
            quality_penalty += 0.1;
        }

        // Penalty for too many claims (might indicate over-splitting)
        if claims.len() > 15 {
            quality_penalty += 0.1;
        }

        // Penalty for low average confidence
        if avg_confidence < 0.6 {
            quality_penalty += 0.1;
        }

        (avg_confidence - quality_penalty).max(0.0).min(1.0)
    }
}

/// Claim type inference utilities
pub struct ClaimTypeInferer;

impl ClaimTypeInferer {
    /// Infer the type of claim from text
    pub fn infer_claim_type(claim_text: &str) -> ClaimType {
        let text_lower = claim_text.to_lowercase();

        // Requirement claims
        if text_lower.contains("must") || text_lower.contains("should") ||
           text_lower.contains("required") || text_lower.contains("shall") ||
           text_lower.contains("needs to") || text_lower.contains("has to") {
            ClaimType::Requirement
        }
        // Conditional claims
        else if text_lower.contains("when") || text_lower.contains("if") ||
                text_lower.contains("unless") || text_lower.contains("provided that") ||
                text_lower.contains("assuming") || text_lower.contains("given that") {
            ClaimType::Conditional
        }
        // Causal claims
        else if text_lower.contains("causes") || text_lower.contains("leads to") ||
                text_lower.contains("results in") || text_lower.contains("because") ||
                text_lower.contains("due to") || text_lower.contains("therefore") {
            ClaimType::Causal
        }
        // Quantitative claims
        else if claim_text.chars().any(|c| c.is_ascii_digit()) ||
                text_lower.contains("percent") || text_lower.contains("percentage") ||
                text_lower.contains("times") || text_lower.contains("rate") {
            ClaimType::Quantitative
        }
        // Default to factual
        else {
            ClaimType::Factual
        }
    }

    /// Assess verifiability of a claim
    pub fn assess_verifiability(claim_text: &str) -> VerifiabilityLevel {
        let text_lower = claim_text.to_lowercase();

        // Testable claims
        if text_lower.contains("test") || text_lower.contains("verify") ||
           text_lower.contains("validate") || text_lower.contains("measure") ||
           text_lower.contains("benchmark") || text_lower.contains("check") {
            VerifiabilityLevel::DirectlyVerifiable
        }
        // Quantifiable claims
        else if claim_text.chars().any(|c| c.is_ascii_digit()) ||
                text_lower.contains("performance") || text_lower.contains("latency") ||
                text_lower.contains("throughput") || text_lower.contains("memory") ||
                text_lower.contains("cpu") || text_lower.contains("efficiency") {
            VerifiabilityLevel::IndirectlyVerifiable
        }
        // Observable claims
        else if text_lower.contains("visible") || text_lower.contains("observable") ||
                text_lower.contains("detectable") || text_lower.contains("monitor") ||
                text_lower.contains("log") || text_lower.contains("trace") {
            VerifiabilityLevel::LowVerifiability
        }
        // Default to qualitative
        else {
            VerifiabilityLevel::LowVerifiability
        }
    }
}

/// Text processing utilities
pub struct TextProcessor;

impl TextProcessor {
    /// Normalize text for processing
    pub fn normalize_text(text: &str) -> String {
        text.trim()
            .replace(" ,", ",")
            .replace(" .", ".")
            .replace("  ", " ")
            .replace("  ", " ") // Run twice to handle multiple spaces
    }

    /// Check if text has subject-verb-object structure
    pub fn has_subject_verb_structure(text: &str) -> bool {
        // Simple heuristic: check for basic sentence structure
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.len() < 3 {
            return false;
        }

        // Look for verb patterns (very simplified)
        let verbs = ["is", "are", "was", "were", "has", "have", "had",
                    "does", "do", "did", "can", "could", "will", "would",
                    "should", "may", "might", "must", "shall"];

        let action_verbs = ["create", "read", "write", "delete", "process",
                           "handle", "manage", "provide", "return", "accept",
                           "send", "receive", "store", "load", "save"];

        let has_verb = words.iter().any(|word|
            verbs.contains(word) ||
            action_verbs.iter().any(|av| word.starts_with(av))
        );

        has_verb
    }

    /// Extract keywords from text
    pub fn extract_keywords(text: &str) -> Vec<String> {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to",
            "for", "of", "with", "by", "as", "is", "are", "was", "were",
            "has", "have", "had", "will", "would", "could", "should", "may"
        ];

        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2)
            .filter(|word| !stop_words.contains(word))
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty())
            .map(|word| word.to_string())
            .collect()
    }

    /// Calculate text complexity score
    pub fn calculate_complexity(text: &str) -> f32 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();

        if word_count == 0 {
            return 0.0;
        }

        let avg_word_length: f32 = words.iter()
            .map(|w| w.len() as f32)
            .sum::<f32>() / word_count as f32;

        let sentence_count = text.split('.').count() +
                           text.split('!').count() +
                           text.split('?').count() - 2; // Subtract 2 for empty splits

        let complexity = (avg_word_length * 0.3) +
                        (word_count as f32 * 0.1) +
                        (sentence_count as f32 * 0.2);

        complexity.min(10.0) // Cap at 10
    }
}

/// ID generation utilities
pub struct IdGenerator;

impl IdGenerator {
    /// Generate a deterministic claim ID
    pub fn generate_claim_id(task_id: uuid::Uuid, sentence_index: usize, clause_index: usize) -> uuid::Uuid {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        task_id.hash(&mut hasher);
        sentence_index.hash(&mut hasher);
        clause_index.hash(&mut hasher);

        let hash = hasher.finish();
        uuid::Uuid::from_u128(hash as u128)
    }

    /// Generate a unique scope identifier
    pub fn generate_scope_id(working_spec_id: &str, component: &str) -> String {
        format!("{}_{}", working_spec_id, component)
    }
}
