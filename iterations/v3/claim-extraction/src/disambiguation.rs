//! Stage 1: Contextual Disambiguation
//! 
//! Identifies and resolves ambiguities in sentences to prepare for
//! claim extraction. Based on V2 disambiguation logic with Rust adaptations.

use crate::types::*;
use anyhow::Result;
use tracing::{info, warn, debug};
use regex::Regex;
use std::collections::HashMap;

/// Stage 1: Contextual disambiguation of sentences
#[derive(Debug)]
pub struct DisambiguationStage {
    ambiguity_detector: AmbiguityDetector,
    context_resolver: ContextResolver,
}

impl DisambiguationStage {
    pub fn new() -> Self {
        Self {
            ambiguity_detector: AmbiguityDetector::new(),
            context_resolver: ContextResolver::new(),
        }
    }

    /// Process a sentence through disambiguation
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DisambiguationResult> {
        debug!("Starting disambiguation for: {}", sentence);

        // Identify ambiguities
        let ambiguities = self.identify_ambiguities(sentence, context).await?;
        debug!("Identified {} ambiguities", ambiguities.len());

        // Resolve ambiguities
        let disambiguated_sentence = self.resolve_ambiguities(sentence, &ambiguities, context).await?;
        
        // Detect unresolvable ambiguities
        let unresolvable = self.detect_unresolvable_ambiguities(sentence, context).await?;

        Ok(DisambiguationResult {
            original_sentence: sentence.to_string(),
            disambiguated_sentence,
            ambiguities_resolved: ambiguities.len() as u32,
            unresolvable_ambiguities: unresolvable,
        })
    }

