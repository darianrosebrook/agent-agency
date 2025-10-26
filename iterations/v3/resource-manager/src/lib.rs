//! Resource Manager Service
//!
//! Centralized resource lifecycle management, pooling, and adaptive allocation.
//! Extracted from apple-silicon monolith to provide focused resource services.

pub mod resource_management;
pub mod pools;
pub mod monitoring;

// Re-export common types
pub use agent_agency_common_types::*;

use serde::{Deserialize, Serialize};

// Re-export key functionality
pub use resource_management::*;
pub use pools::*;
pub use monitoring::*;

/// Main service struct for resource management
#[derive(Debug)]
pub struct ResourceManagerService {
    // Service configuration and state
    pools: std::collections::HashMap<String, Box<dyn ResourcePool>>,
    monitor: ResourceMonitor,
}

impl ResourceManagerService {
    /// Create a new resource manager service
    pub fn new() -> Self {
        Self {
            pools: std::collections::HashMap::new(),
            monitor: ResourceMonitor::new(),
        }
    }

    /// Register a resource pool
    pub fn register_pool(&mut self, name: String, pool: Box<dyn ResourcePool>) {
        self.pools.insert(name, pool);
    }

    /// Allocate resources from a specific pool
    pub async fn allocate_resources(
        &self,
        pool_name: &str,
        requirements: ResourceRequirements,
    ) -> Result<ResourceAllocation, ResourceError> {
        let pool = self.pools.get(pool_name)
            .ok_or_else(|| ResourceError::PoolNotFound(pool_name.to_string()))?;

        let allocation = pool.allocate(requirements).await?;

        // Update monitoring
        self.monitor.record_allocation(&allocation).await;

        Ok(allocation)
    }

    /// Release resources back to pool
    pub async fn release_resources(
        &self,
        pool_name: &str,
        allocation_id: &str,
    ) -> Result<(), ResourceError> {
        let pool = self.pools.get(pool_name)
            .ok_or_else(|| ResourceError::PoolNotFound(pool_name.to_string()))?;

        pool.release(allocation_id).await?;

        // Update monitoring
        self.monitor.record_release(allocation_id).await;

        Ok(())
    }

    /// Get resource utilization metrics
    pub async fn get_utilization(&self) -> ResourceUtilization {
        self.monitor.get_utilization().await
    }

    /// Adapt resource allocation based on usage patterns
    pub async fn adapt_allocation(&mut self) -> Result<(), ResourceError> {
        for (pool_name, pool) in &mut self.pools {
            pool.adapt().await?;
            tracing::info!("Adapted allocation for pool: {}", pool_name);
        }
        Ok(())
    }
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub memory_mb: Option<u64>,
    pub cpu_cores: Option<f32>,
    pub gpu_memory_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<u64>,
    pub priority: ResourcePriority,
}

/// Resource allocation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub pool_name: String,
    pub allocated_resources: ResourceRequirements,
    pub allocated_at: chrono::DateTime<chrono::Utc>,
    pub ttl_seconds: Option<u64>,
}

/// Resource priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourcePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub total_cpu_cores: f32,
    pub used_cpu_cores: f32,
    pub active_allocations: usize,
    pub pool_utilizations: std::collections::HashMap<String, PoolUtilization>,
}

/// Pool-specific utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolUtilization {
    pub pool_name: String,
    pub utilization_percent: f64,
    pub active_allocations: usize,
    pub total_capacity: usize,
}

/// Resource management errors
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Pool not found: {0}")]
    PoolNotFound(String),

    #[error("Resource allocation failed: {message}")]
    AllocationFailed { message: String },

    #[error("Resource release failed: {message}")]
    ReleaseFailed { message: String },

    #[error("Insufficient resources: {resource_type}")]
    InsufficientResources { resource_type: String },

    #[error("Resource timeout")]
    Timeout,

    #[error("Resource monitoring error: {message}")]
    MonitoringError { message: String },
}
