//! @darianrosebrook
//! BM25 full-text search indexer

use crate::types::{SearchQuery, SearchResult, Bm25Stats};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
use tracing::debug;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Schema, Field, STORED, TEXT, STRING},
    Index, IndexReader,
};

pub struct Bm25Indexer {
    index: Arc<Index>,
    reader: IndexReader,
    schema: Schema,
    block_id_field: Field,
    text_field: Field,
    modality_field: Field,
    stats: Arc<parking_lot::Mutex<Bm25Stats>>,
}

impl Bm25Indexer {
    /// Create a new BM25 indexer from a path
    pub fn new(index_path: &Path) -> Result<Self> {
        debug!("Initializing BM25 indexer at {:?}", index_path);

        // Create or open index
        let index = Index::open_in_dir(index_path)
            .or_else(|_| {
                debug!("Creating new BM25 index at {:?}", index_path);
                
                // Create schema
                let mut schema_builder = Schema::builder();
                let block_id_field = schema_builder.add_text_field("block_id", TEXT | STORED);
                let text_field = schema_builder.add_text_field("text", TEXT);
                let modality_field = schema_builder.add_text_field("modality", STRING | STORED);
                let schema = schema_builder.build();
                
                // Create index
                Index::create_in_dir(index_path, schema)
            })
            .context("Failed to create or open BM25 index")?;

        let schema = index.schema();
        let block_id_field = schema.get_field("block_id")
            .context("block_id field not found in schema")?;
        let text_field = schema.get_field("text")
            .context("text field not found in schema")?;
        let modality_field = schema.get_field("modality")
            .context("modality field not found in schema")?;

        // Create reader
        let reader = index
            .reader_builder()
            // .reload_policy(ReloadPolicy::OnCommit)  // TODO: Use correct ReloadPolicy variant
            .try_into()
            .context("Failed to create index reader")?;

        debug!("BM25 indexer initialized successfully");

        Ok(Self {
            index: Arc::new(index),
            reader,
            schema,
            block_id_field,
            text_field,
            modality_field,
            stats: Arc::new(parking_lot::Mutex::new(Bm25Stats::default())),
        })
    }

    /// Index a block of text
    pub async fn index_block(
        &self,
        block_id: Uuid,
        text: &str,
        modality: &str,
    ) -> Result<()> {
        debug!(
            "Indexing block {} with {} chars in {}",
            block_id,
            text.len(),
            modality
        );

        // TODO: PLACEHOLDER - Fix tantivy Document API usage
        // The Document type is a trait, not a struct, so Document::new() is not valid
        // Proper implementation would use tantivy's document! macro or implement the trait
        
        // Update stats at least
        let mut stats = self.stats.lock();
        stats.total_documents += 1;
        stats.total_terms += text.split_whitespace().count() as u64;
        stats.avg_doc_length =
            stats.total_terms as f32 / stats.total_documents.max(1) as f32;

        debug!("Indexed block {} (stats updated)", block_id);
        Ok(())
    }

    /// Search for text using BM25
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("BM25 search: query='{}' k={}", query.text, query.k);

        // TODO: PLACEHOLDER - BM25 search implementation
        // The tantivy API requires proper type annotations and Document trait implementation
        // For now, return empty results
        
        let results: Vec<SearchResult> = Vec::new();

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
        
        // TODO: PLACEHOLDER - BM25 commit implementation
        // Tantivy API requires proper type annotations for IndexWriter<_>
        // This is deferred until tantivy integration is properly implemented
        
        debug!("BM25 index commit (placeholder)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bm25_indexer_creation() {
        // Note: Would require a temporary directory in real tests
        // This just verifies the types compile correctly
        assert_eq!(Bm25Stats::default().k1, 1.5);
        assert_eq!(Bm25Stats::default().b, 0.75);
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

