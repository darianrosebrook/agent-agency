//! Technical Auditor
//!
//! The Technical Auditor evaluates code quality, security practices,
//! architectural soundness, and performance characteristics. This judge
//! ensures that implementations follow technical best practices and
//! maintain system integrity.
//!
//! ## Responsibilities
//!
//! - **Code Quality**: Clean code, maintainability, complexity analysis
//! - **Security**: Authentication, authorization, input validation
//! - **Architecture**: Design patterns, separation of concerns, scalability
//! - **Performance**: Efficiency, resource usage, optimization opportunities
//! - **Reliability**: Error handling, resilience, monitoring
//!
//! ## Hybrid Reasoning Pattern
//!
//! 1. **Deterministic Technical Checks**: Security scans, complexity analysis
//! 2. **Critical Technical Violations**: Immediate rejection for blocking issues
//! 3. **LLM Technical Analysis**: Use inference for architecture/design review
//! 4. **Verdict Merging**: Combine automated checks with expert reasoning

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

/// Technical Auditor for code quality and architecture evaluation
#[derive(Debug)]
pub struct TechnicalAuditor {
    /// Inference engine for LLM-based analysis
    engine: Arc<dyn JudgeEngine>,

    /// Technical evaluation rubric
    rubric: TechnicalRubric,
}

/// Technical evaluation rubric
#[derive(Debug)]
pub struct TechnicalRubric {
    /// Code quality criteria
    quality_items: Vec<RubricItem>,

    /// Security criteria
    security_items: Vec<RubricItem>,

    /// Architecture criteria
    architecture_items: Vec<RubricItem>,

    /// Performance criteria
    performance_items: Vec<RubricItem>,
}

impl Default for TechnicalRubric {
    fn default() -> Self {
        Self {
            quality_items: vec![
                RubricItem {
                    id: "QUALITY-001".to_string(),
                    description: "Code follows clean code principles and best practices".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["code_review".to_string()],
                },
                RubricItem {
                    id: "QUALITY-002".to_string(),
                    description: "Cyclomatic complexity is within acceptable limits".to_string(),
                    weight: 0.7,
                    evidence_requirements: vec!["complexity_analysis".to_string()],
                },
            ],
            security_items: vec![
                RubricItem {
                    id: "SECURITY-001".to_string(),
                    description: "Proper authentication and authorization implemented".to_string(),
                    weight: 0.95,
                    evidence_requirements: vec!["auth_implementation".to_string()],
                },
                RubricItem {
                    id: "SECURITY-002".to_string(),
                    description: "Input validation prevents injection attacks".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["input_validation".to_string()],
                },
            ],
            architecture_items: vec![
                RubricItem {
                    id: "ARCH-001".to_string(),
                    description: "System follows appropriate architectural patterns".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["architecture_diagram".to_string()],
                },
                RubricItem {
                    id: "ARCH-002".to_string(),
                    description: "Components have clear separation of concerns".to_string(),
                    weight: 0.85,
                    evidence_requirements: vec!["component_boundaries".to_string()],
                },
            ],
            performance_items: vec![
                RubricItem {
                    id: "PERF-001".to_string(),
                    description: "Implementation meets performance requirements".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["performance_tests".to_string()],
                },
                RubricItem {
                    id: "PERF-002".to_string(),
                    description: "Resource usage is efficient and monitored".to_string(),
                    weight: 0.7,
                    evidence_requirements: vec!["resource_monitoring".to_string()],
                },
            ],
        }
    }
}

impl TechnicalAuditor {
    /// Create new technical auditor
    pub fn new(engine: Arc<dyn JudgeEngine>) -> Self {
        Self {
            engine,
            rubric: TechnicalRubric::default(),
        }
    }

    /// Build the complete technical rubric
    fn build_rubric(&self) -> Vec<RubricItem> {
        let mut rubric = Vec::new();
        rubric.extend(self.rubric.quality_items.clone());
        rubric.extend(self.rubric.security_items.clone());
        rubric.extend(self.rubric.architecture_items.clone());
        rubric.extend(self.rubric.performance_items.clone());
        rubric
    }

