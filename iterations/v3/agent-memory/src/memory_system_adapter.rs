//! Memory System Adapter - Implements contracts::MemorySystem trait
//!
//! This adapter bridges the agent-memory MemorySystem implementation
//! with the agent-agency-contracts MemorySystem trait interface.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use async_trait::async_trait;
use agent_agency_contracts::ports::memory_system::MemorySystem as ContractsMemorySystem;
use agent_agency_contracts::types::memory::{
    MemoryId, TemporalContext, ExperienceOutcome, TemporalQuery, Experience,
    TaskPriority as ContractsTaskPriority,
};
use agent_agency_contracts::errors::{MemoryResult, MemoryError as ContractsMemoryError};
use crate::MemorySystem as AgentMemorySystem;
use crate::memory_types::{AgentExperience, MemoryType, TaskPriority};
use chrono::{DateTime, Utc};

/// Adapter that implements contracts::MemorySystem trait for agent-memory MemorySystem
#[derive(Debug)]
pub struct MemorySystemAdapter {
    inner: Arc<AgentMemorySystem>,
}

impl MemorySystemAdapter {
    /// Create a new adapter wrapping the agent-memory MemorySystem
    pub fn new(inner: Arc<AgentMemorySystem>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl ContractsMemorySystem for MemorySystemAdapter {
    async fn store_experience(&self, experience: Experience) -> MemoryResult<MemoryId> {
        // Convert contracts Experience to agent-memory AgentExperience
        let agent_experience = AgentExperience {
            id: experience.id,
            agent_id: "autonomous_executor".to_string(),
            task_id: experience.metadata
                .get("task_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| experience.id.to_string()),
            context: crate::memory_types::ExperienceContext {
                description: experience.description.clone(),
                domain: experience.domain.clone(),
                task_type: experience.task_type.clone(),
                temporal_context: None,
            },
            input: experience.metadata
                .get("input")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| experience.description.clone()),
            output: experience.metadata
                .get("output")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Experience outcome: {}", experience.outcome.success)),
            outcome: crate::memory_types::ExperienceOutcome {
                success: experience.outcome.success,
                quality_score: experience.outcome.quality_score,
                error_message: experience.outcome.error_message,
                metadata: experience.outcome.metadata,
                performance_score: experience.outcome.performance_score,
                execution_time_ms: experience.outcome.execution_time_ms,
                learned_capabilities: experience.outcome.learned_capabilities,
            },
            memory_type: match experience.memory_type {
                agent_agency_contracts::types::memory::MemoryType::Episodic => MemoryType::Episodic,
                agent_agency_contracts::types::memory::MemoryType::Semantic => MemoryType::Semantic,
                agent_agency_contracts::types::memory::MemoryType::Procedural => MemoryType::Procedural,
                agent_agency_contracts::types::memory::MemoryType::Working => MemoryType::Working,
            },
            timestamp: experience.temporal_context
                .as_ref()
                .map(|tc| tc.timestamp)
                .unwrap_or_else(Utc::now),
            metadata: experience.metadata,
            content: experience.description,
        };

        let memory_id = self.inner.store_experience(agent_experience).await
            .map_err(|e| ContractsMemoryError::OperationFailed {
                operation: "store_experience".to_string(),
                reason: e.to_string(),
            })?;

        Ok(MemoryId(memory_id))
    }

    async fn retrieve_temporal_context(&self, query: TemporalQuery) -> MemoryResult<Vec<TemporalContext>> {
        use crate::memory_manager::MemoryQuery;

        // Build memory query from temporal query
        let memory_query = MemoryQuery {
            agent_id: Some("autonomous_executor".to_string()),
            task_type: None, // Will be filtered by memory_type
            memory_type: query.memory_type_filter.map(|mt| match mt {
                agent_agency_contracts::types::memory::MemoryType::Episodic => MemoryType::Episodic,
                agent_agency_contracts::types::memory::MemoryType::Semantic => MemoryType::Semantic,
                agent_agency_contracts::types::memory::MemoryType::Procedural => MemoryType::Procedural,
                agent_agency_contracts::types::memory::MemoryType::Working => MemoryType::Working,
            }),
            time_range: if query.start_time.is_some() || query.end_time.is_some() {
                Some(crate::memory_types::TimeRange {
                    start: query.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30)),
                    end: query.end_time.unwrap_or_else(Utc::now),
                })
            } else {
                None
            },
            limit: query.limit,
        };

        // Search memories
        let experiences = self.inner.manager().search_memories(memory_query).await
            .map_err(|e| ContractsMemoryError::OperationFailed {
                operation: "retrieve_temporal_context".to_string(),
                reason: e.to_string(),
            })?;

        // Convert to temporal contexts
        let mut contexts = Vec::new();
        for (idx, exp) in experiences.iter().enumerate() {
            contexts.push(TemporalContext {
                timestamp: exp.timestamp,
                duration_ms: exp.metadata
                    .get("duration_ms")
                    .and_then(|v| v.as_u64()),
                sequence_number: Some(idx as u64),
                priority: match exp.metadata.get("priority") {
                    Some(v) if v.as_str() == Some("Critical") => ContractsTaskPriority::Critical,
                    Some(v) if v.as_str() == Some("High") => ContractsTaskPriority::High,
                    Some(v) if v.as_str() == Some("Normal") => ContractsTaskPriority::Normal,
                    _ => ContractsTaskPriority::Low,
                },
            });
        }

        Ok(contexts)
    }

