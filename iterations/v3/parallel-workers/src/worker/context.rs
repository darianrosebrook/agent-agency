//! Worker execution context and isolation

use crate::types::*;
use crate::error::{WorkerError, WorkerExecutionResult};

/// Execution context for a worker
#[derive(Debug, Clone)]
pub struct WorkerExecutionContext {
    pub subtask: SubTask,
    pub workspace_root: std::path::PathBuf,
    pub isolated_environment: std::collections::HashMap<String, String>,
    pub resource_limits: ResourceLimits,
    pub communication_channel: tokio::sync::mpsc::UnboundedSender<WorkerMessage>,
    pub cancellation_token: tokio_util::sync::CancellationToken,
}

impl WorkerExecutionContext {
    /// Create a new execution context
    pub fn new(
        subtask: SubTask,
        workspace_root: std::path::PathBuf,
        communication_channel: tokio::sync::mpsc::UnboundedSender<WorkerMessage>,
    ) -> Self {
        Self {
            isolated_environment: Self::create_isolated_environment(&subtask),
            resource_limits: ResourceLimits::from_subtask(&subtask),
            workspace_root,
            subtask,
            communication_channel,
            cancellation_token: tokio_util::sync::CancellationToken::new(),
        }
    }

    /// Send a progress update
    pub async fn send_progress(&self, completed: u32, total: u32, status: String) -> WorkerExecutionResult<()> {
        let message = WorkerMessage::Progress {
            worker_id: WorkerId("temp-worker-id".to_string()), // TODO: Pass worker ID
            subtask_id: self.subtask.id.clone(),
            completed,
            total,
            status,
            timestamp: chrono::Utc::now(),
        };

        self.communication_channel.send(message)
            .map_err(|_| WorkerError::Communication {
                message: "Failed to send progress message".to_string(),
            })
    }

    /// Send a completion message
    pub async fn send_completed(&self, result: WorkerResult) -> WorkerExecutionResult<()> {
        let message = WorkerMessage::Completed {
            worker_id: WorkerId("temp-worker-id".to_string()), // TODO: Pass worker ID
            subtask_id: self.subtask.id.clone(),
            result,
            timestamp: chrono::Utc::now(),
        };

        self.communication_channel.send(message)
            .map_err(|_| WorkerError::Communication {
                message: "Failed to send completion message".to_string(),
            })
    }

    /// Send a failure message
    pub async fn send_failed(&self, error: WorkerError) -> WorkerExecutionResult<()> {
        let message = WorkerMessage::Failed {
            worker_id: WorkerId("temp-worker-id".to_string()), // TODO: Pass worker ID
            subtask_id: self.subtask.id.clone(),
            error,
            recoverable: false, // TODO: Determine if recoverable
            timestamp: chrono::Utc::now(),
        };

        self.communication_channel.send(message)
            .map_err(|_| WorkerError::Communication {
                message: "Failed to send failure message".to_string(),
            })
    }

    /// Send a blockage message
    pub async fn send_blocked(&self, reason: BlockageReason, context: String) -> WorkerExecutionResult<()> {
        let message = WorkerMessage::Blocked {
            worker_id: WorkerId("temp-worker-id".to_string()), // TODO: Pass worker ID
            subtask_id: self.subtask.id.clone(),
            reason,
            context,
            timestamp: chrono::Utc::now(),
        };

        self.communication_channel.send(message)
            .map_err(|_| WorkerError::Communication {
                message: "Failed to send blockage message".to_string(),
            })
    }

