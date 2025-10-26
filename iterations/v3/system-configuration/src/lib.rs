//! Common Pipeline Patterns - Unified pipeline abstractions for Agent Agency V3
//!
//! This crate provides shared pipeline patterns and abstractions that can be used
//! across different pipeline implementations throughout the codebase. It eliminates
//! duplication by providing:
//!
//! ## Core Abstractions
//!
//! - **Pipeline Stage**: Pluggable processing stages with async execution
//! - **Pipeline Metrics**: Standardized metrics collection across all pipelines
//! - **Pipeline Config**: Common configuration patterns for pipeline setup
//! - **Pipeline Error**: Unified error handling for pipeline operations
//! - **Pipeline Cache**: Shared caching abstractions for performance optimization
//!
//! ## Pipeline Types
//!
//! - **Sequential Pipeline**: Stages execute in order, passing data between stages
//! - **Parallel Pipeline**: Stages execute concurrently with result aggregation
//! - **Streaming Pipeline**: Continuous processing with backpressure handling
//! - **Validation Pipeline**: Multi-stage validation with error accumulation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use system_configuration::{PipelineStage, SequentialPipeline, SequentialPipelineConfig};
//!
//! // Create a pipeline with stages
//! let config = SequentialPipelineConfig::default();
//! let mut pipeline: SequentialPipeline<String> = SequentialPipeline::new(config);
//!
//! // Add processing stages (implement PipelineStage trait for your stages)
//! // pipeline.add_stage(Box::new(MyProcessingStage::new())).await;
//!
//! // Execute pipeline
//! // let result = pipeline.execute("input data".to_string()).await;
//! ```
//!
//! @author @darianrosebrook

pub mod traits;
pub mod metrics;
pub mod config;
pub mod error;
pub mod cache;
pub mod sequential;
pub mod parallel;
pub mod streaming;
pub mod validation;
pub mod geometry;
pub mod types;
pub mod pattern_types;

// Re-export main types
pub use traits::*;
pub use metrics::*;
pub use config::*;
pub use error::*;
pub use cache::*;
pub use sequential::*;
pub use parallel::*;
pub use streaming::*;
pub use validation::*;
