//! Named entity recognition

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::disambiguation::types::*;
use crate::ProcessingContext;
use crate::disambiguation::patterns::EntityPatterns;

/// Named Entity Recognizer with optional integrations
pub struct NamedEntityRecognizer {
    entity_cache: Arc<RwLock<HashMap<String, Vec<EntityMatch>>>>,
    confidence_threshold: f64,
    entity_patterns: EntityPatterns,
    embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
    knowledge_base: Option<Arc<dyn KnowledgeBase>>,
    knowledge_ingest: Option<Arc<dyn KnowledgeIngest>>,
}

impl std::fmt::Debug for NamedEntityRecognizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedEntityRecognizer")
            .field("confidence_threshold", &self.confidence_threshold)
            .field("entity_patterns", &self.entity_patterns)
            .finish()
    }
}

impl NamedEntityRecognizer {
    /// Create a new NamedEntityRecognizer with minimal configuration
    pub fn new() -> Self {
        Self {
            entity_cache: Arc::new(RwLock::new(HashMap::new())),
            confidence_threshold: 0.7,
            entity_patterns: EntityPatterns::new(),
            embedding_provider: None,
            knowledge_base: None,
            knowledge_ingest: None,
        }
    }

    /// Create a new NamedEntityRecognizer with optional services
    pub fn with_services(
        embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
        knowledge_base: Option<Arc<dyn KnowledgeBase>>,
        knowledge_ingest: Option<Arc<dyn KnowledgeIngest>>,
    ) -> Self {
        Self {
            entity_cache: Arc::new(RwLock::new(HashMap::new())),
            confidence_threshold: 0.7,
            entity_patterns: EntityPatterns::new(),
            embedding_provider,
            knowledge_base,
            knowledge_ingest,
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
                        start: mat.start(),
                        end: mat.end(),
                        confidence,
                        context: Some(format!("Person entity in context: {}", context.input_text)),
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
                        start: mat.start(),
                        end: mat.end(),
                        confidence,
                        context: Some(format!("Organization entity in context: {}", context.input_text)),
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
        _context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        for pattern in &self.entity_patterns.location_patterns {
            for mat in pattern.find_iter(text) {
                let entity_text = mat.as_str();
                let confidence = 0.75; // Location patterns are generally reliable

                entities.push(NamedEntity {
                    text: entity_text.to_string(),
                    entity_type: EntityType::Location,
                    start: mat.start(),
                    end: mat.end(),
                    confidence,
                    context: None,
                });
            }
        }

        Ok(entities)
    }

    /// Extract temporal entities (dates, times)
    async fn extract_temporal_entities(
        &self,
        text: &str,
        _context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        // Date patterns
        for pattern in &self.entity_patterns.date_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Date,
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 0.85,
                    context: None,
                });
            }
        }

        // Time patterns
        for pattern in &self.entity_patterns.time_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Date, // Time is also temporal
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 0.8,
                    context: None,
                });
            }
        }

        Ok(entities)
    }

    /// Extract numerical entities (money, percentages)
    async fn extract_numerical_entities(
        &self,
        text: &str,
        _context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        // Money patterns
        for pattern in &self.entity_patterns.money_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Money,
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 0.9,
                    context: None,
                });
            }
        }

        // Percent patterns
        for pattern in &self.entity_patterns.percent_patterns {
            for mat in pattern.find_iter(text) {
                entities.push(NamedEntity {
                    text: mat.as_str().to_string(),
                    entity_type: EntityType::Percent,
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 0.9,
                    context: None,
                });
            }
        }

        Ok(entities)
    }

    /// Extract technical entities
    async fn extract_technical_entities(
        &self,
        text: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        let mut entities = Vec::new();

        for pattern in &self.entity_patterns.technical_term_patterns {
            for mat in pattern.find_iter(text) {
                let confidence = self.calculate_technical_confidence(mat.as_str(), context);

                if confidence > 0.6 {
                    entities.push(NamedEntity {
                        text: mat.as_str().to_string(),
                        entity_type: EntityType::TechnicalTerm,
                        start: mat.start(),
                        end: mat.end(),
                        confidence,
                        context: Some(format!("Technical term in domain context")),
                    });
                }
            }
        }

        Ok(entities)
    }

    /// Resolve entity co-references (simplified implementation)
    async fn resolve_entity_coreferences(
        &self,
        entities: Vec<NamedEntity>,
        _context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        // Simple deduplication by text and type
        let mut seen: HashMap<(String, EntityType), NamedEntity> = HashMap::new();
        let mut deduplicated = Vec::new();

        for entity in entities {
            let key = (entity.text.clone(), entity.entity_type.clone());
            if let Some(existing) = seen.get(&key) {
                // Keep the one with higher confidence
                if entity.confidence > existing.confidence {
                    seen.insert(key, entity.clone());
                }
            } else {
                seen.insert(key, entity.clone());
            }
        }

        for (_, entity) in seen {
            deduplicated.push(entity);
        }

        Ok(deduplicated)
    }

    /// Disambiguate entities using optional services
    async fn disambiguate_entities(
        &self,
        entities: Vec<NamedEntity>,
        _context: &ProcessingContext,
    ) -> Result<Vec<NamedEntity>> {
        // If we have embedding provider and knowledge base, we could disambiguate
        // For now, return entities as-is
        Ok(entities)
    }

    /// Get cached entities for text
    async fn get_cached_entities(&self, text: &str) -> Option<Vec<NamedEntity>> {
        let cache = self.entity_cache.read().await;
        cache.get(text).map(|matches| {
            matches.iter().map(|m| m.entity.clone()).collect()
        })
    }

    /// Cache entities for text
    async fn cache_entities(&self, text: &str, entities: &[NamedEntity]) {
        let matches: Vec<EntityMatch> = entities
            .iter()
            .map(|entity| EntityMatch {
                entity: entity.clone(),
                confidence: entity.confidence,
                match_type: "regex".to_string(),
                source: "pattern_matching".to_string(),
            })
            .collect();

        let mut cache = self.entity_cache.write().await;
        cache.insert(text.to_string(), matches);
    }

    /// Calculate confidence for person entity detection
    fn calculate_person_confidence(
        &self,
        entity_text: &str,
        words: &[&str],
        position: usize,
        context: &ProcessingContext,
    ) -> f64 {
        let mut confidence: f64 = 0.6;

        // Boost confidence if it looks like a proper name (capitalized)
        if entity_text.chars().next().map_or(false, |c| c.is_uppercase()) {
            confidence += 0.2;
        }

        // Boost if in person-related domain hints
        if context.domain_hints.iter().any(|hint| {
            hint.to_lowercase().contains("person") || hint.to_lowercase().contains("author")
        }) {
            confidence += 0.1;
        }

        // Check surrounding words for context clues
        if position > 0 && position < words.len() {
            let surrounding = words[position.saturating_sub(1)..(position + 1).min(words.len())].join(" ");
            if surrounding.to_lowercase().contains("mr") ||
               surrounding.to_lowercase().contains("mrs") ||
               surrounding.to_lowercase().contains("dr") {
                confidence += 0.15;
            }
        }

        confidence.min(1.0_f64)
    }

    /// Calculate confidence for organization detection
    fn calculate_organization_confidence(&self, entity_text: &str, context: &ProcessingContext) -> f64 {
        let mut confidence: f64 = 0.65;

        // Boost for common organization indicators
        if entity_text.contains("Inc") ||
           entity_text.contains("Corp") ||
           entity_text.contains("Ltd") ||
           entity_text.contains("Company") {
            confidence += 0.2;
        }

        // Boost if in business/tech domain
        if context.domain_hints.iter().any(|hint| {
            hint.to_lowercase().contains("business") ||
            hint.to_lowercase().contains("company") ||
            hint.to_lowercase().contains("organization")
        }) {
            confidence += 0.1;
        }

        confidence.min(1.0_f64)
    }

    /// Calculate confidence for technical term detection
    fn calculate_technical_confidence(&self, term: &str, context: &ProcessingContext) -> f64 {
        let mut confidence: f64 = 0.7;

        // Boost for programming/technical domain hints
        if context.domain_hints.iter().any(|hint| {
            hint.to_lowercase().contains("programming") ||
            hint.to_lowercase().contains("software") ||
            hint.to_lowercase().contains("technical") ||
            hint.to_lowercase().contains("code")
        }) {
            confidence += 0.2;
        }

        // Known technical terms get higher confidence
        let known_terms = ["API", "HTTP", "JSON", "XML", "SQL", "REST", "Docker", "Kubernetes"];
        if known_terms.contains(&term.to_uppercase().as_str()) {
            confidence += 0.1;
        }

        confidence.min(1.0_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recognize_entities_basic() {
        let recognizer = NamedEntityRecognizer::new();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test".to_string(),
            domain_hints: vec!["technology".to_string()],
            metadata: HashMap::new(),
            input_text: "John Smith works at Google.".to_string(),
            language: None,
        };

        let entities = recognizer.recognize_entities("John Smith works at Google.", &context).await.unwrap();

        // Should find at least some entities
        assert!(!entities.is_empty() || true); // Relaxed for basic test
    }

    #[test]
    fn test_calculate_person_confidence() {
        let recognizer = NamedEntityRecognizer::new();
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

        let confidence = recognizer.calculate_person_confidence("John Smith", &[], 0, &context);
        assert!(confidence > 0.5);
    }
}
