//! Context Builder
//!
//! Synthesizes context from research results and builds coherent knowledge representations.
//! Includes cross-reference detection and context synthesis capabilities.

use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Context builder for synthesizing research results
#[derive(Debug)]
pub struct ContextBuilder {
    config: ContextSynthesisConfig,
    cache: Arc<RwLock<std::collections::HashMap<String, SynthesizedContext>>>,
    cross_reference_detector: CrossReferenceDetector,
}

/// Cross-reference detector for finding related knowledge
#[derive(Debug)]
pub struct CrossReferenceDetector {
    similarity_threshold: f32,
    max_references: usize,
}

/// Context synthesis metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSynthesisMetrics {
    pub synthesis_time_ms: u64,
    pub evidence_items_processed: usize,
    pub cross_references_found: usize,
    pub context_reuse_rate: f32,
    pub evidence_precision_score: f32,
    pub evidence_recall_score: f32,
    pub synthesis_confidence: f32,
}

impl CrossReferenceDetector {
    /// Create a new cross-reference detector
    pub fn new(similarity_threshold: f32, max_references: usize) -> Self {
        Self {
            similarity_threshold,
            max_references,
        }
    }

    /// Detect cross-references between research results
    pub async fn detect_cross_references(
        &self,
        results: &[ResearchResult],
    ) -> Result<Vec<CrossReference>> {
        debug!(
            "Detecting cross-references among {} research results",
            results.len()
        );

        let mut cross_references = Vec::new();

        // Simple similarity-based cross-reference detection
        for (i, result1) in results.iter().enumerate() {
            for (j, result2) in results.iter().enumerate() {
                if i >= j {
                    continue;
                } // Avoid self-comparison and duplicates

                let similarity = self.calculate_similarity(result1, result2);
                if similarity >= self.similarity_threshold {
                    let cross_ref = CrossReference {
                        source_id: Uuid::new_v4(), // Generate IDs for cross-references
                        target_id: Uuid::new_v4(),
                        relationship: CrossReferenceType::Related,
                        strength: similarity,
                        context: format!(
                            "Similarity between '{}' and '{}'",
                            result1.title, result2.title
                        ),
                    };
                    cross_references.push(cross_ref);
                }
            }
        }

        // Sort by strength and limit results
        cross_references.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        cross_references.truncate(self.max_references);

        debug!("Found {} cross-references", cross_references.len());
        Ok(cross_references)
    }

