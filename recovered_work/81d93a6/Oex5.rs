//! Budget Checker for CAWS Compliance
//!
//! Enforces budget limits at the tool boundary. Computes budgets
//! at apply time, not based on upstream estimates. Blocks operations
//! that would exceed max_files/max_loc limits.
//!
//! @author @darianrosebrook

use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;
use chrono::{DateTime, Utc};

use crate::types::{ChangeSet, ChangeSetReceipt, ChangeOperation, FileChange};

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("Budget calculation failed: {0}")]
    CalculationError(String),

    #[error("Invalid budget limits: {0}")]
    InvalidLimits(String),
}

/// Current budget state
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetState {
    pub files_used: usize,
    pub loc_used: i64, // Can be negative for deletions
    pub last_updated: DateTime<Utc>,
}

/// Budget limits for a task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub max_files: usize,
    pub max_loc: usize,
}

/// Budget checker that enforces limits at apply time
///
/// **INVARIANT**: Only called from WorkspaceManager.apply_changes()
/// **INVARIANT**: Budgets computed from actual changeset, not estimates
/// **INVARIANT**: Blocks operations that would exceed limits
pub struct BudgetChecker {
    limits: BudgetLimits,
    current: BudgetState,
    tracked_files: HashSet<PathBuf>,
}

impl BudgetChecker {
    /// Create a new budget checker with specified limits
    pub fn new(max_files: usize, max_loc: usize) -> Self {
        Self {
            limits: BudgetLimits { max_files, max_loc },
            current: BudgetState {
                files_used: 0,
                loc_used: 0,
                last_updated: Utc::now(),
            },
            tracked_files: HashSet::new(),
        }
    }

    /// Get current budget state
    pub fn current_state(&self) -> Result<BudgetState, BudgetError> {
        Ok(self.current.clone())
    }

    /// Get budget limits
    pub fn limits(&self) -> BudgetLimits {
        self.limits.clone()
    }

    /// Check if changeset would exceed budget limits
    ///
    /// **IMPORTANT**: This computes the actual impact, not estimates.
    /// Called at apply time to ensure authoritative enforcement.
    pub fn would_exceed(&self, changeset: &ChangeSet) -> Result<bool, BudgetError> {
        let projected = self.projected_state(changeset)?;

        let exceeds_files = projected.files_used > self.limits.max_files;
        let exceeds_loc = projected.loc_used > self.limits.max_loc as i64;

        if exceeds_files || exceeds_loc {
            return Ok(true);
        }

        // Warn at 80% threshold
        let files_pct = (projected.files_used as f64 / self.limits.max_files as f64) * 100.0;
        let loc_pct = (projected.loc_used as f64 / self.limits.max_loc as f64) * 100.0;

        if files_pct >= 80.0 || loc_pct >= 80.0 {
            // Emit warning event (would integrate with observability system)
            eprintln!("WARNING: Approaching budget limits - Files: {:.1}%, LOC: {:.1}%",
                     files_pct, loc_pct);
        }

        Ok(false)
    }

    /// Get projected state if changeset were applied
    pub fn projected_state(&self, changeset: &ChangeSet) -> Result<BudgetState, BudgetError> {
        let mut projected_files = self.tracked_files.clone();
        let mut projected_loc = self.current.loc_used;

        // Calculate impact of changeset
        for change in &changeset.changes {
            match &change.operation {
                ChangeOperation::Create { content } => {
                    // New file
                    if !projected_files.contains(&change.path) {
                        projected_files.insert(change.path.clone());
                    }
                    projected_loc += content.lines().count() as i64;
                }
                ChangeOperation::Modify { expected_content, new_content } => {
                    // Existing file modification - calculate LOC delta
                    projected_files.insert(change.path.clone());
                    let old_lines = expected_content.lines().count() as i64;
                    let new_lines = new_content.lines().count() as i64;
                    projected_loc += new_lines - old_lines; // Actual LOC delta
                }
                ChangeOperation::Delete { .. } => {
                    // File deletion (remove from tracked set)
                    projected_files.remove(&change.path);
                    // LOC impact: we don't know how many lines were removed
                    // Conservative approach: don't reduce LOC count
                }
            }
        }

        Ok(BudgetState {
            files_used: projected_files.len(),
            loc_used: projected_loc,
            last_updated: Utc::now(),
        })
    }

    /// Record that a changeset was successfully applied
    ///
    /// Updates internal tracking state. Only called after successful apply.
    pub fn record_changes(&mut self, receipt: &ChangeSetReceipt) -> Result<(), BudgetError> {
        // Update tracked files and LOC based on receipt
        self.current.files_used = receipt.files_changed;

        // Update LOC delta (can be negative for deletions)
        self.current.loc_used += receipt.loc_delta;

        self.current.last_updated = receipt.applied_at;

        Ok(())
    }

