//! Quality Analyzers for AI Agent Output Evaluation
//!
//! Implements comprehensive quality evaluation analyzers as defined in
//! QUALITY_EVALUATION_PLAN.md for assessing:
//! - Chain-of-Thought reasoning quality
//! - Council decision-making quality
//! - Output quality (code and writing)

use serde::{Deserialize, Serialize};
use std::path::Path;
#[cfg(feature = "full")]
use agent_orchestration::chain_of_thought::{DecisionPoint, RiskAssessment};
#[cfg(feature = "full")]
use agent_constitutional_council::verdict_writer::{VerdictRecord, JudgeVerdictSummary, CouncilMetrics};

/// Reasoning depth score (0.0-1.0)
/// Measures how thoroughly the agent analyzed the problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningDepthScore {
    pub score: f64,
    pub reasoning_length_score: f64,
    pub alternatives_score: f64,
    pub risk_assessment_score: f64,
    pub confidence_calibration_score: f64,
}

impl ReasoningDepthScore {
    /// Analyze reasoning depth from decision points
    #[cfg(feature = "full")]
    pub fn analyze(decisions: &[DecisionPoint]) -> Self {
        if decisions.is_empty() {
            return Self {
                score: 0.0,
                reasoning_length_score: 0.0,
                alternatives_score: 0.0,
                risk_assessment_score: 0.0,
                confidence_calibration_score: 0.0,
            };
        }

        let mut total_reasoning_length = 0.0;
        let mut total_alternatives = 0.0;
        let mut total_risk_assessment = 0.0;
        let mut total_confidence_calibration = 0.0;

        for decision in decisions {
            // Reasoning length score: +0.3 if > 100 chars
            let reasoning_length_score = if decision.reasoning.len() > 100 {
                0.3
            } else if decision.reasoning.len() > 50 {
                0.2
            } else if decision.reasoning.len() > 20 {
                0.1
            } else {
                0.0
            };
            total_reasoning_length += reasoning_length_score;

            // Alternatives score: +0.3 if > 2 alternatives
            let alternatives_score = if decision.alternatives.len() > 2 {
                0.3
            } else if decision.alternatives.len() > 0 {
                0.15
            } else {
                0.0
            };
            total_alternatives += alternatives_score;

            // Risk assessment score: +0.2 if present
            let risk_score = if decision.risk_assessment.is_some() {
                0.2
            } else {
                0.0
            };
            total_risk_assessment += risk_score;

            // Confidence calibration score: +0.2 if realistic (between 0.3 and 0.9)
            let confidence_calibration = if decision.confidence >= 0.3 && decision.confidence <= 0.9 {
                0.2
            } else if decision.confidence > 0.9 {
                0.1 // Overconfident
            } else {
                0.0 // Underconfident
            };
            total_confidence_calibration += confidence_calibration;
        }

        let count = decisions.len() as f64;
        let reasoning_length_score = total_reasoning_length / count;
        let alternatives_score = total_alternatives / count;
        let risk_assessment_score = total_risk_assessment / count;
        let confidence_calibration_score = total_confidence_calibration / count;

        let score = reasoning_length_score + alternatives_score + risk_assessment_score + confidence_calibration_score;

        Self {
            score: score.min(1.0),
            reasoning_length_score,
            alternatives_score,
            risk_assessment_score,
            confidence_calibration_score,
        }
    }

    /// Get quality level description
    pub fn quality_level(&self) -> &'static str {
        match self.score {
            s if s >= 0.9 => "Exceptional depth - thorough analysis, multiple perspectives considered",
            s if s >= 0.7 => "Good depth - solid analysis with some alternatives",
            s if s >= 0.5 => "Adequate depth - basic reasoning, limited alternatives",
            s if s >= 0.3 => "Shallow reasoning - minimal analysis, few alternatives",
            _ => "Poor reasoning - no real analysis, no alternatives",
        }
    }
}

/// Decision quality score (0.0-1.0)
/// Measures evidence gathering, logic soundness, confidence calibration, risk mitigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionQualityScore {
    pub score: f64,
    pub evidence_gathering_score: f64,
    pub logic_soundness_score: f64,
    pub confidence_calibration_score: f64,
    pub risk_mitigation_score: f64,
}

