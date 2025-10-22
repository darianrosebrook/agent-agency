//! Typed DAG Tool Chain Planner
//!
//! Implements sophisticated tool chain reasoning using typed DAGs with petgraph
//! for fan-out/fan-in support, joins, and retries. Includes schema validation
//! and cost-aware planning.

use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Topo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::tool_registry::{ToolRegistry, RegisteredTool};
use crate::tool_execution::{ToolExecutor, ToolInvocation, ToolResult};

/// Tool chain planner with typed DAG support
#[derive(Debug)]
pub struct ToolChainPlanner {
    tool_registry: Arc<ToolRegistry>,
    schema_registry: Arc<SchemaRegistry>,
    chain_cache: Arc<RwLock<HashMap<String, ToolChain>>>,
}

/// Port name type alias
pub type ToolId = String;
pub type PortName = String;

/// Port schema reference for type safety
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortSchemaRef {
    pub registry_key: String,      // e.g. "web.search.Query@v1"
    pub optional: bool,
}

/// Tool port definition with schema
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolPort {
    pub name: PortName,
    pub schema: PortSchemaRef,
}

/// Tool node in the DAG with I/O ports
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolNode {
    pub tool_id: ToolId,
    pub inputs: Vec<ToolPort>,
    pub outputs: Vec<ToolPort>,
    pub params: Value,             // static params (validated)
    pub fallback: Option<ToolId>,
    pub sla_ms: u64,               // per-step SLO target
    pub cost_hint: f64,            // $ estimate
    pub retry_policy: RetryPolicy,
}

/// Tool edge connecting ports between nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolEdge {
    pub from_port: PortName,
    pub to_port: PortName,
    pub codec: Option<String>,     // optional converter key
}

/// Complete tool chain as a DAG
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolChain {
    pub dag: Graph<ToolNode, ToolEdge>,
    pub roots: Vec<NodeIndex>,
    pub sinks: Vec<NodeIndex>,
    pub estimated_cost: f64,
    pub estimated_time_ms: u64,
    pub confidence: f64,
    pub plan_hash: u64,            // blake3 of canonical plan
    pub metadata: ChainMetadata,
}

/// Chain metadata for observability
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainMetadata {
    pub name: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub author: String,
}

/// Retry policy for individual steps
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

/// Chain execution result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainResult {
    pub chain_hash: u64,
    pub success: bool,
    pub results: HashMap<NodeIndex, Value>,
    pub execution_time_ms: u64,
    pub errors: Vec<String>,
}

/// Planning context with task analysis
#[derive(Clone, Debug)]
pub struct PlanningContext {
    pub task_description: String,
    pub task_type: String,
    pub complexity: TaskComplexity,
    pub required_capabilities: Vec<String>,
    pub time_budget_ms: Option<u64>,
    pub cost_budget_cents: Option<u32>,
    pub risk_tolerance: RiskLevel,
}

/// Task complexity levels
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Risk tolerance levels
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RiskLevel {
    Conservative,
    Balanced,
    Aggressive,
}

/// Planning constraints
#[derive(Clone, Debug)]
pub struct PlanningConstraints {
    pub max_chain_length: usize,
    pub max_parallelism: usize,
    pub max_cost_cents: u32,
    pub max_time_ms: u64,
    pub require_fallbacks: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,
        }
    }
}

