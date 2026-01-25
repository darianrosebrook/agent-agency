//! Memory System
//!
//! Unified memory combining graph + vector with decay and provenance.

use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::decay::DecayEngine;
use crate::graph::{EdgeType, ExpandedNode, GraphEdge, GraphNode, KnowledgeGraph, NodeId};
use crate::vector::{SearchResult, VectorStore};
use crate::MemoryError;

/// Memory system combining graph and vector stores
pub struct MemorySystem {
    /// Vector store for semantic similarity
    vectors: Arc<RwLock<VectorStore>>,
    /// Knowledge graph for relationships
    graph: Arc<RwLock<KnowledgeGraph>>,
    /// Decay engine
    decay: DecayEngine,
}

impl MemorySystem {
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(RwLock::new(VectorStore::new())),
            graph: Arc::new(RwLock::new(KnowledgeGraph::new())),
            decay: DecayEngine::new(),
        }
    }

    /// Store content in both vector and graph stores
    pub fn store(
        &self,
        id: impl Into<String>,
        content: impl Into<String>,
        memory_type: impl Into<String>,
    ) -> Result<String, MemoryError> {
        let id = id.into();
        let content = content.into();
        let memory_type = memory_type.into();

        // Add to vector store
        {
            let mut vectors = self.vectors.write().unwrap();
            let entry = vectors.create_entry(&id, &content);
            vectors.add(entry);
        }

        // Add to graph
        {
            let mut graph = self.graph.write().unwrap();
            let node = GraphNode::new(&id, &content, &memory_type);
            graph.add_node(node)?;
        }

        Ok(id)
    }

    /// Store with explicit relationships
    pub fn store_with_relations(
        &self,
        id: impl Into<String>,
        content: impl Into<String>,
        memory_type: impl Into<String>,
        relations: &[(String, EdgeType)],
    ) -> Result<String, MemoryError> {
        let id = self.store(id, content, memory_type)?;

        // Add relationships
        {
            let mut graph = self.graph.write().unwrap();
            for (target_id, edge_type) in relations {
                if graph.get_node(target_id).is_some() {
                    let edge = GraphEdge::new(id.clone(), target_id.clone(), edge_type.clone());
                    graph.add_edge(edge)?;
                }
            }
        }

        Ok(id)
    }

    /// Multi-hop recall with provenance
    pub fn recall(&self, query: &str, max_hops: usize, limit: usize) -> RecallResult {
        // 1. Vector similarity search to find seeds
        let seeds: Vec<NodeId> = {
            let vectors = self.vectors.read().unwrap();
            vectors
                .search(query, limit)
                .into_iter()
                .map(|r| r.entry.id)
                .collect()
        };

        // 2. Graph expansion from seeds
        let expanded: Vec<ExpandedNode> = {
            let graph = self.graph.read().unwrap();
            graph.expand(&seeds, max_hops)
        };

        // 3. Apply decay weights
        let items: Vec<RecallItem> = expanded
            .into_iter()
            .map(|exp| {
                let weight = self
                    .decay
                    .calculate_weight(&exp.node.decay_category, exp.node.initial_weight);

                // Decay based on hop distance
                let hop_decay = 0.9_f64.powi(exp.hop_distance as i32);
                let final_weight = weight * hop_decay;

                RecallItem {
                    id: exp.node.id.clone(),
                    content: exp.node.content.clone(),
                    memory_type: exp.node.node_type.clone(),
                    weight: final_weight,
                    hop_distance: exp.hop_distance,
                    provenance: exp.path,
                }
            })
            .collect();

        // 4. Sort by weight and limit
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_items.truncate(limit);

        // 5. Build provenance chain
        let provenance = build_provenance_chain(&sorted_items);

        RecallResult {
            items: sorted_items,
            provenance,
            query: query.to_string(),
            max_hops,
            timestamp: Utc::now(),
        }
    }

    /// Record successful use of a memory item
    pub fn record_use(&self, id: &str) -> Result<(), MemoryError> {
        // Update graph node
        {
            let mut graph = self.graph.write().unwrap();
            if let Some(node) = graph.get_node_mut(id) {
                node.decay_category.record_use();
                node.updated_at = Utc::now();
            }
        }

        // Update vector entry
        {
            let mut vectors = self.vectors.write().unwrap();
            if let Some(entry) = vectors.get_mut(id) {
                entry.decay_category.record_use();
            }
        }

        Ok(())
    }

    /// Record exploration of a memory item
    pub fn record_exploration(&self, id: &str) -> Result<(), MemoryError> {
        {
            let mut graph = self.graph.write().unwrap();
            if let Some(node) = graph.get_node_mut(id) {
                node.decay_category.record_exploration();
                node.updated_at = Utc::now();
            }
        }

        {
            let mut vectors = self.vectors.write().unwrap();
            if let Some(entry) = vectors.get_mut(id) {
                entry.decay_category.record_exploration();
            }
        }

        Ok(())
    }

    /// Record failure when using a memory item
    pub fn record_failure(&self, id: &str) -> Result<(), MemoryError> {
        {
            let mut graph = self.graph.write().unwrap();
            if let Some(node) = graph.get_node_mut(id) {
                node.decay_category.record_failure();
                node.updated_at = Utc::now();
            }
        }

        {
            let mut vectors = self.vectors.write().unwrap();
            if let Some(entry) = vectors.get_mut(id) {
                entry.decay_category.record_failure();
            }
        }

        Ok(())
    }

    /// Semantic search only (no graph expansion)
    pub fn semantic_search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let vectors = self.vectors.read().unwrap();
        vectors.search(query, limit)
    }

    /// Get a memory item by ID
    pub fn get(&self, id: &str) -> Option<RecallItem> {
        let graph = self.graph.read().unwrap();
        let node = graph.get_node(id)?;

        let weight = self
            .decay
            .calculate_weight(&node.decay_category, node.initial_weight);

        Some(RecallItem {
            id: node.id.clone(),
            content: node.content.clone(),
            memory_type: node.node_type.clone(),
            weight,
            hop_distance: 0,
            provenance: vec![id.to_string()],
        })
    }

    /// Remove a memory item
    pub fn remove(&self, id: &str) -> Result<(), MemoryError> {
        {
            let mut graph = self.graph.write().unwrap();
            graph.remove_node(id);
        }

        {
            let mut vectors = self.vectors.write().unwrap();
            vectors.remove(id);
        }

        Ok(())
    }

    /// Garbage collect decayed items
    pub fn gc(&self) -> usize {
        let mut removed = 0;

        // Find items to remove
        let to_remove: Vec<String> = {
            let graph = self.graph.read().unwrap();
            graph
                .all_nodes()
                .iter()
                .filter(|node| {
                    let weight = self
                        .decay
                        .calculate_weight(&node.decay_category, node.initial_weight);
                    self.decay.should_remove(weight)
                })
                .map(|node| node.id.clone())
                .collect()
        };

        // Remove them
        for id in to_remove {
            if self.remove(&id).is_ok() {
                removed += 1;
            }
        }

        removed
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        let graph = self.graph.read().unwrap();
        let vectors = self.vectors.read().unwrap();

        MemoryStats {
            graph_nodes: graph.node_count(),
            graph_edges: graph.edge_count(),
            vector_entries: vectors.len(),
        }
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// A recalled memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallItem {
    pub id: String,
    pub content: String,
    pub memory_type: String,
    pub weight: f64,
    pub hop_distance: usize,
    pub provenance: Vec<String>,
}