impl DecisionQualityScore {
    /// Analyze decision quality from decision points
    #[cfg(feature = "full")]
    pub fn analyze(decisions: &[DecisionPoint]) -> Self {
        if decisions.is_empty() {
            return Self {
                score: 0.0,
                evidence_gathering_score: 0.0,
                logic_soundness_score: 0.0,
                confidence_calibration_score: 0.0,
                risk_mitigation_score: 0.0,
            };
        }

        let mut total_evidence = 0.0;
        let mut total_logic = 0.0;
        let mut total_confidence = 0.0;
        let mut total_risk_mitigation = 0.0;

        for decision in decisions {
            // Evidence gathering: Check if reasoning references specific evidence
            let evidence_score = if decision.reasoning.contains("because") 
                || decision.reasoning.contains("due to")
                || decision.reasoning.contains("based on")
                || decision.reasoning.contains("evidence")
                || decision.reasoning.contains("data") {
                0.25
            } else {
                0.0
            };
            total_evidence += evidence_score;

            // Logic soundness: Check if reasoning is coherent (has logical connectors)
            let logic_score = if decision.reasoning.contains("therefore")
                || decision.reasoning.contains("thus")
                || decision.reasoning.contains("consequently")
                || decision.reasoning.contains("however")
                || decision.reasoning.contains("alternatively") {
                0.25
            } else if decision.reasoning.len() > 50 {
                0.15 // Basic coherence
            } else {
                0.0
            };
            total_logic += logic_score;

            // Confidence calibration: Realistic confidence levels
            let confidence_score = if decision.confidence >= 0.3 && decision.confidence <= 0.9 {
                0.25
            } else if decision.confidence > 0.9 {
                0.1 // Overconfident
            } else {
                0.0 // Underconfident
            };
            total_confidence += confidence_score;

            // Risk mitigation: Check if risk assessment includes mitigation strategies
            let risk_mitigation_score = if let Some(risk) = &decision.risk_assessment {
                if !risk.mitigation_strategies.is_empty() {
                    0.25
                } else if !risk.fallback_options.is_empty() {
                    0.15
                } else {
                    0.05
                }
            } else {
                0.0
            };
            total_risk_mitigation += risk_mitigation_score;
        }

        let count = decisions.len() as f64;
        let evidence_gathering_score = total_evidence / count;
        let logic_soundness_score = total_logic / count;
        let confidence_calibration_score = total_confidence / count;
        let risk_mitigation_score = total_risk_mitigation / count;

        let score = evidence_gathering_score + logic_soundness_score + confidence_calibration_score + risk_mitigation_score;

        Self {
            score: score.min(1.0),
            evidence_gathering_score,
            logic_soundness_score,
            confidence_calibration_score,
            risk_mitigation_score,
        }
    }
}

/// Council transparency score (0.0-1.0)
/// Measures transparency of council decision-making process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilTransparencyScore {
    pub score: f64,
    pub verdict_reasoning_score: f64,
    pub consensus_quality_score: f64,
    pub violation_detection_score: f64,
    pub judge_coordination_score: f64,
}

impl CouncilTransparencyScore {
    /// Analyze council transparency from verdict record
    #[cfg(feature = "full")]
    pub fn analyze(verdict: &VerdictRecord) -> Self {
        // Verdict reasoning quality: Check if rationale is comprehensive
        let verdict_reasoning_score = if verdict.final_decision.rationale.len() > 100 {
            0.3
        } else if verdict.final_decision.rationale.len() > 50 {
            0.2
        } else if verdict.final_decision.rationale.len() > 20 {
            0.1
        } else {
            0.0
        };

        // Consensus quality: Check consensus strength
        let consensus_quality_score = if verdict.council_metrics.consensus_strength >= 0.8 {
            0.3
        } else if verdict.council_metrics.consensus_strength >= 0.6 {
            0.2
        } else if verdict.council_metrics.consensus_strength >= 0.4 {
            0.1
        } else {
            0.0
        };

        // Violation detection: Check if violations match expected patterns
        let violation_detection_score = if !verdict.consensus_violations.is_empty() {
            // Violations detected - good transparency
            0.2
        } else if verdict.council_metrics.total_violations > 0 {
            // Violations in metrics but not in consensus_violations
            0.1
        } else {
            // No violations detected
            0.0
        };

        // Judge coordination: Check if all judges participated
        let judge_coordination_score = if verdict.council_metrics.judges_participated == 4 {
            0.2 // All 4 judges participated
        } else if verdict.council_metrics.judges_participated >= 3 {
            0.15
        } else if verdict.council_metrics.judges_participated >= 2 {
            0.1
        } else {
            0.0
        };

        let score: f64 = verdict_reasoning_score + consensus_quality_score + violation_detection_score + judge_coordination_score;

        Self {
            score: score.min(1.0),
            verdict_reasoning_score,
            consensus_quality_score,
            violation_detection_score,
            judge_coordination_score,
        }
    }
}

