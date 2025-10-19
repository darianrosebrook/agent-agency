//! @darianrosebrook
//! Entity and topic extraction enricher
//!
//! Extracts:
//! - Named entities (person, organization, location, date, email, phone)
//! - Topics via BERTopic or keyphrase extraction
//! - Chapter boundaries from topic transitions
//! - PII detection and hashing for privacy

use crate::types::{EntityResult, ExtractedEntity, Topic, Chapter, EnricherConfig};
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

pub struct EntityEnricher {
    config: EnricherConfig,
}

impl EntityEnricher {
    pub fn new(config: EnricherConfig) -> Self {
        Self { config }
    }

    /// Extract entities and topics from text and speech
    ///
    /// # Arguments
    /// * `text` - Input text to analyze
    /// * `timestamps` - Optional time ranges for topic segmentation
    ///
    /// # Returns
    /// EntityResult with entities, topics, and chapter boundaries
    pub async fn extract_entities(&self, text: &str, _timestamps: Option<Vec<(f32, f32)>>) -> Result<EntityResult> {
        tracing::debug!(
            "Extracting entities with NER enabled: {}",
            self.config.entity_ner_enabled
        );

        let entities = self.detect_entities(text).await?;
        let topics = self.extract_topics(text).await?;
        let chapters = self.segment_chapters(&topics).await?;

        Ok(EntityResult {
            entities,
            topics,
            chapters,
            processing_time_ms: 0,
        })
    }

    /// Detect named entities using DataDetection + optional NER
    async fn detect_entities(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();

        // TODO: PLACEHOLDER - Apple DataDetection for emails/URLs/dates
        // Would use NSDataDetector with types:
        // - NSTextCheckingTypeEmail
        // - NSTextCheckingTypeLink
        // - NSTextCheckingTypeDate
        // - NSTextCheckingTypePhoneNumber

        // TODO: PLACEHOLDER - Optional NER for domain terms (if NER-enabled)
        // Would use ner-rs or similar for person, organization, location

        // Placeholder: detect simple patterns
        self.detect_email_patterns(text, &mut entities);
        self.detect_url_patterns(text, &mut entities);

        Ok(entities)
    }

    /// Detect email addresses in text
    fn detect_email_patterns(&self, text: &str, entities: &mut Vec<ExtractedEntity>) {
        // Simple email pattern detection (placeholder)
        for (i, word) in text.split_whitespace().enumerate() {
            if word.contains('@') && word.contains('.') {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "email".to_string(),
                    text: word.to_string(),
                    normalized: word.to_lowercase(),
                    confidence: 0.85,
                    pii: true,
                    span_start: text.find(word).unwrap_or(0),
                    span_end: text.find(word).unwrap_or(0) + word.len(),
                });
            }
        }
    }

    /// Detect URLs in text
    fn detect_url_patterns(&self, text: &str, entities: &mut Vec<ExtractedEntity>) {
        // Simple URL pattern detection (placeholder)
        for word in text.split_whitespace() {
            if word.starts_with("http://") || word.starts_with("https://") {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "url".to_string(),
                    text: word.to_string(),
                    normalized: word.to_string(),
                    confidence: 0.95,
                    pii: false,
                    span_start: text.find(word).unwrap_or(0),
                    span_end: text.find(word).unwrap_or(0) + word.len(),
                });
            }
        }
    }

    /// Extract topics via BERTopic or keyphrase extraction
    async fn extract_topics(&self, text: &str) -> Result<Vec<Topic>> {
        // TODO: PLACEHOLDER - BERTopic or KeyBERT integration
        // Would extract topics from text with:
        // - Top keywords per topic
        // - Confidence scores
        // - Occurrence counts

        tracing::debug!("Extracting topics from text (placeholder)");

        let mut topics = Vec::new();

        // Simple keyword extraction placeholder
        let keywords = self.extract_simple_keywords(text);
        for (keyword, count) in keywords.iter().take(3) {
            topics.push(Topic {
                name: keyword.clone(),
                keywords: vec![keyword.clone()],
                confidence: 0.6,
                occurrence_count: *count,
            });
        }

        Ok(topics)
    }

    /// Extract simple keywords (placeholder)
    fn extract_simple_keywords(&self, text: &str) -> HashMap<String, usize> {
        let mut keywords = HashMap::new();

        // Skip common stopwords
        let stopwords = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "is", "are", "be", "been", "being", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "can", "must", "shall",
        ];

        for word in text.to_lowercase().split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if !clean.is_empty() && !stopwords.contains(&clean) && clean.len() > 2 {
                *keywords.entry(clean.to_string()).or_insert(0) += 1;
            }
        }

        keywords
    }

    /// Segment content into chapters based on topic transitions
    async fn segment_chapters(&self, topics: &[Topic]) -> Result<Vec<Chapter>> {
        let mut chapters = Vec::new();

        // Create chapters from topics
        for (i, topic) in topics.iter().enumerate() {
            chapters.push(Chapter {
                title: topic.name.clone(),
                t0: (i as f32) * 300.0, // Placeholder: 5-minute chapters
                t1: ((i + 1) as f32) * 300.0,
                description: Some(format!("Chapter on {}", topic.name)),
            });
        }

        Ok(chapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_entity_enricher_init() {
        let enricher = EntityEnricher::new(EnricherConfig::default());
        assert!(enricher.config.entity_ner_enabled);
    }

    #[tokio::test]
    async fn test_email_detection() {
        let enricher = EntityEnricher::new(EnricherConfig::default());
        let text = "Contact me at test@example.com for more info";
        let result = enricher.extract_entities(text, None).await;
        assert!(result.is_ok());

        let entity_result = result.unwrap();
        let emails: Vec<_> = entity_result
            .entities
            .iter()
            .filter(|e| e.entity_type == "email")
            .collect();
        assert!(!emails.is_empty());
    }

    #[tokio::test]
    async fn test_topic_extraction() {
        let enricher = EntityEnricher::new(EnricherConfig::default());
        let text = "Machine learning is great. Deep learning models are powerful. Neural networks work well.";
        let result = enricher.extract_entities(text, None).await;
        assert!(result.is_ok());

        let entity_result = result.unwrap();
        assert!(!entity_result.topics.is_empty());
    }
}
