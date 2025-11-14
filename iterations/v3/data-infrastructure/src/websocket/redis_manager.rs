//! Redis-backed session management for WebSocket connections
//!
//! Provides distributed session storage for multi-instance deployments.
//! Adapted from open-webui patterns for agent-agency.
//!
//! @author @darianrosebrook

use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Redis session manager for WebSocket connections
pub struct RedisSessionManager {
    redis_client: Option<redis::Client>,
    local_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    session_ttl_seconds: u64,
}

impl RedisSessionManager {
    /// Create a new Redis session manager
    ///
    /// If `redis_url` is None, operates in local-only mode (single instance)
    pub async fn new(redis_url: Option<&str>) -> Result<Self, redis::RedisError> {
        let redis_client = if let Some(url) = redis_url {
            Some(redis::Client::open(url)?)
        } else {
            None
        };

        Ok(Self {
            redis_client,
            local_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_ttl_seconds: 86400, // 24 hours
        })
    }

    /// Register a session for a user
    pub async fn register_session(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), redis::RedisError> {
        // Update local cache
        {
            let mut sessions = self.local_sessions.write().await;
            sessions
                .entry(user_id.to_string())
                .or_insert_with(Vec::new)
                .push(session_id.to_string());
        }

        // Update Redis if available
        if let Some(ref _client) = self.redis_client {
            match self.register_session_redis(user_id, session_id).await {
                Ok(_) => {
                    info!(
                        "Registered session {} for user {} in Redis",
                        session_id, user_id
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to register session in Redis: {}. Using local cache only.",
                        e
                    );
                    // Continue with local cache only
                }
            }
        }

        Ok(())
    }

    async fn register_session_redis(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), redis::RedisError> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_async_connection().await?;
            let key = format!("user_sessions:{}", user_id);

            // Add to Redis set
            conn.sadd::<_, _, ()>(&key, session_id).await?;

            // Set expiration (24 hours)
            conn.expire::<_, ()>(&key, self.session_ttl_seconds as i64)
                .await?;
        }

        Ok(())
    }

    /// Unregister a session for a user
    pub async fn unregister_session(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), redis::RedisError> {
        // Update local cache
        {
            let mut sessions = self.local_sessions.write().await;
            if let Some(user_sessions) = sessions.get_mut(user_id) {
                user_sessions.retain(|s| s != session_id);
                if user_sessions.is_empty() {
                    sessions.remove(user_id);
                }
            }
        }

        // Update Redis if available
        if let Some(ref _client) = self.redis_client {
            match self.unregister_session_redis(user_id, session_id).await {
                Ok(_) => {
                    info!(
                        "Unregistered session {} for user {} from Redis",
                        session_id, user_id
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to unregister session from Redis: {}. Using local cache only.",
                        e
                    );
                    // Continue with local cache only
                }
            }
        }

        Ok(())
    }

    async fn unregister_session_redis(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), redis::RedisError> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_async_connection().await?;
            let key = format!("user_sessions:{}", user_id);

            // Remove from Redis set
            conn.srem::<_, _, ()>(&key, session_id).await?;
        }

        Ok(())
    }

    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<String>, redis::RedisError> {
        // Check local cache first
        {
            let sessions = self.local_sessions.read().await;
            if let Some(user_sessions) = sessions.get(user_id) {
                return Ok(user_sessions.clone());
            }
        }

        // Fallback to Redis if available
        if let Some(ref _client) = self.redis_client {
            match self.get_user_sessions_redis(user_id).await {
                Ok(sessions) => {
                    // Update local cache
                    {
                        let mut local = self.local_sessions.write().await;
                        local.insert(user_id.to_string(), sessions.clone());
                    }
                    return Ok(sessions);
                }
                Err(e) => {
                    warn!(
                        "Failed to get sessions from Redis: {}. Using local cache only.",
                        e
                    );
                }
            }
        }

        // Return empty if not found
        Ok(Vec::new())
    }

    async fn get_user_sessions_redis(
        &self,
        user_id: &str,
    ) -> Result<Vec<String>, redis::RedisError> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_async_connection().await?;
            let key = format!("user_sessions:{}", user_id);
            let sessions: Vec<String> = conn.smembers(&key).await?;
            return Ok(sessions);
        }

        Ok(Vec::new())
    }

    /// Broadcast a message to all sessions of a user
    ///
    /// This is a helper method that returns the session IDs.
    /// Actual message sending is handled by the WebSocket manager.
    pub async fn get_user_session_ids(&self, user_id: &str) -> Vec<String> {
        self.get_user_sessions(user_id).await.unwrap_or_else(|e| {
            error!("Failed to get user sessions: {}", e);
            Vec::new()
        })
    }

    /// Check if Redis is available
    pub fn is_redis_available(&self) -> bool {
        self.redis_client.is_some()
    }

    /// Clean up old sessions (called periodically)
    pub async fn cleanup_old_sessions(&self) {
        // Redis handles TTL automatically, so we only need to clean local cache
        // In a production system, you might want to sync with Redis periodically
        info!("Session cleanup: Redis handles TTL automatically");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_session_management() {
        let manager = RedisSessionManager::new(None).await.unwrap();

        // Register sessions
        manager.register_session("user1", "session1").await.unwrap();
        manager.register_session("user1", "session2").await.unwrap();
        manager.register_session("user2", "session3").await.unwrap();

        // Get sessions
        let user1_sessions = manager.get_user_sessions("user1").await.unwrap();
        assert_eq!(user1_sessions.len(), 2);
        assert!(user1_sessions.contains(&"session1".to_string()));
        assert!(user1_sessions.contains(&"session2".to_string()));

        // Unregister session
        manager
            .unregister_session("user1", "session1")
            .await
            .unwrap();
        let user1_sessions = manager.get_user_sessions("user1").await.unwrap();
        assert_eq!(user1_sessions.len(), 1);
        assert_eq!(user1_sessions[0], "session2");
    }
}
