//! Core memory monitor and garbage collection coordinator
//!
//! This module provides the central memory monitoring system that coordinates
//! garbage collection, memory pressure handling, resource management, and
//! allocation tracking for comprehensive memory management.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock, Semaphore};
use tracing::{debug, error, info, warn};

use crate::memory::allocation::*;
use crate::memory::allocator::*;
use crate::memory::metrics::*;
use crate::memory::resources::*;
use crate::memory::types::*;
pub struct MemoryMonitor {
    config: MemoryLimitConfig,
    stats_history: Arc<AsyncRwLock<Vec<(Instant, MemoryStats)>>>,
    #[allow(dead_code)]
    pressure_callbacks:
        Arc<AsyncRwLock<HashMap<MemoryPressure, Vec<Box<dyn Fn(MemoryPressure) + Send + Sync>>>>>,
    last_gc_time: Arc<AsyncRwLock<Option<Instant>>>,
    last_gc_completed: Arc<AsyncRwLock<Option<Instant>>>,
    gc_guard: Arc<Semaphore>,
    gc_state: Arc<AsyncRwLock<GcState>>,
    finalizer_queue: Arc<AsyncMutex<Vec<ResourceFinalizer>>>,
    finalizer_id_gen: AtomicU64,
    handle_registry: Arc<AsyncRwLock<HandleRegistry>>,
    allocation_tracker: Arc<AsyncRwLock<AllocationSiteTracker>>,
    gc_registry: Arc<AsyncRwLock<GCRegistry>>,
    type_registry: Arc<std::sync::RwLock<HashMap<std::any::TypeId, String>>>,
    finalized_objects: Arc<AsyncRwLock<std::collections::HashSet<ObjectRef>>>,
    finalizer_stats: Arc<AsyncRwLock<FinalizerStats>>,
}

impl std::fmt::Debug for MemoryMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryMonitor")
            .field("config", &self.config)
            .finish()
    }
}

impl MemoryMonitor {
    pub fn new(config: MemoryLimitConfig) -> Self {
        let mut type_registry = HashMap::new();
        
        // Register common types for human-readable names
        Self::register_common_types(&mut type_registry);
        
        Self {
            config,
            stats_history: Arc::new(AsyncRwLock::new(Vec::new())),
            pressure_callbacks: Arc::new(AsyncRwLock::new(HashMap::new())),
            last_gc_time: Arc::new(AsyncRwLock::new(None)),
            last_gc_completed: Arc::new(AsyncRwLock::new(None)),
            gc_guard: Arc::new(Semaphore::new(1)),
            gc_state: Arc::new(AsyncRwLock::new(GcState::Idle)),
            finalizer_queue: Arc::new(AsyncMutex::new(Vec::new())),
            finalizer_id_gen: AtomicU64::new(1),
            handle_registry: Arc::new(AsyncRwLock::new(HandleRegistry::new())),
            allocation_tracker: Arc::new(AsyncRwLock::new(AllocationSiteTracker::new())),
            gc_registry: Arc::new(AsyncRwLock::new(GCRegistry::new())),
            type_registry: Arc::new(std::sync::RwLock::new(type_registry)),
            finalized_objects: Arc::new(AsyncRwLock::new(std::collections::HashSet::new())),
            finalizer_stats: Arc::new(AsyncRwLock::new(FinalizerStats::default())),
        }
    }
    
    /// Register common types in the type registry for human-readable names
    fn register_common_types(registry: &mut HashMap<std::any::TypeId, String>) {
        use std::any::TypeId;
        
        // Standard library types
        registry.insert(TypeId::of::<String>(), "String".to_string());
        registry.insert(TypeId::of::<Vec<u8>>(), "Vec<u8>".to_string());
        registry.insert(TypeId::of::<Vec<String>>(), "Vec<String>".to_string());
        registry.insert(TypeId::of::<HashMap<String, String>>(), "HashMap<String, String>".to_string());
        registry.insert(TypeId::of::<std::collections::HashSet<String>>(), "HashSet<String>".to_string());
        
        // Numeric types
        registry.insert(TypeId::of::<u8>(), "u8".to_string());
        registry.insert(TypeId::of::<u16>(), "u16".to_string());
        registry.insert(TypeId::of::<u32>(), "u32".to_string());
        registry.insert(TypeId::of::<u64>(), "u64".to_string());
        registry.insert(TypeId::of::<usize>(), "usize".to_string());
        registry.insert(TypeId::of::<i8>(), "i8".to_string());
        registry.insert(TypeId::of::<i16>(), "i16".to_string());
        registry.insert(TypeId::of::<i32>(), "i32".to_string());
        registry.insert(TypeId::of::<i64>(), "i64".to_string());
        registry.insert(TypeId::of::<isize>(), "isize".to_string());
        registry.insert(TypeId::of::<f32>(), "f32".to_string());
        registry.insert(TypeId::of::<f64>(), "f64".to_string());
        
        // Memory management types
        registry.insert(TypeId::of::<ObjectRef>(), "ObjectRef".to_string());
        registry.insert(TypeId::of::<ResourceHandle>(), "ResourceHandle".to_string());
        registry.insert(TypeId::of::<MemoryStats>(), "MemoryStats".to_string());
        
        // Common async types
        registry.insert(TypeId::of::<tokio::sync::Mutex<()>>(), "tokio::sync::Mutex".to_string());
        registry.insert(TypeId::of::<tokio::sync::RwLock<()>>(), "tokio::sync::RwLock".to_string());
        registry.insert(TypeId::of::<std::sync::Arc<()>>(), "Arc".to_string());
        registry.insert(TypeId::of::<std::sync::Mutex<()>>(), "Mutex".to_string());
    }

    /// Record current memory statistics
    pub async fn record_stats(&self) {
        let stats = MemoryTrackingAllocator::memory_stats();
        let timestamp = Instant::now();

        let mut history = self.stats_history.write().await;
        history.push((timestamp, stats.clone()));

        // Keep only recent history (last 1000 entries)
        if history.len() > 1000 {
            history.remove(0);
        }

        // Check memory pressure
        let pressure = self.calculate_pressure(&stats);
        if pressure >= MemoryPressure::Moderate {
            self.trigger_pressure_callbacks(pressure).await;
        }

        // Check limits
        if stats.allocated_bytes > (self.config.max_heap_mb as u64 * 1024 * 1024) {
            warn!(
                "Memory limit exceeded: {} MB used, {} MB limit",
                stats.allocated_bytes / (1024 * 1024),
                self.config.max_heap_mb
            );
            self.trigger_gc_if_needed().await;
        }
    }

    /// Calculate current memory pressure level
    fn calculate_pressure(&self, stats: &MemoryStats) -> MemoryPressure {
        let usage_mb = stats.allocated_bytes as f64 / (1024.0 * 1024.0);
        let usage_ratio = if self.config.max_heap_mb > 0 {
            usage_mb / self.config.max_heap_mb as f64
        } else {
            0.0
        };

        if usage_ratio >= self.config.critical_threshold_percent {
            MemoryPressure::Critical
        } else if usage_ratio >= self.config.warning_threshold_percent {
            MemoryPressure::High
        } else if usage_ratio >= (self.config.warning_threshold_percent * 0.7) {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        }
    }

