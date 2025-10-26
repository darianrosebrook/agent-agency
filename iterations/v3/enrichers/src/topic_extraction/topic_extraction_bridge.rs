//! Topic Extraction Bridge
//!
//! Provides topic extraction and keyphrase analysis using
//! keyword matching, pattern recognition, and frequency analysis.

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Result of topic extraction
#[derive(Debug, Clone)]
pub struct TopicExtractionResult {
    /// The identified topic
    pub topic: String,
    /// Keywords associated with this topic
    pub keywords: Vec<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Number of occurrences in the text
    pub occurrence_count: u32,
}

/// Topic extraction bridge for actively extracting topics and key phrases from text
#[derive(Debug)]
pub struct TopicExtractionBridge {
    stopwords: std::collections::HashSet<String>,
    topic_keywords: HashMap<String, Vec<String>>,
    keyphrase_patterns: Vec<Regex>,
}

impl TopicExtractionBridge {
    /// Create a new topic extraction bridge
    pub fn new() -> Result<Self> {
        tracing::debug!("Initializing topic extraction bridge with pattern matching");

        Ok(Self {
            stopwords: Self::load_stopwords(),
            topic_keywords: Self::load_topic_keywords(),
            keyphrase_patterns: vec![
                Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b")?, // Two-word phrases
                Regex::new(r"\b[A-Z][a-z]+\s+[a-z]+\s+[A-Z][a-z]+\b")?, // Three-word phrases
                Regex::new(r"\b(?:artificial intelligence|machine learning|deep learning|neural network)\b")?,
                Regex::new(r"\b(?:business strategy|market analysis|financial planning|project management)\b")?,
                Regex::new(r"\b(?:health care|medical research|clinical trial|patient care)\b")?,
                Regex::new(r"\b(?:environmental protection|climate change|sustainable development|renewable energy)\b")?,
                Regex::new(r"\b(?:educational technology|online learning|student engagement|curriculum development)\b")?,
            ],
        })
    }

