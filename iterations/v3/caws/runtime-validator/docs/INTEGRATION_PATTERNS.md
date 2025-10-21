# CAWS Runtime Validator Integration Patterns

**Version**: 1.0.0  
**Date**: December 2024  
**Purpose**: Comprehensive guide to integrating with the CAWS runtime-validator

---

## Overview

This document provides detailed integration patterns for using the CAWS runtime-validator across different components of the V3 system. It covers common use cases, best practices, and implementation examples.

## Architecture Overview

The CAWS runtime-validator provides a centralized service for:

- **Policy Validation**: Enforcing CAWS policies across all components
- **Budget Checking**: Validating change budgets and resource limits
- **Waiver Management**: Handling waiver requests and approvals
- **Language Analysis**: Multi-language code analysis and validation
- **Integration Interfaces**: MCP and orchestration-specific integration traits

## Core Components

### 1. CawsValidator

The core validation engine that enforces CAWS policies:

```rust
use caws_runtime_validator::{CawsValidator, CawsPolicy};

let policy = CawsPolicy::default();
let validator = Arc::new(CawsValidator::new(policy));
```

### 2. Integration Traits

Specialized integration interfaces for different system components:

- **McpCawsIntegration**: MCP tool validation and execution recording
- **OrchestrationIntegration**: Task validation and execution mode checking
- **DefaultOrchestrationIntegration**: Default orchestration implementation

### 3. Language Analyzers

Multi-language code analysis capabilities:

```rust
use caws_runtime_validator::analyzers::{
    LanguageAnalyzerRegistry, RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer
};

let mut registry = LanguageAnalyzerRegistry::new();
registry.register(Box::new(RustAnalyzer::new()));
registry.register(Box::new(TypeScriptAnalyzer::new()));
registry.register(Box::new(JavaScriptAnalyzer::new()));
```

## Integration Patterns

### Pattern 1: MCP Integration

#### Basic MCP Integration

```rust
use caws_runtime_validator::integration::McpCawsIntegration;
use agent_agency_mcp::types::{ToolManifest, ToolCapability, ToolParameter};

pub struct MCPServer {
    caws_integration: Arc<McpCawsIntegration>,
}

impl MCPServer {
    pub fn new() -> Self {
        Self {
            caws_integration: Arc::new(McpCawsIntegration::new()),
        }
    }
    
    pub async fn validate_tool(&self, manifest: &ToolManifest) -> Result<ToolValidationResult> {
        let validation_result = self.caws_integration
            .validate_tool_manifest(manifest)
            .await?;
            
        Ok(ToolValidationResult {
            is_valid: validation_result.is_valid,
            violations: validation_result.violations,
            recommendations: validation_result.recommendations,
        })
    }
    
    pub async fn record_execution(
        &self,
        tool_name: &str,
        duration: Duration,
        success: bool,
    ) -> Result<ExecutionRecord> {
        let record = self.caws_integration
            .record_tool_execution(tool_name, duration, success)
            .await?;
            
        Ok(ExecutionRecord {
            tool_name: record.tool_name,
            execution_time: record.execution_time,
            success: record.success,
            recorded: record.recorded,
        })
    }
}
```

#### Advanced MCP Integration with Custom Validation

```rust
use caws_runtime_validator::integration::McpCawsIntegration;

pub struct AdvancedMCPServer {
    caws_integration: Arc<McpCawsIntegration>,
    custom_validators: Vec<Box<dyn CustomValidator>>,
}

impl AdvancedMCPServer {
    pub async fn validate_tool_comprehensive(
        &self,
        manifest: &ToolManifest,
    ) -> Result<ComprehensiveValidationResult> {
        // CAWS validation
        let caws_result = self.caws_integration
            .validate_tool_manifest(manifest)
            .await?;
            
        // Custom validation
        let mut custom_violations = Vec::new();
        for validator in &self.custom_validators {
            if let Some(violation) = validator.validate(manifest)? {
                custom_violations.push(violation);
            }
        }
        
        Ok(ComprehensiveValidationResult {
            caws_validation: caws_result,
            custom_violations,
            overall_valid: caws_result.is_valid && custom_violations.is_empty(),
        })
    }
}
```