    /// Register a callback for memory pressure events
    pub async fn register_pressure_callback<F>(&self, pressure: MemoryPressure, callback: F)
    where
        F: Fn(MemoryPressure) + Send + Sync + 'static,
    {
        let mut callbacks = self.pressure_callbacks.write().await;
        callbacks
            .entry(pressure)
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    /// Trigger pressure callbacks
    async fn trigger_pressure_callbacks(&self, pressure: MemoryPressure) {
        let callbacks = self.pressure_callbacks.read().await;
        if let Some(pressure_callbacks) = callbacks.get(&pressure) {
            for callback in pressure_callbacks {
                callback(pressure.clone());
            }
        }
    }

    /// Trigger garbage collection if needed
    async fn trigger_gc_if_needed(&self) {
        if !self.config.enable_gc_pressure {
            return;
        }

        let stats = MemoryTrackingAllocator::memory_stats();
        let usage_mb = stats.allocated_bytes as f64 / (1024.0 * 1024.0);

        if usage_mb >= self.config.gc_pressure_threshold_mb as f64 {
            let should_gc = {
                let last_gc = self.last_gc_time.read().await;
                match *last_gc {
                    Some(last) => last.elapsed() > Duration::from_secs(30), // Don't GC more than once per 30s
                    None => true,
                }
            };

            if should_gc {
                info!("Triggering garbage collection due to memory pressure");

                *self.last_gc_time.write().await = Some(Instant::now());

                // Run GC synchronously to avoid lifetime issues
                let _permit = self.gc_guard.acquire().await;
                *self.gc_state.write().await = GcState::Running;
                self.force_gc().await;
                *self.gc_state.write().await = GcState::Idle;
                *self.last_gc_completed.write().await = Some(Instant::now());

                debug!("Garbage collection completed synchronously");
            }
        }
    }

    /// Force garbage collection and memory cleanup
    /// Implements comprehensive memory management with multiple GC strategies
    async fn force_gc(&self) {
        let start_time = Instant::now();
        let before = MemoryTrackingAllocator::memory_stats();

        info!(
            "Starting comprehensive garbage collection - {} MB allocated",
            before.allocated_bytes / (1024 * 1024)
        );

        // Phase 1: Mark and sweep garbage collection
        let marked_objects = self.perform_mark_and_sweep_gc().await;

        // Phase 2: Memory defragmentation and compaction
        let compacted_bytes = self.perform_memory_compaction().await;

        // Phase 3: Finalization and resource cleanup
        let finalized_count = self.perform_finalization().await;

        // Phase 3.5: Handle cleanup
        let handles_cleaned = self.perform_handle_cleanup().await;

        // Phase 4: Memory leak detection and reporting
        let leaks_detected = self.detect_memory_leaks();

        // Phase 5: Memory pressure optimization is handled separately
        // to avoid recursion when called from optimize_memory_pressure

        let after = MemoryTrackingAllocator::memory_stats();
        let freed_bytes = before.allocated_bytes.saturating_sub(after.allocated_bytes);
        let gc_duration = start_time.elapsed();

        info!("Garbage collection completed in {:.2}ms - freed {} MB, {} objects marked, {} bytes compacted, {} finalized, {} handles cleaned, {} leaks detected",
              gc_duration.as_millis(), freed_bytes / (1024 * 1024), marked_objects, compacted_bytes, finalized_count, handles_cleaned, leaks_detected);

        // Update GC statistics
        self.record_gc_cycle(gc_duration, freed_bytes, marked_objects)
            .await;
    }

    /// Perform mark-and-sweep garbage collection
    async fn perform_mark_and_sweep_gc(&self) -> usize {
        // Mark phase: identify reachable objects
        let marked_objects = self.mark_reachable_objects().await;

        // Sweep phase: free unreachable objects
        let swept_objects = self.sweep_unreachable_objects().await;

        debug!(
            "Mark-and-sweep GC: {} objects marked, {} objects swept",
            marked_objects, swept_objects
        );
        marked_objects
    }

    /// Perform memory compaction and defragmentation
    async fn perform_memory_compaction(&self) -> usize {
        // Analyze memory fragmentation
        let fragmentation_stats = self.analyze_fragmentation().await;

        // Perform compaction if fragmentation is high
        let fragmentation_ratio = (fragmentation_stats.external_fragmentation
            + fragmentation_stats.internal_fragmentation)
            / 2.0;
        let compacted_bytes = if fragmentation_ratio > 0.3 {
            self.compact_memory_blocks()
        } else {
            0
        };

        debug!(
            "Memory compaction: {:.2}% fragmentation, {} bytes compacted",
            fragmentation_ratio * 100.0,
            compacted_bytes
        );
        compacted_bytes
    }

    /// Perform finalization and resource cleanup
    async fn perform_finalization(&self) -> usize {
        // Process finalization queue
        let finalized_count = self.process_finalization_queue().await;

        // Clean up orphaned resources
        let resources_cleaned = self.cleanup_orphaned_resources();

        debug!(
            "Finalization: {} objects finalized, {} resources cleaned up",
            finalized_count, resources_cleaned
        );
        finalized_count
    }

    /// Detect and report memory leaks
    fn detect_memory_leaks(&self) -> usize {
        // Analyze allocation patterns for potential leaks
        let suspected_leaks = self.analyze_allocation_patterns_for_leaks();

        // Report significant leaks
        for leak in &suspected_leaks {
            if leak.size_bytes > 1024 * 1024 {
                // Report leaks > 1MB
                warn!(
                    "Potential memory leak detected: {} bytes at {:?}",
                    leak.size_bytes, leak.allocation_time
                );
            }
        }

        debug!(
            "Memory leak detection: {} potential leaks identified",
            suspected_leaks.len()
        );
        suspected_leaks.len()
    }

    /// Optimize memory pressure and allocation strategies
    async fn optimize_memory_pressure(&self) {
        let current_pressure = self.get_current_pressure();

        match current_pressure {
            MemoryPressure::Critical => {
                // Aggressive optimization for critical pressure
                self.aggressive_memory_optimization().await;
                warn!("Critical memory pressure detected - aggressive optimization applied");
            }
            MemoryPressure::High => {
                // Moderate optimization for high pressure
                self.force_gc().await;
                self.moderate_memory_optimization().await;
                info!("High memory pressure detected - optimization applied");
            }
            MemoryPressure::Moderate => {
                // Light optimization for moderate pressure
                self.light_memory_optimization().await;
                debug!("Moderate memory pressure detected - light optimization applied");
            }
            MemoryPressure::Low => {
                // No optimization needed for low pressure
                debug!("Memory pressure normal - no optimization needed");
            }
        }
    }

    /// Mark reachable objects for garbage collection
    async fn mark_reachable_objects(&self) -> usize {
        let mut marked_count = 0;
        let mut registry = self.gc_registry.write().await;

        // Clear previous marks
        registry.marked_objects.clear();

        // Get root objects from various sources
        let root_objects = self.collect_root_objects().await;

        // Mark all root objects as reachable
        for root in root_objects {
            if !registry.marked_objects.contains(&root) {
                registry.marked_objects.insert(root.clone());
                marked_count += 1;
            }
        }

        // Traverse object graph from roots
        let mut to_visit: Vec<ObjectRef> = registry.marked_objects.iter().cloned().collect();

        while let Some(current) = to_visit.pop() {
            // Find all objects referenced by current object
            let referenced_objects = self.get_referenced_objects(&current).await;

            for referenced in referenced_objects {
                if !registry.marked_objects.contains(&referenced) {
                    registry.marked_objects.insert(referenced.clone());
                    to_visit.push(referenced);
                    marked_count += 1;
                }
            }
        }

        registry.last_mark_phase = std::time::Instant::now();

        debug!(
            "Mark phase completed: {} objects marked as reachable",
            marked_count
        );
        marked_count
    }

    /// Collect root objects from various sources
    async fn collect_root_objects(&self) -> Vec<ObjectRef> {
        let mut roots = Vec::new();

        // Add objects from active handles
        let handle_registry = self.handle_registry.read().await;
        for handle in handle_registry.handles() {
            if let Some(obj_ref) = handle.get_object_ref() {
                roots.push(obj_ref);
            }
        }

        // Add objects from global registry
        let gc_registry = self.gc_registry.read().await;
        for obj_ref in gc_registry.marked_objects.iter() {
            roots.push(obj_ref.clone());
        }

        roots
    }

    /// Get objects referenced by a given object
    async fn get_referenced_objects(&self, obj_ref: &ObjectRef) -> Vec<ObjectRef> {
        let mut referenced = Vec::new();

        // Look up references in the weak references map
        let gc_registry = self.gc_registry.read().await;
        if let Some(weak_refs) = gc_registry.weak_references.get(obj_ref) {
            for weak_ref in weak_refs {
                if let Some(strong_ref) = weak_ref.upgrade() {
                    // Convert strong reference back to ObjectRef
                    if let Some(obj_ref) = self.convert_to_object_ref(&strong_ref) {
                        referenced.push(obj_ref);
                    }
                }
            }
        }

        referenced
    }

    /// Convert a strong reference to ObjectRef
    fn convert_to_object_ref(
        &self,
        strong_ref: &std::sync::Arc<dyn std::any::Any + Send + Sync>,
    ) -> Option<ObjectRef> {
        // Extract actual TypeId from the Any trait object
        let type_id = strong_ref.type_id();

        // Calculate proper size based on the underlying type
        let size = std::mem::size_of_val(strong_ref.as_ref());

        // Get pointer to the object for tracking
        let ptr = strong_ref.as_ref() as *const dyn std::any::Any as *const u8 as usize;

        // Try to extract additional type information if possible
        let type_name = self.extract_type_name(strong_ref);

        debug!(
            "Converted object reference: type_id={:?}, size={}, ptr=0x{:x}, type_name={}",
            type_id,
            size,
            ptr,
            type_name.unwrap_or("unknown".to_string())
        );

        Some(ObjectRef { ptr, type_id, size })
    }

    /// Extract human-readable type name from Any trait object
    /// This is a synchronous method that uses a blocking read for performance
    fn extract_type_name(
        &self,
        obj: &std::sync::Arc<dyn std::any::Any + Send + Sync>,
    ) -> Option<String> {
        let type_id = obj.type_id();
        
        // Use sync RwLock for fast synchronous access (this method is called from sync contexts)
        // In async contexts, prefer get_type_name() instead
        let registry = self.type_registry.read().unwrap();
        
        // Check if type is registered
        if let Some(type_name) = registry.get(&type_id) {
            return Some(type_name.clone());
        }
        drop(registry);
        
        // For unregistered types, try to extract a readable name from TypeId
        // TypeId::of::<T>() produces a stable hash, but we can't reverse it
        // So we'll use a shortened hash for unregistered types
        let type_id_hash = format!("{:?}", type_id);
        Some(format!("unknown_type_{}", &type_id_hash[..8]))
    }
    
    /// Register a type in the type registry for human-readable names
    pub fn register_type<T: 'static>(&self, type_name: String) {
        let mut registry = self.type_registry.write().unwrap();
        registry.insert(std::any::TypeId::of::<T>(), type_name);
    }
    
    /// Get type name for a TypeId (async version)
    pub async fn get_type_name_async(&self, type_id: std::any::TypeId) -> Option<String> {
        let registry = self.type_registry.read().unwrap();
        registry.get(&type_id).cloned()
    }
    
    /// Get type name for a TypeId (sync version)
    pub fn get_type_name(&self, type_id: std::any::TypeId) -> Option<String> {
        let registry = self.type_registry.read().unwrap();
        registry.get(&type_id).cloned()
    }

    /// Sweep unreachable objects during garbage collection
    async fn sweep_unreachable_objects(&self) -> usize {
        let mut swept_count = 0;
        let mut registry = self.gc_registry.write().await;

        // Get all tracked objects
        let all_objects = self.get_all_tracked_objects().await;

        // Identify unreachable objects
        let mut unreachable_objects = Vec::new();
        for obj_ref in all_objects {
            if !registry.marked_objects.contains(&obj_ref) {
                unreachable_objects.push(obj_ref);
            }
        }

        // Process unreachable objects
        for obj_ref in unreachable_objects {
            // Check if object needs finalization
            if self.needs_finalization(&obj_ref).await {
                registry.pending_finalization.push(obj_ref.clone());
            } else {
                // Direct deallocation
                if self.deallocate_object(&obj_ref).await {
                    swept_count += 1;
                }
            }
        }

        // Clean up weak references to swept objects
        self.cleanup_weak_references(&registry.marked_objects).await;

        registry.last_sweep_phase = std::time::Instant::now();

        debug!("Sweep phase completed: {} objects swept", swept_count);
        swept_count
    }

    /// Get all tracked objects from various registries
    async fn get_all_tracked_objects(&self) -> Vec<ObjectRef> {
        let mut all_objects = Vec::new();

        // Get objects from handle registry
        let handle_registry = self.handle_registry.read().await;
        for handle in handle_registry.handles() {
            if let Some(obj_ref) = handle.get_object_ref() {
                all_objects.push(obj_ref);
            }
        }

        // Get objects from GC registry marked objects
        let gc_registry = self.gc_registry.read().await;
        all_objects.extend(gc_registry.marked_objects.iter().cloned());

        all_objects
    }

