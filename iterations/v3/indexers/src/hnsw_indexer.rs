//! @darianrosebrook
//! HNSW (Hierarchical Navigable Small World) approximate nearest neighbor search

use crate::types::{HnswMetadata, VectorQuery, VectorSearchResult};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet, BinaryHeap, BTreeMap};
use std::sync::Arc;
use std::cmp::Reverse;
use tracing::{debug, warn, info};
use uuid::Uuid;
use rand::prelude::*;
use priority_queue::PriorityQueue;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Simplified HNSW index for vector search
struct SimpleHnswIndex {
    vectors: Vec<Vec<f32>>,
    dimension: usize,
}

impl SimpleHnswIndex {
    fn new(dimension: usize, _max_neighbors: usize) -> Self {
        Self {
            vectors: Vec::new(),
            dimension,
        }
    }

    fn insert(&mut self, vector: &[f32]) -> Result<usize> {
        if vector.len() != self.dimension {
            return Err(anyhow::anyhow!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            ));
        }

        let id = self.vectors.len();
        self.vectors.push(vector.to_vec());
        Ok(id)
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        if query.len() != self.dimension {
            return Err(anyhow::anyhow!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            ));
        }

        let mut results: Vec<(usize, f32)> = self
            .vectors
            .iter()
            .enumerate()
            .map(|(id, vector)| {
                let distance = cosine_distance(query, vector);
                (id, distance)
            })
            .collect();

        // Sort by distance (ascending) and take top k
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);

        Ok(results)
    }
}

/// Calculate cosine distance between two vectors
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 1.0; // Maximum distance for dimension mismatch
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // Maximum distance for zero vectors
    }

    1.0 - (dot_product / (norm_a * norm_b))
}

pub struct HnswIndexer {
    index: Arc<Mutex<SimpleHnswIndex>>,
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

        // TODO: Implement proper HNSW (Hierarchical Navigable Small World) index
        // - Implement hierarchical graph structure with multiple layers
        // - Add navigable small world connectivity algorithm
        // - Support dynamic index updates and insertions
        // - Implement efficient nearest neighbor search with pruning
        // - Add index persistence and loading from disk
        // - Support parallel index construction and querying
        // - Implement index optimization and memory management
        // PLACEHOLDER: Using simplified linear search
        // - [ ] Use established HNSW library (hnswlib, faiss, or ann-search) for production-grade implementation
        // - [ ] Implement hierarchical graph construction with multiple layers
        // - [ ] Add proper neighbor selection and pruning algorithms
        // - [ ] Support different distance metrics (cosine, euclidean, manhattan, etc.)
        // - [ ] Implement efficient index construction and incremental updates
        // - [ ] Add memory-mapped index persistence and loading
        // - [ ] Support parallel index construction and querying
        // Create simplified HNSW index
        let index = Arc::new(Mutex::new(SimpleHnswIndex::new(
            metadata.dim,
            32, // default max_neighbors
        )));

        Ok(Self {
            index,
            metadata,
            id_to_uuid: Arc::new(Mutex::new(HashMap::new())),
            uuid_to_id: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        })
    }

    /// Insert a vector into the index
    pub async fn insert(&mut self, block_id: Uuid, vector: &[f32]) -> Result<()> {
        debug!(
            "Inserting vector for block {} ({}d)",
            block_id,
            vector.len()
        );

        // Validate vector dimensions
        if vector.len() != self.metadata.dim {
            return Err(anyhow::anyhow!(
                "Vector dimension mismatch: expected {}, got {}",
                self.metadata.dim,
                vector.len()
            ));
        }

        // Insert into HNSW index
        let mut index = self.index.lock();
        let actual_id = index
            .insert(vector)
            .context("Failed to insert vector into HNSW index")?;

        // Update ID mappings
        {
            let mut id_to_uuid = self.id_to_uuid.lock();
            let mut uuid_to_id = self.uuid_to_id.lock();

            id_to_uuid.insert(actual_id, block_id);
            uuid_to_id.insert(block_id, actual_id);
        }

        // Update metadata
        self.metadata.node_count += 1;

        debug!(
            "Successfully inserted vector for block {} with ID {}",
            block_id, actual_id
        );
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
        let index = self.index.lock();
        let neighbors = index
            .search(&query.vector, query.k)
            .context("Failed to perform HNSW search")?;

        // Convert results to VectorSearchResult
        let results: Vec<crate::types::VectorSearchResult> = neighbors
            .into_iter()
            .filter_map(|(id, distance)| {
                // Get UUID from ID mapping
                let id_to_uuid = self.id_to_uuid.lock();
                let block_id = id_to_uuid.get(&id)?;

                // Convert distance to similarity score
                let similarity = match self.metadata.metric.as_str() {
                    "cosine" => 1.0 - distance,     // Cosine distance to similarity
                    "l2" => 1.0 / (1.0 + distance), // L2 distance to similarity
                    "ip" => distance,               // Inner product is already similarity-like
                    _ => 1.0 - distance,            // Default to cosine-like conversion
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
