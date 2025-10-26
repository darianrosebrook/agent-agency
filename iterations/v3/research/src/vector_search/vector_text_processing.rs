//! Text Processing for Vector Search
//!
//! Handles text preprocessing, normalization, and tokenization for embedding generation.

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;

/// Text processor for vector search operations
pub struct TextProcessor;

impl TextProcessor {
    /// Create a new text processor
    pub fn new() -> Self {
        Self
    }

    /// Preprocess text for general search operations
    pub fn preprocess_text(&self, text: &str) -> String {
        // Clean and normalize text
        let cleaned = text.trim().to_lowercase();

        // Remove extra whitespace
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        let normalized = whitespace_regex.replace_all(&cleaned, " ");

        // Truncate if too long (most embedding models have limits)
        if normalized.len() > 512 {
            format!("{}...", &normalized[..512])
        } else {
            normalized.to_string()
        }
    }

    /// Preprocess text specifically for embedding generation
    pub fn preprocess_text_for_embedding(&self, text: &str) -> Result<String> {
        // Clean and normalize text
        let cleaned_text = text
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();

        // Tokenize and limit length
        let tokens: Vec<&str> = cleaned_text.split_whitespace().collect();
        let limited_tokens = if tokens.len() > 512 {
            &tokens[..512]
        } else {
            &tokens
        };

        Ok(limited_tokens.join(" "))
    }

    /// Create a cache key from text
    pub fn create_cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Hash text for consistent cache keys
    pub fn hash_text(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// Extract keywords from text
    pub fn extract_keywords(&self, text: &str, max_keywords: usize) -> Vec<String> {
        let words: Vec<String> = text
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| word.len() > 2)
            .map(|word| word.to_lowercase())
            .collect();

        let mut word_counts = std::collections::HashMap::new();
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }

        let mut keyword_vec: Vec<(String, usize)> = word_counts.into_iter().collect();
        keyword_vec.sort_by(|a, b| b.1.cmp(&a.1));

        keyword_vec
            .into_iter()
            .take(max_keywords)
            .map(|(word, _)| word)
            .collect()
    }

    /// Remove common stop words
    pub fn remove_stop_words(&self, text: &str) -> String {
        let stop_words: HashSet<&str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
            "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "must", "can", "shall",
            "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they", "me",
            "him", "her", "us", "them", "my", "your", "his", "its", "our", "their"
        ].into();

        text.split_whitespace()
            .filter(|word| !stop_words.contains(&word.to_lowercase().as_str()))
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Normalize text for consistent processing
    pub fn normalize_text(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new()
    }
}
