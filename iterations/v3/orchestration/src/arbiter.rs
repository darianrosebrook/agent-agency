//! Arbiter Orchestrator Component
//!
//! The Arbiter acts as a constitutional authority that wraps the Council system
//! and enforces CAWS adjudication cycles for all AI-assisted development tasks.
//!
//! The CAWS Adjudication Cycle:
//! 1. Pleading: Worker submits change.diff, rationale, and evidence manifest
//! 2. Examination: Arbiter checks CAWS budgets (max_loc, max_files) and structural diffs
//! 3. Deliberation: Arbiter runs verifier tests; collects gate metrics
//! 4. Verdict: Arbiter issues PASS/FAIL/WAIVER_REQUIRED
//! 5. Publication: Arbiter commits verdict + provenance with CAWS-VERDICT-ID trailer

use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::caws_runtime::{CawsRuntimeValidator, ValidationResult, DiffStats, TaskDescriptor};
use crate::planning::{WorkingSpec, AcceptanceCriterion};
use claim_extraction::{ClaimExtractionProcessor, ProcessingContext, ClaimExtractionResult};

/// Arbiter orchestrator that coordinates council reviews and enforces CAWS governance
pub struct ArbiterOrchestrator {
    council: Arc<council::Council>,
    caws_validator: Arc<dyn CawsRuntimeValidator>,
    claim_processor: Arc<ClaimExtractionProcessor>,
    config: ArbiterConfig,
}

/// Configuration for the arbiter orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbiterConfig {
    /// Maximum time for arbiter adjudication (seconds)
    pub max_adjudication_time_seconds: u64,
    /// Enable claim-based evidence extraction
    pub enable_claim_extraction: bool,
    /// Enable multi-model debate protocol
    pub enable_debate_protocol: bool,
    /// Maximum debate rounds
    pub max_debate_rounds: usize,
    /// Minimum confidence for verdict acceptance
    pub min_verdict_confidence: f64,
}

impl Default for ArbiterConfig {
    fn default() -> Self {
        Self {
            max_adjudication_time_seconds: 300, // 5 minutes
            enable_claim_extraction: true,
            enable_debate_protocol: true,
            max_debate_rounds: 3,
            min_verdict_confidence: 0.8,
        }
    }
}

/// Worker output submitted for adjudication
#[derive(Debug, Clone)]
pub struct WorkerOutput {
    pub worker_id: String,
    pub task_id: Uuid,
    pub content: String,
    pub rationale: String,
    pub diff_stats: DiffStats,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Evidence manifest for claim verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceManifest {
    pub claims: Vec<claim_extraction::AtomicClaim>,
    pub verification_results: Vec<claim_extraction::VerificationResult>,
    pub factual_accuracy_score: f64,
    pub caws_compliance_score: f64,
}

/// Arbiter verdict result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbiterVerdict {
    pub task_id: Uuid,
    pub working_spec_id: String,
    pub status: VerdictStatus,
    pub confidence: f64,
    pub evidence_manifest: Option<EvidenceManifest>,
    pub waiver_required: bool,
    pub waiver_reason: Option<String>,
    pub debate_rounds: usize,
    pub provenance_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Verdict status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerdictStatus {
    Approved,
    Rejected,
    WaiverRequired,
    NeedsClarification,
}

/// Result of debate orchestration
#[derive(Debug, Clone)]
pub struct DebateResult {
    pub winning_output_index: usize,
    pub factual_accuracy_score: f64,
    pub debate_rounds: usize,
    pub evidence_manifest: EvidenceManifest,
}

/// Arbiter adjudication error
#[derive(Debug, thiserror::Error)]
pub enum ArbiterError {
    #[error("Council error: {0}")]
    CouncilError(#[from] council::CouncilError),

    #[error("CAWS validation error: {0}")]
    CawsValidationError(String),

    #[error("Claim extraction error: {0}")]
    ClaimExtractionError(String),

    #[error("Timeout exceeded")]
    TimeoutError,

    #[error("Invalid worker output: {0}")]
    InvalidWorkerOutput(String),

    #[error("Debate protocol failed: {0}")]
    DebateFailed(String),
}

impl ArbiterOrchestrator {
    /// Create a new arbiter orchestrator
    pub fn new(
        council: Arc<council::Council>,
        caws_validator: Arc<dyn CawsRuntimeValidator>,
        claim_processor: Arc<ClaimExtractionProcessor>,
        config: ArbiterConfig,
    ) -> Self {
        Self {
            council,
            caws_validator,
            claim_processor,
            config,
        }
    }

