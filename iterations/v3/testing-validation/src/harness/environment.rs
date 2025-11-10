//! Test environment for E2E testing
//!
//! Provides functional implementations of TestEnvironment and TestWorkspace
//! for testing scenarios. These implementations use real file system operations
//! and can be extended with observability and advanced file operations as needed.

use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Metrics collected during testing
#[derive(Debug, Clone)]
pub struct TestMetrics {
    pub iterations: f64,
    pub model_calls: f64,
    pub errors: f64,
    pub duration_ms: f64,
}

/// Test environment wrapper for E2E testing
#[derive(Debug)]
pub struct TestEnvironment {
    temp_dir: TempDir,
    metrics: Arc<RwLock<HashMap<String, f64>>>,
}

impl TestEnvironment {
    pub async fn new() -> Result<Self> {
        // Create temporary directory for testing
        let temp_dir = tempfile::tempdir()?;
        let metrics = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self { temp_dir, metrics })
    }

    /// Create a test workspace
    pub async fn create_workspace(&self, name: &str) -> Result<TestWorkspace> {
        let workspace_path = self.temp_dir.path().join(name);

        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(&workspace_path).await?;

        // Initialize git repository
        let workspace = TestWorkspace::new(workspace_path);
        workspace.init_git().await?;

        Ok(workspace)
    }

    /// Record a test metric
    pub async fn record_metric(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), value);
        Ok(())
    }

    /// Get collected metrics
    pub async fn get_metrics(&self) -> Result<HashMap<String, f64>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// Clean up the test environment
    pub async fn cleanup(&self) -> Result<()> {
        // Cleanup is handled by TempDir drop
        Ok(())
    }
}

/// Test workspace wrapper for file operations during testing
#[derive(Debug)]
pub struct TestWorkspace {
    path: PathBuf,
}

impl TestWorkspace {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Initialize git repository in the workspace
    pub async fn init_git(&self) -> Result<()> {
        use std::process::Command;

        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .current_dir(&self.path)
            .output()?;

        // Configure git user
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&self.path)
            .output()?;

        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&self.path)
            .output()?;

        // Initial commit
        Command::new("git")
            .args(&["add", "."])
            .current_dir(&self.path)
            .output()?;

        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&self.path)
            .output()?;

        Ok(())
    }

    /// Execute a command in the workspace directory
    pub async fn execute_command(&self, cmd: &str, args: &[&str]) -> Result<std::process::Output> {
        use std::process::Command;
        let output = Command::new(cmd)
            .args(args)
            .current_dir(&self.path)
            .output()?;
        Ok(output)
    }
}

/// Create a default process output for error cases
pub fn default_process_output() -> std::process::Output {
    use std::process::{Command, ExitStatus};
    
    // Create a default exit status by running a command that fails
    // This is cross-platform compatible
    let status = Command::new("false")
        .output()
        .map(|o| o.status)
        .unwrap_or_else(|_| {
            // Fallback: create exit status manually if false command doesn't exist
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                ExitStatus::from_raw(1)
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::ExitStatusExt;
                ExitStatus::from_raw(1)
            }
            #[cfg(not(any(unix, windows)))]
            {
                // For other platforms, try to create a failed status
                // This is a best-effort fallback
                Command::new("sh")
                    .args(&["-c", "exit 1"])
                    .status()
                    .unwrap_or_else(|_| {
                        // Last resort: we can't create a proper exit status
                        // Return a status that indicates failure
                        // Note: This may not compile on all platforms
                        ExitStatus::from_raw(1)
                    })
            }
        });
    
    std::process::Output {
        status,
        stdout: vec![],
        stderr: vec![],
    }
}

