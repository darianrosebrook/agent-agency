//! Memory Lifecycle Management
//!
//! Automated lifecycle transitions for memory items based on usage patterns and importance.

use crate::long_term_management::*;

/// Memory lifecycle manager
pub struct MemoryLifecycleManager {
    manager: LongTermMemoryManager,
    transition_rules: Vec<LifecycleTransitionRule>,
}

impl MemoryLifecycleManager {
    pub fn new(manager: LongTermMemoryManager) -> Self {
        Self {
            manager,
            transition_rules: Self::default_transition_rules(),
        }
    }

    /// Process lifecycle transitions for memories
    pub async fn process_lifecycle_transitions(
        &self,
        memories: Vec<(crate::memory_types::MemoryId, MemoryLifecycleMetadata)>,
    ) -> crate::MemoryResult<LifecycleTransitionResult> {
        let mut transitions = Vec::new();
        let mut stay_count = 0;

        for (memory_id, metadata) in memories {
            let action = self.manager.evaluate_lifecycle(&metadata).await?;

            match action {
                MemoryLifecycleAction::StayActive |
                MemoryLifecycleAction::StayAging |
                MemoryLifecycleAction::StayArchived |
                MemoryLifecycleAction::KeepArchived => {
                    stay_count += 1;
                }
                _ => {
                    transitions.push(LifecycleTransition {
                        memory_id,
                        from_state: metadata.state,
                        to_state: self.action_to_state(&action),
                        action,
                        metadata: metadata.clone(),
                    });
                }
            }
        }

        Ok(LifecycleTransitionResult {
            transitions,
            unchanged_count: stay_count,
            processing_timestamp: chrono::Utc::now(),
        })
    }

    /// Apply lifecycle transitions
    pub async fn apply_transitions(
        &self,
        transitions: &[LifecycleTransition],
    ) -> crate::MemoryResult<TransitionApplicationResult> {
        let mut applied = Vec::new();
        let mut failed = Vec::new();

        for transition in transitions {
            match self.apply_single_transition(transition).await {
                Ok(_) => applied.push(transition.memory_id.clone()),
                Err(e) => failed.push((transition.memory_id.clone(), e.to_string())),
            }
        }

        Ok(TransitionApplicationResult {
            applied_transitions: applied.len(),
            failed_transitions: failed,
            total_processed: transitions.len(),
        })
    }

    /// Apply decay to memories
    pub async fn apply_decay_cycle(
        &self,
        mut metadata_list: Vec<MemoryLifecycleMetadata>,
    ) -> crate::MemoryResult<DecayApplicationResult> {
        let mut decayed_count = 0;
        let mut total_decay_factor = 0.0;

        for metadata in &mut metadata_list {
            let old_importance = metadata.importance_score;
            self.manager.apply_decay(metadata).await?;

            if (old_importance - metadata.importance_score).abs() > 0.01 {
                decayed_count += 1;
            }

            total_decay_factor += metadata.decay_factor;
        }

        let average_decay_factor = if !metadata_list.is_empty() {
            total_decay_factor / metadata_list.len() as f32
        } else {
            1.0
        };

        Ok(DecayApplicationResult {
            memories_decayed: decayed_count,
            total_memories: metadata_list.len(),
            average_decay_factor,
        })
    }

    /// Apply reinforcement based on access patterns
    pub async fn apply_reinforcement_cycle(
        &self,
        mut metadata_list: Vec<(crate::memory_types::MemoryId, MemoryLifecycleMetadata)>,
        access_patterns: std::collections::HashMap<crate::memory_types::MemoryId, AccessPattern>,
    ) -> crate::MemoryResult<ReinforcementApplicationResult> {
        let mut reinforced_count = 0;
        let mut total_importance_gain = 0.0;

        for (memory_id, metadata) in &mut metadata_list {
            // Use the memory ID directly as the key for access patterns
            if let Some(access_pattern) = access_patterns.get(memory_id) {
                let old_importance = metadata.importance_score;
                self.manager.apply_reinforcement(metadata, access_pattern).await?;

                let importance_gain = metadata.importance_score - old_importance;
                if importance_gain > 0.01 {
                    reinforced_count += 1;
                    total_importance_gain += importance_gain;
                }
            }
        }

        Ok(ReinforcementApplicationResult {
            memories_reinforced: reinforced_count,
            total_memories: metadata_list.len(),
            average_importance_gain: if reinforced_count > 0 {
                total_importance_gain / reinforced_count as f32
            } else {
                0.0
            },
        })
    }

