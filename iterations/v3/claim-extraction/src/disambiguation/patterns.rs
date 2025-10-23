//! Entity pattern recognition using regex

use regex::Regex;
use crate::disambiguation::types::*;

/// Pattern-based entity recognition using regex
#[derive(Clone)]
pub struct EntityPatterns {
    pub person_patterns: Vec<Regex>,
    pub organization_patterns: Vec<Regex>,
    pub location_patterns: Vec<Regex>,
    pub date_patterns: Vec<Regex>,
    pub time_patterns: Vec<Regex>,
    pub money_patterns: Vec<Regex>,
    pub percent_patterns: Vec<Regex>,
    pub technical_term_patterns: Vec<Regex>,
}

impl EntityPatterns {
    /// Create new entity patterns with compiled regexes
    pub fn new() -> Self {
        Self {
            person_patterns: vec![
                Regex::new(r"\b(?:Mr\.|Ms\.|Dr\.|Prof\.)?\s*[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b").unwrap(),
                Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b").unwrap(),
            ],
            organization_patterns: vec![
                Regex::new(r"\b[A-Z][a-zA-Z\s]+(?:Inc|Corp|LLC|Ltd|Company|Co)\b").unwrap(),
                Regex::new(r"\b[A-Z][A-Z]+\b").unwrap(), // Acronyms
            ],
            location_patterns: vec![
                Regex::new(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\s+(?:City|State|Country|Street|Avenue|Road|Blvd)\b").unwrap(),
                Regex::new(r"\b(?:New York|Los Angeles|Chicago|Houston|Phoenix|Philadelphia|San Antonio|San Diego|Dallas|San Jose)\b").unwrap(),
            ],
            date_patterns: vec![
                Regex::new(r"\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},?\s+\d{4}\b").unwrap(),
                Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b").unwrap(),
                Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap(),
            ],
            time_patterns: vec![
                Regex::new(r"\b\d{1,2}:\d{2}(?::\d{2})?\s*(?:AM|PM|am|pm)?\b").unwrap(),
                Regex::new(r"\b(?:morning|afternoon|evening|night|noon|midnight)\b").unwrap(),
            ],
            money_patterns: vec![
                Regex::new(r"\$\d+(?:,\d{3})*(?:\.\d{2})?\b").unwrap(),
                Regex::new(r"\b\d+(?:,\d{3})*(?:\.\d{2})?\s*(?:dollars?|USD|cents?)\b").unwrap(),
            ],
            percent_patterns: vec![
                Regex::new(r"\b\d+(?:\.\d+)?%\b").unwrap(),
                Regex::new(r"\b\d+(?:\.\d+)?\s*percent\b").unwrap(),
            ],
            technical_term_patterns: vec![
                Regex::new(r"\b(?:API|HTTP|JSON|XML|SQL|REST|GraphQL|OAuth|JWT|CRUD|MVC|ORM|CI/CD|DevOps|SaaS|PaaS|IaaS)\b").unwrap(),
                Regex::new(r"\b(?:Docker|Kubernetes|AWS|Azure|GCP|React|Vue|Angular|Node\.js|Python|Rust|Go|Java|C\+\+)\b").unwrap(),
                Regex::new(r"\b(?:database|server|client|frontend|backend|microservice|container|deployment|repository|framework)\b").unwrap(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_patterns_creation() {
        let patterns = EntityPatterns::new();

        // Test that patterns are compiled and non-empty
        assert!(!patterns.person_patterns.is_empty());
        assert!(!patterns.organization_patterns.is_empty());
        assert!(!patterns.location_patterns.is_empty());
        assert!(!patterns.date_patterns.is_empty());
        assert!(!patterns.time_patterns.is_empty());
        assert!(!patterns.money_patterns.is_empty());
        assert!(!patterns.percent_patterns.is_empty());
        assert!(!patterns.technical_term_patterns.is_empty());
    }

    #[test]
    fn test_person_patterns() {
        let patterns = EntityPatterns::new();

        // Test person pattern matching
        let text = "John Smith is a developer.";
        let mut found = false;
        for pattern in &patterns.person_patterns {
            if pattern.is_match(text) {
                found = true;
                break;
            }
        }
        assert!(found, "Should match person names");
    }

    #[test]
    fn test_technical_term_patterns() {
        let patterns = EntityPatterns::new();

        // Test technical term pattern matching
        let text = "This API uses JSON and runs on AWS.";
        let mut found = false;
        for pattern in &patterns.technical_term_patterns {
            if pattern.is_match(text) {
                found = true;
                break;
            }
        }
        assert!(found, "Should match technical terms");
    }
}