    /// Identify ambiguities in a sentence given context
    pub async fn identify_ambiguities(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<Ambiguity>> {
        let mut ambiguities = Vec::new();

        // Detect pronouns
        ambiguities.extend(self.ambiguity_detector.detect_pronouns(sentence)?);

        // Detect technical terms
        ambiguities.extend(self.ambiguity_detector.detect_technical_terms(sentence, context)?);

        // Detect scope boundaries
        ambiguities.extend(self.ambiguity_detector.detect_scope_boundaries(sentence)?);

        // Detect temporal references
        ambiguities.extend(self.ambiguity_detector.detect_temporal_references(sentence)?);

        Ok(ambiguities)
    }

    /// Resolve ambiguities using context
    pub async fn resolve_ambiguities(
        &self,
        sentence: &str,
        ambiguities: &[Ambiguity],
        context: &ProcessingContext,
    ) -> Result<String> {
        let mut resolved_sentence = sentence.to_string();

        // Sort ambiguities by position (reverse order to avoid position shifts)
        let mut sorted_ambiguities = ambiguities.to_vec();
        sorted_ambiguities.sort_by(|a, b| b.position.0.cmp(&a.position.0));

        for ambiguity in sorted_ambiguities {
            let resolution = self.context_resolver.resolve_ambiguity(&ambiguity, context)?;
            if let Some(resolution) = resolution {
                resolved_sentence = format!(
                    "{}{}{}",
                    &resolved_sentence[..ambiguity.position.0],
                    resolution,
                    &resolved_sentence[ambiguity.position.1..]
                );
            }
        }

        Ok(resolved_sentence)
    }

    /// Detect ambiguities that cannot be resolved
    pub async fn detect_unresolvable_ambiguities(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<UnresolvableAmbiguity>> {
        let ambiguities = self.identify_ambiguities(sentence, context).await?;
        let mut unresolvable = Vec::new();

        for ambiguity in ambiguities {
            if let Some(reason) = self.context_resolver.check_unresolvable(&ambiguity, context) {
                unresolvable.push(UnresolvableAmbiguity {
                    ambiguity,
                    reason,
                    suggested_context: vec!["Additional context needed".to_string()],
                });
            }
        }

        Ok(unresolvable)
    }
}

/// Detects various types of ambiguities in text
#[derive(Debug)]
struct AmbiguityDetector {
    pronoun_regex: Regex,
    technical_term_patterns: Vec<Regex>,
    scope_boundary_patterns: Vec<Regex>,
    temporal_patterns: Vec<Regex>,
}

impl AmbiguityDetector {
    fn new() -> Self {
        Self {
            pronoun_regex: Regex::new(r"\b(it|this|that|they|them|their|these|those)\b").unwrap(),
            technical_term_patterns: vec![
                Regex::new(r"\b(API|UI|UX|DB|SQL|HTTP|JSON|XML)\b").unwrap(),
                Regex::new(r"\b(function|method|class|interface|type)\b").unwrap(),
            ],
            scope_boundary_patterns: vec![
                Regex::new(r"\b(in|within|inside|outside|across|between)\s+([a-zA-Z_]+)\b").unwrap(),
            ],
            temporal_patterns: vec![
                Regex::new(r"\b(before|after|during|while|when|then|now|later)\b").unwrap(),
            ],
        }
    }

    fn detect_pronouns(&self, sentence: &str) -> Result<Vec<Ambiguity>> {
        let mut ambiguities = Vec::new();
        
        for mat in self.pronoun_regex.find_iter(sentence) {
            ambiguities.push(Ambiguity {
                ambiguity_type: AmbiguityType::Pronoun,
                position: (mat.start(), mat.end()),
                original_text: mat.as_str().to_string(),
                possible_resolutions: vec![
                    "the system".to_string(),
                    "the component".to_string(),
                    "the function".to_string(),
                ],
                confidence: 0.8,
            });
        }
        
        Ok(ambiguities)
    }

    fn detect_technical_terms(&self, sentence: &str, context: &ProcessingContext) -> Result<Vec<Ambiguity>> {
        let mut ambiguities = Vec::new();
        
        for pattern in &self.technical_term_patterns {
            for mat in pattern.find_iter(sentence) {
                ambiguities.push(Ambiguity {
                    ambiguity_type: AmbiguityType::TechnicalTerm,
                    position: (mat.start(), mat.end()),
                    original_text: mat.as_str().to_string(),
                    possible_resolutions: self.get_technical_resolutions(mat.as_str(), context),
                    confidence: 0.7,
                });
            }
        }
        
        Ok(ambiguities)
    }

    fn detect_scope_boundaries(&self, sentence: &str) -> Result<Vec<Ambiguity>> {
        let mut ambiguities = Vec::new();
        
        for pattern in &self.scope_boundary_patterns {
            for mat in pattern.find_iter(sentence) {
                ambiguities.push(Ambiguity {
                    ambiguity_type: AmbiguityType::ScopeBoundary,
                    position: (mat.start(), mat.end()),
                    original_text: mat.as_str().to_string(),
                    possible_resolutions: vec![
                        "in the specified component".to_string(),
                        "within the defined scope".to_string(),
                    ],
                    confidence: 0.6,
                });
            }
        }
        
        Ok(ambiguities)
    }

    fn detect_temporal_references(&self, sentence: &str) -> Result<Vec<Ambiguity>> {
        let mut ambiguities = Vec::new();
        
        for pattern in &self.temporal_patterns {
            for mat in pattern.find_iter(sentence) {
                ambiguities.push(Ambiguity {
                    ambiguity_type: AmbiguityType::TemporalReference,
                    position: (mat.start(), mat.end()),
                    original_text: mat.as_str().to_string(),
                    possible_resolutions: vec![
                        "during execution".to_string(),
                        "at runtime".to_string(),
                        "when called".to_string(),
                    ],
                    confidence: 0.5,
                });
            }
        }
        
        Ok(ambiguities)
    }

    fn get_technical_resolutions(&self, term: &str, context: &ProcessingContext) -> Vec<String> {
        match term.to_uppercase().as_str() {
            "API" => vec!["Application Programming Interface".to_string()],
            "UI" => vec!["User Interface".to_string()],
            "UX" => vec!["User Experience".to_string()],
            "DB" => vec!["Database".to_string()],
            "SQL" => vec!["Structured Query Language".to_string()],
            _ => vec![format!("{} (technical term)", term)],
        }
    }
}

/// Resolves ambiguities using available context
#[derive(Debug)]
struct ContextResolver {
    domain_context: HashMap<String, String>,
}

impl ContextResolver {
    fn new() -> Self {
        let mut domain_context = HashMap::new();
        domain_context.insert("system".to_string(), "the Agent Agency system".to_string());
        domain_context.insert("component".to_string(), "the current component".to_string());
        domain_context.insert("function".to_string(), "the specified function".to_string());
        
        Self { domain_context }
    }

    fn resolve_ambiguity(&self, ambiguity: &Ambiguity, context: &ProcessingContext) -> Result<Option<String>> {
        match ambiguity.ambiguity_type {
            AmbiguityType::Pronoun => {
                // Use domain hints to resolve pronouns
                if let Some(hint) = context.domain_hints.first() {
                    Ok(Some(hint.clone()))
                } else {
                    Ok(Some("the system".to_string()))
                }
            }
            AmbiguityType::TechnicalTerm => {
                Ok(ambiguity.possible_resolutions.first().cloned())
            }
            AmbiguityType::ScopeBoundary => {
                Ok(Some(format!("in {}", context.working_spec_id)))
            }
            AmbiguityType::TemporalReference => {
                Ok(Some("during execution".to_string()))
            }
            AmbiguityType::Quantifier => {
                Ok(Some("all instances".to_string()))
            }
        }
    }

    fn check_unresolvable(&self, ambiguity: &Ambiguity, context: &ProcessingContext) -> Option<UnresolvableReason> {
        match ambiguity.ambiguity_type {
            AmbiguityType::Pronoun if context.domain_hints.is_empty() => {
                Some(UnresolvableReason::InsufficientContext)
            }
            AmbiguityType::TechnicalTerm if ambiguity.possible_resolutions.len() > 3 => {
                Some(UnresolvableReason::MultipleValidInterpretations)
            }
            AmbiguityType::ScopeBoundary if context.surrounding_context.is_empty() => {
                Some(UnresolvableReason::InsufficientContext)
            }
            _ => None,
        }
    }
}

// Types imported from types.rs - no need to redefine here
