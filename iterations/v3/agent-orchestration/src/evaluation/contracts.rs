//! Evaluation Contracts (Traits)
//!
//! Defines formal contracts for scenarios, evaluators, and reporters.
//! These traits enable composable evaluation components and independent testing.

use crate::audit_trail::AuditEvent;
use crate::chain_of_thought::{CoordinationEvent, DecisionPoint};
use crate::evaluation::framework::{AgentEvaluation, EvaluationReport, EvaluationScenario};
use std::sync::Arc;

/// Scenario trait for test scenarios
///
/// Scenarios define the problem space, expected behaviors, and evaluation criteria.
pub trait Scenario: Send + Sync {
    /// Get scenario identifier
    fn id(&self) -> &str;

    /// Get scenario definition
    fn definition(&self) -> &EvaluationScenario;

    /// Set up scenario environment (e.g., create test files)
    fn setup(&self) -> Result<(), String>;

    /// Clean up scenario environment
    fn cleanup(&self) -> Result<(), String>;

    /// Verify ground truth (did agent solve the problem correctly?)
    fn verify_ground_truth(
        &self,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
        audit_entries: &[AuditEvent],
    ) -> Result<bool, String>;
}

/// Evaluator trait for computing evaluation metrics
///
/// Evaluators analyze execution data and produce evaluation scores.
pub trait Evaluator: Send + Sync {
    /// Get evaluator name
    fn name(&self) -> &str;

    /// Evaluate agent execution
    fn evaluate(
        &self,
        scenario: &EvaluationScenario,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
        audit_entries: &[AuditEvent],
    ) -> Result<AgentEvaluation, String>;

    /// Get evaluation dimensions this evaluator focuses on
    fn dimensions(&self) -> Vec<&str>;
}

/// Reporter trait for generating evaluation reports
///
/// Reporters format evaluation results for different output formats.
pub trait Reporter: Send + Sync {
    /// Get reporter name
    fn name(&self) -> &str;

    /// Render evaluation report
    fn render(&self, report: &EvaluationReport) -> Result<String, String>;

    /// Get output format (e.g., "markdown", "html", "junit")
    fn format(&self) -> &str;
}

/// Oracle trait for ground truth verification
///
/// Oracles verify whether agent execution correctly solved the problem.
pub trait Oracle: Send + Sync {
    /// Get oracle identifier
    fn id(&self) -> &str;

    /// Verify execution against ground truth
    fn verify(
        &self,
        scenario: &EvaluationScenario,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
        audit_entries: &[AuditEvent],
    ) -> Result<OracleResult, String>;
}

/// Oracle verification result
#[derive(Debug, Clone)]
pub struct OracleResult {
    /// Whether the execution was correct
    pub correct: bool,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Detailed explanation of verification
    pub explanation: String,

    /// Specific issues found (if any)
    pub issues: Vec<String>,
}

/// Default heuristic-based Oracle implementation
///
/// Uses pattern matching on decisions and events to verify expected behaviors.
/// This is a fallback Oracle that provides basic verification when no specialized
/// Oracle is available.
pub struct HeuristicOracle;

