//! Durable Learning State Persistence
//!
//! Implements comprehensive persistence layer for reflexive learning system,
//! enabling durable storage and retrieval of learning state, worker improvements,
//! performance metrics, and configuration across system restarts.

use crate::types::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Learning persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPersistenceConfig {
    /// Base directory for learning data storage
    pub storage_path: PathBuf,
    /// Maximum age of learning data to retain (days)
    pub retention_days: u32,
    /// Maximum size of learning data (MB)
    pub max_storage_size_mb: u64,
    /// Enable compression for stored data
    pub compression_enabled: bool,
    /// Backup frequency (hours)
    pub backup_frequency_hours: u32,
    /// Number of backup files to retain
    pub backup_retention_count: u32,
}

impl Default for LearningPersistenceConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./learning-data"),
            retention_days: 90,
            max_storage_size_mb: 1024, // 1GB
            compression_enabled: true,
            backup_frequency_hours: 24,
            backup_retention_count: 7,
        }
    }
}

/// Learning state snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStateSnapshot {
    /// Unique snapshot identifier
    pub id: Uuid,
    /// Timestamp of snapshot creation
    pub timestamp: DateTime<Utc>,
    /// Worker performance models
    pub worker_models: HashMap<String, WorkerPerformanceModel>,
    /// Learning metrics and KPIs
    pub learning_metrics: LearningMetricsSnapshot,
    /// Task execution history
    pub task_history: Vec<TaskExecutionRecord>,
    /// Resource allocation patterns
    pub resource_patterns: Vec<ResourceAllocationPattern>,
    /// System configuration snapshot
    pub system_config: SystemConfigurationSnapshot,
}

/// Worker performance model for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceModel {
    /// Worker identifier
    pub worker_id: String,
    /// Performance metrics by task type
    pub task_performance: HashMap<TaskType, TaskPerformanceMetrics>,
    /// Learning progress indicators
    pub learning_progress: LearningProgressIndicators,
    /// Adaptation parameters
    pub adaptation_params: AdaptationParameters,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Task performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPerformanceMetrics {
    /// Number of tasks completed
    pub tasks_completed: u64,
    /// Average execution time
    pub avg_execution_time: f64,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
    /// Quality score (0.0-1.0)
    pub avg_quality_score: f64,
    /// Resource efficiency score
    pub resource_efficiency: f64,
    /// Recent performance trend
    pub performance_trend: PerformanceTrend,
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving { rate: f64 },
    Declining { rate: f64 },
    Stable,
    Volatile { variance: f64 },
}

/// Learning progress indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningProgressIndicators {
    /// Experience points accumulated
    pub experience_points: f64,
    /// Skill level by task type
    pub skill_levels: HashMap<TaskType, f64>,
    /// Learning rate adaptation
    pub learning_rate: f64,
    /// Confidence in predictions
    pub prediction_confidence: f64,
    /// Number of successful adaptations
    pub adaptation_count: u64,
}

/// Adaptation parameters for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationParameters {
    /// Exploration vs exploitation balance
    pub exploration_rate: f64,
    /// Learning rate for parameter updates
    pub adaptation_learning_rate: f64,
    /// Risk tolerance for experimentation
    pub risk_tolerance: f64,
    /// Resource allocation preferences
    pub resource_preferences: HashMap<String, f64>,
}

/// Learning metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetricsSnapshot {
    /// Overall system learning effectiveness
    pub system_learning_effectiveness: f64,
    /// Worker improvement rates
    pub worker_improvement_rates: HashMap<String, f64>,
    /// Task completion efficiency trends
    pub task_efficiency_trends: Vec<EfficiencyDataPoint>,
    /// Resource utilization optimization
    pub resource_optimization_score: f64,
    /// Quality improvement over time
    pub quality_improvement_trend: Vec<QualityDataPoint>,
}

/// Efficiency data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyDataPoint {
    pub timestamp: DateTime<Utc>,
    pub efficiency_score: f64,
    pub tasks_completed: u64,
    pub avg_execution_time: f64,
}

