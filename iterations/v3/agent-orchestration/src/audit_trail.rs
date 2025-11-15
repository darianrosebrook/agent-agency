//! Comprehensive Audit Trail System for Agent Agency V3
//!
//! This module provides enterprise-grade audit trail capabilities similar to Cursor/Claude Code,
//! enabling complete observability of agent operations, decisions, and performance.
//!
//! ## Features
//!
//! - **File Operations Audit**: Complete tracking of file reads, writes, searches
//! - **Terminal Commands Audit**: All commands executed with results and performance
//! - **Council Decision Audit**: Council votes, reasoning, and consensus processes
//! - **Agent Thinking Audit**: Reasoning steps, decision trees, alternatives considered
//! - **Performance Metrics**: Execution times, resource usage, success rates
//! - **Error Recovery Audit**: Error handling decisions and recovery actions
//! - **Learning Audit**: Agent learning and optimization improvements
//!
//! ## Usage
//!
//! ```rust
//! use agent_agency_orchestration::audit_trail::{AuditTrailManager, AuditConfig};
//!
//! let config = AuditConfig {
//!     enable_file_audit: true,
//!     enable_terminal_audit: true,
//!     enable_council_audit: true,
//!     enable_performance_audit: true,
//!     log_level: AuditLogLevel::Detailed,
//!     retention_days: 30,
//!     max_file_size_mb: 100,
//! };
//!
//! let audit_manager = AuditTrailManager::new(config);
//!
//! // Audit a file operation
//! audit_manager.file_auditor().record_file_read("src/main.rs", 1500).await;
//!
//! // Audit a terminal command
//! let cmd_audit = audit_manager.terminal_auditor()
//!     .record_command_start("cargo build", correlation_id).await;
//! // ... execute command ...
//! audit_manager.terminal_auditor()
//!     .record_command_complete(cmd_audit, exit_code, stdout, stderr, duration).await;
//! ```

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Central audit trail manager coordinating all audit operations

#[derive(Debug)]
pub struct AuditTrailManager {
    config: AuditConfig,
    db_pool: Option<PgPool>,
    file_auditor: Arc<FileOperationsAuditor>,
    terminal_auditor: Arc<TerminalAuditor>,
    council_auditor: Arc<CouncilAuditor>,
    agent_thinking_auditor: Arc<AgentThinkingAuditor>,
    performance_auditor: Arc<PerformanceAuditor>,
    error_recovery_auditor: Arc<ErrorRecoveryAuditor>,
    learning_auditor: Arc<LearningAuditor>,
    global_stats: Arc<RwLock<GlobalAuditStats>>,
    // Evaluation framework storage (feature-gated)
    #[cfg(feature = "evaluation")]
    decision_points: Arc<RwLock<Vec<crate::chain_of_thought::DecisionPoint>>>,
    #[cfg(feature = "evaluation")]
    coordination_events: Arc<RwLock<Vec<crate::chain_of_thought::CoordinationEvent>>>,
    // Indexes for O(log n) query performance (feature-gated)
    #[cfg(feature = "evaluation")]
    decision_points_by_plan_id: Arc<RwLock<BTreeMap<Uuid, Vec<usize>>>>,
    #[cfg(feature = "evaluation")]
    decision_points_by_timestamp: Arc<RwLock<BTreeMap<DateTime<Utc>, Vec<usize>>>>,
    #[cfg(feature = "evaluation")]
    coordination_events_by_timestamp: Arc<RwLock<BTreeMap<DateTime<Utc>, Vec<usize>>>>,
    #[cfg(feature = "evaluation")]
    coordination_events_by_plan_id: Arc<RwLock<BTreeMap<Uuid, Vec<usize>>>>,
}

/// Configuration for audit trail system
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct AuditConfig {
    /// Enable file operations auditing
    pub enable_file_audit: bool,
    /// Enable terminal commands auditing
    pub enable_terminal_audit: bool,
    /// Enable council decision auditing
    pub enable_council_audit: bool,
    /// Enable agent thinking auditing
    pub enable_thinking_audit: bool,
    /// Enable performance metrics auditing
    pub enable_performance_audit: bool,
    /// Enable error recovery auditing
    pub enable_error_recovery_audit: bool,
    /// Enable learning auditing
    pub enable_learning_audit: bool,
    /// Audit log verbosity level
    pub log_level: AuditLogLevel,
    /// Retention period in days
    pub retention_days: u32,
    /// Maximum log file size in MB
    pub max_file_size_mb: u32,
    /// Output format for audit logs
    pub output_format: AuditOutputFormat,
    /// Enable real-time streaming of audit events
    pub enable_streaming: bool,
}

/// Audit log verbosity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub enum AuditLogLevel {
    /// Minimal logging - only critical operations
    Minimal,
    /// Standard logging - key operations and decisions
    #[default]
    Standard,
    /// Detailed logging - comprehensive operation tracking
    Detailed,
    /// Debug logging - all operations including internal state
    Debug,
}

/// Output format for audit logs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum AuditOutputFormat {
    /// JSON format for structured analysis
    Json,
    /// Human-readable structured text
    StructuredText,
    /// Binary format for efficient storage
    Binary,
    /// Multiple formats simultaneously
    MultiFormat,
    /// CSV format for spreadsheet analysis
    Csv,
    /// Plain text format
    Text,
}

impl Default for AuditOutputFormat {
    fn default() -> Self {
        AuditOutputFormat::Json
    }
}

/// Global audit statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GlobalAuditStats {
    /// Total audit events recorded
    pub total_events: u64,
    /// Events by category
    pub events_by_category: HashMap<AuditCategory, u64>,
    /// Start time of audit collection
    #[schemars(with = "String")]
    pub collection_start: DateTime<Utc>,
    /// Performance metrics
    #[schemars(with = "String")]
    pub performance_metrics: AuditPerformanceMetrics,
    /// Error counts
    pub error_counts: HashMap<String, u64>,
}

impl Default for GlobalAuditStats {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_by_category: HashMap::new(),
            collection_start: Utc::now(),
            performance_metrics: AuditPerformanceMetrics::default(),
            error_counts: HashMap::new(),
        }
    }
}

/// Performance metrics for audit system itself
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct AuditPerformanceMetrics {
    /// Average time to record an audit event (microseconds)
    pub avg_record_time_us: u64,
    /// Peak memory usage (bytes)
    pub peak_memory_bytes: u64,
    /// Total audit log size (bytes)
    pub total_log_size_bytes: u64,
    /// Audit events per second
    pub events_per_second: f64,
}

/// Base audit event structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditEvent {
    /// Unique event ID
    #[schemars(with = "String")]
    pub event_id: Uuid,
    /// Event timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for distributed tracing
    pub correlation_id: Option<String>,
    /// Parent event ID (for nested operations)
    #[schemars(with = "String")]
    pub parent_event_id: Option<Uuid>,
    /// Event category
    #[schemars(with = "String")]
    pub category: AuditCategory,
    /// Event severity
    #[schemars(with = "String")]
    pub severity: AuditSeverity,
    /// User/Agent identifier
    pub actor: String,
    /// Operation or action performed
    pub operation: String,
    /// Human-readable message
    pub message: Option<String>,
    /// Operation identifier for tracing
    pub operation_id: Option<String>,
    /// Target of the operation (file path, command, etc.)
    pub target: Option<String>,
    /// Operation parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Operation result
    pub result: AuditResult,
    /// Performance metrics
    pub performance: Option<AuditPerformance>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// Tags for filtering and searching
    pub tags: Vec<String>,
}

/// Audit event categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum AuditCategory {
    FileOperation,
    TerminalCommand,
    CouncilDecision,
    AgentThinking,
    Operation,
    Performance,
    ErrorRecovery,
    Error,
    Waiver,
    Learning,
    SystemHealth,
}

/// Audit event severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Low,
    Medium,
    High,
    Error,
    Critical,
}

impl Default for AuditSeverity {
    fn default() -> Self {
        AuditSeverity::Info
    }
}

/// Audit operation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AuditResult {
    Success {
        data: Option<serde_json::Value>,
    },
    Failure {
        error_message: String,
        error_code: Option<String>,
        recoverable: bool,
    },
    InProgress,
    Cancelled,
    Partial {
        completed_steps: Vec<String>,
        remaining_steps: Vec<String>,
        progress_percentage: f32,
    },
}

/// Performance metrics for audit events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditPerformance {
    /// Duration of the operation
    pub duration: Duration,
    /// CPU time used (if available)
    pub cpu_time_us: Option<u64>,
    /// Memory used (if available)
    pub memory_bytes: Option<u64>,
    /// I/O operations performed
    pub io_operations: Option<u64>,
    /// Network bytes transferred (if applicable)
    pub network_bytes: Option<u64>,
}

