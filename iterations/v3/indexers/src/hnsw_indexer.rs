//! @darianrosebrook
//! HNSW (Hierarchical Navigable Small World) approximate nearest neighbor search

use crate::types::{VectorQuery, VectorSearchResult, HnswMetadata};
use anyhow::{Context, Result};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, info, warn};
use hnsw_rs::prelude::*;
use std::collections::HashMap;
use parking_lot::Mutex;

pub struct HnswIndexer {
    index: Arc<Hnsw<f32, DistCosine>>,
    metadata: HnswMetadata,
    id_to_uuid: Arc<Mutex<HashMap<usize, Uuid>>>,
    uuid_to_id: Arc<Mutex<HashMap<Uuid, usize>>>,
    next_id: Arc<Mutex<usize>>,
}

impl HnswIndexer {
    /// Create a new HNSW indexer for a specific model
    pub fn new(metadata: HnswMetadata) -> Result<Self> {
        debug!(
            "HNSW indexer initialized for model {} (dim={}, metric={})",
            metadata.model_id, metadata.dim, metadata.metric
        );

        // Create HNSW index with cosine similarity (most common for embeddings)
        let hnsw_params = HnswParams::<DistCosine>::new()
            .with_max_nb_connection(metadata.max_neighbors)
            .with_ef_construction(metadata.ef_construction)
            .with_ef_search(metadata.ef_search);
        
        let index = Arc::new(Hnsw::<f32, DistCosine>::new(
            metadata.dim,
            &hnsw_params,
        ).context("Failed to create HNSW index")?);

        Ok(Self {
            index,
            metadata,
            id_to_uuid: Arc::new(Mutex::new(HashMap::new())),
            uuid_to_id: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        })
    }

    /// Insert a vector into the index
    pub async fn insert(
        &mut self,
        block_id: Uuid,
        vector: &[f32],
    ) -> Result<()> {
        debug!(
            "Inserting vector for block {} ({}d)",
            block_id,
            vector.len()
        );

        // Validate vector dimensions
        if vector.len() != self.metadata.dim as usize {
            return Err(anyhow::anyhow!(
                "Vector dimension mismatch: expected {}, got {}",
                self.metadata.dim,
                vector.len()
            ));
        }

        // Get next available ID
        let id = {
            let mut next_id = self.next_id.lock();
            let current_id = *next_id;
            *next_id += 1;
            current_id
        };

        // Insert into HNSW index
        self.index
            .insert(vector, id)
            .context("Failed to insert vector into HNSW index")?;

        // Update ID mappings
        {
            let mut id_to_uuid = self.id_to_uuid.lock();
            let mut uuid_to_id = self.uuid_to_id.lock();
            
            id_to_uuid.insert(id, block_id);
            uuid_to_id.insert(block_id, id);
        }

        // Update metadata
        self.metadata.node_count += 1;

        debug!("Successfully inserted vector for block {} with ID {}", block_id, id);
        Ok(())
    }

    /// Search for nearest neighbors
    pub async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorSearchResult>> {
        debug!(
            "HNSW search: model_id={} k={} vec_dim={}",
            query.model_id,
            query.k,
            query.vector.len()
        );

        // Validate query vector dimensions
        if query.vector.len() != self.metadata.dim as usize {
            return Err(anyhow::anyhow!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.metadata.dim,
                query.vector.len()
            ));
        }

        // Perform HNSW search
        let neighbors = self.index
            .search(query.vector, query.k)
            .context("Failed to perform HNSW search")?;

        // Convert results to VectorSearchResult
        let results = neighbors
            .into_iter()
            .filter_map(|(id, distance)| {
                // Get UUID from ID mapping
                let id_to_uuid = self.id_to_uuid.lock();
                let block_id = id_to_uuid.get(&id)?;
                
                // Convert distance to similarity score
                let similarity = match self.metadata.metric.as_str() {
                    "cosine" => 1.0 - distance,  // Cosine distance to similarity
                    "l2" => 1.0 / (1.0 + distance),  // L2 distance to similarity
                    "ip" => distance,  // Inner product is already similarity-like
                    _ => 1.0 - distance,  // Default to cosine-like conversion
                };

                Some(VectorSearchResult {
                    block_id: *block_id,
                    similarity,
                    modality: self.metadata.modality.clone(),
                })
            })
            .collect();

        debug!("HNSW search returned {} results", results.len());
        Ok(results)
    }

    /// Get metadata
    pub fn metadata(&self) -> &HnswMetadata {
        &self.metadata
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.metadata.node_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_metadata_creation() {
        let metadata = HnswMetadata {
            model_id: "clip-vit-b32".to_string(),
            modality: "image".to_string(),
            dim: 512,
            metric: "cosine".to_string(),
            max_neighbors: 16,
            ef_construction: 200,
            ef_search: 100,
            node_count: 0,
        };

        assert_eq!(metadata.dim, 512);
        assert_eq!(metadata.metric, "cosine");
    }

    #[tokio::test]
    async fn test_hnsw_indexer_init() {
        let metadata = HnswMetadata {
            model_id: "e5-small-v2".to_string(),
            modality: "text".to_string(),
            dim: 1536,
            metric: "cosine".to_string(),
            max_neighbors: 16,
            ef_construction: 200,
            ef_search: 100,
            node_count: 0,
        };

        let indexer = HnswIndexer::new(metadata);
        assert_eq!(indexer.node_count(), 0);
    }
}

