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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Central audit trail manager coordinating all audit operations
#[derive(Debug)]
pub struct AuditTrailManager {
    config: AuditConfig,
    file_auditor: Arc<FileOperationsAuditor>,
    terminal_auditor: Arc<TerminalAuditor>,
    council_auditor: Arc<CouncilAuditor>,
    agent_thinking_auditor: Arc<AgentThinkingAuditor>,
    performance_auditor: Arc<PerformanceAuditor>,
    error_recovery_auditor: Arc<ErrorRecoveryAuditor>,
    learning_auditor: Arc<LearningAuditor>,
    global_stats: Arc<RwLock<GlobalAuditStats>>,
}

/// Configuration for audit trail system
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditLogLevel {
    /// Minimal logging - only critical operations
    Minimal,
    /// Standard logging - key operations and decisions
    Standard,
    /// Detailed logging - comprehensive operation tracking
    Detailed,
    /// Debug logging - all operations including internal state
    Debug,
}

/// Output format for audit logs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditOutputFormat {
    /// JSON format for structured analysis
    Json,
    /// Human-readable structured text
    StructuredText,
    /// Binary format for efficient storage
    Binary,
    /// Multiple formats simultaneously
    MultiFormat,
}

/// Global audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAuditStats {
    /// Total audit events recorded
    pub total_events: u64,
    /// Events by category
    pub events_by_category: HashMap<AuditCategory, u64>,
    /// Start time of audit collection
    pub collection_start: DateTime<Utc>,
    /// Performance metrics
    pub performance_metrics: AuditPerformanceMetrics,
    /// Error counts
    pub error_counts: HashMap<String, u64>,
}

/// Performance metrics for audit system itself
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub event_id: Uuid,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for distributed tracing
    pub correlation_id: Option<String>,
    /// Parent event ID (for nested operations)
    pub parent_event_id: Option<Uuid>,
    /// Event category
    pub category: AuditCategory,
    /// Event severity
    pub severity: AuditSeverity,
    /// User/Agent identifier
    pub actor: String,
    /// Operation or action performed
    pub operation: String,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditCategory {
    FileOperation,
    TerminalCommand,
    CouncilDecision,
    AgentThinking,
    Performance,
    ErrorRecovery,
    Learning,
    SystemHealth,
}

/// Audit event severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Performance metrics for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            file_auditor: Arc::new(FileOperationsAuditor::new(config.clone(), global_stats.clone())),
            terminal_auditor: Arc::new(TerminalAuditor::new(config.clone(), global_stats.clone())),
            council_auditor: Arc::new(CouncilAuditor::new(config.clone(), global_stats.clone())),
            agent_thinking_auditor: Arc::new(AgentThinkingAuditor::new(config.clone(), global_stats.clone())),
            performance_auditor: Arc::new(PerformanceAuditor::new(config.clone(), global_stats.clone())),
            error_recovery_auditor: Arc::new(ErrorRecoveryAuditor::new(config.clone(), global_stats.clone())),
            learning_auditor: Arc::new(LearningAuditor::new(config.clone(), global_stats.clone())),
            global_stats,
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

    /// Export audit trail for analysis
    pub async fn export_audit_trail(&self, format: AuditOutputFormat, time_range: Option<(DateTime<Utc>, DateTime<Utc>)>) -> Result<String, AuditError> {
        // Implementation would collect and format audit events
        // This is a placeholder for the actual implementation
        Ok("Audit trail export not yet implemented".to_string())
    }

    /// Search audit events
    pub async fn search_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, AuditError> {
        // Implementation would search through audit logs
        // This is a placeholder for the actual implementation
        Ok(vec![])
    }

    /// Clean up old audit logs based on retention policy
    pub async fn cleanup_old_logs(&self) -> Result<u64, AuditError> {
        // Implementation would remove logs older than retention_days
        Ok(0)
    }
}

/// Audit query for searching events
#[derive(Debug, Clone)]
pub struct AuditQuery {
    pub category: Option<AuditCategory>,
    pub severity: Option<AuditSeverity>,
    pub actor: Option<String>,
    pub operation: Option<String>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub correlation_id: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
}

/// Audit error type
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Query error: {0}")]
    Query(String),
}

impl From<String> for AuditError {
    fn from(s: String) -> Self {
        AuditError::Config(s)
    }
}

mod auditors {
    use super::*;

    /// File operations auditor
    #[derive(Debug)]
    pub struct FileOperationsAuditor {
        config: AuditConfig,
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl FileOperationsAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self { config, global_stats }
        }

        pub async fn record_file_read(&self, file_path: &str, bytes_read: u64) -> Result<(), AuditError> {
            self.record_file_operation("read", file_path, bytes_read, None).await
        }

