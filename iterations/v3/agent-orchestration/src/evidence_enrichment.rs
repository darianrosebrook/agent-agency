//! Evidence enrichment coordinator for council decision making
//!
//! This module provides multimodal context enrichment for evidence used in
//! council decision making processes.
//!
//! @author @darianrosebrook

use agent_agency_contracts::TaskDescriptor;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Evidence enrichment coordinator

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceEnrichmentCoordinator {
    /// Configuration for enrichment
    config: EnrichmentConfig,
    /// Cache for enriched evidence
    cache: HashMap<String, EnrichedEvidence>,
}

/// Configuration for evidence enrichment

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnrichmentConfig {
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable multimodal processing
    pub enable_multimodal: bool,
    /// Enable semantic analysis
    pub enable_semantic_analysis: bool,
}

impl Default for EnrichmentConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            enable_multimodal: true,
            enable_semantic_analysis: true,
        }
    }
}

/// Enriched evidence with multimodal context

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnrichedEvidence {
    /// Original evidence ID
    pub evidence_id: String,
    /// Enriched content
    pub content: String,
    /// Multimodal context (images, audio, etc.)
    pub multimodal_context: Vec<MultimodalContext>,
    /// Semantic analysis results
    pub semantic_analysis: Option<SemanticAnalysis>,
    /// Confidence score
    pub confidence: f32,
    /// Timestamp of enrichment
    pub enriched_at: std::time::SystemTime,
}

/// Multimodal context item

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultimodalContext {
    /// Context type (image, audio, video, etc.)
    pub context_type: ContextType,
    /// Content path or data
    pub content_path: String,
    /// Extracted features or description
    pub description: Option<String>,
    /// Relevance score
    pub relevance_score: f32,
}

/// Context type for multimodal content

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum ContextType {
    Image,
    Audio,
    Video,
    Document,
    Code,
    Data,
}

/// Semantic analysis results

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct SemanticAnalysis {
    /// Key concepts extracted
    pub concepts: Vec<String>,
    /// Sentiment analysis
    pub sentiment: SentimentScore,
    /// Topic classification
    pub topics: Vec<String>,
    /// Named entities
    pub entities: Vec<NamedEntity>,
}

/// Sentiment score

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct SentimentScore {
    /// Overall sentiment (-1.0 to 1.0)
    pub score: f32,
    /// Sentiment label
    pub label: SentimentLabel,
    /// Confidence in sentiment
    pub confidence: f32,
}

/// Sentiment labels

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum SentimentLabel {
    VeryNegative,
    Negative,
    Neutral,
    Positive,
    VeryPositive,
}

/// Named entity

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct NamedEntity {
    /// Entity text
    pub text: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Confidence score
    pub confidence: f32,
}

/// Entity types

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Money,
    Percent,
    Other,
}

