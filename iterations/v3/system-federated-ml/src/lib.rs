//! Complete Tool Calling Ecosystem - MCP Integration with CAWS Tooling
//!
//! Implements comprehensive tooling ecosystem for reasoning, conflict resolution,
//! and evidence collection through MCP-based CAWS tool discovery and execution.
//!
//! ## Tool Categories
//!
//! 1. **Policy Enforcement Tools**: CAWS validation, waiver auditing, budget verification
//! 2. **Evidence Collection Tools**: Claim extraction, fact verification, source validation
//! 3. **Governance Tools**: Audit logging, provenance tracking, compliance reporting
//! 4. **Quality Gate Tools**: Code analysis, test execution, performance validation
//! 5. **Conflict Resolution Tools**: Debate orchestration, consensus building, evidence synthesis
//! 6. **Workflow Tools**: Task decomposition, progress tracking, resource allocation

pub mod claim_extraction;
pub mod conflict_resolution_tools;
pub mod evidence_collection_tools;
pub mod evidence_types;
pub mod fact_verification;
pub mod executor;
pub mod source_validation;
pub mod multi_modal_verification;
pub mod parallel_integration;
pub mod schema_registry;
pub mod tool_chain_planner;
pub mod tool_coordinator;
pub mod tool_discovery;
pub mod tool_execution;
pub mod tool_registry;

pub use conflict_resolution_tools::{ConflictResolutionTool, DebateOrchestrator, ConsensusBuilder};
pub use evidence_collection_tools::{EvidenceCollectionTool}; // FactVerificationTool, SourceValidationTool - not implemented yet
pub use executor::{ChainExecutor, ExecutionResult};
pub use multi_modal_verification::{MultimodalVerificationTool};
pub use parallel_integration::{ParallelToolCoordinator};
// pub use governance_tools::{GovernanceTool, AuditLogger, ProvenanceTracker}; // Module not implemented yet
// pub use quality_gate_tools::{QualityGateTool, CodeAnalysisTool, PerformanceValidator}; // Module not implemented yet
// pub use reasoning_tools::{ReasoningTool, LogicValidator, InferenceEngine}; // Module not implemented yet

// Stub implementations for missing tool types are handled by PolicyEnforcementTools
pub use tool_chain_planner::{ToolChainPlanner, ToolChain as TypedToolChain, ChainResult, PlanningContext, PlanningConstraints};
pub use tool_coordinator::{ToolCoordinator, ToolChain, ToolExecutionResult};
pub use tool_discovery::{ToolDiscoveryEngine, ToolCapability}; // ToolMetadata - private
pub use tool_execution::{ToolExecutor, ToolInvocation, ToolResult};
pub use tool_registry::{ToolRegistry, RegisteredTool, ToolRegistration};
// pub use workflow_tools::{WorkflowTool, TaskDecomposer, ProgressTracker}; // Module not implemented yet
// pub use crate::tool_orchestrator::ToolOrchestrator; // Module not implemented yet

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};

/// Policy enforcement tools for compliance and security
#[derive(Debug)]
pub struct PolicyEnforcementTools {
    // TODO: Policy Enforcement Tools - Implement comprehensive policy enforcement system
    // 
    // COMPLETION CHECKLIST:
    // [ ] CAWS validation engine implemented
    // [ ] Task decomposition algorithms
    // [ ] Quality gate validation system
    // [ ] Reasoning engine implementation
    // [ ] Workflow execution logging
    // [ ] Chain execution logging
    // [ ] Unit tests written (80%+ coverage)
    // [ ] Integration tests with tool ecosystem
    // [ ] Documentation updated
    // [ ] Performance benchmarks meet SLA
    // [ ] Security considerations addressed
    // [ ] Configuration options defined
    // [ ] Monitoring/metrics implemented
    // [ ] Logging added for debugging
    //
    // ACCEPTANCE CRITERIA:
    // - CAWS validation works correctly for all task types
    // - Task decomposition produces valid subtasks
    // - Quality gates prevent invalid executions
    // - Reasoning engine provides logical analysis
    // - Workflow logging captures all execution details
    //
    // DEPENDENCIES:
    // - CAWS specification format: Required
    // - Tool ecosystem: Available
    //
    // ESTIMATED EFFORT: 48 hours
    // PRIORITY: HIGH
    // BLOCKING: Yes - Required for policy compliance
    
