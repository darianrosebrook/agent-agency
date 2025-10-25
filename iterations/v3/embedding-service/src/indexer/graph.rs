//! Graph indexing and relationship management
//!
//! Graph-based indexing for diagrams, knowledge graphs, and
//! relational data with adjacency lists and property management.

use super::super::types::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Graph node properties
#[derive(Debug, Clone)]
pub struct NodeProperty {
    pub node_type: NodeType,
    pub label: String,
    pub properties: HashMap<String, PropertyValue>,
    pub embedding: Option<EmbeddingVector>,
}

/// Node type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Entity,
    Concept,
    Document,
    Image,
    Relationship,
    Custom(String),
}

/// Property value types
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
}

/// Graph edge representation
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: Uuid,
    pub target: Uuid,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub properties: HashMap<String, PropertyValue>,
}

/// Edge type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    RelatedTo,
    Contains,
    References,
    SimilarTo,
    ParentOf,
    Custom(String),
}

/// Graph indexer for relationship management
#[derive(Debug)]
pub struct GraphIndexer {
    /// Diagram graph adjacency lists
    graph_adjacency: HashMap<Uuid, Vec<Uuid>>,
    /// Graph node metadata and properties
    node_properties: HashMap<Uuid, NodeProperty>,
    /// Edge information
    edges: HashMap<(Uuid, Uuid), GraphEdge>,
    /// Reverse adjacency for efficient queries
    reverse_adjacency: HashMap<Uuid, Vec<Uuid>>,
}

