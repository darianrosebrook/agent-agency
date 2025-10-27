//! Test environment and workspace management
//!
//! Provides isolated test execution environments with:
//! - Temporary directories for test execution
//! - Git worktree management for isolated code workspaces
//! - Automatic cleanup on test completion/failure
//! - Performance metrics collection

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::{TempDir, tempdir};
use tracing::{info, warn, error};

/// Test environment that manages lifecycle and cleanup
pub struct TestEnvironment {
    temp_dir: TempDir,
    workspaces: Arc<Mutex<Vec<TestWorkspace>>>,
    metrics: Arc<Mutex<TestMetrics>>,
}

impl TestEnvironment {
    /// Create a new test environment
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating new test environment");

        let temp_dir = tempdir()?;
        info!("Test environment created at: {}", temp_dir.path().display());

        Ok(Self {
            temp_dir,
            workspaces: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(TestMetrics::default())),
        })
    }

    /// Create an isolated workspace for a test
    pub async fn create_workspace(&self, name: &str) -> Result<TestWorkspace, Box<dyn std::error::Error + Send + Sync>> {
        let workspace_path = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&workspace_path)?;

        let workspace = TestWorkspace::new(workspace_path, name.to_string()).await?;

        {
            let mut workspaces = self.workspaces.lock().await;
            workspaces.push(workspace.clone());
        }

        info!("Created workspace '{}' at {}", name, workspace.path().display());
        Ok(workspace)
    }

    /// Record a performance metric
    pub async fn record_metric(&self, metric: &str, value: f64) {
        let mut metrics = self.metrics.lock().await;
        metrics.record(metric, value);
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> TestMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    /// Clean up all resources
    pub async fn cleanup(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Cleaning up test environment");

        // Clean up workspaces
        {
            let workspaces = self.workspaces.lock().await;
            for workspace in workspaces.iter() {
                if let Err(e) = workspace.cleanup().await {
                    error!("Failed to cleanup workspace {}: {}", workspace.name, e);
                }
            }
        }

        // TempDir will be automatically cleaned up when dropped
        info!("Test environment cleanup complete");
        Ok(())
    }
}

/// Isolated workspace for test execution
#[derive(Clone)]
pub struct TestWorkspace {
    path: PathBuf,
    name: String,
}

impl TestWorkspace {
    /// Create a new test workspace
    pub async fn new(path: PathBuf, name: String) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            path,
            name,
        })
    }

    /// Copy files to workspace
    pub async fn copy_files(&self, source_dir: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.copy_dir_recursive(source_dir, &self.path).await
    }

    /// Execute a command in the workspace
    pub async fn execute_command(&self, command: &str, args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command;

        info!("Executing command in workspace {}: {} {:?}", self.name, command, args);

        let output = Command::new(command)
            .args(args)
            .current_dir(&self.path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Command failed in workspace {}: {}", self.name, stderr);
        } else {
            info!("Command succeeded in workspace {}", self.name);
        }

        Ok(output)
    }

    /// Get workspace path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Clean up workspace resources
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, just log cleanup (tempfile handles actual cleanup)
        info!("Cleaning up workspace {}", self.name);
        Ok(())
    }

    /// Helper to recursively copy directories
    async fn copy_dir_recursive(&self, from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        std::fs::create_dir_all(to)?;

        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = to.join(file_name);

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dest_path).await?;
            } else {
                std::fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }
}

/// Performance and execution metrics
#[derive(Debug, Clone, Default)]
pub struct TestMetrics {
    pub start_time: Option<std::time::Instant>,
    pub end_time: Option<std::time::Instant>,
    pub measurements: std::collections::HashMap<String, Vec<f64>>,
}

impl TestMetrics {
    /// Start timing
    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
    }

    /// Stop timing
    pub fn stop(&mut self) {
        self.end_time = Some(std::time::Instant::now());
    }

    /// Record a measurement
    pub fn record(&mut self, metric: &str, value: f64) {
        self.measurements.entry(metric.to_string()).or_insert_with(Vec::new).push(value);
    }

    /// Get total duration
    pub fn duration(&self) -> Option<std::time::Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }

    /// Get average value for a metric
    pub fn average(&self, metric: &str) -> Option<f64> {
        self.measurements.get(metric).and_then(|values| {
            if values.is_empty() {
                None
            } else {
                Some(values.iter().sum::<f64>() / values.len() as f64)
            }
        })
    }

    /// Get all metrics as a summary
    pub fn summary(&self) -> std::collections::HashMap<String, f64> {
        let mut summary = std::collections::HashMap::new();

        for (metric, values) in &self.measurements {
            if let Some(avg) = self.average(metric) {
                summary.insert(format!("{}_avg", metric), avg);
                summary.insert(format!("{}_count", metric), values.len() as f64);
                summary.insert(format!("{}_min", metric), values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
                summary.insert(format!("{}_max", metric), values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
            }
        }

        if let Some(duration) = self.duration() {
            summary.insert("total_duration_ms".to_string(), duration.as_millis() as f64);
        }

        summary
    }
}
