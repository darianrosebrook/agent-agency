//! Tool Chain Bridge - Integration with system-federated-ml ToolChainPlanner
//!
//! Bridges the planning system with the existing tool chain planning infrastructure
//! for sophisticated DAG-based tool execution.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use agent_agency_contracts::{
    planning_io::{ExecutionPlan as ContractExecutionPlan, Milestone as ContractMilestone, PlanState, MilestoneState, DependencyGraph},
    WorkingSpec,
};

/// Bridge to tool chain planner
pub struct ToolChainBridge {
    /// Reference to the actual tool chain planner from system-federated-ml
    tool_chain_planner: std::sync::Arc<system_federated_ml::tool_chain_planner::ToolChainPlanner>,

    /// Schema registry for tool I/O validation
    schema_registry: std::sync::Arc<system_federated_ml::tool_chain_planner::SchemaRegistry>,

    /// Tool registry for accessing available tools
    tool_registry: std::sync::Arc<system_federated_ml::tool_registry::ToolRegistry>,
}

impl ToolChainBridge {
    /// Create new tool chain bridge with real dependencies
    pub fn new(
        tool_chain_planner: std::sync::Arc<system_federated_ml::tool_chain_planner::ToolChainPlanner>,
        schema_registry: std::sync::Arc<system_federated_ml::tool_chain_planner::SchemaRegistry>,
        tool_registry: std::sync::Arc<system_federated_ml::tool_registry::ToolRegistry>,
    ) -> Self {
        Self {
            tool_chain_planner,
            schema_registry,
            tool_registry,
        }
    }

    /// Generate execution plan using tool chain planner
    pub async fn generate_from_working_spec(&self, working_spec: WorkingSpec) -> Result<ContractExecutionPlan> {
        // Convert working spec to planning context
        let planning_context = self.convert_working_spec_to_planning_context(&working_spec)?;

        // Create planning constraints based on working spec
        let constraints = self.create_planning_constraints(&working_spec)?;

        // Generate tool chain using the real tool chain planner
        let tool_chain = self.tool_chain_planner.plan_chain(&planning_context, &constraints).await?;

        // Convert tool chain back to execution plan format
        self.convert_tool_chain_to_execution_plan(tool_chain, working_spec).await
    }

    /// Convert working spec to tool chain planning context
    fn convert_working_spec_to_planning_context(&self, working_spec: &WorkingSpec) -> Result<system_federated_ml::tool_chain_planner::PlanningContext> {
        // Extract task description from working spec
        let task_description = self.extract_task_description(working_spec);

        // Determine task type and complexity
        let task_type = self.determine_task_type(working_spec);
        let complexity = self.determine_task_complexity(working_spec);

        // Extract required capabilities
        let required_capabilities = self.extract_required_capabilities(working_spec);

        // Determine risk tolerance
        let risk_tolerance = self.determine_risk_tolerance(working_spec);

        Ok(system_federated_ml::tool_chain_planner::PlanningContext {
            task_description,
            task_type,
            complexity,
            required_capabilities,
            time_budget_ms: working_spec.constraints.max_time_ms,
            cost_budget_cents: working_spec.constraints.max_cost_cents,
            risk_tolerance,
        })
    }

    /// Create planning constraints from working spec
    fn create_planning_constraints(&self, working_spec: &WorkingSpec) -> Result<system_federated_ml::tool_chain_planner::PlanningConstraints> {
        Ok(system_federated_ml::tool_chain_planner::PlanningConstraints {
            max_chain_length: 5, // Default reasonable limit
            max_parallelism: 3,  // Allow some parallelism
            max_cost_cents: working_spec.constraints.max_cost_cents.unwrap_or(100),
            max_time_ms: working_spec.constraints.max_time_ms,
            require_fallbacks: working_spec.risk_tier > 1, // Require fallbacks for high-risk work
        })
    }

