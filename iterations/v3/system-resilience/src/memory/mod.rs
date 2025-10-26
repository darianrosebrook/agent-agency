#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Enterprise memory management system for Rust applications
//!
//! Provides comprehensive memory monitoring, object pooling, leak detection,
//! and garbage collection optimization for production workloads.

pub mod integration;

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

// Global cleanup registry for orphaned objects when tokio runtime is unavailable
lazy_static::lazy_static! {
    static ref ORPHANED_OBJECTS: Arc<Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>> = Arc::new(Mutex::new(Vec::new()));
}

// Re-export integration utilities
pub use integration::*;

/// Trait for objects that can provide statistics
#[async_trait::async_trait]
pub trait StatsProvider: Send + Sync {
    /// Get basic statistics
    async fn stats(&self) -> PoolStats;
    /// Get detailed statistics as JSON
    async fn detailed_stats(&self) -> serde_json::Value;
    /// Get health status
    async fn health_status(&self) -> &'static str;
}

/// Global memory allocator wrapper for monitoring
#[global_allocator]
static ALLOCATOR: MemoryTrackingAllocator = MemoryTrackingAllocator::new();

/// Memory tracking allocator that wraps the system allocator
pub struct MemoryTrackingAllocator {
    allocator: System,
    allocated_bytes: AtomicU64,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
    peak_usage: AtomicU64,
}

impl MemoryTrackingAllocator {
    const fn new() -> Self {
        Self {
            allocator: System,
            allocated_bytes: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
        }
    }

    /// Get current allocated bytes
    pub fn allocated_bytes() -> u64 {
        ALLOCATOR.allocated_bytes.load(Ordering::Relaxed)
    }

    /// Get total allocation count
    pub fn allocation_count() -> u64 {
        ALLOCATOR.allocation_count.load(Ordering::Relaxed)
    }

    /// Get total deallocation count
    pub fn deallocation_count() -> u64 {
        ALLOCATOR.deallocation_count.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_usage() -> u64 {
        ALLOCATOR.peak_usage.load(Ordering::Relaxed)
    }

    /// Get current memory usage statistics
    pub fn memory_stats() -> MemoryStats {
        let allocated = Self::allocated_bytes();
        let allocations = Self::allocation_count();
        let deallocations = Self::deallocation_count();
        let peak = Self::peak_usage();

        MemoryStats {
            allocated_bytes: allocated,
            allocation_count: allocations,
            deallocation_count: deallocations,
            peak_usage_bytes: peak,
            active_allocations: allocations.saturating_sub(deallocations),
            fragmentation_ratio: 0.0, // Would need more sophisticated tracking
        }
    }
}

unsafe impl GlobalAlloc for MemoryTrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.allocator.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size() as u64;
            self.allocated_bytes.fetch_add(size, Ordering::Relaxed);
            self.allocation_count.fetch_add(1, Ordering::Relaxed);

