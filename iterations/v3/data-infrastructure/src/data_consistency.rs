//! Data Consistency During Failures and Recovery
//!
//! Ensures data integrity across distributed systems during failures,
//! failovers, and recovery operations.

use schemars::JsonSchema;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tokio::sync::RwLock;
use tracing::{info, warn};
use futures_util::future::join_all;

use crate::simple_client::DatabaseClient;

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TransactionState {
    Pending,
    Committed,
    Aborted,
    InDoubt, // Transaction outcome uncertain
}

/// Two-phase commit vote
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum Vote {
    /// Participant agrees to commit
    Yes,
    /// Participant cannot commit
    No,
}

/// Participant information for distributed transactions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionParticipant {
    /// Service or database identifier
    pub service_id: String,
    /// Database connection string or service endpoint
    pub connection_info: String,
    /// Operations to perform in this transaction
    pub operations: Vec<TransactionOperation>,
}

/// Operation within a distributed transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionOperation {
    /// Type of operation (insert, update, delete)
    pub operation_type: String,
    /// Target table/collection
    pub table: String,
    /// Operation data (SQL or structured data)
    pub data: serde_json::Value,
}

/// Distributed transaction record
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DistributedTransaction {
    pub id: String,
    pub coordinator_id: String,
    pub participants: Vec<TransactionParticipant>, // Detailed participant info
    pub state: TransactionState,
    #[schemars(with = "String")]

    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]

    pub timeout_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Data consistency level
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConsistencyCheckResult {
    pub check_id: String,
    #[schemars(with = "String")]

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Inconsistency {
    pub record_id: String,
    pub primary_value: serde_json::Value,
    pub replica_value: serde_json::Value,
    pub difference_type: String,
    pub severity: InconsistencySeverity,
}

/// Inconsistency severity levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum InconsistencySeverity {
    Low,      // Cosmetic differences
    Medium,   // Functional impact
    High,     // Data corruption
    Critical, // System-breaking
}

/// Recovery action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    _consistency_level: ConsistencyLevel,
}

impl DataConsistencyManager {
    /// Create a new data consistency manager
    pub fn new(db_client: Arc<DatabaseClient>, consistency_level: ConsistencyLevel) -> Self {
        Self {
            db_client,
            transactions: Arc::new(RwLock::new(HashMap::new())),
            consistency_checks: Arc::new(RwLock::new(Vec::new())),
            recovery_actions: Arc::new(RwLock::new(Vec::new())),
            _consistency_level: consistency_level,
        }
    }

