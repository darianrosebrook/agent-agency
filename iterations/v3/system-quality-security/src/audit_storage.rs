//! Enhanced Audit Storage - Database persistence for audit trails
//!
//! Provides comprehensive database storage, querying, and analytics capabilities
//! for audit events with proper indexing, retention, and performance optimization.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::audit::{AuditEventType, AuditRecord, AuditResult, AuditSeverity};

/// Enhanced audit storage with advanced querying and analytics
pub struct AuditStorage {
    pool: Arc<PgPool>,
}

impl AuditStorage {
    /// Create new audit storage instance
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .context("Failed to connect to database for audit storage")?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .context("Failed to test audit database connection")?;

        info!("Audit storage database connection established");
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Store an audit record
    pub async fn store_record(&self, record: &AuditRecord) -> Result<Uuid> {
        let event_type_str = match record.event_type {
            AuditEventType::Authentication => "authentication",
            AuditEventType::Authorization => "authorization",
            AuditEventType::DataAccess => "data_access",
            AuditEventType::Configuration => "configuration",
            AuditEventType::SystemIntegrity => "system_integrity",
            AuditEventType::PolicyViolation => "policy_violation",
            AuditEventType::Administrative => "administrative",
            AuditEventType::ResourceUsage => "resource_usage",
        };

        let severity_str = match record.severity {
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        };

        let result_str = match &record.result {
            AuditResult::Success => "success",
            AuditResult::Failure(_) => "failure",
            AuditResult::Denied => "denied",
            AuditResult::Timeout => "timeout",
        };

        let details_str = match &record.result {
            AuditResult::Failure(msg) => Some(msg.clone()),
            _ => None,
        };

        // Convert context to JSON
        let context_json =
            serde_json::to_string(&record.context).unwrap_or_else(|_| "{}".to_string());

        // Convert metadata to JSON
        let metadata_json =
            serde_json::to_string(&record.metadata).unwrap_or_else(|_| "{}".to_string());

        let stored_id = sqlx::query_scalar(
            r#"
            INSERT INTO audit_events (
                id, timestamp, event_type, severity, actor, resource, action, result,
                details, context, metadata, source_ip, user_agent, session_id, request_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            RETURNING id
            "#,
        )
        .bind(record.id)
        .bind(record.timestamp)
        .bind(event_type_str)
        .bind(severity_str)
        .bind(&record.actor)
        .bind(&record.resource)
        .bind(&record.action)
        .bind(result_str)
        .bind(details_str)
        .bind(context_json)
        .bind(metadata_json)
        .bind(&record.source_ip)
        .bind(&record.user_agent)
        .bind(&record.session_id)
        .bind(&record.request_id)
        .fetch_one(&*self.pool)
        .await
        .context("Failed to store audit record")?;

        debug!("Stored audit record: {} ({})", stored_id, event_type_str);
        Ok(stored_id)
    }

    /// Store multiple audit records in batch
    pub async fn store_batch(&self, records: &[AuditRecord]) -> Result<usize> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut tx = self.pool.begin().await?;
        let mut stored_count = 0;

        for record in records {
            match self.store_record_in_transaction(&mut tx, record).await {
                Ok(_) => stored_count += 1,
                Err(e) => {
                    warn!("Failed to store audit record {}: {}", record.id, e);
                    // Continue with other records
                }
            }
        }

        tx.commit().await.context("Failed to commit audit batch")?;

        info!("Stored {} audit records in batch", stored_count);
        Ok(stored_count)
    }

