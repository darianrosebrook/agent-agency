//! Vector Store
//!
//! Semantic similarity search using vector embeddings.
//! Note: In a real implementation, this would use a proper embedding model.
//! This implementation uses a simple TF-IDF-like approach for demonstration.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::decay::DecayCategory;

/// A vector entry in the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub content: String,
    pub content_hash: String,
    /// Normalized term frequencies (simple embedding representation)
    pub embedding: Vec<f64>,
    pub metadata: HashMap<String, String>,
    pub decay_category: DecayCategory,
    pub initial_weight: f64,
    pub created_at: DateTime<Utc>,
}

impl VectorEntry {
    pub fn new(id: impl Into<String>, content: impl Into<String>, embedding: Vec<f64>) -> Self {
        let content = content.into();
        let content_hash = compute_hash(&content);

        Self {
            id: id.into(),
            content,
            content_hash,
            embedding,
            metadata: HashMap::new(),
            decay_category: DecayCategory::fresh(),
            initial_weight: 1.0,
            created_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.initial_weight = weight;
        self
    }
}

/// Simple vector store for semantic similarity search
pub struct VectorStore {
    entries: HashMap<String, VectorEntry>,
    /// Vocabulary for embedding generation
    vocabulary: HashMap<String, usize>,
    /// Inverse document frequency for each term
    idf: HashMap<String, f64>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            vocabulary: HashMap::new(),
            idf: HashMap::new(),
        }
    }

    /// Add an entry to the store
    pub fn add(&mut self, mut entry: VectorEntry) {
        // Rebuild embedding if needed
        if entry.embedding.is_empty() {
            entry.embedding = self.create_embedding(&entry.content);
        }

        // Update vocabulary and IDF
        self.update_vocabulary(&entry.content);

        self.entries.insert(entry.id.clone(), entry);
    }

    /// Create an entry from content (generates embedding)
    pub fn create_entry(&self, id: impl Into<String>, content: impl Into<String>) -> VectorEntry {
        let content = content.into();
        let embedding = self.create_embedding(&content);
        VectorEntry::new(id, content, embedding)
    }

    /// Search for similar entries
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_embedding = self.create_embedding(query);

        let mut results: Vec<SearchResult> = self
            .entries
            .values()
            .map(|entry| {
                let similarity = cosine_similarity(&query_embedding, &entry.embedding);
                SearchResult {
                    entry: entry.clone(),
                    similarity,
                }
            })
            .collect();

        // Sort by similarity descending
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(limit);
        results
    }

    /// Get an entry by ID
    pub fn get(&self, id: &str) -> Option<&VectorEntry> {
        self.entries.get(id)
    }

    /// Get a mutable entry by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut VectorEntry> {
        self.entries.get_mut(id)
    }

    /// Remove an entry
    pub fn remove(&mut self, id: &str) -> Option<VectorEntry> {
        self.entries.remove(id)
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries
    pub fn all_entries(&self) -> Vec<&VectorEntry> {
        self.entries.values().collect()
    }

    /// Create a simple TF-IDF-like embedding
    fn create_embedding(&self, text: &str) -> Vec<f64> {
        let tokens = tokenize(text);
        let total_tokens = tokens.len() as f64;

        if total_tokens == 0.0 {
            return vec![0.0; self.vocabulary.len().max(1)];
        }

        // Count term frequencies
        let mut tf: HashMap<&str, f64> = HashMap::new();
        for token in &tokens {
            *tf.entry(token.as_str()).or_default() += 1.0;
        }

        // Build embedding vector
        let vocab_size = self.vocabulary.len().max(1);
        let mut embedding = vec![0.0; vocab_size];

        for (term, count) in tf {
            if let Some(&idx) = self.vocabulary.get(term) {
                let term_freq = count / total_tokens;
                let idf = self.idf.get(term).copied().unwrap_or(1.0);
                if idx < embedding.len() {
                    embedding[idx] = term_freq * idf;
                }
            }
        }

        // Normalize
        let norm: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        embedding
    }

    /// Update vocabulary and IDF with new content
    fn update_vocabulary(&mut self, text: &str) {
        let tokens = tokenize(text);

        for token in tokens {
            if !self.vocabulary.contains_key(&token) {
                let idx = self.vocabulary.len();
                self.vocabulary.insert(token.clone(), idx);
            }
        }

        // Recalculate IDF
        let n = self.entries.len() as f64 + 1.0;
        for term in self.vocabulary.keys() {
            let docs_with_term = self
                .entries
                .values()
                .filter(|e| e.content.to_lowercase().contains(&term.to_lowercase()))
                .count() as f64
                + 1.0;
            self.idf.insert(term.clone(), (n / docs_with_term).ln() + 1.0);
        }
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: VectorEntry,
    pub similarity: f64,
}

/// Tokenize text into lowercase words
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() > 1)
        .map(String::from)
        .collect()
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        // Pad shorter vector with zeros
        let max_len = a.len().max(b.len());
        let a_padded: Vec<f64> = a.iter().cloned().chain(std::iter::repeat(0.0)).take(max_len).collect();
        let b_padded: Vec<f64> = b.iter().cloned().chain(std::iter::repeat(0.0)).take(max_len).collect();
        return cosine_similarity_impl(&a_padded, &b_padded);
    }
    cosine_similarity_impl(a, b)
}

fn cosine_similarity_impl(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

/// Compute SHA-256 hash
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_entry_creation() {
        let entry = VectorEntry::new("test", "Test content", vec![0.5, 0.5]);
        assert_eq!(entry.id, "test");
        assert!(!entry.content_hash.is_empty());
    }

    #[test]
    fn test_vector_store_add() {
        let mut store = VectorStore::new();
        let entry = store.create_entry("e1", "Hello world");
        store.add(entry);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_vector_store_search() {
        let mut store = VectorStore::new();

        // Use more distinctive content
        store.add(store.create_entry("e1", "Rust programming language systems code"));
        store.add(store.create_entry("e2", "Python programming language scripting"));
        store.add(store.create_entry("e3", "Cooking recipes kitchen food chef"));

        let results = store.search("rust programming", 3);
        assert!(!results.is_empty());

        // Programming entries should have higher similarity to "rust programming" than cooking
        let cooking_result = results.iter().find(|r| r.entry.id == "e3");
        let programming_results: Vec<_> = results.iter().filter(|r| r.entry.id != "e3").collect();

        if let Some(cooking) = cooking_result {
            // At least one programming result should have higher similarity
            let any_higher = programming_results.iter().any(|r| r.similarity > cooking.similarity);
            assert!(any_higher || programming_results.is_empty());
        }
    }

    #[test]
    fn test_cosine_similarity() {
        // Same vector should have similarity 1.0
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        // Orthogonal vectors should have similarity 0.0
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 0.001);

        // Opposite vectors should have similarity -1.0
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello, World! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        // Single letter words should be filtered
        assert!(!tokens.contains(&"a".to_string()));
    }

    #[test]
    fn test_entry_with_metadata() {
        let entry = VectorEntry::new("test", "Content", vec![])
            .with_metadata("source", "file.rs")
            .with_weight(0.8);

        assert_eq!(entry.metadata.get("source"), Some(&"file.rs".to_string()));
        assert_eq!(entry.initial_weight, 0.8);
    }

    #[test]
    fn test_vector_store_get_remove() {
        let mut store = VectorStore::new();
        store.add(store.create_entry("e1", "Test content"));

        assert!(store.get("e1").is_some());
        assert!(store.get("nonexistent").is_none());

        let removed = store.remove("e1");
        assert!(removed.is_some());
        assert!(store.get("e1").is_none());
    }
}
