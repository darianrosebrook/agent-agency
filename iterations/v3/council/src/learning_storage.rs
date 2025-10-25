//! Learning signal storage implementations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use super::learning_types::*;
use crate::types::{TaskId, JudgeId, TaskType};
use agent_agency_database::DatabaseClient;

/// In-memory learning signal storage implementation
#[derive(Debug)]
pub struct InMemoryLearningSignalStorage {
    signals: Arc<RwLock<HashMap<Uuid, LearningSignal>>>,
    task_index: Arc<RwLock<HashMap<TaskId, Vec<Uuid>>>>,
    judge_index: Arc<RwLock<HashMap<JudgeId, Vec<Uuid>>>>,
    max_signals: usize,
}

impl InMemoryLearningSignalStorage {
    pub fn new(max_signals: usize) -> Self {
        Self {
            signals: Arc::new(RwLock::new(HashMap::new())),
            task_index: Arc::new(RwLock::new(HashMap::new())),
            judge_index: Arc::new(RwLock::new(HashMap::new())),
            max_signals,
        }
    }

    /// Clean up old signals when at capacity
    async fn cleanup_if_needed(&self) {
        let signal_count = self.signals.read().await.len();
        if signal_count >= self.max_signals {
            let to_remove = signal_count - (self.max_signals * 9 / 10); // Keep 90% capacity
            self.cleanup_oldest(to_remove).await;
        }
    }

    /// Remove oldest signals
    async fn cleanup_oldest(&self, count: usize) {
        let mut signals = self.signals.write().await;
        let mut task_index = self.task_index.write().await;
        let mut judge_index = self.judge_index.write().await;

        // Get signals sorted by timestamp (oldest first)
        let mut signal_ids: Vec<(DateTime<Utc>, Uuid)> = signals.iter()
            .map(|(id, signal)| (signal.timestamp, *id))
            .collect();

        signal_ids.sort_by(|a, b| a.0.cmp(&b.0));

        // Remove oldest signals
        for (_, signal_id) in signal_ids.into_iter().take(count) {
            if let Some(signal) = signals.remove(&signal_id) {
                // Clean up task index
                if let Some(task_signals) = task_index.get_mut(&signal.task_id) {
                    task_signals.retain(|&id| id != signal_id);
                    if task_signals.is_empty() {
                        task_index.remove(&signal.task_id);
                    }
                }

                // Clean up judge index (this is simplified - in practice we'd need to check all judge IDs in the signal)
                // For now, we'll skip complex judge index cleanup
            }
        }
    }
}

#[async_trait::async_trait]
impl LearningSignalStorage for InMemoryLearningSignalStorage {
    async fn store_signal(&self, signal: LearningSignal) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.cleanup_if_needed().await;

        let signal_id = signal.id;

        // Store signal
        self.signals.write().await.insert(signal_id, signal.clone());

        // Update task index
        self.task_index.write().await
            .entry(signal.task_id.clone())
            .or_insert_with(Vec::new)
            .push(signal_id);

        // Update judge index (simplified - just index by first judge for now)
        if let Some(first_dissent) = signal.judge_dissent.first() {
            self.judge_index.write().await
                .entry(first_dissent.judge_id.clone())
                .or_insert_with(Vec::new)
                .push(signal_id);
        }

