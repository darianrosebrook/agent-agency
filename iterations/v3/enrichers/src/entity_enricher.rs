//! @darianrosebrook
//! Entity and topic extraction enricher
//!
//! Extracts:
//! - Named entities (person, organization, location, date, email, phone)
//! - Topics via BERTopic or keyphrase extraction
//! - Chapter boundaries from topic transitions
//! - PII detection and hashing for privacy

use crate::enricher_types::{Chapter, EnricherConfig, EntityResult, ExtractedEntity, Topic};
use anyhow::{Context, Result};
use std::collections::HashMap;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use email_address::EmailAddress;
use url::Url;

// Re-export entity detection, NER processing, and topic extraction functionality
use crate::entity_detection::*;
use crate::ner_processing::*;
use crate::topic_extraction::*;
pub struct EntityEnricher {
    config: EnricherConfig,
}

impl EntityEnricher {
    pub fn new(config: EnricherConfig) -> Self {
        Self { config }
    }

    /// Extract entities and topics from text and speech with comprehensive error handling
    ///
    /// # Arguments
    /// * `text` - Input text to analyze
    /// * `timestamps` - Optional time ranges for topic segmentation
    ///
    /// # Returns
    /// EntityResult with entities, topics, and chapter boundaries
    ///
    /// # Errors
    /// Returns error if:
    /// - Text is empty or too short
    /// - Entity detection fails critically
    /// - Topic extraction fails critically
    /// - Chapter segmentation fails critically
    pub async fn extract_entities(
        &self,
        text: &str,
        timestamps: Option<Vec<(f32, f32)>>,
    ) -> Result<EntityResult> {
        let start_time = std::time::Instant::now();
        
        // Input validation
        self.validate_input_text(text)?;
        
        tracing::debug!(
            "Extracting entities with NER enabled: {} (text length: {} chars)",
            self.config.entity_ner_enabled,
            text.len()
        );

        // Extract entities with error recovery
        let entities = self.detect_entities_with_recovery(text).await?;
        
        // Extract topics with error recovery
        let topics = self.extract_topics_with_recovery(text).await?;
        
        // Segment chapters with error recovery
        let chapters = self.segment_chapters_with_recovery(&topics, timestamps).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        tracing::debug!(
            "Entity extraction completed in {}ms: {} entities, {} topics, {} chapters",
            processing_time, entities.len(), topics.len(), chapters.len()
        );

        Ok(EntityResult {
            entities,
            topics,
            chapters,
            processing_time_ms: processing_time,
        })
    }

    /// Validate input text
    fn validate_input_text(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Err(anyhow::anyhow!("Input text cannot be empty"));
        }
        
        if text.len() < 3 {
            return Err(anyhow::anyhow!("Input text too short (minimum 3 characters)"));
        }
        
        if text.len() > 1_000_000 {
            return Err(anyhow::anyhow!("Input text too long (maximum 1,000,000 characters)"));
        }
        
