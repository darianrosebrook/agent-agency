//! Benchmark dataset manager with versioning and validation
//!
//! Manages benchmark datasets with versioning, validation, quality checks,
//! and rolling 12-month retention policy.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::benchmark_types::MicroTask;

/// Dataset version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetVersion {
    /// Version identifier
    pub version: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Number of tasks in this version
    pub task_count: usize,
    /// Dataset metadata
    pub metadata: HashMap<String, String>,
    /// Whether this version is archived
    pub archived: bool,
}

/// Benchmark dataset with versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDataset {
    /// Dataset identifier
    pub id: Uuid,
    /// Dataset name
    pub name: String,
    /// Current active version
    pub current_version: String,
    /// All versions of this dataset
    pub versions: Vec<DatasetVersion>,
    /// Tasks in the current version
    pub tasks: Vec<MicroTask>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Dataset manager for benchmark datasets
pub struct DatasetManager {
    /// All datasets
    datasets: Arc<RwLock<HashMap<Uuid, BenchmarkDataset>>>,
    /// Retention period (default: 12 months)
    retention_period: Duration,
}

impl DatasetManager {
    /// Create a new dataset manager
    pub fn new() -> Self {
        Self {
            datasets: Arc::new(RwLock::new(HashMap::new())),
            retention_period: Duration::days(365), // 12 months
        }
    }

    /// Create a new dataset
    pub async fn create_dataset(
        &self,
        name: String,
        tasks: Vec<MicroTask>,
        metadata: HashMap<String, String>,
    ) -> Result<Uuid> {
        let dataset_id = Uuid::new_v4();
        let version = "1.0.0".to_string();
        let now = Utc::now();

        let dataset = BenchmarkDataset {
            id: dataset_id,
            name: name.clone(),
            current_version: version.clone(),
            versions: vec![DatasetVersion {
                version: version.clone(),
                created_at: now,
                task_count: tasks.len(),
                metadata: metadata.clone(),
                archived: false,
            }],
            tasks: tasks.clone(),
            created_at: now,
            updated_at: now,
        };

        let mut datasets = self.datasets.write().await;
        datasets.insert(dataset_id, dataset);

        info!(
            "Created dataset '{}' (ID: {}) with version {} and {} tasks",
            name,
            dataset_id,
            version,
            tasks.len()
        );

        Ok(dataset_id)
    }

