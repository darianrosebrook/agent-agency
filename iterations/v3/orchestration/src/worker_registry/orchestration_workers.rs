//! Worker Registry for Orchestration
//!
//! This module provides the worker registry trait and implementations
//! for managing worker endpoints and health reporting.

use async_trait::async_trait;
use anyhow::Result;

/// Registry for managing worker endpoints and health
#[async_trait]
pub trait WorkerRegistry: Send + Sync {
    /// Get worker endpoint for a given worker ID
    async fn get_worker_endpoint(&self, worker_id: &str) -> Result<String>;
    /// Report worker health status
    async fn report_worker_health(&self, worker_id: &str, healthy: bool) -> Result<()>;
}

/// Simple static worker registry implementation
pub struct StaticWorkerRegistry {
    default_endpoint: String,
}

impl StaticWorkerRegistry {
    pub fn new(default_endpoint: String) -> Self {
        Self { default_endpoint }
    }
}

#[async_trait]
impl WorkerRegistry for StaticWorkerRegistry {
    async fn get_worker_endpoint(&self, _worker_id: &str) -> Result<String> {
        Ok(self.default_endpoint.clone())
    }

    async fn report_worker_health(&self, _worker_id: &str, _healthy: bool) -> Result<()> {
        // Static registry doesn't track health
        Ok(())
    }
}
