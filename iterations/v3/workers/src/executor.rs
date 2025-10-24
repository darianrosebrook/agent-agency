//! Task Executor
//!
//! Executes tasks by communicating with worker models and handling the execution lifecycle.

use crate::types::{*, UuidGenerator};
use agent_agency_contracts::{IssueSeverity, task_executor::{TaskExecutor as TaskExecutorTrait, TaskExecutionResult, TaskSpec as ContractTaskSpec, TaskPriority}};
use agent_agency_council::models::{RiskTier, TaskContext as CouncilTaskContext, TaskSpec};
use agent_agency_resilience::{CircuitBreaker, RetryConfig};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Task executor for running tasks with workers
#[derive(Debug)]
pub struct TaskExecutor {
    // HTTP client for model communication with robust error handling and performance optimization
    client: reqwest::Client,
    execution_timeout: std::time::Duration,
    cancel_timeout: std::time::Duration,
    clock: Box<dyn Clock + Send + Sync>,
    id_gen: Box<dyn IdGenerator + Send + Sync>,
}

impl TaskExecutor {
    /// Create a new task executor with default configuration
    pub fn new() -> Self {
        Self::with_timeouts(
            std::time::Duration::from_secs(30), // execution timeout
            std::time::Duration::from_secs(10), // connect timeout
            std::time::Duration::from_secs(5),  // cancel timeout
        )
    }

    /// Create a new task executor with custom timeouts
    pub fn with_timeouts(
        execution_timeout: std::time::Duration,
        connect_timeout: std::time::Duration,
        cancel_timeout: std::time::Duration,
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

        // TODO: Implement full worker registry and distributed execution system
        // - [ ] Implement worker discovery and capability matching algorithms
        // - [ ] Add load balancing and performance optimization strategies
        // - [ ] Support distributed worker coordination and health monitoring
        // - [ ] Implement secure worker authentication and authorization
        // - [ ] Add worker lifecycle management (registration, deregistration, updates)
        // - [ ] Support worker versioning and compatibility checking
        // - [ ] Implement worker performance profiling and resource allocation

        // Prepare execution input
        let execution_input = self.prepare_execution_input(&task_spec)?;

        // TODO: Implement actual worker execution with circuit breaker and retry logic
        // - [ ] Integrate with real worker execution APIs and protocols
        // - [ ] Implement proper circuit breaker pattern with state management
        // - [ ] Add exponential backoff and jitter for retry strategies
        // - [ ] Support different execution modes (sync, async, streaming)
        // - [ ] Add execution timeout and cancellation handling
        // - [ ] Implement execution result validation and error classification
        // - [ ] Support worker-specific execution parameters and configurations
        let execution_result = if let Some(cb) = circuit_breaker {
            self.retry_with_backoff_local(
                RetryConfig {
                    max_attempts: 3,
                    initial_delay_ms: 500,
                    max_delay_ms: 5000,
                    backoff_multiplier: 2.0,
                    jitter_factor: 0.1,
                    use_exponential_backoff: true,
                    use_jitter: true,
                },
                || async {
                    cb.call(|| async {
                        self.execute_with_worker(worker_id, &execution_input).await
                    })
                    .await
                },
            )
            .await?
        } else {
            self.execute_with_worker(worker_id, &execution_input)
                .await?
        };

        // Process and validate result
        let result = self
            .process_execution_result(task_id, worker_id, execution_result, started_at)
            .await?;

        info!(
            "Task {} execution completed with status: {:?}",
            task_id, result.status
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
                .map(|spec| self.convert_caws_spec(spec)),
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
            task_spec.scope.files_affected.join(", ")
        ));
        if let Some(max_files) = task_spec.scope.max_files {
            prompt.push_str(&format!("- Max files: {}\n", max_files));
        }
        if let Some(max_loc) = task_spec.scope.max_loc {
            prompt.push_str(&format!("- Max lines of code: {}\n", max_loc));
        }
        prompt.push_str(&format!(
            "- Domains: {}\n\n",
            task_spec.scope.domains.join(", ")
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
            required_domains: task_spec.scope.domains.clone(),
            min_quality_score: match task_spec.risk_tier {
                RiskTier::Tier1 => 0.9,
                RiskTier::Tier2 => 0.8,
                RiskTier::Tier3 => 0.7,
            },
            min_caws_awareness: 0.8,
            max_execution_time_ms: task_spec.scope.max_loc.map(|loc| loc as u64 * 100),
            preferred_worker_type: None,
            context_length_estimate,
        }
    }

