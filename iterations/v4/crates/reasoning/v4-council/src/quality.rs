//! Quality Judge
//!
//! Evaluates task completion and correctness against acceptance criteria.

use async_trait::async_trait;
use v4_types::council::JudgeType;

use crate::traits::{
    IssueSeverity, Judge, JudgeError, JudgeEvaluationResult, JudgeEvidence, JudgeIssue,
};

/// Quality judge that evaluates task completion and correctness
pub struct QualityJudge {
    /// Judge identifier
    id: String,
}

impl QualityJudge {
    /// Create a new quality judge
    pub fn new() -> Self {
        Self {
            id: format!("quality-{}", uuid::Uuid::new_v4()),
        }
    }

    /// Create with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Evaluate goal completion
    fn evaluate_goals(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();
        let goals = &evidence.task_spec.goals;

        if goals.is_empty() {
            issues.push(JudgeIssue {
                code: "GOAL-001".to_string(),
                severity: IssueSeverity::Minor,
                description: "No goals defined for task".to_string(),
                location: None,
                suggested_fix: Some("Define clear goals for the task".to_string()),
            });
            return (0.7, issues); // Uncertain without goals
        }

        // Check if we have evidence of work toward goals
        let has_operators = !evidence.proposed_operators.is_empty();
        let has_changes = !evidence.code_changes.is_empty();

        if !has_operators && !has_changes {
            issues.push(JudgeIssue {
                code: "GOAL-002".to_string(),
                severity: IssueSeverity::Major,
                description: "No operators or changes to achieve goals".to_string(),
                location: None,
                suggested_fix: Some("Add operators or make changes to achieve goals".to_string()),
            });
            return (0.3, issues);
        }

        // Score based on having evidence of work
        let score = if has_operators && has_changes {
            1.0
        } else if has_operators || has_changes {
            0.8
        } else {
            0.3
        };

        (score, issues)
    }

    /// Evaluate acceptance criteria
    fn evaluate_acceptance_criteria(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();
        let criteria = &evidence.task_spec.acceptance_criteria;

        if criteria.is_empty() {
            issues.push(JudgeIssue {
                code: "AC-001".to_string(),
                severity: IssueSeverity::Minor,
                description: "No acceptance criteria defined".to_string(),
                location: None,
                suggested_fix: Some("Define Given/When/Then acceptance criteria".to_string()),
            });
            return (0.7, issues); // Uncertain without criteria
        }

        // With acceptance criteria defined, we can at least verify they exist
        // Actual verification would require runtime test execution
        let score = if evidence.test_results.is_some() {
            let results = evidence.test_results.as_ref().unwrap();
            if results.all_passed() {
                1.0
            } else {
                // Proportional to pass rate
                results.pass_rate()
            }
        } else {
            // Without test results, give partial credit for having criteria
            0.75
        };

        (score, issues)
    }

    /// Evaluate completeness of changes
    fn evaluate_completeness(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        // Check for placeholder patterns in operators
        let placeholder_patterns = [
            "${",
            "TODO",
            "FIXME",
            "XXX",
            "HACK",
            "placeholder",
            "stub",
            "not implemented",
        ];

        let mut placeholder_count = 0;
        for op in &evidence.proposed_operators {
            for pattern in &placeholder_patterns {
                if op.to_lowercase().contains(&pattern.to_lowercase()) {
                    placeholder_count += 1;
                    break;
                }
            }
        }

        if placeholder_count > 0 {
            issues.push(JudgeIssue {
                code: "COMP-001".to_string(),
                severity: IssueSeverity::Major,
                description: format!(
                    "Found {} operators with placeholder patterns",
                    placeholder_count
                ),
                location: None,
                suggested_fix: Some("Replace placeholders with actual values".to_string()),
            });
        }

        // Check change completeness
        let has_substantive_changes = evidence
            .code_changes
            .iter()
            .any(|c| c.lines_added > 0 || c.lines_removed > 0);

        if !has_substantive_changes && !evidence.code_changes.is_empty() {
            issues.push(JudgeIssue {
                code: "COMP-002".to_string(),
                severity: IssueSeverity::Minor,
                description: "Code changes present but no lines modified".to_string(),
                location: None,
                suggested_fix: Some("Verify changes are complete".to_string()),
            });
        }

        let score = if placeholder_count > 0 {
            (1.0 - (placeholder_count as f64 * 0.2)).max(0.2)
        } else {
            1.0
        };

        (score, issues)
    }

