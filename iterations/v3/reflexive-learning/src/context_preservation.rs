//! Context preservation engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContext {
    pub session_id: Uuid,
    pub task_type: String,
    pub algorithm_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub progress: LearningProgress,
    pub environment: LearningEnvironment,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningProgress {
    pub epoch: u32,
    pub iterations: u64,
    pub loss_history: Vec<f64>,
    pub accuracy_history: Vec<f64>,
    pub best_model_state: Option<serde_json::Value>,
    pub checkpoint_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEnvironment {
    pub hardware_specs: HardwareSpecs,
    pub software_versions: HashMap<String, String>,
    pub configuration: HashMap<String, serde_json::Value>,
    pub data_characteristics: DataCharacteristics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    pub cpu_cores: usize,
    pub memory_gb: usize,
    pub gpu_info: Option<String>,
    pub disk_space_gb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCharacteristics {
    pub dataset_size: usize,
    pub feature_count: usize,
    pub data_types: Vec<String>,
    pub has_labels: bool,
    pub class_distribution: Option<HashMap<String, usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub tags: Vec<String>,
    pub description: String,
    pub author: String,
    pub project: String,
    pub is_public: bool,
}

#[derive(Debug)]
pub struct ContextPreservationEngine {
    contexts: Arc<RwLock<HashMap<Uuid, LearningContext>>>,
    metadata: Arc<RwLock<HashMap<Uuid, ContextMetadata>>>,
    storage_path: Option<std::path::PathBuf>,
    max_contexts: usize,
}

impl ContextPreservationEngine {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            storage_path: None,
            max_contexts: 1000,
        }
    }

    pub fn with_storage_path(mut self, path: std::path::PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    pub fn with_max_contexts(mut self, max: usize) -> Self {
        self.max_contexts = max;
        self
    }

    /// Capture and store learning context
    pub async fn capture_context(
        &self,
        session_id: Uuid,
        task_type: String,
        algorithm_type: String,
        parameters: HashMap<String, serde_json::Value>,
        progress: LearningProgress,
        environment: LearningEnvironment,
        metadata: Option<ContextMetadata>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now();

        let context = LearningContext {
            session_id,
            task_type,
            algorithm_type,
            parameters,
            progress,
            environment,
            created_at: now,
            updated_at: now,
            version: 1,
        };

        let mut contexts = self.contexts.write().await;

        // Check storage limits
        if contexts.len() >= self.max_contexts {
            // Remove oldest context
            if let Some(oldest_id) = self.find_oldest_context(&contexts).await {
                contexts.remove(&oldest_id);
                let mut metadata_store = self.metadata.write().await;
                metadata_store.remove(&oldest_id);
            }
        }

        contexts.insert(session_id, context);

        if let Some(meta) = metadata {
            let mut metadata_store = self.metadata.write().await;
            metadata_store.insert(session_id, meta);
        }

        // Persist to storage if configured
        if let Some(path) = &self.storage_path {
            self.persist_context(&context, path).await?;
        }

        Ok(())
    }

    /// Update existing learning context
    pub async fn update_context(
        &self,
        session_id: Uuid,
        progress_update: Option<LearningProgress>,
        parameter_updates: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut contexts = self.contexts.write().await;

        if let Some(context) = contexts.get_mut(&session_id) {
            if let Some(progress) = progress_update {
                context.progress = progress;
            }

            if let Some(params) = parameter_updates {
                context.parameters.extend(params);
            }

            context.updated_at = chrono::Utc::now();
            context.version += 1;

            // Persist updated context
            if let Some(path) = &self.storage_path {
                self.persist_context(context, path).await?;
            }
        }

        Ok(())
    }

    /// Retrieve learning context by session ID
    pub async fn get_context(&self, session_id: &Uuid) -> Option<LearningContext> {
        let contexts = self.contexts.read().await;
        contexts.get(session_id).cloned()
    }

    /// Search contexts by criteria
    pub async fn search_contexts(
        &self,
        task_type: Option<&str>,
        algorithm_type: Option<&str>,
        tags: Option<&[String]>,
        author: Option<&str>,
        limit: usize,
    ) -> Vec<(LearningContext, Option<ContextMetadata>)> {
        let contexts = self.contexts.read().await;
        let metadata = self.metadata.read().await;

        contexts
            .values()
            .filter(|ctx| {
                task_type.map_or(true, |t| ctx.task_type == t) &&
                algorithm_type.map_or(true, |a| ctx.algorithm_type == a)
            })
            .filter(|ctx| {
                if let Some(tag_list) = tags {
                    if let Some(meta) = metadata.get(&ctx.session_id) {
                        tag_list.iter().any(|tag| meta.tags.contains(tag))
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .filter(|ctx| {
                author.map_or(true, |a| {
                    metadata.get(&ctx.session_id)
                        .map(|m| m.author == a)
                        .unwrap_or(false)
                })
            })
            .take(limit)
            .map(|ctx| {
                let meta = metadata.get(&ctx.session_id).cloned();
                (ctx.clone(), meta)
            })
            .collect()
    }

    /// List all available contexts with metadata
    pub async fn list_contexts(&self, limit: usize) -> Vec<(LearningContext, Option<ContextMetadata>)> {
        let contexts = self.contexts.read().await;
        let metadata = self.metadata.read().await;

        contexts
            .values()
            .take(limit)
            .map(|ctx| {
                let meta = metadata.get(&ctx.session_id).cloned();
                (ctx.clone(), meta)
            })
            .collect()
    }

    /// Delete context by session ID
    pub async fn delete_context(&self, session_id: &Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut contexts = self.contexts.write().await;
        let mut metadata = self.metadata.write().await;

        let context_removed = contexts.remove(session_id).is_some();
        let metadata_removed = metadata.remove(session_id).is_some();

        // Remove from persistent storage if configured
        if let Some(path) = &self.storage_path {
            self.delete_persisted_context(session_id, path).await?;
        }

        Ok(context_removed || metadata_removed)
    }

    /// Clean up old contexts based on age
    pub async fn cleanup_old_contexts(&self, max_age_days: i64) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(max_age_days);
        let mut contexts = self.contexts.write().await;
        let mut metadata = self.metadata.write().await;

        let to_remove: Vec<Uuid> = contexts
            .iter()
            .filter(|(_, ctx)| ctx.updated_at < cutoff)
            .map(|(id, _)| *id)
            .collect();

        let removed_count = to_remove.len();

        for id in &to_remove {
            contexts.remove(id);
            metadata.remove(id);

            // Remove from persistent storage
            if let Some(path) = &self.storage_path {
                self.delete_persisted_context(id, path).await?;
            }
        }

        Ok(removed_count)
    }

    /// Analyze context patterns for learning insights
    pub async fn analyze_patterns(&self) -> Result<ContextAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let contexts = self.contexts.read().await;

        let mut task_type_distribution = HashMap::new();
        let mut algorithm_performance = HashMap::new();
        let mut learning_trends = Vec::new();

        for context in contexts.values() {
            // Count task types
            *task_type_distribution.entry(context.task_type.clone()).or_insert(0) += 1;

            // Analyze algorithm performance
            if !context.progress.accuracy_history.is_empty() {
                let final_accuracy = context.progress.accuracy_history.last().copied().unwrap_or(0.0);
                let algo_key = context.algorithm_type.clone();
                algorithm_performance.entry(algo_key)
                    .or_insert_with(Vec::new)
                    .push(final_accuracy);
            }

            // Extract learning trends
            if context.progress.loss_history.len() > 1 {
                let initial_loss = context.progress.loss_history[0];
                let final_loss = context.progress.loss_history.last().copied().unwrap_or(initial_loss);
                let improvement = initial_loss - final_loss;

                learning_trends.push(LearningTrend {
                    session_id: context.session_id,
                    algorithm: context.algorithm_type.clone(),
                    improvement_rate: improvement / context.progress.epoch as f64,
                    convergence_speed: context.progress.iterations as f64 / context.progress.epoch as f64,
                });
            }
        }

        // Calculate average performance per algorithm
        let mut avg_performance = HashMap::new();
        for (algo, accuracies) in &algorithm_performance {
            let avg = accuracies.iter().sum::<f64>() / accuracies.len() as f64;
            avg_performance.insert(algo.clone(), avg);
        }

        Ok(ContextAnalysis {
            task_type_distribution,
            algorithm_performance: avg_performance,
            learning_trends,
            total_contexts: contexts.len(),
        })
    }

    /// Generate learning recommendations based on context analysis
    pub async fn generate_recommendations(&self, current_task: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let analysis = self.analyze_patterns().await?;
        let mut recommendations = Vec::new();

        // Find best performing algorithms for similar tasks
        if let Some(best_algo) = analysis.algorithm_performance
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(algo, _)| algo.clone())
        {
            recommendations.push(format!("Consider using {} algorithm for {} tasks", best_algo, current_task));
        }

        // Analyze learning trends
        let avg_convergence = analysis.learning_trends
            .iter()
            .map(|t| t.convergence_speed)
            .sum::<f64>() / analysis.learning_trends.len().max(1) as f64;

        if avg_convergence > 1000.0 {
            recommendations.push("Consider increasing batch size for faster convergence".to_string());
        }

        Ok(recommendations)
    }

    async fn find_oldest_context(&self, contexts: &HashMap<Uuid, LearningContext>) -> Option<Uuid> {
        contexts
            .iter()
            .min_by_key(|(_, ctx)| ctx.updated_at)
            .map(|(id, _)| *id)
    }

    async fn persist_context(&self, context: &LearningContext, base_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filename = format!("context_{}.json", context.session_id);
        let path = base_path.join(filename);

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(context)?;
        tokio::fs::write(&path, json).await?;

        Ok(())
    }

    async fn delete_persisted_context(&self, session_id: &Uuid, base_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filename = format!("context_{}.json", session_id);
        let path = base_path.join(filename);

        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalysis {
    pub task_type_distribution: HashMap<String, usize>,
    pub algorithm_performance: HashMap<String, f64>,
    pub learning_trends: Vec<LearningTrend>,
    pub total_contexts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningTrend {
    pub session_id: Uuid,
    pub algorithm: String,
    pub improvement_rate: f64,
    pub convergence_speed: f64,
}
