//! Context synthesis and summarization

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::info;

use crate::research_types::*;
use crate::{ConfigurationUpdate, ContextBuilder};

use super::events::EventEmitter;

/// Context synthesizer for combining and summarizing research results

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContextSynthesizerr {
    context_builder: Arc<ContextBuilder>,
    config: ResearchAgentConfig,
    event_emitter: Arc<EventEmitter>,
}

impl ContextSynthesizer {
    /// Create a new context synthesizer
    pub async fn new(config: ResearchAgentConfig) -> Result<Self> {
        let context_builder = Arc::new(ContextBuilder::new(config.context_synthesis.clone()));
        let event_emitter = Arc::new(EventEmitter::new());

        Ok(Self {
            context_builder,
            config,
            event_emitter,
        })
    }

    /// Synthesize context from research results
    pub async fn synthesize(
        &self,
        query_id: Uuid,
        results: Vec<ResearchResult>,
    ) -> Result<SynthesizedContext> {
        info!("Synthesizing context for query {} with {} results", query_id, results.len());

        if results.is_empty() {
            return Ok(SynthesizedContext {
                query_id,
                synthesized_text: "No research results available for context synthesis.".to_string(),
                key_insights: vec![],
                confidence_score: 0.0,
                sources_used: vec![],
                synthesis_method: "empty_results".to_string(),
                processing_time_ms: 0,
            });
        }

        // Combine all content
        let combined_content = self.combine_results_content(&results);

        // Generate synthesis using context builder
        let synthesis_result = self.context_builder
            .build_context(&combined_content, self.config.context_synthesis.max_context_length)
            .await?;

        // Extract key insights
        let key_insights = self.extract_key_insights(&results);

        // Calculate overall confidence
        let confidence_score = self.calculate_synthesis_confidence(&results);

        let context = SynthesizedContext {
            query_id,
            synthesized_text: synthesis_result.context,
            key_insights,
            confidence_score,
            sources_used: results.iter().map(|r| r.source.clone()).collect(),
            synthesis_method: "hybrid_context_building".to_string(),
            processing_time_ms: synthesis_result.processing_time_ms,
        };

        info!("Context synthesis completed for query {}", query_id);
        Ok(context)
    }

    /// Combine content from multiple research results
    fn combine_results_content(&self, results: &[ResearchResult]) -> String {
        let mut combined = String::new();

        for (i, result) in results.iter().enumerate() {
            combined.push_str(&format!("=== Source {}: {} ===\n", i + 1, result.title));
            combined.push_str(&format!("URL: {}\n", result.url.as_deref().unwrap_or("N/A")));
            combined.push_str(&format!("Relevance: {:.2}\n", result.relevance_score));
            combined.push_str(&format!("Content:\n{}\n\n", result.content));
        }

        combined
    }

    /// Extract key insights from research results
    fn extract_key_insights(&self, results: &[ResearchResult]) -> Vec<String> {
        let mut insights = Vec::new();

        // Simple extraction based on summaries and high-relevance content
        for result in results {
            if result.relevance_score > 0.7 {
                if let Some(summary) = &result.summary {
                    insights.push(format!("{}: {}", result.title, summary));
                } else {
                    // Extract first sentence as insight
                    if let Some(first_sentence) = result.content.split('.').next() {
                        insights.push(format!("{}: {}", result.title, first_sentence.trim()));
                    }
                }
            }
        }

        // Limit to top 5 insights
        insights.into_iter().take(5).collect()
    }

    /// Calculate confidence score for the synthesis
    fn calculate_synthesis_confidence(&self, results: &[ResearchResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }

        let avg_relevance: f32 = results.iter().map(|r| r.relevance_score).sum::<f32>() / results.len() as f32;
        let avg_confidence: f32 = results.iter().map(|r| r.confidence_score).sum::<f32>() / results.len() as f32;

        // Weighted combination
        (avg_relevance * 0.6) + (avg_confidence * 0.4)
    }

    /// Update configuration
    pub async fn update_config(&self, update: ConfigurationUpdate) -> Result<()> {
        match update {
            ConfigurationUpdate::ContextSynthesis(config) => {
                // Update context synthesis configuration
                info!("Context synthesis configuration updated");
            }
            _ => {} // Other updates don't affect synthesis
        }
        Ok(())
    }
}
