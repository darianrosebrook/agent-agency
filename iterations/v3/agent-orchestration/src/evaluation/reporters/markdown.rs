//! Markdown Reporter
//!
//! Generates Markdown-formatted evaluation reports suitable for PR comments and documentation.

use crate::evaluation::contracts::Reporter;
use crate::evaluation::framework::EvaluationReport;

/// Markdown reporter for evaluation results
pub struct MarkdownReporter;

impl MarkdownReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for MarkdownReporter {
    fn name(&self) -> &str {
        "markdown"
    }

    fn render(&self, report: &EvaluationReport) -> Result<String, String> {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "# Evaluation Report: {}\n\n",
            report.scenario.name
        ));
        output.push_str(&format!(
            "**Scenario ID**: `{}`\n\n",
            report.scenario.scenario_id
        ));
        output.push_str(&format!(
            "**Description**: {}\n\n",
            report.scenario.description
        ));

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "**Average Score**: {:.2}%\n\n",
            report.summary.average_score * 100.0
        ));

        // Score distribution
        if !report.summary.score_distribution.is_empty() {
            output.push_str("### Score Distribution\n\n");
            output.push_str("| Dimension | Score |\n");
            output.push_str("|-----------|-------|\n");
            for (dimension, score) in &report.summary.score_distribution {
                output.push_str(&format!("| {} | {:.2}% |\n", dimension, score * 100.0));
            }
            output.push_str("\n");
        }

        // Strength areas
        if !report.summary.strength_areas.is_empty() {
            output.push_str("### Strengths\n\n");
            for strength in &report.summary.strength_areas {
                output.push_str(&format!("- {}\n", strength));
            }
            output.push_str("\n");
        }

        // Improvement areas
        if !report.summary.improvement_areas.is_empty() {
            output.push_str("### Areas for Improvement\n\n");
            for area in &report.summary.improvement_areas {
                output.push_str(&format!("- {}\n", area));
            }
            output.push_str("\n");
        }

        // Trend analysis
        output.push_str("### Trend Analysis\n\n");
        output.push_str(&format!(
            "**Performance Trend**: {:?}\n",
            report.summary.trend_analysis.performance_trend
        ));
        output.push_str(&format!(
            "**Learning Rate**: {:.2}\n",
            report.summary.trend_analysis.learning_rate
        ));
        output.push_str(&format!(
            "**Consistency Score**: {:.2}\n",
            report.summary.trend_analysis.consistency_score
        ));
        output.push_str(&format!(
            "**Adaptability Growth**: {:.2}\n\n",
            report.summary.trend_analysis.adaptability_growth
        ));

        // Detailed evaluations
        if !report.evaluations.is_empty() {
            output.push_str("## Detailed Evaluations\n\n");
            for (idx, eval) in report.evaluations.iter().enumerate() {
                output.push_str(&format!("### Evaluation #{}\n\n", idx + 1));
                output.push_str(&format!(
                    "**Overall Score**: {:.2}%\n\n",
                    eval.overall_score * 100.0
                ));

                output.push_str("#### Dimensions\n\n");
                output.push_str("| Dimension | Score |\n");
                output.push_str("|-----------|-------|\n");
                output.push_str(&format!(
                    "| Functional Correctness | {:.2}% |\n",
                    eval.dimensions.functional_correctness * 100.0
                ));
                output.push_str(&format!(
                    "| Process Quality | {:.2}% |\n",
                    eval.dimensions.process_quality * 100.0
                ));
                output.push_str(&format!(
                    "| Adaptability | {:.2}% |\n",
                    eval.dimensions.adaptability * 100.0
                ));
                output.push_str(&format!(
                    "| Efficiency | {:.2}% |\n",
                    eval.dimensions.efficiency * 100.0
                ));
                output.push_str(&format!(
                    "| Safety | {:.2}% |\n\n",
                    eval.dimensions.safety * 100.0
                ));

                output.push_str("#### Process Quality Metrics\n\n");
                output.push_str(&format!(
                    "- Reasoning Depth: {:.2}\n",
                    eval.process_quality.reasoning_depth
                ));
                output.push_str(&format!(
                    "- Decision Quality: {:.2}\n",
                    eval.process_quality.decision_quality
                ));
                output.push_str(&format!(
                    "- Risk Assessment: {:.2}\n",
                    eval.process_quality.risk_assessment
                ));
                output.push_str(&format!(
                    "- Coordination Quality: {:.2}\n",
                    eval.process_quality.coordination_quality
                ));
                output.push_str(&format!(
                    "- Iterative Improvement: {:.2}\n\n",
                    eval.process_quality.iterative_improvement
                ));
            }
        }

        // Recommendations
        if !report.recommendations.is_empty() {
            output.push_str("## Recommendations\n\n");
            for (idx, rec) in report.recommendations.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", idx + 1, rec));
            }
            output.push_str("\n");
        }

        Ok(output)
    }

    fn format(&self) -> &str {
        "markdown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::framework::{
        EvaluationReport, EvaluationScenario, EvaluationSummary, PerformanceTrend, ProblemType,
        ScenarioDifficulty, TrendAnalysis,
    };

    fn create_test_report() -> EvaluationReport {
        EvaluationReport {
            report_id: uuid::Uuid::new_v4(),
            scenario: EvaluationScenario {
                scenario_id: "test-001".to_string(),
                name: "Test Scenario".to_string(),
                description: "A test scenario".to_string(),
                difficulty: ScenarioDifficulty::Intermediate,
                problem_type: ProblemType::CompilationError,
                expected_behaviors: vec![],
                evaluation_criteria: vec![],
            },
            evaluations: vec![],
            summary: EvaluationSummary {
                average_score: 0.85,
                score_distribution: std::collections::HashMap::new(),
                strength_areas: vec!["Good reasoning".to_string()],
                improvement_areas: vec!["Better error handling".to_string()],
                trend_analysis: TrendAnalysis {
                    performance_trend: PerformanceTrend::Improving,
                    learning_rate: 0.7,
                    consistency_score: 0.8,
                    adaptability_growth: 0.6,
                },
            },
            recommendations: vec!["Improve error handling".to_string()],
        }
    }

    #[test]
    fn test_markdown_reporter() {
        let reporter = MarkdownReporter::new();
        let report = create_test_report();

        let result = reporter.render(&report);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("# Evaluation Report"));
        assert!(markdown.contains("Test Scenario"));
        assert!(markdown.contains("85.00%"));
    }
}