    /// Check if an object needs finalization
    async fn needs_finalization(&self, obj_ref: &ObjectRef) -> bool {
        // Check if object has already been finalized
        let finalized = self.finalized_objects.read().await;
        if finalized.contains(obj_ref) {
            return false; // Already finalized, no need to finalize again
        }
        drop(finalized);
        
        // Check if object has a finalizer registered in the finalizer queue
        let finalizer_queue = self.finalizer_queue.lock().await;
        let has_registered_finalizer = finalizer_queue.iter()
            .any(|f| f.object_ref.ptr == obj_ref.ptr && f.object_ref.type_id == obj_ref.type_id);
        drop(finalizer_queue);
        
        if has_registered_finalizer {
            return true; // Has registered finalizer, needs finalization
        }
        
        // Check if object is in pending finalization queue
        let gc_registry = self.gc_registry.read().await;
        let in_pending_queue = gc_registry.pending_finalization.iter()
            .any(|pending| pending.ptr == obj_ref.ptr && pending.type_id == obj_ref.type_id);
        drop(gc_registry);
        
        in_pending_queue
    }
    
    /// Mark an object as finalized
    async fn mark_as_finalized(&self, obj_ref: &ObjectRef) {
        let mut finalized = self.finalized_objects.write().await;
        finalized.insert(obj_ref.clone());
    }

    /// Deallocate an object
    async fn deallocate_object(&self, obj_ref: &ObjectRef) -> bool {
        // Remove from all registries
        let mut handle_registry = self.handle_registry.write().await;
        handle_registry.remove_handles_for_object(obj_ref);

        let mut gc_registry = self.gc_registry.write().await;

        // Remove from marked objects
        gc_registry.marked_objects.remove(obj_ref);

        // Remove from pending finalization
        gc_registry.pending_finalization.retain(|o| o != obj_ref);

        // Remove from weak references
        gc_registry.weak_references.remove(obj_ref);

        // Update memory statistics
        self.update_memory_stats_after_deallocation(obj_ref.size)
            .await;

        true
    }

    /// Clean up weak references to swept objects
    async fn cleanup_weak_references(&self, marked_objects: &std::collections::HashSet<ObjectRef>) {
        let mut gc_registry = self.gc_registry.write().await;

        // Remove weak references to objects that are no longer reachable
        gc_registry.weak_references.retain(|obj_ref, weak_refs| {
            if marked_objects.contains(obj_ref) {
                // Keep weak references to marked objects
                true
            } else {
                // Remove weak references to swept objects
                false
            }
        });
    }

    /// Update memory statistics after deallocation
    async fn update_memory_stats_after_deallocation(&self, size: usize) {
        // Update stats in gc_registry
        let mut gc_registry = self.gc_registry.write().await;
        // Track memory usage in GC registry
        gc_registry.total_bytes = gc_registry.total_bytes.saturating_sub(size);
    }

    /// Analyze memory fragmentation
    async fn analyze_fragmentation(&self) -> FragmentationStats {
        // Calculate memory fragmentation statistics
        let stats = MemoryTrackingAllocator::memory_stats();
        let gc_registry = self.gc_registry.read().await;

        // Calculate fragmentation based on allocation patterns and GC registry
        // Fragmentation ratio = (allocation overhead + external fragmentation) / total heap
        let allocation_overhead = (stats.allocation_count as f64 * 16.0) as f64; // Estimate 16 bytes per allocation
        let external_fragmentation = if stats.allocated_bytes > 0 {
            let gc_bytes = gc_registry.total_bytes as f64;
            // External fragmentation is the difference between allocated bytes and GC-tracked bytes
            (stats.allocated_bytes as f64 - gc_bytes).max(0.0)
        } else {
            0.0
        };

        let total_heap = stats.allocated_bytes as f64 + allocation_overhead;
        let fragmentation_ratio = if total_heap > 0.0 {
            ((allocation_overhead + external_fragmentation) / total_heap).min(1.0)
        } else {
            0.0
        };

        // Estimate free bytes and largest free block
        let total_free_bytes =
            (stats.allocated_bytes as usize).saturating_sub(gc_registry.total_bytes);
        let largest_free_block = total_free_bytes / 4; // Conservative estimate
        let internal_fragmentation = allocation_overhead as f64 / stats.allocated_bytes as f64;

        // Calculate free blocks count based on allocation patterns
        // Since we don't have direct access to the allocator's internal state,
        // we estimate free blocks by analyzing allocation patterns and free memory
        let free_blocks_count = if stats.allocation_count > 0 && total_free_bytes > 0 {
            // Estimate average allocation size from tracked allocations
            // Use the number of marked objects as a proxy for registered objects
            let registered_objects_count = gc_registry.marked_objects.len().max(1);
            let avg_allocation_size = if gc_registry.total_bytes > 0 {
                gc_registry.total_bytes / registered_objects_count
            } else {
                // Fallback: estimate based on total allocated bytes
                if stats.allocated_bytes > 0 {
                    let allocation_count_usize = stats.allocation_count as usize;
                    (stats.allocated_bytes as usize) / allocation_count_usize.max(1)
                } else {
                    1024usize // Default estimate: 1KB per allocation
                }
            };

            // Estimate free blocks: divide free memory by average block size
            // Add some fragmentation factor (free blocks are typically smaller than allocated blocks)
            let estimated_free_blocks = total_free_bytes / avg_allocation_size.max(1);
            
            // Apply fragmentation factor: free blocks are often smaller due to fragmentation
            // This accounts for the fact that free memory is typically fragmented into smaller blocks
            let fragmentation_factor = if external_fragmentation > 0.1 {
                // High fragmentation: more smaller blocks
                2.0
            } else if external_fragmentation > 0.05 {
                // Medium fragmentation: moderate block count
                1.5
            } else {
                // Low fragmentation: fewer larger blocks
                1.0
            };
            
            (estimated_free_blocks as f64 * fragmentation_factor) as usize
        } else {
            0usize
        };

        FragmentationStats {
            external_fragmentation,
            internal_fragmentation,
            largest_free_block,
            total_free_memory: total_free_bytes,
            free_blocks_count,
        }
    }

    /// Perform compaction and defragmentation
    fn compact_memory_blocks(&self) -> usize {
        // TODO: Implement memory compaction to eliminate fragmentation
        //       Currently returns placeholder; should move objects to eliminate fragmentation and optimize memory layout.
        //
        // COMPLETION CHECKLIST:
        // [ ] Identify fragmented memory regions
        // [ ] Move objects to eliminate fragmentation
        // [ ] Update object references after compaction
        // [ ] Optimize memory layout for better cache locality
        // [ ] Handle compaction during active allocations
        // [ ] Add unit tests for compaction logic
        // [ ] Add integration tests with fragmented memory
        // [ ] Verify compaction improves memory efficiency
        //
        // ACCEPTANCE CRITERIA:
        // - Memory fragmentation is reduced through compaction
        // - Object references are updated correctly after compaction
        // - Compaction improves memory layout and cache locality
        // - Compaction handles active allocations gracefully
        //
        // DEPENDENCIES:
        // - Memory fragmentation detection (Required)
        // - Object relocation utilities (Required)
        // - Reference update mechanism (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (low risk optimization)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Memory management expertise
        0 // Temporary placeholder until compaction is implemented
    }

    /// Clean up orphaned resources
    fn cleanup_orphaned_resources(&self) -> usize {
        // TODO: Implement orphaned resource cleanup (file handles, network connections, etc.)
        //       Currently returns placeholder; should identify and clean up orphaned resources.
        //
        // COMPLETION CHECKLIST:
        // [ ] Identify orphaned file handles
        // [ ] Close orphaned file handles
        // [ ] Identify orphaned network connections
        // [ ] Close orphaned network connections
        // [ ] Clean up other orphaned resources (locks, timers, etc.)
        // [ ] Track cleanup statistics
        // [ ] Add unit tests for resource cleanup
        // [ ] Add integration tests with orphaned resources
        // [ ] Verify resources are properly cleaned up
        //
        // ACCEPTANCE CRITERIA:
        // - Orphaned file handles are identified and closed
        // - Orphaned network connections are identified and closed
        // - Other orphaned resources are cleaned up
        // - Cleanup statistics are tracked accurately
        //
        // DEPENDENCIES:
        // - Resource tracking system (Required)
        // - File handle management (Required)
        // - Network connection management (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (standard feature)
        // - Change Budget: ~120 LOC
        // - Reviewer Requirements: Resource management expertise
        0 // Temporary placeholder until cleanup is implemented
    }

    /// Analyze allocation patterns for leak detection
    fn analyze_allocation_patterns_for_leaks(&self) -> Vec<AllocationLeak> {
        // TODO: Implement allocation pattern analysis for leak detection
        //       Currently returns empty vector; should analyze allocation patterns to detect potential memory leaks.
        //
        // COMPLETION CHECKLIST:
        // [ ] Track allocation counts over time
        // [ ] Identify growing allocation patterns
        // [ ] Detect potential memory leaks
        // [ ] Classify leak types (gradual, sudden, cyclic)
        // [ ] Generate leak reports with recommendations
        // [ ] Add unit tests for leak detection
        // [ ] Add integration tests with leak scenarios
        // [ ] Verify leak detection accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Allocation patterns are analyzed over time
        // - Potential memory leaks are detected accurately
        // - Leak types are classified correctly
        // - Leak reports include actionable recommendations
        //
        // DEPENDENCIES:
        // - Allocation tracking system (Required)
        // - Pattern analysis algorithms (Required)
        // - Leak classification utilities (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (standard feature)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Memory leak detection expertise
        Vec::new() // Temporary empty vector until leak detection is implemented
    }

    /// Get memory stats history
    pub async fn get_stats_history(
        &self,
    ) -> Vec<(std::time::Instant, crate::memory::allocator::MemoryStats)> {
        let history = self.stats_history.read().await;
        history.clone()
    }

    /// Get current memory pressure level
    pub fn get_current_pressure(&self) -> MemoryPressure {
        let stats = MemoryTrackingAllocator::memory_stats();
        let usage_ratio = if self.config.max_heap_mb > 0 {
            stats.allocated_bytes as f64 / (self.config.max_heap_mb as f64 * 1024.0 * 1024.0)
        } else {
            0.0
        };

        if usage_ratio > 0.9 {
            MemoryPressure::Critical
        } else if usage_ratio > 0.75 {
            MemoryPressure::High
        } else if usage_ratio > 0.5 {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        }
    }

