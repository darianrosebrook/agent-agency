//! Memory Deduplication
//!
//! Identify and merge duplicate or highly similar memories.

use crate::consolidation::*;

/// Deduplication configuration
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    pub similarity_threshold: f32,
    pub time_window_hours: u64,
    pub preserve_most_recent: bool,
    pub merge_strategy: MergeStrategy,
}

/// Merge strategies for duplicate memories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    KeepNewest,
    KeepOldest,
    MergeContent,
    CreateSummary,
}

/// Duplicate detection result
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub canonical_memory: crate::memory_types::MemoryId,
    pub duplicate_memories: Vec<crate::memory_types::MemoryId>,
    pub similarity_score: f32,
    pub merge_recommendation: MergeRecommendation,
}

/// Merge recommendation
#[derive(Debug, Clone)]
pub enum MergeRecommendation {
    KeepCanonical,
    MergeIntoCanonical,
    CreateNewMerged,
    DiscardAll,
}

/// Memory deduplicator
pub struct MemoryDeduplicator {
    config: DeduplicationConfig,
}

impl MemoryDeduplicator {
    pub fn new(config: DeduplicationConfig) -> Self {
        Self { config }
    }

    /// Find duplicate memories across the entire memory store
    pub async fn find_duplicates(&self, memories: Vec<crate::memory_types::Memory>) -> crate::MemoryResult<Vec<DuplicateGroup>> {
        let mut duplicate_groups = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        for i in 0..memories.len() {
            if processed_ids.contains(&memories[i].id) {
                continue;
            }

            let mut duplicates = Vec::new();
            let mut max_similarity: f32 = 0.0;

            // Find duplicates for this memory
            for j in (i + 1)..memories.len() {
                if processed_ids.contains(&memories[j].id) {
                    continue;
                }

                let similarity = self.calculate_similarity(&memories[i], &memories[j]).await?;

                if similarity >= self.config.similarity_threshold {
                    duplicates.push(memories[j].id.clone());
                    max_similarity = max_similarity.max(similarity);
                    processed_ids.insert(memories[j].id.clone());
                }
            }

            if !duplicates.is_empty() {
                let recommendation = self.generate_merge_recommendation(&memories[i], &duplicates, max_similarity);
                let group = DuplicateGroup {
                    canonical_memory: memories[i].id.clone(),
                    duplicate_memories: duplicates,
                    similarity_score: max_similarity,
                    merge_recommendation: recommendation,
                };
                duplicate_groups.push(group);
            }

            processed_ids.insert(memories[i].id.clone());
        }

        Ok(duplicate_groups)
    }

    /// Find near-duplicate memories within a time window
    pub async fn find_temporal_duplicates(
        &self,
        memories: Vec<crate::memory_types::Memory>,
    ) -> crate::MemoryResult<Vec<DuplicateGroup>> {
        let mut duplicate_groups = Vec::new();
        let time_window = chrono::Duration::hours(self.config.time_window_hours as i64);

        // Group memories by time windows
        let mut time_groups = std::collections::HashMap::new();

        for memory in memories {
            let window_start = memory.created_at.timestamp() / (self.config.time_window_hours * 3600) * (self.config.time_window_hours * 3600);
            time_groups.entry(window_start).or_insert_with(Vec::new).push(memory);
        }

        // Find duplicates within each time window
        for (_window, window_memories) in time_groups {
            let window_duplicates = self.find_duplicates(window_memories).await?;
            duplicate_groups.extend(window_duplicates);
        }

        Ok(duplicate_groups)
    }

    /// Merge duplicate memories according to strategy
    pub async fn merge_duplicates(&self, group: &DuplicateGroup) -> crate::MemoryResult<crate::memory_types::Memory> {
        match &group.merge_recommendation {
            MergeRecommendation::KeepCanonical => {
                // Return canonical memory unchanged
                // In practice, this would fetch from database
                Err(crate::MemoryError::NotFound("Canonical memory not accessible".to_string()))
            }
            MergeRecommendation::MergeIntoCanonical => {
                // Merge duplicate content into canonical
                self.merge_into_canonical(group).await
            }
            MergeRecommendation::CreateNewMerged => {
                // Create new merged memory
                self.create_merged_memory(group).await
            }
            MergeRecommendation::DiscardAll => {
                Err(crate::MemoryError::Other("Cannot merge - discard all requested".to_string()))
            }
        }
    }

    /// Calculate similarity between two memories
    async fn calculate_similarity(&self, a: &crate::memory_types::Memory, b: &crate::memory_types::Memory) -> crate::MemoryResult<f32> {
        // Multi-faceted similarity calculation
        let content_similarity = self.calculate_content_similarity(a, b)?;
        let temporal_similarity = self.calculate_temporal_similarity(a, b);
        let contextual_similarity = self.calculate_contextual_similarity(a, b);

        // Weighted combination
        let similarity = 0.6 * content_similarity +
                        0.2 * temporal_similarity +
                        0.2 * contextual_similarity;

        Ok(similarity)
    }