    /// Check if execution should be cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Get the cancellation token
    pub fn cancellation_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancellation_token
    }

    /// Get working directory for the subtask
    pub fn working_directory(&self) -> &std::path::Path {
        &self.workspace_root
    }

    /// Get files included in the subtask scope
    pub fn files(&self) -> &[std::path::PathBuf] {
        &self.subtask.scope.files
    }

    /// Check if a file is in scope
    pub fn is_file_in_scope(&self, file_path: &std::path::Path) -> bool {
        // Check included files
        if self.subtask.scope.files.iter().any(|f| f == file_path) {
            return true;
        }

        // Check excluded files
        if self.subtask.scope.directories.iter().any(|f| f == file_path) {
            return false;
        }

        // Check patterns
        let file_str = file_path.to_string_lossy();

        // Check included patterns
        for pattern in &self.subtask.scope.patterns {
            if file_str.contains(pattern) {
                return true;
            }
        }

        // Check excluded patterns
        for pattern in &self.subtask.scope.patterns {
            if file_str.contains(pattern) {
                return false;
            }
        }

        // Default: not in scope
        false
    }

    /// Get time remaining for execution
    pub fn time_remaining(&self) -> Option<std::time::Duration> {
        // TODO: Track actual start time and calculate remaining time
        Some(std::time::Duration::from_secs(300)) // Default 5 minutes
    }

    /// Check if time limit exceeded
    pub fn is_time_limit_exceeded(&self) -> bool {
        self.time_remaining()
            .map(|remaining| remaining.as_secs() == 0)
            .unwrap_or(false)
    }

    /// Create isolated environment variables
    fn create_isolated_environment(subtask: &SubTask) -> std::collections::HashMap<String, String> {
        let mut env = std::collections::HashMap::new();

        // Basic isolation
        env.insert("WORKER_SUBTASK_ID".to_string(), subtask.id.0.clone());
        env.insert("WORKER_PARENT_TASK_ID".to_string(), subtask.parent_id.0.clone());
        env.insert("WORKER_PRIORITY".to_string(), format!("{:?}", subtask.priority));
        env.insert("WORKER_TIME_BUDGET_SECS".to_string(), "300".to_string()); // Default 5 minutes

        // Specialty-specific environment
        match &subtask.specialty {
            WorkerSpecialty::CompilationErrors { error_codes } => {
                env.insert("SPECIALTY_TYPE".to_string(), "compilation_errors".to_string());
                env.insert("COMPILATION_ERROR_CODES".to_string(), error_codes.join(","));
            }
            WorkerSpecialty::Refactoring { strategies } => {
                env.insert("SPECIALTY_TYPE".to_string(), "refactoring".to_string());
                env.insert("REFACTORING_STRATEGIES".to_string(), strategies.join(","));
            }
            WorkerSpecialty::Testing { frameworks } => {
                env.insert("SPECIALTY_TYPE".to_string(), "testing".to_string());
                env.insert("TEST_FRAMEWORKS".to_string(), frameworks.join(","));
            }
            WorkerSpecialty::Documentation { formats } => {
                env.insert("SPECIALTY_TYPE".to_string(), "documentation".to_string());
                env.insert("DOC_FORMATS".to_string(), formats.join(","));
            }
            WorkerSpecialty::TypeSystem { domains } => {
                env.insert("SPECIALTY_TYPE".to_string(), "type_system".to_string());
                env.insert("TYPE_DOMAINS".to_string(), format!("{:?}", domains));
            }
            WorkerSpecialty::AsyncPatterns { patterns } => {
                env.insert("SPECIALTY_TYPE".to_string(), "async_patterns".to_string());
                env.insert("ASYNC_PATTERNS".to_string(), patterns.join(","));
            }
            WorkerSpecialty::Custom { domain, capabilities } => {
                env.insert("SPECIALTY_TYPE".to_string(), "custom".to_string());
                env.insert("CUSTOM_DOMAIN".to_string(), domain.clone());
                env.insert("CUSTOM_CAPABILITIES".to_string(), capabilities.join(","));
            }
        }

        env
    }
}

/// Resource limits for worker execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<f32>,
    pub max_file_handles: Option<u32>,
    pub time_limit: std::time::Duration,
    pub network_access: bool,
    pub filesystem_write_access: bool,
}

impl ResourceLimits {
    /// Create resource limits from subtask
    pub fn from_subtask(subtask: &SubTask) -> Self {
        Self {
            max_memory_mb: Some(512), // 512MB default
            max_cpu_percent: Some(50.0), // 50% CPU default
            max_file_handles: Some(100), // 100 file handles default
            time_limit: std::time::Duration::from_secs(300), // Default 5 minutes
            network_access: true, // Allow network by default
            filesystem_write_access: true, // Allow writes by default
        }
    }

    /// Create strict resource limits
    pub fn strict() -> Self {
        Self {
            max_memory_mb: Some(256),
            max_cpu_percent: Some(25.0),
            max_file_handles: Some(10),
            time_limit: std::time::Duration::from_secs(60),
            network_access: false,
            filesystem_write_access: true,
        }
    }

    /// Create permissive resource limits
    pub fn permissive() -> Self {
        Self {
            max_memory_mb: Some(2048),
            max_cpu_percent: Some(80.0),
            max_file_handles: Some(1000),
            time_limit: std::time::Duration::from_secs(3600), // 1 hour
            network_access: true,
            filesystem_write_access: true,
        }
    }
}

/// Execution sandbox for safe worker execution
pub struct ExecutionSandbox {
    working_directory: std::path::PathBuf,
    temp_directory: std::path::PathBuf,
    resource_limits: ResourceLimits,
}

impl ExecutionSandbox {
    /// Create a new execution sandbox
    pub fn new(working_directory: std::path::PathBuf, resource_limits: ResourceLimits) -> WorkerExecutionResult<Self> {
        let temp_directory = std::env::temp_dir()
            .join("parallel-workers")
            .join(format!("worker-{}", uuid::Uuid::new_v4()));

        std::fs::create_dir_all(&temp_directory)
            .map_err(|e| WorkerError::IsolationFailure {
                message: format!("Failed to create temp directory: {}", e),
            })?;

        Ok(Self {
            working_directory,
            temp_directory,
            resource_limits,
        })
    }

