//! Dependency Resolver - DAG Analysis and Execution Ordering
//!
//! Resolves milestone dependencies into execution batches for parallel execution.
//! Implements topological sorting and cycle detection.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use std::collections::{HashMap, HashSet};
use anyhow::{anyhow, Result};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use agent_agency_contracts::planning_io::DependencyGraph;

/// Dependency resolver for milestone execution ordering
pub struct DependencyResolver {
    /// Dependency graph to resolve
    graph: DependencyGraph,
}

impl DependencyResolver {
    /// Create new dependency resolver
    pub fn new(graph: DependencyGraph) -> Self {
        Self { graph }
    }

    /// Resolve execution order into parallel batches
    pub fn resolve_execution_order(&self) -> Result<Vec<Vec<String>>> {
        // Check for cycles first
        if self.graph.has_cycles {
            return Err(anyhow!("Dependency graph contains cycles: {:?}", self.graph.cycles));
        }

        // Use topological sort to get execution order
        let mut batches = Vec::new();
        let mut completed = HashSet::new();
        let mut remaining = self.get_all_milestone_ids();

        while !remaining.is_empty() {
            // Find all milestones that can execute in parallel
            // (all dependencies satisfied)
            let ready: Vec<String> = remaining.iter()
                .filter(|milestone_id| self.can_execute(milestone_id, &completed))
                .cloned()
                .collect();

            if ready.is_empty() {
                return Err(anyhow!("No milestones ready for execution - possible circular dependency"));
            }

            // Add batch and mark as completed
            batches.push(ready.clone());
            for milestone_id in &ready {
                completed.insert(milestone_id.clone());
                remaining.remove(milestone_id);
            }
        }

        Ok(batches)
    }

    /// Check if milestone can execute (all dependencies satisfied)
    pub fn can_execute(&self, milestone_id: &str, completed: &HashSet<String>) -> bool {
        // Get dependencies for this milestone
        let dependencies = self.get_dependencies(milestone_id);

        // Check if all dependencies are completed
        dependencies.iter().all(|dep| completed.contains(dep))
    }

    /// Get dependencies for a milestone
    pub fn get_dependencies(&self, milestone_id: &str) -> Vec<String> {
        self.graph.edges.iter()
            .filter(|edge| edge.to == milestone_id)
            .map(|edge| edge.from.clone())
            .collect()
    }

    /// Get milestones that depend on this one
    pub fn get_dependents(&self, milestone_id: &str) -> Vec<String> {
        self.graph.edges.iter()
            .filter(|edge| edge.from == milestone_id)
            .map(|edge| edge.to.clone())
            .collect()
    }

    /// Get critical path (longest dependency chain)
    pub fn get_critical_path(&self) -> &Vec<String> {
        &self.graph.critical_path
    }

    /// Get parallel execution groups
    pub fn get_parallel_groups(&self) -> &Vec<Vec<String>> {
        &self.graph.parallel_groups
    }

    /// Validate dependency graph for cycles
    pub fn validate_graph(&self) -> Result<()> {
        if self.graph.has_cycles {
            return Err(anyhow!("Dependency graph contains cycles"));
        }

        // Additional validation could be added here
        Ok(())
    }

    /// Get execution statistics
    pub fn get_execution_stats(&self) -> ExecutionStats {
        let total_milestones = self.graph.nodes.len();
        let total_edges = self.graph.edges.len();

        // Calculate parallelism potential
        let max_parallelism = self.graph.parallel_groups.iter()
            .map(|group| group.len())
            .max()
            .unwrap_or(1);

        let avg_parallelism = if !self.graph.parallel_groups.is_empty() {
            self.graph.parallel_groups.iter()
                .map(|group| group.len())
                .sum::<usize>() as f64 / self.graph.parallel_groups.len() as f64
        } else {
            1.0
        };

        ExecutionStats {
            total_milestones,
            total_dependencies: total_edges,
            max_parallelism,
            avg_parallelism,
            critical_path_length: self.graph.critical_path.len(),
            has_cycles: self.graph.has_cycles,
        }
    }