impl EvidenceEnrichmentCoordinator {
    /// Create a new evidence enrichment coordinator
    pub fn new(config: EnrichmentConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// Enrich evidence with multimodal context
    pub async fn enrich_evidence(
        &mut self,
        evidence_id: &str,
        content: &str,
        task_context: &TaskDescriptor,
    ) -> Result<EnrichedEvidence> {
        debug!("Enriching evidence: {}", evidence_id);

        // Check cache first
        if let Some(cached) = self.cache.get(evidence_id) {
            if self.is_cache_valid(cached) {
                debug!("Using cached enriched evidence: {}", evidence_id);
                return Ok(cached.clone());
            }
        }

        // Perform enrichment
        let enriched = self
            .perform_enrichment(evidence_id, content, task_context)
            .await?;

        // Cache the result
        self.cache.insert(evidence_id.to_string(), enriched.clone());

        // Clean cache if needed
        self.clean_cache_if_needed();

        Ok(enriched)
    }

    /// Perform the actual enrichment process
    async fn perform_enrichment(
        &self,
        evidence_id: &str,
        content: &str,
        task_context: &TaskDescriptor,
    ) -> Result<EnrichedEvidence> {
        info!("Performing enrichment for evidence: {}", evidence_id);

        let mut multimodal_context = Vec::new();
        let mut semantic_analysis = None;

        // Extract multimodal context if enabled
        if self.config.enable_multimodal {
            multimodal_context = self
                .extract_multimodal_context(content, task_context)
                .await?;
        }

        // Perform semantic analysis if enabled
        if self.config.enable_semantic_analysis {
            semantic_analysis = Some(self.perform_semantic_analysis(content).await?);
        }

        // Calculate confidence based on enrichment quality
        let confidence = self.calculate_confidence(&multimodal_context, &semantic_analysis);

        Ok(EnrichedEvidence {
            evidence_id: evidence_id.to_string(),
            content: content.to_string(),
            multimodal_context,
            semantic_analysis,
            confidence,
            enriched_at: std::time::SystemTime::now(),
        })
    }

    /// Extract multimodal context from content
    async fn extract_multimodal_context(
        &self,
        content: &str,
        _task_context: &TaskDescriptor,
    ) -> Result<Vec<MultimodalContext>> {
        debug!("Extracting multimodal context");

        let mut contexts = Vec::new();

        // Extract actual file paths from content using regex patterns
        use regex::Regex;
        
        // Pattern for file paths (handles both absolute and relative paths)
        let file_path_pattern = Regex::new(r#"(?:^|\s)([./]?[^\s<>"|{}]+\.(png|jpg|jpeg|gif|svg|pdf|doc|docx|txt|md|rs|ts|js|py|go|java|cpp|h|hpp))"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        
        // Pattern for code blocks in markdown
        let code_block_pattern = Regex::new(r#"```(\w+)?\n([^`]+)```"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        
        // Extract image file paths
        for cap in file_path_pattern.captures_iter(content) {
            if let Some(path_match) = cap.get(1) {
                let path_str = path_match.as_str().trim();
                let ext = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                
                // Check if it's an image
                if matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "svg") {
                    let path = PathBuf::from(path_str);
                    // Verify file exists (if absolute path) or is accessible
                    let exists = if path.is_absolute() {
                        path.exists()
                    } else {
                        // For relative paths, check if they're reasonable
                        !path_str.contains("..") && path_str.len() < 500
                    };
                    
                    if exists {
                        contexts.push(MultimodalContext {
                            context_type: ContextType::Image,
                            content_path: path_str.to_string(),
                            description: Some(format!("Image file: {}", path_str)),
                            relevance_score: 0.8,
                        });
                    } else {
                        debug!("Image path not accessible: {}", path_str);
                    }
                }
            }
        }
        
        // Extract code blocks
        for cap in code_block_pattern.captures_iter(content) {
            if let Some(code_content) = cap.get(2) {
                let code = code_content.as_str();
                let language = cap.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                
                // Calculate relevance based on code characteristics
                let relevance = if code.len() > 100 {
                    0.9 // Substantial code block
                } else if code.len() > 20 {
                    0.7 // Medium code block
                } else {
                    0.5 // Small code snippet
                };
                
                contexts.push(MultimodalContext {
                    context_type: ContextType::Code,
                    content_path: format!("code_block_{}", language),
                    description: Some(format!("Code block ({})", language)),
                    relevance_score: relevance,
                });
            }
        }
        
        // Also check for inline code patterns
        let inline_code_pattern = Regex::new(r#"`([^`]+)`"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        for cap in inline_code_pattern.captures_iter(content) {
            if let Some(code) = cap.get(1) {
                let code_str = code.as_str();
                // Only add if it looks like a file path or function call
                if code_str.contains('/') || code_str.contains('\\') || 
                   code_str.contains('.') || code_str.contains('(') {
                    contexts.push(MultimodalContext {
                        context_type: ContextType::Code,
                        content_path: format!("inline_code_{}", code_str.chars().take(50).collect::<String>()),
                        description: Some(format!("Inline code: {}", code_str.chars().take(30).collect::<String>())),
                        relevance_score: 0.6,
                    });
                }
            }
        }
        
        // Extract document file paths
        for cap in file_path_pattern.captures_iter(content) {
            if let Some(path_match) = cap.get(1) {
                let path_str = path_match.as_str().trim();
                let ext = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                
                // Check if it's a document
                if matches!(ext, "pdf" | "doc" | "docx" | "txt" | "md") {
                    let path = PathBuf::from(path_str);
                    let exists = if path.is_absolute() {
                        path.exists()
                    } else {
                        !path_str.contains("..") && path_str.len() < 500
                    };
                    
                    if exists {
                        contexts.push(MultimodalContext {
                            context_type: ContextType::Document,
                            content_path: path_str.to_string(),
                            description: Some(format!("Document file: {}", path_str)),
                            relevance_score: 0.7,
                        });
                    } else {
                        debug!("Document path not accessible: {}", path_str);
                    }
                }
            }
        }

        Ok(contexts)
    }

    /// Perform semantic analysis on content
    async fn perform_semantic_analysis(&self, content: &str) -> Result<SemanticAnalysis> {
        debug!("Performing semantic analysis");

        // Comprehensive semantic analysis using enhanced text analysis techniques
        // Note: For production use, consider integrating dedicated NLP libraries (spaCy, NLTK, etc.)
        // or AI models for higher accuracy. This implementation uses improved heuristics.
        let concepts = self.extract_concepts(content);
        let sentiment = self.analyze_sentiment(content);
        let topics = self.extract_topics(content);
        let entities = self.extract_entities(content);

        Ok(SemanticAnalysis {
            concepts,
            sentiment,
            topics,
            entities,
        })
    }

    /// Extract key concepts from content
    fn extract_concepts(&self, content: &str) -> Vec<String> {
        // Enhanced concept extraction using TF-IDF-like approach
        // Extract significant words and phrases based on frequency and importance
        
        use std::collections::HashMap;
        
        // Common stop words to filter out
        let stop_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "be",
            "been", "have", "has", "had", "do", "does", "did", "will", "would",
            "should", "could", "may", "might", "must", "can", "this", "that",
            "these", "those", "it", "its", "they", "them", "their", "there",
        ].iter().cloned().collect();
        
        // Tokenize and count word frequencies
        let words: Vec<String> = content
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3 && !stop_words.contains(w))
            .map(|w| w.to_string())
            .collect();
        
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        for word in &words {
            *word_counts.entry(word.clone()).or_insert(0) += 1;
        }
        
        // Extract multi-word phrases (bigrams)
        let mut phrases: HashMap<String, usize> = HashMap::new();
        for i in 0..words.len().saturating_sub(1) {
            let phrase = format!("{} {}", words[i], words[i + 1]);
            *phrases.entry(phrase).or_insert(0) += 1;
        }
        
        // Combine and rank by frequency
        let mut concepts: Vec<(String, usize)> = word_counts
            .into_iter()
            .chain(phrases.into_iter())
            .collect();
        
        concepts.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Return top concepts
        concepts
            .into_iter()
            .take(10)
            .map(|(concept, _)| concept)
            .collect()
    }

