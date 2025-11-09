//! Operations stage - safe file and workspace operations with rollback capabilities
//!
//! Consolidates functionality from the original file-ops crate:
//! - Atomic changeset operations with validation
//! - Workspace management (Git and temp workspaces)
//! - Allow-list enforcement and budget controls
//! - Waiver system for budget exceedances
//! - Safe file editing with rollback

use schemars::JsonSchema;
use crate::data_processing_types::*;
use crate::DataProcessingResult;
use crate::DataProcessingError;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Result from operations
pub type OperationResult = DataProcessingResult<ProcessingOutput>;

/// Stage for file and workspace operations
#[async_trait]
pub trait OperationsStage: Send + Sync {
    /// Get the name of this operations stage
    fn name(&self) -> &'static str;

    /// Execute file operations for processed data
    async fn execute_operations(&self, input: DataInput, content: ProcessedContent) -> OperationResult;

    /// Validate and apply file changesets
    async fn apply_changeset(&self, changeset: &FileChangeset) -> DataProcessingResult<OperationId>;

    /// Rollback operations to previous state
    async fn rollback_operations(&self, operation_id: &OperationId) -> DataProcessingResult<()>;

    /// Get supported operation types
    fn supported_operations(&self) -> &[OperationType];
}

/// Types of operations supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
pub enum OperationType {
    FileWrite,
    FileMove,
    FileDelete,
    DirectoryCreate,
    WorkspaceCommit,
    ChangesetApply,
}

/// Unique identifier for operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct OperationId (pub String);

impl OperationId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Changeset of file operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileChangeset {
    pub id: OperationId,
    pub operations: Vec<FileOperation>,
    pub allowlist: AllowList,
    pub budgets: Budgets,
    pub description: String,
}

/// Individual file operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileOperation {
    pub operation_type: FileOperationType,
    pub path: PathBuf,
    pub content: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of file operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FileOperationType {
    Create,
    Update,
    Delete,
    Move { to: PathBuf },
    Copy { to: PathBuf },
}

/// Allow-list for file operations (from original file-ops)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AllowList {
    pub globs: Vec<String>,
}

/// Budget constraints for operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Budgets {
    pub max_files: usize,
    pub max_loc: usize,
    pub max_file_size_bytes: u64,
}

/// Validation result for changesets
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<BudgetViolation>,
    pub waiver_required: bool,
}

/// Budget violation details
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetViolation {
    pub violation_type: ViolationType,
    pub actual_value: usize,
    pub budget_limit: usize,
    pub severity: ViolationSeverity,
    pub description: String,
}

/// Types of violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ViolationType {
    TooManyFiles,
    TooManyLines,
    FileTooLarge,
    BlockedPath,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Default implementation combining all operation capabilities
pub struct DefaultOperationsStage {
    workspace_manager: WorkspaceManager,
    changeset_validator: ChangesetValidator,
    operation_history: OperationHistory,
}

impl DefaultOperationsStage {
    /// Create a new default operations stage
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            workspace_manager: WorkspaceManager::new().await?,
            changeset_validator: ChangesetValidator::new(),
            operation_history: OperationHistory::new(),
        })
    }
}

