//! Technical Judge
//!
//! Evaluates code quality, architecture, and security.

use async_trait::async_trait;
use v4_types::council::JudgeType;

use crate::traits::{
    IssueSeverity, Judge, JudgeError, JudgeEvaluationResult, JudgeEvidence, JudgeIssue,
};

/// Technical judge that evaluates code quality and architecture
pub struct TechnicalJudge {
    /// Judge identifier
    id: String,
    /// Minimum coverage threshold
    min_coverage: f64,
    /// Maximum complexity allowed
    max_complexity: u32,
}

impl TechnicalJudge {
    /// Create a new technical judge
    pub fn new() -> Self {
        Self {
            id: format!("technical-{}", uuid::Uuid::new_v4()),
            min_coverage: 0.80,
            max_complexity: 10,
        }
    }

    /// Create with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            min_coverage: 0.80,
            max_complexity: 10,
        }
    }

    /// Set minimum coverage threshold
    pub fn with_min_coverage(mut self, coverage: f64) -> Self {
        self.min_coverage = coverage;
        self
    }

    /// Evaluate test coverage
    fn evaluate_coverage(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        let score = match &evidence.test_results {
            Some(results) => {
                if let Some(coverage) = results.coverage {
                    if coverage < self.min_coverage * 100.0 {
                        issues.push(JudgeIssue {
                            code: "COV-001".to_string(),
                            severity: IssueSeverity::Major,
                            description: format!(
                                "Code coverage {:.1}% is below minimum {:.1}%",
                                coverage,
                                self.min_coverage * 100.0
                            ),
                            location: None,
                            suggested_fix: Some("Add more unit tests".to_string()),
                        });
                        coverage / 100.0 / self.min_coverage // Proportional score
                    } else {
                        1.0
                    }
                } else {
                    // No coverage data - assume adequate
                    0.8
                }
            }
            None => {
                issues.push(JudgeIssue {
                    code: "COV-002".to_string(),
                    severity: IssueSeverity::Minor,
                    description: "No test results provided".to_string(),
                    location: None,
                    suggested_fix: Some("Run tests before evaluation".to_string()),
                });
                0.7 // Uncertain without test data
            }
        };

        (score.min(1.0), issues)
    }

    /// Evaluate test pass rate
    fn evaluate_test_pass_rate(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        let score = match &evidence.test_results {
            Some(results) => {
                let pass_rate = results.pass_rate();
                if results.failed > 0 {
                    issues.push(JudgeIssue {
                        code: "TEST-001".to_string(),
                        severity: if pass_rate < 0.9 {
                            IssueSeverity::Critical
                        } else {
                            IssueSeverity::Major
                        },
                        description: format!(
                            "{} tests failed out of {}",
                            results.failed, results.total_tests
                        ),
                        location: None,
                        suggested_fix: Some(format!(
                            "Fix failing tests: {:?}",
                            results.failed_tests
                        )),
                    });
                }
                pass_rate
            }
            None => 0.7, // Assume some tests pass
        };

        (score, issues)
    }

    /// Evaluate code change patterns
    fn evaluate_change_patterns(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();
        let mut deductions = 0.0;

        for change in &evidence.code_changes {
            // Check for very large changes
            let total_lines = change.lines_added + change.lines_removed;
            if total_lines > 500 {
                issues.push(JudgeIssue {
                    code: "CHURN-001".to_string(),
                    severity: IssueSeverity::Minor,
                    description: format!(
                        "Large change ({} lines) in {}",
                        total_lines, change.path
                    ),
                    location: Some(change.path.clone()),
                    suggested_fix: Some("Consider breaking into smaller changes".to_string()),
                });
                deductions += 0.05;
            }

            // Check for deletions without additions (suspicious)
            if change.lines_removed > 0 && change.lines_added == 0 {
                if change.lines_removed > 100 {
                    issues.push(JudgeIssue {
                        code: "CHURN-002".to_string(),
                        severity: IssueSeverity::Minor,
                        description: format!(
                            "Large deletion ({} lines) in {} with no additions",
                            change.lines_removed, change.path
                        ),
                        location: Some(change.path.clone()),
                        suggested_fix: Some("Verify deletion is intentional".to_string()),
                    });
                    deductions += 0.05;
                }
            }
        }

        // Check overall change count
        if evidence.code_changes.len() > 20 {
            issues.push(JudgeIssue {
                code: "CHURN-003".to_string(),
                severity: IssueSeverity::Major,
                description: format!("Many files changed: {}", evidence.code_changes.len()),
                location: None,
                suggested_fix: Some("Consider smaller, focused changes".to_string()),
            });
            deductions += 0.1;
        }

        let score = (1.0_f64 - deductions).max(0.0);
        (score, issues)
    }

    /// Evaluate security patterns
    fn evaluate_security(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        // Check for security-sensitive patterns in operators
        let security_patterns = [
            ("exec", "Command execution"),
            ("eval", "Code evaluation"),
            ("shell", "Shell access"),
            ("sql", "SQL query"),
            ("password", "Password handling"),
            ("token", "Token handling"),
            ("secret", "Secret handling"),
        ];

        for op in &evidence.proposed_operators {
            let op_lower = op.to_lowercase();
            for (pattern, concern) in &security_patterns {
                if op_lower.contains(pattern) {
                    issues.push(JudgeIssue {
                        code: "SEC-001".to_string(),
                        severity: IssueSeverity::Minor,
                        description: format!("Security-sensitive operation: {} ({})", op, concern),
                        location: None,
                        suggested_fix: Some(
                            "Ensure proper input validation and sanitization".to_string(),
                        ),
                    });
                }
            }
        }

        let score = if issues.is_empty() {
            1.0
        } else {
            (1.0 - (issues.len() as f64 * 0.1)).max(0.5)
        };

        (score, issues)
    }

    /// Evaluate architecture patterns
    fn evaluate_architecture(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        // Check for changes to critical files
        let critical_patterns = [
            ("Cargo.toml", "Package manifest"),
            ("package.json", "Package manifest"),
            ("*.lock", "Lock file"),
            ("docker", "Container configuration"),
            ("ci", "CI configuration"),
            ("github/workflows", "CI workflows"),
        ];

        for change in &evidence.code_changes {
            for (pattern, description) in &critical_patterns {
                let matches = if pattern.contains('*') {
                    change.path.ends_with(pattern.trim_start_matches('*'))
                } else {
                    change.path.to_lowercase().contains(&pattern.to_lowercase())
                };

                if matches {
                    issues.push(JudgeIssue {
                        code: "ARCH-001".to_string(),
                        severity: IssueSeverity::Minor,
                        description: format!("{} modified: {}", description, change.path),
                        location: Some(change.path.clone()),
                        suggested_fix: Some("Ensure changes are intentional".to_string()),
                    });
                }
            }
        }

        let score = if issues.is_empty() {
            1.0
        } else {
            (1.0 - (issues.len() as f64 * 0.05)).max(0.7)
        };

        (score, issues)
    }
}

