//! Graph Algorithms for Planning
//!
//! Shared graph algorithms for critical path calculation and parallel group identification.
//! These algorithms are used across multiple planning modules for consistent dependency analysis.
//!
//! @author @darianrosebrook

use agent_agency_contracts::planning_io::{DependencyEdge, DependencyNode};
use anyhow::Result;
use petgraph::graph::NodeIndex;
use petgraph::{algo, Direction, Graph};
use std::collections::HashMap;
use tracing::warn;

/// Calculate critical path through dependency graph using longest path algorithm (CPM - Critical Path Method)
///
/// The critical path is the longest path through the dependency graph, representing
/// the minimum time required to complete all tasks.
///
/// # Arguments
/// * `nodes` - Map of node IDs to dependency nodes
/// * `edges` - List of dependency edges between nodes
///
/// # Returns
/// Vector of node IDs representing the critical path from source to sink
///
/// # Algorithm
/// 1. Build weighted graph from nodes and edges
/// 2. Find source nodes (nodes with no incoming edges)
/// 3. Calculate longest path from each source using topological sort + dynamic programming
/// 4. Return the path with maximum total weight
pub fn calculate_critical_path(
    nodes: &HashMap<String, DependencyNode>,
    edges: &[DependencyEdge],
) -> Result<Vec<String>> {
    // Build graph from nodes and edges
    let mut graph = Graph::<String, f64>::new();
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Add nodes to graph
    for (node_id, _node) in nodes {
        let idx = graph.add_node(node_id.clone());
        node_indices.insert(node_id.clone(), idx);
    }

    // Add edges with weights (estimated_time_ms as weight)
    for edge in edges {
        if let (Some(&from_idx), Some(&to_idx)) =
            (node_indices.get(&edge.from), node_indices.get(&edge.to))
        {
            // Use estimated_time_ms from target node as edge weight
            let weight = nodes
                .get(&edge.to)
                .map(|n| n.estimated_time_ms as f64)
                .unwrap_or(edge.weight);
            graph.add_edge(from_idx, to_idx, weight);
        }
    }

    // Find source nodes (nodes with no incoming edges)
    let source_nodes: Vec<NodeIndex> = graph
        .node_indices()
        .filter(|&idx| {
            graph
                .edges_directed(idx, Direction::Incoming)
                .next()
                .is_none()
        })
        .collect();

    if source_nodes.is_empty() {
        // No source nodes - return empty path or all nodes if no dependencies
        if nodes.is_empty() {
            return Ok(vec![]);
        }
        // If no source nodes but nodes exist, there might be cycles
        // Return first node as fallback
        return Ok(nodes.keys().take(1).cloned().collect());
    }

    // Calculate longest path from each source node
    let mut max_path_length = 0.0;
    let mut critical_path = Vec::new();

    for &source_idx in &source_nodes {
        // Use DFS to find longest path from this source
        let path = find_longest_path(&graph, source_idx, &node_indices);
        let path_length: f64 = path
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, node_id)| {
                if let Some(&prev_idx) = node_indices.get(&path[i - 1]) {
                    if let Some(&curr_idx) = node_indices.get(node_id) {
                        if let Some(edge) = graph.find_edge(prev_idx, curr_idx) {
                            return graph[edge];
                        }
                    }
                }
                0.0
            })
            .sum();

        if path_length > max_path_length {
            max_path_length = path_length;
            critical_path = path;
        }
    }

    Ok(critical_path)
}

