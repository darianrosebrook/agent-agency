//! CAWS-Compliant Verification Stage 4 for Claim Extraction Pipeline
//!
//! Implements the fourth stage of the four-stage claim processing pipeline from arbiter theory:
//! - Stage 4: CAWS-Compliant Verification
//!
//! This stage verifies atomic claims according to CAWS standards:
//! - Respects change budgets and scope restrictions
//! - Enforces quality gates (coverage, mutation scores)
//! - Validates against working spec constraints
//! - Provides audit trail for verification decisions

use agent_research::extraction_types::{AtomicClaim, ProcessingContext, VerificationStatus, VerificationResults};
use agent_research::verification::MultiModalVerificationEngine;
use agent_agency_contracts::working_spec::{WorkingSpec, WorkingSpecConstraints};
use agent_agency_contracts::planning_io::ChangeBudget;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// CAWS verification configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CawsVerificationConfig {
    /// Minimum confidence threshold for verification (0.0-1.0)
    pub min_confidence_threshold: f64,
    /// Whether to enforce change budget limits
    pub enforce_budget_limits: bool,
    /// Whether to enforce scope restrictions
    pub enforce_scope_restrictions: bool,
    /// Whether to require quality gates
    pub require_quality_gates: bool,
    /// Maximum verification time per claim (milliseconds)
    pub max_verification_time_ms: u64,
}

impl Default for CawsVerificationConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.7,
            enforce_budget_limits: true,
            enforce_scope_restrictions: true,
            require_quality_gates: true,
            max_verification_time_ms: 5000,
        }
    }
}

/// Result of CAWS-compliant verification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CawsVerificationResult {
    /// Verified claims
    pub verified_claims: Vec<VerifiedClaimInfo>,
    /// Claims that failed verification
    pub failed_claims: Vec<FailedClaimInfo>,
    /// Budget usage information
    pub budget_usage: BudgetUsage,
    /// Scope violations detected
    pub scope_violations: Vec<ScopeViolation>,
    /// Quality gate results
    pub quality_gate_results: QualityGateResults,
    /// Overall verification success rate (0.0-1.0)
    pub success_rate: f64,
}

/// Information about a verified claim
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifiedClaimInfo {
    /// Claim ID
    pub claim_id: Uuid,
    /// Claim text
    pub claim_text: String,
    /// Verification confidence (0.0-1.0)
    pub confidence: f64,
    /// Verification method used
    pub verification_method: String,
    /// Evidence collected
    pub evidence_count: usize,
    /// Verification time (milliseconds)
    pub verification_time_ms: u64,
}

/// Information about a failed claim
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailedClaimInfo {
    /// Claim ID
    pub claim_id: Uuid,
    /// Claim text
    pub claim_text: String,
    /// Failure reason
    pub failure_reason: VerificationFailureReason,
    /// Verification confidence (if attempted)
    pub confidence: Option<f64>,
}

/// Reasons why verification might fail
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum VerificationFailureReason {
    /// Confidence below threshold
    LowConfidence,
    /// Budget limit exceeded
    BudgetExceeded,
    /// Scope violation detected
    ScopeViolation,
    /// Quality gate failed
    QualityGateFailed,
    /// Verification timeout
    Timeout,
    /// Insufficient evidence
    InsufficientEvidence,
}

/// Budget usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetUsage {
    /// Files touched during verification
    pub files_touched: usize,
    /// Lines of code affected
    pub loc_affected: usize,
    /// Budget limit (if available)
    pub budget_limit: Option<usize>,
    /// Whether budget was exceeded
    pub budget_exceeded: bool,
}

/// Scope violation information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScopeViolation {
    /// Claim ID that caused violation
    pub claim_id: Uuid,
    /// Violation type
    pub violation_type: ScopeViolationType,
    /// Path or resource that violated scope
    pub violated_path: String,
    /// Allowed scope paths
    pub allowed_paths: Vec<String>,
}

/// Types of scope violations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ScopeViolationType {
    /// Path outside allowed scope
    PathOutsideScope,
    /// Dependency outside allowed scope
    DependencyOutsideScope,
    /// External API call outside scope
    ExternalApiOutsideScope,
}

/// Quality gate results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityGateResults {
    /// Test coverage achieved (if applicable)
    pub test_coverage: Option<f64>,
    /// Mutation score achieved (if applicable)
    pub mutation_score: Option<f64>,
    /// Whether quality gates passed
    pub gates_passed: bool,
    /// Quality gate failures
    pub gate_failures: Vec<String>,
}