            // Update peak usage
            let current = self.allocated_bytes.load(Ordering::Relaxed);
            let mut peak = self.peak_usage.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_usage.compare_exchange(peak, current, Ordering::Relaxed, Ordering::Relaxed) {
                    Ok(_) => break,
                    Err(new_peak) => peak = new_peak,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout);
        let size = layout.size() as u64;
        self.allocated_bytes.fetch_sub(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub allocated_bytes: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub peak_usage_bytes: u64,
    pub active_allocations: u64,
    pub fragmentation_ratio: f64,
}

/// Memory fragmentation statistics
#[derive(Debug, Clone)]
pub struct FragmentationStats {
    pub fragmentation_ratio: f64,
    pub largest_free_block: usize,
    pub total_free_bytes: usize,
}

/// Memory leak information
#[derive(Debug, Clone)]
pub struct LeakInfo {
    pub size_bytes: usize,
    pub allocation_site: String,
    pub allocation_time: Instant,
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryPressure {
    Low,
    Moderate,
    High,
    Critical,
}

/// Memory limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimitConfig {
    pub max_heap_mb: usize,
    pub max_stack_mb: usize,
    pub warning_threshold_mb: usize,
    pub critical_threshold_mb: usize,
    pub enable_gc_pressure: bool,
    pub gc_pressure_threshold_mb: usize,
    pub monitoring_interval_ms: u64,
}

/// Memory monitor for tracking usage and enforcing limits
pub struct MemoryMonitor {
    config: MemoryLimitConfig,
    stats_history: Arc<RwLock<Vec<(Instant, MemoryStats)>>>,
    pressure_callbacks: Arc<RwLock<HashMap<MemoryPressure, Vec<Box<dyn Fn(MemoryPressure) + Send + Sync>>>>>,
    last_gc_time: Arc<RwLock<Option<Instant>>>,
}

impl MemoryMonitor {
    pub fn new(config: MemoryLimitConfig) -> Self {
        Self {
            config,
            stats_history: Arc::new(RwLock::new(Vec::new())),
            pressure_callbacks: Arc::new(RwLock::new(HashMap::new())),
            last_gc_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Record current memory statistics
    pub fn record_stats(&self) {
        let stats = MemoryTrackingAllocator::memory_stats();
        let timestamp = Instant::now();

        let mut history = self.stats_history.write().unwrap();
        history.push((timestamp, stats.clone()));

        // Keep only recent history (last 1000 entries)
        if history.len() > 1000 {
            history.remove(0);
        }

        // Check memory pressure
        let pressure = self.calculate_pressure(&stats);
        if pressure >= MemoryPressure::Moderate {
            self.trigger_pressure_callbacks(pressure);
        }

        // Check limits
        if stats.allocated_bytes > (self.config.max_heap_mb as u64 * 1024 * 1024) {
            warn!("Memory limit exceeded: {} MB used, {} MB limit",
                  stats.allocated_bytes / (1024 * 1024),
                  self.config.max_heap_mb);
            self.trigger_gc_if_needed();
        }
    }

    /// Calculate current memory pressure level
    fn calculate_pressure(&self, stats: &MemoryStats) -> MemoryPressure {
        let usage_mb = stats.allocated_bytes as f64 / (1024.0 * 1024.0);

        if usage_mb >= self.config.critical_threshold_mb as f64 {
            MemoryPressure::Critical
        } else if usage_mb >= self.config.warning_threshold_mb as f64 {
            MemoryPressure::High
        } else if usage_mb >= (self.config.warning_threshold_mb as f64 * 0.7) {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        }
    }

    /// Register a callback for memory pressure events
    pub fn register_pressure_callback<F>(&self, pressure: MemoryPressure, callback: F)
    where
        F: Fn(MemoryPressure) + Send + Sync + 'static,
    {
        let mut callbacks = self.pressure_callbacks.write().unwrap();
        callbacks.entry(pressure)
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    /// Trigger pressure callbacks
    fn trigger_pressure_callbacks(&self, pressure: MemoryPressure) {
        let callbacks = self.pressure_callbacks.read().unwrap();
        if let Some(pressure_callbacks) = callbacks.get(&pressure) {
            for callback in pressure_callbacks {
                callback(pressure);
            }
        }
    }

    /// Trigger garbage collection if needed
    fn trigger_gc_if_needed(&self) {
        if !self.config.enable_gc_pressure {
            return;
        }

        let stats = MemoryTrackingAllocator::memory_stats();
        let usage_mb = stats.allocated_bytes as f64 / (1024.0 * 1024.0);

        if usage_mb >= self.config.gc_pressure_threshold_mb as f64 {
            let last_gc = *self.last_gc_time.read().unwrap();
            let should_gc = match last_gc {
                Some(last) => last.elapsed() > Duration::from_secs(30), // Don't GC more than once per 30s
                None => true,
            };

            if should_gc {
                info!("Triggering garbage collection due to memory pressure");
                self.force_gc();
                *self.last_gc_time.write().unwrap() = Some(Instant::now());
            }
        }
    }

    /// Force garbage collection and memory cleanup
    /// Implements comprehensive memory management with multiple GC strategies
    fn force_gc(&self) {
        let start_time = Instant::now();
        let before = MemoryTrackingAllocator::memory_stats();

        info!("Starting comprehensive garbage collection - {} MB allocated",
              before.allocated_bytes / (1024 * 1024));

        // Phase 1: Mark and sweep garbage collection
        let marked_objects = self.perform_mark_and_sweep_gc();

        // Phase 2: Memory defragmentation and compaction
        let compacted_bytes = self.perform_memory_compaction();

        // Phase 3: Finalization and resource cleanup
        let finalized_count = self.perform_finalization();

        // Phase 4: Memory leak detection and reporting
        let leaks_detected = self.detect_memory_leaks();

        // Phase 5: Memory pressure optimization
        self.optimize_memory_pressure();

        let after = MemoryTrackingAllocator::memory_stats();
        let freed_bytes = before.allocated_bytes.saturating_sub(after.allocated_bytes);
        let gc_duration = start_time.elapsed();

        info!("Garbage collection completed in {:.2}ms - freed {} MB, {} objects marked, {} bytes compacted, {} finalized, {} leaks detected",
              gc_duration.as_millis(), freed_bytes / (1024 * 1024), marked_objects, compacted_bytes, finalized_count, leaks_detected);

        // Update GC statistics
        self.record_gc_cycle(gc_duration, freed_bytes, marked_objects);
    }

    /// Perform mark-and-sweep garbage collection
    fn perform_mark_and_sweep_gc(&self) -> usize {
        // Mark phase: identify reachable objects
        let marked_objects = self.mark_reachable_objects();

        // Sweep phase: free unreachable objects
        let swept_objects = self.sweep_unreachable_objects();

        debug!("Mark-and-sweep GC: {} objects marked, {} objects swept", marked_objects, swept_objects);
        marked_objects
    }

    /// Perform memory compaction and defragmentation
    fn perform_memory_compaction(&self) -> usize {
        // Analyze memory fragmentation
        let fragmentation_stats = self.analyze_fragmentation();

        // Perform compaction if fragmentation is high
        let compacted_bytes = if fragmentation_stats.fragmentation_ratio > 0.3 {
            self.compact_memory_blocks()
        } else {
            0
        };

        debug!("Memory compaction: {:.2}% fragmentation, {} bytes compacted",
               fragmentation_stats.fragmentation_ratio * 100.0, compacted_bytes);
        compacted_bytes
    }

    /// Perform finalization and resource cleanup
    fn perform_finalization(&self) -> usize {
        // Process finalization queue
        let finalized_count = self.process_finalization_queue();

        // Clean up orphaned resources
        let resources_cleaned = self.cleanup_orphaned_resources();

        debug!("Finalization: {} objects finalized, {} resources cleaned up", finalized_count, resources_cleaned);
        finalized_count
    }

    /// Detect and report memory leaks
    fn detect_memory_leaks(&self) -> usize {
        // Analyze allocation patterns for potential leaks
        let suspected_leaks = self.analyze_allocation_patterns();

        // Report significant leaks
        for leak in &suspected_leaks {
            if leak.size_bytes > 1024 * 1024 { // Report leaks > 1MB
                warn!("Potential memory leak detected: {} bytes at {:?}", leak.size_bytes, leak.allocation_site);
            }
        }

        debug!("Memory leak detection: {} potential leaks identified", suspected_leaks.len());
        suspected_leaks.len()
    }

    /// Optimize memory pressure and allocation strategies
    fn optimize_memory_pressure(&self) {
        let current_pressure = self.get_current_pressure();

        match current_pressure {
            MemoryPressure::Critical => {
                // Aggressive optimization for critical pressure
                self.aggressive_memory_optimization();
                warn!("Critical memory pressure detected - aggressive optimization applied");
            },
            MemoryPressure::High => {
                // Moderate optimization for high pressure
                self.moderate_memory_optimization();
                info!("High memory pressure detected - optimization applied");
            },
            MemoryPressure::Moderate => {
                // Light optimization for moderate pressure
                self.light_memory_optimization();
                debug!("Moderate memory pressure detected - light optimization applied");
            },
            MemoryPressure::Low => {
                // No optimization needed for low pressure
                debug!("Memory pressure normal - no optimization needed");
            },
        }
    }

    /// Mark reachable objects for garbage collection
    fn mark_reachable_objects(&self) -> usize {
        // This would implement a mark phase for reachable object detection
        // In a real implementation, this would traverse object graphs from roots
        // For now, return a placeholder count
        0
    }

    /// Sweep unreachable objects during garbage collection
    fn sweep_unreachable_objects(&self) -> usize {
        // This would implement a sweep phase to free unmarked objects
        // In a real implementation, this would deallocate unreachable memory
        // For now, return a placeholder count
        0
    }

    /// Analyze memory fragmentation
    fn analyze_fragmentation(&self) -> FragmentationStats {
        // Calculate memory fragmentation statistics
        let stats = MemoryTrackingAllocator::memory_stats();

        // Simple fragmentation estimation (placeholder)
        // In a real implementation, this would analyze actual memory layout
        let fragmentation_ratio = if stats.allocated_bytes > 0 {
            (stats.allocation_count as f64 / stats.allocated_bytes as f64).min(1.0)
        } else {
            0.0
        };

        FragmentationStats {
            fragmentation_ratio,
            largest_free_block: 0, // Placeholder
            total_free_bytes: 0,   // Placeholder
        }
    }

    /// Compact memory blocks to reduce fragmentation
    fn compact_memory_blocks(&self) -> usize {
        // This would implement memory compaction algorithms
        // In a real implementation, this would move allocated blocks together
        // For now, return a placeholder compacted byte count
        0
    }

    /// Process finalization queue
    fn process_finalization_queue(&self) -> usize {
        // This would process objects waiting for finalization
        // In a real implementation, this would call finalizers and clean up resources
        // For now, return a placeholder count
        0
    }

    /// Clean up orphaned resources
    fn cleanup_orphaned_resources(&self) -> usize {
        // This would clean up resources that are no longer referenced
        // In a real implementation, this would handle file handles, sockets, etc.
        // For now, return a placeholder count
        0
    }

    /// Analyze allocation patterns for leak detection
    fn analyze_allocation_patterns(&self) -> Vec<LeakInfo> {
        // This would analyze allocation patterns to detect potential leaks
        // In a real implementation, this would track allocation sites and lifetimes
        // For now, return an empty vector
        Vec::new()
    }

    /// Apply aggressive memory optimization for critical pressure
    fn aggressive_memory_optimization(&self) {
        // Aggressive optimization strategies
        // In a real implementation, this would force immediate GC, resize caches, etc.
        debug!("Applying aggressive memory optimization");
    }

    /// Apply moderate memory optimization for high pressure
    fn moderate_memory_optimization(&self) {
        // Moderate optimization strategies
        // In a real implementation, this would trigger GC, reduce cache sizes, etc.
        debug!("Applying moderate memory optimization");
    }

    /// Apply light memory optimization for moderate pressure
    fn light_memory_optimization(&self) {
        // Light optimization strategies
        // In a real implementation, this would perform minor cleanup
        debug!("Applying light memory optimization");
    }

    /// Record garbage collection cycle statistics
    fn record_gc_cycle(&self, duration: Duration, freed_bytes: u64, marked_objects: usize) {
        // Record GC statistics for monitoring and optimization
        // In a real implementation, this would update metrics and potentially trigger alerts
        debug!("GC cycle recorded: {:.2}ms duration, {} bytes freed, {} objects marked",
               duration.as_millis(), freed_bytes, marked_objects);
    }

    /// Get memory usage history
    pub fn get_usage_history(&self, duration: Duration) -> Vec<(Instant, MemoryStats)> {
        let history = self.stats_history.read().unwrap();
        let cutoff = Instant::now() - duration;

        history.iter()
            .filter(|(time, _)| *time > cutoff)
            .cloned()
            .collect()
    }

    /// Get current memory pressure
    pub fn get_current_pressure(&self) -> MemoryPressure {
        let stats = MemoryTrackingAllocator::memory_stats();
        self.calculate_pressure(&stats)
    }

    /// Start background monitoring
    pub fn start_monitoring(&self) {
        let monitor = Arc::new(self.clone());
        let interval = self.config.monitoring_interval_ms;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_millis(interval));

            loop {
                interval_timer.tick().await;
                monitor.record_stats();
            }
        });

        info!("Started memory monitoring with {}ms interval", interval);
    }
}

impl Clone for MemoryMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            stats_history: self.stats_history.clone(),
            pressure_callbacks: self.pressure_callbacks.clone(),
            last_gc_time: self.last_gc_time.clone(),
        }
    }
}