/// Quality data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDataPoint {
    pub timestamp: DateTime<Utc>,
    pub avg_quality_score: f64,
    pub quality_variance: f64,
}

/// Task execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionRecord {
    pub task_id: Uuid,
    pub worker_id: String,
    pub task_type: TaskType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub success: bool,
    pub quality_score: f64,
    pub resource_usage: ResourceUsageMetrics,
    pub learning_insights: Vec<String>,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub cpu_seconds: f64,
    pub memory_bytes: u64,
    pub network_bytes: u64,
    pub tokens_used: u64,
    pub api_calls: u64,
}

/// Resource allocation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocationPattern {
    pub pattern_id: Uuid,
    pub task_type: TaskType,
    pub complexity: TaskComplexity,
    pub optimal_worker_assignment: Vec<String>,
    pub resource_requirements: ResourceRequirements,
    pub success_probability: f64,
    pub last_validated: DateTime<Utc>,
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_cpu_cores: u32,
    pub min_memory_gb: f64,
    pub max_execution_time: chrono::Duration,
    pub network_bandwidth_mbps: f64,
    pub api_rate_limit: u32,
}

/// System configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigurationSnapshot {
    pub learning_config: LearningConfigSnapshot,
    pub worker_config: HashMap<String, WorkerConfiguration>,
    pub resource_limits: ResourceLimitsSnapshot,
    pub quality_thresholds: QualityThresholdsSnapshot,
}

/// Learning configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfigSnapshot {
    pub learning_rate: f64,
    pub exploration_rate: f64,
    pub adaptation_interval: chrono::Duration,
    pub evaluation_period: chrono::Duration,
    pub max_concurrent_tasks: u32,
}

/// Worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfiguration {
    pub capabilities: Vec<TaskType>,
    pub max_concurrent_tasks: u32,
    pub resource_limits: WorkerResourceLimits,
    pub specialization_score: HashMap<TaskType, f64>,
}

/// Worker resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResourceLimits {
    pub max_cpu_percent: f64,
    pub max_memory_gb: f64,
    pub max_tasks_per_hour: u32,
}

/// Resource limits snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsSnapshot {
    pub total_cpu_cores: u32,
    pub total_memory_gb: f64,
    pub max_concurrent_workers: u32,
    pub network_bandwidth_mbps: f64,
}

/// Quality thresholds snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholdsSnapshot {
    pub min_quality_score: f64,
    pub max_error_rate: f64,
    pub min_success_rate: f64,
    pub max_resource_waste: f64,
}

/// Learning persistence manager
pub struct LearningPersistenceManager {
    /// Configuration for persistence
    config: LearningPersistenceConfig,
    /// Current learning state
    current_state: Arc<RwLock<LearningStateSnapshot>>,
    /// Storage backend
    storage: Arc<dyn LearningStorageBackend>,
    /// Backup manager
    backup_manager: BackupManager,
}

impl LearningPersistenceManager {
    /// Create a new learning persistence manager
    pub async fn new(config: LearningPersistenceConfig) -> Result<Self> {
        // Ensure storage directory exists
        tokio::fs::create_dir_all(&config.storage_path)
            .await
            .context("Failed to create learning data storage directory")?;

        // Initialize storage backend
        let storage = Arc::new(FileSystemStorage::new(config.clone()).await?);

        // Initialize backup manager
        let backup_manager = BackupManager::new(config.clone(), storage.clone()).await?;

        // Load or create initial state
        let current_state = Self::load_or_create_initial_state(&*storage).await?;

        Ok(Self {
            config,
            current_state: Arc::new(RwLock::new(current_state)),
            storage,
            backup_manager,
        })
    }

    /// Load existing state or create initial state
    async fn load_or_create_initial_state(storage: &dyn LearningStorageBackend) -> Result<LearningStateSnapshot> {
        match storage.load_latest_snapshot().await {
            Ok(snapshot) => {
                info!("Loaded existing learning state snapshot: {}", snapshot.id);
                Ok(snapshot)
            }
            Err(_) => {
                info!("No existing learning state found, creating initial state");
                Self::create_initial_state()
            }
        }
    }

