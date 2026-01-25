//! Event Types for Event Sourcing
//!
//! All state changes in the system are represented as events.
//! Events are append-only and content-hashed for audit trail.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Unique identifier for an event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub uuid::Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Sequence number for ordering events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SequenceNumber(pub u64);

impl SequenceNumber {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

/// Domain events for the agent system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    // Task lifecycle
    TaskCreated {
        task_id: String,
        title: String,
        description: String,
        risk_tier: u8,
    },
    TaskDecomposed {
        task_id: String,
        subtask_ids: Vec<String>,
    },
    TaskStarted {
        task_id: String,
        worker_id: String,
    },
    TaskCompleted {
        task_id: String,
        result_hash: String,
    },
    TaskFailed {
        task_id: String,
        error: String,
    },

    // Governance
    InvariantChecked {
        invariant_id: String,
        task_id: String,
        passed: bool,
        details: Option<String>,
    },
    CouncilConvened {
        task_id: String,
        constitutional_score: f64,
        technical_score: f64,
        quality_score: f64,
        aggregate_score: f64,
        passed: bool,
    },
    GatePassed {
        gate_id: String,
        task_id: String,
        score: f64,
    },
    GateFailed {
        gate_id: String,
        task_id: String,
        score: f64,
        threshold: f64,
    },

    // Memory
    MemoryStored {
        key: String,
        content_hash: String,
        memory_type: String,
    },
    MemoryRecalled {
        key: String,
        provenance: Vec<String>,
    },
    MemoryDecayed {
        key: String,
        old_weight: f64,
        new_weight: f64,
    },

    // Execution
    ToolInvoked {
        tool_name: String,
        args_hash: String,
        task_id: String,
    },
    ToolCompleted {
        tool_name: String,
        result_hash: String,
        success: bool,
    },
    WorkerSpawned {
        worker_id: String,
        task_id: String,
        worker_type: String,
    },
    WorkerCompleted {
        worker_id: String,
        status: String,
    },

    // Authorization
    ExecutionAuthorized {
        task_id: String,
        certificate_hash: String,
        worker_type: String,
    },
    ExecutionDenied {
        task_id: String,
        reason: String,
    },
}

impl AgentEvent {
    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::TaskCreated { .. } => "TaskCreated",
            Self::TaskDecomposed { .. } => "TaskDecomposed",
            Self::TaskStarted { .. } => "TaskStarted",
            Self::TaskCompleted { .. } => "TaskCompleted",
            Self::TaskFailed { .. } => "TaskFailed",
            Self::InvariantChecked { .. } => "InvariantChecked",
            Self::CouncilConvened { .. } => "CouncilConvened",
            Self::GatePassed { .. } => "GatePassed",
            Self::GateFailed { .. } => "GateFailed",
            Self::MemoryStored { .. } => "MemoryStored",
            Self::MemoryRecalled { .. } => "MemoryRecalled",
            Self::MemoryDecayed { .. } => "MemoryDecayed",
            Self::ToolInvoked { .. } => "ToolInvoked",
            Self::ToolCompleted { .. } => "ToolCompleted",
            Self::WorkerSpawned { .. } => "WorkerSpawned",
            Self::WorkerCompleted { .. } => "WorkerCompleted",
            Self::ExecutionAuthorized { .. } => "ExecutionAuthorized",
            Self::ExecutionDenied { .. } => "ExecutionDenied",
        }
    }

    /// Get the associated task ID if any
    pub fn task_id(&self) -> Option<&str> {
        match self {
            Self::TaskCreated { task_id, .. }
            | Self::TaskDecomposed { task_id, .. }
            | Self::TaskStarted { task_id, .. }
            | Self::TaskCompleted { task_id, .. }
            | Self::TaskFailed { task_id, .. }
            | Self::InvariantChecked { task_id, .. }
            | Self::CouncilConvened { task_id, .. }
            | Self::GatePassed { task_id, .. }
            | Self::GateFailed { task_id, .. }
            | Self::ToolInvoked { task_id, .. }
            | Self::ExecutionAuthorized { task_id, .. }
            | Self::ExecutionDenied { task_id, .. } => Some(task_id),
            _ => None,
        }
    }
}