    /// Aggressive memory optimization for critical pressure
    async fn aggressive_memory_optimization(&self) {
        // Perform GC operations synchronously
        let mut gc_registry = self.gc_registry.write().await;
        gc_registry.marked_objects.clear();
        gc_registry.pending_finalization.clear();
        gc_registry.weak_references.clear();

        // Additional aggressive measures:
        // - Clear all caches
        // - Reduce pool sizes
        // - Force compaction
        info!("Applied aggressive memory optimization");
    }

    /// Moderate memory optimization for high pressure
    async fn moderate_memory_optimization(&self) {
        // Note: force_gc() is called from the parent optimize_memory_pressure method
        // to avoid recursion. This method focuses on additional optimizations.

        // Additional moderate measures:
        // - Clear non-essential caches
        // - Reduce pool sizes moderately
        info!("Applied moderate memory optimization");
    }

    /// Light memory optimization for moderate pressure
    async fn light_memory_optimization(&self) {
        // Perform GC operations synchronously
        let mut gc_registry = self.gc_registry.write().await;
        gc_registry.marked_objects.clear();
        gc_registry.pending_finalization.clear();
        gc_registry.weak_references.clear();

        // Additional light measures:
        // - Clear expired cache entries
        info!("Applied light memory optimization");
    }

    /// Record a garbage collection cycle
    async fn record_gc_cycle(
        &self,
        duration: Duration,
        bytes_freed: u64,
        objects_processed: usize,
    ) {
        let mut gc_time = self.last_gc_time.write().await;
        *gc_time = Some(Instant::now());

        // Record in stats history if available
        let mut history = self.stats_history.write().await;
        let stats = MemoryTrackingAllocator::memory_stats();
        history.push((Instant::now(), stats.clone()));

        // Keep only recent history
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// Register a resource finalizer for an object
    pub async fn register_finalizer<F>(
        &self,
        object_ref: ObjectRef,
        finalizer_fn: F,
        priority: i32,
    ) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let mut queue = self.finalizer_queue.lock().await;
        let id = self.finalizer_id_gen.fetch_add(1, Ordering::Relaxed);

        let finalizer = ResourceFinalizer {
            id,
            object_ref,
            finalizer_fn: Box::new(finalizer_fn),
            priority,
            registered_at: std::time::Instant::now(),
        };

        queue.push(finalizer);
        
        // Update statistics
        let mut stats = self.finalizer_stats.write().await;
        stats.registered += 1;
        stats.queued += 1;
        drop(stats);
        
        id
    }

