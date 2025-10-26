//! Core data processing pipeline
//!
//! Defines the pluggable pipeline architecture where stages can be
//! composed and executed in sequence. Now uses common-pipeline framework
//! for standardized patterns while maintaining domain-specific functionality.

use crate::data_processing_types::*;
use crate::{DataProcessingResult, DataProcessingError};
use common_pipeline::{
    SequentialPipeline, SequentialPipelineConfig, PipelineStage as CommonPipelineStage,
    ExecutablePipeline, PipelineResult as CommonPipelineResult, PipelineHealth,
};
use std::collections::HashMap;
use async_trait::async_trait;
use std::sync::Arc;

/// Configuration for the data processing pipeline
/// Now wraps SequentialPipelineConfig with domain-specific settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineConfig {
    /// Base sequential pipeline configuration
    #[serde(flatten)]
    pub base: SequentialPipelineConfig,
    /// Domain-specific configuration
    pub max_concurrent_operations: usize,
    pub enable_domain_specific_features: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            base: SequentialPipelineConfig::default(),
            max_concurrent_operations: 10,
            enable_domain_specific_features: true,
        }
    }
}

impl From<PipelineConfig> for SequentialPipelineConfig {
    fn from(config: PipelineConfig) -> Self {
        config.base
    }
}

/// Result from pipeline processing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineResult {
    pub success: bool,
    pub output: Option<ProcessingOutput>,
    pub errors: Vec<String>,
    pub stats: ProcessingStats,
}

/// Composite stage that runs all data processing stages in sequence
pub struct DataProcessingCompositeStage {
    stages: Vec<Box<dyn PipelineStage>>,
}

#[async_trait]
impl CommonPipelineStage<DataInput, ProcessingOutput> for DataProcessingCompositeStage {
    fn name(&self) -> &str {
        "data_processing_pipeline"
    }

    async fn process(&self, input: DataInput) -> common_pipeline::PipelineResult<ProcessingOutput> {
        use crate::data_processing_types::*;
        use chrono::Utc;

        let mut current_data = input.clone();
        let mut accumulated_metadata = HashMap::new();
        let mut all_entities = Vec::new();
        let mut all_relationships = Vec::new();
        let mut all_visual_elements = Vec::new();
        let mut final_text_content = None;
        let mut final_structured_data = None;
        let mut final_embeddings = None;
        let mut final_audio_transcript = None;

        let start_time = std::time::Instant::now();

        for stage in &self.stages {
            match stage.process(current_data).await {
                Ok(output) => {
                    // Merge metadata from this stage
                    accumulated_metadata.extend(output.extracted_metadata.clone());

                    // Accumulate content from different stages
                    if let Some(text) = &output.processed_content.text_content {
                        final_text_content = Some(text.clone());
                    }
                    if let Some(structured) = &output.processed_content.structured_data {
                        final_structured_data = Some(structured.clone());
                    }
                    if let Some(embeddings) = &output.processed_content.embeddings {
                        final_embeddings = Some(embeddings.clone());
                    }
                    if let Some(transcript) = &output.processed_content.audio_transcript {
                        final_audio_transcript = Some(transcript.clone());
                    }

                    // Accumulate entities, relationships, and visual elements
                    all_entities.extend(output.processed_content.entities.clone());
                    all_relationships.extend(output.processed_content.relationships.clone());
                    all_visual_elements.extend(output.processed_content.visual_elements.clone());

                    // Create new input for next stage based on output
                    current_data = DataInput {
                        id: output.id.clone(),
                        source: DataSource::Stream(StreamSource {
                            stream_id: format!("stage_output_{}", stage.name()),
                            content_type: ContentType::Structured,
                        }),
                        content: DataContent::Structured(serde_json::to_value(&output.processed_content)
                            .map_err(|e| common_pipeline::PipelineError::Serialization(e.to_string()))?),
                        metadata: output.extracted_metadata.clone(),
                        priority: input.priority,
                        processing_options: input.processing_options.clone(),
                    };
                }
                Err(e) => {
                    return Err(common_pipeline::PipelineError::StageError {
                        stage: stage.name().to_string(),
                        message: e.to_string(),
                    });
                }
            }
        }

        let processing_time = start_time.elapsed();

        // Construct final processing output
        let final_output = ProcessingOutput {
            id: ProcessingId::new(),
            original_input: input,
            processed_content: ProcessedContent {
                text_content: final_text_content,
                structured_data: final_structured_data,
                embeddings: final_embeddings,
                entities: all_entities,
                relationships: all_relationships,
                visual_elements: all_visual_elements,
                audio_transcript: final_audio_transcript,
            },
            extracted_metadata: accumulated_metadata,
            processing_stats: ProcessingStats {
                total_processing_time_ms: processing_time.as_millis() as u64,
                stages_processed: self.stages.len() as u32,
                data_size_bytes: 0, // TODO: calculate actual size
                memory_usage_bytes: 0, // TODO: track memory usage
                cpu_time_ms: processing_time.as_millis() as u64,
            },
            created_at: Utc::now(),
        };

        Ok(final_output)
    }

