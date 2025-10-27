//! Core data processing pipeline
//!
//! Defines the pluggable pipeline architecture where stages can be
//! composed and executed in sequence. Now uses common-pipeline framework
//! for standardized patterns while maintaining domain-specific functionality.

use crate::data_processing_types::*;
use crate::{DataProcessingResult, DataProcessingError};
use system_configuration::{SequentialPipeline, SequentialPipelineConfig, PipelineStage as SystemPipelineStage, ExecutablePipeline};
use system_configuration::PipelineResult as SystemPipelineResult;
use std::default::Default;
use std::collections::HashMap;
use uuid::Uuid;

// Local pipeline stage trait for domain-specific stages
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

/// Adapter to make our domain PipelineStage compatible with system PipelineStage
pub struct PipelineStageAdapter {
    stage: Box<dyn PipelineStage>,
}

impl std::fmt::Debug for PipelineStageAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineStageAdapter")
            .field("stage", &"<dyn PipelineStage>")
            .finish()
    }
}

impl PipelineStageAdapter {
    pub fn new(stage: Box<dyn PipelineStage>) -> Self {
        Self { stage }
    }
}

#[async_trait]
impl SystemPipelineStage<DataInput, DataInput> for PipelineStageAdapter {
    fn name(&self) -> &str {
        self.stage.name()
    }

    async fn process(&self, input: DataInput) -> SystemPipelineResult<DataInput> {
        // Convert our result to system result
        match self.stage.process(input).await {
            Ok(output) => {
                // Convert ProcessingOutput back to DataInput for the next stage
                // This is a simplified conversion - in practice you might want more sophisticated logic
                Ok(DataInput {
                    id: ProcessingId::new(),
                    source: DataSource::Api(ApiSource {
                        endpoint: "pipeline_stage".to_string(),
                        method: "POST".to_string(),
                        parameters: HashMap::new(),
                    }),
                    content: DataContent::Text("processed_output".to_string()),
                    metadata: output.extracted_metadata,
                    processing_context: ProcessingContext {
                        request_id: uuid::Uuid::new_v4().to_string(),
                        user_id: None,
                        project_scope: None,
                        priority: ProcessingPriority::Normal,
                        deadline: None,
                        tags: vec![],
                    },
                })
            }
            Err(e) => Err(system_configuration::PipelineError::stage_error(format!("{}", e), "pipeline_stage")),
        }
    }
}

/// Composite stage that runs all data processing stages in sequence
pub struct DataProcessingCompositeStage {
    stages: Vec<Box<dyn PipelineStage>>,
}

#[async_trait]
impl PipelineStage for DataProcessingCompositeStage {
    fn name(&self) -> &'static str {
        "data_processing_pipeline"
    }

    async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
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
                            .map_err(|e| DataProcessingError::Operation(e.to_string()))?),
                        metadata: output.extracted_metadata.clone(),
                        processing_context: ProcessingContext {
                            request_id: input.processing_context.request_id.clone(),
                            user_id: input.processing_context.user_id.clone(),
                            project_scope: input.processing_context.project_scope.clone(),
                            priority: input.processing_context.priority.clone(),
                            deadline: input.processing_context.deadline.clone(),
                            tags: input.processing_context.tags.clone(),
                        },
                    };
                }
                Err(e) => {
                    return Err(DataProcessingError::Operation(format!("Stage {} failed: {}", stage.name(), e)));
                }
            }
        }

        let processing_time = start_time.elapsed();

        // Construct final processing output
        let entities_count = all_entities.len();
        let relationships_count = all_relationships.len();
        let embeddings_count = if final_embeddings.is_some() { 1 } else { 0 };

        let final_output = ProcessingOutput {
            id: ProcessingId::new(),
            original_input: input,
            processed_content: ProcessedContent {
                data: ProcessedContentData::Structured(serde_json::Value::Object(serde_json::Map::new())),
                content_type: ContentType::Structured,
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
                processing_time_ms: processing_time.as_millis() as u64,
                bytes_processed: 0, // TODO: calculate actual size
                entities_extracted: entities_count,
                relationships_found: relationships_count,
                embeddings_generated: embeddings_count,
                errors_encountered: vec![],
            },
            created_at: Utc::now(),
        };

        Ok(final_output)
    }

}

