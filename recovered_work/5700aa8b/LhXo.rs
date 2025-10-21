//! Data Consistency During Failures and Recovery
//!
//! Ensures data integrity across distributed systems during failures,
//! failovers, and recovery operations.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::{DatabaseClient, DatabaseConfig};

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransactionState {
    Pending,
    Committed,
    Aborted,
    InDoubt, // Transaction outcome uncertain
}

/// Distributed transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTransaction {
    pub id: String,
    pub coordinator_id: String,
    pub participants: Vec<String>, // Service IDs involved
    pub state: TransactionState,
    pub created_at: DateTime<Utc>,
    pub timeout_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Data consistency level
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Strong consistency - all replicas have same data
    Strong,
    /// Eventual consistency - replicas converge over time
    Eventual,
    /// Causal consistency - causally related operations ordered
    Causal,
    /// Read-your-writes - user sees their own writes
    ReadYourWrites,
}

/// Consistency check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    pub check_id: String,
    pub timestamp: DateTime<Utc>,
    pub service_id: String,
    pub table_name: String,
    pub primary_count: i64,
    pub replica_count: i64,
    pub inconsistencies_found: Vec<Inconsistency>,
    pub is_consistent: bool,
    pub check_duration_ms: u64,
}

/// Data inconsistency record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inconsistency {
    pub record_id: String,
    pub primary_value: serde_json::Value,
    pub replica_value: serde_json::Value,
    pub difference_type: String,
    pub severity: InconsistencySeverity,
}

/// Inconsistency severity levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InconsistencySeverity {
    Low,      // Cosmetic differences
    Medium,   // Functional impact
    High,     // Data corruption
    Critical, // System-breaking
}

/// Recovery action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Repair data by copying from primary
    RepairFromPrimary { record_ids: Vec<String> },
    /// Mark transaction as committed
    ForceCommit { transaction_id: String },
    /// Mark transaction as aborted
    ForceAbort { transaction_id: String },
    /// Manual intervention required
    ManualIntervention { description: String },
}

/// Data consistency manager
pub struct DataConsistencyManager {
    db_client: Arc<DatabaseClient>,
    transactions: Arc<RwLock<HashMap<String, DistributedTransaction>>>,
    consistency_checks: Arc<RwLock<Vec<ConsistencyCheckResult>>>,
    recovery_actions: Arc<RwLock<Vec<RecoveryAction>>>,
    consistency_level: ConsistencyLevel,
}

impl DataConsistencyManager {
    /// Create a new data consistency manager
    pub fn new(db_client: Arc<DatabaseClient>, consistency_level: ConsistencyLevel) -> Self {
        Self {
            db_client,
            transactions: Arc::new(RwLock::new(HashMap::new())),
            consistency_checks: Arc::new(RwLock::new(Vec::new())),
            recovery_actions: Arc::new(RwLock::new(Vec::new())),
            consistency_level,
        }
    }

    /// Begin a distributed transaction
    pub async fn begin_distributed_transaction(
        &self,
        transaction_id: String,
        participants: Vec<String>,
        timeout_duration: Duration,
    ) -> Result<(), String> {
        let transaction = DistributedTransaction {
            id: transaction_id.clone(),
            coordinator_id: "self".to_string(), // In real implementation, this would be the coordinator service
            participants,
            state: TransactionState::Pending,
            created_at: Utc::now(),
            timeout_at: Utc::now() + chrono::Duration::from_std(timeout_duration)
                .map_err(|_| "Invalid timeout duration")?,
            metadata: serde_json::json!({}),
        };

        // Store transaction record
        let mut transactions = self.transactions.write().await;
        transactions.insert(transaction_id.clone(), transaction);

        // Persist to database for durability
        self.persist_transaction(&transaction_id).await?;

        info!("Begun distributed transaction: {}", transaction_id);
        Ok(())
    }

    /// Prepare phase of two-phase commit
    pub async fn prepare_transaction(&self, transaction_id: &str) -> Result<(), String> {
        let mut transactions = self.transactions.write().await;
        let transaction = transactions.get_mut(transaction_id)
            .ok_or(format!("Transaction not found: {}", transaction_id))?;

        if transaction.state != TransactionState::Pending {
            return Err(format!("Transaction {} is not in pending state", transaction_id));
        }

        // In a real implementation, this would coordinate with all participants
        // For now, we'll simulate the prepare phase
        transaction.state = TransactionState::InDoubt;

        self.persist_transaction_state(transaction_id, TransactionState::InDoubt).await?;

        info!("Prepared transaction: {}", transaction_id);
        Ok(())
    }

