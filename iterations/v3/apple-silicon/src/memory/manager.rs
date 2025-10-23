//! Memory manager implementation
//!
//! This module contains the core MemoryManager struct and its implementation
//! for monitoring and controlling memory usage, including multi-tenant support.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use anyhow::Result;
use chrono;
#[cfg(feature = "torch")]
use tch::{Tensor, Device, Kind, Cuda};
use candle_core::{Tensor as CandleTensor, DType};
use sysinfo::System;
use uuid::Uuid;

use super::compression::ModelUsageStats;

/// Tenant identifier
pub type TenantId = Uuid;

/// Tenant configuration for memory allocation
#[derive(Debug, Clone)]
pub struct TenantMemoryConfig {
    pub tenant_id: TenantId,
    pub max_memory_mb: u64,
    pub priority: TenantPriority,
    pub guaranteed_memory_mb: u64,
    pub burst_limit_mb: Option<u64>,
    pub isolation_level: IsolationLevel,
}

/// Tenant priority for resource allocation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TenantPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Memory isolation levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsolationLevel {
    /// No isolation - tenants can share resources freely
    None,
    /// Soft isolation - tenants have quotas but can borrow from others
    Soft,
    /// Hard isolation - tenants are strictly limited to their quotas
    Hard,
}

/// Tenant memory usage tracking
#[derive(Debug, Clone)]
pub struct TenantMemoryUsage {
    pub tenant_id: TenantId,
    pub allocated_memory_mb: u64,
    pub used_memory_mb: u64,
    pub model_count: usize,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub memory_pressure: crate::MemoryPressure,
}

/// Multi-tenant memory manager
#[derive(Debug)]
pub struct MultiTenantMemoryManager {
    config: crate::MemoryConfig,
    tenant_configs: Arc<RwLock<HashMap<TenantId, TenantMemoryConfig>>>,
    tenant_usage: Arc<RwLock<HashMap<TenantId, TenantMemoryUsage>>>,
    global_status: Arc<RwLock<crate::MemoryStatus>>,
    monitoring_active: Arc<RwLock<bool>>,
    model_usage: Arc<RwLock<HashMap<String, ModelUsageStats>>>,
    model_inactivity_threshold_secs: u64,
}

/// Memory allocation request
#[derive(Debug, Clone)]
pub struct MemoryAllocationRequest {
    pub tenant_id: TenantId,
    pub requested_memory_mb: u64,
    pub model_name: String,
    pub allocation_type: AllocationType,
}

/// Memory allocation response
#[derive(Debug)]
pub enum MemoryAllocationResponse {
    Granted { allocation_id: String },
    Denied { reason: String, available_memory_mb: u64 },
    Queued { estimated_wait_time_secs: u64 },
}

/// Allocation types for different use cases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationType {
    /// Model loading - persistent allocation
    ModelLoad,
    /// Inference execution - temporary allocation
    Inference,
    /// Caching - may be evicted under pressure
    Cache,
    /// Background processing - low priority
    Background,
}

