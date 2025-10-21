/// Model registry for hot-swapping
///
/// Manages model versions, metadata, and lifecycle during hot-swapping operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Model registry for tracking model versions and metadata
pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<String, ModelEntry>>>,
    version_history: Arc<RwLock<Vec<ModelVersion>>>,
}

/// Model entry in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Unique model identifier
    pub id: String,
    /// Current active version
    pub active_version: String,
    /// Model metadata
    pub metadata: ModelMetadata,
    /// Deployment status
    pub status: DeploymentStatus,
    /// Performance metrics
    pub performance: ModelPerformance,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model architecture type
    pub architecture: String,
    /// Input/output specifications
    pub input_spec: TensorSpec,
    pub output_spec: TensorSpec,
    /// Training framework used
    pub framework: String,
    /// Model size in MB
    pub size_mb: f32,
    /// Target hardware
    pub target_hardware: Vec<String>,
}

/// Tensor specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorSpec {
    /// Tensor shape (batch_size, ...)
    pub shape: Vec<usize>,
    /// Data type
    pub dtype: String,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Model is being prepared for deployment
    Preparing,
    /// Model is ready for deployment
    Ready,
    /// Model is currently active
    Active,
    /// Model is being phased out
    Deprecated,
    /// Model has been retired
    Retired,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f32,
    /// Throughput in requests per second
    pub throughput_rps: f32,
    /// Error rate (0.0-1.0)
    pub error_rate: f32,
    /// Accuracy score (0.0-1.0)
    pub accuracy_score: f32,
    /// Memory usage in MB
    pub memory_usage_mb: f32,
}

/// Model version record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Model identifier
    pub model_id: String,
    /// Version identifier
    pub version: String,
    /// Version type
    pub version_type: VersionType,
    /// Deployment timestamp
    pub deployed_at: chrono::DateTime<chrono::Utc>,
    /// Performance at deployment time
    pub initial_performance: ModelPerformance,
}

