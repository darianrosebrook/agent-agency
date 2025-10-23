//! Integration between Multimodal Ingestors and Hierarchical Context Manager
//!
//! Connects the ingestors system with our enhanced context management to leverage
//! rich multimodal data (videos, slides, diagrams, speech) in context building.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::context::manager::{HierarchicalContextManager, ContextBundle, ContextBudget};
use crate::types::{Task, ModelContext, IterationContext};

/// Multimodal context enricher
pub struct MultimodalContextEnricher {
    context_manager: Arc<HierarchicalContextManager>,
    ingestor_cache: Arc<RwLock<HashMap<String, IngestorResult>>>,
    enrichment_config: EnrichmentConfig,
}

#[derive(Clone, Debug)]
pub struct EnrichmentConfig {
    pub enable_video_context: bool,
    pub enable_slide_context: bool,
    pub enable_diagram_context: bool,
    pub enable_speech_context: bool,
    pub max_segments_per_type: usize,
    pub quality_threshold: f32,
    pub temporal_relevance_window: f32, // seconds
}

impl Default for EnrichmentConfig {
    fn default() -> Self {
        Self {
            enable_video_context: true,
            enable_slide_context: true,
            enable_diagram_context: true,
            enable_speech_context: true,
            max_segments_per_type: 5,
            quality_threshold: 0.7,
            temporal_relevance_window: 300.0, // 5 minutes
        }
    }
}

/// Cached ingestor result
#[derive(Clone, Debug)]
pub struct IngestorResult {
    pub document_id: String,
    pub uri: String,
    pub segments: Vec<RichSegment>,
    pub speech_turns: Option<Vec<SpeechTurn>>,
    pub diagram_data: Option<DiagramData>,
    pub quality_score: f32,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// Rich segment with multimodal content
#[derive(Clone, Debug)]
pub struct RichSegment {
    pub id: String,
    pub segment_type: SegmentType,
    pub content: String,
    pub timestamp: Option<f32>,
    pub bbox: Option<BoundingBox>,
    pub ocr_confidence: Option<f32>,
    pub visual_embedding: Option<Vec<f32>>, // For similarity search
    pub relevance_score: f64,
}

/// Speech turn data
#[derive(Clone, Debug)]
pub struct SpeechTurn {
    pub speaker_id: Option<String>,
    pub text: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
    pub word_timings: Vec<WordTiming>,
}

/// Word-level timing data
#[derive(Clone, Debug)]
pub struct WordTiming {
    pub start_time: f32,
    pub end_time: f32,
    pub word: String,
}

/// Diagram data for context
#[derive(Clone, Debug)]
pub struct DiagramData {
    pub entities: Vec<DiagramEntity>,
    pub relationships: Vec<String>,
    pub visual_summary: String,
}

/// Diagram entity
#[derive(Clone, Debug)]
pub struct DiagramEntity {
    pub name: String,
    pub entity_type: String,
    pub attributes: HashMap<String, String>,
}

/// Segment type
#[derive(Clone, Debug, PartialEq)]
pub enum SegmentType {
    Slide,
    Speech,
    Diagram,
    Scene,
}

/// Bounding box
#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl MultimodalContextEnricher {
    pub fn new(context_manager: Arc<HierarchicalContextManager>) -> Self {
        Self {
            context_manager,
            ingestor_cache: Arc::new(RwLock::new(HashMap::new())),
            enrichment_config: EnrichmentConfig::default(),
        }
    }

    /// Enrich context with multimodal data from relevant documents
    pub async fn enrich_context(
        &self,
        task: &Task,
        base_context: &ContextBundle,
        budget: &ContextBudget,
    ) -> Result<ContextBundle, EnrichmentError> {
        info!("Enriching context with multimodal data for task: {}", task.description);

        // Find relevant documents based on task content
        let relevant_docs = self.find_relevant_documents(task).await?;

        if relevant_docs.is_empty() {
            debug!("No relevant multimodal documents found");
            return Ok(base_context.clone());
        }

        // Extract and rank relevant segments
        let enriched_segments = self.extract_relevant_segments(&relevant_docs, task).await?;

        if enriched_segments.is_empty() {
            debug!("No relevant segments found in documents");
            return Ok(base_context.clone());
        }

        // Build enriched context bundle
        let enriched_bundle = self.build_enriched_bundle(
            base_context,
            &enriched_segments,
            budget,
        ).await?;

        info!("Context enriched with {} multimodal segments", enriched_segments.len());

        Ok(enriched_bundle)
    }