    /// Execute a command within the sandbox
    pub async fn execute_command(
        &self,
        command: &str,
        args: &[&str],
        env: &std::collections::HashMap<String, String>,
    ) -> WorkerExecutionResult<std::process::Output> {
        // Check resource limits
        if !self.check_resource_limits() {
            return Err(WorkerError::ResourceLimitsExceeded {
                resource_type: "general".to_string(),
            });
        }

        // Build the command
        let mut cmd = tokio::process::Command::new(command);
        cmd.args(args)
           .current_dir(&self.working_directory)
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        // Set environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Add sandbox-specific environment
        cmd.env("SANDBOX_TEMP_DIR", &self.temp_directory);
        cmd.env("SANDBOX_WORKING_DIR", &self.working_directory);

        // Execute with timeout
        match tokio::time::timeout(self.resource_limits.time_limit, cmd.output()).await {
            Ok(result) => result.map_err(|e| WorkerError::ExecutionTimeout {
                timeout_secs: self.resource_limits.time_limit.as_secs(),
            }),
            Err(_) => Err(WorkerError::ExecutionTimeout {
                timeout_secs: self.resource_limits.time_limit.as_secs(),
            }),
        }
    }

    /// Check if current resource usage is within limits
    fn check_resource_limits(&self) -> bool {
        // TODO: Implement actual resource checking
        // For now, always return true
        true
    }

    /// Get the temporary directory for this sandbox
    pub fn temp_directory(&self) -> &std::path::Path {
        &self.temp_directory
    }

    /// Clean up the sandbox
    pub fn cleanup(self) -> WorkerExecutionResult<()> {
        // Remove temporary directory
        if self.temp_directory.exists() {
            std::fs::remove_dir_all(&self.temp_directory)
                .map_err(|e| WorkerError::CleanupFailure {
                    message: format!("Failed to cleanup sandbox: {}", e),
                })?;
        }

        Ok(())
    }
}

impl Drop for ExecutionSandbox {
    fn drop(&mut self) {
        // Attempt cleanup on drop
        if self.temp_directory.exists() {
            let _ = std::fs::remove_dir_all(&self.temp_directory);
        }
    }
}

/// Safe file operations within worker context
pub struct SafeFileOperations {
    context: WorkerExecutionContext,
}

impl SafeFileOperations {
    pub fn new(context: WorkerExecutionContext) -> Self {
        Self { context }
    }

    /// Read a file safely (only if in scope)
    pub async fn read_file(&self, path: &std::path::Path) -> WorkerExecutionResult<String> {
        if !self.context.is_file_in_scope(path) {
            return Err(WorkerError::ResourceLimitsExceeded {
                resource_type: "file_access".to_string(),
            });
        }

        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| WorkerError::Io {
                message: format!("Failed to read file {}: {}", path.display(), e),
                source: Some(Box::new(e)),
            })
    }

    /// Write to a file safely (only if in scope and write access allowed)
    pub async fn write_file(&self, path: &std::path::Path, content: &str) -> WorkerExecutionResult<()> {
        if !self.context.is_file_in_scope(path) {
            return Err(WorkerError::ResourceLimitsExceeded {
                resource_type: "file_access".to_string(),
            });
        }

        if !self.context.resource_limits.filesystem_write_access {
            return Err(WorkerError::ResourceLimitsExceeded {
                resource_type: "write_access".to_string(),
            });
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| WorkerError::Io {
                    message: format!("Failed to create parent directory: {}", e),
                    source: Some(Box::new(e)),
                })?;
        }

        tokio::fs::write(path, content)
            .await
            .map_err(|e| WorkerError::Io {
                message: format!("Failed to write file {}: {}", path.display(), e),
                source: Some(Box::new(e)),
            })
    }

    /// List files in a directory safely
    pub async fn list_directory(&self, path: &std::path::Path) -> WorkerExecutionResult<Vec<std::path::PathBuf>> {
        if !self.context.is_file_in_scope(path) {
            return Err(WorkerError::ResourceLimitsExceeded {
                resource_type: "file_access".to_string(),
            });
        }

        let mut entries = tokio::fs::read_dir(path)
            .await
            .map_err(|e| WorkerError::Io {
                message: format!("Failed to read directory {}: {}", path.display(), e),
                source: Some(Box::new(e)),
            })?;

        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| WorkerError::Io {
            message: format!("Failed to read directory entry: {}", e),
            source: Some(Box::new(e)),
        })? {
            let path = entry.path();
            if self.context.is_file_in_scope(&path) {
                files.push(path);
            }
        }

        Ok(files)
    }
}
