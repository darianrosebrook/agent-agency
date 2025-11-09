//! Research Evidence Collector Adapter
//!
//! Adapts the real agent-research evidence collector to implement the contracts::ResearchEvidenceCollector trait.
//! This adapter enables dependency injection and breaks the direct dependency from orchestration to research.
//!
//! @author @darianrosebrook

#[cfg(feature = "research")]
use async_trait::async_trait;
#[cfg(feature = "research")]
use std::sync::Arc;
#[cfg(feature = "research")]
use uuid::Uuid;

#[cfg(feature = "research")]
use agent_agency_contracts::{
    ResearchEvidenceCollector as ContractsResearchEvidenceCollector,
    types::research::{Evidence, EvidenceType, EvidenceQuery, ValidationResult, EvidenceStats},
    errors::ResearchResult,
};
#[cfg(feature = "research")]
use crate::planning::evidence::{ResearchEvidenceCollector, ResearchEvidence as PlanningResearchEvidence, ProcessingContext as PlanningProcessingContext};

/// Adapter that wraps agent-research::EvidenceCollector to implement contracts::ResearchEvidenceCollector
#[cfg(feature = "research")]
pub struct ResearchEvidenceAdapter {
    /// The underlying evidence collector implementation
    #[allow(dead_code)] // Reserved for future use
    evidence_collector: Arc<agent_research::evidence::collector::EvidenceCollector>,
}

#[cfg(feature = "research")]
impl ResearchEvidenceAdapter {
    /// Create a new research evidence adapter
    pub fn new(evidence_collector: Arc<agent_research::evidence::collector::EvidenceCollector>) -> Self {
        Self { evidence_collector }
    }
}

#[cfg(feature = "research")]
#[async_trait]
impl ContractsResearchEvidenceCollector for ResearchEvidenceAdapter {
    async fn collect_evidence(&self, query: EvidenceQuery) -> ResearchResult<Vec<Evidence>> {
        // Convert contracts EvidenceQuery to agent-research types
        let atomic_claim = agent_research::extraction_types::AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: query.query.clone(),
            claim_type: agent_research::extraction_types::ClaimType::Factual, // Default to factual
            verifiability: agent_research::extraction_types::VerifiabilityLevel::DirectlyVerifiable,
            scope: agent_research::extraction_types::ClaimScope {
                working_spec_id: "contracts-adapter".to_string(),
                component_boundaries: vec![],
                data_impact: agent_research::extraction_types::DataImpact::ReadOnly,
            },
            confidence: 0.8,
            contextual_brackets: vec![],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec![],
            verification_requirements: vec![],
            position: (0, 0),
            sentence_fragment: "".to_string(),
            evidence_links: vec![],
            temporal_context: None,
            verification_status: agent_research::extraction_types::VerificationStatus::Unverified,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let processing_context = agent_research::extraction_types::ProcessingContext {
            task_id: Uuid::new_v4(),
            working_spec_id: "contracts-adapter".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: query.query.clone(),
            domain_hints: vec!["contracts".to_string()],
            metadata: query.context.clone(),
            input_text: query.query.clone(),
            language: None,
        };

        // Collect evidence using the real research collector
        // Note: EvidenceCollector requires &mut self, but we have Arc
        // Create a new instance for this call since we can't mutate through Arc
        let mut collector = agent_research::evidence::collector::EvidenceCollector::new();
        let research_evidence = collector.collect_evidence(&atomic_claim, &processing_context).await
            .map_err(|e| {
                let error_msg = format!("{}", e);
                agent_agency_contracts::errors::PlanningError::PlanGenerationFailed {
                    reason: format!("Research evidence collection failed: {}", error_msg)
                }
            })?;

        // Convert back to contracts types
        let contracts_evidence = research_evidence.into_iter().map(|ev| {
            self.convert_research_evidence_to_contracts(ev)
        }).collect();

        Ok(contracts_evidence)
    }