impl AuditTrailManager {
    /// Create a new audit trail manager
    pub fn new(config: AuditConfig) -> Self {
        let global_stats = Arc::new(RwLock::new(GlobalAuditStats {
            total_events: 0,
            events_by_category: HashMap::new(),
            collection_start: Utc::now(),
            performance_metrics: AuditPerformanceMetrics {
                avg_record_time_us: 0,
                peak_memory_bytes: 0,
                total_log_size_bytes: 0,
                events_per_second: 0.0,
            },
            error_counts: HashMap::new(),
        }));

        Self {
            config: config.clone(),
            db_pool: None,
            file_auditor: Arc::new(FileOperationsAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            terminal_auditor: Arc::new(TerminalAuditor::new(config.clone(), global_stats.clone())),
            council_auditor: Arc::new(CouncilAuditor::new(config.clone(), global_stats.clone())),
            agent_thinking_auditor: Arc::new(AgentThinkingAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            performance_auditor: Arc::new(PerformanceAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            error_recovery_auditor: Arc::new(ErrorRecoveryAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            learning_auditor: Arc::new(LearningAuditor::new(config.clone(), global_stats.clone())),
            global_stats,
            #[cfg(feature = "evaluation")]
            decision_points: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "evaluation")]
            coordination_events: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "evaluation")]
            decision_points_by_plan_id: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "evaluation")]
            decision_points_by_timestamp: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "evaluation")]
            coordination_events_by_timestamp: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "evaluation")]
            coordination_events_by_plan_id: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Create a new audit trail manager with database persistence
    pub async fn with_db_pool(config: AuditConfig, db_url: Option<&str>) -> Self {
        let db_pool = if let Some(url) = db_url {
            Some(
                PgPoolOptions::new()
                    .max_connections(5)
                    .connect(url)
                    .await
                    .expect("Failed to connect to database for audit logging"),
            )
        } else {
            None
        };

        let global_stats = Arc::new(RwLock::new(GlobalAuditStats {
            total_events: 0,
            events_by_category: HashMap::new(),
            collection_start: Utc::now(),
            performance_metrics: AuditPerformanceMetrics {
                avg_record_time_us: 0,
                peak_memory_bytes: 0,
                total_log_size_bytes: 0,
                events_per_second: 0.0,
            },
            error_counts: HashMap::new(),
        }));

        Self {
            config: config.clone(),
            db_pool,
            file_auditor: Arc::new(FileOperationsAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            terminal_auditor: Arc::new(TerminalAuditor::new(config.clone(), global_stats.clone())),
            council_auditor: Arc::new(CouncilAuditor::new(config.clone(), global_stats.clone())),
            agent_thinking_auditor: Arc::new(AgentThinkingAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            performance_auditor: Arc::new(PerformanceAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            error_recovery_auditor: Arc::new(ErrorRecoveryAuditor::new(
                config.clone(),
                global_stats.clone(),
            )),
            learning_auditor: Arc::new(LearningAuditor::new(config.clone(), global_stats.clone())),
            global_stats,
            #[cfg(feature = "evaluation")]
            decision_points: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "evaluation")]
            coordination_events: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "evaluation")]
            decision_points_by_plan_id: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "evaluation")]
            decision_points_by_timestamp: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "evaluation")]
            coordination_events_by_timestamp: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(feature = "evaluation")]
            coordination_events_by_plan_id: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Get file operations auditor
    pub fn file_auditor(&self) -> Arc<FileOperationsAuditor> {
        self.file_auditor.clone()
    }

    /// Get terminal commands auditor
    pub fn terminal_auditor(&self) -> Arc<TerminalAuditor> {
        self.terminal_auditor.clone()
    }

    /// Get council decision auditor
    pub fn council_auditor(&self) -> Arc<CouncilAuditor> {
        self.council_auditor.clone()
    }

    /// Get agent thinking auditor
    pub fn agent_thinking_auditor(&self) -> Arc<AgentThinkingAuditor> {
        self.agent_thinking_auditor.clone()
    }

    /// Get performance auditor
    pub fn performance_auditor(&self) -> Arc<PerformanceAuditor> {
        self.performance_auditor.clone()
    }

    /// Get error recovery auditor
    pub fn error_recovery_auditor(&self) -> Arc<ErrorRecoveryAuditor> {
        self.error_recovery_auditor.clone()
    }

    /// Get learning auditor
    pub fn learning_auditor(&self) -> Arc<LearningAuditor> {
        self.learning_auditor.clone()
    }

    /// Get current global statistics
    pub async fn get_global_stats(&self) -> GlobalAuditStats {
        self.global_stats.read().await.clone()
    }

    /// Query decision points (evaluation framework)
    ///
    /// Performance: O(log n + k) where k is the number of results
    /// Uses BTreeMap indexes for efficient querying by plan_id and timestamp
    #[cfg(feature = "evaluation")]
    pub async fn query_decision_points(
        &self,
        plan_id: Option<Uuid>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<crate::chain_of_thought::DecisionPoint> {
        let decisions = self.decision_points.read().await;
        let mut candidate_indices = std::collections::HashSet::new();

        // Use plan_id index if provided (O(log n))
        if let Some(pid) = plan_id {
            if let Some(indices) = self.decision_points_by_plan_id.read().await.get(&pid) {
                candidate_indices.extend(indices.iter().copied());
            } else {
                // No decisions for this plan_id
                return Vec::new();
            }
        }

        // Use timestamp index for time window (O(log n + k))
        let timestamp_index = self.decision_points_by_timestamp.read().await;
        let time_range_indices: Vec<usize> = if since.is_some() || until.is_some() {
            let start_bound = since
                .map(|t| std::ops::Bound::Included(t))
                .unwrap_or(std::ops::Bound::Unbounded);
            let end_bound = until
                .map(|t| std::ops::Bound::Included(t))
                .unwrap_or(std::ops::Bound::Unbounded);

            timestamp_index
                .range((start_bound, end_bound))
                .flat_map(|(_, indices)| indices.iter().copied())
                .collect()
        } else {
            // No time filter - use all indices
            timestamp_index
                .values()
                .flat_map(|indices| indices.iter().copied())
                .collect()
        };

        // Intersect candidate sets if both filters provided
        let final_indices: Vec<usize> = if plan_id.is_some() && (since.is_some() || until.is_some())
        {
            time_range_indices
                .into_iter()
                .filter(|idx| candidate_indices.contains(idx))
                .collect()
        } else if plan_id.is_some() {
            candidate_indices.into_iter().collect()
        } else {
            time_range_indices
        };

        // Retrieve actual decision points
        let mut results: Vec<_> = final_indices
            .iter()
            .filter_map(|idx| decisions.get(*idx))
            .cloned()
            .collect();

        // Sort by timestamp (already mostly sorted due to timestamp index)
        results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Apply limit
        if let Some(limit_val) = limit {
            results.truncate(limit_val);
        }

        results
    }

    /// Query coordination events (evaluation framework)
    ///
    /// Performance: O(log n + k) where k is the number of results
    /// Uses BTreeMap timestamp index for efficient time-window queries
    #[cfg(feature = "evaluation")]
    pub async fn query_coordination_events(
        &self,
        plan_id: Option<Uuid>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<crate::chain_of_thought::CoordinationEvent> {
        let events = self.coordination_events.read().await;
        let timestamp_index = self.coordination_events_by_timestamp.read().await;

        // Use timestamp index for time window (O(log n + k))
        let start_bound = since
            .map(|t| std::ops::Bound::Included(t))
            .unwrap_or(std::ops::Bound::Unbounded);
        let end_bound = until
            .map(|t| std::ops::Bound::Included(t))
            .unwrap_or(std::ops::Bound::Unbounded);

        let mut indices: Vec<usize> = timestamp_index
            .range((start_bound, end_bound))
            .flat_map(|(_, indices)| indices.iter().copied())
            .collect();

        // Filter by plan_id if provided using plan_id index for O(log n) performance
        if let Some(pid) = plan_id {
            let plan_id_index = self.coordination_events_by_plan_id.read().await;
            if let Some(plan_indices) = plan_id_index.get(&pid) {
                // Use plan_id index to get event indices for this plan
                // Intersect with timestamp-filtered indices
                let plan_indices_set: HashSet<usize> = plan_indices.iter().copied().collect();
                indices.retain(|idx| plan_indices_set.contains(idx));
            } else {
                // No events found for this plan_id - return empty result
                indices.clear();
            }
        }

        // Retrieve actual events
        let mut results: Vec<_> = indices
            .iter()
            .filter_map(|idx| events.get(*idx))
            .cloned()
            .collect();

        // Sort by timestamp (already mostly sorted due to timestamp index)
        results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Apply limit
        if let Some(limit_val) = limit {
            results.truncate(limit_val);
        }

        results
    }

    /// Record coordination event (evaluation framework)
    #[cfg(feature = "evaluation")]
    pub async fn record_coordination_event(
        &self,
        event: crate::chain_of_thought::CoordinationEvent,
    ) -> Result<(), AuditError> {
        let mut events = self.coordination_events.write().await;
        let index = events.len();
        events.push(event.clone());

        // Update timestamp index for O(log n) queries
        self.coordination_events_by_timestamp
            .write()
            .await
            .entry(event.timestamp)
            .or_insert_with(Vec::new)
            .push(index);

        // Update plan_id index for O(log n) queries
        // Extract plan_id from event details if present
        if let Some(plan_id_value) = event.details.get("plan_id") {
            if let Some(plan_id_str) = plan_id_value.as_str() {
                if let Ok(plan_id) = Uuid::parse_str(plan_id_str) {
                    self.coordination_events_by_plan_id
                        .write()
                        .await
                        .entry(plan_id)
                        .or_insert_with(Vec::new)
                        .push(index);
                }
            } else if let Some(plan_id_json) = plan_id_value.as_object() {
                // Handle JSON object format if needed
                if let Some(plan_id_str) = plan_id_json.get("value").and_then(|v| v.as_str()) {
                    if let Ok(plan_id) = Uuid::parse_str(plan_id_str) {
                        self.coordination_events_by_plan_id
                            .write()
                            .await
                            .entry(plan_id)
                            .or_insert_with(Vec::new)
                            .push(index);
                    }
                }
            }
        }

        Ok(())
    }

    /// Export audit trail for analysis
    pub async fn export_audit_trail(
        &self,
        format: AuditOutputFormat,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Result<String, AuditError> {
        if let Some(ref pool) = self.db_pool {
            let mut query = "SELECT * FROM audit_events".to_string();
            let mut params: Vec<serde_json::Value> = Vec::new();

            if let Some((start, end)) = time_range {
                query.push_str(" WHERE timestamp >= $1 AND timestamp <= $2");
                params.push(serde_json::to_value(start)?);
                params.push(serde_json::to_value(end)?);
            }

            query.push_str(" ORDER BY timestamp ASC");

            let mut query_builder = sqlx::query_as::<_, AuditEventRow>(&query);
            for param in params {
                query_builder = query_builder.bind(param);
            }

            let rows = query_builder.fetch_all(pool).await.map_err(|e| {
                AuditError::StorageError(format!("Failed to export audit events: {}", e))
            })?;

            let events = rows
                .into_iter()
                .map(|row| row.into_audit_event())
                .collect::<Result<Vec<_>, _>>()?;

            // Format based on requested output format
            match format {
                AuditOutputFormat::Json => serde_json::to_string_pretty(&events).map_err(|e| {
                    AuditError::StorageError(format!("Failed to serialize audit events: {}", e))
                }),
                AuditOutputFormat::StructuredText => Ok(self.format_audit_events_as_text(&events)),
                AuditOutputFormat::Binary => Err(AuditError::StorageError(
                    "Binary format not yet implemented".to_string(),
                )),
                AuditOutputFormat::MultiFormat => Err(AuditError::StorageError(
                    "Multi-format output not yet implemented".to_string(),
                )),
                AuditOutputFormat::Csv => self.format_audit_events_as_csv(&events),
                AuditOutputFormat::Text => Ok(self.format_audit_events_as_text(&events)),
            }
        } else {
            Err(AuditError::StorageError(
                "Database not configured for audit export".to_string(),
            ))
        }
    }

    /// Format audit events as CSV
    fn format_audit_events_as_csv(&self, events: &[AuditEvent]) -> Result<String, AuditError> {
        let mut csv =
            String::from("timestamp,category,severity,actor,operation,target,result,tags\n");

        for event in events {
            let result_str = match &event.result {
                AuditResult::Success { .. } => "SUCCESS".to_string(),
                AuditResult::Failure { .. } => "FAILURE".to_string(),
                AuditResult::Partial { .. } => "PARTIAL".to_string(),
                AuditResult::InProgress => "IN_PROGRESS".to_string(),
                AuditResult::Cancelled => "CANCELLED".to_string(),
            };

            let tags_str = event.tags.join(";");

            csv.push_str(&format!(
                "{},{:?},{:?},{},{},{},{},{}\n",
                event.timestamp.to_rfc3339(),
                event.category,
                event.severity,
                event.actor,
                event.operation,
                event.target.as_deref().unwrap_or(""),
                result_str,
                tags_str
            ));
        }

        Ok(csv)
    }

    /// Format audit events as human-readable text
    fn format_audit_events_as_text(&self, events: &[AuditEvent]) -> String {
        let mut text = format!("Audit Trail Export - {} events\n", events.len());
        text.push_str("=".repeat(80).as_str());
        text.push('\n');

        for event in events {
            text.push_str(&format!(
                "[{}] {}: {} - {} ({:?})\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                event.actor,
                event.operation,
                event.target.as_deref().unwrap_or("N/A"),
                event.result
            ));

            if !event.tags.is_empty() {
                text.push_str(&format!("  Tags: {}\n", event.tags.join(", ")));
            }

            text.push('\n');
        }

        text
    }

    /// Search audit events
    pub async fn search_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, AuditError> {
        if let Some(ref pool) = self.db_pool {
            // Build dynamic query based on provided filters
            let mut sql = "SELECT * FROM audit_events WHERE 1=1".to_string();
            let mut params: Vec<serde_json::Value> = Vec::new();
            let mut param_count = 0;

            if let Some(category) = &query.category {
                param_count += 1;
                sql.push_str(&format!(" AND category @> ${}", param_count));
                params.push(serde_json::to_value(category)?);
            }

            if let Some(severity) = &query.severity {
                param_count += 1;
                sql.push_str(&format!(" AND severity @> ${}", param_count));
                params.push(serde_json::to_value(severity)?);
            }

            if let Some(actor) = &query.actor {
                param_count += 1;
                sql.push_str(&format!(" AND actor = ${}", param_count));
                params.push(serde_json::to_value(actor)?);
            }

            if let Some(operation) = &query.operation {
                param_count += 1;
                sql.push_str(&format!(" AND operation = ${}", param_count));
                params.push(serde_json::to_value(operation)?);
            }

            if let Some((start, end)) = &query.time_range {
                param_count += 1;
                sql.push_str(&format!(" AND timestamp >= ${}", param_count));
                params.push(serde_json::to_value(start)?);

                param_count += 1;
                sql.push_str(&format!(" AND timestamp <= ${}", param_count));
                params.push(serde_json::to_value(end)?);
            }

            sql.push_str(" ORDER BY timestamp DESC");

            if let Some(limit) = query.limit {
                sql.push_str(&format!(" LIMIT {}", limit));
            }

            // Execute query with dynamic parameters
            let mut query_builder = sqlx::query_as::<_, AuditEventRow>(&sql);

            for param in params {
                query_builder = query_builder.bind(param);
            }

            let rows = query_builder.fetch_all(pool).await.map_err(|e| {
                AuditError::StorageError(format!("Failed to search audit events: {}", e))
            })?;

            // Convert to AuditEvents
            let events = rows
                .into_iter()
                .map(|row| row.into_audit_event())
                .collect::<Result<Vec<_>, _>>()?;

            Ok(events)
        } else {
            Err(AuditError::StorageError(
                "Database not configured for audit searches".to_string(),
            ))
        }
    }

    /// Clean up old audit logs based on retention policy
    pub async fn cleanup_old_logs(&self) -> Result<u64, AuditError> {
        if let Some(ref pool) = self.db_pool {
            let cutoff_date =
                Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

            let result = sqlx::query("DELETE FROM audit_events WHERE timestamp < $1")
                .bind(cutoff_date)
                .execute(pool)
                .await
                .map_err(|e| {
                    AuditError::StorageError(format!("Failed to cleanup old audit logs: {}", e))
                })?;

            let deleted_count = result.rows_affected();

            info!(
                "Cleaned up {} audit events older than {} days",
                deleted_count, self.config.retention_days
            );

            Ok(deleted_count)
        } else {
            Ok(0) // No database configured, nothing to clean up
        }
    }

    /// Persist audit event to database
    #[allow(dead_code)] // Reserved for future use
    async fn persist_audit_event(
        &self,
        pool: &PgPool,
        event: &AuditEvent,
    ) -> Result<(), AuditError> {
        // Create audit_events table if it doesn't exist
        self.ensure_audit_table_exists(pool).await?;

        // Insert audit event
        sqlx::query(
            r#"
            INSERT INTO audit_events (
                id, timestamp, correlation_id, parent_event_id, category, severity, actor, operation, message, operation_id, target,
                parameters, result, performance, context, tags
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#
        )
        .bind(event.event_id)
        .bind(event.timestamp)
        .bind(&event.correlation_id)
        .bind(&event.parent_event_id)
        .bind(serde_json::to_value(&event.category)?)
        .bind(serde_json::to_value(&event.severity)?)
        .bind(&event.actor)
        .bind(&event.operation)
        .bind(&event.message)
        .bind(&event.operation_id)
        .bind(&event.target)
        .bind(serde_json::to_value(&event.parameters)?)
        .bind(serde_json::to_value(&event.result)?)
        .bind(event.performance.as_ref().map(|p| serde_json::to_value(p)).transpose()?)
        .bind(serde_json::to_value(&event.context)?)
        .bind(&event.tags)
        .execute(pool)
        .await
        .map_err(|e| AuditError::StorageError(format!("Failed to insert audit event: {}", e)))?;

        Ok(())
    }

    /// Ensure audit_events table exists
    #[allow(dead_code)] // Reserved for future use
    async fn ensure_audit_table_exists(&self, pool: &PgPool) -> Result<(), AuditError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_events (
                id UUID PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL,
                correlation_id TEXT,
                parent_event_id UUID,
                category JSONB NOT NULL,
                severity JSONB NOT NULL,
                actor TEXT NOT NULL,
                operation TEXT NOT NULL,
                target TEXT,
                parameters JSONB NOT NULL,
                result JSONB NOT NULL,
                performance JSONB,
                context JSONB NOT NULL,
                tags TEXT[] NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            -- Create indexes for efficient querying
            CREATE INDEX IF NOT EXISTS idx_audit_events_timestamp ON audit_events (timestamp);
            CREATE INDEX IF NOT EXISTS idx_audit_events_category ON audit_events USING GIN (category);
            CREATE INDEX IF NOT EXISTS idx_audit_events_actor ON audit_events (actor);
            CREATE INDEX IF NOT EXISTS idx_audit_events_operation ON audit_events (operation);
            CREATE INDEX IF NOT EXISTS idx_audit_events_tags ON audit_events USING GIN (tags);
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| AuditError::StorageError(format!("Failed to create audit table: {}", e)))?;

        Ok(())
    }

    /// Query audit events for deterministic replays
    pub async fn query_events_for_replay(
        &self,
        task_id: &str,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        if let Some(ref pool) = self.db_pool {
            let events = sqlx::query_as::<_, AuditEventRow>(
                "SELECT * FROM audit_events WHERE context->>'task_id' = $1 ORDER BY timestamp ASC",
            )
            .bind(task_id)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AuditError::StorageError(format!("Failed to query audit events: {}", e))
            })?;

            // Convert rows back to AuditEvent
            let audit_events = events
                .into_iter()
                .map(|row| row.into_audit_event())
                .collect::<Result<Vec<_>, _>>()?;

            Ok(audit_events)
        } else {
            Err(AuditError::StorageError(
                "Database not configured for audit queries".to_string(),
            ))
        }
    }

    /// Record task execution start
    ///
    /// Records when a task begins execution on a worker.
    pub async fn record_task_execution_start(
        &self,
        task_id: Uuid,
        execution_id: Uuid,
        worker_id: Option<Uuid>,
        correlation_id: Option<String>,
    ) -> Result<(), AuditError> {
        let execution_id_str = execution_id.to_string();
        let worker_id_str = worker_id
            .map(|w| w.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            correlation_id: correlation_id.clone(),
            parent_event_id: None,
            category: AuditCategory::Operation,
            severity: AuditSeverity::Info,
            actor: "orchestrator".to_string(),
            operation: "task_execution_start".to_string(),
            message: Some(format!(
                "Task {} execution started on worker {}",
                task_id, worker_id_str
            )),
            operation_id: Some(execution_id_str.clone()),
            target: Some(worker_id_str.clone()),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "execution_id".to_string(),
                    serde_json::Value::String(execution_id_str.clone()),
                );
                params.insert(
                    "task_id".to_string(),
                    serde_json::Value::String(task_id.to_string()),
                );
                if let Some(wid) = worker_id {
                    params.insert(
                        "worker_id".to_string(),
                        serde_json::Value::String(wid.to_string()),
                    );
                }
                params
            },
            result: AuditResult::InProgress,
            performance: None,
            context: {
                let mut ctx = HashMap::new();
                ctx.insert(
                    "execution_id".to_string(),
                    serde_json::Value::String(execution_id_str),
                );
                ctx.insert(
                    "task_id".to_string(),
                    serde_json::Value::String(task_id.to_string()),
                );
                if let Some(wid) = worker_id {
                    ctx.insert(
                        "worker_id".to_string(),
                        serde_json::Value::String(wid.to_string()),
                    );
                }
                ctx
            },
            tags: vec![
                "orchestration".to_string(),
                "execution".to_string(),
                "task_start".to_string(),
            ],
        };

        // Log the audit event using structured logging
        tracing::info!(
            audit_event = ?event,
            category = ?event.category,
            operation = %event.operation,
            task_id = %task_id,
            execution_id = %execution_id,
            worker_id = ?worker_id,
            "Task execution started"
        );

        Ok(())
    }

    /// Record task execution completion
    ///
    /// Records when a task completes execution (success or failure).
    pub async fn record_task_execution_completion(
        &self,
        result: &agent_agency_contracts::task_executor::TaskExecutionResult,
        correlation_id: Option<String>,
    ) -> Result<(), AuditError> {
        let execution_id_str = result.execution_id.to_string();
        let worker_id_str = result
            .worker_id
            .map(|w| w.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            correlation_id: correlation_id.clone(),
            parent_event_id: None,
            category: AuditCategory::Operation,
            severity: if result.success {
                AuditSeverity::Info
            } else {
                AuditSeverity::Warning
            },
            actor: "orchestrator".to_string(),
            operation: "task_execution_completion".to_string(),
            message: Some(format!(
                "Task {} execution {}",
                result.task_id,
                if result.success {
                    "completed successfully"
                } else {
                    "failed"
                }
            )),
            operation_id: Some(execution_id_str.clone()),
            target: Some(worker_id_str.clone()),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "execution_id".to_string(),
                    serde_json::Value::String(execution_id_str.clone()),
                );
                params.insert(
                    "task_id".to_string(),
                    serde_json::Value::String(result.task_id.to_string()),
                );
                params.insert(
                    "success".to_string(),
                    serde_json::Value::Bool(result.success),
                );
                params.insert(
                    "error_count".to_string(),
                    serde_json::Value::Number((result.errors.len() as u64).into()),
                );
                params.insert(
                    "duration_ms".to_string(),
                    serde_json::Value::Number((result.duration_ms as u64).into()),
                );
                if let Some(wid) = result.worker_id {
                    params.insert(
                        "worker_id".to_string(),
                        serde_json::Value::String(wid.to_string()),
                    );
                }
                params
            },
            result: if result.success {
                AuditResult::Success {
                    data: Some(serde_json::json!({
                        "output": result.output,
                        "duration_ms": result.duration_ms,
                    })),
                }
            } else {
                AuditResult::Failure {
                    error_message: result.errors.join("; "),
                    error_code: Some("task_execution_failed".to_string()),
                    recoverable: true,
                }
            },
            performance: Some(AuditPerformance {
                duration: std::time::Duration::from_millis(result.duration_ms),
                cpu_time_us: None,
                memory_bytes: None,
                io_operations: None,
                network_bytes: None,
            }),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert(
                    "execution_id".to_string(),
                    serde_json::Value::String(execution_id_str),
                );
                ctx.insert(
                    "task_id".to_string(),
                    serde_json::Value::String(result.task_id.to_string()),
                );
                ctx.insert(
                    "duration_ms".to_string(),
                    serde_json::Value::Number((result.duration_ms as u64).into()),
                );
                ctx.insert(
                    "success".to_string(),
                    serde_json::Value::Bool(result.success),
                );
                if let Some(wid) = result.worker_id {
                    ctx.insert(
                        "worker_id".to_string(),
                        serde_json::Value::String(wid.to_string()),
                    );
                }
                ctx
            },
            tags: {
                let mut tags = vec![
                    "orchestration".to_string(),
                    "execution".to_string(),
                    "task_completion".to_string(),
                ];
                if result.success {
                    tags.push("success".to_string());
                } else {
                    tags.push("failure".to_string());
                }
                tags
            },
        };

        // Log the audit event using structured logging
        tracing::info!(
            audit_event = ?event,
            category = ?event.category,
            operation = %event.operation,
            task_id = %result.task_id,
            execution_id = %result.execution_id,
            worker_id = ?result.worker_id,
            success = result.success,
            duration_ms = result.duration_ms,
            "Task execution completed"
        );

        Ok(())
    }

    /// Record execution result for audit trail
    /// Note: TaskExecutionResult (contract type) doesn't contain artifacts/working_spec/quality_report
    /// These should be stored/retrieved separately if needed
    pub async fn record_execution(
        &self,
        result: &agent_agency_contracts::task_executor::TaskExecutionResult,
    ) -> Result<(), AuditError> {
        let execution_id_str = result.execution_id.to_string();
        let worker_id_str = result
            .worker_id
            .map(|w| w.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let event = AuditEventRow {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            correlation_id: Some(execution_id_str.clone()),
            parent_event_id: None,
            category: serde_json::json!(AuditCategory::CouncilDecision),
            severity: serde_json::json!(AuditSeverity::Info),
            actor: "orchestrator".to_string(),
            operation: "task_execution".to_string(),
            message: Some(format!("Task {} executed", result.execution_id)),
            operation_id: Some(execution_id_str.clone()),
            target: Some(worker_id_str.clone()),
            parameters: serde_json::json!({
                "execution_id": result.execution_id,
                "task_id": result.task_id,
                "worker_id": result.worker_id,
                "success": result.success,
                "error_count": result.errors.len(),
            }),
            result: serde_json::json!(if result.success {
                "completed"
            } else {
                "failed"
            }),
            performance: None, // No timing info available
            context: serde_json::json!({
                "execution_id": result.execution_id,
                "task_id": result.task_id,
                "worker_id": result.worker_id,
                "duration_ms": result.duration_ms,
            }),
            tags: vec!["orchestration".to_string(), "execution".to_string()],
        };

        // Log the audit event using structured logging with proper audit context
        tracing::info!(
            audit_event = ?event,
            category = %event.category,
            severity = %event.severity,
            actor = %event.actor,
            operation = %event.operation,
            correlation_id = ?event.correlation_id,
            "Audit event recorded"
        );

        Ok(())
    }

    /// Record orchestration decision point
    pub async fn record_orchestration_decision(
        &self,
        decision: crate::chain_of_thought::DecisionPoint,
    ) -> Result<(), AuditError> {
        // Store decision point for evaluation framework
        #[cfg(feature = "evaluation")]
        {
            let mut decisions = self.decision_points.write().await;
            let index = decisions.len();
            decisions.push(decision.clone());

            // Update indexes for O(log n) queries
            if let Some(plan_id) = decision.context.plan_id {
                self.decision_points_by_plan_id
                    .write()
                    .await
                    .entry(plan_id)
                    .or_insert_with(Vec::new)
                    .push(index);
            }

            self.decision_points_by_timestamp
                .write()
                .await
                .entry(decision.timestamp)
                .or_insert_with(Vec::new)
                .push(index);
        }

        self.agent_thinking_auditor()
            .record_decision_point(
                format!("{:?}", decision.decision_type).as_str(),
                decision
                    .alternatives
                    .into_iter()
                    .map(|a| a.option)
                    .collect(),
                decision.chosen_option.as_str(),
                decision.reasoning.as_str(),
                Some(decision.confidence as f32),
            )
            .await
    }

    /// Record worker coordination event
    pub async fn record_worker_coordination(
        &self,
        trace: crate::chain_of_thought::CoordinationTrace,
    ) -> Result<(), AuditError> {
        // Record as performance metric since coordination involves resource utilization
        let mut metadata = HashMap::new();
        metadata.insert(
            "task_id".to_string(),
            serde_json::Value::String(trace.task_id.to_string()),
        );
        metadata.insert(
            "coordination_events".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                trace.coordination_events.len() as u64
            )),
        );
        metadata.insert(
            "worker_assignments".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                trace.worker_assignments.len() as u64
            )),
        );
        metadata.insert(
            "cpu_utilization".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(trace.resource_utilization.cpu_utilization).unwrap(),
            ),
        );

        self.performance_auditor()
            .record_operation_performance(
                "worker_coordination",
                Duration::from_millis(0), // Coordination overhead timing not tracked here
                true,
                metadata,
            )
            .await
    }

    /// Record council evaluation process
    pub async fn record_council_evaluation(
        &self,
        trace: crate::chain_of_thought::CouncilEvaluationTrace,
    ) -> Result<(), AuditError> {
        // Record council consensus result
        let vote_distribution = trace
            .individual_verdicts
            .iter()
            .map(|v| (v.verdict.clone(), 1))
            .collect();

        self.council_auditor()
            .record_council_consensus(
                trace.session_id.to_string().as_str(),
                trace.final_decision.as_str(),
                vote_distribution,
                trace.aggregation_process.final_consensus_score as f32,
                Duration::from_millis(1000), // Default duration - can be made configurable later
            )
            .await
    }

    /// Record chain-of-thought reasoning step
    pub async fn record_chain_of_thought(
        &self,
        _task_id: Uuid,
        phase: crate::chain_of_thought::ChainOfThoughtPhase,
        content: String,
        confidence: f64,
    ) -> Result<(), AuditError> {
        self.agent_thinking_auditor()
            .record_reasoning_step(
                format!("{:?}", phase).as_str(),
                content.as_str(),
                vec![], // No alternatives for basic reasoning steps
                "",     // No chosen alternative for basic reasoning
                confidence as f32,
                Duration::from_millis(100), // Default reasoning step duration
            )
            .await
    }

    /// Record heartbeat for progress monitoring
    pub async fn record_heartbeat(
        &self,
        task_id: Uuid,
        component: &str,
        progress: crate::chain_of_thought::ProgressIndicator,
        _estimated_remaining: Option<std::time::Duration>,
    ) -> Result<(), AuditError> {
        // Delegate to performance auditor for operation performance tracking
        let progress_desc = match &progress {
            crate::chain_of_thought::ProgressIndicator::Percentage(p) => {
                format!("{:.1}% complete", p)
            }
            crate::chain_of_thought::ProgressIndicator::Steps { current, total } => {
                format!("Step {}/{}", current, total)
            }
            crate::chain_of_thought::ProgressIndicator::Phase(phase) => format!("Phase: {}", phase),
            crate::chain_of_thought::ProgressIndicator::WaitingFor { resource, .. } => {
                format!("Waiting for: {}", resource)
            }
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "component".to_string(),
            serde_json::Value::String(component.to_string()),
        );
        metadata.insert(
            "progress".to_string(),
            serde_json::Value::String(progress_desc),
        );
        metadata.insert(
            "task_id".to_string(),
            serde_json::Value::String(task_id.to_string()),
        );

        self.performance_auditor()
            .record_operation_performance("heartbeat", Duration::from_millis(0), true, metadata)
            .await
    }

    /// Record timeout warning
    pub async fn record_timeout_warning(
        &self,
        _task_id: Uuid,
        component: &str,
        operation: &str,
        elapsed: std::time::Duration,
        timeout_threshold: std::time::Duration,
    ) -> Result<(), AuditError> {
        // Delegate to error recovery auditor since timeouts indicate potential issues
        let mut context = HashMap::new();
        context.insert(
            "component".to_string(),
            serde_json::Value::String(component.to_string()),
        );
        context.insert(
            "operation".to_string(),
            serde_json::Value::String(operation.to_string()),
        );
        context.insert(
            "elapsed_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(elapsed.as_millis() as u64)),
        );
        context.insert(
            "threshold_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                timeout_threshold.as_millis() as u64
            )),
        );

        self.error_recovery_auditor()
            .record_error_recovery_attempt(
                "timeout_warning",
                "monitoring",
                false, // Not a successful recovery yet
                Duration::from_millis(0),
                context,
            )
            .await
    }

    /// Record stuck operation detection
    pub async fn record_stuck_detection(
        &self,
        _task_id: Uuid,
        component: &str,
        stuck_state: crate::chain_of_thought::StuckState,
        last_activity: DateTime<Utc>,
    ) -> Result<(), AuditError> {
        // Delegate to error recovery auditor since stuck operations need recovery
        let stuck_description = match &stuck_state {
            crate::chain_of_thought::StuckState::NoProgress { duration_ms, .. } => {
                format!("No progress for {}ms", duration_ms)
            }
            crate::chain_of_thought::StuckState::WaitingForResource { resource, .. } => {
                format!("Waiting for resource: {}", resource)
            }
            crate::chain_of_thought::StuckState::DeadlockDetected { resources, .. } => {
                format!("Deadlock detected with {} resources", resources.len())
            }
            crate::chain_of_thought::StuckState::TimeoutImminent {
                elapsed_ms,
                threshold_ms,
            } => format!("Timeout imminent: {}/{}ms", elapsed_ms, threshold_ms),
        };

        let mut context = HashMap::new();
        context.insert(
            "component".to_string(),
            serde_json::Value::String(component.to_string()),
        );
        context.insert(
            "stuck_description".to_string(),
            serde_json::Value::String(stuck_description),
        );
        context.insert(
            "last_activity".to_string(),
            serde_json::Value::String(last_activity.to_rfc3339()),
        );
        context.insert(
            "stuck_state".to_string(),
            serde_json::to_value(&stuck_state).unwrap(),
        );

        self.error_recovery_auditor()
            .record_error_recovery_attempt(
                "stuck_operation",
                "detection",
                false, // Not recovered yet
                Utc::now()
                    .signed_duration_since(last_activity)
                    .to_std()
                    .unwrap_or(Duration::from_secs(0)),
                context,
            )
            .await
    }

    /// Record error propagation tracking
    pub async fn record_error_propagation(
        &self,
        _error_id: Uuid,
        source_component: &str,
        target_component: &str,
        error_chain: Vec<crate::chain_of_thought::ErrorLink>,
    ) -> Result<(), AuditError> {
        // Delegate to error recovery auditor for error tracking
        let mut context = HashMap::new();
        context.insert(
            "source_component".to_string(),
            serde_json::Value::String(source_component.to_string()),
        );
        context.insert(
            "target_component".to_string(),
            serde_json::Value::String(target_component.to_string()),
        );
        context.insert(
            "chain_length".to_string(),
            serde_json::Value::Number(serde_json::Number::from(error_chain.len() as u64)),
        );
        context.insert(
            "error_chain".to_string(),
            serde_json::to_value(&error_chain).unwrap(),
        );

        self.error_recovery_auditor()
            .record_error_recovery_attempt(
                "error_propagation",
                "tracking",
                false,                    // Error propagation is not a recovery success
                Duration::from_millis(0), // Duration not applicable for propagation tracking
                context,
            )
            .await
    }
}

