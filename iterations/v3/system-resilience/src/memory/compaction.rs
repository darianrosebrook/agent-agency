//! Memory compaction and defragmentation analysis
//!
//! This module provides memory compaction analysis and simulation capabilities
//! for optimizing memory layout and reducing fragmentation.

use crate::memory::types::*;

/// Memory compaction analysis and results
#[derive(Debug, Clone)]
pub struct CompactionAnalysis {
    /// Current fragmentation ratio before compaction
    pub fragmentation_before: f64,
    /// Estimated fragmentation ratio after compaction
    pub fragmentation_after: f64,
    /// Estimated bytes that can be freed through compaction
    pub estimated_bytes_freed: usize,
    /// Estimated compaction time in milliseconds
    pub estimated_duration_ms: u64,
    /// Memory blocks after simulated compaction
    pub compacted_layout: Vec<MemoryBlock>,
    /// Compaction plan with actions to perform
    pub compaction_plan: Vec<CompactionAction>,
}

/// Individual compaction action
#[derive(Debug, Clone)]
pub struct CompactionAction {
    /// Action type
    pub action_type: CompactionActionType,
    /// Size of data to move
    pub size: usize,
    /// Object reference being moved
    pub object_ref: ObjectRef,
    /// Estimated cost of this action
    pub cost_estimate: u64,
}

/// Types of compaction actions
#[derive(Debug, Clone)]
pub enum CompactionActionType {
    /// Move object to new location
    MoveObject,
    /// Coalesce adjacent free blocks
    CoalesceFree,
    /// Split oversized free block
    SplitFree,
    /// Update references after move
    UpdateReferences,
}

/// Compaction result metrics
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Compaction duration in milliseconds
    pub duration_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}