    /// Get utilization percentages
    pub fn utilization(&self) -> (f64, f64) { // (files_pct, loc_pct)
        let files_pct = if self.limits.max_files > 0 {
            (self.current.files_used as f64 / self.limits.max_files as f64) * 100.0
        } else {
            0.0
        };

        let loc_pct = if self.limits.max_loc > 0 {
            ((self.current.loc_used as f64 / self.limits.max_loc as f64) * 100.0).max(0.0)
        } else {
            0.0
        };

        (files_pct, loc_pct)
    }

    /// Check if we're at or over any limit
    pub fn is_over_limit(&self) -> bool {
        self.current.files_used > self.limits.max_files ||
        self.current.loc_used > self.limits.max_loc as i64
    }

    /// Restore budget checker to a previous state
    pub fn restore_state(&mut self, state: BudgetState) -> Result<(), BudgetError> {
        self.current = state;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChangeSet;

    #[test]
    fn test_budget_checker_creation() {
        let checker = BudgetChecker::new(10, 1000);
        assert_eq!(checker.limits.max_files, 10);
        assert_eq!(checker.limits.max_loc, 1000);
        assert_eq!(checker.current.files_used, 0);
        assert_eq!(checker.current.loc_used, 0);
    }

    #[test]
    fn test_within_budget() {
        let checker = BudgetChecker::new(5, 100);

        let changeset = ChangeSet::new(vec![
            FileChange {
                path: PathBuf::from("src/main.rs"),
                operation: ChangeOperation::Create {
                    content: "fn main() {}\n".to_string(),
                },
            }
        ], "Test creation".to_string());

        assert!(!checker.would_exceed(&changeset).unwrap());
    }

    #[test]
    fn test_exceeds_file_budget() {
        let mut checker = BudgetChecker::new(1, 100); // Only 1 file allowed

        // First file should be OK
        let changeset1 = ChangeSet::new(vec![
            FileChange {
                path: PathBuf::from("src/main.rs"),
                operation: ChangeOperation::Create {
                    content: "fn main() {}\n".to_string(),
                },
            }
        ], "First file".to_string());

        assert!(!checker.would_exceed(&changeset1).unwrap());

        // Record the first file
        let receipt1 = ChangeSetReceipt {
            changeset_id: changeset1.id,
            applied_at: Utc::now(),
            files_changed: 1,
            loc_delta: 1,
            sha256_tree: "dummy".to_string(),
            checkpoint_id: "test".to_string(),
        };
        checker.record_changes(&receipt1).unwrap();

        // Second file should exceed budget
        let changeset2 = ChangeSet::new(vec![
            FileChange {
                path: PathBuf::from("src/lib.rs"),
                operation: ChangeOperation::Create {
                    content: "pub fn test() {}\n".to_string(),
                },
            }
        ], "Second file".to_string());

        assert!(checker.would_exceed(&changeset2).unwrap());
    }

    #[test]
    fn test_exceeds_loc_budget() {
        let checker = BudgetChecker::new(10, 5); // Only 5 LOC allowed

        let changeset = ChangeSet::new(vec![
            FileChange {
                path: PathBuf::from("src/main.rs"),
                operation: ChangeOperation::Create {
                    content: "line1\nline2\nline3\nline4\nline5\nline6\n".to_string(), // 6 lines
                },
            }
        ], "Large file".to_string());

        assert!(checker.would_exceed(&changeset).unwrap());
    }

    #[test]
    fn test_utilization_calculation() {
        let mut checker = BudgetChecker::new(10, 100);

        let changeset = ChangeSet::new(vec![
            FileChange {
                path: PathBuf::from("src/main.rs"),
                operation: ChangeOperation::Create {
                    content: "line1\nline2\nline3\nline4\nline5\n".to_string(), // 5 lines
                },
            }
        ], "Test file".to_string());

        // Record the changeset
        let receipt = ChangeSetReceipt {
            changeset_id: changeset.id,
            applied_at: Utc::now(),
            files_changed: 1,
            loc_delta: 5,
            sha256_tree: "dummy".to_string(),
            checkpoint_id: "test".to_string(),
        };
        checker.record_changes(&receipt).unwrap();

        let (files_pct, loc_pct) = checker.utilization();
        assert_eq!(files_pct, 10.0); // 1/10 = 10%
        assert_eq!(loc_pct, 5.0);   // 5/100 = 5%
    }

    #[test]
    fn test_budget_warning_threshold() {
        let checker = BudgetChecker::new(10, 100);

        let changeset = ChangeSet::new(vec![
            FileChange {
                path: PathBuf::from("src/main.rs"),
                operation: ChangeOperation::Create {
                    content: "x\n".repeat(80), // 80 lines = 80% of budget
                },
            }
        ], "Large file".to_string());

        // Should not exceed but should trigger warning
        assert!(!checker.would_exceed(&changeset).unwrap());
        // Warning would be printed to stderr (tested manually)
    }
}