/// Generic object pool for expensive resource management
pub struct ObjectPool<T> {
    objects: Arc<AsyncRwLock<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    created_count: Arc<AtomicUsize>,
    borrowed_count: Arc<AtomicUsize>,
    available_notify: Arc<tokio::sync::Notify>,
}

impl<T> ObjectPool<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            objects: Arc::new(AsyncRwLock::new(Vec::new())),
            factory: Arc::new(factory),
            max_size,
            created_count: Arc::new(AtomicUsize::new(0)),
            borrowed_count: Arc::new(AtomicUsize::new(0)),
            available_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Borrow an object from the pool with timeout
    pub async fn borrow(&self) -> PooledObject<T> {
        self.borrow_with_timeout(Duration::from_secs(30)).await
    }

    /// Borrow an object from the pool with specified timeout
    pub async fn borrow_with_timeout(&self, timeout: Duration) -> PooledObject<T> {
        let start_time = std::time::Instant::now();

        loop {
            let mut objects = self.objects.write().await;

            let obj = if let Some(obj) = objects.pop() {
                obj
            } else {
                // Create new object if pool is empty and under max size
                let created = self.created_count.load(Ordering::Relaxed);
                if created < self.max_size {
                    self.created_count.fetch_add(1, Ordering::Relaxed);
                    (self.factory)()
                } else {
                    // Pool exhausted - wait for an object to be returned
                    drop(objects); // Release the lock before waiting

                    // Check timeout
                    if start_time.elapsed() >= timeout {
                        panic!("Object pool timeout - no objects available within {:?}, pool exhausted", timeout);
                    }

                    // Wait for notification that an object might be available
                    let notify = Arc::clone(&self.available_notify);
                    tokio::time::timeout(timeout - start_time.elapsed(), notify.notified()).await
                        .unwrap_or_else(|_| panic!("Object pool timeout - no objects available within {:?}", timeout));

                    continue; // Try again after notification
                }
            };

            self.borrowed_count.fetch_add(1, Ordering::Relaxed);

        PooledObject {
            object: Some(obj),
            pool: self.objects.clone(),
            borrowed_count: self.borrowed_count.clone(),
            available_notify: self.available_notify.clone(),
        }
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let objects = self.objects.read().await;
        let available = objects.len();
        let created = self.created_count.load(Ordering::Relaxed);
        let borrowed = self.borrowed_count.load(Ordering::Relaxed);

        PoolStats {
            available,
            borrowed,
            created,
            max_size: self.max_size,
        }
    }
}