    /// Convert lifecycle action to state
    fn action_to_state(&self, action: &MemoryLifecycleAction) -> MemoryLifecycleState {
        match action {
            MemoryLifecycleAction::TransitionToActive => MemoryLifecycleState::Active,
            MemoryLifecycleAction::TransitionToAging => MemoryLifecycleState::Aging,
            MemoryLifecycleAction::TransitionToArchival => MemoryLifecycleState::Archival,
            MemoryLifecycleAction::TransitionToForgotten => MemoryLifecycleState::Forgotten,
            _ => MemoryLifecycleState::Active, // Default fallback
        }
    }

    /// Apply single transition (placeholder implementation)
    async fn apply_single_transition(&self, _transition: &LifecycleTransition) -> crate::MemoryResult<()> {
        // In practice, this would update the memory in the database
        Ok(())
    }

    /// Default transition rules
    fn default_transition_rules() -> Vec<LifecycleTransitionRule> {
        vec![
            LifecycleTransitionRule {
                from_state: MemoryLifecycleState::Active,
                to_state: MemoryLifecycleState::Aging,
                condition: TransitionCondition::AgeThreshold(30), // 30 days
            },
            LifecycleTransitionRule {
                from_state: MemoryLifecycleState::Aging,
                to_state: MemoryLifecycleState::Archival,
                condition: TransitionCondition::ImportanceThreshold(0.3),
            },
            LifecycleTransitionRule {
                from_state: MemoryLifecycleState::Archival,
                to_state: MemoryLifecycleState::Forgotten,
                condition: TransitionCondition::AgeThreshold(365), // 1 year
            },
        ]
    }
}

/// Lifecycle transition rule
#[derive(Debug, Clone)]
pub struct LifecycleTransitionRule {
    pub from_state: MemoryLifecycleState,
    pub to_state: MemoryLifecycleState,
    pub condition: TransitionCondition,
}

/// Transition condition
#[derive(Debug, Clone)]
pub enum TransitionCondition {
    AgeThreshold(u64), // Days
    ImportanceThreshold(f32),
    AccessCountThreshold(u32),
    Custom(String),
}

/// Lifecycle transition record
#[derive(Debug, Clone)]
pub struct LifecycleTransition {
    pub memory_id: crate::memory_types::MemoryId,
    pub from_state: MemoryLifecycleState,
    pub to_state: MemoryLifecycleState,
    pub action: MemoryLifecycleAction,
    pub metadata: MemoryLifecycleMetadata,
}

/// Lifecycle transition result
#[derive(Debug, Clone)]
pub struct LifecycleTransitionResult {
    pub transitions: Vec<LifecycleTransition>,
    pub unchanged_count: usize,
    pub processing_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Transition application result
#[derive(Debug, Clone)]
pub struct TransitionApplicationResult {
    pub applied_transitions: usize,
    pub failed_transitions: Vec<(crate::memory_types::MemoryId, String)>,
    pub total_processed: usize,
}

/// Decay application result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayApplicationResult {
    pub memories_decayed: usize,
    pub total_memories: usize,
    pub average_decay_factor: f32,
}

/// Reinforcement application result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementApplicationResult {
    pub memories_reinforced: usize,
    pub total_memories: usize,
    pub average_importance_gain: f32,
}

/// Lifecycle management statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStats {
    pub active_memories: usize,
    pub aging_memories: usize,
    pub archival_memories: usize,
    pub archived_memories: usize,
    pub forgotten_memories: usize,
    pub transitions_today: usize,
    pub average_importance_score: f32,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