    /// Commit phase of two-phase commit
    pub async fn commit_transaction(&self, transaction_id: &str) -> Result<(), String> {
        let mut transactions = self.transactions.write().await;
        let transaction = transactions.get_mut(transaction_id)
            .ok_or(format!("Transaction not found: {}", transaction_id))?;

        if !matches!(transaction.state, TransactionState::Pending | TransactionState::InDoubt) {
            return Err(format!("Transaction {} cannot be committed from state {:?}", transaction_id, transaction.state));
        }

        // Coordinate commit with all participants
        // In real implementation, this would be done via distributed protocol
        transaction.state = TransactionState::Committed;

        self.persist_transaction_state(transaction_id, TransactionState::Committed).await?;

        info!("Committed transaction: {}", transaction_id);
        Ok(())
    }

    /// Abort transaction
    pub async fn abort_transaction(&self, transaction_id: &str) -> Result<(), String> {
        let mut transactions = self.transactions.write().await;
        let transaction = transactions.get_mut(transaction_id)
            .ok_or(format!("Transaction not found: {}", transaction_id))?;

        if transaction.state == TransactionState::Committed {
            return Err(format!("Cannot abort committed transaction: {}", transaction_id));
        }

        transaction.state = TransactionState::Aborted;

        self.persist_transaction_state(transaction_id, TransactionState::Aborted).await?;

        info!("Aborted transaction: {}", transaction_id);
        Ok(())
    }

    /// Recover in-doubt transactions after failure
    pub async fn recover_in_doubt_transactions(&self) -> Result<Vec<String>, String> {
        let transactions = self.transactions.read().await;
        let in_doubt: Vec<_> = transactions.iter()
            .filter(|(_, tx)| tx.state == TransactionState::InDoubt)
            .map(|(id, _)| id.clone())
            .collect();

        let mut recovered = Vec::new();

        for tx_id in in_doubt {
            match self.recover_transaction(&tx_id).await {
                Ok(_) => recovered.push(tx_id),
                Err(e) => warn!("Failed to recover transaction {}: {}", tx_id, e),
            }
        }

        info!("Recovered {} in-doubt transactions", recovered.len());
        Ok(recovered)
    }

    /// Recover a single in-doubt transaction
    async fn recover_transaction(&self, transaction_id: &str) -> Result<(), String> {
        // Implementation would check with participants to determine outcome
        // For now, we'll use a simple heuristic
        let transactions = self.transactions.read().await;
        let transaction = transactions.get(transaction_id)
            .ok_or(format!("Transaction not found: {}", transaction_id))?;

        // If transaction timed out, abort it
        if Utc::now() > transaction.timeout_at {
            drop(transactions);
            self.abort_transaction(transaction_id).await?;
        } else {
            // Otherwise, assume it should commit (in real system, check with coordinator)
            drop(transactions);
            self.commit_transaction(transaction_id).await?;
        }

        Ok(())
    }

