//! Quality Evaluator
//!
//! The Quality Evaluator assesses testing completeness, requirements coverage,
//! reliability metrics, and overall software quality assurance. This judge
//! ensures that implementations are thoroughly tested and meet quality standards.
//!
//! ## Responsibilities
//!
//! - **Testing Coverage**: Unit, integration, and end-to-end test completeness
//! - **Requirements Traceability**: All requirements have acceptance criteria
//! - **Quality Metrics**: Code coverage, mutation scores, reliability measures
//! - **Documentation**: API docs, user guides, deployment instructions
//! - **Reliability**: Error rates, uptime requirements, failure handling
//!
//! ## Hybrid Reasoning Pattern
//!
//! 1. **Deterministic Quality Checks**: Coverage analysis, test validation
//! 2. **Critical Quality Gaps**: Immediate rejection for missing critical tests
//! 3. **LLM Quality Analysis**: Use inference for requirements completeness review
//! 4. **Verdict Merging**: Combine automated metrics with expert assessment

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

/// Quality Evaluator for testing and requirements completeness
#[derive(Debug)]
pub struct QualityEvaluator {
    /// Inference engine for LLM-based analysis
    engine: Arc<dyn JudgeEngine>,

    /// Quality evaluation rubric
    rubric: QualityRubric,
}

/// Quality evaluation rubric
#[derive(Debug)]
pub struct QualityRubric {
    /// Testing criteria
    testing_items: Vec<RubricItem>,

    /// Requirements criteria
    requirements_items: Vec<RubricItem>,

    /// Documentation criteria
    documentation_items: Vec<RubricItem>,

    /// Reliability criteria
    reliability_items: Vec<RubricItem>,
}

impl Default for QualityRubric {
    fn default() -> Self {
        Self {
            testing_items: vec![
                RubricItem {
                    id: "TEST-001".to_string(),
                    description: "Comprehensive test coverage for all critical paths".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["test_coverage_report".to_string()],
                },
                RubricItem {
                    id: "TEST-002".to_string(),
                    description: "Acceptance criteria validated through automated tests".to_string(),
                    weight: 0.85,
                    evidence_requirements: vec!["acceptance_tests".to_string()],
                },
            ],
            requirements_items: vec![
                RubricItem {
                    id: "REQ-001".to_string(),
                    description: "All requirements have clear, testable acceptance criteria".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["requirements_traceability".to_string()],
                },
                RubricItem {
                    id: "REQ-002".to_string(),
                    description: "Requirements are complete and unambiguous".to_string(),
                    weight: 0.75,
                    evidence_requirements: vec!["requirements_review".to_string()],
                },
            ],
            documentation_items: vec![
                RubricItem {
                    id: "DOC-001".to_string(),
                    description: "API and user documentation is current and accurate".to_string(),
                    weight: 0.7,
                    evidence_requirements: vec!["documentation_review".to_string()],
                },
                RubricItem {
                    id: "DOC-002".to_string(),
                    description: "Deployment and operational docs exist".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["deployment_docs".to_string()],
                },
            ],
            reliability_items: vec![
                RubricItem {
                    id: "RELIABILITY-001".to_string(),
                    description: "System meets uptime and performance SLAs".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["sla_definitions".to_string()],
                },
                RubricItem {
                    id: "RELIABILITY-002".to_string(),
                    description: "Error handling and recovery mechanisms implemented".to_string(),
                    weight: 0.85,
                    evidence_requirements: vec!["error_handling_review".to_string()],
                },
            ],
        }
    }
}

impl QualityEvaluator {
    /// Create new quality evaluator
    pub fn new(engine: Arc<dyn JudgeEngine>) -> Self {
        Self {
            engine,
            rubric: QualityRubric::default(),
        }
    }

    /// Build the complete quality rubric
    fn build_rubric(&self) -> Vec<RubricItem> {
        let mut rubric = Vec::new();
        rubric.extend(self.rubric.testing_items.clone());
        rubric.extend(self.rubric.requirements_items.clone());
        rubric.extend(self.rubric.documentation_items.clone());
        rubric.extend(self.rubric.reliability_items.clone());
        rubric
    }

    /// Build LLM prompt for quality analysis
    fn build_prompt(&self, ctx: &ReviewContext) -> JudgePrompt {
        let rubric = self.build_rubric();

        JudgePrompt {
            role: JudgeType::Quality,
            objective: "Evaluate the testing completeness, requirements coverage, documentation quality, and reliability characteristics of this implementation. Assess test coverage, acceptance criteria validation, documentation adequacy, and system reliability.".to_string(),
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
impl Judge for QualityEvaluator {
    #[instrument(skip(self, ctx), fields(judge = "quality", spec_id = %ctx.working_spec.id))]
    async fn review_spec(&self, ctx: &ReviewContext) -> CouncilResult<JudgeVerdict> {
        debug!("📊 Quality Evaluator reviewing spec {}", ctx.working_spec.id);

        // STEP 1: Run deterministic quality checks
        let quality_violations = self.run_deterministic_checks(ctx);

        // STEP 2: Check for blocking quality violations
        if JudgeUtils::has_blocking_violations(&quality_violations) {
            debug!("🚫 Quality Evaluator: Blocking violations detected");
            return Ok(JudgeVerdict {
                label: VerdictLabel::Fail,
                score: 0.0,
                rationale: format!(
                    "Rejected due to critical quality violations: {}",
                    quality_violations.iter()
                        .map(|v| v.description.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                violations: quality_violations,
                evidence_refs: vec!["quality_analysis".to_string()],
            });
        }

        // STEP 3: Build LLM prompt for quality analysis
        let prompt = self.build_prompt(ctx);

        // STEP 4: Execute engine
        let req = JudgeUtils::build_request(prompt, 256);
        let llm_verdict = self.engine.complete(req).await
            .map_err(|e| CouncilError::Engine(e))?;

        // STEP 5: Merge findings
        let merged_verdict = JudgeUtils::merge_verdicts(quality_violations, llm_verdict.parsed);

        debug!(
            "📊 Quality Evaluator verdict: {} (score: {:.2})",
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

impl QualityEvaluator {
    /// Run deterministic quality checks
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

        // Check for test coverage mentions
        if !spec_text.contains("test") && !spec_text.contains("coverage") {
            violations.push(Violation {
                rule_id: "QUALITY-TEST-001".to_string(),
                severity: Severity::Medium,
                waivable: false,
                description: "No testing strategy or coverage requirements specified".to_string(),
            });
        }

        // Check for acceptance criteria
        if ctx.working_spec.acceptance_criteria.is_empty() {
            violations.push(Violation {
                rule_id: "QUALITY-REQ-001".to_string(),
                severity: Severity::High,
                waivable: false,
                description: "No acceptance criteria defined for requirements validation".to_string(),
            });
        }

        // Check for documentation mentions
        if !spec_text.contains("doc") && !spec_text.contains("documentation") {
            violations.push(Violation {
                rule_id: "QUALITY-DOC-001".to_string(),
                severity: Severity::Low,
                waivable: true,
                description: "No documentation requirements specified".to_string(),
            });
        }

        // Check for performance/reliability requirements
        if !spec_text.contains("performance") && !spec_text.contains("latency") {
            violations.push(Violation {
                rule_id: "QUALITY-RELIABILITY-001".to_string(),
                severity: Severity::Info,
                waivable: true,
                description: "No performance or reliability requirements specified".to_string(),
            });
        }

        violations
    }
}
