//! Main disambiguation stage orchestrator

use crate::disambiguation::context::ContextResolver;
use crate::disambiguation::detection::AmbiguityDetector;
use crate::disambiguation::disambiguation_types::*;
use crate::disambiguation::entities::NamedEntityRecognizer;
use crate::disambiguation::types::{Ambiguity, AmbiguityType};
use crate::ProcessingContext;
use anyhow::Result;
use std::sync::Arc;
use tracing::debug;
// Explicit imports from contracts - use fully qualified names to avoid conflicts
use agent_agency_contracts::types::research::{
    EmbeddingProvider, KnowledgeBase, KnowledgeIngest,
    UnresolvableAmbiguity as ContractsUnresolvableAmbiguity, UnresolvableReason,
};

/// Main disambiguation stage that orchestrates the entire process
// #[derive(Debug)] // Removed due to trait object issues
pub struct DisambiguationStage {
    detector: AmbiguityDetector,
    resolver: ContextResolver,
    recognizer: NamedEntityRecognizer,
}

impl std::fmt::Debug for DisambiguationStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisambiguationStage")
            .field("detector", &"AmbiguityDetector")
            .field("resolver", &"ContextResolver")
            .field("recognizer", &"NamedEntityRecognizer")
            .finish()
    }
}

impl DisambiguationStage {
    /// Create a minimal DisambiguationStage with no optional services
    pub fn minimal() -> Self {
        Self {
            detector: AmbiguityDetector::new(),
            resolver: ContextResolver::new(),
            recognizer: NamedEntityRecognizer::new(),
        }
    }

    /// Create a DisambiguationStage with optional services
    pub fn with_services(
        embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
        knowledge_base: Option<Arc<dyn KnowledgeBase>>,
        knowledge_ingest: Option<Arc<dyn KnowledgeIngest>>,
    ) -> Self {
        Self {
            detector: AmbiguityDetector::new(),
            resolver: ContextResolver::with_services(
                embedding_provider.clone(),
                knowledge_base.clone(),
                knowledge_ingest.clone(),
            ),
            recognizer: NamedEntityRecognizer::with_services(
                embedding_provider,
                knowledge_base,
                knowledge_ingest,
            ),
        }
    }

    /// Process a sentence through disambiguation
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DisambiguationResult> {
        debug!("Starting disambiguation for: {}", sentence);

        // Step 1: Identify ambiguities using detector
        let ambiguities = self.identify_ambiguities(sentence, context).await?;
        debug!("Identified {} ambiguities", ambiguities.len());

        // Step 2: Resolve referential ambiguities (pronouns, etc.)
        let disambiguated_sentence = self
            .resolve_referential_ambiguities(sentence, &ambiguities, context)
            .await?;

        // Step 3: Count resolved ambiguities
        let ambiguities_resolved = ambiguities.len() as u32;

        // Step 4: Detect unresolvable ambiguities
        let unresolvable = self.detect_unresolvable_ambiguities(&ambiguities, context);

        Ok(DisambiguationResult {
            original_sentence: sentence.to_string(),
            disambiguated_sentence,
            ambiguities_resolved,
            unresolvable_ambiguities: unresolvable,
        })
    }