        pub async fn record_file_write(&self, file_path: &str, bytes_written: u64) -> Result<(), AuditError> {
            self.record_file_operation("write", file_path, bytes_written, None).await
        }

        pub async fn record_file_search(&self, pattern: &str, files_searched: usize, matches_found: usize, duration: Duration) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert("pattern".to_string(), serde_json::Value::String(pattern.to_string()));
            parameters.insert("files_searched".to_string(), serde_json::Value::Number(files_searched.into()));
            parameters.insert("matches_found".to_string(), serde_json::Value::Number(matches_found.into()));

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
            ).await
        }

        async fn record_file_operation(&self, operation: &str, file_path: &str, bytes: u64, duration: Option<Duration>) -> Result<(), AuditError> {
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
            ).await
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
            *stats.events_by_category.entry(event.category.clone()).or_insert(0) += 1;

            // In a real implementation, this would write to persistent storage
            // For now, just log to console
            if self.config.log_level != AuditLogLevel::Minimal {
                println!("📁 FILE AUDIT: {} {} {:?}", event.operation, event.target.as_deref().unwrap_or(""), event.result);
            }

            Ok(())
        }
    }

    /// Terminal commands auditor
    #[derive(Debug)]
    pub struct TerminalAuditor {
        config: AuditConfig,
        global_stats: Arc<RwLock<GlobalAuditStats>>,
        active_commands: Arc<RwLock<HashMap<String, CommandAudit>>>,
    }

    #[derive(Debug, Clone)]
    struct CommandAudit {
        command_id: String,
        command: String,
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

        pub async fn record_command_start(&self, command: &str, correlation_id: Option<String>) -> String {
            let command_id = Uuid::new_v4().to_string();
            let audit = CommandAudit {
                command_id: command_id.clone(),
                command: command.to_string(),
                start_time: Instant::now(),
                correlation_id: correlation_id.clone(),
            };

            self.active_commands.write().await.insert(command_id.clone(), audit);

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
                let result = if success {
                    AuditResult::Success { data: None }
                } else {
                    AuditResult::Failure {
                        error_message: stderr.unwrap_or_else(|| "Command failed".to_string()),
                        error_code: Some(exit_code.to_string()),
                        recoverable: exit_code != 130, // SIGINT is not recoverable
                    }
                };

                let mut parameters = HashMap::new();
                parameters.insert("exit_code".to_string(), serde_json::Value::Number(exit_code.into()));
                if let Some(stdout) = stdout {
                    parameters.insert("stdout_length".to_string(), serde_json::Value::Number(stdout.len().into()));
                }
                if let Some(stderr) = stderr {
                    parameters.insert("stderr_length".to_string(), serde_json::Value::Number(stderr.len().into()));
                }

                let event = AuditEvent {
                    event_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    correlation_id: audit.correlation_id,
                    parent_event_id: None,
                    category: AuditCategory::TerminalCommand,
                    severity: if success { AuditSeverity::Info } else { AuditSeverity::Warning },
                    actor: "agent".to_string(),
                    operation: "command_complete".to_string(),
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
                Err(AuditError::Config(format!("Command {} not found", command_id)))
            }
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats.events_by_category.entry(event.category.clone()).or_insert(0) += 1;

            if let AuditResult::Failure { .. } = &event.result {
                *stats.error_counts.entry("terminal_command".to_string()).or_insert(0) += 1;
            }

            if self.config.log_level != AuditLogLevel::Minimal {
                let status = match &event.result {
                    AuditResult::Success { .. } => "✅",
                    AuditResult::Failure { .. } => "❌",
                    AuditResult::InProgress => "⏳",
                    AuditResult::Cancelled => "🚫",
                };
                println!("💻 TERMINAL: {} {} ({}ms)",
                    status,
                    event.target.as_deref().unwrap_or(""),
                    event.performance.as_ref().map(|p| p.duration.as_millis()).unwrap_or(0)
                );
            }

            Ok(())
        }
    }

    /// Council decision auditor
    #[derive(Debug)]
    pub struct CouncilAuditor {
        config: AuditConfig,
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl CouncilAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self { config, global_stats }
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
            parameters.insert("session_id".to_string(), serde_json::Value::String(session_id.to_string()));
            parameters.insert("judge_id".to_string(), serde_json::Value::String(judge_id.to_string()));
            parameters.insert("decision".to_string(), serde_json::Value::String(decision.to_string()));
            parameters.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(confidence as f64).unwrap()));

            let mut context = HashMap::new();
            context.insert("reasoning".to_string(), serde_json::Value::String(reasoning.to_string()));

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: Some(session_id.to_string()),
                parent_event_id: None,
                category: AuditCategory::CouncilDecision,
                severity: AuditSeverity::Info,
                actor: judge_id.to_string(),
                operation: "vote".to_string(),
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
            parameters.insert("session_id".to_string(), serde_json::Value::String(session_id.to_string()));
            parameters.insert("final_decision".to_string(), serde_json::Value::String(final_decision.to_string()));
            parameters.insert("consensus_strength".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(consensus_strength as f64).unwrap()));

            let vote_dist_json = serde_json::to_value(&vote_distribution).unwrap_or(serde_json::Value::Null);
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
            *stats.events_by_category.entry(event.category.clone()).or_insert(0) += 1;

            if self.config.log_level != AuditLogLevel::Minimal {
                println!("🏛️  COUNCIL: {} {} - {}",
                    event.operation,
                    event.target.as_deref().unwrap_or(""),
                    event.parameters.get("decision").or_else(|| event.parameters.get("final_decision")).and_then(|v| v.as_str()).unwrap_or("")
                );
            }

            Ok(())
        }
    }

    /// Agent thinking auditor
    #[derive(Debug)]
    pub struct AgentThinkingAuditor {
        config: AuditConfig,
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl AgentThinkingAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self { config, global_stats }
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
            parameters.insert("step_name".to_string(), serde_json::Value::String(step_name.to_string()));
            parameters.insert("chosen_alternative".to_string(), serde_json::Value::String(chosen_alternative.to_string()));
            parameters.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(confidence as f64).unwrap()));
            parameters.insert("alternatives_count".to_string(), serde_json::Value::Number(alternatives_considered.len().into()));

            let mut context = HashMap::new();
            context.insert("reasoning".to_string(), serde_json::Value::String(reasoning.to_string()));
            context.insert("alternatives".to_string(), serde_json::to_value(&alternatives_considered).unwrap_or(serde_json::Value::Null));

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::AgentThinking,
                severity: AuditSeverity::Debug,
                actor: "agent".to_string(),
                operation: "reasoning_step".to_string(),
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
            parameters.insert("decision_type".to_string(), serde_json::Value::String(decision_type.to_string()));
            parameters.insert("chosen_option".to_string(), serde_json::Value::String(chosen_option.to_string()));
            parameters.insert("options_count".to_string(), serde_json::Value::Number(options.len().into()));
            if let Some(risk) = risk_assessment {
                parameters.insert("risk_assessment".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(risk as f64).unwrap()));
            }

            let mut context = HashMap::new();
            context.insert("reasoning".to_string(), serde_json::Value::String(reasoning.to_string()));
            context.insert("options".to_string(), serde_json::to_value(&options).unwrap_or(serde_json::Value::Null));

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::AgentThinking,
                severity: AuditSeverity::Info,
                actor: "agent".to_string(),
                operation: "decision_point".to_string(),
                target: Some(decision_type.to_string()),
                parameters,
                result: AuditResult::Success { data: None },
                performance: None,
                context,
                tags: vec!["thinking".to_string(), "decision".to_string()],
            };

            self.write_event(event).await
        }

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats.events_by_category.entry(event.category.clone()).or_insert(0) += 1;

            if self.config.log_level == AuditLogLevel::Detailed || self.config.log_level == AuditLogLevel::Debug {
                println!("🧠 THINKING: {} {} (confidence: {:.2})",
                    event.operation,
                    event.target.as_deref().unwrap_or(""),
                    event.parameters.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0)
                );
            }

            Ok(())
        }
    }

    /// Performance auditor
    #[derive(Debug)]
    pub struct PerformanceAuditor {
        config: AuditConfig,
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl PerformanceAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self { config, global_stats }
        }

        pub async fn record_operation_performance(
            &self,
            operation: &str,
            duration: Duration,
            success: bool,
            metadata: HashMap<String, serde_json::Value>,
        ) -> Result<(), AuditError> {
            let mut parameters = HashMap::new();
            parameters.insert("operation".to_string(), serde_json::Value::String(operation.to_string()));
            parameters.insert("duration_ms".to_string(), serde_json::Value::Number(duration.as_millis().into()));
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
                severity: if success { AuditSeverity::Info } else { AuditSeverity::Warning },
                actor: "system".to_string(),
                operation: "performance_metric".to_string(),
                target: Some(operation.to_string()),
                parameters,
                result: if success { AuditResult::Success { data: None } } else { AuditResult::Failure {
                    error_message: "Operation failed".to_string(),
                    error_code: None,
                    recoverable: true,
                }},
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
            *stats.events_by_category.entry(event.category.clone()).or_insert(0) += 1;

            if self.config.log_level != AuditLogLevel::Minimal {
                let duration_ms = event.parameters.get("duration_ms")
                    .and_then(|v| v.as_u64()).unwrap_or(0);
                let success = event.parameters.get("success")
                    .and_then(|v| v.as_bool()).unwrap_or(false);
                let status = if success { "✅" } else { "❌" };
                println!("⚡ PERFORMANCE: {} {} - {}ms", status, event.target.as_deref().unwrap_or(""), duration_ms);
            }

            Ok(())
        }
    }

    /// Error recovery auditor
    #[derive(Debug)]
    pub struct ErrorRecoveryAuditor {
        config: AuditConfig,
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl ErrorRecoveryAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self { config, global_stats }
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
            parameters.insert("error_type".to_string(), serde_json::Value::String(error_type.to_string()));
            parameters.insert("recovery_strategy".to_string(), serde_json::Value::String(recovery_strategy.to_string()));
            parameters.insert("success".to_string(), serde_json::Value::Bool(success));
            parameters.insert("duration_ms".to_string(), serde_json::Value::Number(duration.as_millis().into()));

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::ErrorRecovery,
                severity: if success { AuditSeverity::Info } else { AuditSeverity::Warning },
                actor: "recovery_system".to_string(),
                operation: "error_recovery".to_string(),
                target: Some(error_type.to_string()),
                parameters,
                result: if success { AuditResult::Success { data: None } } else { AuditResult::Failure {
                    error_message: "Recovery failed".to_string(),
                    error_code: None,
                    recoverable: false,
                }},
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

        async fn write_event(&self, event: AuditEvent) -> Result<(), AuditError> {
            let mut stats = self.global_stats.write().await;
            stats.total_events += 1;
            *stats.events_by_category.entry(event.category.clone()).or_insert(0) += 1;

            *stats.error_counts.entry("recovery_attempt".to_string()).or_insert(0) += 1;
            if event.parameters.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                *stats.error_counts.entry("recovery_success".to_string()).or_insert(0) += 1;
            }

            if self.config.log_level != AuditLogLevel::Minimal {
                let strategy = event.parameters.get("recovery_strategy").and_then(|v| v.as_str()).unwrap_or("");
                let success = event.parameters.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                let status = if success { "✅" } else { "❌" };
                println!("🔄 RECOVERY: {} {} - {}", status, event.target.as_deref().unwrap_or(""), strategy);
            }

            Ok(())
        }
    }

    /// Learning auditor
    #[derive(Debug)]
    pub struct LearningAuditor {
        config: AuditConfig,
        global_stats: Arc<RwLock<GlobalAuditStats>>,
    }

    impl LearningAuditor {
        pub fn new(config: AuditConfig, global_stats: Arc<RwLock<GlobalAuditStats>>) -> Self {
            Self { config, global_stats }
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
            parameters.insert("insight_type".to_string(), serde_json::Value::String(insight_type.to_string()));
            parameters.insert("impact".to_string(), serde_json::Value::String(impact.to_string()));
            parameters.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(confidence as f64).unwrap()));
            parameters.insert("source".to_string(), serde_json::Value::String(source.to_string()));

            let mut context = HashMap::new();
            context.insert("description".to_string(), serde_json::Value::String(description.to_string()));

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::Learning,
                severity: AuditSeverity::Info,
                actor: "learning_system".to_string(),
                operation: "insight_gained".to_string(),
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
            parameters.insert("optimization_type".to_string(), serde_json::Value::String(optimization_type.to_string()));
            parameters.insert("expected_improvement".to_string(), serde_json::Value::String(expected_improvement.to_string()));
            parameters.insert("risk_level".to_string(), serde_json::Value::String(risk_level.to_string()));

            let mut context = HashMap::new();
            context.insert("description".to_string(), serde_json::Value::String(description.to_string()));

            let event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                correlation_id: None,
                parent_event_id: None,
                category: AuditCategory::Learning,
                severity: AuditSeverity::Info,
                actor: "learning_system".to_string(),
                operation: "optimization_applied".to_string(),
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
            *stats.events_by_category.entry(event.category.clone()).or_insert(0) += 1;

            if self.config.log_level != AuditLogLevel::Minimal {
                println!("🎓 LEARNING: {} {} - {}",
                    event.operation,
                    event.target.as_deref().unwrap_or(""),
                    event.parameters.get("impact").or_else(|| event.parameters.get("expected_improvement")).and_then(|v| v.as_str()).unwrap_or("")
                );
            }

            Ok(())
        }
    }
}

// Re-export auditors for convenience
pub use auditors::*;
