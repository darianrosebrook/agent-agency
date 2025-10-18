//! Structured logging implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub component: String,
    pub operation: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub context: LogContext,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub enable_json: bool,
    pub enable_file_logging: bool,
    pub log_file_path: Option<String>,
    pub max_file_size_mb: usize,
    pub max_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "INFO".to_string(),
            format: LogFormat::Json,
            enable_json: true,
            enable_file_logging: false,
            log_file_path: None,
            max_file_size_mb: 100,
            max_files: 5,
        }
    }
}

#[derive(Debug)]
pub struct StructuredLogger {
    config: LoggingConfig,
}

impl StructuredLogger {
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter = EnvFilter::try_from_env("AGENT_LOG_LEVEL")
            .unwrap_or_else(|_| EnvFilter::new(&self.config.level));

        let subscriber = tracing_subscriber::registry().with(filter);

        match self.config.format {
            LogFormat::Json => {
                let json_layer = fmt::layer()
                    .json()
                    .with_current_span(false)
                    .with_span_list(false);
                subscriber.with(json_layer).init();
            }
            LogFormat::Pretty => {
                let pretty_layer = fmt::layer().pretty();
                subscriber.with(pretty_layer).init();
            }
            LogFormat::Compact => {
                let compact_layer = fmt::layer().compact();
                subscriber.with(compact_layer).init();
            }
        }

        info!(
            component = "observability",
            operation = "logger_init",
            level = %self.config.level,
            format = ?self.config.format,
            "Structured logging initialized"
        );

        Ok(())
    }

    pub fn create_context(
        component: &str,
        operation: &str,
        request_id: Option<String>,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> LogContext {
        LogContext {
            request_id,
            user_id,
            session_id,
            component: component.to_string(),
            operation: operation.to_string(),
            tags: HashMap::new(),
        }
    }

    pub fn log_info(context: &LogContext, message: &str, metadata: Option<HashMap<String, serde_json::Value>>) {
        let metadata = metadata.unwrap_or_default();
        info!(
            request_id = %context.request_id.as_deref().unwrap_or(""),
            user_id = %context.user_id.as_deref().unwrap_or(""),
            session_id = %context.session_id.as_deref().unwrap_or(""),
            component = %context.component,
            operation = %context.operation,
            tags = ?context.tags,
            metadata = ?metadata,
            "{message}"
        );
    }

    pub fn log_warn(context: &LogContext, message: &str, metadata: Option<HashMap<String, serde_json::Value>>) {
        let metadata = metadata.unwrap_or_default();
        warn!(
            request_id = %context.request_id.as_deref().unwrap_or(""),
            user_id = %context.user_id.as_deref().unwrap_or(""),
            session_id = %context.session_id.as_deref().unwrap_or(""),
            component = %context.component,
            operation = %context.operation,
            tags = ?context.tags,
            metadata = ?metadata,
            "{message}"
        );
    }

    pub fn log_error(context: &LogContext, message: &str, error: Option<&dyn std::error::Error>, metadata: Option<HashMap<String, serde_json::Value>>) {
        let metadata = metadata.unwrap_or_default();
        let mut error_msg = message.to_string();
        if let Some(err) = error {
            error_msg.push_str(&format!(": {}", err));
        }

        error!(
            request_id = %context.request_id.as_deref().unwrap_or(""),
            user_id = %context.user_id.as_deref().unwrap_or(""),
            session_id = %context.session_id.as_deref().unwrap_or(""),
            component = %context.component,
            operation = %context.operation,
            tags = ?context.tags,
            metadata = ?metadata,
            error = ?error.map(|e| e.to_string()),
            "{error_msg}"
        );
    }

    pub fn log_performance(
        context: &LogContext,
        operation: &str,
        duration_ms: u64,
        success: bool,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) {
        let mut metadata = metadata.unwrap_or_default();
        metadata.insert("duration_ms".to_string(), duration_ms.into());
        metadata.insert("success".to_string(), success.into());

        if success {
            info!(
                request_id = %context.request_id.as_deref().unwrap_or(""),
                user_id = %context.user_id.as_deref().unwrap_or(""),
                session_id = %context.session_id.as_deref().unwrap_or(""),
                component = %context.component,
                operation = %operation,
                duration_ms = duration_ms,
                success = success,
                tags = ?context.tags,
                metadata = ?metadata,
                "Operation completed successfully"
            );
        } else {
            warn!(
                request_id = %context.request_id.as_deref().unwrap_or(""),
                user_id = %context.user_id.as_deref().unwrap_or(""),
                session_id = %context.session_id.as_deref().unwrap_or(""),
                component = %context.component,
                operation = %operation,
                duration_ms = duration_ms,
                success = success,
                tags = ?context.tags,
                metadata = ?metadata,
                "Operation failed"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context_creation() {
        let context = StructuredLogger::create_context(
            "test_component",
            "test_operation",
            Some("req-123".to_string()),
            Some("user-456".to_string()),
            Some("session-789".to_string()),
        );

        assert_eq!(context.component, "test_component");
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.request_id, Some("req-123".to_string()));
        assert_eq!(context.user_id, Some("user-456".to_string()));
        assert_eq!(context.session_id, Some("session-789".to_string()));
    }
}
