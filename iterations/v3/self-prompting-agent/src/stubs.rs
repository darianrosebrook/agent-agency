//! Temporary stubs for file_ops types to enable compilation testing
//!
//! These will be removed once file_ops dependency issues are resolved

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Stub ChangeSetId
pub type ChangeSetId = String;

/// Stub ChangeSet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub patches: Vec<Patch>,
}

/// Stub Patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub path: String,
    pub hunks: Vec<Hunk>,
    pub expected_prev_sha256: Option<String>,
}

/// Stub Hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: String,
}

/// Stub Workspace trait
#[async_trait::async_trait]
pub trait Workspace: Send + Sync {
    async fn apply_changeset(&mut self, changeset: ChangeSet) -> Result<(), FileOpsError>;
    async fn create_checkpoint(&mut self) -> Result<String, FileOpsError>;
}

/// Stub WorkspaceFactory
pub struct WorkspaceFactory;

/// Stub AllowList
pub type AllowList = Vec<String>;

/// Stub Budgets
#[derive(Debug, Clone)]
pub struct Budgets {
    pub max_files: usize,
    pub max_loc: usize,
}

/// Stub FileOpsError
#[derive(Debug, thiserror::Error)]
pub enum FileOpsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),

    #[error("Blocked: {0}")]
    Blocked(String),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl WorkspaceFactory {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_workspace(&self, _root: &std::path::Path, _allow_list: AllowList, _budgets: Budgets) -> Result<Box<dyn Workspace>, FileOpsError> {
        // Stub implementation - always succeeds
        Ok(Box::new(StubWorkspace))
    }
}

struct StubWorkspace;

#[async_trait::async_trait]
impl Workspace for StubWorkspace {
    async fn apply_changeset(&mut self, _changeset: ChangeSet) -> Result<(), FileOpsError> {
        Ok(())
    }

    async fn create_checkpoint(&mut self) -> Result<String, FileOpsError> {
        Ok("stub-checkpoint".to_string())
    }
}
