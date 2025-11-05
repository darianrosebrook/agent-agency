//! Port traits for research operations
//!
//! Uses BoxFuture instead of async_trait to keep contracts macro-free
//! and object-safe. All traits support Arc<dyn Trait> usage.

use std::future::Future;
use std::pin::Pin;

use super::dto::{EntityKey, Embedding};
use super::errors::{EmbeddingError, KnowledgeError};

/// Boxed future type alias for object-safe async traits
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for embedding providers - unified interface
/// 
/// Object-safe: supports Arc<dyn EmbeddingProvider>
pub trait EmbeddingProvider: Send + Sync {
    /// Embed a single text string
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Embedding, EmbeddingError>>;
    
    /// Embed multiple texts in batch (prevents N+1)
    fn embed_many<'a>(&'a self, texts: &'a [String]) -> BoxFuture<'a, Result<Vec<Embedding>, EmbeddingError>>;
}

/// Trait for knowledge base operations
pub trait KnowledgeBase: Send + Sync {
    /// Lookup a single entity by key
    fn lookup<'a>(&'a self, key: &'a EntityKey) -> BoxFuture<'a, Result<Option<String>, KnowledgeError>>;
    
    /// Search for entities matching query
    fn search<'a>(&'a self, query: &'a str, limit: usize) -> BoxFuture<'a, Result<Vec<String>, KnowledgeError>>;
    
    /// Batch lookup multiple entities (prevents N+1)
    fn batch_lookup<'a>(&'a self, keys: &'a [EntityKey]) -> BoxFuture<'a, Result<Vec<Option<String>>, KnowledgeError>>;
}

/// Trait for knowledge ingestion
pub trait KnowledgeIngest: Send + Sync {
    /// Ingest content into knowledge base
    fn ingest<'a>(&'a self, content: &'a str) -> BoxFuture<'a, Result<(), KnowledgeError>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_embedding_provider_object_safety() {
        // Dummy implementation for testing
        struct TestProvider;
        
        impl EmbeddingProvider for TestProvider {
            fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Embedding, EmbeddingError>> {
                Box::pin(async move {
                    Ok(Embedding(vec![0.0; 128]))
                })
            }
            
            fn embed_many<'a>(&'a self, _texts: &'a [String]) -> BoxFuture<'a, Result<Vec<Embedding>, EmbeddingError>> {
                Box::pin(async move {
                    Ok(vec![])
                })
            }
        }

        // Test that trait object works
        let provider: Arc<dyn EmbeddingProvider> = Arc::new(TestProvider);
        assert!(Arc::strong_count(&provider) == 1);
    }
}