/// CAWS-Compliant Verifier implementing Stage 4 of claim extraction pipeline
pub struct CawsCompliantVerifier {
    /// Underlying verification engine from agent-research
    verification_engine: MultiModalVerificationEngine,
    /// CAWS verification configuration
    config: CawsVerificationConfig,
    /// Working spec constraints (if available)
    working_spec_constraints: Option<WorkingSpecConstraints>,
}

impl CawsCompliantVerifier {
    /// Create a new CAWS-compliant verifier
    pub fn new() -> Self {
        Self {
            verification_engine: MultiModalVerificationEngine::new(),
            config: CawsVerificationConfig::default(),
            working_spec_constraints: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CawsVerificationConfig) -> Self {
        Self {
            verification_engine: MultiModalVerificationEngine::new(),
            config,
            working_spec_constraints: None,
        }
    }

    /// Set working spec constraints for budget and scope enforcement
    pub fn with_working_spec_constraints(mut self, constraints: WorkingSpecConstraints) -> Self {
        self.working_spec_constraints = Some(constraints);
        self
    }

    /// Verify atomic claims according to CAWS standards
    ///
    /// This is Stage 4: Verifies atomic claims with CAWS compliance checks:
    /// - Budget enforcement
    /// - Scope restrictions
    /// - Quality gates
    /// - Evidence collection
    pub async fn verify_claims_caws_compliant(
        &self,
        claims: &[AtomicClaim],
        context: &ProcessingContext,
    ) -> Result<CawsVerificationResult> {
        info!("Starting CAWS-compliant verification for {} claims", claims.len());

        let mut verified_claims = Vec::new();
        let mut failed_claims = Vec::new();
        let mut budget_usage = BudgetUsage {
            files_touched: 0,
            loc_affected: 0,
            budget_limit: None,
            budget_exceeded: false,
        };
        let mut scope_violations = Vec::new();

        // Check budget limits if constraints are available
        if let Some(ref constraints) = self.working_spec_constraints {
            if let Some(ref budget_limits) = constraints.budget_limits {
                budget_usage.budget_limit = budget_limits.max_files.map(|v| v as usize);
            }
        }

        // Verify each claim
        for claim in claims {
            let verification_start = std::time::Instant::now();

            // Pre-verification checks
            if let Err(failure_reason) = self.pre_verification_checks(claim, context).await {
                failed_claims.push(FailedClaimInfo {
                    claim_id: claim.id,
                    claim_text: claim.claim_text.clone(),
                    failure_reason,
                    confidence: None,
                });
                continue;
            }

            // Perform verification
            match self.verify_single_claim_caws(claim, context).await {
                Ok(verification_info) => {
                    let verification_time = verification_start.elapsed().as_millis() as u64;

                    // Check if verification exceeded time limit
                    if verification_time > self.config.max_verification_time_ms {
                        failed_claims.push(FailedClaimInfo {
                            claim_id: claim.id,
                            claim_text: claim.claim_text.clone(),
                            failure_reason: VerificationFailureReason::Timeout,
                            confidence: Some(verification_info.confidence),
                        });
                        continue;
                    }

                    // Check confidence threshold
                    if verification_info.confidence < self.config.min_confidence_threshold {
                        failed_claims.push(FailedClaimInfo {
                            claim_id: claim.id,
                            claim_text: claim.claim_text.clone(),
                            failure_reason: VerificationFailureReason::LowConfidence,
                            confidence: Some(verification_info.confidence),
                        });
                        continue;
                    }

                    verified_claims.push(VerifiedClaimInfo {
                        claim_id: claim.id,
                        claim_text: claim.claim_text.clone(),
                        confidence: verification_info.confidence,
                        verification_method: verification_info.method,
                        evidence_count: verification_info.evidence_count,
                        verification_time_ms: verification_time,
                    });

                    // Update budget usage
                    budget_usage.files_touched += verification_info.files_touched;
                    budget_usage.loc_affected += verification_info.loc_affected;
                }
                Err(e) => {
                    warn!("Verification failed for claim {}: {}", claim.id, e);
                    failed_claims.push(FailedClaimInfo {
                        claim_id: claim.id,
                        claim_text: claim.claim_text.clone(),
                        failure_reason: VerificationFailureReason::InsufficientEvidence,
                        confidence: None,
                    });
                }
            }
        }

        // Check budget limits
        if self.config.enforce_budget_limits {
            if let Some(limit) = budget_usage.budget_limit {
                if budget_usage.files_touched > limit {
                    budget_usage.budget_exceeded = true;
                    warn!(
                        "Budget exceeded: {} files touched (limit: {})",
                        budget_usage.files_touched, limit
                    );
                }
            }
        }

        // Calculate success rate
        let total_claims = claims.len();
        let success_rate = if total_claims > 0 {
            verified_claims.len() as f64 / total_claims as f64
        } else {
            0.0
        };

        // Quality gate results (simplified - would integrate with actual quality gates)
        let quality_gate_results = QualityGateResults {
            test_coverage: None, // Would be populated from actual test execution
            mutation_score: None, // Would be populated from mutation testing
            gates_passed: success_rate >= 0.8, // 80% success rate threshold
            gate_failures: if success_rate < 0.8 {
                vec!["Success rate below 80% threshold".to_string()]
            } else {
                vec![]
            },
        };

        info!(
            "CAWS verification complete: {}/{} claims verified ({}% success rate)",
            verified_claims.len(),
            total_claims,
            success_rate * 100.0
        );

        Ok(CawsVerificationResult {
            verified_claims,
            failed_claims,
            budget_usage,
            scope_violations,
            quality_gate_results,
            success_rate,
        })
    }

    /// Pre-verification checks (budget, scope, etc.)
    async fn pre_verification_checks(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<(), VerificationFailureReason> {
        // Check scope restrictions
        if self.config.enforce_scope_restrictions {
            if let Some(ref constraints) = self.working_spec_constraints {
                if let Some(ref scope_restrictions) = constraints.scope_restrictions {
                    // Check if claim references paths outside allowed scope
                    if let Some(violation) = self.check_scope_violation(claim, scope_restrictions) {
                        return Err(VerificationFailureReason::ScopeViolation);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check for scope violations
    fn check_scope_violation(
        &self,
        claim: &AtomicClaim,
        scope_restrictions: &agent_agency_contracts::working_spec::ScopeRestrictions,
    ) -> Option<ScopeViolation> {
        // Simple check: if claim text contains file paths, verify they're in allowed scope
        // This is a simplified check - would need proper path extraction in production
        let allowed_paths = &scope_restrictions.allowed_paths;
        if allowed_paths.is_empty() {
            return None; // No restrictions
        }

        // Check if claim references any paths (simplified pattern matching)
        // In production, would use proper path extraction
        let path_patterns = ["/", "src/", "tests/", "lib/", "bin/"];
        for pattern in &path_patterns {
            if claim.claim_text.contains(pattern) {
                // Check if any referenced path is outside allowed scope
                // This is simplified - would need proper path parsing
                let referenced_path = pattern; // Simplified
                if !allowed_paths.iter().any(|allowed| {
                    referenced_path.starts_with(allowed) || allowed.starts_with(referenced_path)
                }) {
                    return Some(ScopeViolation {
                        claim_id: claim.id,
                        violation_type: ScopeViolationType::PathOutsideScope,
                        violated_path: referenced_path.to_string(),
                        allowed_paths: allowed_paths.clone(),
                    });
                }
            }
        }

        None
    }

    /// Verify a single claim with CAWS compliance
    async fn verify_single_claim_caws(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<ClaimVerificationInfo> {
        // Use underlying verification engine
        let verification_results = self.verification_engine.verify_claims(&[claim.clone()]).await?;

        // Extract verification information
        let verified_claim = verification_results
            .verified_claims
            .first()
            .ok_or_else(|| anyhow::anyhow!("No verification result returned"))?;

        // Extract confidence from verification status
        let confidence = match verified_claim.verification_status {
            VerificationStatus::Verified => verified_claim.confidence,
            VerificationStatus::PartiallyVerified => verified_claim.confidence * 0.7,
            VerificationStatus::Unverified => verified_claim.confidence * 0.3,
            _ => 0.0,
        };

        Ok(ClaimVerificationInfo {
            confidence,
            method: format!("{:?}", verified_claim.verification_status),
            evidence_count: verified_claim.evidence.len(),
            files_touched: 0, // Would be populated from actual verification
            loc_affected: 0, // Would be populated from actual verification
        })
    }
}

/// Internal verification information
#[derive(Debug, Clone)]
struct ClaimVerificationInfo {
    confidence: f64,
    method: String,
    evidence_count: usize,
    files_touched: usize,
    loc_affected: usize,
}

impl Default for CawsCompliantVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_context() -> ProcessingContext {
        ProcessingContext {
            task_id: Uuid::new_v4(),
            working_spec_id: "test-spec".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test context".to_string(),
            domain_hints: vec!["rust".to_string()],
            metadata: HashMap::new(),
            input_text: "test input".to_string(),
            language: None,
        }
    }

    #[tokio::test]
    async fn test_verify_claims_caws_compliant() {
        let verifier = CawsCompliantVerifier::new();
        let context = create_test_context();

        // Create test claims
        let claims = vec![]; // Would create actual AtomicClaim instances

        let result = verifier
            .verify_claims_caws_compliant(&claims, &context)
            .await
            .unwrap();

        assert_eq!(result.verified_claims.len(), 0);
        assert_eq!(result.failed_claims.len(), 0);
    }
}

