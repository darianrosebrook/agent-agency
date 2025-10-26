//! Sequential pipeline implementation
//!
//! This module provides a sequential pipeline that executes stages in order,
//! passing the output of each stage as input to the next stage.

use crate::{
    traits::{ExecutablePipeline, PipelineStage, StagedPipeline},
    config::SequentialPipelineConfig,
    error::{PipelineError, PipelineResult},
    metrics::PipelineMetrics,
};
use async_trait::async_trait;
use futures::TryFutureExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Sequential pipeline that executes stages in order
#[derive(Debug)]
pub struct SequentialPipeline<Input, Output> {
    config: SequentialPipelineConfig,
    stages: Arc<RwLock<Vec<Box<dyn PipelineStage<Input, Output>>>>>,
    metrics: PipelineMetrics,
}

impl<Input, Output> SequentialPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static + std::fmt::Debug,
    Output: Clone + Send + Sync + 'static + std::fmt::Debug,
{
    /// Create a new sequential pipeline
    pub fn new(config: SequentialPipelineConfig) -> Self {
        Self {
            config,
            stages: Arc::new(RwLock::new(Vec::new())),
            metrics: PipelineMetrics::new(),
        }
    }

    /// Add a stage to the pipeline
    pub async fn add_stage(&mut self, stage: Box<dyn PipelineStage<Input, Output>>) {
        let mut stages = self.stages.write().await;
        stages.push(stage);
        debug!("Added stage to sequential pipeline, total stages: {}", stages.len());
    }

    /// Execute the pipeline sequentially
    async fn execute_internal(&self, input: Input) -> PipelineResult<Output> {
        let stages = self.stages.read().await;
        let mut current_input = input;
        let mut final_output = None;

        for (index, stage) in stages.iter().enumerate() {
            let stage_name = stage.name();
            let start_time = std::time::Instant::now();

            debug!("Executing stage {}: {}", index, stage_name);

            match tokio::time::timeout(
                self.config.stage_timeout,
                stage.process(current_input)
            ).await {
                Ok(Ok(output)) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    self.metrics.record_stage_execution(&stage_name, duration, true).await;
                    debug!("Stage {} completed successfully in {}ms", stage_name, duration);

                    // Use output as input for next stage
                    current_input = self.extract_next_input(&output)?;
                    final_output = Some(output);
                }
                Ok(Err(e)) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    self.metrics.record_stage_execution(&stage_name, duration, false).await;
                    self.metrics.record_error(&format!("stage_{}", stage_name)).await;

                    warn!("Stage {} failed: {}", stage_name, e);

                    if !self.config.continue_on_stage_failure {
                        return Err(PipelineError::stage_error(stage_name, e.to_string()));
                    }
                }
                Err(_) => {
                    self.metrics.record_stage_execution(&stage_name, self.config.stage_timeout.as_millis() as u64, false).await;
                    self.metrics.record_error("stage_timeout").await;

                    warn!("Stage {} timed out after {:?}", stage_name, self.config.stage_timeout);

                    if !self.config.continue_on_stage_failure {
                        return Err(PipelineError::timeout(format!("Stage {} timed out", stage_name)));
                    }
                }
            }
        }

        final_output.ok_or_else(|| PipelineError::Execution("No stages produced output".to_string()))
    }

    /// Extract input for the next stage from current output
    fn extract_next_input(&self, output: &Output) -> PipelineResult<Input> {
        // Default implementation assumes Output can be converted to Input
        // In practice, this might need custom logic for each pipeline type
        // For now, we'll use a simple approach that works for basic cases
        todo!("Implement extract_next_input for specific pipeline types")
    }
}