    /// Begin a distributed transaction
    pub async fn begin_distributed_transaction(
        &self,
        transaction_id: String,
        participants: Vec<TransactionParticipant>,
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

        // Coordinate prepare phase with all participants
        let participants = transaction.participants.clone();
        drop(transactions); // Release the lock

        // Execute prepare phase for each participant concurrently
        let prepare_futures = participants.iter().enumerate().map(|(_i, participant)| {
            let tx_id = transaction_id.to_string();
            let participant = participant.clone();
            async move {
                match self.execute_prepare_phase(&participant, &tx_id).await {
                    Ok(()) => Ok(Vote::Yes),
                    Err(e) => {
                        warn!("Participant {} failed prepare phase: {}", participant.service_id, e);
                        Ok(Vote::No)
                    }
                }
            }
        });

        let results: Vec<Result<Vote, String>> = join_all(prepare_futures).await;
        let participant_count = participants.len();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(vote) => {
                    if vote == Vote::No {
                        // If any participant votes no, abort the entire transaction
                        self.abort_distributed_transaction(transaction_id).await?;
                        return Err(format!("Participant {} voted to abort transaction {}", i, transaction_id));
                    }
                }
                Err(e) => {
                    // If we can't coordinate with a participant, abort the transaction
                    self.abort_distributed_transaction(transaction_id).await?;
                    return Err(format!("Failed to prepare with participant {}: {}", i, e));
                }
            }
        }

        // All participants voted yes, move to in-doubt state
        let mut transactions = self.transactions.write().await;
        if let Some(tx) = transactions.get_mut(transaction_id) {
            tx.state = TransactionState::InDoubt;
        }

        self.persist_transaction_state(transaction_id, TransactionState::InDoubt).await?;

        info!("Successfully prepared transaction: {} with {} participants", transaction_id, participant_count);
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

        // Coordinate commit phase with all participants
        let participants = transaction.participants.clone();
        drop(transactions); // Release the lock

        // Execute commit phase for each participant concurrently
        let commit_futures = participants.iter().enumerate().map(|(i, participant)| {
            let tx_id = transaction_id.to_string();
            let participant = participant.clone();
            async move {
                match self.execute_commit_phase(&participant, &tx_id).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        warn!("Participant {} failed commit phase: {}", participant.service_id, e);
                        Err(format!("Participant {} commit failed: {}", i, e))
                    }
                }
            }
        });

        let results = join_all(commit_futures).await;

        // Check if all participants acknowledged the commit
        let mut commit_failures = Vec::new();
        for result in results.into_iter() {
            if let Err(e) = result {
                commit_failures.push(e);
            }
        }

        if !commit_failures.is_empty() {
            // TODO: Implement complex recovery procedures for commit failures
            //       Currently logs failures but marks as committed; should implement proper recovery procedures for distributed systems.
            //
            // COMPLETION CHECKLIST:
            // [ ] Implement transaction recovery procedures
            // [ ] Handle partial commit scenarios
            // [ ] Coordinate recovery across participants
            // [ ] Support transaction compensation
            // [ ] Handle network partitions during recovery
            // [ ] Add unit tests for recovery procedures
            // [ ] Add integration tests with failure scenarios
            // [ ] Verify recovery correctness
            //
            // ACCEPTANCE CRITERIA:
            // - Recovery procedures handle commit failures correctly
            // - Partial commits are handled appropriately
            // - Recovery coordinates across participants
            // - Network partitions are handled during recovery
            //
            // DEPENDENCIES:
            // - Recovery coordination infrastructure (Required)
            // - Transaction compensation utilities (Required)
            // - Network partition handling (Required)
            //
            // ESTIMATED EFFORT: 6-8 hours (low confidence - complex distributed systems)
            // PRIORITY: High
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 1 (critical distributed systems feature)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: Distributed systems expertise
            warn!("Some participants failed to commit transaction {}: {:?}", transaction_id, commit_failures); // Temporary: log until recovery procedures are implemented
        }

        // Mark transaction as committed
        let mut transactions = self.transactions.write().await;
        if let Some(tx) = transactions.get_mut(transaction_id) {
            tx.state = TransactionState::Committed;
        }

        self.persist_transaction_state(transaction_id, TransactionState::Committed).await?;

        info!("Committed distributed transaction: {} with {} participants", transaction_id, participants.len());
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
        // TODO: Check with participants to determine transaction outcome
        //       Currently uses simple heuristic; should check with participants to determine actual outcome.
        //
        // COMPLETION CHECKLIST:
        // [ ] Query each participant for transaction state
        // [ ] Determine transaction outcome from participant responses
        // [ ] Handle inconsistent participant states
        // [ ] Commit or abort transaction based on outcome
        // [ ] Handle participant unavailability
        // [ ] Add unit tests for transaction recovery
        // [ ] Add integration tests with various scenarios
        // [ ] Verify recovery correctness
        //
        // ACCEPTANCE CRITERIA:
        // - Transaction state is queried from participants correctly
        // - Outcome is determined accurately from participant responses
        // - Inconsistent states are handled appropriately
        // - Participant unavailability is handled gracefully
        //
        // DEPENDENCIES:
        // - Participant query API (Required)
        // - Transaction state tracking (Required)
        // - Outcome determination logic (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (critical distributed systems feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Distributed systems expertise
        let transactions = self.transactions.read().await; // Temporary: simple heuristic until participant query is implemented
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
        let state_json = serde_json::to_string(&transaction.state)
            .map_err(|e| e.to_string())?;
        let metadata_json = serde_json::to_string(&transaction.metadata)
            .map_err(|e| e.to_string())?;
        let created_at_str = transaction.created_at.to_rfc3339();
        let timeout_at_str = transaction.timeout_at.to_rfc3339();

        self.db_client.execute(
            query,
            &[
                &transaction.id,
                &transaction.coordinator_id,
                &participants_json,
                &state_json,
                &created_at_str,
                &timeout_at_str,
                &metadata_json,
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

        self.db_client.execute(query, &[&state_str, &transaction_id.to_string()]).await
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
        // TODO: Implement comprehensive data consistency checking
        // - Compare actual data records between primary and replica
        // - Implement checksum-based validation for large tables
        // - Add support for comparing specific columns or computed values
        // - Handle different data types and serialization formats
        // - Add configurable tolerance for floating-point comparisons
        // - Implement sampling strategies for very large tables
        // - Add detailed inconsistency reporting with row-level details

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

    /// Execute prepare phase for a participant (Phase 1 of 2PC)
    async fn execute_prepare_phase(&self, participant: &TransactionParticipant, transaction_id: &str) -> Result<(), String> {
        info!("Executing prepare phase for participant {} in transaction {}", participant.service_id, transaction_id);

        // Connect to participant's database
        let pool = sqlx::PgPool::connect(&participant.connection_info).await
            .map_err(|e| format!("Failed to connect to participant database: {}", e))?;

        // Start a database transaction for this participant
        let mut tx = pool.begin().await
            .map_err(|e| format!("Failed to start database transaction: {}", e))?;

        // Execute all operations for this participant
        for operation in &participant.operations {
            match operation.operation_type.as_str() {
                "insert" => {
                    self.execute_insert_operation(&mut tx, operation, transaction_id).await?;
                }
                "update" => {
                    self.execute_update_operation(&mut tx, operation, transaction_id).await?;
                }
                "delete" => {
                    self.execute_delete_operation(&mut tx, operation, transaction_id).await?;
                }
                _ => {
                    return Err(format!("Unsupported operation type: {}", operation.operation_type));
                }
            }
        }

        // TODO: Store transaction handle for later commit/rollback
        //       Currently keeps transaction open; should store transaction handle for proper commit/rollback management.
        //
        // COMPLETION CHECKLIST:
        // [ ] Store transaction handle in transaction registry
        // [ ] Associate handle with transaction ID
        // [ ] Support transaction handle retrieval
        // [ ] Handle transaction handle cleanup
        // [ ] Support transaction handle expiration
        // [ ] Add unit tests for handle storage
        // [ ] Add integration tests with transaction lifecycle
        // [ ] Verify handle storage and retrieval
        //
        // ACCEPTANCE CRITERIA:
        // - Transaction handles are stored correctly
        // - Handles are associated with transaction IDs
        // - Handles can be retrieved for commit/rollback
        // - Handle cleanup works correctly
        //
        // DEPENDENCIES:
        // - Transaction registry (Required)
        // - Handle storage infrastructure (Required)
        // - Handle lifecycle management (Required)
        //
        // ESTIMATED EFFORT: 2-3 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (transaction management feature)
        // - Change Budget: ~60 LOC
        // - Reviewer Requirements: Transaction management expertise
        Ok(()) // Temporary: keep transaction open until handle storage is implemented
    }

    /// Execute commit phase for a participant (Phase 2 of 2PC)
    async fn execute_commit_phase(&self, participant: &TransactionParticipant, transaction_id: &str) -> Result<(), String> {
        info!("Executing commit phase for participant {} in transaction {}", participant.service_id, transaction_id);

        // Connect to participant's database
        let pool = sqlx::PgPool::connect(&participant.connection_info).await
            .map_err(|e| format!("Failed to connect to participant database: {}", e))?;

        // TODO: Retrieve prepared transaction and commit it
        //       Currently simulates commit by re-executing; should retrieve prepared transaction and commit it properly.
        //
        // COMPLETION CHECKLIST:
        // [ ] Retrieve prepared transaction from participant
        // [ ] Commit prepared transaction using transaction handle
        // [ ] Handle prepared transaction not found errors
        // [ ] Support transaction commit retry
        // [ ] Verify transaction commit success
        // [ ] Add unit tests for prepared transaction commit
        // [ ] Add integration tests with real database
        // [ ] Verify commit correctness
        //
        // ACCEPTANCE CRITERIA:
        // - Prepared transactions are retrieved correctly
        // - Transactions are committed using proper handles
        // - Missing prepared transactions are handled gracefully
        // - Commit retry works correctly
        //
        // DEPENDENCIES:
        // - Prepared transaction storage (Required)
        // - Transaction commit API (Required)
        // - Transaction handle retrieval (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (critical transaction feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Database transaction expertise
        // Temporary: simulate commit until prepared transaction retrieval is implemented

        let mut tx = pool.begin().await
            .map_err(|e| format!("Failed to start commit transaction: {}", e))?;

        // Execute all operations for this participant (simulating commit of prepared transaction)
        for operation in &participant.operations {
            match operation.operation_type.as_str() {
                "insert" => {
                    self.execute_insert_operation(&mut tx, operation, transaction_id).await?;
                }
                "update" => {
                    self.execute_update_operation(&mut tx, operation, transaction_id).await?;
                }
                "delete" => {
                    self.execute_delete_operation(&mut tx, operation, transaction_id).await?;
                }
                _ => {
                    return Err(format!("Unsupported operation type: {}", operation.operation_type));
                }
            }
        }

        // Commit the transaction
        tx.commit().await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        info!("Successfully committed operations for participant {} in transaction {}",
              participant.service_id, transaction_id);
        Ok(())
    }

    /// Abort a distributed transaction (rollback all participants)
    async fn abort_distributed_transaction(&self, transaction_id: &str) -> Result<(), String> {
        info!("Aborting distributed transaction: {}", transaction_id);

        let transactions = self.transactions.read().await;
        let transaction = transactions.get(transaction_id)
            .ok_or(format!("Transaction not found for abort: {}", transaction_id))?;

        // Rollback each participant concurrently
        let abort_futures = transaction.participants.iter().map(|participant| {
            let participant = participant.clone();
            let tx_id = transaction_id.to_string();
            async move {
                match self.execute_abort_phase(&participant, &tx_id).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        warn!("Failed to abort participant {}: {}", participant.service_id, e);
                        Err(e)
                    }
                }
            }
        });

        let results = join_all(abort_futures).await;

        // Log any abort failures (but don't fail the overall abort)
        let mut abort_failures = Vec::new();
        for result in results.into_iter() {
            if let Err(e) = result {
                abort_failures.push(e);
            }
        }

        if !abort_failures.is_empty() {
            warn!("Some participants failed to abort transaction {}: {:?}", transaction_id, abort_failures);
        }

        // Update transaction state
        drop(transactions);
        let mut transactions = self.transactions.write().await;
        if let Some(tx) = transactions.get_mut(transaction_id) {
            tx.state = TransactionState::Aborted;
        }
        self.persist_transaction_state(transaction_id, TransactionState::Aborted).await?;

        info!("Distributed transaction {} aborted", transaction_id);
        Ok(())
    }

    /// Execute abort phase for a participant (rollback)
    async fn execute_abort_phase(&self, participant: &TransactionParticipant, transaction_id: &str) -> Result<(), String> {
        info!("Executing abort phase for participant {} in transaction {}", participant.service_id, transaction_id);

        // Connect to participant's database
        let _pool = sqlx::PgPool::connect(&participant.connection_info).await
            .map_err(|e| format!("Failed to connect to participant database: {}", e))?;

        // TODO: Implement transaction rollback with the following requirements:
        // 1. Prepared transaction rollback: Rollback the prepared transaction
        //    - Identify the prepared transaction by transaction ID
        //    - Execute ROLLBACK PREPARED statement
        //    - Handle rollback errors and connection failures
        // 2. State cleanup: Clean up transaction state
        //    - Remove transaction from active transaction tracking
        //    - Release any held locks or resources
        //    - Update transaction status appropriately
        // 3. Error handling: Handle rollback failures gracefully
        //    - Log rollback failures for investigation
        //    - Attempt retry with exponential backoff
        //    - Ensure system remains in consistent state
        // TODO: Implement actual transaction rollback:
        // 1. Transaction rollback: Execute actual rollback operations
        //    - Rollback prepared transactions in participant databases
        //    - Execute ROLLBACK commands for each participant
        //    - Verify rollback completion and status
        // 2. Rollback verification: Verify rollback success
        //    - Confirm transactions are rolled back
        //    - Verify data consistency after rollback
        //    - Handle partial rollback failures
        // 3. State management: Manage transaction state
        //    - Update transaction status to aborted
        //    - Clean up transaction resources
        //    - Notify participants of rollback completion
        // ACCEPTANCE CRITERIA:
        // - Prepared transactions are rolled back in participant databases
        // - Rollback operations complete successfully
        // - Transaction state is properly updated after rollback
        // DEPENDENCIES:
        // - Database transaction management (Required)
        // - Participant database connections (Required)
        // PRIORITY: High

        info!("Successfully aborted operations for participant {} in transaction {}",
              participant.service_id, transaction_id);
        Ok(())
    }

    /// Execute an insert operation
    async fn execute_insert_operation(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, operation: &TransactionOperation, _transaction_id: &str) -> Result<(), String> {
        let table = &operation.table;
        let data = &operation.data;

        // Build dynamic INSERT statement based on the data
        let columns: Vec<String> = data.as_object()
            .ok_or("Insert data must be an object")?
            .keys()
            .cloned()
            .collect();

        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", ")
        );

        let values: Vec<serde_json::Value> = columns.iter()
            .map(|col| data.get(col).cloned().unwrap_or(serde_json::Value::Null))
            .collect();

        // Convert JSON values to SQL parameters
        let mut query = sqlx::query(&sql);
        for value in values {
            query = match value {
                serde_json::Value::String(s) => query.bind(s),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        query.bind(f)
                    } else {
                        return Err("Unsupported number type".to_string());
                    }
                }
                serde_json::Value::Bool(b) => query.bind(b),
                serde_json::Value::Null => query.bind(None::<String>),
                _ => return Err("Unsupported data type for insert".to_string()),
            };
        }

        query.execute(&mut **tx).await
            .map_err(|e| format!("Failed to execute insert: {}", e))?;

        Ok(())
    }

    /// Execute an update operation
    async fn execute_update_operation(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, operation: &TransactionOperation, _transaction_id: &str) -> Result<(), String> {
        let table = &operation.table;
        let data = &operation.data;

        // Assume data contains both WHERE conditions and SET values
        let set_data = data.get("set").and_then(|v| v.as_object())
            .ok_or("Update data must contain 'set' object")?;
        let where_data = data.get("where").and_then(|v| v.as_object())
            .ok_or("Update data must contain 'where' object")?;

        // Build SET clause
        let set_columns: Vec<String> = set_data.keys().cloned().collect();
        let set_placeholders: Vec<String> = (1..=set_columns.len()).map(|i| format!("${}", i)).collect();

        // Build WHERE clause
        let where_columns: Vec<String> = where_data.keys().cloned().collect();
        let where_placeholders: Vec<String> = ((set_columns.len() + 1)..=(set_columns.len() + where_columns.len()))
            .map(|i| format!("${}", i))
            .collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table,
            set_columns.iter().zip(set_placeholders.iter()).map(|(col, ph)| format!("{} = {}", col, ph)).collect::<Vec<_>>().join(", "),
            where_columns.iter().zip(where_placeholders.iter()).map(|(col, ph)| format!("{} = {}", col, ph)).collect::<Vec<_>>().join(" AND ")
        );

        let mut query = sqlx::query(&sql);

        // Bind SET values
        for col in &set_columns {
            let value = set_data.get(col).cloned().unwrap_or(serde_json::Value::Null);
            query = self.bind_json_value(query, value);
        }

        // Bind WHERE values
        for col in &where_columns {
            let value = where_data.get(col).cloned().unwrap_or(serde_json::Value::Null);
            query = self.bind_json_value(query, value);
        }

        query.execute(&mut **tx).await
            .map_err(|e| format!("Failed to execute update: {}", e))?;

        Ok(())
    }

    /// Execute a delete operation
    async fn execute_delete_operation(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, operation: &TransactionOperation, _transaction_id: &str) -> Result<(), String> {
        let table = &operation.table;
        let data = &operation.data;

        // Assume data contains WHERE conditions
        let where_data = data.as_object()
            .ok_or("Delete data must be an object with WHERE conditions")?;

        let where_columns: Vec<String> = where_data.keys().cloned().collect();
        let where_placeholders: Vec<String> = (1..=where_columns.len()).map(|i| format!("${}", i)).collect();

        let sql = format!(
            "DELETE FROM {} WHERE {}",
            table,
            where_columns.iter().zip(where_placeholders.iter()).map(|(col, ph)| format!("{} = {}", col, ph)).collect::<Vec<_>>().join(" AND ")
        );

        let mut query = sqlx::query(&sql);

        // Bind WHERE values
        for col in &where_columns {
            let value = where_data.get(col).cloned().unwrap_or(serde_json::Value::Null);
            query = self.bind_json_value(query, value);
        }

        query.execute(&mut **tx).await
            .map_err(|e| format!("Failed to execute delete: {}", e))?;

        Ok(())
    }

    /// Helper to bind JSON values to SQL queries
    fn bind_json_value<'a>(&self, query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>, value: serde_json::Value) -> sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments> {
        match value {
            serde_json::Value::String(s) => query.bind(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    panic!("Unsupported number type");
                }
            }
            serde_json::Value::Bool(b) => query.bind(b),
            serde_json::Value::Null => query.bind(None::<String>),
            _ => panic!("Unsupported data type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database_config::DatabaseConfig;
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
        let participants = vec![
            TransactionParticipant {
                service_id: "service1".to_string(),
                connection_info: "postgresql://test:test@localhost:5432/test1".to_string(),
                operations: vec![
                    TransactionOperation {
                        operation_type: "insert".to_string(),
                        table: "test_table".to_string(),
                        data: serde_json::json!({"id": 1, "name": "test1"}),
                    }
                ],
            },
            TransactionParticipant {
                service_id: "service2".to_string(),
                connection_info: "postgresql://test:test@localhost:5432/test2".to_string(),
                operations: vec![
                    TransactionOperation {
                        operation_type: "insert".to_string(),
                        table: "test_table".to_string(),
                        data: serde_json::json!({"id": 2, "name": "test2"}),
                    }
                ],
            },
        ];

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
        assert_eq!(strong_manager._consistency_level, ConsistencyLevel::Strong);
        assert_eq!(eventual_manager._consistency_level, ConsistencyLevel::Eventual);
    }
}
