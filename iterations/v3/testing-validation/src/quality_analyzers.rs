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
            // Expanded evidence keywords for better detection
            let evidence_keywords = [
                "because", "due to", "based on", "evidence", "data",
                "shows", "indicates", "suggests", "demonstrates", "reveals",
                "according to", "from", "using", "with", "found", "detected",
                "analysis", "results", "findings", "observation", "measurement"
            ];
            
            let has_evidence = evidence_keywords.iter().any(|keyword| {
                decision.reasoning.to_lowercase().contains(keyword)
            });
            
            let evidence_score = if has_evidence {
                0.25
            } else if decision.reasoning.len() > 100 {
                // Long reasoning likely contains evidence even if not explicitly stated
                0.1
            } else {
                0.0
            };
            total_evidence += evidence_score;

            // Logic soundness: Check if reasoning is coherent (has logical connectors)
            // Expanded logic keywords for better detection
            let logic_keywords = [
                "therefore", "thus", "consequently", "however", "alternatively",
                "since", "as", "given that", "so", "hence", "accordingly",
                "but", "yet", "although", "while", "whereas", "if", "then",
                "in order to", "for", "to", "because of", "as a result"
            ];
            
            let has_logic = logic_keywords.iter().any(|keyword| {
                decision.reasoning.to_lowercase().contains(keyword)
            });
            
            let logic_score = if has_logic {
                0.25
            } else if decision.reasoning.len() > 50 {
                0.15 // Basic coherence from length
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
    /// Uses REAL compilation checks and language-specific analysis
    pub fn analyze(code_path: &Path) -> Self {
        use std::process::Command;
        
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

        // Detect file type from extension
        let file_ext = code_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Compilation score: Use REAL compilation check
        let compilation_score = match file_ext {
            "rs" => {
                // Check if it compiles with cargo check
                let mut compiles = false;
                if let Some(parent) = code_path.parent() {
                    if parent.ends_with("src") {
                        if let Some(workspace_root) = parent.parent() {
                            let output = Command::new("cargo")
                                .args(&["check", "--manifest-path"])
                                .arg(workspace_root.join("Cargo.toml"))
                                .current_dir(workspace_root)
                                .output();
                            
                            if let Ok(result) = output {
                                compiles = result.status.success();
                            }
                        }
                    }
                }
                if compiles {
                    0.3 // Compiles successfully
                } else if code_content.contains("fn ") || code_content.contains("pub fn ") {
                    0.15 // Basic structure
                } else {
                    0.0
                }
            }
            "ts" | "tsx" => {
                // Check if it compiles with tsc
                let output = Command::new("tsc")
                    .args(&["--noEmit", code_path.to_string_lossy().as_ref()])
                    .output();
                
                let compiles = if let Ok(result) = output {
                    result.status.success()
                } else {
                    false
                };
                
                if compiles {
                    0.3 // Compiles successfully
                } else if code_content.contains("function") || code_content.contains("const") || code_content.contains("interface") {
                    0.15 // Basic structure
                } else {
                    0.0
                }
            }
            "py" => {
                // Check if it compiles with py_compile
                let output = Command::new("python3")
                    .args(&["-m", "py_compile", code_path.to_string_lossy().as_ref()])
                    .output();
                
                let compiles = if let Ok(result) = output {
                    result.status.success()
                } else {
                    false
                };
                
                if compiles {
                    0.3 // Compiles successfully
                } else if code_content.contains("def ") || code_content.contains("class ") {
                    0.15 // Basic structure
                } else {
                    0.0
                }
            }
            _ => {
                // Unknown type - basic structure check
                if code_content.contains("function") || code_content.contains("fn ") || code_content.contains("def ") {
                    0.1
                } else {
                    0.0
                }
            }
        };

        // Structure score: Language-specific patterns
        let structure_score = match file_ext {
            "rs" => {
                if code_content.contains("use ") && code_content.contains("mod ") {
                    0.2
                } else if code_content.contains("use ") {
                    0.15
                } else {
                    0.05
                }
            }
            "ts" | "tsx" => {
                // TypeScript: Check for imports/exports and type annotations
                let has_imports = code_content.contains("import ");
                let has_exports = code_content.contains("export ");
                let has_types = code_content.contains(": ") && (code_content.contains(": string") || code_content.contains(": number") || code_content.contains(": boolean") || code_content.contains(": any") || code_content.contains("interface ") || code_content.contains("type "));
                
                if has_imports && has_exports && has_types {
                    0.2
                } else if (has_imports || has_exports) && has_types {
                    0.18
                } else if has_imports || has_exports {
                    0.15
                } else {
                    0.05
                }
            }
            "py" => {
                if code_content.contains("import ") && code_content.contains("from ") {
                    0.2
                } else if code_content.contains("import ") {
                    0.15
                } else {
                    0.05
                }
            }
            _ => 0.05
        };

        // Error handling score: Language-specific patterns
        let error_handling_score = match file_ext {
            "rs" => {
                if code_content.contains("Result<") || code_content.contains("Option<") {
                    0.2
                } else if code_content.contains("match ") || code_content.contains("if let ") {
                    0.15
                } else if code_content.contains("unwrap()") || code_content.contains("expect(") {
                    0.05
                } else {
                    0.0
                }
            }
            "ts" | "tsx" => {
                // TypeScript: Check for error handling patterns
                let has_try_catch = code_content.contains("try {") && code_content.contains("catch");
                let has_error_handling = code_content.contains("catch") || code_content.contains("throw") || code_content.contains("Error");
                let has_conditional_error = code_content.contains("if (") && (code_content.contains("error") || code_content.contains("Error"));
                let has_null_check = code_content.contains("if (") && (code_content.contains("!== null") || code_content.contains("!== undefined") || code_content.contains("?."));
                
                if has_try_catch {
                    0.2
                } else if has_error_handling {
                    0.15
                } else if has_conditional_error || has_null_check {
                    0.1
                } else {
                    0.0
                }
            }
            "py" => {
                if code_content.contains("try:") && code_content.contains("except") {
                    0.2
                } else if code_content.contains("except") || code_content.contains("raise") {
                    0.15
                } else {
                    0.0
                }
            }
            _ => 0.0
        };

        // Test coverage score: Language-specific test patterns
        // Check for tests in same file OR test files in same directory
        let test_coverage_score = match file_ext {
            "rs" => {
                // Enhanced test detection: Check for various Rust test patterns
                let has_test_attr = code_content.contains("#[test]") || code_content.contains("#[tokio::test]");
                let has_test_mod = code_content.contains("#[cfg(test)]") || code_content.contains("#[cfg(test)]");
                let has_assert = code_content.contains("assert!") || code_content.contains("assert_eq!") || code_content.contains("assert_ne!");
                let _has_should_panic = code_content.contains("#[should_panic]");
                
                // Inline tests: Multiple indicators suggest comprehensive testing
                let has_inline_tests = has_test_attr || has_test_mod;
                let has_comprehensive_tests = has_inline_tests && has_assert;
                
                // Check for test files in same directory with better pattern matching
                let has_test_file = if let Some(parent) = code_path.parent() {
                    let file_name = code_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let base_name = file_name.replace(".rs", "");
                    
                    // Check for common Rust test file patterns
                    let test_patterns = vec![
                        format!("{}_test.rs", base_name),
                        format!("test_{}.rs", base_name),
                        format!("{}_tests.rs", base_name),
                        "test.rs".to_string(),
                        "tests.rs".to_string(),
                    ];
                    
                    test_patterns.iter().any(|pattern| {
                        parent.join(pattern).exists()
                    }) || parent.join("tests").is_dir() || parent.join("tests").join(format!("{}.rs", base_name)).exists()
                } else {
                    false
                };
                
                if has_comprehensive_tests {
                    0.2 // Full credit for comprehensive inline tests
                } else if has_inline_tests {
                    0.18 // Good credit for basic inline tests
                } else if has_test_file {
                    0.15 // Partial credit for test files
                } else if has_assert {
                    0.1 // Minimal credit for assertion code
                } else {
                    0.0
                }
            }
            "ts" | "tsx" => {
                // Enhanced test detection: Check for various test patterns
                let has_describe = code_content.contains("describe(") || code_content.contains("describe (");
                let has_it = code_content.contains("it(") || code_content.contains("it (");
                let has_test = code_content.contains("test(") || code_content.contains("test (");
                let has_before_each = code_content.contains("beforeEach") || code_content.contains("beforeAll");
                let has_after_each = code_content.contains("afterEach") || code_content.contains("afterAll");
                let has_expect = code_content.contains("expect(") || code_content.contains("assert(");
                
                // Inline tests: Multiple indicators suggest comprehensive testing
                let has_inline_tests = (has_describe || has_test) && (has_it || has_expect);
                let has_comprehensive_tests = has_inline_tests && (has_before_each || has_after_each);
                
                // Check for test files in same directory with better pattern matching
                let has_test_file = if let Some(parent) = code_path.parent() {
                    let file_name = code_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let base_name = file_name.replace(&format!(".{}", file_ext), "");
                    
                    // Check for common test file patterns
                    let test_patterns = vec![
                        format!("{}.test.ts", base_name),
                        format!("{}.test.tsx", base_name),
                        format!("{}.spec.ts", base_name),
                        format!("{}.spec.tsx", base_name),
                        format!("{}_test.ts", base_name),
                        format!("{}_test.tsx", base_name),
                        format!("test_{}.ts", base_name),
                        format!("test_{}.tsx", base_name),
                    ];
                    
                    test_patterns.iter().any(|pattern| {
                        parent.join(pattern).exists()
                    }) || parent.join("__tests__").is_dir() || parent.join("tests").is_dir()
                } else {
                    false
                };
                
                if has_comprehensive_tests {
                    0.2 // Full credit for comprehensive inline tests
                } else if has_inline_tests {
                    0.18 // Good credit for basic inline tests
                } else if has_test_file {
                    0.15 // Partial credit for test files
                } else if has_expect || has_test {
                    0.1 // Minimal credit for test-related code
                } else {
                    0.0
                }
            }
            "py" => {
                // Enhanced test detection: Check for various Python test patterns
                let has_test_function = code_content.contains("def test_");
                let has_unittest = code_content.contains("import unittest") || code_content.contains("from unittest");
                let has_pytest = code_content.contains("import pytest") || code_content.contains("from pytest");
                let has_assert = code_content.contains("assert ") || code_content.contains("self.assert");
                let _has_setup_teardown = code_content.contains("setUp") || code_content.contains("tearDown") || code_content.contains("fixture");
                
                // Inline tests: Multiple indicators suggest comprehensive testing
                let has_inline_tests = has_test_function || has_unittest || has_pytest;
                let has_comprehensive_tests = has_inline_tests && has_assert;
                
                // Check for test files in same directory with better pattern matching
                let has_test_file = if let Some(parent) = code_path.parent() {
                    let file_name = code_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let base_name = file_name.replace(".py", "");
                    
                    // Check for common Python test file patterns
                    let test_patterns = vec![
                        format!("test_{}.py", base_name),
                        format!("{}_test.py", base_name),
                        format!("test_{}.py", base_name.replace("_", "")),
                        format!("tests_{}.py", base_name),
                    ];
                    
                    test_patterns.iter().any(|pattern| {
                        parent.join(pattern).exists()
                    }) || parent.join("tests").is_dir() || parent.join("test").is_dir()
                } else {
                    false
                };
                
                if has_comprehensive_tests {
                    0.2 // Full credit for comprehensive inline tests
                } else if has_inline_tests {
                    0.18 // Good credit for basic inline tests
                } else if has_test_file {
                    0.15 // Partial credit for test files
                } else if has_assert {
                    0.1 // Minimal credit for assertion code
                } else {
                    0.0
                }
            }
            _ => 0.0
        };

        // Documentation score: Language-specific doc patterns
        // Give partial credit for regular comments, full credit for doc comments
        let documentation_score = match file_ext {
            "rs" => {
                // Enhanced doc comment detection: Check for comprehensive documentation
                let doc_comment_count = code_content.matches("///").count() + code_content.matches("//!").count();
                let doc_params = code_content.matches("# Arguments").count() + code_content.matches("/// # Arguments").count();
                let doc_returns = code_content.matches("# Returns").count() + code_content.matches("/// # Returns").count();
                let doc_examples = code_content.matches("# Examples").count() + code_content.matches("/// # Examples").count();
                let _doc_panics = code_content.matches("# Panics").count() + code_content.matches("/// # Panics").count();
                
                // Comprehensive docs have multiple sections
                let has_comprehensive_docs = doc_comment_count > 0 && (doc_params > 0 || doc_returns > 0 || doc_examples > 0);
                let has_basic_docs = doc_comment_count > 0;
                
                // Count regular comments (//)
                let comment_count = code_content.matches("//").count();
                
                // Check for module-level documentation
                let has_module_docs = code_content.contains("//!") && doc_comment_count > 2;
                
                if has_comprehensive_docs {
                    0.2 // Full credit for comprehensive doc comments
                } else if has_basic_docs || has_module_docs {
                    0.18 // Good credit for basic doc comments
                } else if comment_count > 10 {
                    0.15 // Good documentation with regular comments
                } else if comment_count > 5 {
                    0.12 // Moderate documentation
                } else if comment_count > 0 {
                    0.1 // Basic documentation
                } else {
                    0.0
                }
            }
            "ts" | "tsx" => {
                // Enhanced JSDoc detection: Check for comprehensive documentation patterns
                let jsdoc_start = code_content.matches("/**").count();
                let jsdoc_params = code_content.matches("@param").count();
                let jsdoc_returns = code_content.matches("@returns").count() + code_content.matches("@return").count();
                let jsdoc_examples = code_content.matches("@example").count();
                let _jsdoc_throws = code_content.matches("@throws").count();
                let jsdoc_descriptions = code_content.matches("* @").count();
                
                // Comprehensive JSDoc has multiple elements
                let has_comprehensive_jsdoc = jsdoc_start > 0 && (jsdoc_params > 0 || jsdoc_returns > 0 || jsdoc_examples > 0);
                let has_basic_jsdoc = jsdoc_start > 0 || jsdoc_descriptions > 0;
                
                // Count regular comments (//)
                let comment_count = code_content.matches("//").count();
                
                // Check for type documentation in interfaces/types
                let has_type_docs = code_content.contains("interface ") && comment_count > 3;
                
                if has_comprehensive_jsdoc {
                    0.2 // Full credit for comprehensive JSDoc
                } else if has_basic_jsdoc {
                    0.18 // Good credit for basic JSDoc
                } else if comment_count > 10 {
                    0.15 // Good documentation with regular comments
                } else if comment_count > 5 || has_type_docs {
                    0.12 // Moderate documentation
                } else if comment_count > 0 {
                    0.1 // Basic documentation
                } else {
                    0.0
                }
            }
            "py" => {
                // Enhanced docstring detection: Check for comprehensive documentation
                let triple_quotes = code_content.matches("\"\"\"").count();
                let single_quotes = code_content.matches("'''").count();
                // Docstrings come in pairs (opening and closing), so divide by 2
                let docstring_count = (triple_quotes / 2) + (single_quotes / 2);
                
                // Check for comprehensive docstring patterns
                let has_args_section = code_content.contains("Args:") || code_content.contains("Parameters:");
                let has_returns_section = code_content.contains("Returns:") || code_content.contains("Return:");
                let _has_raises_section = code_content.contains("Raises:") || code_content.contains("Exceptions:");
                let has_examples = code_content.contains("Example:") || code_content.contains("Examples:");
                
                // Comprehensive docstrings have multiple sections
                let has_comprehensive_docs = docstring_count > 0 && (has_args_section || has_returns_section || has_examples);
                let has_basic_docs = docstring_count > 0;
                
                // Count regular comments (#)
                let comment_count = code_content.matches("#").count();
                
                // Check for module-level docstrings
                let has_module_docs = code_content.starts_with("\"\"\"") || code_content.starts_with("'''");
                
                if has_comprehensive_docs {
                    0.2 // Full credit for comprehensive docstrings
                } else if has_basic_docs || has_module_docs {
                    0.18 // Good credit for basic docstrings
                } else if comment_count > 10 {
                    0.15 // Good documentation with regular comments
                } else if comment_count > 5 {
                    0.12 // Moderate documentation
                } else if comment_count > 0 {
                    0.1 // Basic documentation
                } else {
                    0.0
                }
            }
            _ => {
                // Generic: count all comment types
                let comment_count = code_content.matches("//").count() 
                    + code_content.matches("#").count() 
                    + code_content.matches("/*").count();
                
                if comment_count > 5 {
                    0.15
                } else if comment_count > 0 {
                    0.1
                } else {
                    0.0
                }
            }
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
        let score =
            reasoning_depth * 0.25 +
            decision_quality * 0.25 +
            council_transparency * 0.15 +
            output_quality * 0.35;

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