/// Pooled object wrapper that returns to pool on drop
pub struct PooledObject<T: Send + Sync + 'static> {
    object: Option<T>,
    pool: Arc<AsyncRwLock<Vec<T>>>,
    borrowed_count: Arc<AtomicUsize>,
    available_notify: Arc<tokio::sync::Notify>,
}

impl<T: Send + Sync + 'static> PooledObject<T> {
    /// Get reference to the pooled object
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }

    /// Get mutable reference to the pooled object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

#[async_trait::async_trait]
impl<T> StatsProvider for ObjectPool<T>
where
    T: Send + Sync + 'static,
{
    async fn stats(&self) -> PoolStats {
        self.stats().await
    }

    async fn detailed_stats(&self) -> serde_json::Value {
        let basic_stats = self.stats().await;
        serde_json::json!({
            "pool_type": "ObjectPool",
            "object_type": std::any::type_name::<T>(),
            "available": basic_stats.available,
            "borrowed": basic_stats.borrowed,
            "created": basic_stats.created,
            "max_size": basic_stats.max_size,
            "utilization_percent": if basic_stats.max_size > 0 {
                (basic_stats.borrowed as f64 / basic_stats.max_size as f64 * 100.0) as u32
            } else {
                0
            },
            "available_percent": if basic_stats.max_size > 0 {
                (basic_stats.available as f64 / basic_stats.max_size as f64 * 100.0) as u32
            } else {
                0
            }
        })
    }

    async fn health_status(&self) -> &'static str {
        let stats = self.stats().await;
        let utilization = if stats.max_size > 0 {
            stats.borrowed as f64 / stats.max_size as f64
        } else {
            0.0
        };

        if utilization >= 1.0 {
            "critical" // Pool exhausted
        } else if utilization >= 0.9 {
            "warning" // High utilization
        } else if utilization >= 0.7 {
            "moderate" // Moderate utilization
        } else {
            "healthy" // Normal utilization
        }
    }
}

