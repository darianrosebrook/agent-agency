//! HTML Reporter
//!
//! Generates HTML-formatted evaluation reports for local viewing.

use crate::evaluation::contracts::Reporter;
use crate::evaluation::framework::EvaluationReport;

/// HTML reporter for local viewing
pub struct HtmlReporter {
    include_charts: bool,
}

impl HtmlReporter {
    pub fn new() -> Self {
        Self {
            include_charts: false,
        }
    }

    pub fn with_charts(include_charts: bool) -> Self {
        Self { include_charts }
    }
}

impl Default for HtmlReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for HtmlReporter {
    fn name(&self) -> &str {
        "html"
    }

    fn render(&self, report: &EvaluationReport) -> Result<String, String> {
        let mut output = String::new();

        // HTML header
        output.push_str(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Evaluation Report: "#,
        );
        output.push_str(&html_escape(&report.scenario.name));
        output.push_str(r#"</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h1 { color: #333; border-bottom: 3px solid #4CAF50; padding-bottom: 10px; }
        h2 { color: #555; margin-top: 30px; }
        h3 { color: #777; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #4CAF50; color: white; }
        tr:hover { background-color: #f5f5f5; }
        .score-high { color: #4CAF50; font-weight: bold; }
        .score-medium { color: #FF9800; font-weight: bold; }
        .score-low { color: #F44336; font-weight: bold; }
        .badge { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 12px; }
        .badge-success { background-color: #4CAF50; color: white; }
        .badge-warning { background-color: #FF9800; color: white; }
        .badge-danger { background-color: #F44336; color: white; }
        .summary-box { background: #f9f9f9; padding: 20px; border-radius: 8px; margin: 20px 0; }
        .metric-bar { background: #e0e0e0; height: 20px; border-radius: 10px; overflow: hidden; margin: 5px 0; }
        .metric-fill { height: 100%; background: linear-gradient(90deg, #4CAF50, #8BC34A); }
    </style>
</head>
<body>
    <div class="container">
"#);

        // Header
        output.push_str(&format!(
            "<h1>Evaluation Report: {}</h1>\n",
            html_escape(&report.scenario.name)
        ));
        output.push_str(&format!(
            "<p><strong>Scenario ID</strong>: <code>{}</code></p>\n",
            html_escape(&report.scenario.scenario_id)
        ));
        output.push_str(&format!(
            "<p><strong>Description</strong>: {}</p>\n",
            html_escape(&report.scenario.description)
        ));

        // Summary box
        output.push_str("<div class=\"summary-box\">\n");
        output.push_str("<h2>Summary</h2>\n");

        let score_class = if report.summary.average_score >= 0.8 {
            "score-high"
        } else if report.summary.average_score >= 0.6 {
            "score-medium"
        } else {
            "score-low"
        };

        output.push_str(&format!(
            "<p><strong>Average Score</strong>: <span class=\"{}\">{:.2}%</span></p>\n",
            score_class,
            report.summary.average_score * 100.0
        ));

        // Score distribution table
        if !report.summary.score_distribution.is_empty() {
            output.push_str("<h3>Score Distribution</h3>\n");
            output.push_str("<table>\n");
            output.push_str("<tr><th>Dimension</th><th>Score</th></tr>\n");
            for (dimension, score) in &report.summary.score_distribution {
                let dim_score_class = if *score >= 0.8 {
                    "score-high"
                } else if *score >= 0.6 {
                    "score-medium"
                } else {
                    "score-low"
                };
                output.push_str(&format!(
                    "<tr><td>{}</td><td class=\"{}\">{:.2}%</td></tr>\n",
                    html_escape(dimension),
                    dim_score_class,
                    score * 100.0
                ));
            }
            output.push_str("</table>\n");
        }

        output.push_str("</div>\n");

        // Strength areas
        if !report.summary.strength_areas.is_empty() {
            output.push_str("<h2>Strengths</h2>\n<ul>\n");
            for strength in &report.summary.strength_areas {
                output.push_str(&format!("<li>{}</li>\n", html_escape(strength)));
            }
            output.push_str("</ul>\n");
        }

        // Improvement areas
        if !report.summary.improvement_areas.is_empty() {
            output.push_str("<h2>Areas for Improvement</h2>\n<ul>\n");
            for area in &report.summary.improvement_areas {
                output.push_str(&format!("<li>{}</li>\n", html_escape(area)));
            }
            output.push_str("</ul>\n");
        }

        // Detailed evaluations
        if !report.evaluations.is_empty() {
            output.push_str("<h2>Detailed Evaluations</h2>\n");
            for (idx, eval) in report.evaluations.iter().enumerate() {
                output.push_str(&format!("<h3>Evaluation #{}</h3>\n", idx + 1));

                output.push_str("<table>\n");
                output.push_str("<tr><th>Dimension</th><th>Score</th></tr>\n");

                let dimensions = [
                    (
                        "Functional Correctness",
                        eval.dimensions.functional_correctness,
                    ),
                    ("Process Quality", eval.dimensions.process_quality),
                    ("Adaptability", eval.dimensions.adaptability),
                    ("Efficiency", eval.dimensions.efficiency),
                    ("Safety", eval.dimensions.safety),
                ];

                for (name, score) in &dimensions {
                    let dim_score_class = if *score >= 0.8 {
                        "score-high"
                    } else if *score >= 0.6 {
                        "score-medium"
                    } else {
                        "score-low"
                    };
                    output.push_str(&format!(
                        "<tr><td>{}</td><td class=\"{}\">{:.2}%</td></tr>\n",
                        name,
                        dim_score_class,
                        score * 100.0
                    ));
                }

                output.push_str("</table>\n");
            }
        }

        // Recommendations
        if !report.recommendations.is_empty() {
            output.push_str("<h2>Recommendations</h2>\n<ol>\n");
            for rec in &report.recommendations {
                output.push_str(&format!("<li>{}</li>\n", html_escape(rec)));
            }
            output.push_str("</ol>\n");
        }

        // Footer
        output.push_str(
            r#"
    </div>
</body>
</html>"#,
        );

        Ok(output)
    }

    fn format(&self) -> &str {
        "html"
    }
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
    fn test_html_reporter() {
        let reporter = HtmlReporter::new();
        let report = create_test_report();

        let result = reporter.render(&report);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Evaluation Report"));
        assert!(html.contains("85.00%"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("test & test"), "test &amp; test");
        assert_eq!(html_escape("test < test"), "test &lt; test");
        assert_eq!(html_escape("test > test"), "test &gt; test");
    }
}
