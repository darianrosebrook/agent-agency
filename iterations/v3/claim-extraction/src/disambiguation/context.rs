//! Context resolution for disambiguation

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use crate::disambiguation::types::*;
use crate::ProcessingContext;

/// Context resolver for disambiguating ambiguities
pub struct ContextResolver {
    domain_context: HashMap<String, String>,
    embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
    knowledge_base: Option<Arc<dyn KnowledgeBase>>,
    knowledge_ingest: Option<Arc<dyn KnowledgeIngest>>,
}

impl ContextResolver {
    /// Create a new ContextResolver
    pub fn new() -> Self {
        let mut domain_context = HashMap::new();
        domain_context.insert("system".to_string(), "the Agent Agency system".to_string());
        domain_context.insert("component".to_string(), "the current component".to_string());
        domain_context.insert("function".to_string(), "the specified function".to_string());

        Self {
            domain_context,
            embedding_provider: None,
            knowledge_base: None,
            knowledge_ingest: None,
        }
    }

    /// Create a ContextResolver with optional services
    pub fn with_services(
        embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
        knowledge_base: Option<Arc<dyn KnowledgeBase>>,
        knowledge_ingest: Option<Arc<dyn KnowledgeIngest>>,
    ) -> Self {
        let mut resolver = Self::new();
        resolver.embedding_provider = embedding_provider;
        resolver.knowledge_base = knowledge_base;
        resolver.knowledge_ingest = knowledge_ingest;
        resolver
    }

    /// Build V2 referent map from context (ported from V2)
    pub async fn build_v2_referent_map(
        &self,
        context: &ProcessingContext,
    ) -> Result<HashMap<String, ReferentInfo>> {
        let mut referent_map = HashMap::new();

        // Use domain hints as primary referents
        for hint in &context.domain_hints {
            referent_map.insert(
                hint.clone(),
                ReferentInfo {
                    entity: hint.clone(),
                    confidence: 0.9,
                    source: "domain_hint".to_string(),
                },
            );
        }

        // Extract entities from surrounding context
        let context_entities = self.extract_context_entities(context);
        for entity in context_entities {
            referent_map.insert(
                entity.clone(),
                ReferentInfo {
                    entity,
                    confidence: 0.8,
                    source: "surrounding_context".to_string(),
                },
            );
        }

        Ok(referent_map)
    }

    /// Find referent for a pronoun using context map
    pub fn find_referent_for_pronoun(
        &self,
        pronoun: &str,
        context_map: &HashMap<String, ReferentInfo>,
    ) -> Option<ReferentInfo> {
        context_map.get(pronoun).cloned()
    }

    /// Get possible resolutions for a pronoun based on context
    pub fn get_pronoun_resolutions(&self, pronoun: &str, context: &ProcessingContext) -> Vec<String> {
        let mut resolutions = Vec::new();

        // Use domain hints from context
        for hint in &context.domain_hints {
            resolutions.push(hint.clone());
        }

        // Add default system-level resolutions
        match pronoun.to_lowercase().as_str() {
            "it" | "this" | "that" => {
                resolutions.extend(vec![
                    "the system".to_string(),
                    "the component".to_string(),
                    "the function".to_string(),
                    "the process".to_string(),
                ]);
            }
            "they" | "them" | "these" | "those" => {
                resolutions.extend(vec![
                    "the components".to_string(),
                    "the systems".to_string(),
                    "the processes".to_string(),
                ]);
            }
            "we" | "us" => {
                resolutions.extend(vec![
                    "the development team".to_string(),
                    "the system architects".to_string(),
                ]);
            }
            _ => {
                resolutions.push("the system".to_string());
            }
        }

        // Add context from surrounding text if available
        if !context.surrounding_context.is_empty() {
            resolutions.push(format!("context: {}", context.surrounding_context));
        }

        resolutions
    }

    /// Resolve ambiguity using context
    pub async fn resolve_ambiguity(
        &self,
        ambiguity: &Ambiguity,
        context: &ProcessingContext,
    ) -> Result<Option<String>> {
        match ambiguity.ambiguity_type {
            AmbiguityType::Pronoun => {
                // Use domain hints to resolve pronouns
                if let Some(hint) = context.domain_hints.first() {
                    Ok(Some(hint.clone()))
                } else {
                    Ok(Some("the system".to_string()))
                }
            }
            AmbiguityType::TechnicalTerm => Ok(ambiguity.possible_resolutions.first().cloned()),
            AmbiguityType::ScopeBoundary => Ok(Some(format!("in {}", context.working_spec_id))),
            AmbiguityType::TemporalReference => Ok(Some("during execution".to_string())),
            AmbiguityType::Quantifier => Ok(Some("all instances".to_string())),
        }
    }

