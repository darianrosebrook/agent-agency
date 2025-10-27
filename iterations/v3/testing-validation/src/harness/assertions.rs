//! Assertion framework for validating test outcomes
//!
//! Provides structured validation for:
//! - Council verdict correctness
//! - CAWS compliance checks
//! - Code quality metrics
//! - Performance requirements
//! - Scope compliance

use std::collections::HashMap;
use tracing::{info, warn, error};
use regex::Regex;

/// Framework for asserting test outcomes
pub struct AssertionFramework {
    results: Vec<AssertionResult>,
}

impl AssertionFramework {
    /// Create a new assertion framework
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Assert that a Council verdict is approved
    pub fn assert_council_approved(&mut self, verdict: &CouncilVerdict, description: &str) {
        let passed = verdict.approved;
        self.record_assertion(
            AssertionType::CouncilApproval,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("Council rejected task: {}", verdict.reason.as_deref().unwrap_or("no reason provided")))
            },
        );
    }

    /// Assert CAWS compliance
    pub fn assert_caws_compliant(&mut self, compliance_result: &CawsComplianceResult, description: &str) {
        let passed = compliance_result.compliant;
        self.record_assertion(
            AssertionType::CawsCompliance,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("CAWS violations: {:?}", compliance_result.violations))
            },
        );
    }

    /// Assert code compilation
    pub fn assert_code_compiles(&mut self, output: &std::process::Output, description: &str) {
        let passed = output.status.success();
        self.record_assertion(
            AssertionType::CodeCompilation,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr)))
            },
        );
    }

    /// Assert test execution
    pub fn assert_tests_pass(&mut self, output: &std::process::Output, description: &str) {
        let passed = output.status.success();
        self.record_assertion(
            AssertionType::TestExecution,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("Tests failed: {}", String::from_utf8_lossy(&output.stderr)))
            },
        );
    }

    /// Assert coverage meets threshold
    pub fn assert_coverage_threshold(&mut self, coverage: f64, threshold: f64, description: &str) {
        let passed = coverage >= threshold;
        self.record_assertion(
            AssertionType::CoverageThreshold,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("Coverage {:.2}% below threshold {:.2}%", coverage * 100.0, threshold * 100.0))
            },
        );
    }

    /// Assert mutation score meets threshold
    pub fn assert_mutation_score(&mut self, score: f64, threshold: f64, description: &str) {
        let passed = score >= threshold;
        self.record_assertion(
            AssertionType::MutationScore,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("Mutation score {:.2}% below threshold {:.2}%", score * 100.0, threshold * 100.0))
            },
        );
    }

    /// Assert scope compliance (no files modified outside allowed paths)
    pub fn assert_scope_compliance(&mut self, modified_files: &[String], allowed_patterns: &[Regex], description: &str) {
        let violations: Vec<&String> = modified_files.iter()
            .filter(|file| !allowed_patterns.iter().any(|pattern| pattern.is_match(file)))
            .collect();

        let passed = violations.is_empty();
        self.record_assertion(
            AssertionType::ScopeCompliance,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("Files modified outside scope: {:?}", violations))
            },
        );
    }

    /// Assert citations are valid and match sources
    pub fn assert_citation_integrity(&mut self, citations: &[Citation], sources: &[SourceFile], description: &str) {
        let mut invalid_citations = Vec::new();

        for citation in citations {
            let source_exists = sources.iter().any(|source| source.matches_citation(citation));
            if !source_exists {
                invalid_citations.push(citation.clone());
            }
        }

        let passed = invalid_citations.is_empty();
        self.record_assertion(
            AssertionType::CitationIntegrity,
            passed,
            description,
            if passed {
                None
            } else {
                Some(format!("Invalid citations: {:?}", invalid_citations))
            },
        );
    }

    /// Assert no hallucination detected in generated content
    pub fn assert_no_hallucination(&mut self, content: &str, fact_checker: &FactChecker, description: &str) {
        let hallucination_detected = fact_checker.detect_hallucination(content);
        let passed = !hallucination_detected;
        self.record_assertion(
            AssertionType::HallucinationCheck,
            passed,
            description,
            if passed {
                None
            } else {
                Some("Hallucination detected in generated content".to_string())
            },
        );
    }

    /// Get overall test result
    pub fn overall_result(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }

    /// Get summary of failed assertions
    pub fn failure_summary(&self) -> Vec<String> {
        self.results.iter()
            .filter(|r| !r.passed)
            .map(|r| format!("{}: {}", r.description, r.error_message.as_deref().unwrap_or("unknown error")))
            .collect()
    }

    /// Get all assertion results
    pub fn results(&self) -> &[AssertionResult] {
        &self.results
    }

    /// Record an assertion result
    fn record_assertion(&mut self, assertion_type: AssertionType, passed: bool, description: &str, error_message: Option<String>) {
        let type_str = assertion_type.as_str().to_string();

        let result = AssertionResult {
            assertion_type,
            passed,
            description: description.to_string(),
            error_message,
        };

        if passed {
            info!("✓ {}: {}", type_str, description);
        } else {
            error!("✗ {}: {} - {}", type_str, description, result.error_message.as_deref().unwrap_or("unknown error"));
        }

        self.results.push(result);
    }
}

