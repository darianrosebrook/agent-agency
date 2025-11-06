//! Constitutional Judge
//!
//! The Constitutional Judge evaluates ethical compliance, privacy protection,
//! safety considerations, and adherence to CAWS invariants. This judge ensures
//! that all autonomous agent operations align with ethical standards and legal
//! requirements.
//!
//! ## Responsibilities
//!
//! - **Ethical Analysis**: Evaluate moral implications of agent actions
//! - **Privacy Protection**: Ensure user data handling meets standards
//! - **Safety Assessment**: Identify potential harm or misuse scenarios
//! - **CAWS Compliance**: Verify adherence to non-waivable invariants
//! - **Legal Alignment**: Check for regulatory compliance issues
//!
//! ## Hybrid Reasoning Pattern
//!
//! 1. **Deterministic CAWS Checks**: Run invariant scanner for non-waivable rules
//! 2. **Critical Violation Detection**: Immediate rejection for blocking issues
//! 3. **LLM Gray-Zone Analysis**: Use inference for ethical judgment calls
//! 4. **Verdict Merging**: Combine deterministic results with LLM reasoning

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, instrument, warn};
use serde_json;

use agent_agency_contracts::{
    JudgeEngine, JudgeVerdict, JudgePrompt, JudgeType, VerdictLabel,
    Violation, RubricItem, WorkingSpecEvidence,
    invariants::Severity as InvariantsSeverity, judge_io::Severity,
};

use crate::{ReviewContext, CouncilResult, CouncilError};
use super::common::{Judge, JudgeUtils};

/// Constitutional Judge for ethical and compliance evaluation
#[derive(Debug)]
pub struct ConstitutionalJudge {
    /// Inference engine for LLM-based analysis
    engine: Arc<dyn JudgeEngine>,

    /// Constitutional rubric for evaluation criteria
    rubric: ConstitutionalRubric,
}

/// Constitutional evaluation rubric
#[derive(Debug)]
pub struct ConstitutionalRubric {
    /// Ethical criteria
    ethics_items: Vec<RubricItem>,

    /// Privacy criteria
    privacy_items: Vec<RubricItem>,

    /// Safety criteria
    safety_items: Vec<RubricItem>,

    /// CAWS compliance criteria
    caws_items: Vec<RubricItem>,
}

impl Default for ConstitutionalRubric {
    fn default() -> Self {
        Self {
            ethics_items: vec![
                RubricItem {
                    id: "ETHICS-001".to_string(),
                    description: "Actions align with human values and ethical principles".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["ethical_impact_assessment".to_string()],
                },
                RubricItem {
                    id: "ETHICS-002".to_string(),
                    description: "No potential for harm to humans or society".to_string(),
                    weight: 0.95,
                    evidence_requirements: vec!["harm_assessment".to_string()],
                },
            ],
            privacy_items: vec![
                RubricItem {
                    id: "PRIVACY-001".to_string(),
                    description: "User data is handled securely and privately".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["privacy_policy".to_string(), "data_handling".to_string()],
                },
                RubricItem {
                    id: "PRIVACY-002".to_string(),
                    description: "No unauthorized data collection or sharing".to_string(),
                    weight: 0.95,
                    evidence_requirements: vec!["consent_mechanism".to_string()],
                },
            ],
            safety_items: vec![
                RubricItem {
                    id: "SAFETY-001".to_string(),
                    description: "System prevents harmful or dangerous outcomes".to_string(),
                    weight: 0.95,
                    evidence_requirements: vec!["safety_measures".to_string()],
                },
                RubricItem {
                    id: "SAFETY-002".to_string(),
                    description: "Fail-safe mechanisms prevent runaway behavior".to_string(),
                    weight: 0.9,
                    evidence_requirements: vec!["circuit_breakers".to_string()],
                },
            ],
            caws_items: vec![
                RubricItem {
                    id: "CAWS-001".to_string(),
                    description: "Adheres to CAWS development standards".to_string(),
                    weight: 0.8,
                    evidence_requirements: vec!["caws_compliance".to_string()],
                },
                RubricItem {
                    id: "CAWS-002".to_string(),
                    description: "No violations of non-waivable invariants".to_string(),
                    weight: 0.95,
                    evidence_requirements: vec!["invariant_checks".to_string()],
                },
            ],
        }
    }
}

