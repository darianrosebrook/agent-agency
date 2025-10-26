//! Resource pool implementations
//!
//! Concrete implementations of different resource pooling strategies.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    ResourcePool, ResourceRequirements, ResourceAllocation, ResourceError,
};

/// Memory pool implementation
pub struct MemoryPool {
    name: String,
    total_memory_mb: u64,
    allocated_memory_mb: Arc<RwLock<u64>>,
    allocations: Arc<RwLock<HashMap<String, ResourceAllocation>>>,
    max_allocations: usize,
}

impl MemoryPool {
    pub fn new(name: String, total_memory_mb: u64, max_allocations: usize) -> Self {
        Self {
            name,
            total_memory_mb,
            allocated_memory_mb: Arc::new(RwLock::new(0)),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            max_allocations,
        }
    }
}

#[async_trait]
impl ResourcePool for MemoryPool {
    async fn allocate(&self, requirements: ResourceRequirements) -> Result<ResourceAllocation, ResourceError> {
        let memory_needed = requirements.memory_mb
            .ok_or_else(|| ResourceError::AllocationFailed {
                message: "Memory requirement not specified".to_string(),
            })?;

        let mut allocated_memory = self.allocated_memory_mb.write().await;
        let mut allocations = self.allocations.write().await;

        // Check if we have enough memory
        if *allocated_memory + memory_needed > self.total_memory_mb {
            return Err(ResourceError::InsufficientResources {
                resource_type: "memory".to_string(),
            });
        }

        // Check allocation limit
        if allocations.len() >= self.max_allocations {
            return Err(ResourceError::AllocationFailed {
                message: "Maximum allocations reached".to_string(),
            });
        }

        // Allocate memory
        *allocated_memory += memory_needed;

        let allocation_id = format!("mem_{}_{}", self.name, chrono::Utc::now().timestamp_millis());

        let allocation = ResourceAllocation {
            allocation_id: allocation_id.clone(),
            pool_name: self.name.clone(),
            allocated_resources: requirements.clone(),
            allocated_at: chrono::Utc::now(),
            ttl_seconds: None, // Could be configurable
        };

        allocations.insert(allocation_id, allocation.clone());

        Ok(allocation)
    }

    async fn release(&self, allocation_id: &str) -> Result<(), ResourceError> {
        let mut allocations = self.allocations.write().await;
        let mut allocated_memory = self.allocated_memory_mb.write().await;

        let allocation = allocations.remove(allocation_id)
            .ok_or_else(|| ResourceError::ReleaseFailed {
                message: format!("Allocation not found: {}", allocation_id),
            })?;

        // Free the memory
        if let Some(memory_mb) = allocation.allocated_resources.memory_mb {
            *allocated_memory = allocated_memory.saturating_sub(memory_mb);
        }

        Ok(())
    }

    async fn utilization(&self) -> f64 {
        let allocated_memory = self.allocated_memory_mb.read().await;
        if self.total_memory_mb == 0 {
            0.0
        } else {
            (*allocated_memory as f64 / self.total_memory_mb as f64) * 100.0
        }
    }

    async fn adapt(&mut self) -> Result<(), ResourceError> {
        let allocated_memory = self.allocated_memory_mb.read().await;
        let utilization = if self.total_memory_mb > 0 {
            *allocated_memory as f64 / self.total_memory_mb as f64
        } else {
            0.0
        };

        // Check for over-allocation
        if *allocated_memory > self.total_memory_mb {
            return Err(ResourceError::AllocationFailed {
                message: "Pool over-allocated during adaptation".to_string(),
            });
        }

        // Intelligent resource rebalancing based on usage patterns
        if utilization > 0.9 {
            // High utilization: implement memory compaction/defagmentation
            tracing::info!("Memory pool '{}' at {:.1}% utilization, performing adaptation", self.name, utilization * 100.0);

            // TODO: Implement memory pool adaptation and optimization with acceptance criteria:
            // - [ ] Implement memory defragmentation and compaction algorithms
            // - [ ] Add allocation rebalancing across different memory regions
            // - [ ] Integrate with garbage collection for managed memory cleanup
            // - [ ] Implement predictive scaling based on allocation patterns
            // - [ ] Add memory pressure monitoring and proactive optimization
        } else if utilization > 0.7 {
            // Moderate utilization: monitor and prepare for scaling
            tracing::debug!("Memory pool '{}' at {:.1}% utilization, monitoring for scaling", self.name, utilization * 100.0);
        }

        // Predictive scaling based on allocation trends would go here
        // This could track allocation rates and predict future needs

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn capacity(&self) -> usize {
        self.max_allocations
    }

    fn active_count(&self) -> usize {
        // Track actual number of active allocations by counting entries in the allocations map
        self.allocations.try_read().map(|allocs| allocs.len()).unwrap_or(0)
    }

    fn total_memory_mb(&self) -> u64 {
        self.total_memory_mb
    }

    async fn allocated_memory_mb(&self) -> u64 {
        *self.allocated_memory_mb.read().await
    }
}

impl std::fmt::Debug for MemoryPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryPool")
            .field("name", &self.name)
            .field("total_memory_mb", &self.total_memory_mb)
            .field("max_allocations", &self.max_allocations)
            .finish()
    }
}

