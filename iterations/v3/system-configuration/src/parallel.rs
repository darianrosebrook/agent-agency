//! Parallel pipeline implementation
//!
//! This module provides a parallel pipeline that can execute stages concurrently
//! and aggregate their results according to configurable strategies.

use crate::{
    traits::{ExecutablePipeline, PipelineStage, StagedPipeline},
    config::{ParallelPipelineConfig, AggregationStrategy},
    error::{PipelineError, PipelineResult},
    metrics::PipelineMetrics,
};
use async_trait::async_trait;
use futures::{future::join_all, TryFutureExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Parallel pipeline that executes stages concurrently
#[derive(Debug)]
pub struct ParallelPipeline<Input, Output> {
    config: ParallelPipelineConfig,
    stages: Arc<RwLock<Vec<Box<dyn PipelineStage<Input, Output>>>>>,
    metrics: PipelineMetrics,
}

impl<Input, Output> ParallelPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static + std::fmt::Debug,
    Output: Clone + Send + Sync + 'static + std::fmt::Debug,
{
    /// Create a new parallel pipeline
    pub fn new(config: ParallelPipelineConfig) -> Self {
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
        debug!("Added stage to parallel pipeline, total stages: {}", stages.len());
    }

    /// Execute stages in parallel
    async fn execute_parallel(&self, input: Input) -> PipelineResult<Vec<Output>> {
        let stages = self.stages.read().await;

        if stages.is_empty() {
            return Err(PipelineError::Execution("No stages configured".to_string()));
        }

        // TODO: Implement true parallel execution
        // Current implementation executes sequentially due to trait object lifetime issues
        // with tokio::spawn and Send requirements. This needs a major redesign.

        let mut successful_results = Vec::new();
        let mut failures = Vec::new();

        for (index, stage) in stages.iter().enumerate() {
            let start_time = std::time::Instant::now();
            let stage_name = stage.name();

            match tokio::time::timeout(
                self.config.parallel_timeout,
                stage.process(input.clone())
            ).await {
                Ok(Ok(output)) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    self.metrics.record_stage_execution(stage_name, duration, true).await;
                    debug!("Stage {} (index {}) completed successfully in {}ms",
                           stage_name, index, duration);
                    successful_results.push(output);
                }
                Ok(Err(e)) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    self.metrics.record_stage_execution(stage_name, duration, false).await;
                    self.metrics.record_error(&format!("stage_{}", stage_name)).await;
                    warn!("Stage {} (index {}) failed: {}", stage_name, index, e);
                    failures.push(e);
                }
                Err(_) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    self.metrics.record_stage_execution(stage_name, duration, false).await;
                    self.metrics.record_error("stage_timeout").await;
                    warn!("Stage {} (index {}) timed out", stage_name, index);
                    failures.push(PipelineError::timeout(format!("Stage {} timed out", stage_name)));
                }
            }
        }

        // Apply aggregation strategy
        self.apply_aggregation_strategy(successful_results, failures).await
    }

    /// Apply the configured aggregation strategy
    async fn apply_aggregation_strategy(
        &self,
        successful_results: Vec<Output>,
        failures: Vec<PipelineError>
    ) -> PipelineResult<Vec<Output>> {
        match self.config.aggregation_strategy {
            AggregationStrategy::AllRequired => {
                if !failures.is_empty() {
                    return Err(PipelineError::Execution(
                        format!("{} stages failed, all required to succeed", failures.len())
                    ));
                }
                Ok(successful_results)
            }

            AggregationStrategy::AnyRequired => {
                if successful_results.is_empty() {
                    return Err(PipelineError::Execution("All stages failed".to_string()));
                }
                Ok(successful_results)
            }

            AggregationStrategy::MajorityRequired => {
                let total_stages = successful_results.len() + failures.len();
                let majority_threshold = (total_stages + 1) / 2;

                if successful_results.len() < majority_threshold {
                    return Err(PipelineError::Execution(
                        format!("Only {}/{} stages succeeded, majority required",
                                successful_results.len(), total_stages)
                    ));
                }
                Ok(successful_results)
            }

            AggregationStrategy::Weighted => {
                // TODO: Implement weighted aggregation strategy with acceptance criteria:
                // - [ ] Define weight assignment mechanism for different pipeline stages
                // - [ ] Implement weighted scoring algorithm for result aggregation
                // - [ ] Add configurable weight thresholds for success/failure decisions
                // - [ ] Support partial success scenarios based on weighted importance
                // - [ ] Provide weighted aggregation metrics and performance analysis
                // For now, treat as AllRequired - could be extended with weights
                if !failures.is_empty() {
                    return Err(PipelineError::Execution("Weighted aggregation not implemented".to_string()));
                }
                Ok(successful_results)
            }
        }
    }
}