    /// Convert tool chain to execution plan
    async fn convert_tool_chain_to_execution_plan(
        &self,
        tool_chain: system_federated_ml::tool_chain_planner::ToolChain,
        working_spec: WorkingSpec,
    ) -> Result<ContractExecutionPlan> {
        use agent_agency_contracts::planning_io::{DependencyNode, DependencyEdge, DependencyNodeType, DependencyEdgeType};

        // Create milestones from tool chain nodes
        let mut milestones = Vec::new();
        let mut node_indices = std::collections::HashMap::new();

        // Map tool chain nodes to milestones
        for (idx, node) in tool_chain.dag.node_references() {
            let milestone_id = format!("TC-{}", node.tool_id);
            let milestone = self.create_milestone_from_tool_node(&node, &milestone_id, &working_spec)?;
            milestones.push(milestone);
            node_indices.insert(idx, milestone_id);
        }

        // Create dependency graph from tool chain edges
        let mut edges = Vec::new();
        for edge in tool_chain.dag.edge_references() {
            let from_id = node_indices.get(&edge.source()).unwrap().clone();
            let to_id = node_indices.get(&edge.target()).unwrap().clone();

            edges.push(DependencyEdge {
                from: from_id,
                to: to_id,
                edge_type: DependencyEdgeType::Hard, // Tool chains have hard dependencies
                weight: 1.0,
                metadata: std::collections::HashMap::new(),
            });
        }

        // Create nodes for dependency graph
        let mut nodes = std::collections::HashMap::new();
        for milestone in &milestones {
            nodes.insert(milestone.id.clone(), DependencyNode {
                milestone_id: milestone.id.clone(),
                node_type: DependencyNodeType::Milestone,
                estimated_cost: milestone.estimated_effort,
                estimated_time_ms: (milestone.estimated_effort * 3600.0 * 1000.0) as u64,
                resource_requirements: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
            });
        }

        // Determine root and sink milestones
        let roots = tool_chain.roots.iter()
            .filter_map(|idx| node_indices.get(idx))
            .cloned()
            .collect();

        let sinks = tool_chain.sinks.iter()
            .filter_map(|idx| node_indices.get(idx))
            .cloned()
            .collect();

        // Calculate critical path
        let critical_path = if !roots.is_empty() && !sinks.is_empty() {
            vec![roots[0].clone(), sinks[0].clone()] // Simplified
        } else {
            vec![]
        };

        let dependency_graph = agent_agency_contracts::planning_io::DependencyGraph {
            nodes,
            edges,
            critical_path,
            parallel_groups: vec![roots, sinks], // Simplified parallel groups
            has_cycles: false,
            cycles: vec![],
        };

        Ok(ContractExecutionPlan {
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
            metadata: Default::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            approved_at: None,
            completed_at: None,
        })
    }

