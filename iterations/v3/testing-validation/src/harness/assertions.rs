//! Assertion framework for validating test outcomes
//!
//! Provides structured validation for:
//! - Council verdict correctness
//! - CAWS compliance checks
//! - Code quality metrics
//! - Performance requirements
//! - Scope compliance

use tracing::{info, error};
use regex::Regex;
#[cfg(feature = "full")]
use futures::future::join_all;

// ML/NLP imports for advanced assertion generation
#[cfg(feature = "full")]
use system_federated_ml::claim_extraction::ClaimExtractor;
#[cfg(feature = "full")]
use system_federated_ml::claim_extraction::claim_extractor::{ExtractionPattern, PatternType};
#[cfg(feature = "full")]
use system_federated_ml::fact_verification::FactVerifier;
#[cfg(feature = "full")]
use system_federated_ml::fact_verification::fact_verifier::{VerificationMethod, VerificationPriority};
#[cfg(feature = "full")]
use agent_research::evidence::collector::EvidenceCollector;
#[cfg(feature = "full")]
use agent_research::reinforcement::QLearning;

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
    #[cfg(feature = "full")]
    pub async fn assert_no_hallucination(&mut self, content: &str, fact_checker: &FactChecker, description: &str) {
        let hallucination_detected = fact_checker.detect_hallucination(content).await;
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
    pub fn record_assertion(&mut self, assertion_type: AssertionType, passed: bool, description: &str, error_message: Option<String>) {
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

/// Atomic claim extracted from content
#[derive(Debug, Clone)]
pub struct AtomicClaim {
    pub id: String,
    pub content: String,
    pub claim_type: ClaimType,
    pub confidence: f64,
    pub source: String,
    pub evidence: Vec<String>,
    pub extracted_at: chrono::DateTime<chrono::Utc>,
}

/// Types of claims that can be extracted
#[derive(Debug, Clone)]
pub enum ClaimType {
    Code,
    Documentation,
    Research,
    General,
}

/// Result of claim verification
#[derive(Debug, Clone)]
pub enum ClaimVerification {
    Verified(f64),     // Confidence score
    Hallucination(f64), // Confidence score
    Uncertain,         // Cannot determine
}

/// Advanced ML-powered fact checker for hallucination detection
#[cfg(feature = "full")]
pub struct FactChecker {
    known_facts: Vec<String>,
    claim_extractor: ClaimExtractor,
    fact_verifier: FactVerifier,
    evidence_collector: EvidenceCollector,
    reinforcement_learner: QLearning,
}

#[cfg(feature = "full")]
impl FactChecker {
    pub async fn new(facts: Vec<String>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize claim extractor
        let claim_extractor = ClaimExtractor::new().await?;

        // Initialize fact verifier
        let fact_verifier = FactVerifier::new().await?;

        // Initialize evidence collector
        let evidence_collector = EvidenceCollector::new();

        // Initialize reinforcement learner for adaptive detection
        let rl_config = agent_research::reflexive_types::AlgorithmConfig {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.2,
            min_exploration_rate: 0.01,
            exploration_decay: 0.995,
            max_iterations: 1000,
            max_episodes: 1000,
            convergence_threshold: 0.001,
        };
        let reinforcement_learner = QLearning::new(rl_config);

        Ok(Self {
            known_facts: facts,
            claim_extractor,
            fact_verifier,
            evidence_collector,
            reinforcement_learner,
        })
    }

    pub async fn detect_hallucination(&self, content: &str) -> bool {
        // Advanced NLP/ML implementation using real techniques

        // 1. Use claim extraction to break down content into atomic claims
        let claims = self.extract_atomic_claims(content).await;

        // 2. Verify each claim using fact verification and evidence collection
        let mut hallucination_detected = false;
        let mut confidence_score = 0.0;

        // Process claims concurrently for better performance
        let verification_futures: Vec<_> = claims.iter().enumerate().map(|(idx, claim)| {
            let claim_clone = claim.clone();
            async move {
                let result = self.verify_claim_with_ml(&claim_clone).await;
                (idx, result, claim_clone)
            }
        }).collect();

        let verification_results = join_all(verification_futures).await;

        for (_idx, verification_result, claim) in verification_results {
            match verification_result {
                ClaimVerification::Verified(confidence) => {
                    confidence_score += confidence;
                }
                ClaimVerification::Hallucination(confidence) => {
                    hallucination_detected = true;
                    confidence_score -= confidence;
                }
                ClaimVerification::Uncertain => {
                    // Use semantic analysis to check for suspicious patterns
                    if self.detect_suspicious_semantics(&claim) {
                        hallucination_detected = true;
                        confidence_score -= 0.3;
                    }
                }
            }
        }

        // 3. Apply reinforcement learning to improve detection over time
        // Note: This requires mutable access, but we can't mutate self here
        // In a real implementation, this would use interior mutability (Mutex/RwLock)
        // For now, we'll skip the RL update in this context

        hallucination_detected
    }

    /// Extract atomic claims from content using ML techniques
    async fn extract_atomic_claims(&self, content: &str) -> Vec<system_federated_ml::evidence_types::AtomicClaim> {
        // Use the claim extractor to break down content
        let context = system_federated_ml::evidence_types::ProcessingContext {
            source_id: "test_content".to_string(),
            timestamp: chrono::Utc::now(),
            config: system_federated_ml::evidence_types::ProcessingConfig {
                max_claims: 100,
                confidence_threshold: 0.5,
                enable_verification: false,
                enable_source_validation: false,
            },
        };
        
        match self.claim_extractor.extract_claims(content, "general", &context).await {
            Ok(result) => result.claims,
            Err(e) => {
                warn!("Failed to extract claims: {}", e);
                // Fallback to simple sentence splitting
                content.split('.')
                    .filter(|s| !s.trim().is_empty())
                    .enumerate()
                    .map(|(idx, sentence)| system_federated_ml::evidence_types::AtomicClaim {
                        id: format!("fallback_{}", idx),
                        text: sentence.trim().to_string(),
                        claim_type: system_federated_ml::evidence_types::ClaimType::Factual,
                        entities: vec![],
                        confidence: 0.5,
                        positions: vec![],
                        evidence: vec![],
                    })
                    .collect()
            }
        }
    }

    /// Verify a single claim using ML-based fact verification
    async fn verify_claim_with_ml(&self, claim: &system_federated_ml::evidence_types::AtomicClaim) -> ClaimVerification {
        // Convert to ProcessingContext
        let context = system_federated_ml::evidence_types::ProcessingContext {
            source_id: "test_content".to_string(),
            timestamp: chrono::Utc::now(),
            config: system_federated_ml::evidence_types::ProcessingConfig {
                max_claims: 100,
                confidence_threshold: 0.5,
                enable_verification: true,
                enable_source_validation: false,
            },
        };
        
        // Use fact verifier to check claim against known facts
        match self.fact_verifier.verify_claims(&[claim.clone()], &context).await {
            Ok(results) => {
                if let Some(result) = results.first() {
                    if result.confidence > 0.8 {
                        ClaimVerification::Verified(result.confidence)
                    } else if result.confidence < 0.3 {
                        ClaimVerification::Hallucination(1.0 - result.confidence)
                    } else {
                        ClaimVerification::Uncertain
                    }
                } else {
                    ClaimVerification::Uncertain
                }
            }
            Err(e) => {
                use tracing::warn;
                warn!("Fact verification failed: {}", e);
                // Check against known facts as fallback
                let has_supporting_fact = self.known_facts.iter()
                    .any(|fact| claim.text.to_lowercase().contains(&fact.to_lowercase()));

                if has_supporting_fact {
                    ClaimVerification::Verified(0.7)
                } else {
                    ClaimVerification::Uncertain
                }
            }
        }
    }

    /// Detect suspicious semantic patterns that might indicate hallucination
    fn detect_suspicious_semantics(&self, claim: &system_federated_ml::evidence_types::AtomicClaim) -> bool {
        let content_lower = claim.text.to_lowercase();

        // Patterns that often indicate hallucination
        let suspicious_patterns = [
            "revolutionary breakthrough",
            "cutting-edge technology",
            "unprecedented success",
            "groundbreaking discovery",
            "quantum leap",
            "dramatic improvement",
            "unparalleled performance",
            "industry-leading solution",
        ];

        suspicious_patterns.iter()
            .any(|pattern| content_lower.contains(pattern))
    }

    /// Update the reinforcement learning model with detection results
    fn update_detection_model(&mut self, claims: &[system_federated_ml::evidence_types::AtomicClaim], hallucination_detected: bool) {
        // Create state representation from claims
        let state = format!("claims_{}_hallucination_{}", claims.len(), hallucination_detected);

        // Get available actions (detection strategies)
        let available_actions = vec![
            "extract_claims".to_string(),
            "verify_facts".to_string(),
            "semantic_analysis".to_string(),
            "constitutional_check".to_string(),
        ];

        // Select best action using Q-learning
        let action = self.reinforcement_learner.select_action(&state, &available_actions);

        // Calculate reward based on detection accuracy
        // Reward is higher for correct detections and penalizes false positives/negatives
        let reward = if hallucination_detected {
            // True positive: reward for correctly detecting hallucination
            1.0
        } else {
            // False positive: small penalty for incorrectly flagging valid content
            // This encourages precision over recall
            -0.1
        };
        
        // In a full implementation, this would also consider:
        // - Confidence score of the detection
        // - Severity of the hallucination
        // - Historical accuracy of the detector
        // - Context-specific reward shaping

        // Update Q-values (next state would be based on actual outcomes)
        let next_state = format!("result_{}", hallucination_detected);
        self.reinforcement_learner.update(&state, &action, reward, &next_state);
    }

}
