//! OpenMetrics Reporter
//!
//! Generates OpenMetrics format for Prometheus integration.

use crate::evaluation::framework::EvaluationReport;
use crate::evaluation::contracts::Reporter;

/// OpenMetrics reporter for Prometheus
pub struct MetricsReporter {
    metric_prefix: String,
}

impl MetricsReporter {
    pub fn new() -> Self {
        Self {
            metric_prefix: "agent_evaluation".to_string(),
        }
    }
    
    pub fn with_prefix(prefix: String) -> Self {
        Self {
            metric_prefix: prefix,
        }
    }
}

impl Default for MetricsReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for MetricsReporter {
    fn name(&self) -> &str {
        "openmetrics"
    }
    
    fn render(&self, report: &EvaluationReport) -> Result<String, String> {
        let mut output = String::new();
        
        // Write OpenMetrics header
        output.push_str("# TYPE agent_evaluation_overall_score gauge\n");
        output.push_str("# HELP agent_evaluation_overall_score Overall evaluation score (0.0-1.0)\n");
        
        // Overall score metric
        output.push_str(&format!(
            "{}_{{scenario_id=\"{}\",scenario_name=\"{}\"}} {:.4}\n",
            self.metric_prefix,
            escape_label_value(&report.scenario.scenario_id),
            escape_label_value(&report.scenario.name),
            report.summary.average_score
        ));
        
        // Dimension metrics
        output.push_str("# TYPE agent_evaluation_dimension_score gauge\n");
        output.push_str("# HELP agent_evaluation_dimension_score Evaluation dimension scores (0.0-1.0)\n");
        
        for (dimension, score) in &report.summary.score_distribution {
            output.push_str(&format!(
                "{}_{{scenario_id=\"{}\",dimension=\"{}\"}} {:.4}\n",
                self.metric_prefix,
                escape_label_value(&report.scenario.scenario_id),
                escape_label_value(dimension),
                score
            ));
        }
        
        // Process quality metrics
        if !report.evaluations.is_empty() {
            output.push_str("# TYPE agent_evaluation_process_quality gauge\n");
            output.push_str("# HELP agent_evaluation_process_quality Process quality metrics (0.0-1.0)\n");
            
            for (idx, eval) in report.evaluations.iter().enumerate() {
                let labels = format!(
                    "scenario_id=\"{}\",evaluation_index=\"{}\"",
                    escape_label_value(&report.scenario.scenario_id),
                    idx
                );
                
                output.push_str(&format!(
                    "{}_{{{},metric=\"reasoning_depth\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.process_quality.reasoning_depth
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"decision_quality\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.process_quality.decision_quality
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"risk_assessment\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.process_quality.risk_assessment
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"coordination_quality\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.process_quality.coordination_quality
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"iterative_improvement\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.process_quality.iterative_improvement
                ));
            }
        }
        
        // Adaptability metrics
        if !report.evaluations.is_empty() {
            output.push_str("# TYPE agent_evaluation_adaptability gauge\n");
            output.push_str("# HELP agent_evaluation_adaptability Adaptability metrics (0.0-1.0)\n");
            
            for (idx, eval) in report.evaluations.iter().enumerate() {
                let labels = format!(
                    "scenario_id=\"{}\",evaluation_index=\"{}\"",
                    escape_label_value(&report.scenario.scenario_id),
                    idx
                );
                
                output.push_str(&format!(
                    "{}_{{{},metric=\"uncertainty_management\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.adaptability_metrics.uncertainty_management
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"failure_recovery\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.adaptability_metrics.failure_recovery
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"resource_adaptation\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.adaptability_metrics.resource_adaptation
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"strategy_flexibility\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.adaptability_metrics.strategy_flexibility
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"learning_velocity\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.adaptability_metrics.learning_velocity
                ));
            }
        }
        
        // Safety metrics
        if !report.evaluations.is_empty() {
            output.push_str("# TYPE agent_evaluation_safety gauge\n");
            output.push_str("# HELP agent_evaluation_safety Safety assessment metrics (0.0-1.0)\n");
            
            for (idx, eval) in report.evaluations.iter().enumerate() {
                let labels = format!(
                    "scenario_id=\"{}\",evaluation_index=\"{}\"",
                    escape_label_value(&report.scenario.scenario_id),
                    idx
                );
                
                output.push_str(&format!(
                    "{}_{{{},metric=\"risk_avoidance\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.safety_assessment.risk_avoidance
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"error_handling\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.safety_assessment.error_handling
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"boundary_compliance\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.safety_assessment.boundary_compliance
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"recovery_safety\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.safety_assessment.recovery_safety
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"audit_completeness\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.safety_assessment.audit_completeness
                ));
            }
        }
        
        // Learning indicators
        if !report.evaluations.is_empty() {
            output.push_str("# TYPE agent_evaluation_learning gauge\n");
            output.push_str("# HELP agent_evaluation_learning Learning indicator metrics (0.0-1.0)\n");
            
            for (idx, eval) in report.evaluations.iter().enumerate() {
                let labels = format!(
                    "scenario_id=\"{}\",evaluation_index=\"{}\"",
                    escape_label_value(&report.scenario.scenario_id),
                    idx
                );
                
                output.push_str(&format!(
                    "{}_{{{},metric=\"pattern_recognition\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.learning_indicators.pattern_recognition
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"solution_generalization\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.learning_indicators.solution_generalization
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"feedback_integration\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.learning_indicators.feedback_integration
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"self_optimization\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.learning_indicators.self_optimization
                ));
                output.push_str(&format!(
                    "{}_{{{},metric=\"knowledge_retention\"}} {:.4}\n",
                    self.metric_prefix, labels, eval.learning_indicators.knowledge_retention
                ));
            }
        }
        
        // Trend metrics
        output.push_str("# TYPE agent_evaluation_trend gauge\n");
        output.push_str("# HELP agent_evaluation_trend Trend analysis metrics\n");
        
        let trend_value = match report.summary.trend_analysis.performance_trend {
            crate::evaluation::framework::PerformanceTrend::Improving => 1.0,
            crate::evaluation::framework::PerformanceTrend::Stable => 0.5,
            crate::evaluation::framework::PerformanceTrend::Declining => 0.0,
            crate::evaluation::framework::PerformanceTrend::Inconsistent => 0.25,
        };
        
        output.push_str(&format!(
            "{}_{{scenario_id=\"{}\",metric=\"performance_trend\"}} {:.4}\n",
            self.metric_prefix,
            escape_label_value(&report.scenario.scenario_id),
            trend_value
        ));
        output.push_str(&format!(
            "{}_{{scenario_id=\"{}\",metric=\"learning_rate\"}} {:.4}\n",
            self.metric_prefix,
            escape_label_value(&report.scenario.scenario_id),
            report.summary.trend_analysis.learning_rate
        ));
        output.push_str(&format!(
            "{}_{{scenario_id=\"{}\",metric=\"consistency_score\"}} {:.4}\n",
            self.metric_prefix,
            escape_label_value(&report.scenario.scenario_id),
            report.summary.trend_analysis.consistency_score
        ));
        output.push_str(&format!(
            "{}_{{scenario_id=\"{}\",metric=\"adaptability_growth\"}} {:.4}\n",
            self.metric_prefix,
            escape_label_value(&report.scenario.scenario_id),
            report.summary.trend_analysis.adaptability_growth
        ));
        
        Ok(output)
    }
    
    fn format(&self) -> &str {
        "openmetrics"
    }
}

/// Escape label values for OpenMetrics format
fn escape_label_value(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::framework::{EvaluationReport, EvaluationScenario, ScenarioDifficulty, ProblemType, EvaluationSummary, TrendAnalysis, PerformanceTrend};

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
    fn test_metrics_reporter() {
        let reporter = MetricsReporter::new();
        let report = create_test_report();
        
        let result = reporter.render(&report);
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert!(metrics.contains("# TYPE"));
        assert!(metrics.contains("agent_evaluation"));
        assert!(metrics.contains("0.8500"));
    }
    
    #[test]
    fn test_escape_label_value() {
        assert_eq!(escape_label_value("test"), "test");
        assert_eq!(escape_label_value("test\"test"), "test\\\"test");
        assert_eq!(escape_label_value("test\\test"), "test\\\\test");
    }
}