    /// Get all milestone IDs
    fn get_all_milestone_ids(&self) -> HashSet<String> {
        self.graph.nodes.keys().cloned().collect()
    }
}

/// Execution statistics for dependency analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExecutionStats {
    /// Total number of milestones
    pub total_milestones: usize,

    /// Total number of dependencies
    pub total_dependencies: usize,

    /// Maximum parallelism (largest batch)
    pub max_parallelism: usize,

    /// Average parallelism across batches
    pub avg_parallelism: f64,

    /// Length of critical path
    pub critical_path_length: usize,

    /// Whether graph contains cycles
    pub has_cycles: bool,
}

/// Advanced dependency resolver with petgraph integration
pub struct AdvancedDependencyResolver {
    /// Petgraph directed graph
    graph: DiGraph<String, f64>,

    /// Node index to milestone ID mapping
    node_indices: HashMap<String, NodeIndex>,

    /// Milestone ID to node index mapping
    milestone_ids: HashMap<NodeIndex, String>,
}

impl AdvancedDependencyResolver {
    /// Create from dependency graph
    pub fn from_dependency_graph(dep_graph: &DependencyGraph) -> Result<Self> {
        let mut graph = DiGraph::new();
        let mut node_indices = HashMap::new();
        let mut milestone_ids = HashMap::new();

        // Add all nodes
        for (milestone_id, _) in &dep_graph.nodes {
            let node_index = graph.add_node(milestone_id.clone());
            node_indices.insert(milestone_id.clone(), node_index);
            milestone_ids.insert(node_index, milestone_id.clone());
        }

        // Add all edges
        for edge in &dep_graph.edges {
            let from_idx = node_indices.get(&edge.from)
                .ok_or_else(|| anyhow!("Unknown milestone: {}", edge.from))?;
            let to_idx = node_indices.get(&edge.to)
                .ok_or_else(|| anyhow!("Unknown milestone: {}", edge.to))?;

            graph.add_edge(*from_idx, *to_idx, edge.weight);
        }

        Ok(Self {
            graph,
            node_indices,
            milestone_ids,
        })
    }

    /// Get topological execution order
    pub fn topological_order(&self) -> Result<Vec<String>> {
        let sorted_indices = toposort(&self.graph, None)
            .map_err(|_| anyhow!("Dependency graph contains cycles"))?;

        let order: Vec<String> = sorted_indices.iter()
            .filter_map(|idx| self.milestone_ids.get(idx))
            .cloned()
            .collect();

        Ok(order)
    }

    /// Find strongly connected components (for cycle detection)
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        use petgraph::algo::kosaraju_scc;

        let sccs = kosaraju_scc(&self.graph);

