/**
 * Sandbox Module - P0-8 Implementation
 *
 * Isolated execution environment for safe code and task execution.
 * Provides containerized/sandboxed environments to prevent security risks.
 */

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::time::Duration;

/// Sandbox execution modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxMode {
    /// Docker container isolation
    Docker,
    /// Firejail sandboxing
    Firejail,
    /// systemd-nspawn container
    Nspawn,
    /// Bubblewrap sandboxing
    Bubblewrap,
    /// No isolation (for testing only)
    None,
}

/// Sandbox resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub disk_mb: Option<u64>,
    pub network_enabled: bool,
    pub timeout_seconds: Option<u64>,
}

/// Sandbox execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxContext {
    pub id: Uuid,
    pub mode: SandboxMode,
    pub limits: ResourceLimits,
    pub environment_vars: HashMap<String, String>,
    pub working_directory: Option<PathBuf>,
    pub network_access: bool,
    pub filesystem_access: Vec<PathBuf>,
}

/// Sandbox execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Sandbox execution request
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub command: Vec<String>,
    pub context: SandboxContext,
    pub input_data: Option<String>,
}

/// Sandbox result type
pub type SandboxResult<T> = Result<T, SandboxError>;

/// Sandbox operation errors
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Sandbox creation failed: {reason}")]
    CreationFailed { reason: String },

    #[error("Execution timeout: {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },

    #[error("Resource limit exceeded: {resource}")]
    ResourceLimitExceeded { resource: String },

    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },

    #[error("Execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("Sandbox not available: {mode:?}")]
    SandboxUnavailable { mode: SandboxMode },

    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },
}

/// Sandbox interface for isolated execution
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Execute a command in the sandbox
    async fn execute(&self, request: ExecutionRequest) -> SandboxResult<ExecutionResult>;

    /// Validate sandbox configuration
    async fn validate_config(&self, context: &SandboxContext) -> SandboxResult<()>;

    /// Get sandbox status and capabilities
    async fn get_status(&self) -> SandboxResult<SandboxStatus>;

    /// Clean up sandbox resources
    async fn cleanup(&self, sandbox_id: &Uuid) -> SandboxResult<()>;
}

/// Sandbox status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxStatus {
    pub available_modes: Vec<SandboxMode>,
    pub active_sandboxes: u32,
    pub resource_usage: ResourceUsage,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub active_containers: u32,
}

/// Production sandbox implementation
pub struct ProductionSandbox {
    active_sandboxes: Arc<RwLock<HashMap<Uuid, SandboxInstance>>>,
}

#[derive(Debug)]
struct SandboxInstance {
    context: SandboxContext,
    start_time: DateTime<Utc>,
    process_id: Option<u32>,
}