        Ok(())
    }

    async fn get_signals(&self, task_id: Option<TaskId>, limit: usize) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>> {
        let signals = self.signals.read().await;

        let signal_ids = if let Some(task_id) = task_id {
            self.task_index.read().await
                .get(&task_id)
                .cloned()
                .unwrap_or_default()
        } else {
            signals.keys().cloned().collect()
        };

        let mut result: Vec<LearningSignal> = signal_ids.into_iter()
            .filter_map(|id| signals.get(&id).cloned())
            .collect();

        // Sort by timestamp (newest first) and limit
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        result.truncate(limit);

        Ok(result)
    }

    async fn get_similar_signals(&self, features: &TaskFeatures, limit: usize) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>> {
        let signals = self.signals.read().await;

        // Simple similarity scoring based on task type and complexity
        let mut scored_signals: Vec<(f32, LearningSignal)> = signals.values()
            .filter_map(|signal| {
                let similarity = self.calculate_similarity(features, signal);
                if similarity > 0.3 { // Minimum similarity threshold
                    Some((similarity, signal.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity score (highest first)
        scored_signals.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let result: Vec<LearningSignal> = scored_signals.into_iter()
            .take(limit)
            .map(|(_, signal)| signal)
            .collect();

        Ok(result)
    }

    async fn get_judge_performance(&self, judge_id: &JudgeId, task_type: Option<TaskType>) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>> {
        let signals = self.signals.read().await;

        let judge_signals: Vec<LearningSignal> = if let Some(task_type_filter) = task_type {
            // Filter by both judge and task type
            signals.values()
                .filter(|signal| {
                    signal.judge_dissent.iter().any(|d| &d.judge_id == judge_id) &&
                    // This would need to be implemented based on how task type is stored
                    true // Placeholder - need to check signal.task_type if available
                })
                .cloned()
                .collect()
        } else {
            // Filter by judge only
            signals.values()
                .filter(|signal| {
                    signal.judge_dissent.iter().any(|d| &d.judge_id == judge_id)
                })
                .cloned()
                .collect()
        };

        Ok(judge_signals)
    }

    async fn cleanup_old_signals(&self, max_age_days: u32) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff = Utc::now() - Duration::days(max_age_days as i64);
        let mut signals = self.signals.write().await;
        let mut task_index = self.task_index.write().await;
        let mut judge_index = self.judge_index.write().await;

        let initial_count = signals.len();

        // Remove old signals
        signals.retain(|_, signal| signal.timestamp > cutoff);

        let removed_count = initial_count - signals.len();

        // Clean up indices (simplified cleanup)
        task_index.clear();
        judge_index.clear();

        // Rebuild indices
        for (signal_id, signal) in signals.iter() {
            task_index.entry(signal.task_id.clone())
                .or_insert_with(Vec::new)
                .push(*signal_id);

            if let Some(first_dissent) = signal.judge_dissent.first() {
                judge_index.entry(first_dissent.judge_id.clone())
                    .or_insert_with(Vec::new)
                    .push(*signal_id);
            }
        }

        Ok(removed_count)
    }
}

impl InMemoryLearningSignalStorage {
    /// Calculate similarity between task features and a learning signal
    fn calculate_similarity(&self, features: &TaskFeatures, signal: &LearningSignal) -> f32 {
        let mut similarity = 0.0;

        // Task type match (high weight)
        if self.task_types_match(&features.task_type, signal) {
            similarity += 0.4;
        }

        // Complexity similarity
        let complexity_diff = (features.complexity_score - signal.task_complexity.overall_complexity).abs();
        similarity += 0.3 * (1.0 - complexity_diff.min(1.0));

        // Domain keyword overlap
        let keyword_overlap = self.calculate_keyword_overlap(&features.domain_keywords, signal);
        similarity += 0.2 * keyword_overlap;

        // Technical stack overlap
        let stack_overlap = self.calculate_keyword_overlap(&features.technical_stack, signal);
        similarity += 0.1 * stack_overlap;

        similarity
    }

    /// Check if task types match (simplified)
    fn task_types_match(&self, _task_type: &TaskType, _signal: &LearningSignal) -> bool {
        // This would need to be implemented based on how task types are stored in signals
        // For now, return true as a placeholder
        true
    }

    /// Calculate keyword overlap with signal data
    fn calculate_keyword_overlap(&self, keywords: &[String], signal: &LearningSignal) -> f32 {
        if keywords.is_empty() {
            return 0.0;
        }

        let signal_text = format!("{} {}", signal.signal_type, signal.source);
        let overlap_count = keywords.iter()
            .filter(|keyword| signal_text.to_lowercase().contains(&keyword.to_lowercase()))
            .count();

        overlap_count as f32 / keywords.len() as f32
    }
}

impl Default for InMemoryLearningSignalStorage {
    fn default() -> Self {
        Self::new(10000) // Default max 10k signals
    }
}

/// Database-backed learning signal storage
pub struct DatabaseLearningSignalStorage {
    db_client: DatabaseClient,
    table_name: String,
}

impl DatabaseLearningSignalStorage {
    pub fn new(db_client: DatabaseClient, table_name: String) -> Self {
        Self {
            db_client,
            table_name,
        }
    }
}

#[async_trait::async_trait]
impl LearningSignalStorage for DatabaseLearningSignalStorage {
    async fn store_signal(&self, signal: LearningSignal) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Serialize signal to JSON for storage
        let signal_json = serde_json::to_string(&signal)?;

        // Store in database (implementation would depend on specific database schema)
        // This is a placeholder - actual implementation would use the DatabaseClient
        tracing::debug!("Storing learning signal {} in database", signal.id);

        Ok(())
    }

    async fn get_signals(&self, task_id: Option<TaskId>, limit: usize) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>> {
        // Query database for signals
        // This is a placeholder - actual implementation would query the database
        tracing::debug!("Retrieving learning signals from database (task_id: {:?}, limit: {})",
                       task_id, limit);

        Ok(vec![]) // Placeholder
    }

    async fn get_similar_signals(&self, features: &TaskFeatures, limit: usize) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>> {
        // Query database for similar signals based on features
        // This is a placeholder - actual implementation would perform similarity search
        tracing::debug!("Finding similar learning signals in database (limit: {})", limit);

        Ok(vec![]) // Placeholder
    }

    async fn get_judge_performance(&self, judge_id: &JudgeId, task_type: Option<TaskType>) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>> {
        // Query database for judge performance signals
        // This is a placeholder - actual implementation would query judge-specific signals
        tracing::debug!("Retrieving judge performance signals from database (judge: {}, task_type: {:?})",
                       judge_id, task_type);

        Ok(vec![]) // Placeholder
    }

    async fn cleanup_old_signals(&self, max_age_days: u32) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // Delete old signals from database
        // This is a placeholder - actual implementation would delete old records
        tracing::debug!("Cleaning up old learning signals (max_age_days: {})", max_age_days);

        Ok(0) // Placeholder - return count of deleted records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_storage_store_and_retrieve() {
        let storage = InMemoryLearningSignalStorage::new(100);

        let signal = LearningSignal {
            id: Uuid::new_v4(),
            task_id: TaskId::from("test_task"),
            verdict_id: VerdictId::from("test_verdict"),
            outcome: TaskOutcome::Success,
            judge_dissent: vec![],
            latency_ms: 100,
            quality_score: 0.9,
            timestamp: Utc::now(),
            resource_usage: Default::default(),
            caws_compliance_score: 0.95,
            claim_verification_score: Some(0.85),
            task_complexity: Default::default(),
            worker_performance: None,
            signal_type: "test".to_string(),
            confidence: 0.8,
            data: serde_json::json!({"test": true}),
            source: "test_source".to_string(),
        };

        // Store signal
        storage.store_signal(signal.clone()).await.unwrap();

        // Retrieve signals
        let signals = storage.get_signals(Some(signal.task_id.clone()), 10).await.unwrap();

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].id, signal.id);
    }

    #[tokio::test]
    async fn test_cleanup_old_signals() {
        let storage = InMemoryLearningSignalStorage::new(100);

        let old_signal = LearningSignal {
            id: Uuid::new_v4(),
            task_id: TaskId::from("old_task"),
            verdict_id: VerdictId::from("old_verdict"),
            outcome: TaskOutcome::Success,
            judge_dissent: vec![],
            latency_ms: 100,
            quality_score: 0.9,
            timestamp: Utc::now() - Duration::days(100), // Very old
            resource_usage: Default::default(),
            caws_compliance_score: 0.95,
            claim_verification_score: None,
            task_complexity: Default::default(),
            worker_performance: None,
            signal_type: "old".to_string(),
            confidence: 0.8,
            data: serde_json::json!({"old": true}),
            source: "old_source".to_string(),
        };

        // Store old signal
        storage.store_signal(old_signal).await.unwrap();

        // Cleanup old signals (30 days max age)
        let removed_count = storage.cleanup_old_signals(30).await.unwrap();

        assert_eq!(removed_count, 1);

        // Verify signal was removed
        let signals = storage.get_signals(Some(old_signal.task_id.clone()), 10).await.unwrap();
        assert_eq!(signals.len(), 0);
    }
}