    /// Find documents relevant to the task
    async fn find_relevant_documents(&self, task: &Task) -> Result<Vec<IngestorResult>, EnrichmentError> {
        let cache = self.ingestor_cache.read().await;

        // Simple keyword-based relevance for now
        // In practice, this would use semantic search over document metadata
        let relevant_docs: Vec<IngestorResult> = cache.values()
            .filter(|doc| self.is_document_relevant(doc, task))
            .cloned()
            .collect();

        Ok(relevant_docs)
    }

    /// Check if document is relevant to task
    fn is_document_relevant(&self, doc: &IngestorResult, task: &Task) -> bool {
        let task_lower = task.description.to_lowercase();
        let uri_lower = doc.uri.to_lowercase();

        // Check URI relevance
        let uri_relevant = task.keywords().iter().any(|keyword| {
            uri_lower.contains(&keyword.to_lowercase())
        });

        // Check content relevance (simplified)
        let content_relevant = doc.segments.iter().any(|segment| {
            task.keywords().iter().any(|keyword| {
                segment.content.to_lowercase().contains(&keyword.to_lowercase())
            })
        });

        // Quality threshold
        let quality_ok = doc.quality_score >= self.enrichment_config.quality_threshold;

        uri_relevant || content_relevant && quality_ok
    }

    /// Extract and rank relevant segments from documents
    async fn extract_relevant_segments(
        &self,
        docs: &[IngestorResult],
        task: &Task,
    ) -> Result<Vec<RichSegment>, EnrichmentError> {
        let mut all_segments = Vec::new();

        for doc in docs {
            // Extract segments based on enabled types
            if self.enrichment_config.enable_slide_context {
                all_segments.extend(self.extract_slide_segments(doc, task));
            }
            if self.enrichment_config.enable_speech_context {
                all_segments.extend(self.extract_speech_segments(doc, task));
            }
            if self.enrichment_config.enable_diagram_context {
                all_segments.extend(self.extract_diagram_segments(doc, task));
            }
            if self.enrichment_config.enable_video_context {
                all_segments.extend(self.extract_video_segments(doc, task));
            }
        }

        // Rank and limit segments
        self.rank_and_limit_segments(all_segments, task)
    }

    /// Extract slide segments
    fn extract_slide_segments(&self, doc: &IngestorResult, task: &Task) -> Vec<RichSegment> {
        doc.segments.iter()
            .filter(|seg| matches!(seg.segment_type, SegmentType::Slide))
            .filter(|seg| self.is_segment_relevant(seg, task))
            .cloned()
            .collect()
    }