/// Legacy single-tenant memory manager for backward compatibility
/// Memory manager for monitoring and controlling memory usage
#[derive(Debug)]
pub struct MemoryManager {
    config: crate::MemoryConfig, // Will be imported from parent crate
    current_status: Arc<RwLock<crate::MemoryStatus>>, // Will be imported from parent crate
    monitoring_active: Arc<RwLock<bool>>,
    model_usage: Arc<RwLock<HashMap<String, ModelUsageStats>>>,
    model_inactivity_threshold_secs: u64,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: crate::MemoryConfig) -> Self {
        let total_memory = config.max_memory_mb as u64;
        Self {
            config,
            current_status: Arc::new(RwLock::new(crate::MemoryStatus {
                total_memory_mb: total_memory,
                used_memory_mb: 0,
                available_memory_mb: total_memory,
                memory_pressure: crate::MemoryPressure::Normal,
                cache_size_mb: 0,
                model_memory_mb: 0,
                timestamp: chrono::Utc::now(),
            })),
            monitoring_active: Arc::new(RwLock::new(false)),
            model_usage: Arc::new(RwLock::new(HashMap::new())),
            model_inactivity_threshold_secs: 300, // 5 minutes default
        }
    }

    /// Start memory monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut active = self.monitoring_active.write().await;
        *active = true;

        info!("Memory monitoring started");
        Ok(())
    }

    /// Stop memory monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut active = self.monitoring_active.write().await;
        *active = false;

        info!("Memory monitoring stopped");
        Ok(())
    }

    /// Get current memory status
    pub async fn get_memory_status(&self) -> crate::MemoryStatus {
        let status = self.current_status.read().await;
        status.clone()
    }

    /// Update memory status
    pub async fn update_memory_status(
        &self,
        used_memory_mb: u64,
        cache_size_mb: u64,
        model_memory_mb: u64,
    ) -> Result<()> {
        let mut status = self.current_status.write().await;
        status.used_memory_mb = used_memory_mb;
        status.cache_size_mb = cache_size_mb;
        status.model_memory_mb = model_memory_mb;
        status.available_memory_mb = status.total_memory_mb - used_memory_mb;
        status.timestamp = chrono::Utc::now();

        // Update memory pressure
        let usage_percent = (used_memory_mb as f32 / status.total_memory_mb as f32) * 100.0;
        status.memory_pressure = match () {
            _ if usage_percent < 70.0 => crate::MemoryPressure::Normal,
            _ if usage_percent <= 75.0 => crate::MemoryPressure::Warning,
            _ if usage_percent < 85.0 => crate::MemoryPressure::Medium,
            _ if usage_percent < 90.0 => crate::MemoryPressure::High,
            _ => crate::MemoryPressure::Critical,
        };

        if usage_percent > 90.0 {
            warn!(
                "High memory usage: {:.1}% ({}/{} MB)",
                usage_percent, used_memory_mb, status.total_memory_mb
            );
        }

        Ok(())
    }

    /// Check if memory cleanup is needed
    pub async fn needs_cleanup(&self) -> bool {
        let status = self.current_status.read().await;
        let usage_percent = (status.used_memory_mb as f32 / status.total_memory_mb as f32) * 100.0;
        usage_percent > self.config.cleanup_threshold_percent as f32
    }

    /// Perform memory cleanup
    pub async fn cleanup_memory(&self) -> Result<u64> {
        let mut status = self.current_status.write().await;

        // 1. Cache cleanup - clean expired/unused entries
        let cache_cleaned = self.perform_cache_cleanup(&mut status).await?;

        // 2. Memory defragmentation - optimize memory layout
        let defrag_cleaned = self.perform_memory_defragmentation(&mut status).await?;

        // 3. Model memory optimization - unload unused models
        let model_cleaned = self.perform_model_memory_optimization(&mut status).await?;

        // 4. Buffer cleanup - free unused GPU/ANE buffers
        let buffer_cleaned = self.perform_buffer_cleanup(&mut status).await?;

        let total_cleaned = cache_cleaned + defrag_cleaned + model_cleaned + buffer_cleaned;

        info!("Memory cleanup completed: {} MB freed", total_cleaned);
        Ok(total_cleaned)
    }

    async fn perform_cache_cleanup(&self, status: &mut crate::MemoryStatus) -> Result<u64> {
        // TODO: Implement Apple Silicon unified memory cache cleanup
        // - Integrate with macOS memory pressure notifications
        // - Implement intelligent cache eviction based on access patterns
        // - Support compressed memory management for unified memory
        // - Add memory pressure-based automatic cleanup triggers
        // - Implement cache size optimization and memory reclamation
        Ok(0)
    }

    async fn perform_memory_defragmentation(&self, status: &mut crate::MemoryStatus) -> Result<u64> {
        // TODO: Implement Apple Silicon memory defragmentation
        // - Work with macOS virtual memory system for defragmentation
        // - Implement memory compaction strategies for unified memory
        // - Add fragmentation monitoring and threshold-based cleanup
        // - Support memory layout optimization for performance
        // - Implement defragmentation scheduling based on memory pressure
        Ok(0)
    }

    async fn perform_model_memory_optimization(&self, status: &mut crate::MemoryStatus) -> Result<u64> {
        // TODO: Implement ML model memory optimization for Apple Silicon
        // - Optimize memory layout for Neural Engine and GPU acceleration
        // - Implement model weight quantization and compression
        // - Support memory-mapped model loading for large models
        // - Add model memory usage profiling and optimization
        // - Implement memory-efficient model inference strategies
        Ok(0)
    }

    async fn perform_buffer_cleanup(&self, status: &mut crate::MemoryStatus) -> Result<u64> {
        // TODO: Implement buffer memory cleanup and optimization
        // - Track and clean up unused GPU and Neural Engine buffers
        // - Implement buffer pooling and reuse strategies
        // - Add buffer memory fragmentation detection and cleanup
        // - Support buffer memory compression and optimization
        // - Implement buffer lifecycle management and automatic cleanup
        Ok(0)
    }

    /// Allocate tensor memory with torch optimization
    #[cfg(feature = "torch")]
    pub fn allocate_tensor(&self, shape: &[i64], dtype: Kind, device: Device) -> Result<Tensor> {
        let tensor = Tensor::zeros(shape, (dtype, device));
        info!("Allocated tensor with shape {:?}, dtype: {:?}, device: {:?}", shape, dtype, device);
        Ok(tensor)
    }

    /// Move tensor to optimal device for memory efficiency
    #[cfg(feature = "torch")]
    pub fn optimize_tensor_device(&self, tensor: &Tensor) -> Result<Tensor> {
        // For Apple Silicon, prefer MPS (Metal Performance Shaders) when available
        // Fall back to CPU for memory efficiency
        #[cfg(feature = "torch")]
        let device = if tch::utils::has_mps() {
            Device::MPS
        } else {
            Device::Cpu
        };

        #[cfg(not(feature = "torch"))]
        let device = Device::Cpu;

        let optimized = tensor.to_device(device);
        info!("Optimized tensor device to {:?}", device);
        Ok(optimized)
    }

    /// Get memory usage of a tensor in bytes
    #[cfg(feature = "torch")]
    pub fn get_tensor_memory_usage(&self, tensor: &Tensor) -> u64 {
        let element_size = match tensor.kind() {
            Kind::Float | Kind::Double => 8,
            Kind::Int | Kind::Int64 => 8,
            Kind::Int32 => 4,
            Kind::Int16 => 2,
            Kind::Int8 | Kind::Uint8 => 1,
            _ => 4, // Default to 4 bytes
        };

        let num_elements = tensor.numel();
        (element_size * num_elements) as u64
    }

    /// Monitor torch tensor memory usage
    #[cfg(feature = "torch")]
    pub async fn track_tensor_memory(&self, tensor_id: &str, tensor: &Tensor) -> Result<()> {
        let memory_usage = self.get_tensor_memory_usage(tensor);
        let mut model_usage = self.model_usage.write().await;

        let usage = model_usage.entry(tensor_id.to_string()).or_insert_with(|| ModelUsageStats {
            model_name: tensor_id.to_string(),
            memory_usage_mb: 0.0,
            last_accessed: chrono::Utc::now(),
            access_count: 0,
            compression_ratio: 1.0,
        });

        usage.memory_usage_mb = memory_usage as f64 / (1024.0 * 1024.0);
        usage.last_accessed = chrono::Utc::now();
        usage.access_count += 1;

        debug!("Tracked tensor {}: {} MB", tensor_id, usage.memory_usage_mb);
        Ok(())
    }

    /// Optimize tensor memory layout for better performance
    #[cfg(feature = "torch")]
    pub fn optimize_tensor_layout(&self, tensor: &Tensor) -> Result<Tensor> {
        // Use contiguous memory layout for better performance
        let contiguous = tensor.contiguous();
        // Pin memory for faster GPU transfers if applicable
        #[cfg(feature = "torch")]
        let optimized = if Cuda::is_available() || tch::utils::has_mps() {
            contiguous.pin_memory()
        } else {
            contiguous
        };

        #[cfg(not(feature = "torch"))]
        let optimized = contiguous;

        Ok(optimized)
    }

    /// Create a memory-efficient tensor from candle tensor
    #[cfg(feature = "torch")]
    pub fn candle_to_torch_tensor(&self, candle_tensor: &CandleTensor) -> Result<Tensor> {
        // Convert candle tensor to torch tensor for unified memory management
        // This would require actual conversion logic based on candle tensor format
        // For now, create a placeholder torch tensor with same shape
        let shape: Vec<i64> = candle_tensor.shape().dims().iter().map(|&x| x as i64).collect();

        let dtype = match candle_tensor.dtype() {
            DType::F32 => Kind::Float,
            DType::F64 => Kind::Double,
            DType::I32 => Kind::Int,
            DType::I64 => Kind::Int64,
            DType::U8 => Kind::Uint8,
            _ => Kind::Float,
        };

        let device = Device::Cpu; // Default to CPU, could be optimized
        let torch_tensor = Tensor::zeros(&shape, (dtype, device));

        info!("Converted candle tensor to torch tensor with shape {:?}", shape);
        Ok(torch_tensor)
    }

    /// Get memory statistics for torch operations
    #[cfg(feature = "torch")]
    pub async fn get_torch_memory_stats(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();

        // Get GPU memory if available
        #[cfg(feature = "torch")]
        if tch::utils::has_cuda() {
            if let Ok(gpu_memory) = tch::Cuda::memory_summary() {
                stats.insert("gpu_allocated".to_string(), gpu_memory.allocated);
                stats.insert("gpu_reserved".to_string(), gpu_memory.reserved);
            }
        }

        // Get MPS memory if available
        #[cfg(feature = "torch")]
        if tch::utils::has_mps() {
            // MPS memory stats would be queried differently
            // For now, placeholder
            stats.insert("mps_available".to_string(), 1);
        }

        // Get CPU memory info
        let system = sysinfo::System::new_all();
        let total_memory = system.total_memory();
        let available_memory = system.available_memory();
        let used_memory = total_memory - available_memory;

        stats.insert("cpu_total".to_string(), total_memory);
        stats.insert("cpu_used".to_string(), used_memory);
        stats.insert("cpu_available".to_string(), available_memory);

        Ok(stats)
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(crate::MemoryConfig::default())
    }
}

