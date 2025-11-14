//! Chat Service - Database-backed chat session and message management
//!
//! Provides APIs for creating chat sessions, sending messages,
//! and retrieving conversation history.

use crate::database_metrics::DatabaseMetrics;
use crate::simple_client::DatabaseClient;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use std::time::Instant;
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
    pub pinned: bool,
    pub folder_id: Option<Uuid>,
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
    metrics: Option<Arc<DatabaseMetrics>>,
}

impl ChatService {
    /// Create a new chat service
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self {
            db_client,
            metrics: None,
        }
    }

    /// Create a new chat service with metrics
    pub fn with_metrics(db_client: Arc<DatabaseClient>, metrics: Arc<DatabaseMetrics>) -> Self {
        Self {
            db_client,
            metrics: Some(metrics),
        }
    }

    /// Record query execution time
    fn record_query_time(&self, start: Instant, success: bool) {
        if let Some(metrics) = &self.metrics {
            let duration = start.elapsed();
            metrics.record_query_execution(duration);
            if success {
                metrics.record_successful_query();
            } else {
                metrics.record_failed_query();
            }
        }
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
            "#,
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
        .bind(false) // pinned
        .bind(None::<Uuid>) // folder_id
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
            pinned: false,
            folder_id: None,
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
        let start = Instant::now();

        // Get next sequence number using optimized function
        // This uses the database function which is more efficient than MAX()
        let next_sequence: i32 = sqlx::query_scalar("SELECT get_next_sequence_number($1)")
            .bind(session_id)
            .fetch_one(self.db_client.pool())
            .await
            .map_err(|e| {
                self.record_query_time(start, false);
                e
            })?;

        let message_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO chat_messages (
                id, session_id, role, content, metadata, created_at,
                token_count, model_used, sequence_number
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
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
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);

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

    /// Get messages for a chat session with pagination
    pub async fn get_session_messages(
        &self,
        session_id: Uuid,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<ChatMessage>> {
        let start = Instant::now();
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
            "#,
        )
        .bind(session_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db_client.pool())
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);

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

    /// Get messages for a chat session using cursor-based pagination (more efficient)
    pub async fn get_session_messages_cursor(
        &self,
        session_id: Uuid,
        cursor: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<ChatMessage>> {
        let start = Instant::now();
        let cursor = cursor.unwrap_or(0);
        let limit = limit.unwrap_or(50);

        let rows = sqlx::query(
            r#"
            SELECT id, session_id, role, content, metadata, created_at,
                   edited_at, token_count, model_used, sequence_number
            FROM get_chat_messages_cursor($1, $2, $3)
            "#,
        )
        .bind(session_id)
        .bind(cursor)
        .bind(limit)
        .fetch_all(self.db_client.pool())
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);

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

    /// Get total message count for a session (for pagination)
    pub async fn get_message_count(&self, session_id: Uuid) -> Result<i32> {
        let start = Instant::now();

        let count: i32 = sqlx::query_scalar("SELECT get_chat_messages_count($1)")
            .bind(session_id)
            .fetch_one(self.db_client.pool())
            .await
            .map_err(|e| {
                self.record_query_time(start, false);
                e
            })?;

        self.record_query_time(start, true);
        Ok(count)
    }

    /// Get chat session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<ChatSession>> {
        let start = Instant::now();

        let row = sqlx::query(
            r#"
            SELECT id, workspace_id, tenant_id, title, created_at, updated_at,
                   last_message_at, message_count, metadata, archived, pinned, folder_id
            FROM chat_sessions WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(self.db_client.pool())
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);

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
            pinned: r.get("pinned"),
            folder_id: r.get("folder_id"),
        }))
    }

    /// List chat sessions for a workspace with pagination
    pub async fn list_workspace_sessions(
        &self,
        workspace_id: Uuid,
        limit: Option<i32>,
        offset: Option<i32>,
        archived: Option<bool>,
    ) -> Result<Vec<ChatSession>> {
        let start = Instant::now();
        let limit = limit.unwrap_or(20);
        let offset = offset.unwrap_or(0);
        let archived_filter = archived.unwrap_or(false);

        // This query uses the composite index idx_chat_sessions_workspace_archived_updated
        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, tenant_id, title, created_at, updated_at,
                   last_message_at, message_count, metadata, archived, pinned, folder_id
            FROM chat_sessions
            WHERE workspace_id = $1 AND archived = $2
            ORDER BY updated_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(workspace_id)
        .bind(archived_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db_client.pool())
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);

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
                pinned: r.get("pinned"),
                folder_id: r.get("folder_id"),
            })
            .collect();

        Ok(sessions)
    }

    /// Get total session count for a workspace (for pagination)
    pub async fn get_session_count(
        &self,
        workspace_id: Uuid,
        archived: Option<bool>,
    ) -> Result<i32> {
        let start = Instant::now();
        let archived_filter = archived.unwrap_or(false);

        let count: i32 = sqlx::query_scalar("SELECT get_chat_sessions_count($1, $2)")
            .bind(workspace_id)
            .bind(archived_filter)
            .fetch_one(self.db_client.pool())
            .await
            .map_err(|e| {
                self.record_query_time(start, false);
                e
            })?;

        self.record_query_time(start, true);
        Ok(count)
    }

    /// Archive a chat session
    pub async fn archive_session(&self, session_id: Uuid) -> Result<()> {
        let start = Instant::now();

        sqlx::query(
            "UPDATE chat_sessions SET archived = true, archived_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(session_id)
        .execute(self.db_client.pool())
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);
        Ok(())
    }

    /// Search chat sessions by text
    pub async fn search_sessions(
        &self,
        workspace_id: Uuid,
        search_text: &str,
        archived: Option<bool>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<ChatSession>> {
        let start = Instant::now();
        let archived_filter = archived.unwrap_or(false);
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, tenant_id, title, created_at, updated_at,
                   last_message_at, message_count, metadata, archived, pinned, folder_id
            FROM search_chat_sessions($1, $2, $3, $4, $5)
            "#,
        )
        .bind(workspace_id)
        .bind(search_text)
        .bind(archived_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db_client.pool())
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);

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
                pinned: r.get("pinned"),
                folder_id: r.get("folder_id"),
            })
            .collect();

        Ok(sessions)
    }

    /// Search messages within a session
    pub async fn search_messages(
        &self,
        session_id: Uuid,
        search_text: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<ChatMessage>> {
        let start = Instant::now();
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, session_id, role, content, created_at,
                   edited_at, token_count, model_used, sequence_number
            FROM search_chat_messages($1, $2, $3, $4)
            "#,
        )
        .bind(session_id)
        .bind(search_text)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db_client.pool())
        .await
        .map_err(|e| {
            self.record_query_time(start, false);
            e
        })?;

        self.record_query_time(start, true);

        let messages = rows
            .into_iter()
            .map(|row| ChatMessage {
                id: row.get("id"),
                session_id: row.get("session_id"),
                role: row.get("role"),
                content: row.get("content"),
                metadata: serde_json::json!({}), // Search results don't include full metadata
                created_at: row.get("created_at"),
                edited_at: row.get("edited_at"),
                token_count: row.get("token_count"),
                model_used: row.get("model_used"),
                sequence_number: row.get("sequence_number"),
            })
            .collect();

        Ok(messages)
    }

    /// Pin or unpin a session
    pub async fn pin_session(&self, session_id: Uuid, pinned: bool) -> Result<()> {
        let start = Instant::now();

        sqlx::query("UPDATE chat_sessions SET pinned = $1, updated_at = NOW() WHERE id = $2")
            .bind(pinned)
            .bind(session_id)
            .execute(self.db_client.pool())
            .await
            .map_err(|e| {
                self.record_query_time(start, false);
                e
            })?;

        self.record_query_time(start, true);
        Ok(())
    }

    /// Bulk archive sessions
    pub async fn bulk_archive_sessions(&self, session_ids: &[Uuid]) -> Result<i32> {
        let start = Instant::now();

        let count: i32 = sqlx::query_scalar("SELECT bulk_archive_sessions($1)")
            .bind(session_ids)
            .fetch_one(self.db_client.pool())
            .await
            .map_err(|e| {
                self.record_query_time(start, false);
                e
            })?;

        self.record_query_time(start, true);
        Ok(count)
    }

    /// Bulk delete sessions
    pub async fn bulk_delete_sessions(&self, session_ids: &[Uuid]) -> Result<i32> {
        let start = Instant::now();

        let count: i32 = sqlx::query_scalar("SELECT bulk_delete_sessions($1)")
            .bind(session_ids)
            .fetch_one(self.db_client.pool())
            .await
            .map_err(|e| {
                self.record_query_time(start, false);
                e
            })?;

        self.record_query_time(start, true);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::{ChatService, ChatSession};
    use sqlx::Row;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_chat_service_creation() {
        // Skip test if database is not available (for CI environments)
        if std::env::var("SKIP_DB_TESTS").is_ok() {
            return;
        }

        // Create database client with real connection
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency".to_string());
        
        let config = crate::database_config::DatabaseConfig {
            database_url: database_url.clone(),
            ..Default::default()
        };

        let db_client = match crate::simple_client::DatabaseClient::new(config).await {
            Ok(client) => std::sync::Arc::new(client),
            Err(e) => {
                eprintln!("Skipping test: Database not available: {}", e);
                return;
            }
        };

        // Create ChatService instance
        let chat_service = ChatService::new(db_client.clone());

        // Test 1: Create a chat session
        let workspace_id = Some(Uuid::new_v4());
        let tenant_id = Some(Uuid::new_v4());
        let title = Some("Test Chat Session".to_string());
        let metadata = serde_json::json!({
            "test": "data",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let session = chat_service
            .create_session(workspace_id, tenant_id, title.clone(), &metadata)
            .await
            .expect("Chat session creation should succeed");

        // Verify session was created correctly
        assert_eq!(session.title, title);
        assert_eq!(session.workspace_id, workspace_id);
        assert_eq!(session.tenant_id, tenant_id);
        assert_eq!(session.message_count, 0);
        assert_eq!(session.archived, false);
        assert_eq!(session.metadata, metadata);
        assert!(session.id != Uuid::nil());

        // Test 2: Query the session from database to verify persistence
        let query_result = sqlx::query(
            r#"
            SELECT id, workspace_id, tenant_id, title, created_at, updated_at,
                   last_message_at, message_count, metadata, archived, pinned, folder_id
            FROM chat_sessions
            WHERE id = $1
            "#
        )
        .bind(session.id)
        .fetch_optional(db_client.pool())
        .await;

        match query_result {
            Ok(Some(row)) => {
                let db_session = ChatSession {
                    id: row.try_get("id").unwrap(),
                    workspace_id: row.try_get("workspace_id").unwrap(),
                    tenant_id: row.try_get("tenant_id").unwrap(),
                    title: row.try_get("title").unwrap(),
                    created_at: row.try_get("created_at").unwrap(),
                    updated_at: row.try_get("updated_at").unwrap(),
                    last_message_at: row.try_get("last_message_at").unwrap(),
                    message_count: row.try_get("message_count").unwrap(),
                    metadata: row.try_get("metadata").unwrap(),
                    archived: row.try_get("archived").unwrap(),
                    pinned: row.try_get("pinned").unwrap(),
                    folder_id: row.try_get("folder_id").unwrap(),
                };
                
                assert_eq!(db_session.id, session.id);
                assert_eq!(db_session.title, title);
                assert_eq!(db_session.workspace_id, workspace_id);
                assert_eq!(db_session.tenant_id, tenant_id);
                assert_eq!(db_session.message_count, 0);
                assert_eq!(db_session.archived, false);
            }
            Ok(None) => {
                panic!("Chat session not found in database after creation");
            }
            Err(e) => {
                panic!("Failed to query chat session: {}", e);
            }
        }

        // Test 3: Test service creation with metrics
        let metrics = std::sync::Arc::new(crate::database_metrics::DatabaseMetrics::new());
        let chat_service_with_metrics = ChatService::with_metrics(db_client.clone(), metrics.clone());
        
        // Create another session with metrics enabled
        let session2 = chat_service_with_metrics
            .create_session(None, None, Some("Test Session 2".to_string()), &serde_json::json!({}))
            .await
            .expect("Chat session creation with metrics should succeed");

        assert_eq!(session2.title, Some("Test Session 2".to_string()));
        assert!(session2.id != Uuid::nil());
        assert!(session2.id != session.id);

        // Test 4: Test error handling - try to create session with invalid data
        // (This would require additional validation, but we can test that the service handles it)
        let invalid_metadata = serde_json::json!({
            "nested": {
                "deep": {
                    "structure": "test"
                }
            }
        });
        
        let session3 = chat_service
            .create_session(None, None, None, &invalid_metadata)
            .await
            .expect("Chat session should accept any valid JSON metadata");

        assert_eq!(session3.metadata, invalid_metadata);

        // Cleanup: Delete test sessions
        let _ = sqlx::query("DELETE FROM chat_sessions WHERE id = $1")
            .bind(session.id)
            .execute(db_client.pool())
            .await;
        
        let _ = sqlx::query("DELETE FROM chat_sessions WHERE id = $1")
            .bind(session2.id)
            .execute(db_client.pool())
            .await;
        
        let _ = sqlx::query("DELETE FROM chat_sessions WHERE id = $1")
            .bind(session3.id)
            .execute(db_client.pool())
            .await;
    }
}
