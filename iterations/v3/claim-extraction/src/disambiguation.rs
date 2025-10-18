//! Stage 1: Contextual Disambiguation
//!
//! Identifies and resolves ambiguities in sentences to prepare for
//! claim extraction. Based on V2 disambiguation logic with Rust adaptations.

use crate::types::*;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use tracing::debug;

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
}

impl ContextResolver {
    fn new() -> Self {
        let mut domain_context = HashMap::new();
        domain_context.insert("system".to_string(), "the Agent Agency system".to_string());
        domain_context.insert("component".to_string(), "the current component".to_string());
        domain_context.insert("function".to_string(), "the specified function".to_string());

        Self { domain_context }
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
            entities.extend(self.analyze_conversation_history(conversation_history));
        }
        
        // 2. Named entity recognition
        entities.extend(self.perform_named_entity_recognition(&context.input_text));
        
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
    fn analyze_conversation_history(&self, conversation_history: &serde_json::Value) -> Vec<String> {
        let mut entities = Vec::new();
        
        if let Some(messages) = conversation_history.as_array() {
            for message in messages {
                if let Some(text) = message.get("content").and_then(|v| v.as_str()) {
                    entities.extend(self.perform_named_entity_recognition(text));
                }
            }
        }
        
        entities
    }
    
    /// Perform named entity recognition on text
    fn perform_named_entity_recognition(&self, text: &str) -> Vec<String> {
        let mut entities = Vec::new();
        
        // Simple NER using patterns and heuristics
        // In a real implementation, you would use a proper NLP library
        
        // Person names (capitalized words that could be names)
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if word.len() > 2 && word.chars().next().unwrap().is_uppercase() {
                // Check if it's likely a person name
                if self.is_likely_person_name(word, &words, i) {
                    entities.push(word.to_string());
                }
            }
        }
        
        // Organization names (patterns like "Company Inc", "Corp", etc.)
        for (i, word) in words.iter().enumerate() {
            if word.eq_ignore_ascii_case("inc") || word.eq_ignore_ascii_case("corp") || 
               word.eq_ignore_ascii_case("llc") || word.eq_ignore_ascii_case("ltd") {
                if i > 0 {
                    entities.push(words[i-1].to_string());
                }
            }
        }
        
        // Technical terms and concepts
        let technical_terms = [
            "API", "HTTP", "JSON", "XML", "SQL", "REST", "GraphQL",
            "Docker", "Kubernetes", "AWS", "Azure", "GCP",
            "React", "Vue", "Angular", "Node.js", "Python", "Rust", "Go"
        ];
        
        for term in &technical_terms {
            if text.to_uppercase().contains(term) {
                entities.push(term.to_string());
            }
        }
        
        entities
    }
    
    /// Check if a word is likely a person name
    fn is_likely_person_name(&self, word: &str, words: &[&str], index: usize) -> bool {
        // Simple heuristics for person name detection
        if word.len() < 2 || word.len() > 20 {
            return false;
        }
        
        // Check for common name patterns
        let name_indicators = ["Mr.", "Ms.", "Dr.", "Prof.", "Sir", "Madam"];
        if index > 0 {
            let prev_word = words[index - 1];
            if name_indicators.iter().any(|indicator| prev_word.eq_ignore_ascii_case(indicator)) {
                return true;
            }
        }
        
        // Check if it's followed by a last name
        if index + 1 < words.len() {
            let next_word = words[index + 1];
            if next_word.len() > 2 && next_word.chars().next().unwrap().is_uppercase() {
                return true;
            }
        }
        
        // Check for common first names
        let common_first_names = [
            "John", "Jane", "Mike", "Sarah", "David", "Lisa", "Chris", "Amy",
            "Alex", "Sam", "Tom", "Kate", "Ben", "Emma", "Ryan", "Anna"
        ];
        
        common_first_names.iter().any(|name| word.eq_ignore_ascii_case(name))
    }
    
    /// Link entities to knowledge bases
    fn link_entities_to_knowledge_bases(&self, entities: &[String]) -> Vec<String> {
        let mut linked_entities = Vec::new();
        
        for entity in entities {
            // In a real implementation, you would query knowledge bases like:
            // - Wikipedia API
            // - Wikidata
            // - Domain-specific knowledge bases
            
            // For now, add related entities based on simple rules
            match entity.to_lowercase().as_str() {
                "api" => linked_entities.extend(vec![
                    "REST API".to_string(),
                    "GraphQL".to_string(),
                    "HTTP".to_string()
                ]),
                "database" => linked_entities.extend(vec![
                    "SQL".to_string(),
                    "PostgreSQL".to_string(),
                    "MySQL".to_string()
                ]),
                "javascript" => linked_entities.extend(vec![
                    "Node.js".to_string(),
                    "TypeScript".to_string(),
                    "React".to_string()
                ]),
                _ => {}
            }
        }
        
        linked_entities
    }
    
    /// Integrate entity context with conversation context
    fn integrate_entity_context(&self, entities: &[String], context: &ProcessingContext) -> Vec<String> {
        let mut contextual_entities = Vec::new();
        
        // Add entities based on conversation context
        if let Some(domain) = context.metadata.get("domain").and_then(|v| v.as_str()) {
            match domain {
                "software_development" => contextual_entities.extend(vec![
                    "code".to_string(),
                    "programming".to_string(),
                    "development".to_string()
                ]),
                "data_science" => contextual_entities.extend(vec![
                    "machine learning".to_string(),
                    "analytics".to_string(),
                    "statistics".to_string()
                ]),
                "devops" => contextual_entities.extend(vec![
                    "deployment".to_string(),
                    "infrastructure".to_string(),
                    "monitoring".to_string()
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
            entities.extend(self.analyze_conversation_history(conversation_history));
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
        
        // Find entity relationships (simplified)
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
                    confidence: (*frequency as f64 / historical_analysis.total_entities as f64).min(1.0),
                    resolution_method: "frequency_analysis".to_string(),
                });
                total_confidence += (*frequency as f64 / historical_analysis.total_entities as f64).min(1.0);
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
                },
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
                },
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
                },
                _ => {}
            }
            
            // Calculate integration confidence based on overlap
            let conversation_set: std::collections::HashSet<_> = conversation_entities.iter().collect();
            let domain_set: std::collections::HashSet<_> = integration.domain_entities.iter().collect();
            let intersection: std::collections::HashSet<_> = conversation_set.intersection(&domain_set).collect();
            
            if !conversation_set.is_empty() {
                integration.integration_confidence = intersection.len() as f64 / conversation_set.len() as f64;
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
            if entity1_lower.starts_with(&entity2_lower[..3]) || 
               entity2_lower.starts_with(&entity1_lower[..3]) {
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
            if (entity1_lower.contains(term1) && entity2_lower.contains(term2)) ||
               (entity1_lower.contains(term2) && entity2_lower.contains(term1)) {
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
        confidence.max(0.0).min(1.0)
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
        let context_aware_disambiguation = self.perform_context_aware_disambiguation(context, &historical_analysis);
        let domain_integration = self.integrate_domain_hints_with_context(context, &conversation_entities);

        referent_map
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
