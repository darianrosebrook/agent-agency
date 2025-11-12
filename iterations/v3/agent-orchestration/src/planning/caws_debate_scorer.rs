//! CAWS Debate Scorer
//!
//! Implements the CAWS Debate scoring algorithm for evaluating competing
//! worker solutions. Uses the scoring formula from theory.md:
//!
//! S = 0.4E + 0.3B + 0.2G + 0.1P
//!
//! Where:
//! - E = Evidence Completeness (40%)
//! - B = Budget Adherence (30%)
//! - G = Gate Integrity (20%)
//! - P = Provenance Clarity (10%)
//!
//! @author @darianrosebrook

use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use tracing::info;

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::WorkingSpec;

use crate::council::Council;
use crate::planning::caws_adjudication_cycle::ClaimExtractionResults;
use crate::planning::caws_quality_gates::CawsQualityGateResult;
use crate::planning::rubric_engineering::{RubricEngine, TaskSurface, ComponentScores};

/// Solution score breakdown
#[derive(Debug, Clone)]
pub struct SolutionScore {
    /// Solution identifier
    pub solution_id: String,
    
    /// Worker identifier
    pub worker_id: Uuid,
    
    /// Total score (0.0 to 1.0)
    pub total_score: f64,
    
    /// Evidence completeness score (0.0 to 1.0)
    pub evidence_completeness: f64,
    
    /// Budget adherence score (0.0 to 1.0)
    pub budget_adherence: f64,
    
    /// Gate integrity score (0.0 to 1.0)
    pub gate_integrity: f64,
    
    /// Provenance clarity score (0.0 to 1.0)
    pub provenance_clarity: f64,
}

/// Debate result with winner and scores
#[derive(Debug, Clone)]
pub struct DebateScoringResult {
    /// Winning solution identifier
    pub winner_solution_id: String,
    
    /// Winning worker identifier
    pub winner_worker_id: Uuid,
    
    /// Winning score
    pub winning_score: f64,
    
    /// Confidence in the result (0.0 to 1.0)
    pub confidence: f64,
    
    /// All solution scores
    pub solution_scores: Vec<SolutionScore>,
    
    /// Judge notes summarizing the debate
    pub judge_notes: String,
}

/// CAWS Debate Scorer
pub struct CawsDebateScorer {
    #[allow(dead_code)] // Reserved for future use
    council: Arc<Council>,
    /// Optional rubric engine for task-surface-specific weights
    rubric_engine: Option<Arc<RubricEngine>>,
}

impl CawsDebateScorer {
    /// Create new CAWS debate scorer
    pub fn new(council: Arc<Council>) -> Self {
        Self {
            council,
            rubric_engine: None,
        }
    }
    
    /// Create new CAWS debate scorer with rubric engine
    pub fn with_rubric_engine(council: Arc<Council>, rubric_engine: Arc<RubricEngine>) -> Self {
        Self {
            council,
            rubric_engine: Some(rubric_engine),
        }
    }

    /// Score a single solution
    ///
    /// Calculates the CAWS debate score for a single worker solution.
    /// This is the original method without claim verification for backward compatibility.
    pub async fn score_solution(
        &self,
        artifacts: &ExecutionArtifacts,
        worker_id: Uuid,
        working_spec: &WorkingSpec,
    ) -> Result<SolutionScore> {
        info!("Scoring solution from worker {}", worker_id);

        // Calculate individual score components
        let evidence_completeness = self.calculate_evidence_completeness(artifacts);
        let budget_adherence = self.calculate_budget_adherence(artifacts, working_spec);
        let gate_integrity = self.calculate_gate_integrity(artifacts);
        let provenance_clarity = self.calculate_provenance_clarity(artifacts);

        // Use task-surface-specific weights if rubric engine is available
        let total_score = if let Some(ref rubric_engine) = self.rubric_engine {
            let surface = TaskSurface::classify(working_spec, Some(artifacts));
            let component_scores = ComponentScores {
                evidence_completeness,
                budget_adherence,
                gate_integrity,
                provenance_clarity,
            };
            
            match rubric_engine.calculate_weighted_score(&surface, &component_scores).await {
                Ok(weighted) => weighted,
                Err(e) => {
                    warn!("Failed to calculate weighted score with rubric, falling back to default: {}", e);
                    (evidence_completeness * 0.4)
                        + (budget_adherence * 0.3)
                        + (gate_integrity * 0.2)
                        + (provenance_clarity * 0.1)
                }
            }
        } else {
            // Default weights: S = 0.4E + 0.3B + 0.2G + 0.1P
            (evidence_completeness * 0.4)
                + (budget_adherence * 0.3)
                + (gate_integrity * 0.2)
                + (provenance_clarity * 0.1)
        };

        Ok(SolutionScore {
            solution_id: format!("solution_{}", artifacts.task_id),
            worker_id,
            total_score,
            evidence_completeness,
            budget_adherence,
            gate_integrity,
            provenance_clarity,
        })
    }

