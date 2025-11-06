//! Global memory allocator with tracking and statistics
//!
//! This module provides a custom global allocator that tracks memory allocations,
//! deallocations, and usage statistics for monitoring and debugging.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};

/// Global memory allocator instance
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