impl Default for TechnicalJudge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Judge for TechnicalJudge {
    fn judge_type(&self) -> JudgeType {
        JudgeType::Technical
    }

    fn judge_id(&self) -> &str {
        &self.id
    }

    async fn evaluate(&self, evidence: &JudgeEvidence) -> Result<JudgeEvaluationResult, JudgeError> {
        let start = std::time::Instant::now();

        let mut all_issues = Vec::new();
        let mut scores = Vec::new();

        // Run all evaluations
        let (coverage_score, coverage_issues) = self.evaluate_coverage(evidence);
        scores.push(("coverage", coverage_score, 0.20));
        all_issues.extend(coverage_issues);

        let (test_score, test_issues) = self.evaluate_test_pass_rate(evidence);
        scores.push(("test_pass_rate", test_score, 0.25));
        all_issues.extend(test_issues);

        let (change_score, change_issues) = self.evaluate_change_patterns(evidence);
        scores.push(("change_patterns", change_score, 0.15));
        all_issues.extend(change_issues);

        let (security_score, security_issues) = self.evaluate_security(evidence);
        scores.push(("security", security_score, 0.25));
        all_issues.extend(security_issues);

        let (arch_score, arch_issues) = self.evaluate_architecture(evidence);
        scores.push(("architecture", arch_score, 0.15));
        all_issues.extend(arch_issues);

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
        if coverage_score < 0.8 {
            recommendations.push("Increase test coverage".to_string());
        }
        if test_score < 1.0 {
            recommendations.push("Fix failing tests".to_string());
        }
        if security_score < 1.0 {
            recommendations.push("Review security-sensitive operations".to_string());
        }

        // Confidence based on test data availability
        let confidence = if evidence.test_results.is_some() {
            0.95
        } else {
            0.7
        };

        Ok(JudgeEvaluationResult {
            judge_id: self.id.clone(),
            judge_type: JudgeType::Technical,
            score: weighted_score,
            confidence,
            reasoning: format!("Technical evaluation: {}", reasoning),
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
            title: "Test task".to_string(),
            description: "A test task".to_string(),
            goals: vec!["Goal 1".to_string()],
            risk_tier: 2,
            constraints: TaskConstraints::default(),
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-1".to_string(),
                given: "Given state".to_string(),
                when: "When action".to_string(),
                then: "Then result".to_string(),
            }],
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_perfect_tests() {
        let judge = TechnicalJudge::new();
        let mut evidence = JudgeEvidence::new(make_task_spec());
        evidence.test_results = Some(TestEvidence {
            total_tests: 100,
            passed: 100,
            failed: 0,
            skipped: 0,
            coverage: Some(90.0),
            failed_tests: vec![],
        });

        let result = judge.evaluate(&evidence).await.unwrap();
        assert!(result.score >= 0.9);
        assert!(!result.is_veto());
    }

    #[tokio::test]
    async fn test_failing_tests() {
        let judge = TechnicalJudge::new();
        let mut evidence = JudgeEvidence::new(make_task_spec());
        evidence.test_results = Some(TestEvidence {
            total_tests: 100,
            passed: 50,
            failed: 50,
            skipped: 0,
            coverage: Some(80.0),
            failed_tests: vec!["test_1".to_string()],
        });

        let result = judge.evaluate(&evidence).await.unwrap();
        // Test pass rate is 50%, coverage is good
        // Weighted: 0.5*0.25 + 0.8*0.25 + 1.0*0.20 + 1.0*0.15 + 1.0*0.15 = 0.825
        assert!(result.score < 0.9);
        assert!(result.issues.iter().any(|i| i.code == "TEST-001"));
    }

    #[tokio::test]
    async fn test_low_coverage() {
        let judge = TechnicalJudge::new();
        let mut evidence = JudgeEvidence::new(make_task_spec());
        evidence.test_results = Some(TestEvidence {
            total_tests: 10,
            passed: 10,
            failed: 0,
            skipped: 0,
            coverage: Some(30.0), // Very low coverage
            failed_tests: vec![],
        });

        let result = judge.evaluate(&evidence).await.unwrap();
        assert!(result.issues.iter().any(|i| i.code == "COV-001"));
    }

    #[tokio::test]
    async fn test_large_changes() {
        let judge = TechnicalJudge::new();
        let mut evidence = JudgeEvidence::new(make_task_spec());
        evidence.code_changes = vec![CodeChange {
            path: "large_file.rs".to_string(),
            change_type: ChangeType::Modified,
            lines_added: 600,
            lines_removed: 0,
            description: "Large addition".to_string(),
        }];

        let result = judge.evaluate(&evidence).await.unwrap();
        assert!(result.issues.iter().any(|i| i.code == "CHURN-001"));
    }
}
