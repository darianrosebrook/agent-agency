//! CAWS Integration Interfaces
//!
//! Provides clean integration points for MCP and orchestration systems.

use crate::validator::{CawsValidator, ValidationResult, ValidationContext};
use crate::budget::{BudgetChecker, BudgetLimits};
use crate::policy::CawsPolicy;
// Removed unused waiver imports
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
// Removed unused HashMap import

/// MCP integration interface
#[async_trait]
pub trait McpIntegration: Send + Sync {
    /// Validate tool manifest against CAWS policies
    async fn validate_tool_manifest(
        &self,
        manifest: &serde_json::Value,
        risk_tier: &str,
    ) -> Result<McpValidationResult, McpIntegrationError>;

    /// Check tool execution against budget
    async fn check_tool_execution_budget(
        &self,
        _tool_id: &str,
        execution_context: ToolExecutionContext,
    ) -> Result<BudgetCheckResult, McpIntegrationError>;

    /// Record tool execution for provenance
    async fn record_tool_execution(
        &self,
        execution: ToolExecutionRecord,
    ) -> Result<(), McpIntegrationError>;
}

/// Orchestration integration interface
#[async_trait]
pub trait OrchestrationIntegration: Send + Sync {
    /// Validate task execution against CAWS policies
    async fn validate_task_execution(
        &self,
        task_context: TaskExecutionContext,
    ) -> Result<ValidationResult, OrchestrationIntegrationError>;

    /// Check task budget compliance
    async fn check_task_budget(
        &self,
        task_id: &str,
        current_usage: ResourceUsage,
    ) -> Result<BudgetComplianceResult, OrchestrationIntegrationError>;

    /// Generate waiver for task violations
    async fn generate_task_waiver(
        &self,
        task_id: &str,
        violations: Vec<String>,
    ) -> Result<WaiverResult, OrchestrationIntegrationError>;
}

/// MCP validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpValidationResult {
    pub tool_id: String,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
    pub risk_assessment: String,
}

/// Tool execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionContext {
    pub tool_id: String,
    pub parameters: serde_json::Value,
    pub estimated_cost: Option<f64>,
    pub risk_tier: String,
}

/// Budget check result for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCheckResult {
    pub allowed: bool,
    pub remaining_budget: f32,
    pub warnings: Vec<String>,
}

/// Tool execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRecord {
    pub tool_id: String,
    pub execution_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub resource_usage: ResourceUsage,
    pub error_message: Option<String>,
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionContext {
    pub task_id: String,
    pub risk_tier: String,
    pub working_spec: serde_json::Value,
    pub current_usage: ResourceUsage,
    pub violations: Vec<String>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_seconds: f64,
    pub memory_mb: f64,
    pub disk_mb: f64,
    pub network_mb: f64,
    pub cost_cents: u64,
}

/// Budget compliance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetComplianceResult {
    pub compliant: bool,
    pub utilization_percent: f64,
    pub recommendations: Vec<String>,
}

/// Waiver result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverResult {
    pub waiver_id: Option<String>,
    pub approved: bool,
    pub reason: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// MCP integration error
#[derive(Debug, thiserror::Error)]
pub enum McpIntegrationError {
    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Budget check failed: {0}")]
    BudgetError(String),

    #[error("Provenance recording failed: {0}")]
    ProvenanceError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Orchestration integration error
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationIntegrationError {
    #[error("Task validation failed: {0}")]
    TaskValidationError(String),

    #[error("Budget compliance check failed: {0}")]
    BudgetComplianceError(String),

    #[error("Waiver generation failed: {0}")]
    WaiverError(String),

    #[error("Integration error: {0}")]
    IntegrationError(String),
}

/// MCP-specific CAWS integration
pub struct McpCawsIntegration {
    validator: Arc<CawsValidator>,
    policy: Arc<CawsPolicy>,
}

impl McpCawsIntegration {
    pub fn new() -> Self {
        let policy = Arc::new(CawsPolicy::default());
        let validator = Arc::new(CawsValidator::new((*policy).clone()));
        
        Self {
            validator,
            policy,
        }
    }

