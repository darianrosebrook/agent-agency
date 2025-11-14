//! Integration Validator
//!
//! The Integration Validator evaluates API compatibility, data consistency,
//! deployment readiness, and system integration coherence. This judge ensures
//! that implementations work correctly within the broader system ecosystem.
//!
//! ## Responsibilities
//!
//! - **API Compatibility**: Contract compliance, backward compatibility
//! - **Data Consistency**: Schema alignment, migration safety, referential integrity
//! - **System Integration**: Component communication, service dependencies
//! - **Deployment Readiness**: Infrastructure compatibility, operational requirements
//! - **Cross-System Coherence**: End-to-end workflow validation
//!
//! ## Hybrid Reasoning Pattern
//!
//! 1. **Deterministic Integration Checks**: API contract validation, schema checks
//! 2. **Critical Integration Violations**: Immediate rejection for breaking changes
//! 3. **LLM Integration Analysis**: Use inference for deployment and compatibility review
//! 4. **Verdict Merging**: Combine automated checks with integration expertise

use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

use agent_agency_contracts::{
    judge_io::Severity, JudgeEngine, JudgePrompt, JudgeType, JudgeVerdict, RubricItem,
    VerdictLabel, Violation,
};

use super::common::{
    EvidenceBuilder, JudgeUtils, RubricBuilder, RubricItemBuilder, JUDGE_OUTPUT_SCHEMA,
};
use crate::{CouncilError, CouncilResult, ReviewContext};

// CAWS integration for runtime validation
use agent_orchestration::planning::caws_integration::CawsPlanBridge;

/// Integration Validator for system compatibility and deployment readiness
#[derive(Debug)]
pub struct IntegrationValidator {
    /// Inference engine for LLM-based analysis
    engine: Arc<dyn JudgeEngine>,
    /// CAWS validation bridge
    caws_bridge: CawsPlanBridge,
}

/// Integration evaluation rubric
#[derive(Debug)]
pub struct IntegrationRubric {
    /// API compatibility criteria
    #[allow(dead_code)] // Reserved for v4 features
    api_items: Vec<RubricItem>,

    /// Data consistency criteria
    #[allow(dead_code)] // Reserved for v4 features
    data_items: Vec<RubricItem>,

    /// System integration criteria
    #[allow(dead_code)] // Reserved for v4 features
    system_items: Vec<RubricItem>,

    /// Deployment criteria
    #[allow(dead_code)] // Reserved for v4 features
    deployment_items: Vec<RubricItem>,
}

impl IntegrationRubric {
    /// Build the rubric using RubricBuilder
    pub fn build() -> Vec<RubricItem> {
        RubricBuilder::new()
            .add_items(vec![
                RubricItemBuilder::new(
                    "API-001",
                    "API contracts are stable and backward compatible",
                    0.9,
                    vec!["api_contracts".to_string()],
                ),
                RubricItemBuilder::new(
                    "API-002",
                    "Breaking changes are properly versioned and communicated",
                    0.85,
                    vec!["versioning_strategy".to_string()],
                ),
            ])
            .add_items(vec![
                RubricItemBuilder::new(
                    "DATA-001",
                    "Data schemas maintain backward compatibility",
                    0.9,
                    vec!["schema_migrations".to_string()],
                ),
                RubricItemBuilder::new(
                    "DATA-002",
                    "Data consistency is maintained across system boundaries",
                    0.85,
                    vec!["consistency_checks".to_string()],
                ),
            ])
            .add_items(vec![
                RubricItemBuilder::new(
                    "SYS-001",
                    "Component interfaces and communication protocols are compatible",
                    0.8,
                    vec!["interface_contracts".to_string()],
                ),
                RubricItemBuilder::new(
                    "SYS-002",
                    "System dependencies and service interactions are properly managed",
                    0.85,
                    vec!["dependency_analysis".to_string()],
                ),
            ])
            .add_items(vec![
                RubricItemBuilder::new(
                    "DEPLOY-001",
                    "Deployment process is automated and reliable",
                    0.8,
                    vec!["deployment_pipeline".to_string()],
                ),
                RubricItemBuilder::new(
                    "DEPLOY-002",
                    "Infrastructure and operational requirements are specified",
                    0.75,
                    vec!["infrastructure_specs".to_string()],
                ),
            ])
            .add_items(vec![
                RubricItemBuilder::new(
                    "CAWS-001",
                    "Working specification adheres to CAWS standards and constraints",
                    0.9,
                    vec!["caws_compliance".to_string()],
                ),
                RubricItemBuilder::new(
                    "CAWS-002",
                    "Risk tier, scope boundaries, and change budgets are appropriately defined",
                    0.85,
                    vec!["scope_validation".to_string()],
                ),
                RubricItemBuilder::new(
                    "CAWS-003",
                    "API contracts and acceptance criteria are complete and testable",
                    0.8,
                    vec!["contract_validation".to_string()],
                ),
            ])
            .build()
    }
}