/// Types of assertions that can be made
#[derive(Debug, Clone)]
pub enum AssertionType {
    CouncilApproval,
    CawsCompliance,
    CodeCompilation,
    TestExecution,
    CoverageThreshold,
    MutationScore,
    ScopeCompliance,
    CitationIntegrity,
    HallucinationCheck,
}

impl AssertionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssertionType::CouncilApproval => "Council Approval",
            AssertionType::CawsCompliance => "CAWS Compliance",
            AssertionType::CodeCompilation => "Code Compilation",
            AssertionType::TestExecution => "Test Execution",
            AssertionType::CoverageThreshold => "Coverage Threshold",
            AssertionType::MutationScore => "Mutation Score",
            AssertionType::ScopeCompliance => "Scope Compliance",
            AssertionType::CitationIntegrity => "Citation Integrity",
            AssertionType::HallucinationCheck => "Hallucination Check",
        }
    }
}

/// Result of a single assertion
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub assertion_type: AssertionType,
    pub passed: bool,
    pub description: String,
    pub error_message: Option<String>,
}

/// Council verdict structure for testing
#[derive(Debug, Clone)]
pub struct CouncilVerdict {
    pub approved: bool,
    pub reason: Option<String>,
    pub confidence_score: f64,
}

/// CAWS compliance result
#[derive(Debug, Clone)]
pub struct CawsComplianceResult {
    pub compliant: bool,
    pub violations: Vec<String>,
    pub score: f64,
}

/// Citation structure for research validation
#[derive(Debug, Clone)]
pub struct Citation {
    pub source_name: String,
    pub page_or_section: Option<String>,
    pub quote: Option<String>,
}

/// Source file for citation validation
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub content: String,
}

impl SourceFile {
    pub fn matches_citation(&self, citation: &Citation) -> bool {
        self.name == citation.source_name
    }
}

/// Simple fact checker for hallucination detection
pub struct FactChecker {
    known_facts: Vec<String>,
}

impl FactChecker {
    pub fn new(facts: Vec<String>) -> Self {
        Self { known_facts: facts }
    }

    pub fn detect_hallucination(&self, content: &str) -> bool {
        // Simple implementation - check if content contains claims not in known facts
        // In a real implementation, this would use NLP/ML techniques
        let content_lower = content.to_lowercase();

        // Look for patterns that might indicate hallucination
        let hallucination_indicators = [
            "according to",
            "research shows",
            "studies indicate",
            "experts agree",
        ];

        for indicator in &hallucination_indicators {
            if content_lower.contains(indicator) {
                // Check if the claim can be verified against known facts
                let sentence = content_lower.split('.')
                    .find(|s| s.contains(indicator))
                    .unwrap_or("");

                let has_supporting_fact = self.known_facts.iter()
                    .any(|fact| sentence.contains(&fact.to_lowercase()));

                if !has_supporting_fact {
                    return true; // Potential hallucination
                }
            }
        }

        false
    }
}