/// Database row representation of audit event

#[derive(Debug, Serialize, Deserialize, JsonSchema, sqlx::FromRow)]
struct AuditEventRow {
    #[schemars(with = "String")]
    id: uuid::Uuid,
    #[schemars(with = "String")]
    timestamp: chrono::DateTime<chrono::Utc>,
    correlation_id: Option<String>,
    #[schemars(with = "Option<String>")]
    parent_event_id: Option<uuid::Uuid>,
    category: serde_json::Value,
    severity: serde_json::Value,
    actor: String,
    operation: String,
    message: Option<String>,
    operation_id: Option<String>,
    target: Option<String>,
    parameters: serde_json::Value,
    result: serde_json::Value,
    performance: Option<serde_json::Value>,
    context: serde_json::Value,
    tags: Vec<String>,
}

impl AuditEventRow {
    /// Convert database row back to AuditEvent
    fn into_audit_event(self) -> Result<AuditEvent, AuditError> {
        Ok(AuditEvent {
            event_id: self.id,
            timestamp: self.timestamp,
            correlation_id: self.correlation_id,
            parent_event_id: self.parent_event_id,
            category: serde_json::from_value(self.category).map_err(|e| {
                AuditError::StorageError(format!("Failed to deserialize category: {}", e))
            })?,
            severity: serde_json::from_value(self.severity).map_err(|e| {
                AuditError::StorageError(format!("Failed to deserialize severity: {}", e))
            })?,
            actor: self.actor,
            operation: self.operation,
            message: self.message,
            operation_id: self.operation_id,
            target: self.target,
            parameters: serde_json::from_value(self.parameters).map_err(|e| {
                AuditError::StorageError(format!("Failed to deserialize parameters: {}", e))
            })?,
            result: serde_json::from_value(self.result).map_err(|e| {
                AuditError::StorageError(format!("Failed to deserialize result: {}", e))
            })?,
            performance: self
                .performance
                .map(|p| serde_json::from_value(p))
                .transpose()
                .map_err(|e| {
                    AuditError::StorageError(format!("Failed to deserialize performance: {}", e))
                })?,
            context: serde_json::from_value(self.context).map_err(|e| {
                AuditError::StorageError(format!("Failed to deserialize context: {}", e))
            })?,
            tags: self.tags,
        })
    }
}