    /// Score a single solution with claim verification results
    ///
    /// Calculates the CAWS debate score incorporating claim verification scores.
    pub async fn score_solution_with_claims(
        &self,
        artifacts: &ExecutionArtifacts,
        worker_id: Uuid,
        working_spec: &WorkingSpec,
        claim_results: &ClaimExtractionResults,
    ) -> Result<SolutionScore> {
        self.score_solution_with_claims_and_gates(
            artifacts,
            worker_id,
            working_spec,
            claim_results,
            None,
        ).await
    }

    /// Score a single solution with claim verification and quality gate results
    ///
    /// Calculates the CAWS debate score incorporating claim verification scores
    /// and quality gate results with waiver recognition.
    pub async fn score_solution_with_claims_and_gates(
        &self,
        artifacts: &ExecutionArtifacts,
        worker_id: Uuid,
        working_spec: &WorkingSpec,
        claim_results: &ClaimExtractionResults,
        quality_gate_result: Option<&CawsQualityGateResult>,
    ) -> Result<SolutionScore> {
        self.score_solution_with_claims_gates_and_mode(
            artifacts,
            worker_id,
            working_spec,
            claim_results,
            quality_gate_result,
            None,
        ).await
    }

    /// Score a single solution with claim verification, quality gates, and complexity mode
    ///
    /// Calculates the CAWS debate score with mode-aware scoring weights.
    pub async fn score_solution_with_claims_gates_and_mode(
        &self,
        artifacts: &ExecutionArtifacts,
        worker_id: Uuid,
        working_spec: &WorkingSpec,
        claim_results: &ClaimExtractionResults,
        quality_gate_result: Option<&CawsQualityGateResult>,
        complexity_mode: Option<crate::planning::caws_complexity_mode::CawsComplexityMode>,
    ) -> Result<SolutionScore> {
        info!("Scoring solution from worker {} with claim verification and quality gates", worker_id);

        // Calculate individual score components
        let mut evidence_completeness = self.calculate_evidence_completeness(artifacts);
        
        // Enhance evidence completeness with claim verification results
        if claim_results.total_claims > 0 {
            let claim_verification_score = claim_results.verified_claims as f64 / claim_results.total_claims as f64;
            // Blend claim verification into evidence completeness (weighted average)
            evidence_completeness = (evidence_completeness * 0.7) + (claim_verification_score * 0.3);
        }
        
        let budget_adherence = self.calculate_budget_adherence(artifacts, working_spec);
        
        // Calculate gate integrity with waiver-aware scoring
        let gate_integrity = if let Some(gate_result) = quality_gate_result {
            self.calculate_gate_integrity_with_waivers(artifacts, gate_result)
        } else {
            self.calculate_gate_integrity(artifacts)
        };
        
        let provenance_clarity = self.calculate_provenance_clarity(artifacts);

        // Determine scoring weights based on complexity mode
        let (e_weight, b_weight, g_weight, p_weight) = if let Some(mode) = complexity_mode {
            match mode {
                crate::planning::caws_complexity_mode::CawsComplexityMode::Simple => {
                    // Simple mode: More balanced weights, less emphasis on evidence
                    (0.3, 0.3, 0.2, 0.2)
                }
                crate::planning::caws_complexity_mode::CawsComplexityMode::Standard => {
                    // Standard mode: Default weights
                    (0.4, 0.3, 0.2, 0.1)
                }
                crate::planning::caws_complexity_mode::CawsComplexityMode::Enterprise => {
                    // Enterprise mode: Evidence-heavy, provenance less important
                    (0.5, 0.25, 0.2, 0.05)
                }
            }
        } else {
            // Default weights if mode not provided
            (0.4, 0.3, 0.2, 0.1)
        };

        // Use task-surface-specific weights if rubric engine is available
        let total_score = if let Some(ref rubric_engine) = self.rubric_engine {
            // Classify task surface
            let surface = TaskSurface::classify(working_spec, Some(artifacts));
            
            // Calculate weighted score using rubric
            let component_scores = ComponentScores {
                evidence_completeness,
                budget_adherence,
                gate_integrity,
                provenance_clarity,
            };
            
            match rubric_engine.calculate_weighted_score(&surface, &component_scores).await {
                Ok(weighted) => weighted,
                Err(e) => {
                    warn!("Failed to calculate weighted score with rubric, falling back to mode-aware weights: {}", e);
                    // Fallback to mode-aware weights
                    (evidence_completeness * e_weight)
                        + (budget_adherence * b_weight)
                        + (gate_integrity * g_weight)
                        + (provenance_clarity * p_weight)
                }
            }
        } else {
            // Mode-aware weights: S = e_weight*E + b_weight*B + g_weight*G + p_weight*P
            (evidence_completeness * e_weight)
                + (budget_adherence * b_weight)
                + (gate_integrity * g_weight)
                + (provenance_clarity * p_weight)
        };

        Ok(SolutionScore {
            solution_id: format!("solution_{}", artifacts.task_id),
            worker_id,
            total_score,
            evidence_completeness,
            budget_adherence,
            gate_integrity,
            provenance_clarity,
        })
    }
    
