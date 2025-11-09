//! CAWS compliance checker for worker tasks
//!
//! Validates that worker tasks comply with CAWS (Coding Agent Workflow System) standards.

use schemars::JsonSchema;
use crate::worker_errors::WorkerError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use development_tools::validator::{CawsValidator, ValidationContext, DiffStats};
use development_tools::policy::CawsPolicy;
use tracing::{info, warn};

/// CAWS compliance check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CawsCheckResult {
    pub compliant: bool,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// CAWS compliance checker
pub struct CawsChecker {
    validator: Arc<CawsValidator>,
    policy: Arc<CawsPolicy>,
}

impl CawsChecker {
    pub fn new() -> Self {
        let policy = Arc::new(CawsPolicy::default());
        let validator = Arc::new(CawsValidator::new((*policy).clone()));
        
        Self {
            validator,
            policy,
        }
    }

    pub async fn check_compliance(&self, task: &str) -> Result<CawsCheckResult, WorkerError> {
        info!("Checking CAWS compliance for task");
        
        // Parse task specification (assumes JSON or YAML format)
        let task_spec: serde_json::Value = match serde_json::from_str(task) {
            Ok(spec) => spec,
            Err(_) => {
                // Try YAML parsing
                match serde_yaml::from_str::<serde_json::Value>(task) {
                    Ok(spec) => spec,
                    Err(e) => {
                        warn!("Failed to parse task specification: {}", e);
                        // Return basic validation result for unparseable tasks
                        return Ok(CawsCheckResult {
                            compliant: false,
                            violations: vec![format!("Invalid task specification format: {}", e)],
                            recommendations: vec!["Task specification must be valid JSON or YAML".to_string()],
                        });
                    }
                }
            }
        };
        
        // Extract task information
        let task_id = task_spec.get("id")
            .and_then(|v| v.as_str())
            .or_else(|| task_spec.get("task_id").and_then(|v| v.as_str()))
            .unwrap_or("unknown")
            .to_string();
        
        // Extract risk tier from task spec or default to medium
        let risk_tier = task_spec.get("risk_tier")
            .and_then(|v| v.as_str())
            .or_else(|| {
                task_spec.get("risk_tier")
                    .and_then(|v| v.as_u64())
                    .map(|n| match n {
                        1 => "high",
                        2 => "medium",
                        3 => "low",
                        _ => "medium",
                    })
            })
            .unwrap_or("medium")
            .to_string();
        
        // Extract scope and budget information from task spec
        let scope = task_spec.get("scope")
            .or_else(|| task_spec.get("scope_in"))
            .cloned()
            .unwrap_or(serde_json::json!([]));
        
        let change_budget = task_spec.get("change_budget")
            .or_else(|| task_spec.get("budget"))
            .cloned()
            .unwrap_or(serde_json::json!({}));
        
        // Calculate diff stats from task spec if available
        let diff_stats = DiffStats {
            files_changed: change_budget.get("max_files")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
                .unwrap_or(0),
            lines_added: change_budget.get("max_loc")
                .or_else(|| change_budget.get("max_lines"))
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
                .unwrap_or(0),
            lines_deleted: 0,
            files_modified: scope.as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default(),
        };
        
        // Build validation context
        let validation_context = ValidationContext {
            task_id: task_id.clone(),
            risk_tier: risk_tier.clone(),
            working_spec: task_spec.clone(),
            diff_stats,
            test_results: task_spec.get("test_results").cloned()
                .and_then(|v| serde_json::from_value(v).ok()),
            security_scan: task_spec.get("security_scan").cloned()
                .and_then(|v| serde_json::from_value(v).ok()),
        };
        
        // Perform validation
        let validation_result = self.validator.validate(validation_context).await;
        
        // Convert violations to string format
        let violations: Vec<String> = validation_result.violations
            .iter()
            .map(|v| {
                let mut msg = format!("[{}] {}", v.rule_id, v.message);
                if let Some(ref remediation) = v.remediation {
                    msg.push_str(&format!(" - {}", remediation));
                }
                if let Some(ref location) = v.location {
                    if let Some(ref file) = location.file {
                        msg.push_str(&format!(" ({}", file));
                        if let Some(line) = location.line {
                            msg.push_str(&format!(":{}", line));
                        }
                        msg.push(')');
                    }
                }
                msg
            })
            .collect();
        
        // Generate recommendations based on violations
        let mut recommendations = Vec::new();
        if validation_result.compliance_score < 0.8 {
            recommendations.push("Consider reviewing and addressing violations to improve compliance score".to_string());
        }
        if !violations.is_empty() {
            recommendations.push(format!("Address {} violation(s) to achieve full compliance", violations.len()));
        }
        if validation_result.compliance_score >= 0.8 && violations.is_empty() {
            recommendations.push("Task appears compliant with CAWS standards".to_string());
        }
        
        info!(
            "CAWS compliance check completed: compliant={}, score={:.2}, violations={}",
            validation_result.passed,
            validation_result.compliance_score,
            violations.len()
        );
        
        Ok(CawsCheckResult {
            compliant: validation_result.passed,
            violations,
            recommendations,
        })
    }
}
