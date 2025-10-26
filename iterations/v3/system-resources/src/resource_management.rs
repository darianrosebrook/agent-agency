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
            // Mock utilization calculation - would be implemented based on actual pool metrics
            let utilization = pool.utilization().await;
            let capacity = pool.capacity() as f64;
            let _used_capacity = (capacity * utilization / 100.0) as usize;

            pool_utilizations.insert(name.clone(), crate::PoolUtilization {
                pool_name: name.clone(),
                utilization_percent: utilization,
                active_allocations: pool.active_count(),
                total_capacity: pool.capacity(),
            });

            active_allocations += pool.active_count();
            // Mock memory/CPU tracking - would be implemented based on actual resource types
            total_memory += 8192; // Mock 8GB per pool
            used_memory += (8192.0 * utilization / 100.0) as u64;
            total_cpu += 4.0; // Mock 4 cores per pool
            used_cpu += 4.0 * utilization as f32 / 100.0;
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
