//! Memory Service Adapter
//!
//! Adapts `agent-memory` implementations to `data-interfaces` service traits.

use async_trait::async_trait;
use data_interfaces::service_contracts::{
    MemoryService, ServiceError, MemoryContent,
};
use agent_agency_contracts::types::memory::{MemoryType, MemoryId};
use std::sync::Arc;
use agent_memory::memory_manager::{MemoryManager, MemoryQuery as AgentMemoryQuery};
use agent_memory::memory_types::{AgentExperience, MemoryConfig, ExperienceOutcome, ExperienceContext};
use sqlx::PgPool;

/// Adapter for memory service
pub struct MemoryServiceAdapter {
    memory_manager: Arc<MemoryManager>,
}

impl MemoryServiceAdapter {
    /// Create a new memory service adapter
    pub async fn new(config: MemoryConfig, db_pool: PgPool) -> Result<Self, ServiceError> {
        let memory_manager = MemoryManager::new(config, db_pool)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to create MemoryManager: {}", e)))?;
        
        Ok(Self {
            memory_manager: Arc::new(memory_manager),
        })
    }
}

#[async_trait]
impl MemoryService for MemoryServiceAdapter {
    async fn store_memory(
        &self,
        memory_type: MemoryType,
        content: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<MemoryId, ServiceError> {
        // Convert MemoryType to agent-memory's MemoryType
        use agent_memory::memory_types::MemoryType as AgentMemoryType;
        let agent_memory_type = match memory_type {
            MemoryType::Episodic => AgentMemoryType::Episodic,
            MemoryType::Semantic => AgentMemoryType::Semantic,
            MemoryType::Procedural => AgentMemoryType::Procedural,
            MemoryType::Working => AgentMemoryType::Working,
        };
        
        // Convert metadata from Option<Value> to HashMap<String, Value>
        let metadata_map = metadata
            .and_then(|v| serde_json::from_value::<std::collections::HashMap<String, serde_json::Value>>(v).ok())
            .unwrap_or_default();
        
        // Create AgentExperience from content
        let experience_id = uuid::Uuid::new_v4();
        let experience = AgentExperience {
            id: experience_id,
            agent_id: "default-agent".to_string(), // TODO: Get from context
            task_id: "default-task".to_string(),   // TODO: Get from context
            content: content.clone(),
            input: content.clone(),
            output: content,
            context: ExperienceContext {
                description: "Memory storage operation".to_string(),
                domain: vec!["memory".to_string()],
                task_type: "store".to_string(),
                temporal_context: None,
            },
            outcome: ExperienceOutcome {
                success: true,
                quality_score: 1.0,
                error_message: None,
                metadata: metadata_map.clone(),
                performance_score: None,
                execution_time_ms: None,
                learned_capabilities: vec![],
            },
            memory_type: agent_memory_type,
            timestamp: chrono::Utc::now(),
            metadata: metadata_map,
        };
        
        // Store experience - returns Uuid (MemoryId type alias)
        let memory_id_uuid = self.memory_manager.store_experience(experience)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to store memory: {}", e)))?;
        
        // Wrap Uuid in MemoryId newtype wrapper
        Ok(MemoryId(memory_id_uuid))
    }
    
    async fn retrieve_memory(
        &self,
        memory_id: &MemoryId,
    ) -> Result<MemoryContent, ServiceError> {
        // Unwrap MemoryId newtype to get Uuid
        let memory_id_uuid = memory_id.0;
        
        // Retrieve experience
        let experience = self.memory_manager.retrieve_memory(memory_id_uuid)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to retrieve memory: {}", e)))?;

        // Convert to MemoryContent
        use agent_agency_contracts::types::memory::MemoryType as ContractsMemoryType;
        let memory_type = match experience.memory_type {
            agent_memory::memory_types::MemoryType::Episodic => ContractsMemoryType::Episodic,
            agent_memory::memory_types::MemoryType::Semantic => ContractsMemoryType::Semantic,
            agent_memory::memory_types::MemoryType::Procedural => ContractsMemoryType::Procedural,
            agent_memory::memory_types::MemoryType::Working => ContractsMemoryType::Working,
        };

        Ok(MemoryContent {
            memory_id: *memory_id,
            memory_type,
            content: experience.output,
            metadata: serde_json::to_value(experience.metadata).ok(),
            created_at: experience.timestamp,
        })
    }
    
    async fn query_memories(
        &self,
        query: data_interfaces::service_contracts::MemoryQuery,
    ) -> Result<Vec<MemoryContent>, ServiceError> {
        // Convert query to agent-memory query format
        // MemoryQuery in agent-memory has: agent_id, task_type, memory_type, time_range, limit
        let agent_query = AgentMemoryQuery {
            agent_id: None, // TODO: Get from context
            task_type: None, // TODO: Extract from query_text if needed
            memory_type: query.memory_type.map(|mt| {
                match mt {
                    MemoryType::Episodic => agent_memory::memory_types::MemoryType::Episodic,
                    MemoryType::Semantic => agent_memory::memory_types::MemoryType::Semantic,
                    MemoryType::Procedural => agent_memory::memory_types::MemoryType::Procedural,
                    MemoryType::Working => agent_memory::memory_types::MemoryType::Working,
                }
            }),
            time_range: None, // TODO: Add time range support if needed
            limit: query.limit,
        };
        
        // Search memories
        let experiences = self.memory_manager.search_memories(agent_query)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to query memories: {}", e)))?;
        
        // Convert to MemoryContent
        let results: Vec<MemoryContent> = experiences.into_iter().map(|exp| {
            use agent_agency_contracts::types::memory::MemoryType as ContractsMemoryType;
            let memory_type = match exp.memory_type {
                agent_memory::memory_types::MemoryType::Episodic => ContractsMemoryType::Episodic,
                agent_memory::memory_types::MemoryType::Semantic => ContractsMemoryType::Semantic,
                agent_memory::memory_types::MemoryType::Procedural => ContractsMemoryType::Procedural,
                agent_memory::memory_types::MemoryType::Working => ContractsMemoryType::Working,
            };

            MemoryContent {
                memory_id: MemoryId(exp.id),
                memory_type,
                content: exp.output,
                metadata: serde_json::to_value(exp.metadata).ok(),
                created_at: exp.timestamp,
            }
        }).collect();
        
        Ok(results)
    }
}
