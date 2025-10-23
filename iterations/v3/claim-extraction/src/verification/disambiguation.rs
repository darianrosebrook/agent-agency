//! Entity disambiguation logic and strategies
//!
//! This module handles entity disambiguation using multiple strategies:
//! exact matching, fuzzy matching, and context-based disambiguation.

use std::collections::{HashMap, HashSet};
use regex::Regex;

use anyhow::Result;
use crate::verification::types::*;

/// Entity disambiguation engine
pub struct EntityDisambiguator;

impl EntityDisambiguator {
    /// Perform comprehensive entity disambiguation using multiple strategies
    pub async fn disambiguate_entity(&self, entity: &Entity, context: &str) -> Result<EntityDisambiguation> {
        let mut candidates = Vec::new();

        // Strategy 1: Exact match within context
        if let Some(exact_match) = self.find_exact_match(entity, context) {
            candidates.push(EntityCandidate {
                entity: exact_match,
                similarity_score: 1.0,
                context_match: true,
                source: "exact_match".to_string(),
            });
        }

        // Strategy 2: Fuzzy matching with similar entities
        let fuzzy_matches = self.find_fuzzy_entity_matches(entity, context);
        candidates.extend(fuzzy_matches);

        // Strategy 3: Context-based disambiguation
        let context_matches = self.find_entity_context_matches(entity, context);
        candidates.extend(context_matches);

        // Select best match
        let best_match = self.select_best_entity_match(&candidates);

        let method = if candidates.iter().any(|c| c.similarity_score >= 0.9) {
            DisambiguationMethod::ExactMatch
        } else if candidates.iter().any(|c| c.context_match) {
            DisambiguationMethod::ContextBased
        } else {
            DisambiguationMethod::FuzzyMatch
        };

        Ok(EntityDisambiguation {
            original_entity: entity.clone(),
            candidates,
            best_match,
            disambiguation_method: method,
        })
    }

    /// Find exact matches for entity in context
    fn find_exact_match(&self, entity: &Entity, context: &str) -> Option<Entity> {
        let context_lower = context.to_lowercase();
        let entity_text_lower = entity.text.to_lowercase();

        // Look for exact matches or close variations
        if context_lower.contains(&entity_text_lower) {
            Some(Entity {
                id: format!("exact_{}", entity.id),
                text: entity.text.clone(),
                entity_type: entity.entity_type.clone(),
                confidence: 0.95,
                position: (0, entity.text.len()), // Placeholder position
                metadata: HashMap::from([
                    ("match_type".to_string(), "exact".to_string()),
                    ("source".to_string(), "context_match".to_string()),
                ]),
            })
        } else {
            None
        }
    }

    /// Find fuzzy matches using string similarity
    fn find_fuzzy_entity_matches(&self, entity: &Entity, context: &str) -> Vec<EntityCandidate> {
        let mut candidates = Vec::new();
        let words: Vec<&str> = context.split_whitespace().collect();

        for (i, &word) in words.iter().enumerate() {
            let similarity = self.calculate_string_similarity(&entity.text, word);
            if similarity > 0.7 {
                // Look for context around the word
                let start = i.saturating_sub(2);
                let end = (i + 3).min(words.len());
                let context_window = words[start..end].join(" ");

                candidates.push(EntityCandidate {
                    entity: Entity {
                        id: format!("fuzzy_{}_{}", entity.id, i),
                        text: word.to_string(),
                        entity_type: self.infer_entity_type(word),
                        confidence: similarity,
                        position: (0, word.len()),
                        metadata: HashMap::from([
                            ("similarity".to_string(), similarity.to_string()),
                            ("context_window".to_string(), context_window),
                        ]),
                    },
                    similarity_score: similarity,
                    context_match: false,
                    source: "fuzzy_match".to_string(),
                });
            }
        }

        candidates
    }

