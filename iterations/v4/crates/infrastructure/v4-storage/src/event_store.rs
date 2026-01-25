//! Event Store
//!
//! Append-only storage for domain events with content hashing.

use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::events::{AgentEvent, EventId, SequenceNumber, StoredEvent};
use crate::schema;
use crate::StorageError;

/// Event store for persisting and querying events
pub struct EventStore {
    conn: Arc<Mutex<Connection>>,
    current_sequence: Arc<Mutex<u64>>,
}

impl EventStore {
    /// Create a new event store with an in-memory database
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        Self::from_connection(conn)
    }

    /// Create a new event store with a file-based database
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    /// Create from an existing connection
    fn from_connection(conn: Connection) -> Result<Self, StorageError> {
        schema::initialize_schema(&conn)?;

        // Get the current max sequence number
        let max_seq: u64 = conn
            .query_row("SELECT COALESCE(MAX(sequence), 0) FROM events", [], |row| {
                row.get(0)
            })?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            current_sequence: Arc::new(Mutex::new(max_seq)),
        })
    }

    /// Append an event to the store
    pub fn append(&self, event: AgentEvent) -> Result<StoredEvent, StorageError> {
        self.append_with_correlation(event, None)
    }

    /// Append an event with a correlation ID
    pub fn append_with_correlation(
        &self,
        event: AgentEvent,
        correlation_id: Option<String>,
    ) -> Result<StoredEvent, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut seq = self.current_sequence.lock().unwrap();

        *seq += 1;
        let sequence = SequenceNumber(*seq);
        let mut stored = StoredEvent::new(event, sequence);

        if let Some(corr_id) = correlation_id {
            stored = stored.with_correlation(corr_id);
        }

        let payload = serde_json::to_string(&stored.event)?;
        let (aggregate_type, aggregate_id) = extract_aggregate(&stored.event);

        conn.execute(
            r#"
            INSERT INTO events (
                id, sequence, event_type, aggregate_type, aggregate_id,
                timestamp, content_hash, correlation_id, payload
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                stored.id.to_string(),
                stored.sequence.0,
                stored.event.event_type(),
                aggregate_type,
                aggregate_id,
                stored.timestamp.to_rfc3339(),
                stored.content_hash,
                stored.correlation_id,
                payload,
            ],
        )?;

        Ok(stored)
    }

    /// Get all events
    pub fn get_all(&self) -> Result<Vec<StoredEvent>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sequence, timestamp, content_hash, correlation_id, payload
             FROM events ORDER BY sequence ASC",
        )?;

        let events = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let sequence: u64 = row.get(1)?;
                let timestamp_str: String = row.get(2)?;
                let content_hash: String = row.get(3)?;
                let correlation_id: Option<String> = row.get(4)?;
                let payload: String = row.get(5)?;

                Ok((
                    id_str,
                    sequence,
                    timestamp_str,
                    content_hash,
                    correlation_id,
                    payload,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(id_str, sequence, timestamp_str, content_hash, correlation_id, payload)| {
                    parse_stored_event(
                        &id_str,
                        sequence,
                        &timestamp_str,
                        content_hash,
                        correlation_id,
                        &payload,
                    )
                },
            )
            .collect();

        Ok(events)
    }

    /// Get events by type
    pub fn get_by_type(&self, event_type: &str) -> Result<Vec<StoredEvent>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sequence, timestamp, content_hash, correlation_id, payload
             FROM events WHERE event_type = ?1 ORDER BY sequence ASC",
        )?;

        let events = stmt
            .query_map([event_type], |row| {
                let id_str: String = row.get(0)?;
                let sequence: u64 = row.get(1)?;
                let timestamp_str: String = row.get(2)?;
                let content_hash: String = row.get(3)?;
                let correlation_id: Option<String> = row.get(4)?;
                let payload: String = row.get(5)?;

                Ok((
                    id_str,
                    sequence,
                    timestamp_str,
                    content_hash,
                    correlation_id,
                    payload,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(id_str, sequence, timestamp_str, content_hash, correlation_id, payload)| {
                    parse_stored_event(
                        &id_str,
                        sequence,
                        &timestamp_str,
                        content_hash,
                        correlation_id,
                        &payload,
                    )
                },
            )
            .collect();

        Ok(events)
    }

    /// Get events for an aggregate
    pub fn get_by_aggregate(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sequence, timestamp, content_hash, correlation_id, payload
             FROM events WHERE aggregate_type = ?1 AND aggregate_id = ?2
             ORDER BY sequence ASC",
        )?;

        let events = stmt
            .query_map([aggregate_type, aggregate_id], |row| {
                let id_str: String = row.get(0)?;
                let sequence: u64 = row.get(1)?;
                let timestamp_str: String = row.get(2)?;
                let content_hash: String = row.get(3)?;
                let correlation_id: Option<String> = row.get(4)?;
                let payload: String = row.get(5)?;

                Ok((
                    id_str,
                    sequence,
                    timestamp_str,
                    content_hash,
                    correlation_id,
                    payload,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(id_str, sequence, timestamp_str, content_hash, correlation_id, payload)| {
                    parse_stored_event(
                        &id_str,
                        sequence,
                        &timestamp_str,
                        content_hash,
                        correlation_id,
                        &payload,
                    )
                },
            )
            .collect();

        Ok(events)
    }

    /// Get events after a sequence number
    pub fn get_after(&self, sequence: SequenceNumber) -> Result<Vec<StoredEvent>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sequence, timestamp, content_hash, correlation_id, payload
             FROM events WHERE sequence > ?1 ORDER BY sequence ASC",
        )?;

        let events = stmt
            .query_map([sequence.0], |row| {
                let id_str: String = row.get(0)?;
                let sequence: u64 = row.get(1)?;
                let timestamp_str: String = row.get(2)?;
                let content_hash: String = row.get(3)?;
                let correlation_id: Option<String> = row.get(4)?;
                let payload: String = row.get(5)?;

                Ok((
                    id_str,
                    sequence,
                    timestamp_str,
                    content_hash,
                    correlation_id,
                    payload,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(id_str, sequence, timestamp_str, content_hash, correlation_id, payload)| {
                    parse_stored_event(
                        &id_str,
                        sequence,
                        &timestamp_str,
                        content_hash,
                        correlation_id,
                        &payload,
                    )
                },
            )
            .collect();

        Ok(events)
    }

    /// Get events by correlation ID
    pub fn get_by_correlation(
        &self,
        correlation_id: &str,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sequence, timestamp, content_hash, correlation_id, payload
             FROM events WHERE correlation_id = ?1 ORDER BY sequence ASC",
        )?;

        let events = stmt
            .query_map([correlation_id], |row| {
                let id_str: String = row.get(0)?;
                let sequence: u64 = row.get(1)?;
                let timestamp_str: String = row.get(2)?;
                let content_hash: String = row.get(3)?;
                let correlation_id: Option<String> = row.get(4)?;
                let payload: String = row.get(5)?;

                Ok((
                    id_str,
                    sequence,
                    timestamp_str,
                    content_hash,
                    correlation_id,
                    payload,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(id_str, sequence, timestamp_str, content_hash, correlation_id, payload)| {
                    parse_stored_event(
                        &id_str,
                        sequence,
                        &timestamp_str,
                        content_hash,
                        correlation_id,
                        &payload,
                    )
                },
            )
            .collect();

        Ok(events)
    }

    /// Get the current sequence number
    pub fn current_sequence(&self) -> SequenceNumber {
        let seq = self.current_sequence.lock().unwrap();
        SequenceNumber(*seq)
    }

    /// Get the total number of events
    pub fn count(&self) -> Result<u64, StorageError> {
        let conn = self.conn.lock().unwrap();
        let count: u64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Verify all events in the store
    pub fn verify_all(&self) -> Result<Vec<(EventId, bool)>, StorageError> {
        let events = self.get_all()?;
        Ok(events.iter().map(|e| (e.id, e.verify())).collect())
    }
}

/// Extract aggregate type and ID from an event
fn extract_aggregate(event: &AgentEvent) -> (Option<String>, Option<String>) {
    match event {
        AgentEvent::TaskCreated { task_id, .. }
        | AgentEvent::TaskDecomposed { task_id, .. }
        | AgentEvent::TaskStarted { task_id, .. }
        | AgentEvent::TaskCompleted { task_id, .. }
        | AgentEvent::TaskFailed { task_id, .. } => (Some("task".to_string()), Some(task_id.clone())),

        AgentEvent::WorkerSpawned { worker_id, .. }
        | AgentEvent::WorkerCompleted { worker_id, .. } => {
            (Some("worker".to_string()), Some(worker_id.clone()))
        }

        _ => (None, None),
    }
}

/// Parse a stored event from database fields
fn parse_stored_event(
    id_str: &str,
    sequence: u64,
    timestamp_str: &str,
    content_hash: String,
    correlation_id: Option<String>,
    payload: &str,
) -> Option<StoredEvent> {
    let id = uuid::Uuid::parse_str(id_str).ok()?;
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
        .ok()?
        .with_timezone(&Utc);
    let event: AgentEvent = serde_json::from_str(payload).ok()?;

    Some(StoredEvent {
        id: EventId(id),
        sequence: SequenceNumber(sequence),
        timestamp,
        event,
        content_hash,
        correlation_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store_in_memory() {
        let store = EventStore::in_memory().unwrap();
        assert_eq!(store.count().unwrap(), 0);
        assert_eq!(store.current_sequence(), SequenceNumber(0));
    }

    #[test]
    fn test_append_event() {
        let store = EventStore::in_memory().unwrap();

        let event = AgentEvent::TaskCreated {
            task_id: "t-1".to_string(),
            title: "Test task".to_string(),
            description: "A test task".to_string(),
            risk_tier: 1,
        };

        let stored = store.append(event).unwrap();
        assert_eq!(stored.sequence, SequenceNumber(1));
        assert!(stored.verify());

        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(store.current_sequence(), SequenceNumber(1));
    }

    #[test]
    fn test_append_multiple_events() {
        let store = EventStore::in_memory().unwrap();

        for i in 0..5 {
            let event = AgentEvent::TaskCreated {
                task_id: format!("t-{}", i),
                title: format!("Task {}", i),
                description: "Test".to_string(),
                risk_tier: 1,
            };
            store.append(event).unwrap();
        }

        assert_eq!(store.count().unwrap(), 5);
        assert_eq!(store.current_sequence(), SequenceNumber(5));
    }

    #[test]
    fn test_get_all_events() {
        let store = EventStore::in_memory().unwrap();

        store
            .append(AgentEvent::TaskCreated {
                task_id: "t-1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                risk_tier: 1,
            })
            .unwrap();

        store
            .append(AgentEvent::TaskStarted {
                task_id: "t-1".to_string(),
                worker_id: "w-1".to_string(),
            })
            .unwrap();

        let events = store.get_all().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].sequence, SequenceNumber(1));
        assert_eq!(events[1].sequence, SequenceNumber(2));
    }

    #[test]
    fn test_get_by_type() {
        let store = EventStore::in_memory().unwrap();

        store
            .append(AgentEvent::TaskCreated {
                task_id: "t-1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                risk_tier: 1,
            })
            .unwrap();

        store
            .append(AgentEvent::TaskStarted {
                task_id: "t-1".to_string(),
                worker_id: "w-1".to_string(),
            })
            .unwrap();

        store
            .append(AgentEvent::TaskCreated {
                task_id: "t-2".to_string(),
                title: "Test 2".to_string(),
                description: "Test".to_string(),
                risk_tier: 2,
            })
            .unwrap();

        let created_events = store.get_by_type("TaskCreated").unwrap();
        assert_eq!(created_events.len(), 2);
    }

    #[test]
    fn test_get_by_aggregate() {
        let store = EventStore::in_memory().unwrap();

        store
            .append(AgentEvent::TaskCreated {
                task_id: "t-1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                risk_tier: 1,
            })
            .unwrap();

        store
            .append(AgentEvent::TaskStarted {
                task_id: "t-1".to_string(),
                worker_id: "w-1".to_string(),
            })
            .unwrap();

        store
            .append(AgentEvent::TaskCompleted {
                task_id: "t-1".to_string(),
                result_hash: "abc".to_string(),
            })
            .unwrap();

        store
            .append(AgentEvent::TaskCreated {
                task_id: "t-2".to_string(),
                title: "Other".to_string(),
                description: "Other".to_string(),
                risk_tier: 1,
            })
            .unwrap();

        let t1_events = store.get_by_aggregate("task", "t-1").unwrap();
        assert_eq!(t1_events.len(), 3);

        let t2_events = store.get_by_aggregate("task", "t-2").unwrap();
        assert_eq!(t2_events.len(), 1);
    }

    #[test]
    fn test_get_after_sequence() {
        let store = EventStore::in_memory().unwrap();

        for i in 1..=5 {
            store
                .append(AgentEvent::TaskCreated {
                    task_id: format!("t-{}", i),
                    title: format!("Task {}", i),
                    description: "Test".to_string(),
                    risk_tier: 1,
                })
                .unwrap();
        }

        let events = store.get_after(SequenceNumber(3)).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].sequence, SequenceNumber(4));
        assert_eq!(events[1].sequence, SequenceNumber(5));
    }

    #[test]
    fn test_correlation_id() {
        let store = EventStore::in_memory().unwrap();

        store
            .append_with_correlation(
                AgentEvent::TaskCreated {
                    task_id: "t-1".to_string(),
                    title: "Test".to_string(),
                    description: "Test".to_string(),
                    risk_tier: 1,
                },
                Some("request-123".to_string()),
            )
            .unwrap();

        store
            .append_with_correlation(
                AgentEvent::TaskStarted {
                    task_id: "t-1".to_string(),
                    worker_id: "w-1".to_string(),
                },
                Some("request-123".to_string()),
            )
            .unwrap();

        store
            .append(AgentEvent::TaskCreated {
                task_id: "t-2".to_string(),
                title: "Other".to_string(),
                description: "Other".to_string(),
                risk_tier: 1,
            })
            .unwrap();

        let correlated = store.get_by_correlation("request-123").unwrap();
        assert_eq!(correlated.len(), 2);
    }

    #[test]
    fn test_verify_all() {
        let store = EventStore::in_memory().unwrap();

        store
            .append(AgentEvent::TaskCreated {
                task_id: "t-1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                risk_tier: 1,
            })
            .unwrap();

        store
            .append(AgentEvent::TaskCompleted {
                task_id: "t-1".to_string(),
                result_hash: "abc".to_string(),
            })
            .unwrap();

        let results = store.verify_all().unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(_, valid)| *valid));
    }
}
