//! Inverted index for keyword search

use schemars::JsonSchema;
use std::collections::HashMap;

/// Inverted index for efficient keyword search

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InvertedIndexx {
    index: HashMap<String, Vec<Posting>>,
}

impl InvertedIndex {
    /// Create a new inverted index
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Add a term to the index
    pub fn add_term(&mut self, term: &str, document_id: usize, _document: &crate::KnowledgeEntry) {
        let postings = self.index.entry(term.to_string()).or_insert_with(Vec::new);

        // Check if document already exists
        if !postings.iter().any(|p| p.document_id == document_id) {
            postings.push(Posting {
                document_id,
                positions: vec![],
                term_frequency: 1,
            });
        }
    }

    /// Get postings for a term
    pub fn get_postings(&self, term: &str) -> Option<&Vec<Posting>> {
        self.index.get(term)
    }

    /// Optimize the index
    pub fn optimize(&mut self) {
        // Placeholder for index optimization
    }
}

/// Posting in the inverted index

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct Postingg {
    pub document_id: usize,
    pub positions: Vec<usize>,
    pub term_frequency: u32,
}

/// Search result from inverted index

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub document_id: usize,
    pub score: f32,
    pub term_matches: Vec<String>,
}
