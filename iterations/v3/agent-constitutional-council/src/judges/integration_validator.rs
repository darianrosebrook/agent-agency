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

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, instrument};
use serde_json;

use agent_agency_contracts::{
    JudgeEngine, JudgeVerdict, JudgePrompt, JudgeType, VerdictLabel,
    Violation, judge_io::Severity, RubricItem, WorkingSpecEvidence,
};

use crate::{ReviewContext, CouncilResult, CouncilError};
use super::{Judge, JudgeUtils};

/// Integration Validator for system compatibility and deployment readiness
#[derive(Debug)]
pub struct IntegrationValidator {
    /// Inference engine for LLM-based analysis
    engine: Arc<dyn JudgeEngine>,

    /// Integration evaluation rubric
    rubric: IntegrationRubric,
}

/// Integration evaluation rubric
#[derive(Debug)]
pub struct IntegrationRubric {
    /// API compatibility criteria
    api_items: Vec<RubricItem>,

    /// Data consistency criteria
    data_items: Vec<RubricItem>,

    /// System integration criteria
    system_items: Vec<RubricItem>,

    /// Deployment criteria
    deployment_items: Vec<RubricItem>,
}

impl Default for IntegrationRubric {
    fn default() -> Self {
        Self {
            api_items: vec![
                RubricItem {
                    id: "API-001".to_string(),
                    description: "API contracts are stable and backward compatible".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["api_contracts".to_string()],
                },
                RubricItem {
                    id: "API-002".to_string(),
                    description: "Breaking changes are properly versioned and communicated".to_string(),
                    weight: 0.85,
                    evidence_requirements: vec!["versioning_strategy".to_string()],
                },
            ],
            data_items: vec![
                RubricItem {
                    id: "DATA-001".to_string(),
                    description: "Data schemas maintain backward compatibility".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["schema_migrations".to_string()],
                },
                RubricItem {
                    id: "DATA-002".to_string(),
                    description: "Data consistency is maintained across system boundaries".to_string(),
                    weight: 0.85,
                    evidence_requirements: vec!["consistency_checks".to_string()],
                },
            ],
            system_items: vec![
                RubricItem {
                    id: "SYS-001".to_string(),
                    description: "Component interfaces and communication protocols are compatible".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["interface_contracts".to_string()],
                },
                RubricItem {
                    id: "SYS-002".to_string(),
                    description: "System dependencies and service interactions are properly managed".to_string(),
                    weight: 0.85,
                    evidence_requirements: vec!["dependency_analysis".to_string()],
                },
            ],
            deployment_items: vec![
                RubricItem {
                    id: "DEPLOY-001".to_string(),
                    description: "Deployment process is automated and reliable".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["deployment_pipeline".to_string()],
                },
                RubricItem {
                    id: "DEPLOY-002".to_string(),
                    description: "Infrastructure and operational requirements are specified".to_string(),
                    weight: 0.75,
                    evidence_requirements: vec!["infrastructure_specs".to_string()],
                },
            ],
        }
    }
}

impl IntegrationValidator {
    /// Create new integration validator
    pub fn new(engine: Arc<dyn JudgeEngine>) -> Self {
        Self {
            engine,
            rubric: IntegrationRubric::default(),
        }
    }

    /// Build the complete integration rubric
    fn build_rubric(&self) -> Vec<RubricItem> {
        let mut rubric = Vec::new();
        rubric.extend(self.rubric.api_items.clone());
        rubric.extend(self.rubric.data_items.clone());
        rubric.extend(self.rubric.system_items.clone());
        rubric.extend(self.rubric.deployment_items.clone());
        rubric
    }

