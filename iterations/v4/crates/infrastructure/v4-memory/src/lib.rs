//! V4 Memory
//!
//! Graph + vector memory system with Sterling-style decay.
//!
//! ## Overview
//!
//! This crate provides:
//! - **KnowledgeGraph**: Relationship-based storage with multi-hop traversal
//! - **VectorStore**: Semantic similarity search using embeddings
//! - **DecayEngine**: Sterling-style decay categories for memory management
//! - **MemorySystem**: Unified interface combining all components
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    MemorySystem                      │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │  recall(query)                                       │
//! │      │                                               │
//! │      ▼                                               │
//! │  ┌─────────────────┐    ┌─────────────────┐         │
//! │  │   VectorStore   │    │ KnowledgeGraph  │         │
//! │  │  (similarity)   │───►│  (expansion)    │         │
//! │  └─────────────────┘    └─────────────────┘         │
//! │           │                      │                   │
//! │           └──────────┬───────────┘                   │
//! │                      ▼                               │
//! │              ┌─────────────┐                         │
//! │              │ DecayEngine │                         │
//! │              │  (weights)  │                         │
//! │              └─────────────┘                         │
//! │                      │                               │
//! │                      ▼                               │
//! │              RecallResult with Provenance            │
//! │                                                      │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Decay Categories (from Sterling)
//!
//! Memory items are categorized for decay:
//!
//! - **SuccessPath**: Recently used, high retention
//! - **Explored**: Accessed but not used, moderate decay
//! - **NegativeEvidence**: Tried and failed, faster decay
//! - **Fresh**: New entries, moderate initial weight
//!
//! ## Example
//!
//! ```rust,ignore
//! use v4_memory::MemorySystem;
//!
//! let memory = MemorySystem::new();
//!
//! // Store some knowledge
//! memory.store("rust-async", "Rust async/await syntax", "concept")?;
//! memory.store("tokio", "Tokio async runtime", "library")?;
//!
//! // Store with relationships
//! memory.store_with_relations(
//!     "tokio-spawn",
//!     "tokio::spawn for concurrent tasks",
//!     "api",
//!     &[("tokio".to_string(), EdgeType::PartOf)],
//! )?;
//!
//! // Recall with multi-hop expansion
//! let result = memory.recall("async rust", 2, 10);
//!
//! for item in result.items {
//!     println!("{}: {} (weight: {:.2})", item.id, item.content, item.weight);
//! }
//!
//! // Record successful use (improves retention)
//! memory.record_use("rust-async")?;
//!
//! // Garbage collect decayed items
//! let removed = memory.gc();
//! ```

pub mod decay;
pub mod graph;
pub mod system;
pub mod vector;

// Re-export main types
pub use decay::{DecayCategory, DecayConfig, DecayEngine};
pub use graph::{EdgeType, ExpandedNode, GraphEdge, GraphNode, KnowledgeGraph, NodeId};
pub use system::{MemoryStats, MemorySystem, ProvenanceChain, ProvenanceEntry, RecallItem, RecallResult};
pub use vector::{SearchResult, VectorEntry, VectorStore};