    fn can_handle(&self, _input: &DataInput) -> bool {
        // Domain-specific validation can be added here
        true
    }
}

/// The main data processing pipeline
/// Now wraps SequentialPipeline with domain-specific functionality
pub struct DataPipeline {
    config: PipelineConfig,
    sequential_pipeline: Arc<SequentialPipeline<DataInput, ProcessingOutput>>,
    /// Keep domain-specific stages for backward compatibility
    domain_stages: Vec<Box<dyn PipelineStage>>,
}

impl DataPipeline {
    /// Create a new pipeline with the given configuration
    pub async fn new(config: PipelineConfig) -> DataProcessingResult<Self> {
        let sequential_config = config.clone().into();

        // Create domain-specific stages
        let domain_stages = Self::create_default_stages(&config).await?;

        // Create a single composite stage that wraps all domain stages
        let composite_stage = DataProcessingCompositeStage {
            stages: domain_stages.clone(),
        };

        let mut sequential_pipeline = SequentialPipeline::new(sequential_config);
        sequential_pipeline.add_stage(Box::new(composite_stage)).await;

        Ok(Self {
            config,
            sequential_pipeline: Arc::new(sequential_pipeline),
            domain_stages,
        })
    }

    /// Create the default set of pipeline stages
    async fn create_default_stages(_config: &PipelineConfig) -> DataProcessingResult<Vec<Box<dyn PipelineStage>>> {
        let mut stages = Vec::new();

        // Add ingestion stage
        stages.push(Box::new(crate::ingestion::DefaultIngestionStage::new().await?) as Box<dyn PipelineStage>);

        // Add enrichment stage
        stages.push(Box::new(crate::enrichment::DefaultEnrichmentStage::new().await?) as Box<dyn PipelineStage>);

        // Add indexing stage
        stages.push(Box::new(crate::indexing::DefaultIndexingStage::new().await?) as Box<dyn PipelineStage>);

        // Add knowledge integration stage
        stages.push(Box::new(crate::knowledge::DefaultKnowledgeStage::new().await?) as Box<dyn PipelineStage>);

        // Add operations stage
        stages.push(Box::new(crate::operations::DefaultOperationsStage::new().await?) as Box<dyn PipelineStage>);

        Ok(stages)
    }

    /// Process data through all pipeline stages
    pub async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
        // Delegate to the sequential pipeline
        match self.sequential_pipeline.execute(input).await {
            Ok(output) => Ok(output),
            Err(e) => Err(crate::DataProcessingError::Other(format!("Pipeline execution failed: {}", e))),
        }

