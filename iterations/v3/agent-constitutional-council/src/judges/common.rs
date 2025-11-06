//! Common infrastructure for constitutional judges
//!
//! This module provides shared types and utilities for implementing
//! constitutional judges with consistent patterns and behaviors.

use async_trait::async_trait;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use agent_agency_contracts::{
    JudgeEngine, JudgeVerdict, JudgePrompt, JudgeType, VerdictLabel,
    Violation, judge_io::Severity, RubricItem, WorkingSpecEvidence,
};

use crate::{ReviewContext, CouncilResult, CouncilError};

/// Fluent API for building judge rubrics
#[derive(Debug, Clone)]
pub struct RubricBuilder {
    items: Vec<RubricItem>,
}

impl Default for RubricBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RubricBuilder {
    /// Create a new empty rubric builder
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    /// Add a rubric item
    pub fn add_item(mut self, item: RubricItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple rubric items
    pub fn add_items(mut self, items: Vec<RubricItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Build the final rubric
    pub fn build(self) -> Vec<RubricItem> {
        self.items
    }
}

/// Extension trait for JudgeType to provide string conversion
pub trait JudgeTypeExt {
    fn as_str(&self) -> &'static str;
}

impl JudgeTypeExt for agent_agency_contracts::JudgeType {
    fn as_str(&self) -> &'static str {
        match self {
            agent_agency_contracts::JudgeType::Constitutional => "constitutional",
            agent_agency_contracts::JudgeType::Technical => "technical",
            agent_agency_contracts::JudgeType::Quality => "quality",
            agent_agency_contracts::JudgeType::Integration => "integration",
            agent_agency_contracts::JudgeType::Security => "security",
            agent_agency_contracts::JudgeType::Performance => "performance",
            // Add other variants as needed
            _ => "unknown",
        }
    }
}

/// Common trait for all constitutional judges with default implementations
#[async_trait]
pub trait Judge: Send + Sync {
    /// The judge's specific judge type
    fn judge_type(&self) -> JudgeType;

    /// The judge's evaluation rubric
    fn rubric(&self) -> Vec<RubricItem>;

    /// Build the judge's specific LLM prompt
    fn build_prompt(&self, ctx: &ReviewContext) -> JudgePrompt;

    /// Run deterministic checks specific to this judge
    fn run_deterministic_checks(&self, ctx: &ReviewContext) -> Vec<Violation>;

    /// Review a working spec and return a verdict
    async fn review_spec(&self, ctx: &ReviewContext) -> CouncilResult<JudgeVerdict> {
        // STEP 1: Run deterministic checks
        let violations = self.run_deterministic_checks(ctx);

        // STEP 2: Check for blocking violations
        if JudgeUtils::has_blocking_violations(&violations) {
            return Ok(JudgeVerdict {
                label: VerdictLabel::Fail,
                score: 0.0,
                rationale: format!(
                    "Rejected due to critical violations: {}",
                    violations.iter()
                        .map(|v| v.description.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                violations,
                evidence_refs: vec![format!("{}_analysis", self.judge_type().as_str().to_lowercase())],
            });
        }

        // STEP 3: Build LLM prompt
        let prompt = self.build_prompt(ctx);

        // STEP 4: Execute engine (this would need to be passed in or available via context)
        // For now, this is a placeholder - judges will need their engine reference
        // This shows the pattern but concrete judges will implement this part
        self.execute_llm_evaluation(ctx, prompt, violations).await
    }

    /// Execute LLM evaluation and merge with violations
    async fn execute_llm_evaluation(
        &self,
        _ctx: &ReviewContext,
        _prompt: JudgePrompt,
        _violations: Vec<Violation>,
    ) -> CouncilResult<JudgeVerdict> {
        // This is implemented by concrete judges that have access to their engine
        Err(CouncilError::Engine(agent_agency_contracts::EngineError::InferenceFailed {
            message: "LLM evaluation not implemented".to_string()
        }))
    }
}

/// Common working spec evidence builder
pub struct EvidenceBuilder;

impl EvidenceBuilder {
    /// Build working spec evidence from review context
    pub fn from_context(ctx: &ReviewContext) -> WorkingSpecEvidence {
        WorkingSpecEvidence {
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
        }
    }
}

/// Common JSON schema for judge outputs
pub const JUDGE_OUTPUT_SCHEMA: &str = r#"{
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
}"#;

/// Common judge implementation utilities
pub struct JudgeUtils;

impl JudgeUtils {
    /// Build engine request for a judge
    pub fn build_request(prompt: JudgePrompt, max_tokens: usize) -> agent_agency_contracts::EngineRequest {
        agent_agency_contracts::EngineRequest {
            prompt,
            max_tokens,
            temperature: 0.1, // Low temperature for consistent judgments
            seed: Some(42),   // Reproducible results
        }
    }