    /// Extract topics from text
    pub async fn extract_topics(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        tracing::debug!("Extracting topics with enhanced pattern matching ({} chars)", text.len());

        let mut results = Vec::new();

        // Extract topics based on keyword matching
        results.extend(self.extract_topics_by_keywords(text)?);

        // Extract keyphrases using regex patterns
        results.extend(self.extract_keyphrases(text)?);

        // Extract topics using TF-IDF-like scoring
        results.extend(self.extract_topics_by_frequency(text)?);

        // Merge similar topics and calculate final scores
        let merged_results = self.merge_similar_topics(results);

        // Sort by confidence and occurrence count
        let mut final_results = merged_results;
        final_results.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
        });

        // Limit to top topics
        final_results.truncate(10);

        tracing::debug!("Extracted {} topics", final_results.len());
        Ok(final_results)
    }

    /// Extract topics by matching against known topic keywords
    fn extract_topics_by_keywords(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        let mut results = Vec::new();
        let text_lower = text.to_lowercase();

        for (topic, keywords) in &self.topic_keywords {
            let mut occurrence_count = 0;
            let mut matched_keywords = Vec::new();

            for keyword in keywords {
                let count = text_lower.matches(keyword).count();
                if count > 0 {
                    occurrence_count += count as u32;
                    matched_keywords.push(keyword.clone());
                }
            }

            if occurrence_count > 0 {
                let confidence = (matched_keywords.len() as f32 / keywords.len() as f32).min(1.0);
                results.push(TopicExtractionResult {
                    topic: topic.clone(),
                    keywords: matched_keywords,
                    confidence,
                    occurrence_count,
                });
            }
        }

        Ok(results)
    }

    /// Extract keyphrases using regex patterns
    fn extract_keyphrases(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        let mut results = Vec::new();
        let mut phrase_counts = HashMap::new();

        for pattern in &self.keyphrase_patterns {
            for mat in pattern.find_iter(text) {
                let phrase = mat.as_str().to_string();
                *phrase_counts.entry(phrase).or_insert(0) += 1;
            }
        }

        for (phrase, count) in phrase_counts {
            if count >= 2 { // Require at least 2 occurrences
                let confidence = (count as f32 / 10.0).min(1.0); // Normalize confidence
                results.push(TopicExtractionResult {
                    topic: phrase.clone(),
                    keywords: vec![phrase],
                    confidence,
                    occurrence_count: count,
                });
            }
        }

        Ok(results)
    }

    /// Extract topics using frequency analysis (simple TF-IDF approximation)
    fn extract_topics_by_frequency(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        let words = self.extract_simple_keywords(text);

        let mut results = Vec::new();
        for (word, count) in words.iter().filter(|(_, &count)| count >= 3) {
            if !self.stopwords.contains(word) && word.len() > 4 {
                let confidence = (count as f32 / 20.0).min(0.8); // Cap at 0.8 for frequency-based
                results.push(TopicExtractionResult {
                    topic: word.clone(),
                    keywords: vec![word.clone()],
                    confidence,
                    occurrence_count: *count,
                });
            }
        }

        Ok(results)
    }

    /// Extract simple keywords from text
    fn extract_simple_keywords(&self, text: &str) -> HashMap<String, usize> {
        let mut word_counts = HashMap::new();

        for word in text.split_whitespace() {
            let clean_word = word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();

            if clean_word.len() >= 3 && !self.stopwords.contains(&clean_word) {
                *word_counts.entry(clean_word).or_insert(0) += 1;
            }
        }

        word_counts
    }

    /// Merge similar topics to avoid duplicates
    fn merge_similar_topics(&self, mut results: Vec<TopicExtractionResult>) -> Vec<TopicExtractionResult> {
        let mut merged: Vec<TopicExtractionResult> = Vec::new();

        'outer: for result in results {
            for existing in &mut merged {
                if self.topics_similar(&result.topic, &existing.topic) {
                    // Merge topics
                    existing.confidence = existing.confidence.max(result.confidence);
                    existing.occurrence_count += result.occurrence_count;
                    existing.keywords.extend(result.keywords);
                    existing.keywords.sort();
                    existing.keywords.dedup();
                    continue 'outer;
                }
            }
            merged.push(result);
        }

        merged
    }

    /// Check if two topics are similar
    fn topics_similar(&self, topic1: &str, topic2: &str) -> bool {
        use strsim::jaro_winkler;
        jaro_winkler(topic1, topic2) > 0.85
    }

    /// Load stopwords for filtering
    fn load_stopwords() -> std::collections::HashSet<String> {
        let stopwords = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
            "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they", "me",
            "him", "her", "us", "them", "my", "your", "his", "its", "our", "their", "what", "which",
            "who", "when", "where", "why", "how", "all", "any", "both", "each", "few", "more",
            "most", "other", "some", "such", "no", "nor", "not", "only", "own", "same", "so",
            "than", "too", "very", "can", "will", "just", "should", "now", "here", "there",
            "then", "once", "also", "been", "being", "have", "has", "had", "having", "do", "does",
            "did", "doing", "would", "could", "should", "may", "might", "must", "shall", "will",
            "was", "were", "be", "been", "being", "is", "am", "are", "were", "was", "be", "being",
            "have", "has", "had", "having", "do", "does", "did", "doing", "would", "could",
            "should", "may", "might", "must", "shall", "will",
        ];

        stopwords.into_iter().map(|s| s.to_string()).collect()
    }

    /// Load topic keywords for classification
    fn load_topic_keywords() -> HashMap<String, Vec<String>> {
        let mut topics = HashMap::new();

        // AI/ML topics
        topics.insert("Artificial Intelligence".to_string(), vec![
            "artificial intelligence", "ai", "machine learning", "ml", "deep learning",
            "neural network", "neural networks", "computer vision", "nlp", "natural language processing",
            "reinforcement learning", "supervised learning", "unsupervised learning"
        ]);

        // Business topics
        topics.insert("Business".to_string(), vec![
            "business", "strategy", "market", "analysis", "financial", "planning", "management",
            "entrepreneurship", "startup", "venture capital", "investment", "revenue", "profit",
            "customer", "client", "stakeholder"
        ]);

        // Technology topics
        topics.insert("Technology".to_string(), vec![
            "technology", "software", "hardware", "programming", "coding", "development",
            "engineering", "architecture", "infrastructure", "cloud", "server", "database",
            "api", "framework", "library", "tool", "platform"
        ]);

        // Health/Medical topics
        topics.insert("Healthcare".to_string(), vec![
            "health", "medical", "patient", "doctor", "hospital", "clinic", "treatment",
            "diagnosis", "therapy", "medicine", "pharmaceutical", "clinical trial", "research"
        ]);

        // Education topics
        topics.insert("Education".to_string(), vec![
            "education", "learning", "teaching", "student", "teacher", "school", "university",
            "college", "curriculum", "course", "training", "certification", "skill"
        ]);

        // Environment topics
        topics.insert("Environment".to_string(), vec![
            "environment", "climate", "sustainable", "renewable", "energy", "carbon",
            "emission", "pollution", "conservation", "green", "ecology", "nature"
        ]);

        topics
    }
}
