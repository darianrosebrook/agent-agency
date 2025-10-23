//! Feature store with lineage tracking and PII policies

use crate::types::TaskId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

/// Feature vector for task characterization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub task_complexity: f64,
    pub estimated_lines: f64,
    pub file_count: f64,
    pub error_count: f64,
    pub pattern_hash: String,
    pub domain_features: HashMap<String, f64>,
    pub pii_redacted: bool,
}

/// Feature store trait for reproducible feature extraction
#[async_trait]
pub trait FeatureStore: Send + Sync {
    /// Get features for a task
    async fn get_task_features(&self, task_id: &TaskId) -> anyhow::Result<FeatureVector>;
    
    /// Get feature schema version
    fn version(&self) -> String;
    
    /// Check if features contain PII
    fn contains_pii(&self, features: &FeatureVector) -> bool;
    
    /// Redact PII from features
    fn redact_pii(&self, features: &mut FeatureVector);
}

/// In-memory feature store implementation
pub struct InMemoryFeatureStore {
    features: DashMap<TaskId, FeatureVector>,
    version: String,
}

impl InMemoryFeatureStore {
    /// Create a new in-memory feature store
    pub fn new(version: String) -> Self {
        Self {
            features: DashMap::new(),
            version,
        }
    }
    
    /// Store features for a task
    pub fn store_features(&self, task_id: TaskId, features: FeatureVector) {
        self.features.insert(task_id, features);
    }
    
    /// Get all stored features
    pub fn get_all_features(&self) -> Vec<(TaskId, FeatureVector)> {
        self.features.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect()
    }
}

#[async_trait]
impl FeatureStore for InMemoryFeatureStore {
    async fn get_task_features(&self, task_id: &TaskId) -> anyhow::Result<FeatureVector> {
        self.features
            .get(task_id)
            .map(|f| f.clone())
            .ok_or_else(|| anyhow::anyhow!("Features not found for task {}", task_id))
    }
    
    fn version(&self) -> String {
        self.version.clone()
    }
    
    fn contains_pii(&self, features: &FeatureVector) -> bool {
        !features.pii_redacted && !features.domain_features.is_empty()
    }
    
    fn redact_pii(&self, features: &mut FeatureVector) {
        features.domain_features.clear();
        features.pii_redacted = true;
    }
}

/// Feature extractor for task analysis
pub struct TaskFeatureExtractor {
    feature_store: Arc<dyn FeatureStore>,
}

impl TaskFeatureExtractor {
    /// Create a new feature extractor
    pub fn new(feature_store: Arc<dyn FeatureStore>) -> Self {
        Self { feature_store }
    }
    
    /// Extract features from a task
    pub async fn extract_features(&self, task: &crate::types::ComplexTask) -> anyhow::Result<FeatureVector> {
        let complexity = self.calculate_complexity(task);
        let estimated_lines = self.estimate_lines(task);
        let file_count = self.count_files(task);
        let error_count = self.count_errors(task);
        let pattern_hash = self.generate_pattern_hash(task);
        let domain_features = self.extract_domain_features(task);
        
        Ok(FeatureVector {
            task_complexity: complexity,
            estimated_lines,
            file_count,
            error_count,
            pattern_hash,
            domain_features,
            pii_redacted: false,
        })
    }
    
    /// Calculate task complexity score
    fn calculate_complexity(&self, task: &crate::types::ComplexTask) -> f64 {
        // Simple complexity calculation based on scope and requirements
        let scope_size = task.scope.files.len();
        let requirements_count = task.quality_requirements.required_gates.len();
        
        // Normalize to 0-1 range
        (scope_size as f64 * 0.1 + requirements_count as f64 * 0.2).min(1.0)
    }
    
    /// Estimate lines of code
    fn estimate_lines(&self, task: &crate::types::ComplexTask) -> f64 {
        // Rough estimation based on scope
        task.scope.files.len() as f64 * 50.0 // Assume 50 lines per file on average
    }
    
    /// Count files in scope
    fn count_files(&self, task: &crate::types::ComplexTask) -> f64 {
        task.scope.files.len() as f64
    }
    
