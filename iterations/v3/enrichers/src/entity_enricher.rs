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

/// Apple DataDetection bridge for structured data extraction
#[derive(Debug)]
struct DataDetectionBridge {
    // In a real implementation, this would contain Swift bridge handles
}

impl DataDetectionBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing Apple DataDetection bridge");
        Ok(Self {})
    }

    async fn detect_data_types(&self, text: &str) -> Result<Vec<DataDetection>> {
        // Simulate Apple DataDetection processing
        // In a real implementation, this would:
        // 1. Create NSDataDetector with NSTextCheckingTypes
        // 2. Use NSDataDetector.matches(in:options:range:)
        // 3. Parse results into structured data
        
        tracing::debug!("Detecting data types in text ({} chars)", text.len());
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        let mut detections = Vec::new();
        
        // Simulate email detection
        if let Some(email_match) = self.find_email(text) {
            detections.push(DataDetection {
                data_type: "email".to_string(),
                text: email_match.text,
                normalized: email_match.text.to_lowercase(),
                confidence: 0.95,
                is_pii: true,
                range: email_match.range,
            });
        }
        
        // Simulate URL detection
        if let Some(url_match) = self.find_url(text) {
            detections.push(DataDetection {
                data_type: "url".to_string(),
                text: url_match.text,
                normalized: url_match.text,
                confidence: 0.98,
                is_pii: false,
                range: url_match.range,
            });
        }
        
        // Simulate phone number detection
        if let Some(phone_match) = self.find_phone(text) {
            detections.push(DataDetection {
                data_type: "phone".to_string(),
                text: phone_match.text,
                normalized: self.normalize_phone(&phone_match.text),
                confidence: 0.90,
                is_pii: true,
                range: phone_match.range,
            });
        }
        
        // Simulate date detection
        if let Some(date_match) = self.find_date(text) {
            detections.push(DataDetection {
                data_type: "date".to_string(),
                text: date_match.text,
                normalized: date_match.text,
                confidence: 0.85,
                is_pii: false,
                range: date_match.range,
            });
        }
        
        Ok(detections)
    }

    fn find_email(&self, text: &str) -> Option<TextMatch> {
        let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").ok()?;
        if let Some(mat) = email_regex.find(text) {
            Some(TextMatch {
                text: mat.as_str().to_string(),
                range: TextRange {
                    start: mat.start(),
                    end: mat.end(),
                },
            })
        } else {
            None
        }
    }

    fn find_url(&self, text: &str) -> Option<TextMatch> {
        let url_regex = Regex::new(r"https?://[^\s]+").ok()?;
        if let Some(mat) = url_regex.find(text) {
            Some(TextMatch {
                text: mat.as_str().to_string(),
                range: TextRange {
                    start: mat.start(),
                    end: mat.end(),
                },
            })
        } else {
            None
        }
    }

    fn find_phone(&self, text: &str) -> Option<TextMatch> {
        let phone_regex = Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").ok()?;
        if let Some(mat) = phone_regex.find(text) {
            Some(TextMatch {
                text: mat.as_str().to_string(),
                range: TextRange {
                    start: mat.start(),
                    end: mat.end(),
                },
            })
        } else {
            None
        }
    }

    fn find_date(&self, text: &str) -> Option<TextMatch> {
        let date_regex = Regex::new(r"\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b").ok()?;
        if let Some(mat) = date_regex.find(text) {
            Some(TextMatch {
                text: mat.as_str().to_string(),
                range: TextRange {
                    start: mat.start(),
                    end: mat.end(),
                },
            })
        } else {
            None
        }
    }

    fn normalize_phone(&self, phone: &str) -> String {
        // Remove all non-digit characters and format
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 10 {
            format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10])
        } else {
            phone.to_string()
        }
    }
}

/// DataDetection result
#[derive(Debug)]
struct DataDetection {
    data_type: String,
    text: String,
    normalized: String,
    confidence: f32,
    is_pii: bool,
    range: TextRange,
}

/// Text match with range
#[derive(Debug)]
struct TextMatch {
    text: String,
    range: TextRange,
}