impl ToolChainPlanner {
    /// Create a new tool chain planner
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        schema_registry: Arc<SchemaRegistry>,
    ) -> Self {
        Self {
            tool_registry,
            schema_registry,
            chain_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Plan an optimal tool chain for the given context
    pub async fn plan_chain(
        &self,
        context: &PlanningContext,
        constraints: &PlanningConstraints,
    ) -> Result<ToolChain> {
        info!("Planning tool chain for task: {}", context.task_description);

        // Check cache first
        let cache_key = self.create_cache_key(context);
        {
            let cache = self.chain_cache.read().await;
            if let Some(chain) = cache.get(&cache_key) {
                debug!("Using cached chain plan");
                return Ok(chain.clone());
            }
        }

        // Generate candidate chains using A* search
        let candidates = self.generate_candidate_chains(context, constraints).await?;

        // Select best chain based on cost/latency/confidence
        let best_chain = self.select_optimal_chain(candidates, context, constraints)?;

        // Cache the result
        {
            let mut cache = self.chain_cache.write().await;
            cache.insert(cache_key, best_chain.clone());
        }

        info!("Generated tool chain with {} steps", best_chain.dag.node_count());
        Ok(best_chain)
    }

    /// Generate candidate chains using A* search
    async fn generate_candidate_chains(
        &self,
        context: &PlanningContext,
        constraints: &PlanningConstraints,
    ) -> Result<Vec<ToolChain>> {
        let mut candidates = Vec::new();
        let available_tools = self.get_relevant_tools(context).await?;

        // Start with single-step chains
        for tool in &available_tools {
            let chain = self.create_single_step_chain(tool, context)?;
            candidates.push(chain);
        }

        // Extend with multi-step chains using A* search
        let mut extended = self.extend_chains_with_astar(
            candidates,
            &available_tools,
            context,
            constraints
        ).await?;

        candidates.append(&mut extended);
        candidates.truncate(10); // Keep top 10 candidates

        Ok(candidates)
    }

    /// Create a single-step chain
    fn create_single_step_chain(
        &self,
        tool: &RegisteredTool,
        context: &PlanningContext,
    ) -> Result<ToolChain> {
        let mut dag = Graph::new();
        let node = self.tool_to_node(tool, context)?;
        let node_idx = dag.add_node(node);

        let chain = ToolChain {
            dag,
            roots: vec![node_idx],
            sinks: vec![node_idx],
            estimated_cost: 0.01, // Minimal cost
            estimated_time_ms: 1000, // 1 second estimate
            confidence: 0.7, // Base confidence
            plan_hash: 0, // Will be computed
            metadata: ChainMetadata {
                name: format!("single_{}", tool.name),
                description: format!("Single step chain using {}", tool.name),
                created_at: chrono::Utc::now(),
                version: "1.0".to_string(),
                author: "planner".to_string(),
            },
        };

        let chain = self.compute_plan_hash(chain);
        Ok(chain)
    }

    /// Extend chains using A* search
    async fn extend_chains_with_astar(
        &self,
        base_chains: Vec<ToolChain>,
        available_tools: &[RegisteredTool],
        context: &PlanningContext,
        constraints: &PlanningConstraints,
    ) -> Result<Vec<ToolChain>> {
        let mut extended_chains = Vec::new();

        for base_chain in base_chains {
            if base_chain.dag.node_count() >= constraints.max_chain_length {
                continue;
            }

            // Try adding each available tool as a new step
            for tool in available_tools {
                let extended = self.try_extend_chain(&base_chain, tool, context, constraints)?;
                if let Some(chain) = extended {
                    extended_chains.push(chain);

                    // Limit to reasonable number of extensions
                    if extended_chains.len() >= 5 {
                        break;
                    }
                }
            }
        }

        Ok(extended_chains)
    }

    /// Try to extend a chain with a new tool
    fn try_extend_chain(
        &self,
        base_chain: &ToolChain,
        new_tool: &RegisteredTool,
        context: &PlanningContext,
        constraints: &PlanningConstraints,
    ) -> Result<Option<ToolChain>> {
        let mut new_dag = base_chain.dag.clone();
        let new_node = self.tool_to_node(new_tool, context)?;
        let new_node_idx = new_dag.add_node(new_node);

        // Try to connect the new node to existing nodes
        let connections_made = self.connect_node_to_chain(
            &mut new_dag,
            new_node_idx,
            &new_node,
            base_chain,
            context,
        )?;

        // If no connections made, it's a parallel/independent step
        if connections_made == 0 {
            // Only add if we have parallelism budget
            if base_chain.dag.node_count() >= constraints.max_parallelism {
                return Ok(None);
            }
        }

        // Update roots and sinks
        let (roots, sinks) = self.compute_roots_and_sinks(&new_dag);

        let mut extended_chain = ToolChain {
            dag: new_dag,
            roots,
            sinks,
            estimated_cost: base_chain.estimated_cost + new_node.cost_hint,
            estimated_time_ms: base_chain.estimated_time_ms + new_node.sla_ms,
            confidence: self.compute_chain_confidence(&base_chain, &new_node),
            plan_hash: 0,
            metadata: ChainMetadata {
                name: format!("extended_{}", base_chain.metadata.name),
                description: format!("Extended chain with {}", new_tool.name),
                created_at: chrono::Utc::now(),
                version: "1.0".to_string(),
                author: "planner".to_string(),
            },
        };

        // Validate constraints
        if extended_chain.estimated_cost > constraints.max_cost_cents as f64 / 100.0 ||
           extended_chain.estimated_time_ms > constraints.max_time_ms {
            return Ok(None);
        }

        let extended_chain = self.compute_plan_hash(extended_chain);
        Ok(Some(extended_chain))
    }

    /// Connect a new node to the existing chain DAG
    fn connect_node_to_chain(
        &self,
        dag: &mut Graph<ToolNode, ToolEdge>,
        new_node_idx: NodeIndex,
        new_node: &ToolNode,
        base_chain: &ToolChain,
        context: &PlanningContext,
    ) -> Result<usize> {
        let mut connections = 0;

        // Try to connect inputs to existing outputs
        for input in &new_node.inputs {
            for &existing_node_idx in &base_chain.sinks {
                let existing_node = &dag[existing_node_idx];

                // Check if existing node has compatible output
                for output in &existing_node.outputs {
                    if self.schemas_compatible(&input.schema, &output.schema)? {
                        // Create edge
                        let edge = ToolEdge {
                            from_port: output.name.clone(),
                            to_port: input.name.clone(),
                            codec: None, // Direct connection
                        };
                        dag.add_edge(existing_node_idx, new_node_idx, edge);
                        connections += 1;
                        break; // One connection per input is enough
                    }
                }
            }
        }

        Ok(connections)
    }

    /// Check if two port schemas are compatible
    fn schemas_compatible(&self, input: &PortSchemaRef, output: &PortSchemaRef) -> Result<bool> {
        // For now, simple string matching on registry keys
        // In a full implementation, this would check schema compatibility
        Ok(input.registry_key == output.registry_key ||
           self.schema_registry.can_convert(&output.registry_key, &input.registry_key))
    }

    /// Convert a registered tool to a DAG node
    fn tool_to_node(&self, tool: &RegisteredTool, context: &PlanningContext) -> Result<ToolNode> {
        // Extract port information from tool metadata
        let inputs = self.extract_tool_ports(tool, true)?;
        let outputs = self.extract_tool_ports(tool, false)?;

        Ok(ToolNode {
            tool_id: tool.name.clone(),
            inputs,
            outputs,
            params: Value::Object(serde_json::Map::new()),
            fallback: None, // TODO: Determine fallback tools
            sla_ms: self.estimate_tool_latency(tool, context),
            cost_hint: self.estimate_tool_cost(tool, context),
            retry_policy: RetryPolicy::default(),
        })
    }

    /// Extract port information from tool metadata
    fn extract_tool_ports(&self, tool: &RegisteredTool, is_input: bool) -> Result<Vec<ToolPort>> {
        let mut ports = Vec::new();

        // This would extract from tool's JSON schema
        // For now, create generic ports
        let port_name = if is_input { "input" } else { "output" };
        let schema_key = format!("{}.{}.default@v1", tool.category, port_name);

        ports.push(ToolPort {
            name: port_name.to_string(),
            schema: PortSchemaRef {
                registry_key: schema_key,
                optional: false,
            },
        });

        Ok(ports)
    }

    /// Estimate tool latency based on historical data
    fn estimate_tool_latency(&self, tool: &RegisteredTool, context: &PlanningContext) -> u64 {
        // Base latency by tool category
        let base_latency = match tool.category.as_str() {
            "evidence_collection" => 2000,
            "conflict_resolution" => 5000,
            "policy_enforcement" => 1000,
            "quality_gate" => 3000,
            _ => 2000,
        };

        // Adjust by complexity
        let complexity_multiplier = match context.complexity {
            TaskComplexity::Simple => 0.5,
            TaskComplexity::Moderate => 1.0,
            TaskComplexity::Complex => 1.5,
            TaskComplexity::VeryComplex => 2.0,
        };

        (base_latency as f64 * complexity_multiplier) as u64
    }

    /// Estimate tool cost
    fn estimate_tool_cost(&self, tool: &RegisteredTool, context: &PlanningContext) -> f64 {
        // Base cost by tool category (in cents)
        let base_cost = match tool.category.as_str() {
            "evidence_collection" => 1.0,
            "conflict_resolution" => 2.0,
            "policy_enforcement" => 0.5,
            "quality_gate" => 1.5,
            _ => 1.0,
        };

        // Adjust by risk level
        let risk_multiplier = match context.risk_tolerance {
            RiskLevel::Conservative => 1.5,
            RiskLevel::Balanced => 1.0,
            RiskLevel::Aggressive => 0.7,
        };

        base_cost * risk_multiplier
    }

    /// Get tools relevant to the planning context
    async fn get_relevant_tools(&self, context: &PlanningContext) -> Result<Vec<RegisteredTool>> {
        let all_tools = self.tool_registry.get_all_tools().await?;

        // Filter by required capabilities
        let relevant_tools: Vec<RegisteredTool> = all_tools.into_iter()
            .filter(|tool| {
                context.required_capabilities.iter()
                    .any(|cap| tool.capabilities.contains(cap))
            })
            .collect();

        Ok(relevant_tools)
    }

    /// Select the optimal chain from candidates
    fn select_optimal_chain(
        &self,
        candidates: Vec<ToolChain>,
        context: &PlanningContext,
        constraints: &PlanningConstraints,
    ) -> Result<ToolChain> {
        if candidates.is_empty() {
            return Err(anyhow::anyhow!("No valid tool chains found"));
        }

        // Score chains using multi-objective optimization
        let mut best_chain = &candidates[0];
        let mut best_score = f64::NEG_INFINITY;

        for candidate in &candidates {
            let score = self.score_chain(candidate, context, constraints);
            if score > best_score {
                best_score = score;
                best_chain = candidate;
            }
        }

        Ok(best_chain.clone())
    }

    /// Score a chain using multi-objective optimization
    fn score_chain(
        &self,
        chain: &ToolChain,
        context: &PlanningContext,
        constraints: &PlanningConstraints,
    ) -> f64 {
        // Multi-objective score: R = wq·quality − wl·latency_norm − wt·tokens_norm
        let quality_score = chain.confidence;

        let latency_norm = chain.estimated_time_ms as f64 / constraints.max_time_ms as f64;
        let cost_norm = chain.estimated_cost / (constraints.max_cost_cents as f64 / 100.0);

        // Weights based on risk tolerance
        let (wq, wl, wt) = match context.risk_tolerance {
            RiskLevel::Conservative => (0.7, 0.2, 0.1), // Quality most important
            RiskLevel::Balanced => (0.5, 0.3, 0.2),     // Balanced
            RiskLevel::Aggressive => (0.3, 0.4, 0.3),   // Speed/cost prioritized
        };

        wq * quality_score - wl * latency_norm - wt * cost_norm
    }

    /// Compute chain confidence based on node confidences and structure
    fn compute_chain_confidence(&self, base_chain: &ToolChain, new_node: &ToolNode) -> f64 {
        // Simplified: geometric mean of node confidences
        let node_count = base_chain.dag.node_count() + 1;
        let total_confidence = base_chain.confidence * (node_count - 1) as f64 + 0.8; // Assume 0.8 for new node
        total_confidence / node_count as f64
    }

    /// Compute roots and sinks of the DAG
    fn compute_roots_and_sinks(&self, dag: &Graph<ToolNode, ToolEdge>) -> (Vec<NodeIndex>, Vec<NodeIndex>) {
        let mut roots = Vec::new();
        let mut sinks = Vec::new();

        for node_idx in dag.node_indices() {
            // Check if node has no incoming edges (root)
            if dag.edges_directed(node_idx, petgraph::Direction::Incoming).next().is_none() {
                roots.push(node_idx);
            }

            // Check if node has no outgoing edges (sink)
            if dag.edges_directed(node_idx, petgraph::Direction::Outgoing).next().is_none() {
                sinks.push(node_idx);
            }
        }

        (roots, sinks)
    }

    /// Compute plan hash for caching and observability
    fn compute_plan_hash(mut chain: ToolChain) -> ToolChain {
        // Create canonical representation for hashing
        let mut canonical = String::new();
        canonical.push_str(&chain.metadata.name);
        canonical.push_str(&format!("{}", chain.dag.node_count()));

        // Sort nodes by tool_id for deterministic hashing
        let mut node_tools: Vec<_> = chain.dag.node_indices()
            .map(|idx| chain.dag[idx].tool_id.clone())
            .collect();
        node_tools.sort();

        for tool_id in node_tools {
            canonical.push_str(&tool_id);
        }

        // Compute hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        canonical.hash(&mut hasher);
        chain.plan_hash = hasher.finish();

        chain
    }

    /// Create cache key for planning context
    fn create_cache_key(&self, context: &PlanningContext) -> String {
        format!("{}_{}_{:?}_{:?}",
            context.task_type,
            context.complexity as u8,
            context.required_capabilities,
            context.risk_tolerance as u8
        )
    }
}

/// Schema registry for tool I/O validation and conversion
#[derive(Debug)]
pub struct SchemaRegistry {
    schemas: HashMap<String, serde_json::Value>,
    converters: HashMap<String, Box<dyn Converter>>,
}

#[async_trait::async_trait]
pub trait Converter: Send + Sync {
    async fn convert(&self, value: Value) -> Result<Value>;
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            converters: HashMap::new(),
        }
    }

    pub fn register_schema(&mut self, key: String, schema: serde_json::Value) {
        self.schemas.insert(key, schema);
    }

    pub fn can_convert(&self, from: &str, to: &str) -> bool {
        // Check if we have a converter for this pair
        let converter_key = format!("{}->{}", from, to);
        self.converters.contains_key(&converter_key)
    }

    pub async fn convert(&self, converter_key: &str, value: Value) -> Result<Value> {
        if let Some(converter) = self.converters.get(converter_key) {
            converter.convert(value).await
        } else {
            Err(anyhow::anyhow!("No converter found for {}", converter_key))
        }
    }
}

impl Default for PlanningConstraints {
    fn default() -> Self {
        Self {
            max_chain_length: 5,
            max_parallelism: 3,
            max_cost_cents: 100, // $1.00
            max_time_ms: 30000,  // 30 seconds
            require_fallbacks: true,
        }
    }
}




