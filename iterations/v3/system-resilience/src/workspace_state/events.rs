//! Workspace state events for unified workspace state management
//!
//! Provides event types for file watching, state capture, embedding generation,
//! and context generation operations.

use super::state_types::StateId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Workspace state events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceStateEvent {
    /// File was created
    FileCreated {
        path: PathBuf,
        state_id: Option<StateId>,
    },
    
    /// File was modified
    FileModified {
        path: PathBuf,
        state_id: Option<StateId>,
    },
    
    /// File was deleted
    FileDeleted {
        path: PathBuf,
        state_id: Option<StateId>,
    },
    
    /// State was captured
    StateCaptured {
        state_id: StateId,
        duration_ms: u64,
    },
    
    /// Embedding was generated
    EmbeddingGenerated {
        path: PathBuf,
        success: bool,
        duration_ms: u64,
    },
    
    /// Context was generated
    ContextGenerated {
        context_type: ContextType,
        files_selected: usize,
        duration_ms: u64,
    },
}

/// Context type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextType {
    Code,
    Documentation,
    Config,
    General,
}

impl std::fmt::Display for ContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextType::Code => write!(f, "code"),
            ContextType::Documentation => write!(f, "documentation"),
            ContextType::Config => write!(f, "config"),
            ContextType::General => write!(f, "general"),
        }
    }
}

