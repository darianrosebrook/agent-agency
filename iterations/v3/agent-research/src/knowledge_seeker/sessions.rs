//! Research session management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{ResearchSession, ResearchQuery};

/// Session manager for research sessions

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
#[derive(Debug, Serialize, Deserialize) ]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, ResearchSession>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub async fn new() -> Result<Self> {
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new research session
    pub async fn create_session(&self, name: String, description: Option<String>) -> ResearchSession {
        let session = ResearchSession {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            queries: vec![],
            status: crate::SessionStatus::Active,
        };

        self.sessions.write().await.insert(session.id, session.clone());
        session
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Option<ResearchSession> {
        self.sessions.read().await.get(&session_id).cloned()
    }

    /// Add query to session
    pub async fn add_query_to_session(&self, session_id: Uuid, query: ResearchQuery) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.queries.push(query);
            session.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    /// Complete session
    pub async fn complete_session(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = crate::SessionStatus::Completed;
            session.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
}
