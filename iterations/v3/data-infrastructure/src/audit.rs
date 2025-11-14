use crate::simple_client::DatabaseClient;
use chrono::{DateTime, Utc};
/**
 * Audit Logging Module
 *
 * @author @darianrosebrook
 *
 * Comprehensive audit trail implementation for task operations and system events.
 * Provides compliance-ready logging with proper context and metadata.
 */
use schemars::JsonSchema;
use serde_json::Value;
use sqlx::Row;
use std::net::IpAddr;

/// Audit event types for different operations
#[derive(Debug, Clone, JsonSchema)]
pub enum AuditEventType {
    TaskCreated,
    TaskUpdated,
    TaskPaused,
    TaskResumed,
    TaskCancelled,
    TaskCompleted,
    TaskFailed,
    ApiCall,
    AuthAttempt,
    SystemEvent,
    SecurityEvent,
    ConfigurationChange,
}

/// Audit severity levels
#[derive(Debug, Clone, JsonSchema)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit context for logging operations
#[derive(Debug, Clone, JsonSchema)]
pub struct AuditContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub source: String,
}

/// Audit logger for recording events
#[derive(Clone)]
pub struct AuditLogger {
    db_client: DatabaseClient,
}

impl AuditLogger {
    pub fn new(db_client: DatabaseClient) -> Self {
        Self { db_client }
    }

    /// Log a task audit event
    pub async fn log_task_event(
        &self,
        task_id: &str,
        action: &str,
        old_state: Option<&str>,
        new_state: Option<&str>,
        context: &AuditContext,
        details: Option<Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = r#"
            INSERT INTO task_audit_logs (
                task_id, user_id, action, old_state, new_state,
                details, ip_address, user_agent, session_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        let details_json = details.unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        let user_agent_str = context
            .user_agent
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");

        self.db_client
            .execute(
                query,
                &[
                    &task_id,
                    &context.user_id.as_ref().map(|s| s.as_str()).unwrap_or(""),
                    &action,
                    &old_state.unwrap_or(""),
                    &new_state.unwrap_or(""),
                    &details_json.to_string().as_str(),
                    &context
                        .ip_address
                        .as_ref()
                        .map(|ip| ip.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                        .as_str(),
                    &user_agent_str,
                    &context
                        .session_id
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(""),
                ],
            )
            .await?;

        Ok(())
    }

    /// Log a general audit event
    pub async fn log_audit_event(
        &self,
        event_type: &str,
        severity: AuditSeverity,
        context: &AuditContext,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
        action: Option<&str>,
        details: Option<Value>,
        success: bool,
        error_message: Option<&str>,
        processing_time_ms: Option<i32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = r#"
            INSERT INTO audit_events (
                event_type, severity, source, user_id, session_id,
                resource_type, resource_id, action, details,
                ip_address, user_agent, success, error_message, processing_time_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#;

        let severity_str = match severity {
            AuditSeverity::Debug => "debug",
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        };

        let details_json = details.unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        let user_agent_str = context
            .user_agent
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");

        self.db_client
            .execute(
                query,
                &[
                    &event_type,
                    &severity_str,
                    &context.source.as_str(),
                    &context.user_id.as_ref().map(|s| s.as_str()).unwrap_or(""),
                    &context
                        .session_id
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(""),
                    &resource_type.unwrap_or(""),
                    &resource_id.unwrap_or(""),
                    &action.unwrap_or(""),
                    &details_json.to_string().as_str(),
                    &context
                        .ip_address
                        .as_ref()
                        .map(|ip| ip.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                        .as_str(),
                    &user_agent_str,
                    &success.to_string().as_str(),
                    &error_message.unwrap_or(""),
                    &processing_time_ms.unwrap_or(0).to_string().as_str(),
                ],
            )
            .await?;

        Ok(())
    }

    /// Log an API call audit event
    pub async fn log_api_call(
        &self,
        method: &str,
        endpoint: &str,
        status_code: u16,
        processing_time_ms: i32,
        context: &AuditContext,
        request_body: Option<Value>,
        response_body: Option<Value>,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event_type = "api_call";
        let severity = if status_code >= 500 {
            AuditSeverity::Error
        } else if status_code >= 400 {
            AuditSeverity::Warning
        } else {
            AuditSeverity::Info
        };

        let details = serde_json::json!({
            "method": method,
            "endpoint": endpoint,
            "status_code": status_code,
            "request_body": request_body,
            "response_body": response_body,
        });

        self.log_audit_event(
            event_type,
            severity,
            context,
            Some("api"),
            Some(endpoint),
            Some(method),
            Some(details),
            success,
            error_message,
            Some(processing_time_ms),
        )
        .await
    }

    /// Get task audit trail
    pub async fn get_task_audit_trail(
        &self,
        _task_id: &str,
        limit: Option<i64>,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let _limit_val = limit.unwrap_or(100);
        let _since_ts = since.map(|dt| dt.to_rfc3339());

        let query = r#"
            SELECT id, ts, user_id, action, old_state, new_state, details,
                   ip_address, user_agent, session_id
            FROM get_task_audit_trail($1, $2, $3)
        "#;

        let rows = self.db_client.query(query, &[]).await?;

        let mut results = Vec::new();
        for row in rows {
            let audit_entry = serde_json::json!({
                "id": row.get::<String, &str>("id").to_string(),
                "timestamp": row.get::<chrono::DateTime<chrono::Utc>, &str>("ts"),
                "user_id": row.get::<Option<String>, &str>("user_id"),
                "action": row.get::<String, &str>("action"),
                "old_state": row.get::<Option<serde_json::Value>, &str>("old_state"),
                "new_state": row.get::<Option<serde_json::Value>, &str>("new_state"),
                "details": row.get::<Option<serde_json::Value>, &str>("details"),
                "ip_address": row.get::<Option<String>, &str>("ip_address"),
                "user_agent": row.get::<Option<String>, &str>("user_agent"),
                "session_id": row.get::<Option<String>, &str>("session_id"),
            });
            results.push(audit_entry);
        }

        Ok(results)
    }
}

/// Helper function to extract audit context from request headers
pub fn extract_audit_context(
    headers: &axum::http::HeaderMap,
    remote_addr: Option<std::net::SocketAddr>,
) -> AuditContext {
    let user_id = headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let session_id = headers
        .get("x-session-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let ip_address = remote_addr.map(|addr| addr.ip());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    AuditContext {
        user_id,
        session_id,
        ip_address,
        user_agent,
        source: "api-server".to_string(),
    }
}
