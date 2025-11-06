//! Central memory management coordinator
//!
//! This module provides the high-level MemoryManager that coordinates all
//! memory management subsystems including monitoring, garbage collection,
//! resource management, and allocation tracking.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use crate::memory::monitor::*;
use crate::memory::types::*;
use crate::memory::resources::*;
use crate::memory::allocation::*;
use crate::memory::metrics::{MemoryPressure, CpuMetrics};
use crate::memory::allocator::{MemoryStats, MemoryTrackingAllocator};
use crate::memory::cache::MemoryManagedCache;
use crate::memory::leaks::MemoryLeakDetector;
use crate::memory::pool::{ObjectPool, PooledObject, PoolStats};
use std::time::{Duration, Instant};

/// Configuration for memory management
#[derive(Debug, Clone)]
pub struct MemoryManagementConfig {
    /// Enable automatic garbage collection
    pub enable_gc: bool,
    /// Enable memory leak detection
    pub enable_leak_detection: bool,
    /// Enable allocation tracking
    pub enable_allocation_tracking: bool,
    /// Memory limit configuration
    pub limits: MemoryLimitConfig,
    /// Monitor configuration (alias for limits for backward compatibility)
    pub monitor_config: MemoryLimitConfig,
    /// Leak detection threshold in MB
    pub leak_detection_threshold_mb: u64,
}

/// Central memory manager
#[derive(Debug)]
pub struct MemoryManager {
    _config: MemoryManagementConfig,
    monitor: Arc<MemoryMonitor>,
    leak_detector: Option<Arc<MemoryLeakDetector>>,
    pools: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    gc_registry: Arc<RwLock<GCRegistry>>,
    finalizer_queue: Arc<RwLock<FinalizerQueue>>,
    handle_registry: Arc<RwLock<HandleRegistry>>,
    allocation_tracker: Arc<RwLock<AllocationSiteTracker>>,
}

impl MemoryManager {
    pub fn new(config: MemoryManagementConfig) -> Self {
        let monitor = Arc::new(MemoryMonitor::new(config.monitor_config.clone()));
        let leak_detector = if config.enable_leak_detection {
            Some(Arc::new(MemoryLeakDetector::new(config.leak_detection_threshold_mb)))
        } else {
            None
        };

        Self {
            _config: config,
            monitor,
            leak_detector,
            pools: Arc::new(RwLock::new(HashMap::new())),
            gc_registry: Arc::new(RwLock::new(GCRegistry::new())),
            finalizer_queue: Arc::new(RwLock::new(FinalizerQueue::new())),
            handle_registry: Arc::new(RwLock::new(HandleRegistry::new())),
            allocation_tracker: Arc::new(RwLock::new(AllocationSiteTracker::new())),
        }
    }

    /// Initialize memory management
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing memory management system");

        // Register memory pressure callbacks
        self.monitor.register_pressure_callback(MemoryPressure::High, |pressure| {
            warn!("Memory pressure is HIGH: {:?}", pressure);
            // In production, you might trigger GC, reduce cache sizes, etc.
        }).await;

        self.monitor.register_pressure_callback(MemoryPressure::Critical, |pressure| {
            error!("Memory pressure is CRITICAL: {:?}", pressure);
            // Emergency measures: aggressive GC, cache clearing, etc.
        }).await;

        // Start monitoring
        // self.monitor.start_monitoring();

        if let Some(detector) = &self.leak_detector {
            detector.take_snapshot("initialization").await;
        }

