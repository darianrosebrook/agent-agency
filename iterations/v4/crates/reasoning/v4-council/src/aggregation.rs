//! Verdict Aggregation
//!
//! Aggregates judge results into a final council verdict with veto logic.

use serde::{Deserialize, Serialize};
use v4_types::council::{CouncilVerdict, JudgeScores, JudgeType};

use crate::traits::JudgeEvaluationResult;

/// Configuration for verdict aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Threshold for automatic veto (any score below this = rejection)
    pub veto_threshold: f64,
    /// Threshold for approval (aggregate must be >= this)
    pub approval_threshold: f64,
    /// Threshold for conditional approval
    pub conditional_threshold: f64,
    /// Weight for constitutional judge
    pub constitutional_weight: f64,
    /// Weight for technical judge
    pub technical_weight: f64,
    /// Weight for quality judge
    pub quality_weight: f64,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            veto_threshold: 0.5,
            approval_threshold: 0.90,
            conditional_threshold: 0.70,
            constitutional_weight: 0.30,
            technical_weight: 0.35,
            quality_weight: 0.35,
        }
    }
}

impl AggregationConfig {
    /// Create a strict configuration
    pub fn strict() -> Self {
        Self {
            veto_threshold: 0.6,
            approval_threshold: 0.95,
            conditional_threshold: 0.80,
            ..Default::default()
        }
    }

    /// Create a lenient configuration
    pub fn lenient() -> Self {
        Self {
            veto_threshold: 0.4,
            approval_threshold: 0.85,
            conditional_threshold: 0.65,
            ..Default::default()
        }
    }
}

/// Aggregated verdict with full details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedVerdict {
    /// The final verdict
    pub verdict: CouncilVerdict,
    /// Individual judge scores
    pub scores: JudgeScores,
    /// Whether any judge vetoed
    pub vetoed: bool,
    /// Which judge(s) vetoed (if any)
    pub vetoing_judges: Vec<JudgeType>,
    /// Combined reasoning from all judges
    pub reasoning: String,
    /// All issues from all judges
    pub all_issues: Vec<AggregatedIssue>,
    /// Recommendations from all judges
    pub recommendations: Vec<String>,
    /// Confidence in the verdict (average of judge confidences)
    pub confidence: f64,
    /// Total processing time
    pub total_processing_time_ms: u64,
}

/// An issue aggregated from judges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedIssue {
    /// Source judge
    pub source: JudgeType,
    /// Issue code
    pub code: String,
    /// Description
    pub description: String,
    /// Severity as string
    pub severity: String,
}

