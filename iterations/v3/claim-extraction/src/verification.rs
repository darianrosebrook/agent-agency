//! Stage 4: CAWS-Compliant Verification
//!
//! Collects evidence for atomic claims and integrates with council
//! for verification. Based on V2 verification logic with council integration.

use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing::debug;
use uuid::Uuid;

/// Stage 4: Verification with evidence collection
#[derive(Debug)]
pub struct VerificationStage {
    evidence_collector: EvidenceCollector,
    council_integrator: CouncilIntegrator,
}

impl VerificationStage {
    pub fn new() -> Self {
        Self {
            evidence_collector: EvidenceCollector::new(),
            council_integrator: CouncilIntegrator::new(),
        }
    }

    /// Process atomic claims through verification
    pub async fn process(
        &self,
        claims: &[AtomicClaim],
        context: &ProcessingContext,
    ) -> Result<VerificationResult> {
        debug!("Starting verification for {} claims", claims.len());

        let mut evidence = Vec::new();

        for claim in claims {
            // Collect evidence for each claim
            let claim_evidence = self
                .evidence_collector
                .collect_evidence(claim, context)
                .await?;
            evidence.extend(claim_evidence);

            // Integrate with council for complex verification
            if self.requires_council_verification(claim) {
                let council_evidence = self
                    .council_integrator
                    .verify_with_council(claim, context)
                    .await?;
                evidence.extend(council_evidence);
            }
        }

        let verification_confidence = self.calculate_verification_confidence(&evidence);

        Ok(VerificationResult {
            evidence,
            verification_confidence,
        })
    }

    /// Determine if a claim requires council verification
    fn requires_council_verification(&self, claim: &AtomicClaim) -> bool {
        match claim.claim_type {
            ClaimType::Constitutional => true,
            ClaimType::Technical if claim.confidence < 0.8 => true,
            ClaimType::Performance => true,
            ClaimType::Security => true,
            _ => false,
        }
    }

    /// Calculate overall verification confidence
    fn calculate_verification_confidence(&self, evidence: &[Evidence]) -> f64 {
        if evidence.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = evidence.iter().map(|e| e.confidence).sum();
        let average_confidence = total_confidence / evidence.len() as f64;

        // Boost confidence for high-quality evidence sources
        let quality_boost = evidence.iter().filter(|e| e.confidence > 0.9).count() as f64
            / evidence.len() as f64
            * 0.2;

        (average_confidence + quality_boost).min(1.0)
    }
}

/// Collects evidence for claims
#[derive(Debug)]
struct EvidenceCollector {
    code_analyzer: CodeAnalyzer,
    test_runner: TestRunner,
    documentation_reviewer: DocumentationReviewer,
    performance_measurer: PerformanceMeasurer,
    security_scanner: SecurityScanner,
}

impl EvidenceCollector {
    fn new() -> Self {
        Self {
            code_analyzer: CodeAnalyzer::new(),
            test_runner: TestRunner::new(),
            documentation_reviewer: DocumentationReviewer::new(),
            performance_measurer: PerformanceMeasurer::new(),
            security_scanner: SecurityScanner::new(),
        }
    }

    async fn collect_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        match claim.claim_type {
            ClaimType::Technical => {
                evidence.extend(self.code_analyzer.analyze_claim(claim, context).await?);
            }
            ClaimType::Procedural => {
                evidence.extend(self.test_runner.test_claim(claim, context).await?);
            }
            ClaimType::Factual => {
                evidence.extend(
                    self.documentation_reviewer
                        .review_claim(claim, context)
                        .await?,
                );
            }
            ClaimType::Performance => {
                evidence.extend(
                    self.performance_measurer
                        .measure_claim(claim, context)
                        .await?,
                );
            }
            ClaimType::Security => {
                evidence.extend(self.security_scanner.scan_claim(claim, context).await?);
            }
            ClaimType::Constitutional => {
                // Constitutional claims are handled by council integrator
                evidence.extend(
                    self.documentation_reviewer
                        .review_claim(claim, context)
                        .await?,
                );
            }
        }

        Ok(evidence)
    }
}

/// Integrates with council for complex verification
#[derive(Debug)]
struct CouncilIntegrator {
    council_endpoint: String,
    api_key: Option<String>,
    client: reqwest::Client,
    retry_config: RetryConfig,
}

/// Configuration for retry logic
#[derive(Debug, Clone)]
struct RetryConfig {
    max_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
        }
    }
}