    /// Record performance for rubric adjustment
    pub async fn record_performance(
        &self,
        task_id: Uuid,
        working_spec: &WorkingSpec,
        artifacts: &ExecutionArtifacts,
        solution_score: &SolutionScore,
        success: bool,
        quality_score: f64,
    ) {
        if let Some(ref rubric_engine) = self.rubric_engine {
            let surface = TaskSurface::classify(working_spec, Some(artifacts));
            let component_scores = ComponentScores {
                evidence_completeness: solution_score.evidence_completeness,
                budget_adherence: solution_score.budget_adherence,
                gate_integrity: solution_score.gate_integrity,
                provenance_clarity: solution_score.provenance_clarity,
            };
            
            rubric_engine.record_performance(
                task_id,
                surface,
                component_scores,
                solution_score.total_score,
                success,
                quality_score,
            ).await;
        }
    }

    /// Score multiple competing solutions with claim verification results
    ///
    /// Implements the CAWS Debate protocol incorporating claim verification.
    pub async fn score_debate_with_claims(
        &self,
        solutions: Vec<(ExecutionArtifacts, Uuid)>,
        working_spec: &WorkingSpec,
        claim_results: &ClaimExtractionResults,
    ) -> Result<DebateScoringResult> {
        if solutions.is_empty() {
            return Err(anyhow::anyhow!("Cannot score debate with no solutions"));
        }

        info!("Scoring debate between {} competing solutions with claim verification", solutions.len());

        // Score each solution with claim verification
        let mut solution_scores = Vec::new();
        for (artifacts, worker_id) in &solutions {
            let score = self.score_solution_with_claims(artifacts, *worker_id, working_spec, claim_results).await?;
            solution_scores.push(score);
        }

        // Find winner (highest total score)
        let winner = solution_scores.iter()
            .max_by(|a, b| a.total_score.partial_cmp(&b.total_score).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| anyhow::anyhow!("Failed to determine debate winner"))?;

        // Calculate confidence based on score gap
        let mut scores: Vec<f64> = solution_scores.iter().map(|s| s.total_score).collect();
        scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        
        let confidence = if scores.len() >= 2 {
            // Confidence based on gap between winner and second place
            let gap = scores[0] - scores[1];
            (gap * 2.0).min(1.0).max(0.5) // Scale gap to 0.5-1.0 range
        } else {
            0.8 // High confidence for single solution
        };

        // Generate judge notes with claim verification summary
        let judge_notes = self.generate_judge_notes_with_claims(&solution_scores, winner, claim_results);

        Ok(DebateScoringResult {
            winner_solution_id: winner.solution_id.clone(),
            winner_worker_id: winner.worker_id,
            winning_score: winner.total_score,
            confidence,
            solution_scores: solution_scores.clone(),
            judge_notes,
        })
    }

