//! Database Schema
//!
//! SQLite schema for event storage and projections.

use rusqlite::Connection;

/// Initialize the database schema
pub fn initialize_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Events table - append-only event log
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            sequence INTEGER NOT NULL UNIQUE,
            event_type TEXT NOT NULL,
            aggregate_type TEXT,
            aggregate_id TEXT,
            timestamp TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            correlation_id TEXT,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
        [],
    )?;

    // Index for efficient queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_aggregate
         ON events(aggregate_type, aggregate_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_correlation ON events(correlation_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)",
        [],
    )?;

    // Tasks projection - current state of tasks
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            risk_tier INTEGER NOT NULL DEFAULT 1,
            worker_id TEXT,
            result_hash TEXT,
            error TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
        [],
    )?;

    // Workers projection - current state of workers
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS workers (
            id TEXT PRIMARY KEY,
            worker_type TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'idle',
            current_task_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
        [],
    )?;

    // Council verdicts - historical record
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS council_verdicts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            constitutional_score REAL NOT NULL,
            technical_score REAL NOT NULL,
            quality_score REAL NOT NULL,
            aggregate_score REAL NOT NULL,
            passed INTEGER NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        )
        "#,
        [],
    )?;

    // Gate results - historical record
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS gate_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            gate_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            score REAL NOT NULL,
            threshold REAL NOT NULL,
            passed INTEGER NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
        [],
    )?;

    // Snapshots for rebuilding projections
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            aggregate_type TEXT NOT NULL,
            aggregate_id TEXT NOT NULL,
            sequence INTEGER NOT NULL,
            state TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(aggregate_type, aggregate_id)
        )
        "#,
        [],
    )?;

    // Metadata table for tracking schema version
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
        [],
    )?;

    // Set schema version
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', '1')",
        [],
    )?;

    Ok(())
}

/// Get the current schema version
pub fn get_schema_version(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT value FROM metadata WHERE key = 'schema_version'",
        [],
        |row| row.get(0),
    )
    .ok()
}

/// Check if the schema is initialized
pub fn is_initialized(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='events'",
        [],
        |_| Ok(()),
    )
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_initialize_schema() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Verify tables exist
        assert!(is_initialized(&conn));
        assert_eq!(get_schema_version(&conn), Some("1".to_string()));
    }

    #[test]
    fn test_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Initialize twice should not fail
        initialize_schema(&conn).unwrap();
        initialize_schema(&conn).unwrap();

        assert!(is_initialized(&conn));
    }

    #[test]
    fn test_not_initialized() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(!is_initialized(&conn));
        assert_eq!(get_schema_version(&conn), None);
    }
}
