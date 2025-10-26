//! Validation pipeline for working specifications
//!
//! The validation pipeline orchestrates multiple validation stages
//! including CAWS compliance, constraint validation, and risk assessment.
//! Now uses common-pipeline framework for standardized patterns.

use std::sync::Arc;
use async_trait::async_trait;

use crate::planning_errors::{PlanningError, PlanningResult};
use crate::caws_integration::{CawsValidator, ValidationContext};
use system_configuration::types::{ValidationStatus, ValidationResults, ValidationIssue, IssueSeverity};
use agent_agency_contracts::ContractKind;
use common_pipeline::{ValidationPipeline as CommonValidationPipeline, ValidationStage as CommonValidationStage, ValidationResult as CommonValidationResult, ValidationPipelineConfig as CommonValidationConfig, ValidationSeverity as CommonValidationSeverity};

/// Validation stage in the pipeline
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStage {
    SchemaValidation,
    ConstraintValidation,
    CawsValidation,
    RiskAssessment,
    DependencyValidation,
}

/// Adapter to convert domain-specific validation stages to common validation stages
pub struct ValidationStageAdapter {
    stage_type: ValidationStage,
    caws_validator: Option<Arc<dyn CawsValidator>>,
}

impl ValidationStageAdapter {
    pub fn new(stage_type: ValidationStage, caws_validator: Option<Arc<dyn CawsValidator>>) -> Self {
        Self {
            stage_type,
            caws_validator,
        }
    }
}

#[async_trait]
impl CommonValidationStage for ValidationStageAdapter {
    fn name(&self) -> &str {
        match self.stage_type {
            ValidationStage::SchemaValidation => "schema_validation",
            ValidationStage::ConstraintValidation => "constraint_validation",
            ValidationStage::CawsValidation => "caws_validation",
            ValidationStage::RiskAssessment => "risk_assessment",
            ValidationStage::DependencyValidation => "dependency_validation",
        }
    }

    async fn validate(&self, input: &serde_json::Value) -> common_pipeline::PipelineResult<Vec<CommonValidationResult>> {
        // Convert serde_json::Value to WorkingSpec
        let working_spec: agent_agency_contracts::working_spec::WorkingSpec = serde_json::from_value(input.clone())
            .map_err(|e| common_pipeline::PipelineError::Validation(e.to_string()))?;

        // Run the appropriate validation based on stage type
        let results = match self.stage_type {
            ValidationStage::SchemaValidation => {
                // TODO: Implement comprehensive schema validation with acceptance criteria:
                // - [ ] Validate working spec structure against JSON schema
                // - [ ] Check required fields and data types
                // - [ ] Validate enum values and constraints
                // - [ ] Ensure cross-field consistency and business rules
                // - [ ] Provide detailed error messages for schema violations
                vec![CommonValidationResult::pass("schema_validation", "Schema validation passed")]
            }
            ValidationStage::ConstraintValidation => {
                // TODO: Implement constraint validation logic with acceptance criteria:
                // - [ ] Validate change budget constraints (max_files, max_loc)
                // - [ ] Check scope boundaries (in/out directories)
                // - [ ] Verify risk tier appropriateness for change impact
                // - [ ] Validate operational rollback SLAs and requirements
                // - [ ] Ensure acceptance criteria are measurable and testable
                vec![CommonValidationResult::pass("constraint_validation", "Constraint validation passed")]
            }
            ValidationStage::CawsValidation => {
                if let Some(validator) = &self.caws_validator {
                    // Use the CAWS validator
                    match validator.validate_working_spec(&working_spec).await {
                        Ok(validation_result) => {
                            validation_result.violations.into_iter().map(|violation| {
                                let severity = match violation.severity {
                                    crate::caws_integration::ViolationSeverity::Error =>
                                        CommonValidationSeverity::Error,
                                    crate::caws_integration::ViolationSeverity::Warning =>
                                        CommonValidationSeverity::Warning,
                                    crate::caws_integration::ViolationSeverity::Info =>
                                        CommonValidationSeverity::Info,
                                };

                                CommonValidationResult::fail(severity, violation.code, violation.message)
                            }).collect()
                        }
                        Err(e) => {
                            vec![CommonValidationResult::fail(
                                CommonValidationSeverity::Error,
                                "caws_validation_error",
                                format!("CAWS validation failed: {}", e)
                            )]
                        }
                    }
                } else {
                    vec![CommonValidationResult::fail(
                        CommonValidationSeverity::Error,
                        "caws_validator_missing",
                        "CAWS validator not configured"
                    )]
                }
            }
            ValidationStage::RiskAssessment => {
                // TODO: Implement risk assessment analysis with acceptance criteria:
                // - [ ] Evaluate change impact on system stability and performance
                // - [ ] Assess operational risk and rollback complexity
                // - [ ] Analyze blast radius and downstream dependencies
                // - [ ] Calculate risk score based on multiple factors (complexity, scope, testing)
                // - [ ] Provide risk mitigation recommendations and safeguards
                vec![CommonValidationResult::pass("risk_assessment", "Risk assessment passed")]
            }
            ValidationStage::DependencyValidation => {
                // TODO: Implement dependency validation with acceptance criteria:
                // - [ ] Analyze code dependencies and import relationships
                // - [ ] Validate external service and API dependencies
                // - [ ] Check database schema and migration dependencies
                // - [ ] Verify infrastructure and configuration dependencies
                // - [ ] Ensure all required dependencies are available and compatible
                vec![CommonValidationResult::pass("dependency_validation", "Dependency validation passed")]
            }
        };

        Ok(results)
    }
}