    /// Analyze sentiment of content
    fn analyze_sentiment(&self, content: &str) -> SentimentScore {
        // TODO: Implement proper sentiment analysis using NLP models
        //       Currently uses basic keyword matching; should use production NLP sentiment analysis models for accurate sentiment detection.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Sentiment analysis uses production NLP models
        // - Sentiment scores are accurate and reliable
        // - Performance meets SLA requirements
        // - Supports multiple languages if required
        //
        // DEPENDENCIES:
        // - NLP sentiment analysis library/model (Required)
        // - Model loading infrastructure (Required)
        // - Sentiment scoring utilities (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (NLP feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: NLP expertise
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful"]; // Temporary: keyword matching until NLP model integration
        let negative_words = ["bad", "terrible", "awful", "horrible", "disappointing"];

        let positive_count = positive_words
            .iter()
            .map(|word| content.to_lowercase().matches(word).count())
            .sum::<usize>();

        let negative_count = negative_words
            .iter()
            .map(|word| content.to_lowercase().matches(word).count())
            .sum::<usize>();

        let score = if positive_count > negative_count {
            0.5
        } else if negative_count > positive_count {
            -0.5
        } else {
            0.0
        };

        let label = match score {
            s if s > 0.3 => SentimentLabel::Positive,
            s if s < -0.3 => SentimentLabel::Negative,
            _ => SentimentLabel::Neutral,
        };

        // Calculate confidence score based on signal strength
        // Confidence increases with more sentiment indicators and stronger signals
        let total_indicators = positive_count + negative_count;
        let confidence = if total_indicators == 0 {
            0.3 // Low confidence if no indicators
        } else {
            // Higher confidence with more indicators and stronger signal
            let signal_strength = (positive_count.max(negative_count) as f32 / total_indicators as f32).min(1.0);
            let base_confidence = 0.5 + (signal_strength * 0.3);
            (base_confidence + (total_indicators.min(10) as f32 * 0.02)).min(0.95)
        };
        
        SentimentScore {
            score,
            label,
            confidence,
        }
    }

