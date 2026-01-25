//! Knowledge Graph
//!
//! Relationship-based memory storage with multi-hop traversal.

use std::collections::{HashMap, HashSet, VecDeque};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::decay::DecayCategory;
use crate::MemoryError;

/// Node identifier
pub type NodeId = String;

/// Edge identifier
pub type EdgeId = String;

/// A node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub content: String,
    pub node_type: String,
    pub metadata: HashMap<String, String>,
    pub content_hash: String,
    pub decay_category: DecayCategory,
    pub initial_weight: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GraphNode {
    pub fn new(
        id: impl Into<String>,
        content: impl Into<String>,
        node_type: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let content = content.into();
        let content_hash = compute_hash(&content);

        Self {
            id,
            content,
            node_type: node_type.into(),
            metadata: HashMap::new(),
            content_hash,
            decay_category: DecayCategory::fresh(),
            initial_weight: 1.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.initial_weight = weight;
        self
    }
}

/// Edge type for relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// A is related to B
    RelatedTo,
    /// A is a part of B
    PartOf,
    /// A depends on B
    DependsOn,
    /// A references B
    References,
    /// A is similar to B
    SimilarTo,
    /// A causes B
    Causes,
    /// Custom relationship
    Custom(String),
}

impl EdgeType {
    pub fn name(&self) -> &str {
        match self {
            Self::RelatedTo => "related_to",
            Self::PartOf => "part_of",
            Self::DependsOn => "depends_on",
            Self::References => "references",
            Self::SimilarTo => "similar_to",
            Self::Causes => "causes",
            Self::Custom(name) => name,
        }
    }
}