    /// Evaluate description alignment
    fn evaluate_description_alignment(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        let description = evidence.task_spec.description.to_lowercase();
        let title = evidence.task_spec.title.to_lowercase();

        // Simple heuristic: check if operators seem to match the task
        let mut alignment_score: f64 = 0.8; // Base score

        // Boost score if operators align with description
        for op in &evidence.proposed_operators {
            let op_lower = op.to_lowercase();

            // Check for read operations when task mentions reading
            if (description.contains("read") || title.contains("read"))
                && op_lower.contains("read")
            {
                alignment_score = (alignment_score + 0.05).min(1.0);
            }

            // Check for write operations when task mentions writing
            if (description.contains("write")
                || title.contains("write")
                || description.contains("create"))
                && (op_lower.contains("write") || op_lower.contains("create"))
            {
                alignment_score = (alignment_score + 0.05).min(1.0);
            }

            // Check for search operations when task mentions searching
            if (description.contains("search")
                || title.contains("search")
                || description.contains("find"))
                && (op_lower.contains("search") || op_lower.contains("find"))
            {
                alignment_score = (alignment_score + 0.05).min(1.0);
            }
        }

        // Penalize if no alignment detected
        if evidence.proposed_operators.len() > 3 && alignment_score <= 0.8 {
            issues.push(JudgeIssue {
                code: "ALIGN-001".to_string(),
                severity: IssueSeverity::Minor,
                description: "Operator types may not align well with task description".to_string(),
                location: None,
                suggested_fix: Some("Review operators for relevance to task".to_string()),
            });
        }

        (alignment_score, issues)
    }

    /// Evaluate output quality
    fn evaluate_output_quality(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        // Check test results quality
        if let Some(ref results) = evidence.test_results {
            if results.skipped > results.total_tests / 4 {
                issues.push(JudgeIssue {
                    code: "QUAL-001".to_string(),
                    severity: IssueSeverity::Minor,
                    description: format!(
                        "High number of skipped tests: {}/{}",
                        results.skipped, results.total_tests
                    ),
                    location: None,
                    suggested_fix: Some("Review why tests are being skipped".to_string()),
                });
            }
        }

        // Check change quality
        let total_additions: u32 = evidence.code_changes.iter().map(|c| c.lines_added).sum();
        let total_deletions: u32 = evidence.code_changes.iter().map(|c| c.lines_removed).sum();

        if total_additions > 1000 || total_deletions > 1000 {
            issues.push(JudgeIssue {
                code: "QUAL-002".to_string(),
                severity: IssueSeverity::Minor,
                description: format!(
                    "Large change volume: +{} / -{} lines",
                    total_additions, total_deletions
                ),
                location: None,
                suggested_fix: Some("Consider breaking into smaller changes".to_string()),
            });
        }

        let score = if issues.is_empty() {
            1.0
        } else {
            (1.0 - (issues.len() as f64 * 0.1)).max(0.6)
        };

        (score, issues)
    }
}

impl Default for QualityJudge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Judge for QualityJudge {
    fn judge_type(&self) -> JudgeType {
        JudgeType::Quality
    }

    fn judge_id(&self) -> &str {
        &self.id
    }

