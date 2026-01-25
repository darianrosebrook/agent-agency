//! Operator Graph with Cycle Detection
//!
//! Implements INV-CORE-07: Termination Guarantee - All loops must have
//! bounded iteration limits.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use v4_types::OperatorType;

/// A node in the operator graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorNode {
    /// Unique node identifier
    pub id: String,
    /// The operator at this node
    pub operator: OperatorType,
    /// IDs of nodes this node depends on (edges pointing to this node)
    pub depends_on: Vec<String>,
    /// IDs of nodes that depend on this node (edges pointing from this node)
    pub dependents: Vec<String>,
    /// Execution order (computed via topological sort)
    pub order: Option<usize>,
    /// Depth in the graph (for bounding)
    pub depth: u32,
}

impl OperatorNode {
    /// Create a new operator node
    pub fn new(id: impl Into<String>, operator: OperatorType) -> Self {
        Self {
            id: id.into(),
            operator,
            depends_on: Vec::new(),
            dependents: Vec::new(),
            order: None,
            depth: 0,
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, node_id: impl Into<String>) {
        self.depends_on.push(node_id.into());
    }

    /// Check if this node has dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.depends_on.is_empty()
    }
}

/// Operator graph for execution planning
///
/// This graph enforces INV-CORE-07 by having a bounded max_depth
/// that prevents infinite loops.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorGraph {
    /// All nodes in the graph
    nodes: HashMap<String, OperatorNode>,
    /// Maximum allowed depth (INV-CORE-07: bounded iterations)
    max_depth: u32,
    /// Root node IDs (nodes with no dependencies)
    roots: Vec<String>,
    /// Whether the graph has been validated
    validated: bool,
}