/// Verdict reasoning quality score (0.0-1.0)
/// Measures quality of council verdict reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictReasoningQualityScore {
    pub score: f64,
    pub consensus_strength_score: f64,
    pub judge_participation_score: f64,
    pub reasoning_completeness_score: f64,
    pub efficiency_score: f64,
    pub violation_accuracy_score: f64,
}

impl VerdictReasoningQualityScore {
    /// Analyze verdict reasoning quality from verdict record
    #[cfg(feature = "full")]
    pub fn analyze(verdict: &VerdictRecord) -> Self {
        // Consensus strength: +0.3 if > 0.8
        let consensus_strength_score = if verdict.council_metrics.consensus_strength >= 0.8 {
            0.3
        } else if verdict.council_metrics.consensus_strength >= 0.6 {
            0.2
        } else {
            0.1
        };

        // Judge participation: +0.2 if all 4 judges participated
        let judge_participation_score = if verdict.council_metrics.judges_participated == 4 {
            0.2
        } else if verdict.council_metrics.judges_participated >= 3 {
            0.15
        } else {
            0.1
        };

        // Reasoning completeness: Check key reasoning points
        let total_reasoning_points: usize = verdict.judge_verdicts.iter()
            .map(|j| j.key_reasoning.len())
            .sum();
        let reasoning_completeness_score = if total_reasoning_points >= 12 {
            0.2 // 3+ points per judge on average
        } else if total_reasoning_points >= 8 {
            0.15
        } else if total_reasoning_points >= 4 {
            0.1
        } else {
            0.0
        };

        // Efficiency: +0.1 if evaluation duration < 5000ms
        let efficiency_score = if verdict.council_metrics.evaluation_duration_ms < 5000 {
            0.1
        } else if verdict.council_metrics.evaluation_duration_ms < 10000 {
            0.05
        } else {
            0.0
        };

        // Violation accuracy: Check if total violations matches expected
        let violation_accuracy_score = if verdict.council_metrics.total_violations > 0 {
            0.2 // Violations detected
        } else {
            0.1 // No violations (could be good or bad)
        };

        let score: f64 = consensus_strength_score + judge_participation_score + reasoning_completeness_score + efficiency_score + violation_accuracy_score;

        Self {
            score: score.min(1.0),
            consensus_strength_score,
            judge_participation_score,
            reasoning_completeness_score,
            efficiency_score,
            violation_accuracy_score,
        }
    }

    /// Get quality level description
    pub fn quality_level(&self) -> &'static str {
        match self.score {
            s if s >= 0.9 => "Exceptional - clear rationale, all judges aligned, comprehensive analysis",
            s if s >= 0.7 => "Good - solid reasoning, minor disagreements, good coverage",
            s if s >= 0.5 => "Adequate - basic reasoning, some disagreements, partial coverage",
            s if s >= 0.3 => "Poor - unclear reasoning, significant disagreements, gaps",
            _ => "Very poor - no clear reasoning, major conflicts, missing analysis",
        }
    }
}

/// Code quality score (0.0-1.0)
/// Measures code quality against mid-level engineer standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityScore {
    pub score: f64,
    pub compilation_score: f64,
    pub structure_score: f64,
    pub error_handling_score: f64,
    pub test_coverage_score: f64,
    pub documentation_score: f64,
}

