//! Semantic analysis and synonym generation
//!
//! This module handles semantic parsing, intent analysis, and synonym generation.

use std::collections::{HashMap, HashSet};
use anyhow::Result;

/// Semantic analyzer for meaning extraction
pub struct SemanticAnalyzer {
    // Basic synonym dictionary
    synonym_map: HashMap<String, Vec<String>>,
    // Intent keywords
    intent_keywords: HashMap<String, Vec<String>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut synonym_map = HashMap::new();
        let mut intent_keywords = HashMap::new();
        
        // Initialize synonym dictionary
        synonym_map.insert("implement".to_string(), vec![
            "create".to_string(), "build".to_string(), "develop".to_string(), 
            "construct".to_string(), "make".to_string()
        ]);
        synonym_map.insert("function".to_string(), vec![
            "method".to_string(), "procedure".to_string(), "routine".to_string(),
            "operation".to_string()
        ]);
        synonym_map.insert("test".to_string(), vec![
            "verify".to_string(), "check".to_string(), "validate".to_string(),
            "examine".to_string()
        ]);
        synonym_map.insert("error".to_string(), vec![
            "bug".to_string(), "issue".to_string(), "problem".to_string(),
            "fault".to_string()
        ]);
        synonym_map.insert("fix".to_string(), vec![
            "repair".to_string(), "resolve".to_string(), "correct".to_string(),
            "debug".to_string()
        ]);
        
        // Initialize intent keywords
        intent_keywords.insert("implementation".to_string(), vec![
            "implement".to_string(), "create".to_string(), "build".to_string(),
            "develop".to_string(), "code".to_string()
        ]);
        intent_keywords.insert("testing".to_string(), vec![
            "test".to_string(), "verify".to_string(), "validate".to_string(),
            "check".to_string()
        ]);
        intent_keywords.insert("debugging".to_string(), vec![
            "debug".to_string(), "fix".to_string(), "repair".to_string(),
            "resolve".to_string()
        ]);
        intent_keywords.insert("optimization".to_string(), vec![
            "optimize".to_string(), "improve".to_string(), "enhance".to_string(),
            "performance".to_string()
        ]);
        
        Self {
            synonym_map,
            intent_keywords,
        }
    }

    /// Analyze semantic content and intent
    pub async fn analyze_semantics(&self, text: &str) -> Result<SemanticAnalysis> {
        let words = self.tokenize(text);
        let intent = self.detect_intent(&words);
        let synonyms = self.extract_synonyms(&words);
        let semantic_score = self.calculate_semantic_score(&words, &intent);
        
        Ok(SemanticAnalysis {
            intent,
            synonyms,
            semantic_score,
        })
    }

    /// Generate synonyms for a term
    pub fn generate_synonyms(&self, term: &str) -> Vec<String> {
        let normalized_term = term.to_lowercase();
        self.synonym_map.get(&normalized_term)
            .cloned()
            .unwrap_or_else(|| {
                // Generate basic synonyms using word variations
                let mut synonyms = Vec::new();
                
                // Add plural/singular variations
                if normalized_term.ends_with('s') {
                    synonyms.push(normalized_term.trim_end_matches('s').to_string());
                } else {
                    synonyms.push(format!("{}s", normalized_term));
                }
                
                // Add common variations
                if normalized_term.contains("ing") {
                    let base = normalized_term.replace("ing", "");
                    synonyms.push(format!("{}ed", base));
                    synonyms.push(format!("{}s", base));
                }
                
                synonyms
            })
    }

    /// Tokenize text into words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| {
                // Remove punctuation and convert to lowercase
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// Detect intent from words
    fn detect_intent(&self, words: &[String]) -> String {
        let mut intent_scores = HashMap::new();
        
        for (intent, keywords) in &self.intent_keywords {
            let mut score = 0.0;
            for keyword in keywords {
                if words.contains(keyword) {
                    score += 1.0;
                }
            }
            if score > 0.0 {
                intent_scores.insert(intent.clone(), score);
            }
        }
        
        // Find intent with highest score
        intent_scores.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(intent, _)| intent)
            .unwrap_or_else(|| "general".to_string())
    }

    /// Extract synonyms from words
    fn extract_synonyms(&self, words: &[String]) -> Vec<String> {
        let mut synonyms = HashSet::new();
        
        for word in words {
            if let Some(word_synonyms) = self.synonym_map.get(word) {
                synonyms.extend(word_synonyms.iter().cloned());
            }
        }
        
        synonyms.into_iter().collect()
    }

    /// Calculate semantic score based on word complexity and intent confidence
    fn calculate_semantic_score(&self, words: &[String], intent: &str) -> f64 {
        let word_count = words.len();
        let unique_words = words.iter().collect::<HashSet<_>>().len();
        
        // Calculate lexical diversity
        let lexical_diversity = if word_count > 0 {
            unique_words as f64 / word_count as f64
        } else {
            0.0
        };
        
        // Calculate intent confidence
        let intent_confidence = if intent != "general" {
            let intent_keywords = self.intent_keywords.get(intent)
                .map(|keywords| keywords.len())
                .unwrap_or(0);
            if intent_keywords > 0 {
                let matched_keywords = words.iter()
                    .filter(|word| {
                        self.intent_keywords.get(intent)
                            .map(|keywords| keywords.contains(word))
                            .unwrap_or(false)
                    })
                    .count();
                matched_keywords as f64 / intent_keywords as f64
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        // Combine scores (weighted average)
        (lexical_diversity * 0.4 + intent_confidence * 0.6).min(1.0).max(0.0)
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic analysis result
pub struct SemanticAnalysis {
    pub intent: String,
    pub synonyms: Vec<String>,
    pub semantic_score: f64,
}