impl HeuristicOracle {
    /// Create new heuristic Oracle
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Oracle for HeuristicOracle {
    fn id(&self) -> &str {
        "heuristic"
    }

    fn verify(
        &self,
        scenario: &EvaluationScenario,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
        _audit_entries: &[AuditEvent],
    ) -> Result<OracleResult, String> {
        use crate::evaluation::framework::BehaviorImportance;

        // Check if scenario has expected behaviors
        let critical_behaviors: Vec<_> = scenario
            .expected_behaviors
            .iter()
            .filter(|b| matches!(b.importance, BehaviorImportance::Critical))
            .collect();

        if critical_behaviors.is_empty() {
            return Ok(OracleResult {
                correct: true,
                confidence: 1.0,
                explanation: "No critical behaviors to verify".to_string(),
                issues: vec![],
            });
        }

        let mut issues = Vec::new();
        let mut verified_count = 0;

        // Verify each critical behavior
        for behavior in &critical_behaviors {
            let behavior_name = behavior.behavior.as_str();
            let verified = match behavior_name {
                "problem_identification" => decisions.iter().any(|d| {
                    d.reasoning.to_lowercase().contains("problem")
                        || d.reasoning.to_lowercase().contains("issue")
                        || d.reasoning.to_lowercase().contains("error")
                }),
                "reasoning_transparency" => decisions
                    .iter()
                    .any(|d| !d.reasoning.is_empty() && d.reasoning.len() > 20),
                "solution_exploration" => decisions.iter().any(|d| d.alternatives.len() > 1),
                "risk_assessment" => decisions.iter().any(|d| d.risk_assessment.is_some()),
                _ => {
                    // Unknown behavior - log but don't fail
                    issues.push(format!(
                        "Unknown behavior '{}' - cannot verify",
                        behavior_name
                    ));
                    true
                }
            };

            if verified {
                verified_count += 1;
            } else {
                issues.push(format!(
                    "Critical behavior '{}' not verified",
                    behavior_name
                ));
            }
        }

        let correct = verified_count == critical_behaviors.len();
        let confidence = if critical_behaviors.is_empty() {
            1.0
        } else {
            verified_count as f64 / critical_behaviors.len() as f64
        };

        Ok(OracleResult {
            correct,
            confidence,
            explanation: format!(
                "Heuristic verification: {}/{} critical behaviors verified",
                verified_count,
                critical_behaviors.len()
            ),
            issues,
        })
    }
}

/// Composite evaluator that combines multiple evaluators
pub struct CompositeEvaluator {
    evaluators: Vec<Arc<dyn Evaluator>>,
    name: String,
}

impl CompositeEvaluator {
    /// Create new composite evaluator
    pub fn new(name: String) -> Self {
        Self {
            evaluators: Vec::new(),
            name,
        }
    }

    /// Add an evaluator to the composite
    pub fn add_evaluator(&mut self, evaluator: Arc<dyn Evaluator>) {
        self.evaluators.push(evaluator);
    }

    /// Evaluate using all evaluators and combine results
    pub fn evaluate_composite(
        &self,
        scenario: &EvaluationScenario,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
        audit_entries: &[AuditEvent],
    ) -> Result<AgentEvaluation, String> {
        if self.evaluators.is_empty() {
            return Err("No evaluators in composite".to_string());
        }

        // Run all evaluators
        let mut evaluations = Vec::new();
        for evaluator in &self.evaluators {
            match evaluator.evaluate(scenario, decisions, events, audit_entries) {
                Ok(eval) => evaluations.push(eval),
                Err(e) => {
                    // Log error but continue with other evaluators
                    eprintln!("Evaluator {} failed: {}", evaluator.name(), e);
                }
            }
        }

        if evaluations.is_empty() {
            return Err("All evaluators failed".to_string());
        }

        // Combine evaluations (average scores)
        let combined = self.combine_evaluations(&evaluations);
        Ok(combined)
    }

    /// Combine multiple evaluations into one
    fn combine_evaluations(&self, evaluations: &[AgentEvaluation]) -> AgentEvaluation {
        let count = evaluations.len() as f64;

        // Average all scores
        let overall_score = evaluations.iter().map(|e| e.overall_score).sum::<f64>() / count;

        // Average dimensions
        let functional_correctness = evaluations
            .iter()
            .map(|e| e.dimensions.functional_correctness)
            .sum::<f64>()
            / count;

        let process_quality = evaluations
            .iter()
            .map(|e| e.dimensions.process_quality)
            .sum::<f64>()
            / count;

        let adaptability = evaluations
            .iter()
            .map(|e| e.dimensions.adaptability)
            .sum::<f64>()
            / count;

        let efficiency = evaluations
            .iter()
            .map(|e| e.dimensions.efficiency)
            .sum::<f64>()
            / count;

        let safety = evaluations.iter().map(|e| e.dimensions.safety).sum::<f64>() / count;

        // Use first evaluation as template (they should all have same scenario_id)
        let template = &evaluations[0];

        AgentEvaluation {
            evaluation_id: template.evaluation_id,
            scenario_id: template.scenario_id.clone(),
            timestamp: template.timestamp,
            overall_score,
            dimensions: crate::evaluation::framework::EvaluationDimensions {
                functional_correctness,
                process_quality,
                adaptability,
                efficiency,
                safety,
            },
            process_quality: template.process_quality.clone(),
            adaptability_metrics: template.adaptability_metrics.clone(),
            safety_assessment: template.safety_assessment.clone(),
            learning_indicators: template.learning_indicators.clone(),
        }
    }
}

impl Evaluator for CompositeEvaluator {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(
        &self,
        scenario: &EvaluationScenario,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
        audit_entries: &[AuditEvent],
    ) -> Result<AgentEvaluation, String> {
        self.evaluate_composite(scenario, decisions, events, audit_entries)
    }