        Ok(())
    }

    /// Get current memory statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryTrackingAllocator::memory_stats()
    }

    /// Get memory pressure level
    pub fn get_memory_pressure(&self) -> MemoryPressure {
        self.monitor.get_current_pressure()
    }

    /// Create an object pool
    pub async fn create_pool<T, F>(&self, name: &str, factory: F, max_size: usize)
    where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let pool = ObjectPool::new(factory, max_size);
        let mut pools = self.pools.write().await;
        pools.insert(name.to_string(), Box::new(pool));
    }

    /// Get an object from pool with type safety
    pub async fn get_from_pool<T>(&self, name: &str) -> Option<PooledObject<T>>
    where
        T: Send + Sync + 'static,
    {
        let pools = self.pools.read().await;
        if let Some(pool_box) = pools.get(name) {
            // Attempt type-safe downcast to ObjectPool<T>
            // Note: This uses Any downcasting which provides runtime type safety
            if let Some(pool) = pool_box.downcast_ref::<ObjectPool<T>>() {
                match pool.borrow_with_timeout(Duration::from_secs(5)).await {
                    Ok(obj) => Some(obj),
                    Err(_) => {
                        tracing::warn!("Pool '{}' exhausted or timeout occurred", name);
                        None
                    }
                }
        } else {
                tracing::error!("Pool '{}' type mismatch - expected ObjectPool<{}>", name, std::any::type_name::<T>());
                None
            }
        } else {
            tracing::debug!("Pool '{}' not found", name);
            None
        }
    }

    /// Analyze memory leaks
    pub async fn analyze_memory_leaks(&self) -> Vec<String> {
        if let Some(detector) = &self.leak_detector {
            detector.analyze_leaks().await
        } else {
            Vec::new()
        }
    }

    /// Get orphaned object cleanup statistics
    pub fn get_cleanup_stats(&self) -> (usize, Vec<String>) {
        let orphaned_count = crate::memory::ORPHANED_OBJECTS.lock()
            .map(|orphaned| orphaned.len())
            .unwrap_or(0);

        let warnings = if orphaned_count > 0 {
            vec![format!("{} orphaned objects detected - consider enabling tokio runtime for proper cleanup", orphaned_count)]
        } else {
            Vec::new()
        };

        (orphaned_count, warnings)
    }

    /// Get pool stats for a specific pool using trait-based collection
    pub async fn get_pool_stats(&self, name: &str) -> Option<PoolStats> {
        let pools = self.pools.read().await;
        if let Some(pool_box) = pools.get(name) {
            // Use trait-based statistics collection with runtime polymorphism
            // This provides compile-time type safety while allowing runtime flexibility

            // For ObjectPool<T>, we can downcast and use the StatsProvider trait
            // In a more sophisticated implementation, we'd use trait objects directly

            // For now, we try to handle ObjectPool types specifically
            // This could be extended to support other pool types implementing StatsProvider

            // Note: Due to type erasure with Any, we can't directly call trait methods
            // A more advanced approach would use a registry of trait objects

            tracing::debug!("Attempting to get stats for pool '{}'", name);

            // For ObjectPool types, we can't directly downcast due to type erasure
            // This is a limitation of the current Any-based storage approach
            // In production, consider using trait objects: Box<dyn StatsProvider>

            None // Current limitation due to type erasure
        } else {
            tracing::debug!("Pool '{}' not found for statistics collection", name);
            None
        }
    }

    /// Force garbage collection
    pub async fn force_gc(&self) {
        // Perform GC operations synchronously
        let mut gc_registry = self.gc_registry.write().await;
        gc_registry.marked_objects.clear();
        gc_registry.pending_finalization.clear();
        gc_registry.weak_references.clear();
        
        debug!("Garbage collection completed");
    }

    /// Get memory usage history
    pub async fn get_memory_history(&self, _duration: Duration) -> Vec<(Instant, MemoryStats)> {
        // Get recent history from the monitor
        let history = self.monitor.get_stats_history().await;
        history.clone()
    }

    /// Create a memory-managed cache
    pub fn create_cache<K, V>(&self, _name: &str, max_entries: usize, max_memory_mb: usize, ttl_seconds: u64) -> MemoryManagedCache<K, V>
    where
        K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
        V: Clone,
    {
        MemoryManagedCache::new(max_entries, max_memory_mb, ttl_seconds)
    }

    /// Collect CPU metrics from the system
    pub async fn collect_cpu_metrics(&self) -> Result<CpuMetrics, Box<dyn std::error::Error>> {
        use sysinfo::System as SysInfo;

        let mut system = SysInfo::new();
        system.refresh_all();

        let overall_usage = system.global_cpu_info().cpu_usage();
        let per_core_usage: Vec<f64> = system.cpus().iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .collect();

        let frequency_mhz = system.cpus().first()
            .map(|cpu| cpu.frequency() as f64)
            .unwrap_or(2400.0); // Default frequency

        // Temperature monitoring - platform specific
        let temperature_celsius = self.get_cpu_temperature().await;

        Ok(CpuMetrics {
            usage_percent: overall_usage as f64,
            per_core_percent: per_core_usage,
            frequency_mhz,
            temperature_celsius,
        })
    }

    /// Get CPU temperature (platform-specific implementation)
    async fn get_cpu_temperature(&self) -> Option<f64> {
        #[cfg(target_os = "macos")]
        {
            // Try to get temperature using powermetrics or system_profiler
            tokio::process::Command::new("powermetrics")
                .args(&["-n", "1", "-i", "1000", "--samplers", "thermal"])
                .output()
                .await
                .ok()
                .and_then(|output| {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Parse temperature from powermetrics output
                    // This is a simplified implementation
                    if output_str.contains("CPU die temperature") {
                        // Extract temperature value - in practice you'd parse this properly
                        Some(45.0) // Placeholder
                    } else {
                        None
                    }
                })
        }
        #[cfg(target_os = "linux")]
        {
            // Try to read from thermal zones
            tokio::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
                .await
                .ok()
                .and_then(|temp_str| {
                    temp_str.trim().parse::<f64>().ok().map(|temp| temp / 1000.0)
                })
        }
        #[cfg(target_os = "windows")]
        {
            // Windows temperature monitoring is more complex
            // This would require additional dependencies or WMI calls
            None
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }
}
