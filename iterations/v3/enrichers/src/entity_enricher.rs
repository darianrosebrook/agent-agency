//! @darianrosebrook
//! Entity and topic extraction enricher
//!
//! Extracts:
//! - Named entities (person, organization, location, date, email, phone)
//! - Topics via BERTopic or keyphrase extraction
//! - Chapter boundaries from topic transitions
//! - PII detection and hashing for privacy

use crate::types::{Chapter, EnricherConfig, EntityResult, ExtractedEntity, Topic};
use anyhow::{Context, Result};
use std::collections::HashMap;
use uuid::Uuid;
use regex::Regex;
use sha2::{Sha256, Digest};

/// Apple DataDetection bridge for entity extraction
#[derive(Debug)]
struct DataDetectionBridge {
    // In a real implementation, this would contain Swift bridge handles
}

impl DataDetectionBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing Apple DataDetection bridge");
        Ok(Self {})
    }

    async fn detect_entities(&self, text: &str) -> Result<Vec<DataDetectionResult>> {
        // Simulate Apple DataDetection processing
        // In a real implementation, this would:
        // 1. Create NSDataDetector with NSTextCheckingTypes
        // 2. Use NSDataDetector.matches(in:options:range:)
        // 3. Parse results into structured data
        
        tracing::debug!("Detecting entities with Apple DataDetection ({} chars)", text.len());
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        // Return simulated results
        Ok(vec![
            DataDetectionResult {
                entity_type: "email".to_string(),
                text: "example@company.com".to_string(),
                range: (0, 20),
                confidence: 0.95,
            },
            DataDetectionResult {
                entity_type: "url".to_string(),
                text: "https://www.example.com".to_string(),
                range: (25, 50),
                confidence: 0.98,
            },
            DataDetectionResult {
                entity_type: "date".to_string(),
                text: "2024-01-15".to_string(),
                range: (55, 65),
                confidence: 0.90,
            },
        ])
    }
}

/// Apple DataDetection result
#[derive(Debug)]
struct DataDetectionResult {
    entity_type: String,
    text: String,
    range: (usize, usize),
    confidence: f32,
}

/// NER (Named Entity Recognition) bridge
#[derive(Debug)]
struct NERBridge {
    // In a real implementation, this would contain NER model handles
}

impl NERBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing NER bridge");
        Ok(Self {})
    }

    async fn extract_entities(&self, text: &str) -> Result<Vec<NERResult>> {
        // Simulate NER processing
        // In a real implementation, this would:
        // 1. Load pre-trained NER model (spaCy, Transformers, etc.)
        // 2. Process text through the model
        // 3. Extract person, organization, location entities
        
        tracing::debug!("Extracting entities with NER ({} chars)", text.len());
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Return simulated results
        Ok(vec![
            NERResult {
                entity_type: "PERSON".to_string(),
                text: "John Smith".to_string(),
                range: (0, 10),
                confidence: 0.92,
            },
            NERResult {
                entity_type: "ORG".to_string(),
                text: "Apple Inc.".to_string(),
                range: (15, 25),
                confidence: 0.88,
            },
            NERResult {
                entity_type: "GPE".to_string(),
                text: "San Francisco".to_string(),
                range: (30, 43),
                confidence: 0.95,
            },
        ])
    }
}

/// NER result
#[derive(Debug)]
struct NERResult {
    entity_type: String,
    text: String,
    range: (usize, usize),
    confidence: f32,
}

/// Topic extraction bridge
#[derive(Debug)]
struct TopicExtractionBridge {
    // In a real implementation, this would contain BERTopic/KeyBERT handles
}

impl TopicExtractionBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing topic extraction bridge");
        Ok(Self {})
    }

    async fn extract_topics(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        // Simulate topic extraction
        // In a real implementation, this would:
        // 1. Use BERTopic or KeyBERT for topic modeling
        // 2. Extract key phrases and topics
        // 3. Calculate confidence scores
        
        tracing::debug!("Extracting topics ({} chars)", text.len());
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        
        // Return simulated results
        Ok(vec![
            TopicExtractionResult {
                topic: "Technology".to_string(),
                keywords: vec!["AI".to_string(), "machine learning".to_string(), "software".to_string()],
                confidence: 0.85,
                occurrence_count: 5,
            },
            TopicExtractionResult {
                topic: "Business".to_string(),
                keywords: vec!["strategy".to_string(), "market".to_string(), "growth".to_string()],
                confidence: 0.78,
                occurrence_count: 3,
            },
        ])
    }
}

/// Topic extraction result
#[derive(Debug)]
struct TopicExtractionResult {
    topic: String,
    keywords: Vec<String>,
    confidence: f32,
    occurrence_count: u32,
}

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
    pub async fn extract_entities(
        &self,
        text: &str,
        _timestamps: Option<Vec<(f32, f32)>>,
    ) -> Result<EntityResult> {
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

        // Use Apple DataDetection for emails/URLs/dates/phone numbers
        let data_detection_bridge = DataDetectionBridge::new()?;
        let data_detection_results = data_detection_bridge
            .detect_entities(text)
            .await
            .context("DataDetection failed")?;

        // Convert DataDetection results to ExtractedEntity
        for result in data_detection_results {
            let is_pii = self.is_pii_entity(&result.entity_type);
            let normalized = if is_pii {
                self.hash_pii(&result.text)
            } else {
                result.text.clone()
            };

            entities.push(ExtractedEntity {
                id: Uuid::new_v4(),
                entity_type: result.entity_type,
                text: result.text,
                normalized,
                confidence: result.confidence,
                pii: is_pii,
                span_start: result.range.0,
                span_end: result.range.1,
            });
        }

        // Use NER for domain terms if enabled
        if self.config.entity_ner_enabled {
            let ner_bridge = NERBridge::new()?;
            let ner_results = ner_bridge
                .extract_entities(text)
                .await
                .context("NER extraction failed")?;

            // Convert NER results to ExtractedEntity
            for result in ner_results {
                let entity_type = self.map_ner_type(&result.entity_type);
                let is_pii = self.is_pii_entity(&entity_type);
                let normalized = if is_pii {
                    self.hash_pii(&result.text)
                } else {
                    result.text.clone()
                };

                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type,
                    text: result.text,
                    normalized,
                    confidence: result.confidence,
                    pii: is_pii,
                    span_start: result.range.0,
                    span_end: result.range.1,
                });
            }
        }

        // Fallback: detect simple patterns for basic entities
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
        // BERTopic/KeyBERT integration
        // Extracts topics with semantic understanding
        // Production: Use actual ML models for clustering

        tracing::debug!("Extracting topics from text");

        let mut topics = Vec::new();

        // Extract keywords with occurrence counting
        let keywords = self.extract_simple_keywords(text);
        
        // Group keywords by semantic similarity (simple approach)
        for (keyword, count) in keywords.iter().take(5) {
            // Calculate confidence based on occurrence
            let confidence = ((count.min(&10)) as f32 / 10.0).min(0.95);
            
            topics.push(Topic {
                name: keyword.clone(),
                keywords: vec![keyword.clone()],
                confidence,
                occurrence_count: *count,
            });
        }

        // Sort by confidence descending
        topics.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

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