    async fn validate_evidence(&self, evidence: &Evidence) -> ResearchResult<ValidationResult> {
        // Convert contracts Evidence to agent-research AtomicClaim for validation
        let atomic_claim = agent_research::extraction_types::AtomicClaim {
            id: Uuid::parse_str(&evidence.id).unwrap_or_else(|_| Uuid::new_v4()),
            claim_text: evidence.content.clone(),
            claim_type: self.map_evidence_type_to_claim_type(evidence.evidence_type.clone()),
            verifiability: agent_research::extraction_types::VerifiabilityLevel::DirectlyVerifiable,
            scope: agent_research::extraction_types::ClaimScope {
                working_spec_id: "contracts-adapter".to_string(),
                component_boundaries: vec![],
                data_impact: agent_research::extraction_types::DataImpact::ReadOnly,
            },
            confidence: evidence.confidence,
            contextual_brackets: vec![],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec![],
            verification_requirements: vec![],
            position: (0, 0),
            sentence_fragment: "".to_string(),
            evidence_links: vec![],
            temporal_context: None,
            verification_status: agent_research::extraction_types::VerificationStatus::Unverified,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // TODO: Integrate real research validation pipeline from agent-research
        //       Currently returns basic validation result based on confidence threshold; should use full research validation pipeline.
        //
        // COMPLETION CHECKLIST:
        // [ ] Connect to agent-research validation services
        // [ ] Run full validation pipeline on claims and evidence
        // [ ] Calculate validation confidence scores using research algorithms
        // [ ] Include validation evidence and reasoning in results
        // [ ] Handle validation errors and timeouts gracefully
        // [ ] Add unit tests with mock validation results
        // [ ] Add integration tests with real research validation
        // [ ] Verify validation results improve claim verification accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Research validation pipeline is integrated and functional
        // - Validation results include confidence scores, evidence, and reasoning
        // - Validation errors and timeouts are handled gracefully
        // - Validation accuracy improves over basic confidence threshold approach
        //
        // DEPENDENCIES:
        // - agent-research validation services (Required)
        // - Research validation pipeline API (Required)
        // - Evidence and claim data structures (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (standard feature)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Research validation domain expertise
        let validation_result = ValidationResult {
            valid: evidence.confidence > 0.5,
            score: evidence.confidence,
            issues: if evidence.confidence < 0.5 {
                vec!["Low confidence evidence".to_string()]
            } else {
                vec![]
            },
            warnings: vec![],
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        };

        Ok(validation_result)
    }

    async fn search_evidence(&self, criteria: serde_json::Value) -> ResearchResult<Vec<Evidence>> {
        // TODO: Implement comprehensive evidence search from research database
        //       Currently returns empty results; should implement comprehensive search that queries the research evidence database using criteria for accurate evidence retrieval.
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
        // - Evidence is searched from research database
        // - Search criteria are properly applied
        // - Search results are accurate and relevant
        // - Search handles database errors gracefully
        //
        // DEPENDENCIES:
        // - Research evidence database connection (Required)
        // - Search query utilities (Required)
        // - Criteria parsing and validation (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (research evidence search functionality)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Database search and research evidence expertise
        warn!("search_evidence not fully implemented - returning empty results");
        Ok(Vec::new())
    }

    async fn get_evidence_stats(&self) -> ResearchResult<EvidenceStats> {
        // Return basic stats - in a full implementation, this would query the research system
        Ok(EvidenceStats {
            total_evidence: 0,
            average_confidence: 0.0,
            validations_performed: 0,
            average_validation_score: 0.0,
            collection_success_rate: 0.0,
            last_collection_time: None,
        })
    }
}

#[cfg(feature = "research")]
#[async_trait]
impl ResearchEvidenceCollector for ResearchEvidenceAdapter {
    async fn collect_evidence(&self, context: &PlanningProcessingContext) -> anyhow::Result<Vec<PlanningResearchEvidence>> {
        // Convert planning ProcessingContext to contracts EvidenceQuery
        let query = EvidenceQuery {
            query: format!("Task {} milestone {}", context.task_id, context.milestone_id),
            evidence_types: context.evidence_types.iter().map(|et| match et {
                crate::planning::evidence::ResearchEvidenceType::CodeReview | crate::planning::evidence::ResearchEvidenceType::CodeAnalysis => EvidenceType::CodeAnalysis,
                crate::planning::evidence::ResearchEvidenceType::TestExecution => EvidenceType::TestResults,
                crate::planning::evidence::ResearchEvidenceType::PerformanceMetrics | crate::planning::evidence::ResearchEvidenceType::Performance => EvidenceType::PerformanceMetrics,
                crate::planning::evidence::ResearchEvidenceType::SecurityScan | crate::planning::evidence::ResearchEvidenceType::Security => EvidenceType::SecurityScan,
                crate::planning::evidence::ResearchEvidenceType::Constitutional => EvidenceType::ConstitutionalReference,
                crate::planning::evidence::ResearchEvidenceType::Documentation => EvidenceType::Documentation,
            }).collect(),
            context: std::collections::HashMap::new(),
            limit: None,
            min_confidence: None,
        };

        // Use the contracts implementation via ContractsResearchEvidenceCollector trait
        let contracts_evidence = ContractsResearchEvidenceCollector::collect_evidence(self, query).await
            .map_err(|e| anyhow::anyhow!("Research evidence collection failed: {:?}", e))?;

        // Convert contracts Evidence to planning ResearchEvidence
        Ok(contracts_evidence.into_iter().map(|ev| PlanningResearchEvidence {
            id: Uuid::parse_str(&ev.id).unwrap_or_else(|_| Uuid::new_v4()),
            content: ev.content,
            evidence_type: match ev.evidence_type {
                EvidenceType::CodeAnalysis => crate::planning::evidence::ResearchEvidenceType::CodeAnalysis,
                EvidenceType::TestResults => crate::planning::evidence::ResearchEvidenceType::TestExecution,
                EvidenceType::PerformanceMetrics => crate::planning::evidence::ResearchEvidenceType::PerformanceMetrics,
                EvidenceType::SecurityScan => crate::planning::evidence::ResearchEvidenceType::SecurityScan,
                EvidenceType::ConstitutionalReference => crate::planning::evidence::ResearchEvidenceType::Constitutional,
                EvidenceType::Documentation => crate::planning::evidence::ResearchEvidenceType::Documentation,
                _ => crate::planning::evidence::ResearchEvidenceType::CodeAnalysis, // Default
            },
            confidence: ev.confidence,
            source: ev.source,
            timestamp: ev.timestamp,
        }).collect())
    }
}

#[cfg(feature = "research")]
impl ResearchEvidenceAdapter {
    /// Convert agent-research Evidence to contracts Evidence
    fn convert_research_evidence_to_contracts(&self, research_ev: agent_research::extraction_types::Evidence) -> Evidence {
        Evidence {
            id: research_ev.id.to_string(),
            evidence_type: self.map_research_evidence_type_to_contracts(research_ev.evidence_type),
            content: research_ev.content,
            source: format!("{:?}", research_ev.source), // Convert EvidenceSource to string
            confidence: research_ev.confidence,
            relevance: research_ev.relevance,
            timestamp: research_ev.timestamp,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Map contracts EvidenceType to agent-research ClaimType
    fn map_evidence_type_to_claim_type(&self, evidence_type: EvidenceType) -> agent_research::extraction_types::ClaimType {
        match evidence_type {
            EvidenceType::CodeAnalysis | EvidenceType::TestResults => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::Documentation => agent_research::extraction_types::ClaimType::Factual, // Definitional -> Factual
            EvidenceType::ResearchFindings | EvidenceType::PerformanceMetrics => agent_research::extraction_types::ClaimType::Quantitative, // Comparative -> Quantitative
            EvidenceType::SecurityScan => agent_research::extraction_types::ClaimType::Security,
            EvidenceType::ConstitutionalReference => agent_research::extraction_types::ClaimType::Constitutional, // Normative -> Constitutional
            EvidenceType::CouncilDecision => agent_research::extraction_types::ClaimType::Constitutional, // Normative -> Constitutional
            EvidenceType::MultiModalAnalysis => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::ExternalSource => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::TestResult => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::UserFeedback => agent_research::extraction_types::ClaimType::Behavioral, // Evaluative -> Behavioral
            EvidenceType::Measurement => agent_research::extraction_types::ClaimType::Quantitative,
            EvidenceType::LogicalAnalysis => agent_research::extraction_types::ClaimType::Factual, // Definitional -> Factual
            EvidenceType::Supporting => agent_research::extraction_types::ClaimType::Factual,
        }
    }

    /// Map agent-research EvidenceType to contracts EvidenceType
    fn map_research_evidence_type_to_contracts(&self, research_type: agent_research::extraction_types::EvidenceType) -> EvidenceType {
        match research_type {
            agent_research::extraction_types::EvidenceType::CodeAnalysis => EvidenceType::CodeAnalysis,
            agent_research::extraction_types::EvidenceType::TestResults => EvidenceType::TestResults,
            agent_research::extraction_types::EvidenceType::TestExecution => EvidenceType::TestResults, // Map TestExecution to TestResults
            agent_research::extraction_types::EvidenceType::Documentation => EvidenceType::Documentation,
            agent_research::extraction_types::EvidenceType::ResearchFindings => EvidenceType::ResearchFindings,
            agent_research::extraction_types::EvidenceType::PerformanceMetrics => EvidenceType::PerformanceMetrics,
            agent_research::extraction_types::EvidenceType::SecurityScan => EvidenceType::SecurityScan,
            agent_research::extraction_types::EvidenceType::ConstitutionalReference => EvidenceType::ConstitutionalReference,
            agent_research::extraction_types::EvidenceType::CouncilDecision => EvidenceType::CouncilDecision,
            agent_research::extraction_types::EvidenceType::MultiModalAnalysis => EvidenceType::MultiModalAnalysis,
            agent_research::extraction_types::EvidenceType::ExternalSource => EvidenceType::ExternalSource,
            agent_research::extraction_types::EvidenceType::TestResult => EvidenceType::TestResult,
            agent_research::extraction_types::EvidenceType::UserFeedback => EvidenceType::UserFeedback,
            agent_research::extraction_types::EvidenceType::Measurement => EvidenceType::Measurement,
            agent_research::extraction_types::EvidenceType::LogicalAnalysis => EvidenceType::LogicalAnalysis,
            agent_research::extraction_types::EvidenceType::Supporting => EvidenceType::Supporting,
        }
    }
}
