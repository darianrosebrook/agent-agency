//! CAWS Adjudication Cycle
//!
//! Implements the five-stage CAWS Adjudication Cycle:
//! 1. Pleading - Worker presents completed work
//! 2. Examination - Council reviews evidence and claims
//! 3. Deliberation - Council debates and evaluates
//! 4. Verdict - Council reaches decision
//! 5. Publication - Verdict published and work merged
//!
//! @author @darianrosebrook

use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, debug};

use agent_agency_contracts::WorkingSpec;
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::final_verdict::FinalVerdictContract;
use agent_agency_contracts::planning_io::ExecutionPlan as ContractExecutionPlan;

use crate::council::Council;
use crate::planning::council_integration::CouncilIntegration;
use crate::planning::caws_debate_scorer::CawsDebateScorer;
use crate::planning::worktree_manager::WorktreeManager;
use crate::planning::caws_quality_gates::CawsQualityGateExecutor;

/// CAWS Adjudication Cycle stages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjudicationStage {
    Pleading,
    Examination,
    Deliberation,
    Verdict,
    Publication,
}

/// Adjudication cycle result
#[derive(Debug, Clone)]
pub struct AdjudicationResult {
    pub stage: AdjudicationStage,
    pub verdict: FinalVerdictContract,
    pub approved: bool,
    pub needs_refinement: bool,
    pub refinement_reason: Option<String>,
    /// Claim extraction results from examination stage
    pub claim_extraction_results: Option<ClaimExtractionResults>,
}

/// Claim extraction results from examination stage
#[derive(Debug, Clone)]
pub struct ClaimExtractionResults {
    pub total_claims: usize,
    pub verified_claims: usize,
    pub verification_confidence: f64,
    pub evidence_count: usize,
}

/// CAWS Adjudication Cycle coordinator
pub struct CawsAdjudicationCycle {
    council: Arc<Council>,
    council_integration: Arc<dyn CouncilIntegration>,
    debate_scorer: Arc<CawsDebateScorer>,
    worktree_manager: Option<Arc<WorktreeManager>>,
    /// Claim extraction processor for factual verification (always-on)
    claim_extractor: Option<Arc<agent_research::ClaimExtractionProcessor>>,
    /// CAWS tool registry for dynamic tool discovery and invocation
    #[cfg(feature = "mcp")]
    tool_registry: Option<Arc<CawsToolRegistry>>,
    #[cfg(not(feature = "mcp"))]
    tool_registry: Option<()>, // Placeholder when mcp feature disabled
    
    /// CAWS quality gates executor for waiver-aware gate checking
    quality_gates_executor: Option<CawsQualityGateExecutor>,
}

impl CawsAdjudicationCycle {
    /// Create new CAWS adjudication cycle
    pub fn new(
        council: Arc<Council>,
        council_integration: Arc<dyn CouncilIntegration>,
        debate_scorer: Arc<CawsDebateScorer>,
    ) -> Self {
        Self::with_worktree_manager(
            council,
            council_integration,
            debate_scorer,
            None,
        )
    }

    /// Create new CAWS adjudication cycle with worktree manager
    pub fn with_worktree_manager(
        council: Arc<Council>,
        council_integration: Arc<dyn CouncilIntegration>,
        debate_scorer: Arc<CawsDebateScorer>,
        worktree_manager: Option<Arc<WorktreeManager>>,
    ) -> Self {
        Self {
            council,
            council_integration,
            debate_scorer,
            worktree_manager,
            claim_extractor: Some(Arc::new(agent_research::ClaimExtractionProcessor::new())),
            #[cfg(feature = "mcp")]
            tool_registry: None,
            #[cfg(not(feature = "mcp"))]
            tool_registry: None,
            quality_gates_executor: None,
        }
    }

