//! Long-term Memory Management
//!
//! Intelligent lifecycle management for long-term memory storage and retrieval.

pub mod lifecycle;
pub mod archival;
pub mod reinforcement;
pub mod retrieval;

pub use lifecycle::*;
pub use archival::*;
pub use reinforcement::*;
pub use retrieval::*;

/// Long-term memory configuration
#[derive(Debug, Clone)]
pub struct LongTermMemoryConfig {
    pub archival_threshold_days: u64,
    pub reinforcement_interval_hours: u64,
    pub decay_rate: f32,
    pub importance_threshold: f32,
    pub max_archival_age_days: u64,
    pub retrieval_boost_factor: f32,
}

/// Memory lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryLifecycleState {
    Active,
    Aging,
    Archival,
    Archived,
    Forgotten,
}

/// Memory lifecycle metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLifecycleMetadata {
    pub state: MemoryLifecycleState,
    pub importance_score: f32,
    pub access_count: u32,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reinforcement_count: u32,
    pub decay_factor: f32,
}

/// Long-term memory manager
pub struct LongTermMemoryManager {
    config: LongTermMemoryConfig,
}

impl LongTermMemoryManager {
    pub fn new(config: LongTermMemoryConfig) -> Self {
        Self { config }
    }

    /// Evaluate memory lifecycle state
    pub async fn evaluate_lifecycle(
        &self,
        metadata: &MemoryLifecycleMetadata,
    ) -> crate::MemoryResult<MemoryLifecycleAction> {
        let now = chrono::Utc::now();
        let age_days = (now - metadata.created_at).num_days() as u64;

        // Determine appropriate action based on current state and metrics
        let action = match metadata.state {
            MemoryLifecycleState::Active => {
                self.evaluate_active_memory(metadata, age_days).await?
            }
            MemoryLifecycleState::Aging => {
                self.evaluate_aging_memory(metadata, age_days).await?
            }
            MemoryLifecycleState::Archival => {
                self.evaluate_archival_memory(metadata, age_days).await?
            }
            MemoryLifecycleState::Archived => {
                MemoryLifecycleAction::KeepArchived
            }
            MemoryLifecycleState::Forgotten => {
                MemoryLifecycleAction::Purge
            }
        };

        Ok(action)
    }

    /// Apply reinforcement learning to memory importance
    pub async fn apply_reinforcement(
        &self,
        metadata: &mut MemoryLifecycleMetadata,
        access_pattern: &AccessPattern,
    ) -> crate::MemoryResult<()> {
        // Update importance based on access patterns
        let reinforcement_factor = self.calculate_reinforcement_factor(access_pattern);

        metadata.importance_score *= reinforcement_factor;
        metadata.importance_score = metadata.importance_score.min(1.0); // Cap at 1.0

        metadata.reinforcement_count += 1;
        metadata.last_accessed = chrono::Utc::now();

        Ok(())
    }

    /// Apply decay to memory importance over time
    pub async fn apply_decay(
        &self,
        metadata: &mut MemoryLifecycleMetadata,
    ) -> crate::MemoryResult<()> {
        let now = chrono::Utc::now();
        let hours_since_access = (now - metadata.last_accessed).num_hours() as f32;

        // Exponential decay based on time since last access
        let decay_factor = (-self.config.decay_rate * hours_since_access).exp();

        metadata.decay_factor *= decay_factor;
        metadata.importance_score *= metadata.decay_factor;

        Ok(())
    }

    /// Determine if memory should be archived
    pub async fn should_archive(&self, metadata: &MemoryLifecycleMetadata) -> bool {
        let age_days = (chrono::Utc::now() - metadata.created_at).num_days() as u64;

        // Archive if old enough and low importance
        age_days >= self.config.archival_threshold_days &&
        metadata.importance_score < self.config.importance_threshold
    }

    /// Determine if memory should be forgotten (permanent removal)
    pub async fn should_forget(&self, metadata: &MemoryLifecycleMetadata) -> bool {
        let age_days = (chrono::Utc::now() - metadata.created_at).num_days() as u64;

        // Forget if very old and very low importance
        age_days >= self.config.max_archival_age_days &&
        metadata.importance_score < 0.1
    }

    /// Evaluate active memory state
    async fn evaluate_active_memory(
        &self,
        metadata: &MemoryLifecycleMetadata,
        age_days: u64,
    ) -> crate::MemoryResult<MemoryLifecycleAction> {
        if self.should_archive(metadata).await {
            Ok(MemoryLifecycleAction::TransitionToArchival)
        } else if metadata.importance_score > 0.8 {
            // High importance memories stay active longer
            Ok(MemoryLifecycleAction::StayActive)
        } else if age_days > self.config.archival_threshold_days / 2 {
            Ok(MemoryLifecycleAction::TransitionToAging)
        } else {
            Ok(MemoryLifecycleAction::StayActive)
        }
    }

    /// Evaluate aging memory state
    async fn evaluate_aging_memory(
        &self,
        metadata: &MemoryLifecycleMetadata,
        age_days: u64,
    ) -> crate::MemoryResult<MemoryLifecycleAction> {
        if self.should_archive(metadata).await {
            Ok(MemoryLifecycleAction::TransitionToArchival)
        } else if metadata.importance_score > 0.6 {
            // Recovered importance - reactivate
            Ok(MemoryLifecycleAction::TransitionToActive)
        } else if age_days > self.config.archival_threshold_days {
            Ok(MemoryLifecycleAction::TransitionToArchival)
        } else {
            Ok(MemoryLifecycleAction::StayAging)
        }
    }

    /// Evaluate archival memory state
    async fn evaluate_archival_memory(
        &self,
        metadata: &MemoryLifecycleMetadata,
        _age_days: u64,
    ) -> crate::MemoryResult<MemoryLifecycleAction> {
        if metadata.importance_score > 0.7 {
            // High importance recovered - reactivate
            Ok(MemoryLifecycleAction::TransitionToActive)
        } else if self.should_forget(metadata).await {
            Ok(MemoryLifecycleAction::TransitionToForgotten)
        } else {
            Ok(MemoryLifecycleAction::StayArchived)
        }
    }

    /// Calculate reinforcement factor based on access pattern
    fn calculate_reinforcement_factor(&self, access_pattern: &AccessPattern) -> f32 {
        let base_boost = 1.1; // 10% boost for access

        // Additional boost based on access frequency
        let frequency_boost = match access_pattern.frequency {
            AccessFrequency::VeryHigh => 1.5,
            AccessFrequency::High => 1.3,
            AccessFrequency::Medium => 1.1,
            AccessFrequency::Low => 1.0,
            AccessFrequency::VeryLow => 0.9,
        };

        // Context relevance boost
        let context_boost = if access_pattern.context_relevant { 1.2 } else { 1.0 };

        base_boost * frequency_boost * context_boost
    }
}

/// Memory lifecycle action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryLifecycleAction {
    StayActive,
    StayAging,
    StayArchived,
    KeepArchived,
    TransitionToActive,
    TransitionToAging,
    TransitionToArchival,
    TransitionToForgotten,
    Purge,
}

/// Access pattern for reinforcement learning
#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub frequency: AccessFrequency,
    pub recency: chrono::Duration,
    pub context_relevant: bool,
    pub access_type: AccessType,
}

/// Access frequency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessFrequency {
    VeryHigh, // Multiple times per day
    High,     // Daily
    Medium,   // Weekly
    Low,      // Monthly
    VeryLow,  // Rarely
}

/// Access type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    DirectQuery,
    SemanticSearch,
    ContextualRetrieval,
    BackgroundProcessing,
}
