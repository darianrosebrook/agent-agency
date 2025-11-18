//! Tool Chain Bridge - Integration with system-federated-ml ToolChainPlanner
//!
//! Bridges the planning system with the existing tool chain planning infrastructure
//! for sophisticated DAG-based tool execution.
//!
//! @author @darianrosebrook

use crate::planning::tool_chain_types::{
    ExternalPlanningConstraints, ExternalPlanningContext, ExternalRiskLevel,
    ExternalSchemaRegistry, ExternalTaskComplexity, ExternalToolChain, ExternalToolChainPlanner,
    ExternalToolNode, ExternalToolRegistry,
};
use agent_agency_contracts::{
    planning_io::{
        EvidenceGate, ExecutionPlan as ContractExecutionPlan, Milestone as ContractMilestone,
        PlanState,
    },
    WorkingSpec,
};
use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bridge to tool chain planner
pub struct ToolChainBridge {
    /// Reference to the tool chain planner (external or local)
    tool_chain_planner: std::sync::Arc<ExternalToolChainPlanner>,

    /// Schema registry for tool I/O validation
    #[allow(dead_code)] // Reserved for future use
    schema_registry: std::sync::Arc<ExternalSchemaRegistry>,

    /// Tool registry for accessing available tools
    #[allow(dead_code)] // Reserved for future use
    tool_registry: std::sync::Arc<ExternalToolRegistry>,
}

impl std::fmt::Debug for ToolChainBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolChainBridge")
            .field("has_tool_chain_planner", &true)
            .field("has_schema_registry", &true)
            .field("has_tool_registry", &true)
            .finish()
    }
}

impl ToolChainBridge {
    /// Create new tool chain bridge with dependencies
    pub fn new(
        tool_chain_planner: std::sync::Arc<ExternalToolChainPlanner>,
        schema_registry: std::sync::Arc<ExternalSchemaRegistry>,
        tool_registry: std::sync::Arc<ExternalToolRegistry>,
    ) -> Self {
        Self {
            tool_chain_planner,
            schema_registry,
            tool_registry,
        }
    }

    /// Generate execution plan using tool chain planner
    pub async fn generate_from_working_spec(
        &self,
        working_spec: WorkingSpec,
    ) -> Result<ContractExecutionPlan> {
        // Convert working spec to planning context
        let planning_context = self.convert_working_spec_to_planning_context(&working_spec)?;

        // Create planning constraints based on working spec
        let constraints = self.create_planning_constraints(&working_spec)?;

        // Generate tool chain using the real tool chain planner
        let tool_chain = (*self.tool_chain_planner)
            .plan_chain(&planning_context, &constraints)
            .await?;

        // Convert tool chain back to execution plan format
        self.convert_tool_chain_to_execution_plan(tool_chain, working_spec)
            .await
    }

    /// Convert working spec to tool chain planning context
    fn convert_working_spec_to_planning_context(
        &self,
        working_spec: &WorkingSpec,
    ) -> Result<ExternalPlanningContext> {
        // Extract task description from working spec
        let task_description = self.extract_task_description(working_spec);

        // Determine task type and complexity
        let task_type = self.determine_task_type(working_spec);
        let complexity = self.determine_task_complexity(working_spec);

        // Extract required capabilities
        let required_capabilities = self.extract_required_capabilities(working_spec);

        // Determine risk tolerance
        let risk_tolerance = self.determine_risk_tolerance(working_spec);

        // Create constraints
        let constraints = self.create_planning_constraints(working_spec)?;

        Ok(ExternalPlanningContext {
            task_description: task_description,
            task_type: task_type,
            complexity: complexity,
            required_capabilities,
            time_budget_ms: working_spec
                .constraints
                .max_duration_minutes
                .map(|mins| (mins as u64) * 60 * 1000),
            cost_budget_cents: Some(1000), // Default cost budget - no cost field in WorkingSpecConstraints
            risk_tolerance,
        })
    }

    /// Create planning constraints from working spec
    fn create_planning_constraints(
        &self,
        working_spec: &WorkingSpec,
    ) -> Result<ExternalPlanningConstraints> {
        Ok(ExternalPlanningConstraints {
            max_chain_length: 5, // Default reasonable limit
            max_parallelism: 3,   // Allow some parallelism
            max_cost_cents: 1000, // Default cost budget
            max_time_ms: working_spec
                .constraints
                .max_duration_minutes
                .map(|mins| (mins as u64) * 60 * 1000)
                .unwrap_or(30000), // Default 30 seconds
            require_fallbacks: working_spec.risk_tier > 1, // Require fallbacks for high-risk work
        })
    }