/// Text range
#[derive(Debug)]
struct TextRange {
    start: usize,
    end: usize,
}

/// Named Entity Recognition bridge
#[derive(Debug)]
struct NERBridge {
    // In a real implementation, this would contain NER model handles
}

impl NERBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing NER bridge");
        Ok(Self {})
    }

    async fn extract_entities(&self, text: &str) -> Result<Vec<NEREntity>> {
        // Simulate NER processing
        // In a real implementation, this would:
        // 1. Load a pre-trained NER model (e.g., spaCy, Transformers)
        // 2. Process text through the model
        // 3. Extract entities with confidence scores
        
        tracing::debug!("Extracting named entities from text ({} chars)", text.len());
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let mut entities = Vec::new();
        
        // Simple heuristic-based entity detection
        for word in text.split_whitespace() {
            if word.chars().next().map_or(false, |c| c.is_uppercase()) && word.len() > 2 {
                // Simple heuristic: capitalized words might be entities
                let entity_type = if word.ends_with("Inc.") || word.ends_with("Corp.") || word.ends_with("LLC") {
                    "organization"
                } else if word.chars().all(|c| c.is_alphabetic()) {
                    "person"
                } else {
                    "other"
                };
                
                entities.push(NEREntity {
                    text: word.to_string(),
                    entity_type: entity_type.to_string(),
                    confidence: 0.7,
                    start: text.find(word).unwrap_or(0),
                    end: text.find(word).unwrap_or(0) + word.len(),
                });
            }
        }
        
        Ok(entities)
    }
}

/// NER entity result
#[derive(Debug)]
struct NEREntity {
    text: String,
    entity_type: String,
    confidence: f32,
    start: usize,
    end: usize,
}

/// BERTopic bridge for advanced topic modeling
#[derive(Debug)]
struct BERTopicBridge {
    // In a real implementation, this would contain BERTopic model handles
}

impl BERTopicBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing BERTopic bridge");
        Ok(Self {})
    }

    async fn extract_topics(&self, text: &str) -> Result<Vec<BERTopicResult>> {
        // Simulate BERTopic processing
        // In a real implementation, this would:
        // 1. Load a pre-trained BERTopic model
        // 2. Process text through the model
        // 3. Extract topics with keywords and confidence scores
        
        tracing::debug!("Extracting topics with BERTopic ({} chars)", text.len());
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        
        let mut topics = Vec::new();
        
        // Simple heuristic-based topic extraction
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_freq: HashMap<&str, usize> = words.iter()
            .filter(|w| w.len() > 3 && w.chars().all(|c| c.is_alphabetic()))
            .fold(HashMap::new(), |mut acc, word| {
                *acc.entry(word).or_insert(0) += 1;
                acc
            });
        
        // Create topics based on frequent words
        let mut sorted_words: Vec<_> = word_freq.iter().collect();
        sorted_words.sort_by(|a, b| b.1.cmp(a.1));
        
        for (i, (word, count)) in sorted_words.iter().take(3).enumerate() {
            topics.push(BERTopicResult {
                topic_name: format!("Topic {}", i + 1),
                keywords: vec![word.to_string()],
                confidence: 0.8 - (i as f32 * 0.1),
                occurrence_count: **count,
            });
        }
        
        Ok(topics)
    }
}

/// BERTopic result
#[derive(Debug)]
struct BERTopicResult {
    topic_name: String,
    keywords: Vec<String>,
    confidence: f32,
    occurrence_count: usize,
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

        tracing::debug!("Detecting entities in text ({} chars)", text.len());

        // Use Apple DataDetection bridge for structured data
        let data_detection_bridge = DataDetectionBridge::new()?;
        let detected_data = data_detection_bridge
            .detect_data_types(text)
            .await
            .context("DataDetection failed")?;

        // Convert DataDetection results to entities
        for detection in detected_data {
            let entity = ExtractedEntity {
                id: Uuid::new_v4(),
                entity_type: detection.data_type,
                text: detection.text,
                normalized: detection.normalized,
                confidence: detection.confidence,
                pii: detection.is_pii,
                span_start: detection.range.start,
                span_end: detection.range.end,
            };
            entities.push(entity);
        }

