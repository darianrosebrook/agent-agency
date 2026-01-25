//! Provenance Chain for Audit Trail
//!
//! Implements INV-CORE-05: Provenance Required - Every decision must have
//! an auditable provenance chain.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single link in the provenance chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    /// Unique identifier for this entry
    pub id: String,
    /// Type of action taken
    pub action_type: ActionType,
    /// Human-readable description
    pub description: String,
    /// Actor (agent/judge/system) that performed the action
    pub actor: String,
    /// Input hash (for verification)
    pub input_hash: String,
    /// Output hash (for verification)
    pub output_hash: String,
    /// Timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Hash of the previous entry (for chain integrity)
    pub previous_hash: Option<String>,
}

impl ProvenanceEntry {
    /// Compute the hash of this entry
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(format!("{:?}", self.action_type).as_bytes());
        hasher.update(self.description.as_bytes());
        hasher.update(self.actor.as_bytes());
        hasher.update(self.input_hash.as_bytes());
        hasher.update(self.output_hash.as_bytes());
        hasher.update(self.timestamp.timestamp().to_le_bytes());
        if let Some(ref prev) = self.previous_hash {
            hasher.update(prev.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

/// Types of actions in the provenance chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// Task received for processing
    TaskReceived,
    /// Operator proposed
    OperatorProposed,
    /// Operator validated
    OperatorValidated,
    /// Graph constructed
    GraphConstructed,
    /// Rule applied
    RuleApplied,
    /// Council review requested
    CouncilReviewRequested,
    /// Judge evaluation
    JudgeEvaluation,
    /// Verdict aggregated
    VerdictAggregated,
    /// Execution authorized
    ExecutionAuthorized,
    /// Execution denied
    ExecutionDenied,
    /// Certificate generated
    CertificateGenerated,
}

/// Full provenance chain for a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// Unique chain identifier
    pub chain_id: String,
    /// Task this chain is for
    pub task_id: String,
    /// Ordered list of entries
    entries: Vec<ProvenanceEntry>,
    /// Hash of the entire chain (computed on demand)
    #[serde(skip)]
    chain_hash: Option<String>,
}

impl ProvenanceChain {
    /// Create a new provenance chain for a task
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            chain_id: uuid::Uuid::new_v4().to_string(),
            task_id: task_id.into(),
            entries: Vec::new(),
            chain_hash: None,
        }
    }

    /// Add an entry to the chain
    pub fn add_entry(
        &mut self,
        action_type: ActionType,
        description: impl Into<String>,
        actor: impl Into<String>,
        input_hash: impl Into<String>,
        output_hash: impl Into<String>,
    ) {
        let previous_hash = self.entries.last().map(|e| e.compute_hash());

        let entry = ProvenanceEntry {
            id: uuid::Uuid::new_v4().to_string(),
            action_type,
            description: description.into(),
            actor: actor.into(),
            input_hash: input_hash.into(),
            output_hash: output_hash.into(),
            timestamp: chrono::Utc::now(),
            previous_hash,
        };

        self.entries.push(entry);
        self.chain_hash = None; // Invalidate cached hash
    }

    /// Get all entries in the chain
    pub fn entries(&self) -> &[ProvenanceEntry] {
        &self.entries
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Compute the hash of the entire chain
    pub fn compute_chain_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.as_bytes());
        hasher.update(self.task_id.as_bytes());
        for entry in &self.entries {
            hasher.update(entry.compute_hash().as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    /// Verify the chain integrity
    pub fn verify_integrity(&self) -> Result<(), ProvenanceError> {
        if self.entries.is_empty() {
            return Ok(());
        }

        // First entry should have no previous hash
        if self.entries[0].previous_hash.is_some() {
            return Err(ProvenanceError::InvalidChain(
                "First entry should not have previous hash".to_string(),
            ));
        }

        // Each subsequent entry should reference the previous entry's hash
        for i in 1..self.entries.len() {
            let expected_prev_hash = self.entries[i - 1].compute_hash();
            match &self.entries[i].previous_hash {
                Some(hash) if hash == &expected_prev_hash => {}
                Some(hash) => {
                    return Err(ProvenanceError::HashMismatch {
                        entry_index: i,
                        expected: expected_prev_hash,
                        found: hash.clone(),
                    });
                }
                None => {
                    return Err(ProvenanceError::InvalidChain(format!(
                        "Entry {} missing previous hash",
                        i
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get the last entry in the chain
    pub fn last_entry(&self) -> Option<&ProvenanceEntry> {
        self.entries.last()
    }

    /// Create a summary of the chain
    pub fn summary(&self) -> ProvenanceSummary {
        ProvenanceSummary {
            chain_id: self.chain_id.clone(),
            task_id: self.task_id.clone(),
            entry_count: self.entries.len(),
            chain_hash: self.compute_chain_hash(),
            first_action: self.entries.first().map(|e| e.action_type),
            last_action: self.entries.last().map(|e| e.action_type),
            actors: self
                .entries
                .iter()
                .map(|e| e.actor.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        }
    }
}

/// Summary of a provenance chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceSummary {
    /// Chain identifier
    pub chain_id: String,
    /// Task identifier
    pub task_id: String,
    /// Number of entries
    pub entry_count: usize,
    /// Hash of the entire chain
    pub chain_hash: String,
    /// First action in the chain
    pub first_action: Option<ActionType>,
    /// Last action in the chain
    pub last_action: Option<ActionType>,
    /// All actors involved
    pub actors: Vec<String>,
}

/// Errors related to provenance
#[derive(Debug, thiserror::Error)]
pub enum ProvenanceError {
    #[error("Invalid chain: {0}")]
    InvalidChain(String),

    #[error("Hash mismatch at entry {entry_index}: expected {expected}, found {found}")]
    HashMismatch {
        entry_index: usize,
        expected: String,
        found: String,
    },

    #[error("Missing provenance for decision")]
    MissingProvenance,
}

/// Compute SHA-256 hash of arbitrary content
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_chain_creation() {
        let chain = ProvenanceChain::new("task-123");
        assert_eq!(chain.task_id, "task-123");
        assert!(chain.is_empty());
    }

    #[test]
    fn test_add_entries() {
        let mut chain = ProvenanceChain::new("task-123");

        chain.add_entry(
            ActionType::TaskReceived,
            "Task received for processing",
            "system",
            "input-hash-1",
            "output-hash-1",
        );

        assert_eq!(chain.len(), 1);
        assert!(chain.entries[0].previous_hash.is_none());

        chain.add_entry(
            ActionType::OperatorProposed,
            "Proposed Seek operator",
            "symbolic-reasoner",
            "output-hash-1",
            "output-hash-2",
        );

        assert_eq!(chain.len(), 2);
        assert!(chain.entries[1].previous_hash.is_some());
    }

    #[test]
    fn test_chain_integrity() {
        let mut chain = ProvenanceChain::new("task-123");

        chain.add_entry(
            ActionType::TaskReceived,
            "Task received",
            "system",
            "h1",
            "h2",
        );
        chain.add_entry(
            ActionType::OperatorProposed,
            "Proposed operators",
            "reasoner",
            "h2",
            "h3",
        );

        assert!(chain.verify_integrity().is_ok());
    }

    #[test]
    fn test_chain_hash_determinism() {
        let mut chain1 = ProvenanceChain::new("task-123");
        chain1.chain_id = "fixed-chain-id".to_string();
        chain1.entries.push(ProvenanceEntry {
            id: "entry-1".to_string(),
            action_type: ActionType::TaskReceived,
            description: "Test".to_string(),
            actor: "test".to_string(),
            input_hash: "a".to_string(),
            output_hash: "b".to_string(),
            timestamp: chrono::DateTime::from_timestamp(1000, 0).unwrap(),
            previous_hash: None,
        });

        let mut chain2 = ProvenanceChain::new("task-123");
        chain2.chain_id = "fixed-chain-id".to_string();
        chain2.entries.push(ProvenanceEntry {
            id: "entry-1".to_string(),
            action_type: ActionType::TaskReceived,
            description: "Test".to_string(),
            actor: "test".to_string(),
            input_hash: "a".to_string(),
            output_hash: "b".to_string(),
            timestamp: chrono::DateTime::from_timestamp(1000, 0).unwrap(),
            previous_hash: None,
        });

        assert_eq!(chain1.compute_chain_hash(), chain2.compute_chain_hash());
    }

    #[test]
    fn test_summary() {
        let mut chain = ProvenanceChain::new("task-456");
        chain.add_entry(
            ActionType::TaskReceived,
            "Received",
            "system",
            "h1",
            "h2",
        );
        chain.add_entry(
            ActionType::ExecutionAuthorized,
            "Authorized",
            "arbiter",
            "h2",
            "h3",
        );

        let summary = chain.summary();
        assert_eq!(summary.entry_count, 2);
        assert_eq!(summary.first_action, Some(ActionType::TaskReceived));
        assert_eq!(summary.last_action, Some(ActionType::ExecutionAuthorized));
        assert!(summary.actors.contains(&"system".to_string()));
        assert!(summary.actors.contains(&"arbiter".to_string()));
    }
}