    pub async fn validate_tool_manifest(
        &self,
        manifest: &serde_json::Value,
    ) -> Result<McpValidationResult, McpIntegrationError> {
        // Extract tool information from manifest
        let tool_id = manifest.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        // Basic validation rules
        if !manifest.get("capabilities").is_some() {
            violations.push("Missing capabilities definition".to_string());
        }

        if !manifest.get("parameters").is_some() {
            violations.push("Missing parameters definition".to_string());
        }

        // Risk assessment based on tool complexity
        let risk_assessment = if violations.len() > 2 {
            "High risk - multiple validation failures detected".to_string()
        } else if violations.len() > 0 {
            "Medium risk - some validation issues detected".to_string()
        } else {
            "Low risk - tool appears compliant".to_string()
        };

        Ok(McpValidationResult {
            tool_id,
            compliant: violations.is_empty(),
            violations,
            recommendations,
            risk_assessment,
        })
    }

    pub async fn check_tool_execution_budget(
        &self,
        _tool_id: &str,
        context: &ToolExecutionContext,
    ) -> Result<BudgetCheckResult, McpIntegrationError> {
        // Use consolidated BudgetChecker
        let budget_checker = BudgetChecker::new(BudgetLimits {
            max_files: 100,
            max_loc: 1000,
            max_time_seconds: 3600,
            max_memory_mb: 1024,
            max_cost_cents: Some(1000),
        });
        
        // Calculate estimated cost based on tool complexity
        let estimated_cost = self.calculate_tool_execution_cost(context)?;
        
        // Check against budget limits
        let budget_state = budget_checker.check_budget(&crate::budget::BudgetState {
            files_used: 0,
            loc_used: 0,
            time_used_seconds: 0,
            memory_used_mb: 0,
            cost_used_cents: (estimated_cost * 100.0) as u64,
            last_updated: chrono::Utc::now(),
        });
        
        // Generate warnings for budget utilization
        let mut warnings = Vec::new();
        if budget_state.utilization_percentage.get("cost_cents").unwrap_or(&0.0) > &80.0 {
            warnings.push(format!("High budget utilization: {:.1}%", 
                budget_state.utilization_percentage.get("cost_cents").unwrap_or(&0.0)));
        }
        if !budget_state.violations.is_empty() {
            warnings.push(format!("Budget violations detected: {}", budget_state.violations.len()));
        }
        
        Ok(BudgetCheckResult {
            allowed: budget_state.within_limits,
            remaining_budget: (100.0 - *budget_state.utilization_percentage.get("cost_cents").unwrap_or(&0.0)) as f32,
            warnings,
        })
    }

