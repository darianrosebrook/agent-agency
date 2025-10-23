//! Memory management module for Apple Silicon
//!
//! This module provides comprehensive memory management capabilities
//! for Apple Silicon systems, including monitoring, optimization,
//! and quantization features.

// Re-export public types from submodules
pub use self::metrics::*;
pub use self::compression::*;
pub use self::analysis::*;
pub use self::manager::{
    *,
    // Re-export multi-tenant types
    TenantId, TenantMemoryConfig, TenantPriority, IsolationLevel, TenantMemoryUsage,
    MultiTenantMemoryManager, MemoryAllocationRequest, MemoryAllocationResponse,
    AllocationType, TenantDashboard, TenantSummary, DashboardSummary,
};
pub use self::quantization::*;

// Submodules
pub mod metrics;
pub mod compression;
pub mod analysis;
pub mod manager;
pub mod quantization;

#[cfg(test)]
pub mod multi_tenant_tests;