    /// Store single record within existing transaction
    pub async fn store_record_in_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        record: &AuditRecord,
    ) -> Result<Uuid> {
        let event_type_str = match record.event_type {
            AuditEventType::Authentication => "authentication",
            AuditEventType::Authorization => "authorization",
            AuditEventType::DataAccess => "data_access",
            AuditEventType::Configuration => "configuration",
            AuditEventType::SystemIntegrity => "system_integrity",
            AuditEventType::PolicyViolation => "policy_violation",
            AuditEventType::Administrative => "administrative",
            AuditEventType::ResourceUsage => "resource_usage",
        };

        let severity_str = match record.severity {
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        };

        let result_str = match &record.result {
            AuditResult::Success => "success",
            AuditResult::Failure(_) => "failure",
            AuditResult::Denied => "denied",
            AuditResult::Timeout => "timeout",
        };

        let details_str = match &record.result {
            AuditResult::Failure(msg) => Some(msg.clone()),
            _ => None,
        };

        let context_json =
            serde_json::to_string(&record.context).unwrap_or_else(|_| "{}".to_string());

        let metadata_json =
            serde_json::to_string(&record.metadata).unwrap_or_else(|_| "{}".to_string());

        let stored_id = sqlx::query_scalar(
            r#"
            INSERT INTO audit_events (
                id, timestamp, event_type, severity, actor, resource, action, result,
                details, context, metadata, source_ip, user_agent, session_id, request_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            RETURNING id
            "#,
        )
        .bind(record.id)
        .bind(record.timestamp)
        .bind(event_type_str)
        .bind(severity_str)
        .bind(&record.actor)
        .bind(&record.resource)
        .bind(&record.action)
        .bind(result_str)
        .bind(details_str)
        .bind(context_json)
        .bind(metadata_json)
        .bind(&record.source_ip)
        .bind(&record.user_agent)
        .bind(&record.session_id)
        .bind(&record.request_id)
        .fetch_one(&mut **tx)
        .await
        .context("Failed to store audit record in transaction")?;

        Ok(stored_id)
    }

    /// Query audit records with advanced filtering
    pub async fn query_records(
        &self,
        filters: AuditQueryFilters,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<AuditRecord>> {
        let mut query = String::from(
            r#"
            SELECT id, timestamp, event_type, severity, actor, resource, action, result,
                   details, context, metadata, source_ip, user_agent, session_id, request_id
            FROM audit_events
            WHERE 1=1
            "#,
        );

        let mut bind_values: Vec<serde_json::Value> = Vec::new();
        let mut param_count = 0;

        // Add filters
        if let Some(event_types) = &filters.event_types {
            if !event_types.is_empty() {
                let placeholders: Vec<String> = (0..event_types.len())
                    .map(|_| {
                        param_count += 1;
                        format!("${}", param_count)
                    })
                    .collect();
                query.push_str(&format!(
                    " AND event_type = ANY(ARRAY[{}])",
                    placeholders.join(",")
                ));

                for event_type in event_types {
                    let event_type_str = match event_type {
                        AuditEventType::Authentication => "authentication",
                        AuditEventType::Authorization => "authorization",
                        AuditEventType::DataAccess => "data_access",
                        AuditEventType::Configuration => "configuration",
                        AuditEventType::SystemIntegrity => "system_integrity",
                        AuditEventType::PolicyViolation => "policy_violation",
                        AuditEventType::Administrative => "administrative",
                        AuditEventType::ResourceUsage => "resource_usage",
                    };
                    bind_values.push(serde_json::Value::String(event_type_str.to_string()));
                }
            }
        }

        if let Some(severities) = &filters.severities {
            if !severities.is_empty() {
                let placeholders: Vec<String> = (0..severities.len())
                    .map(|_| {
                        param_count += 1;
                        format!("${}", param_count)
                    })
                    .collect();
                query.push_str(&format!(
                    " AND severity = ANY(ARRAY[{}])",
                    placeholders.join(",")
                ));

                for severity in severities {
                    let severity_str = match severity {
                        AuditSeverity::Info => "info",
                        AuditSeverity::Warning => "warning",
                        AuditSeverity::Error => "error",
                        AuditSeverity::Critical => "critical",
                    };
                    bind_values.push(serde_json::Value::String(severity_str.to_string()));
                }
            }
        }

        if let Some(actors) = &filters.actors {
            if !actors.is_empty() {
                let placeholders: Vec<String> = (0..actors.len())
                    .map(|_| {
                        param_count += 1;
                        format!("${}", param_count)
                    })
                    .collect();
                query.push_str(&format!(
                    " AND actor = ANY(ARRAY[{}])",
                    placeholders.join(",")
                ));

                for actor in actors {
                    bind_values.push(serde_json::Value::String(actor.clone()));
                }
            }
        }

        if let Some(resources) = &filters.resources {
            if !resources.is_empty() {
                let placeholders: Vec<String> = (0..resources.len())
                    .map(|_| {
                        param_count += 1;
                        format!("${}", param_count)
                    })
                    .collect();
                query.push_str(&format!(
                    " AND resource = ANY(ARRAY[{}])",
                    placeholders.join(",")
                ));

                for resource in resources {
                    bind_values.push(serde_json::Value::String(resource.clone()));
                }
            }
        }

        if let Some(results) = &filters.results {
            if !results.is_empty() {
                let placeholders: Vec<String> = (0..results.len())
                    .map(|_| {
                        param_count += 1;
                        format!("${}", param_count)
                    })
                    .collect();
                query.push_str(&format!(
                    " AND result = ANY(ARRAY[{}])",
                    placeholders.join(",")
                ));

                for result in results {
                    let result_str = match result {
                        AuditResult::Success => "success",
                        AuditResult::Failure(_) => "failure",
                        AuditResult::Denied => "denied",
                        AuditResult::Timeout => "timeout",
                    };
                    bind_values.push(serde_json::Value::String(result_str.to_string()));
                }
            }
        }

        if let Some(start_time) = filters.start_time {
            param_count += 1;
            query.push_str(&format!(" AND timestamp >= ${}", param_count));
            bind_values.push(serde_json::Value::String(start_time.to_rfc3339()));
        }

        if let Some(end_time) = filters.end_time {
            param_count += 1;
            query.push_str(&format!(" AND timestamp <= ${}", param_count));
            bind_values.push(serde_json::Value::String(end_time.to_rfc3339()));
        }

        if let Some(session_id) = filters.session_id {
            param_count += 1;
            query.push_str(&format!(" AND session_id = ${}", param_count));
            bind_values.push(serde_json::Value::String(session_id.to_string()));
        }

        if let Some(request_id) = filters.request_id {
            param_count += 1;
            query.push_str(&format!(" AND request_id = ${}", param_count));
            bind_values.push(serde_json::Value::String(request_id.to_string()));
        }

        // Add ordering and limits
        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = limit {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
            bind_values.push(serde_json::Value::Number(serde_json::Number::from(limit)));
        }

        if let Some(offset) = offset {
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
            bind_values.push(serde_json::Value::Number(serde_json::Number::from(offset)));
        }

        // Execute query - bind values individually
        let mut sql_query = sqlx::query(&query);
        for value in &bind_values {
            match value {
                serde_json::Value::String(s) => {
                    sql_query = sql_query.bind(s.clone());
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        sql_query = sql_query.bind(i);
                    } else if let Some(f) = n.as_f64() {
                        sql_query = sql_query.bind(f);
                    }
                }
                // TODO: Support additional JSON value types in SQL query binding:
                // 1. Boolean binding: Bind boolean values to SQL boolean columns
                //    - Handle serde_json::Value::Bool(true/false)
                //    - Map to appropriate SQL boolean type
                // 2. Null binding: Handle null/None values properly
                //    - Bind serde_json::Value::Null to SQL NULL
                //    - Handle Option<T> types appropriately
                // 3. Array/Object binding: Support complex types
                //    - Serialize arrays/objects to JSON strings for JSONB columns
                //    - Handle nested structures appropriately
                // ACCEPTANCE CRITERIA:
                // - Boolean values bind correctly to boolean SQL columns
                // - Null values bind correctly and don't cause errors
                // - Arrays and objects serialize properly for JSONB columns
                // - All JSON value types are handled without panics or errors
                // DEPENDENCIES:
                // - SQL query builder with type support (Required)
                // - JSON serialization for complex types (Required)
                // PRIORITY: Medium
                _ => {}
            }
        }
        let rows = sql_query
            .fetch_all(&*self.pool)
            .await
            .context("Failed to query audit records")?;

        let mut records = Vec::new();
        for row in rows {
            let record = self.row_to_audit_record(row)?;
            records.push(record);
        }

        info!("Queried {} audit records", records.len());
        Ok(records)
    }

    /// Get audit statistics
    pub async fn get_statistics(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<AuditStatistics> {
        let stats = sqlx::query_as::<_, AuditStatisticsRow>(
            r#"
            SELECT
                COUNT(*) as total_events,
                COUNT(*) FILTER (WHERE event_type = 'authentication') as authentication_events,
                COUNT(*) FILTER (WHERE event_type = 'authorization') as authorization_events,
                COUNT(*) FILTER (WHERE event_type = 'data_access') as data_access_events,
                COUNT(*) FILTER (WHERE event_type = 'policy_violation') as policy_violations,
                COUNT(*) FILTER (WHERE severity = 'critical') as critical_events,
                COUNT(*) FILTER (WHERE severity = 'error') as error_events,
                COUNT(*) FILTER (WHERE result = 'failure') as failed_operations,
                COUNT(DISTINCT actor) as unique_actors,
                MIN(timestamp) as oldest_event,
                MAX(timestamp) as newest_event
            FROM audit_events
            WHERE ($1::timestamptz IS NULL OR timestamp >= $1)
              AND ($2::timestamptz IS NULL OR timestamp <= $2)
            "#,
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&*self.pool)
        .await
        .context("Failed to get audit statistics")?;

        Ok(stats.into())
    }

    /// Cleanup old audit records based on retention policy
    pub async fn cleanup_old_records(&self, retention_days: i32) -> Result<i64> {
        let deleted_count: i64 = sqlx::query_scalar("SELECT cleanup_old_audit_events($1)")
            .bind(retention_days)
            .fetch_one(&*self.pool)
            .await
            .context("Failed to cleanup old audit records")?;

        info!(
            "Cleaned up {} old audit records (retention: {} days)",
            deleted_count, retention_days
        );
        Ok(deleted_count)
    }

    /// Get recent audit activity
    pub async fn get_recent_activity(&self, limit: Option<i64>) -> Result<Vec<AuditRecord>> {
        let limit_val = limit.unwrap_or(100);
        let rows = sqlx::query(
            r#"
            SELECT id, timestamp, event_type, severity, actor, resource, action, result,
                   details, context, metadata, source_ip, user_agent, session_id, request_id
            FROM recent_audit_activity
            LIMIT $1
            "#,
        )
        .bind(limit_val)
        .fetch_all(&*self.pool)
        .await
        .context("Failed to get recent audit activity")?;

        let mut records = Vec::new();
        for row in rows {
            let record = self.row_to_audit_record(row)?;
            records.push(record);
        }

        Ok(records)
    }

    /// Health check for audit storage
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").execute(&*self.pool).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Convert database row to AuditRecord
    fn row_to_audit_record(&self, row: sqlx::postgres::PgRow) -> Result<AuditRecord> {
        let id: Uuid = row.get("id");
        let timestamp: DateTime<Utc> = row.get("timestamp");
        let event_type_str: String = row.get("event_type");
        let severity_str: String = row.get("severity");
        let actor: String = row.get("actor");
        let resource: String = row.get("resource");
        let action: String = row.get("action");
        let result_str: String = row.get("result");
        let details: Option<String> = row.get("details");
        let context_json: serde_json::Value = row.get("context");
        let metadata_json: serde_json::Value = row.get("metadata");
        let source_ip: Option<String> = row.get("source_ip");
        let user_agent: Option<String> = row.get("user_agent");
        let session_id: Option<String> = row
            .get::<Option<Uuid>, _>("session_id")
            .map(|uuid| uuid.to_string());
        let request_id: Option<String> = row
            .get::<Option<Uuid>, _>("request_id")
            .map(|uuid| uuid.to_string());

        // Parse event type
        let event_type = match event_type_str.as_str() {
            "authentication" => AuditEventType::Authentication,
            "authorization" => AuditEventType::Authorization,
            "data_access" => AuditEventType::DataAccess,
            "configuration" => AuditEventType::Configuration,
            "system_integrity" => AuditEventType::SystemIntegrity,
            "policy_violation" => AuditEventType::PolicyViolation,
            "administrative" => AuditEventType::Administrative,
            "resource_usage" => AuditEventType::ResourceUsage,
            _ => AuditEventType::Administrative, // Default fallback
        };

        // Parse severity
        let severity = match severity_str.as_str() {
            "info" => AuditSeverity::Info,
            "warning" => AuditSeverity::Warning,
            "error" => AuditSeverity::Error,
            "critical" => AuditSeverity::Critical,
            _ => AuditSeverity::Info, // Default fallback
        };

        // Parse result
        let result = match result_str.as_str() {
            "success" => AuditResult::Success,
            "failure" => {
                AuditResult::Failure(details.unwrap_or_else(|| "Unknown error".to_string()))
            }
            "denied" => AuditResult::Denied,
            "timeout" => AuditResult::Timeout,
            _ => AuditResult::Success, // Default fallback
        };

        // Parse context and metadata
        let context: HashMap<String, serde_json::Value> =
            serde_json::from_value(context_json).unwrap_or_default();
        let metadata: HashMap<String, String> =
            serde_json::from_value(metadata_json).unwrap_or_default();

        Ok(AuditRecord {
            id,
            timestamp,
            event_type,
            severity,
            actor,
            resource,
            action,
            result,
            context,
            source_ip,
            user_agent,
            session_id,
            request_id,
            metadata,
        })
    }

    /// Get database pool for direct access
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Query filters for audit records
#[derive(Debug, Clone, Default)]
pub struct AuditQueryFilters {
    pub event_types: Option<Vec<AuditEventType>>,
    pub severities: Option<Vec<AuditSeverity>>,
    pub actors: Option<Vec<String>>,
    pub resources: Option<Vec<String>>,
    pub results: Option<Vec<AuditResult>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub session_id: Option<Uuid>,
    pub request_id: Option<Uuid>,
}

impl AuditQueryFilters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_event_types(mut self, event_types: Vec<AuditEventType>) -> Self {
        self.event_types = Some(event_types);
        self
    }

    pub fn with_severities(mut self, severities: Vec<AuditSeverity>) -> Self {
        self.severities = Some(severities);
        self
    }

    pub fn with_actors(mut self, actors: Vec<String>) -> Self {
        self.actors = Some(actors);
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }
}

/// Audit statistics
#[derive(Debug, Clone)]
pub struct AuditStatistics {
    pub total_events: i64,
    pub authentication_events: i64,
    pub authorization_events: i64,
    pub data_access_events: i64,
    pub policy_violations: i64,
    pub critical_events: i64,
    pub error_events: i64,
    pub failed_operations: i64,
    pub unique_actors: i64,
    pub oldest_event: Option<DateTime<Utc>>,
    pub newest_event: Option<DateTime<Utc>>,
}

impl From<AuditStatisticsRow> for AuditStatistics {
    fn from(row: AuditStatisticsRow) -> Self {
        AuditStatistics {
            total_events: row.total_events,
            authentication_events: row.authentication_events,
            authorization_events: row.authorization_events,
            data_access_events: row.data_access_events,
            policy_violations: row.policy_violations,
            critical_events: row.critical_events,
            error_events: row.error_events,
            failed_operations: row.failed_operations,
            unique_actors: row.unique_actors,
            oldest_event: row.oldest_event,
            newest_event: row.newest_event,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AuditStatisticsRow {
    total_events: i64,
    authentication_events: i64,
    authorization_events: i64,
    data_access_events: i64,
    policy_violations: i64,
    critical_events: i64,
    error_events: i64,
    failed_operations: i64,
    unique_actors: i64,
    oldest_event: Option<DateTime<Utc>>,
    newest_event: Option<DateTime<Utc>>,
}