    /// Query processed data across all stages
    pub async fn query(&self, query: DataQuery) -> DataProcessingResult<Vec<RetrievedData>> {
        let mut all_results = Vec::new();

        // Query each stage that supports querying
        for stage in &self.stages {
            if let Ok(results) = stage.query(&query).await {
                all_results.extend(results);
            }
        }

        // Sort by relevance and deduplicate
        all_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.dedup_by_key(|r| r.id.clone());

        // Limit results
        all_results.truncate(query.limit);

        Ok(all_results)
    }

    /// Get pipeline statistics
    pub async fn get_stats(&self) -> DataProcessingResult<PipelineStats> {
        // This would aggregate stats from all stages
        // For now, return placeholder stats
        Ok(PipelineStats {
            total_processed: 0,
            active_operations: 0,
            queue_depth: 0,
            error_rate: 0.0,
            avg_processing_time_ms: 0.0,
        })
    }

    /// Add a custom pipeline stage
    pub fn add_stage(&mut self, stage: Box<dyn PipelineStage>) {
        self.stages.push(stage);
    }

    /// Remove a pipeline stage by name
    pub fn remove_stage(&mut self, name: &str) {
        self.stages.retain(|stage| stage.name() != name);
    }
}

/// Trait for pipeline stages that can process data
#[async_trait]
pub trait PipelineStage: Send + Sync {
    /// Get the name of this stage
    fn name(&self) -> &'static str;

    /// Process data through this stage
    async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput>;

    /// Query data from this stage (optional)
    async fn query(&self, _query: &DataQuery) -> DataProcessingResult<Vec<RetrievedData>> {
        Ok(vec![]) // Default implementation returns no results
    }

    /// Get statistics for this stage
    async fn get_stats(&self) -> DataProcessingResult<ProcessingStats> {
        Ok(ProcessingStats {
            processing_time_ms: 0,
            bytes_processed: 0,
            entities_extracted: 0,
            relationships_found: 0,
            embeddings_generated: 0,
            errors_encountered: vec![],
        })
    }
}

/// Create a custom pipeline with specific stages
pub fn create_custom_pipeline(
    config: PipelineConfig,
    stages: Vec<Box<dyn PipelineStage>>,
) -> DataPipeline {
    DataPipeline {
        config,
        stages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock pipeline stage for testing
    struct MockStage {
        name: &'static str,
    }

    #[async_trait]
    impl PipelineStage for MockStage {
        fn name(&self) -> &'static str {
            self.name
        }

        async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
            Ok(ProcessingOutput {
                id: input.id.clone(),
                original_input: input,
                processed_content: ProcessedContent {
                    text_content: Some("processed".to_string()),
                    structured_data: None,
                    embeddings: None,
                    entities: vec![],
                    relationships: vec![],
                    visual_elements: vec![],
                    audio_transcript: None,
                },
                extracted_metadata: HashMap::new(),
                processing_stats: ProcessingStats {
                    processing_time_ms: 10,
                    bytes_processed: 100,
                    entities_extracted: 0,
                    relationships_found: 0,
                    embeddings_generated: 0,
                    errors_encountered: vec![],
                },
                created_at: chrono::Utc::now(),
            })
        }
    }

    #[tokio::test]
    async fn test_pipeline_creation() {
        let config = PipelineConfig::default();
        let pipeline = DataPipeline::new(config).await;
        assert!(pipeline.is_ok());
    }

    #[tokio::test]
    async fn test_custom_pipeline() {
        let config = PipelineConfig::default();
        let stages = vec![
            Box::new(MockStage { name: "mock1" }) as Box<dyn PipelineStage>,
            Box::new(MockStage { name: "mock2" }) as Box<dyn PipelineStage>,
        ];

        let pipeline = create_custom_pipeline(config, stages);
        assert_eq!(pipeline.stages.len(), 2);
    }

    #[tokio::test]
    async fn test_pipeline_stage_removal() {
        let config = PipelineConfig::default();
        let mut pipeline = DataPipeline::new(config).await.unwrap();

        let initial_count = pipeline.stages.len();
        pipeline.remove_stage("nonexistent");
        assert_eq!(pipeline.stages.len(), initial_count);
    }
}