    /// Merge deterministic violations with LLM verdict
    pub fn merge_verdicts(
        deterministic_violations: Vec<Violation>,
        llm_verdict: JudgeVerdict,
    ) -> JudgeVerdict {
        // If there are critical deterministic violations, override LLM score
        let has_critical_deterministic = deterministic_violations.iter()
            .any(|v| v.severity == Severity::Critical);

        let mut merged_violations = deterministic_violations;
        merged_violations.extend(llm_verdict.violations);

        let final_score = if has_critical_deterministic {
            0.0 // Always fail with critical violations
        } else {
            llm_verdict.score
        };

        let final_label = if has_critical_deterministic {
            VerdictLabel::Fail
        } else {
            llm_verdict.label
        };

        JudgeVerdict {
            score: final_score,
            label: final_label,
            rationale: llm_verdict.rationale,
            violations: merged_violations,
            evidence_refs: llm_verdict.evidence_refs,
        }
    }

    /// Check if violations contain non-waivable failures
    pub fn has_blocking_violations(violations: &[Violation]) -> bool {
        violations.iter().any(|v| {
            v.severity == Severity::Critical && !v.waivable
        })
    }
}

/// Helper for creating standardized rubric items
pub struct RubricItemBuilder;

impl RubricItemBuilder {
    /// Create a rubric item with standard structure
    pub fn new(id: &str, description: &str, weight: f32, evidence: Vec<String>) -> RubricItem {
        RubricItem {
            id: id.to_string(),
            description: description.to_string(),
            weight,
            evidence_requirements: evidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::JudgeType;

    /// Mock judge for testing common infrastructure
    struct MockJudge;

    #[async_trait]
    impl Judge for MockJudge {
        fn judge_type(&self) -> JudgeType {
            JudgeType::Technical
        }

        fn rubric(&self) -> Vec<RubricItem> {
            vec![RubricItemBuilder::new(
                "TEST-001",
                "Test rubric item",
                0.8,
                vec!["test_evidence".to_string()],
            )]
        }

        fn build_prompt(&self, _ctx: &ReviewContext) -> JudgePrompt {
            // Mock implementation
            JudgePrompt {
                role: JudgeType::Technical,
                objective: "Test objective".to_string(),
                rubric: self.rubric(),
                evidence: WorkingSpecEvidence {
                    spec_text: "Test spec".to_string(),
                    acceptance_criteria: vec![],
                    risk_tier: "high".to_string(),
                    context: serde_json::Value::Null,
                },
                output_schema: JUDGE_OUTPUT_SCHEMA.to_string(),
            }
        }

        fn run_deterministic_checks(&self, _ctx: &ReviewContext) -> Vec<Violation> {
            vec![]
        }
    }

    #[test]
    fn test_rubric_builder() {
        let rubric = RubricBuilder::new()
            .add_item(RubricItemBuilder::new("TEST-001", "First item", 0.8, vec!["evidence1".to_string()]))
            .add_item(RubricItemBuilder::new("TEST-002", "Second item", 0.9, vec!["evidence2".to_string()]))
            .build();

        assert_eq!(rubric.len(), 2);
        assert_eq!(rubric[0].id, "TEST-001");
        assert_eq!(rubric[1].id, "TEST-002");
    }

    #[test]
    fn test_judge_type() {
        let judge = MockJudge;
        assert_eq!(judge.judge_type(), JudgeType::Technical);
    }

    #[test]
    fn test_judge_rubric() {
        let judge = MockJudge;
        let rubric = judge.rubric();
        assert_eq!(rubric.len(), 1);
        assert_eq!(rubric[0].id, "TEST-001");
    }
}
