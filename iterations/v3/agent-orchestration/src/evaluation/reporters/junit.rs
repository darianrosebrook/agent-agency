//! JUnit Reporter
//!
//! Generates JUnit XML format for CI integration.

use crate::evaluation::framework::EvaluationReport;
use crate::evaluation::contracts::Reporter;
use std::io::Write;

/// JUnit reporter for CI integration
pub struct JUnitReporter {
    test_suite_name: String,
}

impl JUnitReporter {
    pub fn new() -> Self {
        Self {
            test_suite_name: "Agent Evaluation".to_string(),
        }
    }
    
    pub fn with_test_suite_name(name: String) -> Self {
        Self {
            test_suite_name: name,
        }
    }
}

impl Default for JUnitReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for JUnitReporter {
    fn name(&self) -> &str {
        "junit"
    }
    
    fn render(&self, report: &EvaluationReport) -> Result<String, String> {
        let mut output = Vec::new();
        
        // Write XML header
        writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
            .map_err(|e| format!("Failed to write XML header: {}", e))?;
        
        // Calculate totals
        let total_tests = report.evaluations.len();
        let failures = report.evaluations.iter()
            .filter(|e| e.overall_score < 0.7) // Threshold for failure
            .count();
        let errors = report.evaluations.iter()
            .filter(|e| e.overall_score < 0.5) // Threshold for error
            .count();
        
        // Calculate total time (estimate based on evaluations)
        let total_time = total_tests as f64 * 1.0; // Assume 1 second per evaluation
        
        // Write testsuite element
        writeln!(
            output,
            r#"<testsuite name="{}" tests="{}" failures="{}" errors="{}" time="{:.2}">"#,
            self.test_suite_name, total_tests, failures, errors, total_time
        ).map_err(|e| format!("Failed to write testsuite: {}", e))?;
        
        // Write properties
        writeln!(output, r#"  <properties>"#)
            .map_err(|e| format!("Failed to write properties: {}", e))?;
        writeln!(output, r#"    <property name="scenario_id" value="{}"/>"#, report.scenario.scenario_id)
            .map_err(|e| format!("Failed to write property: {}", e))?;
        writeln!(output, r#"    <property name="scenario_name" value="{}"/>"#, report.scenario.name)
            .map_err(|e| format!("Failed to write property: {}", e))?;
        writeln!(output, r#"    <property name="average_score" value="{:.2}"/>"#, report.summary.average_score)
            .map_err(|e| format!("Failed to write property: {}", e))?;
        writeln!(output, r#"  </properties>"#)
            .map_err(|e| format!("Failed to close properties: {}", e))?;
        
        // Write test cases
        for (idx, eval) in report.evaluations.iter().enumerate() {
            let test_name = format!("evaluation_{}", idx + 1);
            let classname = format!("{}.{}", self.test_suite_name, report.scenario.scenario_id);
            
            if eval.overall_score < 0.5 {
                // Error case
                writeln!(
                    output,
                    r#"  <testcase name="{}" classname="{}" time="1.0">"#,
                    test_name, classname
                ).map_err(|e| format!("Failed to write testcase: {}", e))?;
                writeln!(
                    output,
                    r#"    <error message="Score {:.2}% below error threshold" type="EvaluationError">"#,
                    eval.overall_score * 100.0
                ).map_err(|e| format!("Failed to write error: {}", e))?;
                writeln!(output, r#"      Overall score: {:.2}%"#, eval.overall_score * 100.0)
                    .map_err(|e| format!("Failed to write error details: {}", e))?;
                writeln!(output, r#"    </error>"#)
                    .map_err(|e| format!("Failed to close error: {}", e))?;
                writeln!(output, r#"  </testcase>"#)
                    .map_err(|e| format!("Failed to close testcase: {}", e))?;
            } else if eval.overall_score < 0.7 {
                // Failure case
                writeln!(
                    output,
                    r#"  <testcase name="{}" classname="{}" time="1.0">"#,
                    test_name, classname
                ).map_err(|e| format!("Failed to write testcase: {}", e))?;
                writeln!(
                    output,
                    r#"    <failure message="Score {:.2}% below failure threshold" type="EvaluationFailure">"#,
                    eval.overall_score * 100.0
                ).map_err(|e| format!("Failed to write failure: {}", e))?;
                writeln!(output, r#"      Overall score: {:.2}%"#, eval.overall_score * 100.0)
                    .map_err(|e| format!("Failed to write failure details: {}", e))?;
                writeln!(output, r#"    </failure>"#)
                    .map_err(|e| format!("Failed to close failure: {}", e))?;
                writeln!(output, r#"  </testcase>"#)
                    .map_err(|e| format!("Failed to close testcase: {}", e))?;
            } else {
                // Success case
                writeln!(
                    output,
                    r#"  <testcase name="{}" classname="{}" time="1.0"/>"#,
                    test_name, classname
                ).map_err(|e| format!("Failed to write testcase: {}", e))?;
            }
        }
        
        // Write system-out
        writeln!(output, r#"  <system-out>"#)
            .map_err(|e| format!("Failed to write system-out: {}", e))?;
        writeln!(output, r#"    Average Score: {:.2}%"#, report.summary.average_score * 100.0)
            .map_err(|e| format!("Failed to write system-out content: {}", e))?;
        if !report.summary.strength_areas.is_empty() {
            writeln!(output, r#"    Strengths: {}"#, report.summary.strength_areas.join(", "))
                .map_err(|e| format!("Failed to write strengths: {}", e))?;
        }
        if !report.summary.improvement_areas.is_empty() {
            writeln!(output, r#"    Improvement Areas: {}"#, report.summary.improvement_areas.join(", "))
                .map_err(|e| format!("Failed to write improvement areas: {}", e))?;
        }
        writeln!(output, r#"  </system-out>"#)
            .map_err(|e| format!("Failed to close system-out: {}", e))?;
        
        // Close testsuite
        writeln!(output, r#"</testsuite>"#)
            .map_err(|e| format!("Failed to close testsuite: {}", e))?;
        
        String::from_utf8(output)
            .map_err(|e| format!("Failed to convert to string: {}", e))
    }
    
    fn format(&self) -> &str {
        "junit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::framework::{EvaluationReport, EvaluationScenario, ScenarioDifficulty, ProblemType, EvaluationSummary, TrendAnalysis, PerformanceTrend, AgentEvaluation, EvaluationDimensions, ProcessQualityMetrics, AdaptabilityMetrics, SafetyAssessment, LearningIndicators};

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
            evaluations: vec![
                AgentEvaluation {
                    evaluation_id: uuid::Uuid::new_v4(),
                    scenario_id: "test-001".to_string(),
                    timestamp: chrono::Utc::now(),
                    overall_score: 0.85,
                    dimensions: EvaluationDimensions {
                        functional_correctness: 0.9,
                        process_quality: 0.8,
                        adaptability: 0.7,
                        efficiency: 0.85,
                        safety: 0.9,
                    },
                    process_quality: ProcessQualityMetrics {
                        reasoning_depth: 0.8,
                        decision_quality: 0.85,
                        risk_assessment: 0.75,
                        coordination_quality: 0.8,
                        iterative_improvement: 0.7,
                    },
                    adaptability_metrics: AdaptabilityMetrics {
                        uncertainty_management: 0.7,
                        failure_recovery: 0.8,
                        resource_adaptation: 0.75,
                        strategy_flexibility: 0.7,
                        learning_velocity: 0.65,
                    },
                    safety_assessment: SafetyAssessment {
                        risk_avoidance: 0.9,
                        error_handling: 0.85,
                        boundary_compliance: 0.9,
                        recovery_safety: 0.85,
                        audit_completeness: 0.9,
                    },
                    learning_indicators: LearningIndicators {
                        pattern_recognition: 0.7,
                        solution_generalization: 0.75,
                        feedback_integration: 0.8,
                        self_optimization: 0.7,
                        knowledge_retention: 0.75,
                    },
                },
            ],
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
    fn test_junit_reporter() {
        let reporter = JUnitReporter::new();
        let report = create_test_report();
        
        let result = reporter.render(&report);
        assert!(result.is_ok());
        
        let xml = result.unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<testsuite"));
        assert!(xml.contains("<testcase"));
    }
}
