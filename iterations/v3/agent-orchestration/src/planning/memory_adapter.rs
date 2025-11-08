//! Memory System Adapter
//!
//! Adapts the real agent_memory::MemorySystem to implement the contracts::MemorySystem trait.
//! This adapter enables dependency injection and breaks the direct dependency from orchestration to memory.
//!
//! @author @darianrosebrook

#[cfg(feature = "memory")]
use async_trait::async_trait;
#[cfg(feature = "memory")]
use std::sync::Arc;
#[cfg(feature = "memory")]
use anyhow::Result;

#[cfg(feature = "memory")]
use agent_agency_contracts::{
    MemorySystem,
    types::memory::{MemoryId, TemporalContext, ExperienceOutcome, TemporalQuery, Experience},
    errors::MemoryResult,
};

/// Adapter that wraps agent_memory::MemorySystem to implement contracts::MemorySystem
#[cfg(feature = "memory")]
pub struct MemorySystemAdapter {
    /// The underlying memory system implementation
    memory_system: Arc<agent_memory::MemorySystem>,
}

#[cfg(feature = "memory")]
impl MemorySystemAdapter {
    /// Create a new memory system adapter
    pub fn new(memory_system: Arc<agent_memory::MemorySystem>) -> Self {
        Self { memory_system }
    }
}