/// CPU resource pool
pub struct CpuPool {
    name: String,
    total_cores: f32,
    allocated_cores: Arc<RwLock<f32>>,
    allocations: Arc<RwLock<HashMap<String, ResourceAllocation>>>,
}

impl CpuPool {
    pub fn new(name: String, total_cores: f32) -> Self {
        Self {
            name,
            total_cores,
            allocated_cores: Arc::new(RwLock::new(0.0)),
            allocations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ResourcePool for CpuPool {
    async fn allocate(&self, requirements: ResourceRequirements) -> Result<ResourceAllocation, ResourceError> {
        let cores_needed = requirements.cpu_cores
            .ok_or_else(|| ResourceError::AllocationFailed {
                message: "CPU requirement not specified".to_string(),
            })?;

        let mut allocated_cores = self.allocated_cores.write().await;

        // Check if we have enough cores
        if *allocated_cores + cores_needed > self.total_cores {
            return Err(ResourceError::InsufficientResources {
                resource_type: "cpu".to_string(),
            });
        }

        // Allocate cores
        *allocated_cores += cores_needed;

        let allocation_id = format!("cpu_{}_{}", self.name, chrono::Utc::now().timestamp_millis());

        let allocation = ResourceAllocation {
            allocation_id: allocation_id.clone(),
            pool_name: self.name.clone(),
            allocated_resources: requirements.clone(),
            allocated_at: chrono::Utc::now(),
            ttl_seconds: None,
        };

        let mut allocations = self.allocations.write().await;
        allocations.insert(allocation_id, allocation.clone());

        Ok(allocation)
    }

    async fn release(&self, allocation_id: &str) -> Result<(), ResourceError> {
        let mut allocations = self.allocations.write().await;
        let mut allocated_cores = self.allocated_cores.write().await;

        let allocation = allocations.remove(allocation_id)
            .ok_or_else(|| ResourceError::ReleaseFailed {
                message: format!("Allocation not found: {}", allocation_id),
            })?;

        // Free the cores
        if let Some(cpu_cores) = allocation.allocated_resources.cpu_cores {
            *allocated_cores = (*allocated_cores - cpu_cores).max(0.0);
        }

        Ok(())
    }

    async fn utilization(&self) -> f64 {
        let allocated_cores = self.allocated_cores.read().await;
        if self.total_cores == 0.0 {
            0.0
        } else {
            (*allocated_cores as f64 / self.total_cores as f64) * 100.0
        }
    }

    async fn adapt(&mut self) -> Result<(), ResourceError> {
        // CPU pool adaptation logic
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn capacity(&self) -> usize {
        self.total_cores as usize
    }

    fn active_count(&self) -> usize {
        // Track actual number of active allocations by counting entries in the allocations map
        self.allocations.try_read().map(|allocs| allocs.len()).unwrap_or(0)
    }

    fn total_cpu_cores(&self) -> f32 {
        self.total_cores
    }

    async fn allocated_cpu_cores(&self) -> f32 {
        *self.allocated_cores.read().await
    }
}

impl std::fmt::Debug for CpuPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpuPool")
            .field("name", &self.name)
            .field("total_cores", &self.total_cores)
            .finish()
    }
}