impl ProductionSandbox {
    pub fn new() -> Self {
        Self {
            active_sandboxes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a sandbox mode is available on the system
    async fn check_mode_availability(&self, mode: &SandboxMode) -> bool {
        match mode {
            SandboxMode::Docker => Self::check_command_exists("docker").await,
            SandboxMode::Firejail => Self::check_command_exists("firejail").await,
            SandboxMode::Nspawn => Self::check_command_exists("systemd-nspawn").await,
            SandboxMode::Bubblewrap => Self::check_command_exists("bwrap").await,
            SandboxMode::None => true, // Always available for testing
        }
    }

    /// Check if a command exists on the system
    async fn check_command_exists(command: &str) -> bool {
        match Command::new("which").arg(command).output().await {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Execute command in Docker container
    async fn execute_docker(&self, request: &ExecutionRequest) -> SandboxResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        // Build Docker command
        let mut docker_cmd = Command::new("docker");
        docker_cmd.arg("run")
            .arg("--rm")
            .arg("--network")
            .arg(if request.context.network_access { "bridge" } else { "none" });

        // Add resource limits
        if let Some(cpu) = request.context.limits.cpu_cores {
            docker_cmd.arg("--cpus").arg(cpu.to_string());
        }
        if let Some(memory) = request.context.limits.memory_mb {
            docker_cmd.arg("--memory").arg(format!("{}m", memory));
        }

        // Add environment variables
        for (key, value) in &request.context.environment_vars {
            docker_cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        // Use Alpine Linux as base image for security
        docker_cmd.arg("alpine:latest");

        // Add the command to execute
        docker_cmd.args(&request.command);

        // Set timeout
        let timeout_duration = request.context.limits.timeout_seconds
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(300)); // 5 minutes default

        // Execute with timeout
        match tokio::time::timeout(timeout_duration, docker_cmd.output()).await {
            Ok(Ok(output)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();

                Ok(ExecutionResult {
                    exit_code,
                    stdout,
                    stderr,
                    execution_time_ms: execution_time,
                    success: output.status.success(),
                    error_message: if output.status.success() { None } else { Some("Command failed".to_string()) },
                })
            }
            Ok(Err(e)) => Err(SandboxError::ExecutionFailed {
                message: format!("Docker execution error: {:?}", e)
            }),
            Err(_) => Err(SandboxError::Timeout {
                timeout_seconds: timeout_duration.as_secs()
            }),
        }
    }

    /// Execute command with Firejail sandboxing
    async fn execute_firejail(&self, request: &ExecutionRequest) -> SandboxResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        let mut cmd = Command::new("firejail");
        cmd.arg("--quiet");

        // Add resource limits
        if let Some(memory) = request.context.limits.memory_mb {
            cmd.arg("--rlimit-as").arg((memory * 1024 * 1024).to_string());
        }

        // Network restrictions
        if !request.context.network_access {
            cmd.arg("--net=none");
        }

        // Filesystem restrictions
        cmd.arg("--private");

        // Add the command to execute
        cmd.args(&request.command);

        // Set timeout
        let timeout_duration = request.context.limits.timeout_seconds
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(300));

        match tokio::time::timeout(timeout_duration, cmd.output()).await {
            Ok(Ok(output)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();

                Ok(ExecutionResult {
                    exit_code,
                    stdout,
                    stderr,
                    execution_time_ms: execution_time,
                    success: output.status.success(),
                    error_message: if output.status.success() { None } else { Some("Command failed".to_string()) },
                })
            }
            Ok(Err(e)) => Err(SandboxError::ExecutionFailed {
                message: format!("Firejail execution error: {:?}", e)
            }),
            Err(_) => Err(SandboxError::Timeout {
                timeout_seconds: timeout_duration.as_secs()
            }),
        }
    }

    /// Execute without sandboxing (for testing only)
    async fn execute_unrestricted(&self, request: &ExecutionRequest) -> SandboxResult<ExecutionResult> {
        warn!("Executing command without sandboxing - this should only be used for testing");

        let start_time = std::time::Instant::now();
        let mut cmd = Command::new(&request.command[0]);
        cmd.args(&request.command[1..]);

        // Set environment variables
        for (key, value) in &request.context.environment_vars {
            cmd.env(key, value);
        }

        // Set timeout
        let timeout_duration = request.context.limits.timeout_seconds
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(300));

        match tokio::time::timeout(timeout_duration, cmd.output()).await {
            Ok(Ok(output)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();

                Ok(ExecutionResult {
                    exit_code,
                    stdout,
                    stderr,
                    execution_time_ms: execution_time,
                    success: output.status.success(),
                    error_message: if output.status.success() { None } else { Some("Command failed".to_string()) },
                })
            }
            Ok(Err(e)) => Err(SandboxError::ExecutionFailed {
                message: format!("Unrestricted execution error: {:?}", e)
            }),
            Err(_) => Err(SandboxError::Timeout {
                timeout_seconds: timeout_duration.as_secs()
            }),
        }
    }

    /// Register an active sandbox instance
    async fn register_sandbox(&self, context: SandboxContext) -> Uuid {
        let instance = SandboxInstance {
            context: context.clone(),
            start_time: Utc::now(),
            process_id: None,
        };

        let mut sandboxes = self.active_sandboxes.write().await;
        sandboxes.insert(context.id, instance);

        context.id
    }

    /// Unregister a sandbox instance
    async fn unregister_sandbox(&self, sandbox_id: &Uuid) {
        let mut sandboxes = self.active_sandboxes.write().await;
        sandboxes.remove(sandbox_id);
    }
}