impl MultiTenantMemoryManager {
    /// Create a new multi-tenant memory manager
    pub fn new(config: crate::MemoryConfig) -> Self {
        let total_memory = config.max_memory_mb as u64;
        Self {
            config,
            tenant_configs: Arc::new(RwLock::new(HashMap::new())),
            tenant_usage: Arc::new(RwLock::new(HashMap::new())),
            global_status: Arc::new(RwLock::new(crate::MemoryStatus {
                total_memory_mb: total_memory,
                used_memory_mb: 0,
                available_memory_mb: total_memory,
                memory_pressure: crate::MemoryPressure::Normal,
                cache_size_mb: 0,
                model_memory_mb: 0,
                timestamp: chrono::Utc::now(),
            })),
            monitoring_active: Arc::new(RwLock::new(false)),
            model_usage: Arc::new(RwLock::new(HashMap::new())),
            model_inactivity_threshold_secs: 300, // 5 minutes default
        }
    }

    /// Register a tenant with memory configuration
    pub async fn register_tenant(&self, config: TenantMemoryConfig) -> Result<()> {
        let mut tenant_configs = self.tenant_configs.write().await;

        // Validate configuration
        if config.guaranteed_memory_mb > config.max_memory_mb {
            return Err(anyhow::anyhow!("Guaranteed memory cannot exceed max memory"));
        }

        if let Some(burst_limit) = config.burst_limit_mb {
            if burst_limit < config.max_memory_mb {
                return Err(anyhow::anyhow!("Burst limit must be >= max memory"));
            }
        }

        tenant_configs.insert(config.tenant_id, config.clone());

        // Initialize tenant usage tracking
        let mut tenant_usage = self.tenant_usage.write().await;
        tenant_usage.insert(config.tenant_id, TenantMemoryUsage {
            tenant_id: config.tenant_id,
            allocated_memory_mb: 0,
            used_memory_mb: 0,
            model_count: 0,
            last_activity: chrono::Utc::now(),
            memory_pressure: crate::MemoryPressure::Normal,
        });

        info!("Registered tenant {} with {} MB max memory",
              config.tenant_id, config.max_memory_mb);
        Ok(())
    }

