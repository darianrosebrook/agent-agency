//! Autonomous File Editor
//!
//! Integrates file operations from data-infrastructure with agent-orchestration
//! to enable safe, autonomous file editing capabilities.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use system_common_interfaces::{
    file_operations::{FileOperationsService, Workspace, Changeset, AllowList, Budgets,
                     ChangesetId, Patch, Hunk, FileResult, FileOpsError},
};
use tracing::{info, warn, error, instrument};

/// Autonomous file editor that integrates with agent orchestration

#[derive(Debug, Serialize)]
pub struct AutonomousFileEditor {
    /// File operations service
    #[serde(skip)]
    file_ops: Arc<dyn FileOperationsService>,
    /// Default repository path
    repo_path: std::path::PathBuf,
}

impl AutonomousFileEditor {
    /// Create a new autonomous file editor
    pub fn new(file_ops: Arc<dyn FileOperationsService>, repo_path: std::path::PathBuf) -> Self {
        Self { file_ops, repo_path }
    }

    /// Apply autonomous file changes with validation and safety checks
    #[instrument(skip(self, changes), fields(task_id = %task_id))]
    pub async fn apply_changes(
        &self,
        task_id: &str,
        changes: Vec<FileChange>,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> Result<ChangesetId, AutonomousFileEditError> {
        info!("Applying {} autonomous file changes for task {}", changes.len(), task_id);

        // Convert FileChange to Changeset
        let changeset = self.create_changeset(task_id, changes)?;

        // Validate changeset
        self.file_ops.validate_changeset(&changeset, allowlist, budgets).await
            .map_err(|e| AutonomousFileEditError::Validation(e.to_string()))?;

        // Create workspace
        let mut workspace = self.file_ops.create_workspace(task_id, &self.repo_path).await
            .map_err(|e| AutonomousFileEditError::Workspace(e.to_string()))?;

        // Apply changeset
        let changeset_id = workspace.apply(&changeset, allowlist, budgets).await
            .map_err(|e| AutonomousFileEditError::Application(e.to_string()))?;

        info!("Successfully applied changeset {} with {} patches", changeset_id.0, changeset.patches.len());

        Ok(changeset_id)
    }

    /// Preview changes without applying them
    pub async fn preview_changes(
        &self,
        changes: Vec<FileChange>,
    ) -> Result<ChangesetPreview, AutonomousFileEditError> {
        let changeset = self.create_changeset("preview", changes)?;

        Ok(ChangesetPreview {
            changeset: changeset.clone(),
            risk_assessment: self.assess_risk(&changeset),
            validation_warnings: vec![], // Would implement validation warnings
        })
    }

    /// Rollback changes
    pub async fn rollback_changes(
        &self,
        task_id: &str,
        changeset_id: &ChangesetId,
    ) -> Result<(), AutonomousFileEditError> {
        let workspace = self.file_ops.create_workspace(task_id, &self.repo_path).await
            .map_err(|e| AutonomousFileEditError::Workspace(e.to_string()))?;

        workspace.revert(changeset_id).await
            .map_err(|e| AutonomousFileEditError::Rollback(e.to_string()))?;

        info!("Successfully rolled back changeset {}", changeset_id.0);
        Ok(())
    }

    /// Promote changes to main repository
    pub async fn promote_changes(
        &self,
        task_id: &str,
    ) -> Result<(), AutonomousFileEditError> {
        let workspace = self.file_ops.create_workspace(task_id, &self.repo_path).await
            .map_err(|e| AutonomousFileEditError::Workspace(e.to_string()))?;

        workspace.promote().await
            .map_err(|e| AutonomousFileEditError::Promotion(e.to_string()))?;

        info!("Successfully promoted changes for task {}", task_id);
        Ok(())
    }

    /// Create a changeset from file changes
    fn create_changeset(&self, task_id: &str, changes: Vec<FileChange>) -> Result<Changeset, AutonomousFileEditError> {
        let patches = changes.into_iter()
            .map(|change| change.to_patch())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Changeset {
            id: ChangesetId(uuid::Uuid::new_v4().to_string()),
            description: format!("Autonomous file changes for task {}", task_id),
            patches,
            metadata: system_common_interfaces::ChangesetMetadata {
                author: "autonomous-agent".to_string(),
                timestamp: chrono::Utc::now(),
                risk_tier: 2, // Medium risk by default
                tags: vec!["autonomous".to_string(), "agent-generated".to_string()],
            },
        })
    }