    async fn evaluate(&self, evidence: &JudgeEvidence) -> Result<JudgeEvaluationResult, JudgeError> {
        let start = std::time::Instant::now();

        let mut all_issues = Vec::new();
        let mut scores = Vec::new();

        // Run all evaluations
        let (goal_score, goal_issues) = self.evaluate_goals(evidence);
        scores.push(("goals", goal_score, 0.25));
        all_issues.extend(goal_issues);

        let (ac_score, ac_issues) = self.evaluate_acceptance_criteria(evidence);
        scores.push(("acceptance_criteria", ac_score, 0.30));
        all_issues.extend(ac_issues);

        let (completeness_score, completeness_issues) = self.evaluate_completeness(evidence);
        scores.push(("completeness", completeness_score, 0.20));
        all_issues.extend(completeness_issues);

        let (alignment_score, alignment_issues) = self.evaluate_description_alignment(evidence);
        scores.push(("alignment", alignment_score, 0.10));
        all_issues.extend(alignment_issues);

        let (output_score, output_issues) = self.evaluate_output_quality(evidence);
        scores.push(("output_quality", output_score, 0.15));
        all_issues.extend(output_issues);

        // Calculate weighted score
        let total_weight: f64 = scores.iter().map(|(_, _, w)| w).sum();
        let weighted_score: f64 = scores.iter().map(|(_, s, w)| s * w).sum::<f64>() / total_weight;

        // Build reasoning
        let reasoning = scores
            .iter()
            .map(|(name, score, _)| format!("{}: {:.2}", name, score))
            .collect::<Vec<_>>()
            .join(", ");

        // Build recommendations
        let mut recommendations = Vec::new();
        if goal_score < 0.8 {
            recommendations.push("Clarify task goals".to_string());
        }
        if ac_score < 0.8 {
            recommendations.push("Add or verify acceptance criteria".to_string());
        }
        if completeness_score < 1.0 {
            recommendations.push("Remove placeholder values".to_string());
        }

        // Confidence based on evidence available
        let confidence = match (
            evidence.test_results.is_some(),
            !evidence.code_changes.is_empty(),
        ) {
            (true, true) => 0.95,
            (true, false) | (false, true) => 0.8,
            (false, false) => 0.6,
        };

        Ok(JudgeEvaluationResult {
            judge_id: self.id.clone(),
            judge_type: JudgeType::Quality,
            score: weighted_score,
            confidence,
            reasoning: format!("Quality evaluation: {}", reasoning),
            issues: all_issues,
            recommendations,
            processing_time_ms: start.elapsed().as_millis() as u64,
            evaluated_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{ChangeType, CodeChange, TestEvidence};
    use v4_types::task::{AcceptanceCriterion, TaskConstraints, TaskSpec};

    fn make_task_spec() -> TaskSpec {
        TaskSpec {
            id: "task-123".to_string(),
            version: "1.0".to_string(),
            title: "Read configuration".to_string(),
            description: "Read and parse the configuration file".to_string(),
            goals: vec![
                "Parse config.json".to_string(),
                "Validate settings".to_string(),
            ],
            risk_tier: 2,
            constraints: TaskConstraints::default(),
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    id: "AC-1".to_string(),
                    given: "A valid config file exists".to_string(),
                    when: "The parser runs".to_string(),
                    then: "Settings are loaded correctly".to_string(),
                },
            ],
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_complete_task() {
        let judge = QualityJudge::new();
        let mut evidence = JudgeEvidence::new(make_task_spec());
        evidence.proposed_operators = vec!["Seek::ReadFile".to_string()];
        evidence.code_changes = vec![CodeChange {
            path: "src/config.rs".to_string(),
            change_type: ChangeType::Modified,
            lines_added: 50,
            lines_removed: 10,
            description: "Added config parsing".to_string(),
        }];
        evidence.test_results = Some(TestEvidence {
            total_tests: 10,
            passed: 10,
            failed: 0,
            skipped: 0,
            coverage: Some(85.0),
            failed_tests: vec![],
        });

        let result = judge.evaluate(&evidence).await.unwrap();
        assert!(result.score >= 0.8);
        assert!(!result.is_veto());
    }

    #[tokio::test]
    async fn test_incomplete_task() {
        let judge = QualityJudge::new();
        let mut evidence = JudgeEvidence::new(make_task_spec());
        evidence.proposed_operators = vec!["TODO: ${implement_later}".to_string()];

        let result = judge.evaluate(&evidence).await.unwrap();
        // Has operators (even with TODO) so goals pass with 0.8
        // Completeness is penalized for placeholder
        assert!(result.score < 0.95);
        assert!(result.issues.iter().any(|i| i.code == "COMP-001"));
    }

    #[tokio::test]
    async fn test_no_goals() {
        let judge = QualityJudge::new();
        let mut spec = make_task_spec();
        spec.goals.clear();
        let evidence = JudgeEvidence::new(spec);

        let result = judge.evaluate(&evidence).await.unwrap();
        assert!(result.issues.iter().any(|i| i.code == "GOAL-001"));
    }

    #[tokio::test]
    async fn test_no_evidence() {
        let judge = QualityJudge::new();
        let evidence = JudgeEvidence::new(make_task_spec());

        let result = judge.evaluate(&evidence).await.unwrap();
        // No operators and no changes: goals score 0.3
        // Weighted: 0.3*0.25 + 0.75*0.30 + 1.0*0.20 + 0.8*0.10 + 1.0*0.15 = ~0.73
        assert!(result.score < 0.8);
        assert!(result.issues.iter().any(|i| i.code == "GOAL-002"));
    }
}