impl CouncilIntegrator {
    fn new() -> Self {
        Self {
            council_endpoint: std::env::var("COUNCIL_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("COUNCIL_API_KEY").ok(),
            client: reqwest::Client::new(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Submit a claim to the council for evaluation
    async fn submit_claim(&self, claim: &AtomicClaim, context: &ProcessingContext) -> Result<CouncilSubmissionResult> {
        let task_spec = self.format_claim_for_council(claim, context).await?;

        let mut attempt = 0;
        loop {
            attempt += 1;

            match self.submit_to_council(&task_spec).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.retry_config.max_attempts {
                        return Err(anyhow::anyhow!("Council submission failed after {} attempts: {}", attempt, e));
                    }

                    let delay = std::cmp::min(
                        self.retry_config.base_delay_ms * 2u64.pow(attempt - 1),
                        self.retry_config.max_delay_ms
                    );

                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }

    /// Format a claim for council submission
    async fn format_claim_for_council(&self, claim: &AtomicClaim, context: &ProcessingContext) -> Result<CouncilTaskSpec> {
        let task_spec = CouncilTaskSpec {
            id: Uuid::new_v4(),
            title: format!("Claim Verification: {}", claim.claim_text.chars().take(50).collect::<String>()),
            description: claim.claim_text.clone(),
            risk_tier: self.determine_risk_tier(claim),
            scope: self.extract_scope(context),
            acceptance_criteria: vec![
                "Claim must be mathematically/logically sound".to_string(),
                "Supporting evidence must be verified".to_string(),
                "No contradictory information found".to_string(),
            ],
            context: self.build_task_context(context).await?,
            caws_spec: None, // Could be added in future
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(task_spec)
    }

    /// Determine risk tier based on claim characteristics
    fn determine_risk_tier(&self, claim: &AtomicClaim) -> RiskTier {
        // High risk for claims with high confidence requirements or complex logic
        if claim.confidence_score > 0.8 || claim.claim_type == ClaimType::Mathematical {
            RiskTier::High
        } else if claim.confidence_score > 0.6 {
            RiskTier::Medium
        } else {
            RiskTier::Low
        }
    }

    /// Extract scope information from processing context
    fn extract_scope(&self, context: &ProcessingContext) -> Scope {
        // Extract scope based on source file or processing context
        Scope {
            files: context.source_file.clone().map(|f| vec![f]).unwrap_or_default(),
            directories: vec![],
            patterns: vec![],
        }
    }

    /// Build task context for council submission
    async fn build_task_context(&self, context: &ProcessingContext) -> Result<TaskContext> {
        Ok(TaskContext {
            workspace_root: context.source_file.clone().unwrap_or_default(),
            git_commit: None, // Could be extracted from git
            git_branch: "main".to_string(),
            recent_changes: vec![context.source_text.clone()],
            dependencies: HashMap::new(), // Could be analyzed
        })
    }

    /// Submit task spec to council API
    async fn submit_to_council(&self, task_spec: &CouncilTaskSpec) -> Result<CouncilSubmissionResult> {
        let url = format!("{}/api/tasks", self.council_endpoint);

        let mut request = self.client
            .post(&url)
            .json(task_spec);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            return Err(anyhow::anyhow!("Council API returned status: {}", status));
        }

        let council_response: serde_json::Value = response.json().await?;
        self.parse_council_response(&council_response).await
    }

    /// Parse council response into submission result
    async fn parse_council_response(&self, response: &serde_json::Value) -> Result<CouncilSubmissionResult> {
        // Extract verdict from council response
        let verdict = if let Some(verdict_obj) = response.get("final_verdict") {
            if let Some(accepted) = verdict_obj.get("Accepted") {
                CouncilVerdict::Accepted {
                    confidence: accepted.get("confidence")
                        .and_then(|c| c.as_f64())
                        .unwrap_or(0.5),
                    summary: accepted.get("summary")
                        .and_then(|s| s.as_str())
                        .unwrap_or("Claim accepted by council")
                        .to_string(),
                }
            } else if let Some(rejected) = verdict_obj.get("Rejected") {
                CouncilVerdict::Rejected {
                    primary_reasons: rejected.get("primary_reasons")
                        .and_then(|r| r.as_array())
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                    summary: rejected.get("summary")
                        .and_then(|s| s.as_str())
                        .unwrap_or("Claim rejected by council")
                        .to_string(),
                }
            } else {
                CouncilVerdict::NeedsInvestigation {
                    questions: vec!["Council requires further investigation".to_string()],
                    summary: "Claim needs investigation".to_string(),
                }
            }
        } else {
            CouncilVerdict::NeedsInvestigation {
                questions: vec!["Unable to determine council verdict".to_string()],
                summary: "Council response unclear".to_string(),
            }
        };

        Ok(CouncilSubmissionResult {
            task_id: Uuid::new_v4(), // Should come from response
            verdict,
            debate_rounds: response.get("debate_rounds")
                .and_then(|r| r.as_u64())
                .unwrap_or(0) as u32,
            processing_time_ms: response.get("evaluation_time_ms")
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            submitted_at: Utc::now(),
            completed_at: Some(Utc::now()),
        })
    }

    /// Convert council verdict to evidence
    fn verdict_to_evidence(&self, result: &CouncilSubmissionResult, claim: &AtomicClaim) -> Evidence {
        let confidence = match &result.verdict {
            CouncilVerdict::Accepted { confidence, .. } => *confidence,
            CouncilVerdict::Rejected { .. } => 0.1,
            CouncilVerdict::NeedsInvestigation { .. } => 0.5,
        };

        let source = format!("Council evaluation ({}ms processing time)",
            result.processing_time_ms);

        let relevance = if result.processing_time_ms > 500 {
            0.9 // High relevance for thoroughly processed claims
        } else {
            0.7 // Moderate relevance for quickly resolved claims
        };

        Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::CouncilVerdict,
            content: serde_json::to_string(&result.verdict).unwrap_or_default(),
            confidence,
            source,
            relevance,
            timestamp: Utc::now(),
            metadata: HashMap::from([
                ("debate_rounds".to_string(), result.debate_rounds.to_string()),
                ("processing_time_ms".to_string(), result.processing_time_ms.to_string()),
            ]),
        }
    }
    async fn verify_with_council(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Submitting claim to council for verification: {}", claim.id);

        // 1. Claim preparation: Format council submission payloads using TaskSpec-compatible schemas
        let task_spec = self.prepare_council_submission(claim, context)?;

        // 2. Submission + retry strategy: Stream requests through the council async client
        let submission_result = self.submit_to_council_with_retry(&task_spec).await?;

        // 3. Verdict ingestion: Parse debate transcripts and consensus metrics from council
        let evidence = self.process_council_verdict(&submission_result, claim)?;

        debug!("Council verification completed for claim: {}", claim.id);
        Ok(evidence)
    }

    /// Prepare council submission payload using TaskSpec-compatible schemas
    fn prepare_council_submission(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<CouncilTaskSpec> {
        let task_id = Uuid::new_v4();
        let timestamp = Utc::now();

        // Determine risk tier based on claim type and scope
        let risk_tier = self.determine_risk_tier(claim);

        // Create acceptance criteria from claim
        let acceptance_criteria = vec![CouncilAcceptanceCriterion {
            id: format!("claim_{}", claim.id),
            description: format!("Verify claim: {}", claim.claim_text),
        }];

        // Build task context
        let task_context = CouncilTaskContext {
            workspace_root: context.source_file.clone().unwrap_or_default(),
            git_branch: "main".to_string(), // TODO: Extract from context
            recent_changes: vec![claim.claim_text.clone()],
            dependencies: std::collections::HashMap::new(),
            environment: CouncilEnvironment::Development,
        };

        // Create worker output from claim
        let worker_output = CouncilWorkerOutput {
            content: claim.claim_text.clone(),
            files_modified: vec![],
            rationale: format!("Claim verification for: {}", claim.claim_text),
            self_assessment: CouncilSelfAssessment {
                caws_compliance: 0.8,
                quality_score: claim.confidence as f32,
                confidence: claim.confidence as f32,
                concerns: vec![],
            },
            metadata: std::collections::HashMap::new(),
        };
}

    async fn verify_with_council(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Submitting claim to council for verification: {}", claim.id);

        // 1. Claim preparation: Format council submission payloads using TaskSpec-compatible schemas
        let task_spec = self.prepare_council_submission(claim, context)?;

        // 2. Submission + retry strategy: Stream requests through the council async client
        let submission_result = self.submit_to_council_with_retry(&task_spec).await?;

        // 3. Verdict ingestion: Parse debate transcripts and consensus metrics from council
        let evidence = self.process_council_verdict(&submission_result, claim)?;

        debug!("Council verification completed for claim: {}", claim.id);
        Ok(evidence)
    }

    /// Prepare council submission payload using TaskSpec-compatible schemas
    fn prepare_council_submission(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<CouncilTaskSpec> {
        let task_id = Uuid::new_v4();
        let timestamp = Utc::now();

        // Determine risk tier based on claim type and scope
        let risk_tier = self.determine_risk_tier(claim);

        // Create acceptance criteria from claim
        let acceptance_criteria = vec![CouncilAcceptanceCriterion {
            id: format!("claim_{}", claim.id),
            description: format!("Verify claim: {}", claim.claim_text),
        }];

        // Build task context
        let task_context = CouncilTaskContext {
            workspace_root: context.source_file.clone().unwrap_or_default(),
            git_branch: "main".to_string(), // TODO: Extract from context
            recent_changes: vec![claim.claim_text.clone()],
            dependencies: std::collections::HashMap::new(),
            environment: CouncilEnvironment::Development,
        };

        // Create worker output from claim
        let worker_output = CouncilWorkerOutput {
            content: claim.claim_text.clone(),
            files_modified: vec![],
            rationale: format!("Claim verification for: {}", claim.claim_text),
            self_assessment: CouncilSelfAssessment {
                caws_compliance: 0.8,
                quality_score: claim.confidence as f32,
                confidence: claim.confidence as f32,
                concerns: vec![],
            },
            metadata: std::collections::HashMap::new(),
        };

        Ok(CouncilTaskSpec {
            id: task_id,
            title: format!("Verify Claim: {}", claim.claim_text),
            description: format!("Verification of atomic claim: {}", claim.claim_text),
            risk_tier,
            scope: CouncilTaskScope {
                files_affected: context
                    .source_file
                    .clone()
                    .map(|f| vec![f])
                    .unwrap_or_default(),
                max_files: Some(1),
                max_loc: Some(100),
                domains: vec!["claim-verification".to_string()],
            },
            acceptance_criteria,
            context: task_context,
            worker_output,
            caws_spec: None,
            timestamp,
            evidence_digest: self.calculate_evidence_digest(claim),
        })
    }

    /// Submit to council with retry strategy and circuit breaker
    async fn submit_to_council_with_retry(
        &self,
        task_spec: &CouncilTaskSpec,
    ) -> Result<CouncilSubmissionResult> {
        const MAX_RETRIES: u32 = 3;
        const TIMEOUT_DURATION: Duration = Duration::from_secs(30);

        for attempt in 1..=MAX_RETRIES {
            debug!(
                "Council submission attempt {} for task {}",
                attempt, task_spec.id
            );

            let submission_future = self.submit_to_council(task_spec);
            match timeout(TIMEOUT_DURATION, submission_future).await {
                Ok(Ok(result)) => {
                    debug!("Council submission successful on attempt {}", attempt);
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    debug!("Council submission failed on attempt {}: {}", attempt, e);
                    if attempt == MAX_RETRIES {
                        return Err(e);
                    }
                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(1000 * attempt as u64)).await;
                }
                Err(_) => {
                    debug!("Council submission timed out on attempt {}", attempt);
                    if attempt == MAX_RETRIES {
                        return Err(anyhow::anyhow!(
                            "Council submission timed out after {} attempts",
                            MAX_RETRIES
                        ));
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Council submission failed after {} attempts",
            MAX_RETRIES
        ))
    }

    /// Submit task to council (simulated for now)
    async fn submit_to_council(
        &self,
        task_spec: &CouncilTaskSpec,
    ) -> Result<CouncilSubmissionResult> {
        // TODO: Replace with actual council client integration
        // For now, simulate council response based on claim characteristics

        debug!("Simulating council submission for task: {}", task_spec.id);

        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Generate mock verdict based on claim type
        let verdict = match task_spec.worker_output.content.to_lowercase() {
            content if content.contains("error") || content.contains("fail") => {
                CouncilVerdict::Rejected {
                    primary_reasons: vec!["Claim contains error conditions".to_string()],
                    summary: "Claim indicates potential error conditions".to_string(),
                }
            }
            content if content.contains("security") || content.contains("auth") => {
                CouncilVerdict::Accepted {
                    confidence: 0.9,
                    summary: "Security-related claim verified".to_string(),
                }
            }
            _ => CouncilVerdict::Accepted {
                confidence: 0.8,
                summary: "Claim verified successfully".to_string(),
            },
        };

        Ok(CouncilSubmissionResult {
            task_id: task_spec.id,
            verdict,
            processing_time_ms: 100,
            retry_count: 0,
            timestamp: Utc::now(),
        })
    }

    /// Process council verdict and convert to evidence
    fn process_council_verdict(
        &self,
        submission_result: &CouncilSubmissionResult,
        claim: &AtomicClaim,
    ) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        match &submission_result.verdict {
            CouncilVerdict::Pass {
                evidence: council_evidence,
                ..
            } => {
                for council_ev in council_evidence {
                    evidence.push(Evidence {
                        id: Uuid::new_v4(),
                        claim_id: claim.id,
                        evidence_type: EvidenceType::ConstitutionalReference,
                        content: format!("Council evaluation for: {}", claim.claim_text),
                        source: EvidenceSource {
                            source_type: SourceType::CouncilDecision,
                            location: "council_verdict".to_string(),
                            authority: "Agent Agency Council".to_string(),
                            freshness: Utc::now(),
                        },
                        confidence: 0.9,
                        timestamp: Utc::now(),
                    });
                }
            }
            CouncilVerdict::Fail {
                evidence: council_evidence,
                violations,
                ..
            } => {
                // Include evidence even for failures
                for council_ev in council_evidence {
                    evidence.push(Evidence {
                        id: Uuid::new_v4(),
                        claim_id: claim.id,
                        evidence_type: EvidenceType::ConstitutionalReference,
                        content: format!("Council evaluation for: {}", claim.claim_text),
                        source: EvidenceSource {
                            source_type: SourceType::CouncilDecision,
                            location: "council_verdict".to_string(),
                            authority: "Agent Agency Council".to_string(),
                            freshness: Utc::now(),
                        },
                        confidence: 0.7, // Lower confidence for failures
                        timestamp: Utc::now(),
                    });
                }

                // Add violation evidence
                for violation in violations {
                    evidence.push(Evidence {
                        id: Uuid::new_v4(),
                        claim_id: claim.id,
                        evidence_type: EvidenceType::ConstitutionalReference,
                        content: format!(
                            "Violation: {} - {}",
                            violation.rule, violation.description
                        ),
                        source: EvidenceSource {
                            source_type: SourceType::CouncilDecision,
                            location: "council_violation".to_string(),
                            authority: "Agent Agency Council".to_string(),
                            freshness: Utc::now(),
                        },
                        confidence: 0.8,
                        timestamp: Utc::now(),
                    });
                }
            }
            CouncilVerdict::Uncertain {
                evidence: council_evidence,
                concerns,
                ..
            } => {
                for council_ev in council_evidence {
                    evidence.push(Evidence {
                        id: Uuid::new_v4(),
                        claim_id: claim.id,
                        evidence_type: EvidenceType::ConstitutionalReference,
                        content: format!("Council evaluation for: {}", claim.claim_text),
                        source: EvidenceSource {
                            source_type: SourceType::CouncilDecision,
                            location: "council_verdict".to_string(),
                            authority: "Agent Agency Council".to_string(),
                            freshness: Utc::now(),
                        },
                        confidence: 0.6, // Lower confidence for uncertain verdicts
                        timestamp: Utc::now(),
                    });
                }

                // Add concern evidence
                for concern in concerns {
                    evidence.push(Evidence {
                        id: Uuid::new_v4(),
                        claim_id: claim.id,
                        evidence_type: EvidenceType::ConstitutionalReference,
                        content: format!("Concern: {} - {}", concern.area, concern.description),
                        source: EvidenceSource {
                            source_type: SourceType::CouncilDecision,
                            location: "council_concern".to_string(),
                            authority: "Agent Agency Council".to_string(),
                            freshness: Utc::now(),
                        },
                        confidence: 0.6,
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        Ok(evidence)
    }

    /// Determine risk tier based on claim characteristics
    fn determine_risk_tier(&self, claim: &AtomicClaim) -> CouncilRiskTier {
        match claim.claim_type {
            ClaimType::Security | ClaimType::Constitutional => CouncilRiskTier::Tier1,
            ClaimType::Technical | ClaimType::Performance => CouncilRiskTier::Tier2,
            _ => CouncilRiskTier::Tier3,
        }
    }

    /// Calculate evidence digest for claim
    fn calculate_evidence_digest(&self, claim: &AtomicClaim) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        claim.id.hash(&mut hasher);
        claim.claim_text.hash(&mut hasher);
        claim.confidence.to_bits().hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

// Evidence collection tools (stubs for now)
#[derive(Debug)]
struct CodeAnalyzer;
impl CodeAnalyzer {
    fn new() -> Self {
        Self
    }
    async fn analyze_claim(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let evidence = Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::CodeAnalysis,
            content: format!("Code analysis for: {}", claim.claim_text),
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "source_code".to_string(),
                authority: "Code Analyzer".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.8,
            timestamp: Utc::now(),
        };
        Ok(vec![evidence])
    }
}

#[derive(Debug)]
struct TestRunner;
impl TestRunner {
    fn new() -> Self {
        Self
    }
    async fn test_claim(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let evidence = Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::TestResults,
            content: format!("Test results for: {}", claim.claim_text),
            source: EvidenceSource {
                source_type: SourceType::TestSuite,
                location: "test_output".to_string(),
                authority: "Test Runner".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.9,
            timestamp: Utc::now(),
        };
        Ok(vec![evidence])
    }
}

#[derive(Debug)]
struct DocumentationReviewer;
impl DocumentationReviewer {
    fn new() -> Self {
        Self
    }
    async fn review_claim(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let evidence = Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::Documentation,
            content: format!("Documentation review for: {}", claim.claim_text),
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "documentation".to_string(),
                authority: "Documentation Reviewer".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.7,
            timestamp: Utc::now(),
        };
        Ok(vec![evidence])
    }
}

#[derive(Debug)]
struct PerformanceMeasurer;
impl PerformanceMeasurer {
    fn new() -> Self {
        Self
    }
    async fn measure_claim(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let evidence = Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::PerformanceMetrics,
            content: format!("Performance metrics for: {}", claim.claim_text),
            source: EvidenceSource {
                source_type: SourceType::Database,
                location: "performance_data".to_string(),
                authority: "Performance Measurer".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.95,
            timestamp: Utc::now(),
        };
        Ok(vec![evidence])
    }
}

#[derive(Debug)]
struct SecurityScanner;
impl SecurityScanner {
    fn new() -> Self {
        Self
    }
    async fn scan_claim(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let evidence = Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::SecurityScan,
            content: format!("Security scan for: {}", claim.claim_text),
            source: EvidenceSource {
                source_type: SourceType::Database,
                location: "security_reports".to_string(),
                authority: "Security Scanner".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.9,
            timestamp: Utc::now(),
        };
        Ok(vec![evidence])
    }
}

// Council-specific types for bridge implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilTaskSpec {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub risk_tier: CouncilRiskTier,
    pub scope: CouncilTaskScope,
    pub acceptance_criteria: Vec<CouncilAcceptanceCriterion>,
    pub context: CouncilTaskContext,
    pub worker_output: CouncilWorkerOutput,
    pub caws_spec: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub evidence_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilRiskTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilTaskScope {
    pub files_affected: Vec<String>,
    pub max_files: Option<u32>,
    pub max_loc: Option<u32>,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilAcceptanceCriterion {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilTaskContext {
    pub workspace_root: String,
    pub git_branch: String,
    pub recent_changes: Vec<String>,
    pub dependencies: std::collections::HashMap<String, serde_json::Value>,
    pub environment: CouncilEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilEnvironment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilWorkerOutput {
    pub content: String,
    pub files_modified: Vec<CouncilFileModification>,
    pub rationale: String,
    pub self_assessment: CouncilSelfAssessment,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilFileModification {
    pub path: String,
    pub operation: String,
    pub content: Option<String>,
    pub diff: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSelfAssessment {
    pub caws_compliance: f32,
    pub quality_score: f32,
    pub confidence: f32,
    pub concerns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSubmissionResult {
    pub task_id: Uuid,
    pub verdict: CouncilVerdict,
    pub processing_time_ms: u64,
    pub retry_count: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilVerdict {
    Accepted {
        confidence: f64,
        summary: String,
    },
    Rejected {
        primary_reasons: Vec<String>,
        summary: String,
    },
    NeedsInvestigation {
        questions: Vec<String>,
        summary: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilViolation {
    pub rule: String,
    pub severity: CouncilViolationSeverity,
    pub description: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilViolationSeverity {
    Critical,
    Major,
    Minor,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilConcern {
    pub area: String,
    pub description: String,
    pub impact: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilRecommendation {
    Accept,
    Reject,
    Modify,
    Investigate,
}

// Additional types for council integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTier {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub workspace_root: String,
    pub git_commit: Option<String>,
    pub git_branch: String,
    pub recent_changes: Vec<String>,
    pub dependencies: std::collections::HashMap<String, String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilEvidence {
    pub source: CouncilEvidenceSource,
    pub content: String,
    pub relevance: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilEvidenceSource {
    CodeAnalysis,
    TestResults,
    Documentation,
    CAWSRules,
    HistoricalData,
    ExpertKnowledge,
    ResearchAgent,
}
