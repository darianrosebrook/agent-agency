//! Validation pipeline for working specifications
//!
//! The validation pipeline orchestrates multiple validation stages
//! including CAWS compliance, constraint validation, and risk assessment.
//! Uses system-configuration validation pipeline for standardized patterns.

use schemars::JsonSchema;
use std::sync::Arc;
use async_trait::async_trait;

use crate::planning_agent::planning_errors::{PlanningError, PlanningResult};
use crate::planning_agent::planning_caws_integration::{CawsValidator, ValidationContext};
use system_configuration::validation::{ValidationPipeline as SystemValidationPipeline, ValidationStage as SystemValidationStage, ValidationResult, ValidationSeverity, ValidationResults};
use system_configuration::config::{PipelineConfig, ValidationPipelineConfig as SystemValidationPipelineConfig};
use system_configuration::types::{ValidationStatus, ValidationIssue, IssueSeverity};
use system_configuration::error::{PipelineResult, PipelineError};
use agent_agency_contracts::ContractKind;

/// Validation stage in the pipeline

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ValidationStage {
    SchemaValidation,
    ConstraintValidation,
    CawsValidation,
    RiskAssessment,
    DependencyValidation,
}

/// Adapter to convert domain-specific validation stages to system-configuration validation stages

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ValidationStageAdapter {
    stage_type: ValidationStage,
    #[serde(skip)]
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
impl SystemValidationStage for ValidationStageAdapter {
    fn name(&self) -> &str {
        match self.stage_type {
            ValidationStage::SchemaValidation => "schema_validation",
            ValidationStage::ConstraintValidation => "constraint_validation",
            ValidationStage::CawsValidation => "caws_validation",
            ValidationStage::RiskAssessment => "risk_assessment",
            ValidationStage::DependencyValidation => "dependency_validation",
        }
    }

    async fn validate(&self, input: &serde_json::Value) -> PipelineResult<Vec<ValidationResult>> {
        // Convert serde_json::Value to WorkingSpec
        let working_spec: agent_agency_contracts::working_spec::WorkingSpec = serde_json::from_value(input.clone())
            .map_err(|e| PipelineError::Validation(e.to_string()))?;

        // Run the appropriate validation based on stage type
        let results = match self.stage_type {
            ValidationStage::SchemaValidation => {
                self.validate_schema(&working_spec)
            }
            ValidationStage::ConstraintValidation => {
                self.validate_constraints(&working_spec)
            }
            ValidationStage::CawsValidation => {
                if let Some(validator) = &self.caws_validator {
                    // Use the CAWS validator
                    match validator.validate_working_spec(&working_spec, &ValidationContext {
                        risk_tier: agent_agency_contracts::task_request::RiskTier::from(working_spec.risk_tier),
                        environment: agent_agency_contracts::task_request::Environment::Development,
                        options: Default::default(),
                    }).await {
                        Ok(validation_result) => {
                            validation_result.violations.into_iter().map(|violation| {
                                let severity = match violation.severity {
                                    crate::planning_agent::planning_caws_integration::ViolationSeverity::Error =>
                                        ValidationSeverity::Error,
                                    crate::planning_agent::planning_caws_integration::ViolationSeverity::Warning =>
                                        ValidationSeverity::Warning,
                                    crate::planning_agent::planning_caws_integration::ViolationSeverity::Info =>
                                        ValidationSeverity::Info,
                                };

                                ValidationResult::fail(severity, violation.code, violation.message)
                            }).collect()
                        }
                        Err(e) => {
                            vec![ValidationResult::fail(
                                ValidationSeverity::Error,
                                "caws_validation_error",
                                format!("CAWS validation failed: {}", e)
                            )]
                        }
                    }
                } else {
                    vec![ValidationResult::fail(
                        ValidationSeverity::Error,
                        "caws_validator_missing",
                        "CAWS validator not configured"
                    )]
                }
            }
            ValidationStage::RiskAssessment => {
                self.assess_risk(&working_spec)
            }
            ValidationStage::DependencyValidation => {
                self.validate_dependencies(&working_spec)
            }
        };

        Ok(results)
    }
}

// Helper methods for ValidationStageAdapter (not part of trait)
impl ValidationStageAdapter {
    /// Validate working spec schema structure
    fn validate_schema(&self, spec: &agent_agency_contracts::working_spec::WorkingSpec) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Check required fields
        if spec.id.is_empty() {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "missing_id",
                "Working spec must have a non-empty ID"
            ));
        }

        if spec.title.is_empty() {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "missing_title",
                "Working spec must have a non-empty title"
            ));
        }

        // Validate ID format (should be PREFIX-NUMBER)
        if !spec.id.is_empty() && !self.is_valid_id_format(&spec.id) {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "invalid_id_format",
                "ID must follow format PREFIX-NUMBER (e.g., FEAT-001, FIX-042)"
            ));
        }

        // Validate risk tier
        if spec.risk_tier < 1 || spec.risk_tier > 3 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "invalid_risk_tier",
                "Risk tier must be between 1 and 3"
            ));
        }

        // Validate change budget
        if spec.change_budget.max_files == 0 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "invalid_max_files",
                "Max files must be greater than 0"
            ));
        }

        if spec.change_budget.max_loc == 0 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "invalid_max_loc",
                "Max lines of code must be greater than 0"
            ));
        }

        // Validate scope
        if spec.scope.is_empty() || spec.scope.iter().all(|s| s.allowed_paths.is_empty()) {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "empty_scope_in",
                "Scope must include at least one input directory"
            ));
        }

        // Validate acceptance criteria
        if spec.acceptance_criteria.is_empty() {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "missing_acceptance_criteria",
                "Working spec must have at least one acceptance criterion"
            ));
        }

        // Validate each acceptance criterion
        for (i, criterion) in spec.acceptance_criteria.iter().enumerate() {
            if criterion.given.is_empty() {
                results.push(ValidationResult::fail(
                    ValidationSeverity::Error,
                    format!("acceptance_criterion_{}_missing_given", i),
                    format!("Acceptance criterion {} must have a 'given' condition", i + 1)
                ));
            }

            if criterion.when.is_empty() {
                results.push(ValidationResult::fail(
                    ValidationSeverity::Error,
                    format!("acceptance_criterion_{}_missing_when", i),
                    format!("Acceptance criterion {} must have a 'when' action", i + 1)
                ));
            }

            if criterion.then.is_empty() {
                results.push(ValidationResult::fail(
                    ValidationSeverity::Error,
                    format!("acceptance_criterion_{}_missing_then", i),
                    format!("Acceptance criterion {} must have a 'then' outcome", i + 1)
                ));
            }
        }

        if results.is_empty() {
            results.push(ValidationResult::pass("schema_validation", "Schema validation passed"));
        }

        results
    }

    /// Validate constraints and business rules
    fn validate_constraints(&self, spec: &agent_agency_contracts::working_spec::WorkingSpec) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Collect all allowed paths from scope restrictions
        let all_allowed_paths: Vec<String> = spec.scope.iter().flat_map(|s| s.allowed_paths.clone()).collect();

        // Validate change budget constraints
        if spec.change_budget.max_files > 100 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Warning,
                "large_change_budget",
                "Change budget exceeds recommended maximum of 100 files"
            ));
        }

        if spec.change_budget.max_loc > 10000 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Warning,
                "large_loc_budget",
                "Lines of code budget exceeds recommended maximum of 10,000"
            ));
        }

        // Validate scope boundaries
        for dir in all_allowed_paths.iter() {
            if dir.contains("node_modules") || dir.contains(".git") || dir.contains("target/") {
                results.push(ValidationResult::fail(
                    ValidationSeverity::Error,
                    "invalid_scope_directory",
                    format!("Scope directory '{}' should not include build artifacts or dependencies", dir)
                ));
            }
        }

        // Validate risk tier appropriateness
        let is_high_risk_change = spec.change_budget.max_files > 50 || 
                                 spec.change_budget.max_loc > 5000 ||
                                 all_allowed_paths.len() > 10;

        if is_high_risk_change && spec.risk_tier < 2 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Warning,
                "risk_tier_mismatch",
                "High-impact changes should use risk tier 2 or higher"
            ));
        }

        // Validate operational rollback SLAs
        if spec.operational_rollback_slo.is_empty() {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "missing_rollback_slo",
                "Operational rollback SLO must be specified"
            ));
        }

        // Validate acceptance criteria are measurable
        for (i, criterion) in spec.acceptance_criteria.iter().enumerate() {
            if !self.is_measurable_criterion(criterion) {
                results.push(ValidationResult::fail(
                    ValidationSeverity::Warning,
                    format!("acceptance_criterion_{}_not_measurable", i),
                    format!("Acceptance criterion {} should be more specific and measurable", i + 1)
                ));
            }
        }

        if results.is_empty() {
            results.push(ValidationResult::pass("constraint_validation", "Constraint validation passed"));
        }

        results
    }

    /// Assess risk factors
    fn assess_risk(&self, spec: &agent_agency_contracts::working_spec::WorkingSpec) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        let mut risk_score = 0.0;
        
        // Collect all allowed paths from scope restrictions
        let all_allowed_paths: Vec<String> = spec.scope.iter().flat_map(|s| s.allowed_paths.clone()).collect();

        // Calculate complexity risk
        let complexity_risk = if spec.change_budget.max_files > 25 { 0.3 } else { 0.1 };
        risk_score += complexity_risk;

        // Calculate scope risk
        let scope_risk = if all_allowed_paths.len() > 5 { 0.2 } else { 0.05 };
        risk_score += scope_risk;

        // Calculate testing risk
        let testing_risk = if spec.acceptance_criteria.len() < 3 { 0.2 } else { 0.05 };
        risk_score += testing_risk;

        // Calculate rollback risk
        let rollback_risk = if spec.operational_rollback_slo.contains("5m") { 0.1 }
                           else if spec.operational_rollback_slo.contains("1h") { 0.2 }
                           else { 0.3 };
        risk_score += rollback_risk;

        // Assess risk level
        if risk_score > 0.7 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Error,
                "high_risk_change",
                format!("High risk change detected (score: {:.2}). Consider breaking into smaller changes.", risk_score)
            ));
        } else if risk_score > 0.5 {
            results.push(ValidationResult::fail(
                ValidationSeverity::Warning,
                "medium_risk_change",
                format!("Medium risk change detected (score: {:.2}). Ensure adequate testing and monitoring.", risk_score)
            ));
        }

        // Provide risk mitigation recommendations
        if risk_score > 0.4 {
            let mut recommendations = Vec::new();
            
            if complexity_risk > 0.2 {
                recommendations.push("Consider breaking into smaller, focused changes");
            }
            if scope_risk > 0.1 {
                recommendations.push("Limit scope to fewer directories");
            }
            if testing_risk > 0.1 {
                recommendations.push("Add more comprehensive acceptance criteria");
            }
            if rollback_risk > 0.2 {
                recommendations.push("Improve rollback procedures and reduce rollback time");
            }

            if !recommendations.is_empty() {
                results.push(ValidationResult::fail(
                    ValidationSeverity::Info,
                    "risk_mitigation_recommendations",
                    format!("Risk mitigation: {}", recommendations.join("; "))
                ));
            }
        }

        if results.is_empty() {
            results.push(ValidationResult::pass("risk_assessment", "Risk assessment passed"));
        }

        results
    }

    /// Validate dependencies
    fn validate_dependencies(&self, spec: &agent_agency_contracts::working_spec::WorkingSpec) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // Collect all allowed paths from scope restrictions
        let all_allowed_paths: Vec<String> = spec.scope.iter().flat_map(|s| s.allowed_paths.clone()).collect();

        // Check for external service dependencies
        let external_services = self.extract_external_dependencies(spec);
        for service in external_services {
            results.push(ValidationResult::fail(
                ValidationSeverity::Info,
                "external_dependency",
                format!("External dependency detected: {}. Ensure service availability.", service)
            ));
        }

        // Check for database dependencies
        if all_allowed_paths.iter().any(|dir| dir.contains("migration") || dir.contains("schema")) {
            results.push(ValidationResult::fail(
                ValidationSeverity::Warning,
                "database_dependency",
                "Database schema changes detected. Ensure migration compatibility and rollback procedures."
            ));
        }

        // Check for API dependencies
        if all_allowed_paths.iter().any(|dir| dir.contains("api") || dir.contains("endpoint")) {
            results.push(ValidationResult::fail(
                ValidationSeverity::Warning,
                "api_dependency",
                "API changes detected. Ensure backward compatibility and versioning strategy."
            ));
        }

        // Check for configuration dependencies
        if all_allowed_paths.iter().any(|dir| dir.contains("config") || dir.contains("env")) {
            results.push(ValidationResult::fail(
                ValidationSeverity::Info,
                "config_dependency",
                "Configuration changes detected. Ensure environment compatibility."
            ));
        }

        if results.is_empty() {
            results.push(ValidationResult::pass("dependency_validation", "Dependency validation passed"));
        }

        results
    }

    /// Check if ID follows the correct format
    fn is_valid_id_format(&self, id: &str) -> bool {
        let parts: Vec<&str> = id.split('-').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let prefix = parts[0];
        let number = parts[1];
        
        // Check prefix is uppercase letters
        if !prefix.chars().all(|c| c.is_ascii_uppercase()) || prefix.is_empty() {
            return false;
        }
        
        // Check number is digits
        if !number.chars().all(|c| c.is_ascii_digit()) || number.is_empty() {
            return false;
        }
        
        true
    }

    /// Check if acceptance criterion is measurable
    fn is_measurable_criterion(&self, criterion: &agent_agency_contracts::working_spec::AcceptanceCriterion) -> bool {
        let measurable_keywords = ["should", "must", "will", "verify", "check", "ensure", "confirm"];
        let text = format!("{} {} {}", criterion.given, criterion.when, criterion.then).to_lowercase();
        
        measurable_keywords.iter().any(|keyword| text.contains(keyword))
    }

    /// Extract external dependencies from working spec
    fn extract_external_dependencies(&self, spec: &agent_agency_contracts::working_spec::WorkingSpec) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Check title and description for external services
        let text = format!("{} {}", spec.title, spec.description).to_lowercase();
        
        let external_services = [
            "api", "database", "redis", "postgresql", "mysql", "mongodb",
            "elasticsearch", "kafka", "rabbitmq", "s3", "dynamodb",
            "firebase", "auth0", "stripe", "paypal", "twilio", "sendgrid"
        ];
        
        for service in &external_services {
            if text.contains(service) {
                dependencies.push(service.to_string());
            }
        }
        
        dependencies
    }
}