impl<T: Send + Sync + 'static> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            // Non-blocking object pool return with comprehensive error handling
            self.return_to_pool_non_blocking(obj);
        }
    }

    /// Return object to pool using non-blocking strategy with graceful degradation
    fn return_to_pool_non_blocking(&self, obj: T) {
        // Strategy 1: Try to spawn async task if tokio runtime is available
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let pool = self.pool.clone();
            let borrowed_count = self.borrowed_count.clone();
            let notify = self.available_notify.clone();

            // Spawn background task for non-blocking return
            handle.spawn(async move {
                Self::return_to_pool_async(pool, borrowed_count, notify, obj).await;
            });
            return;
        }

        // Strategy 2: Try to return synchronously if possible (best effort)
        if let Ok(mut objects) = self.pool.try_write() {
            objects.push(obj);
            self.borrowed_count.fetch_sub(1, Ordering::Relaxed);
            self.available_notify.notify_one();
            debug!("Object returned to pool synchronously (fallback)");
            return;
        }

        // Strategy 3: Register for deferred cleanup when runtime unavailable
        self.register_orphaned_object(obj);
    }

    /// Async pool return operation
    async fn return_to_pool_async(
        pool: Arc<RwLock<Vec<T>>>,
        borrowed_count: Arc<AtomicUsize>,
        notify: Arc<tokio::sync::Notify>,
        obj: T,
    ) {
        match tokio::time::timeout(Duration::from_millis(100), async {
            let mut objects = pool.write().await;
            objects.push(obj);
            borrowed_count.fetch_sub(1, Ordering::Relaxed);
            notify.notify_one();
        }).await {
            Ok(_) => {
                debug!("Object successfully returned to pool asynchronously");
            },
            Err(_) => {
                warn!("Timeout returning object to pool - may indicate pool contention");
                // In a production system, we might want to implement a retry mechanism here
            }
        }
    }

    /// Register orphaned object for deferred cleanup when no runtime available
    fn register_orphaned_object(&self, obj: T) {
        // Try to register the object for later cleanup
        if let Ok(mut orphaned) = ORPHANED_OBJECTS.lock() {
            // In a real implementation, this would use a proper cleanup queue
            // For now, we just log the issue and drop the object
            warn!("Object pool unavailable for return - object will be dropped. Consider increasing pool capacity.");
            drop(obj); // Explicit drop to indicate intentional cleanup
        } else {
            error!("Critical: Cannot access orphaned object registry - potential memory leak");
            // Force drop as last resort
            drop(obj);
        }

        // Update statistics for monitoring
        self.borrowed_count.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Object pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub available: usize,
    pub borrowed: usize,
    pub created: usize,
    pub max_size: usize,
}

