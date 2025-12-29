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
    /// 
    /// Returns a DisambiguationResult with a hard gate: if unresolvable ambiguities
    /// are detected and cannot be resolved with available context, the result will
    /// indicate that extraction should be skipped.
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DisambiguationResult> {
        debug!("Starting disambiguation for: {}", sentence);

        // Step 1: Identify ambiguities using detector
        let ambiguities = self.identify_ambiguities(sentence, context).await?;
        debug!("Identified {} ambiguities", ambiguities.len());

        // Step 2: Detect unresolvable ambiguities BEFORE attempting resolution
        // This is the hard gate - if we can't resolve with available context, skip extraction
        let unresolvable = self.detect_unresolvable_ambiguities(&ambiguities, context);
        
        // Hard gate: If there are unresolvable ambiguities that require external knowledge,
        // we should skip extraction rather than guess
        let has_critical_unresolvable = unresolvable.iter().any(|ua| {
            matches!(
                ua.reason,
                UnresolvableReason::InsufficientContext
                    | UnresolvableReason::AmbiguousReference
                    | UnresolvableReason::MissingInformation
                    | UnresolvableReason::ConflictingEvidence
            )
        });
        
        if has_critical_unresolvable {
            debug!(
                "Hard gate triggered: {} unresolvable ambiguities detected, skipping extraction",
                unresolvable.len()
            );
            // Return result indicating extraction should be skipped
            return Ok(DisambiguationResult {
                original_sentence: sentence.to_string(),
                disambiguated_sentence: sentence.to_string(), // Keep original
                ambiguities_resolved: 0,
                unresolvable_ambiguities: unresolvable,
            });
        }

        // Step 3: Resolve referential ambiguities (pronouns, etc.)
        let disambiguated_sentence = self
            .resolve_referential_ambiguities(sentence, &ambiguities, context)
            .await?;

        // Step 4: Count resolved ambiguities
        let ambiguities_resolved = ambiguities.len() as u32;

        Ok(DisambiguationResult {
            original_sentence: sentence.to_string(),
            disambiguated_sentence,
            ambiguities_resolved,
            unresolvable_ambiguities: unresolvable,
        })
    }
    
    /// Check if a sentence should skip extraction due to unresolvable ambiguities
    /// 
    /// This is the hard gate function that determines whether extraction should proceed.
    /// Returns true if extraction should be skipped.
    pub async fn should_skip_extraction(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<bool> {
        let ambiguities = self.identify_ambiguities(sentence, context).await?;
        let unresolvable = self.detect_unresolvable_ambiguities(&ambiguities, context);
        
        // Skip if there are critical unresolvable ambiguities
        let should_skip = unresolvable.iter().any(|ua| {
            matches!(
                ua.reason,
                UnresolvableReason::InsufficientContext
                    | UnresolvableReason::AmbiguousReference
                    | UnresolvableReason::MissingInformation
                    | UnresolvableReason::ConflictingEvidence
            )
        });
        
        Ok(should_skip)
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
                    // These ambiguity types are handled by resolver but not replaced in text
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