### Pattern 2: Orchestration Integration

#### Basic Orchestration Integration

```rust
use caws_runtime_validator::integration::{
    OrchestrationIntegration, DefaultOrchestrationIntegration
};

pub struct TaskOrchestrator {
    caws_integration: Arc<DefaultOrchestrationIntegration>,
}

impl TaskOrchestrator {
    pub fn new() -> Self {
        Self {
            caws_integration: Arc::new(DefaultOrchestrationIntegration::new()),
        }
    }
    
    pub async fn validate_task_execution(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
        diff_stats: &DiffStats,
    ) -> Result<ValidationResult> {
        let runtime_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: working_spec.risk_tier,
            scope_in: working_spec.scope_in.clone(),
            change_budget_max_files: working_spec.change_budget_max_files,
            change_budget_max_loc: working_spec.change_budget_max_loc,
        };
        
        let runtime_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: task_descriptor.task_id.clone(),
            scope_in: task_descriptor.scope_in.clone(),
            risk_tier: task_descriptor.risk_tier,
            execution_mode: self.convert_execution_mode(&task_descriptor.execution_mode),
        };
        
        let runtime_diff = caws_runtime_validator::DiffStats {
            files_changed: diff_stats.files_changed,
            lines_added: diff_stats.lines_added,
            lines_removed: diff_stats.lines_removed,
            lines_modified: diff_stats.lines_modified,
        };
        
        let validation_result = self.caws_integration
            .validate_task_execution(
                &runtime_spec,
                &runtime_descriptor,
                &runtime_diff,
                &[], // patches
                &[], // language_hints
                true, // tests_added
                true, // deterministic
                vec![], // waivers
            )
            .await?;
            
        Ok(self.convert_validation_result(&validation_result))
    }
    
    fn convert_execution_mode(
        &self,
        mode: &ExecutionMode,
    ) -> caws_runtime_validator::ExecutionMode {
        match mode {
            ExecutionMode::Strict => caws_runtime_validator::ExecutionMode::Strict,
            ExecutionMode::Auto => caws_runtime_validator::ExecutionMode::Auto,
            ExecutionMode::DryRun => caws_runtime_validator::ExecutionMode::DryRun,
        }
    }
    
    fn convert_validation_result(
        &self,
        result: &caws_runtime_validator::OrchestrationValidationResult,
    ) -> ValidationResult {
        ValidationResult {
            task_id: result.task_id.clone(),
            snapshot: ComplianceSnapshot {
                within_scope: result.snapshot.within_scope,
                within_budget: result.snapshot.within_budget,
                tests_added: result.snapshot.tests_added,
                deterministic: result.snapshot.deterministic,
            },
            violations: result.violations.iter().map(|v| self.convert_violation(v)).collect(),
        }
    }
}
```

#### Advanced Orchestration with Execution Mode Management

```rust
use caws_runtime_validator::integration::DefaultOrchestrationIntegration;

pub struct AdvancedTaskOrchestrator {
    caws_integration: Arc<DefaultOrchestrationIntegration>,
    execution_mode_manager: ExecutionModeManager,
}

impl AdvancedTaskOrchestrator {
    pub async fn execute_task_with_caws_validation(
        &self,
        task: &Task,
    ) -> Result<TaskExecutionResult> {
        // Pre-execution validation
        let pre_validation = self.validate_task_execution(&task).await?;
        
        // Check execution mode
        let execution_decision = self.caws_integration
            .check_execution_mode(
                caws_runtime_validator::ExecutionMode::Strict,
                &pre_validation.violations,
            )
            .await?;
            
        if !execution_decision.allowed {
            // Try alternative execution mode
            let alternative_decision = self.caws_integration
                .check_execution_mode(
                    execution_decision.recommended_mode,
                    &pre_validation.violations,
                )
                .await?;
                
            if !alternative_decision.allowed {
                return Err(anyhow::anyhow!("Task execution not allowed due to CAWS violations"));
            }
        }
        
        // Execute task
        let execution_result = self.execute_task(task).await?;
        
        // Post-execution validation
        let post_validation = self.validate_execution_result(&execution_result).await?;
        
        // Generate waiver if needed
        if !post_validation.violations.is_empty() {
            if let Some(waiver) = self.generate_waiver_if_eligible(&task, &post_validation).await? {
                return Ok(TaskExecutionResult {
                    execution: execution_result,
                    validation: post_validation,
                    waiver: Some(waiver),
                });
            }
        }
        
        Ok(TaskExecutionResult {
            execution: execution_result,
            validation: post_validation,
            waiver: None,
        })
    }
    
    async fn generate_waiver_if_eligible(
        &self,
        task: &Task,
        validation: &ValidationResult,
    ) -> Result<Option<WaiverResult>> {
        let waiver_eligibility = self.determine_waiver_eligibility(task, validation);
        
        if waiver_eligibility.eligible {
            let waiver_result = self.caws_integration
                .generate_waiver_if_eligible(
                    &task.id,
                    validation.violations.iter().map(|v| v.message.clone()).collect(),
                    &waiver_eligibility,
                )
                .await?;
                
            Ok(waiver_result)
        } else {
            Ok(None)
        }
    }
}
```