impl IntegrationValidator {
    /// Create new integration validator
    pub fn new(engine: Arc<dyn JudgeEngine>) -> Self {
        Self {
            engine,
            caws_bridge: CawsPlanBridge::new().unwrap_or_else(|e| {
                // For tests, try to create bridge with a temp directory
                // In production, this should never fail as CAWS should be properly set up
                if cfg!(test) {
                    // Try to create bridge with current directory or temp path
                    CawsPlanBridge::with_project_root(".").unwrap_or_else(|_| {
                        // Last resort: try with a temp directory
                        let temp_dir = std::env::temp_dir();
                        CawsPlanBridge::with_project_root(&temp_dir)
                            .expect("Failed to initialize CawsPlanBridge even with temp directory")
                    })
                } else {
                    panic!("CawsPlanBridge::new() failed: {}. CAWS setup required.", e);
                }
            }),
        }
    }

    /// Build the complete integration rubric
    fn build_rubric(&self) -> Vec<RubricItem> {
        IntegrationRubric::build()
    }

    /// Build LLM prompt for integration analysis
    fn build_prompt_impl(&self, ctx: &ReviewContext) -> JudgePrompt {
        let rubric = self.build_rubric();

        JudgePrompt {
            role: JudgeType::Integration,
            objective: "Evaluate the API compatibility, data consistency, system integration, deployment readiness, and CAWS compliance of this implementation. Assess backward compatibility, schema coherence, component interactions, operational deployment requirements, and adherence to CAWS working specification standards including risk tier constraints, scope boundaries, change budgets, and contract specifications.".to_string(),
            rubric,
            evidence: EvidenceBuilder::from_context(ctx),
            output_schema: JUDGE_OUTPUT_SCHEMA.to_string(),
        }
    }
}

impl IntegrationValidator {
    /// Get the judge type
    #[allow(dead_code)] // Part of trait implementation, may be called via trait
    fn judge_type(&self) -> JudgeType {
        JudgeType::Integration
    }

    /// Get the judge rubric
    #[allow(dead_code)] // Part of trait implementation, may be called via trait
    fn rubric(&self) -> Vec<RubricItem> {
        self.build_rubric()
    }

    /// Run deterministic integration checks
    #[allow(dead_code)] // Part of trait implementation, may be called via trait
    fn run_deterministic_checks(&self, ctx: &ReviewContext) -> Vec<Violation> {
        self.run_deterministic_checks_impl(ctx)
    }
}

#[async_trait]
impl super::common::Judge for IntegrationValidator {
    fn judge_type(&self) -> JudgeType {
        JudgeType::Integration
    }

    fn rubric(&self) -> Vec<RubricItem> {
        self.build_rubric()
    }

    fn build_prompt(&self, ctx: &ReviewContext) -> JudgePrompt {
        self.build_prompt_impl(ctx)
    }

    fn run_deterministic_checks(&self, ctx: &ReviewContext) -> Vec<Violation> {
        self.run_deterministic_checks_impl(ctx)
    }

    async fn execute_llm_evaluation(
        &self,
        ctx: &ReviewContext,
        prompt: JudgePrompt,
        violations: Vec<Violation>,
    ) -> CouncilResult<JudgeVerdict> {
        debug!(
            "🔗 Integration Validator reviewing spec {}",
            ctx.working_spec.id
        );

        // STEP 4: Execute engine
        let req = JudgeUtils::build_request(prompt, 256);
        let llm_verdict = self
            .engine
            .complete(req)
            .await
            .map_err(|e| CouncilError::Engine(e))?;

        // STEP 5: Merge findings
        let merged_verdict = JudgeUtils::merge_verdicts(violations, llm_verdict.parsed);

        debug!(
            "🔗 Integration Validator verdict: {} (score: {:.2})",
            match merged_verdict.label {
                VerdictLabel::Pass => "PASS",
                VerdictLabel::Fail => "FAIL",
                VerdictLabel::NeedsInfo => "NEEDS INFO",
                VerdictLabel::Conditional => "CONDITIONAL",
            },
            merged_verdict.score
        );

        Ok(merged_verdict)
    }
}