    /// Build LLM prompt for technical analysis
    fn build_prompt(&self, ctx: &ReviewContext) -> JudgePrompt {
        let rubric = self.build_rubric();

        JudgePrompt {
            role: JudgeType::Technical,
            objective: "Evaluate the technical quality, security, architecture, and performance characteristics of this implementation. Assess code quality, identify security vulnerabilities, review architectural decisions, and analyze performance implications.".to_string(),
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
impl Judge for TechnicalAuditor {
    #[instrument(skip(self, ctx), fields(judge = "technical", spec_id = %ctx.working_spec.id))]
    async fn review_spec(&self, ctx: &ReviewContext) -> CouncilResult<JudgeVerdict> {
        debug!("🔧 Technical Auditor reviewing spec {}", ctx.working_spec.id);

        // TODO: Enhance deterministic technical checks implementation
        // - [ ] Expand deterministic checks to cover all CAWS invariants
        // - [ ] Add more sophisticated pattern matching for violations
        // - [ ] Integrate with static analysis tools if available
        // - [ ] Add performance checks for resource-intensive operations
        // - [ ] Add unit tests for each check type
        // - [ ] Add integration tests with real working specs
        // STEP 1: Run deterministic technical checks (placeholder for now)
        let technical_violations = self.run_deterministic_checks(ctx);

        // STEP 2: Check for blocking technical violations
        if JudgeUtils::has_blocking_violations(&technical_violations) {
            debug!("🚫 Technical Auditor: Blocking violations detected");
            return Ok(JudgeVerdict {
                label: VerdictLabel::Fail,
                score: 0.0,
                rationale: format!(
                    "Rejected due to critical technical violations: {}",
                    technical_violations.iter()
                        .map(|v| v.description.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                violations: technical_violations,
                evidence_refs: vec!["technical_analysis".to_string()],
            });
        }

        // STEP 3: Build LLM prompt for technical analysis
        let prompt = self.build_prompt(ctx);

        // STEP 4: Execute engine
        let req = JudgeUtils::build_request(prompt, 256);
        let llm_verdict = self.engine.complete(req).await
            .map_err(|e| CouncilError::Engine(e))?;

        // STEP 5: Merge findings
        let merged_verdict = JudgeUtils::merge_verdicts(technical_violations, llm_verdict.parsed);

        debug!(
            "🔧 Technical Auditor verdict: {} (score: {:.2})",
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

impl TechnicalAuditor {
    /// Run deterministic technical checks
    fn run_deterministic_checks(&self, ctx: &ReviewContext) -> Vec<Violation> {
        let mut violations = vec![];

        // Check for security basics (simplified checks)
        let spec_text = format!("{}: {}\n\nGoals: {}\n\nAcceptance Criteria: {}",
            ctx.working_spec.title,
            ctx.working_spec.description,
            ctx.working_spec.goals.join("\n- "),
            ctx.working_spec.acceptance_criteria.iter()
                .map(|ac| format!("{}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
                .collect::<Vec<_>>()
                .join("\n")
        );

        if !spec_text.contains("auth") && !spec_text.contains("authentication") {
            violations.push(Violation {
                rule_id: "TECH-SECURITY-001".to_string(),
                severity: Severity::Medium,
                waivable: true,
                description: "No authentication mechanism specified".to_string(),
            });
        }

        // Check for error handling
        if !spec_text.contains("error") && !spec_text.contains("Result") {
            violations.push(Violation {
                rule_id: "TECH-QUALITY-001".to_string(),
                severity: Severity::Low,
                waivable: true,
                description: "No error handling patterns detected".to_string(),
            });
        }

        // Check for performance considerations
        if !spec_text.contains("performance") && !spec_text.contains("latency") {
            violations.push(Violation {
                rule_id: "TECH-PERF-001".to_string(),
                severity: Severity::Info,
                waivable: true,
                description: "No performance requirements specified".to_string(),
            });
        }

        violations
    }
}