### Pattern 3: Workers Integration

#### Basic Workers Integration

```rust
use caws_runtime_validator::{
    CawsValidator, integration::DefaultOrchestrationIntegration,
    analyzers::LanguageAnalyzerRegistry,
};

pub struct CawsChecker {
    runtime_validator: Arc<CawsValidator>,
    orchestration_integration: Arc<DefaultOrchestrationIntegration>,
    language_analyzers: Arc<LanguageAnalyzerRegistry>,
}

impl CawsChecker {
    pub fn new() -> Self {
        let policy = caws_runtime_validator::CawsPolicy::default();
        let runtime_validator = Arc::new(CawsValidator::new(policy.clone()));
        let orchestration_integration = Arc::new(DefaultOrchestrationIntegration::new());
        
        let mut language_analyzers = LanguageAnalyzerRegistry::new();
        language_analyzers.register(Box::new(RustAnalyzer::new()));
        language_analyzers.register(Box::new(TypeScriptAnalyzer::new()));
        language_analyzers.register(Box::new(JavaScriptAnalyzer::new()));
        
        Self {
            runtime_validator,
            orchestration_integration,
            language_analyzers: Arc::new(language_analyzers),
        }
    }
    
    pub async fn validate_autonomous_execution(
        &self,
        working_spec: &WorkingSpec,
        artifacts: &ExecutionArtifacts,
    ) -> Result<ValidationResult> {
        // Convert to runtime-validator types
        let runtime_spec = self.convert_working_spec(working_spec);
        let task_descriptor = self.create_task_descriptor(artifacts);
        let diff_stats = self.convert_diff_stats(&artifacts.diff_stats);
        
        // Validate with runtime-validator
        let validation_result = self.orchestration_integration
            .validate_task_execution(
                &runtime_spec,
                &task_descriptor,
                &diff_stats,
                &[], // patches
                &[], // language_hints
                artifacts.test_results.total > 0, // tests_added
                self.is_deterministic(artifacts), // deterministic
                vec![], // waivers
            )
            .await?;
            
        Ok(self.convert_validation_result(&validation_result))
    }
}
```

#### Advanced Workers Integration with Language Analysis

