//! Judge verdicts and related structures
//!
//! Core verdict types for judge evaluations, including approval,
//! refinement, and rejection verdicts with associated metadata.

use schemars::JsonSchema;
use crate::judge_backup::risk::{RiskAssessment, RiskLevel};

/// Judge verdict on a working specification

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum JudgeVerdict {
    /// Approve the working specification for execution
    Approve {
        confidence: f64,
        reasoning: String,
        quality_score: f64,
        risk_assessment: RiskAssessment,
    },

    /// Request refinements before approval
    Refine {
        confidence: f64,
        reasoning: String,
        required_changes: Vec<RequiredChange>,
        priority: ChangePriority,
        estimated_effort: EffortEstimate,
    },

    /// Reject the working specification
    Reject {
        confidence: f64,
        reasoning: String,
        critical_issues: Vec<CriticalIssue>,
        alternative_approaches: Vec<String>,
    },
}

impl JudgeVerdict {
    /// Extract confidence from the verdict
    pub fn confidence(&self) -> f64 {
        match self {
            JudgeVerdict::Approve { confidence, .. } => *confidence,
            JudgeVerdict::Refine { confidence, .. } => *confidence,
            JudgeVerdict::Reject { .. } => 0.0, // Reject verdicts have no confidence
        }
    }

    /// Convert verdict to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            JudgeVerdict::Approve { .. } => "approve",
            JudgeVerdict::Refine { .. } => "refine",
            JudgeVerdict::Reject { .. } => "reject",
        }
    }
}

/// Required change for refinement

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct RequiredChange {
    pub category: ChangeCategory,
    pub description: String,
    pub impact: ChangeImpact,
    pub rationale: String,
}

/// Change category

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
enum ChangeCategory {
    Quality,
    Security,
    Performance,
    Maintainability,
    Scalability,
    Testing,
    Documentation,
    Requirements,
}

/// Change impact level

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum ChangeImpact {
    Minor,
    Moderate,
    Major,
    Breaking,
}

/// Change priority

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Copy)]
enum ChangePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Effort estimate for changes

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct EffortEstimate {
    pub person_hours: f64,
    pub complexity: ComplexityLevel,
    pub dependencies: Vec<String>,
}

/// Complexity level

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Copy)]
enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Critical issue for rejection

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct CriticalIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub description: String,
    pub evidence: Vec<String>,
}

/// Issue severity

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Copy)]
enum IssueSeverity {
    High,
    Critical,
}
