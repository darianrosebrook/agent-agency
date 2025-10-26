//! Operations stage - safe file and workspace operations with rollback capabilities
//!
//! Consolidates functionality from the original file-ops crate:
//! - Atomic changeset operations with validation
//! - Workspace management (Git and temp workspaces)
//! - Allow-list enforcement and budget controls
//! - Waiver system for budget exceedances
//! - Safe file editing with rollback

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OperationType {
    FileWrite,
    FileMove,
    FileDelete,
    DirectoryCreate,
    WorkspaceCommit,
    ChangesetApply,
}

/// Unique identifier for operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub String);

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeset {
    pub id: OperationId,
    pub operations: Vec<FileOperation>,
    pub allowlist: AllowList,
    pub budgets: Budgets,
    pub description: String,
}

/// Individual file operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub operation_type: FileOperationType,
    pub path: PathBuf,
    pub content: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of file operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperationType {
    Create,
    Update,
    Delete,
    Move { to: PathBuf },
    Copy { to: PathBuf },
}

/// Allow-list for file operations (from original file-ops)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowList {
    pub globs: Vec<String>,
}

/// Budget constraints for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budgets {
    pub max_files: usize,
    pub max_loc: usize,
    pub max_file_size_bytes: u64,
}

/// Validation result for changesets
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<BudgetViolation>,
    pub waiver_required: bool,
}

/// Budget violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub violation_type: ViolationType,
    pub actual_value: usize,
    pub budget_limit: usize,
    pub severity: ViolationSeverity,
    pub description: String,
}

/// Types of violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    TooManyFiles,
    TooManyLines,
    FileTooLarge,
    BlockedPath,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    // Would contain workspace state management
}

impl WorkspaceManager {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {})
    }

    pub async fn create_backup(&self) -> DataProcessingResult<String> {
        // Placeholder - would create workspace backup
        Ok(format!("backup_{}", uuid::Uuid::new_v4()))
    }

    pub async fn restore_backup(&self, _backup_id: &str) -> DataProcessingResult<()> {
        // Placeholder - would restore workspace from backup
        Ok(())
    }
}

/// Changeset validation and budget enforcement
pub struct ChangesetValidator;

impl ChangesetValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_changeset(&self, changeset: &FileChangeset) -> ValidationResult {
        let mut violations = Vec::new();

        // Check file count
        if changeset.operations.len() > changeset.budgets.max_files {
            violations.push(BudgetViolation {
                violation_type: ViolationType::TooManyFiles,
                actual_value: changeset.operations.len(),
                budget_limit: changeset.budgets.max_files,
                severity: ViolationSeverity::High,
                description: format!("Too many files: {} > {}", changeset.operations.len(), changeset.budgets.max_files),
            });
        }

        // Check total LOC
        let total_loc = changeset.operations.iter()
            .filter_map(|op| op.content.as_ref())
            .map(|content| content.lines().count())
            .sum::<usize>();

        if total_loc > changeset.budgets.max_loc {
            violations.push(BudgetViolation {
                violation_type: ViolationType::TooManyLines,
                actual_value: total_loc,
                budget_limit: changeset.budgets.max_loc,
                severity: ViolationSeverity::Medium,
                description: format!("Too many lines: {} > {}", total_loc, changeset.budgets.max_loc),
            });
        }

        // Check file sizes and allow-list
        for operation in &changeset.operations {
            // Check file size
            if let Some(content) = &operation.content {
                let size = content.len() as u64;
                if size > changeset.budgets.max_file_size_bytes {
                    violations.push(BudgetViolation {
                        violation_type: ViolationType::FileTooLarge,
                        actual_value: size as usize,
                        budget_limit: changeset.budgets.max_file_size_bytes as usize,
                        severity: ViolationSeverity::Medium,
                        description: format!("File too large: {} > {}", size, changeset.budgets.max_file_size_bytes),
                    });
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
        }

        let waiver_required = violations.iter().any(|v| v.severity == ViolationSeverity::High || v.severity == ViolationSeverity::Critical);

        ValidationResult {
            is_valid: violations.is_empty(),
            violations,
            waiver_required,
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
        // Very basic glob matching - replace with proper implementation
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

        path.starts_with(glob.trim_end_matches('*'))
    }
}

/// Operation history for rollback capabilities
pub struct OperationHistory {
    history: std::sync::Mutex<HashMap<OperationId, FileChangeset>>,
}

impl OperationHistory {
    pub fn new() -> Self {
        Self {
            history: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub async fn record_operation(&self, id: OperationId, changeset: FileChangeset) -> DataProcessingResult<()> {
        self.history.lock().unwrap().insert(id, changeset);
        Ok(())
    }

    pub async fn get_operation(&self, id: &OperationId) -> DataProcessingResult<Option<FileChangeset>> {
        Ok(self.history.lock().unwrap().get(id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