    /// Unregister a tenant
    pub async fn unregister_tenant(&self, tenant_id: TenantId) -> Result<()> {
        let mut tenant_configs = self.tenant_configs.write().await;
        let mut tenant_usage = self.tenant_usage.write().await;

        if tenant_configs.remove(&tenant_id).is_none() {
            return Err(anyhow::anyhow!("Tenant {} not found", tenant_id));
        }

        if let Some(usage) = tenant_usage.remove(&tenant_id) {
            if usage.allocated_memory_mb > 0 {
                warn!("Unregistering tenant {} with {} MB still allocated",
                      tenant_id, usage.allocated_memory_mb);
            }
        }

        info!("Unregistered tenant {}", tenant_id);
        Ok(())
    }

    /// Request memory allocation for a tenant
    pub async fn request_allocation(
        &self,
        request: MemoryAllocationRequest
    ) -> Result<MemoryAllocationResponse> {
        let tenant_configs = self.tenant_configs.read().await;
        let mut tenant_usage = self.tenant_usage.write().await;
        let global_status = self.global_status.read().await;

        // Get tenant configuration
        let tenant_config = tenant_configs.get(&request.tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant {} not found", request.tenant_id))?;

        // Get current tenant usage
        let tenant_usage_entry = tenant_usage.get_mut(&request.tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant usage tracking not found"))?;

        // Check tenant-specific limits
        let current_allocation = tenant_usage_entry.allocated_memory_mb;
        let max_allocation = match request.allocation_type {
            AllocationType::ModelLoad | AllocationType::Inference => tenant_config.max_memory_mb,
            AllocationType::Cache => tenant_config.burst_limit_mb.unwrap_or(tenant_config.max_memory_mb),
            AllocationType::Background => tenant_config.guaranteed_memory_mb,
        };

        if current_allocation + request.requested_memory_mb > max_allocation {
            let available_for_tenant = max_allocation.saturating_sub(current_allocation);
            return Ok(MemoryAllocationResponse::Denied {
                reason: format!("Tenant memory limit exceeded. Requested: {} MB, Available: {} MB",
                               request.requested_memory_mb, available_for_tenant),
                available_memory_mb: available_for_tenant,
            });
        }

        // Check global memory availability based on isolation level
        match tenant_config.isolation_level {
            IsolationLevel::Hard => {
                // Hard isolation - strict per-tenant limits only
                // Already checked above, so we can proceed
            }
            IsolationLevel::Soft | IsolationLevel::None => {
                // Check global memory pressure
                if global_status.memory_pressure == crate::MemoryPressure::Critical {
                    let available_global = global_status.available_memory_mb;
                    if available_global < request.requested_memory_mb {
                        return Ok(MemoryAllocationResponse::Denied {
                            reason: format!("Global memory pressure critical. Available: {} MB",
                                           available_global),
                            available_memory_mb: available_global.min(max_allocation - current_allocation),
                        });
                    }
                }
            }
        }

        // Allocation granted - update tracking
        tenant_usage_entry.allocated_memory_mb += request.requested_memory_mb;
        tenant_usage_entry.used_memory_mb += request.requested_memory_mb;
        tenant_usage_entry.last_activity = chrono::Utc::now();

        if matches!(request.allocation_type, AllocationType::ModelLoad) {
            tenant_usage_entry.model_count += 1;
        }

        // Generate allocation ID
        let allocation_id = format!("alloc_{}_{}_{}",
                                   request.tenant_id.simple(),
                                   request.model_name,
                                   chrono::Utc::now().timestamp());

        info!("Granted {} MB allocation for tenant {} ({:?})",
              request.requested_memory_mb, request.tenant_id, request.allocation_type);

        Ok(MemoryAllocationResponse::Granted { allocation_id })
    }

    /// Release memory allocation
    pub async fn release_allocation(
        &self,
        tenant_id: TenantId,
        allocation_id: &str,
        released_memory_mb: u64
    ) -> Result<()> {
        let mut tenant_usage = self.tenant_usage.write().await;

        let tenant_usage_entry = tenant_usage.get_mut(&tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant {} not found", tenant_id))?;

        if released_memory_mb > tenant_usage_entry.allocated_memory_mb {
            return Err(anyhow::anyhow!("Cannot release more memory than allocated"));
        }

        tenant_usage_entry.allocated_memory_mb = tenant_usage_entry.allocated_memory_mb.saturating_sub(released_memory_mb);
        tenant_usage_entry.used_memory_mb = tenant_usage_entry.used_memory_mb.saturating_sub(released_memory_mb);

        debug!("Released {} MB for tenant {}", released_memory_mb, tenant_id);
        Ok(())
    }

    /// Get tenant memory usage statistics
    pub async fn get_tenant_usage(&self, tenant_id: TenantId) -> Result<TenantMemoryUsage> {
        let tenant_usage = self.tenant_usage.read().await;
        tenant_usage.get(&tenant_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Tenant {} not found", tenant_id))
    }

    /// Get all tenant usage statistics
    pub async fn get_all_tenant_usage(&self) -> Result<HashMap<TenantId, TenantMemoryUsage>> {
        let tenant_usage = self.tenant_usage.read().await;
        Ok(tenant_usage.clone())
    }

    /// Perform memory balancing across tenants
    pub async fn balance_memory(&self) -> Result<()> {
        let tenant_configs = self.tenant_configs.read().await;
        let mut tenant_usage = self.tenant_usage.write().await;
        let global_status = self.global_status.read().await;

        // Skip balancing if memory pressure is normal
        if global_status.memory_pressure == crate::MemoryPressure::Normal {
            return Ok(());
        }

        // Sort tenants by priority for eviction preference
        let mut tenant_ids_by_priority: Vec<TenantId> = tenant_configs.keys()
            .filter(|id| tenant_usage.contains_key(id))
            .cloned()
            .collect();

        tenant_ids_by_priority.sort_by(|a, b| {
            let a_priority = &tenant_configs.get(a).unwrap().priority;
            let b_priority = &tenant_configs.get(b).unwrap().priority;
            a_priority.cmp(b_priority)
        });

        // Calculate total over-allocation
        let total_allocated: u64 = tenant_usage.values()
            .map(|usage| usage.allocated_memory_mb)
            .sum();

        let target_reduction = if global_status.memory_pressure == crate::MemoryPressure::High {
            (total_allocated as f64 * 0.2) as u64 // Reduce by 20%
        } else {
            (total_allocated as f64 * 0.4) as u64 // Reduce by 40%
        };

        let mut total_reduced = 0u64;

        // Evict from lowest priority tenants first
        for tenant_id in tenant_ids_by_priority {
            if total_reduced >= target_reduction {
                break;
            }

            let tenant_config = tenant_configs.get(&tenant_id).unwrap();
            let tenant_usage_entry = tenant_usage.get_mut(&tenant_id).unwrap();

            // Calculate how much we can evict from this tenant
            let guaranteed = tenant_config.guaranteed_memory_mb;
            let current = tenant_usage_entry.allocated_memory_mb;

            if current > guaranteed {
                let evict_amount = (current - guaranteed).min(target_reduction - total_reduced);

                tenant_usage_entry.allocated_memory_mb = tenant_usage_entry.allocated_memory_mb.saturating_sub(evict_amount);
                tenant_usage_entry.used_memory_mb = tenant_usage_entry.used_memory_mb.saturating_sub(evict_amount);

                total_reduced += evict_amount;

                warn!("Evicted {} MB from tenant {} due to memory pressure",
                      evict_amount, tenant_id);
            }
        }

        info!("Memory balancing completed: reduced {} MB total allocation", total_reduced);
        Ok(())
    }

    /// Update tenant memory pressure indicators
    pub async fn update_tenant_pressure_indicators(&self) -> Result<()> {
        let tenant_configs = self.tenant_configs.read().await;
        let mut tenant_usage = self.tenant_usage.write().await;
        let global_status = self.global_status.read().await;

        for (tenant_id, config) in tenant_configs.iter() {
            if let Some(usage) = tenant_usage.get_mut(tenant_id) {
                // Calculate tenant-specific pressure
                let utilization_ratio = if config.max_memory_mb > 0 {
                    usage.allocated_memory_mb as f64 / config.max_memory_mb as f64
                } else {
                    0.0
                };

                // Factor in global pressure
                let global_pressure_factor = match global_status.memory_pressure {
                    crate::MemoryPressure::Normal => 1.0,
                    crate::MemoryPressure::Warning => 1.05,
                    crate::MemoryPressure::Medium => 1.1,
                    crate::MemoryPressure::High => 1.2,
                    crate::MemoryPressure::Critical => 1.5,
                };

                let adjusted_ratio = utilization_ratio * global_pressure_factor;

                usage.memory_pressure = if adjusted_ratio < 0.7 {
                    crate::MemoryPressure::Normal
                } else if adjusted_ratio < 0.9 {
                    crate::MemoryPressure::High
                } else {
                    crate::MemoryPressure::Critical
                };
            }
        }

        Ok(())
    }

    /// Start monitoring for multi-tenant memory manager
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut monitoring_active = self.monitoring_active.write().await;
        if *monitoring_active {
            return Ok(());
        }

        *monitoring_active = true;

        let monitoring_active_clone = self.monitoring_active.clone();
        let tenant_usage_clone = self.tenant_usage.clone();
        let global_status_clone = self.global_status.clone();
        let model_usage_clone = self.model_usage.clone();
        let inactivity_threshold = self.model_inactivity_threshold_secs;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Check if monitoring is still active
                {
                    let active = monitoring_active_clone.read().await;
                    if !*active {
                        break;
                    }
                }

                // Update tenant pressure indicators
                // (This would call update_tenant_pressure_indicators if we had access to self)

                // Clean up inactive models
                let mut model_usage = model_usage_clone.write().await;
                let now = std::time::Instant::now();
                let threshold_duration = std::time::Duration::from_secs(inactivity_threshold);

                model_usage.retain(|model_name, usage| {
                    let inactive_duration = now.duration_since(usage.last_accessed);
                    if inactive_duration > threshold_duration {
                        warn!("Removing inactive model: {} (inactive for {:.1}s)",
                              model_name, inactive_duration.as_secs_f64());
                        false
                    } else {
                        true
                    }
                });
            }
        });

        info!("Multi-tenant memory monitoring started");
        Ok(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut monitoring_active = self.monitoring_active.write().await;
        *monitoring_active = false;
        info!("Multi-tenant memory monitoring stopped");
        Ok(())
    }

    /// Get comprehensive tenant dashboard
    pub async fn get_tenant_dashboard(&self) -> Result<TenantDashboard> {
        let tenant_configs = self.tenant_configs.read().await;
        let tenant_usage = self.tenant_usage.read().await;
        let global_status = self.global_status.read().await;

        let mut tenant_summaries = Vec::new();
        let mut total_allocated = 0u64;
        let mut total_used = 0u64;
        let mut critical_tenants = 0;

        for (tenant_id, config) in tenant_configs.iter() {
            if let Some(usage) = tenant_usage.get(tenant_id) {
                let summary = TenantSummary {
                    tenant_id: *tenant_id,
                    name: format!("Tenant-{}", tenant_id.simple()),
                    priority: config.priority.clone(),
                    allocated_memory_mb: usage.allocated_memory_mb,
                    used_memory_mb: usage.used_memory_mb,
                    max_memory_mb: config.max_memory_mb,
                    guaranteed_memory_mb: config.guaranteed_memory_mb,
                    utilization_percentage: if config.max_memory_mb > 0 {
                        (usage.allocated_memory_mb as f64 / config.max_memory_mb as f64 * 100.0) as u32
                    } else {
                        0
                    },
                    memory_pressure: usage.memory_pressure.clone(),
                    model_count: usage.model_count,
                    last_activity: usage.last_activity,
                    isolation_level: config.isolation_level.clone(),
                };

                if usage.memory_pressure == crate::MemoryPressure::Critical {
                    critical_tenants += 1;
                }

                total_allocated += usage.allocated_memory_mb;
                total_used += usage.used_memory_mb;

                tenant_summaries.push(summary);
            }
        }

        // Sort by priority and then by utilization
        tenant_summaries.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| b.utilization_percentage.cmp(&a.utilization_percentage))
        });

        Ok(TenantDashboard {
            global_memory_status: global_status.clone(),
            tenant_summaries,
            summary: DashboardSummary {
                total_tenants: tenant_configs.len(),
                active_tenants: tenant_usage.len(),
                total_allocated_memory_mb: total_allocated,
                total_used_memory_mb: total_used,
                critical_tenants,
                memory_efficiency: if total_allocated > 0 {
                    (total_used as f64 / total_allocated as f64 * 100.0) as u32
                } else {
                    100
                },
                last_updated: chrono::Utc::now(),
            },
        })
    }
}

/// Tenant dashboard for monitoring
#[derive(Debug, Clone)]
pub struct TenantDashboard {
    pub global_memory_status: crate::MemoryStatus,
    pub tenant_summaries: Vec<TenantSummary>,
    pub summary: DashboardSummary,
}

/// Individual tenant summary
#[derive(Debug, Clone)]
pub struct TenantSummary {
    pub tenant_id: TenantId,
    pub name: String,
    pub priority: TenantPriority,
    pub allocated_memory_mb: u64,
    pub used_memory_mb: u64,
    pub max_memory_mb: u64,
    pub guaranteed_memory_mb: u64,
    pub utilization_percentage: u32,
    pub memory_pressure: crate::MemoryPressure,
    pub model_count: usize,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub isolation_level: IsolationLevel,
}

/// Dashboard summary statistics
#[derive(Debug, Clone)]
pub struct DashboardSummary {
    pub total_tenants: usize,
    pub active_tenants: usize,
    pub total_allocated_memory_mb: u64,
    pub total_used_memory_mb: u64,
    pub critical_tenants: u32,
    pub memory_efficiency: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for MultiTenantMemoryManager {
    fn default() -> Self {
        Self::new(crate::MemoryConfig::default())
    }
}
