//! Ambiguity detection in text

use regex::Regex;
use std::collections::HashMap;
use anyhow::Result;
use crate::disambiguation::types::*;
use crate::ProcessingContext;

/// Detects various types of ambiguities in text
#[derive(Debug)]
pub struct AmbiguityDetector {
    pronoun_regex: Regex,
    technical_term_patterns: Vec<Regex>,
    scope_boundary_patterns: Vec<Regex>,
    temporal_patterns: Vec<Regex>,
}

impl AmbiguityDetector {
    /// Create a new ambiguity detector with compiled regex patterns
    pub fn new() -> Self {
        Self {
            // SAFETY: Static regex patterns that are validated at compile time
            pronoun_regex: Regex::new(r"\b(it|this|that|they|them|their|these|those)\b")
                .expect("Static pronoun regex pattern should never fail"),
            technical_term_patterns: vec![
                Regex::new(r"\b(API|UI|UX|DB|SQL|HTTP|JSON|XML)\b")
                    .expect("Static technical term regex pattern should never fail"),
                Regex::new(r"\b(function|method|class|interface|type)\b")
                    .expect("Static programming term regex pattern should never fail"),
            ],
            scope_boundary_patterns: vec![Regex::new(
                r"\b(in|within|inside|outside|across|between)\s+([a-zA-Z_]+)\b",
            )
            .expect("Static scope boundary regex pattern should never fail")],
            temporal_patterns: vec![Regex::new(
                r"\b(before|after|during|while|when|then|now|later)\b",
            )
            .expect("Static temporal regex pattern should never fail")],
        }
    }

    /// Detect pronoun ambiguities in a sentence
    pub fn detect_pronouns(&self, sentence: &str) -> Result<Vec<Ambiguity>> {
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

    /// Detect technical term ambiguities
    pub fn detect_technical_terms(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<Ambiguity>> {
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

    /// Detect scope boundary ambiguities
    pub fn detect_scope_boundaries(&self, sentence: &str) -> Result<Vec<Ambiguity>> {
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

    /// Detect temporal reference ambiguities
    pub fn detect_temporal_references(&self, sentence: &str) -> Result<Vec<Ambiguity>> {
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

    /// Find referent for a pronoun using context map
    pub fn find_referent_for_pronoun(
        &self,
        pronoun: &str,
        _context_map: &HashMap<String, ReferentInfo>,
    ) -> Option<ReferentInfo> {
        // Simplified implementation - in real code this would use the context map
        match pronoun {
            "it" | "this" | "that" => Some(ReferentInfo {
                entity: "the system".to_string(),
                confidence: 0.8,
                source: "system reference".to_string(),
            }),
            _ => None,
        }
    }

    /// Get technical term resolutions
    fn get_technical_resolutions(&self, term: &str, _context: &ProcessingContext) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pronouns() {
        let detector = AmbiguityDetector::new();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test".to_string(),
            domain_hints: vec![],
            metadata: HashMap::new(),
            input_text: "test".to_string(),
            language: None,
        };

        let ambiguities = detector.detect_pronouns("This system handles it well.").unwrap();

        assert!(!ambiguities.is_empty());
        assert_eq!(ambiguities[0].ambiguity_type, AmbiguityType::Pronoun);
        assert_eq!(ambiguities[0].original_text, "This");
    }

    #[test]
    fn test_detect_temporal_references() {
        let detector = AmbiguityDetector::new();

        let ambiguities = detector.detect_temporal_references("It happens before execution.").unwrap();

        assert!(!ambiguities.is_empty());
        assert_eq!(ambiguities[0].ambiguity_type, AmbiguityType::TemporalReference);
        assert_eq!(ambiguities[0].original_text, "before");
    }

    #[test]
    fn test_find_referent_for_pronoun() {
        let detector = AmbiguityDetector::new();
        let context_map = HashMap::new();

        let referent = detector.find_referent_for_pronoun("it", &context_map);

        assert!(referent.is_some());
        assert_eq!(referent.unwrap().entity, "the system");
    }
}