    /// Build LLM prompt for integration analysis
    fn build_prompt(&self, ctx: &ReviewContext) -> JudgePrompt {
        let rubric = self.build_rubric();

        JudgePrompt {
            role: JudgeType::Integration,
            objective: "Evaluate the API compatibility, data consistency, system integration, and deployment readiness of this implementation. Assess backward compatibility, schema coherence, component interactions, and operational deployment requirements.".to_string(),
            rubric,
            evidence: WorkingSpecEvidence {
                spec_text: format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
                    ctx.working_spec.title,
                    ctx.working_spec.description,
                    ctx.working_spec.goals.join("\n- "),
                    ctx.working_spec.acceptance_criteria.iter()
                        .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                acceptance_criteria: ctx.working_spec.acceptance_criteria.iter()
                    .map(|ac| format!("{}: {}", ac.id, ac.then))
                    .collect(),
                risk_tier: ctx.working_spec.risk_tier.to_string(),
                context: serde_json::to_value(&ctx.working_spec.context).unwrap_or(serde_json::Value::Null),
            },
            output_schema: r#"{
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "required": ["score", "label", "rationale", "violations", "evidence_refs"],
                "properties": {
                    "score": {"type": "number", "minimum": 0.0, "maximum": 1.0},
                    "label": {"type": "string", "enum": ["Pass", "Fail", "NeedsInfo", "Conditional"]},
                    "rationale": {"type": "string"},
                    "violations": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["rule_id", "severity", "waivable", "description"],
                            "properties": {
                                "rule_id": {"type": "string"},
                                "severity": {"type": "string", "enum": ["Info", "Low", "Medium", "High", "Critical"]},
                                "waivable": {"type": "boolean"},
                                "description": {"type": "string"}
                            }
                        }
                    },
                    "evidence_refs": {"type": "array", "items": {"type": "string"}}
                }
            }"#.to_string(),
        }
    }
}

#[async_trait]
impl Judge for IntegrationValidator {
    #[instrument(skip(self, ctx), fields(judge = "integration", spec_id = %ctx.working_spec.id))]
    async fn review_spec(&self, ctx: &ReviewContext) -> CouncilResult<JudgeVerdict> {
        debug!("🔗 Integration Validator reviewing spec {}", ctx.working_spec.id);

        // STEP 1: Run deterministic integration checks
        let integration_violations = self.run_deterministic_checks(ctx);

        // STEP 2: Check for blocking integration violations
        if JudgeUtils::has_blocking_violations(&integration_violations) {
            debug!("🚫 Integration Validator: Blocking violations detected");
            return Ok(JudgeVerdict {
                label: VerdictLabel::Fail,
                score: 0.0,
                rationale: format!(
                    "Rejected due to critical integration violations: {}",
                    integration_violations.iter()
                        .map(|v| v.description.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                violations: integration_violations,
                evidence_refs: vec!["integration_analysis".to_string()],
            });
        }

        // STEP 3: Build LLM prompt for integration analysis
        let prompt = self.build_prompt(ctx);

        // STEP 4: Execute engine
        let req = JudgeUtils::build_request(prompt, 256);
        let llm_verdict = self.engine.complete(req).await
            .map_err(|e| CouncilError::Engine(e))?;

        // STEP 5: Merge findings
        let merged_verdict = JudgeUtils::merge_verdicts(integration_violations, llm_verdict.parsed);

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
    fn run_deterministic_checks(&self, ctx: &ReviewContext) -> Vec<Violation> {
        let mut violations = vec![];

        let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
            ctx.working_spec.title,
            ctx.working_spec.description,
            ctx.working_spec.goals.join("\n- "),
            ctx.working_spec.acceptance_criteria.iter()
                .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
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
            && !spec_text.contains("migration") {
            violations.push(Violation {
                rule_id: "INTEGRATION-DATA-001".to_string(),
                severity: Severity::Medium,
                waivable: true,
                description: "Data/schema changes detected without migration plan".to_string(),
            });
        }

        // Check for external dependencies
        if (spec_text.contains("external") || spec_text.contains("third-party"))
            && !spec_text.contains("fallback") {
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
