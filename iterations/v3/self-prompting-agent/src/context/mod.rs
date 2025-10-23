//! Hierarchical Context Management
//!
//! Multi-level context system with working memory, episodic memory, and semantic memory.
//! Features intelligent token allocation, compression, and calibrated retrieval.

pub mod manager;
pub mod budget;

// Re-export main types
pub use manager::{
    HierarchicalContextManager,
    ContextBundle,
    ContextStats,
    ContextError,
    WorkingMemory,
    EpisodicMemory,
    SemanticMemory,
    CalibratedRetriever,
    ContextCompressor,
};

pub use budget::{
    ContextBudget,
    Allocation,
    ContextUtility,
    TokenEstimates,
    ContextAllocator,
    MemoryWeights,
    BudgetTracker,
    BudgetEfficiencyReport,
    AdaptiveBudgetAdjuster,
    TaskPerformance,
    AdjustmentStrategy,
    OptimizationStrategy,
};
