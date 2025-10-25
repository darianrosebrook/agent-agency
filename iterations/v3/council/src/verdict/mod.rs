//! Verdict Storage and Management System
//!
//! Provides persistent storage and retrieval of council verdicts, consensus results,
//! and debate sessions for audit trails and performance analysis.

pub mod types;
pub mod storage;
pub mod cache;
pub mod store;

// Re-export main types and implementations
pub use types::*;
pub use storage::{VerdictStorage, MemoryVerdictStorage, DatabaseVerdictStorage};
pub use cache::{VerdictCache, CacheManager};
pub use store::VerdictStore;

/// Create a new verdict store with default configuration
pub fn create_verdict_store() -> VerdictStore {
    VerdictStore::new()
}

/// Create a verdict store with custom cache configuration
pub fn create_verdict_store_with_cache(config: CacheConfig) -> VerdictStore {
    VerdictStore::with_cache_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verdict_module_integration() {
        let store = create_verdict_store();

        // Test basic functionality
        let health = store.health_check().await;
        match health.status {
            StorageHealth::Healthy => {} // Should be healthy
            _ => panic!("Verdict store should be healthy"),
        }
    }
}
