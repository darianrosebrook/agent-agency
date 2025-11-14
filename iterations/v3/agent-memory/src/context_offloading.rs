//! Context Offloading Module
//!
//! Handles offloading of context to external storage systems
//! for memory optimization and retrieval.

use crate::memory_types::{ContextualMemory, TaskContext};
use crate::MemoryResult;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use system_common_interfaces::memory::{
    MemoryId, MemoryQuery, MemoryRecord, MemoryService, WorkspaceId,
};
use uuid::Uuid;

/// Context offloading service using real memory persistence
pub struct ContextOffloadingService {
    memory_service: Arc<dyn MemoryService>,
    workspace_id: WorkspaceId,
}

impl ContextOffloadingService {
    /// Create a new context offloading service with a memory backend
    pub fn new(memory_service: Arc<dyn MemoryService>, workspace_id: WorkspaceId) -> Self {
        Self {
            memory_service,
            workspace_id,
        }
    }

    /// Offload context to external storage
    pub async fn offload_context(&self, context: TaskContext) -> MemoryResult<String> {
        // Convert TaskContext to MemoryRecord for storage
        let memory_record = MemoryRecord {
            id: MemoryId(uuid::Uuid::new_v4().to_string()),
            workspace_id: self.workspace_id.clone(),
            embedding: None, // No embedding for task context
            content: serde_json::to_string(&context).map_err(|e| {
                crate::MemoryError::Serialization(format!("Failed to serialize context: {}", e))
            })?,
            metadata: HashMap::from([
                (
                    "context_type".to_string(),
                    serde_json::json!("task_context"),
                ),
                ("task_id".to_string(), serde_json::json!(context.task_id)),
                ("agent_id".to_string(), serde_json::json!(context.agent_id)),
                (
                    "task_type".to_string(),
                    serde_json::json!(context.task_type),
                ),
            ]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed: Some(Utc::now()),
            importance: 0.7,    // Moderate importance for task contexts
            decay_factor: 0.95, // Slow decay
        };

        // Store in memory service
        let stored = self
            .memory_service
            .create(memory_record)
            .await
            .map_err(|e| {
                crate::MemoryError::Persistence(format!("Failed to store context: {}", e))
            })?;

        Ok(stored.id.0)
    }

    /// Retrieve offloaded context
    pub async fn retrieve_context(&self, context_id: &str) -> MemoryResult<TaskContext> {
        // Query memory service for the context
        let memory_id = MemoryId(context_id.to_string());

        let record = self.memory_service.get(&memory_id).await.map_err(|e| {
            crate::MemoryError::Persistence(format!("Failed to retrieve context: {}", e))
        })?;

        match record {
            Some(record) => {
                // Deserialize the TaskContext from content
                let context: TaskContext = serde_json::from_str(&record.content).map_err(|e| {
                    crate::MemoryError::Serialization(format!(
                        "Failed to deserialize context: {}",
                        e
                    ))
                })?;

                // Update last accessed time
                self.memory_service
                    .touch(&memory_id, Utc::now())
                    .await
                    .map_err(|e| {
                        crate::MemoryError::Persistence(format!(
                            "Failed to update access time: {}",
                            e
                        ))
                    })?;

                Ok(context)
            }
            None => Err(crate::MemoryError::NotFound(format!(
                "Context {} not found",
                context_id
            ))),
        }
    }
}