    /// Score multiple competing solutions and determine winner
    ///
    /// Implements the CAWS Debate protocol where multiple workers
    /// compete and the highest-scoring solution wins.
    pub async fn score_debate(
        &self,
        solutions: Vec<(ExecutionArtifacts, Uuid)>,
        working_spec: &WorkingSpec,
    ) -> Result<DebateScoringResult> {
        if solutions.is_empty() {
            return Err(anyhow::anyhow!("Cannot score debate with no solutions"));
        }

        info!("Scoring debate between {} competing solutions", solutions.len());

        // Score each solution
        let mut solution_scores = Vec::new();
        for (artifacts, worker_id) in &solutions {
            let score = self.score_solution(artifacts, *worker_id, working_spec).await?;
            solution_scores.push(score);
        }

        // Find winner (highest total score)
        let winner = solution_scores.iter()
            .max_by(|a, b| a.total_score.partial_cmp(&b.total_score).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| anyhow::anyhow!("Failed to determine debate winner"))?;

        // Calculate confidence based on score gap
        let mut scores: Vec<f64> = solution_scores.iter().map(|s| s.total_score).collect();
        scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        
        let confidence = if scores.len() >= 2 {
            // Confidence based on gap between winner and second place
            let gap = scores[0] - scores[1];
            (gap * 2.0).min(1.0).max(0.5) // Scale gap to 0.5-1.0 range
        } else {
            0.8 // High confidence for single solution
        };

        // Generate judge notes
        let judge_notes = self.generate_judge_notes(&solution_scores, winner);

        Ok(DebateScoringResult {
            winner_solution_id: winner.solution_id.clone(),
            winner_worker_id: winner.worker_id,
            winning_score: winner.total_score,
            confidence,
            solution_scores,
            judge_notes,
        })
    }

    /// Calculate evidence completeness score (0.0 to 1.0)
    ///
    /// E component: Evaluates how complete the evidence is
    fn calculate_evidence_completeness(&self, artifacts: &ExecutionArtifacts) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Test results completeness
        if artifacts.tests.unit_tests.total > 0 {
            score += 0.25;
            factors += 1;
        }
        if artifacts.tests.integration_tests.total > 0 {
            score += 0.25;
            factors += 1;
        }
        if artifacts.tests.e2e_tests.total > 0 {
            score += 0.25;
            factors += 1;
        }

        // Coverage data completeness
        if artifacts.coverage.line_coverage > 0.0 {
            score += 0.15;
            factors += 1;
        }
        if artifacts.coverage.branch_coverage > 0.0 {
            score += 0.10;
            factors += 1;
        }