```rust
use caws_runtime_validator::analyzers::LanguageAnalyzerRegistry;

pub struct AdvancedCawsChecker {
    runtime_validator: Arc<CawsValidator>,
    orchestration_integration: Arc<DefaultOrchestrationIntegration>,
    language_analyzers: Arc<LanguageAnalyzerRegistry>,
    custom_analyzers: Vec<Box<dyn CustomAnalyzer>>,
}

impl AdvancedCawsChecker {
    pub async fn analyze_code_changes(
        &self,
        file_path: &str,
        diff: &str,
    ) -> Result<LanguageAnalysisResult> {
        // Determine file language
        let language = self.determine_language(file_path);
        
        // Get appropriate analyzer
        let analyzer = self.language_analyzers
            .get_analyzer(&language)
            .ok_or_else(|| anyhow::anyhow!("No analyzer found for language: {}", language))?;
            
        // Perform language-specific analysis
        let analysis_result = analyzer.analyze(file_path, diff).await?;
        
        // Run custom analyzers
        let mut custom_violations = Vec::new();
        for custom_analyzer in &self.custom_analyzers {
            if let Some(violations) = custom_analyzer.analyze(file_path, diff).await? {
                custom_violations.extend(violations);
            }
        }
        
        Ok(LanguageAnalysisResult {
            language: analysis_result.language,
            complexity: analysis_result.complexity,
            violations: analysis_result.violations,
            warnings: analysis_result.warnings,
            custom_violations,
            metrics: analysis_result.metrics,
        })
    }
    
    fn determine_language(&self, file_path: &str) -> ProgrammingLanguage {
        if file_path.ends_with(".rs") {
            ProgrammingLanguage::Rust
        } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
            ProgrammingLanguage::TypeScript
        } else if file_path.ends_with(".js") || file_path.ends_with(".jsx") {
            ProgrammingLanguage::JavaScript
        } else {
            ProgrammingLanguage::Unknown
        }
    }
}
```

### Pattern 4: Budget Checking Integration

#### Basic Budget Checking

```rust
use caws_runtime_validator::{BudgetChecker, CawsPolicy};

pub struct BudgetManager {
    budget_checker: Arc<BudgetChecker>,
}

impl BudgetManager {
    pub fn new() -> Self {
        let policy = CawsPolicy::default();
        let budget_checker = Arc::new(BudgetChecker::new(policy));
        
        Self { budget_checker }
    }
    
    pub async fn check_budget(
        &self,
        working_spec: &WorkingSpec,
        diff_stats: &DiffStats,
    ) -> Result<BudgetValidationResult> {
        let runtime_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: working_spec.risk_tier,
            scope_in: working_spec.scope_in.clone(),
            change_budget_max_files: working_spec.change_budget_max_files,
            change_budget_max_loc: working_spec.change_budget_max_loc,
        };
        
        let runtime_diff = caws_runtime_validator::DiffStats {
            files_changed: diff_stats.files_changed,
            lines_added: diff_stats.lines_added,
            lines_removed: diff_stats.lines_removed,
            lines_modified: diff_stats.lines_modified,
        };
        
        let budget_result = self.budget_checker
            .check_budget(&runtime_spec, &runtime_diff)
            .await?;
            
        Ok(BudgetValidationResult {
            within_budget: budget_result.within_budget,
            violations: budget_result.violations,
            budget_usage: self.calculate_budget_usage(&runtime_spec, &runtime_diff),
        })
    }
    
    fn calculate_budget_usage(
        &self,
        spec: &caws_runtime_validator::WorkingSpec,
        diff: &caws_runtime_validator::DiffStats,
    ) -> BudgetUsage {
        let file_usage = (diff.files_changed as f32) / (spec.change_budget_max_files as f32);
        let loc_usage = (diff.lines_added + diff.lines_modified) as f32 / (spec.change_budget_max_loc as f32);
        
        BudgetUsage {
            files_percentage: file_usage * 100.0,
            loc_percentage: loc_usage * 100.0,
            files_remaining: spec.change_budget_max_files.saturating_sub(diff.files_changed),
            loc_remaining: spec.change_budget_max_loc.saturating_sub(diff.lines_added + diff.lines_modified),
        }
    }
}
```

#### Advanced Budget Checking with Predictive Analysis

