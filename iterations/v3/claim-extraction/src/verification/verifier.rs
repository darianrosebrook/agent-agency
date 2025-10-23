//! Main claim verification logic and orchestrator
//!
//! This module contains the `MultiModalVerificationEngine` and its primary methods
//! for orchestrating claim verification across multiple modalities.

use std::sync::Arc;
use lru::LruCache;
use agent_agency_database::DatabaseClient;
use tracing::{info, warn};

use crate::types::*;
use crate::verification::types::{CoreferenceResolution as VerificationCoreferenceResolution, *};
use crate::verification::keyword_matcher::KeywordMatcher;
use crate::verification::code_extractor::CodeExtractor;
use anyhow::Result;
use futures::FutureExt;

// Re-export for convenience
// MultiModalVerificationEngine is defined in this file, not in types module

/// Multi-Modal Verification Engine for claim validation
#[derive(Debug)]
pub struct MultiModalVerificationEngine {
    /// Database client for historical claim lookups
    db_client: Option<Arc<DatabaseClient>>,
    /// Cross-reference validator for consistency checking
    cross_reference_validator: CrossReferenceValidator,
    /// Code behavior analyzer for runtime verification
    code_behavior_analyzer: CodeBehaviorAnalyzer,
    /// Authority attribution checker for source validation
    authority_checker: AuthorityAttributionChecker,
    /// Context dependency resolver for context-aware verification
    context_resolver: ContextDependencyResolver,
    /// Semantic analyzer for meaning extraction and validation
    semantic_analyzer: SemanticAnalyzer,
    /// Coreference resolution cache for performance optimization
    coreference_cache: LruCache<String, VerificationCoreferenceResolution>,
    /// Keyword matcher for text search and relevance analysis
    keyword_matcher: KeywordMatcher,
}

/// Cross-reference validator for consistency across sources
#[derive(Debug)]
struct CrossReferenceValidator {
    reference_finder: ReferenceFinder,
    consistency_checker: ConsistencyChecker,
    relationship_analyzer: RelationshipAnalyzer,
}

/// Code behavior analyzer for runtime verification
#[derive(Debug)]
struct CodeBehaviorAnalyzer {
    behavior_predictor: BehaviorPredictor,
    execution_tracer: ExecutionTracer,
}

/// Authority attribution checker for source validation
#[derive(Debug)]
struct AuthorityAttributionChecker {
    source_validator: SourceValidator,
    authority_scorer: AuthorityScorer,
    credibility_assessor: CredibilityAssessor,
}

/// Context dependency resolver for context-aware verification
#[derive(Debug)]
struct ContextDependencyResolver {
    dependency_analyzer: DependencyAnalyzer,
    context_builder: ContextBuilder,
    scope_resolver: ScopeResolver,
}

/// Semantic analyzer for meaning extraction and validation
#[derive(Debug)]
struct SemanticAnalyzer {
    semantic_parser: SemanticParser,
    meaning_extractor: MeaningExtractor,
    intent_analyzer: IntentAnalyzer,
}

// Placeholder implementations for all the validator components
#[derive(Debug)]
struct ReferenceFinder;
#[derive(Debug)]
struct ConsistencyChecker;
#[derive(Debug)]
struct RelationshipAnalyzer;
#[derive(Debug)]
struct BehaviorPredictor;
#[derive(Debug)]
struct ExecutionTracer;
#[derive(Debug)]
struct SourceValidator;
#[derive(Debug)]
struct AuthorityScorer;
#[derive(Debug)]
struct CredibilityAssessor;
#[derive(Debug)]
struct DependencyAnalyzer;
#[derive(Debug)]
struct ContextBuilder;
#[derive(Debug)]
struct ScopeResolver;
#[derive(Debug)]
struct SemanticParser;
#[derive(Debug)]
struct MeaningExtractor;
#[derive(Debug)]
struct IntentAnalyzer;

impl MultiModalVerificationEngine {
    /// Create a new verification engine with all validators initialized
    pub fn new() -> Self {
        Self::with_database_client(None)
    }

