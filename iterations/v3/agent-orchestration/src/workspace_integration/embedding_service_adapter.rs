//! Embedding Service Adapter
//!
//! Implements EmbeddingServiceTrait for agent-memory::EmbeddingIntegration
//! @author @darianrosebrook

#[cfg(feature = "memory")]
use agent_memory::embedding_integration::EmbeddingIntegration;
use system_resilience::workspace_state::EmbeddingServiceTrait;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Adapter that implements EmbeddingServiceTrait for agent-memory::EmbeddingIntegration
#[cfg(feature = "memory")]
pub struct EmbeddingServiceAdapter {
    embedding_integration: Arc<EmbeddingIntegration>,
}

#[cfg(feature = "memory")]
impl EmbeddingServiceAdapter {
    /// Create new adapter
    pub fn new(embedding_integration: Arc<EmbeddingIntegration>) -> Self {
        Self {
            embedding_integration,
        }
    }
}

#[async_trait::async_trait]
#[cfg(feature = "memory")]
impl EmbeddingServiceTrait for EmbeddingServiceAdapter {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        self.embedding_integration.generate_text_embedding(text).await
            .map_err(|e| e.to_string())
    }
    
    async fn store_file_embedding(
        &self,
        file_path: PathBuf,
        content: &str,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), String> {
        self.embedding_integration.store_file_embedding(
            &file_path,
            content,
            embedding,
            metadata,
        ).await
            .map_err(|e| e.to_string())
    }
    
    async fn search_files_by_similarity(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(PathBuf, f32)>, String> {
        self.embedding_integration.search_files_by_similarity(query, limit).await
            .map_err(|e| e.to_string())
    }
    
    async fn update_file_embedding(
        &self,
        file_path: PathBuf,
        content: &str,
        embedding: Vec<f32>,
    ) -> Result<(), String> {
        // Update is same as store (uses ON CONFLICT UPDATE)
        self.store_file_embedding(file_path, content, embedding, None).await
    }
}

/// Placeholder implementation when memory feature is disabled
#[cfg(not(feature = "memory"))]
pub struct EmbeddingServiceAdapter {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "memory"))]
impl EmbeddingServiceAdapter {
    pub fn new(_embedding_integration: ()) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
#[cfg(not(feature = "memory"))]
impl EmbeddingServiceTrait for EmbeddingServiceAdapter {
    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, String> {
        Err("Embedding service adapter requires memory feature".to_string())
    }
    
    async fn store_file_embedding(
        &self,
        _file_path: PathBuf,
        _content: &str,
        _embedding: Vec<f32>,
        _metadata: Option<serde_json::Value>,
    ) -> Result<(), String> {
        Err("Embedding service adapter requires memory feature".to_string())
    }
    
    async fn search_files_by_similarity(
        &self,
        _query: &str,
        _limit: usize,
    ) -> Result<Vec<(PathBuf, f32)>, String> {
        Err("Embedding service adapter requires memory feature".to_string())
    }
    
    async fn update_file_embedding(
        &self,
        _file_path: PathBuf,
        _content: &str,
        _embedding: Vec<f32>,
    ) -> Result<(), String> {
        Err("Embedding service adapter requires memory feature".to_string())
    }
}

