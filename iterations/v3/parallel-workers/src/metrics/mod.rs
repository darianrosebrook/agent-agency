//! Metrics infrastructure for parallel worker learning system
//!
//! Provides tail-aware quantiles, schema validation, cardinality estimation,
//! and other production-grade metrics capabilities.

pub mod quantiles;
pub mod schema;
pub mod aggregates;
pub mod cardinality;

pub use quantiles::OnlineQuantiles;
pub use schema::{MetricEnvelope, MetricSchema};
pub use aggregates::Aggregates;
pub use cardinality::CardinalityEstimator;
