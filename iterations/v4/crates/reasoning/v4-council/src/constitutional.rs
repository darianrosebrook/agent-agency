//! Constitutional Judge
//!
//! Evaluates ethical compliance and CAWS invariant adherence.

use async_trait::async_trait;
use v4_types::council::JudgeType;

use crate::traits::{
    IssueSeverity, Judge, JudgeError, JudgeEvaluationResult, JudgeEvidence, JudgeIssue,
};

/// Constitutional judge that evaluates ethical compliance and invariants
pub struct ConstitutionalJudge {
    /// Judge identifier
    id: String,
    /// Whether to be strict about invariant violations
    strict_mode: bool,
}

impl ConstitutionalJudge {
    /// Create a new constitutional judge
    pub fn new() -> Self {
        Self {
            id: format!("constitutional-{}", uuid::Uuid::new_v4()),
            strict_mode: true,
        }
    }

    /// Create with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            strict_mode: true,
        }
    }

    /// Disable strict mode
    pub fn lenient(mut self) -> Self {
        self.strict_mode = false;
        self
    }

    /// Evaluate risk tier compliance
    fn evaluate_risk_tier(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();
        let risk_tier = evidence.task_spec.risk_tier;

        // Risk tier 1-2: Low risk, allow most operations
        // Risk tier 3: Medium risk, some restrictions
        // Risk tier 4-5: High risk, strict oversight required

        let score = match risk_tier {
            1..=2 => 1.0,
            3 => {
                // Check for breaking changes
                if evidence.task_spec.constraints.allow_breaking_changes {
                    issues.push(JudgeIssue {
                        code: "RISK-001".to_string(),
                        severity: IssueSeverity::Major,
                        description: "Medium-risk task allows breaking changes".to_string(),
                        location: None,
                        suggested_fix: Some(
                            "Consider disabling breaking changes for this risk tier".to_string(),
                        ),
                    });
                    0.7
                } else {
                    0.9
                }
            }
            4 => {
                issues.push(JudgeIssue {
                    code: "RISK-002".to_string(),
                    severity: IssueSeverity::Major,
                    description: "High-risk task (tier 4) requires additional review".to_string(),
                    location: None,
                    suggested_fix: Some("Ensure manual oversight is in place".to_string()),
                });
                0.6
            }
            5 => {
                issues.push(JudgeIssue {
                    code: "RISK-003".to_string(),
                    severity: IssueSeverity::Critical,
                    description: "Critical-risk task (tier 5) may not be suitable for automation"
                        .to_string(),
                    location: None,
                    suggested_fix: Some("Consider manual execution".to_string()),
                });
                0.3
            }
            _ => {
                issues.push(JudgeIssue {
                    code: "RISK-004".to_string(),
                    severity: IssueSeverity::Critical,
                    description: format!("Invalid risk tier: {}", risk_tier),
                    location: None,
                    suggested_fix: Some("Risk tier must be 1-5".to_string()),
                });
                0.0
            }
        };

        (score, issues)
    }

    /// Evaluate path safety
    fn evaluate_path_safety(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();
        let mut deductions = 0.0;

        // Check for dangerous paths in code changes
        let dangerous_patterns = [
            ("/etc/", "System configuration"),
            ("/var/", "System data"),
            ("~/.ssh", "SSH credentials"),
            (".env", "Environment secrets"),
            ("credentials", "Credentials file"),
            ("password", "Password file"),
            ("secret", "Secret file"),
            ("private", "Private key"),
        ];

        for change in &evidence.code_changes {
            for (pattern, description) in &dangerous_patterns {
                if change.path.to_lowercase().contains(pattern) {
                    issues.push(JudgeIssue {
                        code: "PATH-001".to_string(),
                        severity: IssueSeverity::Critical,
                        description: format!("{} path detected: {}", description, change.path),
                        location: Some(change.path.clone()),
                        suggested_fix: Some("Remove sensitive path from changes".to_string()),
                    });
                    deductions += 0.3;
                }
            }
        }

        let score = (1.0_f64 - deductions).max(0.0);
        (score, issues)
    }

    /// Evaluate constraint compliance
    fn evaluate_constraints(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();
        let constraints = &evidence.task_spec.constraints;

        // Check max files constraint
        if let Some(max_files) = constraints.max_files {
            let file_count = evidence.code_changes.len() as u32;
            if file_count > max_files {
                issues.push(JudgeIssue {
                    code: "CONST-001".to_string(),
                    severity: IssueSeverity::Major,
                    description: format!(
                        "File count {} exceeds maximum {}",
                        file_count, max_files
                    ),
                    location: None,
                    suggested_fix: Some("Reduce number of files changed".to_string()),
                });
            }
        }

        // Check max LOC constraint
        if let Some(max_loc) = constraints.max_loc {
            let total_loc: u32 = evidence
                .code_changes
                .iter()
                .map(|c| c.lines_added + c.lines_removed)
                .sum();
            if total_loc > max_loc {
                issues.push(JudgeIssue {
                    code: "CONST-002".to_string(),
                    severity: IssueSeverity::Major,
                    description: format!("LOC {} exceeds maximum {}", total_loc, max_loc),
                    location: None,
                    suggested_fix: Some("Reduce scope of changes".to_string()),
                });
            }
        }

        // Check blocked paths
        for change in &evidence.code_changes {
            for blocked in &constraints.blocked_paths {
                // Simple glob matching (just * wildcards)
                let pattern = blocked.replace("**", ".*").replace("*", "[^/]*");
                if let Ok(re) = regex_lite::Regex::new(&pattern) {
                    if re.is_match(&change.path) {
                        issues.push(JudgeIssue {
                            code: "CONST-003".to_string(),
                            severity: IssueSeverity::Critical,
                            description: format!(
                                "Path {} matches blocked pattern {}",
                                change.path, blocked
                            ),
                            location: Some(change.path.clone()),
                            suggested_fix: Some("Remove blocked path from changes".to_string()),
                        });
                    }
                }
            }
        }

        let critical_count = issues
            .iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Critical))
            .count();
        let major_count = issues
            .iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Major))
            .count();

        let score = if critical_count > 0 {
            0.0
        } else {
            (1.0 - (major_count as f64 * 0.2)).max(0.0)
        };

        (score, issues)
    }

    /// Evaluate ethical concerns
    fn evaluate_ethics(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
        let mut issues = Vec::new();

        // Check for concerning patterns in description
        let concerning_patterns = [
            ("bypass", "Attempt to bypass security"),
            ("disable.*security", "Attempt to disable security"),
            ("remove.*auth", "Attempt to remove authentication"),
            ("delete.*all", "Bulk deletion operation"),
            ("drop.*table", "Database table drop"),
            ("truncate", "Database truncation"),
        ];

        let description = evidence.task_spec.description.to_lowercase();
        for (pattern, concern) in &concerning_patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(&description) {
                    issues.push(JudgeIssue {
                        code: "ETHICS-001".to_string(),
                        severity: IssueSeverity::Critical,
                        description: format!("Ethical concern: {}", concern),
                        location: None,
                        suggested_fix: Some("Review task intent and ensure proper authorization"
                            .to_string()),
                    });
                }
            }
        }

        let score = if issues.is_empty() { 1.0 } else { 0.2 };
        (score, issues)
    }
}