/// An edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl GraphEdge {
    pub fn new(source: NodeId, target: NodeId, edge_type: EdgeType) -> Self {
        let id = format!("{}-{}-{}", source, edge_type.name(), target);
        Self {
            id,
            source,
            target,
            edge_type,
            weight: 1.0,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

/// Knowledge graph for storing and traversing relationships
pub struct KnowledgeGraph {
    nodes: HashMap<NodeId, GraphNode>,
    edges: HashMap<EdgeId, GraphEdge>,
    /// Adjacency list: node -> outgoing edges
    outgoing: HashMap<NodeId, Vec<EdgeId>>,
    /// Reverse adjacency: node -> incoming edges
    incoming: HashMap<NodeId, Vec<EdgeId>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) -> Result<NodeId, MemoryError> {
        let id = node.id.clone();
        if self.nodes.contains_key(&id) {
            return Err(MemoryError::NodeExists(id));
        }
        self.nodes.insert(id.clone(), node);
        self.outgoing.entry(id.clone()).or_default();
        self.incoming.entry(id.clone()).or_default();
        Ok(id)
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<EdgeId, MemoryError> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(MemoryError::NodeNotFound(edge.source.clone()));
        }
        if !self.nodes.contains_key(&edge.target) {
            return Err(MemoryError::NodeNotFound(edge.target.clone()));
        }

        let id = edge.id.clone();
        self.outgoing
            .entry(edge.source.clone())
            .or_default()
            .push(id.clone());
        self.incoming
            .entry(edge.target.clone())
            .or_default()
            .push(id.clone());
        self.edges.insert(id.clone(), edge);

        Ok(id)
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut GraphNode> {
        self.nodes.get_mut(id)
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: &str) -> Option<&GraphEdge> {
        self.edges.get(id)
    }

    /// Get outgoing edges from a node
    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.outgoing
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.edges.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get incoming edges to a node
    pub fn incoming_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.incoming
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.edges.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get neighbors of a node (outgoing)
    pub fn neighbors(&self, node_id: &str) -> Vec<&GraphNode> {
        self.outgoing_edges(node_id)
            .iter()
            .filter_map(|edge| self.nodes.get(&edge.target))
            .collect()
    }

    /// Multi-hop expansion from seed nodes
    pub fn expand(&self, seeds: &[NodeId], max_hops: usize) -> Vec<ExpandedNode> {
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut result: Vec<ExpandedNode> = Vec::new();
        let mut queue: VecDeque<(NodeId, usize, Vec<NodeId>)> = VecDeque::new();

        // Initialize with seeds
        for seed in seeds {
            if let Some(node) = self.nodes.get(seed) {
                visited.insert(seed.clone());
                result.push(ExpandedNode {
                    node: node.clone(),
                    hop_distance: 0,
                    path: vec![seed.clone()],
                });
                queue.push_back((seed.clone(), 0, vec![seed.clone()]));
            }
        }

        // BFS expansion
        while let Some((current_id, hops, path)) = queue.pop_front() {
            if hops >= max_hops {
                continue;
            }

            for edge in self.outgoing_edges(&current_id) {
                if !visited.contains(&edge.target) {
                    visited.insert(edge.target.clone());

                    if let Some(node) = self.nodes.get(&edge.target) {
                        let mut new_path = path.clone();
                        new_path.push(edge.target.clone());

                        result.push(ExpandedNode {
                            node: node.clone(),
                            hop_distance: hops + 1,
                            path: new_path.clone(),
                        });

                        queue.push_back((edge.target.clone(), hops + 1, new_path));
                    }
                }
            }
        }

        result
    }

    /// Find nodes by type
    pub fn nodes_by_type(&self, node_type: &str) -> Vec<&GraphNode> {
        self.nodes
            .values()
            .filter(|n| n.node_type == node_type)
            .collect()
    }

    /// Remove a node and its edges
    pub fn remove_node(&mut self, id: &str) -> Option<GraphNode> {
        let node = self.nodes.remove(id)?;

        // Remove outgoing edges
        if let Some(outgoing_ids) = self.outgoing.remove(id) {
            for edge_id in outgoing_ids {
                if let Some(edge) = self.edges.remove(&edge_id) {
                    if let Some(incoming) = self.incoming.get_mut(&edge.target) {
                        incoming.retain(|e| e != &edge_id);
                    }
                }
            }
        }

        // Remove incoming edges
        if let Some(incoming_ids) = self.incoming.remove(id) {
            for edge_id in incoming_ids {
                if let Some(edge) = self.edges.remove(&edge_id) {
                    if let Some(outgoing) = self.outgoing.get_mut(&edge.source) {
                        outgoing.retain(|e| e != &edge_id);
                    }
                }
            }
        }

        Some(node)
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get all nodes
    pub fn all_nodes(&self) -> Vec<&GraphNode> {
        self.nodes.values().collect()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A node with expansion context
#[derive(Debug, Clone)]
pub struct ExpandedNode {
    pub node: GraphNode,
    pub hop_distance: usize,
    pub path: Vec<NodeId>,
}

/// Compute SHA-256 hash of content
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> KnowledgeGraph {
        let mut graph = KnowledgeGraph::new();

        graph
            .add_node(GraphNode::new("a", "Node A", "concept"))
            .unwrap();
        graph
            .add_node(GraphNode::new("b", "Node B", "concept"))
            .unwrap();
        graph
            .add_node(GraphNode::new("c", "Node C", "concept"))
            .unwrap();
        graph
            .add_node(GraphNode::new("d", "Node D", "fact"))
            .unwrap();

        graph
            .add_edge(GraphEdge::new(
                "a".to_string(),
                "b".to_string(),
                EdgeType::RelatedTo,
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::new(
                "b".to_string(),
                "c".to_string(),
                EdgeType::DependsOn,
            ))
            .unwrap();
        graph
            .add_edge(GraphEdge::new(
                "a".to_string(),
                "d".to_string(),
                EdgeType::References,
            ))
            .unwrap();

        graph
    }

    #[test]
    fn test_add_node() {
        let mut graph = KnowledgeGraph::new();
        let id = graph
            .add_node(GraphNode::new("test", "Test content", "concept"))
            .unwrap();
        assert_eq!(id, "test");
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_add_duplicate_node() {
        let mut graph = KnowledgeGraph::new();
        graph
            .add_node(GraphNode::new("test", "Test content", "concept"))
            .unwrap();
        let result = graph.add_node(GraphNode::new("test", "Other content", "concept"));
        assert!(matches!(result, Err(MemoryError::NodeExists(_))));
    }

    #[test]
    fn test_add_edge() {
        let mut graph = KnowledgeGraph::new();
        graph
            .add_node(GraphNode::new("a", "A", "concept"))
            .unwrap();
        graph
            .add_node(GraphNode::new("b", "B", "concept"))
            .unwrap();

        let edge_id = graph
            .add_edge(GraphEdge::new(
                "a".to_string(),
                "b".to_string(),
                EdgeType::RelatedTo,
            ))
            .unwrap();

        assert_eq!(graph.edge_count(), 1);
        assert!(graph.get_edge(&edge_id).is_some());
    }

    #[test]
    fn test_neighbors() {
        let graph = make_graph();
        let neighbors = graph.neighbors("a");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_expand_one_hop() {
        let graph = make_graph();
        let expanded = graph.expand(&["a".to_string()], 1);

        // Should include a, b, d (1 hop from a)
        assert_eq!(expanded.len(), 3);

        let ids: Vec<_> = expanded.iter().map(|e| e.node.id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
        assert!(ids.contains(&"d"));
    }

    #[test]
    fn test_expand_two_hops() {
        let graph = make_graph();
        let expanded = graph.expand(&["a".to_string()], 2);

        // Should include a, b, c, d (2 hops from a)
        assert_eq!(expanded.len(), 4);
    }

    #[test]
    fn test_expand_path_tracking() {
        let graph = make_graph();
        let expanded = graph.expand(&["a".to_string()], 2);

        // Find node c's expansion
        let c_expansion = expanded.iter().find(|e| e.node.id == "c").unwrap();
        assert_eq!(c_expansion.hop_distance, 2);
        assert_eq!(c_expansion.path, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_nodes_by_type() {
        let graph = make_graph();
        let concepts = graph.nodes_by_type("concept");
        assert_eq!(concepts.len(), 3);

        let facts = graph.nodes_by_type("fact");
        assert_eq!(facts.len(), 1);
    }

    #[test]
    fn test_remove_node() {
        let mut graph = make_graph();
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);

        let removed = graph.remove_node("b").unwrap();
        assert_eq!(removed.id, "b");

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 1); // Only a->d remains
    }

    #[test]
    fn test_node_with_metadata() {
        let node = GraphNode::new("test", "Content", "concept")
            .with_metadata("source", "file.rs")
            .with_metadata("line", "42");

        assert_eq!(node.metadata.get("source"), Some(&"file.rs".to_string()));
        assert_eq!(node.metadata.get("line"), Some(&"42".to_string()));
    }
}