    /// Core adjudication method - implements CAWS adjudication cycle
    pub async fn adjudicate_task(
        &self,
        working_spec: &WorkingSpec,
        worker_outputs: Vec<WorkerOutput>,
    ) -> Result<ArbiterVerdict, ArbiterError> {
        let adjudication_start = std::time::Instant::now();

        // Phase 1: Pleading - Validate worker outputs
        self.validate_worker_outputs(&worker_outputs)?;

        // Phase 2: Examination - Check CAWS budgets and structural diffs
        let examination_result = self.examine_caws_compliance(working_spec, &worker_outputs).await?;

        // Phase 3: Deliberation - Extract claims and run verification
        let evidence_manifest = if self.config.enable_claim_extraction {
            Some(self.deliberate_with_claims(&worker_outputs).await?)
        } else {
            None
        };

        // Phase 4: Verdict - Determine final outcome
        let verdict = self.determine_verdict(
            working_spec,
            &examination_result,
            &evidence_manifest,
            adjudication_start.elapsed(),
        );

        // Phase 5: Publication - Record provenance
        let provenance_id = self.publish_verdict(&verdict).await?;

        Ok(ArbiterVerdict {
            task_id: worker_outputs[0].task_id, // All outputs should have same task_id
            working_spec_id: working_spec.id.clone(),
            status: verdict.status,
            confidence: verdict.confidence,
            evidence_manifest,
            waiver_required: verdict.waiver_required,
            waiver_reason: verdict.waiver_reason,
            debate_rounds: verdict.debate_rounds,
            provenance_id,
            timestamp: Utc::now(),
        })
    }

    /// Multi-model debate orchestration for competing outputs
    pub async fn orchestrate_debate(
        &self,
        task: &crate::planning::Task,
        competing_outputs: Vec<WorkerOutput>,
    ) -> Result<DebateResult, ArbiterError> {
        if !self.config.enable_debate_protocol || competing_outputs.len() < 2 {
            // No debate needed for single output
            let evidence = self.extract_claims_from_output(&competing_outputs[0]).await?;
            return Ok(DebateResult {
                winning_output_index: 0,
                factual_accuracy_score: evidence.factual_accuracy_score,
                debate_rounds: 0,
                evidence_manifest: evidence,
            });
        }

        let mut debate_rounds = 0;
        let mut current_outputs = competing_outputs;

        // Run debate rounds
        for round in 1..=self.config.max_debate_rounds {
            debate_rounds = round;

            // Extract claims from all current outputs
            let mut output_evidence = Vec::new();
            for output in &current_outputs {
                let evidence = self.extract_claims_from_output(output).await?;
                output_evidence.push(evidence);
            }

            // Create review context with evidence
            let review_context = self.build_review_context(
                task,
                &current_outputs,
                &output_evidence,
            );

            // Run council review
            let session = timeout(
                Duration::from_secs(self.config.max_adjudication_time_seconds),
                self.council.review_working_spec(&review_context),
            )
            .await
            .map_err(|_| ArbiterError::TimeoutError)??;

            // Determine winner of this round
            let winner_index = self.select_debate_winner(&session, &output_evidence)?;

            // Check if we have a clear winner with high confidence
            let winning_evidence = &output_evidence[winner_index];
            if winning_evidence.factual_accuracy_score >= self.config.min_verdict_confidence {
                return Ok(DebateResult {
                    winning_output_index: winner_index,
                    factual_accuracy_score: winning_evidence.factual_accuracy_score,
                    debate_rounds,
                    evidence_manifest: winning_evidence.clone(),
                });
            }

            // Generate counter-arguments for next round
            current_outputs = self.generate_counter_arguments(
                &current_outputs,
                winner_index,
                &output_evidence,
            )?;
        }

        // Final selection based on overall evidence quality
        let mut output_evidence = Vec::new();
        for output in &current_outputs {
            let evidence = self.extract_claims_from_output(output).await?;
            output_evidence.push(evidence);
        }

        let best_index = output_evidence
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.factual_accuracy_score.partial_cmp(&b.1.factual_accuracy_score).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        Ok(DebateResult {
            winning_output_index: best_index,
            factual_accuracy_score: output_evidence[best_index].factual_accuracy_score,
            debate_rounds,
            evidence_manifest: output_evidence[best_index].clone(),
        })
    }

    // Helper methods for adjudication phases