    /// Create initial learning state
    fn create_initial_state() -> LearningStateSnapshot {
        LearningStateSnapshot {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            worker_models: HashMap::new(),
            learning_metrics: LearningMetricsSnapshot {
                system_learning_effectiveness: 0.5,
                worker_improvement_rates: HashMap::new(),
                task_efficiency_trends: Vec::new(),
                resource_optimization_score: 0.5,
                quality_improvement_trend: Vec::new(),
            },
            task_history: Vec::new(),
            resource_patterns: Vec::new(),
            system_config: SystemConfigurationSnapshot {
                learning_config: LearningConfigSnapshot {
                    learning_rate: 0.01,
                    exploration_rate: 0.1,
                    adaptation_interval: chrono::Duration::hours(1),
                    evaluation_period: chrono::Duration::days(1),
                    max_concurrent_tasks: 10,
                },
                worker_config: HashMap::new(),
                resource_limits: ResourceLimitsSnapshot {
                    total_cpu_cores: 8,
                    total_memory_gb: 16.0,
                    max_concurrent_workers: 4,
                    network_bandwidth_mbps: 100.0,
                },
                quality_thresholds: QualityThresholdsSnapshot {
                    min_quality_score: 0.7,
                    max_error_rate: 0.1,
                    min_success_rate: 0.8,
                    max_resource_waste: 0.2,
                },
            },
        }
    }

    /// Save current learning state
    pub async fn save_state(&self) -> Result<()> {
        let state = self.current_state.read().await.clone();
        self.storage.save_snapshot(&state).await?;
        debug!("Saved learning state snapshot: {}", state.id);
        Ok(())
    }

    /// Update worker performance model
    pub async fn update_worker_model(&self, worker_id: String, model: WorkerPerformanceModel) -> Result<()> {
        let mut state = self.current_state.write().await;
        state.worker_models.insert(worker_id, model);
        state.timestamp = Utc::now();
        Ok(())
    }

    /// Get worker performance model
    pub async fn get_worker_model(&self, worker_id: &str) -> Option<WorkerPerformanceModel> {
        let state = self.current_state.read().await;
        state.worker_models.get(worker_id).cloned()
    }

    /// Record task execution
    pub async fn record_task_execution(&self, record: TaskExecutionRecord) -> Result<()> {
        let mut state = self.current_state.write().await;
        state.task_history.push(record);
        state.timestamp = Utc::now();

        // Maintain history size limit
        if state.task_history.len() > 10000 {
            state.task_history = state.task_history.split_off(state.task_history.len() - 5000);
        }

        Ok(())
    }

    /// Update learning metrics
    pub async fn update_learning_metrics(&self, metrics: LearningMetricsSnapshot) -> Result<()> {
        let mut state = self.current_state.write().await;
        state.learning_metrics = metrics;
        state.timestamp = Utc::now();
        Ok(())
    }

    /// Get current learning metrics
    pub async fn get_learning_metrics(&self) -> LearningMetricsSnapshot {
        let state = self.current_state.read().await;
        state.learning_metrics.clone()
    }

    /// Add resource allocation pattern
    pub async fn add_resource_pattern(&self, pattern: ResourceAllocationPattern) -> Result<()> {
        let mut state = self.current_state.write().await;
        state.resource_patterns.push(pattern);
        state.timestamp = Utc::now();
        Ok(())
    }

    /// Get resource patterns for task type
    pub async fn get_resource_patterns(&self, task_type: &TaskType) -> Vec<ResourceAllocationPattern> {
        let state = self.current_state.read().await;
        state.resource_patterns.iter()
            .filter(|p| &p.task_type == task_type)
            .cloned()
            .collect()
    }

    /// Update system configuration
    pub async fn update_system_config(&self, config: SystemConfigurationSnapshot) -> Result<()> {
        let mut state = self.current_state.write().await;
        state.system_config = config;
        state.timestamp = Utc::now();
        Ok(())
    }