    /// Extract speech segments
    fn extract_speech_segments(&self, doc: &IngestorResult, task: &Task) -> Vec<RichSegment> {
        if let Some(speech_turns) = &doc.speech_turns {
            speech_turns.iter()
                .filter(|turn| self.is_speech_relevant(turn, task))
                .map(|turn| RichSegment {
                    id: format!("speech_{}", turn.start_time),
                    segment_type: SegmentType::Speech,
                    content: turn.text.clone(),
                    timestamp: Some(turn.start_time),
                    bbox: None,
                    ocr_confidence: Some(turn.confidence),
                    visual_embedding: None,
                    relevance_score: self.calculate_speech_relevance(turn, task),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Extract diagram segments
    fn extract_diagram_segments(&self, doc: &IngestorResult, task: &Task) -> Vec<RichSegment> {
        if let Some(diagram_data) = &doc.diagram_data {
            vec![RichSegment {
                id: format!("diagram_{}", doc.document_id),
                segment_type: SegmentType::Diagram,
                content: diagram_data.visual_summary.clone(),
                timestamp: None,
                bbox: None,
                ocr_confidence: None,
                visual_embedding: None, // Would be computed from diagram
                relevance_score: self.calculate_diagram_relevance(diagram_data, task),
            }]
        } else {
            Vec::new()
        }
    }

    /// Extract video scene segments
    fn extract_video_segments(&self, doc: &IngestorResult, task: &Task) -> Vec<RichSegment> {
        doc.segments.iter()
            .filter(|seg| matches!(seg.segment_type, SegmentType::Scene))
            .filter(|seg| self.is_segment_relevant(seg, task))
            .cloned()
            .collect()
    }

    /// Check if segment is relevant to task
    fn is_segment_relevant(&self, segment: &RichSegment, task: &Task) -> bool {
        let task_keywords = task.keywords();
        let content_lower = segment.content.to_lowercase();

        task_keywords.iter().any(|keyword| {
            content_lower.contains(&keyword.to_lowercase())
        })
    }

    /// Check if speech turn is relevant
    fn is_speech_relevant(&self, turn: &SpeechTurn, task: &Task) -> bool {
        let task_keywords = task.keywords();
        let text_lower = turn.text.to_lowercase();

        task_keywords.iter().any(|keyword| {
            text_lower.contains(&keyword.to_lowercase())
        })
    }

    /// Calculate speech relevance score
    fn calculate_speech_relevance(&self, turn: &SpeechTurn, task: &Task) -> f64 {
        let task_keywords = task.keywords();
        let text_lower = turn.text.to_lowercase();

        let keyword_matches = task_keywords.iter()
            .filter(|keyword| text_lower.contains(&keyword.to_lowercase()))
            .count();

        let base_score = keyword_matches as f64 / task_keywords.len() as f64;
        let confidence_boost = turn.confidence as f64 * 0.2;

        (base_score + confidence_boost).min(1.0)
    }

    /// Calculate diagram relevance score
    fn calculate_diagram_relevance(&self, diagram: &DiagramData, task: &Task) -> f64 {
        let task_keywords = task.keywords();
        let summary_lower = diagram.visual_summary.to_lowercase();

        let keyword_matches = task_keywords.iter()
            .filter(|keyword| summary_lower.contains(&keyword.to_lowercase()))
            .count();

        keyword_matches as f64 / task_keywords.len() as f64
    }

    /// Rank and limit segments by relevance
    fn rank_and_limit_segments(
        &self,
        mut segments: Vec<RichSegment>,
        task: &Task,
    ) -> Result<Vec<RichSegment>, EnrichmentError> {
        // Sort by relevance score descending
        segments.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Group by type and limit each group
        let mut limited_segments = Vec::new();
        let mut type_counts = HashMap::new();

        for segment in segments {
            let count = type_counts.entry(segment.segment_type.clone()).or_insert(0);
            if *count < self.enrichment_config.max_segments_per_type {
                limited_segments.push(segment);
                *count += 1;
            }
        }

        Ok(limited_segments)
    }

    /// Build enriched context bundle
    async fn build_enriched_bundle(
        &self,
        base_bundle: &ContextBundle,
        enriched_segments: &[RichSegment],
        budget: &ContextBudget,
    ) -> Result<ContextBundle, EnrichmentError> {
        // Estimate token usage
        let base_tokens = estimate_tokens(&base_bundle.working_memory);
        let enriched_content = self.format_enriched_segments(enriched_segments);
        let enriched_tokens = estimate_tokens(&enriched_content);

        let total_tokens = base_tokens + enriched_tokens;

        // Check if we need compression
        if total_tokens > budget.effective_capacity() {
            warn!("Enriched context exceeds budget ({} > {}), applying compression",
                  total_tokens, budget.effective_capacity());

            // Compress by prioritizing highest relevance segments
            let compressed = self.compress_enriched_content(
                &base_bundle.working_memory,
                enriched_segments,
                budget,
            );

            Ok(ContextBundle::new("".to_string(), vec![], vec![], vec![])
                .compressed(compressed, vec!["multimodal_enrichment".to_string()]))
        } else {
            // Combine base context with enriched segments
            let combined_content = format!(
                "{}\n\n--- Multimodal Context ---\n{}",
                base_bundle.working_memory,
                enriched_content
            );

            Ok(ContextBundle::new(
                combined_content,
                base_bundle.episodic_memory.clone(),
                base_bundle.semantic_memory.clone(),
                vec!["multimodal_enrichment".to_string()],
            ))
        }
    }

    /// Format enriched segments for context
    fn format_enriched_segments(&self, segments: &[RichSegment]) -> String {
        let mut content = String::new();

        for segment in segments {
            let segment_type = match segment.segment_type {
                SegmentType::Slide => " Slide",
                SegmentType::Speech => " Speech",
                SegmentType::Diagram => " Diagram",
                SegmentType::Scene => " Scene",
            };

            let timestamp_info = if let Some(ts) = segment.timestamp {
                format!(" ({:.1}s)", ts)
            } else {
                String::new()
            };

            content.push_str(&format!(
                "## {}{} (relevance: {:.2})\n{}\n\n",
                segment_type,
                timestamp_info,
                segment.relevance_score,
                segment.content
            ));
        }

        content
    }

    /// Compress enriched content when over budget
    fn compress_enriched_content(
        &self,
        base_content: &str,
        segments: &[RichSegment],
        budget: &ContextBudget,
    ) -> String {
        // Sort segments by relevance and include only top ones
        let mut sorted_segments: Vec<&RichSegment> = segments.iter()
            .filter(|s| s.relevance_score > 0.5) // Only high-relevance segments
            .collect();

        sorted_segments.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        let mut compressed = format!("{}\n\nKey multimodal insights:\n", base_content);
        let mut total_tokens = estimate_tokens(&compressed);

        for segment in sorted_segments {
            let segment_summary = format!(
                "- {}: {}...\n",
                match segment.segment_type {
                    SegmentType::Slide => "Slide",
                    SegmentType::Speech => "Speech",
                    SegmentType::Diagram => "Diagram",
                    SegmentType::Scene => "Scene",
                },
                segment.content.chars().take(100).collect::<String>()
            );

            let segment_tokens = estimate_tokens(&segment_summary);
            if total_tokens + segment_tokens <= budget.effective_capacity() {
                compressed.push_str(&segment_summary);
                total_tokens += segment_tokens;
            } else {
                break;
            }
        }

        compressed
    }

    /// Update ingestor cache with new results
    pub async fn update_cache(&self, results: Vec<IngestorResult>) -> Result<(), EnrichmentError> {
        let mut cache = self.ingestor_cache.write().await;

        for result in results {
            cache.insert(result.document_id.clone(), result);
        }

        debug!("Updated ingestor cache with {} documents", cache.len());
        Ok(())
    }

    /// Get enrichment statistics
    pub async fn get_stats(&self) -> EnrichmentStats {
        let cache = self.ingestor_cache.read().await;

        let total_docs = cache.len();
        let total_segments: usize = cache.values()
            .map(|doc| doc.segments.len())
            .sum();

        let docs_by_type: HashMap<String, usize> = cache.values()
            .fold(HashMap::new(), |mut acc, doc| {
                let key = match doc.segments.first() {
                    Some(seg) => format!("{:?}", seg.segment_type),
                    None => "unknown".to_string(),
                };
                *acc.entry(key).or_insert(0) += 1;
                acc
            });

        EnrichmentStats {
            total_documents: total_docs,
            total_segments,
            documents_by_type: docs_by_type,
            cache_size_bytes: estimate_cache_size(&cache),
        }
    }
}

/// Enrichment statistics
#[derive(Clone, Debug)]
pub struct EnrichmentStats {
    pub total_documents: usize,
    pub total_segments: usize,
    pub documents_by_type: HashMap<String, usize>,
    pub cache_size_bytes: usize,
}

/// Enrichment errors
#[derive(Debug, thiserror::Error)]
pub enum EnrichmentError {
    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Content extraction error: {0}")]
    ExtractionError(String),

    #[error("Relevance calculation error: {0}")]
    RelevanceError(String),

    #[error("Context building error: {0}")]
    ContextError(String),
}

/// Helper trait for extracting keywords from tasks
pub trait TaskKeywords {
    fn keywords(&self) -> Vec<String>;
}

impl TaskKeywords for Task {
    fn keywords(&self) -> Vec<String> {
        // Simple keyword extraction - split on whitespace and filter common words
        let common_words = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];

        self.description
            .split_whitespace()
            .filter(|word| word.len() > 3) // Skip short words
            .filter(|word| !common_words.contains(&word.to_lowercase().as_str()))
            .map(|word| word.to_lowercase())
            .collect()
    }
}

/// Estimate token count for text
fn estimate_tokens(text: &str) -> usize {
    // Rough approximation: 1 token per 4 characters
    (text.len() / 4).max(1)
}

/// Estimate cache size in bytes
fn estimate_cache_size(cache: &HashMap<String, IngestorResult>) -> usize {
    cache.values()
        .map(|doc| {
            std::mem::size_of::<IngestorResult>() +
            doc.segments.iter().map(|seg| std::mem::size_of::<RichSegment>() + seg.content.len()).sum::<usize>() +
            doc.speech_turns.as_ref().map_or(0, |turns| turns.len() * std::mem::size_of::<SpeechTurn>()) +
            doc.diagram_data.as_ref().map_or(0, |_| std::mem::size_of::<DiagramData>())
        })
        .sum()
}