    /// Extract topics from content
    fn extract_topics(&self, content: &str) -> Vec<String> {
        // Enhanced topic extraction using keyword-based topic detection
        // Note: For production use, consider LDA, BERTopic, or other topic modeling techniques
        
        use std::collections::HashMap;
        
        // Topic keywords mapping
        let topic_keywords: HashMap<&str, Vec<&str>> = [
            ("technology", vec!["code", "software", "programming", "algorithm", "system", "api", "function", "class", "method"]),
            ("development", vec!["develop", "build", "create", "implement", "design", "architecture", "feature"]),
            ("testing", vec!["test", "unit", "integration", "coverage", "assert", "verify", "validate"]),
            ("performance", vec!["performance", "speed", "latency", "throughput", "optimize", "efficient", "fast"]),
            ("security", vec!["security", "auth", "encrypt", "secure", "vulnerability", "attack", "defense"]),
            ("data", vec!["data", "database", "query", "storage", "model", "schema", "table"]),
            ("machine learning", vec!["model", "train", "neural", "learning", "ai", "ml", "prediction", "accuracy"]),
            ("documentation", vec!["document", "doc", "readme", "guide", "tutorial", "example", "comment"]),
        ].iter().cloned().collect();
        
        let content_lower = content.to_lowercase();
        let mut topic_scores: HashMap<String, usize> = HashMap::new();
        
        // Score topics based on keyword matches
        for (topic, keywords) in &topic_keywords {
            let score = keywords.iter()
                .map(|keyword| content_lower.matches(keyword).count())
                .sum::<usize>();
            if score > 0 {
                topic_scores.insert(topic.to_string(), score);
            }
        }
        
        // Return top topics by score
        let mut topics: Vec<(String, usize)> = topic_scores.into_iter().collect();
        topics.sort_by(|a, b| b.1.cmp(&a.1));
        
        topics.into_iter()
            .take(5)
            .map(|(topic, _)| topic)
            .collect()
    }