    /// Find context-based matches using semantic patterns
    fn find_entity_context_matches(&self, entity: &Entity, context: &str) -> Vec<EntityCandidate> {
        let mut candidates = Vec::new();
        let context_lower = context.to_lowercase();

        // Define patterns for different entity types
        let patterns = match entity.entity_type {
            EntityType::CodeEntity => vec![
                r"\b(function|method|class|struct|module)\s+\w+\b",
                r"\b\w+\(\)\s*\{",
                r"\bconst\s+\w+\s*=",
            ],
            EntityType::SystemComponent => vec![
                r"\b(api|service|database|server)\s+\w+\b",
                r"\bendpoint\s+[\w/]+\b",
                r"\btable\s+\w+\b",
            ],
            _ => vec![r"\b\w+\b"],
        };

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for capture in regex.find_iter(&context_lower) {
                    let matched_text = capture.as_str();
                    let similarity = self.calculate_semantic_similarity(&entity.text, matched_text);

                    if similarity > 0.6 {
                        candidates.push(EntityCandidate {
                            entity: Entity {
                                id: format!("context_{}_{}", entity.id, candidates.len()),
                                text: matched_text.to_string(),
                                entity_type: entity.entity_type.clone(),
                                confidence: similarity,
                                position: capture.range(),
                                metadata: HashMap::from([
                                    ("pattern".to_string(), pattern.to_string()),
                                    ("context_match".to_string(), "true".to_string()),
                                ]),
                            },
                            similarity_score: similarity,
                            context_match: true,
                            source: "context_pattern".to_string(),
                        });
                    }
                }
            }
        }

        candidates
    }

    /// Select the best entity match from candidates
    fn select_best_entity_match(&self, candidates: &[EntityCandidate]) -> Option<EntityCandidate> {
        if candidates.is_empty() {
            return None;
        }

        let mut best_candidate = None;
        let mut best_score = 0.0;

        for candidate in candidates {
            let score = candidate.similarity_score * candidate.entity.confidence
                      * if candidate.context_match { 1.2 } else { 1.0 };

            if score > best_score {
                best_score = score;
                best_candidate = Some(candidate.clone());
            }
        }

        best_candidate
    }

    /// Calculate string similarity using Levenshtein distance
    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 && len2 == 0 {
            return 1.0;
        }
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        let max_len = len1.max(len2);
        let distance = self.levenshtein_distance(s1, s2);

        1.0 - (distance as f64 / max_len as f64)
    }

    /// Calculate semantic similarity based on word overlap and entity types
    fn calculate_semantic_similarity(&self, s1: &str, s2: &str) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();
        let words1: HashSet<_> = s1_lower.split_whitespace().collect();
        let words2: HashSet<_> = s2_lower.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.len() + words2.len() - intersection;

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Infer entity type from text content
    fn infer_entity_type(&self, text: &str) -> EntityType {
        let text_lower = text.to_lowercase();

        // Check for code-related keywords
        if CODE_ENTITIES.iter().any(|&entity| text_lower.contains(entity)) {
            return EntityType::CodeEntity;
        }

        // Check for system components
        let system_keywords = ["api", "service", "database", "server", "endpoint", "table"];
        if system_keywords.iter().any(|&kw| text_lower.contains(kw)) {
            return EntityType::SystemComponent;
        }

        // Default to concept
        EntityType::Concept
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();

        let len1 = chars1.len();
        let len2 = chars2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };

                matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                    .min(matrix[i][j - 1] + 1) // insertion
                    .min(matrix[i - 1][j - 1] + cost); // substitution
            }
        }

        matrix[len1][len2]
    }
}

/// Common code/system entities for disambiguation
static CODE_ENTITIES: &[&str] = &[
    "function", "method", "class", "struct", "module", "package", "library",
    "api", "endpoint", "service", "database", "table", "column", "query",
    "algorithm", "model", "component", "system", "application", "server",
    "client", "user", "admin", "developer", "code", "implementation",
];

/// Public API function for entity disambiguation
pub async fn disambiguate_entity(entity: &Entity, context: &str) -> Result<EntityDisambiguation> {
    let disambiguator = EntityDisambiguator;
    disambiguator.disambiguate_entity(entity, context).await
}