#[cfg(feature = "memory")]
#[async_trait]
impl MemorySystem for MemorySystemAdapter {
    async fn store_experience(&self, experience: Experience) -> MemoryResult<MemoryId> {
        // Convert contracts Experience to agent_memory types
        let memory_experience = agent_memory::memory_types::ExperienceContext {
            description: experience.description.clone(),
            domain: experience.domain.clone(),
            task_type: experience.task_type.clone(),
            temporal_context: experience.temporal_context.map(|tc| {
                agent_memory::memory_types::TemporalContext {
                    timestamp: tc.timestamp,
                    duration: tc.duration_ms.map(|ms| chrono::Duration::milliseconds(ms as i64)),
                    sequence_number: tc.sequence_number,
                    priority: match tc.priority {
                        agent_agency_contracts::types::memory::TaskPriority::Low => agent_memory::memory_types::TaskPriority::Low,
                        agent_agency_contracts::types::memory::TaskPriority::Normal => agent_memory::memory_types::TaskPriority::Normal,
                        agent_agency_contracts::types::memory::TaskPriority::High => agent_memory::memory_types::TaskPriority::High,
                        agent_agency_contracts::types::memory::TaskPriority::Critical => agent_memory::memory_types::TaskPriority::Critical,
                    },
                }
            }),
        };

        // Store the experience using the real memory system
        let memory_id = self.memory_system.store_experience(memory_experience).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "memory".to_string()
            })?;

        Ok(MemoryId(memory_id))
    }

    async fn retrieve_temporal_context(&self, query: TemporalQuery) -> MemoryResult<Vec<TemporalContext>> {
        // Convert contracts query to agent_memory types
        // This is a simplified implementation - in practice, we'd need more comprehensive query support
        let temporal_query = agent_memory::memory_types::TemporalQuery {
            start_time: query.start_time,
            end_time: query.end_time,
            priority_filter: query.priority_filter.map(|p| match p {
                agent_agency_contracts::types::memory::TaskPriority::Low => agent_memory::memory_types::TaskPriority::Low,
                agent_agency_contracts::types::memory::TaskPriority::Normal => agent_memory::memory_types::TaskPriority::Normal,
                agent_agency_contracts::types::memory::TaskPriority::High => agent_memory::memory_types::TaskPriority::High,
                agent_agency_contracts::types::memory::TaskPriority::Critical => agent_memory::memory_types::TaskPriority::Critical,
            }),
            limit: query.limit,
        };

        // Retrieve temporal contexts using the real memory system
        let contexts = self.memory_system.retrieve_temporal_context(temporal_query).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "memory".to_string()
            })?;

        // Convert back to contracts types
        let contracts_contexts = contexts.into_iter().map(|tc| {
            TemporalContext {
                timestamp: tc.timestamp,
                duration_ms: tc.duration.map(|d| d.num_milliseconds() as u64),
                sequence_number: tc.sequence_number,
                priority: match tc.priority {
                    agent_memory::memory_types::TaskPriority::Low => agent_agency_contracts::types::memory::TaskPriority::Low,
                    agent_memory::memory_types::TaskPriority::Normal => agent_agency_contracts::types::memory::TaskPriority::Normal,
                    agent_memory::memory_types::TaskPriority::High => agent_agency_contracts::types::memory::TaskPriority::High,
                    agent_memory::memory_types::TaskPriority::Critical => agent_agency_contracts::types::memory::TaskPriority::Critical,
                },
            }
        }).collect();

        Ok(contracts_contexts)
    }

    async fn record_outcome(&self, memory_id: MemoryId, outcome: ExperienceOutcome) -> MemoryResult<()> {
        // Convert contracts types to agent_memory types
        let memory_outcome = agent_memory::memory_types::ExperienceOutcome {
            success: outcome.success,
            quality_score: outcome.quality_score,
            error_message: outcome.error_message,
            metadata: outcome.metadata,
            performance_score: outcome.performance_score,
            execution_time_ms: outcome.execution_time_ms,
            learned_capabilities: outcome.learned_capabilities,
        };

        // Record the outcome using the real memory system
        self.memory_system.record_outcome(memory_id.0, memory_outcome).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "memory".to_string()
            })
    }

    async fn retrieve_experience(&self, memory_id: MemoryId) -> MemoryResult<Experience> {
        // Retrieve the experience using the real memory system
        let memory_experience = self.memory_system.retrieve_experience(memory_id.0).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "memory".to_string()
            })?;

        // Convert back to contracts types
        // This is a simplified conversion - in practice, we'd need more comprehensive type mapping
        let experience = Experience {
            id: memory_experience.id,
            description: memory_experience.description,
            memory_type: match memory_experience.memory_type {
                agent_memory::memory_types::MemoryType::Episodic => agent_agency_contracts::types::memory::MemoryType::Episodic,
                agent_memory::memory_types::MemoryType::Semantic => agent_agency_contracts::types::memory::MemoryType::Semantic,
                agent_memory::memory_types::MemoryType::Procedural => agent_agency_contracts::types::memory::MemoryType::Procedural,
                agent_memory::memory_types::MemoryType::Working => agent_agency_contracts::types::memory::MemoryType::Working,
            },
            temporal_context: memory_experience.temporal_context.map(|tc| {
                TemporalContext {
                    timestamp: tc.timestamp,
                    duration_ms: tc.duration.map(|d| d.num_milliseconds() as u64),
                    sequence_number: tc.sequence_number,
                    priority: match tc.priority {
                        agent_memory::memory_types::TaskPriority::Low => agent_agency_contracts::types::memory::TaskPriority::Low,
                        agent_memory::memory_types::TaskPriority::Normal => agent_agency_contracts::types::memory::TaskPriority::Normal,
                        agent_memory::memory_types::TaskPriority::High => agent_agency_contracts::types::memory::TaskPriority::High,
                        agent_memory::memory_types::TaskPriority::Critical => agent_agency_contracts::types::memory::TaskPriority::Critical,
                    },
                }
            }),
            outcome: ExperienceOutcome {
                success: memory_experience.outcome.success,
                quality_score: memory_experience.outcome.quality_score,
                error_message: memory_experience.outcome.error_message,
                metadata: memory_experience.outcome.metadata,
                performance_score: memory_experience.outcome.performance_score,
                execution_time_ms: memory_experience.outcome.execution_time_ms,
                learned_capabilities: memory_experience.outcome.learned_capabilities,
            },
            domain: memory_experience.domain,
            task_type: memory_experience.task_type,
            metadata: memory_experience.metadata,
        };

        Ok(experience)
    }

    async fn search_experiences(&self, query: serde_json::Value) -> MemoryResult<Vec<Experience>> {
        // For now, return empty vector - this would need proper implementation
        // based on the search capabilities of the underlying memory system
        warn!("search_experiences not fully implemented - returning empty results");
        Ok(Vec::new())
    }
}