/// Version type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionType {
    /// Major version with breaking changes
    Major,
    /// Minor version with new features
    Minor,
    /// Patch version with bug fixes
    Patch,
    /// Experimental/canary version
    Experimental,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            version_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new model
    pub async fn register_model(&self, entry: ModelEntry) -> Result<()> {
        let mut models = self.models.write().await;

        if models.contains_key(&entry.id) {
            return Err(anyhow::anyhow!("Model {} already exists", entry.id));
        }

        models.insert(entry.id.clone(), entry.clone());

        // Record version
        let version = ModelVersion {
            model_id: entry.id.clone(),
            version: entry.active_version.clone(),
            version_type: VersionType::Major,
            deployed_at: entry.created_at,
            initial_performance: entry.performance.clone(),
        };

        let mut history = self.version_history.write().await;
        history.push(version);

        info!("Registered new model: {}", entry.id);
        Ok(())
    }

    /// Update model to new version
    pub async fn update_model_version(&self, model_id: &str, new_version: &str, performance: ModelPerformance) -> Result<()> {
        let mut models = self.models.write().await;

        let model = models.get_mut(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model {} not found", model_id))?;

        let old_version = model.active_version.clone();
        model.active_version = new_version.to_string();
        model.performance = performance.clone();
        model.updated_at = chrono::Utc::now();
        model.status = DeploymentStatus::Active;

        // Record version change
        let version = ModelVersion {
            model_id: model_id.to_string(),
            version: new_version.to_string(),
            version_type: VersionType::Minor, // Assume minor update
            deployed_at: model.updated_at,
            initial_performance: performance,
        };

        let mut history = self.version_history.write().await;
        history.push(version);

        info!("Updated model {} from version {} to {}", model_id, old_version, new_version);
        Ok(())
    }

    /// Get model entry by ID
    pub async fn get_model(&self, model_id: &str) -> Result<ModelEntry> {
        let models = self.models.read().await;
        models.get(model_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Model {} not found", model_id))
    }

    /// List all registered models
    pub async fn list_models(&self) -> Vec<ModelEntry> {
        let models = self.models.read().await;
        models.values().cloned().collect()
    }

    /// Get active models only
    pub async fn get_active_models(&self) -> Vec<ModelEntry> {
        let models = self.models.read().await;
        models.values()
            .filter(|model| matches!(model.status, DeploymentStatus::Active))
            .cloned()
            .collect()
    }

    /// Deprecate a model version
    pub async fn deprecate_model(&self, model_id: &str) -> Result<()> {
        let mut models = self.models.write().await;

        let model = models.get_mut(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model {} not found", model_id))?;

        model.status = DeploymentStatus::Deprecated;
        model.updated_at = chrono::Utc::now();

        info!("Deprecated model: {}", model_id);
        Ok(())
    }

    /// Remove a retired model
    pub async fn remove_model(&self, model_id: &str) -> Result<()> {
        let mut models = self.models.write().await;

        if !models.contains_key(model_id) {
            return Err(anyhow::anyhow!("Model {} not found", model_id));
        }

        models.remove(model_id);

        info!("Removed model: {}", model_id);
        Ok(())
    }

    /// Get version history for a model
    pub async fn get_version_history(&self, model_id: &str) -> Vec<ModelVersion> {
        let history = self.version_history.read().await;
        history.iter()
            .filter(|version| version.model_id == model_id)
            .cloned()
            .collect()
    }

    /// Validate model compatibility for hot-swapping
    pub async fn validate_compatibility(&self, source_model: &str, target_model: &str) -> Result<CompatibilityResult> {
        let models = self.models.read().await;

        let source = models.get(source_model)
            .ok_or_else(|| anyhow::anyhow!("Source model {} not found", source_model))?;

        let target = models.get(target_model)
            .ok_or_else(|| anyhow::anyhow!("Target model {} not found", target_model))?;

        // Check input/output compatibility
        let input_compatible = source.metadata.input_spec.shape == target.metadata.input_spec.shape &&
                              source.metadata.input_spec.dtype == target.metadata.input_spec.dtype;

        let output_compatible = source.metadata.output_spec.shape == target.metadata.output_spec.shape &&
                               source.metadata.output_spec.dtype == target.metadata.output_spec.dtype;

        let compatible = input_compatible && output_compatible;

        let issues = Vec::new(); // Would populate with specific compatibility issues

        Ok(CompatibilityResult {
            compatible,
            input_compatible,
            output_compatible,
            issues,
        })
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        let models = self.models.read().await;
        let history = self.version_history.read().await;

        let total_models = models.len();
        let active_models = models.values()
            .filter(|model| matches!(model.status, DeploymentStatus::Active))
            .count();
        let total_versions = history.len();

        RegistryStatistics {
            total_models,
            active_models,
            total_versions,
            last_updated: models.values()
                .map(|model| model.updated_at)
                .max()
                .unwrap_or(chrono::Utc::now()),
        }
    }
}

/// Compatibility check result
#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub compatible: bool,
    pub input_compatible: bool,
    pub output_compatible: bool,
    pub issues: Vec<String>,
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub total_models: usize,
    pub active_models: usize,
    pub total_versions: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_registration() {
        let registry = ModelRegistry::new();

        let metadata = ModelMetadata {
            architecture: "Transformer".to_string(),
            input_spec: TensorSpec {
                shape: vec![1, 512],
                dtype: "float32".to_string(),
            },
            output_spec: TensorSpec {
                shape: vec![1, 1000],
                dtype: "float32".to_string(),
            },
            framework: "PyTorch".to_string(),
            size_mb: 100.0,
            target_hardware: vec!["CPU".to_string(), "GPU".to_string()],
        };

        let performance = ModelPerformance {
            avg_response_time_ms: 50.0,
            throughput_rps: 20.0,
            error_rate: 0.01,
            accuracy_score: 0.95,
            memory_usage_mb: 200.0,
        };

        let entry = ModelEntry {
            id: "test_model".to_string(),
            active_version: "1.0.0".to_string(),
            metadata,
            status: DeploymentStatus::Ready,
            performance,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        registry.register_model(entry).await.unwrap();

        let retrieved = registry.get_model("test_model").await.unwrap();
        assert_eq!(retrieved.id, "test_model");
        assert_eq!(retrieved.active_version, "1.0.0");
    }

    #[tokio::test]
    async fn test_model_version_update() {
        let registry = ModelRegistry::new();

        // Register initial model
        let entry = ModelEntry {
            id: "test_model".to_string(),
            active_version: "1.0.0".to_string(),
            metadata: ModelMetadata {
                architecture: "Transformer".to_string(),
                input_spec: TensorSpec { shape: vec![1, 512], dtype: "float32".to_string() },
                output_spec: TensorSpec { shape: vec![1, 1000], dtype: "float32".to_string() },
                framework: "PyTorch".to_string(),
                size_mb: 100.0,
                target_hardware: vec!["CPU".to_string()],
            },
            status: DeploymentStatus::Active,
            performance: ModelPerformance {
                avg_response_time_ms: 50.0,
                throughput_rps: 20.0,
                error_rate: 0.01,
                accuracy_score: 0.95,
                memory_usage_mb: 200.0,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        registry.register_model(entry).await.unwrap();

        // Update to new version
        let new_performance = ModelPerformance {
            avg_response_time_ms: 40.0,
            throughput_rps: 25.0,
            error_rate: 0.005,
            accuracy_score: 0.97,
            memory_usage_mb: 180.0,
        };

        registry.update_model_version("test_model", "1.1.0", new_performance).await.unwrap();

        let updated = registry.get_model("test_model").await.unwrap();
        assert_eq!(updated.active_version, "1.1.0");
        assert_eq!(updated.performance.avg_response_time_ms, 40.0);
    }
}