/// Aggregate results from all judges into a verdict
///
/// Veto logic: Any score < veto_threshold (default 0.5) = automatic rejection
pub fn aggregate(
    results: &[JudgeEvaluationResult],
    config: &AggregationConfig,
) -> Result<AggregatedVerdict, AggregationError> {
    // Validate we have all three judges
    let constitutional = results
        .iter()
        .find(|r| r.judge_type == JudgeType::Constitutional);
    let technical = results
        .iter()
        .find(|r| r.judge_type == JudgeType::Technical);
    let quality = results.iter().find(|r| r.judge_type == JudgeType::Quality);

    // All three judges are required
    let constitutional =
        constitutional.ok_or_else(|| AggregationError::MissingJudge(JudgeType::Constitutional))?;
    let technical =
        technical.ok_or_else(|| AggregationError::MissingJudge(JudgeType::Technical))?;
    let quality = quality.ok_or_else(|| AggregationError::MissingJudge(JudgeType::Quality))?;

    // Check for vetoes
    let mut vetoing_judges = Vec::new();
    if constitutional.score < config.veto_threshold {
        vetoing_judges.push(JudgeType::Constitutional);
    }
    if technical.score < config.veto_threshold {
        vetoing_judges.push(JudgeType::Technical);
    }
    if quality.score < config.veto_threshold {
        vetoing_judges.push(JudgeType::Quality);
    }

    let vetoed = !vetoing_judges.is_empty();

    // Calculate weighted aggregate
    let total_weight =
        config.constitutional_weight + config.technical_weight + config.quality_weight;
    let aggregate = (constitutional.score * config.constitutional_weight
        + technical.score * config.technical_weight
        + quality.score * config.quality_weight)
        / total_weight;

    // Build scores
    let scores = JudgeScores {
        constitutional: constitutional.score,
        technical: technical.score,
        quality: quality.score,
        aggregate,
    };

    // Determine verdict
    let verdict = if vetoed {
        // Veto = automatic rejection
        let reasons: Vec<String> = vetoing_judges
            .iter()
            .map(|j| format!("{:?} judge vetoed (score < {})", j, config.veto_threshold))
            .collect();
        CouncilVerdict::Rejected {
            score: aggregate,
            reasons,
        }
    } else if aggregate >= config.approval_threshold {
        CouncilVerdict::Approved { score: aggregate }
    } else if aggregate >= config.conditional_threshold {
        // Gather requirements from issues
        let requirements: Vec<String> = results
            .iter()
            .flat_map(|r| &r.recommendations)
            .cloned()
            .take(5)
            .collect();
        CouncilVerdict::ConditionalApproval {
            score: aggregate,
            requirements,
        }
    } else {
        // Below conditional threshold
        let reasons: Vec<String> = results
            .iter()
            .flat_map(|r| {
                r.issues
                    .iter()
                    .filter(|i| matches!(i.severity, crate::traits::IssueSeverity::Critical | crate::traits::IssueSeverity::Major))
                    .map(|i| i.description.clone())
            })
            .take(5)
            .collect();
        CouncilVerdict::Rejected {
            score: aggregate,
            reasons,
        }
    };

    // Combine reasoning
    let reasoning = format!(
        "Constitutional: {} ({:.2}), Technical: {} ({:.2}), Quality: {} ({:.2}). Aggregate: {:.2}",
        if constitutional.score >= config.veto_threshold {
            "PASS"
        } else {
            "VETO"
        },
        constitutional.score,
        if technical.score >= config.veto_threshold {
            "PASS"
        } else {
            "VETO"
        },
        technical.score,
        if quality.score >= config.veto_threshold {
            "PASS"
        } else {
            "VETO"
        },
        quality.score,
        aggregate
    );

    // Aggregate issues
    let all_issues: Vec<AggregatedIssue> = results
        .iter()
        .flat_map(|r| {
            r.issues.iter().map(|i| AggregatedIssue {
                source: r.judge_type,
                code: i.code.clone(),
                description: i.description.clone(),
                severity: format!("{:?}", i.severity),
            })
        })
        .collect();

    // Aggregate recommendations
    let recommendations: Vec<String> = results
        .iter()
        .flat_map(|r| r.recommendations.clone())
        .collect();

    // Average confidence
    let confidence = (constitutional.confidence + technical.confidence + quality.confidence) / 3.0;

    // Total processing time
    let total_processing_time_ms = constitutional.processing_time_ms
        + technical.processing_time_ms
        + quality.processing_time_ms;

    Ok(AggregatedVerdict {
        verdict,
        scores,
        vetoed,
        vetoing_judges,
        reasoning,
        all_issues,
        recommendations,
        confidence,
        total_processing_time_ms,
    })
}

/// Errors from aggregation
#[derive(Debug, thiserror::Error)]
pub enum AggregationError {
    #[error("Missing result from {0:?} judge")]
    MissingJudge(JudgeType),