    async fn record_outcome(&self, memory_id: MemoryId, outcome: ExperienceOutcome) -> MemoryResult<()> {
        // Convert contracts MemoryId to agent-memory MemoryId (Uuid)
        let agent_memory_id: crate::memory_types::MemoryId = memory_id.0;
        
        // Retrieve the experience first
        let mut experience = self.inner.manager().retrieve_memory(agent_memory_id).await
            .map_err(|e| ContractsMemoryError::NotFound {
                memory_id: memory_id.to_string(),
            })?;

        // Update outcome
        experience.outcome = crate::memory_types::ExperienceOutcome {
            success: outcome.success,
            quality_score: outcome.quality_score,
            error_message: outcome.error_message,
            metadata: outcome.metadata,
            performance_score: outcome.performance_score,
            execution_time_ms: outcome.execution_time_ms,
            learned_capabilities: outcome.learned_capabilities,
        };

        // Update metadata
        let mut metadata = experience.metadata;
        metadata.insert("outcome_updated".to_string(), serde_json::json!(Utc::now()));
        self.inner.manager().update_memory_metadata(agent_memory_id, metadata).await
            .map_err(|e| ContractsMemoryError::OperationFailed {
                operation: "record_outcome".to_string(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    async fn retrieve_experience(&self, memory_id: MemoryId) -> MemoryResult<Experience> {
        // Convert contracts MemoryId to agent-memory MemoryId (Uuid)
        let agent_memory_id: crate::memory_types::MemoryId = memory_id.0;
        
        let agent_experience = self.inner.manager().retrieve_memory(agent_memory_id).await
            .map_err(|e| ContractsMemoryError::NotFound {
                memory_id: memory_id.to_string(),
            })?;

        // Convert to contracts Experience
        let experience = Experience {
            id: agent_experience.id,
            description: agent_experience.content.clone(),
            memory_type: match agent_experience.memory_type {
                MemoryType::Episodic => agent_agency_contracts::types::memory::MemoryType::Episodic,
                MemoryType::Semantic => agent_agency_contracts::types::memory::MemoryType::Semantic,
                MemoryType::Procedural => agent_agency_contracts::types::memory::MemoryType::Procedural,
                MemoryType::Working => agent_agency_contracts::types::memory::MemoryType::Working,
            },
            temporal_context: Some(TemporalContext {
                timestamp: agent_experience.timestamp,
                duration_ms: agent_experience.outcome.execution_time_ms,
                sequence_number: None,
                priority: ContractsTaskPriority::Normal, // Default priority
            }),
            outcome: ExperienceOutcome {
                success: agent_experience.outcome.success,
                quality_score: agent_experience.outcome.quality_score,
                error_message: agent_experience.outcome.error_message,
                metadata: agent_experience.outcome.metadata,
                performance_score: agent_experience.outcome.performance_score,
                execution_time_ms: agent_experience.outcome.execution_time_ms,
                learned_capabilities: agent_experience.outcome.learned_capabilities,
            },
            domain: agent_experience.context.domain.clone(),
            task_type: agent_experience.context.task_type.clone(),
            metadata: {
                let mut meta = agent_experience.metadata.clone();
                meta.insert("agent_id".to_string(), serde_json::json!(agent_experience.agent_id));
                meta.insert("task_id".to_string(), serde_json::json!(agent_experience.task_id));
                meta.insert("input".to_string(), serde_json::json!(agent_experience.input));
                meta.insert("output".to_string(), serde_json::json!(agent_experience.output));
                meta
            },
        };

        Ok(experience)
    }

    async fn search_experiences(&self, query: serde_json::Value) -> MemoryResult<Vec<Experience>> {
        use crate::memory_manager::MemoryQuery;

        // Parse query JSON to build MemoryQuery
        let memory_query = MemoryQuery {
            agent_id: query.get("agent_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            task_type: query.get("task_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            memory_type: query.get("memory_type")
                .and_then(|v| v.as_str())
                .and_then(|s| match s {
                    "Episodic" => Some(MemoryType::Episodic),
                    "Semantic" => Some(MemoryType::Semantic),
                    "Procedural" => Some(MemoryType::Procedural),
                    "Working" => Some(MemoryType::Working),
                    _ => None,
                }),
            time_range: if query.get("start_time").is_some() || query.get("end_time").is_some() {
                Some(crate::memory_types::TimeRange {
                    start: query.get("start_time")
                        .and_then(|v| v.as_str())
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30)),
                    end: query.get("end_time")
                        .and_then(|v| v.as_str())
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                })
            } else {
                None
            },
            limit: query.get("limit")
                .and_then(|v| v.as_u64())
                .map(|u| u as usize),
        };

        // Search memories
        let experiences = self.inner.manager().search_memories(memory_query).await
            .map_err(|e| ContractsMemoryError::OperationFailed {
                operation: "search_experiences".to_string(),
                reason: e.to_string(),
            })?;

        // Convert to contracts Experiences
        let mut results = Vec::new();
        for agent_exp in experiences {
            results.push(Experience {
                id: agent_exp.id,
                description: agent_exp.content.clone(),
                memory_type: match agent_exp.memory_type {
                    MemoryType::Episodic => agent_agency_contracts::types::memory::MemoryType::Episodic,
                    MemoryType::Semantic => agent_agency_contracts::types::memory::MemoryType::Semantic,
                    MemoryType::Procedural => agent_agency_contracts::types::memory::MemoryType::Procedural,
                    MemoryType::Working => agent_agency_contracts::types::memory::MemoryType::Working,
                },
                temporal_context: Some(TemporalContext {
                    timestamp: agent_exp.timestamp,
                    duration_ms: agent_exp.outcome.execution_time_ms,
                    sequence_number: None,
                    priority: ContractsTaskPriority::Normal,
                }),
                outcome: ExperienceOutcome {
                    success: agent_exp.outcome.success,
                    quality_score: agent_exp.outcome.quality_score,
                    error_message: agent_exp.outcome.error_message,
                    metadata: agent_exp.outcome.metadata,
                    performance_score: agent_exp.outcome.performance_score,
                    execution_time_ms: agent_exp.outcome.execution_time_ms,
                    learned_capabilities: agent_exp.outcome.learned_capabilities,
                },
                domain: agent_exp.context.domain.clone(),
                task_type: agent_exp.context.task_type.clone(),
                metadata: {
                    let mut meta = agent_exp.metadata.clone();
                    meta.insert("agent_id".to_string(), serde_json::json!(agent_exp.agent_id));
                    meta.insert("task_id".to_string(), serde_json::json!(agent_exp.task_id));
                    meta.insert("input".to_string(), serde_json::json!(agent_exp.input));
                    meta.insert("output".to_string(), serde_json::json!(agent_exp.output));
                    meta
                },
            });
        }

        Ok(results)
    }
}

