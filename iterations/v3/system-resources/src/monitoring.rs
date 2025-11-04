//! Resource monitoring and metrics
//!
//! Tracks resource utilization, performance metrics, and allocation patterns.

use schemars::JsonSchema;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{ResourceAllocation, ResourceUtilization, PoolUtilization};

/// Resource monitor for tracking utilization and metrics
#[derive(Debug)]
pub struct ResourceMonitor {
    allocations: Arc<RwLock<HashMap<String, ResourceAllocation>>>,
    allocation_history: Arc<RwLock<Vec<AllocationEvent>>>,
    utilization_history: Arc<RwLock<Vec<UtilizationSnapshot>>>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            allocation_history: Arc::new(RwLock::new(Vec::new())),
            utilization_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a resource allocation
    pub async fn record_allocation(&self, allocation: &ResourceAllocation) {
        let mut allocations = self.allocations.write().await;
        allocations.insert(allocation.allocation_id.clone(), allocation.clone());

        let event = AllocationEvent {
            event_type: AllocationEventType::Allocated,
            allocation_id: allocation.allocation_id.clone(),
            pool_name: allocation.pool_name.clone(),
            timestamp: chrono::Utc::now(),
            resources: allocation.allocated_resources.clone(),
        };

        let mut history = self.allocation_history.write().await;
        history.push(event);
    }

    /// Record a resource release
    pub async fn record_release(&self, allocation_id: &str) {
        let mut allocations = self.allocations.write().await;
        if let Some(allocation) = allocations.remove(allocation_id) {
            let event = AllocationEvent {
                event_type: AllocationEventType::Released,
                allocation_id: allocation_id.to_string(),
                pool_name: allocation.pool_name,
                timestamp: chrono::Utc::now(),
                resources: allocation.allocated_resources,
            };

            let mut history = self.allocation_history.write().await;
            history.push(event);
        }
    }

    /// Get current resource utilization
    pub async fn get_utilization(&self) -> ResourceUtilization {
        let allocations = self.allocations.read().await;

        // Mock utilization calculation - would be based on actual pool metrics
        let mut total_memory = 0u64;
        let mut used_memory = 0u64;
        let mut total_cpu = 0.0f32;
        let mut used_cpu = 0.0f32;
        let mut pool_utilizations = HashMap::new();

        // Group allocations by pool
        let mut pool_allocations: HashMap<String, Vec<&ResourceAllocation>> = HashMap::new();
        for allocation in allocations.values() {
            pool_allocations.entry(allocation.pool_name.clone())
                .or_insert_with(Vec::new)
                .push(allocation);
        }

        for (pool_name, allocs) in pool_allocations {
            let active_count = allocs.len();
            let total_capacity = 100; // Mock capacity

            // Calculate pool utilization based on allocations
            let utilization_percent = if total_capacity > 0 {
                (active_count as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            };

            pool_utilizations.insert(pool_name.clone(), PoolUtilization {
                pool_name: pool_name.clone(),
                utilization_percent,
                active_allocations: active_count,
                total_capacity,
            });

            // Mock resource accumulation
            total_memory += 16384; // 16GB per pool
            used_memory += (16384.0 * utilization_percent / 100.0) as u64;
            total_cpu += 8.0; // 8 cores per pool
            used_cpu += 8.0 * utilization_percent as f32 / 100.0;
        }

        ResourceUtilization {
            total_memory_mb: total_memory,
            used_memory_mb: used_memory,
            total_cpu_cores: total_cpu,
            used_cpu_cores: used_cpu,
            active_allocations: allocations.len(),
            pool_utilizations,
        }
    }

    /// Record utilization snapshot
    pub async fn record_utilization_snapshot(&self, utilization: &ResourceUtilization) {
        let snapshot = UtilizationSnapshot {
            timestamp: chrono::Utc::now(),
            utilization: utilization.clone(),
        };

        let mut history = self.utilization_history.write().await;
        history.push(snapshot);

        // Keep only last 1000 snapshots to prevent unbounded growth
        if history.len() > 1000 {
            let keep_count = 1000;
            let drain_count = history.len() - keep_count;
            history.drain(0..drain_count);
        }
    }

    /// Get allocation history
    pub async fn get_allocation_history(&self, limit: usize) -> Vec<AllocationEvent> {
        let history = self.allocation_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get utilization trends
    pub async fn get_utilization_trends(&self, hours: u32) -> Vec<UtilizationSnapshot> {
        let history = self.utilization_history.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours as i64);

        history.iter()
            .filter(|snapshot| snapshot.timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// Get resource allocation efficiency metrics
    pub async fn get_efficiency_metrics(&self) -> EfficiencyMetrics {
        let history = self.allocation_history.read().await;

        let mut total_allocations = 0;
        let mut failed_allocations = 0;
        let average_allocation_time = std::time::Duration::from_secs(0);
        let _allocation_count = 0;

        for event in history.iter() {
            match event.event_type {
                AllocationEventType::Allocated => {
                    total_allocations += 1;
                }
                AllocationEventType::Released => {
                    // Could calculate allocation duration here
                }
                AllocationEventType::Failed => {
                    failed_allocations += 1;
                }
            }
        }

        EfficiencyMetrics {
            total_allocations,
            failed_allocations,
            success_rate: if total_allocations + failed_allocations > 0 {
                total_allocations as f64 / (total_allocations + failed_allocations) as f64
            } else {
                1.0
            },
            average_allocation_duration: average_allocation_time,
        }
    }
}

/// Allocation event for tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct AllocationEvent {
    pub event_type: AllocationEventType,
    pub allocation_id: String,
    pub pool_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resources: crate::ResourceRequirements,
}

/// Type of allocation event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum AllocationEventType {
    Allocated,
    Released,
    Failed,
}

/// Utilization snapshot for historical tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct UtilizationSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub utilization: ResourceUtilization,
}

/// Efficiency metrics for resource allocation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct EfficiencyMetrics {
    pub total_allocations: usize,
    pub failed_allocations: usize,
    pub success_rate: f64,
    pub average_allocation_duration: std::time::Duration,
}