#[async_trait]
impl OperationsStage for DefaultOperationsStage {
    fn name(&self) -> &'static str {
        "default_operations"
    }

    async fn execute_operations(&self, input: DataInput, content: ProcessedContent) -> OperationResult {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();

        // Create operations based on processed content
        let operations = self.create_operations_from_content(&input, &content);

        // Create changeset
        let changeset = FileChangeset {
            id: OperationId::new(),
            operations,
            allowlist: AllowList {
                globs: vec![
                    "docs/**/*.md".to_string(),
                    "iterations/v3/**/*.rs".to_string(),
                    "models/**/*.json".to_string(),
                ],
            },
            budgets: Budgets {
                max_files: 10,
                max_loc: 1000,
                max_file_size_bytes: 1024 * 1024, // 1MB
            },
            description: format!("Data processing operations for {}", input.id),
        };

        // Validate changeset
        let validation = self.changeset_validator.validate_changeset(&changeset);

        if !validation.is_valid && !validation.waiver_required {
            return Err(DataProcessingError::Validation(format!(
                "Changeset validation failed: {} violations",
                validation.violations.len()
            )));
        }

        // Apply changeset
        match self.apply_changeset(&changeset).await {
            Ok(operation_id) => {
                // Record in history
                if let Err(e) = self.operation_history.record_operation(operation_id.clone(), changeset.clone()).await {
                    errors.push(format!("Failed to record operation in history: {}", e));
                }

                // Create metadata
                let mut metadata = input.metadata.clone();
                metadata.insert("operations_applied".to_string(), changeset.operations.len().into());
                metadata.insert("operation_id".to_string(), operation_id.0.into());
                metadata.insert("changeset_validated".to_string(), validation.is_valid.into());

                let stats = ProcessingStats {
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    bytes_processed: 0, // File operations don't process bytes in the same way
                    entities_extracted: 0,
                    relationships_found: 0,
                    embeddings_generated: 0,
                    errors_encountered: errors,
                };

                Ok(ProcessingOutput {
                    id: input.id.clone(),
                    original_input: input,
                    processed_content: content,
                    extracted_metadata: metadata,
                    processing_stats: stats,
                    created_at: chrono::Utc::now(),
                })
            }
            Err(e) => {
                errors.push(format!("Failed to apply changeset: {}", e));
                Err(DataProcessingError::Other(format!("Operation execution failed: {}", e)))
            }
        }
    }

    async fn apply_changeset(&self, changeset: &FileChangeset) -> DataProcessingResult<OperationId> {
        // Create backup before applying changes
        let backup_id = match self.workspace_manager.create_backup().await {
            Ok(id) => id,
            Err(e) => return Err(DataProcessingError::Other(format!("Failed to create backup: {}", e))),
        };

        // Apply each operation
        for operation in &changeset.operations {
            match self.execute_file_operation(operation).await {
                Ok(_) => {}
                Err(e) => {
                    // Rollback on failure
                    let _ = self.workspace_manager.restore_backup(&backup_id).await;
                    return Err(e);
                }
            }
        }

        Ok(changeset.id.clone())
    }

    async fn rollback_operations(&self, operation_id: &OperationId) -> DataProcessingResult<()> {
        if let Some(_changeset) = self.operation_history.get_operation(operation_id).await? {
            // Find the backup for this operation
            let backup_id = format!("backup_{}", operation_id.0);
            self.workspace_manager.restore_backup(&backup_id).await?;
        }

        Ok(())
    }

    fn supported_operations(&self) -> &[OperationType] {
        &[
            OperationType::FileWrite,
            OperationType::FileMove,
            OperationType::FileDelete,
            OperationType::DirectoryCreate,
            OperationType::WorkspaceCommit,
            OperationType::ChangesetApply,
        ]
    }
}

#[async_trait]
impl crate::pipeline::PipelineStage for DefaultOperationsStage {
    fn name(&self) -> &'static str {
        "operations"
    }

    async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
        // For operations, we expect indexed content
        let processed_content = match &input.content {
            DataContent::Structured(data) => {
                // Try to deserialize as ProcessedContent
                match serde_json::from_value(data.clone()) {
                    Ok(content) => content,
                    Err(_) => return Err(DataProcessingError::Validation(
                        "Expected ProcessedContent in structured data".to_string()
                    )),
                }
            }
            _ => return Err(DataProcessingError::Validation(
                "Operations stage expects structured content".to_string()
            )),
        };

        self.execute_operations(input, processed_content).await
    }
}

impl DefaultOperationsStage {
    /// Create file operations from processed content
    fn create_operations_from_content(&self, input: &DataInput, content: &ProcessedContent) -> Vec<FileOperation> {
        let mut operations = Vec::new();

        // Create documentation files based on extracted entities and relationships
        if !content.entities.is_empty() {
            let entities_file = format!("docs/entities/{}.json", input.id.0);
            let entities_content = serde_json::to_string_pretty(&content.entities)
                .unwrap_or_else(|_| "[]".to_string());

            operations.push(FileOperation {
                operation_type: FileOperationType::Create,
                path: PathBuf::from(entities_file),
                content: Some(entities_content),
                metadata: HashMap::from([
                    ("content_type".to_string(), "entities".into()),
                    ("entity_count".to_string(), content.entities.len().into()),
                ]),
            });
        }

        if !content.relationships.is_empty() {
            let relationships_file = format!("docs/relationships/{}.json", input.id.0);
            let relationships_content = serde_json::to_string_pretty(&content.relationships)
                .unwrap_or_else(|_| "[]".to_string());

            operations.push(FileOperation {
                operation_type: FileOperationType::Create,
                path: PathBuf::from(relationships_file),
                content: Some(relationships_content),
                metadata: HashMap::from([
                    ("content_type".to_string(), "relationships".into()),
                    ("relationship_count".to_string(), content.relationships.len().into()),
                ]),
            });
        }

        // Create summary file
        let summary_file = format!("docs/summaries/{}.md", input.id.0);
        let summary_content = self.generate_summary_markdown(input, content);

        operations.push(FileOperation {
            operation_type: FileOperationType::Create,
            path: PathBuf::from(summary_file),
            content: Some(summary_content),
            metadata: HashMap::from([
                ("content_type".to_string(), "summary".into()),
                ("format".to_string(), "markdown".into()),
            ]),
        });

        operations
    }