impl GraphIndexer {
    /// Create a new graph indexer
    pub fn new() -> Self {
        Self {
            graph_adjacency: HashMap::new(),
            node_properties: HashMap::new(),
            edges: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, id: Uuid, properties: NodeProperty) -> Result<()> {
        self.node_properties.insert(id, properties);
        Ok(())
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<()> {
        let source = edge.source;
        let target = edge.target;

        // Add to adjacency list
        self.graph_adjacency
            .entry(source)
            .or_insert_with(Vec::new)
            .push(target);

        // Add to reverse adjacency
        self.reverse_adjacency
            .entry(target)
            .or_insert_with(Vec::new)
            .push(source);

        // Store edge information
        self.edges.insert((source, target), edge);

        Ok(())
    }

    /// Remove a node and all its connections
    pub fn remove_node(&mut self, id: Uuid) -> Result<()> {
        // Remove from adjacency lists
        self.graph_adjacency.remove(&id);
        self.reverse_adjacency.remove(&id);

        // Remove all edges involving this node
        self.edges.retain(|(s, t), _| *s != id && *t != id);

        // Remove from other nodes' adjacency lists
        for neighbors in self.graph_adjacency.values_mut() {
            neighbors.retain(|&n| n != id);
        }

        for neighbors in self.reverse_adjacency.values_mut() {
            neighbors.retain(|&n| n != id);
        }

        // Remove properties
        self.node_properties.remove(&id);

        Ok(())
    }

    /// Get neighbors of a node
    pub fn get_neighbors(&self, node_id: Uuid) -> Vec<Uuid> {
        self.graph_adjacency.get(&node_id).cloned().unwrap_or_default()
    }

    /// Get nodes that reference this node
    pub fn get_references(&self, node_id: Uuid) -> Vec<Uuid> {
        self.reverse_adjacency.get(&node_id).cloned().unwrap_or_default()
    }

    /// Find shortest path between two nodes
    pub fn shortest_path(&self, start: Uuid, end: Uuid) -> Option<Vec<Uuid>> {
        if !self.node_properties.contains_key(&start) || !self.node_properties.contains_key(&end) {
            return None;
        }

        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parent_map = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if current == end {
                // Reconstruct path
                let mut path = vec![end];
                let mut current_node = end;

                while let Some(&parent) = parent_map.get(&current_node) {
                    path.push(parent);
                    current_node = parent;
                }

                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.graph_adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent_map.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }

    /// Find nodes by property value
    pub fn find_nodes_by_property(&self, property_key: &str, property_value: &PropertyValue) -> Vec<Uuid> {
        self.node_properties
            .iter()
            .filter_map(|(id, props)| {
                if props.properties.get(property_key) == Some(property_value) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find nodes by type
    pub fn find_nodes_by_type(&self, node_type: &NodeType) -> Vec<Uuid> {
        self.node_properties
            .iter()
            .filter_map(|(id, props)| {
                if &props.node_type == node_type {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculate node centrality (simplified)
    pub fn calculate_centrality(&self, node_id: Uuid) -> f64 {
        let outgoing = self.get_neighbors(node_id).len() as f64;
        let incoming = self.get_references(node_id).len() as f64;

        (outgoing + incoming) / 2.0 // Simple degree centrality
    }

    /// Get graph statistics
    pub fn get_statistics(&self) -> GraphStatistics {
        let total_nodes = self.node_properties.len();
        let total_edges = self.edges.len();

        let node_types = self.node_properties.values()
            .fold(HashMap::new(), |mut acc, props| {
                *acc.entry(format!("{:?}", props.node_type)).or_insert(0) += 1;
                acc
            });

        let avg_degree = if total_nodes > 0 {
            self.graph_adjacency.values().map(|neighbors| neighbors.len()).sum::<usize>() as f64 / total_nodes as f64
        } else {
            0.0
        };

        GraphStatistics {
            total_nodes,
            total_edges,
            node_types,
            average_degree: avg_degree,
            connected_components: self.count_connected_components(),
        }
    }

    /// Export graph in GraphML format
    pub fn export_graphml(&self) -> Result<String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n");

        // Add node definitions
        for (id, properties) in &self.node_properties {
            xml.push_str(&format!("  <node id=\"{}\">\n", id));
            xml.push_str(&format!("    <data key=\"label\">{}</data>\n", properties.label));
            xml.push_str(&format!("    <data key=\"type\">{:?}</data>\n", properties.node_type));
            xml.push_str("  </node>\n");
        }

        // Add edge definitions
        for ((source, target), edge) in &self.edges {
            xml.push_str(&format!("  <edge source=\"{}\" target=\"{}\">\n", source, target));
            xml.push_str(&format!("    <data key=\"type\">{:?}</data>\n", edge.edge_type));
            xml.push_str(&format!("    <data key=\"weight\">{}</data>\n", edge.weight));
            xml.push_str("  </edge>\n");
        }

        xml.push_str("</graphml>\n");
        Ok(xml)
    }

    fn count_connected_components(&self) -> usize {
        let mut visited = HashSet::new();
        let mut components = 0;

        for &node_id in self.node_properties.keys() {
            if !visited.contains(&node_id) {
                components += 1;
                self.dfs_visit(node_id, &mut visited);
            }
        }

        components
    }

    fn dfs_visit(&self, node_id: Uuid, visited: &mut HashSet<Uuid>) {
        let mut stack = vec![node_id];
        visited.insert(node_id);

        while let Some(current) = stack.pop() {
            if let Some(neighbors) = self.graph_adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        stack.push(neighbor);
                    }
                }
            }
        }
    }
}

/// Graph statistics
#[derive(Debug, Clone)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_types: HashMap<String, usize>,
    pub average_degree: f64,
    pub connected_components: usize,
}

/// Graph query builder for complex traversals
pub struct GraphQueryBuilder {
    start_node: Option<Uuid>,
    filters: Vec<QueryFilter>,
    max_depth: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum QueryFilter {
    NodeType(NodeType),
    Property(String, PropertyValue),
    EdgeType(EdgeType),
}

impl GraphQueryBuilder {
    pub fn new() -> Self {
        Self {
            start_node: None,
            filters: Vec::new(),
            max_depth: None,
        }
    }

    pub fn from_node(mut self, node_id: Uuid) -> Self {
        self.start_node = Some(node_id);
        self
    }

    pub fn with_filter(mut self, filter: QueryFilter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn execute(&self, indexer: &GraphIndexer) -> Vec<Uuid> {
        // Placeholder - would implement actual graph traversal with filters
        if let Some(start) = self.start_node {
            indexer.get_neighbors(start)
        } else {
            Vec::new()
        }
    }
}