```rust
use caws_runtime_validator::{BudgetChecker, CawsPolicy};

pub struct PredictiveBudgetManager {
    budget_checker: Arc<BudgetChecker>,
    historical_data: HistoricalDataStore,
    prediction_model: BudgetPredictionModel,
}

impl PredictiveBudgetManager {
    pub async fn predict_budget_usage(
        &self,
        working_spec: &WorkingSpec,
        planned_changes: &[PlannedChange],
    ) -> Result<BudgetPrediction> {
        // Analyze historical data
        let historical_usage = self.historical_data
            .get_similar_tasks(&working_spec.scope_in)
            .await?;
            
        // Use prediction model
        let predicted_usage = self.prediction_model
            .predict_budget_usage(&working_spec, planned_changes, &historical_usage)
            .await?;
            
        // Validate against current budget
        let budget_validation = self.budget_checker
            .check_budget(&working_spec, &predicted_usage.diff_stats)
            .await?;
            
        Ok(BudgetPrediction {
            predicted_usage,
            within_budget: budget_validation.within_budget,
            confidence: predicted_usage.confidence,
            recommendations: self.generate_recommendations(&predicted_usage, &budget_validation),
        })
    }
}
```

### Pattern 5: Waiver Management Integration

#### Basic Waiver Management

```rust
use caws_runtime_validator::WaiverManager;

pub struct WaiverService {
    waiver_manager: Arc<WaiverManager>,
}

impl WaiverService {
    pub fn new() -> Self {
        let waiver_manager = Arc::new(WaiverManager::new());
        Self { waiver_manager }
    }
    
    pub async fn request_waiver(
        &self,
        task_id: &str,
        violations: Vec<Violation>,
        reason: WaiverReason,
    ) -> Result<WaiverRequest> {
        let waiver_request = WaiverRequest {
            task_id: task_id.to_string(),
            violations,
            reason,
            requested_at: chrono::Utc::now(),
            status: WaiverStatus::Pending,
        };
        
        // Submit to waiver manager
        let waiver_result = self.waiver_manager
            .request_waiver(&waiver_request)
            .await?;
            
        Ok(waiver_request)
    }
    
    pub async fn approve_waiver(
        &self,
        waiver_id: &str,
        approver: &str,
        conditions: Option<Vec<String>>,
    ) -> Result<WaiverApproval> {
        let approval = WaiverApproval {
            waiver_id: waiver_id.to_string(),
            approver: approver.to_string(),
            approved_at: chrono::Utc::now(),
            conditions,
            status: WaiverStatus::Approved,
        };
        
        let result = self.waiver_manager
            .approve_waiver(&approval)
            .await?;
            
        Ok(approval)
    }
}
```

#### Advanced Waiver Management with Automated Approval

```rust
use caws_runtime_validator::WaiverManager;

pub struct AutomatedWaiverService {
    waiver_manager: Arc<WaiverManager>,
    approval_rules: ApprovalRuleEngine,
    risk_assessor: RiskAssessor,
}

impl AutomatedWaiverService {
    pub async fn process_waiver_request(
        &self,
        waiver_request: &WaiverRequest,
    ) -> Result<WaiverProcessingResult> {
        // Assess risk level
        let risk_assessment = self.risk_assessor
            .assess_waiver_risk(waiver_request)
            .await?;
            
        // Check if automatic approval is possible
        if let Some(auto_approval) = self.approval_rules
            .check_auto_approval(waiver_request, &risk_assessment)
            .await? {
            return Ok(WaiverProcessingResult::AutoApproved(auto_approval));
        }
        
        // Check if automatic rejection is required
        if let Some(auto_rejection) = self.approval_rules
            .check_auto_rejection(waiver_request, &risk_assessment)
            .await? {
            return Ok(WaiverProcessingResult::AutoRejected(auto_rejection));
        }
        
        // Requires manual review
        Ok(WaiverProcessingResult::RequiresReview(waiver_request.clone()))
    }
}
```

## Error Handling Patterns

### Pattern 1: Graceful Degradation

```rust
use caws_runtime_validator::integration::DefaultOrchestrationIntegration;

pub struct ResilientOrchestrator {
    caws_integration: Arc<DefaultOrchestrationIntegration>,
    fallback_validator: Arc<FallbackValidator>,
}

impl ResilientOrchestrator {
    pub async fn validate_with_fallback(
        &self,
        task: &Task,
    ) -> Result<ValidationResult> {
        // Try primary CAWS validation
        match self.caws_integration.validate_task_execution(task).await {
            Ok(result) => Ok(result),
            Err(e) => {
                // Log the error
                tracing::warn!("CAWS validation failed, falling back to legacy validator: {}", e);
                
                // Use fallback validator
                self.fallback_validator.validate(task).await
            }
        }
    }
}
```