    /// Create new CAWS adjudication cycle with tool registry
    #[cfg(feature = "mcp")]
    pub fn with_tool_registry(
        council: Arc<Council>,
        council_integration: Arc<dyn CouncilIntegration>,
        debate_scorer: Arc<CawsDebateScorer>,
        worktree_manager: Option<Arc<WorktreeManager>>,
        tool_registry: Option<Arc<CawsToolRegistry>>,
    ) -> Self {
        Self {
            council,
            council_integration,
            debate_scorer,
            worktree_manager,
            claim_extractor: Some(Arc::new(agent_research::ClaimExtractionProcessor::new())),
            tool_registry,
            quality_gates_executor: None,
        }
    }

    /// Create new CAWS adjudication cycle with claim extractor
    pub fn with_claim_extractor(
        council: Arc<Council>,
        council_integration: Arc<dyn CouncilIntegration>,
        debate_scorer: Arc<CawsDebateScorer>,
        worktree_manager: Option<Arc<WorktreeManager>>,
        claim_extractor: Option<Arc<agent_research::ClaimExtractionProcessor>>,
    ) -> Self {
        Self {
            council,
            council_integration,
            debate_scorer,
            worktree_manager,
            claim_extractor: claim_extractor.or_else(|| Some(Arc::new(agent_research::ClaimExtractionProcessor::new()))),
            #[cfg(feature = "mcp")]
            tool_registry: None,
            #[cfg(not(feature = "mcp"))]
            tool_registry: None,
            quality_gates_executor: None,
        }
    }