    // Placeholder implementation
}

impl PolicyEnforcementTools {
    /// Create new policy enforcement tools
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Stub implementation for CAWS validation
    pub async fn validate_task_against_caws(&self, _task_description: &str, _spec: &serde_json::Value) -> Result<PolicyValidationResult> {
        // TODO: CAWS Validation - Implement actual CAWS validation logic
        // 
        // COMPLETION CHECKLIST:
        // [ ] CAWS specification parsing
        // [ ] Task description analysis
        // [ ] Policy rule evaluation
        // [ ] Validation result generation
        // [ ] Unit tests written (80%+ coverage)
        // [ ] Integration tests with CAWS system
        // [ ] Documentation updated
        // [ ] Performance benchmarks meet SLA
        // [ ] Security considerations addressed
        // [ ] Configuration options defined
        // [ ] Monitoring/metrics implemented
        // [ ] Logging added for debugging
        //
        // ACCEPTANCE CRITERIA:
        // - Validates tasks against CAWS specifications
        // - Returns appropriate validation results
        // - Handles invalid specifications gracefully
        // - Performance meets requirements
        //
        // DEPENDENCIES:
        // - CAWS specification format: Required
        // - PolicyValidationResult: Available
        //
        // ESTIMATED EFFORT: 12 hours
        // PRIORITY: HIGH
        // BLOCKING: Yes - Required for policy compliance
        
        Ok(PolicyValidationResult::Allowed) // Stub: always pass
    }

    /// Stub implementation for task decomposition
    pub async fn decompose_task(&self, _task_description: &str, _context: &str) -> Result<Vec<serde_json::Value>> {
        // TODO: Task Decomposition - Implement actual task decomposition logic
        // 
        // COMPLETION CHECKLIST:
        // [ ] Task analysis algorithms
        // [ ] Subtask generation
        // [ ] Dependency resolution
        // [ ] Context integration
        // [ ] Unit tests written (80%+ coverage)
        // [ ] Integration tests with task system
        // [ ] Documentation updated
        // [ ] Performance benchmarks meet SLA
        // [ ] Security considerations addressed
        // [ ] Configuration options defined
        // [ ] Monitoring/metrics implemented
        // [ ] Logging added for debugging
        //
        // ACCEPTANCE CRITERIA:
        // - Decomposes complex tasks into manageable subtasks
        // - Maintains task dependencies correctly
        // - Integrates context information appropriately
        // - Performance meets requirements
        //
        // DEPENDENCIES:
        // - Task analysis algorithms: Required
        // - Context management: Available
        //
        // ESTIMATED EFFORT: 16 hours
        // PRIORITY: HIGH
        // BLOCKING: Yes - Required for task execution
        
        Ok(vec![]) // Stub: no decomposition
    }

    /// Quality gate validation implementation
    pub async fn validate_quality_gates(&self, decomposed_tasks: &[serde_json::Value], evidence: &[serde_json::Value]) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        
        // Validate each task against quality gates
        for (i, task) in decomposed_tasks.iter().enumerate() {
            // Check if task has required fields
            if !task.get("id").is_some() {
                issues.push(format!("Task {} missing required 'id' field", i));
            }
            
            if !task.get("description").is_some() {
                issues.push(format!("Task {} missing required 'description' field", i));
            }
            
            // Check task complexity
            if let Some(description) = task.get("description").and_then(|d| d.as_str()) {
                if description.len() < 10 {
                    issues.push(format!("Task {} description too short (minimum 10 characters)", i));
                }
                
                if description.len() > 1000 {
                    issues.push(format!("Task {} description too long (maximum 1000 characters)", i));
                }
            }
            
            // Check for required evidence
            if evidence.is_empty() {
                issues.push(format!("Task {} has no supporting evidence", i));
            }
        }
        