    /// Generate summary markdown from processed content
    fn generate_summary_markdown(&self, input: &DataInput, content: &ProcessedContent) -> String {
        let mut markdown = String::new();

        markdown.push_str(&format!("# Processing Summary: {}\n\n", input.id.0));
        markdown.push_str(&format!("**Source:** {:?}\n\n", input.source));
        markdown.push_str(&format!("**Processed At:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        if let Some(text) = &content.text_content {
            markdown.push_str("## Content Preview\n\n");
            let preview = if text.len() > 500 {
                format!("{}...", &text[..500])
            } else {
                text.clone()
            };
            markdown.push_str(&format!("{}\n\n", preview));
        }

        if !content.entities.is_empty() {
            markdown.push_str("## Extracted Entities\n\n");
            for entity in &content.entities {
                markdown.push_str(&format!("- **{}** ({:?}, {:.1}% confidence)\n",
                    entity.name, entity.entity_type, entity.confidence * 100.0));
            }
            markdown.push_str("\n");
        }

        if !content.relationships.is_empty() {
            markdown.push_str("## Relationships Found\n\n");
            for relationship in &content.relationships {
                markdown.push_str(&format!("- {} --({})--> {}\n",
                    relationship.source_entity,
                    relationship.relationship_type,
                    relationship.target_entity));
            }
            markdown.push_str("\n");
        }

        if let Some(transcript) = &content.audio_transcript {
            markdown.push_str("## Audio Transcript\n\n");
            markdown.push_str(&format!("{}\n\n", transcript));
        }

        markdown.push_str(&format!("## Statistics\n\n"));
        markdown.push_str(&format!("- Entities: {}\n", content.entities.len()));
        markdown.push_str(&format!("- Relationships: {}\n", content.relationships.len()));
        markdown.push_str(&format!("- Visual Elements: {}\n", content.visual_elements.len()));

        markdown
    }

    /// Execute a single file operation
    async fn execute_file_operation(&self, operation: &FileOperation) -> DataProcessingResult<()> {
        match &operation.operation_type {
            FileOperationType::Create => {
                if let Some(content) = &operation.content {
                    // Ensure parent directory exists
                    if let Some(parent) = operation.path.parent() {
                        tokio::fs::create_dir_all(parent).await
                            .map_err(|e| DataProcessingError::Io(e))?;
                    }

                    tokio::fs::write(&operation.path, content).await
                        .map_err(|e| DataProcessingError::Io(e))?;
                }
            }

            FileOperationType::Update => {
                if let Some(content) = &operation.content {
                    tokio::fs::write(&operation.path, content).await
                        .map_err(|e| DataProcessingError::Io(e))?;
                }
            }

            FileOperationType::Delete => {
                if operation.path.exists() {
                    if operation.path.is_dir() {
                        tokio::fs::remove_dir_all(&operation.path).await
                            .map_err(|e| DataProcessingError::Io(e))?;
                    } else {
                        tokio::fs::remove_file(&operation.path).await
                            .map_err(|e| DataProcessingError::Io(e))?;
                    }
                }
            }

            FileOperationType::Move { to } => {
                tokio::fs::rename(&operation.path, to).await
                    .map_err(|e| DataProcessingError::Io(e))?;
            }

            FileOperationType::Copy { to } => {
                // Ensure parent directory exists
                if let Some(parent) = to.parent() {
                    tokio::fs::create_dir_all(parent).await
                        .map_err(|e| DataProcessingError::Io(e))?;
                }

                tokio::fs::copy(&operation.path, to).await
                    .map_err(|_| DataProcessingError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other, "Copy failed"
                    )))?;
            }
        }

        Ok(())
    }
}

/// Workspace management for safe operations
pub struct WorkspaceManager {
    workspace_root: PathBuf,
    backup_dir: PathBuf,
}

impl WorkspaceManager {
    pub async fn new() -> DataProcessingResult<Self> {
        let workspace_root = std::env::current_dir()
            .map_err(|e| DataProcessingError::Io(e))?;
        
        let backup_dir = workspace_root.join(".backups");
        
        // Ensure backup directory exists
        tokio::fs::create_dir_all(&backup_dir).await
            .map_err(|e| DataProcessingError::Io(e))?;

        Ok(Self {
            workspace_root,
            backup_dir,
        })
    }