    /// Create milestone from tool node
    fn create_milestone_from_tool_node(
        &self,
        node: &system_federated_ml::tool_chain_planner::ToolNode,
        milestone_id: &str,
        working_spec: &WorkingSpec,
    ) -> Result<ContractMilestone> {
        use agent_agency_contracts::planning_io::{Milestone, MilestoneScope, EvidenceGate, MilestonePriority};

        Ok(Milestone {
            id: milestone_id.to_string(),
            objective: format!("Execute tool: {}", node.tool_id),
            scope: MilestoneScope {
                files: vec![], // Tool-specific files would be determined by tool
                directories: vec![],
                will_modify: false, // Tools typically don't modify source files
                allowed_operations: vec!["execute".to_string()],
                parallelism: Some(1),
                resource_requirements: std::collections::HashMap::new(),
            },
            interfaces: vec![], // Would be populated based on tool inputs/outputs
            tests: vec![], // Tool execution has its own validation
            evidence_gate: self.create_tool_evidence_gate(working_spec.risk_tier),
            rollback_plan: "Tool execution cannot be rolled back".to_string(),
            dependencies: vec![], // Set by dependency graph
            state: agent_agency_contracts::planning_io::MilestoneState::Pending,
            assigned_workers: vec![],
            estimated_effort: (node.sla_ms as f64) / (3600.0 * 1000.0), // Convert ms to hours
            priority: MilestonePriority::Normal,
            risk_tier: working_spec.risk_tier,
            is_blocking: false,
            blocking_reason: None,
            metrics: None,
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
    fn create_change_budget(&self, working_spec: &WorkingSpec) -> agent_agency_contracts::planning_io::ChangeBudget {
        use agent_agency_contracts::planning_io::{ChangeBudget, BudgetEnforcement};

        ChangeBudget {
            max_files: working_spec.constraints.max_files,
            max_loc: working_spec.constraints.max_loc,
            max_migrations: working_spec.constraints.max_migrations.unwrap_or(5),
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
    fn create_quality_gates(&self, working_spec: &WorkingSpec) -> agent_agency_contracts::planning_io::QualityGates {
        use agent_agency_contracts::planning_io::{QualityGates, MutationRequirements, SecurityRequirements, PerformanceRequirements, DocumentationRequirements};

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
                    ("high".to_string(), if working_spec.risk_tier == 1 { 0 } else { 2 }),
                ]),
                required_controls: vec![],
                audit_requirements: vec![],
            },
            performance_requirements: PerformanceRequirements {
                max_regressions: 0, // Tools should not cause regressions
                required_benchmarks: vec![],
                slas: vec![],
            },
            documentation_requirements: DocumentationRequirements {
                required_types: vec!["tool_execution".to_string()],
                min_coverage: 0.5,
                required_formats: vec![],
                quality_checks: vec![],
            },
            requires_manual_review: working_spec.risk_tier == 1,
            requires_council_approval: working_spec.risk_tier == 1,
        }
    }

    /// Create evidence requirements
    fn create_evidence_requirements(&self, working_spec: &WorkingSpec) -> Vec<agent_agency_contracts::planning_io::EvidenceRequirement> {
        working_spec.acceptance.iter().enumerate().map(|(i, _)| {
            agent_agency_contracts::planning_io::EvidenceRequirement {
                milestone_id: format!("TC-M{}", i),
                evidence_type: "tool_execution".to_string(),
                collection_method: "automated".to_string(),
                validation_criteria: std::collections::HashMap::new(),
                mandatory: true,
            }
        }).collect()
    }

    /// Helper methods for working spec analysis
    fn extract_task_description(&self, working_spec: &WorkingSpec) -> String {
        format!("{}: {}", working_spec.title, working_spec.description)
    }

    fn determine_task_type(&self, working_spec: &WorkingSpec) -> String {
        // Determine task type based on acceptance criteria
        if working_spec.acceptance.iter().any(|c| c.given.contains("compile") || c.then.contains("compile")) {
            "compilation".to_string()
        } else if working_spec.acceptance.iter().any(|c| c.given.contains("test") || c.then.contains("test")) {
            "testing".to_string()
        } else if working_spec.acceptance.iter().any(|c| c.given.contains("deploy") || c.then.contains("deploy")) {
            "deployment".to_string()
        } else {
            "general".to_string()
        }
    }

    fn determine_task_complexity(&self, working_spec: &WorkingSpec) -> system_federated_ml::tool_chain_planner::TaskComplexity {
        if working_spec.acceptance.len() > 5 || working_spec.file_changes.len() > 10 {
            system_federated_ml::tool_chain_planner::TaskComplexity::VeryComplex
        } else if working_spec.acceptance.len() > 3 || working_spec.file_changes.len() > 5 {
            system_federated_ml::tool_chain_planner::TaskComplexity::Complex
        } else if working_spec.acceptance.len() > 1 {
            system_federated_ml::tool_chain_planner::TaskComplexity::Moderate
        } else {
            system_federated_ml::tool_chain_planner::TaskComplexity::Simple
        }
    }

    fn extract_required_capabilities(&self, working_spec: &WorkingSpec) -> Vec<String> {
        let mut capabilities = vec!["evidence_collection".to_string()]; // Always need evidence

        if working_spec.risk_tier == 1 {
            capabilities.push("security".to_string());
        }

        if working_spec.coverage_targets.line_coverage > 0.8 {
            capabilities.push("quality_gate".to_string());
        }

        capabilities
    }

    fn determine_risk_tolerance(&self, working_spec: &WorkingSpec) -> system_federated_ml::tool_chain_planner::RiskLevel {
        match working_spec.risk_tier {
            1 => system_federated_ml::tool_chain_planner::RiskLevel::Conservative,
            2 => system_federated_ml::tool_chain_planner::RiskLevel::Balanced,
            3 => system_federated_ml::tool_chain_planner::RiskLevel::Aggressive,
            _ => system_federated_ml::tool_chain_planner::RiskLevel::Balanced,
        }
    }

    /// Convert milestone to tool chain execution
    pub async fn milestone_to_tool_chain(&self, milestone: &ContractMilestone) -> Result<ToolChainExecution> {
        // Placeholder implementation
        // Would convert milestone specification to tool chain format

        Err(anyhow!("Tool chain conversion not yet implemented - PLACEHOLDER"))
    }

    /// Execute tool chain and collect results
    pub async fn execute_tool_chain(&self, tool_chain: &ToolChainExecution) -> Result<ExecutionResult> {
        // Placeholder implementation
        // Would execute the tool chain and return results

        Err(anyhow!("Tool chain execution not yet implemented - PLACEHOLDER"))
    }
}

/// Tool chain execution specification
#[derive(Debug, Clone)]
pub struct ToolChainExecution {
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
#[derive(Debug, Clone)]
pub struct ToolSpec {
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
#[derive(Debug, Clone)]
pub struct DataFlow {
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
#[derive(Debug, Clone)]
pub struct ExecutionConstraints {
    /// Maximum execution time
    pub max_execution_time_ms: u64,

    /// Maximum cost
    pub max_cost: Option<f64>,

    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Retry policy
#[derive(Debug, Clone)]
pub struct RetryPolicy {
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
#[derive(Debug, Clone)]
pub struct ExecutionResult {
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
#[derive(Debug, Clone)]
pub struct ToolResult {
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
#[derive(Debug, Clone)]
pub struct EvidenceArtifact {
    /// Artifact type
    pub artifact_type: String,

    /// Artifact data
    pub data: serde_json::Value,

    /// Collection timestamp
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_chain_bridge_creation() {
        let bridge = ToolChainBridge::new();
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
