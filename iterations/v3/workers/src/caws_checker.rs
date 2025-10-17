//! CAWS Checker
//!
//! Provides CAWS compliance checking and validation for worker outputs.

use crate::types::*;
use crate::council::types::{TaskSpec, RiskTier};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// CAWS compliance checker for worker outputs
#[derive(Debug)]
pub struct CawsChecker {
    // TODO: Add CAWS rule engine
    // rules: CawsRuleEngine,
}

impl CawsChecker {
    /// Create a new CAWS checker
    pub fn new() -> Self {
        Self {
            // rules: CawsRuleEngine::new(),
        }
    }

    /// Check CAWS compliance for a task specification
    pub async fn validate_task_spec(&self, task_spec: &TaskSpec) -> Result<CawsValidationResult> {
        info!("Validating CAWS compliance for task: {}", task_spec.title);

        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Check budget compliance
        self.check_budget_compliance(task_spec, &mut violations, &mut warnings)?;

        // Check scope compliance
        self.check_scope_compliance(task_spec, &mut violations, &mut warnings)?;

        // Check acceptance criteria
        self.check_acceptance_criteria(task_spec, &mut violations, &mut suggestions)?;

        // Check risk tier appropriateness
        self.check_risk_tier_appropriateness(task_spec, &mut violations, &mut suggestions)?;

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(&violations, &warnings);
        let is_compliant = violations.is_empty();

        Ok(CawsValidationResult {
            is_compliant,
            compliance_score,
            violations,
            warnings,
            suggestions,
            validated_at: chrono::Utc::now(),
        })
    }

    /// Check CAWS compliance for worker output
    pub async fn validate_worker_output(&self, output: &WorkerOutput, task_spec: &TaskSpec) -> Result<CawsValidationResult> {
        info!("Validating CAWS compliance for worker output");

        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Check budget adherence
        self.check_budget_adherence(output, task_spec, &mut violations, &mut warnings)?;

        // Check quality standards
        self.check_quality_standards(output, task_spec, &mut violations, &mut warnings)?;

        // Check CAWS rule compliance
        self.check_caws_rules(output, task_spec, &mut violations, &mut suggestions)?;

        // Check provenance requirements
        self.check_provenance_requirements(output, &mut violations, &mut warnings)?;

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(&violations, &warnings);
        let is_compliant = violations.is_empty();

        Ok(CawsValidationResult {
            is_compliant,
            compliance_score,
            violations,
            warnings,
            suggestions,
            validated_at: chrono::Utc::now(),
        })
    }

    /// Check budget compliance
    fn check_budget_compliance(
        &self,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if budget limits are reasonable for the task
        if let Some(max_files) = task_spec.scope.max_files {
            if max_files == 0 {
                violations.push(CawsViolation {
                    rule: "Budget Definition".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: "Max files cannot be zero".to_string(),
                    location: Some("task_spec.scope.max_files".to_string()),
                    suggestion: Some("Set max_files to at least 1".to_string()),
                });
            } else if max_files > 50 {
                warnings.push(format!("Large file count limit: {} files", max_files));
            }
        }

        if let Some(max_loc) = task_spec.scope.max_loc {
            if max_loc == 0 {
                violations.push(CawsViolation {
                    rule: "Budget Definition".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: "Max LOC cannot be zero".to_string(),
                    location: Some("task_spec.scope.max_loc".to_string()),
                    suggestion: Some("Set max_loc to at least 1".to_string()),
                });
            } else if max_loc > 10000 {
                warnings.push(format!("Large LOC limit: {} lines", max_loc));
            }
        }

        Ok(())
    }

    /// Check scope compliance
    fn check_scope_compliance(
        &self,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if scope is well-defined
        if task_spec.scope.files_affected.is_empty() {
            warnings.push("No files specified in scope".to_string());
        }

        // Check if domains are specified
        if task_spec.scope.domains.is_empty() {
            warnings.push("No domains specified in scope".to_string());
        }

        // Check for overly broad scopes
        if task_spec.scope.files_affected.len() > 20 {
            warnings.push("Very broad file scope - consider narrowing".to_string());
        }

        Ok(())
    }