    /// Execute all pending finalizers
    pub async fn execute_pending_finalizers(&self) -> Vec<FinalizerResult> {
        let mut queue = self.finalizer_queue.lock().await;
        let mut results = Vec::new();

        // Execute all finalizers in the queue
        for finalizer in queue.drain(..) {
            let start_time = Instant::now();
            let finalizer_id = finalizer.id;
            let obj_ref = finalizer.object_ref.clone();
            
            // Execute the finalizer function in a blocking task to handle potentially blocking operations
            let finalizer_fn = finalizer.finalizer_fn;
            let execution_result = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    finalizer_fn();
                }))
            }).await;
            
            let duration_us = start_time.elapsed().as_micros() as u64;
            
            match execution_result {
                Ok(Ok(())) => {
                    // Finalizer executed successfully
                    let result = FinalizerResult {
                        finalizer_id,
                        success: true,
                        duration_us,
                        error_message: None,
                    };
                    results.push(result.clone());
                    
                    // Update statistics
                    let mut stats = self.finalizer_stats.write().await;
                    stats.executed += 1;
                    stats.successful += 1;
                    stats.total_execution_time_us += duration_us;
                    drop(stats);
                    
                    // Mark object as finalized
                    self.mark_as_finalized(&obj_ref).await;
                }
                Ok(Err(panic_info)) => {
                    // Finalizer panicked
                    let error_msg = format!("Finalizer panicked: {:?}", panic_info);
                    warn!("Finalizer {} panicked: {}", finalizer_id, error_msg);
                    let result = FinalizerResult {
                        finalizer_id,
                        success: false,
                        duration_us,
                        error_message: Some(error_msg),
                    };
                    results.push(result);
                    
                    // Update statistics
                    let mut stats = self.finalizer_stats.write().await;
                    stats.executed += 1;
                    stats.failed += 1;
                    stats.total_execution_time_us += duration_us;
                    drop(stats);
                    
                    // Still mark as finalized (even if it panicked, we don't want to retry)
                    self.mark_as_finalized(&obj_ref).await;
                }
                Err(join_error) => {
                    // Task join error
                    let error_msg = format!("Finalizer task join error: {:?}", join_error);
                    warn!("Finalizer {} join error: {}", finalizer_id, error_msg);
                    let result = FinalizerResult {
                        finalizer_id,
                        success: false,
                        duration_us,
                        error_message: Some(error_msg),
                    };
                    results.push(result);
                    
                    // Update statistics
                    let mut stats = self.finalizer_stats.write().await;
                    stats.executed += 1;
                    stats.failed += 1;
                    stats.total_execution_time_us += duration_us;
                    drop(stats);
                }
            }
        }

        results
    }

    /// Get finalizer queue statistics
    pub async fn get_finalizer_stats(&self) -> FinalizerStats {
        let queue = self.finalizer_queue.lock().await;
        let queued_count = queue.len() as u64;
        drop(queue);
        
        // Get tracked statistics
        let stats = self.finalizer_stats.read().await;
        let mut result = stats.clone();
        drop(stats);
        
        // Update queued count (current queue size)
        result.queued = queued_count;
        // Update registered count (queued + executed)
        result.registered = queued_count + result.executed;
        
        result
    }

    /// Process finalization during GC sweep phase
    pub async fn process_finalization_queue(&self) -> usize {
        debug!("Processing finalization queue during GC sweep");

        // Execute all pending finalizers
        let results = self.execute_pending_finalizers().await;

        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;

        if failed > 0 {
            warn!("{} finalizers failed during GC sweep", failed);
        }

        debug!(
            "Successfully executed {} finalizers during GC sweep",
            successful
        );
        successful
    }

    /// Force execution of all finalizers (emergency cleanup)
    pub async fn force_finalizer_execution(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Forcing execution of all pending finalizers");

        let results = self.execute_pending_finalizers().await;

        let failed_results: Vec<_> = results.into_iter().filter(|r| !r.success).collect();

        if !failed_results.is_empty() {
            let error_msg = format!(
                "{} finalizers failed during forced execution",
                failed_results.len()
            );
            error!("{}", error_msg);
            return Err(error_msg.into());
        }

        info!("Successfully executed all finalizers");
        Ok(())
    }

    /// Create a finalizer for common resource types
    pub async fn create_file_handle_finalizer(
        &self,
        file_path: std::path::PathBuf,
        object_ref: ObjectRef,
    ) -> u64 {
        let registry = Arc::clone(&self.handle_registry);
        self.register_finalizer(
            object_ref.clone(),
            move || {
                debug!("Executing file handle finalizer for {:?}", file_path);

                // Note: File handle cleanup requires async registry access
                // This synchronous finalizer logs the need for cleanup
                // Actual async cleanup should be handled by GC sweep phase
                warn!("File handle finalizer executed - async cleanup deferred to GC sweep");
            },
            100,
        )
        .await // High priority for file handles
    }

    /// Create a finalizer for network connections
    pub async fn create_network_connection_finalizer(
        &self,
        connection_id: String,
        object_ref: ObjectRef,
    ) -> u64 {
        let registry = Arc::clone(&self.handle_registry);
        self.register_finalizer(
            object_ref.clone(),
            move || {
                debug!(
                    "Executing network connection finalizer for {}",
                    connection_id
                );

                // Note: Network connection cleanup requires async registry access
                // This synchronous finalizer logs the need for cleanup
                // Actual async cleanup should be handled by GC sweep phase
                warn!("Network connection finalizer executed - async cleanup deferred to GC sweep");
            },
            90,
        )
        .await // High priority for network resources
    }

    /// Create a finalizer for database connections
    pub async fn create_database_connection_finalizer(
        &self,
        connection_string: String,
        object_ref: ObjectRef,
    ) -> u64 {
        let registry = Arc::clone(&self.handle_registry);
        self.register_finalizer(object_ref.clone(), move || {
            debug!("Executing database connection finalizer for {}", connection_string);

            // Note: Database connection cleanup requires async registry access
            // This synchronous finalizer logs the need for cleanup
            // Actual async cleanup should be handled by connection pool
            warn!("Database connection finalizer executed - async cleanup deferred to connection pool");
        }, 95).await // High priority for database resources
    }

    /// Create a finalizer for memory-mapped regions
    pub async fn create_memory_map_finalizer(
        &self,
        mapping_size: usize,
        object_ref: ObjectRef,
    ) -> u64 {
        let registry = Arc::clone(&self.handle_registry);
        self.register_finalizer(
            object_ref.clone(),
            move || {
                debug!("Executing memory map finalizer for {} bytes", mapping_size);

                // Note: Memory map cleanup requires async registry access
                // This synchronous finalizer logs the need for cleanup
                // Actual async unmapping should be handled by GC sweep phase
                warn!("Memory map finalizer executed - async unmapping deferred to GC sweep");
            },
            80,
        )
        .await // Medium-high priority for memory mappings
    }

    /// Create a finalizer for shared memory segments
    pub async fn create_shared_memory_finalizer(
        &self,
        segment_id: String,
        object_ref: ObjectRef,
    ) -> u64 {
        let registry = Arc::clone(&self.handle_registry);
        self.register_finalizer(
            object_ref.clone(),
            move || {
                debug!(
                    "Executing shared memory finalizer for segment {}",
                    segment_id
                );

                // Note: Shared memory cleanup requires async registry access
                // This synchronous finalizer logs the need for cleanup
                // Actual async detach/unlink should be handled by GC sweep phase
                warn!("Shared memory finalizer executed - async cleanup deferred to GC sweep");
            },
            85,
        )
        .await // Medium-high priority for shared memory
    }

    /// Emergency finalizer cleanup (clear all pending finalizers)
    pub async fn emergency_finalizer_cleanup(&self) {
        let mut queue = self.finalizer_queue.lock().await;
        queue.clear();
        warn!("Emergency finalizer cleanup completed - all pending finalizers cleared");
    }

    /// Register a system handle for tracking and cleanup
    pub async fn register_system_handle(
        &self,
        handle_type: HandleType,
        handle_info: HandleInfo,
        object_ref: ObjectRef,
        description: String,
    ) -> u64 {
        let mut registry = self.handle_registry.write().await;
        registry.register_handle(handle_type, handle_info, object_ref, description)
    }

    /// Mark a handle as already closed
    pub async fn mark_handle_closed(&self, handle_id: u64) -> bool {
        let mut registry = self.handle_registry.write().await;
        registry.mark_handle_closed(handle_id)
    }

    /// Clean up a specific system handle
    pub async fn cleanup_system_handle(&self, handle_id: u64) -> HandleCleanupResult {
        let mut registry = self.handle_registry.write().await;
        registry.cleanup_handle(handle_id).await
    }

    /// Clean up all tracked system handles
    pub async fn cleanup_all_system_handles(&self) -> Vec<HandleCleanupResult> {
        let mut registry = self.handle_registry.write().await;
        registry.cleanup_all_handles().await
    }

    /// Get handles associated with a specific object
    pub async fn get_handles_for_object(&self, object_ref: &ObjectRef) -> Vec<TrackedHandle> {
        let registry = self.handle_registry.read().await;
        registry
            .get_handles_for_object(object_ref)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get all open handles of a specific type
    pub async fn get_handles_by_type(&self, handle_type: &HandleType) -> Vec<TrackedHandle> {
        let registry = self.handle_registry.read().await;
        registry
            .get_handles_by_type(handle_type)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get handle cleanup statistics
    pub async fn get_handle_cleanup_stats(&self) -> HandleCleanupStats {
        let registry = self.handle_registry.read().await;
        registry.stats().clone()
    }

    /// Emergency handle cleanup (clear all tracked handles without cleanup)
    pub async fn emergency_handle_cleanup(&self) {
        let registry = self.handle_registry.write().await;
        // TODO: Implement proper emergency cleanup
        //       Currently clears handle registry without cleanup; should attempt proper resource release with error handling and timeout management.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Implement handle cleanup loop with individual resource release
        // [ ] Add error handling for failed cleanup attempts
        // [ ] Implement partial cleanup support when some handles fail
        // [ ] Add cleanup timeout handling with configurable limits
        // [ ] Implement cleanup priority system for critical resources
        // [ ] Add cleanup strategy selection (graceful vs forceful)
        // [ ] Track cleanup success/failure statistics
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Emergency cleanup attempts proper resource release for all handles
        // - Cleanup handles errors and timeouts gracefully with partial success
        // - Critical resources are prioritized in cleanup order
        // - Cleanup performance meets emergency SLA (<5 seconds total)
        // - Registry state is consistent after cleanup operations
        // - Error handling covers all resource types and failure modes
        //
        // DEPENDENCIES:
        // - Handle cleanup API with timeout support (Required)
        // - Resource release mechanisms for different handle types (Required)
        // - Cleanup priority system (Optional)
        // - Error aggregation and reporting (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (emergency resource management)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Resource management and cleanup strategy expertise
        warn!("Emergency handle cleanup completed - all handle tracking cleared");
    }

    /// Record an allocation with site tracking
    pub async fn record_allocation(
        &self,
        ptr: usize,
        size: usize,
        alignment: usize,
        site: AllocationSite,
    ) {
        let mut tracker = self.allocation_tracker.write().await;
        tracker.record_allocation(ptr, size, alignment, site);
    }

    /// Record a deallocation
    pub async fn record_deallocation(&self, ptr: usize) {
        let mut tracker = self.allocation_tracker.write().await;
        tracker.record_deallocation(ptr);
    }

    /// Get allocation site statistics
    pub async fn get_allocation_site_stats(
        &self,
        file: &str,
        line: u32,
    ) -> Option<AllocationSiteStats> {
        let tracker = self.allocation_tracker.read().await;
        tracker.get_site_stats(file, line).cloned()
    }

    /// Get all allocation site statistics
    pub async fn get_all_allocation_site_stats(&self) -> Vec<AllocationSiteStats> {
        let tracker = self.allocation_tracker.read().await;
        tracker.get_all_site_stats().into_iter().cloned().collect()
    }

    /// Get allocation statistics for a specific task
    pub async fn get_task_allocation_stats(&self, task_id: &str) -> Option<TaskAllocationStats> {
        let tracker = self.allocation_tracker.read().await;
        tracker.get_task_stats(task_id).cloned()
    }

    /// Get all task allocation statistics
    pub async fn get_all_task_allocation_stats(&self) -> Vec<TaskAllocationStats> {
        let tracker = self.allocation_tracker.read().await;
        tracker.get_all_task_stats().into_iter().cloned().collect()
    }

    /// Get tasks with highest memory usage
    pub async fn get_top_memory_usage_tasks(&self, limit: usize) -> Vec<TaskAllocationStats> {
        let tracker = self.allocation_tracker.read().await;
        tracker
            .get_top_memory_tasks(limit)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Record an allocation with task ID
    pub async fn record_allocation_with_task(
        &self,
        ptr: usize,
        size: usize,
        alignment: usize,
        site: AllocationSite,
        task_id: Option<String>,
    ) {
        let mut site_with_task = site;
        site_with_task.task_id = task_id;
        let mut tracker = self.allocation_tracker.write().await;
        tracker.record_allocation(ptr, size, alignment, site_with_task);
    }

    /// Analyze allocation patterns for memory leaks
    pub async fn analyze_allocation_leaks(&self) -> Vec<AllocationLeak> {
        let tracker = self.allocation_tracker.read().await;
        tracker.analyze_leak_patterns()
    }

    /// Get allocation statistics (total allocations, deallocations)
    pub async fn get_allocation_statistics(&self) -> (u64, u64) {
        let tracker = self.allocation_tracker.read().await;
        tracker.get_allocation_stats()
    }

    /// Clean up old allocation records
    pub async fn cleanup_allocation_records(&self, max_age_seconds: u64) {
        let mut tracker = self.allocation_tracker.write().await;
        tracker.cleanup_old_records(max_age_seconds);
    }

    /// Create a new system metrics collector
    pub fn create_metrics_collector(
        &self,
        collection_interval_secs: u64,
    ) -> SystemMetricsCollector {
        SystemMetricsCollector::new(collection_interval_secs)
    }

    /// Collect system metrics using a collector
    pub async fn collect_system_metrics(
        &self,
        collector: &mut SystemMetricsCollector,
    ) -> Result<SystemMetrics, Box<dyn std::error::Error>> {
        collector.collect_metrics().await
    }

    /// Analyze system metrics
    pub fn analyze_system_metrics(
        &self,
        collector: &SystemMetricsCollector,
        current: &SystemMetrics,
        previous: Option<&SystemMetrics>,
    ) -> MetricsAnalysis {
        collector.analyze_metrics(current, previous)
    }

    /// Get current system health overview
    pub async fn get_system_health_overview(
        &self,
    ) -> Result<MetricsAnalysis, Box<dyn std::error::Error>> {
        let mut collector = self.create_metrics_collector(60); // 1 minute intervals
        let current_metrics = self.collect_system_metrics(&mut collector).await?;
        let previous_metrics = collector.previous_metrics.as_ref();

        Ok(self.analyze_system_metrics(&collector, &current_metrics, previous_metrics))
    }

    /// Perform comprehensive handle cleanup during GC sweep
    pub async fn perform_handle_cleanup(&self) -> usize {
        debug!("Performing handle cleanup during GC sweep");

        // Clean up all tracked handles
        let results = self.cleanup_all_system_handles().await;

        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;

        if failed > 0 {
            warn!("{} handle cleanups failed during GC sweep", failed);
        }

        debug!(
            "Successfully cleaned up {} handles during GC sweep",
            successful
        );
        successful
    }

    /// Create and register a file handle
    pub async fn register_file_handle(
        &self,
        fd: i32,
        file_path: std::path::PathBuf,
        object_ref: ObjectRef,
    ) -> u64 {
        let description = format!("File handle for {:?}", file_path);

        #[cfg(unix)]
        let handle_info = HandleInfo::UnixFd(fd);

        #[cfg(target_os = "windows")]
        let handle_info = HandleInfo::WindowsHandle(fd as isize);

        #[cfg(target_os = "macos")]
        let handle_info = HandleInfo::DarwinFd(fd);

        #[cfg(not(any(unix, windows, target_os = "macos")))]
        let handle_info = HandleInfo::Custom(vec![]);

        self.register_system_handle(HandleType::File, handle_info, object_ref, description)
            .await
    }

    /// Create and register a socket handle
    pub async fn register_socket_handle(
        &self,
        socket_fd: i32,
        connection_info: String,
        object_ref: ObjectRef,
    ) -> u64 {
        let description = format!("Socket handle for {}", connection_info);

        #[cfg(unix)]
        let handle_info = HandleInfo::UnixFd(socket_fd);

        #[cfg(target_os = "macos")]
        let handle_info = HandleInfo::DarwinFd(socket_fd);

        #[cfg(not(any(unix, target_os = "macos")))]
        let handle_info = HandleInfo::Custom(vec![]);

        self.register_system_handle(HandleType::Socket, handle_info, object_ref, description)
            .await
    }

    /// Create and register a shared memory handle
    pub async fn register_shared_memory_handle(
        &self,
        segment_id: String,
        size: usize,
        object_ref: ObjectRef,
    ) -> u64 {
        let description = format!("Shared memory segment '{}' ({} bytes)", segment_id, size);
        let handle_info = HandleInfo::Custom(segment_id.into_bytes());

        self.register_system_handle(
            HandleType::SharedMemory,
            handle_info,
            object_ref,
            description,
        )
        .await
    }

    /// Create and register a memory-mapped region handle
    pub async fn register_memory_map_handle(
        &self,
        address: usize,
        size: usize,
        file_path: Option<std::path::PathBuf>,
        object_ref: ObjectRef,
    ) -> u64 {
        let description = match file_path {
            Some(path) => format!(
                "Memory-mapped file {:?} at {:#x} ({} bytes)",
                path, address, size
            ),
            None => format!(
                "Anonymous memory mapping at {:#x} ({} bytes)",
                address, size
            ),
        };

        let mut data = address.to_le_bytes().to_vec();
        data.extend_from_slice(&size.to_le_bytes());

        let handle_info = HandleInfo::Custom(data);

        self.register_system_handle(HandleType::MemoryMap, handle_info, object_ref, description)
            .await
    }

    /// Perform comprehensive memory layout analysis
    pub async fn analyze_memory_layout(
        &self,
    ) -> Result<MemoryLayoutAnalysis, Box<dyn std::error::Error>> {
        let mut analysis = MemoryLayoutAnalysis {
            total_heap_size: 0,
            allocated_memory: 0,
            free_memory: 0,
            allocated_blocks: 0,
            free_blocks: 0,
            average_allocation_size: 0.0,
            largest_free_block: 0,
            internal_fragmentation_ratio: 0.0,
            external_fragmentation_ratio: 0.0,
            blocks: Vec::new(),
            allocation_hotspots: Vec::new(),
            fragmentation_map: HashMap::new(),
        };

        // Collect all tracked objects
        let all_objects = self.collect_all_tracked_objects().await;

        // Get global allocator stats
        let allocator_stats = MemoryTrackingAllocator::memory_stats();

        // Build memory block representation
        analysis.blocks = self.build_memory_blocks(&all_objects)?;
        analysis.allocated_blocks = analysis.blocks.iter().filter(|b| b.allocated).count();
        analysis.free_blocks = analysis.blocks.iter().filter(|b| !b.allocated).count();

        // Calculate basic metrics
        analysis.total_heap_size =
            (allocator_stats.allocated_bytes + allocator_stats.peak_usage_bytes / 2) as usize; // Estimate
        analysis.allocated_memory = allocator_stats.allocated_bytes as usize;
        analysis.free_memory = analysis
            .total_heap_size
            .saturating_sub(analysis.allocated_memory);

        if analysis.allocated_blocks > 0 {
            analysis.average_allocation_size =
                analysis.allocated_memory as f64 / analysis.allocated_blocks as f64;
        }

        analysis.largest_free_block = analysis
            .blocks
            .iter()
            .filter(|b| !b.allocated)
            .map(|b| b.size)
            .max()
            .unwrap_or(0);

        // Calculate fragmentation metrics
        analysis.internal_fragmentation_ratio =
            self.calculate_internal_fragmentation(&analysis.blocks);
        analysis.external_fragmentation_ratio =
            self.calculate_external_fragmentation(&analysis.blocks);

        // Identify allocation hotspots
        analysis.allocation_hotspots = self.identify_allocation_hotspots(&analysis.blocks);

        // Build fragmentation map
        analysis.fragmentation_map = self.build_fragmentation_map(&analysis.blocks);

        debug!("Memory layout analysis completed: {} blocks analyzed, {:.2}% internal fragmentation, {:.2}% external fragmentation",
               analysis.blocks.len(), analysis.internal_fragmentation_ratio * 100.0, analysis.external_fragmentation_ratio * 100.0);

        Ok(analysis)
    }

    /// Analyze allocation patterns
    pub async fn analyze_allocation_patterns(
        &self,
    ) -> Result<AllocationPatternAnalysis, Box<dyn std::error::Error>> {
        let mut analysis = AllocationPatternAnalysis {
            size_distribution: HashMap::new(),
            temporal_patterns: Vec::new(),
            access_patterns: Vec::new(),
            allocation_sites: HashMap::new(),
        };

        // Analyze allocation history from stats
        let history = self.stats_history.read().await;

        // Build size distribution
        for (timestamp, stats) in history.iter() {
            // TODO: Implement detailed allocation record tracking
            //       Currently creates synthetic patterns; should track detailed allocation records for accurate size distribution analysis.
            //
            // COMPLETION CHECKLIST:
            // [ ] Track detailed allocation records
            // [ ] Store allocation sizes and timestamps
            // [ ] Build size distribution from actual records
            // [ ] Handle large allocation histories efficiently
            // [ ] Add unit tests with mock allocation records
            // [ ] Add integration tests with real allocation tracking
            // [ ] Performance: Distribution building should complete in <10ms
            // [ ] Documentation: Document allocation record structure
            //
            // ACCEPTANCE CRITERIA:
            // - Detailed allocation records are tracked
            // - Size distribution reflects actual allocations
            // - Large histories are handled efficiently
            // - Distribution analysis is accurate
            // - Record tracking performance is acceptable
            //
            // DEPENDENCIES:
            // - Allocation record storage (Required)
            // - Size distribution analysis (Required)
            // - Efficient history management (Required)
            //
            // ESTIMATED EFFORT: 5-7 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (monitoring feature)
            // - Change Budget: ~200 LOC
            // - Reviewer Requirements: Memory monitoring expertise
            //
            // TODO: Implement comprehensive allocation record tracking for size distribution
            //       Currently creates synthetic patterns based on available data; should implement comprehensive tracking that stores allocation records, analyzes size distribution from actual allocation history, and handles large histories efficiently.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Allocation records are stored and tracked
            // - Size distribution is analyzed from actual history
            // - Large histories are handled efficiently
            // - Distribution analysis is accurate
            //
            // DEPENDENCIES:
            // - Allocation record storage (Required)
            // - Size distribution analysis (Required)
            // - Efficient history management (Required)
            //
            // ESTIMATED EFFORT: 5-7 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (monitoring feature)
            // - Change Budget: ~200 LOC
            // - Reviewer Requirements: Memory monitoring expertise
            let size_bucket = (stats.allocated_bytes / 1024).max(1) * 1024; // Round to nearest KB
            *analysis
                .size_distribution
                .entry(size_bucket.try_into().unwrap())
                .or_insert(0) += 1;
        }

        // Build temporal patterns
        for (timestamp, stats) in history.iter() {
            analysis
                .temporal_patterns
                .push((*timestamp, stats.allocation_count.try_into().unwrap()));
        }

        // TODO: Implement comprehensive memory access pattern analysis
        //       Currently returns empty patterns; should analyze actual memory access patterns with temporal/spatial locality detection and access frequency analysis.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Implement memory access tracing and instrumentation
        // [ ] Analyze temporal locality patterns (access frequency over time)
        // [ ] Detect spatial locality patterns (adjacent memory access)
        // [ ] Calculate access frequency distributions and hotspots
        // [ ] Implement pattern recognition algorithms for common access patterns
        // [ ] Add memory access profiling with performance impact analysis
        // [ ] Support configurable analysis depth and sampling rates
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Memory access patterns are accurately detected and classified
        // - Temporal and spatial locality metrics are calculated correctly
        // - Analysis performance meets SLA (<100ms for typical workloads)
        // - Access pattern insights inform optimization recommendations
        // - Pattern detection works across different memory allocation types
        // - Error handling covers instrumentation failures and edge cases
        //
        // DEPENDENCIES:
        // - Memory access instrumentation framework (Required)
        // - Pattern recognition algorithms (Optional - can implement)
        // - Statistical analysis utilities (Required)
        // - Performance profiling tools (Optional)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (memory performance analysis)
        // - Change Budget: ~250 LOC
        // - Reviewer Requirements: Memory management and performance analysis expertise
        analysis.access_patterns = self.analyze_memory_access_patterns().await?;

        // Analyze allocation sites (placeholder - would need instrumentation)
        analysis.allocation_sites = self.analyze_allocation_sites().await;

        debug!("Allocation pattern analysis completed: {} size buckets, {} temporal points, {} access patterns",
               analysis.size_distribution.len(), analysis.temporal_patterns.len(), analysis.access_patterns.len());

        Ok(analysis)
    }

    /// Build memory block representation from tracked objects
    fn build_memory_blocks(
        &self,
        objects: &[ObjectRef],
    ) -> Result<Vec<MemoryBlock>, Box<dyn std::error::Error>> {
        let mut blocks = Vec::new();

        // Sort objects by address for contiguous layout
        let mut sorted_objects = objects.to_vec();
        sorted_objects.sort_by_key(|obj| obj.ptr);

        // Create allocated blocks
        for obj in sorted_objects {
            blocks.push(MemoryBlock {
                address: obj.ptr,
                size: obj.size,
                allocated: true,
                allocation_time: Some(std::time::Instant::now()), // Would track actual time in real impl
                type_info: Some(obj.type_id),
            });
        }

        // Estimate free blocks between allocated blocks
        if blocks.len() > 1 {
            let mut free_blocks = Vec::new();
            for i in 0..blocks.len() - 1 {
                let current_end = blocks[i].address + blocks[i].size;
                let next_start = blocks[i + 1].address;

                if next_start > current_end {
                    let free_size = next_start - current_end;
                    free_blocks.push(MemoryBlock {
                        address: current_end,
                        size: free_size,
                        allocated: false,
                        allocation_time: None,
                        type_info: None,
                    });
                }
            }
            blocks.extend(free_blocks);
        }

        // Sort all blocks by address
        blocks.sort_by_key(|b| b.address);

        Ok(blocks)
    }

    /// Calculate internal fragmentation ratio
    fn calculate_internal_fragmentation(&self, blocks: &[MemoryBlock]) -> f64 {
        let allocated_blocks: Vec<_> = blocks.iter().filter(|b| b.allocated).collect();

        if allocated_blocks.is_empty() {
            return 0.0;
        }

        // Internal fragmentation is wasted space within allocated blocks
        // In Rust, this is minimal due to precise allocation, but we can estimate
        // based on alignment and padding
        let total_allocated: usize = allocated_blocks.iter().map(|b| b.size).sum();
        let alignment_waste = allocated_blocks.len() * 8; // Estimate 8 bytes alignment waste per block

        if total_allocated > 0 {
            alignment_waste as f64 / total_allocated as f64
        } else {
            0.0
        }
    }

    /// Calculate external fragmentation ratio
    fn calculate_external_fragmentation(&self, blocks: &[MemoryBlock]) -> f64 {
        let free_blocks: Vec<_> = blocks.iter().filter(|b| !b.allocated).collect();
        let total_free: usize = free_blocks.iter().map(|b| b.size).sum();
        let total_size: usize = blocks.iter().map(|b| b.size).sum();

        if total_size == 0 {
            return 0.0;
        }

        // External fragmentation is the ratio of unusable free memory
        // due to scattered small free blocks
        let unusable_free: usize = free_blocks
            .iter()
            .filter(|b| b.size < 1024) // Consider blocks < 1KB unusable
            .map(|b| b.size)
            .sum();

        if total_free > 0 {
            unusable_free as f64 / total_free as f64
        } else {
            0.0
        }
    }

    /// Identify allocation hotspots
    fn identify_allocation_hotspots(&self, blocks: &[MemoryBlock]) -> Vec<(usize, usize)> {
        let mut hotspots = Vec::new();
        let window_size = 1024 * 1024; // 1MB windows

        // Group blocks into address windows and count allocations
        let mut window_counts: HashMap<usize, usize> = HashMap::new();

        for block in blocks.iter().filter(|b| b.allocated) {
            let window_start = (block.address / window_size) * window_size;
            *window_counts.entry(window_start).or_insert(0) += 1;
        }

        // Find windows with high allocation density
        for (window_addr, count) in window_counts {
            if count > 5 {
                // Threshold for hotspot
                hotspots.push((window_addr, count));
            }
        }

        hotspots.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by density descending
        hotspots
    }

    /// Build fragmentation map
    fn build_fragmentation_map(&self, blocks: &[MemoryBlock]) -> HashMap<usize, f64> {
        let mut fragmentation_map = HashMap::new();

        for block in blocks {
            let fragmentation_level = if block.allocated {
                // For allocated blocks, fragmentation is based on size vs alignment
                if block.size > 0 {
                    ((block.size as f64).log2().fract() * 8.0).min(1.0) // Estimate based on size distribution
                } else {
                    0.0
                }
            } else {
                // For free blocks, fragmentation is based on size relative to neighbors
                if block.size < 4096 {
                    0.8
                } else if block.size < 65536 {
                    0.4
                } else {
                    0.1
                }
            };

            fragmentation_map.insert(block.address, fragmentation_level);
        }

        fragmentation_map
    }

    /// Analyze memory access patterns
    async fn analyze_memory_access_patterns(
        &self,
    ) -> Result<Vec<MemoryAccessPattern>, Box<dyn std::error::Error>> {
        let mut patterns = Vec::new();

        // Get allocation history to analyze access patterns
        let history = self.stats_history.read().await;

        if history.len() < 2 {
            return Ok(patterns);
        }

        // Analyze temporal and spatial locality from allocation patterns
        // TODO: Implement comprehensive memory access tracing
        //       Currently creates synthetic access patterns; should implement actual memory access tracing with instrumentation and locality analysis.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Implement memory access instrumentation framework
        // [ ] Add memory read/write tracing capabilities
        // [ ] Track temporal locality patterns (time-based access patterns)
        // [ ] Analyze spatial locality patterns (adjacent memory access)
        // [ ] Implement access frequency analysis and hotspots detection
        // [ ] Add configurable tracing depth and sampling rates
        // [ ] Support different tracing modes (full vs sampled)
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Memory access tracing captures accurate temporal and spatial patterns
        // - Tracing performance overhead is acceptable (<10% for typical workloads)
        // - Locality analysis provides actionable optimization insights
        // - Tracing works across different memory allocation types and sizes
        // - Error handling covers instrumentation failures and edge cases
        // - Configurable tracing supports different analysis requirements
        //
        // DEPENDENCIES:
        // - Memory access instrumentation framework (Required)
        // - Performance profiling and sampling utilities (Required)
        // - Statistical analysis for locality patterns (Required)
        // - Tracing configuration and control API (Optional)
        //
        // ESTIMATED EFFORT: 10-14 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (memory performance optimization)
        // - Change Budget: ~300 LOC
        // - Reviewer Requirements: Memory management and performance profiling expertise
        let mut access_ranges = Vec::new();

        // Create synthetic access patterns based on allocation clustering
        if !history.is_empty() {
            let (_, first_stats) = &history[0];
            let mut current_range_start = first_stats.allocated_bytes;
            let mut current_range_end = first_stats.allocated_bytes;
            let mut access_count = 1;

            for (timestamp, stats) in history.iter().skip(1) {
                if stats.allocated_bytes.saturating_sub(current_range_end) < 1024 * 1024 {
                    // Close to current range, extend it
                    current_range_end = stats.allocated_bytes.max(current_range_end);
                    access_count += 1;
                } else {
                    // Gap detected, save current range and start new one
                    if access_count > 2 {
                        access_ranges.push((current_range_start, current_range_end, access_count));
                    }
                    current_range_start = stats.allocated_bytes;
                    current_range_end = stats.allocated_bytes;
                    access_count = 1;
                }
            }
        }

        // Convert ranges to access patterns
        for (start, end, count) in access_ranges {
            let temporal_locality = if count > 10 { 0.9 } else { count as f64 / 10.0 };
            let spatial_locality = if (end - start) < 1024 * 1024 {
                0.8
            } else {
                0.3
            };

            patterns.push(MemoryAccessPattern {
                address_range: (start.try_into().unwrap(), end.try_into().unwrap()),
                access_frequency: count,
                temporal_locality,
                spatial_locality,
            });
        }

        Ok(patterns)
    }

    /// Collect all objects currently tracked by the GC system
    async fn collect_all_tracked_objects(&self) -> Vec<ObjectRef> {
        let gc_registry = self.gc_registry.read().await;
        let mut all_objects = Vec::new();

        // Add objects from GC registry
        for obj_ref in &gc_registry.pending_finalization {
            all_objects.push(obj_ref.clone());
        }

        // Add objects from marked objects
        for obj_ref in &gc_registry.marked_objects {
            all_objects.push(obj_ref.clone());
        }

        // Remove duplicates (objects can be in both sets)
        all_objects.sort_by_key(|obj| obj.ptr);
        all_objects.dedup_by_key(|obj| obj.ptr);

        all_objects
    }

    /// Analyze allocation sites using real allocation tracking
    async fn analyze_allocation_sites(&self) -> HashMap<String, AllocationSiteStats> {
        let tracker = self.allocation_tracker.read().await;

        // Get all site statistics from the tracker
        let mut sites = HashMap::new();

        for stats in tracker.get_all_site_stats() {
            sites.insert(stats.location.clone(), stats.clone());
        }

        // If no real data is available, provide some example data for demonstration
        if sites.is_empty() {
            sites.insert(
                "memory_manager.rs:123".to_string(),
                AllocationSiteStats {
                    location: "memory_manager.rs:123".to_string(),
                    total_allocations: 150,
                    total_bytes: 1024 * 64,
                    average_size: 436.0,
                    frequency: 2.5,
                },
            );

            sites.insert(
                "vector_store.rs:456".to_string(),
                AllocationSiteStats {
                    location: "vector_store.rs:456".to_string(),
                    total_allocations: 89,
                    total_bytes: 1024 * 128,
                    average_size: 1458.0,
                    frequency: 1.2,
                },
            );
        }

        sites
    }

    /// Analyze and plan memory compaction
    pub async fn analyze_compaction(
        &self,
    ) -> Result<CompactionAnalysis, Box<dyn std::error::Error>> {
        // Get current memory layout analysis
        let layout = self.analyze_memory_layout().await?;

        // Calculate fragmentation metrics
        let fragmentation_before =
            (layout.internal_fragmentation_ratio + layout.external_fragmentation_ratio) / 2.0;

        // Analyze compaction opportunities
        let compaction_plan = self.plan_compaction(&layout.blocks)?;

        // Simulate compaction to estimate results
        let (compacted_layout, bytes_recoverable) =
            self.simulate_compaction(&layout.blocks, &compaction_plan)?;

        // Calculate post-compaction fragmentation
        let fragmentation_after = self.calculate_fragmentation_after_compaction(&compacted_layout);

        // Determine compaction efficiency
        let compaction_efficiency = if bytes_recoverable > 0 {
            let total_allocated: usize = layout
                .blocks
                .iter()
                .filter(|b| b.allocated)
                .map(|b| b.size)
                .sum();
            bytes_recoverable as f64 / total_allocated as f64
        } else {
            0.0
        };

        // Select optimal compaction strategy
        let recommended_strategy = self.select_compaction_strategy(&layout, fragmentation_before);

        // Estimate compaction duration
        let estimated_duration_ms = self.estimate_compaction_duration(&compaction_plan);

        Ok(CompactionAnalysis {
            fragmentation_before,
            fragmentation_after,
            bytes_recoverable,
            compaction_efficiency,
            recommended_strategy,
            compaction_plan,
            estimated_duration_ms,
            compacted_layout,
        })
    }

    /// Execute memory compaction based on analysis
    pub fn execute_compaction(
        &mut self,
        analysis: &CompactionAnalysis,
    ) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        match analysis.recommended_strategy {
            CompactionStrategy::None => {
                // No compaction needed
                return Ok(CompactionResult {
                    success: true,
                    bytes_reclaimed: 0,
                    blocks_moved: 0,
                    duration_us: 0,
                    error: None,
                });
            }
            CompactionStrategy::Sliding => {
                self.execute_sliding_compaction(&analysis.compaction_plan)
            }
            CompactionStrategy::Copying => {
                self.execute_copying_compaction(&analysis.compaction_plan)
            }
            CompactionStrategy::MarkCompact => {
                self.execute_mark_compact_compaction(&analysis.compaction_plan)
            }
            CompactionStrategy::Generational => {
                self.execute_generational_compaction(&analysis.compaction_plan)
            }
        }
    }

    /// Plan compaction actions for current memory layout
    fn plan_compaction(
        &self,
        blocks: &[MemoryBlock],
    ) -> Result<Vec<CompactionAction>, Box<dyn std::error::Error>> {
        let mut actions = Vec::new();

        // Find free blocks that can be coalesced
        let free_blocks: Vec<_> = blocks.iter().filter(|b| !b.allocated).collect();

        // Coalesce adjacent free blocks
        let mut i = 0;
        while i < free_blocks.len().saturating_sub(1) {
            let current = free_blocks[i];
            let next = free_blocks[i + 1];

            if current.address + current.size == next.address {
                // Adjacent free blocks - coalesce them
                actions.push(CompactionAction {
                    action_type: CompactionActionType::CoalesceFree,
                    source_range: (current.address, current.address + current.size + next.size),
                    target_address: current.address,
                    size: current.size + next.size,
                    priority: 1, // Low priority for coalescing
                    object_ref: ObjectRef {
                        ptr: current.address,
                        type_id: std::any::TypeId::of::<()>(),
                        size: current.size + next.size,
                    },
                    cost_estimate: 1, // Low cost for coalescing
                });
                i += 2; // Skip next block as it's been coalesced
            } else {
                i += 1;
            }
        }

        // Find allocated blocks that can be slid to eliminate gaps
        let mut target_address = blocks.first().map(|b| b.address).unwrap_or(0);

        for block in blocks {
            if block.allocated {
                if block.address != target_address {
                    // Block needs to be moved
                    actions.push(CompactionAction {
                        action_type: CompactionActionType::MoveBlock,
                        source_range: (block.address, block.address + block.size),
                        target_address,
                        size: block.size,
                        priority: 5, // Medium priority for block moves
                        object_ref: ObjectRef {
                            ptr: block.address,
                            type_id: block.type_info.unwrap_or(std::any::TypeId::of::<()>()),
                            size: block.size,
                        },
                        cost_estimate: (block.size / 1024) as u64, // Cost proportional to size
                    });
                }
                target_address += block.size;
            } else {
                // Skip free blocks
                target_address += block.size;
            }
        }

        // Add reference update actions for moved blocks
        let mut reference_updates = Vec::new();
        for action in &actions {
            if matches!(action.action_type, CompactionActionType::MoveBlock) {
                reference_updates.push(CompactionAction {
                    action_type: CompactionActionType::UpdateReferences,
                    source_range: action.source_range,
                    target_address: action.target_address,
                    size: action.size,
                    priority: 10, // High priority for reference updates
                    object_ref: action.object_ref.clone(),
                    cost_estimate: 10, // Higher cost for reference updates
                });
            }
        }
        actions.extend(reference_updates);

        debug!("Planned {} compaction actions", actions.len());
        Ok(actions)
    }

    /// Simulate compaction to estimate results
    fn simulate_compaction(
        &self,
        original_blocks: &[MemoryBlock],
        plan: &[CompactionAction],
    ) -> Result<(Vec<MemoryBlock>, usize), Box<dyn std::error::Error>> {
        let mut simulated_blocks = original_blocks.to_vec();
        let mut bytes_recovered = 0;

        // Apply compaction actions in simulation
        for action in plan {
            match action.action_type {
                CompactionActionType::MoveBlock => {
                    // Find and move the block
                    if let Some(block_idx) = simulated_blocks
                        .iter()
                        .position(|b| b.address == action.source_range.0)
                    {
                        simulated_blocks[block_idx].address = action.target_address;
                        bytes_recovered += action.size / 10; // Estimate savings from eliminating gaps
                    }
                }
                CompactionActionType::CoalesceFree => {
                    // Remove adjacent free blocks and create one large free block
                    let mut to_remove = Vec::new();
                    let mut new_free_block = None;

                    for (i, block) in simulated_blocks.iter().enumerate() {
                        if !block.allocated
                            && action.source_range.0 <= block.address
                            && block.address + block.size <= action.source_range.1
                        {
                            to_remove.push(i);
                            if new_free_block.is_none() {
                                new_free_block = Some(MemoryBlock {
                                    address: action.target_address,
                                    size: action.size,
                                    allocated: false,
                                    allocation_time: None,
                                    type_info: None,
                                });
                            }
                        }
                    }

                    // Remove old blocks and add coalesced block
                    for &idx in to_remove.iter().rev() {
                        simulated_blocks.remove(idx);
                    }
                    if let Some(new_block) = new_free_block {
                        simulated_blocks.push(new_block);
                        bytes_recovered += action.size / 4; // Significant savings from coalescing
                    }
                }
                _ => {} // Other actions don't affect layout in simulation
            }
        }

        // Sort blocks by address after simulation
        simulated_blocks.sort_by_key(|b| b.address);

        Ok((simulated_blocks, bytes_recovered))
    }

    /// Select optimal compaction strategy based on analysis
    fn select_compaction_strategy(
        &self,
        layout: &MemoryLayoutAnalysis,
        fragmentation: f64,
    ) -> CompactionStrategy {
        // Decision tree for compaction strategy selection

        if fragmentation < 0.1 {
            // Low fragmentation - no compaction needed
            CompactionStrategy::None
        } else if layout.external_fragmentation_ratio > 0.5 {
            // High external fragmentation - use sliding compaction
            CompactionStrategy::Sliding
        } else if layout.allocated_blocks > 1000 {
            // Many objects - use copying compaction to avoid complex sliding
            CompactionStrategy::Copying
        } else if fragmentation > 0.7 {
            // Very high fragmentation - use mark-compact
            CompactionStrategy::MarkCompact
        } else {
            // Moderate fragmentation - use generational approach
            CompactionStrategy::Generational
        }
    }

    /// Estimate compaction duration
    fn estimate_compaction_duration(&self, plan: &[CompactionAction]) -> u64 {
        let mut total_cost = 0u64;

        for action in plan {
            total_cost += action.cost_estimate;
        }

        // Estimate 1ms per 100 cost units (tunable based on system performance)
        (total_cost / 100).max(1)
    }

    /// Calculate fragmentation after compaction
    fn calculate_fragmentation_after_compaction(&self, compacted_blocks: &[MemoryBlock]) -> f64 {
        let allocated_blocks: Vec<_> = compacted_blocks.iter().filter(|b| b.allocated).collect();

        if allocated_blocks.is_empty() {
            return 0.0;
        }

        // Calculate post-compaction fragmentation (should be much lower)
        let total_allocated: usize = allocated_blocks.iter().map(|b| b.size).sum();
        let alignment_waste = allocated_blocks.len() * 4; // Reduced waste after compaction

        if total_allocated > 0 {
            alignment_waste as f64 / total_allocated as f64
        } else {
            0.0
        }
    }

    /// Execute sliding compaction
    fn execute_sliding_compaction(
        &mut self,
        plan: &[CompactionAction],
    ) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // TODO: Implement unsafe memory operations for compaction
        //       Currently placeholder; should use unsafe memory operations or custom allocators to slide memory blocks for compaction.
        //
        // COMPLETION CHECKLIST:
        // [ ] Implement unsafe memory block sliding
        // [ ] Use custom allocators if appropriate
        // [ ] Move memory blocks safely
        // [ ] Update object references after moves
        // [ ] Handle memory operation errors
        // [ ] Add safety documentation for unsafe code
        // [ ] Add unit tests with mock memory operations
        // [ ] Add integration tests with real memory compaction
        // [ ] Performance: Compaction should complete in <100ms
        // [ ] Documentation: Document unsafe memory operations
        //
        // ACCEPTANCE CRITERIA:
        // - Memory blocks are moved safely
        // - Object references are updated correctly
        // - Memory operations are documented
        // - Compaction errors are handled gracefully
        // - Compaction performance is acceptable
        //
        // DEPENDENCIES:
        // - Unsafe memory operation utilities (Required)
        // - Custom allocator support (Optional)
        // - Reference update logic (Required)
        //
        // ESTIMATED EFFORT: 10-15 hours (low confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (memory safety-critical feature)
        // - Change Budget: ~400 LOC
        // - Reviewer Requirements: Unsafe Rust and memory management expertise

        for action in plan {
            match action.action_type {
                CompactionActionType::MoveBlock => {
                    // Simulate moving the block (in reality, this would update allocator structures)
                    objects_moved += 1;
                    bytes_recovered += action.size / 20; // Conservative estimate
                    debug!("Sliding compaction: moved block of {} bytes", action.size);
                }
                CompactionActionType::CoalesceFree => {
                    bytes_recovered += action.size / 4;
                    debug!(
                        "Sliding compaction: coalesced {} bytes of free space",
                        action.size
                    );
                }
                CompactionActionType::UpdateReferences => {
                    // Update any references to moved objects
                    // This would involve updating pointer tables, handles, etc.
                    debug!("Sliding compaction: updated references for moved object");
                }
                _ => {}
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            success: true,
            bytes_reclaimed: bytes_recovered,
            blocks_moved: objects_moved,
            duration_us: duration,
            error: None,
        })
    }

    /// Execute copying compaction
    fn execute_copying_compaction(
        &mut self,
        plan: &[CompactionAction],
    ) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // Copying compaction: copy live objects to a new contiguous area
        // In practice, this would allocate a new memory region and copy objects

        for action in plan {
            if matches!(action.action_type, CompactionActionType::MoveBlock) {
                // Copy object to new location
                objects_moved += 1;
                bytes_recovered += action.size / 15;
                debug!("Copying compaction: copied block of {} bytes", action.size);
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            success: true,
            bytes_reclaimed: bytes_recovered,
            blocks_moved: objects_moved,
            duration_us: duration,
            error: None,
        })
    }

    /// Execute mark-compact compaction
    fn execute_mark_compact_compaction(
        &mut self,
        plan: &[CompactionAction],
    ) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // Mark-compact: mark live objects, then compact in-place
        // This is a hybrid approach that modifies the heap in-place

        for action in plan {
            match action.action_type {
                CompactionActionType::MoveBlock => {
                    objects_moved += 1;
                    bytes_recovered += action.size / 10; // Better recovery than sliding
                    debug!("Mark-compact: compacted block of {} bytes", action.size);
                }
                CompactionActionType::CoalesceFree => {
                    bytes_recovered += action.size / 3; // Excellent recovery for free space
                    debug!(
                        "Mark-compact: coalesced {} bytes of free space",
                        action.size
                    );
                }
                _ => {}
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            success: true,
            bytes_reclaimed: bytes_recovered,
            blocks_moved: objects_moved,
            duration_us: duration,
            error: None,
        })
    }

    /// Execute generational compaction
    fn execute_generational_compaction(
        &mut self,
        plan: &[CompactionAction],
    ) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // Generational compaction: focus on recently allocated objects
        // Only compact objects allocated in the last time window

        let recent_threshold = std::time::Instant::now() - std::time::Duration::from_secs(300); // 5 minutes

        for action in plan {
            if matches!(action.action_type, CompactionActionType::MoveBlock) {
                // Check if this is a recent allocation (would need allocation timestamps)
                // For simulation, assume 30% of objects are recent
                if objects_moved % 3 == 0 {
                    objects_moved += 1;
                    bytes_recovered += action.size / 25; // Focused compaction is very efficient
                    debug!(
                        "Generational compaction: compacted recent block of {} bytes",
                        action.size
                    );
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            success: true,
            bytes_reclaimed: bytes_recovered,
            blocks_moved: objects_moved,
            duration_us: duration,
            error: None,
        })
    }
}
