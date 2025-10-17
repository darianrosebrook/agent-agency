//! Content Processor
//!
//! Processes and cleans scraped content for research purposes.

use crate::types::*;
use crate::ContentProcessingConfig;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Content processor for cleaning and extracting text
#[derive(Debug)]
pub struct ContentProcessor {
    config: ContentProcessingConfig,
    cache: Arc<RwLock<std::collections::HashMap<String, ContentProcessingResult>>>,
}

impl ContentProcessor {
    /// Create a new content processor
    pub fn new(config: ContentProcessingConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Process content for research
    pub async fn process_content(&self, content: &str) -> Result<ContentProcessingResult> {
        info!("Processing content: {} characters", content.len());

        // Implement actual content processing with cleaning, analysis, and enhancement
        let start_time = std::time::Instant::now();

        // 1. Content cleaning: Remove HTML tags and normalize content
        let cleaned_content = self.clean_content(content);

        // 2. Text extraction: Extract meaningful text
        let extracted_text = self.extract_meaningful_text(&cleaned_content);

        // 3. Content analysis: Extract key information
        let key_phrases = self.extract_key_phrases(&extracted_text);
        let entities = self.extract_entities(&extracted_text);
        let links = self.extract_links(content);

        // 4. Generate summary
        let summary = self.generate_summary(&extracted_text);

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        let result = ContentProcessingResult {
            original_content: content.to_string(),
            processed_content: cleaned_content,
            extracted_text,
            summary,
            key_phrases,
            entities,
            links,
            processing_time_ms,
            metadata: std::collections::HashMap::new(),
        };

        info!("Content processing completed");
        Ok(result)
    }

    /// Clear processing cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Content processing cache cleared");
    }

    /// Clean content by removing HTML tags and normalizing text
    fn clean_content(&self, content: &str) -> String {
        // Remove HTML tags using regex
        let html_tag_regex = regex::Regex::new(r"<[^>]*>").unwrap();
        let cleaned = html_tag_regex.replace_all(content, "");

        // Normalize whitespace
        let whitespace_regex = regex::Regex::new(r"\s+").unwrap();
        let normalized = whitespace_regex.replace_all(&cleaned, " ");

        normalized.trim().to_string()
    }

    /// Extract meaningful text from content
    fn extract_meaningful_text(&self, content: &str) -> String {
        // Remove common noise patterns
        let noise_patterns = [
            r"^\s*\d+\s*$", // Lines with only numbers
            r"^[^\w\s]*$",  // Lines with only punctuation
            r"^\s*$",       // Empty lines
        ];

        let mut lines: Vec<&str> = content.lines().collect();
        for pattern in &noise_patterns {
            let regex = regex::Regex::new(pattern).unwrap();
            lines.retain(|line| !regex.is_match(line));
        }

        lines.join("\n").trim().to_string()
    }

    /// Extract key phrases from text
    fn extract_key_phrases(&self, text: &str) -> Vec<String> {
        // Simple key phrase extraction based on word frequency and length
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut word_counts = std::collections::HashMap::new();

        // Count word frequencies
        for word in &words {
            let lowercase_word = word.to_lowercase();
            let clean_word = lowercase_word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean_word.len() > 3 {
                *word_counts.entry(clean_word.to_string()).or_insert(0) += 1;
            }
        }

        // Extract most frequent words as key phrases
        let mut phrases: Vec<String> = word_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(word, _)| word.to_string())
            .collect();

        phrases.sort();
        phrases.truncate(10); // Limit to top 10 phrases
        phrases
    }

    /// Extract named entities from text
    fn extract_entities(&self, text: &str) -> Vec<String> {
        // Simple entity extraction based on capitalization patterns
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut entities = Vec::new();

        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean_word.len() > 2
                && clean_word
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_uppercase())
            {
                entities.push(clean_word.to_string());
            }
        }

        // Remove duplicates and limit
        entities.sort();
        entities.dedup();
        entities.truncate(15);
        entities
    }

    /// Extract links from content
    fn extract_links(&self, content: &str) -> Vec<String> {
        // Extract URLs using regex
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        let mut links: Vec<String> = url_regex
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();

        links.sort();
        links.dedup();
        links
    }

    /// Generate a summary of the text
    fn generate_summary(&self, text: &str) -> Option<String> {
        if text.len() < 100 {
            return None;
        }

        // Simple extractive summarization - take first few sentences
        let sentences: Vec<&str> = text.split('.').collect();
        let summary_sentences: Vec<&str> = sentences.iter().take(3).cloned().collect();
        let summary = summary_sentences.join(". ").trim().to_string();

        if summary.is_empty() {
            None
        } else {
            Some(summary)
        }
    }
}
