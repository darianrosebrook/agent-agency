//! Session Manager for Multi-Session Continuity
//!
//! Manages sessions and enables cross-session context retrieval for long-horizon tasks.
//! Links tasks to sessions and preserves context across multiple sessions.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[cfg(feature = "memory")]
use agent_memory::MemorySystem;
#[cfg(feature = "memory")]
use agent_memory::memory_types::TaskContext;

/// Represents a session context for multi-session continuity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session ID
    pub session_id: Uuid,
    /// Tenant/user ID this session belongs to
    pub tenant_id: Uuid,
    /// Session name/description
    pub name: String,
    /// Session description
    pub description: Option<String>,
    /// Tasks linked to this session
    pub task_ids: Vec<Uuid>,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Session status
    pub status: SessionStatus,
    /// Context metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session is paused
    Paused,
    /// Session is completed
    Completed,
    /// Session is archived
    Archived,
}

/// Session Manager for managing sessions and cross-session context
pub struct SessionManager {
    /// Active sessions (session_id -> SessionContext)
    sessions: Arc<RwLock<HashMap<Uuid, SessionContext>>>,
    /// Task to session mapping (task_id -> session_id)
    task_to_session: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    /// Memory system for context retrieval
    #[cfg(feature = "memory")]
    memory_system: Option<Arc<MemorySystem>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(
        #[cfg(feature = "memory")]
        memory_system: Option<Arc<MemorySystem>>,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            task_to_session: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "memory")]
            memory_system,
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        tenant_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        let session = SessionContext {
            session_id,
            tenant_id,
            name,
            description,
            task_ids: Vec::new(),
            created_at: now,
            updated_at: now,
            status: SessionStatus::Active,
            metadata: HashMap::new(),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
        }

        Ok(session_id)
    }

    /// Link a task to a session
    pub async fn link_task_to_session(&self, task_id: Uuid, session_id: Uuid) -> Result<()> {
        // Verify session exists
        {
            let sessions = self.sessions.read().await;
            if !sessions.contains_key(&session_id) {
                return Err(anyhow::anyhow!("Session {} not found", session_id));
            }
        }

        // Add task to session
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                if !session.task_ids.contains(&task_id) {
                    session.task_ids.push(task_id);
                    session.updated_at = Utc::now();
                }
            }
        }

        // Update task to session mapping
        {
            let mut mapping = self.task_to_session.write().await;
            mapping.insert(task_id, session_id);
        }

        Ok(())
    }

    /// Get session context by session ID
    pub async fn get_session_context(&self, session_id: Uuid) -> Result<Option<SessionContext>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(&session_id).cloned())
    }

    /// Get session ID for a task
    pub async fn get_session_for_task(&self, task_id: Uuid) -> Option<Uuid> {
        let mapping = self.task_to_session.read().await;
        mapping.get(&task_id).copied()
    }

    /// Retrieve context from previous sessions (cross-session context retrieval)
    #[cfg(feature = "memory")]
    pub async fn retrieve_cross_session_context(
        &self,
        session_id: Uuid,
        limit: usize,
    ) -> Result<Vec<TaskContext>> {
        if let Some(ref memory) = self.memory_system {
            // Get all tasks in this session
            let task_ids = {
                let sessions = self.sessions.read().await;
                sessions.get(&session_id)
                    .map(|s| s.task_ids.clone())
                    .unwrap_or_default()
            };

            // Retrieve contexts for all tasks in this session
            let mut contexts = Vec::new();
            for task_id in task_ids {
                // Create a search context for this task
                let search_context = TaskContext {
                    task_id: task_id.to_string(),
                    agent_id: "session_manager".to_string(),
                    task_type: "orchestration".to_string(),
                    keywords: vec![],
                    entities: vec![],
                    timestamp: Utc::now(),
                    description: format!("Searching for context for task {}", task_id),
                };

                // Retrieve contextual memories
                match memory.retrieve_contextual_memories(&search_context, limit).await {
                    Ok(memories) => {
                        // Extract TaskContext from contextual memories
                        // Process memories in order (already sorted by relevance)
                        for contextual_memory in memories {
                            let agent_experience = &contextual_memory.memory;
                            
                            // Extract keywords and entities from metadata if available
                            let keywords: Vec<String> = agent_experience.metadata
                                .get("keywords")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_else(|| {
                                    // Fallback: extract keywords from context.domain
                                    agent_experience.context.domain.clone()
                                });
                            
                            let entities: Vec<String> = agent_experience.metadata
                                .get("entities")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            
                            // Create TaskContext from AgentExperience fields
                            let extracted_context = agent_memory::memory_types::TaskContext {
                                task_id: agent_experience.task_id.clone(),
                                agent_id: agent_experience.agent_id.clone(),
                                task_type: agent_experience.context.task_type.clone(),
                                keywords,
                                entities,
                                timestamp: agent_experience.timestamp,
                                description: agent_experience.context.description.clone(),
                            };
                            
                            contexts.push(extracted_context);
                            
                            // Stop if we've reached the limit
                            if contexts.len() >= limit {
                                break;
                            }
                        }
                        
                        // If no contexts were extracted, use search context as fallback
                        if contexts.is_empty() {
                            tracing::debug!("No TaskContext extracted from contextual memories, using search context as fallback");
                            contexts.push(search_context);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to retrieve contextual memories for task {}: {}", task_id, e);
                    }
                }
            }

            Ok(contexts)
        } else {
            Ok(Vec::new())
        }
    }

    /// Update session context
    pub async fn update_session_context(
        &self,
        session_id: Uuid,
        updates: SessionUpdate,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            match updates {
                SessionUpdate::Status(status) => {
                    session.status = status;
                }
                SessionUpdate::Name(name) => {
                    session.name = name;
                }
                SessionUpdate::Description(description) => {
                    session.description = Some(description);
                }
                SessionUpdate::Metadata(metadata) => {
                    session.metadata.extend(metadata);
                }
            }
            session.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session {} not found", session_id))
        }
    }

    /// Get all active sessions for a tenant
    pub async fn get_active_sessions(&self, tenant_id: Uuid) -> Vec<SessionContext> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|s| s.tenant_id == tenant_id && s.status == SessionStatus::Active)
            .cloned()
            .collect()
    }

    /// Archive a session
    pub async fn archive_session(&self, session_id: Uuid) -> Result<()> {
        self.update_session_context(session_id, SessionUpdate::Status(SessionStatus::Archived)).await
    }

    /// Complete a session
    pub async fn complete_session(&self, session_id: Uuid) -> Result<()> {
        self.update_session_context(session_id, SessionUpdate::Status(SessionStatus::Completed)).await
    }
}

/// Session update operations
#[derive(Debug, Clone)]
pub enum SessionUpdate {
    /// Update session status
    Status(SessionStatus),
    /// Update session name
    Name(String),
    /// Update session description
    Description(String),
    /// Update session metadata
    Metadata(HashMap<String, serde_json::Value>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_basic_operations() {
        let manager = SessionManager::new(
            #[cfg(feature = "memory")]
            None,
        );

        // Create a session
        let tenant_id = Uuid::new_v4();
        let session_id = manager.create_session(
            tenant_id,
            "Test Session".to_string(),
            Some("Test description".to_string()),
        ).await.unwrap();

        // Get session context
        let session = manager.get_session_context(session_id).await.unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().name, "Test Session");

        // Link a task to the session
        let task_id = Uuid::new_v4();
        manager.link_task_to_session(task_id, session_id).await.unwrap();

        // Verify task is linked
        let linked_session = manager.get_session_for_task(task_id).await;
        assert_eq!(linked_session, Some(session_id));

        // Get active sessions
        let active_sessions = manager.get_active_sessions(tenant_id).await;
        assert_eq!(active_sessions.len(), 1);
    }
}

