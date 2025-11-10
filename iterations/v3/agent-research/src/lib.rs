#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Claim Extraction & Verification Pipeline

// Import contract types
use agent_agency_contracts as contracts;

pub mod decomposition;
pub mod disambiguation;
pub mod evidence;
pub mod processor;
pub mod qualification;
pub mod extraction_types;
pub mod verification;
pub mod benchmarking;
pub mod benchmark_types;
pub mod benchmark_runner;
pub mod performance_tracker;
pub mod scoring_system;
pub mod sla_validator;

// Multimodal retriever modules
pub mod multimodal_retriever;

// Content processing modules
pub mod content_processor;
pub mod context_builder;
pub mod vector_search;
pub mod web_scraper;
pub mod multimodal_context_provider;

// Planning agent modules (consolidated from planning-agent crate)
pub mod planning_agent;

// Self-prompting agent modules (consolidated from self-prompting-agent crate)
pub mod self_prompting_agent;

// Research types module
pub mod research_types;

// Knowledge seeker module (directory-based)
pub mod knowledge_seeker;
pub use knowledge_seeker::KnowledgeSeeker;

// Learning service module
pub mod learning_service;

// Learning algorithms module
pub mod learning_algorithms;

// Orchestrator module (consolidated from learning_algorithms/orchestrator.rs)
pub mod orchestrator;

// Reinforcement learning module
pub mod reinforcement;

// Reflexive types module
pub mod reflexive_types;



// Re-export contract types for internal use
pub use contracts::{
    // Task execution types
    TaskSpec, TaskExecutionResult, TaskRequirements, TaskContext, TaskScope, ExecutionStatus, Progress,

    // Worker types
    WorkerResult, WorkerHealthStatus,

    // Planning types
    WorkingSpec, AcceptanceCriterion, TestPlan, RollbackPlan,
};

// Re-export internal types
// pub use verification::MultiModalVerificationEngine; // Temporarily disabled due to verification module issues
pub use processor::ClaimExtractionProcessor;
pub use extraction_types::*;
pub use orchestrator::LearningOrchestrator;
pub use agent_agency_contracts::types::research::VerificationMethod;
pub use research_types::ContentProcessingConfig;
pub use content_processor::ContentProcessor;
pub use context_builder::ContextBuilder;
pub use vector_search::VectorSearchEngine;
pub use web_scraper::WebScraper;
pub use multimodal_context_provider::MultimodalContext;

use anyhow::Result;
use std::time::Instant;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Main claim extraction and verification processor
///
/// Integrates with council debate protocol to provide evidence
/// for claim verification during judicial evaluation.

# [derive(Debug)]
pub struct ClaimExtractionAndVerificationProcessor {
    disambiguation_stage: disambiguation::DisambiguationStage,
    qualification_stage: qualification::QualificationStage,
    decomposition_stage: decomposition::DecompositionStage,
    verification_stage: Option<verification::MultiModalVerificationEngine>, // Temporarily optional
}

impl ClaimExtractionAndVerificationProcessor {
    /// Create a new claim extraction processor
    pub fn new() -> Self {
        Self {
            disambiguation_stage: disambiguation::minimal_stage(),
            qualification_stage: qualification::QualificationStage::new(),
            decomposition_stage: decomposition::DecompositionStage::new(),
            verification_stage: Some(verification::MultiModalVerificationEngine::new()), // Re-enabled with CAWS-compliant validation
        }
    }

