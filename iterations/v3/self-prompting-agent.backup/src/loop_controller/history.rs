//! History tracking for changesets, failures, and evaluation results

use std::cell::RefCell;
use crate::evaluation::EvaluationFailureType;
use crate::stubs::{ChangeSetId, ChangeSet};
use super::types::PatchFailureType;

/// Tracks changeset history for rollback capabilities
#[derive(Debug)]
pub struct ChangesetHistory {
    /// History of applied changesets
    history: RefCell<Vec<ChangeSetId>>,
    /// Maximum history size
    max_history_size: usize,
}

impl ChangesetHistory {
    /// Create a new changeset history tracker
    pub fn new(max_history_size: usize) -> Self {
        Self {
            history: RefCell::new(Vec::new()),
            max_history_size,
        }
    }

    /// Record a successfully applied changeset
    pub fn record_changeset(&self, changeset_id: ChangeSetId) {
        let mut history = self.history.borrow_mut();
        history.push(changeset_id);

        // Maintain maximum history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    /// Get the last N changesets
    pub fn recent_changesets(&self, count: usize) -> Vec<ChangeSetId> {
        let history = self.history.borrow();
        history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get all changeset history
    pub fn all_changesets(&self) -> Vec<ChangeSetId> {
        self.history.borrow().clone()
    }

    /// Find a changeset by ID
    pub fn find_changeset(&self, id: &ChangeSetId) -> Option<ChangeSetId> {
        self.history.borrow().iter().find(|cs| *cs == id).cloned()
    }

    /// Check if a changeset was already applied
    pub fn was_applied(&self, changeset_id: &ChangeSetId) -> bool {
        self.history.borrow().contains(changeset_id)
    }

    /// Clear history (useful for testing or reset)
    pub fn clear(&self) {
        self.history.borrow_mut().clear();
    }
}

/// Tracks patch application failures for pattern analysis
#[derive(Debug)]
pub struct PatchFailureHistory {
    /// History of patch failures
    history: RefCell<Vec<PatchFailureType>>,
    /// Maximum history size
    max_history_size: usize,
}

impl PatchFailureHistory {
    /// Create a new patch failure history tracker
    pub fn new(max_history_size: usize) -> Self {
        Self {
            history: RefCell::new(Vec::new()),
            max_history_size,
        }
    }

    /// Record a patch failure
    pub fn record_failure(&self, failure_type: PatchFailureType) {
        let mut history = self.history.borrow_mut();
        history.push(failure_type);

        // Maintain maximum history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    /// Get recent failure patterns
    pub fn recent_failures(&self, count: usize) -> Vec<PatchFailureType> {
        let history = self.history.borrow();
        history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get failure frequency by type
    pub fn failure_frequency(&self) -> std::collections::HashMap<PatchFailureType, usize> {
        let mut frequency = std::collections::HashMap::new();
        for failure in self.history.borrow().iter() {
            *frequency.entry(failure.clone()).or_insert(0) += 1;
        }
        frequency
    }

    /// Get most common failure type
    pub fn most_common_failure(&self) -> Option<PatchFailureType> {
        self.failure_frequency()
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(failure_type, _)| failure_type)
    }

    /// Check if recent failures indicate a pattern
    pub fn has_failure_pattern(&self, failure_type: &PatchFailureType, threshold: usize) -> bool {
        let recent = self.recent_failures(10);
        recent.iter().filter(|f| f == failure_type).count() >= threshold
    }

    /// Clear failure history
    pub fn clear(&self) {
        self.history.borrow_mut().clear();
    }
}

/// Tracks evaluation failures for environment recovery
#[derive(Debug)]
pub struct EvaluationFailureHistory {
    /// History of evaluation failures
    history: RefCell<Vec<EvaluationFailureType>>,
    /// Maximum history size
    max_history_size: usize,
}

impl EvaluationFailureHistory {
    /// Create a new evaluation failure history tracker
    pub fn new(max_history_size: usize) -> Self {
        Self {
            history: RefCell::new(Vec::new()),
            max_history_size,
        }
    }

    /// Record an evaluation failure
    pub fn record_failure(&self, failure_type: EvaluationFailureType) {
        let mut history = self.history.borrow_mut();
        history.push(failure_type);

        // Maintain maximum history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    /// Get recent evaluation failures
    pub fn recent_failures(&self, count: usize) -> Vec<EvaluationFailureType> {
        let history = self.history.borrow();
        history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Check if evaluation failures indicate environment issues
    pub fn indicates_environment_issues(&self) -> bool {
        let recent = self.recent_failures(5);
        let env_failures = recent.iter()
            .filter(|f| matches!(f, EvaluationFailureType::EnvironmentIssue))
            .count();
        env_failures >= 3 // 3 or more environment failures in last 5
    }

    /// Get failure frequency
    pub fn failure_frequency(&self) -> std::collections::HashMap<EvaluationFailureType, usize> {
        let mut frequency = std::collections::HashMap::new();
        for failure in self.history.borrow().iter() {
            *frequency.entry(failure.clone()).or_insert(0) += 1;
        }
        frequency
    }

    /// Clear evaluation failure history
    pub fn clear(&self) {
        self.history.borrow_mut().clear();
    }
}
