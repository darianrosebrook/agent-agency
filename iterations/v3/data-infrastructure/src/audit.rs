/**
 * Audit Logging Module
 *
 * @author @darianrosebrook
 *
 * Comprehensive audit trail implementation for task operations and system events.
 * Provides compliance-ready logging with proper context and metadata.
 */

use std::net::IpAddr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use agent_agency_database::DatabaseClient;

/// Audit event types for different operations
#[derive(Debug, Clone)]
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
}

/// Audit severity levels
#[derive(Debug, Clone)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit context for logging operations
#[derive(Debug, Clone)]
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
        let user_agent_str = context.user_agent.as_ref().map(|s| s.as_str());

        self.db_client.execute(
            query,
            &[
                &task_id,
                &context.user_id,
                &action,
                &old_state,
                &new_state,
                &details_json,
                &context.ip_address,
                &user_agent_str,
                &context.session_id,
            ],
        ).await?;

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
        let user_agent_str = context.user_agent.as_ref().map(|s| s.as_str());

        self.db_client.execute(
            query,
            &[
                &event_type,
                &severity_str,
                &context.source,
                &context.user_id,
                &context.session_id,
                &resource_type,
                &resource_id,
                &action,
                &details_json,
                &context.ip_address,
                &user_agent_str,
                &success,
                &error_message,
                &processing_time_ms,
            ],
        ).await?;

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
        ).await
    }

    /// Get task audit trail
    pub async fn get_task_audit_trail(
        &self,
        task_id: &str,
        limit: Option<i64>,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let limit_val = limit.unwrap_or(100);
        let since_ts = since.map(|dt| dt.to_rfc3339());

        let query = r#"
            SELECT id, ts, user_id, action, old_state, new_state, details,
                   ip_address, user_agent, session_id
            FROM get_task_audit_trail($1, $2, $3)
        "#;

        let rows = self.db_client.query(
            query,
            &[&task_id, &limit_val, &since_ts],
        ).await?;

        let mut results = Vec::new();
        for row in rows {
            let audit_entry = serde_json::json!({
                "id": row.get::<_, Uuid>("id").to_string(),
                "timestamp": row.get::<_, String>("ts"),
                "user_id": row.get::<_, Option<String>>("user_id"),
                "action": row.get::<_, String>("action"),
                "old_state": row.get::<_, Option<String>>("old_state"),
                "new_state": row.get::<_, Option<String>>("new_state"),
                "details": row.get::<_, Value>("details"),
                "ip_address": row.get::<_, Option<String>>("ip_address"),
                "user_agent": row.get::<_, Option<String>>("user_agent"),
                "session_id": row.get::<_, Option<String>>("session_id"),
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
