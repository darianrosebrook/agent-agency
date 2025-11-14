//! Memory System Port
//!
//! Defines the interface for memory system operations.
//! This port enables dependency injection and testing for memory operations.
//!
//! @author @darianrosebrook

use crate::errors::MemoryResult;
use crate::types::memory::{
    Experience, ExperienceOutcome, MemoryId, TemporalContext, TemporalQuery,
};

/// Core memory system interface
/// Implementations provide memory storage and retrieval capabilities
#[async_trait::async_trait]
pub trait MemorySystem: Send + Sync {
    /// Store an experience in memory
    ///
    /// # Arguments
    /// * `experience` - The experience data to store
    ///
    /// # Returns
    /// The memory ID of the stored experience, or an error if storage fails
    async fn store_experience(&self, experience: Experience) -> MemoryResult<MemoryId>;

    /// Retrieve temporal context for a time range or criteria
    ///
    /// # Arguments
    /// * `query` - Query parameters for temporal context retrieval
    ///
    /// # Returns
    /// Vector of matching temporal contexts, or an error if retrieval fails
    async fn retrieve_temporal_context(
        &self,
        query: TemporalQuery,
    ) -> MemoryResult<Vec<TemporalContext>>;

    /// Record the outcome of an experience
    ///
    /// # Arguments
    /// * `memory_id` - ID of the experience to update
    /// * `outcome` - The outcome data to record
    ///
    /// # Returns
    /// Unit result indicating success, or an error if recording fails
    async fn record_outcome(
        &self,
        memory_id: MemoryId,
        outcome: ExperienceOutcome,
    ) -> MemoryResult<()>;

    /// Retrieve a specific experience by ID
    ///
    /// # Arguments
    /// * `memory_id` - The ID of the experience to retrieve
    ///
    /// # Returns
    /// The experience data, or an error if not found or retrieval fails
    async fn retrieve_experience(&self, memory_id: MemoryId) -> MemoryResult<Experience>;

    /// Search for experiences matching criteria
    ///
    /// # Arguments
    /// * `query` - Search criteria (can be extended based on implementation)
    ///
    /// # Returns
    /// Vector of matching experiences, or an error if search fails
    async fn search_experiences(&self, query: serde_json::Value) -> MemoryResult<Vec<Experience>>;
}
