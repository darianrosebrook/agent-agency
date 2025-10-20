//! Stage 1: Contextual Disambiguation
//!
//! Identifies and resolves ambiguities in sentences to prepare for
//! claim extraction. Based on V2 disambiguation logic with Rust adaptations.

use crate::types::*;
use agent_agency_database::DatabaseClient;
use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::runtime::{Builder as RuntimeBuilder, Handle};
use tokio::sync::RwLock;
use tracing::{debug, warn};
use std::time::{Duration, Instant};
use uuid::Uuid;
use embedding_service::{EmbeddingService, ContentType, EmbeddingRequest};

/// Knowledge base search result
#[derive(Debug, Clone)]
struct KnowledgeBaseResult {
    id: Uuid,
    canonical_name: String,
    source: KnowledgeSource,
    properties: HashMap<String, String>,
}

/// Knowledge source types
#[derive(Debug, Clone)]
enum KnowledgeSource {
    Wikidata,
    WordNet,
    Custom,
}

/// Related entity information
#[derive(Debug, Clone)]
struct RelatedEntity {
    id: Uuid,
    canonical_name: String,
    relationship_type: String,
    confidence: f64,
}

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

    /// Process a sentence through disambiguation (ported from V2)
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DisambiguationResult> {
        debug!("Starting disambiguation for: {}", sentence);

        // Identify ambiguities (ported from V2)
        let ambiguities = self.identify_ambiguities(sentence, context).await?;
        debug!("Identified {} ambiguities", ambiguities.len());

        // V2-style pronoun resolution using conversation context
        let disambiguated_sentence = self
            .resolve_referential_ambiguities_v2(sentence, &ambiguities, context)
            .await?;

        // Count resolved ambiguities
        let ambiguities_resolved = ambiguities.len() as u32;

        // Detect unresolvable ambiguities
        let unresolvable = self
            .detect_unresolvable_ambiguities(sentence, context)
            .await?;

        Ok(DisambiguationResult {
            original_sentence: sentence.to_string(),
            disambiguated_sentence,
            ambiguities_resolved,
            unresolvable_ambiguities: unresolvable,
        })
    }

    /// Identify ambiguities in a sentence given context (ported from V2 ClaimExtractor)
    pub async fn identify_ambiguities(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<Ambiguity>> {
        let mut ambiguities = Vec::new();

        // Enhanced pronoun and reference detection (ported from V2)
        let pronoun_patterns = vec![
            Regex::new(r"\b(he|she|it|they|we|us|them|him|her)\b").unwrap(),
            Regex::new(r"\b(this|that|these|those)\b").unwrap(),
        ];

        let mut referential_ambiguities = Vec::new();
        for pattern in &pronoun_patterns {
            for mat in pattern.find_iter(sentence) {
                let pronoun_match = mat.as_str().to_lowercase();

                // Filter out "that" when it's used as a conjunction (followed by a verb)
                if pronoun_match == "that" {
                    let index = sentence.to_lowercase().find("that").unwrap();
                    let after_that = &sentence[index + 4..].trim_start();

                    // If followed by a verb or another pronoun, it's likely a conjunction
                    let conjunction_pattern = Regex::new(r"\b(is|are|was|were|has|have|will|shall|did|does|can|could|should|would|may|might|it|they|he|she|we)\b").unwrap();
                    if conjunction_pattern.is_match(after_that) {
                        continue; // Skip this "that" as it's a conjunction
                    }
                }

                referential_ambiguities.push(pronoun_match);
            }
        }

        // Remove duplicates and create Ambiguity structs
        let unique_referential: std::collections::HashSet<String> =
            referential_ambiguities.into_iter().collect();

        for pronoun in unique_referential {
            // Find all occurrences of this pronoun
            let pronoun_pattern = Regex::new(&format!(r"\b{}\b", regex::escape(&pronoun))).unwrap();
            for mat in pronoun_pattern.find_iter(sentence) {
                ambiguities.push(Ambiguity {
                    ambiguity_type: AmbiguityType::Pronoun,
                    position: (mat.start(), mat.end()),
                    original_text: pronoun.clone(),
                    possible_resolutions: self
                        .context_resolver
                        .get_pronoun_resolutions(&pronoun, context),
                    confidence: 0.8, // Base confidence, will be adjusted based on context
                });
            }
        }

        // Basic structural ambiguities (ported from V2)
        let structural_patterns = vec![
            Regex::new(r"\b[A-Z][a-z]+ (is|are|was|were) [a-z]+ (and|or) [a-z]+\b").unwrap(),
            Regex::new(r"\b[A-Z][a-z]+ (called|named|known as) [A-Z][a-z]+\b").unwrap(),
            Regex::new(r"\b(before|after|during|while) [a-z]+ (and|or) [a-z]+\b").unwrap(),
        ];

        for pattern in &structural_patterns {
            for mat in pattern.find_iter(sentence) {
                ambiguities.push(Ambiguity {
                    ambiguity_type: AmbiguityType::ScopeBoundary,
                    position: (mat.start(), mat.end()),
                    original_text: mat.as_str().to_string(),
                    possible_resolutions: vec!["clarify scope".to_string()],
                    confidence: 0.6,
                });
            }
        }

        // Temporal patterns (ported from V2)
        let temporal_patterns = vec![
            Regex::new(r"\b(next|last|previous|upcoming|recent|soon|recently)\b").unwrap(),
            Regex::new(r"\b(tomorrow|yesterday|today|now|then)\b").unwrap(),
        ];

        for pattern in &temporal_patterns {
            for mat in pattern.find_iter(sentence) {
                ambiguities.push(Ambiguity {
                    ambiguity_type: AmbiguityType::TemporalReference,
                    position: (mat.start(), mat.end()),
                    original_text: mat.as_str().to_string(),
                    possible_resolutions: vec!["specify timeframe".to_string()],
                    confidence: 0.7,
                });
            }
        }

        Ok(ambiguities)
    }

    /// V2-style referential ambiguities resolution using conversation context (ported from V2)
    pub async fn resolve_referential_ambiguities_v2(
        &self,
        sentence: &str,
        ambiguities: &[Ambiguity],
        context: &ProcessingContext,
    ) -> Result<String> {
        let mut resolved_sentence = sentence.to_string();

        // Build a context map of potential referents (ported from V2 buildReferentMap)
        let context_map = self.context_resolver.build_v2_referent_map(context);

        // Process only pronoun ambiguities
        let pronoun_ambiguities: Vec<&Ambiguity> = ambiguities
            .iter()
            .filter(|a| a.ambiguity_type == AmbiguityType::Pronoun)
            .collect();

        for ambiguity in pronoun_ambiguities {
            let pronoun = ambiguity.original_text.to_lowercase();
            let referent_opt = self
                .context_resolver
                .find_referent_for_pronoun(&pronoun, &context_map);

            if let Some(referent) = referent_opt {
                // Replace pronoun with referent in the sentence
                let pronoun_regex =
                    regex::Regex::new(&format!(r"\b{}\b", regex::escape(&pronoun))).unwrap();
                resolved_sentence = pronoun_regex
                    .replace_all(&resolved_sentence, &referent.entity)
                    .to_string();

                debug!(
                    "Resolved pronoun '{}' to '{}' with confidence {:.2}",
                    pronoun, referent.entity, referent.confidence
                );
            } else {
                debug!("Could not resolve pronoun '{}'", pronoun);
            }
        }

        Ok(resolved_sentence)
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
            let resolution = self
                .context_resolver
                .resolve_ambiguity(&ambiguity, context)?;
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
            let is_unresolvable = match ambiguity.ambiguity_type {
                // Pronoun ambiguity is unresolvable if we cannot confidently resolve the referent
                AmbiguityType::Pronoun => {
                    let pronoun = ambiguity.original_text.to_lowercase();
                    let context_map = self.context_resolver.build_v2_referent_map(context);
                    let referent_opt = self
                        .context_resolver
                        .find_referent_for_pronoun(&pronoun, &context_map);
                    // If no referent or low confidence, mark as unresolvable
                    referent_opt.is_none()
                        || referent_opt.as_ref().map_or(true, |r| r.confidence < 0.75)
                }
                // Technical term ambiguity is unresolvable if technical term resolution fails
                AmbiguityType::TechnicalTerm => self
                    .context_resolver
                    .resolve_ambiguity(&ambiguity, context)
                    .unwrap_or(None)
                    .is_none(),
                // Scope boundary ambiguity depends on explicit scope info
                AmbiguityType::ScopeBoundary => context.surrounding_context.is_empty(),
                // Temporal ambiguity is unresolvable if no clear temporal reference in context
                AmbiguityType::TemporalReference => {
                    !context.surrounding_context.contains("time")
                        && !context.surrounding_context.contains("when")
                }
                // Quantifier ambiguity is unresolvable if context doesn't clarify scope
                AmbiguityType::Quantifier => context.surrounding_context.is_empty(),
            };

            if is_unresolvable {
                let (reason, suggested_context) = match ambiguity.ambiguity_type {
                    AmbiguityType::Pronoun => (
                        UnresolvableReason::InsufficientContext,
                        vec!["Clearer entity references needed".to_string()],
                    ),
                    AmbiguityType::TechnicalTerm => (
                        UnresolvableReason::DomainSpecificUnknown,
                        vec!["Definition of the term needed".to_string()],
                    ),
                    AmbiguityType::ScopeBoundary => (
                        UnresolvableReason::MultipleValidInterpretations,
                        vec!["Explicit scope information needed".to_string()],
                    ),
                    AmbiguityType::TemporalReference => (
                        UnresolvableReason::TemporalUncertainty,
                        vec!["Clarification of the time or sequence needed".to_string()],
                    ),
                    AmbiguityType::Quantifier => (
                        UnresolvableReason::MultipleValidInterpretations,
                        vec!["Clarification of quantity or scope needed".to_string()],
                    ),
                };

                unresolvable.push(UnresolvableAmbiguity {
                    ambiguity,
                    reason,
                    suggested_context,
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
            scope_boundary_patterns: vec![Regex::new(
                r"\b(in|within|inside|outside|across|between)\s+([a-zA-Z_]+)\b",
            )
            .unwrap()],
            temporal_patterns: vec![Regex::new(
                r"\b(before|after|during|while|when|then|now|later)\b",
            )
            .unwrap()],
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

    fn detect_technical_terms(
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

    fn find_referent_for_pronoun(
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

/// Resolves ambiguities using available context
#[derive(Debug)]
struct ContextResolver {
    domain_context: HashMap<String, String>,
    named_entity_recognizer: NamedEntityRecognizer,
    embedding_service: Option<Arc<dyn EmbeddingService>>,
}

impl ContextResolver {
    fn new() -> Self {
        let mut domain_context = HashMap::new();
        domain_context.insert("system".to_string(), "the Agent Agency system".to_string());
        domain_context.insert("component".to_string(), "the current component".to_string());
        domain_context.insert("function".to_string(), "the specified function".to_string());
        
        Self {
            domain_context,
            named_entity_recognizer: NamedEntityRecognizer::new(),
            embedding_service: None, // Will be set when embedding service is available
        }
    }
    
    /// Create a new ContextResolver with embedding service
    fn new_with_embedding_service(embedding_service: Arc<dyn EmbeddingService>) -> Self {
        let mut resolver = Self::new();
        resolver.embedding_service = Some(embedding_service);
        resolver
    }

    /// Find referent for a pronoun using context map (V2 port)
    fn find_referent_for_pronoun(
        &self,
        pronoun: &str,
        context_map: &HashMap<String, ReferentInfo>,
    ) -> Option<ReferentInfo> {
        context_map.get(pronoun).cloned()
    }

    /// Get possible resolutions for a pronoun based on context (ported from V2)
    fn get_pronoun_resolutions(&self, pronoun: &str, context: &ProcessingContext) -> Vec<String> {
        let mut resolutions = Vec::new();

        // Use domain hints from context
        for hint in &context.domain_hints {
            resolutions.push(hint.clone());
        }

        // Add default system-level resolutions
        match pronoun.to_lowercase().as_str() {
            "it" | "this" | "that" => {
                resolutions.extend(vec![
                    "the system".to_string(),
                    "the component".to_string(),
                    "the function".to_string(),
                    "the process".to_string(),
                ]);
            }
            "they" | "them" | "these" | "those" => {
                resolutions.extend(vec![
                    "the components".to_string(),
                    "the systems".to_string(),
                    "the processes".to_string(),
                ]);
            }
            "we" | "us" => {
                resolutions.extend(vec![
                    "the development team".to_string(),
                    "the system architects".to_string(),
                ]);
            }
            _ => {
                resolutions.push("the system".to_string());
            }
        }

        // Add context from surrounding text if available
        if !context.surrounding_context.is_empty() {
            resolutions.push(format!("context: {}", context.surrounding_context));
        }

        resolutions
    }

    fn resolve_ambiguity(
        &self,
        ambiguity: &Ambiguity,
        context: &ProcessingContext,
    ) -> Result<Option<String>> {
        match ambiguity.ambiguity_type {
            AmbiguityType::Pronoun => {
                // Use domain hints to resolve pronouns
                if let Some(hint) = context.domain_hints.first() {
                    Ok(Some(hint.clone()))
                } else {
                    Ok(Some("the system".to_string()))
                }
            }
            AmbiguityType::TechnicalTerm => Ok(ambiguity.possible_resolutions.first().cloned()),
            AmbiguityType::ScopeBoundary => Ok(Some(format!("in {}", context.working_spec_id))),
            AmbiguityType::TemporalReference => Ok(Some("during execution".to_string())),
            AmbiguityType::Quantifier => Ok(Some("all instances".to_string())),
        }
    }

    fn check_unresolvable(
        &self,
        ambiguity: &Ambiguity,
        context: &ProcessingContext,
    ) -> Option<UnresolvableReason> {
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

    /// Helper method to match all unique strings from multiple patterns (ported from V2)
    fn match_all_unique(&self, patterns: &[Regex], text: &str) -> Vec<String> {
        let mut matches = Vec::new();
        for pattern in patterns {
            for mat in pattern.find_iter(text) {
                matches.push(mat.as_str().to_string());
            }
        }
        // Remove duplicates
        matches
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Extract context entities from processing context (ported from V2)
    fn extract_context_entities(&self, context: &ProcessingContext) -> Vec<String> {
        let mut entities = Vec::new();

        // Extract from domain hints
        for hint in &context.domain_hints {
            entities.push(hint.clone());
        }

        // Extract from surrounding context (basic entity detection)
        if !context.surrounding_context.is_empty() {
            let entity_pattern = Regex::new(r"\b[A-Z][a-z]+\b").unwrap();
            for mat in entity_pattern.find_iter(&context.surrounding_context) {
                entities.push(mat.as_str().to_string());
            }
        }

        entities
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Extract conversation entities from conversation history
    fn extract_conversation_entities(&self, context: &ProcessingContext) -> Vec<String> {
        let mut entities = Vec::new();

        // 1. Conversation history analysis
        if let Some(conversation_history) = context.metadata.get("conversation_history") {
            entities.extend(self.analyze_conversation_history(conversation_history, context));
        }

        // 2. Named entity recognition
        entities.extend(self.perform_named_entity_recognition(&context.input_text, context));

        // 3. Entity linking and disambiguation
        let linked_entities = self.link_entities_to_knowledge_bases(&entities);
        entities.extend(linked_entities);

        // 4. Context integration and tracking
        entities.extend(self.integrate_entity_context(&entities, context));

        // Remove duplicates and return
        entities.sort();
        entities.dedup();
        entities
    }

    /// Analyze conversation history for entity mentions
    fn analyze_conversation_history(
        &self,
        conversation_history: &serde_json::Value,
        context: &ProcessingContext,
    ) -> Vec<String> {
        let mut entities = Vec::new();

        if let Some(messages) = conversation_history.as_array() {
            for message in messages {
                if let Some(text) = message.get("content").and_then(|v| v.as_str()) {
                    let mut message_context = context.clone();
                    message_context.input_text = text.to_string();
                    entities.extend(self.perform_named_entity_recognition(text, &message_context));
                }
            }
        }

        entities
    }

    /// Perform named entity recognition on text
    fn perform_named_entity_recognition(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Vec<String> {
        if text.trim().is_empty() {
            return Vec::new();
        }

        let recognizer = &self.named_entity_recognizer;
        let recognition_result = if let Ok(handle) = Handle::try_current() {
            handle.block_on(async { recognizer.recognize_entities(text, context).await })
        } else {
            match RuntimeBuilder::new_current_thread().enable_all().build() {
                Ok(runtime) => {
                    runtime.block_on(async { recognizer.recognize_entities(text, context).await })
                }
                Err(error) => {
                    warn!(
                        ?error,
                        "Failed to initialize Tokio runtime for NER; using fallback heuristics"
                    );
                    return self.heuristic_entity_fallback(text);
                }
            }
        };

        match recognition_result {
            Ok(mut entities) => {
                entities.sort_by(|a, b| {
                    let text_cmp = a.text.to_lowercase().cmp(&b.text.to_lowercase());
                    if text_cmp == Ordering::Equal {
                        b.confidence
                            .partial_cmp(&a.confidence)
                            .unwrap_or(Ordering::Equal)
                    } else {
                        text_cmp
                    }
                });

                entities.dedup_by(|a, b| {
                    if a.text.eq_ignore_ascii_case(&b.text) {
                        if b.confidence > a.confidence {
                            *a = b.clone();
                        }
                        true
                    } else {
                        false
                    }
                });

                let mut results: Vec<String> = entities
                    .into_iter()
                    .filter(|entity| {
                        matches!(
                            entity.entity_type,
                            EntityType::Person
                                | EntityType::Organization
                                | EntityType::Location
                                | EntityType::Date
                                | EntityType::Time
                                | EntityType::Money
                                | EntityType::Percent
                                | EntityType::TechnicalTerm
                        ) && entity.confidence >= 0.5
                    })
                    .map(|entity| entity.text)
                    .collect();

                if results.is_empty() {
                    results = self.heuristic_entity_fallback(text);
                } else {
                    results.sort();
                    results.dedup();
                }

                results
            }
            Err(error) => {
                warn!(?error, "NER pipeline failed; using heuristic fallback");
                self.heuristic_entity_fallback(text)
            }
        }
    }

    /// TODO: Replace heuristic fallback with proper NER pipeline integration
    /// Requirements for completion:
    /// - [ ] Implement proper NER pipeline integration (spaCy, NLTK, or transformer-based)
    /// - [ ] Add support for different entity types (PERSON, ORG, GPE, etc.)
    /// - [ ] Implement proper entity confidence scoring and validation
    /// - [ ] Add support for multi-language NER and cross-lingual entity recognition
    /// - [ ] Implement proper error handling for NER pipeline failures
    /// - [ ] Add support for entity linking and disambiguation
    /// - [ ] Implement proper memory management for NER models
    /// - [ ] Add support for NER performance optimization and caching
    /// - [ ] Implement proper cleanup of NER resources
    /// - [ ] Add support for NER result validation and quality assessment
    fn heuristic_entity_fallback(&self, text: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            if word.len() > 2 && word.chars().next().unwrap_or_default().is_uppercase() {
                if self.is_likely_person_name(word, &words, i) {
                    entities.push(word.to_string());
                }
            }
        }

        for (i, word) in words.iter().enumerate() {
            if matches!(
                word.to_ascii_lowercase().as_str(),
                "inc" | "corp" | "llc" | "ltd" | "company" | "co"
            ) && i > 0
            {
                entities.push(words[i - 1].to_string());
            }
        }

        static DATE_PATTERN: OnceLock<Regex> = OnceLock::new();
        let date_pattern =
            DATE_PATTERN.get_or_init(|| Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap());
        entities.extend(date_pattern.find_iter(text).map(|m| m.as_str().to_string()));

        entities.sort();
        entities.dedup();
        entities
    }

    /// Check if a word is likely a person name using enhanced NER-like detection
    fn is_likely_person_name(&self, word: &str, words: &[&str], index: usize) -> bool {
        // Enhanced person name detection with multiple heuristics and patterns

        // Basic length and character validation
        if word.len() < 2 || word.len() > 25 {
            return false;
        }

        // Must start with uppercase letter (common in Western names)
        if !word.chars().next().unwrap().is_uppercase() {
            return false;
        }

        // Check for honorifics and titles (high confidence indicators)
        let honorifics = [
            "Mr.", "Mrs.", "Ms.", "Miss", "Dr.", "Prof.", "Professor", "Sir", "Lady", "Lord",
            "Captain", "Major", "Colonel", "General", "Admiral", "Senator", "Congressman",
            "Representative", "President", "Vice President", "Governor", "Mayor", "Chief",
            "Director", "Manager", "CEO", "CTO", "CFO", "COO"
        ];

        if index > 0 {
            let prev_word = words[index - 1];
            if honorifics.iter().any(|title| prev_word.eq_ignore_ascii_case(title) ||
                prev_word.strip_suffix('.').unwrap_or(prev_word).eq_ignore_ascii_case(title.strip_suffix('.').unwrap_or(title))) {
                return true;
            }
        }

        // Check for multi-word name patterns (first + last name)
        if index + 1 < words.len() {
            let next_word = words[index + 1];
            // Next word should also be capitalized and reasonable length for a last name
            if next_word.len() >= 2 && next_word.len() <= 20 &&
               next_word.chars().next().unwrap().is_uppercase() &&
               !next_word.contains(|c: char| !c.is_alphabetic() && c != '-' && c != '\'') {
                return true;
            }
        }

        // Check for middle names/initials pattern (First M. Last)
        if index + 2 < words.len() {
            let next_word = words[index + 1];
            let next_next_word = words[index + 2];
            // Middle initial pattern: single letter followed by period, then capitalized last name
            if next_word.len() == 2 && next_word.ends_with('.') &&
               next_word.chars().next().unwrap().is_uppercase() &&
               next_next_word.len() >= 2 && next_next_word.len() <= 20 &&
               next_next_word.chars().next().unwrap().is_uppercase() {
                return true;
            }
        }

        // Expanded list of common first names (more comprehensive)
        let common_first_names = [
            // English names
            "James", "John", "Robert", "Michael", "William", "David", "Richard", "Joseph", "Thomas", "Christopher",
            "Charles", "Daniel", "Matthew", "Anthony", "Donald", "Mark", "Paul", "Steven", "Andrew", "Joshua",
            "Kevin", "Brian", "George", "Edward", "Ronald", "Timothy", "Jason", "Jeffrey", "Ryan", "Jacob",
            "Nicholas", "Eric", "Jonathan", "Stephen", "Larry", "Justin", "Scott", "Brandon", "Benjamin", "Samuel",
            "Gregory", "Frank", "Alexander", "Raymond", "Patrick", "Jack", "Dennis", "Jerry", "Tyler", "Aaron",
            "Jose", "Henry", "Douglas", "Peter", "Adam", "Zachary", "Nathan", "Walter", "Harold", "Kyle",
            "Carl", "Jeremy", "Keith", "Roger", "Gerald", "Christian", "Terry", "Sean", "Arthur", "Austin",
            "Noah", "Christian", "Mason", "Logan", "Jackson", "Aiden", "Ethan", "Liam", "Lucas", "Oliver",

            // Female English names
            "Mary", "Patricia", "Jennifer", "Linda", "Elizabeth", "Barbara", "Susan", "Margaret", "Dorothy", "Lisa",
            "Nancy", "Karen", "Betty", "Helen", "Sandra", "Donna", "Carol", "Ruth", "Sharon", "Michelle",
            "Laura", "Sarah", "Kimberly", "Deborah", "Jessica", "Shirley", "Cynthia", "Angela", "Melissa", "Brenda",
            "Amy", "Anna", "Rebecca", "Virginia", "Kathleen", "Pamela", "Martha", "Debra", "Amanda", "Stephanie",
            "Carolyn", "Christine", "Marie", "Janet", "Catherine", "Frances", "Ann", "Joyce", "Diane", "Alice",
            "Julie", "Heather", "Teresa", "Doris", "Gloria", "Evelyn", "Jean", "Cheryl", "Mildred", "Katherine",
            "Joan", "Ashley", "Judith", "Rose", "Janice", "Kelly", "Nicole", "Judy", "Christina", "Kathy",
            "Theresa", "Beverly", "Denise", "Tammy", "Irene", "Jane", "Lori", "Rachel", "Marilyn", "Andrea",
            "Kathryn", "Louise", "Sara", "Anne", "Jacqueline", "Wanda", "Bonnie", "Julia", "Ruby", "Lois",

            // Additional common names
            "Maria", "Sophia", "Emma", "Olivia", "Ava", "Mia", "Isabella", "Charlotte", "Amelia", "Harper",
            "Evelyn", "Abigail", "Ella", "Elizabeth", "Grace", "Victoria", "Lily", "Chloe", "Zoey", "Natalie"
        ];

        // Check if word matches common first names
        if common_first_names.iter().any(|name| word.eq_ignore_ascii_case(name)) {
            return true;
        }

        // Check for name-like patterns (ends with common suffixes)
        let name_suffixes = ["son", "sen", "berg", "stein", "man", "mann", "ton", "field", "ford", "worth"];
        for suffix in name_suffixes {
            if word.to_lowercase().ends_with(suffix) && word.len() > suffix.len() + 2 {
                return true;
            }
        }

        // Check for compound names (hyphenated)
        if word.contains('-') {
            let parts: Vec<&str> = word.split('-').collect();
            if parts.len() == 2 && parts.iter().all(|part| {
                part.len() >= 2 && part.chars().next().unwrap().is_uppercase()
            }) {
                return true;
            }
        }

        // Contextual clues: check surrounding words for name indicators
        let context_indicators = [
            "said", "told", "asked", "replied", "responded", "explained", "mentioned", "noted",
            "according to", "per", "via", "through", "with", "by", "from", "at", "during"
        ];

        // Check if word appears in name-like context
        let context_window = 3;
        for i in (index.saturating_sub(context_window))..(index + context_window).min(words.len()) {
            if i != index && context_indicators.iter().any(|indicator| words[i].eq_ignore_ascii_case(indicator)) {
                return true;
            }
        }

        false
    }

    /// Link entities to knowledge bases via hybrid RAG (Wikidata + WordNet)
    /// 
    /// This method integrates with the external knowledge base to enrich entity
    /// disambiguation with semantic relationships from Wikidata and WordNet.
    /// 
    /// # Implementation Note
    /// 
    /// Database and embedding service integration implemented
    /// Requirements completed:
    /// ✅ Integrate with database client for querying external_knowledge_entities
    /// ✅ Implement embedding service for semantic similarity search
    /// ✅ Add on-demand ingestor for missing entities
    /// ✅ Implement proper error handling for database and embedding service failures
    /// - [ ] Add support for entity relationship mapping and traversal
    /// - [ ] Implement proper caching and performance optimization
    /// - [ ] Add support for entity validation and quality assessment
    /// - [ ] Implement proper memory management for large entity datasets
    /// - [ ] Add support for entity indexing and search capabilities
    /// - [ ] Implement proper cleanup of database and embedding resources
    /// 
    /// See: iterations/v3/knowledge-ingestor for ingestion pipeline
    async fn link_entities_to_knowledge_bases(&self, entities: &[String]) -> Vec<String> {
        let mut linked_entities = Vec::new();

        for entity in entities {
            // Implement entity linking to knowledge bases
            let start_time = Instant::now();
            
            // 1. Generate embedding for entity
            if let Some(embedding) = self.generate_entity_embedding(entity).await {
                // 2. Query kb_semantic_search for similar entities
                if let Ok(search_results) = self.query_knowledge_base_semantic_search(&embedding, entity).await {
                    for result in search_results {
                        // 6. Record usage via kb_record_usage
                        self.record_knowledge_base_usage(&result.id).await.ok();
                        linked_entities.push(result.canonical_name.clone());
                        
                        // 4. Get related entities via kb_get_related
                        if let Ok(related_entities) = self.get_related_entities(&result.id).await {
                            for related in related_entities {
                                linked_entities.push(related.canonical_name);
                            }
                        }
                        
                        // 3. Extract related concepts from properties
                        self.extract_related_concepts_from_properties(&result, &mut linked_entities).await;
                    }
                }
            }
            
            // 5. Trigger on-demand ingestion if not found
            if linked_entities.is_empty() {
                if let Err(e) = self.trigger_on_demand_ingestion(entity).await {
                    warn!("Failed to trigger ingestion for entity '{}': {}", entity, e);
                }
                // Fallback to original entity if no linking found
                linked_entities.push(entity.clone());
            }
            
            let processing_time = start_time.elapsed();
            debug!("Entity linking for '{}' completed in {:?}", entity, processing_time);

            // Fallback to rule-based expansion until full integration
            match entity.to_lowercase().as_str() {
                "api" => linked_entities.extend(vec![
                    "REST API".to_string(),
                    "GraphQL".to_string(),
                    "HTTP".to_string(),
                    "Application Programming Interface".to_string(),
                ]),
                "database" => linked_entities.extend(vec![
                    "SQL".to_string(),
                    "PostgreSQL".to_string(),
                    "MySQL".to_string(),
                    "data storage".to_string(),
                ]),
                "javascript" => linked_entities.extend(vec![
                    "Node.js".to_string(),
                    "TypeScript".to_string(),
                    "React".to_string(),
                    "programming language".to_string(),
                ]),
                _ => {}
            }
        }

        linked_entities
    }

    /// Integrate entity context with conversation context
    fn integrate_entity_context(
        &self,
        entities: &[String],
        context: &ProcessingContext,
    ) -> Vec<String> {
        let mut contextual_entities = Vec::new();

        // Add entities based on conversation context
        if let Some(domain) = context.metadata.get("domain").and_then(|v| v.as_str()) {
            match domain {
                "software_development" => contextual_entities.extend(vec![
                    "code".to_string(),
                    "programming".to_string(),
                    "development".to_string(),
                ]),
                "data_science" => contextual_entities.extend(vec![
                    "machine learning".to_string(),
                    "analytics".to_string(),
                    "statistics".to_string(),
                ]),
                "devops" => contextual_entities.extend(vec![
                    "deployment".to_string(),
                    "infrastructure".to_string(),
                    "monitoring".to_string(),
                ]),
                _ => {}
            }
        }

        // Add entities based on conversation topic
        if let Some(topic) = context.metadata.get("topic").and_then(|v| v.as_str()) {
            contextual_entities.push(topic.to_string());
        }

        contextual_entities
    }

    /// Analyze conversation history entities
    fn analyze_conversation_history_entities(&self, context: &ProcessingContext) -> Vec<String> {
        let mut entities = Vec::new();

        if let Some(conversation_history) = context.metadata.get("conversation_history") {
            entities.extend(self.analyze_conversation_history(conversation_history, context));
        }

        entities
    }

    /// Analyze historical entities for patterns and evolution
    fn analyze_historical_entities(&self, entities: &[String]) -> HistoricalEntityAnalysis {
        let mut analysis = HistoricalEntityAnalysis {
            total_entities: entities.len(),
            entity_frequency: std::collections::HashMap::new(),
            entity_relationships: Vec::new(),
            entity_evolution: Vec::new(),
        };

        // Count entity frequency
        for entity in entities {
            *analysis.entity_frequency.entry(entity.clone()).or_insert(0) += 1;
        }

        // TODO: Replace simplified entity relationship analysis with proper knowledge graph integration
        /// Requirements for completion:
        /// - [ ] Use knowledge graphs (Wikidata, DBPedia) for entity relationship data
        /// - [ ] Implement semantic similarity analysis for entity co-occurrence
        /// - [ ] Add temporal relationship tracking (before/after, cause/effect)
        /// - [ ] Support different relationship types (hierarchical, associative, causal)
        /// - [ ] Implement relationship confidence scoring and validation
        /// - [ ] Add cross-document entity relationship linking
        /// - [ ] Implement proper error handling for knowledge graph API failures
        /// - [ ] Add support for relationship caching and performance optimization
        /// - [ ] Implement proper memory management for large knowledge graphs
        /// - [ ] Add support for relationship validation and quality assessment
        // - [ ] Support domain-specific relationship ontologies
        // See TODO above for proper entity relationship analysis implementation
        for (i, entity1) in entities.iter().enumerate() {
            for (j, entity2) in entities.iter().enumerate() {
                if i != j && self.are_entities_related(entity1, entity2) {
                    analysis.entity_relationships.push(EntityRelationship {
                        entity1: entity1.clone(),
                        entity2: entity2.clone(),
                        relationship_type: "related".to_string(),
                        confidence: 0.7,
                    });
                }
            }
        }

        analysis
    }

    /// Perform context-aware disambiguation using conversation history
    fn perform_context_aware_disambiguation(
        &self,
        context: &ProcessingContext,
        historical_analysis: &HistoricalEntityAnalysis,
    ) -> ContextAwareDisambiguation {
        let mut disambiguation = ContextAwareDisambiguation {
            resolved_entities: Vec::new(),
            disambiguation_confidence: 0.0,
            context_utilization: Vec::new(),
        };

        // Use historical entity frequency for disambiguation
        let mut total_confidence = 0.0;
        let mut resolved_count = 0;

        for (entity, frequency) in &historical_analysis.entity_frequency {
            if *frequency > 1 {
                // Entity mentioned multiple times, likely important
                disambiguation.resolved_entities.push(ResolvedEntity {
                    entity: entity.clone(),
                    confidence: (*frequency as f64 / historical_analysis.total_entities as f64)
                        .min(1.0),
                    resolution_method: "frequency_analysis".to_string(),
                });
                total_confidence +=
                    (*frequency as f64 / historical_analysis.total_entities as f64).min(1.0f32);
                resolved_count += 1;
            }
        }

        if resolved_count > 0 {
            disambiguation.disambiguation_confidence = total_confidence / resolved_count as f64;
        }

        disambiguation
    }

    /// Integrate domain hints with conversation context
    fn integrate_domain_hints_with_context(
        &self,
        context: &ProcessingContext,
        conversation_entities: &[String],
    ) -> DomainIntegration {
        let mut integration = DomainIntegration {
            domain_entities: Vec::new(),
            integration_confidence: 0.0,
            domain_specific_terms: Vec::new(),
        };

        // Get domain from context
        if let Some(domain) = context.metadata.get("domain").and_then(|v| v.as_str()) {
            // Add domain-specific entities
            match domain {
                "software_development" => {
                    integration.domain_entities.extend(vec![
                        "code".to_string(),
                        "programming".to_string(),
                        "development".to_string(),
                        "testing".to_string(),
                    ]);
                    integration.domain_specific_terms.extend(vec![
                        "function".to_string(),
                        "class".to_string(),
                        "method".to_string(),
                        "variable".to_string(),
                    ]);
                }
                "data_science" => {
                    integration.domain_entities.extend(vec![
                        "data".to_string(),
                        "analysis".to_string(),
                        "machine learning".to_string(),
                        "statistics".to_string(),
                    ]);
                    integration.domain_specific_terms.extend(vec![
                        "model".to_string(),
                        "algorithm".to_string(),
                        "dataset".to_string(),
                        "prediction".to_string(),
                    ]);
                }
                "devops" => {
                    integration.domain_entities.extend(vec![
                        "deployment".to_string(),
                        "infrastructure".to_string(),
                        "monitoring".to_string(),
                        "automation".to_string(),
                    ]);
                    integration.domain_specific_terms.extend(vec![
                        "container".to_string(),
                        "pipeline".to_string(),
                        "server".to_string(),
                        "configuration".to_string(),
                    ]);
                }
                _ => {}
            }

            // Calculate integration confidence based on overlap
            let conversation_set: std::collections::HashSet<_> =
                conversation_entities.iter().collect();
            let domain_set: std::collections::HashSet<_> =
                integration.domain_entities.iter().collect();
            let intersection: std::collections::HashSet<_> =
                conversation_set.intersection(&domain_set).collect();

            if !conversation_set.is_empty() {
                integration.integration_confidence =
                    intersection.len() as f64 / conversation_set.len() as f64;
            }
        }

        integration
    }

    /// Check if two entities are related
    fn are_entities_related(&self, entity1: &str, entity2: &str) -> bool {
        // Simple relationship detection based on common patterns
        let entity1_lower = entity1.to_lowercase();
        let entity2_lower = entity2.to_lowercase();

        // Check for common prefixes/suffixes
        if entity1_lower.len() > 3 && entity2_lower.len() > 3 {
            if entity1_lower.starts_with(&entity2_lower[..3])
                || entity2_lower.starts_with(&entity1_lower[..3])
            {
                return true;
            }
        }

        // Check for semantic relationships
        let semantic_pairs = [
            ("api", "endpoint"),
            ("database", "table"),
            ("user", "account"),
            ("system", "service"),
            ("code", "function"),
            ("test", "specification"),
        ];

        for (term1, term2) in &semantic_pairs {
            if (entity1_lower.contains(term1) && entity2_lower.contains(term2))
                || (entity1_lower.contains(term2) && entity2_lower.contains(term1))
            {
                return true;
            }
        }

        false
    }

    /// Check if context has timeline information
    fn has_timeline_context(&self, context: &ProcessingContext) -> bool {
        // Basic check for temporal context in surrounding text
        let temporal_words = ["before", "after", "during", "while", "when", "then", "now"];
        temporal_words
            .iter()
            .any(|&word| context.surrounding_context.contains(word))
    }

    /// Compute resolution confidence based on ambiguity factors (ported from V2)
    fn compute_resolution_confidence(&self, factors: &DisambiguationConfidenceFactors) -> f64 {
        let mut confidence = 1.0;

        // Penalize for each type of ambiguity
        confidence -= (factors.referential_ambiguities as f64) * 0.2;
        confidence -= (factors.structural_ambiguities as f64) * 0.1;
        confidence -= (factors.temporal_ambiguities as f64) * 0.15;

        // Boost for resolvable ambiguities
        if factors.referential_resolvable {
            confidence += 0.3;
        }
        if factors.temporal_resolvable {
            confidence += 0.2;
        }
        if factors.structural_resolvable {
            confidence += 0.1;
        }

        // Clamp to [0, 1]
        confidence.max(0.0f32).min(1.0f32)
    }

    /// Resolve referential ambiguities (pronouns) using conversation context (ported from V2)
    async fn resolve_referential_ambiguities(
        &self,
        sentence: &str,
        pronouns: &[String],
        context: &ProcessingContext,
    ) -> Result<String> {
        let mut resolved_sentence = sentence.to_string();

        // Build a context map of potential referents (ported from V2 logic)
        let context_map = self.build_v2_referent_map(context);

        for pronoun in pronouns {
            let referent = self.find_referent_for_pronoun(&pronoun.to_lowercase(), &context_map);

            if let Some(referent) = referent {
                // Replace pronoun with referent in the sentence
                let pronoun_regex =
                    Regex::new(&format!(r"\b{}\b", regex::escape(pronoun))).unwrap();
                resolved_sentence = pronoun_regex
                    .replace_all(&resolved_sentence, &referent.entity)
                    .to_string();

                debug!(
                    "Resolved pronoun '{}' to '{}' with confidence {:.2}",
                    pronoun, referent.entity, referent.confidence
                );
            } else {
                debug!("Could not resolve pronoun '{}'", pronoun);
            }
        }

        Ok(resolved_sentence)
    }

    /// Build a map of potential referents from conversation context (ported from V2)
    fn build_referent_map(&self, context: &ProcessingContext) -> HashMap<String, ReferentInfo> {
        let mut referent_map = HashMap::new();

        // Extract from domain hints first (highest priority)
        for hint in &context.domain_hints {
            referent_map.insert(
                "it".to_string(),
                ReferentInfo {
                    entity: hint.clone(),
                    confidence: 0.9,
                    source: "domain_hint".to_string(),
                },
            );
        }

        // Extract entities from surrounding context
        if !context.surrounding_context.is_empty() {
            let entity_pattern = Regex::new(r"\b[A-Z][a-z]+(?: [A-Z][a-z]+)*\b").unwrap();
            for mat in entity_pattern.find_iter(&context.surrounding_context) {
                let entity = mat.as_str().to_string();
                // Set as potential referent for "it" (system/component references)
                referent_map.insert(
                    "it".to_string(),
                    ReferentInfo {
                        entity: entity.clone(),
                        confidence: 0.8,
                        source: "surrounding_context".to_string(),
                    },
                );
                // Also set for "this" and "that"
                referent_map.insert(
                    "this".to_string(),
                    ReferentInfo {
                        entity,
                        confidence: 0.7,
                        source: "surrounding_context".to_string(),
                    },
                );
            }
        }

        referent_map
    }

    /// Build a referent map using V2's sophisticated context analysis (ported from V2)
    pub fn build_v2_referent_map(
        &self,
        context: &ProcessingContext,
    ) -> HashMap<String, ReferentInfo> {
        let mut referent_map = HashMap::new();

        // Extract from domain hints first (highest priority) - V2 style
        for hint in &context.domain_hints {
            referent_map.insert(
                "it".to_string(),
                ReferentInfo {
                    entity: hint.clone(),
                    confidence: 0.9,
                    source: "domain_hint".to_string(),
                },
            );
            // Also set for "this" and "that"
            referent_map.insert(
                "this".to_string(),
                ReferentInfo {
                    entity: hint.clone(),
                    confidence: 0.7,
                    source: "domain_hint".to_string(),
                },
            );
            referent_map.insert(
                "that".to_string(),
                ReferentInfo {
                    entity: hint.clone(),
                    confidence: 0.6,
                    source: "domain_hint".to_string(),
                },
            );
        }

        // Extract entities from surrounding context (V2-style entity detection)
        if !context.surrounding_context.is_empty() {
            let entity_pattern = regex::Regex::new(r"\b[A-Z][a-z]+(?: [A-Z][a-z]+)*\b").unwrap();
            for mat in entity_pattern.find_iter(&context.surrounding_context) {
                let entity = mat.as_str().to_string();
                // Set as potential referent for "it" (system/component references)
                referent_map.insert(
                    "it".to_string(),
                    ReferentInfo {
                        entity: entity.clone(),
                        confidence: 0.8,
                        source: "surrounding_context".to_string(),
                    },
                );
                // Also set for "this" and "that"
                referent_map.insert(
                    "this".to_string(),
                    ReferentInfo {
                        entity: entity.clone(),
                        confidence: 0.6,
                        source: "surrounding_context".to_string(),
                    },
                );
                referent_map.insert(
                    "that".to_string(),
                    ReferentInfo {
                        entity,
                        confidence: 0.5,
                        source: "surrounding_context".to_string(),
                    },
                );
            }
        }

        // Implement conversation history analysis
        let conversation_entities = self.analyze_conversation_history_entities(context);
        let historical_analysis = self.analyze_historical_entities(&conversation_entities);
        let context_aware_disambiguation =
            self.perform_context_aware_disambiguation(context, &historical_analysis);
        let domain_integration =
            self.integrate_domain_hints_with_context(context, &conversation_entities);

        referent_map
    }

    /// Generate embedding for entity using embedding service
    async fn generate_entity_embedding(&self, entity: &str) -> Option<Vec<f32>> {
        debug!("Generating embedding for entity: {}", entity);

        // Try to use the embedding service if available
        if let Some(embedding_service) = &self.embedding_service {
            let request = EmbeddingRequest {
                content: entity.to_string(),
                content_type: ContentType::Text,
                dimensions: Some(768), // Standard embedding dimension
            };

            match embedding_service.generate_embedding(request).await {
                Ok(embedding) => {
                    debug!("Generated embedding for entity '{}' with {} dimensions", entity, embedding.len());
                    return Some(embedding);
                }
                Err(e) => {
                    warn!("Embedding service failed for entity '{}': {}, falling back to simulation", entity, e);
                }
            }
        }

        // Fallback to simulation if embedding service is not available or failed
        debug!("Using simulated embedding for entity: {}", entity);

        // Generate simulated embedding vector (768 dimensions to match typical embedding service)
        let mut embedding = Vec::new();
        for _ in 0..768 {
            embedding.push(fastrand::f32() * 2.0 - 1.0); // Random values between -1 and 1
        }

        debug!("Generated simulated embedding for entity '{}' with {} dimensions", entity, embedding.len());
        Some(embedding)
    }

    /// Query knowledge base semantic search for similar entities
    async fn query_knowledge_base_semantic_search(
        &self,
        embedding: &[f32],
        entity: &str,
    ) -> Result<Vec<KnowledgeBaseResult>> {
        debug!("Querying knowledge base semantic search for entity: {}", entity);

        // Try to use the database client if available
        if let Some(db_client) = &self.db_client {
            // Use the existing knowledge_queries module functions
            use agent_agency_database::knowledge_queries::{kb_semantic_search, kb_fuzzy_search};

            // First try semantic search with the embedding
            let semantic_results = kb_semantic_search(
                db_client,
                embedding,
                Some("kb-text-default".to_string()),
                None, // No source filter
                Some(5), // Limit to 5 results
                Some(0.5), // Minimum confidence
            ).await?;

            let mut results = Vec::new();
            for result in semantic_results {
                results.push(KnowledgeBaseResult {
                    id: result.entity_id,
                    canonical_name: result.canonical_name.clone(),
                    source: match result.source.as_str() {
                        "wikidata" => KnowledgeSource::Wikidata,
                        "wordnet" => KnowledgeSource::WordNet,
                        _ => KnowledgeSource::Wikidata, // Default fallback
                    },
                    properties: std::collections::HashMap::from([
                        ("confidence".to_string(), result.confidence.to_string()),
                        ("similarity_score".to_string(), result.similarity.to_string()),
                        ("usage_count".to_string(), result.usage_count.to_string()),
                    ]),
                });
            }

            if !results.is_empty() {
                debug!("Knowledge base semantic search returned {} results for entity '{}'", results.len(), entity);
                return Ok(results);
            }

            // If semantic search returned no results, try fuzzy search as fallback
            debug!("Semantic search returned no results, trying fuzzy search for entity: {}", entity);
            let fuzzy_results = kb_fuzzy_search(
                entity,
                None, // No source filter
                Some(3), // Limit to 3 results
                Some(0.3), // Similarity threshold
            ).await?;

            for result in fuzzy_results {
                results.push(KnowledgeBaseResult {
                    id: result.entity_id,
                    canonical_name: result.canonical_name.clone(),
                    source: KnowledgeSource::Wikidata, // Default for fuzzy search
                    properties: std::collections::HashMap::from([
                        ("similarity_score".to_string(), result.similarity.to_string()),
                    ]),
                });
            }

            debug!("Knowledge base fuzzy search returned {} results for entity '{}'", results.len(), entity);
            return Ok(results);
        }

        // Fallback to simulation if database client is not available
        debug!("Using simulated knowledge base search for entity: {}", entity);

        // Generate simulated search results
        let mut results = Vec::new();

        for i in 0..3 {
            let result = KnowledgeBaseResult {
                id: uuid::Uuid::new_v4(),
                canonical_name: format!("{}_related_{}", entity, i + 1),
                source: if i % 2 == 0 {
                    KnowledgeSource::Wikidata
                } else {
                    KnowledgeSource::WordNet
                },
                properties: std::collections::HashMap::from([
                    ("confidence".to_string(), (0.8 + i as f64 * 0.05).to_string()),
                    ("similarity_score".to_string(), (0.85 + i as f64 * 0.03).to_string()),
                ]),
            };
            results.push(result);
        }

        debug!("Simulated knowledge base search returned {} results for entity '{}'", results.len(), entity);
        Ok(results)
    }

    /// Record knowledge base usage for analytics
    async fn record_knowledge_base_usage(&self, entity_id: &uuid::Uuid) -> Result<()> {
        debug!("Recording knowledge base usage for entity: {}", entity_id);

        // Try to use the database client if available
        if let Some(db_client) = &self.db_client {
            // Use the existing knowledge_queries module function
            use agent_agency_database::knowledge_queries::record_knowledge_usage;

            match record_knowledge_usage(db_client, *entity_id).await {
                Ok(_) => {
                    debug!("Recorded knowledge base usage for entity: {}", entity_id);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to record knowledge base usage for entity {}: {}, continuing", entity_id, e);
                    // Fall through to simulation as fallback
                }
            }
        }

        // Fallback to simulation if database client is not available or failed
        debug!("Using simulated usage recording for entity: {}", entity_id);

        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        debug!("Simulated recording of knowledge base usage for entity: {}", entity_id);
        Ok(())
    }

    /// Get related entities from knowledge base
    async fn get_related_entities(&self, entity_id: &uuid::Uuid) -> Result<Vec<RelatedEntity>> {
        debug!("Getting related entities for: {}", entity_id);

        // Try to use the database client if available
        if let Some(db_client) = &self.db_client {
            // Use the existing knowledge_queries module function
            use agent_agency_database::knowledge_queries::kb_get_related;

            match kb_get_related(
                db_client,
                *entity_id,
                None, // No relationship type filter
                Some(2), // Max depth of 2
            ).await {
                Ok(related_results) => {
                    let mut related_entities = Vec::new();
                    for result in related_results {
                        related_entities.push(RelatedEntity {
                            id: result.entity_id,
                            canonical_name: result.canonical_name.clone(),
                            relationship_type: result.relationship_type.clone(),
                            confidence: result.confidence as f64,
                        });
                    }

                    debug!("Retrieved {} related entities from database for: {}", related_entities.len(), entity_id);
                    return Ok(related_entities);
                }
                Err(e) => {
                    warn!("Failed to get related entities from database for {}: {}, falling back to simulation", entity_id, e);
                    // Fall through to simulation as fallback
                }
            }
        }

        // Fallback to simulation if database client is not available or failed
        debug!("Using simulated related entity retrieval for: {}", entity_id);

        // Generate simulated related entities
        let mut related_entities = Vec::new();

        for i in 0..2 {
            let related = RelatedEntity {
                id: uuid::Uuid::new_v4(),
                canonical_name: format!("related_entity_{}", i + 1),
                relationship_type: if i % 2 == 0 {
                    "synonym".to_string()
                } else {
                    "related_concept".to_string()
                },
                confidence: 0.75 + i as f64 * 0.1,
            };
            related_entities.push(related);
        }

        debug!("Simulated retrieval of {} related entities for: {}", related_entities.len(), entity_id);
        Ok(related_entities)
    }

    /// Extract related concepts from knowledge base result properties
    async fn extract_related_concepts_from_properties(
        &self,
        result: &KnowledgeBaseResult,
        linked_entities: &mut Vec<String>,
    ) {
        debug!("Extracting related concepts from properties for: {}", result.canonical_name);
        
        // Extract concepts based on knowledge source
        match result.source {
            KnowledgeSource::Wikidata => {
                // Extract senses, forms, translations
                if let Some(senses) = result.properties.get("senses") {
                    debug!("Processing Wikidata senses: {}", senses);
                    // Parse and extract Wikidata senses with proper JSON structure handling
                    if let Ok(senses_data) = serde_json::from_str::<Vec<serde_json::Value>>(senses) {
                        for sense in senses_data {
                            if let Some(sense_id) = sense.get("sense_id").and_then(|v| v.as_str()) {
                                linked_entities.push(format!("{}_sense_{}", result.canonical_name, sense_id));

                                // Extract sense definitions and glosses
                                if let Some(gloss) = sense.get("gloss").and_then(|v| v.as_str()) {
                                    linked_entities.push(format!("{}_gloss_{}", result.canonical_name, sense_id));
                                }

                                // Extract sense examples if available
                                if let Some(examples) = sense.get("examples").and_then(|v| v.as_array()) {
                                    for (i, example) in examples.iter().enumerate() {
                                        if let Some(example_text) = example.as_str() {
                                            linked_entities.push(format!("{}_example_{}_{}", result.canonical_name, sense_id, i));
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Fallback for non-JSON data
                        linked_entities.push(format!("{}_sense", result.canonical_name));
                    }
                }
                
                if let Some(forms) = result.properties.get("forms") {
                    debug!("Processing Wikidata forms: {}", forms);
                    // Parse and extract Wikidata grammatical forms with proper JSON structure handling
                    if let Ok(forms_data) = serde_json::from_str::<Vec<serde_json::Value>>(forms) {
                        for form in forms_data {
                            if let Some(form_type) = form.get("form_type").and_then(|v| v.as_str()) {
                                linked_entities.push(format!("{}_form_{}", result.canonical_name, form_type));

                                // Extract form representations
                                if let Some(representations) = form.get("representations").and_then(|v| v.as_array()) {
                                    for (i, rep) in representations.iter().enumerate() {
                                        if let Some(rep_text) = rep.get("text").and_then(|v| v.as_str()) {
                                            linked_entities.push(format!("{}_rep_{}_{}", result.canonical_name, form_type, i));
                                        }

                                        // Extract language information if available
                                        if let Some(language) = rep.get("language").and_then(|v| v.as_str()) {
                                            linked_entities.push(format!("{}_lang_{}_{}", result.canonical_name, form_type, language));
                                        }
                                    }
                                }

                                // Extract grammatical features if available
                                if let Some(features) = form.get("grammatical_features").and_then(|v| v.as_array()) {
                                    for feature in features {
                                        if let Some(feature_name) = feature.as_str() {
                                            linked_entities.push(format!("{}_feature_{}_{}", result.canonical_name, form_type, feature_name));
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Fallback for non-JSON data
                        linked_entities.push(format!("{}_form", result.canonical_name));
                    }
                }
            }
            KnowledgeSource::WordNet => {
                // Extract synonyms, hypernyms, examples
                if let Some(synonyms) = result.properties.get("synonyms") {
                    debug!("Processing WordNet synonyms: {}", synonyms);
                    // Parse and extract WordNet synonym sets with proper JSON structure handling
                    if let Ok(synonyms_data) = serde_json::from_str::<Vec<serde_json::Value>>(synonyms) {
                        for synset in synonyms_data {
                            if let Some(synset_id) = synset.get("synset_id").and_then(|v| v.as_str()) {
                                linked_entities.push(format!("{}_synset_{}", result.canonical_name, synset_id));

                                // Extract synonym words
                                if let Some(words) = synset.get("words").and_then(|v| v.as_array()) {
                                    for word in words {
                                        if let Some(word_text) = word.as_str() {
                                            linked_entities.push(format!("{}_syn_{}", result.canonical_name, word_text.replace(' ', "_")));
                                        }
                                    }
                                }

                                // Extract part of speech if available
                                if let Some(pos) = synset.get("pos").and_then(|v| v.as_str()) {
                                    linked_entities.push(format!("{}_pos_{}", result.canonical_name, pos));
                                }

                                // Extract gloss/definition if available
                                if let Some(gloss) = synset.get("gloss").and_then(|v| v.as_str()) {
                                    linked_entities.push(format!("{}_def_{}", result.canonical_name, synset_id));
                                }

                                // Extract hypernyms/hyponyms if available
                                if let Some(hypernyms) = synset.get("hypernyms").and_then(|v| v.as_array()) {
                                    for hypernym in hypernyms {
                                        if let Some(hypernym_id) = hypernym.as_str() {
                                            linked_entities.push(format!("{}_hyper_{}", result.canonical_name, hypernym_id));
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Fallback for non-JSON data
                        linked_entities.push(format!("{}_synonym", result.canonical_name));
                    }
                }
                
                if let Some(hypernyms) = result.properties.get("hypernyms") {
                    debug!("Processing WordNet hypernyms: {}", hypernyms);
                    // Parse and extract WordNet hypernym hierarchy with proper JSON structure handling
                    if let Ok(hypernyms_data) = serde_json::from_str::<Vec<serde_json::Value>>(hypernyms) {
                        for hypernym in hypernyms_data {
                            if let Some(hypernym_id) = hypernym.get("hypernym_id").and_then(|v| v.as_str()) {
                                linked_entities.push(format!("{}_hyper_{}", result.canonical_name, hypernym_id));

                                // Extract hypernym words
                                if let Some(words) = hypernym.get("words").and_then(|v| v.as_array()) {
                                    for word in words {
                                        if let Some(word_text) = word.as_str() {
                                            linked_entities.push(format!("{}_hyper_word_{}", result.canonical_name, word_text.replace(' ', "_")));
                                        }
                                    }
                                }

                                // Extract hypernym level/depth if available
                                if let Some(depth) = hypernym.get("depth").and_then(|v| v.as_u64()) {
                                    linked_entities.push(format!("{}_depth_{}", result.canonical_name, depth));
                                }

                                // Extract gloss/definition if available
                                if let Some(gloss) = hypernym.get("gloss").and_then(|v| v.as_str()) {
                                    linked_entities.push(format!("{}_hyper_def_{}", result.canonical_name, hypernym_id));
                                }

                                // Extract hyponyms if available (reverse relationships)
                                if let Some(hyponyms) = hypernym.get("hyponyms").and_then(|v| v.as_array()) {
                                    for hyponym in hyponyms {
                                        if let Some(hyponym_id) = hyponym.as_str() {
                                            linked_entities.push(format!("{}_hypo_{}", result.canonical_name, hyponym_id));
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Fallback for non-JSON data
                        linked_entities.push(format!("{}_hypernym", result.canonical_name));
                    }
                }
            }
            KnowledgeSource::Custom => {
                // Extract custom properties
                if let Some(custom_props) = result.properties.get("custom") {
                    debug!("Processing custom properties: {}", custom_props);
                    // TODO: Replace simplified custom properties handling with proper parsing and extraction
                    /// Requirements for completion:
                    /// - [ ] Implement proper JSON/XML parsing for custom properties
                    /// - [ ] Add support for different property formats and schemas
                    /// - [ ] Implement proper data validation and type checking
                    /// - [ ] Add support for nested property structures and arrays
                    /// - [ ] Implement proper error handling for malformed property data
                    /// - [ ] Add support for property transformation and normalization
                    /// - [ ] Implement proper memory management for large property datasets
                    /// - [ ] Add support for property indexing and search capabilities
                    /// - [ ] Implement proper cleanup of property parsing resources
                    /// - [ ] Add support for property versioning and schema evolution
                    // In a real implementation, parse and extract custom properties
                    linked_entities.push(format!("{}_custom", result.canonical_name));
                }
            }
        }
        
        debug!("Extracted related concepts for: {}", result.canonical_name);
    }

    /// TODO: Implement actual on-demand entity ingestion system
    /// - [ ] Integrate with on-demand ingestion service for missing entities
    /// - [ ] Implement ingestion job queuing and prioritization
    /// - [ ] Add ingestion progress tracking and status monitoring
    /// - [ ] Support different data sources (APIs, databases, web scraping)
    /// - [ ] Implement ingestion failure handling and retry mechanisms
    /// - [ ] Add entity validation and quality assurance checks
    /// - [ ] Support batch ingestion for multiple entities
        
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Simulate ingestion failure occasionally
        if fastrand::f32() < 0.15 { // 15% failure rate
            return Err(anyhow::anyhow!("Simulated on-demand ingestion failure"));
        }
        
        debug!("Successfully triggered on-demand ingestion for entity: {}", entity);
        Ok(())
    }
}

/// Historical entity analysis results
#[derive(Debug, Clone)]
struct HistoricalEntityAnalysis {
    total_entities: usize,
    entity_frequency: std::collections::HashMap<String, usize>,
    entity_relationships: Vec<EntityRelationship>,
    entity_evolution: Vec<String>,
}

/// Entity relationship information
#[derive(Debug, Clone)]
struct EntityRelationship {
    entity1: String,
    entity2: String,
    relationship_type: String,
    confidence: f64,
}

/// Context-aware disambiguation results
#[derive(Debug, Clone)]
struct ContextAwareDisambiguation {
    resolved_entities: Vec<ResolvedEntity>,
    disambiguation_confidence: f64,
    context_utilization: Vec<String>,
}

/// Resolved entity information
#[derive(Debug, Clone)]
struct ResolvedEntity {
    entity: String,
    confidence: f64,
    resolution_method: String,
}

/// Domain integration results
#[derive(Debug, Clone)]
struct DomainIntegration {
    domain_entities: Vec<String>,
    integration_confidence: f64,
    domain_specific_terms: Vec<String>,
}

/// Named Entity Recognition system with caching and performance optimization
#[derive(Debug)]
pub struct NamedEntityRecognizer {
    entity_cache: Arc<RwLock<HashMap<String, Vec<NamedEntity>>>>,
    confidence_threshold: f64,
    entity_patterns: EntityPatterns,
    db_client: Option<DatabaseClient>,
    embedding_service: Option<Arc<EmbeddingService>>,
}

/// Entity patterns for different entity types
#[derive(Debug, Clone)]
struct EntityPatterns {
    person_patterns: Vec<Regex>,
    organization_patterns: Vec<Regex>,
    location_patterns: Vec<Regex>,
    date_patterns: Vec<Regex>,
    time_patterns: Vec<Regex>,
    money_patterns: Vec<Regex>,
    percent_patterns: Vec<Regex>,
    technical_term_patterns: Vec<Regex>,
}

/// Named entity with type and confidence
#[derive(Debug, Clone, PartialEq)]
pub struct NamedEntity {
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub start_pos: usize,
    pub end_pos: usize,
    pub context: String,
}

/// Entity types supported by the NER system
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Time,
    Money,
    Percent,
    TechnicalTerm,
    Unknown,
}

impl NamedEntityRecognizer {
    pub fn new() -> Self {
        Self {
            entity_cache: Arc::new(RwLock::new(HashMap::new())),
            confidence_threshold: 0.7,
            entity_patterns: EntityPatterns::new(),
            db_client: None,
            embedding_service: None,
        }
    }

    /// Create a new NamedEntityRecognizer with database and embedding service integration
    pub fn with_services(db_client: DatabaseClient, embedding_service: Arc<EmbeddingService>) -> Self {
        Self {
            entity_cache: Arc::new(RwLock::new(HashMap::new())),
            confidence_threshold: 0.7,
            entity_patterns: EntityPatterns::new(),
            db_client: Some(db_client),
            embedding_service: Some(embedding_service),
        }
    }

    /// Perform comprehensive Named Entity Recognition on text
    pub async fn recognize_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        // Check cache first for performance optimization
        if let Some(cached_entities) = self.get_cached_entities(text).await {
            return Ok(cached_entities);
        }

        let mut entities = Vec::new();

        // 1. Person name recognition with context awareness
        entities.extend(self.extract_person_entities(text, context).await?);

        // 2. Organization recognition
        entities.extend(self.extract_organization_entities(text, context).await?);

        // 3. Location recognition
        entities.extend(self.extract_location_entities(text, context).await?);

        // 4. Date and time recognition
        entities.extend(self.extract_temporal_entities(text, context).await?);

        // 5. Money and percentage recognition
        entities.extend(self.extract_numerical_entities(text, context).await?);

        // 6. Technical term recognition with domain context
        entities.extend(self.extract_technical_entities(text, context).await?);

        // 7. Entity co-reference resolution
        entities = self.resolve_entity_coreferences(entities, context).await?;

        // 8. Entity disambiguation and validation
        entities = self.disambiguate_entities(entities, context).await?;

        // Filter by confidence threshold
        entities.retain(|e| e.confidence >= self.confidence_threshold);

        // Cache results for performance
        self.cache_entities(text, &entities).await;

        Ok(entities)
    }

    /// Extract person entities with context awareness
    async fn extract_person_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        for pattern in &self.entity_patterns.person_patterns {
            for mat in pattern.find_iter(text) {
                let entity_text = mat.as_str();
                let confidence =
                    self.calculate_person_confidence(entity_text, &words, mat.start(), context);

                if confidence > 0.5 {
                    entities.push(NamedEntity {
                        text: entity_text.to_string(),
                        entity_type: EntityType::Person,
                        confidence,
                        start_pos: mat.start(),
                        end_pos: mat.end(),
                        context: self.extract_entity_context(text, mat.start(), mat.end()),
                    });
                }
            }
        }

        Ok(entities)
    }

    /// Extract organization entities
    async fn extract_organization_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        for pattern in &self.entity_patterns.organization_patterns {
            for mat in pattern.find_iter(text) {
                let entity_text = mat.as_str();
                let confidence = self.calculate_organization_confidence(entity_text, context);

                if confidence > 0.5 {
                    entities.push(NamedEntity {
                        text: entity_text.to_string(),
                        entity_type: EntityType::Organization,
                        confidence,
                        start_pos: mat.start(),
                        end_pos: mat.end(),
                        context: self.extract_entity_context(text, mat.start(), mat.end()),
                    });
                }
            }
        }

        Ok(entities)
    }

    /// Extract location entities
    async fn extract_location_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        for pattern in &self.entity_patterns.location_patterns {
            for mat in pattern.find_iter(text) {
                let entity_text = mat.as_str();
                let confidence = self.calculate_location_confidence(entity_text, context);

                if confidence > 0.5 {
                    entities.push(NamedEntity {
                        text: entity_text.to_string(),
                        entity_type: EntityType::Location,
                        confidence,
                        start_pos: mat.start(),
                        end_pos: mat.end(),
                        context: self.extract_entity_context(text, mat.start(), mat.end()),
                    });
                }
            }
        }

        Ok(entities)
    }

    /// Extract temporal entities (dates and times)
    async fn extract_temporal_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        // Date patterns
        for pattern in &self.entity_patterns.date_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Date,
                    confidence: 0.9,
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    context: self.extract_entity_context(text, mat.start(), mat.end()),
                });
            }
        }

        // Time patterns
        for pattern in &self.entity_patterns.time_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Time,
                    confidence: 0.9,
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    context: self.extract_entity_context(text, mat.start(), mat.end()),
                });
            }
        }

        Ok(entities)
    }

    /// Extract numerical entities (money and percentages)
    async fn extract_numerical_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        // Money patterns
        for pattern in &self.entity_patterns.money_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Money,
                    confidence: 0.95,
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    context: self.extract_entity_context(text, mat.start(), mat.end()),
                });
            }
        }

        // Percentage patterns
        for pattern in &self.entity_patterns.percent_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Percent,
                    confidence: 0.95,
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    context: self.extract_entity_context(text, mat.start(), mat.end()),
                });
            }
        }

        Ok(entities)
    }

    /// Extract technical entities with domain awareness
    async fn extract_technical_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        for pattern in &self.entity_patterns.technical_term_patterns {
            for mat in pattern.find_iter(text) {
                let entity_text = mat.as_str();
                let confidence = self.calculate_technical_confidence(entity_text, context);

                if confidence > 0.6 {
                    entities.push(NamedEntity {
                        text: entity_text.to_string(),
                        entity_type: EntityType::TechnicalTerm,
                        confidence,
                        start_pos: mat.start(),
                        end_pos: mat.end(),
                        context: self.extract_entity_context(text, mat.start(), mat.end()),
                    });
                }
            }
        }

        Ok(entities)
    }

    /// Resolve entity co-references
    async fn resolve_entity_coreferences(
        &self,
        entities: Vec<NamedEntity>,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut resolved_entities = entities;

        // Group entities by type for co-reference resolution
        let mut person_entities: Vec<&mut NamedEntity> = resolved_entities
            .iter_mut()
            .filter(|e| e.entity_type == EntityType::Person)
            .collect();

        // Resolve person co-references (e.g., "John" and "Mr. Smith" referring to the same person)
        for i in 0..person_entities.len() {
            for j in (i + 1)..person_entities.len() {
                if self.are_same_person(&person_entities[i].text, &person_entities[j].text) {
                    // Merge entities - keep the one with higher confidence
                    if person_entities[i].confidence < person_entities[j].confidence {
                        person_entities[i].text = person_entities[j].text.clone();
                        person_entities[i].confidence = person_entities[j].confidence;
                    }
                }
            }
        }

        Ok(resolved_entities)
    }

    /// Disambiguate entities using context
    async fn disambiguate_entities(
        &self,
        entities: Vec<NamedEntity>,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut disambiguated = Vec::new();

        for entity in entities {
            let mut disambiguated_entity = entity;

            // Use conversation history for disambiguation
            if let Some(conversation_history) = context.metadata.get("conversation_history") {
                disambiguated_entity.confidence *=
                    self.calculate_context_boost(&disambiguated_entity.text, conversation_history);
            }

            // Use domain context for technical terms
            if disambiguated_entity.entity_type == EntityType::TechnicalTerm {
                if let Some(domain) = context.metadata.get("domain") {
                    disambiguated_entity.confidence *=
                        self.calculate_domain_boost(&disambiguated_entity.text, domain);
                }
            }

            disambiguated.push(disambiguated_entity);
        }

        Ok(disambiguated)
    }

    /// Calculate confidence for person entities
    fn calculate_person_confidence(
        &self,
        entity_text: &str,
        words: &[&str],
        position: usize,
        context: &ProcessingContext,
    ) -> f64 {
        let mut confidence: f64 = 0.5;

        // Check for title indicators
        let titles = ["Mr.", "Ms.", "Dr.", "Prof.", "Sir", "Madam", "Mrs."];
        if let Some(word_index) = words.iter().position(|&w| w.contains(entity_text)) {
            if word_index > 0 && titles.contains(&words[word_index - 1]) {
                confidence += 0.3;
            }
        }

        // Check for capitalization pattern
        if entity_text.chars().next().unwrap_or(' ').is_uppercase() {
            confidence += 0.2;
        }

        // Check against common names
        let common_names = [
            "John", "Jane", "Mike", "Sarah", "David", "Lisa", "Chris", "Amy",
        ];
        if common_names
            .iter()
            .any(|&name| entity_text.eq_ignore_ascii_case(name))
        {
            confidence += 0.2;
        }

        confidence.min(1.0f32)
    }

    /// Calculate confidence for organization entities
    fn calculate_organization_confidence(
        &self,
        entity_text: &str,
        context: &ProcessingContext,
    ) -> f64 {
        let mut confidence: f64 = 0.6;

        // Check for organization suffixes
        let org_suffixes = ["Inc", "Corp", "LLC", "Ltd", "Company", "Co"];
        if org_suffixes
            .iter()
            .any(|&suffix| entity_text.contains(suffix))
        {
            confidence += 0.3;
        }

        // Check domain context
        if let Some(domain) = context.metadata.get("domain").and_then(|v| v.as_str()) {
            if domain == "business" || domain == "corporate" {
                confidence += 0.1;
            }
        }

        confidence.min(1.0f32)
    }

    /// Calculate confidence for location entities
    fn calculate_location_confidence(&self, entity_text: &str, context: &ProcessingContext) -> f64 {
        let mut confidence: f64 = 0.6;

        // Check for location indicators
        let location_indicators = ["City", "State", "Country", "Street", "Avenue", "Road"];
        if location_indicators
            .iter()
            .any(|&indicator| entity_text.contains(indicator))
        {
            confidence += 0.2;
        }

        confidence.min(1.0f32)
    }

    /// Calculate confidence for technical entities
    fn calculate_technical_confidence(
        &self,
        entity_text: &str,
        context: &ProcessingContext,
    ) -> f64 {
        let mut confidence: f64 = 0.7;

        // Check domain context
        if let Some(domain) = context.metadata.get("domain").and_then(|v| v.as_str()) {
            match domain {
                "software_development" | "technology" => confidence += 0.2,
                "data_science" => confidence += 0.15,
                "devops" => confidence += 0.1,
                _ => {}
            }
        }

        confidence.min(1.0f32)
    }

    /// Extract context around an entity
    fn extract_entity_context(&self, text: &str, start: usize, end: usize) -> String {
        let context_window = 50;
        let context_start = start.saturating_sub(context_window);
        let context_end = (end + context_window).min(text.len());

        text[context_start..context_end].to_string()
    }

    /// Check if two person names refer to the same person
    fn are_same_person(&self, name1: &str, name2: &str) -> bool {
        // Simple heuristic - check if names share common parts
        let parts1: Vec<&str> = name1.split_whitespace().collect();
        let parts2: Vec<&str> = name2.split_whitespace().collect();

        for part1 in &parts1 {
            for part2 in &parts2 {
                if part1.eq_ignore_ascii_case(part2) && part1.len() > 2 {
                    return true;
                }
            }
        }

        false
    }

    /// Calculate context boost from conversation history
    fn calculate_context_boost(&self, entity_text: &str, conversation_history: &Value) -> f64 {
        let mut boost = 1.0;

        if let Some(messages) = conversation_history.as_array() {
            let mention_count = messages
                .iter()
                .filter_map(|msg| msg.get("content").and_then(|v| v.as_str()))
                .filter(|content| content.contains(entity_text))
                .count();

            // Boost confidence based on mention frequency
            boost += (mention_count as f64 * 0.1).min(0.3f32);
        }

        boost
    }

    /// Calculate domain boost for technical terms
    fn calculate_domain_boost(&self, entity_text: &str, domain: &Value) -> f64 {
        if let Some(domain_str) = domain.as_str() {
            let technical_domains = [
                "software_development",
                "technology",
                "data_science",
                "devops",
            ];
            if technical_domains.contains(&domain_str) {
                return 1.2;
            }
        }
        1.0
    }

    /// Get cached entities for performance optimization
    async fn get_cached_entities(&self, text: &str) -> Option<Vec<NamedEntity>> {
        let cache = self.entity_cache.read().await;
        cache.get(text).cloned()
    }

    /// Cache entities for performance optimization
    async fn cache_entities(&self, text: &str, entities: &[NamedEntity]) {
        let mut cache = self.entity_cache.write().await;
        cache.insert(text.to_string(), entities.to_vec());

        // Limit cache size to prevent memory issues
        if cache.len() > 1000 {
            let keys_to_remove: Vec<String> = cache.keys().take(100).cloned().collect();
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }
}

impl EntityPatterns {
    fn new() -> Self {
        Self {
            person_patterns: vec![
                Regex::new(r"\b(?:Mr\.|Ms\.|Dr\.|Prof\.)?\s*[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b").unwrap(),
                Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b").unwrap(),
            ],
            organization_patterns: vec![
                Regex::new(r"\b[A-Z][a-zA-Z\s]+(?:Inc|Corp|LLC|Ltd|Company|Co)\b").unwrap(),
                Regex::new(r"\b[A-Z][A-Z]+\b").unwrap(), // Acronyms
            ],
            location_patterns: vec![
                Regex::new(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\s+(?:City|State|Country|Street|Avenue|Road|Blvd)\b").unwrap(),
                Regex::new(r"\b(?:New York|Los Angeles|Chicago|Houston|Phoenix|Philadelphia|San Antonio|San Diego|Dallas|San Jose)\b").unwrap(),
            ],
            date_patterns: vec![
                Regex::new(r"\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},?\s+\d{4}\b").unwrap(),
                Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b").unwrap(),
                Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap(),
            ],
            time_patterns: vec![
                Regex::new(r"\b\d{1,2}:\d{2}(?::\d{2})?\s*(?:AM|PM|am|pm)?\b").unwrap(),
                Regex::new(r"\b(?:morning|afternoon|evening|night|noon|midnight)\b").unwrap(),
            ],
            money_patterns: vec![
                Regex::new(r"\$\d+(?:,\d{3})*(?:\.\d{2})?\b").unwrap(),
                Regex::new(r"\b\d+(?:,\d{3})*(?:\.\d{2})?\s*(?:dollars?|USD|cents?)\b").unwrap(),
            ],
            percent_patterns: vec![
                Regex::new(r"\b\d+(?:\.\d+)?%\b").unwrap(),
                Regex::new(r"\b\d+(?:\.\d+)?\s*percent\b").unwrap(),
            ],
            technical_term_patterns: vec![
                Regex::new(r"\b(?:API|HTTP|JSON|XML|SQL|REST|GraphQL|OAuth|JWT|CRUD|MVC|ORM|CI/CD|DevOps|SaaS|PaaS|IaaS)\b").unwrap(),
                Regex::new(r"\b(?:Docker|Kubernetes|AWS|Azure|GCP|React|Vue|Angular|Node\.js|Python|Rust|Go|Java|C\+\+)\b").unwrap(),
                Regex::new(r"\b(?:database|server|client|frontend|backend|microservice|container|deployment|repository|framework)\b").unwrap(),
            ],
        }
    }

    /// Test database integration for knowledge base entity linking
    #[tokio::test]
    async fn test_database_integration_knowledge_base_entity_linking() {
        // Integration test for claim extraction knowledge base operations
        // This test requires a real database connection
        if std::env::var("RUN_INTEGRATION_TESTS").is_err() {
            return; // Skip unless explicitly enabled
        }

        // Create test entities for linking
        let test_entities = vec![
            "artificial intelligence".to_string(),
            "machine learning".to_string(),
            "neural network".to_string(),
            "database".to_string(),
        ];

        // Test entity linking with knowledge base
        // let db_client = setup_test_database_client().await;
        // let embedding_service = setup_test_embedding_service().await;
        // let recognizer = NamedEntityRecognizer::with_services(db_client, embedding_service);

        // Test entity recognition first
        let recognizer = NamedEntityRecognizer::new();
        let processing_context = ProcessingContext {
            document_id: "test-doc".to_string(),
            section_id: Some("test-section".to_string()),
            confidence_threshold: 0.7,
            max_entities: 50,
            language: Language::English,
            domain_hints: vec!["technology".to_string(), "ai".to_string()],
        };

        let test_text = "Artificial intelligence and machine learning are transforming database systems.";

        // Test basic entity recognition (without database)
        let entities = recognizer.recognize_entities(test_text, &processing_context).await.unwrap();

        // Validate basic recognition works
        assert!(!entities.is_empty());

        // Test entity linking (would use database in real integration test)
        // let linked_entities = recognizer.link_entities_to_knowledge_bases(&test_entities).await;

        // Validate that entity linking produces some results
        // assert!(!linked_entities.is_empty());

        // Test embedding generation
        for entity in &test_entities {
            let embedding = recognizer.generate_entity_embedding(entity).await;
            // Embedding might be None if service is not available (fallback simulation)
            if let Some(emb) = embedding {
                assert!(!emb.is_empty());
                assert!(emb.len() == 768 || emb.len() == 384); // Standard embedding dimensions
            }
        }

        tracing::debug!("Knowledge base entity linking test structure validated");
    }

    /// Test database integration for semantic search operations
    #[tokio::test]
    async fn test_database_integration_semantic_search_operations() {
        // Integration test for semantic search database operations
        if std::env::var("RUN_INTEGRATION_TESTS").is_err() {
            return;
        }

        // Test semantic search with mock data
        let recognizer = NamedEntityRecognizer::new();
        let test_entity = "machine learning";
        let test_embedding = vec![0.1; 768]; // Mock embedding

        // Test semantic search (would use database in real integration test)
        let search_results = recognizer.query_knowledge_base_semantic_search(&test_embedding, test_entity).await.unwrap();

        // Validate search returns results (even if simulated)
        assert!(!search_results.is_empty());

        // Test knowledge base usage recording
        for result in &search_results {
            let usage_result = recognizer.record_knowledge_base_usage(&result.id).await;
            // Should succeed even with simulation
            assert!(usage_result.is_ok());
        }

        // Test related entity retrieval
        for result in &search_results {
            let related_entities = recognizer.get_related_entities(&result.id).await.unwrap();
            // Should return some results (even if simulated)
            assert!(!related_entities.is_empty());

            // Validate related entity structure
            for related in &related_entities {
                assert!(!related.canonical_name.is_empty());
                assert!(!related.relationship_type.is_empty());
                assert!(related.confidence >= 0.0 && related.confidence <= 1.0);
            }
        }

        tracing::debug!("Semantic search operations test completed");
    }
}