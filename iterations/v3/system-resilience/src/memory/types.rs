//! Core memory management types and data structures
//!
//! This module contains fundamental types for memory tracking, garbage collection,
//! and memory analysis used throughout the memory management system.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Object reference for garbage collection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectRef {
    /// Pointer to the object (simplified for demonstration)
    pub ptr: usize,
    /// Type information for the object
    pub type_id: std::any::TypeId,
    /// Size of the object in bytes
    pub size: usize,
}

/// Garbage collection registry for tracking objects and references
#[derive(Debug)]
pub struct GCRegistry {
    /// Objects that were marked as reachable in the last GC cycle
    pub marked_objects: std::collections::HashSet<ObjectRef>,
    /// Objects pending finalization
    pub pending_finalization: Vec<ObjectRef>,
    /// Weak references that need cleanup
    pub weak_references: HashMap<ObjectRef, Vec<std::sync::Weak<dyn std::any::Any + Send + Sync>>>,
    /// Resource handles that need to be closed during GC
    /// TODO: Replace with proper ResourceHandle type once resources module is extracted
    pub handles: HashMap<u64, u64>, // Placeholder: handle ID -> handle ID
    /// Timestamp of last mark phase
    pub last_mark_phase: std::time::Instant,
    /// Timestamp of last sweep phase
    pub last_sweep_phase: std::time::Instant,
    /// Total bytes tracked by GC (sum of all object sizes)
    pub total_bytes: usize,
}

impl GCRegistry {
    /// Create a new GC registry
    pub fn new() -> Self {
        Self {
            marked_objects: std::collections::HashSet::new(),
            pending_finalization: Vec::new(),
            weak_references: HashMap::new(),
            handles: HashMap::new(),
            last_mark_phase: std::time::Instant::now(),
            last_sweep_phase: std::time::Instant::now(),
            total_bytes: 0,
        }
    }
}

/// Memory block information for layout analysis
#[derive(Debug, Clone)]
pub struct MemoryBlock {
    /// Starting address of the block
    pub address: usize,
    /// Size of the block in bytes
    pub size: usize,
    /// Whether the block is allocated (true) or free (false)
    pub allocated: bool,
    /// Allocation timestamp (if allocated)
    pub allocation_time: Option<std::time::Instant>,
    /// Type information (if allocated)
    pub type_info: Option<std::any::TypeId>,
}

/// Memory layout analysis results
#[derive(Debug, Clone)]
pub struct MemoryLayoutAnalysis {
    /// Total heap size
    pub total_heap_size: usize,
    /// Total allocated memory
    pub allocated_memory: usize,
    /// Total free memory
    pub free_memory: usize,
    /// Number of allocated blocks
    pub allocated_blocks: usize,
    /// Number of free blocks
    pub free_blocks: usize,
    /// Average allocation size
    pub average_allocation_size: f64,
    /// Largest free block size
    pub largest_free_block: usize,
    /// Internal fragmentation ratio (wasted space within allocated blocks)
    pub internal_fragmentation_ratio: f64,
    /// External fragmentation ratio (wasted space between allocated blocks)
    pub external_fragmentation_ratio: f64,
    /// Memory blocks in address order
    pub blocks: Vec<MemoryBlock>,
    /// Allocation hotspots (addresses with high allocation density)
    pub allocation_hotspots: Vec<(usize, usize)>, // (address, allocation_count)
    /// Fragmentation map (address -> fragmentation level)
    pub fragmentation_map: HashMap<usize, f64>,
}

/// Allocation pattern analysis
#[derive(Debug, Clone)]
pub struct AllocationPatternAnalysis {
    /// Allocation size distribution (size -> count)
    pub size_distribution: HashMap<usize, usize>,
    /// Allocation frequency by time windows
    pub temporal_patterns: Vec<(std::time::Instant, usize)>,
    /// Memory access patterns (for cache analysis)
    pub access_patterns: Vec<MemoryAccessPattern>,
    /// Allocation site analysis
    pub allocation_sites: HashMap<String, AllocationSiteStats>,
}

/// Memory access pattern for cache efficiency analysis
#[derive(Debug, Clone)]
pub struct MemoryAccessPattern {
    /// Address range
    pub address_range: (usize, usize),
    /// Access frequency
    pub access_frequency: usize,
    /// Temporal locality (how clustered accesses are)
    pub temporal_locality: f64,
    /// Spatial locality (how close accesses are in memory)
    pub spatial_locality: f64,
}

/// Allocation site statistics
#[derive(Debug, Clone)]
pub struct AllocationSiteStats {
    /// Source location (file:line)
    pub location: String,
    /// Total allocations from this site
    pub total_allocations: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Allocation frequency (allocations per second)
    pub frequency: f64,
}