    /// Create a new verification engine with database client
    pub fn with_database_client(db_client: Option<Arc<DatabaseClient>>) -> Self {
        Self {
            db_client,
            cross_reference_validator: CrossReferenceValidator {
                reference_finder: ReferenceFinder,
                consistency_checker: ConsistencyChecker,
                relationship_analyzer: RelationshipAnalyzer,
            },
            code_behavior_analyzer: CodeBehaviorAnalyzer {
                behavior_predictor: BehaviorPredictor,
                execution_tracer: ExecutionTracer,
            },
            authority_checker: AuthorityAttributionChecker {
                source_validator: SourceValidator,
                authority_scorer: AuthorityScorer,
                credibility_assessor: CredibilityAssessor,
            },
            context_resolver: ContextDependencyResolver {
                dependency_analyzer: DependencyAnalyzer,
                context_builder: ContextBuilder,
                scope_resolver: ScopeResolver,
            },
            semantic_analyzer: SemanticAnalyzer {
                semantic_parser: SemanticParser,
                meaning_extractor: MeaningExtractor,
                intent_analyzer: IntentAnalyzer,
            },
            coreference_cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            keyword_matcher: KeywordMatcher,
        }
    }

    /// Verify claims using multi-modal analysis with cross-reference validation
    pub async fn verify_claims(&self, claims: &[AtomicClaim]) -> Result<VerificationResults> {
        let mut results = VerificationResults::default();
        results.total_processed = claims.len();

        for claim in claims {
            let verification_result = self.verify_single_claim(claim).await?;
            let was_verified = verification_result.overall_confidence > 0.7;
            
            // Convert VerificationResult to VerifiedClaim
            let verified_claim = VerifiedClaim {
                id: claim.id,
                claim_text: claim.claim_text.clone(),
                verification_status: if was_verified { 
                    VerificationStatus::Verified 
                } else { 
                    VerificationStatus::Unverified 
                },
                confidence: verification_result.overall_confidence,
                verification_results: if was_verified { 
                    VerificationStatus::Verified 
                } else { 
                    VerificationStatus::Unverified 
                },
                evidence: verification_result.evidence,
                timestamp: chrono::Utc::now(),
                original_claim: claim.claim_text.clone(),
                overall_confidence: verification_result.overall_confidence,
                verification_timestamp: chrono::Utc::now(),
            };
            results.verified_claims.push(verified_claim);

            if was_verified {
                results.successful_verifications += 1;
            }
        }

        info!(
            "Multi-modal verification completed: {}/{} claims verified successfully",
            results.successful_verifications, results.total_processed
        );

        Ok(results)
    }

    /// Verify a single claim using all available verification modalities
    pub async fn verify_single_claim(&self, claim: &AtomicClaim) -> Result<VerificationResult> {
        // 1) Cross-refs (docs/specs/history)
        let xrefs = self.cross_reference_validate(claim).await?;
        // 2) Code behavior (static + optional dynamic)
        let code = self.verify_code_behavior(claim).await?;
        // 3) Authority/credibility
        let auth = self.assess_authority(claim).await?;
        // 4) Context, semantics
        let ctx = self.validate_context_dependencies(claim).await?;
        let sem = self.semantic_validate(claim).await?;

        // Simple weighted fusion (make weights configurable)
        let score =
            0.30 * xrefs.score +
            0.25 * code.score +
            0.20 * auth.score +
            0.15 * ctx.score +
            0.10 * sem.score;

        let status = if score > 0.75 { VerificationStatus::Verified }
                     else if score > 0.5 { VerificationStatus::PartiallyVerified }
                     else { VerificationStatus::Unverified };

        // Combine all evidence
        let mut all_evidence = xrefs.evidence;
        all_evidence.extend(code.evidence);
        all_evidence.extend(auth.evidence);
        all_evidence.extend(ctx.evidence);
        all_evidence.extend(sem.evidence);

        Ok(VerificationResult {
            evidence: all_evidence.into_iter().map(|e| Evidence {
                id: uuid::Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::CodeAnalysis,
                content: e,
                source: EvidenceSource::CodeAnalysis {
                    location: "code_analysis".to_string(),
                    authority: "system".to_string(),
                    freshness: chrono::Utc::now(),
                },
                confidence: 0.8,
                relevance: 0.9,
                timestamp: chrono::Utc::now(),
            }).collect(),
            verification_confidence: score,
            verified_claims: vec![],
            council_verification: CouncilVerificationResult {
                submitted_claims: vec![claim.id],
                council_verdict: "pending".to_string(),
                additional_evidence: vec![],
                verification_timestamp: chrono::Utc::now(),
            },
            overall_confidence: score,
        })
    }