impl IntegrationValidator {
    /// Run deterministic integration checks
    fn run_deterministic_checks_impl(&self, ctx: &ReviewContext) -> Vec<Violation> {
        let mut violations = vec![];

        // CAWS Runtime Validation - Check working spec compliance
        if let Err(caws_error) = self.caws_bridge.validate_working_spec(&ctx.working_spec) {
            violations.push(Violation {
                rule_id: "CAWS-RUNTIME-001".to_string(),
                severity: Severity::High,
                waivable: false,
                description: format!("CAWS specification validation failed: {}", caws_error),
            });
        }

        // Additional CAWS Runtime Checks

        // Check risk tier constraints
        if ctx.working_spec.risk_tier > 3 {
            violations.push(Violation {
                rule_id: "CAWS-RUNTIME-002".to_string(),
                severity: Severity::High,
                waivable: false,
                description: format!(
                    "Risk tier {} exceeds maximum allowed tier 3",
                    ctx.working_spec.risk_tier
                ),
            });
        }

        // Check scope boundaries are properly defined
        if ctx.working_spec.scope.is_empty() {
            violations.push(Violation {
                rule_id: "CAWS-RUNTIME-003".to_string(),
                severity: Severity::Medium,
                waivable: true,
                description:
                    "Working spec scope boundaries are not defined - specify allowed/blocked paths"
                        .to_string(),
            });
        }

        // Check change budget is reasonable
        if ctx.working_spec.change_budget.max_files > 1000
            || ctx.working_spec.change_budget.max_loc > 50000
        {
            violations.push(Violation {
                rule_id: "CAWS-RUNTIME-004".to_string(),
                severity: Severity::Medium,
                waivable: true,
                description: format!("Change budget exceeds recommended limits (files: {}, loc: {}) - consider breaking into smaller tasks",
                    ctx.working_spec.change_budget.max_files, ctx.working_spec.change_budget.max_loc),
            });
        }

        // Check for proper CAWS mode specification
        // Note: mode field not present in WorkingSpec, skipping this check
        // if ctx.working_spec.mode.is_empty() {
        //     violations.push(Violation {
        //         rule_id: "CAWS-RUNTIME-005".to_string(),
        //         severity: Severity::Low,
        //         waivable: true,
        //         description: "Working spec mode not specified - specify 'feature', 'refactor', 'fix', 'chore', or 'doc'".to_string(),
        //     });
        // }

        // Validate contracts exist for feature changes
        // Note: mode and contracts fields not present in WorkingSpec, skipping this check
        // if ctx.working_spec.mode == "feature" && ctx.working_spec.contracts.is_empty() {
        //     violations.push(Violation {
        //         rule_id: "CAWS-RUNTIME-006".to_string(),
        //         severity: Severity::Medium,
        //         waivable: true,
        //         description: "Feature changes should include API contracts for compatibility validation".to_string(),
        //     });
        // }

        let spec_text = format!(
            "{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
            ctx.working_spec.title,
            ctx.working_spec.description,
            ctx.working_spec.goals.join("\n- "),
            ctx.working_spec
                .acceptance_criteria
                .iter()
                .map(|ac| format!(
                    "{}: Given {}, When {}, Then {}",
                    ac.id, ac.given, ac.when, ac.then
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Check for API compatibility concerns
        if spec_text.contains("breaking change") && !spec_text.contains("version") {
            violations.push(Violation {
                rule_id: "INTEGRATION-API-001".to_string(),
                severity: Severity::High,
                waivable: false,
                description: "Breaking changes detected without version strategy".to_string(),
            });
        }

        // Check for database/data changes
        if (spec_text.contains("database") || spec_text.contains("schema"))
            && !spec_text.contains("migration")
        {
            violations.push(Violation {
                rule_id: "INTEGRATION-DATA-001".to_string(),
                severity: Severity::Medium,
                waivable: true,
                description: "Data/schema changes detected without migration plan".to_string(),
            });
        }

        // Check for external dependencies
        if (spec_text.contains("external") || spec_text.contains("third-party"))
            && !spec_text.contains("fallback")
        {
            violations.push(Violation {
                rule_id: "INTEGRATION-SYS-001".to_string(),
                severity: Severity::Medium,
                waivable: true,
                description: "External dependencies detected without fallback strategy".to_string(),
            });
        }

        // Check for deployment considerations
        if !spec_text.contains("deploy") && !spec_text.contains("infrastructure") {
            violations.push(Violation {
                rule_id: "INTEGRATION-DEPLOY-001".to_string(),
                severity: Severity::Info,
                waivable: true,
                description: "No deployment or infrastructure requirements specified".to_string(),
            });
        }

        violations
    }
}