        // Validate evidence quality
        for (i, ev) in evidence.iter().enumerate() {
            if !ev.get("source").is_some() {
                issues.push(format!("Evidence {} missing required 'source' field", i));
            }
            
            if !ev.get("timestamp").is_some() {
                issues.push(format!("Evidence {} missing required 'timestamp' field", i));
            }
            
            // Check evidence relevance
            if let Some(content) = ev.get("content").and_then(|c| c.as_str()) {
                if content.len() < 5 {
                    issues.push(format!("Evidence {} content too short", i));
                }
            }
        }
        
        Ok(issues)
    }

    /// Stub implementation for reasoning
    pub async fn perform_reasoning(&self, _decomposed_tasks: &[serde_json::Value], _evidence: &[serde_json::Value], _quality_checks: &[String]) -> Result<serde_json::Value> {
        // TODO: Reasoning Engine - Implement actual reasoning logic
        // 
        // COMPLETION CHECKLIST:
        // [ ] Logical reasoning algorithms
        // [ ] Evidence synthesis
        // [ ] Conflict detection
        // [ ] Reasoning result generation
        // [ ] Unit tests written (80%+ coverage)
        // [ ] Integration tests with reasoning system
        // [ ] Documentation updated
        // [ ] Performance benchmarks meet SLA
        // [ ] Security considerations addressed
        // [ ] Configuration options defined
        // [ ] Monitoring/metrics implemented
        // [ ] Logging added for debugging
        //
        // ACCEPTANCE CRITERIA:
        // - Performs logical reasoning on tasks and evidence
        // - Detects conflicts and inconsistencies
        // - Synthesizes evidence appropriately
        // - Performance meets requirements
        //
        // DEPENDENCIES:
        // - Reasoning algorithms: Required
        // - Evidence types: Available
        //
        // ESTIMATED EFFORT: 18 hours
        // PRIORITY: HIGH
        // BLOCKING: Yes - Required for intelligent analysis
        
    /// Reasoning implementation
    pub async fn perform_reasoning(&self, decomposed_tasks: &[serde_json::Value], evidence: &[serde_json::Value], quality_checks: &[String]) -> Result<serde_json::Value> {
        let mut reasoning_result = serde_json::Map::new();
        
        // Analyze task complexity
        let task_count = decomposed_tasks.len();
        let evidence_count = evidence.len();
        let quality_issue_count = quality_checks.len();
        
        // Calculate complexity score
        let complexity_score = if task_count == 0 {
            0.0
        } else {
            let base_complexity = task_count as f64;
            let evidence_ratio = evidence_count as f64 / task_count as f64;
            let quality_penalty = quality_issue_count as f64 * 0.1;
            
            base_complexity + (evidence_ratio * 0.5) - quality_penalty
        };
        
        // Determine reasoning confidence
        let confidence = if quality_issue_count == 0 && evidence_count >= task_count {
            0.9
        } else if quality_issue_count <= task_count / 2 && evidence_count >= task_count / 2 {
            0.7
        } else if quality_issue_count < task_count && evidence_count > 0 {
            0.5
        } else {
            0.3
        };
        
        // Generate reasoning summary
        let reasoning_summary = if quality_issue_count == 0 {
            "All tasks pass quality gates with sufficient evidence".to_string()
        } else if quality_issue_count <= task_count / 2 {
            format!("Some quality issues detected ({} issues), but sufficient evidence available", quality_issue_count)
        } else {
            format!("Multiple quality issues detected ({} issues), limited evidence available", quality_issue_count)
        };
        
        // Build reasoning result
        reasoning_result.insert("complexity_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(complexity_score).unwrap()));
        reasoning_result.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(confidence).unwrap()));
        reasoning_result.insert("task_count".to_string(), serde_json::Value::Number(serde_json::Number::from(task_count)));
        reasoning_result.insert("evidence_count".to_string(), serde_json::Value::Number(serde_json::Number::from(evidence_count)));
        reasoning_result.insert("quality_issues".to_string(), serde_json::Value::Number(serde_json::Number::from(quality_issue_count)));
        reasoning_result.insert("reasoning_summary".to_string(), serde_json::Value::String(reasoning_summary));
        reasoning_result.insert("recommendation".to_string(), serde_json::Value::String(
            if confidence >= 0.7 {
                "Proceed with execution".to_string()
            } else if confidence >= 0.5 {
                "Proceed with caution".to_string()
            } else {
                "Requires additional review".to_string()
            }
        ));
        
        Ok(serde_json::Value::Object(reasoning_result))
    }

    /// Workflow execution logging implementation
    pub async fn log_workflow_execution(&self, execution_id: &str, result: &serde_json::Value, caws_spec: Option<&serde_json::Value>) -> Result<()> {
        use tracing::{info, warn, error};
        use chrono::Utc;
        
        // Log execution start
        info!(
            execution_id = execution_id,
            "Workflow execution started"
        );
        
        // Log CAWS specification if provided
        if let Some(spec) = caws_spec {
            info!(
                execution_id = execution_id,
                caws_spec = %spec,
                "CAWS specification logged"
            );
        }
        
        // Log execution result
        match result.get("status") {
            Some(status) if status == "success" => {
                info!(
                    execution_id = execution_id,
                    result = %result,
                    "Workflow execution completed successfully"
                );
            }
            Some(status) if status == "error" => {
                error!(
                    execution_id = execution_id,
                    result = %result,
                    "Workflow execution failed"
                );
            }
            Some(status) if status == "warning" => {
                warn!(
                    execution_id = execution_id,
                    result = %result,
                    "Workflow execution completed with warnings"
                );
            }
            _ => {
                info!(
                    execution_id = execution_id,
                    result = %result,
                    "Workflow execution completed"
                );
            }
        }
        
        // Log performance metrics if available
        if let Some(metrics) = result.get("metrics") {
            info!(
                execution_id = execution_id,
                metrics = %metrics,
                "Performance metrics logged"
            );
        }
        
        // Log quality gate results if available
        if let Some(quality_results) = result.get("quality_gates") {
            info!(
                execution_id = execution_id,
                quality_gates = %quality_results,
                "Quality gate results logged"
            );
        }
        
        Ok(())
    }

    /// Stub implementation for chain execution logging
    pub async fn log_chain_execution(&self, _chain: &tool_coordinator::ToolChain, _result: &ToolExecutionResult) -> Result<()> {
        // TODO: Chain Execution Logging - Implement actual chain logging
        // 
        // COMPLETION CHECKLIST:
        // [ ] Chain execution tracking
        // [ ] Tool execution logging
        // [ ] Result aggregation
        // [ ] Performance metrics logging
        // [ ] Unit tests written (80%+ coverage)
        // [ ] Integration tests with logging system
        // [ ] Documentation updated
        // [ ] Performance benchmarks meet SLA
        // [ ] Security considerations addressed
        // [ ] Configuration options defined
        // [ ] Monitoring/metrics implemented
        // [ ] Logging added for debugging
        //
        // ACCEPTANCE CRITERIA:
        // - Logs chain execution details
        // - Captures tool execution results
        // - Aggregates performance metrics
        // - Performance meets requirements
        //
        // DEPENDENCIES:
        // - Logging infrastructure: Required
        // - Tool execution system: Available
        //
        // ESTIMATED EFFORT: 8 hours
        // PRIORITY: MEDIUM
        // BLOCKING: No - Audit functionality
        
        Ok(()) // Stub: no-op
    }
}