/// The main data processing pipeline
/// Now wraps SequentialPipeline with domain-specific functionality
pub struct DataPipeline {
    config: PipelineConfig,
    sequential_pipeline: Arc<SequentialPipeline<DataInput>>,
    /// Keep domain-specific stages for backward compatibility
    domain_stages: Vec<Box<dyn PipelineStage>>,
}

impl DataPipeline {
    /// Create a new pipeline with the given configuration
    pub async fn new(config: PipelineConfig) -> DataProcessingResult<Self> {
        let sequential_config = config.clone().into();

        // Create domain-specific stages
        let domain_stages = Self::create_default_stages(&config).await?;
        let domain_stages_for_composite = Self::create_default_stages(&config).await?;

        // Create a single composite stage that wraps all domain stages
        let composite_stage = DataProcessingCompositeStage {
            stages: domain_stages_for_composite,
        };

        let mut sequential_pipeline = SequentialPipeline::new(sequential_config);
        let adapter = PipelineStageAdapter::new(Box::new(composite_stage));
        sequential_pipeline.add_stage(Box::new(adapter)).await;

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
        stages.push(Box::new(crate::enrichment::DefaultEnrichmentStage::new(Default::default())) as Box<dyn PipelineStage>);

        // Add indexing stage
        stages.push(Box::new(crate::indexing::DefaultIndexingStage::new().await?) as Box<dyn PipelineStage>);

        // Add knowledge integration stage
        stages.push(Box::new(crate::knowledge::DefaultKnowledgeStage::new().await?) as Box<dyn PipelineStage>);

        // Add operations stage
        stages.push(Box::new(crate::operations::DefaultOperationsStage::new().await?) as Box<dyn PipelineStage>);

        Ok(stages)
    }