#[async_trait]
impl<Input, Output> ExecutablePipeline<Input, Vec<Output>> for ParallelPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static + std::fmt::Debug,
    Output: Clone + Send + Sync + 'static + std::fmt::Debug,
{
    async fn execute(&self, input: Input) -> PipelineResult<Vec<Output>> {
        let start_time = std::time::Instant::now();

        info!("Starting parallel pipeline execution with {} stages",
              self.stages.read().await.len());

        let result = self.execute_parallel(input).await;
        let duration = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();

        self.metrics.record_execution(duration, success).await;

        match &result {
            Ok(results) => {
                info!("Parallel pipeline completed successfully in {}ms with {} results",
                      duration, results.len());
            }
            Err(e) => {
                self.metrics.record_error("pipeline_execution").await;
                warn!("Parallel pipeline failed after {}ms: {}", duration, e);
            }
        }

        result
    }

    fn metrics(&self) -> PipelineResult<serde_json::Value> {
        futures::executor::block_on(async {
            self.metrics.to_json().await
        }).map_err(|e| PipelineError::Metrics(e.to_string()))
    }

    fn health_status(&self) -> PipelineResult<crate::PipelineHealth> {
        futures::executor::block_on(async {
            let stages = self.stages.read().await;
            if stages.is_empty() {
                return Ok(crate::PipelineHealth::Unhealthy);
            }

            // Check concurrent execution limits
            if stages.len() > self.config.max_parallel_stages {
                return Ok(crate::PipelineHealth::Degraded);
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

// Note: ParallelPipeline does not implement StagedPipeline due to trait design limitations.
// The trait expects stages to produce the same type as the pipeline output (Vec<Output>),
// but parallel stages should produce individual Output items that get collected.
// This may be addressed in a future trait redesign.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PipelineConfig;

    // Mock stage for testing
    #[derive(Debug)]
    struct MockStage {
        name: String,
        delay_ms: u64,
        should_fail: bool,
    }

    impl MockStage {
        fn new(name: impl Into<String>, delay_ms: u64, should_fail: bool) -> Self {
            Self {
                name: name.into(),
                delay_ms,
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
            // Simulate processing delay
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;

            if self.should_fail {
                return Err(PipelineError::Execution(format!("Mock stage {} failed", self.name)));
            }

            Ok(format!("{}-{}", input, self.name))
        }
    }

    #[tokio::test]
    async fn test_parallel_pipeline_all_success() {
        let config = ParallelPipelineConfig {
            aggregation_strategy: AggregationStrategy::AllRequired,
            ..Default::default()
        };
        let mut pipeline = ParallelPipeline::new(config);

        let stage1 = Box::new(MockStage::new("stage1", 10, false));
        let stage2 = Box::new(MockStage::new("stage2", 10, false));
        let stage3 = Box::new(MockStage::new("stage3", 10, false));

        pipeline.add_stage(stage1).await;
        pipeline.add_stage(stage2).await;
        pipeline.add_stage(stage3).await;

        let result = pipeline.execute("test".to_string()).await;
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 3);
        // Results may come back in any order due to parallel execution
        assert!(outputs.contains(&"test-stage1".to_string()));
        assert!(outputs.contains(&"test-stage2".to_string()));
        assert!(outputs.contains(&"test-stage3".to_string()));
    }

    #[tokio::test]
    async fn test_parallel_pipeline_any_required() {
        let config = ParallelPipelineConfig {
            aggregation_strategy: AggregationStrategy::AnyRequired,
            ..Default::default()
        };
        let mut pipeline = ParallelPipeline::new(config);

        let stage1 = Box::new(MockStage::new("stage1", 10, true)); // fails
        let stage2 = Box::new(MockStage::new("stage2", 10, false)); // succeeds

        pipeline.add_stage(stage1).await;
        pipeline.add_stage(stage2).await;

        let result = pipeline.execute("test".to_string()).await;
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], "test-stage2");
    }

    #[tokio::test]
    async fn test_parallel_pipeline_all_required_failure() {
        let config = ParallelPipelineConfig {
            aggregation_strategy: AggregationStrategy::AllRequired,
            ..Default::default()
        };
        let mut pipeline = ParallelPipeline::new(config);

        let stage1 = Box::new(MockStage::new("stage1", 10, false));
        let stage2 = Box::new(MockStage::new("stage2", 10, true)); // fails

        pipeline.add_stage(stage1).await;
        pipeline.add_stage(stage2).await;

        let result = pipeline.execute("test".to_string()).await;
        assert!(result.is_err());
    }
}