    pub async fn create_backup(&self) -> DataProcessingResult<String> {
        let backup_id = format!("backup_{}", uuid::Uuid::new_v4());
        let backup_path = self.backup_dir.join(&backup_id);
        
        // Create backup directory
        tokio::fs::create_dir_all(&backup_path).await
            .map_err(|e| DataProcessingError::Io(e))?;

        // Create a manifest file with backup metadata
        let manifest = serde_json::json!({
            "backup_id": backup_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "workspace_root": self.workspace_root.to_string_lossy(),
            "backup_type": "incremental"
        });

        let manifest_path = backup_path.join("manifest.json");
        tokio::fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?).await
            .map_err(|e| DataProcessingError::Io(e))?;

        // Create a snapshot of current workspace state
        let state_path = backup_path.join("workspace_state.json");
        let workspace_state = self.capture_workspace_state().await?;
        tokio::fs::write(state_path, serde_json::to_string_pretty(&workspace_state)?).await
            .map_err(|e| DataProcessingError::Io(e))?;

        Ok(backup_id)
    }

    pub async fn restore_backup(&self, backup_id: &str) -> DataProcessingResult<()> {
        let backup_path = self.backup_dir.join(backup_id);
        
        if !backup_path.exists() {
            return Err(DataProcessingError::NotFound(format!("Backup {} not found", backup_id)));
        }

        // Read the manifest to verify backup
        let manifest_path = backup_path.join("manifest.json");
        let manifest_content = tokio::fs::read_to_string(manifest_path).await
            .map_err(|e| DataProcessingError::Io(e))?;
        
        let _manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        // Read workspace state
        let state_path = backup_path.join("workspace_state.json");
        let state_content = tokio::fs::read_to_string(state_path).await
            .map_err(|e| DataProcessingError::Io(e))?;
        
        let workspace_state: serde_json::Value = serde_json::from_str(&state_content)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        // Restore workspace state
        self.restore_workspace_state(workspace_state).await?;

        Ok(())
    }

    /// Capture current workspace state
    async fn capture_workspace_state(&self) -> DataProcessingResult<serde_json::Value> {
        let mut state = serde_json::Map::new();
        
        // Capture directory structure
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        
        self.scan_directory(&self.workspace_root, &mut dirs, &mut files).await?;
        
        state.insert("directories".to_string(), serde_json::to_value(dirs)?);
        state.insert("files".to_string(), serde_json::to_value(files)?);
        state.insert("captured_at".to_string(), chrono::Utc::now().to_rfc3339().into());
        state.insert("workspace_root".to_string(), self.workspace_root.to_string_lossy().into());
        
        Ok(serde_json::Value::Object(state))
    }

    /// Scan directory recursively
    async fn scan_directory(
        &self,
        dir: &Path,
        dirs: &mut Vec<String>,
        files: &mut Vec<serde_json::Value>,
    ) -> DataProcessingResult<()> {
        use std::collections::VecDeque;
        
        let mut queue = VecDeque::new();
        queue.push_back(dir.to_path_buf());

        while let Some(current_dir) = queue.pop_front() {
            let mut entries = tokio::fs::read_dir(&current_dir).await
                .map_err(|e| DataProcessingError::Io(e))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| DataProcessingError::Io(e))? {
                
                let path = entry.path();
                let relative_path = path.strip_prefix(&self.workspace_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                if path.is_dir() {
                    // Skip hidden directories and common build artifacts
                    if !relative_path.starts_with('.') && 
                       !relative_path.contains("target/") &&
                       !relative_path.contains("node_modules/") {
                        dirs.push(relative_path);
                        queue.push_back(path);
                    }
                } else {
                    // Capture file metadata
                    let metadata = entry.metadata().await
                        .map_err(|e| DataProcessingError::Io(e))?;
                    
                    let file_info = serde_json::json!({
                        "path": relative_path,
                        "size": metadata.len(),
                        "modified": metadata.modified()
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        "is_readonly": metadata.permissions().readonly()
                    });
                    
                    files.push(file_info);
                }
            }
        }

        Ok(())
    }

    /// Restore workspace state
    async fn restore_workspace_state(&self, _state: serde_json::Value) -> DataProcessingResult<()> {
        // TODO: Implement workspace state restoration
        //       Currently placeholder; should compare current state with backup, restore files, and update metadata.
        //
        // COMPLETION CHECKLIST:
        // [ ] Compare current state with backup state
        // [ ] Restore modified/deleted files from backup
        // [ ] Remove files that shouldn't exist in target state
        // [ ] Update file permissions and timestamps to match backup
        // [ ] Handle restoration conflicts and errors
        // [ ] Verify restoration completeness
        // [ ] Add unit tests with various state scenarios
        // [ ] Add integration tests with real workspace restoration
        // [ ] Performance: Restoration should complete in <30s
        // [ ] Documentation: Document restoration process
        //
        // ACCEPTANCE CRITERIA:
        // - Workspace state matches backup state after restoration
        // - Files are restored correctly from backup
        // - File permissions and timestamps are preserved
        // - Restoration conflicts are handled appropriately
        // - Restoration is verified for completeness
        //
        // DEPENDENCIES:
        // - Backup storage access (Required)
        // - File system operations (Required)
        // - State comparison logic (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (data integrity feature)
        // - Change Budget: ~300 LOC
        // - Reviewer Requirements: File system and backup expertise
        // 3. Remove files that shouldn't exist
        // 4. Update file permissions and timestamps
        //
        // TODO: Implement comprehensive workspace state restoration
        //       Currently logs restoration intent only; should implement comprehensive workspace state restoration that restores files from backup, removes files that shouldn't exist, and updates file permissions and timestamps for complete state recovery.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Files are restored from backup correctly
        // - Files that shouldn't exist are removed
        // - File permissions and timestamps are updated
        // - Restoration process is atomic and handles errors gracefully
        //
        // DEPENDENCIES:
        // - Backup storage system (Required)
        // - File system operations utilities (Required)
        // - Permission and timestamp management (Required)
        //
        // ESTIMATED EFFORT: 12-16 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (workspace state management functionality)
        // - Change Budget: ~300 LOC
        // - Reviewer Requirements: File system and backup restoration expertise
        tracing::info!("Workspace state restoration simulated for backup");
        
        Ok(())
    }
}