### Pattern 2: Retry with Exponential Backoff

```rust
use caws_runtime_validator::integration::DefaultOrchestrationIntegration;

pub struct RetryableOrchestrator {
    caws_integration: Arc<DefaultOrchestrationIntegration>,
    max_retries: u32,
    base_delay: Duration,
}

impl RetryableOrchestrator {
    pub async fn validate_with_retry(
        &self,
        task: &Task,
    ) -> Result<ValidationResult> {
        let mut attempt = 0;
        let mut delay = self.base_delay;
        
        loop {
            match self.caws_integration.validate_task_execution(task).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.max_retries => {
                    attempt += 1;
                    tracing::warn!(
                        "CAWS validation attempt {} failed, retrying in {:?}: {}", 
                        attempt, delay, e
                    );
                    
                    tokio::time::sleep(delay).await;
                    delay = delay * 2; // Exponential backoff
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

### Pattern 3: Circuit Breaker

```rust
use caws_runtime_validator::integration::DefaultOrchestrationIntegration;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub struct CircuitBreakerOrchestrator {
    caws_integration: Arc<DefaultOrchestrationIntegration>,
    circuit_open: AtomicBool,
    failure_count: AtomicU32,
    failure_threshold: u32,
}

impl CircuitBreakerOrchestrator {
    pub async fn validate_with_circuit_breaker(
        &self,
        task: &Task,
    ) -> Result<ValidationResult> {
        // Check if circuit is open
        if self.circuit_open.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("CAWS validation circuit breaker is open"));
        }
        
        // Attempt validation
        match self.caws_integration.validate_task_execution(task).await {
            Ok(result) => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
                Ok(result)
            }
            Err(e) => {
                // Increment failure count
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                
                if failures >= self.failure_threshold {
                    // Open circuit
                    self.circuit_open.store(true, Ordering::Relaxed);
                    tracing::error!("CAWS validation circuit breaker opened after {} failures", failures);
                }
                
                Err(e)
            }
        }
    }
}
```

## Testing Patterns

### Pattern 1: Mock Integration for Testing

```rust
use caws_runtime_validator::integration::McpCawsIntegration;
use mockall::mock;