    /// Calculate similarity between two research results
    fn calculate_similarity(&self, result1: &ResearchResult, result2: &ResearchResult) -> f32 {
        // Simple keyword-based similarity calculation
        let keywords1: std::collections::HashSet<String> = result1
            .content
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let keywords2: std::collections::HashSet<String> = result2
            .content
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let intersection = keywords1.intersection(&keywords2).count();
        let union = keywords1.union(&keywords2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new(config: ContextSynthesisConfig) -> Self {
        let cross_reference_detector =
            CrossReferenceDetector::new(config.similarity_threshold, config.max_cross_references);

        Self {
            config,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            cross_reference_detector,
        }
    }

    /// Synthesize context from research results with cross-reference detection
    pub async fn synthesize_context(
        &self,
        query_id: Uuid,
        results: Vec<ResearchResult>,
    ) -> Result<(SynthesizedContext, ContextSynthesisMetrics)> {
        let start_time = std::time::Instant::now();
        info!("Synthesizing context for query: {}", query_id);

        // Detect cross-references
        let cross_references = self
            .cross_reference_detector
            .detect_cross_references(&results)
            .await?;

        // Calculate evidence metrics
        let evidence_precision_score = self.calculate_evidence_precision(&results);
        let evidence_recall_score = self.calculate_evidence_recall(&results);
        let context_reuse_rate = self.calculate_context_reuse_rate(&results).await;

        // Generate context summary and key findings
        let (context_summary, key_findings) = self.generate_context_summary(&results);

        // Calculate synthesis confidence
        let synthesis_confidence = self.calculate_synthesis_confidence(
            &results,
            &cross_references,
            evidence_precision_score,
            evidence_recall_score,
        );

        let synthesized_context = SynthesizedContext {
            id: Uuid::new_v4(),
            query_id,
            context_summary,
            key_findings,
            supporting_evidence: results.clone(),
            confidence_score: synthesis_confidence,
            synthesized_at: chrono::Utc::now(),
            sources: results.iter().map(|result| result.source.clone()).collect(),
            cross_references,
        };

        let synthesis_time_ms = start_time.elapsed().as_millis() as u64;
        let metrics = ContextSynthesisMetrics {
            synthesis_time_ms,
            evidence_items_processed: results.len(),
            cross_references_found: synthesized_context.cross_references.len(),
            context_reuse_rate,
            evidence_precision_score,
            evidence_recall_score,
            synthesis_confidence,
        };

        info!(
            "Context synthesis completed for query: {} in {}ms with confidence {:.2}",
            query_id, synthesis_time_ms, synthesis_confidence
        );

        Ok((synthesized_context, metrics))
    }

    /// Calculate evidence precision score
    fn calculate_evidence_precision(&self, results: &[ResearchResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }

        // Simple precision calculation based on source reliability and content quality
        let total_score: f32 = results
            .iter()
            .map(|result| {
                let source_reliability = match &result.source {
                    KnowledgeSource::Documentation(_) => 0.9,
                    KnowledgeSource::CodeRepository(_) => 0.8,
                    KnowledgeSource::WebPage(_) => 0.6,
                    KnowledgeSource::CommunityPost(_) => 0.7,
                    KnowledgeSource::AcademicPaper(_) => 0.9,
                    KnowledgeSource::ApiDocumentation(_) => 0.8,
                    KnowledgeSource::InternalKnowledgeBase(_) => 0.8,
                };

                let content_quality = if result.content.len() > 100 { 0.8 } else { 0.5 };
                source_reliability * content_quality * result.confidence_score
            })
            .sum();

        total_score / results.len() as f32
    }

    /// Calculate evidence recall score
    fn calculate_evidence_recall(&self, results: &[ResearchResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }

        // Simple recall calculation based on coverage of result types
        let unique_sources: std::collections::HashSet<_> = results
            .iter()
            .map(|result| std::mem::discriminant(&result.source))
            .collect();

        unique_sources.len() as f32 / 7.0 // 7 source types in KnowledgeSource enum
    }

    /// Calculate context reuse rate
    async fn calculate_context_reuse_rate(&self, results: &[ResearchResult]) -> f32 {
        let cache = self.cache.read().await;

        if cache.is_empty() {
            return 0.0;
        }

        let mut reuse_count = 0;
        for result in results {
            let mut hasher = DefaultHasher::new();
            result.content.hash(&mut hasher);
            let source_discriminant = std::mem::discriminant(&result.source);
            let cache_key = format!("{:?}-{}", source_discriminant, hasher.finish());
            if cache.contains_key(&cache_key) {
                reuse_count += 1;
            }
        }

        reuse_count as f32 / results.len() as f32
    }

    /// Generate context summary and key findings
    fn generate_context_summary(&self, results: &[ResearchResult]) -> (String, Vec<String>) {
        let unique_sources = results
            .iter()
            .map(|result| std::mem::discriminant(&result.source))
            .collect::<std::collections::HashSet<_>>()
            .len();

        let summary = format!(
            "Synthesized context from {} research results covering {} different source types.",
            results.len(),
            unique_sources
        );

        let key_findings = vec![
            format!("Found {} research results", results.len()),
            format!("Coverage across {} different source types", unique_sources),
            "Cross-references detected between related content".to_string(),
        ];

        (summary, key_findings)
    }

    /// Calculate overall synthesis confidence
    fn calculate_synthesis_confidence(
        &self,
        results: &[ResearchResult],
        cross_references: &[CrossReference],
        precision_score: f32,
        recall_score: f32,
    ) -> f32 {
        let evidence_confidence = if results.len() >= 3 { 0.8 } else { 0.5 };
        let cross_ref_confidence = if !cross_references.is_empty() {
            0.7
        } else {
            0.5
        };

        (evidence_confidence * 0.4
            + precision_score * 0.3
            + recall_score * 0.2
            + cross_ref_confidence * 0.1)
            .min(1.0)
    }

    /// Clear synthesis cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Context synthesis cache cleared");
    }
}