/// A stored event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Unique event identifier
    pub id: EventId,
    /// Sequence number for ordering
    pub sequence: SequenceNumber,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The event data
    pub event: AgentEvent,
    /// SHA-256 hash of the event content
    pub content_hash: String,
    /// Optional correlation ID for related events
    pub correlation_id: Option<String>,
}

impl StoredEvent {
    /// Create a new stored event
    pub fn new(event: AgentEvent, sequence: SequenceNumber) -> Self {
        let id = EventId::new();
        let timestamp = Utc::now();
        let content_hash = compute_event_hash(&event, &timestamp);

        Self {
            id,
            sequence,
            timestamp,
            event,
            content_hash,
            correlation_id: None,
        }
    }

    /// Create with correlation ID
    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Verify the content hash
    pub fn verify(&self) -> bool {
        let expected = compute_event_hash(&self.event, &self.timestamp);
        self.content_hash == expected
    }
}

/// Compute SHA-256 hash of event content
pub fn compute_event_hash(event: &AgentEvent, timestamp: &DateTime<Utc>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(event.event_type().as_bytes());
    hasher.update(timestamp.to_rfc3339().as_bytes());

    if let Ok(json) = serde_json::to_string(event) {
        hasher.update(json.as_bytes());
    }

    hex::encode(hasher.finalize())
}

/// Event stream for a specific aggregate (e.g., task, worker)
#[derive(Debug, Clone)]
pub struct EventStream {
    /// Aggregate type (e.g., "task", "worker")
    pub aggregate_type: String,
    /// Aggregate ID
    pub aggregate_id: String,
    /// Events in the stream
    pub events: Vec<StoredEvent>,
}

impl EventStream {
    pub fn new(aggregate_type: impl Into<String>, aggregate_id: impl Into<String>) -> Self {
        Self {
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            events: Vec::new(),
        }
    }

    pub fn append(&mut self, event: StoredEvent) {
        self.events.push(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn latest_sequence(&self) -> Option<SequenceNumber> {
        self.events.last().map(|e| e.sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_creation() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_sequence_number_ordering() {
        let seq1 = SequenceNumber(1);
        let seq2 = SequenceNumber(2);
        assert!(seq1 < seq2);
        assert_eq!(seq1.next(), seq2);
    }

    #[test]
    fn test_event_type() {
        let event = AgentEvent::TaskCreated {
            task_id: "t-1".to_string(),
            title: "Test".to_string(),
            description: "Test task".to_string(),
            risk_tier: 1,
        };
        assert_eq!(event.event_type(), "TaskCreated");
        assert_eq!(event.task_id(), Some("t-1"));
    }

    #[test]
    fn test_stored_event_hash_verification() {
        let event = AgentEvent::TaskCreated {
            task_id: "t-1".to_string(),
            title: "Test".to_string(),
            description: "Test task".to_string(),
            risk_tier: 1,
        };
        let stored = StoredEvent::new(event, SequenceNumber(1));
        assert!(stored.verify());
    }

    #[test]
    fn test_event_hash_determinism() {
        let event = AgentEvent::TaskCompleted {
            task_id: "t-1".to_string(),
            result_hash: "abc123".to_string(),
        };
        let timestamp = Utc::now();

        let hash1 = compute_event_hash(&event, &timestamp);
        let hash2 = compute_event_hash(&event, &timestamp);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_event_stream() {
        let mut stream = EventStream::new("task", "t-1");
        assert!(stream.is_empty());

        let event = AgentEvent::TaskCreated {
            task_id: "t-1".to_string(),
            title: "Test".to_string(),
            description: "Test task".to_string(),
            risk_tier: 1,
        };
        stream.append(StoredEvent::new(event, SequenceNumber(1)));

        assert_eq!(stream.len(), 1);
        assert_eq!(stream.latest_sequence(), Some(SequenceNumber(1)));
    }

    #[test]
    fn test_event_with_correlation() {
        let event = AgentEvent::TaskStarted {
            task_id: "t-1".to_string(),
            worker_id: "w-1".to_string(),
        };
        let stored = StoredEvent::new(event, SequenceNumber(1))
            .with_correlation("corr-123");
        assert_eq!(stored.correlation_id, Some("corr-123".to_string()));
    }
}