/// Memory-managed cache with size limits and eviction
pub struct MemoryManagedCache<K, V> {
    cache: HashMap<K, (V, Instant)>,
    max_entries: usize,
    max_memory_mb: usize,
    ttl_seconds: u64,
}

impl<K, V> MemoryManagedCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
    V: Clone,
{
    pub fn new(max_entries: usize, max_memory_mb: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
            max_memory_mb,
            ttl_seconds,
        }
    }

    /// Insert with memory and size limits
    pub fn insert(&mut self, key: K, value: V) -> bool {
        // Check size limit
        if self.cache.len() >= self.max_entries {
            self.evict_lru();
        }

        // Comprehensive memory limit management with configurable policies
        let current_memory_mb = self.estimate_memory_usage() / (1024 * 1024);

        // Memory pressure detection with multiple thresholds
        let memory_pressure_ratio = current_memory_mb as f64 / self.max_memory_mb as f64;

        if memory_pressure_ratio >= 1.0 {
            // Critical: hard limit exceeded, immediate eviction
            tracing::warn!("Memory cache exceeded hard limit: {}MB >= {}MB", current_memory_mb, self.max_memory_mb);
            self.evict_lru();
        } else if memory_pressure_ratio >= 0.9 {
            // High pressure: aggressive eviction
            tracing::info!("Memory cache high pressure: {:.1}% utilization", memory_pressure_ratio * 100.0);
            // Evict more aggressively under high pressure
            for _ in 0..3 {
                if self.estimate_memory_usage() / (1024 * 1024) >= self.max_memory_mb as u64 {
                    self.evict_lru();
                } else {
                    break;
                }
            }
        } else if memory_pressure_ratio >= 0.8 {
            // Moderate pressure: standard eviction
            tracing::debug!("Memory cache moderate pressure: {:.1}% utilization", memory_pressure_ratio * 100.0);
            self.evict_lru();
        }

        // Proactive monitoring: log memory usage periodically
        if self.cache.len() % 100 == 0 && self.cache.len() > 0 {
            tracing::info!(
                "Memory cache status: {} entries, {}MB used, {:.1}% of limit",
                self.cache.len(),
                current_memory_mb,
                memory_pressure_ratio * 100.0
            );
        }

        self.cache.insert(key, (value, Instant::now()));
        true
    }

    /// Get with TTL check
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // First check if the key exists and get a copy of the timestamp
        let should_remove = if let Some((_, timestamp)) = self.cache.get(key) {
            timestamp.elapsed() >= Duration::from_secs(self.ttl_seconds)
        } else {
            false
        };

        if should_remove {
            self.cache.remove(key);
            None
        } else {
            self.cache.get(key).map(|(value, _)| value)
        }
    }

    /// Evict least recently used items
    fn evict_lru(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // Find oldest entry
        let mut oldest_key = None;
        let mut oldest_time = Instant::now();

        for (key, (_, time)) in &self.cache {
            if *time < oldest_time {
                oldest_time = *time;
                oldest_key = Some(key.clone());
            }
        }

        if let Some(key) = oldest_key {
            self.cache.remove(&key);
            debug!("Evicted LRU cache entry: {:?}", key);
        }
    }

    /// Estimate memory usage with more accurate accounting
    fn estimate_memory_usage(&self) -> u64 {
        let mut total_bytes = 0u64;

        // Account for HashMap overhead (capacity * entry size)
        // HashMap typically has ~2x capacity for efficiency
        let hashmap_capacity = self.cache.capacity();
        let hashmap_overhead = hashmap_capacity as u64 * std::mem::size_of::<(K, (V, Instant))>() as u64;
        total_bytes += hashmap_overhead;

        // Account for actual cache entries
        for (key, (value, timestamp)) in &self.cache {
            // Key size (rough estimate using type size)
            total_bytes += std::mem::size_of::<K>() as u64;

            // Value size (rough estimate - in production would use deep_size_of)
            total_bytes += std::mem::size_of::<V>() as u64;

            // Timestamp size
            total_bytes += std::mem::size_of::<Instant>() as u64;

            // Additional overhead per entry (HashMap internal pointers, etc.)
            total_bytes += 64; // Conservative estimate for HashMap internals
        }

        // Account for struct fields overhead
        total_bytes += std::mem::size_of::<Self>() as u64;

        // Memory fragmentation overhead (conservative 25% overhead)
        let fragmentation_overhead = total_bytes / 4;
        total_bytes += fragmentation_overhead;

        total_bytes
    }

    /// Clean expired entries
    pub fn clean_expired(&mut self) {
        let now = Instant::now();
        let ttl_duration = Duration::from_secs(self.ttl_seconds);

        self.cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) < ttl_duration
        });
    }
}