    /// Simulate file content for testing (replace with actual file reading)
    fn simulate_file_content(&self, file_path: &str) -> String {
        // Simulate different types of documentation content
        match file_path {
            "README.md" => {
                "This project implements an agent agency system with multiple components.
                The system includes database integration, council arbitration, and claim extraction.
                Users can verify claims using multi-modal analysis including documentation search.
                The API supports various verification methods and evidence collection."
                    .to_string()
            }
            "docs/architecture.md" => "System Architecture Overview
                The agent agency consists of several key components:
                - Council: Advanced arbitration engine with learning capabilities
                - Database: Real-time health monitoring and performance tracking
                - Claim Extraction: Multi-modal verification pipeline
                - Research: Knowledge seeking and vector search capabilities
                All components integrate through standardized interfaces."
                .to_string(),
            "docs/api.md" => "API Documentation
                The system provides REST APIs for:
                - Claim verification with evidence collection
                - Council arbitration with debate rounds
                - Database health monitoring with metrics
                - Multi-modal analysis with cross-reference validation
                Authentication is required for all endpoints."
                .to_string(),
            _ => "".to_string(),
        }
    }

    /// Check if keyword appears in a relevant context
    fn is_relevant_context(&self, file_path: &str, keyword: &str, content: &str) -> bool {
        // Check if keyword appears near relevant terms
        let content_lower = content.to_lowercase();

        // Define relevant context terms based on file type
        let context_terms = match file_path {
            "README.md" => vec!["system", "project", "implements", "provides", "supports"],
            "docs/architecture.md" => vec![
                "architecture",
                "components",
                "system",
                "integrates",
                "capabilities",
            ],
            "docs/api.md" => vec![
                "api",
                "endpoints",
                "provides",
                "authentication",
                "documentation",
            ],
            _ => vec!["system", "provides", "supports"],
        };

        // Check if keyword appears near context terms
        for term in context_terms {
            if content_lower.contains(&format!("{} {}", term, keyword))
                || content_lower.contains(&format!("{} {}", keyword, term))
            {
                return true;
            }
        }

        // Check for keyword in section headers (lines starting with #)
        for line in content.lines() {
            if line.trim().starts_with('#') && line.to_lowercase().contains(keyword) {
                return true;
            }
        }

        false
    }

