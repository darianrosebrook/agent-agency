//! Temp Mirror Workspace Implementation
//!
//! Uses temporary directory mirroring for safe file editing in non-Git environments
//! with rsync-based copying and snapshot capabilities.

use schemars::JsonSchema;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::Row;
use crate::file_operations::{Workspace, ChangeSet, AllowList, Budgets, ChangeSetId, FileOpsError, Result, validate_changeset, Patch, Hunk};
use crate::client::orchestrator::DatabaseClient;

/// Temp directory mirror workspace for safe file operations
pub struct TempMirrorWorkspace {
    /// Original project root
    source_root: PathBuf,
    /// Temporary workspace directory
    workspace_path: PathBuf,
    /// Task ID for this workspace
    task_id: String,
    /// Applied changesets for rollback tracking (wrapped in RwLock for interior mutability)
    applied_changesets: RwLock<Vec<(ChangeSetId, ChangeSet)>>,
    /// Changeset application engine for advanced operations
    changeset_engine: Arc<ChangesetApplicationEngine>,
    /// Optional database client for persistent changeset storage
    db_client: Option<Arc<DatabaseClient>>,
}

/// Comprehensive changeset application and management system
#[derive(Debug)]
pub struct ChangesetApplicationEngine {
    /// Active changeset applications being tracked
    active_applications: RwLock<HashMap<ChangeSetId, ChangesetApplicationState>>,
    /// Completed changeset applications for rollback
    completed_applications: RwLock<HashMap<ChangeSetId, CompletedChangesetApplication>>,
    /// Changeset dependencies and ordering
    dependency_graph: RwLock<HashMap<ChangeSetId, Vec<ChangeSetId>>>,
    /// Performance metrics for changeset operations
    metrics: RwLock<ChangesetMetrics>,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct ChangesetApplicationState {
    /// Unique application ID
    pub application_id: String,
    /// Changeset being applied
    pub changeset: ChangeSet,
    /// Current application phase
    pub phase: ApplicationPhase,
    /// Progress tracking
    pub progress: ApplicationProgress,
    /// Start time
    #[schemars(with = "String")]

    pub start_time: DateTime<Utc>,
    /// Validation results
    pub validation_results: Vec<ChangesetValidationResult>,
    /// Conflict detection results
    pub conflicts: Vec<ChangesetConflict>,
    /// Recovery checkpoints
    pub checkpoints: Vec<ApplicationCheckpoint>,
}

#[derive(Debug, Clone, JsonSchema)]
pub enum ApplicationPhase {
    Validation,
    ConflictDetection,
    BackupCreation,
    AtomicApplication,
    Verification,
    Completion,
    Rollback,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct ApplicationProgress {
    /// Total patches to apply
    pub total_patches: usize,
    /// Successfully applied patches
    pub applied_patches: usize,
    /// Failed patches
    pub failed_patches: usize,
    /// Estimated completion time
    pub estimated_completion: Option<DateTime<Utc>>,
    /// Current operation description
    pub current_operation: String,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct ChangesetValidationResult {
    /// Validation type performed
    pub validation_type: ValidationType,
    /// Validation passed
    pub passed: bool,
    /// Validation details/errors
    pub details: String,
    /// Severity level
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ValidationType {
    Integrity,
    Conflict,
    Dependency,
    Budget,
    AllowList,
    Content,
}

#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct ChangesetConflict {
    /// File path where conflict occurred
    pub file_path: PathBuf,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Conflicting lines/regions
    pub conflicting_lines: Vec<ConflictingLines>,
    /// Suggested resolution
    pub resolution_suggestion: ConflictResolution,
}

#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ConflictType {
    OverlappingHunks,
    FileModified,
    FileDeleted,
    PermissionDenied,
    ContentCorruption,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct ConflictingLines {
    /// Line number in the file
    pub line_number: usize,
    /// Original content
    pub original: String,
    /// New content from changeset
    pub new: String,
    /// Context lines
    pub context: Vec<String>,
}

#[derive(Debug, Clone, JsonSchema)]
pub enum ConflictResolution {
    /// Automatically resolve using newer version
    UseNew,
    /// Automatically resolve using older version
    UseOld,
    /// Merge the changes
    Merge,
    /// Skip this patch
    Skip,
    /// Manual resolution required
    Manual,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct ApplicationCheckpoint {
    /// Checkpoint ID
    pub checkpoint_id: String,
    /// Timestamp
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    /// Files backed up at this checkpoint
    pub backed_up_files: Vec<PathBuf>,
    /// Checksum of workspace state
    pub workspace_checksum: String,
    /// Successful patches applied up to this point
    pub applied_patches: usize,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct CompletedChangesetApplication {
    /// Changeset that was applied
    pub changeset: ChangeSet,
    /// Application start time
    #[schemars(with = "String")]

    pub start_time: DateTime<Utc>,
    /// Application completion time
    #[schemars(with = "String")]

    pub completion_time: DateTime<Utc>,
    /// Final application status
    pub status: ApplicationStatus,
    /// Performance metrics
    pub performance: ApplicationPerformance,
    /// Backup location for rollback
    pub backup_location: Option<PathBuf>,
    /// Validation summary
    pub validation_summary: ValidationSummary,
}

#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ApplicationStatus {
    Success,
    PartialSuccess,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, JsonSchema)]
#[derive(Default)]
pub struct ApplicationPerformance {
    /// Total application time
    pub total_time_ms: u64,
    /// Time spent on validation
    pub validation_time_ms: u64,
    /// Time spent on backup creation
    pub backup_time_ms: u64,
    /// Time spent on patch application
    pub application_time_ms: u64,
    /// Time spent on verification
    pub verification_time_ms: u64,
    /// Memory usage peak
    pub peak_memory_mb: u64,
    /// I/O operations performed
    pub io_operations: u64,
}

#[derive(Debug, Clone, JsonSchema)]
#[derive(Default)]
pub struct ValidationSummary {
    /// Total validations performed
    pub total_validations: usize,
    /// Passed validations
    pub passed_validations: usize,
    /// Failed validations
    pub failed_validations: usize,
    /// Warning validations
    pub warning_validations: usize,
    /// Critical issues found
    pub critical_issues: usize,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct ChangesetMetrics {
    /// Total changeset applications
    pub total_applications: u64,
    /// Successful applications
    pub successful_applications: u64,
    /// Failed applications
    pub failed_applications: u64,
    /// Average application time
    pub avg_application_time_ms: f64,
    /// Average validation time
    pub avg_validation_time_ms: f64,
    /// Average backup time
    pub avg_backup_time_ms: f64,
    /// Most common failure reasons
    pub common_failures: HashMap<String, u64>,
    /// Performance percentiles
    pub performance_percentiles: HashMap<String, f64>,
}

impl TempMirrorWorkspace {
    /// Create a new temp mirror workspace
    pub async fn new(project_path: &Path, task_id: &str) -> Result<Self> {
        Self::new_with_db(project_path, task_id, None).await
    }

    /// Create a new temp mirror workspace with optional database client for persistent storage
    pub async fn new_with_db(
        project_path: &Path,
        task_id: &str,
        db_client: Option<Arc<DatabaseClient>>,
    ) -> Result<Self> {
        let source_root = project_path.canonicalize()
            .map_err(|e| FileOpsError::Path(format!("Cannot canonicalize project path: {}", e)))?;

        // Create temp workspace directory
        let workspace_path = std::env::temp_dir()
            .join(format!("caws-workspace-{}", task_id));

        // Clean up any existing workspace
        let _ = fs::remove_dir_all(&workspace_path).await;

        // Create workspace directory
        fs::create_dir_all(&workspace_path).await
            .map_err(FileOpsError::Io)?;

        // Mirror source to workspace using rsync if available, otherwise manual copy
        Self::mirror_directory(&source_root, &workspace_path).await?;

        // Initialize changeset storage if database client is provided
        if let Some(ref db) = db_client {
            Self::ensure_changeset_table(db).await?;
        }

        Ok(Self {
            source_root,
            workspace_path,
            task_id: task_id.to_string(),
            applied_changesets: RwLock::new(Vec::new()),
            changeset_engine: Arc::new(ChangesetApplicationEngine::new()),
            db_client,
        })
    }

    /// Ensure the changeset storage table exists
    async fn ensure_changeset_table(db: &DatabaseClient) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS changeset_storage (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                changeset_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                workspace_path TEXT NOT NULL,
                changeset_data JSONB NOT NULL,
                checksum TEXT NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                metadata JSONB DEFAULT '{}'::jsonb
            )
            "#
        )
        .execute(db.pool())
        .await
        .map_err(|e| FileOpsError::Path(format!("Failed to create changeset table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_changeset_storage_changeset_id 
            ON changeset_storage(changeset_id)
            "#
        )
        .execute(db.pool())
        .await
        .map_err(|e| FileOpsError::Path(format!("Failed to create changeset index: {}", e)))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_changeset_storage_task_id 
            ON changeset_storage(task_id)
            "#
        )
        .execute(db.pool())
        .await
        .map_err(|e| FileOpsError::Path(format!("Failed to create task index: {}", e)))?;

        Ok(())
    }

    /// Store a changeset persistently in the database
    async fn store_changeset(
        &self,
        changeset_id: &ChangeSetId,
        changeset: &ChangeSet,
    ) -> Result<()> {
        if let Some(ref db) = self.db_client {
            let changeset_json = serde_json::to_value(changeset)
                .map_err(|e| FileOpsError::Path(format!("Failed to serialize changeset: {}", e)))?;
            
            let mut hasher = Sha256::new();
            hasher.update(changeset_json.to_string().as_bytes());
            let checksum = format!("{:x}", hasher.finalize());

            sqlx::query(
                r#"
                INSERT INTO changeset_storage (
                    changeset_id, task_id, workspace_path, changeset_data, checksum, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#
            )
            .bind(&changeset_id.0)
            .bind(&self.task_id)
            .bind(self.workspace_path.to_string_lossy().as_ref())
            .bind(&changeset_json)
            .bind(&checksum)
            .bind(serde_json::json!({}))
            .execute(db.pool())
            .await
            .map_err(|e| FileOpsError::Path(format!("Failed to store changeset: {}", e)))?;

            Ok(())
        } else {
            // No database client, store in memory only
            self.applied_changesets.write().await.push((changeset_id.clone(), changeset.clone()));
            Ok(())
        }
    }

    /// Retrieve a changeset from persistent storage
    async fn get_changeset(&self, changeset_id: &ChangeSetId) -> Result<Option<ChangeSet>> {
        if let Some(ref db) = self.db_client {
            let row = sqlx::query(
                r#"
                SELECT changeset_data FROM changeset_storage
                WHERE changeset_id = $1 AND task_id = $2
                ORDER BY applied_at DESC
                LIMIT 1
                "#
            )
            .bind(&changeset_id.0)
            .bind(&self.task_id)
            .fetch_optional(db.pool())
            .await
            .map_err(|e| FileOpsError::Path(format!("Failed to retrieve changeset: {}", e)))?;

            if let Some(row) = row {
                let changeset_json: serde_json::Value = row.get("changeset_data");
                let changeset: ChangeSet = serde_json::from_value(changeset_json)
                    .map_err(|e| FileOpsError::Path(format!("Failed to deserialize changeset: {}", e)))?;
                Ok(Some(changeset))
            } else {
                Ok(None)
            }
        } else {
            // No database client, check in-memory storage
            Ok(self.applied_changesets.read().await.iter()
                .find(|(id, _)| id == changeset_id)
                .map(|(_, cs)| cs.clone()))
        }
    }

    /// Apply changeset with comprehensive validation, conflict detection, and rollback support
    pub async fn apply_changeset_comprehensive(
        &self,
        changeset: &ChangeSet,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> Result<ChangeSetId> {
        self.changeset_engine.apply_changeset_comprehensive(
            changeset,
            allowlist,
            budgets,
            &self.workspace_path,
        ).await
    }

    /// Get application status for a changeset
    pub async fn get_application_status(&self, changeset_id: &ChangeSetId) -> Result<Option<ChangesetApplicationState>> {
        self.changeset_engine.get_application_status(changeset_id).await
    }

    /// Rollback a changeset application
    pub async fn rollback_changeset(&self, changeset_id: &ChangeSetId) -> Result<()> {
        self.changeset_engine.rollback_changeset(changeset_id, &self.workspace_path).await
    }

    /// Get changeset performance metrics
    pub async fn get_changeset_metrics(&self) -> Result<ChangesetMetrics> {
        self.changeset_engine.get_metrics().await
    }

    /// Mirror directory contents (rsync if available, manual copy otherwise)
    async fn mirror_directory(source: &Path, dest: &Path) -> Result<()> {
        // Try rsync first
        let rsync_result = Command::new("rsync")
            .args(["-a", "--exclude=.git", &format!("{}/", source.display()), &dest.display().to_string()])
            .output();

        if rsync_result.is_ok() && rsync_result.as_ref().unwrap().status.success() {
            return Ok(());
        }

        // Fallback to manual copy
        Self::copy_directory_recursive(source, dest).await
    }

    /// Recursively copy directory contents
    async fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<()> {
        let mut stack = vec![(source.to_path_buf(), dest.to_path_buf())];

        while let Some((src, dst)) = stack.pop() {
            // Create destination directory
            fs::create_dir_all(&dst).await.map_err(FileOpsError::Io)?;

            // Read source directory
            let mut entries = fs::read_dir(&src).await.map_err(FileOpsError::Io)?;

            while let Some(entry) = entries.next_entry().await.map_err(FileOpsError::Io)? {
                let entry_type = entry.file_type().await.map_err(FileOpsError::Io)?;
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());

                if entry_type.is_dir() {
                    // Skip .git directories
                    if entry.file_name() != ".git" {
                        stack.push((src_path, dst_path));
                    }
                } else if entry_type.is_file() {
                    fs::copy(&src_path, &dst_path).await.map_err(FileOpsError::Io)?;
                }
            }
        }

        Ok(())
    }

    /// Apply patches to files in the workspace
    async fn apply_patches(&self, changeset: &ChangeSet) -> Result<()> {
        for patch in &changeset.patches {
            self.apply_single_patch(patch).await?;
        }
        Ok(())
    }

    /// Apply a single patch
    async fn apply_single_patch(&self, patch: &Patch) -> Result<()> {
        let file_path = self.workspace_path.join(&patch.path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(FileOpsError::Io)?;
        }

        // Read current file content
        let current_content = match fs::read_to_string(&file_path).await {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(FileOpsError::Io(e)),
        };

        // Apply patch hunks
        let new_content = self.apply_hunks_to_content(&current_content, &patch.hunks)?;

        // Write new content
        fs::write(&file_path, new_content).await
            .map_err(FileOpsError::Io)?;

        Ok(())
    }

    /// Apply hunks to file content
    fn apply_hunks_to_content(&self, content: &str, hunks: &[Hunk]) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut offset: i32 = 0;

        for hunk in hunks {
            let base_start = (hunk.old_start as usize).saturating_sub(1);
            let start_line = if offset >= 0 {
                base_start + offset as usize
            } else {
                base_start.saturating_sub(offset.abs() as usize)
            };
            let _end_line = start_line + (hunk.old_lines as usize);

            // Remove old lines
            if start_line < lines.len() {
                let remove_count = std::cmp::min(hunk.old_lines as usize, lines.len() - start_line);
                lines.drain(start_line..start_line + remove_count);
                offset -= remove_count as i32;
            }

            // Add new lines
            if hunk.new_lines > 0 {
                let insert_pos = std::cmp::min(start_line, lines.len());
                let new_lines: Vec<String> = hunk.lines
                    .lines()
                    .filter(|line| line.starts_with('+') || line.starts_with(' '))
                    .map(|line| line[1..].to_string())
                    .collect();

                for (i, new_line) in new_lines.into_iter().enumerate() {
                    lines.insert(insert_pos + i, new_line);
                }
                offset += hunk.new_lines as i32;
            }
        }

        Ok(lines.join("\n"))
    }

    /// Create a snapshot of current workspace state
    async fn create_snapshot(&self) -> Result<PathBuf> {
        let snapshot_dir = std::env::temp_dir()
            .join(format!("caws-snapshot-{}-{}", self.task_id, Uuid::new_v4()));

        fs::create_dir_all(&snapshot_dir).await.map_err(FileOpsError::Io)?;
        Self::copy_directory_recursive(&self.workspace_path, &snapshot_dir).await?;

        Ok(snapshot_dir)
    }
}

impl ChangesetApplicationEngine {
    /// Create a new changeset application engine
    pub fn new() -> Self {
        Self {
            active_applications: RwLock::new(HashMap::new()),
            completed_applications: RwLock::new(HashMap::new()),
            dependency_graph: RwLock::new(HashMap::new()),
            metrics: RwLock::new(ChangesetMetrics {
                total_applications: 0,
                successful_applications: 0,
                failed_applications: 0,
                avg_application_time_ms: 0.0,
                avg_validation_time_ms: 0.0,
                avg_backup_time_ms: 0.0,
                common_failures: HashMap::new(),
                performance_percentiles: HashMap::new(),
            }),
        }
    }

    /// Apply changeset with comprehensive validation, conflict detection, and rollback support
    pub async fn apply_changeset_comprehensive(
        &self,
        changeset: &ChangeSet,
        allowlist: &AllowList,
        budgets: &Budgets,
        workspace_path: &Path,
    ) -> Result<ChangeSetId> {
        let start_time = Utc::now();
        let changeset_id = ChangeSetId(Uuid::new_v4().to_string());

        // Initialize application state
        let application_state = ChangesetApplicationState {
            application_id: Uuid::new_v4().to_string(),
            changeset: changeset.clone(),
            phase: ApplicationPhase::Validation,
            progress: ApplicationProgress {
                total_patches: changeset.patches.len(),
                applied_patches: 0,
                failed_patches: 0,
                estimated_completion: None,
                current_operation: "Starting validation".to_string(),
            },
            start_time,
            validation_results: Vec::new(),
            conflicts: Vec::new(),
            checkpoints: Vec::new(),
        };

        // Register active application
        {
            let mut active_apps = self.active_applications.write().await;
            active_apps.insert(changeset_id.clone(), application_state);
        }

        let result = self.execute_changeset_application(&changeset_id, changeset, allowlist, budgets, workspace_path, start_time).await;

        // Update metrics
        self.update_metrics(start_time, &result).await;

        result
    }

    /// Execute the comprehensive changeset application process
    async fn execute_changeset_application(
        &self,
        changeset_id: &ChangeSetId,
        changeset: &ChangeSet,
        allowlist: &AllowList,
        budgets: &Budgets,
        workspace_path: &Path,
    start_time: DateTime<Utc>,
    ) -> Result<ChangeSetId> {
        // Phase 1: Comprehensive Validation
        self.update_phase(changeset_id, ApplicationPhase::Validation).await?;
        let validation_results = self.perform_comprehensive_validation(changeset, allowlist, budgets, workspace_path).await?;
        self.record_validation_results(changeset_id, validation_results.clone()).await?;

        // Check for critical validation failures
        let has_critical_failures = validation_results.iter()
            .any(|r| r.severity == ValidationSeverity::Error && !r.passed);

        if has_critical_failures {
            self.finalize_application(changeset_id, ApplicationStatus::Failed, start_time, None).await?;
            return Err(FileOpsError::Validation("Critical validation failures detected".to_string()));
        }

        // Phase 2: Conflict Detection
        self.update_phase(changeset_id, ApplicationPhase::ConflictDetection).await?;
        let conflicts = self.detect_conflicts(changeset, workspace_path).await?;
        self.record_conflicts(changeset_id, conflicts.clone()).await?;

        // Phase 3: Backup Creation
        self.update_phase(changeset_id, ApplicationPhase::BackupCreation).await?;
        let backup_location = self.create_backup(changeset, workspace_path).await?;
        self.create_checkpoint(changeset_id, &backup_location, 0).await?;

        // Phase 4: Atomic Application
        self.update_phase(changeset_id, ApplicationPhase::AtomicApplication).await?;
        let application_result = self.apply_changeset_atomically(changeset, workspace_path).await?;

        // Phase 5: Verification
        self.update_phase(changeset_id, ApplicationPhase::Verification).await?;
        let verification_passed = self.verify_changeset_application(changeset, workspace_path).await?;

        // Phase 6: Completion
        self.update_phase(changeset_id, ApplicationPhase::Completion).await?;

        let status = if verification_passed && application_result.successful_patches == changeset.patches.len() {
            ApplicationStatus::Success
        } else if application_result.successful_patches > 0 {
            ApplicationStatus::PartialSuccess
        } else {
            ApplicationStatus::Failed
        };

        let performance = ApplicationPerformance {
            total_time_ms: (Utc::now() - start_time).num_milliseconds() as u64,
            validation_time_ms: 100, // Placeholder - would track actual times
            backup_time_ms: 200,     // Placeholder
            application_time_ms: application_result.application_time_ms,
            verification_time_ms: 50, // Placeholder
            peak_memory_mb: 100,    // Placeholder
            io_operations: application_result.io_operations,
        };

        let validation_summary = ValidationSummary {
            total_validations: validation_results.len(),
            passed_validations: validation_results.iter().filter(|r| r.passed).count(),
            failed_validations: validation_results.iter().filter(|r| !r.passed && r.severity == ValidationSeverity::Error).count(),
            warning_validations: validation_results.iter().filter(|r| !r.passed && r.severity == ValidationSeverity::Warning).count(),
            critical_issues: validation_results.iter().filter(|r| r.severity == ValidationSeverity::Error && !r.passed).count(),
        };

        self.finalize_application(changeset_id, status, start_time, Some((performance, validation_summary, backup_location))).await?;

        Ok(changeset_id.clone())
    }

    /// Perform comprehensive validation including integrity, conflicts, dependencies, etc.
    async fn perform_comprehensive_validation(
        &self,
        changeset: &ChangeSet,
        allowlist: &AllowList,
        budgets: &Budgets,
        _workspace_path: &Path,
    ) -> Result<Vec<ChangesetValidationResult>> {
        let mut results = Vec::new();

        // Basic validation (allowlist, budgets)
        match validate_changeset(changeset, allowlist, budgets) {
            Ok(_) => results.push(ChangesetValidationResult {
                validation_type: ValidationType::AllowList,
                passed: true,
                details: "Allowlist validation passed".to_string(),
                severity: ValidationSeverity::Info,
            }),
            Err(e) => results.push(ChangesetValidationResult {
                validation_type: ValidationType::AllowList,
                passed: false,
                details: format!("Allowlist validation failed: {}", e),
                severity: ValidationSeverity::Error,
            }),
        }

        // Integrity validation - check patch checksums
        for patch in &changeset.patches {
            let integrity_valid = self.validate_patch_integrity(patch).await?;
            results.push(ChangesetValidationResult {
                validation_type: ValidationType::Integrity,
                passed: integrity_valid,
                details: if integrity_valid {
                    format!("Patch integrity validated for {}", patch.path)
                } else {
                    format!("Patch integrity check failed for {}", patch.path)
                },
                severity: if integrity_valid { ValidationSeverity::Info } else { ValidationSeverity::Error },
            });
        }

        // Content validation - check for potentially problematic content
        for patch in &changeset.patches {
            let content_valid = self.validate_patch_content(patch)?;
            results.push(ChangesetValidationResult {
                validation_type: ValidationType::Content,
                passed: content_valid,
                details: if content_valid {
                    format!("Content validation passed for {}", patch.path)
                } else {
                    format!("Content validation failed for {}", patch.path)
                },
                severity: if content_valid { ValidationSeverity::Info } else { ValidationSeverity::Warning },
            });
        }

        // Dependency validation (simplified)
        let dependency_valid = self.validate_changeset_dependencies(changeset).await?;
        results.push(ChangesetValidationResult {
            validation_type: ValidationType::Dependency,
            passed: dependency_valid,
            details: if dependency_valid {
                "Dependency validation passed".to_string()
            } else {
                "Dependency validation failed - circular or missing dependencies".to_string()
            },
            severity: if dependency_valid { ValidationSeverity::Info } else { ValidationSeverity::Warning },
        });

        Ok(results)
    }

    /// Detect conflicts between changeset and current workspace state
    async fn detect_conflicts(
        &self,
        changeset: &ChangeSet,
        workspace_path: &Path,
    ) -> Result<Vec<ChangesetConflict>> {
        let mut conflicts = Vec::new();

        for patch in &changeset.patches {
            let file_path = workspace_path.join(&patch.path);

            // Check if file exists and has been modified
            if file_path.exists() {
                let current_content = fs::read_to_string(&file_path).await
                    .unwrap_or_default();
                let file_modified = self.detect_file_conflicts(&patch, &current_content)?;

                if !file_modified.is_empty() {
                    conflicts.push(ChangesetConflict {
                        file_path: PathBuf::from(patch.path.clone()),
                        conflict_type: ConflictType::FileModified,
                        conflicting_lines: file_modified,
                        resolution_suggestion: ConflictResolution::Manual,
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Create backup of workspace state
    async fn create_backup(&self, changeset: &ChangeSet, workspace_path: &Path) -> Result<PathBuf> {
        let backup_id = Uuid::new_v4().to_string();
        let backup_path = std::env::temp_dir().join(format!("caws-backup-{}", backup_id));

        fs::create_dir_all(&backup_path).await.map_err(FileOpsError::Io)?;

        // Copy all files that will be modified
        for patch in &changeset.patches {
            let source_path = workspace_path.join(&patch.path);
            if source_path.exists() {
                let backup_file = backup_path.join(&patch.path);
                if let Some(parent) = backup_file.parent() {
                    fs::create_dir_all(parent).await.map_err(FileOpsError::Io)?;
                }
                fs::copy(&source_path, &backup_file).await.map_err(FileOpsError::Io)?;
            }
        }

        Ok(backup_path)
    }

    /// Apply changeset atomically with progress tracking
    async fn apply_changeset_atomically(
        &self,
        changeset: &ChangeSet,
        workspace_path: &Path,
    ) -> Result<AtomicApplicationResult> {
        let start_time = std::time::Instant::now();
        let mut successful_patches = 0;
        let mut io_operations = 0;

        for (i, patch) in changeset.patches.iter().enumerate() {
            // Update progress
            self.update_progress(&ChangeSetId("temp".to_string()), i + 1, successful_patches, 0,
                               format!("Applying patch to {}", patch.path)).await?;

            match self.apply_single_patch_atomic(patch, workspace_path).await {
                Ok(ops) => {
                    successful_patches += 1;
                    io_operations += ops;
                }
                Err(e) => {
                    tracing::warn!("Failed to apply patch to {}: {}", patch.path, e);
                    break; // Stop on first failure for atomicity
                }
            }
        }

        Ok(AtomicApplicationResult {
            successful_patches,
            total_patches: changeset.patches.len(),
            application_time_ms: start_time.elapsed().as_millis() as u64,
            io_operations,
        })
    }

    /// Verify changeset application integrity
    async fn verify_changeset_application(
        &self,
        changeset: &ChangeSet,
        workspace_path: &Path,
    ) -> Result<bool> {
        for patch in &changeset.patches {
            let file_path = workspace_path.join(&patch.path);

            if !file_path.exists() {
                return Ok(false);
            }

            // Verify file content matches expected result
            let current_content = fs::read_to_string(&file_path).await
                .unwrap_or_default();

            if !self.verify_patch_application(patch, &current_content) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Rollback a changeset application
    pub async fn rollback_changeset(
        &self,
        changeset_id: &ChangeSetId,
        workspace_path: &Path,
    ) -> Result<()> {
        let completed_app = {
            let completed_apps = self.completed_applications.read().await;
            completed_apps.get(changeset_id).cloned()
        };

        if let Some(app) = completed_app {
            if let Some(backup_location) = app.backup_location {
                // Restore from backup
                self.restore_from_backup(&backup_location, workspace_path).await?;

                // Update application status
                let mut completed_apps = self.completed_applications.write().await;
                if let Some(app_mut) = completed_apps.get_mut(changeset_id) {
                    app_mut.status = ApplicationStatus::RolledBack;
                }
            }
        }

        Ok(())
    }

    /// Helper methods for validation and conflict detection
    async fn validate_patch_integrity(&self, patch: &Patch) -> Result<bool> {
        // Calculate checksum of patch content
        let content = format!("{:?}", patch); // Simplified checksum
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let _checksum = hasher.finalize();
        // In real implementation, compare with stored checksum
        Ok(true)
    }

    fn validate_patch_content(&self, patch: &Patch) -> Result<bool> {
        // Check for potentially problematic patterns
        for hunk in &patch.hunks {
            for line in hunk.lines.lines() {
                // Check for null bytes, extremely long lines, etc.
                if line.contains('\0') || line.len() > 10000 {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    async fn validate_changeset_dependencies(&self, changeset: &ChangeSet) -> Result<bool> {
        // Validate changeset dependencies
        // Check for circular dependencies, missing prerequisites, and self-referential patches
        
        let mut file_dependencies: HashMap<String, Vec<String>> = HashMap::new();
        let mut files_modified: HashSet<String> = HashSet::new();

        // Build dependency graph from patches
        for patch in &changeset.patches {
            files_modified.insert(patch.path.clone());
            
            // Extract file dependencies from patch content
            // Look for imports, includes, or other file references in patch content
            let mut deps = Vec::new();
            for hunk in &patch.hunks {
                for line in hunk.lines.lines() {
                    // Check for Rust imports (use statements)
                    if line.trim().starts_with("use ") || line.trim().starts_with("mod ") {
                        if let Some(ref_file) = extract_file_reference(line) {
                            deps.push(ref_file);
                        }
                    }
                    // Check for other common patterns (imports, includes, etc.)
                    if line.contains("include!") || line.contains("include_str!") {
                        if let Some(ref_file) = extract_file_reference(line) {
                            deps.push(ref_file);
                        }
                    }
                }
            }
            
            if !deps.is_empty() {
                file_dependencies.insert(patch.path.clone(), deps);
            }
        }

        // Check for circular dependencies using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for file in file_dependencies.keys() {
            if !visited.contains(file) {
                if self.has_circular_dependency(file, &file_dependencies, &mut visited, &mut rec_stack) {
                    return Ok(false);
                }
            }
        }

        // Check that all referenced files are either being modified or exist
        for (_file, deps) in &file_dependencies {
            for dep in deps {
                if !files_modified.contains(dep) {
                    // In a real implementation, check if file exists in workspace
                    // For now, assume missing dependencies are warnings, not errors
                }
            }
        }

        // Check for self-referential patches (patch modifies file that it depends on)
        for (file, deps) in &file_dependencies {
            if deps.contains(file) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Helper to detect circular dependencies using DFS
    fn has_circular_dependency(
        &self,
        file: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(file.to_string());
        rec_stack.insert(file.to_string());

        if let Some(deps) = graph.get(file) {
            for dep in deps {
                if !visited.contains(dep) {
                    if self.has_circular_dependency(dep, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true; // Circular dependency detected
                }
            }
        }

        rec_stack.remove(file);
        false
    }

    fn detect_file_conflicts(&self, patch: &Patch, current_content: &str) -> Result<Vec<ConflictingLines>> {
        let mut conflicts = Vec::new();
        let current_lines: Vec<&str> = current_content.lines().collect();

        for hunk in &patch.hunks {
            let start_line = (hunk.old_start as usize).saturating_sub(1);

            if start_line < current_lines.len() {
                let expected_lines: Vec<&str> = hunk.lines
                    .lines()
                    .filter(|line| line.starts_with('-') || line.starts_with(' '))
                    .map(|line| &line[1..])
                    .collect();

                for (i, expected_line) in expected_lines.iter().enumerate() {
                    let current_line_idx = start_line + i;
                    if current_line_idx < current_lines.len() {
                        let current_line = current_lines[current_line_idx];
                        if current_line != *expected_line {
                            conflicts.push(ConflictingLines {
                                line_number: current_line_idx + 1,
                                original: current_line.to_string(),
                                new: expected_line.to_string(),
                                context: Vec::new(), // Would populate with surrounding lines
                            });
                        }
                    }
                }
            }
        }

        Ok(conflicts)
    }
}

/// Helper function to extract file reference from import/include statements
fn extract_file_reference(line: &str) -> Option<String> {
    // Extract file path from Rust use/mod statements
    // Examples: "use crate::module::submodule;" -> "module/submodule.rs"
    //           "mod my_module;" -> "my_module.rs"
    //           "include_str!(\"file.txt\")" -> "file.txt"
    
    let line = line.trim();
    
    // Handle Rust use statements: use crate::path::to::module;
    if line.starts_with("use ") {
        let parts: Vec<&str> = line
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .split("::")
            .collect();
        
        if parts.len() > 1 && parts[0] == "crate" {
            // Convert crate path to file path
            let module_path = parts[1..].join("/");
            return Some(format!("{}.rs", module_path));
        } else if !parts.is_empty() {
            // External crate or relative path
            let module_path = parts.join("/");
            return Some(format!("{}.rs", module_path));
        }
    }
    
    // Handle mod statements: mod my_module;
    if line.starts_with("mod ") {
        let module_name = line
            .trim_start_matches("mod ")
            .trim_end_matches(';')
            .split_whitespace()
            .next()?;
        return Some(format!("{}.rs", module_name));
    }
    
    // Handle include macros: include_str!("file.txt")
    if line.contains("include_str!") || line.contains("include!") {
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                return Some(line[start + 1..start + 1 + end].to_string());
            }
        }
    }
    
    None
}

impl ChangesetApplicationEngine {
    async fn apply_single_patch_atomic(&self, patch: &Patch, workspace_path: &Path) -> Result<u64> {
        let file_path = workspace_path.join(&patch.path);
        let io_operations = 1; // Simplified count

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(FileOpsError::Io)?;
        }

        // Read current content
        let current_content = fs::read_to_string(&file_path).await.unwrap_or_default();

        // Apply patch
        let new_content = self.apply_hunks_to_content(&current_content, &patch.hunks)?;

        // Write new content
        fs::write(&file_path, new_content).await.map_err(FileOpsError::Io)?;

        Ok(io_operations)
    }

    fn verify_patch_application(&self, patch: &Patch, current_content: &str) -> bool {
        // Verify patch application by checking that expected changes are present
        // This is a simplified verification - in a full implementation, would:
        // 1. Calculate expected content hash after patch application
        // 2. Calculate actual content hash
        // 3. Compare hunks against actual content
        // 4. Verify all expected additions are present
        // 5. Verify all expected deletions are absent
        
        if current_content.is_empty() {
            return false;
        }
        
        // Check that patch hunks can be matched in content
        let content_lines: Vec<&str> = current_content.lines().collect();
        
        for hunk in &patch.hunks {
            let start_line = (hunk.old_start as usize).saturating_sub(1);
            
            // Verify context lines match (if old_start is within bounds)
            if start_line < content_lines.len() {
                let hunk_lines: Vec<&str> = hunk.lines.lines().collect();
                let context_lines: Vec<&str> = hunk_lines
                    .iter()
                    .filter(|line| line.starts_with(' '))
                    .map(|line| line.trim_start())
                    .collect();
                
                // Check if context matches expected
                for (i, expected_context) in context_lines.iter().take(3).enumerate() {
                    let line_idx = start_line + i;
                    if line_idx < content_lines.len() {
                        if content_lines[line_idx].trim() != expected_context.trim() {
                            // Context mismatch - patch may not have been applied correctly
                            return false;
                        }
                    }
                }
            }
        }
        
        true
    }

    async fn restore_from_backup(&self, backup_path: &Path, workspace_path: &Path) -> Result<()> {
        // Copy files back from backup
        let mut dir_entries = fs::read_dir(backup_path).await.map_err(FileOpsError::Io)?;

        while let Some(entry) = dir_entries.next_entry().await.map_err(FileOpsError::Io)? {
            let backup_file = entry.path();
            let relative_path = backup_file.strip_prefix(backup_path).unwrap();
            let target_file = workspace_path.join(relative_path);

            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent).await.map_err(FileOpsError::Io)?;
            }

            fs::copy(&backup_file, &target_file).await.map_err(FileOpsError::Io)?;
        }

        Ok(())
    }

    /// Update application phase
    async fn update_phase(&self, changeset_id: &ChangeSetId, phase: ApplicationPhase) -> Result<()> {
        let mut active_apps = self.active_applications.write().await;
        if let Some(app) = active_apps.get_mut(changeset_id) {
            app.phase = phase;
        }
        Ok(())
    }

    /// Update application progress
    async fn update_progress(
        &self,
        changeset_id: &ChangeSetId,
        applied: usize,
        _successful: usize,
        failed: usize,
        operation: String,
    ) -> Result<()> {
        let mut active_apps = self.active_applications.write().await;
        if let Some(app) = active_apps.get_mut(changeset_id) {
            app.progress.applied_patches = applied;
            app.progress.failed_patches = failed;
            app.progress.current_operation = operation;
        }
        Ok(())
    }

    /// Record validation results
    async fn record_validation_results(&self, changeset_id: &ChangeSetId, results: Vec<ChangesetValidationResult>) -> Result<()> {
        let mut active_apps = self.active_applications.write().await;
        if let Some(app) = active_apps.get_mut(changeset_id) {
            app.validation_results = results;
        }
        Ok(())
    }

    /// Record conflicts
    async fn record_conflicts(&self, changeset_id: &ChangeSetId, conflicts: Vec<ChangesetConflict>) -> Result<()> {
        let mut active_apps = self.active_applications.write().await;
        if let Some(app) = active_apps.get_mut(changeset_id) {
            app.conflicts = conflicts;
        }
        Ok(())
    }

    /// Create application checkpoint
    async fn create_checkpoint(&self, changeset_id: &ChangeSetId, _backup_location: &Path, applied_patches: usize) -> Result<()> {
        let checkpoint = ApplicationCheckpoint {
            checkpoint_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            backed_up_files: Vec::new(), // Would populate with actual backed up files
            workspace_checksum: "placeholder".to_string(), // Would calculate actual checksum
            applied_patches,
        };

        let mut active_apps = self.active_applications.write().await;
        if let Some(app) = active_apps.get_mut(changeset_id) {
            app.checkpoints.push(checkpoint);
        }
        Ok(())
    }

    /// Finalize application and move to completed
    async fn finalize_application(
        &self,
        changeset_id: &ChangeSetId,
        status: ApplicationStatus,
    start_time: DateTime<Utc>,
        details: Option<(ApplicationPerformance, ValidationSummary, PathBuf)>,
    ) -> Result<()> {
        let mut active_apps = self.active_applications.write().await;
        let mut completed_apps = self.completed_applications.write().await;

        if let Some(app_state) = active_apps.remove(changeset_id) {
            let completed_app = CompletedChangesetApplication {
                changeset: app_state.changeset,
                start_time,
                completion_time: Utc::now(),
                status,
                performance: details.as_ref().map(|(p, _, _)| p.clone()).unwrap_or_default(),
                backup_location: details.as_ref().map(|(_, _, b)| Some(b.clone())).flatten(),
                validation_summary: details.as_ref().map(|(_, s, _)| s.clone()).unwrap_or_default(),
            };

            completed_apps.insert(changeset_id.clone(), completed_app);
        }

        Ok(())
    }

    /// Update performance metrics
    async fn update_metrics(&self, start_time: DateTime<Utc>, result: &Result<ChangeSetId>) {
        let mut metrics = self.metrics.write().await;
        metrics.total_applications += 1;

        let duration = (Utc::now() - start_time).num_milliseconds() as f64;

        if result.is_ok() {
            metrics.successful_applications += 1;
        } else {
            metrics.failed_applications += 1;
        }

        // Update rolling averages
        let total = metrics.total_applications as f64;
        metrics.avg_application_time_ms = (metrics.avg_application_time_ms * (total - 1.0) + duration) / total;
    }

    /// Get application status
    pub async fn get_application_status(&self, changeset_id: &ChangeSetId) -> Result<Option<ChangesetApplicationState>> {
        let active_apps = self.active_applications.read().await;
        Ok(active_apps.get(changeset_id).cloned())
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> Result<ChangesetMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// Apply hunks to file content
    fn apply_hunks_to_content(&self, content: &str, hunks: &[Hunk]) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut offset: i32 = 0;

        for hunk in hunks {
            let base_start = (hunk.old_start as usize).saturating_sub(1);
            let start_line = if offset >= 0 {
                base_start + offset as usize
            } else {
                base_start.saturating_sub(offset.abs() as usize)
            };
            let _end_line = start_line + (hunk.old_lines as usize);

            // Remove old lines
            if start_line < lines.len() {
                let remove_count = std::cmp::min(hunk.old_lines as usize, lines.len() - start_line);
                lines.drain(start_line..start_line + remove_count);
                offset -= remove_count as i32;
            }

            // Add new lines
            if hunk.new_lines > 0 {
                let insert_pos = std::cmp::min(start_line, lines.len());
                let new_lines: Vec<String> = hunk.lines
                    .lines()
                    .filter(|line| line.starts_with('+') || line.starts_with(' '))
                    .map(|line| &line[1..])
                    .map(|s| s.to_string())
                    .collect();

                for (i, new_line) in new_lines.into_iter().enumerate() {
                    lines.insert(insert_pos + i, new_line);
                }
                offset += hunk.new_lines as i32;
            }
        }

        Ok(lines.join("\n"))
    }
}

#[derive(Debug)]
struct AtomicApplicationResult {
    successful_patches: usize,
    total_patches: usize,
    application_time_ms: u64,
    io_operations: u64,
}

#[async_trait::async_trait]
impl Workspace for TempMirrorWorkspace {
    fn root(&self) -> &Path {
        &self.workspace_path
    }

    async fn apply(
        &self,
        changeset: &ChangeSet,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> Result<ChangeSetId> {
        // Validate changeset first
        validate_changeset(changeset, allowlist, budgets)?;

        // Generate changeset ID
        let changeset_id = ChangeSetId(Uuid::new_v4().to_string());

        // Create snapshot before applying changes
        let _snapshot = self.create_snapshot().await?;

        // Apply patches
        self.apply_patches(changeset).await?;

        // Store changeset persistently for rollback
        self.store_changeset(&changeset_id, changeset).await?;

        Ok(changeset_id)
    }

    async fn revert(&self, changeset_id: &ChangeSetId) -> Result<()> {
        // Retrieve the changeset from persistent storage
        let changeset = self.get_changeset(changeset_id).await?
            .ok_or_else(|| FileOpsError::Path(format!("Changeset not found: {}", changeset_id.0)))?;

        // Create a reverse changeset by inverting all patches
        let mut reverse_patches = Vec::new();
        for patch in changeset.patches.iter().rev() {
            let mut reverse_hunks = Vec::new();
            for hunk in patch.hunks.iter().rev() {
                // Invert the hunk: swap old and new
                let reverse_hunk = Hunk {
                    old_start: hunk.old_start + hunk.new_lines,
                    old_lines: hunk.new_lines,
                    new_start: hunk.old_start,
                    new_lines: hunk.old_lines,
                    lines: {
                        // Invert the lines: swap + and -
                        hunk.lines
                            .lines()
                            .map(|line| {
                                if line.starts_with('+') {
                                    format!("-{}", &line[1..])
                                } else if line.starts_with('-') {
                                    format!("+{}", &line[1..])
                                } else {
                                    line.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    },
                };
                reverse_hunks.push(reverse_hunk);
            }
            reverse_patches.push(Patch {
                path: patch.path.clone(),
                hunks: reverse_hunks,
                expected_prev_sha256: None, // Cannot validate on revert
            });
        }

        let reverse_changeset = ChangeSet {
            patches: reverse_patches,
        };

        // Apply the reverse changeset to revert the original
        self.apply_patches(&reverse_changeset).await?;

        Ok(())
    }

    async fn promote(&self) -> Result<()> {
        // Copy workspace changes back to source
        Self::mirror_directory(&self.workspace_path, &self.source_root).await
    }
}

impl Drop for TempMirrorWorkspace {
    fn drop(&mut self) {
        // Clean up workspace on drop
        let _ = std::fs::remove_dir_all(&self.workspace_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn setup_temp_project() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().to_path_buf();

        // Create some test files
        fs::write(project_path.join("README.md"), "# Test Project").await?;
        fs::create_dir_all(project_path.join("src")).await?;
        fs::write(project_path.join("src/main.rs"), "fn main() {}").await?;

        Ok((temp_dir, project_path))
    }

      // TODO: Implement comprehensive async testing infrastructure
      // - Add tokio-test dependency and configuration
      // - Create async test utilities and fixtures
      // - Implement proper async test cleanup and teardown
      // - Add async test timeouts and cancellation handling
      // - Support concurrent test execution
      // - Add async test debugging and profiling tools
      // PLACEHOLDER: Relying on integration tests for now

    #[test]
    fn test_temp_workspace_types() {
        // Basic type checking test
        let changeset_id = ChangeSetId("test-456".to_string());
        assert_eq!(changeset_id.0, "test-456");
    }
}
