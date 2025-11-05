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
use agent_memory::memory_types::{AgentExperience, MemoryConfig, TimeRange};
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
        
        // Create AgentExperience from content
        let experience = AgentExperience {
            id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(), // TODO: Get from context
            task_id: uuid::Uuid::new_v4(),   // TODO: Get from context
            context: Default::default(),
            input: content.clone(),
            output: content,
            outcome: metadata.unwrap_or_default(),
            memory_type: agent_memory_type,
            timestamp: chrono::Utc::now(),
            metadata: metadata.unwrap_or_default(),
        };
        
        // Store experience
        let memory_id = self.memory_manager.store_experience(experience)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to store memory: {}", e)))?;
        
        Ok(memory_id)
    }
    
    async fn retrieve_memory(
        &self,
        memory_id: &MemoryId,
    ) -> Result<MemoryContent, ServiceError> {
        // Retrieve experience
        let experience = self.memory_manager.retrieve_memory(*memory_id)
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
            memory_id: experience.id,
            memory_type,
            content: experience.output,
            metadata: Some(experience.metadata),
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
                memory_id: exp.id,
                memory_type,
                content: exp.output,
                metadata: Some(exp.metadata),
                created_at: exp.timestamp,
            }
        }).collect();
        
        Ok(results)
    }
}