/// Find longest path from a source node using topological sort and dynamic programming
fn find_longest_path(
    graph: &Graph<String, f64>,
    source: NodeIndex,
    _node_indices: &HashMap<String, NodeIndex>,
) -> Vec<String> {
    // Use topological sort to ensure we process nodes in dependency order
    let topo = match algo::toposort(graph, None) {
        Ok(order) => order,
        Err(_) => {
            // Cycle detected - return path with just source node
            return vec![graph[source].clone()];
        }
    };

    // Find source position in topological order
    let source_pos = topo.iter().position(|&idx| idx == source);
    if source_pos.is_none() {
        return vec![graph[source].clone()];
    }

    // Calculate longest distances from source using dynamic programming
    let mut distances: HashMap<NodeIndex, (f64, Option<NodeIndex>)> = HashMap::new();
    distances.insert(source, (0.0, None));

    // Process nodes in topological order starting from source
    for &node_idx in topo.iter().skip(source_pos.unwrap()) {
        let current_dist = distances.get(&node_idx).map(|(d, _)| *d).unwrap_or(0.0);

        // Update distances for neighbors
        for neighbor in graph.neighbors_directed(node_idx, Direction::Outgoing) {
            if let Some(edge) = graph.find_edge(node_idx, neighbor) {
                let edge_weight = graph[edge];
                let new_dist = current_dist + edge_weight;

                let should_update = distances
                    .get(&neighbor)
                    .map(|(d, _)| new_dist > *d)
                    .unwrap_or(true);

                if should_update {
                    distances.insert(neighbor, (new_dist, Some(node_idx)));
                }
            }
        }
    }

    // Find node with maximum distance (end of critical path)
    let (end_node, _) = distances
        .iter()
        .max_by(|(_, (d1, _)), (_, (d2, _))| {
            d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or((&source, &(0.0, None)));

    // Reconstruct path from source to end_node
    let mut path = Vec::new();
    let mut current = *end_node;

    while let Some((_, prev)) = distances.get(&current) {
        path.push(graph[current].clone());
        if let Some(prev_idx) = prev {
            current = *prev_idx;
        } else {
            break;
        }
    }

    path.reverse();
    path
}

/// Identify parallel execution groups using topological sort levels
///
/// Groups nodes by dependency level - nodes at the same level can execute in parallel.
///
/// # Arguments
/// * `nodes` - Map of node IDs to dependency nodes
/// * `edges` - List of dependency edges between nodes
///
/// # Returns
/// Vector of groups, where each group contains node IDs that can execute in parallel
///
/// # Algorithm
/// 1. Build dependency graph from nodes and edges
/// 2. Perform topological sort to get dependency order
/// 3. Group nodes by dependency level (nodes at same level have no dependencies between them)
/// 4. Return groups ready for parallel execution
pub fn identify_parallel_groups(
    nodes: &HashMap<String, DependencyNode>,
    edges: &[DependencyEdge],
) -> Result<Vec<Vec<String>>> {
    // Build graph from nodes and edges
    let mut graph = Graph::<String, ()>::new();
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Add nodes to graph
    for (node_id, _node) in nodes {
        let idx = graph.add_node(node_id.clone());
        node_indices.insert(node_id.clone(), idx);
    }

    // Add edges
    for edge in edges {
        if let (Some(&from_idx), Some(&to_idx)) =
            (node_indices.get(&edge.from), node_indices.get(&edge.to))
        {
            graph.add_edge(from_idx, to_idx, ());
        }
    }

    // Perform topological sort to get dependency levels
    let topo_order = match algo::toposort(&graph, None) {
        Ok(order) => order,
        Err(_) => {
            // Cycle detected - return all nodes as single group
            warn!("Cycle detected in dependency graph - cannot identify parallel groups");
            return Ok(vec![nodes.keys().cloned().collect()]);
        }
    };

    // Group nodes by dependency level (nodes at same level can run in parallel)
    // Level 0: nodes with no dependencies
    // Level N: nodes that depend on nodes at level N-1
    let mut levels: Vec<Vec<String>> = Vec::new();
    let mut node_levels: HashMap<NodeIndex, usize> = HashMap::new();

    for &node_idx in &topo_order {
        // Find maximum level of dependencies
        // Iterate over all nodes to find which ones have edges pointing to this node
        let max_dep_level = graph
            .node_indices()
            .filter_map(|other_idx| {
                // Check if there's an edge from other_idx to node_idx
                if graph.find_edge(other_idx, node_idx).is_some() {
                    node_levels.get(&other_idx).copied()
                } else {
                    None
                }
            })
            .max()
            .map(|l| l + 1)
            .unwrap_or(0);

        node_levels.insert(node_idx, max_dep_level);

        // Ensure levels vector is large enough
        while levels.len() <= max_dep_level {
            levels.push(Vec::new());
        }

        levels[max_dep_level].push(graph[node_idx].clone());
    }

    // Filter out empty levels
    let parallel_groups: Vec<Vec<String>> = levels
        .into_iter()
        .filter(|group| !group.is_empty())
        .collect();

    Ok(parallel_groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::planning_io::{DependencyEdgeType, DependencyNodeType};

    fn create_test_node(id: &str, time_ms: u64) -> DependencyNode {
        DependencyNode {
            milestone_id: id.to_string(),
            node_type: DependencyNodeType::Milestone,
            estimated_cost: time_ms as f64 / 3600000.0,
            estimated_time_ms: time_ms,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_edge(from: &str, to: &str, weight: f64) -> DependencyEdge {
        DependencyEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type: DependencyEdgeType::Hard,
            weight,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_critical_path_linear() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), create_test_node("A", 100));
        nodes.insert("B".to_string(), create_test_node("B", 200));
        nodes.insert("C".to_string(), create_test_node("C", 300));

        let edges = vec![
            create_test_edge("A", "B", 1.0),
            create_test_edge("B", "C", 1.0),
        ];

        let critical_path = calculate_critical_path(&nodes, &edges).unwrap();
        assert_eq!(critical_path, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_critical_path_branching() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), create_test_node("A", 100));
        nodes.insert("B".to_string(), create_test_node("B", 200));
        nodes.insert("C".to_string(), create_test_node("C", 500)); // Longer path
        nodes.insert("D".to_string(), create_test_node("D", 300));

        let edges = vec![
            create_test_edge("A", "B", 1.0),
            create_test_edge("A", "C", 1.0),
            create_test_edge("B", "D", 1.0),
            create_test_edge("C", "D", 1.0),
        ];

        let critical_path = calculate_critical_path(&nodes, &edges).unwrap();
        // Critical path should be A -> C -> D (longest path)
        assert!(critical_path.contains(&"A".to_string()));
        assert!(critical_path.contains(&"C".to_string()));
        assert!(critical_path.contains(&"D".to_string()));
    }

    #[test]
    fn test_parallel_groups_simple() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), create_test_node("A", 100));
        nodes.insert("B".to_string(), create_test_node("B", 200));
        nodes.insert("C".to_string(), create_test_node("C", 300));

        let edges = vec![
            create_test_edge("A", "B", 1.0),
            create_test_edge("A", "C", 1.0),
        ];

        let groups = identify_parallel_groups(&nodes, &edges).unwrap();
        // Level 0: A
        // Level 1: B, C (can run in parallel)
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec!["A"]);
        assert!(groups[1].contains(&"B".to_string()));
        assert!(groups[1].contains(&"C".to_string()));
    }

    #[test]
    fn test_parallel_groups_independent() {
        let mut nodes = HashMap::new();
        nodes.insert("A".to_string(), create_test_node("A", 100));
        nodes.insert("B".to_string(), create_test_node("B", 200));
        nodes.insert("C".to_string(), create_test_node("C", 300));

        let edges = vec![]; // No dependencies

        let groups = identify_parallel_groups(&nodes, &edges).unwrap();
        // All nodes can run in parallel
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }
}
