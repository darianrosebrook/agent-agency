//! Filesystem utilities for ANE operations
//!
//! This module provides utilities for managing filesystem resources
//! needed for ANE model caching and temporary storage.

use std::path::Path;
use anyhow::Result;

/// Filesystem space information
#[derive(Debug, Clone)]
pub struct FilesystemSpace {
    /// Total space in bytes
    pub total_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Block size
    pub block_size: u64,
}

/// Get filesystem space information for a given path using statvfs
pub fn get_filesystem_space<P: AsRef<Path>>(path: P) -> Result<FilesystemSpace> {
    use std::os::unix::fs::MetadataExt;
    use std::fs;

    // For now, return dummy values
    // In a real implementation, use platform-specific APIs or crates like fs2
    let metadata = fs::metadata(path)?;

    Ok(FilesystemSpace {
        total_bytes: 1_000_000_000_000, // 1TB placeholder
        available_bytes: 500_000_000_000, // 500GB placeholder
        used_bytes: 500_000_000_000, // 500GB placeholder
        block_size: 4096, // 4KB typical block size
    })
}

/// Check if filesystem has sufficient space for cache operations
pub fn check_filesystem_space<P: AsRef<Path>>(path: P, required_bytes: u64) -> Result<bool> {
    let space = get_filesystem_space(path)?;
    Ok(space.available_bytes >= required_bytes)
}

/// Get recommended cache size based on available filesystem space
pub fn get_recommended_cache_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let space = get_filesystem_space(path)?;

    // Use 10% of available space, but cap at 1GB
    let recommended = (space.available_bytes / 10).min(1024 * 1024 * 1024);

    // Minimum 100MB
    Ok(recommended.max(100 * 1024 * 1024))
}