/// Result of a recall operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResult {
    pub items: Vec<RecallItem>,
    pub provenance: ProvenanceChain,
    pub query: String,
    pub max_hops: usize,
    pub timestamp: DateTime<Utc>,
}

/// Provenance chain for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub chain_hash: String,
    pub entries: Vec<ProvenanceEntry>,
}

/// A single provenance entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub item_id: String,
    pub path: Vec<String>,
    pub weight: f64,
}

/// Build provenance chain from recall items
fn build_provenance_chain(items: &[RecallItem]) -> ProvenanceChain {
    let entries: Vec<ProvenanceEntry> = items
        .iter()
        .map(|item| ProvenanceEntry {
            item_id: item.id.clone(),
            path: item.provenance.clone(),
            weight: item.weight,
        })
        .collect();

    // Compute chain hash
    let mut hasher = Sha256::new();
    for entry in &entries {
        hasher.update(entry.item_id.as_bytes());
        hasher.update(entry.weight.to_le_bytes());
    }
    let chain_hash = hex::encode(hasher.finalize());

    ProvenanceChain {
        chain_hash,
        entries,
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub graph_nodes: usize,
    pub graph_edges: usize,
    pub vector_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_system_store() {
        let memory = MemorySystem::new();

        let id = memory.store("m1", "Test content", "concept").unwrap();
        assert_eq!(id, "m1");

        let stats = memory.stats();
        assert_eq!(stats.graph_nodes, 1);
        assert_eq!(stats.vector_entries, 1);
    }

    #[test]
    fn test_memory_system_recall() {
        let memory = MemorySystem::new();

        memory.store("m1", "Rust programming", "concept").unwrap();
        memory.store("m2", "Python programming", "concept").unwrap();
        memory.store("m3", "Cooking recipes", "fact").unwrap();

        let result = memory.recall("programming languages", 1, 10);
        assert!(!result.items.is_empty());
    }

    #[test]
    fn test_memory_system_with_relations() {
        let memory = MemorySystem::new();

        memory.store("m1", "Parent concept", "concept").unwrap();

        memory
            .store_with_relations(
                "m2",
                "Child concept",
                "concept",
                &[("m1".to_string(), EdgeType::PartOf)],
            )
            .unwrap();

        let result = memory.recall("Parent", 1, 10);

        // Should find both m1 and m2 (via expansion)
        let ids: Vec<_> = result.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"m1"));
    }

    #[test]
    fn test_memory_system_record_use() {
        let memory = MemorySystem::new();
        memory.store("m1", "Test content", "concept").unwrap();

        // Record use should not fail
        memory.record_use("m1").unwrap();

        // Item should still exist with updated category
        let item = memory.get("m1").unwrap();
        assert!(item.weight > 0.0);
    }

    #[test]
    fn test_memory_system_get() {
        let memory = MemorySystem::new();
        memory.store("m1", "Test content", "concept").unwrap();

        let item = memory.get("m1").unwrap();
        assert_eq!(item.id, "m1");
        assert_eq!(item.content, "Test content");
        assert_eq!(item.hop_distance, 0);
    }

    #[test]
    fn test_memory_system_remove() {
        let memory = MemorySystem::new();
        memory.store("m1", "Test content", "concept").unwrap();

        memory.remove("m1").unwrap();

        assert!(memory.get("m1").is_none());
        assert_eq!(memory.stats().graph_nodes, 0);
    }

    #[test]
    fn test_provenance_chain() {
        let memory = MemorySystem::new();

        memory.store("m1", "Node A", "concept").unwrap();
        memory
            .store_with_relations(
                "m2",
                "Node B",
                "concept",
                &[("m1".to_string(), EdgeType::RelatedTo)],
            )
            .unwrap();

        let result = memory.recall("Node A", 2, 10);

        // Provenance should include the path
        assert!(!result.provenance.chain_hash.is_empty());
        assert!(!result.provenance.entries.is_empty());
    }

    #[test]
    fn test_semantic_search() {
        let memory = MemorySystem::new();

        memory.store("m1", "Rust async programming", "code").unwrap();
        memory.store("m2", "Python async io", "code").unwrap();

        let results = memory.semantic_search("async rust", 5);
        assert!(!results.is_empty());
    }
}
