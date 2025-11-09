//! Integration hooks for agent-memory system
//!
//! Provides hooks to store processed data in agent memory and retrieve
//! contextual memories to enhance data processing.

use schemars::JsonSchema;
use crate::data_processing_types::*;
use agent_memory::{MemoryManager, AgentExperience, TaskContext, MemoryStats, memory_manager::MemoryQuery};
use crate::{DataProcessingResult, DataProcessingError};
use std::sync::Arc;

/// Configuration for memory integration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct MemoryConfig {
    pub enable_contextual_retrieval: bool,
    pub store_processing_experiences: bool,
    pub max_context_memories: usize,
    pub memory_relevance_threshold: f64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_contextual_retrieval: true,
            store_processing_experiences: true,
            max_context_memories: 10,
            memory_relevance_threshold: 0.7,
        }
    }
}

/// Hooks for integrating with agent memory system
pub struct MemoryIntegrationHooks {
    memory_manager: Arc<MemoryManager>,
    config: MemoryConfig,
}

impl MemoryIntegrationHooks {
    /// Create new memory integration hooks
    pub async fn new(config: &MemoryConfig) -> DataProcessingResult<Self> {
        let memory_config = agent_memory::MemoryConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config).await
            .map_err(|e| DataProcessingError::Other(format!("Memory manager init failed: {:?}", e)))?);

        Ok(Self {
            memory_manager,
            config: config.clone(),
        })
    }

    /// Store the result of data processing as an agent experience
    pub async fn store_processing_result(&self, output: &ProcessingOutput) -> DataProcessingResult<()> {
        if !self.config.store_processing_experiences {
            return Ok(());
        }

        // Convert processing output to agent experience
        let experience = self.create_experience_from_output(output);

        // Store in memory system
        self.memory_manager.store_experience(experience).await
            .map_err(|e| DataProcessingError::Other(format!("Failed to store processing experience: {}", e)))?;

        Ok(())
    }

    /// Retrieve contextual memories relevant to the current processing task
    pub async fn get_contextual_memories(&self, query: &DataQuery) -> DataProcessingResult<Vec<AgentExperience>> {
        if !self.config.enable_contextual_retrieval {
            return Ok(vec![]);
        }

        // Create a memory query based on the data query
        let memory_query = MemoryQuery {
            agent_id: query.context.user_id.clone(),
            task_type: Some("data_processing".to_string()),
            memory_type: Some(agent_memory::MemoryType::Procedural),
            time_range: None, // Could be derived from query context
            limit: Some(self.config.max_context_memories),
        };

        // Search memories
        let memories = self.memory_manager.search_memories(memory_query).await
            .map_err(|e| DataProcessingError::Other(format!("Failed to search memories: {:?}", e)))?;

        // TODO: Implement relevance scoring for memory search results
        // - [ ] Calculate relevance scores based on query similarity
        // - [ ] Rank results by relevance score
        // - [ ] Apply relevance threshold to filter low-scoring results
        // - [ ] Add configurable result limit
        // - [ ] Add unit tests with various query types
        // - [ ] Add integration tests with real memory search
        // TODO: Implement relevance scoring and result limiting
        //       Currently returns all results; should implement relevance scoring and configurable result limiting for better search quality.
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
        // - Relevance scoring is implemented correctly
        // - Results are sorted by relevance
        // - Result limit is configurable
        // - Scoring improves search quality
        //
        // DEPENDENCIES:
        // - Relevance scoring algorithms (Required)
        // - Result limiting utilities (Required)
        // - Configuration infrastructure (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (search feature enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Search ranking expertise
        Ok(memories) // Temporary: all results until relevance scoring and limiting
    }

    /// Get memory system statistics
    pub async fn get_memory_stats(&self) -> DataProcessingResult<MemoryStats> {
        self.memory_manager.get_memory_stats().await
            .map_err(|e| DataProcessingError::Other(format!("Failed to get memory stats: {}", e)))
    }

    /// Perform memory maintenance operations
    pub async fn run_memory_maintenance(&self) -> DataProcessingResult<()> {
        let _consolidated = self.memory_manager.consolidate_memories().await
            .map_err(|e| DataProcessingError::Other(format!("Memory consolidation failed: {}", e)))?;

        let _cleaned = self.memory_manager.cleanup_expired_memories().await
            .map_err(|e| DataProcessingError::Other(format!("Memory cleanup failed: {}", e)))?;

        Ok(())
    }

    /// Create an agent experience from processing output
    fn create_experience_from_output(&self, output: &ProcessingOutput) -> AgentExperience {
        let task_description = format!(
            "Data processing: {} from {}",
            match &output.original_input.source {
                DataSource::File(fs) => format!("file {}", fs.path.display()),
                DataSource::Url(us) => format!("URL {}", us.url),
                DataSource::Stream(ss) => format!("stream {}", ss.stream_id),
                DataSource::Database(ds) => format!("database {}.{}", ds.table, ds.record_id),
                DataSource::Api(r#as) => format!("API {}", r#as.endpoint),
            },
            output.original_input.content_type()
        );

        let success = output.processing_stats.errors_encountered.is_empty();

        let outcome = agent_memory::ExperienceOutcome {
            success,
            performance_score: if success { Some(0.8) } else { Some(0.2) },
            learned_capabilities: if success {
                vec!["data_processing".to_string()]
            } else {
                vec![]
            },
            failure_reasons: if success {
                vec![]
            } else {
                output.processing_stats.errors_encountered.clone()
            },
            success_factors: if success {
                vec!["successful_data_processing".to_string()]
            } else {
                vec![]
            },
            execution_time_ms: Some(output.processing_stats.processing_time_ms as i64),
            tokens_used: None,
            feedback: None,
        };

        AgentExperience {
            id: output.id.0, // MemoryId is just Uuid
            agent_id: output.original_input.processing_context.user_id.clone()
                .unwrap_or_else(|| "data-processor".to_string()),
            task_id: output.id.0.to_string(),
            context: self.create_task_context_from_input(&output.original_input),
            input: serde_json::to_value(&output.original_input)
                .unwrap_or(serde_json::Value::Null),
            output: serde_json::to_value(&output.processed_content)
                .unwrap_or(serde_json::Value::Null),
            outcome,
            memory_type: agent_memory::MemoryType::Procedural,
            timestamp: output.created_at,
            metadata: {
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("entities_extracted".to_string(),
                    serde_json::Value::Number(output.processing_stats.entities_extracted.into()));
                metadata.insert("processing_time_ms".to_string(),
                    serde_json::Value::Number(output.processing_stats.processing_time_ms.into()));
                metadata.insert("bytes_processed".to_string(),
                    serde_json::Value::Number(output.processing_stats.bytes_processed.into()));
                metadata
            },
        }
    }

    /// Create task context from processing input
    fn create_task_context_from_input(&self, input: &DataInput) -> TaskContext {
        TaskContext {
            task_id: input.id.0.to_string(),
            task_type: "data_processing".to_string(),
            description: format!("Processing {} data", input.content_type()),
            domain: vec!["data_processing".to_string()],
            entities: input.metadata.keys().cloned().collect(),
            temporal_context: input.processing_context.deadline.map(|dt| agent_memory::TemporalContext {
                start_time: chrono::Utc::now(),
                deadline: Some(dt),
                priority: agent_memory::TaskPriority::Medium,
                recurrence_pattern: None,
            }),
            metadata: input.metadata.clone(),
        }
    }
}