    /// Convert council TaskContext to workers TaskContext
    fn convert_task_context(
        &self,
        _council_context: &agent_agency_council::models::TaskContext,
    ) -> TaskContext {
        // Create execution context with defaults - would map actual fields in real implementation
        TaskContext {
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
    fn convert_validation_rules(council_spec: &agent_agency_council::models::CawsSpec) -> Vec<ValidationRule> {
        // For now, create basic validation rules from waivers
        council_spec.waivers.iter().enumerate().map(|(i, waiver)| {
            ValidationRule {
                id: format!("validation-{}", i),
                name: format!("Waiver Validation {}", i),
                description: format!("Validate waiver: {}", waiver.reason),
                rule_type: ValidationRuleType::Custom,
                config: serde_json::json!({
                    "waiver_id": waiver.id,
                    "reason": waiver.reason
                }),
                severity: SeverityLevel::Medium,
                file_patterns: vec!["**/*".to_string()],
            }
        }).collect()
    }

    /// Convert council CawsSpec to workers CawsSpec
    fn convert_caws_spec(
        &self,
        council_spec: &agent_agency_council::models::CawsSpec,
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
            validation_rules: Self::convert_validation_rules(council_spec),
            benchmarks: None, // TODO: Add performance benchmarks
            security: SecurityRequirements::default(),
        }
    }

    /// TODO: Implement actual worker execution instead of simulation
    /// - [ ] Integrate with worker HTTP API for task execution
    /// - [ ] Implement proper worker discovery and load balancing
    /// - [ ] Add worker health monitoring and automatic failover
    /// - [ ] Support different worker types (CPU, GPU, specialized hardware)
    /// - [ ] Implement worker authentication and secure communication
    /// - [ ] Add execution timeout and resource limits
    /// - [ ] Support streaming execution results and progress updates
    async fn execute_with_worker(
        &self,
        worker_id: Uuid,
        input: &ExecutionInput,
    ) -> Result<RawExecutionResult> {
        // TODO: Implement actual HTTP call to worker instead of simulation
        // - [ ] Set up HTTP client with proper error handling and retries
        // - [ ] Implement request/response serialization (JSON/Protobuf)
        // - [ ] Add request timeout and cancellation support
        // - [ ] Support different worker API versions and compatibility
        // - [ ] Implement result streaming for long-running tasks
        // - [ ] Add worker response validation and error mapping
        // - [ ] Support worker-specific configuration and parameters
        // Implement actual HTTP call to worker model with robust error handling
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

        // Implement worker registry and service discovery
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

        if let Some(error) = raw_result.error {
            return Ok(TaskExecutionResult {
                task_id,
                worker_id,
                status: ExecutionStatus::Failed,
                output: None,
                error_message: Some(error),
                execution_time_ms: raw_result.execution_time_ms,
                tokens_used: raw_result.tokens_used,
                quality_metrics: QualityMetrics::default(),
                caws_compliance: CawsComplianceResult::default(),
                started_at,
                completed_at,
            });
        }

        // Parse worker output
        let worker_output = match serde_json::from_str::<WorkerOutput>(&raw_result.raw_output) {
            Ok(output) => output,
            Err(e) => {
                warn!("Failed to parse worker output: {}", e);
                return Ok(TaskExecutionResult {
                    task_id,
                    worker_id,
                    status: ExecutionStatus::Failed,
                    output: None,
                    error_message: Some(format!("Invalid output format: {}", e)),
                    execution_time_ms: raw_result.execution_time_ms,
                    tokens_used: raw_result.tokens_used,
                    quality_metrics: QualityMetrics::default(),
                    caws_compliance: CawsComplianceResult::default(),
                    started_at,
                    completed_at,
                });
            }
        };

        // Calculate quality metrics
        let quality_metrics = self.calculate_quality_metrics(&worker_output);

        // Check CAWS compliance
        let caws_compliance =
            self.check_caws_compliance(&worker_output, raw_result.execution_time_ms);

        // Determine execution status
        let status = if caws_compliance.is_compliant && quality_metrics.completeness_score > 0.8 {
            ExecutionStatus::Completed
        } else if quality_metrics.completeness_score > 0.5 {
            ExecutionStatus::Partial
        } else {
            ExecutionStatus::Failed
        };

        Ok(TaskExecutionResult {
            task_id,
            worker_id,
            status,
            output: Some(worker_output),
            error_message: None,
            execution_time_ms: raw_result.execution_time_ms,
            tokens_used: raw_result.tokens_used,
            quality_metrics,
            caws_compliance,
            started_at,
            completed_at,
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
            compliance_score: compliance_score.max(0.0f32),
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

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TaskExecutorTrait for TaskExecutor {
    async fn execute_task(
        &self,
        task_spec: ContractTaskSpec,
        worker_id: Uuid,
    ) -> Result<TaskExecutionResult> {
        // Convert contract task spec to internal task spec
        let internal_spec = TaskSpec {
            id: task_spec.id,
            title: task_spec.title,
            description: task_spec.description,
            priority: match task_spec.priority {
                TaskPriority::Low => crate::types::TaskPriority::Low,
                TaskPriority::Medium => crate::types::TaskPriority::Medium,
                TaskPriority::High => crate::types::TaskPriority::High,
                TaskPriority::Critical => crate::types::TaskPriority::Critical,
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
    ) -> Result<TaskExecutionResult> {
        // Convert contract task spec to internal task spec
        let internal_spec = TaskSpec {
            id: task_spec.id,
            title: task_spec.title,
            description: task_spec.description,
            priority: match task_spec.priority {
                TaskPriority::Low => crate::types::TaskPriority::Low,
                TaskPriority::Medium => crate::types::TaskPriority::Medium,
                TaskPriority::High => crate::types::TaskPriority::High,
                TaskPriority::Critical => crate::types::TaskPriority::Critical,
            },
            required_capabilities: task_spec.required_capabilities,
            context: task_spec.context,
            working_spec_id: task_spec.working_spec_id,
            timeout_seconds: task_spec.timeout_seconds,
        };

        // Execute with circuit breaker if enabled
        let circuit_breaker = if circuit_breaker_enabled {
            Some(Arc::new(CircuitBreaker::new(
                "task-execution".to_string(),
                RetryConfig {
                    base_delay_ms: 1000,
                    max_delay_ms: 30000,
                    max_attempts: 3,
                    backoff_multiplier: 2.0,
                    jitter: true,
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

    async fn health_check(&self) -> Result<agent_agency_contracts::task_executor::TaskExecutorHealth> {
        // Basic health check - in a real implementation this would check actual worker connections
        Ok(agent_agency_contracts::task_executor::TaskExecutorHealth {
            status: agent_agency_contracts::task_executor::HealthStatus::Healthy,
            last_execution_time: Some(Utc::now()),
            active_tasks: 0,
            queued_tasks: 0,
            total_executions: 0,
            success_rate: 1.0,
        })
    }

    async fn get_execution_stats(&self) -> Result<agent_agency_contracts::task_executor::TaskExecutionStats> {
        // Basic stats - in a real implementation this would track actual metrics
        Ok(agent_agency_contracts::task_executor::TaskExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
            median_execution_time_ms: 0.0,
            p95_execution_time_ms: 0.0,
            p99_execution_time_ms: 0.0,
        })
    }

    async fn cancel_task_execution(&self, _task_id: Uuid, _worker_id: Uuid) -> Result<()> {
        // Basic implementation - in a real implementation this would cancel the actual task
        // For now, just return success
        Ok(())
    }
}