/// Task-based allocation statistics
#[derive(Debug, Clone)]
pub struct TaskAllocationStats {
    /// Task ID
    pub task_id: String,
    /// Total allocations by this task
    pub total_allocations: usize,
    /// Total bytes allocated by this task
    pub total_bytes: usize,
    /// Average allocation size for this task
    pub average_size: f64,
    /// Allocation sites used by this task
    pub allocation_sites: Vec<String>,
    /// Peak memory usage by this task
    pub peak_memory_bytes: usize,
    /// Current memory usage by this task
    pub current_memory_bytes: usize,
}

/// Memory limit configuration for monitoring and enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimitConfig {
    /// Maximum heap memory usage in MB
    pub max_heap_mb: u64,
    /// Maximum stack memory per thread in MB
    pub max_stack_mb: u64,
    /// Warning threshold as percentage of max heap (0.0-1.0)
    pub warning_threshold_percent: f64,
    /// Critical threshold as percentage of max heap (0.0-1.0)
    pub critical_threshold_percent: f64,
    /// Enable garbage collection under memory pressure
    pub enable_gc_pressure: bool,
    /// GC pressure threshold in MB
    pub gc_pressure_threshold_mb: f64,
    /// Monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
}

impl Default for MemoryLimitConfig {
    fn default() -> Self {
        Self {
            max_heap_mb: 1024, // 1GB
            max_stack_mb: 8,    // 8MB per thread
            warning_threshold_percent: 0.75,  // 75%
            critical_threshold_percent: 0.875, // 87.5%
            enable_gc_pressure: true,
            gc_pressure_threshold_mb: 800.0, // 800MB
            monitoring_interval_ms: 5000, // 5 seconds
        }
    }
}

/// Fragmentation analysis results
#[derive(Debug, Clone)]
pub struct FragmentationStats {
    /// External fragmentation ratio (wasted space between blocks)
    pub external_fragmentation: f64,
    /// Internal fragmentation ratio (wasted space within blocks)
    pub internal_fragmentation: f64,
    /// Largest contiguous free block
    pub largest_free_block: usize,
    /// Total free memory
    pub total_free_memory: usize,
    /// Number of free blocks
    pub free_blocks_count: usize,
}

/// Memory compaction analysis and planning
#[derive(Debug, Clone)]
pub struct CompactionAnalysis {
    /// Fragmentation level before compaction (0.0-1.0)
    pub fragmentation_before: f64,
    /// Fragmentation level after compaction (0.0-1.0)
    pub fragmentation_after: f64,
    /// Bytes that can be recovered through compaction
    pub bytes_recoverable: usize,
    /// Compaction efficiency (0.0-1.0, higher is better)
    pub compaction_efficiency: f64,
    /// Compaction strategy recommended
    pub recommended_strategy: CompactionStrategy,
    /// Detailed compaction plan (actions to perform)
    pub compaction_plan: Vec<CompactionAction>,
    /// Estimated duration in milliseconds
    pub estimated_duration_ms: u64,
    /// Memory layout after compaction
    pub compacted_layout: Vec<crate::memory::types::MemoryBlock>,
}

/// Result of a compaction operation
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Whether the compaction was successful
    pub success: bool,
    /// Bytes actually reclaimed
    pub bytes_reclaimed: usize,
    /// Number of blocks moved
    pub blocks_moved: usize,
    /// Time taken to perform compaction (in microseconds)
    pub duration_us: u64,
    /// Any error that occurred
    pub error: Option<String>,
}

/// Compaction strategy to use
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// No compaction needed
    None,
    /// Move objects to fill small gaps (low risk)
    Sliding,
    /// Copy live objects to new contiguous space (medium risk)
    Copying,
    /// Mark and sweep with compaction (high risk)
    MarkCompact,
    /// Generational compaction (highest risk, most effective)
    Generational,
}

/// Types of compaction actions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactionActionType {
    /// Coalesce adjacent free blocks
    CoalesceFree,
    /// Move allocated block to new location
    MoveBlock,
    /// Update references to moved block
    UpdateReferences,
    /// Free old memory location
    FreeBlock,
}

/// Actions that can be taken during compaction
#[derive(Debug, Clone)]
pub struct CompactionAction {
    /// Type of action to perform
    pub action_type: CompactionActionType,
    /// Source address range (start, end)
    pub source_range: (usize, usize),
    /// Target address for moved blocks
    pub target_address: usize,
    /// Size of the block being operated on
    pub size: usize,
    /// Priority of this action (higher = more important)
    pub priority: i32,
    /// Object reference for this action
    pub object_ref: ObjectRef,
    /// Estimated cost of performing this action
    pub cost_estimate: u64,
}

/// Statistics provider trait for memory-managed objects
#[async_trait::async_trait]
pub trait StatsProvider: Send + Sync {
    /// Get basic statistics
    async fn stats(&self) -> crate::memory::pool::PoolStats;

    /// Get detailed statistics as JSON
    async fn detailed_stats(&self) -> serde_json::Value;

    /// Get health status
    async fn health_status(&self) -> &'static str;
}