    /// Execute full CAWS adjudication cycle
    pub async fn execute_cycle(
        &self,
        artifacts: &[ExecutionArtifacts],
        working_spec: &WorkingSpec,
        execution_plan: &ContractExecutionPlan,
    ) -> Result<AdjudicationResult> {
        info!("Starting CAWS Adjudication Cycle");

        // Stage 1: Pleading - Worker presents completed work
        info!("Stage 1: Pleading - Presenting work to council");
        let pleading_result = self.stage_pleading(artifacts).await?;

        // Stage 2: Examination - Council reviews evidence
        info!("Stage 2: Examination - Council reviewing evidence");
        let (examination_result, quality_gate_result) = self.stage_examination(execution_plan, working_spec, artifacts).await?;

        // Stage 3: Deliberation - Council debates
        info!("Stage 3: Deliberation - Council deliberating");
        let deliberation_result = self.stage_deliberation(artifacts, working_spec, &examination_result, quality_gate_result.as_ref()).await?;

        // Stage 4: Verdict - Council reaches decision
        info!("Stage 4: Verdict - Council reaching decision");
        let verdict_result = self.stage_verdict(artifacts, working_spec, &examination_result).await?;

        // Stage 5: Publication - Publish verdict and merge
        info!("Stage 5: Publication - Publishing verdict");
        let publication_result = self.stage_publication(&verdict_result, artifacts).await?;

        Ok(AdjudicationResult {
            stage: AdjudicationStage::Publication,
            verdict: verdict_result.verdict.clone().unwrap_or_else(|| {
                // Create default verdict if none provided
                FinalVerdictContract {
                    decision: agent_agency_contracts::final_verdict::FinalDecision::Accept,
                    votes: vec![],
                    dissent: String::new(),
                    remediation: vec![],
                    constitutional_refs: vec![],
                    verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
                        claims_total: examination_result.total_claims as u32,
                        claims_verified: examination_result.verified_claims as u32,
                        coverage_pct: if examination_result.total_claims > 0 {
                            (examination_result.verified_claims as f32 / examination_result.total_claims as f32) * 100.0
                        } else {
                            0.0
                        },
                    },
                }
            }),
            approved: verdict_result.approved,
            needs_refinement: verdict_result.needs_refinement,
            refinement_reason: if verdict_result.refinement_reason.is_empty() {
                None
            } else {
                Some(verdict_result.refinement_reason.clone())
            },
            claim_extraction_results: Some(examination_result),
        })
    }

    /// Stage 1: Pleading - Worker presents completed work
    async fn stage_pleading(
        &self,
        artifacts: &[ExecutionArtifacts],
    ) -> Result<()> {
        // Present each artifact to council
        for artifact in artifacts {
            // Extract milestone ID from artifact metadata
            // ArtifactMetadata doesn't support arbitrary key-value storage, so use task_id as fallback
            let milestone_id = artifact.task_id.to_string();
            
            let worker_id = artifact.provenance.worker_id
                .as_ref()
                .and_then(|w| Uuid::parse_str(w).ok())
                .unwrap_or_else(Uuid::new_v4);

            self.council_integration.present_work(
                &[artifact.clone()],
                milestone_id.as_str(),
                worker_id,
            ).await?;
        }

        Ok(())
    }

    /// Stage 2: Examination - Council reviews evidence and extracts claims
    async fn stage_examination(
        &self,
        execution_plan: &ContractExecutionPlan,
        working_spec: &WorkingSpec,
        artifacts: &[ExecutionArtifacts],
    ) -> Result<(ClaimExtractionResults, Option<crate::planning::caws_quality_gates::CawsQualityGateResult>)> {
        // Convert ContractExecutionPlan to ExecutionPlan for review
        use crate::planning::plan_types::ExecutionPlan as PlanningExecutionPlan;
        let planning_execution_plan = PlanningExecutionPlan {
            contract_plan: execution_plan.clone(),
            orchestration_meta: Default::default(),
            execution_context: Default::default(),
            execution_state: None,
        };

        // Review execution plan
        let review_result = self.council_integration.review_plan(
            &planning_execution_plan,
            working_spec,
        ).await?;

        // If plan needs refinement, that's okay - it will be handled in Phase 5 refinement loop
        // Only reject if council explicitly rejected (not just requesting refinement)
        if !review_result.approved && !review_result.needs_refinement {
            return Err(anyhow::anyhow!("Plan rejected during examination: {}", 
                review_result.refinement_reason));
        }
        
        // If refinement is needed, log it but continue - refinement happens in Phase 5
        if review_result.needs_refinement {
            tracing::info!("Plan requires refinement during examination: {}", review_result.refinement_reason);
        }

        // Extract and verify claims from artifacts if claim extractor is available
        let mut total_claims = 0;
        let mut verified_claims = 0;
        let mut total_confidence = 0.0;
        let mut evidence_count = 0;

        // Discover and invoke CAWS tools for validation if tool registry is available
        #[cfg(feature = "mcp")]
        {
            if let Some(ref registry) = self.tool_registry {
                debug!("Discovering CAWS tools for examination stage");
                
                // Discover tools from MCP registry
                if let Err(e) = registry.discover_tools().await {
                    debug!("Tool discovery failed: {}", e);
                }

                // Get compliance checking tools
                let compliance_tools = registry
                    .get_tools_by_category(&crate::planning::caws_tool_registry::CawsToolCategory::ComplianceChecking)
                    .await;

                // Get quality gate tools
                let quality_tools = registry
                    .get_tools_by_category(&crate::planning::caws_tool_registry::CawsToolCategory::QualityGates)
                    .await;

                debug!(
                    "Found {} compliance tools and {} quality gate tools",
                    compliance_tools.len(),
                    quality_tools.len()
                );

                // Invoke compliance checking tools for validation
                for tool in &compliance_tools {
                    debug!("Invoking compliance tool: {} ({})", tool.name, tool.tool_id);
                    
                    // Prepare tool parameters from execution plan and artifacts
                    let mut parameters = std::collections::HashMap::new();
                    parameters.insert("working_spec_id".to_string(), serde_json::json!(working_spec.id));
                    parameters.insert("execution_plan_id".to_string(), serde_json::json!(execution_plan.id.to_string()));
                    parameters.insert("artifact_count".to_string(), serde_json::json!(artifacts.len()));
                    
                    // Invoke tool
                    match registry.invoke_tool(&tool.tool_id, parameters).await {
                        Ok(result) => {
                            if !result.success {
                                warn!("Compliance tool {} failed: {:?}", tool.name, result.error);
                                // Continue with other tools, but log the failure
                            } else if !result.caws_compliant {
                                warn!("Compliance tool {} reported non-compliance", tool.name);
                                // Continue but note the violation
                            } else {
                                debug!("Compliance tool {} passed validation", tool.name);
                            }
                            registry.increment_usage(&tool.tool_id).await;
                        }
                        Err(e) => {
                            warn!("Failed to invoke compliance tool {}: {}", tool.name, e);
                            // Continue with other tools even if one fails
                        }
                    }
                }

                // Invoke quality gate tools for validation
                for tool in &quality_tools {
                    debug!("Invoking quality gate tool: {} ({})", tool.name, tool.tool_id);
                    
                    // Prepare tool parameters
                    let mut parameters = std::collections::HashMap::new();
                    parameters.insert("working_spec_id".to_string(), serde_json::json!(working_spec.id));
                    parameters.insert("risk_tier".to_string(), serde_json::json!(working_spec.risk_tier));
                    parameters.insert("artifact_count".to_string(), serde_json::json!(artifacts.len()));
                    
                    // Invoke tool
                    match registry.invoke_tool(&tool.tool_id, parameters).await {
                        Ok(result) => {
                            if !result.success {
                                warn!("Quality gate tool {} failed: {:?}", tool.name, result.error);
                                // Continue with other tools
                            } else if !result.caws_compliant {
                                warn!("Quality gate tool {} reported violations", tool.name);
                                // Continue but note the violation
                            } else {
                                debug!("Quality gate tool {} passed validation", tool.name);
                            }
                            registry.increment_usage(&tool.tool_id).await;
                        }
                        Err(e) => {
                            warn!("Failed to invoke quality gate tool {}: {}", tool.name, e);
                            // Continue with other tools even if one fails
                        }
                    }
                }
            }
        }

        // Execute CAWS quality gates with waiver recognition
        let mut quality_gate_result: Option<crate::planning::caws_quality_gates::CawsQualityGateResult> = None;
        
        if let Some(ref executor) = self.quality_gates_executor {
            debug!("Executing CAWS quality gates with waiver recognition");
            
            match executor.execute_quality_gates("ci").await {
                Ok(gate_result) => {
                    quality_gate_result = Some(gate_result.clone());
                    info!(
                        "Quality gates executed: {} violations ({} waived, {} blocking)",
                        gate_result.total_violations,
                        gate_result.waived_violations,
                        gate_result.blocking_violations
                    );
                    
                    if gate_result.active_waivers > 0 {
                        info!("Active waivers: {}", gate_result.active_waivers);
                        for waiver in &gate_result.waivers {
                            debug!("  - {}: {} (expires: {})", waiver.id, waiver.title, waiver.expires_at);
                        }
                    }
                    
                    // If there are blocking violations, this will be considered in deliberation
                    if !gate_result.passed {
                        warn!(
                            "Quality gates failed: {} blocking violations found",
                            gate_result.blocking_violations
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to execute quality gates: {}", e);
                    // Continue examination even if quality gates fail to execute
                }
            }
        } else {
            // Try to initialize quality gates executor if not already set
            if let Ok(executor) = CawsQualityGateExecutor::new(".") {
                debug!("Initialized quality gates executor");
                match executor.execute_quality_gates("ci").await {
                    Ok(gate_result) => {
                        quality_gate_result = Some(gate_result.clone());
                        info!(
                            "Quality gates executed: {} violations ({} waived, {} blocking)",
                            gate_result.total_violations,
                            gate_result.waived_violations,
                            gate_result.blocking_violations
                        );
                    }
                    Err(e) => {
                        debug!("Quality gates execution failed: {}", e);
                    }
                }
            } else {
                debug!("Quality gates executor not available (script not found)");
            }
        }

        // Claim extraction is always-on (research feature is in default features)
        if let Some(ref claim_extractor) = self.claim_extractor {
            debug!("Running claim extraction on artifacts");
            
            for artifact in artifacts {
                // Extract text content from artifact for claim extraction
                let text_content = self.extract_text_from_artifact(artifact);
                
                if !text_content.is_empty() {
                    let processing_context = agent_research::ProcessingContext {
                        task_id: artifact.task_id,
                        working_spec_id: working_spec.id.clone(),
                        source_file: None,
                        line_number: None,
                        surrounding_context: String::new(),
                        domain_hints: vec![],
                        metadata: artifact.metadata.as_ref().map(|m| {
                            let mut map = std::collections::HashMap::new();
                            if let Some(compression) = m.compression_applied {
                                map.insert("compression_applied".to_string(), serde_json::json!(compression));
                            }
                            if let Some(location) = &m.storage_location {
                                map.insert("storage_location".to_string(), serde_json::json!(location));
                            }
                            if let Some(policy) = &m.retention_policy {
                                map.insert("retention_policy".to_string(), serde_json::json!(policy));
                            }
                            for tag in &m.tags {
                                map.insert(format!("tag:{}", tag), serde_json::json!(true));
                            }
                            map
                        }).unwrap_or_default(),
                        input_text: text_content.clone(),
                        language: None,
                    };

                    // Run claim extraction - need to use Arc::get_mut or create new instance
                    // Since we can't mutate through Arc, create a new processor for this extraction
                    let mut extractor = agent_research::ClaimExtractionProcessor::new();
                    match extractor.run(&text_content, &processing_context).await {
                        Ok(extraction_result) => {
                            total_claims += extraction_result.atomic_claims.len();
                            verified_claims += extraction_result.atomic_claims.iter()
                                .filter(|c| matches!(c.verification_status, agent_research::VerificationStatus::Verified))
                                .count();
                            evidence_count += extraction_result.verification_evidence.len();
                            
                            // Calculate average confidence
                            if !extraction_result.atomic_claims.is_empty() {
                                let avg_confidence = extraction_result.atomic_claims.iter()
                                    .map(|c| c.confidence)
                                    .sum::<f64>() / extraction_result.atomic_claims.len() as f64;
                                total_confidence += avg_confidence;
                            }
                        }
                        Err(e) => {
                            debug!("Claim extraction failed for artifact {}: {}", artifact.task_id, e);
                            // Continue with other artifacts even if one fails
                        }
                    }
                }
            }
        }

        let verification_confidence = if total_claims > 0 {
            total_confidence / total_claims as f64
        } else {
            0.0
        };

        Ok((
            ClaimExtractionResults {
                total_claims,
                verified_claims,
                verification_confidence,
                evidence_count,
            },
            quality_gate_result,
        ))
    }

    /// Extract text content from artifact for claim extraction
    fn extract_text_from_artifact(&self, artifact: &ExecutionArtifacts) -> String {
        let mut text_parts = Vec::new();

        // Extract from audit trail events
        for event in &artifact.provenance.audit_trail {
            text_parts.push(event.event.clone());
            if let Some(ref details) = event.details {
                if let Some(details_str) = details.as_str() {
                    text_parts.push(details_str.to_string());
                }
            }
        }

        // TODO: Implement metadata description extraction for artifact analysis
        //       Currently skips metadata extraction; should implement structured extraction from ArtifactMetadata, provenance.audit_trail, or other sources for comprehensive artifact text analysis.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Metadata description fields are extracted from ArtifactMetadata
        // - Provenance audit trail is parsed for additional context
        // - Text extraction handles structured and unstructured metadata
        // - Extraction is robust to missing or malformed metadata
        //
        // DEPENDENCIES:
        // - ArtifactMetadata schema enhancement (Optional)
        // - Provenance audit trail parsing utilities (Required)
        // - Text extraction and normalization utilities (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (artifact analysis enhancement)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Data extraction and parsing expertise

        // Extract from diff content (code changes)
        for diff in &artifact.code_changes.diffs {
            if !diff.diff_content.is_empty() {
                // Extract context lines from diff (non-code lines)
                for line in &diff.diff_content.lines().collect::<Vec<_>>() {
                    if line.starts_with("+++") || line.starts_with("---") || line.starts_with("@@") {
                        text_parts.push(line.to_string());
                    }
                }
            }
        }

        // Extract from new file content (first 1000 chars to avoid huge content)
        for new_file in &artifact.code_changes.new_files {
            let content_preview = new_file.content.chars().take(1000).collect::<String>();
            if !content_preview.is_empty() {
                text_parts.push(format!("New file {}: {}", new_file.path, content_preview));
            }
        }

        text_parts.join("\n")
    }

    /// Stage 3: Deliberation - Council debates and evaluates with claim verification and quality gates
    async fn stage_deliberation(
        &self,
        artifacts: &[ExecutionArtifacts],
        working_spec: &WorkingSpec,
        claim_results: &ClaimExtractionResults,
        quality_gate_result: Option<&crate::planning::caws_quality_gates::CawsQualityGateResult>,
    ) -> Result<()> {
        if artifacts.is_empty() {
            return Err(anyhow::anyhow!("No artifacts to deliberate"));
        }

        // If multiple artifacts present, use CAWS Debate methodology to score competing solutions
        if artifacts.len() > 1 {
            info!("Multiple artifacts present - conducting CAWS Debate");
            
            // Extract worker IDs from artifacts
            let solutions: Vec<(ExecutionArtifacts, Uuid)> = artifacts.iter()
                .map(|artifact| {
                    let worker_id = artifact.provenance.worker_id
                        .as_ref()
                        .and_then(|w| Uuid::parse_str(w).ok())
                        .unwrap_or_else(Uuid::new_v4);
                    (artifact.clone(), worker_id)
                })
                .collect();

            // Score debate and determine winner (claim verification and quality gate results passed to scorer)
            // Use the new method that accepts quality gate results
            let debate_result = if let Some(gate_result) = quality_gate_result {
                // Score with quality gates (waiver-aware)
                let mut solution_scores = Vec::new();
                for (artifact, worker_id) in &solutions {
                    let score = self.debate_scorer.score_solution_with_claims_and_gates(
                        artifact, *worker_id, working_spec, claim_results, Some(gate_result)
                    ).await?;
                    solution_scores.push(score);
                }
                
                // Find winner
                let winner = solution_scores.iter()
                    .max_by(|a, b| a.total_score.partial_cmp(&b.total_score).unwrap_or(std::cmp::Ordering::Equal))
                    .ok_or_else(|| anyhow::anyhow!("Failed to determine debate winner"))?;
                
                // Calculate confidence
                let mut scores: Vec<f64> = solution_scores.iter().map(|s| s.total_score).collect();
                scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
                
                let confidence = if scores.len() >= 2 {
                    let gap = scores[0] - scores[1];
                    (gap * 2.0).min(1.0).max(0.5)
                } else {
                    0.8
                };
                
                crate::planning::caws_debate_scorer::DebateScoringResult {
                    winner_solution_id: winner.solution_id.clone(),
                    winner_worker_id: winner.worker_id,
                    winning_score: winner.total_score,
                    confidence,
                    solution_scores: solution_scores.clone(),
                    judge_notes: self.debate_scorer.generate_judge_notes_with_claims(&solution_scores, winner, claim_results),
                }
            } else {
                // Fallback to scoring without quality gates
                self.debate_scorer.score_debate_with_claims(
                    solutions, 
                    working_spec,
                    claim_results,
                ).await?
            };
            
            info!(
                "CAWS Debate concluded: Winner is solution {} from worker {} with score {:.3}",
                debate_result.winner_solution_id,
                debate_result.winner_worker_id,
                debate_result.winning_score
            );
            
            // Log judge notes for audit trail
            info!("Debate judge notes: {}", debate_result.judge_notes);
        } else {
            // Single artifact - score it for consistency
            let worker_id = artifacts[0].provenance.worker_id
                .as_ref()
                .and_then(|w| Uuid::parse_str(w).ok())
                .unwrap_or_else(Uuid::new_v4);
            
            // Score with quality gates if available
            let score = if let Some(gate_result) = quality_gate_result {
                self.debate_scorer.score_solution_with_claims_and_gates(
                    &artifacts[0],
                    worker_id,
                    working_spec,
                    claim_results,
                    Some(gate_result),
                ).await?
            } else {
                self.debate_scorer.score_solution_with_claims(
                    &artifacts[0],
                    worker_id,
                    working_spec,
                    claim_results,
                ).await?
            };
            
            info!(
                "Single artifact scored: Worker {} with total score {:.3}",
                score.worker_id,
                score.total_score
            );
        }

        Ok(())
    }

    /// Stage 4: Verdict - Council reaches decision with claim verification summary
    async fn stage_verdict(
        &self,
        artifacts: &[ExecutionArtifacts],
        working_spec: &WorkingSpec,
        claim_results: &ClaimExtractionResults,
    ) -> Result<WorkPresentationResult> {
        // Get verdict from council for primary artifact
        if let Some(primary_artifact) = artifacts.first() {
            let mut verdict = self.council_integration.get_verdict(
                primary_artifact,
                working_spec,
            ).await?;

            // Update verification summary with claim extraction results
            verdict.verification_summary = agent_agency_contracts::final_verdict::VerificationSummary {
                claims_total: claim_results.total_claims as u32,
                claims_verified: claim_results.verified_claims as u32,
                coverage_pct: if claim_results.total_claims > 0 {
                    (claim_results.verified_claims as f32 / claim_results.total_claims as f32) * 100.0
                } else {
                    0.0f32
                },
            };

            Ok(WorkPresentationResult {
                approved: matches!(verdict.decision, 
                    agent_agency_contracts::final_verdict::FinalDecision::Accept),
                needs_refinement: !verdict.remediation.is_empty(),
                refinement_reason: if verdict.remediation.is_empty() {
                    String::new()
                } else {
                    verdict.remediation.join(", ")
                },
                verdict: Some(verdict),
            })
        } else {
            Err(anyhow::anyhow!("No artifacts for verdict"))
        }
    }

    /// Stage 5: Publication - Publish verdict and merge work
    async fn stage_publication(
        &self,
        verdict_result: &WorkPresentationResult,
        artifacts: &[ExecutionArtifacts],
    ) -> Result<()> {
        use tracing::{warn, error};
        use std::process::Command;
        
        if verdict_result.approved {
            info!("Verdict approved - merging work");
            
            // Merge worktrees if worktree manager is available
            if let Some(ref worktree_manager) = self.worktree_manager {
                // Get all active worktrees
                let active_worktrees = worktree_manager.list_worktrees().await;
                
                // Match artifacts to worktrees by worker_id
                for artifact in artifacts {
                    let worker_id_str = artifact.provenance.worker_id.as_ref();
                    if let Some(worker_id_str) = worker_id_str {
                        if let Ok(worker_id) = Uuid::parse_str(worker_id_str) {
                            // Find worktree for this worker
                            if let Some(worktree_info) = active_worktrees.iter()
                                .find(|wt| wt.worker_id == worker_id) {
                                
                                info!("Merging worktree {} for worker {}", worktree_info.worktree_id, worker_id);
                                
                                // Merge worktree
                                match worktree_manager.merge_worktree(worktree_info.worktree_id).await {
                                    Ok(merge_result) => {
                                        if !merge_result.conflicts.is_empty() {
                                            warn!("Merge conflicts detected in {} files: {:?}", 
                                                merge_result.conflicts.len(), merge_result.conflicts);
                                            
                                            // TODO: Implement council resolution request for merge conflicts
                                            //       Currently logs conflicts and continues; should implement automatic council resolution request workflow for merge conflict handling.
                                            //
                                            // COMPLETION CHECKLIST:
                                            // [ ] Primary functionality implemented
                                            // [ ] API/data structures defined & stable
                                            // [ ] Error handling + validation aligned with error taxonomy
                                            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                                            // [ ] Integration tests for external systems/contracts
                                            // [ ] Documentation: public API + system behavior
                                            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                                            // [ ] Security posture reviewed (inputs, authz, sandboxing)
                                            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                                            // [ ] Configurability and feature flags defined if relevant
                                            // [ ] Failure-mode cards documented (degradation paths)
                                            //
                                            // ACCEPTANCE CRITERIA:
                                            // - Merge conflicts trigger council resolution request
                                            // - Conflict details are properly formatted for council review
                                            // - Council can resolve conflicts through standard workflow
                                            // - Resolution is applied back to worktree merge process
                                            //
                                            // DEPENDENCIES:
                                            // - Council resolution API (Required)
                                            // - Conflict formatting and presentation utilities (Required)
                                            // - Worktree merge retry mechanism (Required)
                                            //
                                            // ESTIMATED EFFORT: 10-14 hours (medium confidence)
                                            // PRIORITY: Medium
                                            // BLOCKING: No
                                            //
                                            // GOVERNANCE:
                                            // - CAWS Tier: 2 (workflow automation enhancement)
                                            // - Change Budget: ~200 LOC
                                            // - Reviewer Requirements: Council integration and workflow expertise
                                            info!("Merge conflicts will require manual resolution");
                                        } else {
                                            info!("Successfully merged worktree {} ({} files changed)", 
                                                worktree_info.worktree_id, merge_result.files_changed);
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to merge worktree {}: {}", worktree_info.worktree_id, e);
                                        // Continue with other worktrees even if one fails
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Commit verdict to git with CAWS-VERDICT-ID trailer
            if let Some(ref verdict) = verdict_result.verdict {
                let verdict_id = Uuid::new_v4();
                let commit_message = format!(
                    "CAWS Verdict: {:?}\n\nCAWS-VERDICT-ID: {}\n\nDecision: {:?}\nVotes: {}\nDissent: {}",
                    verdict.decision,
                    verdict_id,
                    verdict.decision,
                    verdict.votes.len(),
                    verdict.dissent
                );
                
                // Commit verdict as a git note or annotation
                // Using git notes for verdict storage (non-intrusive)
                let note_output = Command::new("git")
                    .arg("notes")
                    .arg("add")
                    .arg("-m")
                    .arg(&commit_message)
                    .arg("HEAD")
                    .output();
                
                match note_output {
                    Ok(output) if output.status.success() => {
                        info!("Committed CAWS verdict {} to git notes", verdict_id);
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        warn!("Failed to commit verdict to git notes: {}", stderr);
                    }
                    Err(e) => {
                        warn!("Failed to execute git notes command: {}", e);
                    }
                }
            }
        } else {
            info!("Verdict rejected - work will not be merged");
            
            // Cleanup worktrees if verdict is rejected
            if let Some(ref worktree_manager) = self.worktree_manager {
                let active_worktrees = worktree_manager.list_worktrees().await;
                
                for artifact in artifacts {
                    let worker_id_str = artifact.provenance.worker_id.as_ref();
                    if let Some(worker_id_str) = worker_id_str {
                        if let Ok(worker_id) = Uuid::parse_str(worker_id_str) {
                            if let Some(worktree_info) = active_worktrees.iter()
                                .find(|wt| wt.worker_id == worker_id) {
                                
                                info!("Cleaning up rejected worktree {} for worker {}", 
                                    worktree_info.worktree_id, worker_id);
                                
                                if let Err(e) = worktree_manager.cleanup_worktree(worktree_info.worktree_id).await {
                                    warn!("Failed to cleanup rejected worktree {}: {}", worktree_info.worktree_id, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// Re-export WorkPresentationResult from council_integration
use crate::planning::council_integration::WorkPresentationResult;

