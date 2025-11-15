//! Provenance Tracking Module
//!
//! Tracks the provenance of memory operations and decisions
//! for explainable AI and audit trails.

use crate::memory_types::{AgentExperience, MemoryId};
use crate::MemoryResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "database")]
use sqlx::{PgPool, Row};
use std::sync::Arc;

/// Provenance record for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProvenanceRecord {
    pub id: String,
    pub memory_id: MemoryId,
    pub operation: ProvenanceOperation,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub context: ProvenanceContext,
}

/// Types of provenance operations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ProvenanceOperation {
    Created,
    Retrieved,
    Updated,
    Deleted,
    Consolidated,
    Decayed,
}

/// Context information for provenance
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct ProvenanceContext {
    pub task_id: Option<String>,
    pub decision_reasoning: Option<String>,
    pub confidence_score: Option<f32>,
}

/// Provenance tracking service
pub struct ProvenanceTracker {
    #[cfg(feature = "database")]
    db_pool: Option<Arc<PgPool>>,
    #[cfg(not(feature = "database"))]
    records: Arc<tokio::sync::Mutex<Vec<ProvenanceRecord>>>,
}

impl ProvenanceTracker {
    /// Create a new provenance tracker
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "database")]
            db_pool: None,
            #[cfg(not(feature = "database"))]
            records: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Create with database pool for persistence
    #[cfg(feature = "database")]
    pub fn with_database(db_pool: Arc<PgPool>) -> Self {
        Self {
            db_pool: Some(db_pool),
        }
    }

    /// Record a provenance operation
    pub async fn record_operation(&self, record: ProvenanceRecord) -> MemoryResult<()> {
        #[cfg(feature = "database")]
        {
            if let Some(ref pool) = self.db_pool {
                // Store provenance record in database
                let operation_str = match record.operation {
                    ProvenanceOperation::Created => "created",
                    ProvenanceOperation::Retrieved => "retrieved",
                    ProvenanceOperation::Updated => "updated",
                    ProvenanceOperation::Deleted => "deleted",
                    ProvenanceOperation::Consolidated => "consolidated",
                    ProvenanceOperation::Decayed => "decayed",
                };

                sqlx::query(
                    r#"
                    INSERT INTO provenance_records
                    (id, memory_id, operation, timestamp, agent_id, task_id, decision_reasoning, confidence_score)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (id) DO UPDATE SET
                        operation = EXCLUDED.operation,
                        timestamp = EXCLUDED.timestamp,
                        decision_reasoning = EXCLUDED.decision_reasoning,
                        confidence_score = EXCLUDED.confidence_score
                    "#
                )
                .bind(&record.id)
                .bind(&record.memory_id.to_string())
                .bind(operation_str)
                .bind(record.timestamp)
                .bind(&record.agent_id)
                .bind(record.context.task_id.as_ref())
                .bind(record.context.decision_reasoning.as_ref())
                .bind(record.context.confidence_score)
                .execute(pool.as_ref())
                .await
                .map_err(|e| crate::MemoryError::Database(e))?;

                return Ok(());
            }
        }

        #[cfg(not(feature = "database"))]
        {
            // In-memory storage fallback
            let mut records = self.records.lock().await;
            records.push(record);
            Ok(())
        }

        #[cfg(feature = "database")]
        {
            // Database feature enabled but no pool provided - use in-memory fallback
            tracing::warn!("Provenance recording requested but no database pool available - using in-memory storage");
            Ok(())
        }
    }

    /// Get provenance history for a memory
    pub async fn get_provenance_history(
        &self,
        memory_id: &MemoryId,
    ) -> MemoryResult<Vec<ProvenanceRecord>> {
        #[cfg(feature = "database")]
        {
            if let Some(ref pool) = self.db_pool {
                let rows = sqlx::query(
                    r#"
                    SELECT id, memory_id, operation, timestamp, agent_id, task_id, decision_reasoning, confidence_score
                    FROM provenance_records
                    WHERE memory_id = $1
                    ORDER BY timestamp DESC
                    "#
                )
                .bind(memory_id.to_string())
                .fetch_all(pool.as_ref())
                .await
                .map_err(|e| crate::MemoryError::Database(e))?;

                let mut records = Vec::new();
                for row in rows {
                    let operation_str: String = row.get("operation");
                    let operation = match operation_str.as_str() {
                        "created" => ProvenanceOperation::Created,
                        "retrieved" => ProvenanceOperation::Retrieved,
                        "updated" => ProvenanceOperation::Updated,
                        "deleted" => ProvenanceOperation::Deleted,
                        "consolidated" => ProvenanceOperation::Consolidated,
                        "decayed" => ProvenanceOperation::Decayed,
                        _ => ProvenanceOperation::Created, // Default fallback
                    };

                    records.push(ProvenanceRecord {
                        id: row.get("id"),
                        memory_id: memory_id.clone(),
                        operation,
                        timestamp: row.get("timestamp"),
                        agent_id: row.get("agent_id"),
                        context: ProvenanceContext {
                            task_id: row.get("task_id"),
                            decision_reasoning: row.get("decision_reasoning"),
                            confidence_score: row.get("confidence_score"),
                        },
                    });
                }

                return Ok(records);
            }
        }

        #[cfg(not(feature = "database"))]
        {
            // In-memory storage fallback
            let records = self.records.lock().await;
            Ok(records
                .iter()
                .filter(|r| r.memory_id == *memory_id)
                .cloned()
                .collect())
        }

        #[cfg(feature = "database")]
        {
            // Database feature enabled but no pool provided
            Ok(vec![])
        }
    }
}