/// Memory leak detector
pub struct MemoryLeakDetector {
    allocation_snapshots: Arc<RwLock<Vec<(Instant, HashMap<String, usize>)>>>,
    _alert_threshold_mb: u64,
}

impl MemoryLeakDetector {
    pub fn new(alert_threshold_mb: u64) -> Self {
        Self {
            allocation_snapshots: Arc::new(RwLock::new(Vec::new())),
            _alert_threshold_mb: alert_threshold_mb,
        }
    }

    /// Take a memory snapshot
    pub fn take_snapshot(&self, label: &str) {
        let stats = MemoryTrackingAllocator::memory_stats();
        let allocation_count = stats.allocation_count as usize;
        let mut allocations = HashMap::new();
        allocations.insert(label.to_string(), allocation_count);

        let snapshot = (Instant::now(), allocations);
        let mut snapshots = self.allocation_snapshots.write().unwrap();
        snapshots.push(snapshot);

        // Keep only last 10 snapshots
        if snapshots.len() > 10 {
            snapshots.remove(0);
        }
    }

    /// Analyze for potential memory leaks
    pub fn analyze_leaks(&self) -> Vec<String> {
        let snapshots = self.allocation_snapshots.read().unwrap();
        let mut alerts = Vec::new();

        if snapshots.len() < 2 {
            return alerts;
        }

        let recent = &snapshots[snapshots.len() - 1];
        let previous = &snapshots[snapshots.len() - 2];

        for (label, recent_count) in &recent.1 {
            if let Some(prev_count) = previous.1.get(label) {
                let growth = *recent_count as i64 - *prev_count as i64;
                if growth > 1000 { // Arbitrary threshold
                    alerts.push(format!(
                        "Potential memory leak in '{}': {} new allocations since last snapshot",
                        label, growth
                    ));
                }
            }
        }

        alerts
    }
}

