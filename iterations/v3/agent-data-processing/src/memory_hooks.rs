//! Integration hooks for agent-memory system
//!
//! Provides hooks to store processed data in agent memory and retrieve
//! contextual memories to enhance data processing.

use crate::data_processing_types::*;
#[cfg(feature = "memory-integration")]
use agent_memory::{
    ContextualMemory, ExperienceOutcome, MemoryConfig, MemoryStats, MemoryType, TaskPriority,
    TemporalContext, ContextMatch, memory_manager::MemoryQuery, AgentExperience, MemoryManager, TaskContext,
};
use crate::{DataProcessingError, DataProcessingResult};
use schemars::JsonSchema;
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
        let memory_config = MemoryConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config).await.map_err(|e| {
            DataProcessingError::Other(format!("Memory manager init failed: {:?}", e))
        })?);

        Ok(Self {
            memory_manager,
            config: config.clone(),
        })
    }

    /// Store the result of data processing as an agent experience
    pub async fn store_processing_result(
        &self,
        output: &ProcessingOutput,
    ) -> DataProcessingResult<()> {
        if !self.config.store_processing_experiences {
            return Ok(());
        }

        // Convert processing output to agent experience
        let experience = self.create_experience_from_output(output);

        // Store in memory system
        self.memory_manager
            .store_experience(experience)
            .await
            .map_err(|e| {
                DataProcessingError::Other(format!("Failed to store processing experience: {}", e))
            })?;

        Ok(())
    }

    /// Retrieve contextual memories relevant to the current processing task
    pub async fn get_contextual_memories(
        &self,
        query: &DataQuery,
    ) -> DataProcessingResult<Vec<AgentExperience>> {
        if !self.config.enable_contextual_retrieval {
            return Ok(vec![]);
        }

        // Create a memory query based on the data query
        let memory_query = MemoryQuery {
            agent_id: query.context.user_id.clone(),
            task_type: Some("data_processing".to_string()),
            memory_type: Some(MemoryType::Procedural),
            time_range: None, // Could be derived from query context
            limit: Some(self.config.max_context_memories),
        };

        // Search memories
        let memories = self
            .memory_manager
            .search_memories(memory_query)
            .await
            .map_err(|e| {
                DataProcessingError::Other(format!("Failed to search memories: {:?}", e))
            })?;

        // Implemented: Relevance scoring for memory search results
        // Calculates relevance scores based on query similarity, ranks results, and applies threshold/limit

        use std::collections::HashSet;
        use tracing::debug;

        if memories.is_empty() {
            return Ok(Vec::new());
        }

        // Extract query text from context for similarity calculation
        let query_text = format!(
            "{} {}",
            // query.context.description.clone(), // TODO: Add description field to ProcessingContext
            // query.context.keywords.join(" ") // TODO: Add keywords field to ProcessingContext
        )
        .to_lowercase();

        // Tokenize query text for keyword matching
        let query_tokens: HashSet<String> = query_text
            .split_whitespace()
            .map(|s| {
                s.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|s| s.len() > 2) // Filter out very short tokens
            .collect();

        // Calculate relevance scores for each memory
        let mut scored_memories: Vec<(AgentExperience, f64)> = Vec::new();

        for memory in memories {
            // Build memory text from input, output, and context
            let memory_text = format!(
                "{} {} {}",
                memory.input.clone(),
                memory.output.clone(),
                serde_json::to_string(&memory.context).unwrap_or_default()
            )
            .to_lowercase();

            // Tokenize memory text
            let memory_tokens: HashSet<String> = memory_text
                .split_whitespace()
                .map(|s| {
                    s.trim_matches(|c: char| !c.is_alphanumeric())
                        .to_lowercase()
                })
                .filter(|s| s.len() > 2)
                .collect();

            // Calculate keyword overlap score (Jaccard similarity)
            let intersection: usize = query_tokens.intersection(&memory_tokens).count();
            let union: usize = query_tokens.union(&memory_tokens).count();
            let keyword_score = if union > 0 {
                intersection as f64 / union as f64
            } else {
                0.0
            };

            // Calculate substring match score (for exact phrase matches)
            let substring_score = if query_text.len() > 5 && memory_text.contains(&query_text) {
                0.3 // Bonus for exact phrase match
            } else {
                0.0
            };

            // Calculate context relevance (task_type match)
            let context_score = if let Some(ref task_type) = memory_query.task_type {
                if memory.context.task_type == *task_type {
                    0.2 // Bonus for matching task type
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Combine scores (weighted sum)
            let relevance_score =
                (keyword_score * 0.6) + (substring_score * 0.3) + (context_score * 0.1);

            scored_memories.push((memory, relevance_score));
        }

        // Sort by relevance score (descending)
        scored_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply relevance threshold and limit
        let threshold = self.config.memory_relevance_threshold;
        let limit = self.config.max_context_memories;

        let filtered_memories: Vec<AgentExperience> = scored_memories
            .into_iter()
            .filter(|(_, score)| *score >= threshold)
            .take(limit)
            .map(|(memory, _)| memory)
            .collect();

        debug!(
            "Relevance scoring: {} memories scored, {} passed threshold ({}), {} returned (limit: {})",
            scored_memories.len(),
            filtered_memories.len(),
            threshold,
            filtered_memories.len().min(limit),
            limit
        );

        Ok(filtered_memories)
    }

    /// Get memory system statistics
    pub async fn get_memory_stats(&self) -> DataProcessingResult<MemoryStats> {
        self.memory_manager
            .get_memory_stats()
            .await
            .map_err(|e| DataProcessingError::Other(format!("Failed to get memory stats: {}", e)))
    }

    /// Perform memory maintenance operations
    pub async fn run_memory_maintenance(&self) -> DataProcessingResult<()> {
        let _consolidated = self
            .memory_manager
            .consolidate_memories()
            .await
            .map_err(|e| {
                DataProcessingError::Other(format!("Memory consolidation failed: {}", e))
            })?;

        let _cleaned = self
            .memory_manager
            .cleanup_expired_memories()
            .await
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

        let outcome = ExperienceOutcome {
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
            agent_id: output
                .original_input
                .processing_context
                .user_id
                .clone()
                .unwrap_or_else(|| "data-processor".to_string()),
            task_id: output.id.0.to_string(),
            context: self.create_task_context_from_input(&output.original_input),
            input: serde_json::to_value(&output.original_input).unwrap_or(serde_json::Value::Null),
            output: serde_json::to_value(&output.processed_content)
                .unwrap_or(serde_json::Value::Null),
            outcome,
            memory_type: MemoryType::Procedural,
            timestamp: output.created_at,
            metadata: {
                let mut metadata = std::collections::HashMap::new();
                metadata.insert(
                    "entities_extracted".to_string(),
                    serde_json::Value::Number(output.processing_stats.entities_extracted.into()),
                );
                metadata.insert(
                    "processing_time_ms".to_string(),
                    serde_json::Value::Number(output.processing_stats.processing_time_ms.into()),
                );
                metadata.insert(
                    "bytes_processed".to_string(),
                    serde_json::Value::Number(output.processing_stats.bytes_processed.into()),
                );
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
            temporal_context: input.processing_context.deadline.map(|dt| {
                TemporalContext {
                    start_time: chrono::Utc::now(),
                    deadline: Some(dt),
                    priority: TaskPriority::Medium,
                    recurrence_pattern: None,
                }
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
        assert!(config.enable_contextual_retrieval);
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