    /// Calculate content-based similarity
    fn calculate_content_similarity(&self, a: &crate::memory_types::Memory, b: &crate::memory_types::Memory) -> crate::MemoryResult<f32> {
        match (&a.content, &b.content) {
            (crate::memory_types::MemoryContent::Text(text_a), crate::memory_types::MemoryContent::Text(text_b)) => {
                Ok(self.text_similarity(text_a, text_b))
            }
            (crate::memory_types::MemoryContent::Structured(data_a), crate::memory_types::MemoryContent::Structured(data_b)) => {
                Ok(self.structured_similarity(data_a, data_b))
            }
            _ => Ok(0.0), // Different content types are not similar
        }
    }

    /// Calculate temporal similarity (closer in time = more similar)
    fn calculate_temporal_similarity(&self, a: &crate::memory_types::Memory, b: &crate::memory_types::Memory) -> f32 {
        let time_diff = (a.created_at - b.created_at).num_seconds().abs() as f32;
        let max_diff = self.config.time_window_hours as f32 * 3600.0;

        if time_diff >= max_diff {
            0.0
        } else {
            1.0 - (time_diff / max_diff)
        }
    }

    /// Calculate contextual similarity
    fn calculate_contextual_similarity(&self, a: &crate::memory_types::Memory, b: &crate::memory_types::Memory) -> f32 {
        // Compare tags, importance, and other metadata
        let mut similarity = 0.0;
        let mut factors = 0;

        // Importance similarity
        if (a.importance - b.importance).abs() < 0.1 {
            similarity += 1.0;
        }
        factors += 1;

        // Tag similarity (Jaccard coefficient)
        if let (Some(tags_a), Some(tags_b)) = (&a.tags, &b.tags) {
            let intersection: std::collections::HashSet<_> = tags_a.intersection(tags_b).collect();
            let union: std::collections::HashSet<_> = tags_a.union(tags_b).collect();
            if !union.is_empty() {
                similarity += intersection.len() as f32 / union.len() as f32;
            }
        }
        factors += 1;

        similarity / factors as f32
    }

    /// Text similarity using Jaccard coefficient on words
    fn text_similarity(&self, text_a: &str, text_b: &str) -> f32 {
        let words_a: std::collections::HashSet<_> = text_a.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        let words_b: std::collections::HashSet<_> = text_b.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Structured data similarity
    fn structured_similarity(&self, data_a: &serde_json::Value, data_b: &serde_json::Value) -> f32 {
        // Simple JSON similarity based on key overlap
        if let (Some(obj_a), Some(obj_b)) = (data_a.as_object(), data_b.as_object()) {
            let keys_a: std::collections::HashSet<_> = obj_a.keys().collect();
            let keys_b: std::collections::HashSet<_> = obj_b.keys().collect();

            let intersection = keys_a.intersection(&keys_b).count();
            let union = keys_a.union(&keys_b).count();

            if union == 0 {
                0.0
            } else {
                intersection as f32 / union as f32
            }
        } else {
            0.0
        }
    }

    /// Generate merge recommendation
    fn generate_merge_recommendation(
        &self,
        canonical: &crate::memory_types::Memory,
        duplicates: &[crate::memory_types::MemoryId],
        similarity: f32,
    ) -> MergeRecommendation {
        match self.config.merge_strategy {
            MergeStrategy::KeepNewest => MergeRecommendation::KeepCanonical,
            MergeStrategy::KeepOldest => MergeRecommendation::KeepCanonical,
            MergeStrategy::MergeContent => {
                if similarity > 0.8 && duplicates.len() < 3 {
                    MergeRecommendation::MergeIntoCanonical
                } else {
                    MergeRecommendation::CreateNewMerged
                }
            }
            MergeStrategy::CreateSummary => MergeRecommendation::CreateNewMerged,
        }
    }

    /// Merge duplicate content into canonical memory
    async fn merge_into_canonical(&self, _group: &DuplicateGroup) -> crate::MemoryResult<crate::memory_types::Memory> {
        // Implementation would merge content from duplicates into canonical memory
        Err(crate::MemoryError::Other("Merge implementation pending".to_string()))
    }

    /// Create new merged memory from duplicates
    async fn create_merged_memory(&self, _group: &DuplicateGroup) -> crate::MemoryResult<crate::memory_types::Memory> {
        // Implementation would create new memory combining all duplicates
        Err(crate::MemoryError::Other("Create merged implementation pending".to_string()))
    }
}

/// Deduplication statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationStats {
    pub memories_processed: usize,
    pub duplicates_found: usize,
    pub duplicate_groups: usize,
    pub space_saved_bytes: u64,
    pub processing_time_ms: u64,
    pub last_deduplication: chrono::DateTime<chrono::Utc>,
}

/// Deduplication engine trait
#[async_trait::async_trait]
pub trait DeduplicationEngine: Send + Sync {
    /// Run deduplication on memory store
    async fn deduplicate(&self, config: &DeduplicationConfig) -> crate::MemoryResult<DeduplicationStats>;

    /// Preview deduplication without applying changes
    async fn preview_deduplication(&self, config: &DeduplicationConfig) -> crate::MemoryResult<Vec<DuplicateGroup>>;

    /// Get deduplication statistics
    async fn get_stats(&self) -> crate::MemoryResult<DeduplicationStats>;
}