#[async_trait]
impl Sandbox for ProductionSandbox {
    async fn execute(&self, request: ExecutionRequest) -> SandboxResult<ExecutionResult> {
        // Validate configuration first
        self.validate_config(&request.context).await?;

        // Register the sandbox instance
        let sandbox_id = self.register_sandbox(request.context.clone()).await;

        // Execute based on sandbox mode
        let result = match request.context.mode {
            SandboxMode::Docker => self.execute_docker(&request).await,
            SandboxMode::Firejail => self.execute_firejail(&request).await,
            SandboxMode::None => self.execute_unrestricted(&request).await,
            SandboxMode::Nspawn | SandboxMode::Bubblewrap => {
                Err(SandboxError::SandboxUnavailable {
                    mode: request.context.mode.clone()
                })
            }
        };

        // Unregister the sandbox instance
        self.unregister_sandbox(&sandbox_id).await;

        result
    }

    async fn validate_config(&self, context: &SandboxContext) -> SandboxResult<()> {
        // Check if the sandbox mode is available
        if !self.check_mode_availability(&context.mode).await {
            return Err(SandboxError::SandboxUnavailable {
                mode: context.mode.clone()
            });
        }

        // Validate resource limits
        if let Some(cpu) = context.limits.cpu_cores {
            if cpu <= 0.0 || cpu > 64.0 {
                return Err(SandboxError::InvalidConfig {
                    message: "CPU cores must be between 0.1 and 64.0".to_string()
                });
            }
        }

        if let Some(memory) = context.limits.memory_mb {
            if memory == 0 || memory > 65536 { // 64GB max
                return Err(SandboxError::InvalidConfig {
                    message: "Memory limit must be between 1MB and 65536MB".to_string()
                });
            }
        }

        // Validate timeout
        if let Some(timeout) = context.limits.timeout_seconds {
            if timeout == 0 || timeout > 3600 { // 1 hour max
                return Err(SandboxError::InvalidConfig {
                    message: "Timeout must be between 1 and 3600 seconds".to_string()
                });
            }
        }

        Ok(())
    }

    async fn get_status(&self) -> SandboxResult<SandboxStatus> {
        let mut available_modes = Vec::new();

        // Check which sandbox modes are available
        for mode in &[SandboxMode::Docker, SandboxMode::Firejail, SandboxMode::Nspawn, SandboxMode::Bubblewrap] {
            if self.check_mode_availability(mode).await {
                available_modes.push(mode.clone());
            }
        }

        // Always include None for testing
        available_modes.push(SandboxMode::None);

        let active_sandboxes = self.active_sandboxes.read().await.len() as u32;

        Ok(SandboxStatus {
            available_modes,
            active_sandboxes,
            resource_usage: ResourceUsage {
                cpu_percent: 0.0, // Would need system monitoring to get real values
                memory_mb: 0,     // Would need system monitoring to get real values
                active_containers: active_sandboxes,
            },
        })
    }

    async fn cleanup(&self, sandbox_id: &Uuid) -> SandboxResult<()> {
        // Remove from active sandboxes
        self.unregister_sandbox(sandbox_id).await;

        // For Docker, we could clean up stopped containers here
        // For now, just remove from our tracking
        info!("Cleaned up sandbox: {}", sandbox_id);
        Ok(())
    }
}

/// Factory function for creating sandbox instances
pub fn create_sandbox() -> Arc<dyn Sandbox> {
    Arc::new(ProductionSandbox::new())
}

/// Helper function to create a basic sandbox context
pub fn create_basic_context(mode: SandboxMode) -> SandboxContext {
    SandboxContext {
        id: Uuid::new_v4(),
        mode,
        limits: ResourceLimits {
            cpu_cores: Some(1.0),
            memory_mb: Some(512),
            disk_mb: Some(100),
            network_enabled: false,
            timeout_seconds: Some(300),
        },
        environment_vars: HashMap::new(),
        working_directory: None,
        network_access: false,
        filesystem_access: vec![],
    }
}

/// Helper function to create a secure sandbox context for code execution
pub fn create_secure_context() -> SandboxContext {
    let mut context = create_basic_context(SandboxMode::Docker);
    context.limits.memory_mb = Some(256);
    context.limits.timeout_seconds = Some(60);
    context.network_access = false;
    context
}
