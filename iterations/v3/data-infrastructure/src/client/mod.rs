//! Database client module
//!
//! Production-hardened database client with connection pooling,
//! circuit breaker pattern, monitoring, and resilience features.

pub mod orchestrator;

// Re-export main client
pub use orchestrator::DatabaseClient;