/// Changeset validation and budget enforcement
pub struct ChangesetValidator {
    default_budgets: Budgets,
    strict_mode: bool,
}

impl ChangesetValidator {
    pub fn new() -> Self {
        Self {
            default_budgets: Budgets {
                max_files: 50,
                max_loc: 5000,
                max_file_size_bytes: 10 * 1024 * 1024, // 10MB
            },
            strict_mode: false,
        }
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn with_default_budgets(mut self, budgets: Budgets) -> Self {
        self.default_budgets = budgets;
        self
    }

    pub fn validate_changeset(&self, changeset: &FileChangeset) -> ValidationResult {
        let mut violations = Vec::new();
        let budgets = &changeset.budgets;

        // Check file count
        if changeset.operations.len() > budgets.max_files {
            violations.push(BudgetViolation {
                violation_type: ViolationType::TooManyFiles,
                actual_value: changeset.operations.len(),
                budget_limit: budgets.max_files,
                severity: ViolationSeverity::High,
                description: format!("Too many files: {} > {}", changeset.operations.len(), budgets.max_files),
            });
        }

        // Check total LOC
        let total_loc = changeset.operations.iter()
            .filter_map(|op| op.content.as_ref())
            .map(|content| content.lines().count())
            .sum::<usize>();

        if total_loc > budgets.max_loc {
            violations.push(BudgetViolation {
                violation_type: ViolationType::TooManyLines,
                actual_value: total_loc,
                budget_limit: budgets.max_loc,
                severity: ViolationSeverity::Medium,
                description: format!("Too many lines: {} > {}", total_loc, budgets.max_loc),
            });
        }

        // Check file sizes and allow-list
        for operation in &changeset.operations {
            // Check file size
            if let Some(content) = &operation.content {
                let size = content.len() as u64;
                if size > budgets.max_file_size_bytes {
                    violations.push(BudgetViolation {
                        violation_type: ViolationType::FileTooLarge,
                        actual_value: size as usize,
                        budget_limit: budgets.max_file_size_bytes as usize,
                        severity: ViolationSeverity::Medium,
                        description: format!("File too large: {} > {}", size, budgets.max_file_size_bytes),
                    });
                }

                // Additional content validation in strict mode
                if self.strict_mode {
                    self.validate_content_quality(content, &mut violations);
                }
            }

            // Check allow-list
            if !self.is_path_allowed(&operation.path, &changeset.allowlist) {
                violations.push(BudgetViolation {
                    violation_type: ViolationType::BlockedPath,
                    actual_value: 1,
                    budget_limit: 0,
                    severity: ViolationSeverity::Critical,
                    description: format!("Path not allowed: {}", operation.path.display()),
                });
            }

            // Validate operation type specific rules
            self.validate_operation_type(operation, &mut violations);
        }

        // Check for potential security issues
        self.check_security_concerns(changeset, &mut violations);

        let waiver_required = violations.iter().any(|v| v.severity == ViolationSeverity::High || v.severity == ViolationSeverity::Critical);

        ValidationResult {
            is_valid: violations.is_empty(),
            violations,
            waiver_required,
        }
    }

    /// Validate content quality in strict mode
    fn validate_content_quality(&self, content: &str, violations: &mut Vec<BudgetViolation>) {
        // Check for suspicious patterns
        let suspicious_patterns = [
            ("exec(", "Potential code execution"),
            ("eval(", "Potential code evaluation"),
            ("system(", "System command execution"),
            ("shell_exec(", "Shell command execution"),
            ("<script", "Potential XSS"),
            ("javascript:", "Potential XSS"),
            ("data:text/html", "Potential XSS"),
        ];

        for (pattern, description) in &suspicious_patterns {
            if content.contains(pattern) {
                violations.push(BudgetViolation {
                    violation_type: ViolationType::BlockedPath, // Reuse for security
                    actual_value: 1,
                    budget_limit: 0,
                    severity: ViolationSeverity::High,
                    description: format!("Security concern: {}", description),
                });
            }
        }

        // Check for very long lines (potential obfuscation)
        let long_lines = content.lines().filter(|line| line.len() > 1000).count();
        if long_lines > 0 {
            violations.push(BudgetViolation {
                violation_type: ViolationType::TooManyLines, // Reuse
                actual_value: long_lines,
                budget_limit: 0,
                severity: ViolationSeverity::Low,
                description: format!("Found {} very long lines (>1000 chars)", long_lines),
            });
        }
    }

    /// Validate operation type specific rules
    fn validate_operation_type(&self, operation: &FileOperation, violations: &mut Vec<BudgetViolation>) {
        match &operation.operation_type {
            FileOperationType::Delete => {
                // Check if deleting critical files
                let critical_paths = [
                    "Cargo.toml",
                    "package.json",
                    "README.md",
                    ".gitignore",
                ];

                for critical in &critical_paths {
                    if operation.path.ends_with(critical) {
                        violations.push(BudgetViolation {
                            violation_type: ViolationType::BlockedPath,
                            actual_value: 1,
                            budget_limit: 0,
                            severity: ViolationSeverity::High,
                            description: format!("Attempting to delete critical file: {}", critical),
                        });
                    }
                }
            }
            FileOperationType::Move { to } | FileOperationType::Copy { to } => {
                // Check if moving/copying to restricted locations
                if to.starts_with("/") || to.starts_with("C:\\") {
                    violations.push(BudgetViolation {
                        violation_type: ViolationType::BlockedPath,
                        actual_value: 1,
                        budget_limit: 0,
                        severity: ViolationSeverity::Critical,
                        description: format!("Attempting to move/copy to absolute path: {}", to.display()),
                    });
                }
            }
            _ => {}
        }
    }

    /// Check for security concerns
    fn check_security_concerns(&self, changeset: &FileChangeset, violations: &mut Vec<BudgetViolation>) {
        // Check for operations on sensitive directories
        let sensitive_dirs = [
            ".git",
            ".env",
            "node_modules",
            "target",
            ".cargo",
        ];

        for operation in &changeset.operations {
            for sensitive in &sensitive_dirs {
                if operation.path.to_string_lossy().contains(sensitive) {
                    violations.push(BudgetViolation {
                        violation_type: ViolationType::BlockedPath,
                        actual_value: 1,
                        budget_limit: 0,
                        severity: ViolationSeverity::High,
                        description: format!("Operation on sensitive directory: {}", sensitive),
                    });
                }
            }
        }

        // Check for operations that could affect system files
        let system_patterns = [
            "/etc/",
            "/usr/",
            "/bin/",
            "/sbin/",
            "C:\\Windows\\",
            "C:\\Program Files\\",
        ];

        for operation in &changeset.operations {
            for pattern in &system_patterns {
                if operation.path.to_string_lossy().contains(pattern) {
                    violations.push(BudgetViolation {
                        violation_type: ViolationType::BlockedPath,
                        actual_value: 1,
                        budget_limit: 0,
                        severity: ViolationSeverity::Critical,
                        description: format!("Operation on system directory: {}", pattern),
                    });
                }
            }
        }
    }

    fn is_path_allowed(&self, path: &Path, allowlist: &AllowList) -> bool {
        let path_str = path.to_string_lossy();

        for glob in &allowlist.globs {
            // Simple glob matching - would use proper glob library in production
            if self.matches_simple_glob(&path_str, glob) {
                return true;
            }
        }

        false
    }

    fn matches_simple_glob(&self, path: &str, glob: &str) -> bool {
        // Enhanced glob matching with more patterns
        if glob.contains("**") {
            let parts: Vec<&str> = glob.split("**").collect();
            if parts.len() >= 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                return path.starts_with(prefix) &&
                       (suffix.is_empty() || path.ends_with(suffix) ||
                        path.contains(&format!("/{}", suffix)));
            }
        }

        // Handle single asterisk wildcards
        if glob.contains('*') && !glob.contains("**") {
            let parts: Vec<&str> = glob.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        }

        // Exact match
        path == glob || path.starts_with(glob.trim_end_matches('*'))
    }
}