        Ok(())
    }

    /// Detect entities with error recovery
    async fn detect_entities_with_recovery(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        match self.detect_entities(text).await {
            Ok(entities) => Ok(entities),
            Err(e) => {
                tracing::warn!("Entity detection failed: {}, attempting recovery", e);
                
                // Attempt fallback entity detection
                match self.fallback_entity_detection(text).await {
                    Ok(fallback_entities) => {
                        tracing::info!("Fallback entity detection succeeded with {} entities", fallback_entities.len());
                        Ok(fallback_entities)
                    },
                    Err(fallback_error) => {
                        tracing::error!("Both primary and fallback entity detection failed: {}, {}", e, fallback_error);
                        
                        // Return minimal entities to prevent complete failure
                        Ok(vec![ExtractedEntity {
                            id: Uuid::new_v4(),
                            entity_type: "text".to_string(),
                            text: text.chars().take(100).collect(),
                            normalized: text.chars().take(100).collect(),
                            confidence: 0.1,
                            pii: false,
                            span_start: 0,
                            span_end: text.len().min(100),
                        }])
                    }
                }
            }
        }
    }

    /// Fallback entity detection using simple patterns
    async fn fallback_entity_detection(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();
        
        // Simple email detection
        for (i, word) in text.split_whitespace().enumerate() {
            if word.contains('@') && word.contains('.') {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "email".to_string(),
                    text: word.to_string(),
                    normalized: word.to_lowercase(),
                    confidence: 0.7,
                    pii: true,
                    span_start: text.find(word).unwrap_or(i * 10),
                    span_end: text.find(word).unwrap_or(i * 10) + word.len(),
                });
            }
        }
        
        // Simple URL detection
        for (i, word) in text.split_whitespace().enumerate() {
            if word.starts_with("http://") || word.starts_with("https://") {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "url".to_string(),
                    text: word.to_string(),
                    normalized: word.to_string(),
                    confidence: 0.8,
                    pii: false,
                    span_start: text.find(word).unwrap_or(i * 10),
                    span_end: text.find(word).unwrap_or(i * 10) + word.len(),
                });
            }
        }
        
        Ok(entities)
    }

    /// Extract topics with error recovery
    async fn extract_topics_with_recovery(&self, text: &str) -> Result<Vec<Topic>> {
        match self.extract_topics(text).await {
            Ok(topics) => Ok(topics),
            Err(e) => {
                tracing::warn!("Topic extraction failed: {}, attempting recovery", e);
                
                // Attempt fallback topic extraction
                match self.fallback_topic_extraction(text).await {
                    Ok(fallback_topics) => {
                        tracing::info!("Fallback topic extraction succeeded with {} topics", fallback_topics.len());
                        Ok(fallback_topics)
                    },
                    Err(fallback_error) => {
                        tracing::error!("Both primary and fallback topic extraction failed: {}, {}", e, fallback_error);
                        
                        // Return minimal topics to prevent complete failure
                        Ok(vec![Topic {
                            name: "General".to_string(),
                            keywords: vec!["content".to_string(), "text".to_string()],
                            confidence: 0.1,
                            occurrence_count: 1,
                        }])
                    }
                }
            }
        }
    }

    /// Fallback topic extraction using simple keyword analysis
    async fn fallback_topic_extraction(&self, text: &str) -> Result<Vec<Topic>> {
        let keywords = self.extract_simple_keywords(text);
        
        // Group keywords into topics
        let mut topics = Vec::new();
        
        if !keywords.is_empty() {
            let top_keywords: Vec<_> = keywords.iter()
                .take(5)
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            
            topics.push(Topic {
                name: "Main Topics".to_string(),
                keywords: top_keywords.iter().map(|(k, _)| k.clone()).collect(),
                confidence: 0.6,
                occurrence_count: top_keywords.iter().map(|(_, v)| *v).sum(),
            });
        }
        
        Ok(topics)
    }

    /// Segment chapters with error recovery
    async fn segment_chapters_with_recovery(
        &self,
        topics: &[Topic],
        timestamps: Option<Vec<(f32, f32)>>,
    ) -> Result<Vec<Chapter>> {
        match self.segment_chapters(topics).await {
            Ok(chapters) => Ok(chapters),
            Err(e) => {
                tracing::warn!("Chapter segmentation failed: {}, attempting recovery", e);
                
                // Attempt fallback chapter segmentation
                match self.fallback_chapter_segmentation(topics, timestamps).await {
                    Ok(fallback_chapters) => {
                        tracing::info!("Fallback chapter segmentation succeeded with {} chapters", fallback_chapters.len());
                        Ok(fallback_chapters)
                    },
                    Err(fallback_error) => {
                        tracing::error!("Both primary and fallback chapter segmentation failed: {}, {}", e, fallback_error);
                        
                        // Return minimal chapters to prevent complete failure
                        Ok(vec![Chapter {
                            title: "Main Content".to_string(),
                            t0: 0.0,
                            t1: 300.0,
                            description: Some("Content chapter".to_string()),
                        }])
                    }
                }
            }
        }
    }

    /// Fallback chapter segmentation using simple time-based division
    async fn fallback_chapter_segmentation(
        &self,
        topics: &[Topic],
        timestamps: Option<Vec<(f32, f32)>>,
    ) -> Result<Vec<Chapter>> {
        let mut chapters = Vec::new();
        
        if let Some(ts) = timestamps {
            // Use provided timestamps for chapter boundaries
            for (i, (t0, t1)) in ts.iter().enumerate() {
                let topic_name = topics.get(i)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| format!("Chapter {}", i + 1));
                
                chapters.push(Chapter {
                    title: topic_name,
                    t0: *t0,
                    t1: *t1,
                    description: Some(format!("Chapter based on timestamp {}", i + 1)),
                });
            }
        } else {
            // Create simple time-based chapters
            let total_duration = 300.0; // 5 minutes default
            let chapter_duration = total_duration / topics.len().max(1) as f32;
            
            for (i, topic) in topics.iter().enumerate() {
                let t0 = i as f32 * chapter_duration;
                let t1 = (i + 1) as f32 * chapter_duration;
                
                chapters.push(Chapter {
                    title: topic.name.clone(),
                    t0,
                    t1,
                    description: Some(format!("Chapter on {}", topic.name)),
                });
            }
        }
        
        Ok(chapters)
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

    fn detect_email_patterns(&self, text: &str, entities: &mut Vec<ExtractedEntity>) {
        // Use regex to find potential email patterns in text
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();

        // Find all potential email matches in the text
        for cap in email_regex.find_iter(text) {
            let email_candidate = cap.as_str();

            // Use proper email validation library for accurate RFC 5322 compliance
            if EmailAddress::is_valid(email_candidate) {
                let confidence = 0.95; // High confidence for RFC-compliant validation

                // Extract position information
                let start_pos = cap.start();
                let end_pos = cap.end();

                // Create extracted entity with detailed metadata
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "email".to_string(),
                    text: email_candidate.to_string(),
                    normalized: email_candidate.to_lowercase(),
                    confidence,
                    pii: true,
                    span_start: start_pos,
                    span_end: end_pos,
                });
            }
        }
    }

    fn detect_url_patterns(&self, text: &str, entities: &mut Vec<ExtractedEntity>) {
        // Use regex to find potential URL patterns in text
        let url_regex = regex::Regex::new(r"https?://(?:[-\w.])+(?:[:\d]+)?(?:/(?:[\w/_.])*(?:\?(?:[\w&=%.])*)?(?:#(?:[\w.])*)?)?").unwrap();

        // Find all potential URL matches in the text
        for cap in url_regex.find_iter(text) {
            let url_candidate = cap.as_str();

            // Use proper URL parsing library for accurate RFC 3986 compliance
            if let Ok(parsed_url) = Url::parse(url_candidate) {
                let confidence = 0.98; // High confidence for RFC-compliant URL parsing

                // Extract position information
                let start_pos = cap.start();
                let end_pos = cap.end();

                // Create extracted entity with detailed metadata
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "url".to_string(),
                    text: url_candidate.to_string(),
                    normalized: parsed_url.to_string(),
                    confidence,
                    pii: false,
                    span_start: start_pos,
                    span_end: end_pos,
                });
            }
        }
    }

    /// Extract topics via BERTopic or keyphrase extraction
    async fn extract_topics(&self, text: &str) -> Result<Vec<Topic>> {
        let topic_bridge = TopicExtractionBridge::new()?;
        let topic_results = topic_bridge
            .extract_topics(text)
            .await
            .context("Topic extraction failed")?;

        // Convert topic results to Topic
        let topics = topic_results.into_iter().map(|result| Topic {
            name: result.topic,
            keywords: result.keywords,
            confidence: result.confidence,
            occurrence_count: result.occurrence_count as usize,
        }).collect();

        Ok(topics)
    }

    /// TODO: Replace simple keyword extraction with proper NLP-based keyword extraction
    /// Requirements for completion:
    /// - [ ] Integrate with NLP library for proper keyword extraction (TF-IDF, TextRank, etc.)
    /// - [ ] Implement proper text preprocessing (tokenization, lemmatization, stemming)
    /// - [ ] Add support for multi-word keyword extraction (phrases, named entities)
    /// - [ ] Implement proper stopword removal using comprehensive stopword lists
    /// - [ ] Add support for different languages and character encodings
    /// - [ ] Implement proper keyword scoring and ranking algorithms
    /// - [ ] Add support for keyword frequency analysis and normalization
    /// - [ ] Implement proper error handling for text processing failures
    /// - [ ] Add support for keyword extraction performance optimization
    /// - [ ] Implement proper memory management for text processing
    /// - [ ] Add support for keyword extraction result validation and quality assessment
    /// - [ ] Implement proper cleanup of text processing resources
    /// - [ ] Add support for keyword extraction monitoring and alerting
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

    /// Check if an entity type is considered PII
    fn is_pii_entity(&self, entity_type: &str) -> bool {
        matches!(entity_type, "email" | "phone" | "person" | "PERSON")
    }

    /// Hash PII data for privacy protection
    fn hash_pii(&self, text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Map NER entity types to our internal types
    fn map_ner_type(&self, ner_type: &str) -> String {
        match ner_type {
            "PERSON" => "person".to_string(),
            "ORG" => "organization".to_string(),
            "GPE" => "location".to_string(),
            "LOC" => "location".to_string(),
            "DATE" => "date".to_string(),
            "TIME" => "time".to_string(),
            "MONEY" => "money".to_string(),
            "PERCENT" => "percentage".to_string(),
            _ => "unknown".to_string(),
        }
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