    fn dimensions(&self) -> Vec<&str> {
        // Return union of all evaluator dimensions
        let mut dims = std::collections::HashSet::new();
        for evaluator in &self.evaluators {
            for dim in evaluator.dimensions() {
                dims.insert(dim);
            }
        }
        dims.into_iter().collect()
    }
}

/// Composite reporter that generates multiple report formats
pub struct CompositeReporter {
    reporters: Vec<Arc<dyn Reporter>>,
    name: String,
}

impl CompositeReporter {
    /// Create new composite reporter
    pub fn new(name: String) -> Self {
        Self {
            reporters: Vec::new(),
            name,
        }
    }

    /// Add a reporter to the composite
    pub fn add_reporter(&mut self, reporter: Arc<dyn Reporter>) {
        self.reporters.push(reporter);
    }

    /// Render report using all reporters
    pub fn render_all(&self, report: &EvaluationReport) -> Result<Vec<(String, String)>, String> {
        let mut results = Vec::new();

        for reporter in &self.reporters {
            match reporter.render(report) {
                Ok(content) => {
                    results.push((reporter.format().to_string(), content));
                }
                Err(e) => {
                    eprintln!("Reporter {} failed: {}", reporter.name(), e);
                }
            }
        }

        Ok(results)
    }
}

impl Reporter for CompositeReporter {
    fn name(&self) -> &str {
        &self.name
    }

    fn render(&self, report: &EvaluationReport) -> Result<String, String> {
        // Render using first reporter as default
        if let Some(reporter) = self.reporters.first() {
            reporter.render(report)
        } else {
            Err("No reporters in composite".to_string())
        }
    }

    fn format(&self) -> &str {
        // Return first reporter's format
        self.reporters
            .first()
            .map(|r| r.format())
            .unwrap_or("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::framework::{create_code_fix_scenario, EvaluationEngine};

    struct MockEvaluator {
        name: String,
    }

    impl Evaluator for MockEvaluator {
        fn name(&self) -> &str {
            &self.name
        }

        fn evaluate(
            &self,
            scenario: &EvaluationScenario,
            _decisions: &[DecisionPoint],
            _events: &[CoordinationEvent],
            _audit_entries: &[AuditEvent],
        ) -> Result<AgentEvaluation, String> {
            let engine = EvaluationEngine::new();
            engine.evaluate_scenario(&scenario.scenario_id, &[], &[], &[])
        }

        fn dimensions(&self) -> Vec<&str> {
            vec!["functional_correctness", "process_quality"]
        }
    }

    #[test]
    fn test_composite_evaluator() {
        let mut composite = CompositeEvaluator::new("test-composite".to_string());

        let evaluator1 = Arc::new(MockEvaluator {
            name: "evaluator1".to_string(),
        });

        composite.add_evaluator(evaluator1);

        assert_eq!(composite.name(), "test-composite");
        assert_eq!(composite.dimensions().len(), 2);
    }

    struct MockReporter {
        name: String,
        format_name: String,
    }

    impl Reporter for MockReporter {
        fn name(&self) -> &str {
            &self.name
        }

        fn render(&self, _report: &EvaluationReport) -> Result<String, String> {
            Ok("Mock report".to_string())
        }

        fn format(&self) -> &str {
            &self.format_name
        }
    }

    #[test]
    fn test_composite_reporter() {
        let mut composite = CompositeReporter::new("test-reporter".to_string());

        let reporter = Arc::new(MockReporter {
            name: "reporter1".to_string(),
            format_name: "markdown".to_string(),
        });

        composite.add_reporter(reporter);

        assert_eq!(composite.name(), "test-reporter");
        assert_eq!(composite.format(), "markdown");
    }
}