    /// Count errors in task description
    fn count_errors(&self, task: &crate::types::ComplexTask) -> f64 {
        // Simple error count based on description keywords
        let description = task.description.to_lowercase();
        let error_keywords = ["error", "bug", "fix", "issue", "problem", "fail"];
        
        error_keywords.iter()
            .map(|keyword| description.matches(keyword).count() as f64)
            .sum()
    }
    
    /// Generate pattern hash for task
    fn generate_pattern_hash(&self, task: &crate::types::ComplexTask) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        task.scope.files.hash(&mut hasher);
        task.quality_requirements.required_gates.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Extract domain-specific features
    fn extract_domain_features(&self, task: &crate::types::ComplexTask) -> HashMap<String, f64> {
        let mut features = HashMap::new();
        
        // Analyze file extensions for domain hints
        let mut rust_files = 0;
        let mut js_files = 0;
        let mut py_files = 0;
        
        for file in &task.scope.files {
            if file.ends_with(".rs") {
                rust_files += 1;
            } else if file.ends_with(".js") || file.ends_with(".ts") {
                js_files += 1;
            } else if file.ends_with(".py") {
                py_files += 1;
            }
        }
        
        let total_files = task.scope.files.len() as f64;
        if total_files > 0.0 {
            features.insert("rust_ratio".to_string(), rust_files as f64 / total_files);
            features.insert("js_ratio".to_string(), js_files as f64 / total_files);
            features.insert("py_ratio".to_string(), py_files as f64 / total_files);
        }
        
        // Add complexity indicators
        features.insert("has_tests".to_string(), if task.description.to_lowercase().contains("test") { 1.0 } else { 0.0 });
        features.insert("has_docs".to_string(), if task.description.to_lowercase().contains("doc") { 1.0 } else { 0.0 });
        
        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexTask, TaskScope, QualityRequirements};

    fn create_test_task() -> ComplexTask {
        ComplexTask {
            id: crate::types::TaskId::new(),
            description: "Fix compilation errors in Rust code".to_string(),
            scope: TaskScope {
                files: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
                directories: vec!["src/".to_string()],
            },
            quality_requirements: QualityRequirements {
                required_gates: vec!["compilation".to_string(), "tests".to_string()],
                timeout_seconds: Some(300),
            },
            priority: crate::types::Priority::Normal,
            context: crate::types::TaskContext {
                timeout: Some(std::time::Duration::from_secs(300)),
            },
        }
    }

    #[tokio::test]
    async fn test_feature_extraction() {
        let store = Arc::new(InMemoryFeatureStore::new("1.0.0".to_string()));
        let extractor = TaskFeatureExtractor::new(store);
        
        let task = create_test_task();
        let features = extractor.extract_features(&task).await.unwrap();
        
        assert!(features.task_complexity > 0.0);
        assert!(features.estimated_lines > 0.0);
        assert_eq!(features.file_count, 2.0);
        assert!(features.pattern_hash.len() > 0);
    }
    
    #[tokio::test]
    async fn test_feature_store() {
        let store = InMemoryFeatureStore::new("1.0.0".to_string());
        let task_id = crate::types::TaskId::new();
        
        let features = FeatureVector {
            task_complexity: 0.5,
            estimated_lines: 100.0,
            file_count: 2.0,
            error_count: 1.0,
            pattern_hash: "abc123".to_string(),
            domain_features: HashMap::new(),
            pii_redacted: false,
        };
        
        store.store_features(task_id.clone(), features.clone());
        
        let retrieved = store.get_task_features(&task_id).await.unwrap();
        assert_eq!(retrieved.task_complexity, features.task_complexity);
    }
    
    #[test]
    fn test_pii_detection() {
        let store = InMemoryFeatureStore::new("1.0.0".to_string());
        
        let mut features = FeatureVector {
            task_complexity: 0.5,
            estimated_lines: 100.0,
            file_count: 2.0,
            error_count: 1.0,
            pattern_hash: "abc123".to_string(),
            domain_features: HashMap::new(),
            pii_redacted: false,
        };
        
        // Should not contain PII initially
        assert!(!store.contains_pii(&features));
        
        // Add some domain features
        features.domain_features.insert("user_data".to_string(), 0.5);
        
        // Should now contain PII
        assert!(store.contains_pii(&features));
        
        // Redact PII
        store.redact_pii(&mut features);
        assert!(features.pii_redacted);
        assert!(features.domain_features.is_empty());
    }
}