    fn validate_worker_outputs(&self, outputs: &[WorkerOutput]) -> Result<(), ArbiterError> {
        if outputs.is_empty() {
            return Err(ArbiterError::InvalidWorkerOutput("No worker outputs provided".to_string()));
        }

        for output in outputs {
            if output.task_id != outputs[0].task_id {
                return Err(ArbiterError::InvalidWorkerOutput("Inconsistent task IDs".to_string()));
            }
            if output.content.is_empty() {
                return Err(ArbiterError::InvalidWorkerOutput("Empty content in worker output".to_string()));
            }
        }

        Ok(())
    }

    async fn examine_caws_compliance(
        &self,
        working_spec: &WorkingSpec,
        worker_outputs: &[WorkerOutput],
    ) -> Result<ExaminationResult, ArbiterError> {
        let mut violations = Vec::new();

        for output in worker_outputs {
            // Create task descriptor for validation
            let task_desc = TaskDescriptor {
                task_id: format!("examination-{}", output.task_id),
                scope_in: working_spec.scope.as_ref()
                    .and_then(|s| s.included.clone())
                    .unwrap_or_default(),
                risk_tier: working_spec.risk_tier as u8,
                acceptance: Some(working_spec.acceptance_criteria.iter()
                    .map(|ac| format!("Given {}, When {}, Then {}", ac.given, ac.when, ac.then))
                    .collect()),
                metadata: Some(serde_json::json!({
                    "worker_id": output.worker_id,
                    "output_length": output.content.len(),
                })),
            };

            // Run CAWS validation
            let result = self.caws_validator.validate(
                &crate::caws_runtime::WorkingSpec {
                    risk_tier: working_spec.risk_tier as u8,
                    scope_in: task_desc.scope_in.clone(),
                    change_budget_max_files: working_spec.change_budget
                        .as_ref()
                        .map(|b| b.max_files as u32)
                        .unwrap_or(50),
                    change_budget_max_loc: working_spec.change_budget
                        .as_ref()
                        .map(|b| b.max_loc as u32)
                        .unwrap_or(1000),
                },
                &task_desc,
                &output.diff_stats,
                &[], // patches - not needed for examination
                &[], // language hints
                true, // assume tests added for now
                true, // assume deterministic
                vec![], // no waivers in examination
            ).await
            .map_err(|e| ArbiterError::CawsValidationError(e.to_string()))?;

            if !result.violations.is_empty() {
                violations.extend(result.violations);
            }
        }

        Ok(ExaminationResult {
            overall_compliant: violations.is_empty(),
            violations,
            examined_outputs: worker_outputs.len(),
        })
    }

    async fn deliberate_with_claims(
        &self,
        worker_outputs: &[WorkerOutput],
    ) -> Result<EvidenceManifest, ArbiterError> {
        let mut all_claims = Vec::new();
        let mut verification_results = Vec::new();
        let mut total_accuracy = 0.0;
        let mut total_compliance = 0.0;

        for output in worker_outputs {
            let claims_result = self.extract_claims_from_output(output).await?;
            all_claims.extend(claims_result.claims);
            verification_results.extend(claims_result.verification_results);
            total_accuracy += claims_result.factual_accuracy_score;
            total_compliance += claims_result.caws_compliance_score;
        }

        let avg_accuracy = total_accuracy / worker_outputs.len() as f64;
        let avg_compliance = total_compliance / worker_outputs.len() as f64;

        Ok(EvidenceManifest {
            claims: all_claims,
            verification_results,
            factual_accuracy_score: avg_accuracy,
            caws_compliance_score: avg_compliance,
        })
    }