    /// Get system configuration
    pub async fn get_system_config(&self) -> SystemConfigurationSnapshot {
        let state = self.current_state.read().await;
        state.system_config.clone()
    }

    /// Perform maintenance operations
    pub async fn perform_maintenance(&self) -> Result<()> {
        // Clean up old data
        self.cleanup_old_data().await?;

        // Create backup if needed
        self.backup_manager.perform_backup_if_needed().await?;

        // Optimize storage
        self.optimize_storage().await?;

        Ok(())
    }

    /// Clean up old data based on retention policy
    async fn cleanup_old_data(&self) -> Result<()> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        let mut state = self.current_state.write().await;

        // Clean up old task history
        state.task_history.retain(|record| record.end_time > cutoff_date);

        // Clean up old efficiency and quality data points
        state.learning_metrics.task_efficiency_trends
            .retain(|point| point.timestamp > cutoff_date);
        state.learning_metrics.quality_improvement_trend
            .retain(|point| point.timestamp > cutoff_date);

        // Clean up old resource patterns
        state.resource_patterns.retain(|pattern| pattern.last_validated > cutoff_date);

        debug!("Cleaned up old learning data older than {}", cutoff_date);
        Ok(())
    }

    /// Optimize storage usage
    async fn optimize_storage(&self) -> Result<()> {
        // Compress old snapshots if compression is enabled
        if self.config.compression_enabled {
            self.storage.compress_old_snapshots().await?;
        }

        // Remove duplicate or redundant data
        self.deduplicate_data().await?;

        Ok(())
    }

    /// Deduplicate stored data
    async fn deduplicate_data(&self) -> Result<()> {
        let mut state = self.current_state.write().await;

        // Remove duplicate resource patterns
        let mut seen_patterns = HashMap::new();
        state.resource_patterns.retain(|pattern| {
            let key = (pattern.task_type.clone(), pattern.complexity.clone());
            if let Some(existing) = seen_patterns.get(&key) {
                // Keep the more recent pattern
                pattern.last_validated > *existing
            } else {
                seen_patterns.insert(key, pattern.last_validated);
                true
            }
        });

        Ok(())
    }

    /// Export learning state for analysis
    pub async fn export_state(&self, path: &PathBuf) -> Result<()> {
        let state = self.current_state.read().await;
        let json_data = serde_json::to_string_pretty(&*state)
            .context("Failed to serialize learning state")?;

        tokio::fs::write(path, json_data).await
            .context("Failed to write learning state export")?;

        info!("Exported learning state to {}", path.display());
        Ok(())
    }

    /// Import learning state from backup
    pub async fn import_state(&self, path: &PathBuf) -> Result<()> {
        let json_data = tokio::fs::read_to_string(path).await
            .context("Failed to read learning state import file")?;

        let imported_state: LearningStateSnapshot = serde_json::from_str(&json_data)
            .context("Failed to deserialize learning state")?;

        let mut state = self.current_state.write().await;
        *state = imported_state;

        info!("Imported learning state from {}", path.display());
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStatistics> {
        self.storage.get_statistics().await
    }
}

/// Storage backend trait for learning data
#[async_trait::async_trait]
pub trait LearningStorageBackend: Send + Sync {
    /// Save a learning state snapshot
    async fn save_snapshot(&self, snapshot: &LearningStateSnapshot) -> Result<()>;

    /// Load the latest snapshot
    async fn load_latest_snapshot(&self) -> Result<LearningStateSnapshot>;

    /// Load a specific snapshot by ID
    async fn load_snapshot(&self, id: &Uuid) -> Result<LearningStateSnapshot>;

    /// List all available snapshots
    async fn list_snapshots(&self) -> Result<Vec<SnapshotMetadata>>;

    /// Delete a snapshot
    async fn delete_snapshot(&self, id: &Uuid) -> Result<()>;

    /// Compress old snapshots
    async fn compress_old_snapshots(&self) -> Result<()>;

    /// Get storage statistics
    async fn get_statistics(&self) -> Result<StorageStatistics>;
}