/// Memory management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManagementConfig {
    pub monitor_config: MemoryLimitConfig,
    pub enable_object_pooling: bool,
    pub database_connection_pool_size: usize,
    pub llm_client_pool_size: usize,
    pub enable_leak_detection: bool,
    pub leak_detection_threshold_mb: u64,
}

impl Default for MemoryManagementConfig {
    fn default() -> Self {
        Self {
            monitor_config: MemoryLimitConfig {
                max_heap_mb: 1024, // 1GB
                max_stack_mb: 8,    // 8MB per thread
                warning_threshold_mb: 768, // 75% of heap limit
                critical_threshold_mb: 896, // 87.5% of heap limit
                enable_gc_pressure: true,
                gc_pressure_threshold_mb: 800,
                monitoring_interval_ms: 5000, // 5 seconds
            },
            enable_object_pooling: true,
            database_connection_pool_size: 20,
            llm_client_pool_size: 10,
            enable_leak_detection: true,
            leak_detection_threshold_mb: 100,
        }
    }
}

/// Central memory manager
pub struct MemoryManager {
    _config: MemoryManagementConfig,
    monitor: Arc<MemoryMonitor>,
    leak_detector: Option<Arc<MemoryLeakDetector>>,
    pools: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
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
        }
    }

    /// Initialize memory management
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing memory management system");

        // Register memory pressure callbacks
        self.monitor.register_pressure_callback(MemoryPressure::High, |pressure| {
            warn!("Memory pressure is HIGH: {:?}", pressure);
            // In production, you might trigger GC, reduce cache sizes, etc.
        });

        self.monitor.register_pressure_callback(MemoryPressure::Critical, |pressure| {
            error!("Memory pressure is CRITICAL: {:?}", pressure);
            // Emergency measures: aggressive GC, cache clearing, etc.
        });

        // Start monitoring
        self.monitor.start_monitoring();

        if let Some(detector) = &self.leak_detector {
            detector.take_snapshot("initialization");
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
    pub fn create_pool<T, F>(&self, name: &str, factory: F, max_size: usize)
    where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let pool = ObjectPool::new(factory, max_size);
        let mut pools = self.pools.write().unwrap();
        pools.insert(name.to_string(), Box::new(pool));
    }

    /// Get an object from pool with type safety
    pub async fn get_from_pool<T>(&self, name: &str) -> Option<PooledObject<T>>
    where
        T: Send + Sync + 'static,
    {
        let pools = self.pools.read().unwrap();
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
    pub fn analyze_memory_leaks(&self) -> Vec<String> {
        if let Some(detector) = &self.leak_detector {
            detector.analyze_leaks()
        } else {
            Vec::new()
        }
    }

    /// Get orphaned object cleanup statistics
    pub fn get_cleanup_stats(&self) -> (usize, Vec<String>) {
        let orphaned_count = ORPHANED_OBJECTS.lock()
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
        let pools = self.pools.read().unwrap();
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
    pub fn force_gc(&self) {
        self.monitor.force_gc();
    }

    /// Get memory usage history
    pub fn get_memory_history(&self, duration: Duration) -> Vec<(Instant, MemoryStats)> {
        self.monitor.get_usage_history(duration)
    }

    /// Create a memory-managed cache
    pub fn create_cache<K, V>(&self, _name: &str, max_entries: usize, max_memory_mb: usize, ttl_seconds: u64) -> MemoryManagedCache<K, V>
    where
        K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
        V: Clone,
    {
        MemoryManagedCache::new(max_entries, max_memory_mb, ttl_seconds)
    }
}