#[async_trait]
impl<Input, Output> ExecutablePipeline<Input, Output> for SequentialPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static + std::fmt::Debug,
    Output: Clone + Send + Sync + 'static + std::fmt::Debug,
{
    async fn execute(&self, input: Input) -> PipelineResult<Output> {
        let start_time = std::time::Instant::now();

        info!("Starting sequential pipeline execution with {} stages",
              self.stages.read().await.len());

        let result = self.execute_internal(input).await;
        let duration = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();

        self.metrics.record_execution(duration, success).await;

        match &result {
            Ok(_) => {
                info!("Sequential pipeline completed successfully in {}ms", duration);
            }
            Err(e) => {
                self.metrics.record_error("pipeline_execution").await;
                warn!("Sequential pipeline failed after {}ms: {}", duration, e);
            }
        }

        result
    }

    fn metrics(&self) -> PipelineResult<serde_json::Value> {
        // This is a simplified implementation - in practice you'd want async access
        futures::executor::block_on(async {
            self.metrics.to_json().await
        }).map_err(|e| PipelineError::Metrics(e.to_string()))
    }

    fn health_status(&self) -> PipelineResult<crate::PipelineHealth> {
        // Basic health check - check if stages exist and are configured
        futures::executor::block_on(async {
            let stages = self.stages.read().await;
            if stages.is_empty() {
                return Ok(crate::PipelineHealth::Unhealthy);
            }

            // Check if all stages can validate themselves
            for stage in stages.iter() {
                if let Err(e) = stage.validate() {
                    warn!("Stage {} failed validation: {}", stage.name(), e);
                    return Ok(crate::PipelineHealth::Degraded);
                }
            }

            Ok(crate::PipelineHealth::Healthy)
        })
    }
}

impl<Input, Output> StagedPipeline<Input, Output> for SequentialPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static + std::fmt::Debug,
    Output: Clone + Send + Sync + 'static + std::fmt::Debug,
{
    fn add_stage(&mut self, stage: Box<dyn PipelineStage<Input, Output>>) {
        futures::executor::block_on(async {
            SequentialPipeline::add_stage(self, stage).await
        })
    }

    fn remove_stage(&mut self, name: &str) -> PipelineResult<()> {
        futures::executor::block_on(async {
            let mut stages = self.stages.write().await;
            let initial_len = stages.len();

            stages.retain(|stage| stage.name() != name);

            if stages.len() == initial_len {
                return Err(PipelineError::Execution(format!("Stage '{}' not found", name)));
            }

            debug!("Removed stage '{}', remaining stages: {}", name, stages.len());
            Ok(())
        })
    }

    fn stage_names(&self) -> Vec<String> {
        futures::executor::block_on(async {
            let stages = self.stages.read().await;
            stages.iter().map(|stage| stage.name().to_string()).collect()
        })
    }

    fn stage_count(&self) -> usize {
        futures::executor::block_on(async {
            self.stages.read().await.len()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PipelineConfig;

    // Mock stage for testing
    struct MockStage {
        name: String,
        should_fail: bool,
    }

    impl MockStage {
        fn new(name: impl Into<String>, should_fail: bool) -> Self {
            Self {
                name: name.into(),
                should_fail,
            }
        }
    }

    #[async_trait]
    impl PipelineStage<String, String> for MockStage {
        fn name(&self) -> &str {
            &self.name
        }

        async fn process(&self, input: String) -> PipelineResult<String> {
            if self.should_fail {
                return Err(PipelineError::Execution("Mock stage failed".to_string()));
            }
            Ok(format!("{}-processed", input))
        }
    }

    #[tokio::test]
    async fn test_sequential_pipeline_success() {
        let config = SequentialPipelineConfig::default();
        let mut pipeline = SequentialPipeline::new(config);

        let stage1 = Box::new(MockStage::new("stage1", false));
        let stage2 = Box::new(MockStage::new("stage2", false));

        pipeline.add_stage(stage1).await;
        pipeline.add_stage(stage2).await;

        let result = pipeline.execute("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-processed-processed");
    }

    #[tokio::test]
    async fn test_sequential_pipeline_failure() {
        let config = SequentialPipelineConfig {
            continue_on_stage_failure: false,
            ..Default::default()
        };
        let mut pipeline = SequentialPipeline::new(config);

        let stage1 = Box::new(MockStage::new("stage1", false));
        let stage2 = Box::new(MockStage::new("stage2", true)); // This will fail

        pipeline.add_stage(stage1).await;
        pipeline.add_stage(stage2).await;

        let result = pipeline.execute("test".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stage_management() {
        let config = SequentialPipelineConfig::default();
        let mut pipeline = SequentialPipeline::new(config);

        let stage1 = Box::new(MockStage::new("stage1", false));
        let stage2 = Box::new(MockStage::new("stage2", false));

        pipeline.add_stage(stage1).await;
        pipeline.add_stage(stage2).await;

        assert_eq!(pipeline.stage_count(), 2);
        assert_eq!(pipeline.stage_names(), vec!["stage1", "stage2"]);

        pipeline.remove_stage("stage1").unwrap();
        assert_eq!(pipeline.stage_count(), 1);
        assert_eq!(pipeline.stage_names(), vec!["stage2"]);
    }
}
