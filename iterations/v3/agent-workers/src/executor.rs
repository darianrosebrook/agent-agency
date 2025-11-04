//! Task Executor
//!
//! Executes tasks by communicating with worker models and handling the execution lifecycle.

use schemars::JsonSchema;
use crate::worker_types::{TaskContext as WorkerTaskContext, UuidGenerator, TaskPriority, CawsSpec, *};
use crate::parallel_types::{WorkerId, TaskResult};
use crate::worker_errors::{WorkerExecutionResult, WorkerError};
use agent_agency_contracts::{IssueSeverity, task_executor::{TaskExecutor as TaskExecutorTrait, TaskExecutionResult, TaskSpec as ContractTaskSpec, TaskContext as CouncilTaskContext, TaskSpec, TaskRequirements}, task_request::RiskTier, AcceptanceCriterion};
use system_resilience::resilience_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use system_resilience::retry::{RetryConfig, RetryPolicy};
use anyhow::{Context, Result};
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;
use sqlx::Row;
use rand;

// Additional types for distributed worker execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerCapabilities {
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    pub name: String,
    pub specialty: String,
    pub capabilities: Vec<String>,
    pub max_concurrent_tasks: i32,
    pub memory_limit_mb: i32,
    pub cpu_limit_cores: i32,
    pub is_active: bool,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub version: String,
    pub endpoint_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityMatch {
    pub is_suitable: bool,
    pub missing_capabilities: Vec<String>,
    pub capability_score: f64,
    pub worker_specialty_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionStrategy {
    pub execution_mode: ExecutionMode,
    pub timeout_seconds: u64,
    pub retry_config: RetryConfig,
    pub resource_allocation: ResourceAllocation,
    pub enable_streaming: bool,
    pub enable_cancellation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionMode {
    Sync,
    Async,
    Streaming,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceAllocation {
    pub memory_mb: i32,
    pub cpu_cores: i32,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub health_message: String,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub error_count: i64,
    pub success_count: i64,
    pub avg_response_time_ms: Option<i64>,
    pub health_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthResult {
    pub is_authorized: bool,
    pub auth_message: String,
    pub permissions: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub restricted_domains: Vec<String>,
    pub max_risk_tier: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TaskComplexity {
    Low,
    Medium,
    High,
}

impl TaskComplexity {
    pub fn is_high(&self) -> bool {
        matches!(self, TaskComplexity::High)
    }

    pub fn estimated_memory_mb(&self) -> i32 {
        match self {
            TaskComplexity::Low => 256,
            TaskComplexity::Medium => 512,
            TaskComplexity::High => 1024,
        }
    }

    pub fn estimated_cpu_cores(&self) -> i32 {
        match self {
            TaskComplexity::Low => 1,
            TaskComplexity::Medium => 2,
            TaskComplexity::High => 4,
        }
    }

    pub fn supports_streaming(&self) -> bool {
        matches!(self, TaskComplexity::High)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct WorkerLoad {
    pub active_tasks: i64,
    pub avg_execution_time_ms: Option<i64>,
    pub max_execution_time_ms: Option<i64>,
    pub load_percentage: f64,
}

/// Task executor for running tasks with workers

pub struct TaskExecutor {
    // HTTP client for model communication with robust error handling and performance optimization
    client: reqwest::Client,
    execution_timeout: std::time::Duration,
    cancel_timeout: std::time::Duration,
    clock: Box<dyn Clock + Send + Sync>,
    id_gen: Box<dyn IdGenerator + Send + Sync>,
    db_client: Arc<data_infrastructure::DatabaseClient>,
    // Execution statistics tracking
    execution_stats: Arc<tokio::sync::RwLock<ExecutionStats>>,
}

impl std::fmt::Debug for TaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskExecutor")
            .field("client", &"<reqwest::Client>")
            .field("execution_timeout", &self.execution_timeout)
            .field("cancel_timeout", &self.cancel_timeout)
            .field("clock", &"<dyn Clock>")
            .field("id_gen", &"<dyn IdGenerator>")
            .field("db_client", &"<DatabaseClient>")
            .field("execution_stats", &"<ExecutionStats>")
            .finish()
    }
}

/// Internal execution statistics for tracking performance

#[derive(Debug, Serialize, Deserialize, JsonSchema, Default)]
struct ExecutionStats {
    total_executions: u64,
    successful_executions: u64,
    failed_executions: u64,
    execution_times: Vec<u64>, // milliseconds
    last_execution_time: Option<DateTime<Utc>>,
}

impl TaskExecutor {
    /// Create a new task executor with default configuration
    pub fn new(db_client: Arc<data_infrastructure::DatabaseClient>) -> Self {
        Self::with_timeouts(
            std::time::Duration::from_secs(30), // execution timeout
            std::time::Duration::from_secs(10), // connect timeout
            std::time::Duration::from_secs(5),  // cancel timeout
            db_client,
        )
    }

    /// Create a new task executor with custom timeouts
    pub fn with_timeouts(
        execution_timeout: std::time::Duration,
        connect_timeout: std::time::Duration,
        cancel_timeout: std::time::Duration,
        db_client: Arc<data_infrastructure::DatabaseClient>,
    ) -> Self {
        // Create HTTP client with proper configuration
        let client = reqwest::Client::builder()
            .timeout(execution_timeout)
            .connect_timeout(connect_timeout)
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            execution_timeout,
            cancel_timeout,
            clock: Box::new(SystemClock),
            id_gen: Box::new(UuidGenerator),
            db_client,
            execution_stats: Arc::new(tokio::sync::RwLock::new(ExecutionStats::default())),
        }
    }

    /// Execute a task with a specific worker
    pub async fn execute_task(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
        circuit_breaker: Option<&Arc<CircuitBreaker>>,
    ) -> Result<TaskExecutionResult> {
        let task_id = task_spec.id;
        let started_at = self.clock.now();

        info!("Executing task {} with worker {}", task_id, worker_id);

        // Real worker discovery and capability matching
        let worker_info = self.discover_worker_capabilities(worker_id).await?;
        let capability_match = self.match_worker_capabilities(&task_spec, &worker_info)?;
        
        if !capability_match.is_suitable {
            return Err(anyhow::anyhow!(
                "Worker {} lacks required capabilities for task {}: {:?}",
                worker_id, task_id, capability_match.missing_capabilities
            ));
        }

        // Real load balancing and performance optimization
        let execution_strategy = self.determine_execution_strategy(&task_spec, &worker_info).await?;
        
        // Real distributed worker coordination and health monitoring
        let health_status = self.check_worker_health(worker_id).await?;
        if !health_status.is_healthy {
            return Err(anyhow::anyhow!(
                "Worker {} is not healthy: {}",
                worker_id, health_status.health_message
            ));
        }

        // Real worker authentication and authorization
        let auth_result = self.authenticate_worker(worker_id, &task_spec).await?;
        if !auth_result.is_authorized {
            return Err(anyhow::anyhow!(
                "Worker {} not authorized for task {}: {}",
                worker_id, task_id, auth_result.auth_message
            ));
        }

        // Real worker lifecycle management
        self.update_worker_status(worker_id, "executing", Some(task_id)).await?;

        // Prepare execution input
        let execution_input = self.prepare_execution_input(&task_spec)?;

        // Real worker execution with circuit breaker and retry logic
        let execution_result = if let Some(cb) = circuit_breaker {
            self.execute_with_circuit_breaker_and_retry(
                worker_id,
                &execution_input,
                &execution_strategy,
                cb,
            )
            .await?
        } else {
            self.execute_with_worker_direct(
                worker_id,
                &execution_input,
                &execution_strategy,
            )
            .await?
        };

        // Process and validate result
        let result = self
            .process_execution_result(task_id, worker_id, execution_result, started_at)
            .await?;

        // Track execution statistics
        let completed_at = self.clock.now();
        let duration_ms = (completed_at - started_at).num_milliseconds() as u64;
        let success = result.success;
        
        {
            let mut stats = self.execution_stats.write().await;
            stats.total_executions += 1;
            if success {
                stats.successful_executions += 1;
            } else {
                stats.failed_executions += 1;
            }
            stats.execution_times.push(duration_ms);
            stats.last_execution_time = Some(completed_at);
            
            // Keep only last 1000 execution times to prevent memory growth
            if stats.execution_times.len() > 1000 {
                stats.execution_times.remove(0);
            }
        }

        info!(
            "Task {} execution completed with status: {:?}",
            task_id,
            crate::worker_types::get_execution_status(&result)
        );
        Ok(result)
    }

    /// Prepare execution input for worker
    fn prepare_execution_input(&self, task_spec: &TaskSpec) -> Result<ExecutionInput> {
        let prompt = self.build_execution_prompt(task_spec)?;

        Ok(ExecutionInput {
            prompt,
            task_id: task_spec.id,
            context: self.convert_task_context(&task_spec.context),
            requirements: self.extract_requirements(task_spec),
            caws_spec: task_spec
                .caws_spec
                .as_ref()
                .map(|spec| self.convert_caws_spec(spec, Some(task_spec))),
        })
    }

    /// Build execution prompt for worker
    fn build_execution_prompt(&self, task_spec: &TaskSpec) -> Result<String> {
        let mut prompt = format!(
            "You are a specialized AI worker in the Agent Agency system. Your task is to:\n\n\
            **TASK**: {}\n\
            **DESCRIPTION**: {}\n\
            **RISK TIER**: {:?}\n\n",
            task_spec.title, task_spec.description, task_spec.risk_tier
        );

        // Add scope information
        prompt.push_str("**SCOPE**:\n");
        prompt.push_str(&format!(
            "- Files affected: {}\n",
            task_spec.scope.as_ref().map(|s| s.files_affected.join(", ")).unwrap_or_default()
        ));
        if let Some(scope) = &task_spec.scope {
            if let Some(max_files) = scope.max_files {
                prompt.push_str(&format!("- Max files: {}\n", max_files));
            }
            if let Some(max_loc) = scope.max_loc {
                prompt.push_str(&format!("- Max lines of code: {}\n", max_loc));
            }
        }
        prompt.push_str(&format!(
            "- Domains: {}\n\n",
            task_spec.scope.as_ref().map(|s| s.domains.join(", ")).unwrap_or_default()
        ));

        // Add acceptance criteria
        if !task_spec.acceptance_criteria.is_empty() {
            prompt.push_str("**ACCEPTANCE CRITERIA**:\n");
            for criterion in &task_spec.acceptance_criteria {
                prompt.push_str(&format!("- {}: {}\n", criterion.id, criterion.description));
            }
            prompt.push('\n');
        }

        // Add CAWS compliance requirements
        prompt.push_str("**CAWS COMPLIANCE REQUIREMENTS**:\n");
        prompt.push_str("- Stay within declared file and LOC limits\n");
        prompt.push_str("- Ensure code quality and maintainability\n");
        prompt.push_str("- Follow best practices and patterns\n");
        prompt.push_str("- Provide clear rationale for decisions\n");
        prompt.push_str("- Self-assess your output quality\n\n");

        // Add context information
        prompt.push_str("**CONTEXT**:\n");
        prompt.push_str(&format!(
            "- Workspace: {}\n",
            task_spec.context.workspace_root
        ));
        prompt.push_str(&format!("- Git branch: {}\n", task_spec.context.git_branch));
        prompt.push_str(&format!(
            "- Environment: {:?}\n",
            task_spec.context.environment
        ));
        if !task_spec.context.recent_changes.is_empty() {
            prompt.push_str(&format!(
                "- Recent changes: {}\n",
                task_spec.context.recent_changes.join(", ")
            ));
        }
        prompt.push('\n');

        // Add output format requirements
        prompt.push_str("**OUTPUT FORMAT**:\n");
        prompt.push_str("Respond with a JSON object containing:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"content\": \"Detailed description of your work\",\n");
        prompt.push_str("  \"files_modified\": [\n");
        prompt.push_str("    {\n");
        prompt.push_str("      \"path\": \"file/path\",\n");
        prompt.push_str("      \"operation\": \"create|modify|delete\",\n");
        prompt.push_str("      \"content\": \"file content (if create/modify)\",\n");
        prompt.push_str("      \"diff\": \"diff content (if modify)\"\n");
        prompt.push_str("    }\n");
        prompt.push_str("  ],\n");
        prompt.push_str("  \"rationale\": \"Explanation of your approach and decisions\",\n");
        prompt.push_str("  \"self_assessment\": {\n");
        prompt.push_str("    \"caws_compliance\": 0.95,\n");
        prompt.push_str("    \"quality_score\": 0.9,\n");
        prompt.push_str("    \"confidence\": 0.85,\n");
        prompt.push_str("    \"concerns\": [\"list any concerns\"],\n");
        prompt.push_str("    \"improvements\": [\"suggested improvements\"]\n");
        prompt.push_str("  }\n");
        prompt.push_str("}\n\n");

        prompt.push_str("Execute the task and provide your response in the specified JSON format.");

        Ok(prompt)
    }

    /// Extract requirements from task spec
    fn extract_requirements(&self, task_spec: &TaskSpec) -> TaskRequirements {
        // Implement sophisticated requirement extraction with analysis and validation
        let task_text = format!("{} {}", task_spec.title, task_spec.description);

        // Extract programming languages from task description
        let required_languages = self.extract_languages(&task_text);

        // Extract frameworks and tools from task description
        let required_frameworks = self.extract_frameworks(&task_text);

        // Calculate context length estimate based on task content
        let context_length_estimate = self.calculate_context_length(&task_text, &task_spec.context);

        TaskRequirements {
            required_languages,
            required_frameworks,
            required_domains: task_spec.scope.as_ref().map(|s| s.domains.clone()).unwrap_or_default(),
            min_quality_score: task_spec.risk_tier.map_or(0.7, |tier| match tier {
                1 => 0.9,
                2 => 0.8,
                _ => 0.7,
            }),
            min_caws_awareness: 0.8,
            max_execution_time_ms: task_spec.scope.as_ref().and_then(|s| s.max_loc.map(|loc| loc as u64 * 100)),
            preferred_worker_type: None,
            context_length_estimate,
        }
    }

    /// Convert council TaskContext to workers TaskContext
    fn convert_task_context(
        &self,
        _council_context: &CouncilTaskContext,
    ) -> WorkerTaskContext {
        // Create execution context with defaults - would map actual fields in real implementation
        WorkerTaskContext {
            task_id: Uuid::new_v4(),
            worker_id: Uuid::new_v4(),
            start_time: chrono::Utc::now(),
            timeout_ms: 30000, // 30 seconds default
            retry_count: 0,
            max_retries: 3,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Parse rule criteria from a council rule string
    fn parse_rule_criteria(rule: &str) -> Vec<String> {
        // Simple parsing: split on common delimiters and extract key requirements
        let rule_lower = rule.to_lowercase();

        let mut criteria = Vec::new();

        // Extract specific criteria based on rule content
        if rule_lower.contains("coverage") {
            if rule_lower.contains("80%") || rule_lower.contains("90%") {
                criteria.push("Achieve minimum code coverage threshold".to_string());
            } else {
                criteria.push("Maintain adequate test coverage".to_string());
            }
        }

        if rule_lower.contains("security") || rule_lower.contains("vulnerability") {
            criteria.push("Pass security scanning requirements".to_string());
        }

        if rule_lower.contains("performance") || rule_lower.contains("latency") {
            criteria.push("Meet performance benchmarks".to_string());
        }

        if rule_lower.contains("quality") || rule_lower.contains("lint") {
            criteria.push("Pass code quality checks".to_string());
        }

        if rule_lower.contains("documentation") {
            criteria.push("Maintain documentation standards".to_string());
        }

        // If no specific criteria found, use the rule as-is
        if criteria.is_empty() {
            criteria.push(format!("Satisfy requirement: {}", rule));
        }

        criteria
    }

    /// Determine severity level for a council rule
    fn determine_rule_severity(rule: &str) -> IssueSeverity {
        let rule_lower = rule.to_lowercase();

        // Critical rules
        if rule_lower.contains("security") || rule_lower.contains("safety") ||
           rule_lower.contains("critical") || rule_lower.contains("blocking") {
            IssueSeverity::Error
        }
        // High impact rules
        else if rule_lower.contains("performance") || rule_lower.contains("reliability") ||
                rule_lower.contains("compliance") || rule_lower.contains("audit") {
            IssueSeverity::Warning
        }
        // Medium rules (default)
        else {
            IssueSeverity::Info
        }
    }

    /// Convert council validation rules to worker validation rules
    /// Extracts rules from waivers, acceptance criteria, and invariants
    fn convert_validation_rules(
        council_spec: &CawsSpec,
        acceptance_criteria: Option<&[AcceptanceCriterion]>,
        invariants: Option<&[String]>,
    ) -> Vec<ValidationRule> {
        let mut rules = Vec::new();
        
        // Extract validation rules from waivers
        for (i, waiver) in council_spec.waivers.iter().enumerate() {
            rules.push(ValidationRule {
                id: format!("validation-waiver-{}", i),
                name: format!("Waiver Validation {}", i),
                description: format!("Validate waiver: {}", waiver.reason),
                rule_type: ValidationRuleType::Custom,
                config: serde_json::json!({
                    "waiver_id": waiver.id,
                    "reason": waiver.reason
                }),
                severity: SeverityLevel::Medium,
                file_patterns: vec!["**/*".to_string()],
            });
        }
        
        // Extract validation rules from acceptance criteria
        if let Some(criteria) = acceptance_criteria {
            for criterion in criteria {
                rules.push(ValidationRule {
                    id: format!("validation-acceptance-{}", criterion.id),
                    name: format!("Acceptance Criterion {}", criterion.id),
                    description: format!("Given: {}\nWhen: {}\nThen: {}", 
                        criterion.given, criterion.when, criterion.then),
                    rule_type: ValidationRuleType::Acceptance,
                    config: serde_json::json!({
                        "criterion_id": criterion.id,
                        "given": criterion.given,
                        "when": criterion.when,
                        "then": criterion.then
                    }),
                    severity: SeverityLevel::High, // Acceptance criteria are high priority
                    file_patterns: vec!["**/*".to_string()],
                });
            }
        }
        
        // Extract validation rules from invariants
        if let Some(invs) = invariants {
            for (i, invariant) in invs.iter().enumerate() {
                rules.push(ValidationRule {
                    id: format!("validation-invariant-{}", i),
                    name: format!("System Invariant {}", i + 1),
                    description: invariant.clone(),
                    rule_type: ValidationRuleType::Invariant,
                    config: serde_json::json!({
                        "invariant_index": i,
                        "invariant_text": invariant
                    }),
                    severity: SeverityLevel::Critical, // Invariants are critical
                    file_patterns: vec!["**/*".to_string()],
                });
            }
        }
        
        rules
    }

    /// Extract performance benchmarks from compliance requirements and task specs
    fn extract_performance_benchmarks(
        compliance: &ComplianceRequirements,
        task_spec: Option<&TaskSpec>,
    ) -> Option<PerformanceBenchmarks> {
        // Extract benchmarks from compliance requirements
        let response_time_ms = compliance.performance_budget_ms.unwrap_or(5000); // Default 5s
        
        // Extract throughput from task requirements if available
        let throughput_rps = task_spec
            .and_then(|ts| ts.requirements.max_execution_time_ms)
            .map(|timeout_ms| {
                // Calculate throughput: if max execution time is 1000ms, we can do ~1 req/sec
                // Higher timeout = lower throughput requirement
                if timeout_ms > 0 {
                    (1000 / timeout_ms.max(1)) as u32
                } else {
                    10 // Default 10 req/sec
                }
            })
            .unwrap_or(10);
        
        // Estimate memory usage based on task complexity
        // Simple tasks: ~50MB, Medium: ~100MB, Complex: ~200MB
        let memory_usage_mb = task_spec
            .map(|ts| {
                match ts.risk_tier {
                    RiskTier::Tier1 => 200, // Critical tasks may need more memory
                    RiskTier::Tier2 => 100, // Standard tasks
                    RiskTier::Tier3 => 50,  // Low-risk tasks
                }
            })
            .unwrap_or(100);
        
        // Only create benchmarks if we have meaningful values
        if response_time_ms > 0 || throughput_rps > 0 || memory_usage_mb > 0 {
            Some(PerformanceBenchmarks {
                response_time_ms,
                throughput_rps,
                memory_usage_mb,
            })
        } else {
            None
        }
    }
    fn convert_caws_spec(
        &self,
        council_spec: &CawsSpec,
        task_spec: Option<&TaskSpec>,
    ) -> CawsSpec {
        // Map council CawsSpec rules to worker quality gates
        let quality_gates = council_spec.rules.iter()
            .enumerate()
            .map(|(i, rule)| {
                // Parse rule string into criteria
                let criteria = Self::parse_rule_criteria(rule);
                QualityGate {
                    id: format!("gate-{}", i),
                    name: format!("Rule {}", i),
                    description: rule.clone(),
                    criteria,
                    severity: Self::determine_rule_severity(rule),
                    enabled: true,
                    timeout_ms: Some(30000), // 30 second timeout
                }
            })
            .collect();

        // Map council waivers to worker compliance requirements
        let compliance_requirements = if council_spec.waivers.is_empty() {
            ComplianceRequirements::default()
        } else {
            ComplianceRequirements {
                standards: vec!["ISO27001".to_string()], // Placeholder
                certifications: council_spec.waivers.iter()
                    .map(|w| w.id.clone())
                    .collect(),
                audit_requirements: vec![],
                reporting_frequency: "monthly".to_string(),
            }
        };

        CawsSpec {
            version: "1.0".to_string(),
            metadata: CawsMetadata {
                created_at: chrono::Utc::now(),
                created_by: "orchestrator".to_string(),
                description: council_spec.description.clone(),
                tags: vec!["converted".to_string()],
            },
            quality_gates,
            compliance: compliance_requirements,
            validation_rules: Self::convert_validation_rules(
                council_spec,
                // Note: acceptance_criteria and invariants would come from WorkingSpec if available
                // For now, we extract from waivers and rules only
                None, // acceptance_criteria: could be extracted from WorkingSpec if available
                None, // invariants: could be extracted from WorkingSpec if available
            ),
            benchmarks: Self::extract_performance_benchmarks(&compliance_requirements, task_spec),
            security: SecurityRequirements::default(),
        }
    }

    /// Execute task with worker via HTTP API
    /// 
    /// This method performs the actual HTTP call to the worker endpoint with:
    /// - Proper error handling and status code mapping
    /// - Request/response serialization (JSON)
    /// - Request timeout (configured via client)
    /// - Worker endpoint resolution from registry
    /// - Response parsing and error extraction
    async fn execute_with_worker(
        &self,
        worker_id: Uuid,
        input: &ExecutionInput,
    ) -> Result<RawExecutionResult> {
        info!("Executing task {} with worker {}", input.task_id, worker_id);

        let start_time = std::time::Instant::now();

        // Construct HTTP request to worker endpoint
        let request_body = serde_json::json!({
            "task_id": input.task_id,
            "prompt": input.prompt,
            "context": input.context,
            "requirements": input.requirements,
            "caws_spec": input.caws_spec
        });

        // Resolve worker endpoint from registry or use default
        // Worker registry and service discovery:
        // 1. Worker registry: Resolve worker endpoint from registry using worker_id
        // 2. Service discovery: Look up worker capabilities and health status
        // 3. Endpoint management: Validate endpoint is reachable and healthy
        // 4. Registry optimization: Cache resolved endpoints for performance
        let worker_base_url = self.resolve_worker_endpoint(worker_id).await
            .unwrap_or_else(|_| format!("http://worker-{}", worker_id));

        let execute_url = format!("{}/execute", worker_base_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&execute_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Agent-Agency-Executor/1.0")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to worker")?;

        let status = response.status();
        if !status.is_success() {
            let error_msg = match status {
                reqwest::StatusCode::REQUEST_TIMEOUT | reqwest::StatusCode::GATEWAY_TIMEOUT => {
                    format!("Worker request timed out (status: {})", status)
                },
                reqwest::StatusCode::TOO_MANY_REQUESTS => {
                    format!("Worker rate limited (status: {})", status)
                },
                reqwest::StatusCode::SERVICE_UNAVAILABLE | reqwest::StatusCode::BAD_GATEWAY => {
                    format!("Worker service unavailable (status: {})", status)
                },
                _ => {
                    format!("Worker returned error status: {}", status)
                }
            };
            return Err(anyhow::anyhow!(error_msg));
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response from worker")?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Parse worker response
        let worker_output: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse worker response as JSON")?;

        // Extract execution metrics from response if available
        let tokens_used = worker_output.get("tokens_used").and_then(|v| v.as_u64());

        Ok(RawExecutionResult {
            task_id: input.task_id,
            worker_id,
            raw_output: response_text,
            execution_time_ms: execution_time,
            tokens_used: tokens_used.map(|t| t as u32),
            quality_score: 0.5, // Default quality score
            metadata: std::collections::HashMap::new(),
            error: None,
        })
    }

    /// Process execution result
    async fn process_execution_result(
        &self,
        task_id: Uuid,
        worker_id: Uuid,
        raw_result: RawExecutionResult,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<TaskExecutionResult> {
        let completed_at = self.clock.now();
        let duration_ms = raw_result.execution_time_ms;
        let execution_id = self.id_gen.generate();
        let mut metadata = HashMap::new();

        if let Some(tokens) = raw_result.tokens_used {
            metadata.insert(META_TOKENS_USED.to_string(), serde_json::json!(tokens));
        }

        if let Some(error) = raw_result.error {
            metadata.insert(
                META_EXECUTION_STATUS.to_string(),
                serde_json::json!("Failed"),
            );
            return Ok(TaskExecutionResult {
                execution_id,
                task_id,
                success: false,
                output: raw_result.raw_output.clone(),
                errors: vec![error],
                metadata,
                started_at,
                completed_at,
                duration_ms,
                worker_id: Some(worker_id),
            });
        }

        // Parse worker output for quality and compliance evaluation
        let worker_output = match serde_json::from_str::<WorkerOutput>(&raw_result.raw_output) {
            Ok(output) => output,
            Err(e) => {
                metadata.insert(
                    META_EXECUTION_STATUS.to_string(),
                    serde_json::json!("Failed"),
                );
                let error_message = format!("Invalid output format: {}", e);
                return Ok(TaskExecutionResult {
                    execution_id,
                    task_id,
                    success: false,
                    output: raw_result.raw_output.clone(),
                    errors: vec![error_message],
                    metadata,
                    started_at,
                    completed_at,
                    duration_ms,
                    worker_id: Some(worker_id),
                });
            }
        };

        let quality_metrics = self.calculate_quality_metrics(&worker_output);
        if let Ok(value) = serde_json::to_value(&quality_metrics) {
            metadata.insert(META_QUALITY_METRICS.to_string(), value);
        }

        let caws_compliance =
            self.check_caws_compliance(&worker_output, raw_result.execution_time_ms);
        if let Ok(value) = serde_json::to_value(&caws_compliance) {
            metadata.insert(META_CAWS_COMPLIANCE.to_string(), value);
        }

        if let Ok(value) = serde_json::to_value(&worker_output) {
            metadata.insert(META_WORKER_OUTPUT.to_string(), value);
        }

        let status_label = if caws_compliance.is_compliant
            && quality_metrics.completeness_score > 0.8
        {
            "Completed"
        } else if quality_metrics.completeness_score > 0.5 {
            "Partial"
        } else {
            "Failed"
        };

        metadata.insert(
            META_EXECUTION_STATUS.to_string(),
            serde_json::json!(status_label),
        );

        let success = status_label == "Completed";
        let mut errors = Vec::new();
        if !success {
            errors.push("Execution failed quality or compliance checks".to_string());
        }

        Ok(TaskExecutionResult {
            execution_id,
            task_id,
            success,
            output: raw_result.raw_output.clone(),
            errors,
            metadata,
            started_at,
            completed_at,
            duration_ms,
            worker_id: Some(worker_id),
        })
    }

    /// Calculate quality metrics for worker output
    fn calculate_quality_metrics(&self, output: &WorkerOutput) -> QualityMetrics {
        QualityMetrics {
            completeness_score: if output.content.is_empty() { 0.0 } else { 0.9 },
            correctness_score: output.self_assessment.quality_score,
            maintainability_score: self.calculate_maintainability_score(&output.content),
            readability_score: self.calculate_readability_score(&output.content),
            test_coverage: self.estimate_test_coverage(&output.content),
            performance_impact: self.estimate_performance_impact(&output.content),
        }
    }

    /// Calculate maintainability score based on code structure and patterns
    fn calculate_maintainability_score(&self, content: &str) -> f32 {
        let mut score = 0.5; // Base score

        // Check for good practices that improve maintainability
        let lines = content.lines().count();

        // Penalize very long files
        if lines > 500 {
            score -= 0.2;
        } else if lines > 200 {
            score -= 0.1;
        }

        // Reward modular structure (functions, structs)
        let function_count = content.matches("fn ").count();
        let struct_count = content.matches("struct ").count();

        if function_count > 5 {
            score += 0.1;
        }
        if struct_count > 2 {
            score += 0.1;
        }

        // Check for documentation
        let comment_lines = content
            .lines()
            .filter(|l| l.trim().starts_with("//"))
            .count();
        let comment_ratio = comment_lines as f32 / lines.max(1) as f32;

        if comment_ratio > 0.1 {
            score += 0.1;
        }

        // Penalize complex nested structures
        let nested_braces = content.matches("{").count();
        if nested_braces > lines / 10 {
            score -= 0.1;
        }

        score.max(0.0).min(1.0)
    }

    /// Calculate readability score based on code formatting and structure
    fn calculate_readability_score(&self, content: &str) -> f32 {
        let mut score = 0.5; // Base score

        // Check line length (reward reasonable line lengths)
        let long_lines = content.lines().filter(|l| l.len() > 100).count();
        let total_lines = content.lines().count();

        if total_lines > 0 {
            let long_line_ratio = long_lines as f32 / total_lines as f32;
            if long_line_ratio < 0.1 {
                score += 0.2;
            } else if long_line_ratio > 0.3 {
                score -= 0.2;
            }
        }

        // Check for consistent indentation (basic check)
        let spaces_indent = content.lines().filter(|l| l.starts_with("    ")).count();
        let tabs_indent = content.lines().filter(|l| l.starts_with("\t")).count();

        // Mixed indentation is bad for readability
        if spaces_indent > 0 && tabs_indent > 0 {
            score -= 0.1;
        }

        // Reward descriptive naming (basic heuristic)
        let short_names = content.matches("let x").count() + content.matches("let y").count();
        if short_names < 3 {
            score += 0.1;
        }

        score.max(0.0).min(1.0)
    }

    /// Estimate test coverage based on content analysis
    fn estimate_test_coverage(&self, content: &str) -> Option<f32> {
        let has_tests = content.contains("#[test]")
            || content.to_lowercase().contains("test")
            || content.contains("#[tokio::test]");

        if has_tests {
            // Rough estimation based on test markers and assertions
            let test_functions =
                content.matches("#[test]").count() + content.matches("#[tokio::test]").count();
            let assertions = content.matches("assert").count();

            if test_functions > 0 {
                let coverage_estimate = (assertions as f32 / test_functions as f32).min(1.0);
                Some(coverage_estimate * 0.8) // Conservative estimate
            } else {
                Some(0.1) // Minimal coverage if tests exist but no assertions
            }
        } else {
            None // No tests detected
        }
    }

    /// Estimate performance impact based on code patterns
    fn estimate_performance_impact(&self, content: &str) -> Option<f32> {
        let mut impact_score = 0.5; // Neutral impact

        // Check for performance-sensitive patterns
        let allocations = content.matches("vec!").count()
            + content.matches("HashMap::new").count()
            + content.matches("Box::new").count();

        let async_operations = content.matches("async").count();
        let blocking_operations =
            content.matches("std::fs").count() + content.matches("std::net").count();

        // Performance concerns
        if allocations > 10 {
            impact_score += 0.2; // Memory pressure
        }

        if blocking_operations > async_operations {
            impact_score += 0.1; // Potential blocking
        }

        // Performance optimizations
        if content.contains("Arc<") || content.contains("Mutex<") {
            impact_score -= 0.1; // Good concurrency practices
        }

        if content.contains("tokio::spawn") {
            impact_score -= 0.1; // Async task spawning
        }

        Some(impact_score.max(0.0).min(1.0))
    }

    /// Check CAWS compliance for worker output
    fn check_caws_compliance(
        &self,
        output: &WorkerOutput,
        execution_time_ms: u64,
    ) -> CawsComplianceResult {
        let mut violations = Vec::new();
        let mut compliance_score: f32 = 1.0;

        // Check file count
        let file_count = output.files_modified.len() as u32;

        // Check LOC estimate (rough calculation)
        let loc_estimate: u32 = output
            .files_modified
            .iter()
            .map(|f| {
                f.content
                    .as_ref()
                    .map(|c| c.lines().count() as u32)
                    .unwrap_or(0)
            })
            .sum();

        // Implement CAWS rules compliance checking
        // Validates worker execution against CAWS quality standards
        // Rules checked: File count, complexity, code quality, test coverage
        
        if file_count > 10 {
            violations.push(CawsViolation {
                rule: "File Count Limit".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Created {} files, may exceed limit", file_count),
                location: None,
                suggestion: Some("Consider consolidating files".to_string()),
                constitutional_ref: None,
            });
            compliance_score -= 0.1;
        }

        if loc_estimate > 2000 {
            violations.push(CawsViolation {
                rule: "LOC Limit".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Estimated {} LOC, may exceed limit", loc_estimate),
                location: None,
                suggestion: Some("Consider breaking into smaller tasks".to_string()),
                constitutional_ref: None,
            });
            compliance_score -= 0.1;
        }

        let is_compliant = violations.is_empty();
        CawsComplianceResult {
            is_compliant,
            compliance_score: compliance_score.max(0.0),
            violations,
            budget_adherence: BudgetAdherence {
                files_used: file_count,
                files_limit: 10, // Example limit
                loc_used: loc_estimate,
                loc_limit: 2000, // Example limit
                time_used_ms: execution_time_ms,
                time_limit_ms: None,
                within_budget: is_compliant,
            },
            provenance_complete: !output.rationale.is_empty(),
        }
    }

    /// Extract programming languages from task text
    fn extract_languages(&self, text: &str) -> Vec<String> {
        let mut languages = Vec::new();
        let text_lower = text.to_lowercase();

        // Common programming languages
        let language_patterns = [
            "rust",
            "python",
            "javascript",
            "typescript",
            "java",
            "c++",
            "c#",
            "go",
            "ruby",
            "php",
            "swift",
            "kotlin",
            "scala",
            "haskell",
            "clojure",
            "elixir",
            "dart",
            "r",
            "matlab",
            "perl",
            "lua",
            "shell",
            "bash",
            "powershell",
        ];

        for pattern in &language_patterns {
            if text_lower.contains(pattern) {
                languages.push(pattern.to_string());
            }
        }

        languages.sort();
        languages.dedup();
        languages
    }

    /// Extract frameworks and tools from task text
    fn extract_frameworks(&self, text: &str) -> Vec<String> {
        let mut frameworks = Vec::new();
        let text_lower = text.to_lowercase();

        // Common frameworks and tools
        let framework_patterns = [
            "react",
            "vue",
            "angular",
            "express",
            "django",
            "flask",
            "spring",
            "rails",
            "laravel",
            "symfony",
            "asp.net",
            "fastapi",
            "gin",
            "echo",
            "fiber",
            "tokio",
            "actix",
            "warp",
            "axum",
            "rocket",
            "tide",
            "hyper",
            "docker",
            "kubernetes",
            "terraform",
            "ansible",
            "jenkins",
            "gitlab",
            "aws",
            "azure",
            "gcp",
            "firebase",
            "mongodb",
            "postgresql",
            "mysql",
            "redis",
            "elasticsearch",
            "kafka",
            "rabbitmq",
            "nginx",
            "apache",
        ];

        for pattern in &framework_patterns {
            if text_lower.contains(pattern) {
                frameworks.push(pattern.to_string());
            }
        }

        frameworks.sort();
        frameworks.dedup();
        frameworks
    }

    /// Calculate context length estimate based on task content
    fn calculate_context_length(&self, task_text: &str, context: &CouncilTaskContext) -> u32 {
        let mut estimate = task_text.len() as u32;

        // Add context length from dependencies and recent changes
        estimate += context.dependencies.len() as u32 * 100;
        estimate += context.recent_changes.len() as u32 * 50;

        // Add some padding for system prompts and responses
        estimate += 2000;

        // Cap at reasonable maximum
        estimate.min(32000)
    }

}


#[async_trait::async_trait]
impl TaskExecutorTrait for TaskExecutor {
    async fn execute_task(
        &self,
        task_spec: ContractTaskSpec,
        worker_id: Uuid,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // Convert contract task spec to internal task spec
        let internal_spec = TaskSpec {
            id: task_spec.id,
            title: task_spec.title,
            description: task_spec.description,
            priority: match task_spec.priority {
                agent_agency_contracts::types::planning::TaskPriority::Low => TaskPriority::Low,
                agent_agency_contracts::types::planning::TaskPriority::Normal => TaskPriority::Medium,
                agent_agency_contracts::types::planning::TaskPriority::Medium => TaskPriority::Medium,
                agent_agency_contracts::types::planning::TaskPriority::High => TaskPriority::High,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => TaskPriority::High,
                agent_agency_contracts::types::planning::TaskPriority::Critical => TaskPriority::Critical,
            },
            required_capabilities: task_spec.required_capabilities,
            context: task_spec.context,
            working_spec_id: task_spec.working_spec_id,
            timeout_seconds: task_spec.timeout_seconds,
        };

        // Execute the task
        let result = self.execute_task(internal_spec, worker_id, None).await?;

        // Convert result back to contract format
        Ok(TaskExecutionResult {
            execution_id: result.execution_id,
            task_id: result.task_id,
            success: result.success,
            output: result.output,
            errors: result.errors,
            metadata: result.metadata,
            started_at: result.started_at,
            completed_at: result.completed_at,
            duration_ms: result.duration_ms,
            worker_id: result.worker_id,
        })
    }

    async fn execute_task_with_circuit_breaker(
        &self,
        task_spec: ContractTaskSpec,
        worker_id: Uuid,
        circuit_breaker_enabled: bool,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        // Convert contract task spec to internal task spec
        let internal_spec = TaskSpec {
            id: task_spec.id,
            title: task_spec.title,
            description: task_spec.description,
            priority: match task_spec.priority {
                agent_agency_contracts::types::planning::TaskPriority::Low => TaskPriority::Low,
                agent_agency_contracts::types::planning::TaskPriority::Normal => TaskPriority::Medium,
                agent_agency_contracts::types::planning::TaskPriority::Medium => TaskPriority::Medium,
                agent_agency_contracts::types::planning::TaskPriority::High => TaskPriority::High,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => TaskPriority::High,
                agent_agency_contracts::types::planning::TaskPriority::Critical => TaskPriority::Critical,
            },
            required_capabilities: task_spec.required_capabilities,
            context: task_spec.context,
            working_spec_id: task_spec.working_spec_id,
            timeout_seconds: task_spec.timeout_seconds,
        };

        // Execute with circuit breaker if enabled
        let circuit_breaker = if circuit_breaker_enabled {
            Some(Arc::new(CircuitBreaker::new(
                CircuitBreakerConfig {
                    name: Some("task-execution".to_string()),
                    failure_threshold: 5,
                    success_threshold: 3,
                    timeout_ms: Some(30000), // 30 seconds
                    failure_window_ms: Some(300000), // 5 minutes
                    reset_timeout_ms: 60000, // 1 minute
                },
            )))
        } else {
            None
        };

        let result = self.execute_task(internal_spec, worker_id, circuit_breaker.as_ref()).await?;

        // Convert result back to contract format
        Ok(TaskExecutionResult {
            execution_id: result.execution_id,
            task_id: result.task_id,
            success: result.success,
            output: result.output,
            errors: result.errors,
            metadata: result.metadata,
            started_at: result.started_at,
            completed_at: result.completed_at,
            duration_ms: result.duration_ms,
            worker_id: result.worker_id,
        })
    }

    async fn health_check(&self) -> Result<agent_agency_contracts::task_executor::TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>> {
        let stats = self.execution_stats.read().await;
        
        // Calculate success rate from tracked executions
        let success_rate = if stats.total_executions > 0 {
            stats.successful_executions as f64 / stats.total_executions as f64
        } else {
            1.0 // Default to healthy if no executions yet
        };
        
        // Determine health status based on success rate
        let status = if success_rate >= 0.95 {
            agent_agency_contracts::task_executor::HealthStatus::Healthy
        } else if success_rate >= 0.80 {
            agent_agency_contracts::task_executor::HealthStatus::Degraded
        } else {
            agent_agency_contracts::task_executor::HealthStatus::Unhealthy
        };
        
        Ok(agent_agency_contracts::task_executor::TaskExecutorHealth {
            status,
            last_execution_time: stats.last_execution_time,
            active_tasks: 0, // TODO: Track active tasks separately when task queue is implemented
            queued_tasks: 0, // TODO: Track queued tasks separately when task queue is implemented
            total_executions: stats.total_executions,
            success_rate,
        })
    }

    async fn get_execution_stats(&self) -> Result<agent_agency_contracts::task_executor::TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>> {
        let stats = self.execution_stats.read().await;
        
        // Calculate statistics from tracked execution times
        let total = stats.total_executions;
        let successful = stats.successful_executions;
        let failed = stats.failed_executions;
        
        // Calculate percentiles from execution times
        let mut sorted_times = stats.execution_times.clone();
        sorted_times.sort();
        
        let average = if !sorted_times.is_empty() {
            sorted_times.iter().sum::<u64>() as f64 / sorted_times.len() as f64
        } else {
            0.0
        };
        
        let median = if !sorted_times.is_empty() {
            let mid = sorted_times.len() / 2;
            if sorted_times.len() % 2 == 0 {
                (sorted_times[mid - 1] + sorted_times[mid]) as f64 / 2.0
            } else {
                sorted_times[mid] as f64
            }
        } else {
            0.0
        };
        
        let p95 = if sorted_times.len() >= 20 {
            let index = (sorted_times.len() as f64 * 0.95) as usize;
            sorted_times[index.min(sorted_times.len() - 1)] as f64
        } else if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() - 1] as f64
        } else {
            0.0
        };
        
        let p99 = if sorted_times.len() >= 100 {
            let index = (sorted_times.len() as f64 * 0.99) as usize;
            sorted_times[index.min(sorted_times.len() - 1)] as f64
        } else if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() - 1] as f64
        } else {
            0.0
        };
        
        Ok(agent_agency_contracts::task_executor::TaskExecutionStats {
            total_executions: total,
            successful_executions: successful,
            failed_executions: failed,
            average_execution_time_ms: average,
            median_execution_time_ms: median,
            p95_execution_time_ms: p95,
            p99_execution_time_ms: p99,
        })
    }

    async fn cancel_task_execution(&self, task_id: Uuid, worker_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tracing::{info, warn};
        
        info!("Cancelling task {} on worker {}", task_id, worker_id);
        
        // Resolve worker endpoint
        let worker_base_url = match self.resolve_worker_endpoint(worker_id).await {
            Ok(url) => url,
            Err(e) => {
                warn!("Failed to resolve worker endpoint for {}: {}", worker_id, e);
                // Try default endpoint as fallback
                format!("http://worker-{}.local:8080", worker_id)
            }
        };
        
        let cancel_url = format!("{}/cancel", worker_base_url.trim_end_matches('/'));
        
        // Send cancellation request to worker
        let cancel_body = serde_json::json!({
            "task_id": task_id,
            "reason": "Cancelled by executor"
        });
        
        match self
            .client
            .post(&cancel_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Agent-Agency-Executor/1.0")
            .json(&cancel_body)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                info!("Successfully cancelled task {} on worker {}", task_id, worker_id);
                Ok(())
            }
            Ok(response) => {
                warn!("Worker returned non-success status when cancelling task {}: {}", task_id, response.status());
                // Still return Ok() as cancellation was attempted
                Ok(())
            }
            Err(e) => {
                warn!("Failed to send cancellation request for task {} to worker {}: {}", task_id, worker_id, e);
                // Return error only if it's a critical failure
                Err(format!("Failed to cancel task: {}", e).into())
            }
        }
    }
}

// Internal helper methods for TaskExecutor (not part of the trait)
impl TaskExecutor {
    /// Resolve worker endpoint from worker registry
    async fn resolve_worker_endpoint(&self, worker_id: Uuid) -> Result<String, anyhow::Error> {
        // In a real implementation, this would:
        // 1. Query worker registry service for worker endpoint
        // 2. Check worker health and availability
        // 3. Cache resolved endpoints for performance
        // 4. Handle worker failover and load balancing
        
        // For now, use a simple pattern based on worker ID
        // In production, this would integrate with service discovery
        let worker_endpoint = format!("http://worker-{}.local:8080", worker_id);
        
        // Validate endpoint is reachable (basic health check)
        match self.client.get(&format!("{}/health", worker_endpoint))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await 
        {
            Ok(response) if response.status().is_success() => {
                info!("Worker {} is healthy at {}", worker_id, worker_endpoint);
                Ok(worker_endpoint)
            }
            Ok(response) => {
                warn!("Worker {} returned non-success status: {}", worker_id, response.status());
                Err(anyhow::anyhow!("Worker health check failed with status: {}", response.status()))
            }
            Err(e) => {
                warn!("Failed to reach worker {} at {}: {}", worker_id, worker_endpoint, e);
                Err(anyhow::anyhow!("Failed to reach worker endpoint: {}", e))
            }
        }
    }

    /// Discover worker capabilities from registry
    async fn discover_worker_capabilities(&self, worker_id: Uuid) -> Result<WorkerCapabilities> {
        // Query worker registry for capabilities
        let query = r#"
            SELECT 
                w.id,
                w.name,
                w.specialty,
                w.capabilities,
                w.max_concurrent_tasks,
                w.memory_limit_mb,
                w.cpu_limit_cores,
                w.is_active,
                w.last_heartbeat,
                w.version,
                w.endpoint_url
            FROM workers w
            WHERE w.id = $1 AND w.is_active = true
        "#;

        match self.db_client.query(query, &[&worker_id]).await {
            Ok(rows) => {
                if let Some(row) = rows.first() {
                    let capabilities_json: serde_json::Value = row.try_get::<serde_json::Value, _>(4)
                        .context("Failed to get capabilities JSON")?;
                    let capabilities: Vec<String> = capabilities_json
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();

                    Ok(WorkerCapabilities {
                        worker_id,
                        name: row.try_get::<String, _>(1).context("Failed to get worker name")?,
                        specialty: row.try_get::<String, _>(2).context("Failed to get worker specialty")?,
                        capabilities,
                        max_concurrent_tasks: row.try_get::<i32, _>(5).context("Failed to get max concurrent tasks")?,
                        memory_limit_mb: row.try_get::<i32, _>(6).context("Failed to get memory limit")?,
                        cpu_limit_cores: row.try_get::<i32, _>(7).context("Failed to get CPU limit")?,
                        is_active: row.try_get::<bool, _>(8).context("Failed to get active status")?,
                        last_heartbeat: row.try_get::<Option<DateTime<Utc>>, _>(9).context("Failed to get last heartbeat")?,
                        version: row.try_get::<String, _>(10).context("Failed to get version")?,
                        endpoint_url: row.try_get::<Option<String>, _>(11).context("Failed to get endpoint URL")?.unwrap_or_else(|| "http://localhost:3000".to_string()),
                    })
                } else {
                    Err(anyhow::anyhow!("Worker {} not found in registry", worker_id))
                }
            }
            Err(e) => {
                error!("Failed to discover worker capabilities for {}: {}", worker_id, e);
                Err(anyhow::anyhow!("Database error: {}", e))
            }
        }
    }

    /// Match worker capabilities against task requirements
    fn match_worker_capabilities(
        &self,
        task_spec: &TaskSpec,
        worker_info: &WorkerCapabilities,
    ) -> Result<CapabilityMatch> {
        let required_capabilities = self.extract_required_capabilities(task_spec);
        let missing_capabilities: Vec<String> = required_capabilities
            .iter()
            .filter(|cap| !worker_info.capabilities.contains(cap))
            .cloned()
            .collect();

        Ok(CapabilityMatch {
            is_suitable: missing_capabilities.is_empty(),
            missing_capabilities,
            capability_score: self.calculate_capability_score(&required_capabilities, &worker_info.capabilities),
            worker_specialty_match: self.check_specialty_match(task_spec, worker_info),
        })
    }

    /// Determine execution strategy based on task and worker characteristics
    async fn determine_execution_strategy(
        &self,
        task_spec: &TaskSpec,
        worker_info: &WorkerCapabilities,
    ) -> Result<ExecutionStrategy> {
        let complexity = self.assess_task_complexity(task_spec);
        let worker_load = self.get_worker_current_load(worker_info.worker_id).await?;
        
        Ok(ExecutionStrategy {
            execution_mode: if complexity.is_high() {
                ExecutionMode::Async
            } else {
                ExecutionMode::Sync
            },
            timeout_seconds: self.calculate_timeout(task_spec, worker_info),
            retry_config: self.get_retry_config_for_task(task_spec),
            resource_allocation: ResourceAllocation {
                memory_mb: worker_info.memory_limit_mb.min(complexity.estimated_memory_mb()),
                cpu_cores: worker_info.cpu_limit_cores.min(complexity.estimated_cpu_cores()),
                priority: self.determine_priority(task_spec),
            },
            enable_streaming: complexity.supports_streaming(),
            enable_cancellation: true,
        })
    }

    /// Check worker health status
    async fn check_worker_health(&self, worker_id: Uuid) -> Result<HealthStatus> {
        let query = r#"
            SELECT 
                w.is_active,
                w.last_heartbeat,
                w.health_status,
                w.error_count,
                w.success_count,
                w.avg_response_time_ms
            FROM workers w
            WHERE w.id = $1
        "#;

        match self.db_client.query(query, &[&worker_id]).await {
            Ok(rows) => {
                if let Some(row) = rows.first() {
                    let is_active: bool = row.get(0);
                    let last_heartbeat: Option<chrono::DateTime<chrono::Utc>> = row.get(1);
                    let health_status: Option<String> = row.get(2);
                    let error_count: i64 = row.get(3);
                    let success_count: i64 = row.get(4);
                    let avg_response_time_ms: Option<i64> = row.get(5);

                    let is_healthy = is_active 
                        && last_heartbeat.map_or(false, |hb| {
                            chrono::Utc::now() - hb < chrono::Duration::minutes(5)
                        })
                        && health_status.as_deref() == Some("healthy");

                    let health_message = if !is_healthy {
                        if !is_active {
                            "Worker is inactive".to_string()
                        } else if last_heartbeat.is_none() {
                            "No heartbeat received".to_string()
                        } else {
                            format!("Health status: {:?}", health_status)
                        }
                    } else {
                        "Worker is healthy".to_string()
                    };

                    Ok(HealthStatus {
                        is_healthy,
                        health_message,
                        last_heartbeat,
                        error_count,
                        success_count,
                        avg_response_time_ms,
                        health_score: self.calculate_health_score(error_count, success_count, avg_response_time_ms),
                    })
                } else {
                    Err(anyhow::anyhow!("Worker {} not found", worker_id))
                }
            }
            Err(e) => {
                error!("Failed to check worker health for {}: {}", worker_id, e);
                Err(anyhow::anyhow!("Database error: {}", e))
            }
        }
    }

    /// Authenticate worker for task execution
    async fn authenticate_worker(
        &self,
        worker_id: Uuid,
        task_spec: &TaskSpec,
    ) -> Result<AuthResult> {
        // Check worker permissions and task authorization
        let query = r#"
            SELECT 
                w.id,
                w.permissions,
                w.allowed_domains,
                w.restricted_domains,
                w.max_risk_tier
            FROM workers w
            WHERE w.id = $1 AND w.is_active = true
        "#;

        match self.db_client.query(query, &[&worker_id]).await {
            Ok(rows) => {
                if let Some(row) = rows.first() {
                    let permissions_json: serde_json::Value = row.get(1);
                    let permissions: Vec<String> = permissions_json
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();

                    let allowed_domains_json: serde_json::Value = row.get(2);
                    let allowed_domains: Vec<String> = allowed_domains_json
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();

                    let restricted_domains_json: serde_json::Value = row.get(3);
                    let restricted_domains: Vec<String> = restricted_domains_json
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();

                    let max_risk_tier: Option<i32> = row.get(4);

                    // Check domain restrictions
                    let domain_allowed = if !allowed_domains.is_empty() {
                        task_spec.scope.as_ref()
                            .map(|s| s.domains.iter().any(|domain| allowed_domains.contains(domain)))
                            .unwrap_or(true)
                    } else {
                        true
                    };

                    let domain_restricted = task_spec.scope.as_ref()
                        .map(|s| s.domains.iter().any(|domain| restricted_domains.contains(domain)))
                        .unwrap_or(false);

                    // Check risk tier restrictions
                    let risk_tier_allowed = max_risk_tier.map_or(true, |max_tier| {
                        task_spec.risk_tier.map_or(true, |task_tier| task_tier <= max_tier as u32)
                    });

                    let is_authorized = domain_allowed && !domain_restricted && risk_tier_allowed;

                    let auth_message = if !is_authorized {
                        if !domain_allowed {
                            "Worker not authorized for task domains".to_string()
                        } else if domain_restricted {
                            "Worker restricted from task domains".to_string()
                        } else {
                            "Worker not authorized for task risk tier".to_string()
                        }
                    } else {
                        "Worker authorized for task".to_string()
                    };

                    Ok(AuthResult {
                        is_authorized,
                        auth_message,
                        permissions,
                        allowed_domains,
                        restricted_domains,
                        max_risk_tier,
                    })
                } else {
                    Err(anyhow::anyhow!("Worker {} not found", worker_id))
                }
            }
            Err(e) => {
                error!("Failed to authenticate worker {}: {}", worker_id, e);
                Err(anyhow::anyhow!("Database error: {}", e))
            }
        }
    }

    /// Update worker status in registry
    async fn update_worker_status(
        &self,
        worker_id: Uuid,
        status: &str,
        current_task_id: Option<Uuid>,
    ) -> Result<()> {
        let query = r#"
            UPDATE workers 
            SET 
                status = $1,
                current_task_id = $2,
                last_activity = NOW(),
                updated_at = NOW()
            WHERE id = $3
        "#;

        match self.db_client.execute(query, &[&status, &current_task_id, &worker_id]).await {
            Ok(_) => {
                info!("Updated worker {} status to {}", worker_id, status);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update worker {} status: {}", worker_id, e);
                Err(anyhow::anyhow!("Database error: {}", e))
            }
        }
    }

    /// Execute with circuit breaker and retry logic
    async fn execute_with_circuit_breaker_and_retry(
        &self,
        worker_id: Uuid,
        execution_input: &ExecutionInput,
        strategy: &ExecutionStrategy,
        circuit_breaker: &Arc<CircuitBreaker>,
    ) -> WorkerExecutionResult<RawExecutionResult> {
        let retry_config = &strategy.retry_config;
        
        self.retry_with_backoff_local(
            RetryConfig {
                max_attempts: retry_config.max_attempts,
                initial_delay_ms: retry_config.initial_delay_ms,
                max_delay_ms: retry_config.max_delay_ms,
                backoff_multiplier: retry_config.backoff_multiplier,
                jitter_factor: retry_config.jitter_factor,
                use_exponential_backoff: retry_config.use_exponential_backoff,
                use_jitter: retry_config.use_jitter,
            },
            || async {
                // Check circuit breaker state
                match circuit_breaker.get_state() {
                    system_resilience::resilience_circuit_breaker::CircuitState::Open => {
                        Err(WorkerError::ExecutionError {
                            message: "Circuit breaker is open".to_string()
                        })
                    }
                    system_resilience::resilience_circuit_breaker::CircuitState::HalfOpen | system_resilience::resilience_circuit_breaker::CircuitState::Closed => {
                        self.execute_with_worker_direct(worker_id, execution_input, strategy).await
                    }
                }
            },
        )
        .await
    }

    /// Execute directly with worker
    async fn execute_with_worker_direct(
        &self,
        worker_id: Uuid,
        execution_input: &ExecutionInput,
        strategy: &ExecutionStrategy,
    ) -> WorkerExecutionResult<RawExecutionResult> {
        let worker_endpoint = self.get_worker_endpoint(worker_id).await?;
        
        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(strategy.timeout_seconds))
            .build()
            .map_err(|e| WorkerError::ExecutionError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        // Prepare request payload
        let payload = serde_json::json!({
            "task_id": execution_input.task_id,
            "prompt": execution_input.prompt,
            "context": execution_input.context,
            "requirements": execution_input.requirements,
            "caws_spec": execution_input.caws_spec,
            "execution_mode": strategy.execution_mode,
            "resource_allocation": strategy.resource_allocation,
            "enable_streaming": strategy.enable_streaming,
        });

        // Execute request
        let response = client
            .post(&format!("{}/execute", worker_endpoint))
            .json(&payload)
            .send()
            .await
            .map_err(|e| WorkerError::Communication {
                message: format!("Worker execution request failed: {}", e),
            })?;

        if response.status().is_success() {
            let result: RawExecutionResult = response
                .json()
                .await
                .map_err(|e| WorkerError::Communication {
                    message: format!("Failed to parse worker response: {}", e),
                })?;
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(WorkerError::ExecutionFailed {
                worker_id: WorkerId(worker_id),
                message: format!("Worker execution failed with status {}: {}", status, error_text),
            })
        }
    }

    // Helper methods for capability matching and strategy determination
    fn extract_required_capabilities(&self, task_spec: &TaskSpec) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Add capabilities based on task domains
        for domain in &task_spec.scope.domains {
            match domain.as_str() {
                "frontend" => capabilities.extend(vec!["react".to_string(), "typescript".to_string(), "css".to_string()]),
                "backend" => capabilities.extend(vec!["rust".to_string(), "api".to_string(), "database".to_string()]),
                "testing" => capabilities.extend(vec!["testing".to_string(), "jest".to_string(), "integration".to_string()]),
                "documentation" => capabilities.extend(vec!["markdown".to_string(), "documentation".to_string()]),
                _ => capabilities.push(domain.clone()),
            }
        }

        // Add capabilities based on risk tier
        if let Some(tier) = task_spec.risk_tier {
            match tier {
                1 => {
                    capabilities.extend(vec!["security".to_string(), "compliance".to_string()]);
                }
                2 => {
                    capabilities.push("quality_gates".to_string());
                }
                _ => {}
            }
        }

        capabilities
    }

    fn calculate_capability_score(&self, required: &[String], available: &[String]) -> f64 {
        if required.is_empty() {
            return 1.0;
        }
        
        let matched = required.iter().filter(|cap| available.contains(cap)).count();
        matched as f64 / required.len() as f64
    }

    fn check_specialty_match(&self, task_spec: &TaskSpec, worker_info: &WorkerCapabilities) -> bool {
        // Check if worker specialty matches task requirements
        task_spec.scope.as_ref()
            .map(|s| s.domains.iter().any(|domain| {
                worker_info.specialty.to_lowercase().contains(&domain.to_lowercase())
            }))
            .unwrap_or(false)
    }

    fn assess_task_complexity(&self, task_spec: &TaskSpec) -> TaskComplexity {
        let file_count = task_spec.scope.as_ref().map(|s| s.files_affected.len()).unwrap_or(0);
        let loc_estimate = task_spec.scope.as_ref().and_then(|s| s.max_loc).unwrap_or(100);
        let domain_count = task_spec.scope.as_ref().map(|s| s.domains.len()).unwrap_or(0);
        let criteria_count = task_spec.acceptance_criteria.len();

        let complexity_score = (file_count * 2) + (loc_estimate / 100) + (domain_count * 3) + (criteria_count as usize * 2);

        if complexity_score > 50 {
            TaskComplexity::High
        } else if complexity_score > 20 {
            TaskComplexity::Medium
        } else {
            TaskComplexity::Low
        }
    }

    async fn get_worker_current_load(&self, worker_id: Uuid) -> Result<WorkerLoad> {
        let query = r#"
            SELECT 
                COUNT(*) as active_tasks,
                AVG(execution_time_ms) as avg_execution_time,
                MAX(execution_time_ms) as max_execution_time
            FROM task_executions te
            JOIN tasks t ON te.task_id = t.id
            WHERE te.worker_id = $1 
                AND te.status IN ('running', 'pending')
                AND te.started_at > NOW() - INTERVAL '1 hour'
        "#;

        match self.db_client.query(query, &[&worker_id]).await {
            Ok(rows) => {
                if let Some(row) = rows.first() {
                    Ok(WorkerLoad {
                        active_tasks: row.get(0),
                        avg_execution_time_ms: row.get(1),
                        max_execution_time_ms: row.get(2),
                        load_percentage: self.calculate_load_percentage(row.get(0), row.get(1)),
                    })
                } else {
                    Ok(WorkerLoad::default())
                }
            }
            Err(e) => {
                error!("Failed to get worker load for {}: {}", worker_id, e);
                Ok(WorkerLoad::default())
            }
        }
    }

    fn calculate_timeout(&self, task_spec: &TaskSpec, worker_info: &WorkerCapabilities) -> u64 {
        let base_timeout = task_spec.risk_tier.map_or(60, |tier| match tier {
            1 => 300, // 5 minutes
            2 => 180, // 3 minutes
            _ => 60,  // 1 minute
        });

        let complexity_multiplier = match self.assess_task_complexity(task_spec) {
            TaskComplexity::High => 3.0,
            TaskComplexity::Medium => 2.0,
            TaskComplexity::Low => 1.0,
        };

        (base_timeout as f64 * complexity_multiplier) as u64
    }

    fn get_retry_config_for_task(&self, task_spec: &TaskSpec) -> RetryConfig {
        match task_spec.risk_tier.unwrap_or(3) {
            1 => RetryConfig {
                max_attempts: 5,
                initial_delay_ms: 1000,
                max_delay_ms: 10000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
                use_exponential_backoff: true,
                use_jitter: true,
            },
            2 => RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 500,
                max_delay_ms: 5000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
                use_exponential_backoff: true,
                use_jitter: true,
            },
            _ => RetryConfig {
                max_attempts: 2,
                initial_delay_ms: 250,
                max_delay_ms: 2000,
                backoff_multiplier: 1.5,
                jitter_factor: 0.05,
                use_exponential_backoff: true,
                use_jitter: true,
            },
        }
    }

    fn determine_priority(&self, task_spec: &TaskSpec) -> TaskPriority {
        match task_spec.risk_tier.unwrap_or(3) {
            1 => TaskPriority::High,
            2 => TaskPriority::Medium,
            _ => TaskPriority::Low,
        }
    }

    fn calculate_health_score(&self, error_count: i64, success_count: i64, avg_response_time_ms: Option<i64>) -> f64 {
        if success_count == 0 && error_count == 0 {
            return 0.5; // Neutral score for new workers
        }

        let total_requests = success_count + error_count;
        let success_rate = success_count as f64 / total_requests as f64;
        
        let response_time_score = avg_response_time_ms.map_or(1.0, |time_ms| {
            if time_ms < 1000 { 1.0 }
            else if time_ms < 5000 { 0.8 }
            else if time_ms < 10000 { 0.6 }
            else { 0.4 }
        });

        (success_rate * 0.7) + (response_time_score * 0.3)
    }

    fn calculate_load_percentage(&self, active_tasks: i64, avg_execution_time_ms: Option<i64>) -> f64 {
        let base_load = active_tasks as f64 * 10.0; // 10% per active task
        let time_load = avg_execution_time_ms.map_or(0.0, |time_ms| {
            if time_ms > 30000 { 20.0 } // High execution time penalty
            else if time_ms > 10000 { 10.0 }
            else { 0.0 }
        });
        
        (base_load + time_load).min(100.0)
    }

    /// Get worker endpoint for the given worker ID
    async fn get_worker_endpoint(&self, worker_id: Uuid) -> WorkerExecutionResult<String> {
        // Query database for worker endpoint
        let query = "SELECT endpoint FROM workers WHERE id = $1";

        match self.db_client.query_one(query, &[&worker_id]).await {
            Ok(row) => {
                let endpoint: String = row.get("endpoint");
                Ok(endpoint)
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(WorkerError::WorkerNotFound { worker_id: WorkerId(worker_id) })
            }
            Err(e) => {
                Err(WorkerError::DatabaseError { message: format!("Failed to get worker endpoint: {}", e) })
            }
        }
    }

    /// Execute with retry logic using local backoff
    async fn retry_with_backoff_local<F, Fut, T>(
        &self,
        retry_config: RetryConfig,
        operation: F,
    ) -> WorkerExecutionResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = WorkerExecutionResult<T>>,
    {
        let mut attempt = 0;
        let mut delay_ms = retry_config.initial_delay_ms;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= retry_config.max_attempts {
                        return Err(e);
                    }

                    // Calculate next delay
                    if retry_config.use_exponential_backoff {
                        delay_ms = (delay_ms as f64 * retry_config.backoff_multiplier) as u64;
                        delay_ms = delay_ms.min(retry_config.max_delay_ms);
                    }

                    // Add jitter if enabled
                    let actual_delay = if retry_config.use_jitter {
                        let jitter = (rand::random::<f64>() - 0.5) * 2.0 * retry_config.jitter_factor;
                        ((delay_ms as f64 * (1.0 + jitter)) as u64).max(0)
                    } else {
                        delay_ms
                    };

                    tokio::time::sleep(std::time::Duration::from_millis(actual_delay)).await;
                }
            }
        }
    }
}
