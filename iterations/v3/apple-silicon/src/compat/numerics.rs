//! Numeric conversion utilities for Apple Silicon APIs
//!
//! Metal and MPSGraph APIs often require usize ↔ u64 conversions.
//! This module centralizes all such casts with debug assertions.

/// Convert usize to u64 (safe, no overflow possible on 64-bit targets)
#[inline]
pub fn as_u64(x: usize) -> u64 {
    x as u64
}

/// Convert u64 to usize with debug assertion
#[inline]
pub fn as_usize(x: u64) -> usize {
    #[cfg(debug_assertions)]
    assert!(x <= usize::MAX as u64, "u64 value {} exceeds usize::MAX {}", x, usize::MAX);
    x as usize
}

/// Checked conversion from u64 to usize
#[inline]
pub fn u64_to_usize_checked(x: u64) -> Option<usize> {
    if x <= usize::MAX as u64 {
        Some(x as usize)
    } else {
        None
    }
}

/// Calculate threadgroup size for Metal compute shaders
/// Returns (threadgroup_count, threads_per_threadgroup)
pub fn calculate_threadgroups(
    total_threads: usize,
    max_threads_per_group: usize
) -> (u64, u64) {
    let threads_per_group = max_threads_per_group.min(total_threads);
    let group_count = (total_threads + threads_per_group - 1) / threads_per_group;

    (as_u64(group_count), as_u64(threads_per_group))
}

/// Safe division for size calculations
#[inline]
pub fn safe_div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

/// Convert Metal MTLSize to (width, height, depth)
#[inline]
pub fn mtl_size_to_tuple(size: &metal::MTLSize) -> (u64, u64, u64) {
    (size.width, size.height, size.depth)
}

/// Convert tuple to Metal MTLSize
#[inline]
pub fn tuple_to_mtl_size((width, height, depth): (u64, u64, u64)) -> metal::MTLSize {
    metal::MTLSize {
        width,
        height,
        depth,
    }
}