/// Operation history for rollback capabilities
pub struct OperationHistory {
    history: std::sync::Mutex<HashMap<OperationId, FileChangeset>>,
    history_file: PathBuf,
    max_history_size: usize,
}

impl OperationHistory {
    pub fn new() -> Self {
        let history_file = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".operation_history.json");

        Self {
            history: std::sync::Mutex::new(HashMap::new()),
            history_file,
            max_history_size: 1000, // Keep last 1000 operations
        }
    }

    pub fn with_history_file(mut self, file: PathBuf) -> Self {
        self.history_file = file;
        self
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_history_size = max_size;
        self
    }

    pub async fn record_operation(&self, id: OperationId, changeset: FileChangeset) -> DataProcessingResult<()> {
        // Add to in-memory history
        {
            let mut history = self.history.lock().unwrap();
            
            // Check if we need to clean up old entries
            if history.len() >= self.max_history_size {
                self.cleanup_old_entries(&mut history);
            }
            
            history.insert(id.clone(), changeset.clone());
        }

        // Persist to disk
        self.save_history_to_disk().await?;

        Ok(())
    }

    pub async fn get_operation(&self, id: &OperationId) -> DataProcessingResult<Option<FileChangeset>> {
        // Try in-memory first
        {
            let history = self.history.lock().unwrap();
            if let Some(changeset) = history.get(id) {
                return Ok(Some(changeset.clone()));
            }
        }

        // If not in memory, try to load from disk
        self.load_history_from_disk().await?;
        
        // Try again after loading
        let history = self.history.lock().unwrap();
        Ok(history.get(id).cloned())
    }

    /// Load history from disk
    async fn load_history_from_disk(&self) -> DataProcessingResult<()> {
        if !self.history_file.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.history_file).await
            .map_err(|e| DataProcessingError::Io(e))?;

        let history_data: HashMap<String, FileChangeset> = serde_json::from_str(&content)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        let mut history = self.history.lock().unwrap();
        for (id_str, changeset) in history_data {
            let id = OperationId(id_str);
            history.insert(id, changeset);
        }

        Ok(())
    }

    /// Save history to disk
    async fn save_history_to_disk(&self) -> DataProcessingResult<()> {
        // Convert to serializable format
        let serializable_history: HashMap<String, FileChangeset> = {
            let history = self.history.lock().unwrap();
            let mut serializable_history = HashMap::new();
            for (id, changeset) in history.iter() {
                serializable_history.insert(id.0.clone(), changeset.clone());
            }
            serializable_history
        };

        let content = serde_json::to_string_pretty(&serializable_history)
            .map_err(|e| DataProcessingError::Serialization(e))?;

        // Ensure parent directory exists
        if let Some(parent) = self.history_file.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| DataProcessingError::Io(e))?;
        }

        tokio::fs::write(&self.history_file, content).await
            .map_err(|e| DataProcessingError::Io(e))?;

        Ok(())
    }

    /// Clean up old entries to maintain size limit
    fn cleanup_old_entries(&self, history: &mut HashMap<OperationId, FileChangeset>) {
        if history.len() <= self.max_history_size {
            return;
        }

        // Convert to vector and sort by operation ID (which contains timestamp)
        let mut entries: Vec<_> = history.drain().collect();
        
        // Sort by ID (assuming IDs are sortable)
        entries.sort_by(|a, b| a.0.0.cmp(&b.0.0));
        
        // Keep only the most recent entries
        let keep_count = self.max_history_size / 2; // Keep half when cleaning
        entries.truncate(keep_count);
        
        // Put back the kept entries
        for (id, changeset) in entries {
            history.insert(id, changeset);
        }
    }

    /// Get all operations (for debugging/admin purposes)
    pub async fn list_operations(&self) -> DataProcessingResult<Vec<OperationId>> {
        self.load_history_from_disk().await?;
        
        let history = self.history.lock().unwrap();
        Ok(history.keys().cloned().collect())
    }

    /// Clear all history
    pub async fn clear_history(&self) -> DataProcessingResult<()> {
        {
            let mut history = self.history.lock().unwrap();
            history.clear();
        }

        // Remove history file
        if self.history_file.exists() {
            tokio::fs::remove_file(&self.history_file).await
                .map_err(|e| DataProcessingError::Io(e))?;
        }

        Ok(())
    }

    /// Get history statistics
    pub async fn get_stats(&self) -> DataProcessingResult<HistoryStats> {
        self.load_history_from_disk().await?;
        
        let history = self.history.lock().unwrap();
        
        let total_operations = history.len();
        let mut total_files = 0;
        let mut total_loc = 0;
        
        for changeset in history.values() {
            total_files += changeset.operations.len();
            total_loc += changeset.operations.iter()
                .filter_map(|op| op.content.as_ref())
                .map(|content| content.lines().count())
                .sum::<usize>();
        }

        Ok(HistoryStats {
            total_operations,
            total_files_processed: total_files,
            total_loc_processed: total_loc,
            history_file_size: self.get_history_file_size().await?,
        })
    }

    /// Get the size of the history file
    async fn get_history_file_size(&self) -> DataProcessingResult<u64> {
        if self.history_file.exists() {
            let metadata = tokio::fs::metadata(&self.history_file).await
                .map_err(|e| DataProcessingError::Io(e))?;
            Ok(metadata.len())
        } else {
            Ok(0)
        }
    }
}