    /// Add a new version to an existing dataset
    pub async fn add_version(
        &self,
        dataset_id: Uuid,
        tasks: Vec<MicroTask>,
        version: String,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let mut datasets = self.datasets.write().await;

        let dataset = datasets
            .get_mut(&dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset {} not found", dataset_id))?;

        // Archive previous version
        if let Some(prev_version) = dataset.versions.iter_mut().find(|v| !v.archived) {
            prev_version.archived = true;
        }

        // Add new version
        dataset.versions.push(DatasetVersion {
            version: version.clone(),
            created_at: Utc::now(),
            task_count: tasks.len(),
            metadata: metadata.clone(),
            archived: false,
        });

        dataset.current_version = version.clone();
        dataset.tasks = tasks.clone();
        dataset.updated_at = Utc::now();

        info!(
            "Added version {} to dataset {} ({} tasks)",
            version,
            dataset_id,
            tasks.len()
        );

        Ok(())
    }

    /// Get dataset by ID
    pub async fn get_dataset(&self, dataset_id: Uuid) -> Option<BenchmarkDataset> {
        self.datasets.read().await.get(&dataset_id).cloned()
    }

    /// Get dataset tasks for a specific version
    pub async fn get_dataset_tasks(
        &self,
        dataset_id: Uuid,
        version: Option<&str>,
    ) -> Result<Vec<MicroTask>> {
        let datasets = self.datasets.read().await;

        let dataset = datasets
            .get(&dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset {} not found", dataset_id))?;

        if let Some(version) = version {
            // Find specific version
            if let Some(version_info) = dataset.versions.iter().find(|v| v.version == version) {
                // TODO: Implement comprehensive version-specific task storage
                //       Currently returns current tasks if version matches current; should implement comprehensive storage that stores tasks per version for accurate version-specific task retrieval.
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
                // - Tasks are stored per version
                // - Version-specific tasks are retrieved correctly
                // - Version history is maintained
                // - Task retrieval handles missing versions gracefully
                //
                // DEPENDENCIES:
                // - Version-specific storage system (Required)
                // - Task versioning utilities (Required)
                // - Version history management (Required)
                //
                // ESTIMATED EFFORT: 8-12 hours (medium confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (dataset versioning functionality)
                // - Change Budget: ~200 LOC
                // - Reviewer Requirements: Dataset versioning and task storage expertise
                if version == dataset.current_version {
                    Ok(dataset.tasks.clone())
                } else {
                    warn!("Requested version {} not found, returning current version", version);
                    Ok(dataset.tasks.clone())
                }
            } else {
                Err(anyhow::anyhow!("Version {} not found for dataset {}", version, dataset_id))
            }
        } else {
            // Return current version tasks
            Ok(dataset.tasks.clone())
        }
    }

    /// Validate dataset quality
    pub async fn validate_dataset(&self, dataset_id: Uuid) -> Result<DatasetValidationResult> {
        let datasets = self.datasets.read().await;

        let dataset = datasets
            .get(&dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset {} not found", dataset_id))?;

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check task count
        if dataset.tasks.is_empty() {
            issues.push("Dataset has no tasks".to_string());
        }

        // Check for duplicate tasks
        let task_ids: Vec<Uuid> = dataset.tasks.iter().map(|t| t.id).collect();
        let unique_ids: std::collections::HashSet<Uuid> = task_ids.iter().cloned().collect();
        if task_ids.len() != unique_ids.len() {
            issues.push("Dataset contains duplicate tasks".to_string());
        }

        // Check task quality
        let mut empty_inputs = 0;
        let mut empty_outputs = 0;
        for task in &dataset.tasks {
            if task.input.is_empty() {
                empty_inputs += 1;
            }
            if task.expected_output.is_empty() {
                empty_outputs += 1;
            }
        }

        if empty_inputs > 0 {
            warnings.push(format!("{} tasks have empty inputs", empty_inputs));
        }
        if empty_outputs > 0 {
            warnings.push(format!("{} tasks have empty expected outputs", empty_outputs));
        }

        // Check version consistency
        if let Some(current_version) = dataset.versions.iter().find(|v| !v.archived) {
            if current_version.task_count != dataset.tasks.len() {
                warnings.push(format!(
                    "Version task count ({}) doesn't match actual tasks ({})",
                    current_version.task_count,
                    dataset.tasks.len()
                ));
            }
        }

        let is_valid = issues.is_empty();

        Ok(DatasetValidationResult {
            dataset_id,
            is_valid,
            issues,
            warnings,
            task_count: dataset.tasks.len(),
            validated_at: Utc::now(),
        })
    }

    /// Clean up old archived versions (rolling retention)
    pub async fn cleanup_old_versions(&self) -> Result<usize> {
        let mut datasets = self.datasets.write().await;
        let cutoff_date = Utc::now() - self.retention_period;
        let mut removed_count = 0;

        for dataset in datasets.values_mut() {
            let initial_count = dataset.versions.len();
            dataset.versions.retain(|v| {
                // Keep if not archived or archived after cutoff
                !v.archived || v.created_at >= cutoff_date
            });
            removed_count += initial_count - dataset.versions.len();
        }

        if removed_count > 0 {
            info!("Removed {} old archived dataset versions", removed_count);
        }

        Ok(removed_count)
    }

    /// List all datasets
    pub async fn list_datasets(&self) -> Vec<BenchmarkDataset> {
        self.datasets.read().await.values().cloned().collect()
    }
}

/// Dataset validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetValidationResult {
    pub dataset_id: Uuid,
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub task_count: usize,
    pub validated_at: DateTime<Utc>,
}

impl Default for DatasetManager {
    fn default() -> Self {
        Self::new()
    }
}

