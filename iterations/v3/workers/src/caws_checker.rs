//! CAWS Checker
//!
//! Provides CAWS compliance checking and validation for worker outputs.
//! Enhanced with AST-based diff sizing and violation code mapping.

// Import the refactored CAWS modules
use crate::caws::*;

use crate::types::*;
use agent_agency_council::models::{
    FileModification as CouncilFileModification, FileOperation as CouncilFileOperation, RiskTier,
    TaskSpec, WorkerOutput as CouncilWorkerOutput,
};
use agent_agency_database::{CawsViolation as DbCawsViolation, DatabaseClient};
use anyhow::{Context, Result};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

// Re-export the main CawsChecker for backward compatibility
pub use crate::caws::CawsChecker;