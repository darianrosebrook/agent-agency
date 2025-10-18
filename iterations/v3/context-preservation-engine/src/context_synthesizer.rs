use crate::types::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tracing::debug;
use uuid::Uuid;

/// Context synthesizer for creating cross-references and synthesis
#[derive(Debug)]
pub struct ContextSynthesizer {
    /// Synthesizer configuration
    config: ContextPreservationConfig,
}

impl ContextSynthesizer {
    /// Create a new context synthesizer
    pub fn new(config: ContextPreservationConfig) -> Result<Self> {
        debug!("Initializing context synthesizer");
        Ok(Self { config })
    }

    /// Synthesize context
    pub async fn synthesize_context(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
        context_data: &ContextData,
        metadata: &ContextMetadata,
    ) -> Result<Vec<SynthesisResult>> {
        debug!(
            "Synthesizing context: {} for tenant: {}",
            context_id, tenant_id
        );

        if !self.config.synthesis.enabled {
            return Ok(Vec::new());
        }

        let keywords = Self::extract_keywords(&context_data.content, 8);
        let base_confidence = 0.55
            + (metadata.tags.len().min(5) as f64 * 0.05)
            + (metadata.relationships.len().min(5) as f64 * 0.04);

        let mut results = Vec::new();

        let summary = if !metadata.description.is_empty() {
            format!(
                "{} | key themes: {}",
                metadata.description,
                if keywords.is_empty() {
                    "none identified".to_string()
                } else {
                    keywords.join(", ")
                }
            )
        } else {
            format!(
                "Synthesized context for {} with {} tags",
                tenant_id,
                metadata.tags.len()
            )
        };

        let synthesis_type = match metadata.context_type {
            ContextType::Research => SynthesisType::Summarization,
            ContextType::Performance | ContextType::Security => SynthesisType::Aggregation,
            ContextType::Council => SynthesisType::Transformation,
            _ => SynthesisType::Enrichment,
        };

        results.push(SynthesisResult {
            id: Uuid::new_v4(),
            synthesized_context_id: *context_id,
            synthesis_type,
            confidence: base_confidence.min(0.95),
            description: summary,
        });

        if !metadata.tags.is_empty() {
            let tag_description = format!(
                "Tags: {}",
                metadata
                    .tags
                    .iter()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            results.push(SynthesisResult {
                id: Uuid::new_v4(),
                synthesized_context_id: *context_id,
                synthesis_type: SynthesisType::Enrichment,
                confidence: (base_confidence + 0.1).min(0.98),
                description: tag_description,
            });
        }

        if !metadata.relationships.is_empty() {
            let relationship_summary = metadata
                .relationships
                .iter()
                .take(5)
                .map(|rel| format!("{:?} -> {}", rel.relationship_type, rel.related_context_id))
                .collect::<Vec<_>>()
                .join("; ");
            results.push(SynthesisResult {
                id: Uuid::new_v4(),
                synthesized_context_id: *context_id,
                synthesis_type: SynthesisType::Aggregation,
                confidence: (base_confidence + 0.08).min(0.97),
                description: format!("Relationships: {}", relationship_summary),
            });
        }

        if results.len() > self.config.synthesis.max_synthesis_depth as usize {
            results.truncate(self.config.synthesis.max_synthesis_depth as usize);
        }

        Ok(results)
    }

    /// Create cross-references
    pub async fn create_cross_references(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
        context_data: &ContextData,
        metadata: &ContextMetadata,
    ) -> Result<Vec<CrossReference>> {
        debug!(
            "Creating cross-references for context: {} for tenant: {}",
            context_id, tenant_id
        );

        if !self.config.synthesis.enable_cross_references {
            return Ok(Vec::new());
        }

        let mut references = Vec::new();

        for dependency in metadata
            .dependencies
            .iter()
            .take(self.config.synthesis.max_cross_references as usize)
        {
            references.push(CrossReference {
                id: Uuid::new_v4(),
                referenced_context_id: *dependency,
                reference_type: ReferenceType::Dependency,
                strength: 0.9,
                context: format!("{} depends on {}", context_id, dependency),
            });
        }

        for relationship in metadata
            .relationships
            .iter()
            .take(self.config.synthesis.max_cross_references as usize)
        {
            references.push(CrossReference {
                id: Uuid::new_v4(),
                referenced_context_id: relationship.related_context_id,
                reference_type: Self::map_relationship_type(&relationship.relationship_type),
                strength: relationship.strength.clamp(0.0, 1.0),
                context: format!(
                    "{} relationship {:?}: {}",
                    tenant_id, relationship.relationship_type, relationship.description
                ),
            });
        }

        if references.len() > self.config.synthesis.max_cross_references as usize {
            references.truncate(self.config.synthesis.max_cross_references as usize);
        }

        Ok(references)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing context synthesizer health check");

        let storage_ok = self.config.storage.max_context_size > 0
            && self.config.storage.max_contexts_per_tenant > 0
            && self.config.storage.cache_size_limit > 0;

        let synthesis_ok = if self.config.synthesis.enabled {
            self.config.synthesis.max_synthesis_depth > 0
                && (0.0..=1.0).contains(&self.config.synthesis.similarity_threshold)
        } else {
            true
        };

        let cross_ref_ok = !self.config.synthesis.enable_cross_references
            || self.config.synthesis.max_cross_references > 0;

        Ok(storage_ok && synthesis_ok && cross_ref_ok)
    }
}

impl ContextSynthesizer {
    fn extract_keywords(content: &str, limit: usize) -> Vec<String> {
        let stop_words: HashSet<&str> = [
            "the", "and", "for", "with", "that", "this", "from", "have", "will", "into", "about",
            "which", "their", "would", "there", "within", "using",
        ]
        .into_iter()
        .collect();

        let mut frequencies: HashMap<String, usize> = HashMap::new();

        for token in content.split(|c: char| !c.is_alphanumeric()) {
            if token.len() < 3 {
                continue;
            }
            let normalized = token.to_lowercase();
            if stop_words.contains(normalized.as_str()) {
                continue;
            }
            *frequencies.entry(normalized).or_insert(0) += 1;
        }

        let mut ranked: Vec<(String, usize)> = frequencies.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked
            .into_iter()
            .take(limit)
            .map(|(keyword, _)| keyword)
            .collect()
    }

    fn map_relationship_type(relationship_type: &RelationshipType) -> ReferenceType {
        match relationship_type {
            RelationshipType::Dependency => ReferenceType::Dependency,
            RelationshipType::Similarity => ReferenceType::Similarity,
            RelationshipType::Reference => ReferenceType::Indirect,
            RelationshipType::ParentChild | RelationshipType::Sibling => ReferenceType::Direct,
            _ => ReferenceType::Other,
        }
    }
}
