//! Judge Input/Output contracts for structured verdicts
//!
//! Defines the JSON schema-validated structures for judge prompts,
//! verdicts, violations, and evidence. These are used by inference
//! engines to produce deterministic, parseable outputs.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use serde_json;
use schemars::JsonSchema;
use crate::JudgeType;

/// Judge prompt with structured rubric and evidence
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgePrompt {
    /// Judge type (Constitutional, Technical, Quality, Integration)
    pub role: JudgeType,

    /// Primary objective for this judge
    pub objective: String,

    /// Structured rubric items for evaluation
    pub rubric: Vec<RubricItem>,

    /// Working specification evidence to evaluate
    pub evidence: WorkingSpecEvidence,

    /// JSON schema that verdict must conform to
    pub output_schema: String,
}

/// Judge verdict with structured findings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeVerdict {
    /// Overall score (0.0-1.0, higher = better compliance)
    pub score: f32,

    /// Verdict label
    pub label: VerdictLabel,

    /// Human-readable rationale
    pub rationale: String,

    /// Specific violations found
    pub violations: Vec<Violation>,

    /// References to evidence supporting the verdict
    pub evidence_refs: Vec<String>,
}

/// Verdict label categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum VerdictLabel {
    /// Approved for execution
    Pass,

    /// Rejected with critical issues
    Fail,

    /// Approved with required modifications
    NeedsInfo,

    /// Approved conditionally on specified requirements
    Conditional,
}

/// Individual violation found during evaluation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Violation {
    /// Rule identifier (e.g., "CAWS-001")
    pub rule_id: String,

    /// Severity level
    pub severity: Severity,

    /// Whether this violation is waivable
    pub waivable: bool,

    /// Human-readable description
    pub description: String,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum Severity {
    /// Minor issues, suggestions
    Info,

    /// Notable concerns requiring attention
    Low,

    /// Significant problems affecting quality
    Medium,

    /// Critical issues blocking execution
    High,

    /// Catastrophic failures requiring immediate rejection
    Critical,
}

/// Rubric item for judge evaluation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RubricItem {
    /// Criterion identifier
    pub id: String,

    /// Human-readable criterion description
    pub description: String,

    /// Weight in overall evaluation (0.0-1.0)
    pub weight: f32,

    /// Evidence requirements for this criterion
    pub evidence_requirements: Vec<String>,
}

/// Working specification evidence to evaluate
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkingSpecEvidence {
    /// Raw working specification text
    pub spec_text: String,

    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,

    /// Risk tier assessment
    pub risk_tier: String,

    /// Additional context and metadata
    pub context: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_judge_verdict_serialization() {
        let verdict = JudgeVerdict {
            score: 0.85,
            label: VerdictLabel::Conditional,
            rationale: "Good implementation but missing error handling".to_string(),
            violations: vec![Violation {
                rule_id: "CAWS-ERROR-001".to_string(),
                severity: Severity::Medium,
                waivable: true,
                description: "Error handling could be improved".to_string(),
            }],
            evidence_refs: vec!["line_42".to_string()],
        };

        let json = serde_json::to_string(&verdict).unwrap();
        let deserialized: JudgeVerdict = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.score, verdict.score);
        assert!(matches!(deserialized.label, VerdictLabel::Conditional));
        assert_eq!(deserialized.violations.len(), 1);
    }

    #[test]
    fn test_judge_prompt_structure() {
        let prompt = JudgePrompt {
            role: JudgeType::Constitutional,
            objective: "Evaluate ethical compliance".to_string(),
            rubric: vec![RubricItem {
                id: "ETH-001".to_string(),
                description: "No privacy violations".to_string(),
                weight: 0.8,
                evidence_requirements: vec!["Data handling policies".to_string()],
            }],
            evidence: WorkingSpecEvidence {
                spec_text: "Implement user authentication".to_string(),
                acceptance_criteria: vec!["Secure password storage".to_string()],
                risk_tier: "medium".to_string(),
                context: serde_json::Value::Object(serde_json::Map::new()),
            },
            output_schema: "{}".to_string(),
        };

        assert_eq!(prompt.role, JudgeType::Constitutional);
        assert_eq!(prompt.rubric.len(), 1);
        assert!(!prompt.evidence.spec_text.is_empty());
    }
}