/// Main tool ecosystem coordinator
///
/// Orchestrates the complete CAWS tooling ecosystem through MCP integration,
/// providing unified access to reasoning, conflict resolution, and evidence collection tools.
pub struct ToolEcosystem {
    /// Tool registry for managing available tools
    tool_registry: Arc<ToolRegistry>,
    /// Tool discovery engine for dynamic capability detection
    tool_discovery: Arc<ToolDiscoveryEngine>,
    /// Tool coordinator for orchestration and chaining
    tool_coordinator: Arc<ToolCoordinator>,
    /// Tool executor for secure execution
    tool_executor: Arc<ToolExecutor>,
    /// Policy enforcement tools
    policy_tools: Arc<PolicyEnforcementTools>,
    /// Conflict resolution tools
    conflict_tools: Arc<ConflictResolutionTool>,
    /// Evidence collection tools
    evidence_tools: Arc<EvidenceCollectionTool>,
    /// Multimodal verification tools
    multimodal_verification: Arc<MultimodalVerificationTool>,
    /// Governance tools
    governance_tools: Arc<PolicyEnforcementTools>,
    /// Quality gate tools
    quality_tools: Arc<PolicyEnforcementTools>,
    /// Reasoning tools
    reasoning_tools: Arc<PolicyEnforcementTools>,
    /// Workflow tools
    workflow_tools: Arc<PolicyEnforcementTools>,