    /// Simulate source file content for testing
    fn simulate_source_content(&self, file_path: &str) -> String {
        match file_path {
            "src/lib.rs" => "// Main library file for the agent agency system
                // This module provides the core functionality for claim extraction and verification

                /// The main entry point for claim processing
                pub fn process_claims(claims: &[String]) -> Result<Vec<VerifiedClaim>> {
                    // Process each claim through the verification pipeline
                    // This includes multi-modal analysis and evidence collection
                    Ok(vec![])
                }

                /* Future enhancements:
                   - Add support for custom verification strategies
                   - Implement caching for improved performance
                   - Add metrics collection for monitoring
                */"
            .to_string(),
            "src/main.rs" => "// Main application entry point
                // Initializes the agent agency system with all components

                fn main() {
                    // Start the system with database, council, and verification components
                    println!(\"Agent Agency System starting...\");
                }"
            .to_string(),
            "src/index.ts" => "// TypeScript entry point for the web interface
                // Provides API endpoints for claim verification

                export function verifyClaims(claims: string[]): Promise<VerifiedClaim[]> {
                    // Implementation uses multi-modal verification
                    return Promise.resolve([]);
                }"
            .to_string(),
            "src/types.ts" => "// Type definitions for the claim verification system

                export interface VerifiedClaim {
                    text: string;
                    confidence: number;
                    evidence: Evidence[];
                }

                export interface Evidence {
                    type: string;
                    content: string;
                    confidence: number;
                }"
            .to_string(),
            _ => "".to_string(),
        }
    }

    /// Identify context requirements for a claim
    fn identify_context_requirements(&self, claim: &AtomicClaim) -> Vec<String> {
        let mut requirements = Vec::new();
        let text = &claim.claim_text;

        // Check for pronouns that need resolution
        let pronouns = ["it", "this", "that", "these", "those", "they", "them"];
        for pronoun in &pronouns {
            if text.contains(&format!(" {}", pronoun)) || text.contains(&format!("{} ", pronoun)) {
                requirements.push(format!("pronoun_resolution:{}", pronoun));
            }
        }

        // Check for technical terms that need definition
        let technical_indicators = [
            "API",
            "database",
            "algorithm",
            "function",
            "class",
            "method",
            "system",
            "component",
        ];

        for indicator in &technical_indicators {
            if text.to_lowercase().contains(&indicator.to_lowercase()) {
                requirements.push(format!("technical_definition:{}", indicator));
            }
        }

        requirements
    }

    /// Validate cross-references across multiple sources
    async fn cross_reference_validate(&self, claim: &AtomicClaim) -> Result<CheckResult> {
        // discover docs/specs
        let spec_score = self.analyze_specification_coverage(claim, &["docs", "specs"]).await?;
        // docs search (README/api/arch)
        let docs = ["README.md", "docs/architecture.md", "docs/api.md"];
        let kws = self.extract_search_keywords(&claim.claim_text);
        let mut hits = 0usize;
        for f in docs {
            let c = self.simulate_file_content(f);
            let m = self.keyword_matcher.search_keywords_in_content(&c, &kws).await?;
            let (_, rel) = self.keyword_matcher.analyze_keyword_relevance(&c, &m).await?;
            if rel > 0.0 { hits += 1; }
        }
        let doc_score = (hits as f64 / docs.len() as f64).min(1.0);
        // history
        let terms: Vec<String> = self.extract_search_keywords(&claim.claim_text);
        let hist = self.simulate_historical_lookup(&terms).await?;
        let best = hist.iter()
            .filter_map(|h| self.calculate_claim_similarity(claim, h).now_or_never().and_then(|r| r.ok()))
            .fold(0.0, f64::max);

        let score = (0.5 * spec_score + 0.3 * doc_score + 0.2 * best).min(1.0);
        Ok(CheckResult::new(score)
            .with_evidence(format!("spec:{spec_score:.2} docs:{doc_score:.2} hist:{best:.2}")))
    }

    /// Verify code behavior for runtime verification
    async fn verify_code_behavior(&self, _claim: &AtomicClaim) -> Result<CheckResult> {
        // TODO: Implement code behavior analysis
        Ok(CheckResult::new(0.5))
    }

    /// Assess authority and credibility
    async fn assess_authority(&self, claim: &AtomicClaim) -> Result<CheckResult> {
        // toy scoring: README and official docs > comments > random files
        let (mut score, mut ev) = (0.5_f64, vec![]);
        for p in ["README.md","docs/api.md","docs/architecture.md"] {
            let c = self.simulate_file_content(p);
            if c.to_lowercase().contains(&claim.claim_text.to_lowercase()) {
                score += 0.2; ev.push(format!("found in {}", p));
            }
        }
        Ok(CheckResult::new(score.min(1.0)).with_many(ev))
    }

    /// Validate context dependencies
    async fn validate_context_dependencies(&self, claim: &AtomicClaim) -> Result<CheckResult> {
        let reqs = self.identify_context_requirements(claim);
        let available = self.assess_available_context(claim, &reqs);
        let score = if reqs.is_empty() { 1.0 } else { available as f64 / reqs.len() as f64 };
        let scope = self.validate_scope_boundaries(claim);
        let final_score = (0.7*score + 0.3*scope).min(1.0);
        Ok(CheckResult::new(final_score))
    }

    /// Semantic validation
    async fn semantic_validate(&self, _claim: &AtomicClaim) -> Result<CheckResult> {
        // TODO: Implement semantic analysis
        Ok(CheckResult::new(0.6))
    }

    /// Analyze specification coverage
    async fn analyze_specification_coverage(&self, _claim: &AtomicClaim, _paths: &[&str]) -> Result<f64> {
        // TODO: Implement specification analysis
        Ok(0.5)
    }

    /// Extract searchable keywords from claim text
    fn extract_search_keywords(&self, text: &str) -> Vec<String> {
        const STOP: &[&str] = &["the","a","an","and","or","if","then","with","for","of","to","in","on","by","at","is","are","be","this","that"];
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(|w| w.to_lowercase())
            .filter(|w| w.len()>2 && !STOP.contains(&w.as_str()))
            .collect()
    }


    /// Simulate historical lookup
    async fn simulate_historical_lookup(&self, _terms: &[String]) -> Result<Vec<HistoricalClaim>> {
        // TODO: Implement historical lookup
        Ok(vec![])
    }

    /// Calculate claim similarity
    async fn calculate_claim_similarity(&self, _claim: &AtomicClaim, _historical: &HistoricalClaim) -> Result<f64> {
        // TODO: Implement similarity calculation
        Ok(0.5)
    }

    /// Assess available context
    fn assess_available_context(&self, _claim: &AtomicClaim, _reqs: &[String]) -> usize {
        // TODO: Implement context assessment
        1
    }

            /// Validate scope boundaries
            fn validate_scope_boundaries(&self, _claim: &AtomicClaim) -> f64 {
                // TODO: Implement scope validation
                0.8
            }

            /// Check test consistency and relevance
            pub async fn check_test_consistency(&self, code_output: &CodeOutput, test_output: &TestOutput) -> Result<TestConsistency> {
                let mut issues = Vec::new();
                let mut score: f32 = 1.0;

                // Parse code structure to understand what should be tested
                let code_structure = CodeExtractor.parse_code_structure(code_output)?;

                // Check test coverage for public functions
                let public_functions: Vec<_> = code_structure.functions.iter()
                    .filter(|f| f.name.contains("pub") || f.name.contains("public"))
                    .collect();

                let test_coverage = self.check_test_coverage(&public_functions, test_output)?;
                score *= test_coverage.overall_score as f32;
                issues.extend(test_coverage.issues);

                // Check test relevance - do tests match the code they're testing?
                let test_relevance = self.check_test_relevance(code_output, test_output)?;
                score *= test_relevance.overall_score as f32;
                issues.extend(test_relevance.issues);

                // Check test quality (assertions, edge cases)
                let test_quality = self.check_test_quality(test_output)?;
                score *= test_quality.overall_score as f32;
                issues.extend(test_quality.issues);

                Ok(TestConsistency {
                    overall_score: score.max(0.0) as f64,
                    issues,
                    functions_tested: test_coverage.functions_tested,
                    total_functions: public_functions.len(),
                    test_relevance_score: test_relevance.overall_score,
                    test_quality_score: test_quality.overall_score,
                })
            }

            /// Check test coverage for public functions
            fn check_test_coverage(&self, public_functions: &[&FunctionDefinition], test_output: &TestOutput) -> Result<TestCoverage> {
                let mut issues = Vec::new();
                let mut functions_tested = 0;

                for function in public_functions {
                    if self.is_function_tested(&function.name, test_output) {
                        functions_tested += 1;
                    } else {
                        issues.push(format!("Public function '{}' has no tests", function.name));
                    }
                }

                let coverage_score = if public_functions.is_empty() {
                    1.0
                } else {
                    functions_tested as f64 / public_functions.len() as f64
                };

                if coverage_score < 0.8 {
                    issues.push("Test coverage below 80% for public functions".to_string());
                }

                Ok(TestCoverage {
                    overall_score: coverage_score,
                    issues,
                    functions_tested,
                })
            }

            /// Check if a function is tested
            fn is_function_tested(&self, function_name: &str, test_output: &TestOutput) -> bool {
                let test_results = &test_output.test_results;
                // Look for test function names that include the function name
                let test_patterns = [
                    format!("test.*{}", function_name.to_lowercase()),
                    format!("{}.*test", function_name.to_lowercase()),
                    format!("it.*{}", function_name.to_lowercase()),
                ];

                for test_result in test_results {
                    let test_content = &test_result.name;
                    for pattern in &test_patterns {
                        if test_content.to_lowercase().contains(pattern) {
                            return true;
                        }
                    }

                    // Look for direct function calls in test code
                    if test_content.contains(function_name) {
                        return true;
                    }
                }
                false
            }

            /// Check test relevance - do tests match what they're testing?
            fn check_test_relevance(&self, code_output: &CodeOutput, test_output: &TestOutput) -> Result<TestRelevance> {
                let mut issues = Vec::new();
                let mut score: f32 = 1.0;

                // Check if test file names match code file names
                let code_file_name = code_output.file_path
                    .as_ref()
                    .map(|path| path.split('/').last().unwrap_or(""))
                    .unwrap_or("")
                    .split('.')
                    .next()
                    .unwrap_or("");

                let test_file_name = "test_file.rs"
                    .split('/')
                    .last()
                    .unwrap_or("")
                    .split('.')
                    .next()
                    .unwrap_or("");

                if !test_file_name.contains(code_file_name) && !code_file_name.contains(test_file_name) {
                    issues.push("Test file name doesn't correspond to code file".to_string());
                    score -= 0.2;
                }

                // Check if tests actually call the functions they claim to test
                let mut assertions_found = 0;
                let mut total_tests = 0;

                for test_result in &test_output.test_results {
                    let test_content = &test_result.name;
                    let test_lines: Vec<&str> = test_content.lines().collect();
                    
                    for line in &test_lines {
                        if line.contains("it(") || line.contains("test(") || line.contains("#[test]") {
                            total_tests += 1;
                        }
                        if line.contains("assert") || line.contains("expect") || line.contains("should") {
                            assertions_found += 1;
                        }
                    }
                }

                if total_tests > 0 {
                    let assertions_per_test = assertions_found as f64 / total_tests as f64;
                    if assertions_per_test < 1.0 {
                        issues.push(format!("Low assertion density: {:.1} assertions per test", assertions_per_test));
                        score -= 0.1;
                    }
                }

                Ok(TestRelevance {
                    overall_score: score.max(0.0) as f64,
                    issues,
                })
            }

            /// Check test quality (assertions, edge cases, etc.)
            fn check_test_quality(&self, test_output: &TestOutput) -> Result<TestQuality> {
                let mut issues = Vec::new();
                let mut score: f32 = 1.0;

                // Check for edge case testing
                let edge_case_indicators = ["null", "undefined", "empty", "max", "min", "boundary", "edge"];
                let mut edge_cases_found = 0;

                for test_result in &test_output.test_results {
                    let content = &test_result.output;
                    for indicator in &edge_case_indicators {
                        if content.to_lowercase().contains(indicator) {
                            edge_cases_found += 1;
                        }
                    }
                }

                if edge_cases_found < 2 {
                    issues.push("Limited edge case testing detected".to_string());
                    score -= 0.1;
                }

                // Check for error case testing
                let error_indicators = ["error", "exception", "throw", "fail", "panic"];
                let mut error_tests_found = 0;

                for test_result in &test_output.test_results {
                    let content = &test_result.output;
                    for indicator in &error_indicators {
                        if content.to_lowercase().contains(indicator) {
                            error_tests_found += 1;
                        }
                    }
                }

                if error_tests_found == 0 {
                    issues.push("No error case testing detected".to_string());
                    score -= 0.2;
                }

                // Check test isolation (no shared state)
                let mut has_setup_teardown = false;
                let mut total_lines = 0;
                
                for test_result in &test_output.test_results {
                    let content = &test_result.output;
                    if content.contains("beforeEach") || content.contains("before_all") {
                        has_setup_teardown = true;
                    }
                    total_lines += content.lines().count();
                }
                
                if !has_setup_teardown && total_lines > 50 {
                    issues.push("Large test file without setup/teardown - potential state sharing".to_string());
                    score -= 0.1;
                }

                Ok(TestQuality {
                    overall_score: score.max(0.0) as f64,
                    issues,
                })
            }

    /// Process claims for verification (main entry point)
    pub async fn process(&self, claims: &[AtomicClaim], context: &ProcessingContext) -> Result<VerificationResult> {
        let mut evidence = Vec::new();
        let mut overall_confidence = 0.0;
        let mut successful_verifications = 0;
        let mut verified_claims = Vec::new();

        for claim in claims {
            match self.verify_single_claim(claim).await {
                Ok(verification_result) => {
                    evidence.extend(verification_result.evidence.clone());
                    overall_confidence += verification_result.overall_confidence;
                    successful_verifications += 1;
                    
                    // Convert VerificationResult to VerifiedClaim
                    let verified_claim = VerifiedClaim {
                        id: claim.id,
                        claim_text: claim.claim_text.clone(),
                        verification_status: if verification_result.overall_confidence > 0.7 {
                            VerificationStatus::Verified
                        } else {
                            VerificationStatus::Unverified
                        },
                        confidence: verification_result.overall_confidence,
                        evidence: verification_result.evidence,
                        timestamp: chrono::Utc::now(),
                        original_claim: claim.claim_text.clone(),
                        verification_results: if verification_result.overall_confidence > 0.7 {
                            VerificationStatus::Verified
                        } else {
                            VerificationStatus::Unverified
                        },
                        overall_confidence: verification_result.overall_confidence,
                        verification_timestamp: chrono::Utc::now(),
                    };
                    verified_claims.push(verified_claim);
                }
                Err(e) => {
                    warn!("Failed to verify claim {}: {}", claim.id, e);
                }
            }
        }

        let final_confidence = if claims.is_empty() { 0.0 } else { overall_confidence / claims.len() as f64 };

        Ok(VerificationResult {
            evidence,
            verification_confidence: final_confidence,
            verified_claims,
            council_verification: CouncilVerificationResult {
                submitted_claims: claims.iter().map(|c| c.id).collect(),
                council_verdict: "Verified".to_string(),
                additional_evidence: vec![],
                verification_timestamp: chrono::Utc::now(),
            },
            overall_confidence: final_confidence,
        })
    }

    /// Process claims for verification (v2 entry point)
    pub async fn process_v2(&self, claims: &[AtomicClaim], context: &ProcessingContext) -> Result<VerificationResult> {
        self.process(claims, context).await
    }
}

/// Test consistency and relevance check result
#[derive(Debug)]
pub struct TestConsistency {
    pub overall_score: f64,
    pub issues: Vec<String>,
    pub functions_tested: usize,
    pub total_functions: usize,
    pub test_relevance_score: f64,
    pub test_quality_score: f64,
}

/// Test coverage check result
pub struct TestCoverage {
    pub overall_score: f64,
    pub issues: Vec<String>,
    pub functions_tested: usize,
}

/// Test relevance check result
pub struct TestRelevance {
    pub overall_score: f64,
    pub issues: Vec<String>,
}

/// Test quality check result
pub struct TestQuality {
    pub overall_score: f64,
    pub issues: Vec<String>,
}