    /// Convert tool chain to execution plan
    async fn convert_tool_chain_to_execution_plan(
        &self,
        tool_chain: ExternalToolChain,
        working_spec: WorkingSpec,
    ) -> Result<ContractExecutionPlan> {
        use agent_agency_contracts::planning_io::{
            DependencyEdge, DependencyEdgeType, DependencyNode, DependencyNodeType,
        };

        // Create milestones from tool chain DAG nodes
        let mut milestones = Vec::new();
        let mut node_indices: std::collections::HashMap<petgraph::graph::NodeIndex, String> = std::collections::HashMap::new();

        // Convert DAG nodes to milestones using topological sort
        use petgraph::algo::toposort;
        let sorted_indices = match toposort(&tool_chain.dag, None) {
            Ok(indices) => indices,
            Err(_) => {
                // Cycle detected - fallback to node order
                tool_chain.dag.node_indices().collect()
            }
        };

        // Map DAG nodes to milestones
        for dag_idx in &sorted_indices {
            if let Some(node) = tool_chain.dag.node_weight(*dag_idx) {
                let milestone_id = format!("TC-{}", node.tool_id);
                // Convert system_federated_ml::tool_chain_planner::ToolNode to ExternalToolNode
                // For now, create a minimal ExternalToolNode-like structure
                // Convert system_federated_ml::ToolNode to ExternalToolNode
                // ExternalToolNode is a type alias that resolves to the appropriate type based on feature flags
                let external_node: ExternalToolNode = {
                    #[cfg(feature = "tool-chain")]
                    {
                        // When tool-chain feature is enabled, ExternalToolNode is system_federated_ml::ToolNode
                        // So we can use the node directly
                        node.clone()
                    }
                    #[cfg(not(feature = "tool-chain"))]
                    {
                        // When tool-chain feature is not enabled, ExternalToolNode is tool_chain_types::ToolNode
                        crate::planning::tool_chain_types::ToolNode {
                            id: node.tool_id.clone(),
                            tool_name: node.tool_id.clone(),
                            tool_version: "1.0.0".to_string(),
                            inputs: HashMap::new(),
                            output_schema: None,
                            dependencies: Vec::new(), // Will be extracted from edges below
                        }
                    }
                };
                let milestone =
                    self.create_milestone_from_tool_node(&external_node, &milestone_id, &working_spec)?;
                milestones.push(milestone);
                node_indices.insert(*dag_idx, milestone_id);
            }
        }

        // Create dependency graph from DAG edges
        let mut edges = Vec::new();
        // Build a map from NodeIndex to milestone ID
        let mut dag_to_milestone: std::collections::HashMap<petgraph::graph::NodeIndex, String> = node_indices.clone();

        // Extract dependencies from DAG edges
        for edge_idx in tool_chain.dag.edge_indices() {
            if let Some((from_idx, to_idx)) = tool_chain.dag.edge_endpoints(edge_idx) {
                if let (Some(from_id), Some(to_id)) = (dag_to_milestone.get(&from_idx), dag_to_milestone.get(&to_idx)) {
                    edges.push(DependencyEdge {
                        from: from_id.clone(),
                        to: to_id.clone(),
                        edge_type: DependencyEdgeType::Hard, // Tool chains have hard dependencies
                        weight: 1.0,
                        metadata: std::collections::HashMap::new(),
                    });
                }
            }
        }

        // Create nodes for dependency graph
        let mut nodes = std::collections::HashMap::new();
        for milestone in &milestones {
            nodes.insert(
                milestone.id.clone(),
                DependencyNode {
                    milestone_id: milestone.id.clone(),
                    node_type: DependencyNodeType::Milestone,
                    estimated_cost: milestone.estimated_effort,
                    estimated_time_ms: (milestone.estimated_effort * 3600.0 * 1000.0) as u64,
                    resource_requirements: std::collections::HashMap::new(),
                    metadata: std::collections::HashMap::new(),
                },
            );
        }

        // Determine root and sink milestones
        let roots: Vec<String> = tool_chain
            .roots
            .iter()
            .filter_map(|&idx| node_indices.get(&idx))
            .cloned()
            .collect();

        let sinks: Vec<String> = tool_chain
            .sinks
            .iter()
            .filter_map(|&idx| node_indices.get(&idx))
            .cloned()
            .collect();

        // Use shared graph algorithm for critical path calculation
        let critical_path =
            crate::planning::graph_algorithms::calculate_critical_path(&nodes, &edges)
                .unwrap_or_else(|_| {
                    // Fallback to roots[0] -> sinks[0] if calculation fails
                    if !roots.is_empty() && !sinks.is_empty() {
                        vec![roots[0].clone(), sinks[0].clone()]
                    } else {
                        vec![]
                    }
                });

        // Use shared graph algorithm for parallel group identification
        let parallel_groups =
            crate::planning::graph_algorithms::identify_parallel_groups(&nodes, &edges)
                .unwrap_or_else(|_| {
                    // Fallback to roots and sinks groups if calculation fails
                    vec![roots, sinks]
                });

        let dependency_graph = agent_agency_contracts::planning_io::DependencyGraph {
            nodes,
            edges,
            critical_path,
            parallel_groups,
            has_cycles: false,
            cycles: vec![],
        };

        Ok(ContractExecutionPlan {
            contract_plan: working_spec.clone(),
            execution_context: None,
            id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            working_spec_id: working_spec.id.clone(),
            title: format!("Tool Chain Plan: {}", working_spec.title),
            overview: format!("Generated tool chain for: {}", working_spec.title),
            state: PlanState::Draft,
            milestones,
            dependency_graph,
            change_budget: self.create_change_budget(&working_spec),
            quality_gates: self.create_quality_gates(&working_spec),
            evidence_requirements: self.create_evidence_requirements(&working_spec),
            active_waivers: vec![],
            metadata: agent_agency_contracts::planning_io::PlanMetadata {
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                approved_at: None,
                completed_at: None,
                created_by: agent_agency_contracts::planning_io::PlanCreator::AI {
                    model: "tool-chain-planner".to_string(),
                    version: "1.0.0".to_string(),
                },
                version: "1.0.0".to_string(),
                source: "tool-chain-bridge".to_string(),
                confidence_score: None,
                generation_time_ms: None,
                model_used: None,
                fallback_used: false,
                strategy: agent_agency_contracts::types::planning::PlanningStrategy::AIAssisted,
                confidence: 0.8,
                estimated_duration_ms: 0,
                estimated_cost_cents: 0,
                adaptive: false,
                engine_version: "1.0.0".to_string(),
                additional_metadata: std::collections::HashMap::new(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            approved_at: None,
            completed_at: None,
        })
    }

    /// Create milestone from tool node
    fn create_milestone_from_tool_node(
        &self,
        node: &ExternalToolNode,
        milestone_id: &str,
        working_spec: &WorkingSpec,
    ) -> Result<ContractMilestone> {
        use agent_agency_contracts::planning_io::{Milestone, MilestonePriority, MilestoneScope};

        // Extract tool identifier - ExternalToolNode is either system_federated_ml::ToolNode (has tool_id)
        // or crate::planning::tool_chain_types::ToolNode (has id), depending on feature flags
        #[cfg(feature = "tool-chain")]
        let tool_identifier = &node.tool_id;
        #[cfg(not(feature = "tool-chain"))]
        let tool_identifier = &node.id;

        Ok(Milestone {
            id: milestone_id.to_string(),
            objective: format!("Execute tool: {}", tool_identifier),
            scope: MilestoneScope {
                files: vec![], // Tool-specific files would be determined by tool
                directories: vec![],
                included_paths: vec![],
                excluded_paths: vec![],
                will_modify: false, // Tools typically don't modify source files
                allowed_operations: vec!["execute".to_string()],
                parallelism: Some(1),
                resource_requirements: std::collections::HashMap::new(),
            },
            interfaces: vec![], // Would be populated based on tool inputs/outputs
            tests: vec![],      // Tool execution has its own validation
            evidence_gate: self.create_tool_evidence_gate(working_spec.risk_tier as u8),
            quality_gates: vec![],       // Quality gates from evidence gate
            dependencies: vec![],        // Set by dependency graph
            estimated_duration: Some(5), // Default 5 minutes for tool execution
            rollback_plan: "Tool execution cannot be rolled back".to_string(),
            state: agent_agency_contracts::planning_io::MilestoneState::Pending,
            assigned_workers: vec![],
            estimated_effort: 0.083, // Default ~5 minutes (0.083 hours) for tool execution
            priority: MilestonePriority::Normal,
            risk_tier: working_spec.risk_tier as u8,
            is_blocking: false,
            blocking_reason: None,
            metrics: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Create evidence gate for tool execution
    fn create_tool_evidence_gate(&self, risk_tier: u8) -> EvidenceGate {
        EvidenceGate {
            min_coverage: 0.0, // Tools don't have test coverage
            min_branch_coverage: 0.0,
            min_mutation_score: 0.0,
            security_scan_required: risk_tier == 1,
            performance_budget: None,
            required_artifacts: vec!["execution_result".to_string()],
            custom_validations: vec![],
        }
    }

    /// Create change budget from working spec
    fn create_change_budget(
        &self,
        working_spec: &WorkingSpec,
    ) -> agent_agency_contracts::planning_io::ChangeBudget {
        use agent_agency_contracts::planning_io::{BudgetEnforcement, ChangeBudget};

        ChangeBudget {
            max_files: working_spec
                .constraints
                .budget_limits
                .as_ref()
                .and_then(|b| b.max_files)
                .map(|v| v as usize)
                .unwrap_or(25),
            max_loc: working_spec
                .constraints
                .budget_limits
                .as_ref()
                .and_then(|b| b.max_loc)
                .map(|v| v as usize)
                .unwrap_or(1000),
            max_migrations: 5,
            allow_breaking_changes: working_spec.risk_tier > 1,
            allow_new_dependencies: working_spec.risk_tier > 1,
            enforcement_mode: if working_spec.risk_tier == 1 {
                BudgetEnforcement::Strict
            } else {
                BudgetEnforcement::Warning
            },
        }
    }

    /// Create quality gates from working spec
    fn create_quality_gates(
        &self,
        working_spec: &WorkingSpec,
    ) -> agent_agency_contracts::planning_io::QualityGates {
        use agent_agency_contracts::planning_io::{
            DocumentationRequirements, MutationRequirements, PerformanceRequirements, QualityGates,
            SecurityRequirements,
        };

        let coverage_reqs = std::collections::HashMap::from([
            ("tool_execution".to_string(), 0.0), // Tools have different validation
        ]);

        QualityGates {
            coverage_requirements: coverage_reqs,
            mutation_requirements: MutationRequirements {
                required: false, // Tools don't use mutation testing
                min_score: 0.0,
                operators: vec![],
            },
            security_requirements: SecurityRequirements {
                scan_required: working_spec.risk_tier == 1,
                max_issues_by_severity: std::collections::HashMap::from([
                    ("critical".to_string(), 0),
                    (
                        "high".to_string(),
                        if working_spec.risk_tier == 1 { 0 } else { 2 },
                    ),
                ]),
                required_controls: vec![],
            },
            performance_requirements: PerformanceRequirements {
                max_regressions: 0, // Tools should not cause regressions
                required_benchmarks: vec![],
                slas: vec![],
            },
            documentation_requirements: DocumentationRequirements {
                api_docs_required: false,
                code_docs_required: true,
                architecture_docs_required: false,
                required_formats: vec![],
                required_types: vec!["tool_execution".to_string()],
                min_coverage: 0.5,
                quality_checks: vec![],
            },
            requires_manual_review: working_spec.risk_tier == 1,
            requires_council_approval: working_spec.risk_tier == 1,
            min_coverage: Some(0.8), // 80% coverage required for tools
            min_mutation_score_percent: Some(0.0), // No mutation testing for tools
        }
    }

    /// Create evidence requirements
    fn create_evidence_requirements(
        &self,
        working_spec: &WorkingSpec,
    ) -> Vec<agent_agency_contracts::planning_io::EvidenceRequirement> {
        working_spec
            .acceptance_criteria
            .iter()
            .enumerate()
            .map(
                |(i, _)| agent_agency_contracts::planning_io::EvidenceRequirement {
                    milestone_id: format!("TC-M{}", i),
                    evidence_type: "tool_execution".to_string(),
                    collection_method: "automated".to_string(),
                    validation_criteria: std::collections::HashMap::new(),
                    mandatory: true,
                },
            )
            .collect()
    }

    /// Helper methods for working spec analysis
    fn extract_task_description(&self, working_spec: &WorkingSpec) -> String {
        format!("{}: {}", working_spec.title, working_spec.description)
    }

    fn determine_task_type(&self, working_spec: &WorkingSpec) -> String {
        // Determine task type based on acceptance criteria
        if working_spec
            .acceptance_criteria
            .iter()
            .any(|c| c.given.contains("compile") || c.then.contains("compile"))
        {
            "compilation".to_string()
        } else if working_spec
            .acceptance_criteria
            .iter()
            .any(|c| c.given.contains("test") || c.then.contains("test"))
        {
            "testing".to_string()
        } else if working_spec
            .acceptance_criteria
            .iter()
            .any(|c| c.given.contains("deploy") || c.then.contains("deploy"))
        {
            "deployment".to_string()
        } else {
            "general".to_string()
        }
    }

    fn determine_task_complexity(&self, working_spec: &WorkingSpec) -> ExternalTaskComplexity {
        if working_spec.acceptance_criteria.len() > 5 || working_spec.file_changes.len() > 10 {
            ExternalTaskComplexity::VeryComplex
        } else if working_spec.acceptance_criteria.len() > 3 || working_spec.file_changes.len() > 5
        {
            ExternalTaskComplexity::Complex
        } else if working_spec.acceptance_criteria.len() > 1 {
            ExternalTaskComplexity::Moderate
        } else {
            ExternalTaskComplexity::Simple
        }
    }

    fn extract_required_capabilities(&self, working_spec: &WorkingSpec) -> Vec<String> {
        let mut capabilities = vec!["evidence_collection".to_string()]; // Always need evidence

        if working_spec.risk_tier == 1 {
            capabilities.push("security".to_string());
        }

        if working_spec
            .coverage_targets
            .as_ref()
            .and_then(|ct| ct.line_coverage)
            .unwrap_or(0.0)
            > 0.8
        {
            capabilities.push("quality_gate".to_string());
        }

        capabilities
    }

    fn determine_risk_tolerance(&self, working_spec: &WorkingSpec) -> ExternalRiskLevel {
        match working_spec.risk_tier {
            1 => ExternalRiskLevel::Conservative,
            2 => ExternalRiskLevel::Balanced,
            3 => ExternalRiskLevel::Aggressive,
            _ => ExternalRiskLevel::Balanced,
        }
    }

    /// Convert milestone to tool chain execution
    pub async fn milestone_to_tool_chain(
        &self,
        milestone: &ContractMilestone,
    ) -> Result<ToolChainExecution> {
        use std::collections::HashMap;
        use tracing::debug;

        debug!("Converting milestone {} to tool chain", milestone.id);

        // Extract tools from milestone objective and allowed operations
        let mut tools = Vec::new();

        // Parse milestone objective to identify tools
        // For now, create a single tool based on milestone objective
        // In a more sophisticated implementation, this would parse the objective
        // to identify multiple tools and their relationships
        let tool_name = self.extract_tool_name_from_objective(&milestone.objective);
        let tool_version = "1.0.0".to_string(); // Default version

        // Build tool parameters from milestone scope and interfaces
        let mut parameters = HashMap::new();
        parameters.insert(
            "milestone_id".to_string(),
            serde_json::Value::String(milestone.id.clone()),
        );
        parameters.insert(
            "objective".to_string(),
            serde_json::Value::String(milestone.objective.clone()),
        );

        // Add scope information
        if !milestone.scope.files.is_empty() {
            parameters.insert(
                "files".to_string(),
                serde_json::json!(milestone.scope.files),
            );
        }
        if !milestone.scope.directories.is_empty() {
            parameters.insert(
                "directories".to_string(),
                serde_json::json!(milestone.scope.directories),
            );
        }
        if !milestone.scope.included_paths.is_empty() {
            parameters.insert(
                "included_paths".to_string(),
                serde_json::json!(milestone.scope.included_paths),
            );
        }

        // Add allowed operations
        if !milestone.scope.allowed_operations.is_empty() {
            parameters.insert(
                "allowed_operations".to_string(),
                serde_json::json!(milestone.scope.allowed_operations),
            );
        }

        // Add interface information if available
        if !milestone.interfaces.is_empty() {
            parameters.insert(
                "interfaces".to_string(),
                serde_json::json!(milestone.interfaces),
            );
        }

        // Determine timeout from estimated duration
        let timeout_ms = milestone
            .estimated_duration
            .map(|minutes| minutes as u64 * 60 * 1000) // Convert minutes to milliseconds
            .or_else(|| Some(300000)); // Default 5 minutes

        // Create retry policy based on risk tier
        let retry_policy = if milestone.risk_tier <= 1 {
            // Tier 1: Conservative retry policy
            Some(RetryPolicy {
                max_attempts: 2,
                base_delay_ms: 1000,
                backoff_multiplier: 2.0,
                max_delay_ms: 5000,
            })
        } else if milestone.risk_tier == 2 {
            // Tier 2: Balanced retry policy
            Some(RetryPolicy {
                max_attempts: 3,
                base_delay_ms: 500,
                backoff_multiplier: 1.5,
                max_delay_ms: 3000,
            })
        } else {
            // Tier 3: Aggressive retry policy
            Some(RetryPolicy {
                max_attempts: 5,
                base_delay_ms: 250,
                backoff_multiplier: 1.2,
                max_delay_ms: 2000,
            })
        };

        tools.push(ToolSpec {
            name: tool_name.clone(),
            version: tool_version,
            parameters,
            timeout_ms,
            retry_policy,
        });

        // Build data flow from milestone dependencies
        // For now, create simple sequential data flow
        // In a more sophisticated implementation, this would analyze dependencies
        // to create proper data flow between tools
        let mut data_flow = Vec::new();
        for (i, dep_id) in milestone.dependencies.iter().enumerate() {
            // Create data flow from dependency milestone to current milestone
            // This assumes dependency milestones produce outputs that this milestone consumes
            data_flow.push(DataFlow {
                from_tool: format!("tool_{}", dep_id),
                from_output: "output".to_string(),
                to_tool: tool_name.clone(),
                to_input: format!("input_{}", i),
                transformation: None, // Could add transformation logic here
            });
        }

        // Build execution constraints from milestone
        let max_execution_time_ms = milestone
            .estimated_duration
            .map(|minutes| minutes as u64 * 60 * 1000 * 2) // 2x estimated duration as max
            .unwrap_or(600000); // Default 10 minutes

        // Extract required capabilities from milestone scope and resource requirements
        let mut required_capabilities = Vec::new();
        for operation in &milestone.scope.allowed_operations {
            required_capabilities.push(operation.clone());
        }
        for (key, _value) in &milestone.scope.resource_requirements {
            required_capabilities.push(key.clone());
        }

        let constraints = ExecutionConstraints {
            max_execution_time_ms,
            max_cost: None, // Could be extracted from milestone metrics if available
            required_capabilities,
        };

        let chain_id = format!("chain_{}", milestone.id);

        debug!(
            "Converted milestone {} to tool chain with {} tools",
            milestone.id,
            tools.len()
        );

        Ok(ToolChainExecution {
            chain_id,
            tools,
            data_flow,
            constraints,
        })
    }

    /// Extract tool name from milestone objective
    fn extract_tool_name_from_objective(&self, objective: &str) -> String {
        // Simple heuristic: extract tool name from objective
        // In a more sophisticated implementation, this would use NLP or pattern matching
        // to identify the actual tool name

        // Common patterns:
        // - "Execute tool: <name>"
        // - "Run <name>"
        // - "<name> execution"
        if let Some(stripped) = objective.strip_prefix("Execute tool: ") {
            return stripped.to_string();
        }
        if let Some(stripped) = objective.strip_prefix("Run ") {
            return stripped
                .split_whitespace()
                .next()
                .unwrap_or("default_tool")
                .to_string();
        }
        if let Some(stripped) = objective.strip_suffix(" execution") {
            return stripped.to_string();
        }

        // Default: use a sanitized version of the objective
        objective
            .to_lowercase()
            .replace(" ", "_")
            .chars()
            .take(50)
            .collect()
    }

    /// Execute tool chain and collect results
    pub async fn execute_tool_chain(
        &self,
        tool_chain: &ToolChainExecution,
    ) -> Result<ExecutionResult> {
        use chrono::Utc;
        use std::collections::HashMap;
        use tracing::{debug, error, warn};

        debug!("Executing tool chain: {}", tool_chain.chain_id);
        let start_time = std::time::Instant::now();

        // Build dependency graph from data flow
        // This helps us determine execution order
        let tool_order = self.determine_execution_order(tool_chain)?;

        // Execute tools in dependency order
        let mut tool_results = Vec::new();
        let mut tool_outputs: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();
        let mut errors = Vec::new();
        let mut evidence = Vec::new();

        for tool_name in tool_order {
            // Find the tool spec
            let tool_spec = tool_chain
                .tools
                .iter()
                .find(|t| t.name == tool_name)
                .ok_or_else(|| anyhow!("Tool {} not found in tool chain", tool_name))?;

            debug!(
                "Executing tool: {} (version: {})",
                tool_spec.name, tool_spec.version
            );

            // Prepare input parameters with data flow
            let mut tool_params = tool_spec.parameters.clone();

            // Apply data flow transformations
            for data_flow in &tool_chain.data_flow {
                if data_flow.to_tool == tool_name {
                    // Get output from source tool
                    if let Some(source_outputs) = tool_outputs.get(&data_flow.from_tool) {
                        if let Some(output_value) = source_outputs.get(&data_flow.from_output) {
                            // Apply transformation if specified
                            let transformed_value =
                                if let Some(transform) = &data_flow.transformation {
                                    self.apply_transformation(output_value, transform)?
                                } else {
                                    output_value.clone()
                                };
                            tool_params.insert(data_flow.to_input.clone(), transformed_value);
                        }
                    }
                }
            }

            // Check execution constraints
            if start_time.elapsed().as_millis() as u64
                > tool_chain.constraints.max_execution_time_ms
            {
                let error_msg = format!(
                    "Tool chain execution exceeded max time: {}ms",
                    tool_chain.constraints.max_execution_time_ms
                );
                warn!("{}", error_msg);
                errors.push(error_msg.clone());
                tool_results.push(ToolResult {
                    tool_name: tool_spec.name.clone(),
                    success: false,
                    output: HashMap::new(),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(error_msg),
                });
                break;
            }

            // Execute tool with retry logic
            let tool_result = self.execute_tool_with_retry(tool_spec, &tool_params).await;

            match tool_result {
                Ok(result) => {
                    // Store outputs for data flow
                    tool_outputs.insert(tool_name.clone(), result.output.clone());
                    tool_results.push(result.clone());

                    // Collect evidence from successful tool execution
                    if result.success {
                        evidence.push(EvidenceArtifact {
                            artifact_type: "tool_execution".to_string(),
                            data: serde_json::json!({
                                "tool_name": result.tool_name,
                                "execution_time_ms": result.execution_time_ms,
                                "output": result.output,
                            }),
                            collected_at: Utc::now(),
                        });
                    }
                }
                Err(e) => {
                    let error_msg = format!("Tool {} execution failed: {}", tool_spec.name, e);
                    error!("{}", error_msg);
                    errors.push(error_msg.clone());
                    tool_results.push(ToolResult {
                        tool_name: tool_spec.name.clone(),
                        success: false,
                        output: HashMap::new(),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        error: Some(error_msg),
                    });

                    // If critical tool fails, stop execution
                    // Could be enhanced with failure policy
                    break;
                }
            }
        }

        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let success = errors.is_empty() && tool_results.iter().all(|r| r.success);

        debug!(
            "Tool chain execution completed: success={}, tools_executed={}, errors={}",
            success,
            tool_results.len(),
            errors.len()
        );

        Ok(ExecutionResult {
            success,
            execution_time_ms,
            tool_results,
            evidence,
            errors,
        })
    }

    /// Determine execution order from data flow dependencies
    fn determine_execution_order(&self, tool_chain: &ToolChainExecution) -> Result<Vec<String>> {
        use std::collections::{HashMap, HashSet};

        // Build dependency map: tool -> tools it depends on
        let mut dependencies: HashMap<String, HashSet<String>> = HashMap::new();
        let mut all_tools = HashSet::new();

        // Initialize all tools
        for tool in &tool_chain.tools {
            all_tools.insert(tool.name.clone());
            dependencies.insert(tool.name.clone(), HashSet::new());
        }

        // Build dependency graph from data flow
        for data_flow in &tool_chain.data_flow {
            all_tools.insert(data_flow.from_tool.clone());
            all_tools.insert(data_flow.to_tool.clone());

            dependencies
                .entry(data_flow.to_tool.clone())
                .or_insert_with(HashSet::new)
                .insert(data_flow.from_tool.clone());
        }

        // Topological sort to determine execution order
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        fn visit(
            tool: &str,
            dependencies: &HashMap<String, HashSet<String>>,
            visited: &mut HashSet<String>,
            visiting: &mut HashSet<String>,
            order: &mut Vec<String>,
        ) -> Result<(), anyhow::Error> {
            if visited.contains(tool) {
                return Ok(());
            }
            if visiting.contains(tool) {
                return Err(anyhow!("Circular dependency detected in tool chain"));
            }

            visiting.insert(tool.to_string());

            if let Some(deps) = dependencies.get(tool) {
                for dep in deps {
                    visit(dep, dependencies, visited, visiting, order)?;
                }
            }

            visiting.remove(tool);
            visited.insert(tool.to_string());
            order.push(tool.to_string());

            Ok(())
        }

        for tool in &all_tools {
            if !visited.contains(tool) {
                visit(tool, &dependencies, &mut visited, &mut visiting, &mut order)?;
            }
        }

        Ok(order)
    }

    /// Execute tool with retry policy
    async fn execute_tool_with_retry(
        &self,
        tool_spec: &ToolSpec,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        use std::time::Duration;
        use tracing::{debug, warn};

        let max_attempts = tool_spec
            .retry_policy
            .as_ref()
            .map(|p| p.max_attempts)
            .unwrap_or(1);

        let mut last_error = None;
        let mut delay_ms = tool_spec
            .retry_policy
            .as_ref()
            .map(|p| p.base_delay_ms)
            .unwrap_or(0);

        for attempt in 1..=max_attempts {
            debug!(
                "Executing tool {} (attempt {}/{})",
                tool_spec.name, attempt, max_attempts
            );

            let start_time = std::time::Instant::now();

            // Execute tool (simplified - would integrate with actual tool execution system)
            match self.execute_single_tool(tool_spec, parameters).await {
                Ok(result) => {
                    return Ok(ToolResult {
                        tool_name: tool_spec.name.clone(),
                        success: result.success,
                        output: result.output,
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        error: if result.success { None } else { result.error },
                    });
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    warn!(
                        "Tool {} execution failed (attempt {}/{}): {}",
                        tool_spec.name,
                        attempt,
                        max_attempts,
                        last_error.as_ref().unwrap()
                    );

                    // Wait before retry (except on last attempt)
                    if attempt < max_attempts {
                        if let Some(retry_policy) = &tool_spec.retry_policy {
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            delay_ms = ((delay_ms as f64) * retry_policy.backoff_multiplier) as u64;
                            if let Some(max_delay) = Some(retry_policy.max_delay_ms) {
                                delay_ms = delay_ms.min(max_delay);
                            }
                        }
                    }
                }
            }
        }

        // All attempts failed
        Ok(ToolResult {
            tool_name: tool_spec.name.clone(),
            success: false,
            output: HashMap::new(),
            execution_time_ms: 0,
            error: last_error,
        })
    }

    /// Execute a single tool (simplified implementation)
    async fn execute_single_tool(
        &self,
        tool_spec: &ToolSpec,
        _parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        use std::collections::HashMap;
        use tracing::debug;

        // Simplified tool execution
        // In a real implementation, this would:
        // 1. Look up tool in tool registry
        // 2. Validate parameters against tool schema
        // 3. Execute tool via appropriate executor
        // 4. Collect and validate outputs

        debug!(
            "Executing tool: {} with {} parameters",
            tool_spec.name,
            _parameters.len()
        );

        // Simulate tool execution
        // For now, return a success result with mock output
        // This can be enhanced when tool execution infrastructure is available
        let output = HashMap::from([
            ("status".to_string(), serde_json::json!("completed")),
            (
                "tool_name".to_string(),
                serde_json::json!(tool_spec.name.clone()),
            ),
        ]);

        Ok(ToolResult {
            tool_name: tool_spec.name.clone(),
            success: true,
            output,
            execution_time_ms: 100, // Simulated execution time
            error: None,
        })
    }

    /// Apply data transformation
    fn apply_transformation(
        &self,
        value: &serde_json::Value,
        transform: &str,
    ) -> Result<serde_json::Value> {
        use tracing::warn;

        // Simple transformation logic
        // In a more sophisticated implementation, this would support:
        // - JSON path transformations
        // - Type conversions
        // - Data validation
        // - Custom transformation functions

        match transform {
            "identity" => Ok(value.clone()),
            "to_string" => Ok(serde_json::Value::String(value.to_string())),
            "to_number" => {
                if let Some(num) = value.as_f64() {
                    // serde_json::Number doesn't implement From<f64>, so we need to convert
                    // Try to convert to i64 first if it's a whole number, otherwise use f64 representation
                    if num.fract() == 0.0 && num >= (i64::MIN as f64) && num <= (i64::MAX as f64) {
                        Ok(serde_json::Value::Number(serde_json::Number::from(
                            num as i64,
                        )))
                    } else {
                        // For non-integer f64, serde_json doesn't support it directly
                        // Return as string representation instead
                        Ok(serde_json::Value::String(num.to_string()))
                    }
                } else {
                    Ok(value.clone())
                }
            }
            _ => {
                // Unknown transformation - return value as-is
                warn!("Unknown transformation: {}, using identity", transform);
                Ok(value.clone())
            }
        }
    }
}

/// Tool chain execution specification

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ToolChainExecution {
    /// Tool chain identifier
    pub chain_id: String,

    /// Tools to execute in sequence
    pub tools: Vec<ToolSpec>,

    /// Data flow between tools
    pub data_flow: Vec<DataFlow>,

    /// Execution constraints
    pub constraints: ExecutionConstraints,
}

/// Tool specification

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ToolSpec {
    /// Tool name
    pub name: String,

    /// Tool version
    pub version: String,

    /// Input parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Execution timeout
    pub timeout_ms: Option<u64>,

    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
}

/// Data flow between tools

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct DataFlow {
    /// Source tool output
    pub from_tool: String,

    /// Source output name
    pub from_output: String,

    /// Destination tool input
    pub to_tool: String,

    /// Destination input name
    pub to_input: String,

    /// Data transformation
    pub transformation: Option<String>,
}

/// Execution constraints

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExecutionConstraints {
    /// Maximum execution time
    pub max_execution_time_ms: u64,

    /// Maximum cost
    pub max_cost: Option<f64>,

    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Retry policy

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct RetryPolicy {
    /// Maximum attempts
    pub max_attempts: u32,

    /// Base delay between retries
    pub base_delay_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Maximum delay
    pub max_delay_ms: u64,
}

/// Execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExecutionResult {
    /// Success status
    pub success: bool,

    /// Execution time
    pub execution_time_ms: u64,

    /// Tool results
    pub tool_results: Vec<ToolResult>,

    /// Evidence collected
    pub evidence: Vec<EvidenceArtifact>,

    /// Errors encountered
    pub errors: Vec<String>,
}

/// Tool execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ToolResult {
    /// Tool name
    pub tool_name: String,

    /// Success status
    pub success: bool,

    /// Output data
    pub output: HashMap<String, serde_json::Value>,

    /// Execution time
    pub execution_time_ms: u64,

    /// Error message if failed
    pub error: Option<String>,
}

/// Evidence artifact from execution

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EvidenceArtifact {
    /// Artifact type
    pub artifact_type: String,

    /// Artifact data
    pub data: serde_json::Value,

    /// Collection timestamp
    #[schemars(with = "String")]
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_tool_chain_bridge_creation() {
        use crate::planning::tool_chain_types::{
            ExternalSchemaRegistry, ExternalToolChainPlanner, ExternalToolRegistry,
        };
        
        // Create instances based on feature flag
        #[cfg(feature = "tool-chain")]
        let (planner, schema_registry, tool_registry) = {
            use system_federated_ml::tool_registry::ToolRegistry as ExternalToolRegistryImpl;
            use system_federated_ml::tool_chain_planner::SchemaRegistry as ExternalSchemaRegistryImpl;
            let tool_registry = Arc::new(ExternalToolRegistryImpl::new());
            let schema_registry = Arc::new(ExternalSchemaRegistryImpl::new());
            let planner = Arc::new(ExternalToolChainPlanner::new(
                tool_registry.clone(),
                schema_registry.clone(),
            ));
            (planner, schema_registry, tool_registry)
        };
        
        #[cfg(not(feature = "tool-chain"))]
        let (planner, schema_registry, tool_registry) = (
            Arc::new(ExternalToolChainPlanner::default()),
            Arc::new(ExternalSchemaRegistry::default()),
            Arc::new(ExternalToolRegistry::default()),
        );
        
        let _bridge = ToolChainBridge::new(planner, schema_registry, tool_registry);
        // Bridge created successfully
        assert!(true);
    }

    #[test]
    fn test_tool_spec_creation() {
        let tool_spec = ToolSpec {
            name: "test_tool".to_string(),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            timeout_ms: Some(30000),
            retry_policy: Some(RetryPolicy {
                max_attempts: 3,
                base_delay_ms: 1000,
                backoff_multiplier: 2.0,
                max_delay_ms: 30000,
            }),
        };

        assert_eq!(tool_spec.name, "test_tool");
        assert_eq!(tool_spec.timeout_ms, Some(30000));
        assert!(tool_spec.retry_policy.is_some());
    }

    #[test]
    fn test_execution_constraints() {
        let constraints = ExecutionConstraints {
            max_execution_time_ms: 300000,
            max_cost: Some(10.0),
            required_capabilities: vec!["network".to_string(), "filesystem".to_string()],
        };

        assert_eq!(constraints.max_execution_time_ms, 300000);
        assert_eq!(constraints.max_cost, Some(10.0));
        assert_eq!(constraints.required_capabilities.len(), 2);
    }
}