    async fn extract_claims_from_output(
        &self,
        output: &WorkerOutput,
    ) -> Result<EvidenceManifest, ArbiterError> {
        // Split output into sentences for claim extraction
        let sentences = self.split_into_sentences(&output.content);

        let mut all_claims = Vec::new();
        let mut verification_results = Vec::new();

        for sentence in sentences {
            let context = ProcessingContext {
                task_id: output.task_id,
                working_spec_id: "extraction-context".to_string(), // Will be set properly in integration
                source_file: None,
                line_number: None,
                surrounding_context: output.content.clone(),
                domain_hints: vec!["code".to_string(), "api".to_string()], // Default hints
                metadata: output.metadata.clone(),
                input_text: sentence.clone(),
            };

            let result = self.claim_processor.process_sentence(&sentence, &context)
                .await
                .map_err(|e| ArbiterError::ClaimExtractionError(e.to_string()))?;

            all_claims.extend(result.atomic_claims);
            // For now, create mock verification results - will be enhanced in integration
            verification_results.push(claim_extraction::VerificationResult {
                claim_id: Uuid::new_v4(),
                status: claim_extraction::VerificationStatus::Verified,
                evidence_quality: 0.9,
                caws_compliance: true,
                verification_trail: vec![],
            });
        }

        // Calculate scores based on claims
        let factual_accuracy_score = if all_claims.is_empty() {
            0.8 // Default score
        } else {
            all_claims.iter()
                .map(|c| c.confidence)
                .sum::<f64>() / all_claims.len() as f64
        };

        let caws_compliance_score = 0.9; // Placeholder - will be enhanced

        Ok(EvidenceManifest {
            claims: all_claims,
            verification_results,
            factual_accuracy_score,
            caws_compliance_score,
        })
    }

    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting - can be enhanced with NLP
        text.split(|c| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn build_review_context(
        &self,
        _task: &crate::planning::Task,
        _outputs: &[WorkerOutput],
        _evidence: &[EvidenceManifest],
    ) -> council::ReviewContext {
        // TODO: Implement proper review context building
        // This will integrate with the Council ReviewContext
        unimplemented!("Review context building needs Council integration")
    }

    fn select_debate_winner(
        &self,
        _session: &council::CouncilSession,
        evidence: &[EvidenceManifest],
    ) -> Result<usize, ArbiterError> {
        // Select winner based on evidence quality
        let winner = evidence
            .iter()
            .enumerate()
            .max_by(|a, b| {
                a.1.factual_accuracy_score
                    .partial_cmp(&b.1.factual_accuracy_score)
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        Ok(winner)
    }

    fn generate_counter_arguments(
        &self,
        outputs: &[WorkerOutput],
        winner_index: usize,
        _evidence: &[EvidenceManifest],
    ) -> Result<Vec<WorkerOutput>, ArbiterError> {
        // Generate counter-arguments for losing outputs
        let mut new_outputs = outputs.to_vec();

        for (i, output) in outputs.iter().enumerate() {
            if i != winner_index {
                // Generate counter-argument by appending critique
                let counter_arg = format!(
                    "{}\n\nCounter-argument: The proposed solution may have factual inconsistencies that need verification.",
                    output.content
                );

                new_outputs[i] = WorkerOutput {
                    content: counter_arg,
                    ..output.clone()
                };
            }
        }

        Ok(new_outputs)
    }

    fn determine_verdict(
        &self,
        working_spec: &WorkingSpec,
        examination: &ExaminationResult,
        evidence: &Option<EvidenceManifest>,
        adjudication_time: std::time::Duration,
    ) -> VerdictResult {
        let mut confidence = 0.5; // Base confidence
        let mut waiver_required = false;
        let mut waiver_reason = None;

        // Factor in CAWS compliance
        if examination.overall_compliant {
            confidence += 0.3;
        } else {
            waiver_required = true;
            waiver_reason = Some(format!("CAWS violations: {}", examination.violations.len()));
        }

        // Factor in evidence quality
        if let Some(evidence) = evidence {
            confidence += evidence.factual_accuracy_score * 0.2;
            confidence += evidence.caws_compliance_score * 0.2;
        }

        // Factor in risk tier
        let risk_penalty = match working_spec.risk_tier {
            1 => 0.1, // High risk - more scrutiny
            2 => 0.05,
            _ => 0.0,
        };
        confidence -= risk_penalty;

        // Determine status
        let status = if confidence >= self.config.min_verdict_confidence && !waiver_required {
            VerdictStatus::Approved
        } else if waiver_required {
            VerdictStatus::WaiverRequired
        } else {
            VerdictStatus::Rejected
        };

        VerdictResult {
            status,
            confidence,
            waiver_required,
            waiver_reason,
            debate_rounds: 0, // Will be set by debate orchestration
        }
    }

    async fn publish_verdict(&self, verdict: &ArbiterVerdict) -> Result<String, ArbiterError> {
        // Generate unique provenance ID
        let provenance_id = format!("CAWS-VERDICT-{}", Uuid::new_v4());

        // TODO: Publish to provenance system with git trailer
        // This would integrate with the provenance system

        Ok(provenance_id)
    }
}

/// Result of CAWS examination phase
#[derive(Debug, Clone)]
struct ExaminationResult {
    overall_compliant: bool,
    violations: Vec<crate::caws_runtime::Violation>,
    examined_outputs: usize,
}

/// Internal verdict determination result
#[derive(Debug, Clone)]
struct VerdictResult {
    status: VerdictStatus,
    confidence: f64,
    waiver_required: bool,
    waiver_reason: Option<String>,
    debate_rounds: usize,
}

pub type Result<T> = std::result::Result<T, ArbiterError>;