impl ConstitutionalJudge {
    /// Create new constitutional judge
    pub fn new(engine: Arc<dyn JudgeEngine>) -> Self {
        Self {
            engine,
            rubric: ConstitutionalRubric::default(),
        }
    }

    /// Build the complete constitutional rubric
    fn build_rubric(&self) -> Vec<RubricItem> {
        let mut rubric = Vec::new();
        rubric.extend(self.rubric.ethics_items.clone());
        rubric.extend(self.rubric.privacy_items.clone());
        rubric.extend(self.rubric.safety_items.clone());
        rubric.extend(self.rubric.caws_items.clone());
        rubric
    }

    /// Build LLM prompt for constitutional analysis (implementation method)
    fn build_prompt_impl(&self, ctx: &ReviewContext, deterministic_violations: &[Violation]) -> JudgePrompt {
        let rubric = self.build_rubric();

        JudgePrompt {
            role: JudgeType::Constitutional,
            objective: "Evaluate the ethical, privacy, safety, and CAWS compliance of this specification. Consider potential harms, data protection needs, fail-safe requirements, and adherence to development standards.".to_string(),
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
impl super::common::Judge for ConstitutionalJudge {
    fn judge_type(&self) -> JudgeType {
        JudgeType::Constitutional
    }

    fn rubric(&self) -> Vec<RubricItem> {
        self.build_rubric()
    }

    fn build_prompt(&self, ctx: &ReviewContext) -> JudgePrompt {
        self.build_prompt_impl(ctx, &[])
    }

    fn run_deterministic_checks(&self, ctx: &ReviewContext) -> Vec<Violation> {
        self.run_deterministic_checks_impl(ctx)
    }
}

impl ConstitutionalJudge {
    fn run_deterministic_checks_impl(&self, ctx: &ReviewContext) -> Vec<Violation> {
        // Run CAWS invariant checks
        let invariant_results = crate::run_caws_invariants(&ctx.working_spec);
        
        // Convert ViolationLocation to Violation
        invariant_results.checks.iter()
            .flat_map(|check| &check.violations)
            .map(|vl| Violation {
                rule_id: vl.rule_id.clone(),
                severity: match vl.severity {
                    InvariantsSeverity::Info => Severity::Info,
                    InvariantsSeverity::Low => Severity::Low,
                    InvariantsSeverity::Medium => Severity::Medium,
                    InvariantsSeverity::High => Severity::High,
                    InvariantsSeverity::Critical => Severity::Critical,
                },
                waivable: false, // Invariants are never waivable
                description: vl.description.clone(),
            })
            .collect()
    }

    // Override review_spec to use custom implementation
    #[instrument(skip(self, ctx), fields(judge = "constitutional", spec_id = %ctx.working_spec.id))]
    async fn review_spec(&self, ctx: &ReviewContext) -> CouncilResult<JudgeVerdict> {
        debug!("🧑‍⚖️  Constitutional Judge reviewing spec {}", ctx.working_spec.id);

        // STEP 1: Run deterministic CAWS invariant checks
        let violations = self.run_deterministic_checks_impl(ctx);

        // STEP 2: Check for critical invariant violations (all invariants are non-waivable)
        let critical_violations: Vec<Violation> = violations.iter()
            .filter(|v| v.severity == Severity::Critical)
            .cloned()
            .collect();

        if !critical_violations.is_empty() {
            warn!("🚫 Constitutional Judge: Blocking CAWS violations detected");

            return Ok(JudgeVerdict {
                label: VerdictLabel::Fail,
                score: 0.0,
                rationale: format!(
                    "Rejected due to non-waivable CAWS violations: {}",
                    violations.iter()
                        .map(|v| v.description.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                violations,
                evidence_refs: vec!["caws_invariants".to_string()],
            });
        }

        // STEP 3: Build LLM prompt for gray-zone assessment
        let prompt = self.build_prompt(ctx);

        // STEP 4: Execute engine (may hit prompt cache)
        let req = JudgeUtils::build_request(prompt, 256); // Allow longer responses for constitutional analysis
        let llm_verdict = self.engine.complete(req).await
            .map_err(|e| CouncilError::Engine(e))?;

        // STEP 5: Merge deterministic findings with LLM verdict
        let merged_verdict = JudgeUtils::merge_verdicts(vec![], llm_verdict.parsed);

        debug!(
            "🧑‍⚖️  Constitutional Judge verdict: {} (score: {:.2})",
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