/// Snapshot metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub size_bytes: u64,
    pub compressed: bool,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    pub total_snapshots: u64,
    pub total_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub oldest_snapshot: Option<DateTime<Utc>>,
    pub newest_snapshot: Option<DateTime<Utc>>,
}

/// File system storage backend implementation
pub struct FileSystemStorage {
    config: LearningPersistenceConfig,
    snapshots_dir: PathBuf,
}

impl FileSystemStorage {
    /// Create new file system storage
    pub async fn new(config: LearningPersistenceConfig) -> Result<Self> {
        let snapshots_dir = config.storage_path.join("snapshots");
        tokio::fs::create_dir_all(&snapshots_dir).await?;

        Ok(Self {
            config,
            snapshots_dir,
        })
    }
}

#[async_trait::async_trait]
impl LearningStorageBackend for FileSystemStorage {
    async fn save_snapshot(&self, snapshot: &LearningStateSnapshot) -> Result<()> {
        let filename = format!("snapshot_{}_{}.json",
                              snapshot.timestamp.timestamp(),
                              snapshot.id);
        let filepath = self.snapshots_dir.join(filename);

        let json_data = serde_json::to_string_pretty(snapshot)
            .context("Failed to serialize snapshot")?;

        tokio::fs::write(&filepath, json_data).await
            .context("Failed to write snapshot file")?;

        debug!("Saved snapshot {} to {}", snapshot.id, filepath.display());
        Ok(())
    }