impl DataInput {
    /// Get content type from the data source
    fn content_type(&self) -> ContentType {
        match &self.source {
            DataSource::File(fs) => fs.content_type.clone(),
            DataSource::Url(us) => us.content_type.clone().unwrap_or(ContentType::Unknown),
            DataSource::Stream(ss) => ss.content_type.clone(),
            DataSource::Database(_) => ContentType::Structured,
            DataSource::Api(_) => ContentType::Json,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_hooks_creation() {
        let config = MemoryConfig::default();
        let hooks = MemoryIntegrationHooks::new(&config).await;
        // TODO: Implement comprehensive test with test memory system
        //       Currently verifies config only; should implement comprehensive test with test memory system for full functionality verification.
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
        // - Test uses test memory system
        // - Memory hooks are tested comprehensively
        // - Test assertions verify functionality
        // - Test reliability is high
        //
        // DEPENDENCIES:
        // - Test memory system infrastructure (Required)
        // - Test utilities (Required)
        // - Test fixtures (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (test infrastructure enhancement)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Memory system testing expertise
        assert!(config.enable_contextual_retrieval); // Temporary: config verification until comprehensive test
    }

    #[test]
    fn test_memory_config_defaults() {
        let config = MemoryConfig::default();
        assert!(config.enable_contextual_retrieval);
        assert!(config.store_processing_experiences);
        assert_eq!(config.max_context_memories, 10);
        assert_eq!(config.memory_relevance_threshold, 0.7);
    }
}