/// Validation pipeline that orchestrates multiple validation stages
/// Uses system-configuration ValidationPipeline with domain-specific functionality
pub struct ValidationPipeline {
    system_pipeline: SystemValidationPipeline,
    caws_validator: Arc<dyn CawsValidator>,
    config: ValidationPipelineConfig,
}

/// Configuration for the validation pipeline

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        // Create system-configuration pipeline config
        let system_config = SystemValidationPipelineConfig {
            base: PipelineConfig::default(),
            stop_on_first_error: config.strict_mode,
            severity_threshold: if config.strict_mode {
                ValidationSeverity::Warning
            } else {
                ValidationSeverity::Error
            },
            enable_validation_caching: true,
            max_validation_time: std::time::Duration::from_secs(config.caws_timeout_seconds),
            collect_all_errors: !config.skip_expensive_validations,
        };

        let mut system_pipeline = SystemValidationPipeline::new(system_config);

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
            system_pipeline.add_stage(Box::new(adapter));
        }

        Self {
            system_pipeline,
            caws_validator,
            config,
        }
    }

    /// Validate a working specification through all pipeline stages
    pub async fn validate_working_spec(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
    ) -> PlanningResult<crate::planning_agent::types::ValidationResults> {
        // Convert WorkingSpec to JSON for system-configuration pipeline
        let input = serde_json::to_value(working_spec)
            .map_err(|e| PlanningError::ValidationError(format!("Failed to serialize working spec: {}", e)))?;

        // Execute validation through system-configuration pipeline
        let system_results = self.system_pipeline.execute(input).await
            .map_err(|e| PlanningError::ValidationError(format!("Pipeline execution failed: {}", e)))?;

        // Convert system-configuration results to domain-specific results
        let mut all_issues = Vec::new();

        for result in system_results.results {
            let issue = crate::planning_agent::types::ValidationIssue {
                severity: match result.severity {
                    ValidationSeverity::Critical => crate::planning_agent::types::IssueSeverity::Error,
                    ValidationSeverity::Error => crate::planning_agent::types::IssueSeverity::Error,
                    ValidationSeverity::Warning => crate::planning_agent::types::IssueSeverity::Warning,
                    ValidationSeverity::Info => crate::planning_agent::types::IssueSeverity::Info,
                },
                category: result.category,
                description: result.message,
                suggestion: result.suggestion,
            };
            all_issues.push(issue);
        }

        let validation_status = if system_results.overall_passed {
            crate::planning_agent::types::ValidationStatus::Passed
        } else if all_issues.iter().any(|i| i.severity == crate::planning_agent::types::IssueSeverity::Error) {
            crate::planning_agent::types::ValidationStatus::Failed
        } else {
            crate::planning_agent::types::ValidationStatus::PassedWithRefinements
        };

        // Calculate CAWS compliance score
        let total_issues = all_issues.len();
        let error_count = all_issues.iter().filter(|i| i.severity == crate::planning_agent::types::IssueSeverity::Error).count();
        let caws_compliance_score = if total_issues == 0 {
            1.0
        } else {
            (total_issues - error_count) as f64 / total_issues as f64
        };

        // Convert issues to planning agent format
        let planning_issues: Vec<crate::planning_agent::types::ValidationIssue> = all_issues.iter().map(|issue| {
            crate::planning_agent::types::ValidationIssue {
                severity: match issue.severity {
                    crate::planning_agent::types::IssueSeverity::Error => crate::planning_agent::types::IssueSeverity::Error,
                    crate::planning_agent::types::IssueSeverity::Warning => crate::planning_agent::types::IssueSeverity::Warning,
                    crate::planning_agent::types::IssueSeverity::Info => crate::planning_agent::types::IssueSeverity::Info,
                },
                category: issue.category.clone(),
                description: issue.description.clone(),
                suggestion: issue.suggestion.clone(),
            }
        }).collect();

        Ok(crate::planning_agent::types::ValidationResults {
            overall_status: validation_status,
            caws_compliance_score,
            issues: planning_issues,
            applied_refinements: Vec::new(),
        })
    }
}