    /// Process data through all pipeline stages
    pub async fn process(&self, mut input: DataInput) -> DataProcessingResult<ProcessingOutput> {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut total_entities = 0;
        let mut total_relationships = 0;
        let mut total_embeddings = 0;
        let mut total_bytes = match &input.content {
            DataContent::Text(text) => text.len(),
            DataContent::Binary(data) => data.len(),
            DataContent::Structured(_) => 0, // JSON size is hard to estimate
            DataContent::File(_) => 0, // File size is hard to estimate without reading
        };

        // Process through each stage sequentially
        for stage in &self.domain_stages {
            match stage.process(input.clone()).await {
                Ok(output) => {
                    // Update input for next stage
                    input = DataInput {
                        id: output.id.clone(),
                        source: output.original_input.source.clone(),
                        content: match &output.processed_content.data {
                            ProcessedContentData::Text(text) => DataContent::Text(text.clone()),
                            ProcessedContentData::Binary(data) => DataContent::Binary(data.clone()),
                            ProcessedContentData::Structured(value) => DataContent::Structured(value.clone()),
                        },
                        metadata: output.extracted_metadata.clone(),
                        processing_context: output.original_input.processing_context.clone(),
                    };

                    // Aggregate statistics
                    total_entities += output.processed_content.entities.len();
                    total_relationships += output.processed_content.relationships.len();
                    if let Some(embeddings) = &output.processed_content.embeddings {
                        total_embeddings += embeddings.len();
                    }
                    total_bytes += match &output.processed_content.data {
                        ProcessedContentData::Text(text) => text.len(),
                        ProcessedContentData::Binary(data) => data.len(),
                        ProcessedContentData::Structured(_) => 0,
                    };
                }
                Err(e) => {
                    errors.push(format!("Stage {} failed: {}", stage.name(), e));
                    // Continue processing with original input
                }
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Create final output
        let final_output = ProcessingOutput {
            id: input.id.clone(),
            original_input: input.clone(),
            processed_content: ProcessedContent {
                data: match &input.content {
                    DataContent::Text(text) => ProcessedContentData::Text(text.clone()),
                    DataContent::Binary(data) => ProcessedContentData::Binary(data.clone()),
                    DataContent::Structured(value) => ProcessedContentData::Structured(value.clone()),
                    DataContent::File(_) => ProcessedContentData::Text("File content".to_string()), // Placeholder
                },
                content_type: ContentType::Text, // Default content type
                text_content: None,
                structured_data: None,
                embeddings: None,
                entities: vec![], // Will be populated by individual stages
                relationships: vec![], // Will be populated by individual stages
                visual_elements: vec![],
                audio_transcript: None,
            },
            extracted_metadata: HashMap::new(),
            processing_stats: ProcessingStats {
                processing_time_ms: processing_time,
                bytes_processed: total_bytes as u64,
                entities_extracted: total_entities,
                relationships_found: total_relationships,
                embeddings_generated: total_embeddings,
                errors_encountered: errors,
            },
            created_at: chrono::Utc::now(),
        };

        Ok(final_output)
    }

    /// Query processed data across all stages
    pub async fn query(&self, query: DataQuery) -> DataProcessingResult<Vec<RetrievedData>> {
        let mut all_results = Vec::new();

        // Query each stage that supports querying
        for stage in &self.domain_stages {
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
        let mut total_processed = 0;
        let mut total_processing_time = 0.0;
        let mut total_errors = 0;
        let mut active_operations = 0;

        // Aggregate stats from all stages
        for stage in &self.domain_stages {
            match stage.get_stats().await {
                Ok(stats) => {
                    total_processed += stats.bytes_processed;
                    total_processing_time += stats.processing_time_ms as f64;
                    total_errors += stats.errors_encountered.len();
                    active_operations += 1; // Each stage represents an active operation
                }
                Err(_) => {
                    // If we can't get stats from a stage, count it as an error
                    total_errors += 1;
                }
            }
        }

        // Calculate average processing time
        let avg_processing_time_ms = if active_operations > 0 {
            total_processing_time / active_operations as f64
        } else {
            0.0
        };

        // Calculate error rate
        let error_rate = if total_processed > 0 {
            total_errors as f64 / total_processed as f64
        } else {
            0.0
        };

        Ok(PipelineStats {
            total_processed,
            active_operations,
            queue_depth: self.domain_stages.len(), // Use stage count as queue depth proxy
            error_rate,
            avg_processing_time_ms,
        })
    }

    /// Add a custom pipeline stage
    pub fn add_stage(&mut self, stage: Box<dyn PipelineStage>) {
        self.domain_stages.push(stage);
    }

    /// Remove a pipeline stage by name
    pub fn remove_stage(&mut self, name: &str) {
        self.domain_stages.retain(|stage| stage.name() != name);
    }
}


/// Create a custom pipeline with specific stages
pub fn create_custom_pipeline(
    config: PipelineConfig,
    stages: Vec<Box<dyn PipelineStage>>,
) -> DataPipeline {
    // Create a simple sequential pipeline configuration
    let sequential_config = SequentialPipelineConfig {
        base: system_configuration::PipelineConfig {
            name: "Custom data processing pipeline".to_string(),
            description: Some("Custom data processing pipeline".to_string()),
            enable_metrics: true,
            enable_tracing: true,
            timeout: std::time::Duration::from_secs(30),
            max_concurrent_operations: 1,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_recovery_timeout: std::time::Duration::from_secs(30),
            enable_health_monitoring: true,
            health_check_interval: std::time::Duration::from_secs(10),
        },
        max_stage_retries: 3,
        continue_on_stage_failure: true,
        stage_timeout: std::time::Duration::from_secs(30),
        enable_stage_caching: false,
    };
    
    let sequential_pipeline = SequentialPipeline::new(sequential_config);
    
    DataPipeline {
        config,
        sequential_pipeline: Arc::new(sequential_pipeline),
        domain_stages: stages, // Use the provided stages directly
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
                    data: ProcessedContentData::Text("processed".to_string()),
                    content_type: ContentType::Text,
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