impl CodeQualityScore {
    /// Analyze code quality from file path
    /// This is a simplified analyzer - in production, would integrate with actual linting/analysis tools
    pub fn analyze(code_path: &Path) -> Self {
        // Read code file
        let code_content = match std::fs::read_to_string(code_path) {
            Ok(content) => content,
            Err(_) => {
                return Self {
                    score: 0.0,
                    compilation_score: 0.0,
                    structure_score: 0.0,
                    error_handling_score: 0.0,
                    test_coverage_score: 0.0,
                    documentation_score: 0.0,
                };
            }
        };

        // Compilation score: Check for obvious syntax errors (simplified)
        let compilation_score = if code_content.contains("fn ") || code_content.contains("pub fn ") {
            0.2 // Has functions - basic structure
        } else {
            0.0
        };

        // Structure score: Check for good structure patterns
        let structure_score = if code_content.contains("use ") && code_content.contains("mod ") {
            0.2 // Has imports and modules
        } else if code_content.contains("use ") {
            0.15
        } else {
            0.05
        };

        // Error handling score: Check for error handling patterns
        let error_handling_score = if code_content.contains("Result<") || code_content.contains("Option<") {
            0.2 // Uses Result/Option types
        } else if code_content.contains("match ") || code_content.contains("if let ") {
            0.15 // Has pattern matching
        } else if code_content.contains("unwrap()") || code_content.contains("expect(") {
            0.05 // Has some error handling but uses unwrap
        } else {
            0.0
        };

        // Test coverage score: Check for test modules
        let test_coverage_score = if code_content.contains("#[cfg(test)]") || code_content.contains("#[test]") {
            0.2 // Has tests
        } else {
            0.0
        };

        // Documentation score: Check for documentation comments
        let documentation_score = if code_content.contains("///") || code_content.contains("//!") {
            0.2 // Has documentation comments
        } else if code_content.contains("//") {
            0.1 // Has comments
        } else {
            0.0
        };

        let score: f64 = compilation_score + structure_score + error_handling_score + test_coverage_score + documentation_score;

        Self {
            score: score.min(1.0),
            compilation_score,
            structure_score,
            error_handling_score,
            test_coverage_score,
            documentation_score,
        }
    }

    /// Get quality level description
    pub fn quality_level(&self) -> &'static str {
        match self.score {
            s if s >= 0.9 => "Senior-level quality - clean, idiomatic code, comprehensive error handling, excellent test coverage",
            s if s >= 0.7 => "Mid-level quality (TARGET) - generally clean code, good error handling, reasonable structure, adequate test coverage",
            s if s >= 0.5 => "Junior-level quality - functional but rough, basic error handling, some structure issues, limited tests",
            s if s >= 0.3 => "Below standards - works but messy, poor error handling, structure problems, no tests",
            _ => "Unacceptable - doesn't work properly, no error handling, no structure, broken or missing",
        }
    }
}

/// Writing quality score (0.0-1.0)
/// Measures writing quality against mid-level writer standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingQualityScore {
    pub score: f64,
    pub clarity_score: f64,
    pub structure_score: f64,
    pub completeness_score: f64,
    pub grammar_score: f64,
    pub professionalism_score: f64,
}

impl WritingQualityScore {
    /// Analyze writing quality from content
    pub fn analyze(content: &str) -> Self {
        if content.is_empty() {
            return Self {
                score: 0.0,
                clarity_score: 0.0,
                structure_score: 0.0,
                completeness_score: 0.0,
                grammar_score: 0.0,
                professionalism_score: 0.0,
            };
        }

        // Clarity score: Check for clear language patterns
        let clarity_score = if content.len() > 500 {
            0.2 // Substantial content
        } else if content.len() > 200 {
            0.15
        } else if content.len() > 100 {
            0.1
        } else {
            0.05
        };

        // Structure score: Check for structured content (headings, lists, etc.)
        let structure_score = if content.contains("# ") || content.contains("## ") {
            0.2 // Has headings
        } else if content.contains("- ") || content.contains("* ") {
            0.15 // Has lists
        } else if content.contains("\n\n") {
            0.1 // Has paragraphs
        } else {
            0.05
        };

        // Completeness score: Check for comprehensive coverage
        let completeness_score = if content.len() > 1000 {
            0.2 // Comprehensive content
        } else if content.len() > 500 {
            0.15
        } else if content.len() > 200 {
            0.1
        } else {
            0.05
        };

        // Grammar score: Basic grammar checks (simplified)
        let grammar_score = if content.matches('.').count() > content.matches('!').count() + content.matches('?').count() {
            0.2 // Has proper sentence structure
        } else {
            0.1
        };

        // Professionalism score: Check for professional tone
        let professionalism_score = if !content.contains("lol") && !content.contains("omg") && !content.contains("wtf") {
            0.2 // Professional tone
        } else {
            0.0
        };

        let score: f64 = clarity_score + structure_score + completeness_score + grammar_score + professionalism_score;

        Self {
            score: score.min(1.0),
            clarity_score,
            structure_score,
            completeness_score,
            grammar_score,
            professionalism_score,
        }
    }