    #[error("Invalid scores: {0}")]
    InvalidScores(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(judge_type: JudgeType, score: f64) -> JudgeEvaluationResult {
        JudgeEvaluationResult {
            judge_id: format!("{:?}-1", judge_type),
            judge_type,
            score,
            confidence: 0.9,
            reasoning: "Test reasoning".to_string(),
            issues: vec![],
            recommendations: vec![],
            processing_time_ms: 100,
            evaluated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_aggregate_approval() {
        let results = vec![
            make_result(JudgeType::Constitutional, 0.95),
            make_result(JudgeType::Technical, 0.92),
            make_result(JudgeType::Quality, 0.90),
        ];

        let config = AggregationConfig::default();
        let verdict = aggregate(&results, &config).unwrap();

        assert!(matches!(verdict.verdict, CouncilVerdict::Approved { .. }));
        assert!(!verdict.vetoed);
        assert!(verdict.vetoing_judges.is_empty());
    }

    #[test]
    fn test_aggregate_conditional() {
        let results = vec![
            make_result(JudgeType::Constitutional, 0.85),
            make_result(JudgeType::Technical, 0.80),
            make_result(JudgeType::Quality, 0.75),
        ];

        let config = AggregationConfig::default();
        let verdict = aggregate(&results, &config).unwrap();

        assert!(matches!(
            verdict.verdict,
            CouncilVerdict::ConditionalApproval { .. }
        ));
        assert!(!verdict.vetoed);
    }

    #[test]
    fn test_aggregate_rejection() {
        let results = vec![
            make_result(JudgeType::Constitutional, 0.60),
            make_result(JudgeType::Technical, 0.55),
            make_result(JudgeType::Quality, 0.50),
        ];

        let config = AggregationConfig::default();
        let verdict = aggregate(&results, &config).unwrap();

        assert!(matches!(verdict.verdict, CouncilVerdict::Rejected { .. }));
    }

    #[test]
    fn test_aggregate_veto() {
        let results = vec![
            make_result(JudgeType::Constitutional, 0.40), // Below veto threshold
            make_result(JudgeType::Technical, 0.95),
            make_result(JudgeType::Quality, 0.90),
        ];

        let config = AggregationConfig::default();
        let verdict = aggregate(&results, &config).unwrap();

        assert!(matches!(verdict.verdict, CouncilVerdict::Rejected { .. }));
        assert!(verdict.vetoed);
        assert!(verdict.vetoing_judges.contains(&JudgeType::Constitutional));
    }

    #[test]
    fn test_aggregate_multiple_vetos() {
        let results = vec![
            make_result(JudgeType::Constitutional, 0.30),
            make_result(JudgeType::Technical, 0.40),
            make_result(JudgeType::Quality, 0.95),
        ];

        let config = AggregationConfig::default();
        let verdict = aggregate(&results, &config).unwrap();

        assert!(verdict.vetoed);
        assert_eq!(verdict.vetoing_judges.len(), 2);
    }

    #[test]
    fn test_missing_judge() {
        let results = vec![
            make_result(JudgeType::Constitutional, 0.90),
            make_result(JudgeType::Technical, 0.90),
            // Missing Quality judge
        ];

        let config = AggregationConfig::default();
        let result = aggregate(&results, &config);

        assert!(matches!(result, Err(AggregationError::MissingJudge(_))));
    }

    #[test]
    fn test_strict_config() {
        let results = vec![
            make_result(JudgeType::Constitutional, 0.92),
            make_result(JudgeType::Technical, 0.90),
            make_result(JudgeType::Quality, 0.88),
        ];

        let config = AggregationConfig::strict();
        let verdict = aggregate(&results, &config).unwrap();

        // Would be approved with default config, but conditional with strict
        assert!(matches!(
            verdict.verdict,
            CouncilVerdict::ConditionalApproval { .. }
        ));
    }

    #[test]
    fn test_weighted_scores() {
        let results = vec![
            make_result(JudgeType::Constitutional, 1.0),
            make_result(JudgeType::Technical, 0.0),
            make_result(JudgeType::Quality, 0.0),
        ];

        let config = AggregationConfig::default();
        let verdict = aggregate(&results, &config).unwrap();

        // Should be rejected due to vetos, but check aggregate calculation
        // 1.0 * 0.30 + 0.0 * 0.35 + 0.0 * 0.35 = 0.30
        assert!((verdict.scores.aggregate - 0.30).abs() < 0.01);
    }
}
