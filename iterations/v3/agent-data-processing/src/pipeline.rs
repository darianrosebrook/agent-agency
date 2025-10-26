//! Core data processing pipeline
//!
//! Defines the pluggable pipeline architecture where stages can be
//! composed and executed in sequence.

use crate::data_processing_types::*;
use crate::{DataProcessingResult, DataProcessingError};
use std::collections::HashMap;
use async_trait::async_trait;

/// Configuration for the data processing pipeline
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineConfig {
    pub max_concurrent_operations: usize,
    pub processing_timeout_seconds: u64,
    pub enable_circuit_breaker: bool,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 10,
            processing_timeout_seconds: 300, // 5 minutes
            enable_circuit_breaker: true,
            enable_metrics: true,
            enable_tracing: true,
        }
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

/// The main data processing pipeline
pub struct DataPipeline {
    config: PipelineConfig,
    stages: Vec<Box<dyn PipelineStage>>,
}

impl DataPipeline {
    /// Create a new pipeline with the given configuration
    pub async fn new(config: PipelineConfig) -> DataProcessingResult<Self> {
        let stages = Self::create_default_stages(&config).await?;

        Ok(Self {
            config,
            stages,
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
        let mut current_data = input.clone();
        let mut accumulated_metadata = HashMap::new();
        let start_time = std::time::Instant::now();

        for stage in &self.stages {
            match stage.process(current_data).await {
                Ok(output) => {
                    // Merge metadata from this stage
                    accumulated_metadata.extend(output.extracted_metadata);

                    // Create new input for next stage based on output
                    current_data = DataInput {
                        id: output.id,
                        source: DataSource::Stream(StreamSource {
                            stream_id: format!("stage_output_{}", stage.name()),
                            content_type: ContentType::Structured,
                        }),
                        content: DataContent::Structured(serde_json::to_value(&output.processed_content)
                            .unwrap_or(serde_json::Value::Null)),
                        metadata: accumulated_metadata.clone(),
                        processing_context: output.original_input.processing_context.clone(),
                    };
                }
                Err(e) => {
                    let _stats = ProcessingStats {
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        bytes_processed: 0,
                        entities_extracted: 0,
                        relationships_found: 0,
                        embeddings_generated: 0,
                        errors_encountered: vec![e.to_string()],
                    };

                    return Err(DataProcessingError::Other(format!(
                        "Pipeline stage '{}' failed: {}",
                        stage.name(),
                        e
                    )));
                }
            }
        }

        // Create final output
        let final_output = ProcessingOutput {
            id: current_data.id,
            original_input: input,
            processed_content: match current_data.content {
                DataContent::Structured(data) => serde_json::from_value(data)
                    .map_err(|e| DataProcessingError::Serialization(e))?,
                _ => ProcessedContent {
                    text_content: None,
                    structured_data: None,
                    embeddings: None,
                    entities: vec![],
                    relationships: vec![],
                    visual_elements: vec![],
                    audio_transcript: None,
                }
            },
            extracted_metadata: accumulated_metadata,
            processing_stats: ProcessingStats {
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                bytes_processed: 0, // Would be tracked per stage
                entities_extracted: 0,
                relationships_found: 0,
                embeddings_generated: 0,
                errors_encountered: vec![],
            },
            created_at: chrono::Utc::now(),
        };

        Ok(final_output)
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