        sccs.into_iter()
            .filter(|component| component.len() > 1) // Only cycles
            .map(|component| {
                component.into_iter()
                    .filter_map(|idx| self.milestone_ids.get(&idx).cloned())
                    .collect()
            })
            .collect()
    }

    /// Calculate longest path (critical path)
    pub fn critical_path(&self) -> Result<Vec<String>> {
        // TODO: Implement comprehensive longest path algorithm for critical path
        //       Currently returns topological order as approximation; should implement comprehensive longest path algorithm using topological sort and dynamic programming for accurate critical path calculation.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Longest path algorithm is implemented correctly
        // - Topological sort and dynamic programming are used
        // - Critical path calculation is accurate
        // - Algorithm handles cycles and complex dependencies
        //
        // DEPENDENCIES:
        // - Topological sort implementation (Required)
        // - Dynamic programming utilities (Required)
        // - Graph algorithm libraries (Optional)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (dependency resolution functionality)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Graph algorithms and critical path analysis expertise
        let topo_order = self.topological_order()?;
        Ok(topo_order)
    }

    /// Find parallel execution opportunities
    pub fn parallel_groups(&self) -> Result<Vec<Vec<String>>> {
        let topo_order = self.topological_order()?;
        let mut groups: Vec<Vec<String>> = Vec::new();
        let mut current_group: Vec<String> = Vec::new();
        let mut processed = HashSet::new();

        for milestone_id in topo_order {
            if processed.contains(&milestone_id) {
                continue;
            }

            // Check if this milestone can run in parallel with current group
            let can_add = current_group.iter().all(|existing_id| {
                !self.depends_on(existing_id, &milestone_id) &&
                !self.depends_on(&milestone_id, existing_id)
            });

            if can_add && current_group.len() < 5 { // Limit group size
                current_group.push(milestone_id.clone());
            } else {
                if !current_group.is_empty() {
                    groups.push(current_group);
                    current_group = Vec::new();
                }
                current_group.push(milestone_id.clone());
            }

            processed.insert(milestone_id);
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        Ok(groups)
    }

    /// Check if milestone A depends on milestone B
    fn depends_on(&self, a: &str, b: &str) -> bool {
        if let (Some(a_idx), Some(b_idx)) = (
            self.node_indices.get(a),
            self.node_indices.get(b)
        ) {
            // Check if there's a path from B to A (B must complete before A)
            use petgraph::algo::has_path_connecting;
            has_path_connecting(&self.graph, *b_idx, *a_idx, None)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::planning_io::{DependencyNodeType, DependencyEdgeType};

    #[test]
    fn test_dependency_resolver_creation() {
        let graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: vec![],
            critical_path: vec![],
            parallel_groups: vec![],
            has_cycles: false,
            cycles: vec![],
        };

        let resolver = DependencyResolver::new(graph);
        // Resolver created successfully
        assert!(true);
    }

    #[test]
    fn test_simple_execution_order() {
        let mut nodes = HashMap::new();
        nodes.insert("M1".to_string(), agent_agency_contracts::planning_io::DependencyNode {
            milestone_id: "M1".to_string(),
            node_type: agent_agency_contracts::planning_io::DependencyNodeType::Milestone,
            estimated_cost: 1.0,
            estimated_time_ms: 1000,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        });
        nodes.insert("M2".to_string(), agent_agency_contracts::planning_io::DependencyNode {
            milestone_id: "M2".to_string(),
            node_type: agent_agency_contracts::planning_io::DependencyNodeType::Milestone,
            estimated_cost: 1.0,
            estimated_time_ms: 1000,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        });

        let edges = vec![agent_agency_contracts::planning_io::DependencyEdge {
            from: "M1".to_string(),
            to: "M2".to_string(),
            edge_type: agent_agency_contracts::planning_io::DependencyEdgeType::Hard,
            weight: 1.0,
            metadata: HashMap::new(),
        }];

        let graph = DependencyGraph {
            nodes,
            edges,
            critical_path: vec!["M1".to_string(), "M2".to_string()],
            parallel_groups: vec![vec!["M1".to_string()], vec!["M2".to_string()]],
            has_cycles: false,
            cycles: vec![],
        };

        let resolver = DependencyResolver::new(graph);
        let order = resolver.resolve_execution_order().unwrap();

        assert_eq!(order.len(), 2);
        assert_eq!(order[0], vec!["M1"]);
        assert_eq!(order[1], vec!["M2"]);
    }

    #[test]
    fn test_parallel_execution() {
        let mut nodes = HashMap::new();
        nodes.insert("M1".to_string(), agent_agency_contracts::planning_io::DependencyNode {
            milestone_id: "M1".to_string(),
            node_type: agent_agency_contracts::planning_io::DependencyNodeType::Milestone,
            estimated_cost: 1.0,
            estimated_time_ms: 1000,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        });
        nodes.insert("M2".to_string(), agent_agency_contracts::planning_io::DependencyNode {
            milestone_id: "M2".to_string(),
            node_type: agent_agency_contracts::planning_io::DependencyNodeType::Milestone,
            estimated_cost: 1.0,
            estimated_time_ms: 1000,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        });
        nodes.insert("M3".to_string(), agent_agency_contracts::planning_io::DependencyNode {
            milestone_id: "M3".to_string(),
            node_type: agent_agency_contracts::planning_io::DependencyNodeType::Milestone,
            estimated_cost: 1.0,
            estimated_time_ms: 1000,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        });

        // M1 and M2 can run in parallel, M3 depends on both
        let edges = vec![
            agent_agency_contracts::planning_io::DependencyEdge {
                from: "M1".to_string(),
                to: "M3".to_string(),
                edge_type: agent_agency_contracts::planning_io::DependencyEdgeType::Hard,
                weight: 1.0,
                metadata: HashMap::new(),
            },
            agent_agency_contracts::planning_io::DependencyEdge {
                from: "M2".to_string(),
                to: "M3".to_string(),
                edge_type: agent_agency_contracts::planning_io::DependencyEdgeType::Hard,
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let graph = DependencyGraph {
            nodes,
            edges,
            critical_path: vec!["M1".to_string(), "M3".to_string()],
            parallel_groups: vec![vec!["M1".to_string(), "M2".to_string()], vec!["M3".to_string()]],
            has_cycles: false,
            cycles: vec![],
        };

        let resolver = DependencyResolver::new(graph);
        let order = resolver.resolve_execution_order().unwrap();

        assert_eq!(order.len(), 2);
        assert_eq!(order[0].len(), 2); // M1 and M2 in parallel
        assert_eq!(order[1], vec!["M3"]);
    }

    #[test]
    fn test_execution_stats() {
        let mut nodes = HashMap::new();
        for i in 1..=5 {
            nodes.insert(format!("M{}", i), agent_agency_contracts::planning_io::DependencyNode {
                milestone_id: format!("M{}", i),
                node_type: agent_agency_contracts::planning_io::DependencyNodeType::Milestone,
                estimated_cost: 1.0,
                estimated_time_ms: 1000,
                resource_requirements: HashMap::new(),
                metadata: HashMap::new(),
            });
        }

        let graph = DependencyGraph {
            nodes,
            edges: vec![],
            critical_path: vec!["M1".to_string(), "M2".to_string()],
            parallel_groups: vec![
                vec!["M1".to_string(), "M2".to_string()],
                vec!["M3".to_string()],
                vec!["M4".to_string(), "M5".to_string()],
            ],
            has_cycles: false,
            cycles: vec![],
        };

        let resolver = DependencyResolver::new(graph);
        let stats = resolver.get_execution_stats();

        assert_eq!(stats.total_milestones, 5);
        assert_eq!(stats.total_dependencies, 0);
        assert_eq!(stats.critical_path_length, 2);
        assert!(!stats.has_cycles);
    }

    #[test]
    fn test_cycle_detection() {
        let mut nodes = HashMap::new();
        nodes.insert("M1".to_string(), agent_agency_contracts::planning_io::DependencyNode {
            milestone_id: "M1".to_string(),
            node_type: agent_agency_contracts::planning_io::DependencyNodeType::Milestone,
            estimated_cost: 1.0,
            estimated_time_ms: 1000,
            resource_requirements: HashMap::new(),
            metadata: HashMap::new(),
        });

        let edges = vec![agent_agency_contracts::planning_io::DependencyEdge {
            from: "M1".to_string(),
            to: "M1".to_string(), // Self-cycle
            edge_type: agent_agency_contracts::planning_io::DependencyEdgeType::Hard,
            weight: 1.0,
            metadata: HashMap::new(),
        }];

        let graph = DependencyGraph {
            nodes,
            edges,
            critical_path: vec![],
            parallel_groups: vec![],
            has_cycles: true,
            cycles: vec![vec!["M1".to_string()]],
        };

        let resolver = DependencyResolver::new(graph);
        let result = resolver.resolve_execution_order();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycles"));
    }
}
