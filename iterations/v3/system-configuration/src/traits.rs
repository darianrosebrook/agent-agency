//! Core pipeline traits and abstractions
//!
//! This module defines the fundamental traits that all pipeline implementations
//! should follow, ensuring consistency across the codebase.

use crate::cache::{CacheConfig, CacheStats};
use crate::PipelineResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Core trait for pipeline stages that can process data
#[async_trait]
pub trait PipelineStage<Input, Output>: Send + Sync + Debug {
    /// Get the name of this pipeline stage
    fn name(&self) -> &str;

    /// Process input data and return output
    async fn process(&self, input: Input) -> PipelineResult<Output>;

    /// Check if this stage can handle the given input
    fn can_handle(&self, _input: &Input) -> bool {
        // Default implementation accepts all inputs
        true
    }

    /// Get stage-specific configuration
    fn config(&self) -> Option<serde_json::Value> {
        None
    }

    /// Validate stage configuration
    fn validate(&self) -> PipelineResult<()> {
        Ok(())
    }
}

/// Trait for pipelines that can be executed
#[async_trait]
pub trait ExecutablePipeline<Input, Output>: Send + Sync + Debug {
    /// Execute the pipeline with given input
    async fn execute(&self, input: Input) -> PipelineResult<Output>;

    /// Get pipeline metrics
    fn metrics(&self) -> PipelineResult<serde_json::Value>;

    /// Get pipeline health status
    fn health_status(&self) -> PipelineResult<PipelineHealth>;

    /// Reset pipeline state (if applicable)
    async fn reset(&mut self) -> PipelineResult<()> {
        Ok(())
    }
}

/// Trait for pipelines that support stages
pub trait StagedPipeline<Input, Output>: ExecutablePipeline<Input, Output> {
    /// Add a stage to the pipeline
    fn add_stage(&mut self, stage: Box<dyn PipelineStage<Input, Output>>);

    /// Remove a stage by name
    fn remove_stage(&mut self, name: &str) -> PipelineResult<()>;

    /// Get all stage names
    fn stage_names(&self) -> Vec<String>;

    /// Get stage count
    fn stage_count(&self) -> usize;
}

/// Trait for pipelines that can be configured
pub trait ConfigurablePipeline<C>: ExecutablePipeline<(), ()> {
    /// Update pipeline configuration
    fn update_config(&mut self, config: C) -> PipelineResult<()>;

    /// Get current configuration
    fn current_config(&self) -> &C;

    /// Validate configuration
    fn validate_config(&self, config: &C) -> PipelineResult<()>;
}

/// Trait for pipelines that support caching
#[async_trait]
pub trait CacheablePipeline<Input, Output>: ExecutablePipeline<Input, Output> {
    /// Enable caching with given configuration
    async fn enable_caching(&mut self, config: CacheConfig) -> PipelineResult<()>;

    /// Disable caching
    async fn disable_caching(&mut self) -> PipelineResult<()>;

    /// Clear cache
    async fn clear_cache(&mut self) -> PipelineResult<()>;

    /// Get cache statistics
    fn cache_stats(&self) -> PipelineResult<CacheStats>;
}

/// Health status of a pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PipelineHealth {
    /// Pipeline is healthy and ready
    Healthy,
    /// Pipeline has minor issues but is operational
    Degraded,
    /// Pipeline is not operational
    Unhealthy,
    /// Pipeline is starting up
    Starting,
    /// Pipeline is shutting down
    Stopping,
}

impl std::fmt::Display for PipelineHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineHealth::Healthy => write!(f, "healthy"),
            PipelineHealth::Degraded => write!(f, "degraded"),
            PipelineHealth::Unhealthy => write!(f, "unhealthy"),
            PipelineHealth::Starting => write!(f, "starting"),
            PipelineHealth::Stopping => write!(f, "stopping"),
        }
    }
}

/// Trait for components that can report their status
#[async_trait::async_trait]
pub trait StatusReporter {
    /// Get current component status
    fn status(&self) -> crate::types::ComponentStatus;

    /// Get detailed status information
    fn status_details(&self) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({ "status": "ok" }))
    }
}

/// Trait for components that can be validated
#[async_trait::async_trait]
pub trait Validatable {
    /// Validate the component's state
    fn validate(&self) -> anyhow::Result<crate::types::ValidationResult>;
}

/// Trait for components that provide metrics
#[async_trait::async_trait]
pub trait MetricsProvider {
    /// Get current metrics
    fn get_metrics(&self) -> anyhow::Result<serde_json::Value>;

    /// Get a metrics snapshot
    fn get_metrics_snapshot(&self) -> anyhow::Result<crate::pattern_types::MetricsSnapshot>;
}

/// Trait for components that can be health-checked
#[async_trait::async_trait]
pub trait HealthCheckable {
    /// Perform a health check
    async fn health_check(&self) -> anyhow::Result<crate::pattern_types::HealthStatus>;
}