impl Default for ConstitutionalJudge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Judge for ConstitutionalJudge {
    fn judge_type(&self) -> JudgeType {
        JudgeType::Constitutional
    }

    fn judge_id(&self) -> &str {
        &self.id
    }

    async fn evaluate(&self, evidence: &JudgeEvidence) -> Result<JudgeEvaluationResult, JudgeError> {
        let start = std::time::Instant::now();

        let mut all_issues = Vec::new();
        let mut scores = Vec::new();

        // Run all evaluations
        let (risk_score, risk_issues) = self.evaluate_risk_tier(evidence);
        scores.push(("risk_tier", risk_score, 0.25));
        all_issues.extend(risk_issues);

        let (path_score, path_issues) = self.evaluate_path_safety(evidence);
        scores.push(("path_safety", path_score, 0.25));
        all_issues.extend(path_issues);

        let (constraint_score, constraint_issues) = self.evaluate_constraints(evidence);
        scores.push(("constraints", constraint_score, 0.25));
        all_issues.extend(constraint_issues);

        let (ethics_score, ethics_issues) = self.evaluate_ethics(evidence);
        scores.push(("ethics", ethics_score, 0.25));
        all_issues.extend(ethics_issues);

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
        if risk_score < 0.7 {
            recommendations.push("Consider reducing task risk tier".to_string());
        }
        if path_score < 1.0 {
            recommendations.push("Review file paths for safety concerns".to_string());
        }
        if constraint_score < 1.0 {
            recommendations.push("Ensure all constraints are satisfied".to_string());
        }
        if ethics_score < 1.0 {
            recommendations.push("Review task for ethical concerns".to_string());
        }

        // Confidence based on amount of evidence
        let confidence = if evidence.code_changes.is_empty() && evidence.proposed_operators.is_empty()
        {
            0.5 // Less confident with minimal evidence
        } else {
            0.9
        };

        Ok(JudgeEvaluationResult {
            judge_id: self.id.clone(),
            judge_type: JudgeType::Constitutional,
            score: weighted_score,
            confidence,
            reasoning: format!("Constitutional evaluation: {}", reasoning),
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
    use crate::traits::{ChangeType, CodeChange};
    use v4_types::task::{AcceptanceCriterion, TaskConstraints, TaskSpec};

    fn make_task_spec(risk_tier: u8) -> TaskSpec {
        TaskSpec {
            id: "task-123".to_string(),
            version: "1.0".to_string(),
            title: "Test task".to_string(),
            description: "A test task".to_string(),
            goals: vec!["Goal 1".to_string()],
            risk_tier,
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
    async fn test_low_risk_task() {
        let judge = ConstitutionalJudge::new();
        let evidence = JudgeEvidence::new(make_task_spec(1));

        let result = judge.evaluate(&evidence).await.unwrap();
        assert!(result.score >= 0.9);
        assert!(!result.is_veto());
    }

    #[tokio::test]
    async fn test_high_risk_task() {
        let judge = ConstitutionalJudge::new();
        let evidence = JudgeEvidence::new(make_task_spec(5));

        let result = judge.evaluate(&evidence).await.unwrap();
        // Risk tier 5 scores 0.3 (25% weight), other factors score 1.0 (75% weight)
        // Weighted score: 0.3*0.25 + 1.0*0.75 = 0.825
        assert!(result.score < 0.9);
        assert!(result.issues.iter().any(|i| i.code == "RISK-003"));
    }

    #[tokio::test]
    async fn test_dangerous_path() {
        let judge = ConstitutionalJudge::new();
        let mut evidence = JudgeEvidence::new(make_task_spec(1));
        evidence.code_changes.push(CodeChange {
            path: "~/.ssh/id_rsa".to_string(),
            change_type: ChangeType::Modified,
            lines_added: 10,
            lines_removed: 5,
            description: "Modified SSH key".to_string(),
        });

        let result = judge.evaluate(&evidence).await.unwrap();
        // Path safety scores 0.7 (25% weight) due to -0.3 deduction for dangerous path
        // Other factors score 1.0 (75% weight)
        // Weighted score: (1.0 + 0.7 + 1.0 + 1.0) / 4 = 0.925
        assert!(result.score < 1.0);
        assert!(result.issues.iter().any(|i| i.code == "PATH-001"));
    }

    #[tokio::test]
    async fn test_ethical_concern() {
        let judge = ConstitutionalJudge::new();
        let mut spec = make_task_spec(1);
        spec.description = "Bypass security checks and delete all user data".to_string();
        let evidence = JudgeEvidence::new(spec);

        let result = judge.evaluate(&evidence).await.unwrap();
        // Ethics score is 0.2 (25% weight), other factors score 1.0 (75% weight)
        // Weighted score: 0.2*0.25 + 1.0*0.75 = 0.8
        assert!(result.score < 0.9);
        assert!(result.issues.iter().any(|i| i.code == "ETHICS-001"));
    }
}
