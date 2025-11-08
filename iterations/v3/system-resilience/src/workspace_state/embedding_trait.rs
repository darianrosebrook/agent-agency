//! Embedding Trait
//!
//! Trait for embedding functionality to avoid circular dependencies.
//! Implementations can be provided by agent-memory or other crates.

use std::path::PathBuf;

/// Trait for embedding generation and storage
#[async_trait::async_trait]
pub trait EmbeddingServiceTrait: Send + Sync + 'static {
    /// Generate embedding for text content
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String>;
    
    /// Store file embedding in block_vectors table
    async fn store_file_embedding(
        &self,
        file_path: PathBuf,
        content: &str,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), String>;
    
    /// Search files by semantic similarity
    async fn search_files_by_similarity(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(PathBuf, f32)>, String>;
    
    /// Update file embedding (called when file changes)
    async fn update_file_embedding(
        &self,
        file_path: PathBuf,
        content: &str,
        embedding: Vec<f32>,
    ) -> Result<(), String>;
}

/// Embedding service wrapper for unified manager
pub struct EmbeddingServiceWrapper {
    service: Box<dyn EmbeddingServiceTrait>,
}

impl EmbeddingServiceWrapper {
    /// Create new wrapper
    pub fn new(service: Box<dyn EmbeddingServiceTrait>) -> Self {
        Self { service }
    }
    
    /// Generate embedding for text
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        self.service.generate_embedding(text).await
    }
    
    /// Store file embedding
    pub async fn store_file_embedding(
        &self,
        file_path: PathBuf,
        content: &str,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), String> {
        self.service.store_file_embedding(file_path, content, embedding, metadata).await
    }
    
    /// Search files by similarity
    pub async fn search_files_by_similarity(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(PathBuf, f32)>, String> {
        self.service.search_files_by_similarity(query, limit).await
    }
    
    /// Update file embedding
    pub async fn update_file_embedding(
        &self,
        file_path: PathBuf,
        content: &str,
        embedding: Vec<f32>,
    ) -> Result<(), String> {
        self.service.update_file_embedding(file_path, content, embedding).await
    }
}