    /// Process a sentence through the complete 4-stage pipeline
    pub async fn process_sentence(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<ClaimExtractionResult, ClaimExtractionError> {
        let start_time = Instant::now();
        info!("Starting claim extraction for sentence: {}", sentence);

        let mut stages_completed = Vec::new();
        let mut errors = Vec::new();
        let mut disambiguated_sentence = sentence.to_string();
        let mut atomic_claims = Vec::new();
        let mut verification_evidence = Vec::new();
        let mut ambiguities_resolved = 0u32;
        let mut rewrite_suggestions = 0u32;
        let mut unverifiable_breakdown = UnverifiableBreakdown::default();

        // Stage 1: Disambiguation
        match self.disambiguation_stage.process(sentence, context).await {
            Ok(disambiguation_result) => {
                disambiguated_sentence = disambiguation_result.disambiguated_sentence;
                ambiguities_resolved = disambiguation_result.ambiguities_resolved;
                stages_completed.push(ProcessingStage::Disambiguation);
                info!(
                    "Disambiguation completed: {} ambiguities resolved",
                    disambiguation_result.ambiguities_resolved
                );
            }
            Err(e) => {
                let error = ProcessingError {
                    stage: ProcessingStage::Disambiguation,
                    error_type: "DisambiguationFailed".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                };
                errors.push(error);
                warn!(
                    "Disambiguation failed, continuing with original sentence: {}",
                    e
                );
            }
        }

        // Stage 2: Qualification
        match self
            .qualification_stage
            .process(&disambiguated_sentence, context)
            .await
        {
            Ok(qualification_result) => {
                rewrite_suggestions = qualification_result
                    .unverifiable_parts
                    .iter()
                    .filter(|part| part.suggested_rewrite.is_some())
                    .count() as u32;
                for fragment in &qualification_result.unverifiable_parts {
                    match fragment.reason {
                        UnverifiableReason::SubjectiveLanguage => {
                            unverifiable_breakdown.subjective_language += 1
                        }
                        UnverifiableReason::VagueCriteria => {
                            unverifiable_breakdown.vague_criteria += 1
                        }
                        UnverifiableReason::MissingContext => {
                            unverifiable_breakdown.missing_context += 1
                        }
                        UnverifiableReason::OpinionBased => {
                            unverifiable_breakdown.opinion_based += 1
                        }
                        UnverifiableReason::FuturePrediction => {
                            unverifiable_breakdown.future_prediction += 1
                        }
                        UnverifiableReason::EmotionalContent => {
                            unverifiable_breakdown.emotional_content += 1
                        }
                        UnverifiableReason::ImprovementClaim => {
                            unverifiable_breakdown.improvement_claim += 1
                        }
                    }
                }
                stages_completed.push(ProcessingStage::Qualification);
                info!(
                    "Qualification completed: {} verifiable parts found ({} rewrite suggestions)",
                    qualification_result.verifiable_parts.len(),
                    rewrite_suggestions
                );
            }
            Err(e) => {
                let error = ProcessingError {
                    stage: ProcessingStage::Qualification,
                    error_type: "QualificationFailed".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                };
                errors.push(error);
                warn!("Qualification failed, continuing: {}", e);
            }
        }

        // Stage 3: Decomposition
        match self
            .decomposition_stage
            .process(&disambiguated_sentence, context)
            .await
        {
            Ok(decomposition_result) => {
                atomic_claims = decomposition_result.atomic_claims;
                stages_completed.push(ProcessingStage::Decomposition);
                info!(
                    "Decomposition completed: {} atomic claims extracted",
                    atomic_claims.len()
                );
            }
            Err(e) => {
                let error = ProcessingError {
                    stage: ProcessingStage::Decomposition,
                    error_type: "DecompositionFailed".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                };
                errors.push(error);
                warn!("Decomposition failed: {}", e);
            }
        }

        // Stage 4: Verification (evidence collection)
        if !atomic_claims.is_empty() {
            if let Some(ref verification_stage) = self.verification_stage {
                match verification_stage
                    .process(&atomic_claims, context)
                    .await
                {
                    Ok(verification_result) => {
                        // TODO: Convert verification result checks to Evidence format
                        verification_evidence = vec![]; // Temporarily empty until conversion is implemented
                        stages_completed.push(ProcessingStage::Verification);
                        info!(
                            "Verification completed: {} evidence items collected",
                            verification_evidence.len()
                        );
                    }
                    Err(e) => {
                        let error = ProcessingError {
                            stage: ProcessingStage::Verification,
                            error_type: "VerificationFailed".to_string(),
                            message: e.to_string(),
                            recoverable: true,
                        };
                        errors.push(error);
                        warn!("Verification failed: {}", e);
                    }
                }
            } else {
                // Verification stage disabled
                let error = ProcessingError {
                    stage: ProcessingStage::Verification,
                    error_type: "VerificationDisabled".to_string(),
                    message: "Verification stage is currently disabled".to_string(),
                    recoverable: true,
                };
                errors.push(error);
                warn!("Verification stage is disabled, skipping evidence collection");
            }
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        // Capture lengths before moving
        let claims_count = atomic_claims.len() as u32;
        let evidence_count = verification_evidence.len() as u32;

        let result = ClaimExtractionResult {
            original_sentence: sentence.to_string(),
            disambiguated_sentence,
            atomic_claims,
            verification_evidence,
            processing_metadata: ProcessingMetadata {
                processing_time_ms,
                stages_completed,
                ambiguities_resolved,
                claims_extracted: claims_count,
                evidence_collected: evidence_count,
                rewrite_suggestions,
                unverifiable_breakdown,
                errors,
            },
        };

        info!(
            "Claim extraction completed in {}ms with {} claims and {} evidence items",
            processing_time_ms,
            result.processing_metadata.claims_extracted,
            result.processing_metadata.evidence_collected
        );

        Ok(result)
    }
}

impl Default for ClaimExtractionAndVerificationProcessor {
    fn default() -> Self {
        Self::new()
    }
}