        // Optional NER for domain terms (if enabled)
        if self.config.entity_ner_enabled {
            let ner_entities = self.detect_named_entities(text).await?;
            entities.extend(ner_entities);
        }

        // Fallback pattern detection for additional coverage
        self.detect_email_patterns(text, &mut entities);
        self.detect_url_patterns(text, &mut entities);
        self.detect_phone_patterns(text, &mut entities);

        // Hash PII entities for privacy protection
        self.hash_pii_entities(&mut entities);

        tracing::debug!("Detected {} entities", entities.len());
        Ok(entities)
    }

    /// Detect named entities using NER
    async fn detect_named_entities(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        let ner_bridge = NERBridge::new()?;
        let ner_entities = ner_bridge
            .extract_entities(text)
            .await
            .context("NER extraction failed")?;

        let mut entities = Vec::new();
        for ner_entity in ner_entities {
            let entity = ExtractedEntity {
                id: Uuid::new_v4(),
                entity_type: ner_entity.entity_type,
                text: ner_entity.text,
                normalized: ner_entity.text.to_lowercase(),
                confidence: ner_entity.confidence,
                pii: self.is_pii_entity(&ner_entity.entity_type),
                span_start: ner_entity.start,
                span_end: ner_entity.end,
            };
            entities.push(entity);
        }

        Ok(entities)
    }

    /// Check if entity type is PII
    fn is_pii_entity(&self, entity_type: &str) -> bool {
        matches!(entity_type, "person" | "email" | "phone")
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

    /// Detect phone numbers in text
    fn detect_phone_patterns(&self, text: &str, entities: &mut Vec<ExtractedEntity>) {
        let phone_regex = Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap_or_else(|_| Regex::new(r"\d+").unwrap());
        
        for mat in phone_regex.find_iter(text) {
            entities.push(ExtractedEntity {
                id: Uuid::new_v4(),
                entity_type: "phone".to_string(),
                text: mat.as_str().to_string(),
                normalized: self.normalize_phone(mat.as_str()),
                confidence: 0.90,
                pii: true,
                span_start: mat.start(),
                span_end: mat.end(),
            });
        }
    }

    /// Normalize phone number format
    fn normalize_phone(&self, phone: &str) -> String {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 10 {
            format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10])
        } else {
            phone.to_string()
        }
    }

    /// Extract topics via BERTopic or keyphrase extraction
    async fn extract_topics(&self, text: &str) -> Result<Vec<Topic>> {
        tracing::debug!("Extracting topics from text ({} chars)", text.len());

        // Use BERTopic bridge for advanced topic modeling
        let bertopic_bridge = BERTopicBridge::new()?;
        let bertopic_results = bertopic_bridge
            .extract_topics(text)
            .await
            .context("BERTopic extraction failed")?;

        let mut topics = Vec::new();
        for bertopic_result in bertopic_results {
            let topic = Topic {
                name: bertopic_result.topic_name,
                keywords: bertopic_result.keywords,
                confidence: bertopic_result.confidence,
                occurrence_count: bertopic_result.occurrence_count,
            };
            topics.push(topic);
        }

        // Fallback to simple keyword extraction if BERTopic fails
        if topics.is_empty() {
            let keywords = self.extract_simple_keywords(text);
            for (keyword, count) in keywords.iter().take(3) {
                topics.push(Topic {
                    name: keyword.clone(),
                    keywords: vec![keyword.clone()],
                    confidence: 0.8,
                    occurrence_count: *count,
                });
            }
        }

        tracing::debug!("Extracted {} topics", topics.len());
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

    /// Hash PII entities for privacy protection
    fn hash_pii_entities(&self, entities: &mut Vec<ExtractedEntity>) {
        for entity in entities.iter_mut() {
            if entity.pii {
                let mut hasher = Sha256::new();
                hasher.update(entity.text.as_bytes());
                let hash = hasher.finalize();
                entity.normalized = format!("{:x}", hash);
                tracing::debug!("Hashed PII entity: {} -> {}", entity.text, entity.normalized);
            }
        }
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