    async fn load_latest_snapshot(&self) -> Result<LearningStateSnapshot> {
        let mut entries = tokio::fs::read_dir(&self.snapshots_dir).await?;
        let mut latest_snapshot: Option<(DateTime<Utc>, PathBuf)> = None;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("snapshot_") && filename.ends_with(".json") {
                    // Extract timestamp from filename
                    if let Some(timestamp_str) = filename.strip_prefix("snapshot_")
                        .and_then(|s| s.split('_').next()) {
                        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                            let datetime = DateTime::from_timestamp(timestamp, 0)
                                .unwrap_or_else(|| Utc::now());

                            if let Some((latest_time, _)) = &latest_snapshot {
                                if datetime > *latest_time {
                                    latest_snapshot = Some((datetime, path));
                                }
                            } else {
                                latest_snapshot = Some((datetime, path));
                            }
                        }
                    }
                }
            }
        }

        match latest_snapshot {
            Some((_, path)) => {
                let json_data = tokio::fs::read_to_string(&path).await?;
                let snapshot: LearningStateSnapshot = serde_json::from_str(&json_data)?;
                Ok(snapshot)
            }
            None => Err(anyhow::anyhow!("No snapshots found")),
        }
    }

    async fn load_snapshot(&self, id: &Uuid) -> Result<LearningStateSnapshot> {
        let pattern = format!("*_{}*.json", id);
        let mut entries = tokio::fs::read_dir(&self.snapshots_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.contains(&id.to_string()) {
                    let json_data = tokio::fs::read_to_string(&path).await?;
                    let snapshot: LearningStateSnapshot = serde_json::from_str(&json_data)?;
                    return Ok(snapshot);
                }
            }
        }

        Err(anyhow::anyhow!("Snapshot {} not found", id))
    }

    async fn list_snapshots(&self) -> Result<Vec<SnapshotMetadata>> {
        let mut entries = tokio::fs::read_dir(&self.snapshots_dir).await?;
        let mut snapshots = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("snapshot_") && filename.ends_with(".json") {
                    let metadata = tokio::fs::metadata(&path).await?;
                    let size_bytes = metadata.len();

                    // Extract ID and timestamp from filename
                    let parts: Vec<&str> = filename.trim_end_matches(".json")
                        .split('_').collect();

                    if parts.len() >= 3 {
                        if let (Ok(timestamp), Ok(id)) = (
                            parts[1].parse::<i64>(),
                            Uuid::parse_str(parts[2])
                        ) {
                            let datetime = DateTime::from_timestamp(timestamp, 0)
                                .unwrap_or_else(|| Utc::now());

                            snapshots.push(SnapshotMetadata {
                                id,
                                timestamp: datetime,
                                size_bytes,
                                compressed: false, // TODO: Implement compression detection
                            });
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(snapshots)
    }

    async fn delete_snapshot(&self, id: &Uuid) -> Result<()> {
        let pattern = format!("*_{}*.json", id);
        let mut entries = tokio::fs::read_dir(&self.snapshots_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.contains(&id.to_string()) {
                    tokio::fs::remove_file(&path).await?;
                    debug!("Deleted snapshot {}", id);
                    return Ok(());
                }
            }
        }

        Err(anyhow::anyhow!("Snapshot {} not found for deletion", id))
    }

    async fn compress_old_snapshots(&self) -> Result<()> {
        // TODO: Implement snapshot compression using gzip or similar
        // For now, just log that compression would happen
        debug!("Snapshot compression not yet implemented");
        Ok(())
    }

    async fn get_statistics(&self) -> Result<StorageStatistics> {
        let snapshots = self.list_snapshots().await?;
        let total_snapshots = snapshots.len() as u64;
        let total_size_bytes = snapshots.iter().map(|s| s.size_bytes).sum();

        let oldest_snapshot = snapshots.last().map(|s| s.timestamp);
        let newest_snapshot = snapshots.first().map(|s| s.timestamp);

        Ok(StorageStatistics {
            total_snapshots,
            total_size_bytes,
            compressed_size_bytes: 0, // TODO: Implement compression tracking
            oldest_snapshot,
            newest_snapshot,
        })
    }
}

/// Backup manager for learning data
pub struct BackupManager {
    config: LearningPersistenceConfig,
    storage: Arc<dyn LearningStorageBackend>,
    last_backup: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl BackupManager {
    /// Create new backup manager
    pub async fn new(config: LearningPersistenceConfig, storage: Arc<dyn LearningStorageBackend>) -> Result<Self> {
        Ok(Self {
            config,
            storage,
            last_backup: Arc::new(RwLock::new(None)),
        })
    }

    /// Perform backup if needed based on schedule
    pub async fn perform_backup_if_needed(&self) -> Result<()> {
        let last_backup = *self.last_backup.read().await;
        let now = Utc::now();

        let needs_backup = match last_backup {
            Some(last) => {
                let hours_since_backup = (now - last).num_hours();
                hours_since_backup >= self.config.backup_frequency_hours as i64
            }
            None => true,
        };

        if needs_backup {
            self.perform_backup().await?;
            *self.last_backup.write().await = Some(now);
        }

        Ok(())
    }

    /// Perform immediate backup
    pub async fn perform_backup(&self) -> Result<()> {
        let backup_dir = self.config.storage_path.join("backups");
        tokio::fs::create_dir_all(&backup_dir).await?;

        let timestamp = Utc::now().timestamp();
        let backup_filename = format!("backup_{}.tar.gz", timestamp);
        let backup_path = backup_dir.join(backup_filename);

        // TODO: Implement actual tar.gz creation
        // For now, just copy the latest snapshot
        if let Ok(latest_snapshot) = self.storage.load_latest_snapshot().await {
            let json_data = serde_json::to_string_pretty(&latest_snapshot)?;
            tokio::fs::write(&backup_path, json_data).await?;
        }

        // Clean up old backups
        self.cleanup_old_backups().await?;

        info!("Created backup: {}", backup_path.display());
        Ok(())
    }

    /// Clean up old backup files
    async fn cleanup_old_backups(&self) -> Result<()> {
        let backup_dir = self.config.storage_path.join("backups");
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(&backup_dir).await?;
        let mut backup_files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("backup_") && filename.ends_with(".tar.gz") {
                    if let Ok(metadata) = tokio::fs::metadata(&path).await {
                        let modified = metadata.modified()?;
                        backup_files.push((path, modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess backups
        if backup_files.len() > self.config.backup_retention_count as usize {
            for (path, _) in backup_files.iter().skip(self.config.backup_retention_count as usize) {
                tokio::fs::remove_file(path).await?;
                debug!("Removed old backup: {}", path.display());
            }
        }

        Ok(())
    }
}