/// Audit query for searching events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct AuditQuery {
    pub category: Option<AuditCategory>,
    pub severity: Option<AuditSeverity>,
    pub actor: Option<String>,
    pub operation: Option<String>,
    #[schemars(skip)]
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub correlation_id: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
}

/// Audit error type

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
pub enum AuditError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Circuit breaker error: {0}")]
    CircuitBreaker(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

impl From<String> for AuditError {
    fn from(s: String) -> Self {
        AuditError::Config(s)
    }
}

impl From<std::io::Error> for AuditError {
    fn from(err: std::io::Error) -> Self {
        AuditError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AuditError {
    fn from(err: serde_json::Error) -> Self {
        AuditError::Serialization(err.to_string())
    }
}

mod auditors {
    use super::*;

    /// File operations auditor
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct FileOperationsAuditor {
        config: AuditConfig,
        #[serde(skip)]
        #[schemars(skip)]
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl FileOperationsAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self {
                config,
                global_stats,
            }
        }

        pub async fn record_file_read(
            &self,
            file_path: &str,
            bytes_read: u64,
        ) -> Result<(), AuditError> {
            self.record_file_operation("read", file_path, bytes_read, None)
                .await
        }

        pub async fn record_file_write(
            &self,
            file_path: &str,
            bytes_written: u64,
        ) -> Result<(), AuditError> {
            self.record_file_operation("write", file_path, bytes_written, None)
                .await
        }

        pub async fn record_file_search(
            &self,
            pattern: &str,
            files_searched: usize,
            matches_found: usize,
            duration: Duration,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "pattern".to_string(),
                serde_json::Value::String(pattern.to_string()),
            );
            parameters.insert(
                "files_searched".to_string(),
                serde_json::Value::Number(files_searched.into()),
            );
            parameters.insert(
                "matches_found".to_string(),
                serde_json::Value::Number(matches_found.into()),
            );

            self.record_operation(
                "search",
                Some(pattern),
                parameters,
                AuditResult::Success { data: None },
                Some(AuditPerformance {
                    duration,
                    cpu_time_us: None,
                    memory_bytes: None,
                    io_operations: Some(files_searched as u64),
                    network_bytes: None,
                }),
                vec!["file_operation".to_string()],
            )
            .await
        }

        async fn record_file_operation(
            &self,
            operation: &str,
            file_path: &str,
            bytes: u64,
            duration: Option<Duration>,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert("bytes".to_string(), serde_json::Value::Number(bytes.into()));

            self.record_operation(
                operation,
                Some(file_path),
                parameters,
                AuditResult::Success { data: None },
                duration.map(|d| AuditPerformance {
                    duration: d,
                    cpu_time_us: None,
                    memory_bytes: None,
                    io_operations: Some(1),
                    network_bytes: None,
                }),
                vec!["file_operation".to_string()],
            )
            .await
        }

        async fn record_operation(
            &self,
            operation: &str,
            target: Option<&str>,
            parameters: HashMap<String, serde_json::Value>,
            result: AuditResult,
            performance: Option<AuditPerformance>,
            tags: Vec<String>,
        ) -> Result<(), AuditError> {
            if !self.config.enable_file_audit {
                return Ok(());
            }

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None, // Would be set from context
                parent_event_id: None,
                category: AuditCategory::FileOperation,
                message: None,
                operation_id: None,
                severity: AuditSeverity::Info,
                actor: "agent".to_string(),
                operation: operation.to_string(),
                target: target.map(|s| s.to_string()),
                parameters,
                result,
                performance,
                context: HashMap::new(),
                tags,
            };

            self.write_event(event).await
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            // Update global stats
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            // File auditor doesn't persist to database - events are logged to console/files

            if self.config.log_level != AuditLogLevel::Minimal {
                println!(
                    " FILE AUDIT: {} {} {:?}",
                    event.operation,
                    event.target.as_deref().unwrap_or(""),
                    event.result
                );
            }

            Ok(())
        }
    }

    /// Terminal commands auditor
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct TerminalAuditor {
        config: AuditConfig,
        #[serde(skip)]
        #[schemars(skip)]
        global_stats: Arc<RwLock<GlobalAuditStats>>,
        #[serde(skip)]
        #[schemars(skip)]
        active_commands: Arc<RwLock<HashMap<String, CommandAudit>>>,
    }

    #[derive(Debug, Clone)]
    struct CommandAudit {
        command_id: String,
        command: String,
        #[allow(dead_code)] // Reserved for future use
        start_time: Instant,
        correlation_id: Option<String>,
    }

    impl TerminalAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self {
                config,
                global_stats,
                active_commands: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub async fn record_command_start(
            &self,
            command: &str,
            correlation_id: Option<String>,
        ) -> String {
            let command_id = Uuid::new_v4().to_string();
            let audit = CommandAudit {
                command_id: command_id.clone(),
                command: command.to_string(),
                start_time: Instant::now(),
                correlation_id: correlation_id.clone(),
            };

            self.active_commands
                .write()
                .await
                .insert(command_id.clone(), audit);

            // Record start event
            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id,
                parent_event_id: None,
                category: AuditCategory::TerminalCommand,
                severity: AuditSeverity::Info,
                actor: "agent".to_string(),
                operation: "command_start".to_string(),
                message: Some(format!("Terminal command started: {}", command)),
                operation_id: Some(command_id.to_string()),
                target: Some(command.to_string()),
                parameters: HashMap::new(),
                result: AuditResult::InProgress,
                performance: None,
                context: HashMap::new(),
                tags: vec!["terminal".to_string(), "command_start".to_string()],
            };

            let _ = self.write_event(event).await;

            command_id
        }

        pub async fn record_command_complete(
            &self,
            command_id: &str,
            exit_code: i32,
            stdout: Option<String>,
            stderr: Option<String>,
            duration: Duration,
        ) -> Result<(), AuditError> {
            let audit = {
                let mut commands = self.active_commands.write().await;
                commands.remove(command_id)
            };

            if let Some(audit) = audit {
                let success = exit_code == 0;
                let error_message = stderr
                    .clone()
                    .unwrap_or_else(|| "Command failed".to_string());
                let result = if success {
                    AuditResult::Success { data: None }
                } else {
                    AuditResult::Failure {
                        error_message,
                        error_code: Some(exit_code.to_string()),
                        recoverable: exit_code != 130, // SIGINT is not recoverable
                    }
                };

                let mut parameters = HashMap::new();
                parameters.insert(
                    "exit_code".to_string(),
                    serde_json::Value::Number(exit_code.into()),
                );
                if let Some(ref stdout) = stdout {
                    parameters.insert(
                        "stdout_length".to_string(),
                        serde_json::Value::Number(stdout.len().into()),
                    );
                }
                if let Some(ref stderr) = stderr {
                    parameters.insert(
                        "stderr_length".to_string(),
                        serde_json::Value::Number(stderr.len().into()),
                    );
                }

                let event = AuditEvent {
                    event_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    correlation_id: audit.correlation_id,
                    parent_event_id: None,
                    category: AuditCategory::TerminalCommand,
                    severity: if success {
                        AuditSeverity::Info
                    } else {
                        AuditSeverity::Warning
                    },
                    actor: "agent".to_string(),
                    operation: "command_complete".to_string(),
                    message: Some(format!(
                        "Terminal command completed: {} (success: {})",
                        audit.command, success
                    )),
                    operation_id: Some(audit.command_id),
                    target: Some(audit.command),
                    parameters,
                    result,
                    performance: Some(AuditPerformance {
                        duration,
                        cpu_time_us: None,
                        memory_bytes: None,
                        io_operations: None,
                        network_bytes: None,
                    }),
                    context: HashMap::new(),
                    tags: vec!["terminal".to_string(), "command_complete".to_string()],
                };

                self.write_event(event).await
            } else {
                Err(AuditError::Config(format!(
                    "Command {} not found",
                    command_id
                )))
            }
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            if let AuditResult::Failure { .. } = &event.result {
                *stats
                    .error_counts
                    .entry("terminal_command".to_string())
                    .or_insert(0) += 1;
            }

            if self.config.log_level != AuditLogLevel::Minimal {
                let status = match &event.result {
                    AuditResult::Success { .. } => "",
                    AuditResult::Failure { .. } => "",
                    AuditResult::Partial { .. } => "",
                    AuditResult::InProgress => "",
                    AuditResult::Cancelled => "",
                };
                println!(
                    " TERMINAL: {} {} ({}ms)",
                    status,
                    event.target.as_deref().unwrap_or(""),
                    event
                        .performance
                        .as_ref()
                        .map(|p| p.duration.as_millis())
                        .unwrap_or(0)
                );
            }

            Ok(())
        }
    }

    /// Council decision auditor
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct CouncilAuditor {
        config: AuditConfig,
        #[serde(skip)]
        #[schemars(skip)]
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl CouncilAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self {
                config,
                global_stats,
            }
        }

        pub async fn record_council_vote(
            &self,
            session_id: &str,
            judge_id: &str,
            decision: &str,
            reasoning: &str,
            confidence: f32,
            duration: Duration,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.to_string()),
            );
            parameters.insert(
                "judge_id".to_string(),
                serde_json::Value::String(judge_id.to_string()),
            );
            parameters.insert(
                "decision".to_string(),
                serde_json::Value::String(decision.to_string()),
            );
            parameters.insert(
                "confidence".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(confidence as f64).unwrap()),
            );

            let mut context = HashMap::new();
            context.insert(
                "reasoning".to_string(),
                serde_json::Value::String(reasoning.to_string()),
            );

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: Some(session_id.to_string()),
                parent_event_id: None,
                category: AuditCategory::CouncilDecision,
                severity: AuditSeverity::Info,
                actor: judge_id.to_string(),
                operation: "vote".to_string(),
                message: Some(format!(
                    "Judge {} voted on session {}",
                    judge_id, session_id
                )),
                operation_id: Some(session_id.to_string()),
                target: Some(session_id.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: Some(AuditPerformance {
                    duration,
                    cpu_time_us: None,
                    memory_bytes: None,
                    io_operations: None,
                    network_bytes: None,
                }),
                context,
                tags: vec!["council".to_string(), "vote".to_string()],
            };

            self.write_event(event).await
        }

        pub async fn record_council_consensus(
            &self,
            session_id: &str,
            final_decision: &str,
            vote_distribution: HashMap<String, usize>,
            consensus_strength: f32,
            duration: Duration,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.to_string()),
            );
            parameters.insert(
                "final_decision".to_string(),
                serde_json::Value::String(final_decision.to_string()),
            );
            parameters.insert(
                "consensus_strength".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(consensus_strength as f64).unwrap(),
                ),
            );

            let vote_dist_json =
                serde_json::to_value(&vote_distribution).unwrap_or(serde_json::Value::Null);
            parameters.insert("vote_distribution".to_string(), vote_dist_json);

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: Some(session_id.to_string()),
                parent_event_id: None,
                category: AuditCategory::CouncilDecision,
                severity: AuditSeverity::Info,
                actor: "council".to_string(),
                operation: "consensus".to_string(),
                message: Some(format!(
                    "Council reached consensus on session {} with decision: {}",
                    session_id, final_decision
                )),
                operation_id: Some(session_id.to_string()),
                target: Some(session_id.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: Some(AuditPerformance {
                    duration,
                    cpu_time_us: None,
                    memory_bytes: None,
                    io_operations: None,
                    network_bytes: None,
                }),
                context: HashMap::new(),
                tags: vec!["council".to_string(), "consensus".to_string()],
            };

            self.write_event(event).await
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            if self.config.log_level != AuditLogLevel::Minimal {
                println!(
                    "🏛️  COUNCIL: {} {} - {}",
                    event.operation,
                    event.target.as_deref().unwrap_or(""),
                    event
                        .parameters
                        .get("decision")
                        .or_else(|| event.parameters.get("final_decision"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                );
            }

            Ok(())
        }
    }

    /// Agent thinking auditor
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct AgentThinkingAuditor {
        config: AuditConfig,
        #[serde(skip)]
        #[schemars(skip)]
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl AgentThinkingAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self {
                config,
                global_stats,
            }
        }

        pub async fn record_reasoning_step(
            &self,
            step_name: &str,
            reasoning: &str,
            alternatives_considered: Vec<String>,
            chosen_alternative: &str,
            confidence: f32,
            duration: Duration,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "step_name".to_string(),
                serde_json::Value::String(step_name.to_string()),
            );
            parameters.insert(
                "chosen_alternative".to_string(),
                serde_json::Value::String(chosen_alternative.to_string()),
            );
            parameters.insert(
                "confidence".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(confidence as f64).unwrap()),
            );
            parameters.insert(
                "alternatives_count".to_string(),
                serde_json::Value::Number(alternatives_considered.len().into()),
            );

            let mut context = HashMap::new();
            context.insert(
                "reasoning".to_string(),
                serde_json::Value::String(reasoning.to_string()),
            );
            context.insert(
                "alternatives".to_string(),
                serde_json::to_value(&alternatives_considered).unwrap_or(serde_json::Value::Null),
            );

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::AgentThinking,
                severity: AuditSeverity::Debug,
                actor: "agent".to_string(),
                operation: "reasoning_step".to_string(),
                message: Some(format!("Agent completed reasoning step: {}", step_name)),
                operation_id: Some(step_name.to_string()),
                target: Some(step_name.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: Some(AuditPerformance {
                    duration,
                    cpu_time_us: None,
                    memory_bytes: None,
                    io_operations: None,
                    network_bytes: None,
                }),
                context,
                tags: vec!["thinking".to_string(), "reasoning".to_string()],
            };

            self.write_event(event).await
        }

        pub async fn record_decision_point(
            &self,
            decision_type: &str,
            options: Vec<String>,
            chosen_option: &str,
            reasoning: &str,
            risk_assessment: Option<f32>,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "decision_type".to_string(),
                serde_json::Value::String(decision_type.to_string()),
            );
            parameters.insert(
                "chosen_option".to_string(),
                serde_json::Value::String(chosen_option.to_string()),
            );
            parameters.insert(
                "options_count".to_string(),
                serde_json::Value::Number(options.len().into()),
            );
            if let Some(risk) = risk_assessment {
                parameters.insert(
                    "risk_assessment".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(risk as f64).unwrap()),
                );
            }

            let mut context = HashMap::new();
            context.insert(
                "reasoning".to_string(),
                serde_json::Value::String(reasoning.to_string()),
            );
            context.insert(
                "options".to_string(),
                serde_json::to_value(&options).unwrap_or(serde_json::Value::Null),
            );

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::AgentThinking,
                severity: AuditSeverity::Info,
                actor: "agent".to_string(),
                operation: "decision_point".to_string(),
                message: Some(format!("Agent made {} decision", decision_type)),
                operation_id: Some(Uuid::new_v4().to_string()),
                target: Some(decision_type.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: None,
                context,
                tags: vec!["thinking".to_string(), "decision".to_string()],
            };

            self.write_event(event).await
        }

        pub async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            if self.config.log_level == AuditLogLevel::Detailed
                || self.config.log_level == AuditLogLevel::Debug
            {
                println!(
                    " THINKING: {} {} (confidence: {:.2})",
                    event.operation,
                    event.target.as_deref().unwrap_or(""),
                    event
                        .parameters
                        .get("confidence")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                );
            }

            Ok(())
        }
    }

    /// Performance auditor
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct PerformanceAuditor {
        config: AuditConfig,
        #[serde(skip)]
        #[schemars(skip)]
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl PerformanceAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self {
                config,
                global_stats,
            }
        }

        pub async fn record_operation_performance(
            &self,
            operation: &str,
            duration: Duration,
            success: bool,
            metadata: HashMap<String, serde_json::Value>,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "operation".to_string(),
                serde_json::Value::String(operation.to_string()),
            );
            parameters.insert(
                "duration_ms".to_string(),
                serde_json::Value::Number((duration.as_millis() as u64).into()),
            );
            parameters.insert("success".to_string(), serde_json::Value::Bool(success));

            // Add metadata
            for (key, value) in metadata {
                parameters.insert(key, value);
            }

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::Performance,
                severity: if success {
                    AuditSeverity::Info
                } else {
                    AuditSeverity::Warning
                },
                actor: "system".to_string(),
                operation: "performance_metric".to_string(),
                message: Some(format!(
                    "Performance metric recorded for operation: {} (success: {})",
                    operation, success
                )),
                operation_id: Some(operation.to_string()),
                target: Some(operation.to_string()),
                parameters,
                result: if success {
                    AuditResult::Success { data: None }
                } else {
                    AuditResult::Failure {
                        error_message: "Operation failed".to_string(),
                        error_code: None,
                        recoverable: true,
                    }
                },
                performance: Some(AuditPerformance {
                    duration,
                    cpu_time_us: None,
                    memory_bytes: None,
                    io_operations: None,
                    network_bytes: None,
                }),
                context: HashMap::new(),
                tags: vec!["performance".to_string()],
            };

            self.write_event(event).await
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            if self.config.log_level != AuditLogLevel::Minimal {
                let duration_ms = event
                    .parameters
                    .get("duration_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let success = event
                    .parameters
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let status = if success { "" } else { "" };
                println!(
                    " PERFORMANCE: {} {} - {}ms",
                    status,
                    event.target.as_deref().unwrap_or(""),
                    duration_ms
                );
            }

            Ok(())
        }
    }

    /// Error recovery auditor
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct ErrorRecoveryAuditor {
        config: AuditConfig,
        #[serde(skip)]
        #[schemars(skip)]
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl ErrorRecoveryAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self {
                config,
                global_stats,
            }
        }

        pub async fn record_error_recovery_attempt(
            &self,
            error_type: &str,
            recovery_strategy: &str,
            success: bool,
            duration: Duration,
            context: HashMap<String, serde_json::Value>,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "error_type".to_string(),
                serde_json::Value::String(error_type.to_string()),
            );
            parameters.insert(
                "recovery_strategy".to_string(),
                serde_json::Value::String(recovery_strategy.to_string()),
            );
            parameters.insert("success".to_string(), serde_json::Value::Bool(success));
            parameters.insert(
                "duration_ms".to_string(),
                serde_json::Value::Number((duration.as_millis() as u64).into()),
            );

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::ErrorRecovery,
                severity: if success {
                    AuditSeverity::Info
                } else {
                    AuditSeverity::Warning
                },
                actor: "recovery_system".to_string(),
                operation: "error_recovery".to_string(),
                message: Some(format!(
                    "Error recovery {} for error type: {} using strategy: {}",
                    if success { "succeeded" } else { "failed" },
                    error_type,
                    recovery_strategy
                )),
                operation_id: Some(error_type.to_string()),
                target: Some(error_type.to_string()),
                parameters,
                result: if success {
                    AuditResult::Success { data: None }
                } else {
                    AuditResult::Failure {
                        error_message: "Recovery failed".to_string(),
                        error_code: None,
                        recoverable: false,
                    }
                },
                performance: Some(AuditPerformance {
                    duration,
                    cpu_time_us: None,
                    memory_bytes: None,
                    io_operations: None,
                    network_bytes: None,
                }),
                context,
                tags: vec!["error_recovery".to_string()],
            };

            self.write_event(event).await
        }

        /// Record correlation between recovery event and root failure
        pub async fn record_recovery_correlation(
            &self,
            operation_id: &str,
            failure_event_id: &str,
            recovery_success: bool,
            slo_impact: f64,
            context: HashMap<String, serde_json::Value>,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "operation_id".to_string(),
                serde_json::Value::String(operation_id.to_string()),
            );
            parameters.insert(
                "failure_event_id".to_string(),
                serde_json::Value::String(failure_event_id.to_string()),
            );
            parameters.insert(
                "recovery_success".to_string(),
                serde_json::Value::Bool(recovery_success),
            );
            parameters.insert(
                "slo_impact".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(slo_impact).unwrap()),
            );

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::ErrorRecovery,
                severity: if slo_impact > 0.5 {
                    AuditSeverity::High
                } else {
                    AuditSeverity::Medium
                },
                actor: "slo_monitor".to_string(),
                operation: "error_recovery_correlation".to_string(),
                message: Some(format!(
                    "Error recovery correlation recorded - success: {}",
                    recovery_success
                )),
                operation_id: Some(operation_id.to_string()),
                target: Some(operation_id.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: None,
                context,
                tags: vec!["slo".to_string(), "correlation".to_string()],
            };

            self.write_event(event).await
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            *stats
                .error_counts
                .entry("recovery_attempt".to_string())
                .or_insert(0) += 1;
            if event
                .parameters
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                *stats
                    .error_counts
                    .entry("recovery_success".to_string())
                    .or_insert(0) += 1;
            }

            if self.config.log_level != AuditLogLevel::Minimal {
                let strategy = event
                    .parameters
                    .get("recovery_strategy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let success = event
                    .parameters
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let status = if success { "" } else { "" };
                println!(
                    " RECOVERY: {} {} - {}",
                    status,
                    event.target.as_deref().unwrap_or(""),
                    strategy
                );
            }

            Ok(())
        }
    }

    /// Learning auditor
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct LearningAuditor {
        config: AuditConfig,
        #[serde(skip)]
        #[schemars(skip)]
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl LearningAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self {
                config,
                global_stats,
            }
        }

        pub async fn record_learning_insight(
            &self,
            insight_type: &str,
            description: &str,
            impact: &str,
            confidence: f32,
            source: &str,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "insight_type".to_string(),
                serde_json::Value::String(insight_type.to_string()),
            );
            parameters.insert(
                "impact".to_string(),
                serde_json::Value::String(impact.to_string()),
            );
            parameters.insert(
                "confidence".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(confidence as f64).unwrap()),
            );
            parameters.insert(
                "source".to_string(),
                serde_json::Value::String(source.to_string()),
            );

            let mut context = HashMap::new();
            context.insert(
                "description".to_string(),
                serde_json::Value::String(description.to_string()),
            );

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::Learning,
                severity: AuditSeverity::Info,
                actor: "learning_system".to_string(),
                operation: "insight_gained".to_string(),
                message: Some(format!(
                    "Learning system gained insight: {} with impact: {}",
                    insight_type, impact
                )),
                operation_id: Some(insight_type.to_string()),
                target: Some(insight_type.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: None,
                context,
                tags: vec!["learning".to_string(), "insight".to_string()],
            };

            self.write_event(event).await
        }

        pub async fn record_optimization_applied(
            &self,
            optimization_type: &str,
            description: &str,
            expected_improvement: &str,
            risk_level: &str,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "optimization_type".to_string(),
                serde_json::Value::String(optimization_type.to_string()),
            );
            parameters.insert(
                "expected_improvement".to_string(),
                serde_json::Value::String(expected_improvement.to_string()),
            );
            parameters.insert(
                "risk_level".to_string(),
                serde_json::Value::String(risk_level.to_string()),
            );

            let mut context = HashMap::new();
            context.insert(
                "description".to_string(),
                serde_json::Value::String(description.to_string()),
            );

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::Learning,
                severity: AuditSeverity::Info,
                actor: "learning_system".to_string(),
                operation: "optimization_applied".to_string(),
                message: Some(format!(
                    "Optimization applied: {} with expected improvement: {}",
                    optimization_type, expected_improvement
                )),
                operation_id: Some(optimization_type.to_string()),
                target: Some(optimization_type.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: None,
                context,
                tags: vec!["learning".to_string(), "optimization".to_string()],
            };

            self.write_event(event).await
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            if self.config.log_level != AuditLogLevel::Minimal {
                println!(
                    " LEARNING: {} {} - {}",
                    event.operation,
                    event.target.as_deref().unwrap_or(""),
                    event
                        .parameters
                        .get("impact")
                        .or_else(|| event.parameters.get("expected_improvement"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                );
            }

            Ok(())
        }
    }
}

// Re-export auditors for convenience
#[allow(ambiguous_glob_reexports)]
pub use auditors::*;