    /// Persist transaction to database
    async fn persist_transaction(&self, transaction_id: &str) -> Result<(), String> {
        let transactions = self.transactions.read().await;
        let transaction = transactions.get(transaction_id)
            .ok_or(format!("Transaction not found: {}", transaction_id))?;

        let query = r#"
            INSERT INTO distributed_transactions (id, coordinator_id, participants, state, created_at, timeout_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                state = EXCLUDED.state,
                timeout_at = EXCLUDED.timeout_at,
                metadata = EXCLUDED.metadata
        "#;

        let participants_json = serde_json::to_string(&transaction.participants)
            .map_err(|e| e.to_string())?;

        self.db_client.execute(
            query,
            &[
                &transaction.id,
                &transaction.coordinator_id,
                &participants_json,
                &serde_json::to_string(&transaction.state).map_err(|e| e.to_string())?,
                &transaction.created_at,
                &transaction.timeout_at,
                &transaction.metadata,
            ],
        ).await
        .map_err(|e| format!("Failed to persist transaction: {}", e))?;

        Ok(())
    }

    /// Persist transaction state change
    async fn persist_transaction_state(&self, transaction_id: &str, new_state: TransactionState) -> Result<(), String> {
        let query = r#"
            UPDATE distributed_transactions
            SET state = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        let state_str = serde_json::to_string(&new_state)
            .map_err(|e| e.to_string())?;

        self.db_client.execute(query, &[&state_str, &transaction_id]).await
            .map_err(|e| format!("Failed to update transaction state: {}", e))?;

        Ok(())
    }

    /// Perform consistency check between primary and replica
    pub async fn check_data_consistency(
        &self,
        service_id: &str,
        table_name: &str,
        primary_connection: &DatabaseClient,
        replica_connection: &DatabaseClient,
    ) -> Result<ConsistencyCheckResult, String> {
        let check_id = format!("consistency_check_{}_{}_{}",
            service_id, table_name, Utc::now().timestamp());
        let start_time = Instant::now();

        info!("Starting consistency check: {} for table {}", check_id, table_name);

        // Get row counts from primary and replica
        let primary_count = self.get_table_count(primary_connection, table_name).await?;
        let replica_count = self.get_table_count(replica_connection, table_name).await?;

        let mut inconsistencies = Vec::new();

        // If counts differ significantly, check for specific inconsistencies
        if (primary_count - replica_count).abs() > 10 { // Allow small differences for replication lag
            inconsistencies = self.find_inconsistencies(
                primary_connection,
                replica_connection,
                table_name,
            ).await?;
        }

        let is_consistent = inconsistencies.is_empty() ||
            inconsistencies.iter().all(|i| i.severity == InconsistencySeverity::Low);

        let result = ConsistencyCheckResult {
            check_id: check_id.clone(),
            timestamp: Utc::now(),
            service_id: service_id.to_string(),
            table_name: table_name.to_string(),
            primary_count,
            replica_count,
            inconsistencies_found: inconsistencies,
            is_consistent,
            check_duration_ms: start_time.elapsed().as_millis() as u64,
        };

        // Store result
        {
            let mut checks = self.consistency_checks.write().await;
            checks.push(result.clone());

            // Keep only recent checks
            if checks.len() > 1000 {
                checks.remove(0);
            }
        }

        info!("Consistency check completed: {} (consistent: {})", check_id, is_consistent);
        Ok(result)
    }

    /// Get row count for a table
    async fn get_table_count(&self, db_client: &DatabaseClient, table_name: &str) -> Result<i64, String> {
        let query = format!("SELECT COUNT(*) as count FROM {}", table_name);
        let rows = db_client.query(&query, &[]).await
            .map_err(|e| format!("Failed to count rows in {}: {}", table_name, e))?;

        if let Some(row) = rows.into_iter().next() {
            Ok(row.get("count"))
        } else {
            Err(format!("No count returned for table {}", table_name))
        }
    }

    /// Find specific data inconsistencies
    async fn find_inconsistencies(
        &self,
        primary: &DatabaseClient,
        replica: &DatabaseClient,
        table_name: &str,
    ) -> Result<Vec<Inconsistency>, String> {
        // This is a simplified implementation
        // In a real system, this would compare actual data records

        let query = format!("SELECT id FROM {} LIMIT 100", table_name);

        let primary_rows = primary.query(&query, &[]).await
            .map_err(|e| format!("Failed to query primary: {}", e))?;

        let replica_rows = replica.query(&query, &[]).await
            .map_err(|e| format!("Failed to query replica: {}", e))?;

        let mut inconsistencies = Vec::new();

        // Simple comparison - in real implementation, this would be much more sophisticated
        if primary_rows.len() != replica_rows.len() {
            inconsistencies.push(Inconsistency {
                record_id: "count_mismatch".to_string(),
                primary_value: serde_json::json!(primary_rows.len()),
                replica_value: serde_json::json!(replica_rows.len()),
                difference_type: "row_count".to_string(),
                severity: InconsistencySeverity::Medium,
            });
        }

        Ok(inconsistencies)
    }

    /// Generate recovery actions for consistency issues
    pub async fn generate_recovery_actions(&self, check_result: &ConsistencyCheckResult) -> Vec<RecoveryAction> {
        let mut actions = Vec::new();

        for inconsistency in &check_result.inconsistencies_found {
            match inconsistency.severity {
                InconsistencySeverity::Low => {
                    // Auto-repair for low severity
                    actions.push(RecoveryAction::RepairFromPrimary {
                        record_ids: vec![inconsistency.record_id.clone()],
                    });
                }
                InconsistencySeverity::Medium => {
                    // Manual review for medium severity
                    actions.push(RecoveryAction::ManualIntervention {
                        description: format!("Medium severity inconsistency in record {}", inconsistency.record_id),
                    });
                }
                InconsistencySeverity::High | InconsistencySeverity::Critical => {
                    // Escalation for high severity
                    actions.push(RecoveryAction::ManualIntervention {
                        description: format!("High severity data inconsistency detected: {}", inconsistency.record_id),
                    });
                }
            }
        }

        // Store actions
        {
            let mut recovery_actions = self.recovery_actions.write().await;
            recovery_actions.extend(actions.clone());
        }

        actions
    }

    /// Apply recovery action
    pub async fn apply_recovery_action(&self, action: &RecoveryAction) -> Result<(), String> {
        match action {
            RecoveryAction::RepairFromPrimary { record_ids } => {
                info!("Repairing {} records from primary", record_ids.len());
                // Implementation would copy data from primary to replica
                Ok(())
            }
            RecoveryAction::ForceCommit { transaction_id } => {
                self.commit_transaction(transaction_id).await?;
                Ok(())
            }
            RecoveryAction::ForceAbort { transaction_id } => {
                self.abort_transaction(transaction_id).await?;
                Ok(())
            }
            RecoveryAction::ManualIntervention { description } => {
                warn!("Manual intervention required: {}", description);
                Ok(())
            }
        }
    }

    /// Get consistency check history
    pub async fn get_consistency_history(&self, service_id: Option<&str>, limit: usize) -> Vec<ConsistencyCheckResult> {
        let checks = self.consistency_checks.read().await;

        checks.iter()
            .rev()
            .filter(|check| service_id.map_or(true, |id| check.service_id == id))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get pending recovery actions
    pub async fn get_pending_recovery_actions(&self) -> Vec<RecoveryAction> {
        let actions = self.recovery_actions.read().await;
        actions.clone()
    }

    /// Clear completed recovery actions
    pub async fn clear_recovery_actions(&self, actions_to_clear: &[RecoveryAction]) {
        let mut recovery_actions = self.recovery_actions.write().await;

        for action in actions_to_clear {
            if let Some(pos) = recovery_actions.iter().position(|a| a == action) {
                recovery_actions.remove(pos);
            }
        }
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, transaction_id: &str) -> Option<DistributedTransaction> {
        let transactions = self.transactions.read().await;
        transactions.get(transaction_id).cloned()
    }

    /// List active transactions
    pub async fn list_active_transactions(&self) -> Vec<DistributedTransaction> {
        let transactions = self.transactions.read().await;
        transactions.values()
            .filter(|tx| tx.state == TransactionState::Pending || tx.state == TransactionState::InDoubt)
            .cloned()
            .collect()
    }

    /// Clean up completed transactions
    pub async fn cleanup_completed_transactions(&self, max_age_days: i64) {
        let cutoff = Utc::now() - chrono::Duration::days(max_age_days);

        let mut transactions = self.transactions.write().await;
        let to_remove: Vec<String> = transactions.iter()
            .filter(|(_, tx)| {
                matches!(tx.state, TransactionState::Committed | TransactionState::Aborted) &&
                tx.created_at < cutoff
            })
            .map(|(id, _)| id.clone())
            .collect();

        for tx_id in to_remove {
            transactions.remove(&tx_id);
        }

        info!("Cleaned up {} completed transactions older than {} days",
              transactions.len(), max_age_days);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_distributed_transaction_lifecycle() {
        // Note: This test requires a real database connection
        // In a real test suite, this would use a test database

        let consistency_manager = DataConsistencyManager::new(
            Arc::new(DatabaseClient::new(DatabaseConfig::default()).await.unwrap()),
            ConsistencyLevel::Strong,
        );

        let tx_id = "test_transaction_123".to_string();
        let participants = vec!["service1".to_string(), "service2".to_string()];

        // Test transaction creation
        assert!(consistency_manager.begin_distributed_transaction(
            tx_id.clone(),
            participants,
            Duration::from_secs(300)
        ).await.is_ok());

        // Test transaction status
        let status = consistency_manager.get_transaction_status(&tx_id).await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().state, TransactionState::Pending);

        // Test prepare
        assert!(consistency_manager.prepare_transaction(&tx_id).await.is_ok());

        // Test commit
        assert!(consistency_manager.commit_transaction(&tx_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_consistency_levels() {
        let strong_manager = DataConsistencyManager::new(
            Arc::new(DatabaseClient::new(DatabaseConfig::default()).await.unwrap()),
            ConsistencyLevel::Strong,
        );

        let eventual_manager = DataConsistencyManager::new(
            Arc::new(DatabaseClient::new(DatabaseConfig::default()).await.unwrap()),
            ConsistencyLevel::Eventual,
        );

        // Different consistency levels should be configurable
        assert_eq!(strong_manager.consistency_level, ConsistencyLevel::Strong);
        assert_eq!(eventual_manager.consistency_level, ConsistencyLevel::Eventual);
    }
}