impl OperatorGraph {
    /// Default maximum depth
    pub const DEFAULT_MAX_DEPTH: u32 = 100;

    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            max_depth: Self::DEFAULT_MAX_DEPTH,
            roots: Vec::new(),
            validated: false,
        }
    }

    /// Create with a custom max depth
    pub fn with_max_depth(max_depth: u32) -> Self {
        Self {
            nodes: HashMap::new(),
            max_depth,
            roots: Vec::new(),
            validated: false,
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: OperatorNode) -> Result<(), GraphError> {
        if self.nodes.contains_key(&node.id) {
            return Err(GraphError::DuplicateNode(node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        self.validated = false;
        Ok(())
    }

    /// Add an edge (dependency) between nodes
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<(), GraphError> {
        if !self.nodes.contains_key(from) {
            return Err(GraphError::NodeNotFound(from.to_string()));
        }
        if !self.nodes.contains_key(to) {
            return Err(GraphError::NodeNotFound(to.to_string()));
        }

        // Add the dependency
        if let Some(node) = self.nodes.get_mut(to) {
            if !node.depends_on.contains(&from.to_string()) {
                node.depends_on.push(from.to_string());
            }
        }

        // Add the reverse reference
        if let Some(node) = self.nodes.get_mut(from) {
            if !node.dependents.contains(&to.to_string()) {
                node.dependents.push(to.to_string());
            }
        }

        self.validated = false;
        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&OperatorNode> {
        self.nodes.get(id)
    }

    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &OperatorNode> {
        self.nodes.values()
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Detect cycles in the graph
    ///
    /// Uses DFS-based cycle detection. Returns the first cycle found
    /// as a list of node IDs.
    pub fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if let Some(cycle) =
                    self.dfs_detect_cycle(node_id, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle);
                }
            }
        }

        None
    }

    fn dfs_detect_cycle(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());
        path.push(node_id.to_string());

        if let Some(node) = self.nodes.get(node_id) {
            for dep in &node.dependents {
                if !visited.contains(dep) {
                    if let Some(cycle) = self.dfs_detect_cycle(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    // Found a cycle - extract it from the path
                    let cycle_start = path.iter().position(|n| n == dep).unwrap();
                    let mut cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycle.push(dep.clone()); // Complete the cycle
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node_id);
        None
    }

    /// Compute depths for all nodes
    fn compute_depths(&mut self) -> Result<(), GraphError> {
        // Find roots (nodes with no dependencies)
        self.roots.clear();
        for (id, node) in &self.nodes {
            if node.depends_on.is_empty() {
                self.roots.push(id.clone());
            }
        }

        if self.roots.is_empty() && !self.nodes.is_empty() {
            return Err(GraphError::NoRoots);
        }

        // BFS to compute depths
        let mut queue: std::collections::VecDeque<(String, u32)> = self
            .roots
            .iter()
            .map(|id| (id.clone(), 0))
            .collect();

        let mut visited = HashSet::new();

        while let Some((node_id, depth)) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            if depth > self.max_depth {
                return Err(GraphError::MaxDepthExceeded {
                    node_id,
                    depth,
                    max_depth: self.max_depth,
                });
            }

            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.depth = depth;
                for dep in node.dependents.clone() {
                    queue.push_back((dep, depth + 1));
                }
            }
        }

        Ok(())
    }

    /// Validate the graph
    ///
    /// Checks for:
    /// - Cycles (INV-CORE-07 violation)
    /// - Max depth exceeded
    /// - Disconnected nodes
    pub fn validate(&mut self) -> Result<ValidationReport, GraphError> {
        let mut report = ValidationReport::new();

        // Check for cycles
        if let Some(cycle) = self.detect_cycle() {
            return Err(GraphError::CycleDetected(cycle));
        }
        report.cycle_free = true;

        // Compute depths and check bounds
        self.compute_depths()?;
        report.depths_computed = true;

        // Check for disconnected nodes
        let mut reachable = HashSet::new();
        let mut queue: std::collections::VecDeque<String> = self.roots.iter().cloned().collect();

        while let Some(node_id) = queue.pop_front() {
            if reachable.contains(&node_id) {
                continue;
            }
            reachable.insert(node_id.clone());

            if let Some(node) = self.nodes.get(&node_id) {
                for dep in &node.dependents {
                    queue.push_back(dep.clone());
                }
            }
        }

        report.disconnected_nodes = self
            .nodes
            .keys()
            .filter(|id| !reachable.contains(*id))
            .cloned()
            .collect();

        self.validated = true;
        Ok(report)
    }

    /// Get topological order of nodes
    ///
    /// Returns nodes in an order where dependencies come before dependents.
    pub fn topological_order(&self) -> Result<Vec<String>, GraphError> {
        if !self.validated {
            return Err(GraphError::NotValidated);
        }

        let mut result = Vec::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Calculate in-degrees
        for (id, node) in &self.nodes {
            in_degree.insert(id.clone(), node.depends_on.len());
        }

        // Start with nodes that have no dependencies
        let mut queue: std::collections::VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        while let Some(node_id) = queue.pop_front() {
            result.push(node_id.clone());

            if let Some(node) = self.nodes.get(&node_id) {
                for dep in &node.dependents {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            // This shouldn't happen if validation passed
            return Err(GraphError::CycleDetected(vec![]));
        }

        Ok(result)
    }

    /// Get maximum depth in the graph
    pub fn max_actual_depth(&self) -> u32 {
        self.nodes.values().map(|n| n.depth).max().unwrap_or(0)
    }

    /// Create a linear graph from a sequence of operators
    pub fn from_sequence(operators: &[OperatorType]) -> Result<Self, GraphError> {
        let mut graph = Self::new();

        let mut prev_id: Option<String> = None;
        for (i, op) in operators.iter().enumerate() {
            let node_id = format!("node-{}", i);
            let node = OperatorNode::new(&node_id, op.clone());
            graph.add_node(node)?;

            if let Some(ref prev) = prev_id {
                graph.add_edge(prev, &node_id)?;
            }

            prev_id = Some(node_id);
        }

        graph.validate()?;
        Ok(graph)
    }
}

impl Default for OperatorGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Report from graph validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether the graph is cycle-free
    pub cycle_free: bool,
    /// Whether depths were successfully computed
    pub depths_computed: bool,
    /// List of disconnected node IDs
    pub disconnected_nodes: Vec<String>,
}

impl ValidationReport {
    fn new() -> Self {
        Self {
            cycle_free: false,
            depths_computed: false,
            disconnected_nodes: Vec::new(),
        }
    }

    /// Check if the graph is fully valid
    pub fn is_valid(&self) -> bool {
        self.cycle_free && self.depths_computed && self.disconnected_nodes.is_empty()
    }
}

/// Errors related to the operator graph
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Duplicate node ID: {0}")]
    DuplicateNode(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Cycle detected: {0:?}")]
    CycleDetected(Vec<String>),

    #[error("Max depth exceeded at node {node_id}: depth {depth} > max {max_depth}")]
    MaxDepthExceeded {
        node_id: String,
        depth: u32,
        max_depth: u32,
    },

    #[error("Graph has no root nodes")]
    NoRoots,

    #[error("Graph not validated - call validate() first")]
    NotValidated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::operators::SeekOp;

    fn make_seek_op(path: &str) -> OperatorType {
        OperatorType::Seek(SeekOp::ReadFile {
            path: path.to_string(),
        })
    }

    #[test]
    fn test_empty_graph() {
        let graph = OperatorGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut graph = OperatorGraph::new();

        let node1 = OperatorNode::new("n1", make_seek_op("a.rs"));
        let node2 = OperatorNode::new("n2", make_seek_op("b.rs"));

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_duplicate_node_error() {
        let mut graph = OperatorGraph::new();

        let node1 = OperatorNode::new("n1", make_seek_op("a.rs"));
        let node2 = OperatorNode::new("n1", make_seek_op("b.rs"));

        graph.add_node(node1).unwrap();
        assert!(matches!(
            graph.add_node(node2),
            Err(GraphError::DuplicateNode(_))
        ));
    }

    #[test]
    fn test_add_edges() {
        let mut graph = OperatorGraph::new();

        graph
            .add_node(OperatorNode::new("n1", make_seek_op("a.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n2", make_seek_op("b.rs")))
            .unwrap();

        graph.add_edge("n1", "n2").unwrap();

        let n2 = graph.get_node("n2").unwrap();
        assert!(n2.depends_on.contains(&"n1".to_string()));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = OperatorGraph::new();

        graph
            .add_node(OperatorNode::new("n1", make_seek_op("a.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n2", make_seek_op("b.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n3", make_seek_op("c.rs")))
            .unwrap();

        graph.add_edge("n1", "n2").unwrap();
        graph.add_edge("n2", "n3").unwrap();
        graph.add_edge("n3", "n1").unwrap(); // Creates cycle

        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
    }

    #[test]
    fn test_no_cycle() {
        let mut graph = OperatorGraph::new();

        graph
            .add_node(OperatorNode::new("n1", make_seek_op("a.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n2", make_seek_op("b.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n3", make_seek_op("c.rs")))
            .unwrap();

        graph.add_edge("n1", "n2").unwrap();
        graph.add_edge("n2", "n3").unwrap();

        let cycle = graph.detect_cycle();
        assert!(cycle.is_none());
    }

    #[test]
    fn test_validation() {
        let mut graph = OperatorGraph::new();

        graph
            .add_node(OperatorNode::new("n1", make_seek_op("a.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n2", make_seek_op("b.rs")))
            .unwrap();

        graph.add_edge("n1", "n2").unwrap();

        let report = graph.validate().unwrap();
        assert!(report.is_valid());
    }

    #[test]
    fn test_max_depth_exceeded() {
        let mut graph = OperatorGraph::with_max_depth(2);

        // Create a chain of 5 nodes
        for i in 0..5 {
            graph
                .add_node(OperatorNode::new(format!("n{}", i), make_seek_op("a.rs")))
                .unwrap();
            if i > 0 {
                graph
                    .add_edge(&format!("n{}", i - 1), &format!("n{}", i))
                    .unwrap();
            }
        }

        let result = graph.validate();
        assert!(matches!(result, Err(GraphError::MaxDepthExceeded { .. })));
    }

    #[test]
    fn test_topological_order() {
        let mut graph = OperatorGraph::new();

        graph
            .add_node(OperatorNode::new("n1", make_seek_op("a.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n2", make_seek_op("b.rs")))
            .unwrap();
        graph
            .add_node(OperatorNode::new("n3", make_seek_op("c.rs")))
            .unwrap();

        graph.add_edge("n1", "n2").unwrap();
        graph.add_edge("n1", "n3").unwrap();
        graph.add_edge("n2", "n3").unwrap();

        graph.validate().unwrap();

        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 3);

        // n1 must come before n2 and n3
        let pos_n1 = order.iter().position(|n| n == "n1").unwrap();
        let pos_n2 = order.iter().position(|n| n == "n2").unwrap();
        let pos_n3 = order.iter().position(|n| n == "n3").unwrap();

        assert!(pos_n1 < pos_n2);
        assert!(pos_n1 < pos_n3);
        assert!(pos_n2 < pos_n3);
    }

    #[test]
    fn test_from_sequence() {
        let operators = vec![make_seek_op("a.rs"), make_seek_op("b.rs"), make_seek_op("c.rs")];

        let graph = OperatorGraph::from_sequence(&operators).unwrap();
        assert_eq!(graph.node_count(), 3);
        assert!(graph.validated);
    }
}