mock! {
    MockMcpCawsIntegration {}
    
    #[async_trait]
    impl McpCawsIntegration for MockMcpCawsIntegration {
        async fn validate_tool_manifest(
            &self,
            manifest: &ToolManifest,
        ) -> Result<ToolValidationResult>;
        
        async fn record_tool_execution(
            &self,
            tool_name: &str,
            execution_time: Duration,
            success: bool,
        ) -> Result<ExecutionRecord>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_server_with_mock() {
        let mut mock_integration = MockMcpCawsIntegration::new();
        
        // Setup mock expectations
        mock_integration
            .expect_validate_tool_manifest()
            .times(1)
            .returning(|_| Ok(ToolValidationResult {
                is_valid: true,
                violations: vec![],
                recommendations: vec![],
            }));
            
        let server = MCPServer {
            caws_integration: Arc::new(mock_integration),
        };
        
        // Test the server
        let manifest = create_test_manifest();
        let result = server.validate_tool(&manifest).await.unwrap();
        
        assert!(result.is_valid);
    }
}
```

### Pattern 2: Integration Test Fixtures

```rust
pub struct CawsTestFixtures {
    pub policy: CawsPolicy,
    pub validator: Arc<CawsValidator>,
    pub mcp_integration: Arc<McpCawsIntegration>,
    pub orchestration_integration: Arc<DefaultOrchestrationIntegration>,
    pub language_analyzers: Arc<LanguageAnalyzerRegistry>,
    pub budget_checker: Arc<BudgetChecker>,
    pub waiver_manager: Arc<WaiverManager>,
}

impl CawsTestFixtures {
    pub fn new() -> Self {
        let policy = CawsPolicy::default();
        let validator = Arc::new(CawsValidator::new(policy.clone()));
        let mcp_integration = Arc::new(McpCawsIntegration::new());
        let orchestration_integration = Arc::new(DefaultOrchestrationIntegration::new());
        
        let mut language_analyzers = LanguageAnalyzerRegistry::new();
        language_analyzers.register(Box::new(RustAnalyzer::new()));
        language_analyzers.register(Box::new(TypeScriptAnalyzer::new()));
        language_analyzers.register(Box::new(JavaScriptAnalyzer::new()));
        
        let budget_checker = Arc::new(BudgetChecker::new(policy.clone()));
        let waiver_manager = Arc::new(WaiverManager::new());

        Self {
            policy,
            validator,
            mcp_integration,
            orchestration_integration,
            language_analyzers: Arc::new(language_analyzers),
            budget_checker,
            waiver_manager,
        }
    }
}
```

## Performance Optimization Patterns

### Pattern 1: Caching Validation Results

```rust
use caws_runtime_validator::integration::DefaultOrchestrationIntegration;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct CachedOrchestrator {
    caws_integration: Arc<DefaultOrchestrationIntegration>,
    validation_cache: Arc<RwLock<HashMap<String, ValidationResult>>>,
    cache_ttl: Duration,
}

impl CachedOrchestrator {
    pub async fn validate_with_cache(
        &self,
        task: &Task,
    ) -> Result<ValidationResult> {
        let cache_key = self.generate_cache_key(task);
        
        // Check cache first
        {
            let cache = self.validation_cache.read().unwrap();
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok(cached_result.clone());
            }
        }
        
        // Perform validation
        let result = self.caws_integration.validate_task_execution(task).await?;
        
        // Cache the result
        {
            let mut cache = self.validation_cache.write().unwrap();
            cache.insert(cache_key, result.clone());
        }
        
        Ok(result)
    }
    
    fn generate_cache_key(&self, task: &Task) -> String {
        format!("{}_{}_{}", task.id, task.risk_tier, task.scope_hash())
    }
}
```

### Pattern 2: Batch Validation

```rust
use caws_runtime_validator::integration::DefaultOrchestrationIntegration;

pub struct BatchOrchestrator {
    caws_integration: Arc<DefaultOrchestrationIntegration>,
    batch_size: usize,
}

impl BatchOrchestrator {
    pub async fn validate_batch(
        &self,
        tasks: Vec<Task>,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        
        for chunk in tasks.chunks(self.batch_size) {
            let chunk_results = self.validate_chunk(chunk).await?;
            results.extend(chunk_results);
        }
        
        Ok(results)
    }
    
    async fn validate_chunk(
        &self,
        tasks: &[Task],
    ) -> Result<Vec<ValidationResult>> {
        let mut handles = Vec::new();
        
        for task in tasks {
            let integration = self.caws_integration.clone();
            let task = task.clone();
            
            let handle = tokio::spawn(async move {
                integration.validate_task_execution(&task).await
            });
            
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await??;
            results.push(result);
        }
        
        Ok(results)
    }
}
```

## Best Practices

### 1. Error Handling

- Always handle validation errors gracefully
- Provide meaningful error messages
- Implement fallback mechanisms for critical paths
- Log errors for debugging and monitoring

### 2. Performance

- Use caching for frequently validated tasks
- Implement batch processing for multiple validations
- Monitor validation performance and optimize as needed
- Use circuit breakers for external dependencies

### 3. Testing

- Write comprehensive unit tests for integration logic
- Use mocks for external dependencies in tests
- Test error scenarios and edge cases
- Validate performance characteristics

### 4. Monitoring

- Log validation metrics and performance data
- Monitor error rates and validation failures
- Track budget usage and violation patterns
- Alert on unusual validation behavior

### 5. Security

- Validate all inputs before processing
- Sanitize error messages to avoid information leakage
- Implement proper authentication and authorization
- Audit validation decisions and waiver approvals

## Conclusion

These integration patterns provide a comprehensive guide for integrating with the CAWS runtime-validator. Choose the patterns that best fit your use case and adapt them to your specific requirements. The patterns are designed to be flexible and extensible, allowing you to build robust, performant, and maintainable integrations.
