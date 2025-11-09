//! Chat Service - Database-backed chat session and message management
//!
//! Provides APIs for creating chat sessions, sending messages,
//! and retrieving conversation history.

use crate::simple_client::DatabaseClient;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

/// Chat session representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub message_count: i32,
    pub metadata: serde_json::Value,
    pub archived: bool,
}

/// Chat message representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub token_count: Option<i32>,
    pub model_used: Option<String>,
    pub sequence_number: i32,
}

/// Chat service for managing conversations
pub struct ChatService {
    db_client: Arc<DatabaseClient>,
}

impl ChatService {
    /// Create a new chat service
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }

    /// Create a new chat session
    pub async fn create_session(
        &self,
        workspace_id: Option<Uuid>,
        tenant_id: Option<Uuid>,
        title: Option<String>,
        metadata: &serde_json::Value,
    ) -> Result<ChatSession> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO chat_sessions (
                id, workspace_id, tenant_id, title, created_at, updated_at,
                message_count, metadata, archived
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(session_id)
        .bind(workspace_id)
        .bind(tenant_id)
        .bind(&title)
        .bind(now)
        .bind(now)
        .bind(0)
        .bind(metadata)
        .bind(false)
        .execute(self.db_client.pool())
        .await?;

        Ok(ChatSession {
            id: session_id,
            workspace_id,
            tenant_id,
            title,
            created_at: now,
            updated_at: now,
            last_message_at: None,
            message_count: 0,
            metadata: metadata.clone(),
            archived: false,
        })
    }

    /// Send a message to a chat session
    pub async fn send_message(
        &self,
        session_id: Uuid,
        role: String,
        content: String,
        metadata: &serde_json::Value,
        token_count: Option<i32>,
        model_used: Option<String>,
    ) -> Result<ChatMessage> {
        // Get next sequence number
        let next_sequence: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM chat_messages WHERE session_id = $1"
        )
        .bind(session_id)
        .fetch_one(self.db_client.pool())
        .await?;

        let message_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO chat_messages (
                id, session_id, role, content, metadata, created_at,
                token_count, model_used, sequence_number
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(message_id)
        .bind(session_id)
        .bind(&role)
        .bind(&content)
        .bind(metadata)
        .bind(now)
        .bind(token_count)
        .bind(&model_used)
        .bind(next_sequence)
        .execute(self.db_client.pool())
        .await?;

        Ok(ChatMessage {
            id: message_id,
            session_id,
            role,
            content,
            metadata: metadata.clone(),
            created_at: now,
            edited_at: None,
            token_count,
            model_used,
            sequence_number: next_sequence,
        })
    }

    /// Get messages for a chat session
    pub async fn get_session_messages(
        &self,
        session_id: Uuid,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<ChatMessage>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, session_id, role, content, metadata, created_at,
                   edited_at, token_count, model_used, sequence_number
            FROM chat_messages
            WHERE session_id = $1
            ORDER BY sequence_number ASC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(session_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db_client.pool())
        .await?;

        let messages = rows
            .into_iter()
            .map(|row| ChatMessage {
                id: row.get("id"),
                session_id: row.get("session_id"),
                role: row.get("role"),
                content: row.get("content"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                edited_at: row.get("edited_at"),
                token_count: row.get("token_count"),
                model_used: row.get("model_used"),
                sequence_number: row.get("sequence_number"),
            })
            .collect();

        Ok(messages)
    }

    /// Get chat session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<ChatSession>> {
        let row = sqlx::query(
            r#"
            SELECT id, workspace_id, tenant_id, title, created_at, updated_at,
                   last_message_at, message_count, metadata, archived
            FROM chat_sessions WHERE id = $1
            "#
        )
        .bind(session_id)
        .fetch_optional(self.db_client.pool())
        .await?;

        Ok(row.map(|r| ChatSession {
            id: r.get("id"),
            workspace_id: r.get("workspace_id"),
            tenant_id: r.get("tenant_id"),
            title: r.get("title"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            last_message_at: r.get("last_message_at"),
            message_count: r.get("message_count"),
            metadata: r.get("metadata"),
            archived: r.get("archived"),
        }))
    }

    /// List chat sessions for a workspace
    pub async fn list_workspace_sessions(
        &self,
        workspace_id: Uuid,
        limit: Option<i32>,
        archived: Option<bool>,
    ) -> Result<Vec<ChatSession>> {
        let limit = limit.unwrap_or(20);
        let archived_filter = archived.unwrap_or(false);

        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, tenant_id, title, created_at, updated_at,
                   last_message_at, message_count, metadata, archived
            FROM chat_sessions
            WHERE workspace_id = $1 AND archived = $2
            ORDER BY updated_at DESC
            LIMIT $3
            "#
        )
        .bind(workspace_id)
        .bind(archived_filter)
        .bind(limit)
        .fetch_all(self.db_client.pool())
        .await?;

        let sessions = rows
            .into_iter()
            .map(|r| ChatSession {
                id: r.get("id"),
                workspace_id: r.get("workspace_id"),
                tenant_id: r.get("tenant_id"),
                title: r.get("title"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                last_message_at: r.get("last_message_at"),
                message_count: r.get("message_count"),
                metadata: r.get("metadata"),
                archived: r.get("archived"),
            })
            .collect();

        Ok(sessions)
    }

    /// Archive a chat session
    pub async fn archive_session(&self, session_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE chat_sessions SET archived = true, updated_at = NOW() WHERE id = $1"
        )
        .bind(session_id)
        .execute(self.db_client.pool())
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_chat_service_creation() {
        // TODO: Implement comprehensive chat service creation test with test database
        //       Currently just tests service creation with mock client; should implement comprehensive test that uses test database setup for complete chat service validation.
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
        // - Test uses test database setup
        // - Service creation is validated comprehensively
        // - Test covers error cases and edge conditions
        // - Test validates database integration
        //
        // DEPENDENCIES:
        // - Test database infrastructure (Required)
        // - Database setup/teardown utilities (Required)
        // - Test fixtures and data (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (test infrastructure enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Test infrastructure and database testing expertise
    }
}