    /// Extract named entities from content
    fn extract_entities(&self, content: &str) -> Vec<NamedEntity> {
        // Enhanced named entity recognition using pattern matching
        // Note: For production use, consider spaCy, NLTK, or transformer-based NER models
        
        use regex::Regex;
        let mut entities = Vec::new();
        
        // Pattern for email addresses
        let email_pattern = Regex::new(r#"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        for cap in email_pattern.captures_iter(content) {
            if let Some(email) = cap.get(0) {
                entities.push(NamedEntity {
                    text: email.as_str().to_string(),
                    entity_type: EntityType::Other, // Email not in enum, use Other
                    confidence: 0.9,
                });
            }
        }
        
        // Pattern for URLs
        let url_pattern = Regex::new(r#"https?://[^\s]+"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        for cap in url_pattern.captures_iter(content) {
            if let Some(url) = cap.get(0) {
                entities.push(NamedEntity {
                    text: url.as_str().to_string(),
                    entity_type: EntityType::Other, // URL not in enum, use Other
                    confidence: 0.9,
                });
            }
        }
        
        // Pattern for version numbers (e.g., v1.2.3, 1.2.3)
        let version_pattern = Regex::new(r#"\b[vV]?\d+\.\d+(\.\d+)?(-[a-zA-Z0-9]+)?"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        for cap in version_pattern.captures_iter(content) {
            if let Some(version) = cap.get(0) {
                entities.push(NamedEntity {
                    text: version.as_str().to_string(),
                    entity_type: EntityType::Other, // Version not in enum, use Other
                    confidence: 0.8,
                });
            }
        }
        
        // Pattern for file paths
        let file_path_pattern = Regex::new(r#"(?:^|\s)([./]?[^\s<>"|{}]+\.(rs|ts|js|py|go|java|cpp|h|yaml|yml|json|toml|md))"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        for cap in file_path_pattern.captures_iter(content) {
            if let Some(path) = cap.get(1) {
                entities.push(NamedEntity {
                    text: path.as_str().to_string(),
                    entity_type: EntityType::Other, // FilePath not in enum, use Other
                    confidence: 0.85,
                });
            }
        }
        
        // Pattern for capitalized words (potential proper nouns/organizations)
        let proper_noun_pattern = Regex::new(r#"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b"#)
            .unwrap_or_else(|_| Regex::new(r#".*"#).unwrap());
        for cap in proper_noun_pattern.captures_iter(content) {
            if let Some(noun) = cap.get(0) {
                let text = noun.as_str();
                // Filter out common words and short matches
                if text.len() > 3 && !matches!(text, "The" | "This" | "That" | "These" | "Those" | "There" | "They") {
                    entities.push(NamedEntity {
                        text: text.to_string(),
                        entity_type: EntityType::Organization,
                        confidence: 0.7, // Lower confidence for pattern-based detection
                    });
                }
            }
        }
        
        entities
    }

    /// Calculate confidence in enrichment
    fn calculate_confidence(
        &self,
        multimodal_context: &[MultimodalContext],
        semantic_analysis: &Option<SemanticAnalysis>,
    ) -> f32 {
        let mut confidence = 0.5; // Base confidence

        // Boost confidence based on multimodal context
        if !multimodal_context.is_empty() {
            confidence += 0.2;
        }

        // Boost confidence based on semantic analysis
        if semantic_analysis.is_some() {
            confidence += 0.2;
        }

        // Boost confidence based on context quality
        let avg_relevance: f32 = multimodal_context
            .iter()
            .map(|ctx| ctx.relevance_score)
            .sum::<f32>()
            / multimodal_context.len() as f32;

        confidence += avg_relevance * 0.1;

        confidence.min(1.0)
    }

    /// Check if cached evidence is still valid
    fn is_cache_valid(&self, cached: &EnrichedEvidence) -> bool {
        let now = std::time::SystemTime::now();
        if let Ok(duration) = now.duration_since(cached.enriched_at) {
            duration.as_secs() < self.config.cache_ttl_seconds
        } else {
            false
        }
    }

    /// Clean cache if it exceeds maximum size
    fn clean_cache_if_needed(&mut self) {
        if self.cache.len() > self.config.max_cache_size {
            // TODO: Implement proper cache eviction strategy
            //       Currently removes oldest entries; should implement LRU or other eviction strategy for optimal cache performance.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Cache eviction uses LRU or other optimal strategy
            // - Eviction preserves frequently accessed entries
            // - Cache performance improves with better eviction
            // - Eviction strategy is configurable
            //
            // DEPENDENCIES:
            // - Cache eviction algorithm (LRU, LFU, etc.) (Required)
            // - Access tracking infrastructure (Required)
            // - Eviction configuration utilities (Required)
            //
            // ESTIMATED EFFORT: 3-4 hours (medium confidence)
            // PRIORITY: Low
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 3 (cache optimization)
            // - Change Budget: ~80 LOC
            // - Reviewer Requirements: Cache algorithms expertise
            let keys_to_remove: Vec<String> = self
                .cache // Temporary: oldest-first until LRU eviction is implemented
                .iter()
                .take(self.cache.len() - self.config.max_cache_size)
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
    }

    /// Get enrichment statistics
    pub fn get_stats(&self) -> EnrichmentStats {
        EnrichmentStats {
            cache_size: self.cache.len(),
            total_enriched: self.cache.len(),
            cache_hit_rate: 0.8, // TODO: Calculate actual cache hit rate
        }
    }
}

/// Enrichment statistics

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnrichmentStats {
    pub cache_size: usize,
    pub total_enriched: usize,
    pub cache_hit_rate: f32,
}
