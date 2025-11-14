//! Metrics infrastructure for parallel worker learning system
//!
//! Provides tail-aware quantiles, schema validation, cardinality estimation,
//! and other production-grade metrics capabilities.

pub mod aggregates;
pub mod cardinality;
pub mod quantiles;
pub mod schema;

pub use aggregates::Aggregates;
pub use cardinality::CardinalityEstimator;
pub use quantiles::OnlineQuantiles;
pub use schema::{MetricEnvelope, MetricSchema};
