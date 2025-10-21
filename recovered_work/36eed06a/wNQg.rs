//! On-demand knowledge entity ingestion
//!
//! Provides async on-demand ingestion for entities not in the core vocabulary,
//! with caching, rate limiting, and idempotency guarantees.
//!
//! @author @darianrosebrook

use crate::types::*;
use crate::KnowledgeIngestor;
use anyhow::Result;
use governor::{Quota, RateLimiter};
use lru::LruCache;
use nonzero_ext::nonzero;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// On-demand ingestor with caching and rate limiting
pub struct OnDemandIngestor {
    ingestor: Arc<KnowledgeIngestor>,
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    rate_limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock>>,
}

/// Cache entry for on-demand ingestion
#[derive(Debug, Clone)]
struct CacheEntry {
    entity_id: Option<uuid::Uuid>,
    timestamp: std::time::Instant,
    ttl: std::time::Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }
}

impl OnDemandIngestor {
    /// Create a new on-demand ingestor
    pub fn new(ingestor: Arc<KnowledgeIngestor>) -> Self {
        // LRU cache with 10K capacity
        let cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(10_000).unwrap(),
        )));
        
        // Rate limiter: 10 requests per second
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(nonzero!(10u32)),
        ));
        
        Self {
            ingestor,
            cache,
            rate_limiter,
        }
    }
    
    /// Ingest entity if missing from knowledge base (idempotent)
    pub async fn ingest_if_missing(
        &self,
        source: KnowledgeSource,
        entity_key: &str,
    ) -> Result<Option<uuid::Uuid>> {
        let cache_key = format!("{}:{}", source.as_str(), entity_key);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.peek(&cache_key) {
                if !entry.is_expired() {
                    debug!("Cache hit for {}", cache_key);
                    return Ok(entry.entity_id);
                }
            }
        }
        
        // Check if already in database
        match self
            .ingestor
            .db_client()
            .kb_get_entity(source.as_str(), entity_key)
            .await?
        {
            Some(entity) => {
                // Update cache
                self.update_cache(
                    cache_key,
                    Some(entity.id.unwrap()),
                    std::time::Duration::from_secs(3600),
                )
                .await;
                return Ok(Some(entity.id.unwrap()));
            }
            None => {
                // Need to ingest
            }
        }
        
        // Apply rate limiting
        self.rate_limiter.until_ready().await;
        
        // Ingest entity
        match self.ingest_entity(source, entity_key).await {
            Ok(entity_id) => {
                // Update cache with positive result
                self.update_cache(
                    cache_key,
                    Some(entity_id),
                    std::time::Duration::from_secs(3600),
                )
                .await;
                Ok(Some(entity_id))
            }
            Err(e) => {
                warn!("Failed to ingest entity {}: {}", entity_key, e);
                // Cache negative result with short TTL
                self.update_cache(
                    cache_key,
                    None,
                    std::time::Duration::from_secs(60),
                )
                .await;
                Err(e)
            }
        }
    }
    
    /// Ingest a single entity
    async fn ingest_entity(
        &self,
        source: KnowledgeSource,
        entity_key: &str,
    ) -> Result<uuid::Uuid> {
        match source {
            KnowledgeSource::Wikidata => {
                self.ingest_wikidata_entity(entity_key).await
            }
            KnowledgeSource::WordNet => {
                self.ingest_wordnet_entity(entity_key).await
            }
        }
    }
    
    /// Ingest a Wikidata entity by ID
    async fn ingest_wikidata_entity(&self, entity_key: &str) -> Result<uuid::Uuid> {
        // In production, this would fetch from Wikidata API
        // For now, return error indicating not found
        anyhow::bail!("On-demand Wikidata ingestion not yet implemented for {}", entity_key)
    }
    
    /// Ingest a WordNet synset by ID
    async fn ingest_wordnet_entity(&self, entity_key: &str) -> Result<uuid::Uuid> {
        // In production, this would look up in WordNet data
        // For now, return error indicating not found
        anyhow::bail!("On-demand WordNet ingestion not yet implemented for {}", entity_key)
    }
    
    /// Update cache entry
    async fn update_cache(
        &self,
        key: String,
        entity_id: Option<uuid::Uuid>,
        ttl: std::time::Duration,
    ) {
        let mut cache = self.cache.write().await;
        cache.put(
            key,
            CacheEntry {
                entity_id,
                timestamp: std::time::Instant::now(),
                ttl,
            },
        );
    }
    
    /// Clear expired cache entries
    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }
    
    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        (cache.len(), cache.cap().get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry {
            entity_id: Some(uuid::Uuid::new_v4()),
            timestamp: std::time::Instant::now() - std::time::Duration::from_secs(10),
            ttl: std::time::Duration::from_secs(5),
        };
        
        assert!(entry.is_expired());
        
        let entry = CacheEntry {
            entity_id: Some(uuid::Uuid::new_v4()),
            timestamp: std::time::Instant::now(),
            ttl: std::time::Duration::from_secs(60),
        };
        
        assert!(!entry.is_expired());
    }
}

