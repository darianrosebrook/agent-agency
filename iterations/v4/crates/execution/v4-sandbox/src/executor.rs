//! Sandboxed Executor
//!
//! Executes tools within a sandbox environment with policy enforcement.

use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use v4_tools::{Tool, ToolError, ToolRegistry};
use v4_types::operators::OperatorResult;
use v4_types::OperatorType;

use crate::environment::{SandboxEnvironment, SandboxError, SandboxState};
use crate::policy::SandboxPolicy;

/// Sandboxed executor for running tools with policy enforcement
pub struct SandboxedExecutor {
    sandbox: Arc<SandboxEnvironment>,
    registry: Arc<ToolRegistry>,
    config: SandboxedExecutorConfig,
}

/// Configuration for sandboxed executor
#[derive(Debug, Clone)]
pub struct SandboxedExecutorConfig {
    /// Whether to verify content hashes
    pub verify_hashes: bool,
    /// Whether to log all operations
    pub log_operations: bool,
    /// Maximum retries on transient failures
    pub max_retries: u32,
}

impl Default for SandboxedExecutorConfig {
    fn default() -> Self {
        Self {
            verify_hashes: true,
            log_operations: true,
            max_retries: 0,
        }
    }
}

/// Execution record from sandboxed execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxedExecutionRecord {
    /// Operator that was executed
    pub operator: OperatorType,
    /// Result of execution
    pub result: OperatorResult,
    /// Sandbox ID
    pub sandbox_id: String,
    /// When execution started
    pub started_at: chrono::DateTime<Utc>,
    /// When execution completed
    pub completed_at: chrono::DateTime<Utc>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Content hash of result
    pub result_hash: String,
    /// Policy violations encountered (if any)
    pub policy_violations: Vec<String>,
}