    /// Get quality level description
    pub fn quality_level(&self) -> &'static str {
        match self.score {
            s if s >= 0.9 => "Senior-level quality - exceptional clarity and structure, engaging and professional tone, comprehensive coverage",
            s if s >= 0.7 => "Mid-level quality (TARGET) - clear and well-structured, professional tone, good coverage, good grammar",
            s if s >= 0.5 => "Junior-level quality - generally clear, basic structure, adequate coverage, some grammar issues",
            s if s >= 0.3 => "Below standards - unclear in places, poor structure, incomplete coverage, grammar problems",
            _ => "Unacceptable - very unclear, no structure, major gaps, many errors",
        }
    }
}

/// Overall quality score combining all dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallQualityScore {
    pub score: f64,
    pub reasoning_depth: f64,
    pub decision_quality: f64,
    pub council_transparency: f64,
    pub output_quality: f64,
}

impl OverallQualityScore {
    /// Calculate overall quality score from component scores
    /// Formula: ReasoningDepth * 0.25 + DecisionQuality * 0.25 + CouncilTransparency * 0.15 + OutputQuality * 0.35
    pub fn calculate(
        reasoning_depth: f64,
        decision_quality: f64,
        council_transparency: f64,
        output_quality: f64,
    ) -> Self {
        let score = (
            reasoning_depth * 0.25 +
            decision_quality * 0.25 +
            council_transparency * 0.15 +
            output_quality * 0.35
        );

        Self {
            score: score.min(1.0),
            reasoning_depth,
            decision_quality,
            council_transparency,
            output_quality,
        }
    }

    /// Get quality threshold description
    pub fn threshold_description(&self) -> &'static str {
        match self.score {
            s if s >= 0.8 => "Exceeds mid-level standards",
            s if s >= 0.7 => "Meets mid-level standards (TARGET)",
            s if s >= 0.6 => "Approaching mid-level standards",
            _ => "Below mid-level standards",
        }
    }
}

#[cfg(test)]
#[cfg(feature = "full")]
mod tests {
    use super::*;
    use agent_orchestration::chain_of_thought::{DecisionPoint, DecisionType, DecisionContext, Alternative};
    use std::collections::HashMap;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_decision(reasoning: &str, alternatives_count: usize, has_risk: bool) -> DecisionPoint {
        DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: DecisionType::WorkerAssignment,
            timestamp: Utc::now(),
            context: DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: None,
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: None,
            },
            alternatives: (0..alternatives_count)
                .map(|i| Alternative {
                    option: format!("option_{}", i),
                    score: 0.5,
                    reasoning: "test".to_string(),
                    pros: vec![],
                    cons: vec![],
                    confidence: 0.5,
                })
                .collect(),
            chosen_option: "option_0".to_string(),
            reasoning: reasoning.to_string(),
            confidence: 0.7,
            risk_assessment: if has_risk {
                Some(RiskAssessment {
                    risk_level: "low".to_string(),
                    risk_factors: vec![],
                    mitigation_strategies: vec!["strategy1".to_string()],
                    fallback_options: vec![],
                })
            } else {
                None
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_reasoning_depth_analysis() {
        let decisions = vec![
            create_test_decision("This is a very long reasoning that exceeds 100 characters and provides comprehensive analysis of the problem", 3, true),
        ];

        let score = ReasoningDepthScore::analyze(&decisions);
        assert!(score.score > 0.7, "Should have good reasoning depth");
        assert!(score.reasoning_length_score > 0.0);
        assert!(score.alternatives_score > 0.0);
        assert!(score.risk_assessment_score > 0.0);
    }

    #[test]
    fn test_decision_quality_analysis() {
        let decisions = vec![
            create_test_decision("Based on the evidence, we should proceed because the data shows positive results", 2, true),
        ];

        let score = DecisionQualityScore::analyze(&decisions);
        assert!(score.score > 0.0);
        assert!(score.evidence_gathering_score > 0.0);
    }

    #[test]
    fn test_writing_quality_analysis() {
        let content = r#"
# Documentation Title

This is a comprehensive documentation that provides clear explanations.

## Section 1

- Point 1
- Point 2
- Point 3

The documentation covers all aspects thoroughly.
"#;

        let score = WritingQualityScore::analyze(content);
        assert!(score.score > 0.5);
        assert!(score.structure_score > 0.0);
        assert!(score.clarity_score > 0.0);
    }

    #[test]
    fn test_overall_quality_score() {
        let overall = OverallQualityScore::calculate(0.8, 0.75, 0.7, 0.8);
        assert!(overall.score >= 0.7);
        assert_eq!(overall.reasoning_depth, 0.8);
        assert_eq!(overall.decision_quality, 0.75);
    }
}