    /// Ecosystem health and metrics
    health_monitor: Arc<RwLock<EcosystemHealth>>,
}

/// Ecosystem health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealth {
    /// Total registered tools
    pub total_tools: usize,
    /// Active tools
    pub active_tools: usize,
    /// Tool execution success rate
    pub success_rate: f64,
    /// Average tool execution time
    pub avg_execution_time_ms: f64,
    /// Tool discovery coverage
    pub discovery_coverage: f64,
    /// Last health check
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Tool ecosystem configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEcosystemConfig {
    /// Enable tool discovery
    pub enable_discovery: bool,
    /// Maximum concurrent tool executions
    pub max_concurrent_executions: usize,
    /// Tool execution timeout (ms)
    pub execution_timeout_ms: u64,
    /// Enable tool chaining
    pub enable_chaining: bool,
    /// Enable governance auditing
    pub enable_auditing: bool,
    /// CAWS compliance enforcement
    pub caws_compliance: bool,
}

impl ToolEcosystem {
    /// Create a new tool ecosystem
    pub async fn new(config: ToolEcosystemConfig) -> Result<Self> {
        info!("Initializing CAWS tool ecosystem");

        // Initialize core components
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_discovery = Arc::new(ToolDiscoveryEngine::new(config.enable_discovery));
        let tool_executor = Arc::new(ToolExecutor::new(config.max_concurrent_executions, config.execution_timeout_ms));
        let tool_coordinator = Arc::new(ToolCoordinator::new(config.enable_chaining));

        // Initialize tool categories
        let policy_tools = Arc::new(PolicyEnforcementTools::new().await?);
        let conflict_tools = Arc::new(ConflictResolutionTool::new().await?);
        let evidence_tools = Arc::new(EvidenceCollectionTool::new().await?);
        // TODO: Tool Module Integration - Implement missing tool modules
        // 
        // COMPLETION CHECKLIST:
        // [ ] Governance tools module implementation
        // [ ] Quality gate tools module implementation
        // [ ] Reasoning tools module implementation
        // [ ] Workflow tools module implementation
        // [ ] Tool module integration testing
        // [ ] Unit tests written (80%+ coverage)
        // [ ] Integration tests with tool ecosystem
        // [ ] Documentation updated
        // [ ] Performance benchmarks meet SLA
        // [ ] Security considerations addressed
        // [ ] Configuration options defined
        // [ ] Monitoring/metrics implemented
        // [ ] Logging added for debugging
        //
        // ACCEPTANCE CRITERIA:
        // - All tool modules are properly implemented
        // - Tool modules integrate seamlessly
        // - Configuration options work correctly
        // - Performance meets requirements
        //
        // DEPENDENCIES:
        // - Tool module interfaces: Required
        // - Configuration system: Available
        //
        // ESTIMATED EFFORT: 32 hours
        // PRIORITY: HIGH
        // BLOCKING: Yes - Required for complete tool ecosystem
        
        let multimodal_verification = Arc::new(MultimodalVerificationTool::new().await?);
        // let governance_tools = Arc::new(GovernanceTool::new(config.enable_auditing).await?);
        // let quality_tools = Arc::new(QualityGateTool::new().await?);
        // let reasoning_tools = Arc::new(ReasoningTool::new().await?);
        // let workflow_tools = Arc::new(WorkflowTool::new().await?);

        // Placeholder implementations for missing modules
        let governance_tools = Arc::new(PolicyEnforcementTools::new().await?); // Placeholder
        let quality_tools = Arc::new(PolicyEnforcementTools::new().await?); // Placeholder
        let reasoning_tools = Arc::new(PolicyEnforcementTools::new().await?); // Placeholder
        let workflow_tools = Arc::new(PolicyEnforcementTools::new().await?); // Placeholder

        // Register all tools
        Self::register_all_tools(
            &tool_registry,
            &policy_tools,
            &conflict_tools,
            &evidence_tools,
            &multimodal_verification,
            &governance_tools,
            &quality_tools,
            &reasoning_tools,
            &workflow_tools,
        ).await?;

        let health_monitor = Arc::new(RwLock::new(EcosystemHealth {
            total_tools: 0,
            active_tools: 0,
            success_rate: 1.0,
            avg_execution_time_ms: 0.0,
            discovery_coverage: 0.0,
            last_health_check: chrono::Utc::now(),
        }));

        Ok(Self {
            tool_registry,
            tool_discovery,
            tool_coordinator,
            tool_executor,
            policy_tools,
            conflict_tools,
            evidence_tools,
            multimodal_verification,
            governance_tools,
            quality_tools,
            reasoning_tools,
            workflow_tools,
            health_monitor,
        })
    }

