//! Core resource management functionality
//!
//! Adaptive resource allocation, lifecycle management, and optimization.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    ResourceRequirements, ResourceAllocation, ResourceError,
    ResourceUtilization,
};

/// Trait for resource pools
#[async_trait]
pub trait ResourcePool: Send + Sync + std::fmt::Debug {
    /// Allocate resources from this pool
    async fn allocate(&self, requirements: ResourceRequirements) -> Result<ResourceAllocation, ResourceError>;

    /// Release resources back to this pool
    async fn release(&self, allocation_id: &str) -> Result<(), ResourceError>;

    /// Get pool utilization
    async fn utilization(&self) -> f64;

    /// Adapt pool allocation based on usage patterns
    async fn adapt(&mut self) -> Result<(), ResourceError>;

    /// Get pool name
    fn name(&self) -> &str;

    /// Get pool capacity
    fn capacity(&self) -> usize;

    /// Get active allocation count
    fn active_count(&self) -> usize;

    /// Get total memory capacity in MB
    fn total_memory_mb(&self) -> u64 { 0 }

    /// Get total CPU cores capacity
    fn total_cpu_cores(&self) -> f32 { 0.0 }

    /// Get currently allocated memory in MB
    async fn allocated_memory_mb(&self) -> u64 { 0 }

    /// Get currently allocated CPU cores
    async fn allocated_cpu_cores(&self) -> f32 { 0.0 }
}

/// Adaptive resource manager
pub struct AdaptiveResourceManager {
    pools: Arc<RwLock<std::collections::HashMap<String, Box<dyn ResourcePool>>>>,
    adaptation_interval: std::time::Duration,
}

impl AdaptiveResourceManager {
    pub fn new(adaptation_interval: std::time::Duration) -> Self {
        Self {
            pools: Arc::new(RwLock::new(std::collections::HashMap::new())),
            adaptation_interval,
        }
    }

    /// Register a resource pool
    pub async fn register_pool(&self, name: String, pool: Box<dyn ResourcePool>) {
        let mut pools = self.pools.write().await;
        pools.insert(name, pool);
    }

    /// Allocate resources with adaptive pool selection
    pub async fn allocate_adaptive(
        &self,
        requirements: ResourceRequirements,
    ) -> Result<ResourceAllocation, ResourceError> {
        let pools = self.pools.read().await;

        // Find the best pool for the requirements
        let best_pool = self.select_best_pool(&pools, &requirements).await?;

        best_pool.allocate(requirements).await
    }

    /// Get overall resource utilization
    pub async fn get_overall_utilization(&self) -> ResourceUtilization {
        let pools = self.pools.read().await;
        let mut total_memory = 0u64;
        let mut used_memory = 0u64;
        let mut total_cpu = 0.0f32;
        let mut used_cpu = 0.0f32;
        let mut active_allocations = 0usize;
        let mut pool_utilizations = std::collections::HashMap::new();

        for (name, pool) in pools.iter() {
            let utilization = pool.utilization().await;

            pool_utilizations.insert(name.clone(), crate::PoolUtilization {
                pool_name: name.clone(),
                utilization_percent: utilization,
                active_allocations: pool.active_count(),
                total_capacity: pool.capacity(),
            });

            active_allocations += pool.active_count();

            // Aggregate memory and CPU metrics from all pools
            total_memory += pool.total_memory_mb();
            used_memory += pool.allocated_memory_mb().await;
            total_cpu += pool.total_cpu_cores();
            used_cpu += pool.allocated_cpu_cores().await;
        }

        ResourceUtilization {
            total_memory_mb: total_memory,
            used_memory_mb: used_memory,
            total_cpu_cores: total_cpu,
            used_cpu_cores: used_cpu,
            active_allocations,
            pool_utilizations,
        }
    }

    /// Start adaptive management loop
    pub fn start_adaptive_loop(&self) -> tokio::task::JoinHandle<()> {
        let pools = Arc::clone(&self.pools);
        let adaptation_interval = self.adaptation_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(adaptation_interval);

            loop {
                interval.tick().await;

                // Adapt all pools
                let mut pools_write = pools.write().await;
                for (name, pool) in pools_write.iter_mut() {
                    if let Err(e) = pool.adapt().await {
                        tracing::warn!("Failed to adapt pool {}: {:?}", name, e);
                    }
                }
            }
        })
    }

    async fn select_best_pool<'a>(
        &self,
        pools: &'a std::collections::HashMap<String, Box<dyn ResourcePool>>,
        _requirements: &ResourceRequirements,
    ) -> Result<&'a Box<dyn ResourcePool>, ResourceError> {
        // Simple selection logic - prefer pools with lowest utilization
        // More sophisticated selection would consider resource requirements
        let mut best_pool: Option<&Box<dyn ResourcePool>> = None;
        let mut best_utilization = 100.0;

        for (_name, pool) in pools.iter() {
            let utilization = pool.utilization().await;
            if utilization < best_utilization {
                best_utilization = utilization;
                best_pool = Some(pool);
            }
        }

        best_pool.ok_or_else(|| ResourceError::AllocationFailed {
            message: "No suitable pool found".to_string(),
        })
    }
}

/// Resource allocation strategy
#[derive(Debug, Clone)]
pub enum AllocationStrategy {
    /// First-fit: Use first available pool
    FirstFit,
    /// Best-fit: Choose pool with best resource match
    BestFit,
    /// Load-balanced: Distribute across pools
    LoadBalanced,
    /// Priority-based: Consider resource priority
    PriorityBased,
}

/// Resource lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Available,
    Allocated,
    Releasing,
    Failed,
}
