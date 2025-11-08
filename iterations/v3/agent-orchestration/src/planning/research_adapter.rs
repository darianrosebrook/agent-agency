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
    ResearchEvidenceCollector,
    types::research::{Evidence, EvidenceType, EvidenceQuery, ValidationResult, EvidenceStats},
    errors::ResearchResult,
};

/// Adapter that wraps agent-research::EvidenceCollector to implement contracts::ResearchEvidenceCollector
#[cfg(feature = "research")]
pub struct ResearchEvidenceAdapter {
    /// The underlying evidence collector implementation
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
impl ResearchEvidenceCollector for ResearchEvidenceAdapter {
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
        let research_evidence = self.evidence_collector.collect_evidence(&atomic_claim, &processing_context).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "research".to_string()
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
            claim_type: self.map_evidence_type_to_claim_type(evidence.evidence_type),
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

        // TODO: Integrate real research validation pipeline
        // - [ ] Connect to agent-research validation services
        // - [ ] Run full validation pipeline on claims and evidence
        // - [ ] Calculate validation confidence scores
        // - [ ] Include validation evidence and reasoning
        // - [ ] Handle validation errors and timeouts
        // - [ ] Add unit tests with mock validation results
        // - [ ] Add integration tests with real research validation
        // For now, return a basic validation result
        // In a full implementation, this would use the research validation pipeline
        let validation_result = ValidationResult {
            is_valid: evidence.confidence > 0.5,
            score: evidence.confidence,
            issues: if evidence.confidence < 0.5 {
                vec!["Low confidence evidence".to_string()]
            } else {
                vec![]
            },
            metadata: std::collections::HashMap::new(),
        };

        Ok(validation_result)
    }

    async fn search_evidence(&self, criteria: serde_json::Value) -> ResearchResult<Vec<Evidence>> {
        // For now, return empty results
        // In a full implementation, this would search the research evidence database
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
            EvidenceType::Documentation => agent_research::extraction_types::ClaimType::Definitional,
            EvidenceType::ResearchFindings | EvidenceType::PerformanceMetrics => agent_research::extraction_types::ClaimType::Comparative,
            EvidenceType::SecurityScan => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::ConstitutionalReference => agent_research::extraction_types::ClaimType::Normative,
            EvidenceType::CouncilDecision => agent_research::extraction_types::ClaimType::Normative,
            EvidenceType::MultiModalAnalysis => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::ExternalSource => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::TestResult => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::UserFeedback => agent_research::extraction_types::ClaimType::Evaluative,
            EvidenceType::Measurement => agent_research::extraction_types::ClaimType::Factual,
            EvidenceType::LogicalAnalysis => agent_research::extraction_types::ClaimType::Definitional,
            EvidenceType::Supporting => agent_research::extraction_types::ClaimType::Factual,
        }
    }

    /// Map agent-research EvidenceType to contracts EvidenceType
    fn map_research_evidence_type_to_contracts(&self, research_type: agent_research::extraction_types::EvidenceType) -> EvidenceType {
        match research_type {
            agent_research::extraction_types::EvidenceType::CodeAnalysis => EvidenceType::CodeAnalysis,
            agent_research::extraction_types::EvidenceType::TestResults => EvidenceType::TestResults,
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
