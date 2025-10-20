//! @darianrosebrook
//! BM25 full-text search indexer

use crate::types::{Bm25Stats, SearchQuery, SearchResult};
use anyhow::{Context, Result};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

/// In-memory BM25 index backed by thread-safe data structures.
pub struct Bm25Indexer {
    documents: Arc<RwLock<HashMap<Uuid, DocumentRecord>>>,
    inverted_index: Arc<RwLock<HashMap<String, HashMap<Uuid, u32>>>>,
    stats: Arc<Mutex<Bm25Stats>>,
}

#[derive(Clone)]
struct DocumentRecord {
    text: String,
    modality: String,
    term_freqs: HashMap<String, u32>,
    length: usize,
}

impl Bm25Indexer {
    /// Create a new BM25 indexer from a path
    pub fn new(index_path: &Path) -> Result<Self> {
        debug!("Initializing BM25 indexer at {:?}", index_path);

        if !index_path.exists() {
            fs::create_dir_all(index_path)
                .with_context(|| format!("Failed to create index directory at {:?}", index_path))?;
            debug!("Created BM25 index directory at {:?}", index_path);
        }

        Ok(Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            inverted_index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(Bm25Stats::default())),
        })
    }

    /// Index a block of text
    pub async fn index_block(&self, block_id: Uuid, text: &str, modality: &str) -> Result<()> {
        debug!(
            "Indexing block {} with {} chars in {}",
            block_id,
            text.len(),
            modality
        );

        let tokens = Self::tokenize(text);
        let term_freqs = Self::term_frequencies(&tokens);
        let doc_length = tokens.len();

        let mut inverted_index = self.inverted_index.write();
        let mut documents = self.documents.write();
        let mut stats = self.stats.lock();

        if let Some(existing) = documents.remove(&block_id) {
            Self::remove_document(block_id, &existing, &mut inverted_index, &mut stats);
        }

        let record = DocumentRecord {
            text: text.to_string(),
            modality: modality.to_string(),
            term_freqs: term_freqs.clone(),
            length: doc_length,
        };

        for (term, freq) in &record.term_freqs {
            inverted_index
                .entry(term.clone())
                .or_default()
                .insert(block_id, *freq);
        }

        documents.insert(block_id, record);

        stats.total_documents += 1;
        stats.total_terms += doc_length as u64;
        stats.avg_doc_length = if stats.total_documents > 0 {
            stats.total_terms as f32 / stats.total_documents as f32
        } else {
            0.0
        };

        debug!(
            "Indexed block {} (docs={}, avg_len={:.2})",
            block_id, stats.total_documents, stats.avg_doc_length
        );
        Ok(())
    }

    /// Search for text using BM25
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("BM25 search: query='{}' k={}", query.text, query.k);

        let tokens = Self::tokenize(&query.text);
        if tokens.is_empty() {
            debug!("BM25 search: no tokens derived from query");
            return Ok(Vec::new());
        }

        let documents = self.documents.read();
        if documents.is_empty() {
            debug!("BM25 search: no documents indexed");
            return Ok(Vec::new());
        }

        let inverted_index = self.inverted_index.read();
        let stats_snapshot = self.stats.lock().clone();

        let total_docs = stats_snapshot.total_documents as f32;
        if total_docs <= 0.0 {
            return Ok(Vec::new());
        }

        let avg_doc_length = if stats_snapshot.avg_doc_length > 0.0 {
            stats_snapshot.avg_doc_length
        } else {
            1.0
        };

        let mut query_terms: HashMap<String, u32> = HashMap::new();
        for token in tokens {
            *query_terms.entry(token).or_insert(0) += 1;
        }

        let mut scores: HashMap<Uuid, f32> = HashMap::new();

        for term in query_terms.keys() {
            if let Some(postings) = inverted_index.get(term) {
                let doc_freq = postings.len() as f32;
                if doc_freq == 0.0 {
                    continue;
                }

                let idf = ((total_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();

                for (doc_id, &term_freq) in postings {
                    if let Some(doc) = documents.get(doc_id) {
                        let denom = term_freq as f32
                            + stats_snapshot.k1
                                * (1.0 - stats_snapshot.b
                                    + stats_snapshot.b * (doc.length as f32 / avg_doc_length));

                        let score = idf
                            * ((term_freq as f32 * (stats_snapshot.k1 + 1.0)) / denom);

                        *scores.entry(*doc_id).or_insert(0.0) += score;
                    }
                }
            }
        }

        let mut scored_docs: Vec<(Uuid, f32)> = scores.into_iter().collect();
        scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let limit = if query.k == 0 {
            0
        } else {
            query.k.min(scored_docs.len())
        };

        let results: Vec<SearchResult> = scored_docs
            .into_iter()
            .take(limit)
            .filter_map(|(doc_id, score)| {
                documents.get(&doc_id).map(|doc| SearchResult {
                    block_id: doc_id,
                    score,
                    text_snippet: Self::build_snippet(&doc.text, &query.text, 240),
                    modality: doc.modality.clone(),
                })
            })
            .collect();

        debug!("BM25 search returned {} results", results.len());
        Ok(results)
    }

    /// Get BM25 statistics
    pub fn stats(&self) -> Bm25Stats {
        self.stats.lock().clone()
    }

    /// Commit all pending changes
    pub async fn commit(&self) -> Result<()> {
        debug!("Committing BM25 index");
        debug!("BM25 index commit (no-op for in-memory index)");
        Ok(())
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter_map(|token| {
                let normalized = token.trim().to_lowercase();
                if normalized.is_empty() {
                    None
                } else {
                    Some(normalized)
                }
            })
            .collect()
    }

    fn term_frequencies(tokens: &[String]) -> HashMap<String, u32> {
        let mut freqs = HashMap::new();
        for token in tokens {
            *freqs.entry(token.clone()).or_insert(0) += 1;
        }
        freqs
    }

    fn remove_document(
        block_id: Uuid,
        record: &DocumentRecord,
        inverted_index: &mut HashMap<String, HashMap<Uuid, u32>>,
        stats: &mut Bm25Stats,
    ) {
        for term in record.term_freqs.keys() {
            if let Some(postings) = inverted_index.get_mut(term) {
                postings.remove(&block_id);
                if postings.is_empty() {
                    inverted_index.remove(term);
                }
            }
        }

        if stats.total_documents > 0 {
            stats.total_documents -= 1;
        }

        stats.total_terms = stats.total_terms.saturating_sub(record.length as u64);

        stats.avg_doc_length = if stats.total_documents > 0 {
            stats.total_terms as f32 / stats.total_documents as f32
        } else {
            0.0
        };
    }

    fn build_snippet(text: &str, query: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            return text.to_string();
        }

        let lower_text = text.to_lowercase();
        let lower_query = query.to_lowercase();

        if let Some(pos) = lower_text.find(&lower_query) {
            let start = pos.saturating_sub(max_len / 4);
            let end = (pos + lower_query.len() + max_len / 2).min(text.len());
            let snippet = text[start..end].trim();
            return format!(
                "{}{}",
                snippet,
                if end < text.len() { "…" } else { "" }
            );
        }

        let snippet = &text[..max_len];
        format!("{}…", snippet.trim_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_bm25_indexer_creation() {
        assert_eq!(Bm25Stats::default().k1, 1.5);
        assert_eq!(Bm25Stats::default().b, 0.75);
    }

    #[tokio::test]
    async fn test_bm25_index_and_search() {
        let temp_dir = std::env::temp_dir().join(format!("bm25-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let indexer = Bm25Indexer::new(&temp_dir).expect("indexer init");
        let block_id = Uuid::new_v4();

        indexer
            .index_block(
                block_id,
                "Rust enables fearless concurrency and memory safety.",
                "text",
            )
            .await
            .expect("index block");

        let query = SearchQuery {
            text: "memory safety".to_string(),
            project_scope: None,
            k: 5,
            max_tokens: 100,
        };

        let results = indexer.search(&query).await.expect("search");
        assert!(!results.is_empty());
        assert_eq!(results[0].block_id, block_id);
        assert!(results[0].score > 0.0);
    }

    #[tokio::test]
    async fn test_bm25_reindex_updates_stats() {
        let temp_dir = std::env::temp_dir().join(format!("bm25-reindex-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let indexer = Bm25Indexer::new(&temp_dir).expect("indexer init");
        let block_id = Uuid::new_v4();

        indexer
            .index_block(block_id, "First version of the document.", "text")
            .await
            .expect("index block");

        let initial_stats = indexer.stats();
        assert_eq!(initial_stats.total_documents, 1);

        indexer
            .index_block(
                block_id,
                "Updated document content with more terms.",
                "text",
            )
            .await
            .expect("reindex block");

        let updated_stats = indexer.stats();
        assert_eq!(updated_stats.total_documents, 1);
        assert!(updated_stats.total_terms >= initial_stats.total_terms);
    }

    #[tokio::test]
    async fn test_search_query_creation() {
        let query = SearchQuery {
            text: "test query".to_string(),
            project_scope: Some("project-1".to_string()),
            k: 10,
            max_tokens: 100,
        };

        assert_eq!(query.text, "test query");
        assert_eq!(query.k, 10);
    }
}
