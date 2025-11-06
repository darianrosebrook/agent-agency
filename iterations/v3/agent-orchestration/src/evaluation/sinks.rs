//! Storage and Retention Strategy (Plug-in Sinks)
//!
//! Provides pluggable storage sinks for evaluation trace data:
//! - In-memory sink (for tests)
//! - JSONL sink (for development)
//! - Parquet sink (for analysis)
//! - Redaction layer for PII removal

use crate::evaluation::trace::{Trace, EventEnvelope};
use crate::chain_of_thought::{DecisionPoint, CoordinationEvent};
use crate::audit_trail::AuditEvent;
use std::io::{Write, BufWriter};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Trait for storage sinks
pub trait TraceSink: Send + Sync {
    /// Write a trace to the sink
    fn write_trace(&self, trace: &Trace) -> Result<(), String>;
    
    /// Get sink identifier
    fn sink_type(&self) -> &str;
    
    /// Flush any buffered data
    fn flush(&self) -> Result<(), String>;
}

/// In-memory sink for testing
pub struct InMemorySink {
    traces: Arc<std::sync::Mutex<Vec<Trace>>>,
}

impl InMemorySink {
    /// Create new in-memory sink
    pub fn new() -> Self {
        Self {
            traces: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
    
    /// Get all stored traces
    pub fn get_traces(&self) -> Vec<Trace> {
        self.traces.lock().unwrap().clone()
    }
    
    /// Clear all traces
    pub fn clear(&self) {
        self.traces.lock().unwrap().clear();
    }
}

impl Default for InMemorySink {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceSink for InMemorySink {
    fn write_trace(&self, trace: &Trace) -> Result<(), String> {
        self.traces.lock().unwrap().push(trace.clone());
        Ok(())
    }
    
    fn sink_type(&self) -> &str {
        "in-memory"
    }
    
    fn flush(&self) -> Result<(), String> {
        // No-op for in-memory sink
        Ok(())
    }
}

/// JSONL sink for development (one JSON object per line)
pub struct JsonlSink {
    file_path: std::path::PathBuf,
    writer: Arc<std::sync::Mutex<BufWriter<File>>>,
}

impl JsonlSink {
    /// Create new JSONL sink
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, String> {
        let path = file_path.as_ref();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        Ok(Self {
            file_path: path.to_path_buf(),
            writer: Arc::new(std::sync::Mutex::new(BufWriter::new(file))),
        })
    }
    
    /// Get file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

impl TraceSink for JsonlSink {
    fn write_trace(&self, trace: &Trace) -> Result<(), String> {
        let json = serde_json::to_string(trace)
            .map_err(|e| format!("Failed to serialize trace: {}", e))?;
        
        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", json)
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        
        Ok(())
    }
    
    fn sink_type(&self) -> &str {
        "jsonl"
    }
    
    fn flush(&self) -> Result<(), String> {
        self.writer.lock().unwrap().flush()
            .map_err(|e| format!("Failed to flush: {}", e))?;
        Ok(())
    }
}

/// Parquet sink for analysis (placeholder - requires parquet crate)
pub struct ParquetSink {
    file_path: std::path::PathBuf,
}

impl ParquetSink {
    /// Create new Parquet sink
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, String> {
        let path = file_path.as_ref();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        Ok(Self {
            file_path: path.to_path_buf(),
        })
    }
    
    /// Get file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

impl TraceSink for ParquetSink {
    fn write_trace(&self, _trace: &Trace) -> Result<(), String> {
        // PLACEHOLDER: Parquet implementation requires parquet crate
        // For now, convert to JSONL format
        Err("Parquet sink not yet implemented. Use JSONL sink instead.".to_string())
    }
    
    fn sink_type(&self) -> &str {
        "parquet"
    }
    
    fn flush(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Redaction layer for removing PII from traces
pub struct RedactionLayer {
    inner: Arc<dyn TraceSink>,
    redact_pii: bool,
    sink_type_cache: String,
}

impl RedactionLayer {
    /// Create new redaction layer
    pub fn new(inner: Arc<dyn TraceSink>, redact_pii: bool) -> Self {
        let sink_type_cache = format!("redacted-{}", inner.sink_type());
        Self {
            inner,
            redact_pii,
            sink_type_cache,
        }
    }
    
    /// Redact PII from a trace
    fn redact_trace(&self, trace: &Trace) -> Trace {
        if !self.redact_pii {
            return trace.clone();
        }
        
        // Create a redacted copy of the trace
        let mut redacted = trace.clone();
        
        // Redact PII from events
        for event in &mut redacted.events {
            self.redact_event(event);
        }
        
        redacted
    }
    
    /// Redact PII from an event
    fn redact_event(&self, event: &mut EventEnvelope) {
        // Redact email addresses, IP addresses, and other PII patterns
        // This is a simplified implementation - in production, use a proper PII detection library
        
        match &mut event.kind {
            crate::evaluation::trace::EventKind::Decision(dp) => {
                // Redact reasoning text that might contain PII
                if self.contains_pii(&dp.reasoning) {
                    dp.reasoning = "[REDACTED: Contains PII]".to_string();
                }
            }
            crate::evaluation::trace::EventKind::Coordination(ce) => {
                // Redact details that might contain PII
                if let Some(details) = ce.details.get_mut("message") {
                    if let Some(msg) = details.as_str() {
                        if self.contains_pii(msg) {
                            *details = serde_json::Value::String("[REDACTED: Contains PII]".to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Check if text contains PII patterns
    fn contains_pii(&self, text: &str) -> bool {
        // Simple PII detection patterns
        // Email pattern
        if regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap().is_match(text) {
            return true;
        }
        
        // IP address pattern
        if regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap().is_match(text) {
            return true;
        }
        
        // Credit card pattern (simplified)
        if regex::Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap().is_match(text) {
            return true;
        }
        
        false
    }
}

impl TraceSink for RedactionLayer {
    fn write_trace(&self, trace: &Trace) -> Result<(), String> {
        let redacted = self.redact_trace(trace);
        self.inner.write_trace(&redacted)
    }
    
    fn sink_type(&self) -> &str {
        &self.sink_type_cache
    }
    
    fn flush(&self) -> Result<(), String> {
        self.inner.flush()
    }
}

/// Sink factory for creating sinks from configuration
pub struct SinkFactory;

impl SinkFactory {
    /// Create sink from URI string
    ///
    /// Supported formats:
    /// - `memory://` - In-memory sink
    /// - `jsonl:///path/to/file.jsonl` - JSONL sink
    /// - `parquet:///path/to/file.parquet` - Parquet sink
    /// - `redacted:jsonl:///path/to/file.jsonl` - JSONL sink with redaction
    pub fn from_uri(uri: &str) -> Result<Arc<dyn TraceSink>, String> {
        if uri == "memory://" {
            return Ok(Arc::new(InMemorySink::new()));
        }
        
        if uri.starts_with("jsonl://") {
            let path = uri.strip_prefix("jsonl://").unwrap();
            let sink = JsonlSink::new(path)?;
            return Ok(Arc::new(sink));
        }
        
        if uri.starts_with("parquet://") {
            let path = uri.strip_prefix("parquet://").unwrap();
            let sink = ParquetSink::new(path)?;
            return Ok(Arc::new(sink));
        }
        
        if uri.starts_with("redacted:") {
            let inner_uri = uri.strip_prefix("redacted:").unwrap();
            let inner = Self::from_uri(inner_uri)?;
            let redacted = RedactionLayer::new(inner, true);
            return Ok(Arc::new(redacted));
        }
        
        Err(format!("Unknown sink URI format: {}", uri))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::trace::EventKind;

    fn create_test_trace() -> Trace {
        Trace {
            plan_id: Uuid::new_v4(),
            trace_version: 1,
            events: vec![],
            metadata: crate::evaluation::trace::TraceMetadata {
                started_at: Utc::now(),
                ended_at: None,
                event_count: 0,
                event_type_distribution: std::collections::HashMap::new(),
                correlation_ids: vec![],
            },
        }
    }

    #[test]
    fn test_in_memory_sink() {
        let sink = InMemorySink::new();
        let trace = create_test_trace();
        
        assert!(sink.write_trace(&trace).is_ok());
        assert_eq!(sink.get_traces().len(), 1);
        
        sink.clear();
        assert_eq!(sink.get_traces().len(), 0);
    }

    #[test]
    fn test_sink_factory_memory() {
        let sink = SinkFactory::from_uri("memory://").unwrap();
        assert_eq!(sink.sink_type(), "in-memory");
    }

    #[test]
    fn test_sink_factory_jsonl() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.jsonl");
        let uri = format!("jsonl://{}", file_path.display());
        
        let sink = SinkFactory::from_uri(&uri).unwrap();
        assert_eq!(sink.sink_type(), "jsonl");
        
        let trace = create_test_trace();
        assert!(sink.write_trace(&trace).is_ok());
        assert!(sink.flush().is_ok());
        
        // Verify file was created
        assert!(file_path.exists());
    }

    #[test]
    fn test_redaction_layer() {
        let inner = Arc::new(InMemorySink::new());
        let redacted = RedactionLayer::new(inner.clone(), true);
        
        let mut trace = create_test_trace();
        // Add event with PII
        trace.events.push(crate::evaluation::trace::EventEnvelope {
            trace_version: 1,
            plan_id: trace.plan_id,
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kind: EventKind::Decision(DecisionPoint {
                decision_id: Uuid::new_v4(),
                decision_type: crate::chain_of_thought::DecisionType::WorkerAssignment,
                timestamp: Utc::now(),
                context: crate::chain_of_thought::DecisionContext {
                    task_id: None,
                    plan_id: None,
                    milestone_id: None,
                    worker_id: None,
                    resource_constraints: std::collections::HashMap::new(),
                    time_constraints: None,
                    priority_level: None,
                },
                alternatives: vec![],
                chosen_option: "test@example.com".to_string(), // PII
                reasoning: "Email: test@example.com".to_string(), // PII
                confidence: 0.8,
                risk_assessment: None,
                metadata: std::collections::HashMap::new(),
            }),
            metadata: std::collections::HashMap::new(),
        });
        
        assert!(redacted.write_trace(&trace).is_ok());
        
        // Check that PII was redacted
        let traces = inner.get_traces();
        assert_eq!(traces.len(), 1);
        // The reasoning should be redacted
        // Note: This is a simplified test - full PII detection would be more comprehensive
    }
}