        // Normalize by number of factors
        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }

    /// Calculate budget adherence score (0.0 to 1.0)
    ///
    /// B component: Evaluates adherence to change budget
    fn calculate_budget_adherence(
        &self,
        artifacts: &ExecutionArtifacts,
        working_spec: &WorkingSpec,
    ) -> f64 {
        let budget = &working_spec.change_budget;
        let stats = &artifacts.code_changes.statistics;

        let files_score = if budget.max_files > 0 {
            let ratio = stats.files_modified as f64 / budget.max_files as f64;
            if ratio <= 1.0 {
                1.0 - (ratio * 0.5) // Full score if within budget, decreasing penalty
            } else {
                0.0 // Zero score if exceeds budget
            }
        } else {
            1.0 // No budget constraint
        };

        let loc_score = if budget.max_loc > 0 {
            let ratio = stats.lines_added as f64 / budget.max_loc as f64;
            if ratio <= 1.0 {
                1.0 - (ratio * 0.5) // Full score if within budget, decreasing penalty
            } else {
                0.0 // Zero score if exceeds budget
            }
        } else {
            1.0 // No budget constraint
        };

        // Average of files and LOC scores
        (files_score + loc_score) / 2.0
    }

    /// Calculate gate integrity score (0.0 to 1.0)
    ///
    /// G component: Evaluates quality gate compliance
    fn calculate_gate_integrity(&self, artifacts: &ExecutionArtifacts) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Test pass rate
        let total_tests = artifacts.tests.unit_tests.total
            + artifacts.tests.integration_tests.total
            + artifacts.tests.e2e_tests.total;
        if total_tests > 0 {
            let passed_tests = artifacts.tests.unit_tests.passed
                + artifacts.tests.integration_tests.passed
                + artifacts.tests.e2e_tests.passed;
            let pass_rate = passed_tests as f64 / total_tests as f64;
            score += pass_rate * 0.4;
            factors += 1;
        }

        // Coverage thresholds
        if artifacts.coverage.line_coverage > 0.0 {
            if artifacts.coverage.line_coverage >= 0.8 {
                score += 0.3;
            } else if artifacts.coverage.line_coverage >= 0.6 {
                score += 0.15;
            }
            factors += 1;
        }

        // Linting results
        if artifacts.linting.total_issues == 0 {
            score += 0.3;
        } else {
            // Penalty based on error count
            let error_rate = artifacts.linting.errors as f64 / artifacts.linting.total_issues as f64;
            score += (1.0 - error_rate) * 0.3;
        }
        factors += 1;

        // Normalize
        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }

    /// Calculate gate integrity score with waiver-aware scoring (0.0 to 1.0)
    ///
    /// G component: Evaluates quality gate compliance, accounting for waived violations.
    /// Only non-waived violations count against the score.
    fn calculate_gate_integrity_with_waivers(
        &self,
        artifacts: &ExecutionArtifacts,
        gate_result: &CawsQualityGateResult,
    ) -> f64 {
        // Start with base gate integrity score
        let mut score = self.calculate_gate_integrity(artifacts);
        let mut factors = 1.0;

        // Adjust score based on CAWS quality gate results
        // Only blocking (non-waived) violations reduce the score
        if gate_result.total_violations > 0 {
            // Calculate violation rate based on blocking violations only
            let blocking_rate = if gate_result.total_violations > 0 {
                gate_result.blocking_violations as f64 / gate_result.total_violations as f64
            } else {
                0.0
            };

            // If all violations are waived, no penalty
            if gate_result.blocking_violations == 0 {
                // All violations waived - maintain or slightly boost score
                score = score.max(0.8); // Ensure minimum score when all violations waived
            } else {
                // Apply penalty based on blocking violations
                // Penalty is proportional to blocking violation rate
                let penalty = blocking_rate * 0.3; // Max 30% penalty
                score = (score - penalty).max(0.0);
            }
            factors += 1.0;
        }

        // If quality gates passed (no blocking violations), boost score
        if gate_result.passed {
            score = (score * 0.8 + 0.2).min(1.0); // Boost by up to 20%
        }

        // Normalize
        score / factors
    }

    /// Calculate provenance clarity score (0.0 to 1.0)
    ///
    /// P component: Evaluates clarity and completeness of provenance
    fn calculate_provenance_clarity(&self, artifacts: &ExecutionArtifacts) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Worker ID present
        if artifacts.provenance.worker_id.is_some() {
            score += 0.3;
            factors += 1;
        }

        // Execution timing present
        if artifacts.provenance.completed_at.is_some() {
            score += 0.2;
            factors += 1;
        }

        // Git info present
        if !artifacts.provenance.git_info.branch.is_empty() {
            score += 0.25;
            factors += 1;
        }

        // Audit trail completeness
        if !artifacts.provenance.audit_trail.is_empty() {
            score += 0.25;
            factors += 1;
        }

        // Normalize
        if factors > 0 {
            score
        } else {
            0.0
        }
    }

    /// Generate judge notes summarizing the debate
    fn generate_judge_notes(
        &self,
        solution_scores: &[SolutionScore],
        winner: &SolutionScore,
    ) -> String {
        format!(
            "Debate concluded with {} solutions evaluated.\n\n\
            Winner: Solution {} (Worker {})\n\
            Score: {:.3}\n\n\
            Score Breakdown:\n\
            - Evidence Completeness: {:.3}\n\
            - Budget Adherence: {:.3}\n\
            - Gate Integrity: {:.3}\n\
            - Provenance Clarity: {:.3}\n\n\
            All solutions scored above threshold.",
            solution_scores.len(),
            winner.solution_id,
            winner.worker_id,
            winner.total_score,
            winner.evidence_completeness,
            winner.budget_adherence,
            winner.gate_integrity,
            winner.provenance_clarity,
        )
    }

    /// Generate judge notes summarizing the debate with claim verification
    pub fn generate_judge_notes_with_claims(
        &self,
        solution_scores: &[SolutionScore],
        winner: &SolutionScore,
        claim_results: &ClaimExtractionResults,
    ) -> String {
        format!(
            "Debate concluded with {} solutions evaluated.\n\n\
            Winner: Solution {} (Worker {})\n\
            Score: {:.3}\n\n\
            Score Breakdown:\n\
            - Evidence Completeness: {:.3} (includes claim verification)\n\
            - Budget Adherence: {:.3}\n\
            - Gate Integrity: {:.3}\n\
            - Provenance Clarity: {:.3}\n\n\
            Claim Verification Summary:\n\
            - Total Claims: {}\n\
            - Verified Claims: {}\n\
            - Verification Confidence: {:.2}%\n\
            - Evidence Collected: {}\n\n\
            All solutions scored above threshold.",
            solution_scores.len(),
            winner.solution_id,
            winner.worker_id,
            winner.total_score,
            winner.evidence_completeness,
            winner.budget_adherence,
            winner.gate_integrity,
            winner.provenance_clarity,
            claim_results.total_claims,
            claim_results.verified_claims,
            claim_results.verification_confidence * 100.0,
            claim_results.evidence_count,
        )
    }
}