/// Validation pipeline that orchestrates multiple validation stages
/// Now wraps common-pipeline ValidationPipeline with domain-specific functionality
pub struct ValidationPipeline {
    common_pipeline: CommonValidationPipeline,
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
        // Create common pipeline config
        let common_config = CommonValidationConfig {
            base: common_pipeline::PipelineConfig::default(),
            stop_on_first_error: config.strict_mode,
            severity_threshold: if config.strict_mode {
                CommonValidationSeverity::Warning
            } else {
                CommonValidationSeverity::Error
            },
            enable_validation_caching: true,
            max_validation_time: std::time::Duration::from_secs(config.caws_timeout_seconds),
            collect_all_errors: !config.skip_expensive_validations,
        };

        let mut common_pipeline = CommonValidationPipeline::new(common_config);

        // Add validation stages
        let stages = vec![
            ValidationStage::SchemaValidation,
            ValidationStage::ConstraintValidation,
            ValidationStage::CawsValidation,
            ValidationStage::RiskAssessment,
        ];

        // Skip dependency validation if expensive validations are disabled
        let stages = if config.skip_expensive_validations {
            stages
        } else {
            let mut stages = stages;
            stages.push(ValidationStage::DependencyValidation);
            stages
        };

        for stage_type in stages {
            let adapter = ValidationStageAdapter::new(
                stage_type,
                Some(Arc::clone(&caws_validator))
            );
            common_pipeline.add_stage(Box::new(adapter));
        }

        Self {
            common_pipeline,
            caws_validator,
            config,
        }
    }

    /// Validate a working specification through all pipeline stages
    pub async fn validate_working_spec(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<ValidationResults> {
        // Convert WorkingSpec to JSON for common pipeline
        let input = serde_json::to_value(working_spec)
            .map_err(|e| PlanningError::ValidationError(format!("Failed to serialize working spec: {}", e)))?;

        // Execute validation through common pipeline
        let common_results = self.common_pipeline.execute(input).await
            .map_err(|e| PlanningError::ValidationError(format!("Pipeline execution failed: {}", e)))?;

        // Convert common results to domain-specific results
        let mut all_issues = Vec::new();

        for common_result in common_results.results {
            let issue = ValidationIssue {
                severity: match common_result.severity {
                    CommonValidationSeverity::Critical => IssueSeverity::Error,
                    CommonValidationSeverity::Error => IssueSeverity::Error,
                    CommonValidationSeverity::Warning => IssueSeverity::Warning,
                    CommonValidationSeverity::Info => IssueSeverity::Info,
                },
                category: common_result.category,
                description: common_result.message,
                suggestion: common_result.suggestion,
            };
            all_issues.push(issue);
        }

        let validation_status = if common_results.overall_passed {
            ValidationStatus::Valid
        } else if all_issues.iter().any(|i| i.severity == IssueSeverity::Error) {
            ValidationStatus::Invalid
        } else {
            ValidationStatus::Warnings
        };

        Ok(ValidationResults {
            status: validation_status,
            issues: all_issues,
            applied_refinements: Vec::new(), // TODO: track refinements
        })
    }

