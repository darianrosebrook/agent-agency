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

/// Quality Evaluator for testing and requirements completeness
#[derive(Debug)]
pub struct QualityEvaluator {
    /// Inference engine for LLM-based analysis
    engine: Arc<dyn JudgeEngine>,
}

/// Quality evaluation rubric
#[derive(Debug)]
pub struct QualityRubric {
    /// Testing criteria
    #[allow(dead_code)] // Reserved for v4 features
    testing_items: Vec<RubricItem>,

    /// Requirements criteria
    #[allow(dead_code)] // Reserved for v4 features
    requirements_items: Vec<RubricItem>,

    /// Documentation criteria
    #[allow(dead_code)] // Reserved for v4 features
    documentation_items: Vec<RubricItem>,

    /// Reliability criteria
    #[allow(dead_code)] // Reserved for v4 features
    reliability_items: Vec<RubricItem>,
}

impl QualityRubric {
    /// Build the rubric using RubricBuilder
    pub fn build() -> Vec<RubricItem> {
        RubricBuilder::new()
            .add_items(vec![
                RubricItemBuilder::new(
                    "TEST-001",
                    "Comprehensive test coverage for all critical paths",
                    0.9,
                    vec!["test_coverage_report".to_string()],
                ),
                RubricItemBuilder::new(
                    "TEST-002",
                    "Acceptance criteria validated through automated tests",
                    0.85,
                    vec!["acceptance_tests".to_string()],
                ),
            ])
            .add_items(vec![
                RubricItemBuilder::new(
                    "REQ-001",
                    "All requirements have clear, testable acceptance criteria",
                    0.8,
                    vec!["requirements_traceability".to_string()],
                ),
                RubricItemBuilder::new(
                    "REQ-002",
                    "Requirements are complete and unambiguous",
                    0.75,
                    vec!["requirements_review".to_string()],
                ),
            ])
            .add_items(vec![
                RubricItemBuilder::new(
                    "DOC-001",
                    "API and user documentation is current and accurate",
                    0.7,
                    vec!["documentation_review".to_string()],
                ),
                RubricItemBuilder::new(
                    "DOC-002",
                    "Deployment and operational docs exist",
                    0.8,
                    vec!["deployment_docs".to_string()],
                ),
            ])
            .add_items(vec![
                RubricItemBuilder::new(
                    "RELIABILITY-001",
                    "System meets uptime and performance SLAs",
                    0.9,
                    vec!["sla_definitions".to_string()],
                ),
                RubricItemBuilder::new(
                    "RELIABILITY-002",
                    "Error handling and recovery mechanisms implemented",
                    0.85,
                    vec!["error_handling_review".to_string()],
                ),
            ])
            .build()
    }
}

impl QualityEvaluator {
    /// Create new quality evaluator
    pub fn new(engine: Arc<dyn JudgeEngine>) -> Self {
        Self { engine }
    }

    /// Build the complete quality rubric
    fn build_rubric(&self) -> Vec<RubricItem> {
        QualityRubric::build()
    }

    /// Build LLM prompt for quality analysis
    fn build_prompt_impl(&self, ctx: &ReviewContext) -> JudgePrompt {
        let rubric = self.build_rubric();

        JudgePrompt {
            role: JudgeType::Quality,
            objective: "Evaluate the testing completeness, requirements coverage, documentation quality, and reliability characteristics of this implementation. Assess test coverage, acceptance criteria validation, documentation adequacy, and system reliability.".to_string(),
            rubric,
            evidence: EvidenceBuilder::from_context(ctx),
            output_schema: JUDGE_OUTPUT_SCHEMA.to_string(),
        }
    }
}

impl QualityEvaluator {
    /// Run deterministic quality checks
    fn run_deterministic_checks_impl(&self, ctx: &ReviewContext) -> Vec<Violation> {
        let mut violations = vec![];

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
                description: "No acceptance criteria defined for requirements validation"
                    .to_string(),
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

#[async_trait]
impl super::common::Judge for QualityEvaluator {
    fn judge_type(&self) -> JudgeType {
        JudgeType::Quality
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
            "📊 Quality Evaluator reviewing spec {}",
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
