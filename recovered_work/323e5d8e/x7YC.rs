//! Validation pipeline for working specifications
//!
//! The validation pipeline orchestrates multiple validation stages
//! including CAWS compliance, constraint validation, and risk assessment.

use std::sync::Arc;
use async_trait::async_trait;

use crate::error::PlanningResult;
use crate::caws_integration::{CawsValidator, ValidationContext};
use crate::planner::{ValidationStatus, ValidationResults, ValidationIssue, IssueSeverity};
use crate::planner::PlanningConfig;

/// Validation stage in the pipeline
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStage {
    SchemaValidation,
    ConstraintValidation,
    CawsValidation,
    RiskAssessment,
    DependencyValidation,
}

/// Validation pipeline that orchestrates multiple validation stages
pub struct ValidationPipeline {
    caws_validator: Arc<dyn CawsValidator>,
    config: ValidationPipelineConfig,
}

/// Configuration for the validation pipeline
#[derive(Debug, Clone)]
pub struct ValidationPipelineConfig {
    /// Whether to run in strict mode (fail on warnings)
    pub strict_mode: bool,

    /// Maximum time for CAWS validation (seconds)
    pub caws_timeout_seconds: u64,

    /// Whether to skip expensive validations
    pub skip_expensive_validations: bool,
}

impl Default for ValidationPipelineConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            caws_timeout_seconds: 60,
            skip_expensive_validations: false,
        }
    }
}

impl ValidationPipeline {
    /// Create a new validation pipeline
    pub fn new(
        caws_validator: Arc<dyn CawsValidator>,
        config: ValidationPipelineConfig,
    ) -> Self {
        Self {
            caws_validator,
            config,
        }
    }