    /// Check acceptance criteria
    fn check_acceptance_criteria(
        &self,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        suggestions: &mut Vec<String>,
    ) -> Result<()> {
        // Check if acceptance criteria are defined
        if task_spec.acceptance_criteria.is_empty() {
            violations.push(CawsViolation {
                rule: "Acceptance Criteria".to_string(),
                severity: ViolationSeverity::High,
                description: "No acceptance criteria defined".to_string(),
                location: None,
                suggestion: Some("Define clear acceptance criteria for the task".to_string()),
            });
        } else {
            // Check quality of acceptance criteria
            for criterion in &task_spec.acceptance_criteria {
                if criterion.description.len() < 10 {
                    suggestions.push(format!(
                        "Acceptance criterion '{}' is too brief - consider adding more detail",
                        criterion.id
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check risk tier appropriateness
    fn check_risk_tier_appropriateness(
        &self,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        suggestions: &mut Vec<String>,
    ) -> Result<()> {
        // Check if risk tier matches task complexity
        let has_auth_keywords = task_spec.description.to_lowercase().contains("auth") ||
                               task_spec.description.to_lowercase().contains("login") ||
                               task_spec.description.to_lowercase().contains("password");
        
        let has_billing_keywords = task_spec.description.to_lowercase().contains("billing") ||
                                  task_spec.description.to_lowercase().contains("payment") ||
                                  task_spec.description.to_lowercase().contains("money");

        let has_database_keywords = task_spec.description.to_lowercase().contains("database") ||
                                   task_spec.description.to_lowercase().contains("migration") ||
                                   task_spec.description.to_lowercase().contains("schema");

        let is_high_risk_content = has_auth_keywords || has_billing_keywords || has_database_keywords;

        match task_spec.risk_tier {
            RiskTier::Tier1 => {
                // Tier 1 should be for critical systems
                if !is_high_risk_content {
                    suggestions.push("Task may not require Tier 1 risk level - consider Tier 2".to_string());
                }
            }
            RiskTier::Tier2 => {
                // Tier 2 is appropriate for most features
                // No specific checks needed
            }
            RiskTier::Tier3 => {
                // Tier 3 should be for low-risk changes
                if is_high_risk_content {
                    violations.push(CawsViolation {
                        rule: "Risk Tier Classification".to_string(),
                        severity: ViolationSeverity::High,
                        description: "High-risk content detected but task marked as Tier 3".to_string(),
                        location: None,
                        suggestion: Some("Consider upgrading to Tier 1 or Tier 2".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check budget adherence in worker output
    fn check_budget_adherence(
        &self,
        output: &WorkerOutput,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        let files_used = output.files_modified.len() as u32;
        let loc_used: u32 = output.files_modified.iter()
            .map(|f| f.content.as_ref().map(|c| c.lines().count() as u32).unwrap_or(0))
            .sum();

        // Check file count
        if let Some(max_files) = task_spec.scope.max_files {
            if files_used > max_files {
                violations.push(CawsViolation {
                    rule: "File Count Budget".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: format!("Used {} files, exceeds limit of {}", files_used, max_files),
                    location: None,
                    suggestion: Some("Reduce file count or request budget increase".to_string()),
                });
            } else if files_used as f32 / max_files as f32 > 0.8 {
                warnings.push(format!("File count near limit: {}/{}", files_used, max_files));
            }
        }

        // Check LOC count
        if let Some(max_loc) = task_spec.scope.max_loc {
            if loc_used > max_loc {
                violations.push(CawsViolation {
                    rule: "LOC Budget".to_string(),
                    severity: ViolationSeverity::Critical,
                    description: format!("Used {} LOC, exceeds limit of {}", loc_used, max_loc),
                    location: None,
                    suggestion: Some("Reduce LOC or request budget increase".to_string()),
                });
            } else if loc_used as f32 / max_loc as f32 > 0.8 {
                warnings.push(format!("LOC near limit: {}/{}", loc_used, max_loc));
            }
        }

        Ok(())
    }

    /// Check quality standards
    fn check_quality_standards(
        &self,
        output: &WorkerOutput,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check self-assessment quality
        if output.self_assessment.quality_score < 0.7 {
            violations.push(CawsViolation {
                rule: "Quality Standards".to_string(),
                severity: ViolationSeverity::High,
                description: "Worker self-assessment indicates low quality".to_string(),
                location: None,
                suggestion: Some("Review and improve output quality".to_string()),
            });
        }

        // Check confidence level
        if output.self_assessment.confidence < 0.5 {
            warnings.push("Worker has low confidence in output".to_string());
        }

        // Check for concerns
        if !output.self_assessment.concerns.is_empty() {
            warnings.push(format!("Worker raised {} concerns about the output", 
                output.self_assessment.concerns.len()));
        }

        // Check rationale quality
        if output.rationale.len() < 50 {
            violations.push(CawsViolation {
                rule: "Rationale Quality".to_string(),
                severity: ViolationSeverity::Medium,
                description: "Rationale is too brief".to_string(),
                location: None,
                suggestion: Some("Provide more detailed rationale for decisions".to_string()),
            });
        }

        Ok(())
    }

    /// Check CAWS rules compliance
    fn check_caws_rules(
        &self,
        output: &WorkerOutput,
        task_spec: &TaskSpec,
        violations: &mut Vec<CawsViolation>,
        suggestions: &mut Vec<String>,
    ) -> Result<()> {
        // Check CAWS compliance score from self-assessment
        if output.self_assessment.caws_compliance < 0.8 {
            violations.push(CawsViolation {
                rule: "CAWS Compliance".to_string(),
                severity: ViolationSeverity::High,
                description: "Worker self-assessment indicates CAWS compliance issues".to_string(),
                location: None,
                suggestion: Some("Review CAWS requirements and improve compliance".to_string()),
            });
        }

        // Check for hardcoded values in code
        for file_mod in &output.files_modified {
            if let Some(content) = &file_mod.content {
                if content.contains("localhost") || content.contains("127.0.0.1") {
                    warnings.push(format!("Hardcoded localhost found in {}", file_mod.path));
                }
                
                if content.contains("password") || content.contains("secret") {
                    violations.push(CawsViolation {
                        rule: "Security Best Practices".to_string(),
                        severity: ViolationSeverity::Critical,
                        description: "Potential hardcoded secrets detected".to_string(),
                        location: Some(file_mod.path.clone()),
                        suggestion: Some("Use environment variables or secure configuration".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check provenance requirements
    fn check_provenance_requirements(
        &self,
        output: &WorkerOutput,
        violations: &mut Vec<CawsViolation>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if rationale is provided
        if output.rationale.is_empty() {
            violations.push(CawsViolation {
                rule: "Provenance Requirements".to_string(),
                severity: ViolationSeverity::High,
                description: "No rationale provided for decisions".to_string(),
                location: None,
                suggestion: Some("Provide detailed rationale for all decisions".to_string()),
            });
        }

        // Check if self-assessment is complete
        if output.self_assessment.concerns.is_empty() && output.self_assessment.improvements.is_empty() {
            warnings.push("Self-assessment appears incomplete - no concerns or improvements identified".to_string());
        }

        // Check if file modifications are documented
        for file_mod in &output.files_modified {
            match file_mod.operation {
                FileOperation::Create => {
                    if file_mod.content.is_none() {
                        violations.push(CawsViolation {
                            rule: "File Modification Documentation".to_string(),
                            severity: ViolationSeverity::Medium,
                            description: "File creation without content provided".to_string(),
                            location: Some(file_mod.path.clone()),
                            suggestion: Some("Provide file content for creation operations".to_string()),
                        });
                    }
                }
                FileOperation::Modify => {
                    if file_mod.diff.is_none() && file_mod.content.is_none() {
                        violations.push(CawsViolation {
                            rule: "File Modification Documentation".to_string(),
                            severity: ViolationSeverity::Medium,
                            description: "File modification without diff or content provided".to_string(),
                            location: Some(file_mod.path.clone()),
                            suggestion: Some("Provide diff or content for modification operations".to_string()),
                        });
                    }
                }
                FileOperation::Delete => {
                    // Deletion operations don't require content
                }
                FileOperation::Move { .. } => {
                    if file_mod.diff.is_none() {
                        warnings.push(format!("File move operation without diff: {}", file_mod.path));
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculate compliance score
    fn calculate_compliance_score(&self, violations: &[CawsViolation], warnings: &[String]) -> f32 {
        let mut score = 1.0;

        // Deduct points for violations
        for violation in violations {
            let deduction = match violation.severity {
                ViolationSeverity::Critical => 0.3,
                ViolationSeverity::High => 0.2,
                ViolationSeverity::Medium => 0.1,
                ViolationSeverity::Low => 0.05,
            };
            score -= deduction;
        }

        // Deduct smaller points for warnings
        for _warning in warnings {
            score -= 0.02;
        }

        score.max(0.0)
    }

    /// Get CAWS rule violations for a task
    pub async fn get_violations(&self, task_id: Uuid) -> Result<Vec<CawsViolation>> {
        // TODO: Implement database lookup for violations
        // For now, return empty list
        Ok(Vec::new())
    }

    /// Check if a waiver is valid
    pub async fn validate_waiver(&self, waiver: &CawsWaiver) -> Result<bool> {
        // Check if waiver has valid justification
        if waiver.justification.len() < 20 {
            return Ok(false);
        }

        // Check if waiver is time-bounded
        if waiver.time_bounded {
            if let Some(expires_at) = waiver.expires_at {
                if expires_at < chrono::Utc::now() {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Default for CawsChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// CAWS waiver (simplified)
#[derive(Debug, Clone)]
pub struct CawsWaiver {
    pub id: String,
    pub reason: String,
    pub justification: String,
    pub time_bounded: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// CAWS validation result
#[derive(Debug, Clone)]
pub struct CawsValidationResult {
    pub is_compliant: bool,
    pub compliance_score: f32,
    pub violations: Vec<CawsViolation>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_caws_checker_creation() {
        let checker = CawsChecker::new();
        // Basic creation test
        assert!(true);
    }

    #[tokio::test]
    async fn test_validate_task_spec() {
        let checker = CawsChecker::new();
        
        let task_spec = TaskSpec {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "A test task description".to_string(),
            risk_tier: RiskTier::Tier2,
            scope: TaskScope {
                files_affected: vec!["src/test.rs".to_string()],
                max_files: Some(5),
                max_loc: Some(1000),
                domains: vec!["backend".to_string()],
            },
            acceptance_criteria: vec![],
            context: TaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            worker_output: WorkerOutput {
                content: "".to_string(),
                files_modified: vec![],
                rationale: "".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.0,
                    quality_score: 0.0,
                    confidence: 0.0,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: std::collections::HashMap::new(),
            },
            caws_spec: None,
        };

        let result = checker.validate_task_spec(&task_spec).await.unwrap();
        assert!(!result.is_compliant); // Should fail due to no acceptance criteria
        assert!(result.compliance_score < 1.0);
        assert!(!result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_validate_worker_output() {
        let checker = CawsChecker::new();
        
        let task_spec = TaskSpec {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "A test task description".to_string(),
            risk_tier: RiskTier::Tier2,
            scope: TaskScope {
                files_affected: vec!["src/test.rs".to_string()],
                max_files: Some(2),
                max_loc: Some(100),
                domains: vec!["backend".to_string()],
            },
            acceptance_criteria: vec![],
            context: TaskContext {
                workspace_root: "/workspace".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            worker_output: WorkerOutput {
                content: "Test implementation".to_string(),
                files_modified: vec![
                    FileModification {
                        path: "test1.rs".to_string(),
                        operation: FileOperation::Create,
                        content: Some("fn main() {\n    println!(\"test\");\n}".to_string()),
                        diff: None,
                        size_bytes: 50,
                    },
                    FileModification {
                        path: "test2.rs".to_string(),
                        operation: FileOperation::Create,
                        content: Some("fn helper() {\n    // helper function\n}".to_string()),
                        diff: None,
                        size_bytes: 40,
                    },
                    FileModification {
                        path: "test3.rs".to_string(),
                        operation: FileOperation::Create,
                        content: Some("fn extra() {\n    // extra function\n}".to_string()),
                        diff: None,
                        size_bytes: 40,
                    },
                ],
                rationale: "Created three files for the implementation".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.95,
                    quality_score: 0.9,
                    confidence: 0.85,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: std::collections::HashMap::new(),
            },
            caws_spec: None,
        };

        let result = checker.validate_worker_output(&task_spec.worker_output, &task_spec).await.unwrap();
        assert!(!result.is_compliant); // Should fail due to file count exceeding limit
        assert!(result.compliance_score < 1.0);
        assert!(!result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_validate_waiver() {
        let checker = CawsChecker::new();
        
        let valid_waiver = CawsWaiver {
            id: "waiver-001".to_string(),
            reason: "Technical complexity".to_string(),
            justification: "This is a detailed justification for why the waiver is needed".to_string(),
            time_bounded: true,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(30)),
        };

        let invalid_waiver = CawsWaiver {
            id: "waiver-002".to_string(),
            reason: "Technical complexity".to_string(),
            justification: "Short".to_string(), // Too short
            time_bounded: true,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(30)),
        };

        assert!(checker.validate_waiver(&valid_waiver).await.unwrap());
        assert!(!checker.validate_waiver(&invalid_waiver).await.unwrap());
    }

    #[test]
    fn test_calculate_compliance_score() {
        let checker = CawsChecker::new();
        
        let violations = vec![
            CawsViolation {
                rule: "Test Rule".to_string(),
                severity: ViolationSeverity::High,
                description: "Test violation".to_string(),
                location: None,
                suggestion: None,
            }
        ];
        
        let warnings = vec!["Test warning".to_string()];

        let score = checker.calculate_compliance_score(&violations, &warnings);
        assert!(score < 1.0);
        assert!(score >= 0.0);
    }
}
