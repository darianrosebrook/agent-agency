//! Coreference resolution logic and caching
//!
//! This module handles pronoun resolution, entity linking, and coreference chains.

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use lru::LruCache;
use md5;
use regex::Regex;
use once_cell::sync::Lazy;
use anyhow::Result;

use crate::verification::types::*;

/// Static patterns for coreference resolution
static PRONOUNS: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        ("personal", vec!["i", "me", "my", "mine", "you", "your", "yours", "he", "him", "his", "she", "her", "hers", "it", "its", "we", "us", "our", "ours", "they", "them", "their", "theirs"]),
        ("demonstrative", vec!["this", "that", "these", "those"]),
        ("relative", vec!["who", "whom", "whose", "which", "that", "what"]),
    ])
});

/// Common code/system entities for disambiguation
static CODE_ENTITIES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "function", "method", "class", "struct", "module", "package", "library",
        "api", "endpoint", "service", "database", "table", "column", "query",
        "algorithm", "model", "component", "system", "application", "server",
        "client", "user", "admin", "developer", "code", "implementation",
    ]
});

/// Coreference resolver with caching
pub struct CoreferenceResolver {
    cache: LruCache<String, CoreferenceResolution>,
}

impl CoreferenceResolver {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
        }
    }

    /// Perform comprehensive coreference resolution on text
    pub async fn resolve_coreferences(&mut self, text: &str) -> Result<CoreferenceResolution> {
        let start_time = Instant::now();

        // Check cache first
        let cache_key = format!("{:x}", md5::compute(text));
        if let Some(cached_result) = self.cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        // Step 1: Extract entities from text
        let entities = self.extract_entities(text)?;

        // Step 2: Identify pronouns and potential antecedents
        let pronouns = self.identify_pronouns(text);

        // Step 3: Perform coreference resolution
        let chains = self.perform_coreference_resolution(text, &entities, &pronouns)?;

        // Step 4: Calculate confidence score
        let confidence_score = self.calculate_coreference_confidence(&chains, &pronouns);

        // Step 5: Identify unresolved pronouns
        let unresolved_pronouns = self.identify_unresolved_pronouns(&pronouns, &chains);

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        let result = CoreferenceResolution {
            chains,
            unresolved_pronouns,
            confidence_score,
            processing_time_ms,
        };

        // Cache the result
        self.cache.put(cache_key, result.clone());

        Ok(result)
    }

    /// Extract entities from text using rule-based and pattern matching
    fn extract_entities(&self, text: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        let text_lower = text.to_lowercase();

        // Extract code entities (functions, classes, etc.)
        for entity_type in CODE_ENTITIES.iter() {
            let pattern = format!(r"\b(?:the\s+)?{}\b", entity_type);
            if let Ok(regex) = Regex::new(&pattern) {
                for capture in regex.find_iter(&text_lower) {
                    entities.push(Entity {
                        id: format!("entity_{}", entities.len()),
                        text: capture.as_str().to_string(),
                        entity_type: EntityType::CodeEntity,
                        confidence: 0.8,
                        position: (capture.start(), capture.end()),
                        metadata: HashMap::from([("source".to_string(), "pattern_match".to_string())]),
                    });
                }
            }
        }

        // Extract system components
        let system_patterns = [
            r"\b(?:the\s+)?(?:api|endpoint|service|database|server|client)\b",
            r"\b(?:the\s+)?(?:user|admin|developer)\b",
        ];

        for pattern in &system_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for capture in regex.find_iter(&text_lower) {
                    entities.push(Entity {
                        id: format!("entity_{}", entities.len()),
                        text: capture.as_str().to_string(),
                        entity_type: EntityType::SystemComponent,
                        confidence: 0.7,
                        position: (capture.start(), capture.end()),
                        metadata: HashMap::from([("source".to_string(), "pattern_match".to_string())]),
                    });
                }
            }
        }

        Ok(entities)
    }

    /// Identify pronouns in text
    fn identify_pronouns(&self, text: &str) -> Vec<(String, (usize, usize))> {
        let mut pronouns = Vec::new();
        let text_lower = text.to_lowercase();

        for pronoun_list in PRONOUNS.values() {
            for &pronoun in pronoun_list {
                let pattern = format!(r"\b{}\b", pronoun);
                if let Ok(regex) = Regex::new(&pattern) {
                    for capture in regex.find_iter(&text_lower) {
                        pronouns.push((
                            pronoun.to_string(),
                            (capture.start(), capture.end())
                        ));
                    }
                }
            }
        }

        pronouns
    }

    /// Perform coreference resolution using rule-based approach
    fn perform_coreference_resolution(
        &self,
        text: &str,
        entities: &[Entity],
        pronouns: &[(String, (usize, usize))],
    ) -> Result<Vec<CoreferenceChain>> {
        let mut chains: Vec<CoreferenceChain> = Vec::new();

        for (pronoun, pronoun_pos) in pronouns {
            // Find potential antecedents within reasonable distance
            let antecedent_candidates = self.find_antecedent_candidates(
                text, entities, pronoun_pos, 500 // 500 chars window
            );

            if let Some(best_match) = self.select_best_antecedent(pronoun, &antecedent_candidates) {
                // Create or extend coreference chain
                let mut found_chain = false;
                for chain in &mut chains {
                    if chain.representative.id == best_match.id {
                        chain.mentions.push(Entity {
                            id: format!("mention_{}", chain.mentions.len()),
                            text: pronoun.clone(),
                            entity_type: EntityType::Other,
                            confidence: 0.6,
                            position: *pronoun_pos,
                            metadata: HashMap::from([("antecedent".to_string(), best_match.text.clone())]),
                        });
                        chain.confidence = (chain.confidence + 0.6) / 2.0;
                        found_chain = true;
                        break;
                    }
                }

                if !found_chain {
                    chains.push(CoreferenceChain {
                        representative: best_match.clone(),
                        mentions: vec![Entity {
                            id: format!("mention_0"),
                            text: pronoun.clone(),
                            entity_type: EntityType::Other,
                            confidence: 0.6,
                            position: *pronoun_pos,
                            metadata: HashMap::from([("antecedent".to_string(), best_match.text.clone())]),
                        }],
                        confidence: 0.7,
                        chain_type: CoreferenceType::Anaphoric,
                    });
                }
            }
        }

        Ok(chains)
    }

    /// Find potential antecedent candidates within text window
    fn find_antecedent_candidates<'a>(
        &self,
        text: &str,
        entities: &'a [Entity],
        pronoun_pos: &(usize, usize),
        window_size: usize,
    ) -> Vec<&'a Entity> {
        let pronoun_start = pronoun_pos.0;
        let window_start = pronoun_start.saturating_sub(window_size);

        entities.iter()
            .filter(|entity| {
                let entity_end = entity.position.1;
                entity_end < pronoun_start && entity_end >= window_start
            })
            .collect()
    }

    /// Select best antecedent based on pronoun type and entity characteristics
    fn select_best_antecedent<'a>(&self, pronoun: &str, candidates: &[&'a Entity]) -> Option<&'a Entity> {
        if candidates.is_empty() {
            return None;
        }

        // Simple heuristic: prefer entities that match pronoun semantics
        let mut best_candidate = None;
        let mut best_score = 0.0;

        for candidate in candidates {
            let mut score = candidate.confidence;

            // Boost score for semantic matches
            match pronoun {
                "it" | "this" | "that" => {
                    if matches!(candidate.entity_type, EntityType::CodeEntity | EntityType::SystemComponent) {
                        score *= 1.5;
                    }
                }
                "they" | "them" => {
                    if matches!(candidate.entity_type, EntityType::Organization) {
                        score *= 1.3;
                    }
                }
                _ => {}
            }

            if score > best_score {
                best_score = score;
                best_candidate = Some(*candidate);
            }
        }

        best_candidate
    }

    /// Calculate overall confidence score for coreference resolution
    fn calculate_coreference_confidence(
        &self,
        chains: &[CoreferenceChain],
        pronouns: &[(String, (usize, usize))],
    ) -> f64 {
        if pronouns.is_empty() {
            return 1.0;
        }

        let resolved_count = chains.iter().map(|chain| chain.mentions.len()).sum::<usize>();
        let total_pronouns = pronouns.len();

        resolved_count as f64 / total_pronouns as f64
    }

    /// Identify pronouns that could not be resolved
    fn identify_unresolved_pronouns(
        &self,
        pronouns: &[(String, (usize, usize))],
        chains: &[CoreferenceChain],
    ) -> Vec<String> {
        let resolved_positions: HashSet<_> = chains.iter()
            .flat_map(|chain| chain.mentions.iter().map(|mention| mention.position))
            .collect();

        pronouns.iter()
            .filter(|(_, pos)| !resolved_positions.contains(pos))
            .map(|(pronoun, _)| pronoun.clone())
            .collect()
    }
}

/// Public API function for coreference resolution
pub async fn resolve_coreferences(text: &str) -> Result<CoreferenceResolution> {
    let mut resolver = CoreferenceResolver::new();
    resolver.resolve_coreferences(text).await
}