/// Memory errors
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Node already exists: {0}")]
    NodeExists(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Edge not found: {0}")]
    EdgeNotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_memory_workflow() {
        let memory = MemorySystem::new();

        // Store knowledge
        memory.store("concept-1", "Rust ownership model", "concept").unwrap();
        memory.store("concept-2", "Borrow checker", "concept").unwrap();
        memory.store("example-1", "let x = String::new();", "code").unwrap();

        // Add relationships
        memory.store_with_relations(
            "concept-3",
            "Lifetimes in Rust",
            "concept",
            &[
                ("concept-1".to_string(), EdgeType::PartOf),
                ("concept-2".to_string(), EdgeType::RelatedTo),
            ],
        ).unwrap();

        // Recall with expansion
        let result = memory.recall("ownership", 2, 10);
        assert!(!result.items.is_empty());

        // Provenance should be populated
        assert!(!result.provenance.chain_hash.is_empty());

        // Record use of useful items
        for item in result.items.iter().take(2) {
            memory.record_use(&item.id).unwrap();
        }

        // Check stats
        let stats = memory.stats();
        assert_eq!(stats.graph_nodes, 4);
    }

    #[test]
    fn test_decay_over_time() {
        use chrono::Duration;

        let engine = DecayEngine::new();

        // Fresh item
        let fresh = DecayCategory::Fresh {
            created_at: chrono::Utc::now(),
        };
        let fresh_weight = engine.calculate_weight(&fresh, 1.0);

        // Old item
        let old = DecayCategory::Fresh {
            created_at: chrono::Utc::now() - Duration::days(7),
        };
        let old_weight = engine.calculate_weight(&old, 1.0);

        // Fresh should have higher weight than old
        assert!(fresh_weight > old_weight);
    }

    #[test]
    fn test_success_path_retention() {
        let memory = MemorySystem::new();
        memory.store("m1", "Important concept", "concept").unwrap();

        // Record multiple uses
        for _ in 0..5 {
            memory.record_use("m1").unwrap();
        }

        // Item should still have high weight
        let item = memory.get("m1").unwrap();
        assert!(item.weight > 0.9);
    }

    #[test]
    fn test_graph_expansion_depth() {
        // Use the graph directly to test expansion without vector search
        let mut graph = KnowledgeGraph::new();

        // Create a chain: a -> b -> c -> d
        graph.add_node(GraphNode::new("a", "Node A", "concept")).unwrap();
        graph.add_node(GraphNode::new("b", "Node B", "concept")).unwrap();
        graph.add_node(GraphNode::new("c", "Node C", "concept")).unwrap();
        graph.add_node(GraphNode::new("d", "Node D", "concept")).unwrap();

        graph.add_edge(GraphEdge::new("a".to_string(), "b".to_string(), EdgeType::RelatedTo)).unwrap();
        graph.add_edge(GraphEdge::new("b".to_string(), "c".to_string(), EdgeType::RelatedTo)).unwrap();
        graph.add_edge(GraphEdge::new("c".to_string(), "d".to_string(), EdgeType::RelatedTo)).unwrap();

        // 1 hop from a should find a and b
        let result1 = graph.expand(&["a".to_string()], 1);
        let ids1: Vec<_> = result1.iter().map(|e| e.node.id.as_str()).collect();
        assert!(ids1.contains(&"a"));
        assert!(ids1.contains(&"b"));
        assert!(!ids1.contains(&"d")); // d is 3 hops away

        // 3 hops should find all
        let result3 = graph.expand(&["a".to_string()], 3);
        let ids3: Vec<_> = result3.iter().map(|e| e.node.id.as_str()).collect();
        assert!(ids3.contains(&"a"));
        assert!(ids3.contains(&"b"));
        assert!(ids3.contains(&"c"));
        assert!(ids3.contains(&"d"));
    }

    #[test]
    fn test_hop_distance_in_graph() {
        let mut graph = KnowledgeGraph::new();

        graph.add_node(GraphNode::new("a", "Node A", "concept")).unwrap();
        graph.add_node(GraphNode::new("b", "Node B", "concept")).unwrap();
        graph.add_node(GraphNode::new("c", "Node C", "concept")).unwrap();

        graph.add_edge(GraphEdge::new("a".to_string(), "b".to_string(), EdgeType::RelatedTo)).unwrap();
        graph.add_edge(GraphEdge::new("b".to_string(), "c".to_string(), EdgeType::RelatedTo)).unwrap();

        let result = graph.expand(&["a".to_string()], 2);

        // Find items by id
        let a = result.iter().find(|e| e.node.id == "a");
        let b = result.iter().find(|e| e.node.id == "b");
        let c = result.iter().find(|e| e.node.id == "c");

        if let (Some(a), Some(b), Some(c)) = (a, b, c) {
            // Check hop distances
            assert_eq!(a.hop_distance, 0);
            assert_eq!(b.hop_distance, 1);
            assert_eq!(c.hop_distance, 2);
        }
    }
}
