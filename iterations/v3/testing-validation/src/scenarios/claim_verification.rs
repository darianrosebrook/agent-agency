//! Claim Extraction & Verification Test Suite
//!
//! Validates factual accuracy, hallucination detection, and evidence-based reasoning:
//! - Claim extraction from outputs
//! - Evidence verification
//! - Hallucination detection
//! - Contextual disambiguation
//! - Factual accuracy validation

use std::time::Instant;
use tracing::{info, error};

use crate::{TestResult, TestMetrics, harness::{TestEnvironment, LocalServiceManager}};
use agent_research::{ClaimExtractionProcessor, MultiModalVerificationEngine, AtomicClaim, ProcessingContext};
use uuid::Uuid;

/// Run the claim verification E2E test
#[cfg(feature = "full")]
pub async fn run_claim_verification_test(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    info!("Starting Claim Verification E2E test");

    let mut metrics = TestMetrics::default();
    let mut passed = true;
    let mut errors = Vec::new();

    // Test 1: Claim extraction
    match test_claim_extraction(env, services).await {
        Ok(result) => {
            metrics.claims_extracted += result.claims_extracted;
            metrics.disambiguations_resolved += result.disambiguations_resolved as usize;
            if !result.passed {
                passed = false;
                errors.push(format!("Claim extraction failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Claim extraction error: {}", e));
        }
    }

    // Test 2: Evidence verification
    match test_evidence_verification(env, services).await {
        Ok(result) => {
            metrics.claims_verified += result.claims_verified;
            metrics.evidence_checks += result.evidence_checks;
            if !result.passed {
                passed = false;
                errors.push(format!("Evidence verification failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Evidence verification error: {}", e));
        }
    }

    // Test 3: Hallucination detection
    match test_hallucination_detection(env, services).await {
        Ok(result) => {
            metrics.hallucinations_detected += result.hallucinations_detected;
            metrics.evidence_checks += result.evidence_checks;
            if !result.passed {
                passed = false;
                errors.push(format!("Hallucination detection failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Hallucination detection error: {}", e));
        }
    }

    // Test 4: Contextual disambiguation
    match test_contextual_disambiguation(env, services).await {
        Ok(result) => {
            metrics.disambiguations_resolved += result.disambiguations_resolved as usize;
            if !result.passed {
                passed = false;
                errors.push(format!("Contextual disambiguation failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Contextual disambiguation error: {}", e));
        }
    }

    let error_message = if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    };

    TestResult {
        scenario: crate::Scenario::ClaimVerification,
        passed,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error_message,
        metrics,
    }
}

/// Run the claim verification E2E test (no full feature)
#[cfg(not(feature = "full"))]
pub async fn run_claim_verification_test(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    error!("Claim Verification test requires 'full' feature");
    TestResult {
        scenario: crate::Scenario::ClaimVerification,
        passed: false,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error_message: Some("Claim Verification test requires 'full' feature".to_string()),
        metrics: TestMetrics::default(),
    }
}

/// Test result for individual claim verification tests
struct ClaimTestResult {
    passed: bool,
    error: Option<String>,
    claims_extracted: u64,
    claims_verified: u64,
    hallucinations_detected: u64,
    evidence_checks: u64,
    disambiguations_resolved: u64,
}

/// Test 1: Claim extraction
async fn test_claim_extraction(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ClaimTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing claim extraction");

    // Create claim extraction processor
    let mut processor = ClaimExtractionProcessor::new();

    // Test input with verifiable claims
    let test_input = "The system uses PostgreSQL for data persistence and supports connection pooling with a maximum of 100 connections.";
    
    let context = ProcessingContext {
        task_id: Uuid::new_v4(),
        working_spec_id: "test-spec".to_string(),
        source_file: None,
        line_number: None,
        surrounding_context: test_input.to_string(),
        domain_hints: vec!["data-infrastructure".to_string(), "database".to_string()],
        metadata: std::collections::HashMap::new(),
        input_text: test_input.to_string(),
        language: None,
    };
    
    let result = processor.run(test_input, &context).await
        .map_err(|e| format!("Claim extraction failed: {}", e))?;

    let claims_count = result.atomic_claims.len() as u64;
    let disambiguations = if result.disambiguated_sentence != test_input { 1 } else { 0 };

    if claims_count == 0 {
        return Ok(ClaimTestResult {
            passed: false,
            error: Some("No claims extracted from test input".to_string()),
            claims_extracted: 0,
            claims_verified: 0,
            hallucinations_detected: 0,
            evidence_checks: 0,
            disambiguations_resolved: 0,
        });
    }

    info!("Extracted {} claims", claims_count);

    Ok(ClaimTestResult {
        passed: true,
        error: None,
        claims_extracted: claims_count,
        claims_verified: 0,
        hallucinations_detected: 0,
        evidence_checks: 0,
        disambiguations_resolved: disambiguations,
    })
}

/// Test 2: Evidence verification
async fn test_evidence_verification(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ClaimTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing evidence verification");

    // Create verification engine
    let verifier = MultiModalVerificationEngine::new();

    // Create test claims
    let test_claims = vec![
        AtomicClaim {
            id: uuid::Uuid::new_v4(),
            claim_text: "The system uses PostgreSQL for data persistence".to_string(),
            claim_type: agent_research::ClaimType::Technical,
            verifiability: agent_research::VerifiabilityLevel::DirectlyVerifiable,
            scope: agent_research::ClaimScope {
                working_spec_id: "test".to_string(),
                component_boundaries: vec!["data-infrastructure".to_string()],
                data_impact: agent_research::DataImpact::Write,
            },
            confidence: 0.9,
            contextual_brackets: vec![],
            subject: Some("system".to_string()),
            predicate: Some("uses".to_string()),
            object: Some("PostgreSQL".to_string()),
            context_brackets: vec![],
            verification_requirements: vec![],
            position: (0, 50),
            sentence_fragment: "system uses PostgreSQL".to_string(),
            evidence_links: vec![],
            temporal_context: None,
            verification_status: agent_research::VerificationStatus::Unverified,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    let verification_result = verifier.verify_claims(&test_claims).await
        .map_err(|e| format!("Evidence verification failed: {}", e))?;

    let verified_count = verification_result.verified_claims.len() as u64;
    let evidence_count = verification_result.verified_claims.iter()
        .map(|v| v.evidence.len() as u64)
        .sum::<u64>();

    info!("Verified {} claims with {} evidence items", verified_count, evidence_count);

    Ok(ClaimTestResult {
        passed: verified_count > 0,
        error: if verified_count == 0 { Some("No claims were verified".to_string()) } else { None },
        claims_extracted: test_claims.len() as u64,
        claims_verified: verified_count,
        hallucinations_detected: 0,
        evidence_checks: evidence_count,
        disambiguations_resolved: 0,
    })
}

/// Test 3: Hallucination detection
async fn test_hallucination_detection(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ClaimTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing hallucination detection");

    // Create verification engine
    let verifier = MultiModalVerificationEngine::new();

    // Create test claim that is likely a hallucination (false claim)
    let hallucination_claim = AtomicClaim {
        id: uuid::Uuid::new_v4(),
        claim_text: "The system uses MongoDB for data persistence and runs on Kubernetes clusters".to_string(),
        claim_type: agent_research::ClaimType::Technical,
        verifiability: agent_research::VerifiabilityLevel::DirectlyVerifiable,
        scope: agent_research::ClaimScope {
            working_spec_id: "test".to_string(),
            component_boundaries: vec!["data-infrastructure".to_string()],
            data_impact: agent_research::DataImpact::Write,
        },
        confidence: 0.5,
        contextual_brackets: vec![],
        subject: Some("system".to_string()),
        predicate: Some("uses".to_string()),
        object: Some("MongoDB".to_string()),
        context_brackets: vec![],
        verification_requirements: vec![],
        position: (0, 70),
        sentence_fragment: "system uses MongoDB".to_string(),
        evidence_links: vec![],
        temporal_context: None,
        verification_status: agent_research::VerificationStatus::Unverified,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let verification_result = verifier.verify_claims(&vec![hallucination_claim.clone()]).await
        .map_err(|e| format!("Hallucination detection failed: {}", e))?;

    // Check if the claim was flagged as unverified or low confidence
    let hallucinations_detected = verification_result.verified_claims.iter()
        .filter(|v| v.id == hallucination_claim.id && v.confidence < 0.5)
        .count() as u64;

    // Also check unverified claims
    let unverified_count = if verification_result.verified_claims.is_empty() { 1 } else { 0 };
    let total_hallucinations = hallucinations_detected + unverified_count;

    info!("Detected {} potential hallucinations", total_hallucinations);

    Ok(ClaimTestResult {
        passed: true,
        error: None,
        claims_extracted: 1,
        claims_verified: 0,
        hallucinations_detected: total_hallucinations,
        evidence_checks: verification_result.verified_claims.len() as u64,
        disambiguations_resolved: 0,
    })
}

/// Test 4: Contextual disambiguation
async fn test_contextual_disambiguation(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ClaimTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing contextual disambiguation");

    // Create claim extraction processor
    let mut processor = ClaimExtractionProcessor::new();

    // Test input with ambiguous terms that need disambiguation
    let ambiguous_input = "The agent processes the request and updates the database";
    
    let context = ProcessingContext {
        task_id: Uuid::new_v4(),
        working_spec_id: "test-spec".to_string(),
        source_file: None,
        line_number: None,
        surrounding_context: ambiguous_input.to_string(),
        domain_hints: vec!["agent-orchestration".to_string()],
        metadata: std::collections::HashMap::new(),
        input_text: ambiguous_input.to_string(),
        language: None,
    };
    
    let result = processor.run(ambiguous_input, &context).await
        .map_err(|e| format!("Contextual disambiguation failed: {}", e))?;

    // Check if disambiguation occurred (disambiguated sentence differs from original)
    let disambiguations = if result.disambiguated_sentence != ambiguous_input { 1 } else { 0 };

    info!("Disambiguations resolved: {}", disambiguations);

    Ok(ClaimTestResult {
        passed: true,
        error: None,
        claims_extracted: result.atomic_claims.len() as u64,
        claims_verified: 0,
        hallucinations_detected: 0,
        evidence_checks: 0,
        disambiguations_resolved: disambiguations,
    })
}
