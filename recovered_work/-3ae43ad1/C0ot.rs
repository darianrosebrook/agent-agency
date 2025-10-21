//! Core vocabulary selection and loading
//!
//! Defines and loads core vocabulary (high-frequency entities) that should be
//! pre-loaded into the knowledge base for optimal performance.
//!
//! @author @darianrosebrook

/// Core vocabulary list (high-frequency English terms)
/// This is a curated list of ~100 most common technical and general terms
/// In production, this would be expanded to ~10K based on frequency analysis
pub const CORE_VOCABULARY: &[&str] = &[
    // Technical terms
    "api", "database", "server", "client", "application", "system", "service",
    "function", "method", "class", "interface", "type", "variable", "parameter",
    "request", "response", "query", "result", "data", "information", "code",
    "program", "software", "hardware", "network", "protocol", "http", "json",
    "xml", "sql", "rest", "graphql", "authentication", "authorization",
    "security", "encryption", "token", "session", "user", "account", "password",
    "configuration", "deployment", "testing", "debugging", "logging", "monitoring",
    "performance", "optimization", "scalability", "reliability", "availability",
    
    // General high-frequency nouns
    "person", "people", "time", "year", "day", "way", "thing", "man", "woman",
    "child", "world", "life", "hand", "part", "place", "case", "point", "group",
    "problem", "fact", "company", "number", "area", "question", "work", "government",
    "money", "business", "process", "system", "level", "form", "office", "door",
    "health", "power", "ability", "fact", "reason", "result", "change", "kind",
    
    // Common verbs
    "make", "get", "know", "think", "take", "see", "come", "want", "use", "find",
    "give", "tell", "work", "call", "try", "feel", "become", "leave", "put", "mean",
    "keep", "let", "begin", "seem", "help", "show", "hear", "play", "run", "move",
    "live", "believe", "bring", "happen", "write", "provide", "sit", "stand", "lose",
    "pay", "meet", "include", "continue", "set", "learn", "change", "lead", "understand",
];

/// Check if a term is in the core vocabulary
pub fn is_core_vocabulary(term: &str) -> bool {
    let normalized = term.to_lowercase();
    CORE_VOCABULARY.iter().any(|&v| v == normalized)
}

/// Get core vocabulary terms for a specific domain
pub fn get_domain_vocabulary(domain: &str) -> Vec<&'static str> {
    match domain.to_lowercase().as_str() {
        "software" | "technology" => {
            CORE_VOCABULARY.iter()
                .filter(|&&term| is_technical_term(term))
                .copied()
                .collect()
        }
        "general" => {
            CORE_VOCABULARY.to_vec()
        }
        _ => {
            CORE_VOCABULARY.to_vec()
        }
    }
}

/// Check if a term is technical
fn is_technical_term(term: &str) -> bool {
    const TECHNICAL_TERMS: &[&str] = &[
        "api", "database", "server", "client", "application", "system", "service",
        "function", "method", "class", "interface", "type", "variable", "parameter",
        "request", "response", "query", "result", "data", "code", "program",
        "software", "hardware", "network", "protocol", "http", "json", "xml", "sql",
        "rest", "graphql", "authentication", "authorization", "security", "encryption",
        "token", "session", "configuration", "deployment", "testing", "debugging",
        "logging", "monitoring", "performance", "optimization", "scalability",
        "reliability", "availability",
    ];
    
    TECHNICAL_TERMS.contains(&term)
}

/// Priority score for vocabulary terms (higher = more important)
pub fn get_priority_score(term: &str) -> f64 {
    if is_technical_term(term) {
        1.0
    } else {
        0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_core_vocabulary() {
        assert!(is_core_vocabulary("database"));
        assert!(is_core_vocabulary("Database"));
        assert!(is_core_vocabulary("API"));
        assert!(!is_core_vocabulary("nonexistent"));
    }

    #[test]
    fn test_get_domain_vocabulary() {
        let tech_vocab = get_domain_vocabulary("software");
        assert!(tech_vocab.contains(&"api"));
        assert!(tech_vocab.contains(&"database"));
        
        let general_vocab = get_domain_vocabulary("general");
        assert!(general_vocab.len() > tech_vocab.len());
    }

    #[test]
    fn test_is_technical_term() {
        assert!(is_technical_term("api"));
        assert!(is_technical_term("database"));
        assert!(!is_technical_term("person"));
        assert!(!is_technical_term("world"));
    }

    #[test]
    fn test_get_priority_score() {
        assert_eq!(get_priority_score("api"), 1.0);
        assert_eq!(get_priority_score("person"), 0.5);
    }
}

