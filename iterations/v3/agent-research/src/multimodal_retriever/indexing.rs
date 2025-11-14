//! Document and image indexing functionality

use anyhow::Result;
use std::path::Path;

use super::text_search::TextSearchEngine;
use super::visual_search::VisualSearchEngine;

use schemars::JsonSchema;
/// Document indexer for managing content indexing
use serde::{Deserialize, Serialize};
#[derive(Debug)]
pub struct DocumentIndexer {
    text_engine: super::text_search::TextSearchEngine,
    visual_engine: super::visual_search::VisualSearchEngine,
}

impl DocumentIndexer {
    /// Create a new document indexer
    pub fn new(
        text_engine: super::text_search::TextSearchEngine,
        visual_engine: super::visual_search::VisualSearchEngine,
    ) -> Self {
        Self {
            text_engine,
            visual_engine,
        }
    }

    /// Index a text document
    pub async fn index_text_document(&mut self, doc_id: String, content: String) -> Result<()> {
        // Index in text search engine
        self.text_engine.index_document(doc_id, content).await
    }

    /// Index an image
    pub async fn index_image(
        &mut self,
        image_path: &Path,
        metadata: super::core::VisualSearchResult,
    ) -> Result<()> {
        // Index in visual search engine
        self.visual_engine.index_image(image_path, metadata).await
    }

    /// Remove a document from index
    pub async fn remove_document(&mut self, doc_id: &str) -> Result<()> {
        // Remove from both engines
        self.text_engine.remove_document(doc_id).await?;
        self.visual_engine.remove_image(doc_id).await?;
        Ok(())
    }
}