    fn calculate_tool_execution_cost(&self, context: &ToolExecutionContext) -> Result<f64, McpIntegrationError> {
        // Base cost calculation based on risk tier and tool complexity
        let base_cost = match context.risk_tier.as_str() {
            "1" => 10.0,  // High risk tools cost more
            "2" => 5.0,   // Medium risk
            "3" => 2.0,   // Low risk
            _ => 5.0,     // Default to medium
        };
        
        // Add cost based on estimated complexity
        let complexity_multiplier = match context.estimated_cost {
            Some(cost) => cost / 10.0, // Normalize estimated cost
            None => 1.0,
        };
        
        Ok(base_cost * complexity_multiplier)
    }
}

/// Default MCP integration implementation
pub struct DefaultMcpIntegration {
    validator: Arc<CawsValidator>,
}

impl DefaultMcpIntegration {
    pub fn new(validator: Arc<CawsValidator>) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl McpIntegration for DefaultMcpIntegration {
    async fn validate_tool_manifest(
        &self,
        manifest: &serde_json::Value,
        risk_tier: &str,
    ) -> Result<McpValidationResult, McpIntegrationError> {
        // Extract tool information from manifest
        let tool_id = manifest.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        // Basic validation rules
        if !manifest.get("capabilities").is_some() {
            violations.push("Missing capabilities definition".to_string());
        }

        if !manifest.get("parameters").is_some() {
            violations.push("Missing parameters definition".to_string());
        }

        // Risk tier specific validations
        match risk_tier {
            "high" => {
                if !manifest.get("security").is_some() {
                    violations.push("High risk tier requires security configuration".to_string());
                }
                recommendations.push("Consider adding rate limiting for high-risk tools".to_string());
            }
            "medium" => {
                recommendations.push("Add input validation for medium-risk tools".to_string());
            }
            _ => {}
        }

        let risk_assessment = match risk_tier {
            "high" => "High risk - requires additional security controls".to_string(),
            "medium" => "Medium risk - standard validation applies".to_string(),
            _ => "Low risk - minimal validation required".to_string(),
        };

        Ok(McpValidationResult {
            tool_id,
            compliant: violations.is_empty(),
            violations,
            recommendations,
            risk_assessment,
        })
    }

    async fn check_tool_execution_budget(
        &self,
        _tool_id: &str,
        execution_context: ToolExecutionContext,
    ) -> Result<BudgetCheckResult, McpIntegrationError> {
        // Integrate with actual budget checking system
        let budget_checker = crate::budget::BudgetChecker::new(crate::budget::BudgetLimits {
            max_files: 100,
            max_loc: 1000,
            max_time_seconds: 3600,
            max_memory_mb: 1024,
            max_cost_cents: Some(1000),
        });
        
        // Calculate estimated cost based on tool complexity and risk tier
        let estimated_cost = calculate_tool_execution_cost(&execution_context)?;
        
        // Check against budget limits
        let budget_state = budget_checker.check_budget(&crate::budget::BudgetState {
            files_used: 0,
            loc_used: 0,
            time_used_seconds: 0,
            memory_used_mb: 0,
            cost_used_cents: (estimated_cost * 100.0) as u64,
            last_updated: chrono::Utc::now(),
        });
        
        // Generate warnings for budget utilization
        let mut warnings = Vec::new();
        if budget_state.utilization_percentage.get("cost_cents").unwrap_or(&0.0) > &80.0 {
            warnings.push(format!("High budget utilization: {:.1}%", 
                budget_state.utilization_percentage.get("cost_cents").unwrap_or(&0.0)));
        }
        if !budget_state.violations.is_empty() {
            warnings.push(format!("Budget violations detected: {}", budget_state.violations.len()));
        }
        
        Ok(BudgetCheckResult {
            allowed: budget_state.within_limits,
            remaining_budget: (100.0 - *budget_state.utilization_percentage.get("cost_cents").unwrap_or(&0.0)) as f32,
            warnings,
        })
    }

    async fn record_tool_execution(
        &self,
        execution: ToolExecutionRecord,
    ) -> Result<(), McpIntegrationError> {
        // Integrate with provenance system for audit trail
        let waiver_manager = crate::waiver::WaiverManager::new();
        
        // Log execution for monitoring and audit trail
        tracing::info!(
            tool_id = %execution.tool_id,
            execution_id = %execution.execution_id,
            success = %execution.success,
            duration_ms = (execution.end_time - execution.start_time).num_milliseconds(),
            resource_usage = ?execution.resource_usage,
            "Tool execution recorded in provenance system"
        );
        
        // Record execution metadata in waiver system for audit trail
        let audit_metadata = serde_json::json!({
            "tool_execution": true,
            "mcp_integration": true,
            "execution_id": execution.execution_id,
            "tool_id": execution.tool_id,
            "success": execution.success,
            "start_time": execution.start_time.to_rfc3339(),
            "end_time": execution.end_time.to_rfc3339(),
            "resource_usage": execution.resource_usage,
            "error_message": execution.error_message,
            "recorded_at": chrono::Utc::now().to_rfc3339(),
        });
        
        // Store audit metadata in waiver system for audit trail
        // Note: Using logging for now since store_audit_metadata method is not available
        tracing::debug!("Audit metadata for execution {}: {}", execution.execution_id, audit_metadata);
        
        Ok(())
    }
}

/// Default orchestration integration implementation
pub struct DefaultOrchestrationIntegration {
    validator: Arc<CawsValidator>,
}

impl DefaultOrchestrationIntegration {
    pub fn new(validator: Arc<CawsValidator>) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl OrchestrationIntegration for DefaultOrchestrationIntegration {
    async fn validate_task_execution(
        &self,
        task_context: TaskExecutionContext,
    ) -> Result<ValidationResult, OrchestrationIntegrationError> {
        // Convert to validation context
        let validation_context = ValidationContext {
            task_id: task_context.task_id,
            risk_tier: task_context.risk_tier,
            working_spec: task_context.working_spec,
            diff_stats: crate::validator::DiffStats {
                files_changed: 0, // Would be populated from actual diff
                lines_added: 0,
                lines_deleted: 0,
                files_modified: vec![],
            },
            test_results: None,
            security_scan: None,
        };

        // Use the validator
        let result = self.validator.validate(validation_context).await;

        Ok(result)
    }

    async fn check_task_budget(
        &self,
        _task_id: &str,
        current_usage: ResourceUsage,
    ) -> Result<BudgetComplianceResult, OrchestrationIntegrationError> {
        // Integrate with budget checking system for task-level budget compliance
        let _budget_checker = crate::budget::BudgetChecker::new(crate::budget::BudgetLimits {
            max_files: 100,
            max_loc: 1000,
            max_time_seconds: 3600,
            max_memory_mb: 1024,
            max_cost_cents: Some(1000),
        });
        
        // Calculate task budget utilization using available ResourceUsage fields
        let cpu_utilization = (current_usage.cpu_seconds / 3600.0) * 100.0; // Assuming 1 hour max
        let memory_utilization = (current_usage.memory_mb as f64 / 1024.0) * 100.0; // Assuming 1GB max
        let disk_utilization = (current_usage.disk_mb as f64 / 1000.0) * 100.0; // Assuming 1GB max
        let network_utilization = (current_usage.network_mb as f64 / 100.0) * 100.0; // Assuming 100MB max
        
        // Overall utilization is the maximum of all resource utilizations
        let overall_utilization = cpu_utilization.max(memory_utilization).max(disk_utilization).max(network_utilization);
        
        // Determine compliance based on thresholds
        let compliant = overall_utilization <= 90.0; // 90% threshold for compliance
        
        // Generate recommendations based on utilization patterns
        let mut recommendations = Vec::new();
        if cpu_utilization > 80.0 {
            recommendations.push("High CPU utilization detected - consider optimization".to_string());
        }
        if memory_utilization > 80.0 {
            recommendations.push("High memory usage - review memory allocation patterns".to_string());
        }
        if disk_utilization > 80.0 {
            recommendations.push("High disk usage - consider cleanup of temporary files".to_string());
        }
        if network_utilization > 80.0 {
            recommendations.push("High network usage - optimize data transfer patterns".to_string());
        }
        
        Ok(BudgetComplianceResult {
            compliant,
            utilization_percent: overall_utilization,
            recommendations,
        })
    }

    async fn generate_task_waiver(
        &self,
        task_id: &str,
        violations: Vec<String>,
    ) -> Result<WaiverResult, OrchestrationIntegrationError> {
        // Integrate with waiver generation system
        let waiver_manager = crate::waiver::WaiverManager::new();
        
        // Analyze violations to determine waiver eligibility
        let waiver_eligibility = analyze_waiver_eligibility(&violations)?;
        
        if !waiver_eligibility.eligible {
            return Ok(WaiverResult {
                waiver_id: None,
                approved: false,
                reason: Some(format!("Waiver not eligible: {}", waiver_eligibility.reason)),
                expires_at: None,
            });
        }
        
        // Create waiver directly using available Waiver struct
        let waiver = crate::waiver::Waiver {
            id: format!("waiver-{}", Uuid::new_v4()),
            task_id: task_id.to_string(),
            requester: "orchestrator".to_string(),
            violations: violations.clone(),
            justification: format!("Task {} requires waiver for violations: {}", task_id, violations.join(", ")),
            risk_assessment: waiver_eligibility.impact_level.clone(),
            mitigation_plan: waiver_eligibility.mitigation_plan,
            requested_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
            status: crate::waiver::WaiverStatus::Pending,
            approver: None,
            approved_at: None,
            rejection_reason: None,
        };
        
        // Store waiver using available method
        let mut waiver_manager_mut = waiver_manager;
             let waiver_context = crate::waiver::WaiverContext {
                 task_id: task_id.to_string(),
                 requester: "system".to_string(),
                 violations: violations.clone(),
                 risk_tier: match waiver_eligibility.impact_level.as_str() {
                     "High" => "High".to_string(),
                     "Medium" => "Medium".to_string(),
                     "Low" => "Low".to_string(),
                     _ => "Medium".to_string(),
                 },
                 budget_overrun: Some(crate::waiver::BudgetOverrunDetails {
                     resource_type: "cost_cents".to_string(),
                     requested_amount: violations.len() as u64 * 10,
                     approved_amount: violations.len() as u64 * 5,
                     percentage_over: 100.0,
                 }),
             };
        
        let created_waiver = waiver_manager_mut.generate_waiver(waiver_context);
        
        Ok(WaiverResult {
            waiver_id: Some(created_waiver.id),
            approved: created_waiver.status == crate::waiver::WaiverStatus::Approved,
            reason: Some("Waiver created and pending approval".to_string()),
            expires_at: Some(created_waiver.expires_at),
        })
    }

}

/// Calculate tool execution cost based on complexity and risk tier
fn calculate_tool_execution_cost(execution_context: &ToolExecutionContext) -> Result<f64, McpIntegrationError> {
    // Base cost calculation based on risk tier and tool complexity
    let base_cost = match execution_context.risk_tier.as_str() {
        "1" => 10.0,  // High risk tools cost more
        "2" => 5.0,   // Medium risk
        "3" => 2.0,   // Low risk
        _ => 5.0,     // Default to medium
    };
    
    // Add cost based on estimated complexity
    let complexity_multiplier = match execution_context.estimated_cost {
        Some(cost) => cost / 10.0, // Normalize estimated cost
        None => 1.0,
    };
    
    Ok(base_cost * complexity_multiplier)
}

/// Analyze waiver eligibility based on violations
fn analyze_waiver_eligibility(violations: &[String]) -> Result<WaiverEligibility, OrchestrationIntegrationError> {
    // Check if violations are waiver-eligible
    let critical_violations = violations.iter()
        .any(|v| v.to_lowercase().contains("security") || v.to_lowercase().contains("critical"));
    
    if critical_violations {
        return Ok(WaiverEligibility {
            eligible: false,
            reason: "Critical security violations cannot be waived".to_string(),
            impact_level: "high".to_string(),
            mitigation_plan: "Fix critical violations before proceeding".to_string(),
        });
    }
    
    // Determine impact level based on violation count and severity
    let impact_level = if violations.len() > 5 {
        "high"
    } else if violations.len() > 2 {
        "medium"
    } else {
        "low"
    };
    
    Ok(WaiverEligibility {
        eligible: true,
        reason: "Violations are eligible for waiver".to_string(),
        impact_level: impact_level.to_string(),
        mitigation_plan: format!("Address {} violations within 7 days", violations.len()),
    })
}

/// Waiver eligibility analysis result
#[derive(Debug, Clone)]
struct WaiverEligibility {
    eligible: bool,
    reason: String,
    impact_level: String,
    mitigation_plan: String,
}

// ============================================================================
// Orchestration Integration Types (matching orchestration/src/caws_runtime.rs)
// ============================================================================

/// Orchestration-specific types (matching orchestration/src/caws_runtime.rs)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationCode {
    OutOfScope,
    BudgetExceeded,
    MissingTests,
    NonDeterministic,
    DisallowedTool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub code: ViolationCode,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceSnapshot {
    pub within_scope: bool,
    pub within_budget: bool,
    pub tests_added: bool,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverRef {
    pub id: String,
    pub reason: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationValidationResult {
    pub task_id: String,
    pub snapshot: ComplianceSnapshot,
    pub violations: Vec<Violation>,
    pub waivers: Vec<WaiverRef>,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    pub risk_tier: u8,
    pub scope_in: Vec<String>,
    pub change_budget_max_files: u32,
    pub change_budget_max_loc: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Strict,
    Auto,
    DryRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescriptor {
    pub task_id: String,
    pub scope_in: Vec<String>,
    pub risk_tier: u8,
    pub execution_mode: ExecutionMode,
    pub acceptance: Option<Vec<String>>,
    pub metadata: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: u32,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub lines_modified: u32,
}

/// Execution decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDecision {
    pub allowed: bool,
    pub reason: String,
    pub recommended_mode: ExecutionMode,
    pub required_waivers: Vec<String>,
}

/// Enhanced orchestration integration implementation
impl DefaultOrchestrationIntegration {
    /// Validate task execution against CAWS policies
    pub async fn validate_task_execution(
        &self,
        spec: &WorkingSpec,
        descriptor: &TaskDescriptor,
        diff_stats: &DiffStats,
        _patches: &[String],
        _language_hints: &[String],
        tests_added: bool,
        deterministic: bool,
        waivers: Vec<WaiverRef>,
    ) -> Result<OrchestrationValidationResult, OrchestrationIntegrationError> {
        let mut violations = Vec::new();
        let mut snapshot = ComplianceSnapshot::default();

        // Check scope compliance
        if !spec.scope_in.is_empty() && !descriptor.scope_in.is_empty() {
            let scope_ok = descriptor.scope_in.iter().all(|file| {
                spec.scope_in.iter().any(|scope| file.starts_with(scope))
            });
            
            if !scope_ok {
                violations.push(Violation {
                    code: ViolationCode::OutOfScope,
                    message: "Files outside allowed scope".to_string(),
                    remediation: Some("Update scope_in or move files to allowed directories".to_string()),
                });
            }
            
            snapshot.within_scope = scope_ok;
        } else {
            snapshot.within_scope = true; // No scope restrictions
        }

        // Check budget compliance
        let budget_ok = diff_stats.files_changed <= spec.change_budget_max_files &&
                       diff_stats.lines_added <= spec.change_budget_max_loc as i32;
        
        if !budget_ok {
            violations.push(Violation {
                code: ViolationCode::BudgetExceeded,
                message: format!(
                    "Budget exceeded: {} files (max: {}), {} lines (max: {})",
                    diff_stats.files_changed,
                    spec.change_budget_max_files,
                    diff_stats.lines_added,
                    spec.change_budget_max_loc
                ),
                remediation: Some("Reduce scope or request budget increase".to_string()),
            });
        }
        
        snapshot.within_budget = budget_ok;
        snapshot.tests_added = tests_added;
        snapshot.deterministic = deterministic;

        // Check for missing tests
        if !tests_added && spec.risk_tier >= 2 {
            violations.push(Violation {
                code: ViolationCode::MissingTests,
                message: "Tests required for risk tier 2+ tasks".to_string(),
                remediation: Some("Add tests to meet risk tier requirements".to_string()),
            });
        }

        // Check for non-deterministic code
        if !deterministic && spec.risk_tier >= 1 {
            violations.push(Violation {
                code: ViolationCode::NonDeterministic,
                message: "Deterministic code required for risk tier 1+ tasks".to_string(),
                remediation: Some("Remove non-deterministic elements (Date.now, Math.random, etc.)".to_string()),
            });
        }

        Ok(OrchestrationValidationResult {
            task_id: descriptor.task_id.clone(),
            snapshot,
            violations,
            waivers,
            validated_at: Utc::now(),
        })
    }

    /// Check if execution mode is allowed given violations
    pub async fn check_execution_mode(
        &self,
        mode: ExecutionMode,
        violations: &[Violation],
    ) -> Result<ExecutionDecision, OrchestrationIntegrationError> {
        let critical_violations = violations.iter()
            .any(|v| matches!(v.code, ViolationCode::BudgetExceeded | ViolationCode::OutOfScope));

        let allowed = match mode {
            ExecutionMode::Strict => violations.is_empty(),
            ExecutionMode::Auto => !critical_violations,
            ExecutionMode::DryRun => true, // Always allowed
        };

        let reason = if allowed {
            "Execution allowed".to_string()
        } else {
            format!("Execution blocked due to {} violations", violations.len())
        };

        let recommended_mode = if violations.is_empty() {
            ExecutionMode::Strict
        } else if !critical_violations {
            ExecutionMode::Auto
        } else {
            ExecutionMode::DryRun
        };

        let required_waivers = violations.iter()
            .filter(|v| matches!(v.code, ViolationCode::MissingTests | ViolationCode::NonDeterministic))
            .map(|v| v.message.clone())
            .collect();

        Ok(ExecutionDecision {
            allowed,
            reason,
            recommended_mode,
            required_waivers,
        })
    }

    /// Generate waiver if eligible
    pub async fn generate_waiver_if_eligible(
        &self,
        task_id: &str,
        violations: Vec<String>,
        waiver_eligibility: &WaiverEligibility,
    ) -> Result<Option<WaiverResult>, OrchestrationIntegrationError> {
        if !waiver_eligibility.eligible {
            return Ok(None);
        }

        // Create a simple waiver result for now
        Ok(Some(WaiverResult {
            waiver_id: Some(Uuid::new_v4().to_string()),
            approved: false,
            reason: Some(waiver_eligibility.reason.clone()),
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
        }))
    }
}
