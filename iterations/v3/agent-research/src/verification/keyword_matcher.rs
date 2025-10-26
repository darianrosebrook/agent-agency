//! Keyword matching and search functionality
//!
//! This module handles exact/fuzzy/context matching and relevance scoring.

use std::collections::HashMap;
use regex::Regex;
use anyhow::Result;

use crate::verification::types::*;

/// Keyword matcher for text search
#[derive(Debug)]
pub struct KeywordMatcher;

impl KeywordMatcher {
    /// Search for keywords in content
    pub async fn search_keywords_in_content(&self, content: &str, keywords: &[String]) -> Result<Vec<KeywordMatch>> {
        let mut matches = Vec::new();

        for keyword in keywords {
            // Find all occurrences
            if let Ok(regex) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(keyword))) {
                for capture in regex.find_iter(content) {
                    let line_start = content[..capture.start()].rfind('\n').unwrap_or(0);
                    let line_end = content[capture.end()..].find('\n')
                        .map(|i| capture.end() + i)
                        .unwrap_or(content.len());
                    let line_content = &content[line_start..line_end];

                    let line_number = content[..capture.start()].chars()
                        .filter(|&c| c == '\n')
                        .count() + 1;

                    matches.push(KeywordMatch {
                        keyword: keyword.clone(),
                        file_path: "unknown".to_string(), // TODO: pass file path
                        line_number,
                        context: line_content.to_string(),
                        match_type: MatchType::Exact,
                        relevance_score: self.calculate_relevance(keyword, line_content),
                    });
                }
            }
        }

        Ok(matches)
    }

    /// Find exact matches
    pub fn find_exact_matches(&self, content: &str, keywords: &[String]) -> Vec<String> {
        keywords.iter()
            .filter(|k| content.to_lowercase().contains(&k.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Find fuzzy matches
    pub fn find_fuzzy_matches(&self, content: &str, keywords: &[String]) -> Vec<String> {
        let mut matches = Vec::new();
        let content_lower = content.to_lowercase();

        for keyword in keywords {
            // Simple fuzzy matching: contains with some tolerance
            if content_lower.contains(&keyword.to_lowercase()) {
                matches.push(keyword.clone());
            } else {
                // Check for close matches (missing one character, etc.)
                for i in 0..keyword.len() {
                    let modified = format!("{}{}", &keyword[..i], &keyword[i+1..]);
                    if content_lower.contains(&modified.to_lowercase()) {
                        matches.push(keyword.clone());
                        break;
                    }
                }
            }
        }

        matches
    }

    /// Calculate relevance score for a keyword in context
    fn calculate_relevance(&self, keyword: &str, context: &str) -> f64 {
        let context_lower = context.to_lowercase();
        let keyword_lower = keyword.to_lowercase();

        let mut score: f64 = 0.5; // base score

        // Boost if keyword appears at start of line or sentence
        if context_lower.trim_start().starts_with(&keyword_lower) {
            score += 0.3;
        }

        // Boost if keyword appears near technical terms
        let technical_indicators = ["function", "class", "method", "api", "system", "component"];
        for indicator in technical_indicators {
            if context_lower.contains(indicator) &&
               context_lower.find(indicator).unwrap_or(1000) < context_lower.find(&keyword_lower).unwrap_or(0) + 50 {
                score += 0.1;
                break;
            }
        }

        // Boost if in section headers
        if context_lower.contains('#') && context_lower.contains(&keyword_lower) {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// Analyze keyword relevance in content
    pub async fn analyze_keyword_relevance(&self, content: &str, matches: &[KeywordMatch]) -> Result<(f64, f64)> {
        if matches.is_empty() {
            return Ok((0.0, 0.0));
        }

        let total_relevance: f64 = matches.iter().map(|m| m.relevance_score).sum();
        let avg_relevance = total_relevance / matches.len() as f64;

        // Coverage score: how much of the content is relevant
        let coverage = (matches.len() as f64 * 10.0 / content.len() as f64).min(1.0);

        Ok((avg_relevance, coverage))
    }
}