/// Sandboxed executor error
#[derive(Debug, thiserror::Error)]
pub enum SandboxedExecutorError {
    #[error("Sandbox not ready: {0}")]
    SandboxNotReady(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Tool error: {0}")]
    ToolError(String),

    #[error("Sandbox error: {0}")]
    SandboxError(String),

    #[error("No tool available for operator")]
    NoToolAvailable,

    #[error("Execution timeout")]
    Timeout,
}

impl From<SandboxError> for SandboxedExecutorError {
    fn from(err: SandboxError) -> Self {
        SandboxedExecutorError::SandboxError(err.to_string())
    }
}

impl From<ToolError> for SandboxedExecutorError {
    fn from(err: ToolError) -> Self {
        SandboxedExecutorError::ToolError(err.to_string())
    }
}

impl SandboxedExecutor {
    /// Create a new sandboxed executor
    pub fn new(sandbox: Arc<SandboxEnvironment>, registry: Arc<ToolRegistry>) -> Self {
        Self {
            sandbox,
            registry,
            config: SandboxedExecutorConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        sandbox: Arc<SandboxEnvironment>,
        registry: Arc<ToolRegistry>,
        config: SandboxedExecutorConfig,
    ) -> Self {
        Self {
            sandbox,
            registry,
            config,
        }
    }

    /// Execute an operator within the sandbox
    pub async fn execute(
        &self,
        operator: &OperatorType,
    ) -> Result<SandboxedExecutionRecord, SandboxedExecutorError> {
        let started_at = Utc::now();
        let start_instant = Instant::now();

        // Verify sandbox is ready
        let state = self.sandbox.state().await;
        if state != SandboxState::Ready && state != SandboxState::Active {
            return Err(SandboxedExecutorError::SandboxNotReady(format!(
                "Sandbox in state {:?}",
                state
            )));
        }

        // Check operator-specific policies
        let violations = self.check_operator_policies(operator).await?;

        // Find appropriate tool
        let tool = self.find_tool_for_operator(operator)?;

        // Execute the tool
        let result = tool.execute(operator).await?;

        let completed_at = Utc::now();
        let execution_time_ms = start_instant.elapsed().as_millis() as u64;

        // Calculate result hash
        let result_hash = self.hash_result(&result);

        // Record resource usage
        if let Some(data) = &result.data {
            let data_size = serde_json::to_string(data)
                .map(|s| s.len() as u64)
                .unwrap_or(0);
            self.sandbox.record_read(data_size).await;
        }

        Ok(SandboxedExecutionRecord {
            operator: operator.clone(),
            result,
            sandbox_id: self.sandbox.id.clone(),
            started_at,
            completed_at,
            execution_time_ms,
            result_hash,
            policy_violations: violations,
        })
    }

    /// Execute multiple operators in sequence
    pub async fn execute_sequence(
        &self,
        operators: &[OperatorType],
    ) -> Vec<Result<SandboxedExecutionRecord, SandboxedExecutorError>> {
        let mut results = Vec::with_capacity(operators.len());

        for operator in operators {
            let result = self.execute(operator).await;
            let should_continue = result.is_ok();
            results.push(result);

            if !should_continue {
                break;
            }
        }

        results
    }

    /// Check policies specific to the operator
    async fn check_operator_policies(
        &self,
        operator: &OperatorType,
    ) -> Result<Vec<String>, SandboxedExecutorError> {
        let violations = Vec::new();

        match operator {
            OperatorType::Seek(seek_op) => {
                use v4_types::operators::SeekOp;
                match seek_op {
                    SeekOp::ReadFile { path } => {
                        let resolved = self.sandbox.resolve_path(std::path::Path::new(path));
                        if let Err(e) = self.sandbox.can_read(&resolved).await {
                            return Err(SandboxedExecutorError::PolicyViolation(e.to_string()));
                        }
                    }
                    SeekOp::ListDirectory { path } => {
                        let resolved = self.sandbox.resolve_path(std::path::Path::new(path));
                        if let Err(e) = self.sandbox.can_read(&resolved).await {
                            return Err(SandboxedExecutorError::PolicyViolation(e.to_string()));
                        }
                    }
                    SeekOp::SearchCode { path, .. } => {
                        if let Some(p) = path {
                            let resolved = self.sandbox.resolve_path(std::path::Path::new(p));
                            if let Err(e) = self.sandbox.can_read(&resolved).await {
                                return Err(SandboxedExecutorError::PolicyViolation(e.to_string()));
                            }
                        }
                    }
                    SeekOp::WebFetch { url } | SeekOp::WebSearch { query: url } => {
                        // Extract host from URL for network access check
                        if let Some(host) = extract_host(url) {
                            if let Err(e) = self.sandbox.can_access_network(&host).await {
                                return Err(SandboxedExecutorError::PolicyViolation(e.to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
            OperatorType::Memorize(_) => {
                // Memorize operations are internal and don't require filesystem checks
            }
            OperatorType::Perceive(_) => {
                // Perceive operations are read-only interpretations
            }
            OperatorType::Knowledge(_) => {
                // Knowledge operations are pure transformations
            }
            OperatorType::Control(_) => {
                // Control operations are flow control
            }
        }

        Ok(violations)
    }

    /// Find a tool that can handle the operator
    fn find_tool_for_operator(
        &self,
        operator: &OperatorType,
    ) -> Result<Arc<dyn Tool>, SandboxedExecutorError> {
        // Get operator class using the type's method
        let class = operator.class();

        // Find tools that support this operator class
        let tools = self.registry.find_for_operator(class);

        // Find one that can execute this specific operator
        for meta in tools {
            if let Some(tool) = self.registry.get(&meta.id) {
                if tool.can_execute(operator) {
                    return Ok(tool);
                }
            }
        }

        Err(SandboxedExecutorError::NoToolAvailable)
    }

    /// Hash the result for verification
    fn hash_result(&self, result: &OperatorResult) -> String {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(result).unwrap_or_default().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get the sandbox reference
    pub fn sandbox(&self) -> &Arc<SandboxEnvironment> {
        &self.sandbox
    }
}

/// Builder for creating sandbox execution contexts
pub struct SandboxBuilder {
    id: String,
    policy: SandboxPolicy,
    working_dir: Option<std::path::PathBuf>,
    env_vars: std::collections::HashMap<String, String>,
}

impl SandboxBuilder {
    /// Create a new sandbox builder
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            policy: SandboxPolicy::standard(),
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
        }
    }

    /// Set the policy
    pub fn policy(mut self, policy: SandboxPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Build the sandbox environment
    pub fn build(self) -> SandboxEnvironment {
        let mut sandbox = SandboxEnvironment::new(self.id, self.policy);

        if let Some(dir) = self.working_dir {
            sandbox = sandbox.with_working_dir(dir);
        }

        sandbox = sandbox.with_envs(self.env_vars);

        sandbox
    }
}

/// Simple URL host extraction without external dependencies
fn extract_host(url_or_query: &str) -> Option<String> {
    // Try to extract host from URL-like strings
    if url_or_query.starts_with("http://") || url_or_query.starts_with("https://") {
        let without_scheme = url_or_query
            .strip_prefix("https://")
            .or_else(|| url_or_query.strip_prefix("http://"))?;

        // Extract host (everything before first / or end)
        let host = without_scheme.split('/').next()?;
        // Remove port if present
        let host = host.split(':').next()?;
        Some(host.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_tools::ToolRegistry;
    use v4_types::operators::SeekOp;

    fn create_test_sandbox() -> Arc<SandboxEnvironment> {
        Arc::new(
            SandboxBuilder::new("test")
                .policy(SandboxPolicy::permissive())
                .working_dir("/tmp/sandbox-test")
                .build(),
        )
    }

    #[tokio::test]
    async fn test_sandboxed_executor_creation() {
        let sandbox = create_test_sandbox();
        let registry = Arc::new(ToolRegistry::new());

        let executor = SandboxedExecutor::new(sandbox, registry);
        assert_eq!(executor.sandbox.id, "test");
    }

    #[tokio::test]
    async fn test_policy_enforcement() {
        let sandbox = Arc::new(
            SandboxBuilder::new("restricted")
                .policy(SandboxPolicy::restricted())
                .build(),
        );
        let registry = Arc::new(ToolRegistry::new());
        let executor = SandboxedExecutor::new(sandbox.clone(), registry);

        // Initialize sandbox
        sandbox.initialize().await.unwrap();

        // Try to execute an operator that reads from blocked path
        let operator = OperatorType::Seek(SeekOp::ReadFile {
            path: "/etc/passwd".to_string(),
        });

        let result = executor.execute(&operator).await;
        assert!(result.is_err());

        match result {
            Err(SandboxedExecutorError::PolicyViolation(_)) => {}
            _ => panic!("Expected PolicyViolation error"),
        }
    }

    #[tokio::test]
    async fn test_sandbox_builder() {
        let sandbox = SandboxBuilder::new("builder-test")
            .policy(SandboxPolicy::standard())
            .working_dir("/tmp/test")
            .env("FOO", "bar")
            .build();

        assert_eq!(sandbox.id, "builder-test");
        assert_eq!(sandbox.working_dir, std::path::PathBuf::from("/tmp/test"));
        assert_eq!(sandbox.env_vars.get("FOO"), Some(&"bar".to_string()));
    }

    #[tokio::test]
    async fn test_executor_config() {
        let config = SandboxedExecutorConfig {
            verify_hashes: false,
            log_operations: true,
            max_retries: 3,
        };

        let sandbox = create_test_sandbox();
        let registry = Arc::new(ToolRegistry::new());

        let executor = SandboxedExecutor::with_config(sandbox, registry, config.clone());
        assert_eq!(executor.config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_sandbox_state_check() {
        let sandbox = Arc::new(SandboxBuilder::new("state-test").build());
        let registry = Arc::new(ToolRegistry::new());
        let executor = SandboxedExecutor::new(sandbox.clone(), registry);

        // Without initialization, should fail
        let operator = OperatorType::Seek(SeekOp::ReadFile {
            path: "/tmp/test.txt".to_string(),
        });

        let result = executor.execute(&operator).await;
        assert!(matches!(
            result,
            Err(SandboxedExecutorError::SandboxNotReady(_))
        ));
    }
}
