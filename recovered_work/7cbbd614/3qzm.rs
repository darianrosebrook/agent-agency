//! Memory manager for unified memory management

use crate::ResourceUsage;
use anyhow::Result;

/// Memory manager
#[derive(Debug)]
pub struct MemoryManager {
    total_memory_mb: u64,
    allocated_memory_mb: u64,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(total_memory_mb: u64) -> Self {
        Self {
            total_memory_mb,
            allocated_memory_mb: 0,
        }
    }

    /// Allocate memory
    pub fn allocate(&mut self, size_mb: u64) -> Result<()> {
        if self.allocated_memory_mb + size_mb > self.total_memory_mb {
            return Err(anyhow::anyhow!("Insufficient memory"));
        }
        self.allocated_memory_mb += size_mb;
        Ok(())
    }

    /// Deallocate memory
    pub fn deallocate(&mut self, size_mb: u64) {
        self.allocated_memory_mb = self.allocated_memory_mb.saturating_sub(size_mb);
    }

    /// Get current memory usage
    pub fn usage(&self) -> ResourceUsage {
        ResourceUsage {
            cpu_percent: 0.0,
            memory_mb: self.allocated_memory_mb as u32,
            ane_stats: None,
            gpu_memory: None,
            thermal_stats: None,
        }
    }
}
