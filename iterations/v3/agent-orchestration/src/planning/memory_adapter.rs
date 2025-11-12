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
use agent_agency_contracts::{
    MemorySystem,
    types::memory::{MemoryId, TemporalContext, ExperienceOutcome, TemporalQuery, Experience},
    errors::{MemoryResult, MemoryError},
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
        
        
        // Extract timestamp before moving temporal_context
        let timestamp = experience.temporal_context.as_ref()
            .map(|tc| tc.timestamp)
            .unwrap_or_else(|| chrono::Utc::now());
        
        // Convert contracts Experience to agent_memory AgentExperience
        let memory_experience = agent_memory::memory_types::AgentExperience {
            id: experience.id,
            agent_id: "orchestrator".to_string(), // Default agent ID
            task_id: experience.id.to_string(),
            content: experience.description.clone(),
            input: String::new(), // Not available in contracts Experience
            output: format!("Outcome: success={}", experience.outcome.success),
            context: agent_memory::memory_types::ExperienceContext {
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
                            agent_agency_contracts::types::memory::TaskPriority::Medium => agent_memory::memory_types::TaskPriority::Normal, // Map Medium to Normal
                            agent_agency_contracts::types::memory::TaskPriority::High => agent_memory::memory_types::TaskPriority::High,
                            agent_agency_contracts::types::memory::TaskPriority::Urgent => agent_memory::memory_types::TaskPriority::Critical, // Map Urgent to Critical
                            agent_agency_contracts::types::memory::TaskPriority::Critical => agent_memory::memory_types::TaskPriority::Critical,
                        },
                    }
                }),
            },
            outcome: agent_memory::memory_types::ExperienceOutcome {
                success: experience.outcome.success,
                quality_score: experience.outcome.quality_score,
                error_message: experience.outcome.error_message,
                metadata: experience.outcome.metadata,
                performance_score: experience.outcome.performance_score,
                execution_time_ms: experience.outcome.execution_time_ms,
                learned_capabilities: experience.outcome.learned_capabilities,
            },
            memory_type: match experience.memory_type {
                agent_agency_contracts::types::memory::MemoryType::Episodic => agent_memory::memory_types::MemoryType::Episodic,
                agent_agency_contracts::types::memory::MemoryType::Semantic => agent_memory::memory_types::MemoryType::Semantic,
                agent_agency_contracts::types::memory::MemoryType::Procedural => agent_memory::memory_types::MemoryType::Procedural,
                agent_agency_contracts::types::memory::MemoryType::Working => agent_memory::memory_types::MemoryType::Working,
            },
            timestamp,
            metadata: experience.metadata,
        };

        // Store the experience using the real memory system
        let memory_id = self.memory_system.store_experience(memory_experience).await
            .map_err(|e| MemoryError::OperationFailed {
                operation: "store_experience".to_string(),
                reason: e.to_string(),
            })?;

        Ok(MemoryId(memory_id))
    }

    async fn retrieve_temporal_context(&self, query: TemporalQuery) -> MemoryResult<Vec<TemporalContext>> {
        // TODO: Implement comprehensive temporal context retrieval
        //       Currently returns empty vector; should implement proper temporal context retrieval using temporal engine to analyze patterns and extract contexts.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Temporal contexts are retrieved from memory system
        // - Temporal patterns are analyzed correctly
        // - Context extraction is accurate
        // - Time range queries work correctly
        //
        // DEPENDENCIES:
        // - Temporal engine (Required)
        // - Pattern analysis utilities (Required)
        // - Context extraction infrastructure (Required)
        //
        // ESTIMATED EFFORT: 5-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (memory feature)
        // - Change Budget: ~120 LOC
        // - Reviewer Requirements: Temporal analysis expertise
        // Temporary: empty vector until proper implementation
        let start = query.start_time.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
        let end = query.end_time.unwrap_or_else(chrono::Utc::now);
        Ok(Vec::new())
    }

    async fn record_outcome(&self, memory_id: MemoryId, outcome: ExperienceOutcome) -> MemoryResult<()> {
        // Retrieve the existing experience
        let uuid_id = memory_id.0;
        let mut experience = self.memory_system.manager().retrieve_memory(uuid_id).await
            .map_err(|e| MemoryError::NotFound {
                memory_id: uuid_id.to_string(),
            })?;

        // Update the outcome
        experience.outcome = agent_memory::memory_types::ExperienceOutcome {
            success: outcome.success,
            quality_score: outcome.quality_score,
            error_message: outcome.error_message,
            metadata: outcome.metadata,
            performance_score: outcome.performance_score,
            execution_time_ms: outcome.execution_time_ms,
            learned_capabilities: outcome.learned_capabilities,
        };

        // Store the updated experience back
        // Note: This creates a new memory entry - proper implementation would update in place
        self.memory_system.store_experience(experience).await
            .map_err(|e| MemoryError::OperationFailed {
                operation: "record_outcome".to_string(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    async fn retrieve_experience(&self, memory_id: MemoryId) -> MemoryResult<Experience> {
        // Retrieve the experience using the memory manager
        let uuid_id = memory_id.0;
        let memory_experience = self.memory_system.manager().retrieve_memory(uuid_id).await
            .map_err(|e| MemoryError::NotFound {
                memory_id: uuid_id.to_string(),
            })?;

        // Convert back to contracts types
        // AgentExperience has fields nested in context
        // memory_experience.id is MemoryId (Uuid), so we can use it directly
        let experience = Experience {
            id: memory_experience.id,
            description: memory_experience.context.description.clone(),
            memory_type: match memory_experience.memory_type {
                agent_memory::memory_types::MemoryType::Episodic => agent_agency_contracts::types::memory::MemoryType::Episodic,
                agent_memory::memory_types::MemoryType::Semantic => agent_agency_contracts::types::memory::MemoryType::Semantic,
                agent_memory::memory_types::MemoryType::Procedural => agent_agency_contracts::types::memory::MemoryType::Procedural,
                agent_memory::memory_types::MemoryType::Working => agent_agency_contracts::types::memory::MemoryType::Working,
            },
            temporal_context: memory_experience.context.temporal_context.map(|tc| {
                TemporalContext {
                    timestamp: tc.timestamp,
                    duration_ms: tc.duration.map(|d| d.num_milliseconds() as u64),
                    sequence_number: tc.sequence_number,
                    priority: match tc.priority {
                        agent_memory::memory_types::TaskPriority::Low => agent_agency_contracts::types::memory::TaskPriority::Low,
                        agent_memory::memory_types::TaskPriority::Normal => agent_agency_contracts::types::memory::TaskPriority::Normal,
                        agent_memory::memory_types::TaskPriority::Medium => agent_agency_contracts::types::memory::TaskPriority::Medium,
                        agent_memory::memory_types::TaskPriority::High => agent_agency_contracts::types::memory::TaskPriority::High,
                        agent_memory::memory_types::TaskPriority::Urgent => agent_agency_contracts::types::memory::TaskPriority::Urgent,
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
            domain: memory_experience.context.domain.clone(),
            task_type: memory_experience.context.task_type.clone(),
            metadata: memory_experience.metadata,
        };

        Ok(experience)
    }

    async fn search_experiences(&self, query: serde_json::Value) -> MemoryResult<Vec<Experience>> {
        use agent_memory::memory_manager::MemoryQuery;
        use agent_memory::memory_types::{MemoryType, TimeRange};
        use chrono::{DateTime, Utc};

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
                Some(TimeRange {
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

        // Search memories using the real memory system
        let agent_experiences = self.memory_system.manager().search_memories(memory_query).await
            .map_err(|e| MemoryError::OperationFailed {
                operation: "search_experiences".to_string(),
                reason: e.to_string(),
            })?;

        // Convert agent_memory::AgentExperience to contracts::Experience
        let mut results = Vec::new();
        for agent_exp in agent_experiences {
            let experience = Experience {
                id: agent_exp.id,
                description: agent_exp.context.description.clone(),
                domain: agent_exp.context.domain.clone(),
                task_type: agent_exp.context.task_type.clone(),
                memory_type: match agent_exp.memory_type {
                    MemoryType::Episodic => agent_agency_contracts::types::memory::MemoryType::Episodic,
                    MemoryType::Semantic => agent_agency_contracts::types::memory::MemoryType::Semantic,
                    MemoryType::Procedural => agent_agency_contracts::types::memory::MemoryType::Procedural,
                    MemoryType::Working => agent_agency_contracts::types::memory::MemoryType::Working,
                },
                outcome: ExperienceOutcome {
                    success: agent_exp.outcome.success,
                    quality_score: agent_exp.outcome.quality_score,
                    error_message: agent_exp.outcome.error_message,
                    metadata: agent_exp.outcome.metadata,
                    performance_score: agent_exp.outcome.performance_score,
                    execution_time_ms: agent_exp.outcome.execution_time_ms,
                    learned_capabilities: agent_exp.outcome.learned_capabilities,
                },
                temporal_context: agent_exp.context.temporal_context.map(|tc| {
                    TemporalContext {
                        timestamp: tc.timestamp,
                        duration_ms: tc.duration.map(|d| d.num_milliseconds() as u64),
                        sequence_number: tc.sequence_number,
                        priority: match tc.priority {
                            agent_memory::memory_types::TaskPriority::Low => agent_agency_contracts::types::memory::TaskPriority::Low,
                            agent_memory::memory_types::TaskPriority::Normal => agent_agency_contracts::types::memory::TaskPriority::Normal,
                            agent_memory::memory_types::TaskPriority::Medium => agent_agency_contracts::types::memory::TaskPriority::Normal, // Map Medium to Normal
                            agent_memory::memory_types::TaskPriority::High => agent_agency_contracts::types::memory::TaskPriority::High,
                            agent_memory::memory_types::TaskPriority::Urgent => agent_agency_contracts::types::memory::TaskPriority::High, // Map Urgent to High
                            agent_memory::memory_types::TaskPriority::Critical => agent_agency_contracts::types::memory::TaskPriority::Critical,
                        },
                    }
                }),
                metadata: agent_exp.metadata,
            };
            results.push(experience);
        }

        Ok(results)
    }
}