/// Statistics about operation history
#[derive(Debug, Clone, JsonSchema)]
pub struct HistoryStats {
    pub total_operations: usize,
    pub total_files_processed: usize,
    pub total_loc_processed: usize,
    pub history_file_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_operations_stage_creation() {
        let stage = DefaultOperationsStage::new().await;
        assert!(stage.is_ok());
    }

    #[test]
    fn test_changeset_validator() {
        let validator = ChangesetValidator::new();

        let changeset = FileChangeset {
            id: OperationId::new(),
            operations: vec![
                FileOperation {
                    operation_type: FileOperationType::Create,
                    path: PathBuf::from("docs/test.md"),
                    content: Some("# Test\n\nSome content.".to_string()),
                    metadata: HashMap::new(),
                }
            ],
            allowlist: AllowList {
                globs: vec!["docs/**/*.md".to_string()],
            },
            budgets: Budgets {
                max_files: 10,
                max_loc: 100,
                max_file_size_bytes: 1024,
            },
            description: "Test changeset".to_string(),
        };

        let result = validator.validate_changeset(&changeset);
        assert!(result.is_valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_changeset_validator_violations() {
        let validator = ChangesetValidator::new();

        let changeset = FileChangeset {
            id: OperationId::new(),
            operations: vec![
                FileOperation {
                    operation_type: FileOperationType::Create,
                    path: PathBuf::from("blocked.txt"),
                    content: Some("x".repeat(2000)), // Too large
                    metadata: HashMap::new(),
                },
                FileOperation {
                    operation_type: FileOperationType::Create,
                    path: PathBuf::from("docs/another.md"),
                    content: Some("line\n".repeat(200)), // Too many lines
                    metadata: HashMap::new(),
                },
            ],
            allowlist: AllowList {
                globs: vec!["docs/**/*.md".to_string()],
            },
            budgets: Budgets {
                max_files: 1, // Too many files
                max_loc: 100, // Too many lines
                max_file_size_bytes: 1000, // File too large
            },
            description: "Test changeset with violations".to_string(),
        };

        let result = validator.validate_changeset(&changeset);
        assert!(!result.is_valid);
        assert!(!result.violations.is_empty());
        assert!(result.waiver_required);
    }

    #[test]
    fn test_simple_glob_matching() {
        let validator = ChangesetValidator::new();

        assert!(validator.matches_simple_glob("docs/test.md", "docs/**/*.md"));
        assert!(validator.matches_simple_glob("docs/subdir/test.md", "docs/**/*.md"));
        assert!(!validator.matches_simple_glob("src/test.md", "docs/**/*.md"));
    }

    #[tokio::test]
    async fn test_operation_history() {
        let history = OperationHistory::new();

        let changeset = FileChangeset {
            id: OperationId("test_id".to_string()),
            operations: vec![],
            allowlist: AllowList { globs: vec![] },
            budgets: Budgets { max_files: 0, max_loc: 0, max_file_size_bytes: 0 },
            description: "Test".to_string(),
        };

        // Record operation
        history.record_operation(changeset.id.clone(), changeset.clone()).await.unwrap();

        // Retrieve operation
        let retrieved = history.get_operation(&changeset.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, changeset.id);
    }
}