    /// Detect if ambiguity is unresolvable
    pub fn detect_unresolvable_ambiguity(
        &self,
        ambiguity: &Ambiguity,
        context: &ProcessingContext,
    ) -> Option<UnresolvableReason> {
        match ambiguity.ambiguity_type {
            AmbiguityType::Pronoun if context.domain_hints.is_empty() => {
                Some(UnresolvableReason::InsufficientContext)
            }
            AmbiguityType::TechnicalTerm if ambiguity.possible_resolutions.len() > 3 => {
                Some(UnresolvableReason::MultipleValidInterpretations)
            }
            AmbiguityType::ScopeBoundary if context.surrounding_context.is_empty() => {
                Some(UnresolvableReason::InsufficientContext)
            }
            _ => None,
        }
    }

    /// Extract context entities from processing context
    fn extract_context_entities(&self, context: &ProcessingContext) -> Vec<String> {
        let mut entities = Vec::new();

        // Extract from domain hints
        for hint in &context.domain_hints {
            entities.push(hint.clone());
        }

        // Extract from surrounding context (basic entity detection)
        if !context.surrounding_context.is_empty() {
            // Simple capitalization-based entity detection
            let words: Vec<&str> = context.surrounding_context.split_whitespace().collect();
            for word in words {
                if word.len() > 1 && word.chars().next().map_or(false, |c| c.is_uppercase()) {
                    entities.push(word.to_string());
                }
            }
        }

        // Remove duplicates
        entities.sort();
        entities.dedup();
        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_v2_referent_map() {
        let resolver = ContextResolver::new();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "The User component handles authentication.".to_string(),
            domain_hints: vec!["authentication".to_string()],
            metadata: HashMap::new(),
            input_text: "test".to_string(),
            language: None,
        };

        let referent_map = resolver.build_v2_referent_map(&context).await.unwrap();

        assert!(referent_map.contains_key("authentication"));
        assert!(referent_map.contains_key("The"));
    }

    #[test]
    fn test_get_pronoun_resolutions() {
        let resolver = ContextResolver::new();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test".to_string(),
            domain_hints: vec!["system".to_string()],
            metadata: HashMap::new(),
            input_text: "test".to_string(),
            language: None,
        };

        let resolutions = resolver.get_pronoun_resolutions("it", &context);

        assert!(resolutions.contains(&"system".to_string()));
        assert!(resolutions.contains(&"the system".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_ambiguity_pronoun() {
        let resolver = ContextResolver::new();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test".to_string(),
            domain_hints: vec!["authentication".to_string()],
            metadata: HashMap::new(),
            input_text: "test".to_string(),
            language: None,
        };

        let ambiguity = Ambiguity {
            ambiguity_type: AmbiguityType::Pronoun,
            position: (0, 2),
            original_text: "it".to_string(),
            possible_resolutions: vec![],
            confidence: 0.8,
        };

        let resolution = resolver.resolve_ambiguity(&ambiguity, &context).await.unwrap();

        assert_eq!(resolution, Some("authentication".to_string()));
    }

    #[test]
    fn test_detect_unresolvable_ambiguity() {
        let resolver = ContextResolver::new();
        let context = ProcessingContext {
            task_id: uuid::Uuid::new_v4(),
            working_spec_id: "test".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test".to_string(),
            domain_hints: vec![], // Empty domain hints
            metadata: HashMap::new(),
            input_text: "test".to_string(),
            language: None,
        };

        let ambiguity = Ambiguity {
            ambiguity_type: AmbiguityType::Pronoun,
            position: (0, 2),
            original_text: "it".to_string(),
            possible_resolutions: vec![],
            confidence: 0.8,
        };

        let reason = resolver.detect_unresolvable_ambiguity(&ambiguity, &context);

        assert_eq!(reason, Some(UnresolvableReason::InsufficientContext));
    }
}