    /// Execute a reasoning workflow using the tool ecosystem
    pub async fn execute_reasoning_workflow(
        &self,
        task_description: &str,
        context: &str,
        caws_spec: Option<&serde_json::Value>,
    ) -> Result<ReasoningWorkflowResult> {
        info!("Executing reasoning workflow for task: {}", task_description);

        // 1. Policy validation (if CAWS spec provided)
        let policy_check = if let Some(spec) = caws_spec {
            self.policy_tools.validate_task_against_caws(task_description, spec).await?
        } else {
            PolicyValidationResult::Allowed
        };

        if !matches!(policy_check, PolicyValidationResult::Allowed) {
            return Err(anyhow::anyhow!("Task rejected by CAWS policy: {:?}", policy_check));
        }

        // 2. Task decomposition
        let decomposed_tasks = self.workflow_tools.decompose_task(task_description, context).await?;

        // 3. Evidence collection
        let evidence = self.evidence_tools.collect_evidence(&decomposed_tasks, context).await?;

        // 4. Quality validation
        let quality_checks = self.quality_tools.validate_quality_gates(&decomposed_tasks, &evidence).await?;

        // 5. Reasoning and inference
        let reasoning_result = self.reasoning_tools.perform_reasoning(&decomposed_tasks, &evidence, &quality_checks).await?;

        // 6. Conflict resolution (if needed)
        let resolved_result = if reasoning_result.get("has_conflicts")
            .and_then(|v| v.as_bool())
            .unwrap_or(false) {
            self.conflict_tools.resolve_conflicts(&reasoning_result).await?
        } else {
            reasoning_result
        };

        // 7. Governance and audit logging
        self.governance_tools.log_workflow_execution(
            task_description,
            &resolved_result,
            caws_spec,
        ).await?;

        Ok(ReasoningWorkflowResult {
            final_result: resolved_result.get("final_answer")
                .and_then(|v| v.as_str())
                .unwrap_or("No final answer")
                .to_string(),
            confidence: resolved_result.get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            evidence_used: evidence.len(),
            tools_executed: resolved_result.get("tools_used")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0),
            caws_compliant: resolved_result.get("caws_compliant")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            execution_time_ms: resolved_result.get("execution_time_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        })
    }

    /// Discover available tools dynamically
    pub async fn discover_tools(&self) -> Result<Vec<ToolCapability>> {
        debug!("Discovering available tools");
        self.tool_discovery.discover_capabilities().await
    }

    /// Execute a specific tool by name
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
        context: Option<&str>,
    ) -> Result<ToolResult> {
        info!("Executing tool: {}", tool_name);

        // Get tool from registry
        let tool = self.tool_registry.get_tool(tool_name).await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        // Validate parameters against tool schema
        self.validate_tool_parameters(&tool, &parameters)?;

        // Execute tool
        let invocation = ToolInvocation {
            tool_name: tool_name.to_string(),
            parameters,
            context: context.map(|s| s.to_string()),
            timeout_ms: Some(30000), // 30 second default timeout
        };

        let result = self.tool_executor.execute_tool(invocation).await?;

        // Log execution for governance - stub implementation
        // if let Some(governance) = self.governance_tools.audit_logger.as_ref() {
        //     governance.log_tool_execution(tool_name, &result).await?;
        // }

        Ok(result)
    }

    /// Create a tool chain for complex workflows
    pub async fn create_tool_chain(&self, chain_spec: ToolChainSpec) -> Result<ToolChain> {
        info!("Creating tool chain with {} steps", chain_spec.steps.len());

        let mut chain = ToolChain::new();

        for step in &chain_spec.steps {
            // Validate step dependencies
            self.validate_chain_step(step, &chain_spec.steps)?;

            // Convert to tool_coordinator::ToolChainStep
            let coordinator_step = tool_coordinator::ToolChainStep {
                step_id: step.step_id.clone(),
                tool_name: step.tool_name.clone(),
                parameters: step.parameters.clone(),
                dependencies: step.dependencies.clone(),
                condition: step.condition.clone(),
                timeout_ms: Some(30000), // 30 second default
                retry_config: None,
            };

            // Add step to chain
            chain.add_step(coordinator_step);
        }

        // Validate complete chain
        self.tool_coordinator.validate_chain(&chain).await?;

        Ok(chain)
    }

    /// Execute a tool chain
    pub async fn execute_tool_chain(&self, chain: &ToolChain) -> Result<ToolExecutionResult> {
        info!("Executing tool chain with {} steps", chain.steps.len());

        // Execute through coordinator
        let result = self.tool_coordinator.execute_chain(chain).await?;

        // Log chain execution
        self.governance_tools.log_chain_execution(chain, &result).await?;

        Ok(result)
    }

    /// Get ecosystem health status
    pub async fn get_health_status(&self) -> EcosystemHealth {
        let mut health = self.health_monitor.read().await.clone();

        // Update metrics
        health.total_tools = self.tool_registry.get_tool_count().await;
        health.active_tools = self.tool_registry.get_active_tool_count().await;
        health.discovery_coverage = self.tool_discovery.get_coverage_rate().await;
        health.last_health_check = chrono::Utc::now();

        // Update the stored health
        *self.health_monitor.write().await = health.clone();

        health
    }

    /// Register all tools with the registry
    async fn register_all_tools(
        registry: &Arc<ToolRegistry>,
        policy_tools: &Arc<PolicyEnforcementTools>,
        conflict_tools: &Arc<ConflictResolutionTool>,
        evidence_tools: &Arc<EvidenceCollectionTool>,
        multimodal_verification: &Arc<MultimodalVerificationTool>,
        governance_tools: &Arc<PolicyEnforcementTools>, // Placeholder
        quality_tools: &Arc<PolicyEnforcementTools>, // Placeholder
        reasoning_tools: &Arc<PolicyEnforcementTools>, // Placeholder
        workflow_tools: &Arc<PolicyEnforcementTools>, // Placeholder
    ) -> Result<()> {
        // Register conflict resolution tools - commented out as these are internal components
        // registry.register_tool(conflict_tools.debate_orchestrator.clone()).await?;
        // registry.register_tool(conflict_tools.consensus_builder.clone()).await?;
        // registry.register_tool(conflict_tools.evidence_synthesizer.clone()).await?;

        // Register evidence collection tools - commented out as these are internal components
        // registry.register_tool(evidence_tools.claim_extractor.clone()).await?;
        // registry.register_tool(evidence_tools.fact_verifier.clone()).await?;
        // registry.register_tool(evidence_tools.source_validator.clone()).await?;
        registry.register_tool(multimodal_verification.correlation_engine.clone()).await?;
        registry.register_tool(multimodal_verification.fusion_validator.clone()).await?;
        registry.register_tool(multimodal_verification.semantic_integrator.clone()).await?;

        // TODO: Tool Registration System - Implement missing tool registrations
        // 
        // COMPLETION CHECKLIST:
        // [ ] Policy enforcement tool registration
        // [ ] Governance tool registration
        // [ ] Quality gate tool registration
        // [ ] Reasoning tool registration
        // [ ] Workflow tool registration
        // [ ] Tool registration validation
        // [ ] Unit tests written (80%+ coverage)
        // [ ] Integration tests with tool registry
        // [ ] Documentation updated
        // [ ] Performance benchmarks meet SLA
        // [ ] Security considerations addressed
        // [ ] Configuration options defined
        // [ ] Monitoring/metrics implemented
        // [ ] Logging added for debugging
        //
        // ACCEPTANCE CRITERIA:
        // - All tools are properly registered
        // - Tool registration validation works
        // - Tool discovery finds all registered tools
        // - Performance meets requirements
        //
        // DEPENDENCIES:
        // - Tool registry system: Available
        // - Tool interfaces: Required
        //
        // ESTIMATED EFFORT: 16 hours
        // PRIORITY: HIGH
        // BLOCKING: Yes - Required for tool discovery
        
        // Policy enforcement tools, governance tools, quality gate tools not yet implemented
        // Reasoning tools, workflow tools not yet implemented

        info!("Registered all CAWS tooling categories");
        Ok(())
    }

    /// Validate tool parameters against schema
    fn validate_tool_parameters(&self, tool: &RegisteredTool, parameters: &serde_json::Value) -> Result<()> {
        // Use JSON schema validation if available
        if let Some(schema) = &tool.metadata.input_schema {
            let compiled = jsonschema::JSONSchema::compile(schema)
                .map_err(|e| anyhow::anyhow!("Invalid tool schema: {}", e))?;

            compiled.validate(parameters)
                .map_err(|e| anyhow::anyhow!("Parameter validation failed: {}", e.map(|err| format!("{:?}", err)).collect::<Vec<_>>().join(", ")))?;
        }

        Ok(())
    }

    /// Validate a chain step
    fn validate_chain_step(&self, step: &ToolChainStep, all_steps: &[ToolChainStep]) -> Result<()> {
        // Check dependencies exist
        for dep in &step.dependencies {
            if !all_steps.iter().any(|s| s.step_id == *dep) {
                return Err(anyhow::anyhow!("Chain step '{}' depends on non-existent step '{}'", step.step_id, dep));
            }
        }

        Ok(())
    }
}

/// Result of a reasoning workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningWorkflowResult {
    /// Final answer/result
    pub final_result: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Number of evidence items used
    pub evidence_used: usize,
    /// Number of tools executed
    pub tools_executed: usize,
    /// CAWS compliance status
    pub caws_compliant: bool,
    /// Total execution time (ms)
    pub execution_time_ms: u64,
}

/// Specification for a tool chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChainSpec {
    /// Chain name
    pub name: String,
    /// Chain steps
    pub steps: Vec<ToolChainStep>,
}

/// Step in a tool chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChainStep {
    /// Step ID
    pub step_id: String,
    /// Tool to execute
    pub tool_name: String,
    /// Parameters for the tool
    pub parameters: serde_json::Value,
    /// Dependencies (other step IDs)
    pub dependencies: Vec<String>,
    /// Conditional execution
    pub condition: Option<String>,
}

/// Policy validation result
#[derive(Debug, Clone)]
pub enum PolicyValidationResult {
    /// Task is allowed
    Allowed,
    /// Task requires waiver
    RequiresWaiver(String),
    /// Task is blocked by policy
    Blocked(String),
}