    /// Identify ambiguities in a sentence
    async fn identify_ambiguities(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<Ambiguity>> {
        let mut ambiguities = Vec::new();

        // Detect pronouns
        ambiguities.extend(self.detector.detect_pronouns(sentence)?);

        // Detect technical terms
        ambiguities.extend(self.detector.detect_technical_terms(sentence, context)?);

        // Detect scope boundaries
        ambiguities.extend(self.detector.detect_scope_boundaries(sentence)?);

        // Detect temporal references
        ambiguities.extend(self.detector.detect_temporal_references(sentence)?);

        // Additional entity-based ambiguities could be detected here
        // using the recognizer if needed

        Ok(ambiguities)
    }

    /// Resolve referential ambiguities (pronouns, etc.)
    async fn resolve_referential_ambiguities(
        &self,
        sentence: &str,
        ambiguities: &[Ambiguity],
        context: &ProcessingContext,
    ) -> Result<String> {
        let mut disambiguated = sentence.to_string();

        // Build referent map from context
        let referent_map = self.resolver.build_v2_referent_map(context).await?;

        // Process each ambiguity
        for ambiguity in ambiguities {
            match ambiguity.ambiguity_type {
                AmbiguityType::Pronoun => {
                    if let Some(resolution) =
                        self.resolver.resolve_ambiguity(ambiguity, context).await?
                    {
                        // TODO: Implement sophisticated text replacement for disambiguation
                        //       Currently uses simple string replacement; should use more sophisticated replacement that preserves context, handles multiple occurrences, and maintains grammatical correctness.
                        //
                        // COMPLETION CHECKLIST:
                        // [ ] Implement context-aware text replacement
                        // [ ] Handle multiple occurrences of ambiguous text
                        // [ ] Preserve grammatical correctness
                        // [ ] Maintain sentence structure and flow
                        // [ ] Handle edge cases (punctuation, capitalization)
                        // [ ] Add unit tests with various ambiguity types
                        // [ ] Add integration tests with real disambiguation scenarios
                        // [ ] Performance: Replacement should complete in <1ms
                        // [ ] Documentation: Document replacement algorithm
                        //
                        // ACCEPTANCE CRITERIA:
                        // - Text replacement preserves context
                        // - Multiple occurrences are handled correctly
                        // - Grammatical correctness is maintained
                        // - Sentence structure is preserved
                        // - Edge cases are handled appropriately
                        //
                        // DEPENDENCIES:
                        // - Text processing utilities (Required)
                        // - Grammar analysis (Optional)
                        // - Context preservation logic (Required)
                        //
                        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
                        // PRIORITY: Medium
                        // BLOCKING: No
                        //
                        // GOVERNANCE:
                        // - CAWS Tier: 2 (NLP feature)
                        // - Change Budget: ~200 LOC
                        // - Reviewer Requirements: NLP expertise
                        // Case-insensitive replacement for pronouns
                        if ambiguity.ambiguity_type == AmbiguityType::Pronoun {
                            // For pronouns, replace case-insensitively
                            let ambiguity_lower = ambiguity.original_text.to_lowercase();
                            let text_lower = disambiguated.to_lowercase();

                            if let Some(start) = text_lower.find(&ambiguity_lower) {
                                let actual_end = start + ambiguity.original_text.len();
                                let mut result =
                                    String::with_capacity(disambiguated.len() + resolution.len());
                                result.push_str(&disambiguated[..start]);
                                result.push_str(&resolution);
                                result.push_str(&disambiguated[actual_end..]);
                                disambiguated = result;
                            } else {
                                // Fallback to case-sensitive replacement
                                disambiguated =
                                    disambiguated.replace(&ambiguity.original_text, &resolution);
                            }
                        } else {
                            disambiguated =
                                disambiguated.replace(&ambiguity.original_text, &resolution);
                        }
                    }
                }
                AmbiguityType::TechnicalTerm
                | AmbiguityType::ScopeBoundary
                | AmbiguityType::TemporalReference
                | AmbiguityType::Quantifier
                | AmbiguityType::EntityReference
                | AmbiguityType::Other(_) => {
                    // TODO: Implement comprehensive text replacement for resolved ambiguities
                    //       Currently these are handled by resolver but not replaced in text; should implement comprehensive replacement that replaces resolved ambiguities in text for complete disambiguation.
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
                    // - Resolved ambiguities are replaced in text
                    // - Text replacement preserves context and meaning
                    // - Replacement handles all ambiguity types
                    // - Replacement is accurate and contextually appropriate
                    //
                    // DEPENDENCIES:
                    // - Text replacement utilities (Required)
                    // - Ambiguity resolution results (Required)
                    // - Context preservation logic (Required)
                    //
                    // ESTIMATED EFFORT: 8-12 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (text disambiguation functionality)
                    // - Change Budget: ~200 LOC
                    // - Reviewer Requirements: Text processing and ambiguity resolution expertise
                }
            }
        }

        Ok(disambiguated)
    }

    /// Detect which ambiguities are unresolvable
    fn detect_unresolvable_ambiguities(
        &self,
        ambiguities: &[Ambiguity],
        context: &ProcessingContext,
    ) -> Vec<ContractsUnresolvableAmbiguity> {
        ambiguities
            .iter()
            .filter_map(|ambiguity| {
                if let Some(reason) = self
                    .resolver
                    .detect_unresolvable_ambiguity(ambiguity, context)
                {
                    Some(ContractsUnresolvableAmbiguity {
                        ambiguity: ambiguity.original_text.clone(),
                        suggested_context: {
                            let resolutions = self
                                .resolver
                                .get_pronoun_resolutions(&ambiguity.original_text, context);
                            if resolutions.is_empty() {
                                Some("no resolution available".to_string())
                            } else {
                                Some(format!("{:?}", resolutions))
                            }
                        },
                        reason,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Access the named entity recognizer for advanced use cases
    pub fn recognizer(&self) -> &NamedEntityRecognizer {
        &self.recognizer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_process_minimal() {
        let stage = DisambiguationStage::minimal();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test".to_string(),
            domain_hints: vec!["system".to_string()],
            metadata: HashMap::new(),
            input_text: "It works well.".to_string(),
            language: None,
        };

        let result = stage.process("It works well.", &context).await.unwrap();

        assert_eq!(result.original_sentence, "It works well.");
        // Should resolve "It" to "system" based on domain hints
        assert!(result.disambiguated_sentence.contains("system"));
        assert_eq!(result.ambiguities_resolved, 1);
    }

    #[tokio::test]
    async fn test_process_no_ambiguities() {
        let stage = DisambiguationStage::minimal();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test".to_string(),
            domain_hints: vec![],
            metadata: HashMap::new(),
            input_text: "The system works well.".to_string(),
            language: None,
        };

        let result = stage
            .process("The system works well.", &context)
            .await
            .unwrap();

        assert_eq!(result.original_sentence, "The system works well.");
        assert_eq!(result.disambiguated_sentence, "The system works well.");
        assert_eq!(result.ambiguities_resolved, 0);
    }

    #[test]
    fn test_minimal_constructor() {
        let stage = DisambiguationStage::minimal();

        // Should be able to access recognizer
        let _recognizer = stage.recognizer();
    }
}