    /// Assess risk of a changeset
    fn assess_risk(&self, changeset: &Changeset) -> RiskAssessment {
        let mut risk_score: f64 = 0.0;
        let mut risk_factors = Vec::new();

        // Assess based on number of files changed
        if changeset.patches.len() > 10 {
            risk_score += 0.3;
            risk_factors.push("Large number of files changed".to_string());
        }

        // Assess based on file types
        for patch in &changeset.patches {
            if patch.path.ends_with(".rs") {
                risk_score += 0.1; // Rust files are more critical
            }
            if patch.path.contains("Cargo.toml") || patch.path.contains("package.json") {
                risk_score += 0.2; // Dependency files are high risk
                risk_factors.push("Dependency file modification".to_string());
            }
        }

        // Assess based on change size
        let total_lines_changed: usize = changeset.patches.iter()
            .map(|p| p.hunks.iter().map(|h| h.new_lines as usize).sum::<usize>())
            .sum();

        if total_lines_changed > 100 {
            risk_score += 0.2;
            risk_factors.push("Large changeset".to_string());
        }

        RiskAssessment {
            score: risk_score.min(1.0),
            level: if risk_score > 0.7 {
                RiskLevel::High
            } else if risk_score > 0.3 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            },
            factors: risk_factors,
        }
    }
}

/// File change specification

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileChange {
    /// Path to the file to change
    pub path: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Old content (for context/replacement)
    pub old_content: Option<String>,
    /// New content
    pub new_content: String,
    /// Line number to start change (1-indexed)
    pub line_start: Option<usize>,
}

impl FileChange {
    /// Convert to a patch
    pub fn to_patch(&self) -> Result<Patch, AutonomousFileEditError> {
        match self.change_type {
            ChangeType::Create => {
                // For creation, treat as adding all content
                let lines = self.new_content.lines()
                    .map(|line| format!("+{}", line))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(Patch {
                    path: self.path.clone(),
                    hunks: vec![Hunk {
                        old_start: 1,
                        old_lines: 0,
                        new_start: 1,
                        new_lines: self.new_content.lines().count() as usize,
                        lines,
                    }],
                })
            }
            ChangeType::Replace => {
                let old_lines = self.old_content.as_ref()
                    .map(|c| c.lines().count())
                    .unwrap_or(0);
                let new_lines = self.new_content.lines().count();

                let lines = format!(
                    "{}\n{}",
                    self.old_content.as_ref()
                        .unwrap_or(&String::new())
                        .lines()
                        .map(|line| format!("-{}", line))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    self.new_content.lines()
                        .map(|line| format!("+{}", line))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                Ok(Patch {
                    path: self.path.clone(),
                    hunks: vec![Hunk {
                        old_start: self.line_start.unwrap_or(1),
                        old_lines: old_lines as usize,
                        new_start: self.line_start.unwrap_or(1),
                        new_lines,
                        lines,
                    }],
                })
            }
            ChangeType::Insert => {
                let lines = self.new_content.lines()
                    .map(|line| format!("+{}", line))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(Patch {
                    path: self.path.clone(),
                    hunks: vec![Hunk {
                        old_start: self.line_start.unwrap_or(1),
                        old_lines: 0,
                        new_start: self.line_start.unwrap_or(1),
                        new_lines: self.new_content.lines().count(),
                        lines,
                    }],
                })
            }
            ChangeType::Delete => {
                let old_lines = self.old_content.as_ref()
                    .map(|c| c.lines().count())
                    .unwrap_or(0);

                let lines = self.old_content.as_ref()
                    .unwrap_or(&String::new())
                    .lines()
                    .map(|line| format!("-{}", line))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(Patch {
                    path: self.path.clone(),
                    hunks: vec![Hunk {
                        old_start: self.line_start.unwrap_or(1),
                        old_lines,
                        new_start: self.line_start.unwrap_or(1),
                        new_lines: 0,
                        lines,
                    }],
                })
            }
        }
    }
}

/// Type of file change

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Copy)]
pub enum ChangeType {
    /// Create a new file
    Create,
    /// Replace content (requires old_content)
    Replace,
    /// Insert new content
    Insert,
    /// Delete content (requires old_content)
    Delete,
}

/// Risk assessment for a changeset

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAssessment {
    /// Risk score (0.0-1.0, higher is riskier)
    pub score: f64,
    /// Risk level
    pub level: RiskLevel,
    /// Risk factors identified
    pub factors: Vec<String>,
}

/// Risk levels

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Copy)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Preview of changes before application

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangesetPreview {
    /// The changeset that would be applied
    #[schemars(skip)]
    pub changeset: Changeset,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Validation warnings
    pub validation_warnings: Vec<String>,
}

/// Errors that can occur during autonomous file editing

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
pub enum AutonomousFileEditError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Application error: {0}")]
    Application(String),

    #[error("Rollback error: {0}")]
    Rollback(String),

    #[error("Promotion error: {0}")]
    Promotion(String),

    #[error("Change conversion error: {0}")]
    ChangeConversion(String),
}