    /// Validate a working specification through all pipeline stages
    pub async fn validate_working_spec(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<ValidationResults> {
        let mut all_issues = Vec::new();
        let mut applied_refinements = Vec::new();

        // Stage 1: Schema validation
        let schema_issues = self.validate_schema(working_spec).await?;
        all_issues.extend(schema_issues);

        // Stage 2: Constraint validation
        let constraint_issues = self.validate_constraints(working_spec).await?;
        all_issues.extend(constraint_issues);

        // Stage 3: CAWS validation (most expensive)
        let caws_result = self.validate_caws(working_spec).await?;
        all_issues.extend(caws_result.violations.into_iter().map(|v| ValidationIssue {
            severity: match v.severity {
                crate::caws_integration::ViolationSeverity::Error => IssueSeverity::Error,
                crate::caws_integration::ViolationSeverity::Warning => IssueSeverity::Warning,
                crate::caws_integration::ViolationSeverity::Info => IssueSeverity::Info,
            },
            category: v.code,
            description: v.message,
            suggestion: v.suggestion,
        }));

        // Stage 4: Risk assessment
        let risk_issues = self.validate_risk_assessment(working_spec).await?;
        all_issues.extend(risk_issues);

        // Stage 5: Dependency validation (if not skipped)
        if !self.config.skip_expensive_validations {
            let dependency_issues = self.validate_dependencies(working_spec).await?;
            all_issues.extend(dependency_issues);
        }

        // Determine overall status
        let overall_status = self.determine_overall_status(&all_issues);

        // Calculate overall compliance score
        let caws_compliance_score = caws_result.compliance_score;

        Ok(ValidationResults {
            overall_status,
            caws_compliance_score,
            issues: all_issues,
            applied_refinements,
        })
    }

    /// Validate working spec against JSON schema
    async fn validate_schema(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<Vec<ValidationIssue>> {
        // Convert to JSON value for schema validation
        let json_value = serde_json::to_value(working_spec)
            .map_err(|e| crate::error::PlanningError::Serialization(
                crate::error::ContractKind::WorkingSpec,
                e
            ))?;

        // Validate against schema
        let result = agent_agency_contracts::validate_working_spec_value(&json_value);

        match result {
            Ok(()) => Ok(Vec::new()),
            Err(agent_agency_contracts::ContractError::Validation { issues, .. }) => {
                Ok(issues.into_iter().map(|issue| ValidationIssue {
                    severity: match issue.message.to_lowercase() {
                        m if m.contains("required") => IssueSeverity::Error,
                        m if m.contains("invalid") => IssueSeverity::Error,
                        _ => IssueSeverity::Warning,
                    },
                    category: "schema".to_string(),
                    description: issue.message,
                    suggestion: Some(format!("Fix schema validation error at {}", issue.instance_path)),
                }).collect())
            }
            Err(_) => Ok(vec![ValidationIssue {
                severity: IssueSeverity::Error,
                category: "schema".to_string(),
                description: "Schema validation failed".to_string(),
                suggestion: Some("Check working spec structure against schema".to_string()),
            }]),
        }
    }

    /// Validate working spec constraints
    async fn validate_constraints(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Validate budget constraints
        if let Some(budget) = &working_spec.constraints.budget_limits {
            if let Some(max_files) = budget.max_files {
                if max_files == 0 {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        category: "constraints".to_string(),
                        description: "Maximum files limit cannot be zero".to_string(),
                        suggestion: Some("Set max_files to a positive value".to_string()),
                    });
                }
            }

            if let Some(max_loc) = budget.max_loc {
                if max_loc == 0 {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        category: "constraints".to_string(),
                        description: "Maximum LOC limit cannot be zero".to_string(),
                        suggestion: Some("Set max_loc to a positive value".to_string()),
                    });
                }
            }
        }

        // Validate scope restrictions
        if let Some(scope) = &working_spec.constraints.scope_restrictions {
            // Check for conflicting paths
            for allowed in &scope.allowed_paths {
                if scope.blocked_paths.contains(allowed) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        category: "constraints".to_string(),
                        description: format!("Path '{}' is both allowed and blocked", allowed),
                        suggestion: Some("Remove path from one of the lists".to_string()),
                    });
                }
            }
        }

        // Validate acceptance criteria format
        for criterion in &working_spec.acceptance_criteria {
            if !criterion.id.starts_with('A') || !criterion.id[1..].chars().all(|c| c.is_ascii_digit()) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: "acceptance_criteria".to_string(),
                    description: format!("Acceptance criterion ID '{}' should follow format 'A<number>'", criterion.id),
                    suggestion: Some("Use format like 'A1', 'A2', etc.".to_string()),
                });
            }
        }

        Ok(issues)
    }

    /// Run CAWS validation
    async fn validate_caws(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<crate::caws_integration::CawsValidationResult> {
        use tokio::time::{timeout, Duration};

        let context = ValidationContext {
            risk_tier: match working_spec.risk_tier {
                1 => agent_agency_contracts::task_request::RiskTier::Tier1,
                2 => agent_agency_contracts::task_request::RiskTier::Tier2,
                3 => agent_agency_contracts::task_request::RiskTier::Tier3,
                _ => agent_agency_contracts::task_request::RiskTier::Tier2,
            },
            environment: working_spec.context.environment.clone().into(),
            options: crate::caws_integration::ValidationOptions {
                strict_mode: self.config.strict_mode,
                include_suggestions: true,
                skip_expensive: self.config.skip_expensive_validations,
            },
        };

        let result = timeout(
            Duration::from_secs(self.config.caws_timeout_seconds),
            self.caws_validator.validate_working_spec(working_spec, &context)
        ).await;

        match result {
            Ok(Ok(caws_result)) => Ok(caws_result),
            Ok(Err(e)) => Err(crate::error::PlanningError::CawsValidation(e)),
            Err(_) => Err(crate::error::PlanningError::ValidationPipeline {
                stage: "caws_validation".to_string(),
                error: format!("CAWS validation timed out after {} seconds", self.config.caws_timeout_seconds),
            }),
        }
    }

    /// Validate risk assessment
    async fn validate_risk_assessment(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Risk tier specific validations
        match working_spec.risk_tier {
            1 => {
                // T1 validations
                if working_spec.acceptance_criteria.len() < 3 {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        category: "risk_assessment".to_string(),
                        description: "T1 tasks require at least 3 acceptance criteria".to_string(),
                        suggestion: Some("Add more detailed acceptance criteria".to_string()),
                    });
                }

                if working_spec.test_plan.unit_tests.is_empty() {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        category: "risk_assessment".to_string(),
                        description: "T1 tasks require unit tests".to_string(),
                        suggestion: Some("Define unit test specifications".to_string()),
                    });
                }
            }
            2 => {
                // T2 validations
                if working_spec.acceptance_criteria.len() < 2 {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        category: "risk_assessment".to_string(),
                        description: "T2 tasks should have at least 2 acceptance criteria".to_string(),
                        suggestion: Some("Consider adding more acceptance criteria".to_string()),
                    });
                }
            }
            _ => {
                // T3 and other tiers - minimal requirements
                if working_spec.acceptance_criteria.is_empty() {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        category: "risk_assessment".to_string(),
                        description: "Tasks should have acceptance criteria".to_string(),
                        suggestion: Some("Define acceptance criteria for the task".to_string()),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Validate dependencies and external requirements
    async fn validate_dependencies(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for dependency conflicts
        let mut seen_deps = std::collections::HashMap::new();
        for (name, version) in &working_spec.context.dependencies {
            if let Some(existing_version) = seen_deps.get(name) {
                if existing_version != version {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        category: "dependencies".to_string(),
                        description: format!("Dependency '{}' has conflicting versions", name),
                        suggestion: Some("Resolve version conflicts".to_string()),
                    });
                }
            } else {
                seen_deps.insert(name.clone(), version.clone());
            }
        }

        // Validate dependency formats (basic check)
        for (name, version) in &working_spec.context.dependencies {
            if version.trim().is_empty() {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: "dependencies".to_string(),
                    description: format!("Dependency '{}' has empty version", name),
                    suggestion: Some("Specify a valid version constraint".to_string()),
                });
            }
        }

        Ok(issues)
    }

    /// Determine overall validation status from all issues
    fn determine_overall_status(&self, issues: &[ValidationIssue]) -> ValidationStatus {
        let has_errors = issues.iter().any(|i| i.severity == IssueSeverity::Error);
        let has_warnings = issues.iter().any(|i| i.severity == IssueSeverity::Warning);

        if has_errors {
            ValidationStatus::Failed
        } else if has_warnings && self.config.strict_mode {
            ValidationStatus::Failed
        } else if has_warnings {
            ValidationStatus::PassedWithRefinements
        } else {
            ValidationStatus::Passed
        }
    }
}

/// Create a default validation pipeline
pub fn create_validation_pipeline(
    caws_validator: Arc<dyn CawsValidator>,
) -> Arc<ValidationPipeline> {
    Arc::new(ValidationPipeline::new(
        caws_validator,
        ValidationPipelineConfig::default(),
    ))
}
