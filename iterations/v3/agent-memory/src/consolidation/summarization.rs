//! Memory Summarization
//!
//! Automatic summarization of memory clusters and temporal sequences.

use crate::consolidation::*;

/// Summarization configuration
#[derive(Debug, Clone)]
pub struct SummarizationConfig {
    pub max_summary_length: usize,
    pub summary_model: String,
    pub compression_ratio: f32,
    pub temporal_grouping_hours: u64,
}

/// Memory summarizer
pub struct MemorySummarizer {
    config: SummarizationConfig,
}

impl MemorySummarizer {
    pub fn new(config: SummarizationConfig) -> Self {
        Self { config }
    }

    /// Generate summary for a memory cluster
    pub async fn summarize_cluster(&self, cluster: &MemoryCluster) -> crate::MemoryResult<String> {
        // Extract memory contents for summarization
        let memory_contents = self.extract_cluster_contents(cluster).await?;

        // Generate summary using configured method
        let summary = self.generate_summary(&memory_contents).await?;

        Ok(summary)
    }

    /// Generate temporal summary for a sequence of memories
    pub async fn summarize_temporal_sequence(
        &self,
        memories: Vec<crate::memory_types::Memory>,
    ) -> crate::MemoryResult<String> {
        // Group memories by time windows
        let time_windows = self.group_by_time_windows(memories);

        // Generate summary for each time window
        let mut summaries = Vec::new();
        for (window_start, window_memories) in time_windows {
            let window_contents = self.extract_memory_contents(&window_memories).await?;
            let window_summary = self.generate_summary(&window_contents).await?;
            summaries.push(format!("{}: {}", window_start.format("%Y-%m-%d %H:%M"), window_summary));
        }

        // Combine window summaries
        let combined_summary = format!("Temporal Summary:\n{}", summaries.join("\n"));
        self.compress_summary(&combined_summary)
    }

    /// Generate progressive summary (summarize summaries)
    pub async fn summarize_progressive(
        &self,
        summaries: Vec<String>,
    ) -> crate::MemoryResult<String> {
        if summaries.is_empty() {
            return Ok(String::new());
        }

        if summaries.len() == 1 {
            return Ok(summaries[0].clone());
        }

        // Combine all summaries and generate higher-level summary
        let combined_text = summaries.join("\n\n");
        self.generate_summary(&combined_text).await
    }

    /// Extract contents from cluster memories
    async fn extract_cluster_contents(&self, cluster: &MemoryCluster) -> crate::MemoryResult<String> {
        // This would typically fetch actual memory content from the database
        // For now, return a placeholder representation
        let content = format!(
            "Cluster with {} memories, importance score: {:.3}",
            cluster.member_memories.len(),
            cluster.importance_score
        );

        Ok(content)
    }

    /// Extract contents from memory objects
    async fn extract_memory_contents(&self, memories: &[crate::memory_types::Memory]) -> crate::MemoryResult<String> {
        let mut contents = Vec::new();

        for memory in memories {
            // Extract meaningful content from memory (simplified)
            let content = match &memory.content {
                crate::memory_types::MemoryContent::Text(text) => text.clone(),
                crate::memory_types::MemoryContent::Structured(data) => {
                    serde_json::to_string_pretty(data).unwrap_or_default()
                }
                _ => "Complex memory content".to_string(),
            };
            contents.push(content);
        }

        Ok(contents.join("\n\n"))
    }

    /// Generate summary from text content
    async fn generate_summary(&self, content: &str) -> crate::MemoryResult<String> {
        // Simplified summarization - in practice, this would use an LLM or ML model
        if content.len() <= self.config.max_summary_length {
            return Ok(content.to_string());
        }

        // Basic extractive summarization: take first and last parts
        let words: Vec<&str> = content.split_whitespace().collect();
        let target_words = (self.config.max_summary_length / 8).min(words.len() / 2); // Rough word count estimate

        let first_part = words.iter().take(target_words).cloned().collect::<Vec<_>>().join(" ");
        let last_part = words.iter().rev().take(target_words).cloned().collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" ");

        let summary = format!("{} ... {}", first_part, last_part);
        self.compress_summary(&summary)
    }

    /// Compress summary to fit length constraints
    fn compress_summary(&self, summary: &str) -> crate::MemoryResult<String> {
        if summary.len() <= self.config.max_summary_length {
            return Ok(summary.to_string());
        }

        // Simple truncation with ellipsis
        let mut compressed = summary.chars().take(self.config.max_summary_length - 3).collect::<String>();
        compressed.push_str("...");

        Ok(compressed)
    }

    /// Group memories by time windows
    fn group_by_time_windows(&self, memories: Vec<crate::memory_types::Memory>) -> std::collections::HashMap<chrono::DateTime<chrono::Utc>, Vec<crate::memory_types::Memory>> {
        let mut groups = std::collections::HashMap::new();
        let window_duration = chrono::Duration::hours(self.config.temporal_grouping_hours as i64);

        for memory in memories {
            let window_start = memory.created_at.timestamp() / (self.config.temporal_grouping_hours * 3600) * (self.config.temporal_grouping_hours * 3600);
            let window_time = chrono::DateTime::from_timestamp(window_start as i64, 0)
                .unwrap_or(chrono::Utc::now());

            groups.entry(window_time).or_insert_with(Vec::new).push(memory);
        }

        groups
    }
}

/// Abstractive summarization using templates
pub struct TemplateBasedSummarizer {
    templates: Vec<String>,
}

impl TemplateBasedSummarizer {
    pub fn new() -> Self {
        Self {
            templates: vec![
                "Key points: {}".to_string(),
                "Summary: {}".to_string(),
                "Essential information: {}".to_string(),
                "Core concepts: {}".to_string(),
            ],
        }
    }

    /// Generate template-based summary
    pub async fn summarize_with_template(&self, content: &str, template_idx: usize) -> crate::MemoryResult<String> {
        let template = self.templates.get(template_idx)
            .ok_or_else(|| crate::MemoryError::Other("Template not found".to_string()))?;

        // Extract key information (simplified keyword extraction)
        let keywords = self.extract_keywords(content);
        let key_info = keywords.join(", ");

        let summary = template.replace("{}", &key_info);
        Ok(summary)
    }

    /// Extract keywords from content
    fn extract_keywords(&self, content: &str) -> Vec<String> {
        // Simple keyword extraction: words longer than 4 characters, appearing multiple times
        let mut word_counts = std::collections::HashMap::new();

        for word in content.split_whitespace() {
            let clean_word = word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();

            if clean_word.len() > 4 {
                *word_counts.entry(clean_word).or_insert(0) += 1;
            }
        }

        // Return words that appear more than once, sorted by frequency
        let mut keywords: Vec<_> = word_counts.into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(word, _)| word)
            .collect();

        keywords.sort();
        keywords
    }
}

/// Summarization quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizationMetrics {
    pub compression_ratio: f32,
    pub information_retention: f32,
    pub readability_score: f32,
    pub coherence_score: f32,
}

/// Quality evaluator for summaries
pub struct SummaryQualityEvaluator;

impl SummaryQualityEvaluator {
    /// Evaluate summary quality
    pub async fn evaluate_quality(&self, original: &str, summary: &str) -> crate::MemoryResult<SummarizationMetrics> {
        let compression_ratio = summary.len() as f32 / original.len() as f32;

        // Simplified quality metrics
        let information_retention = self.calculate_information_retention(original, summary);
        let readability_score = self.calculate_readability(summary);
        let coherence_score = self.calculate_coherence(summary);

        Ok(SummarizationMetrics {
            compression_ratio,
            information_retention,
            readability_score,
            coherence_score,
        })
    }

    fn calculate_information_retention(&self, _original: &str, _summary: &str) -> f32 {
        // Simplified: assume 70% retention for non-trivial summaries
        0.7
    }

    fn calculate_readability(&self, summary: &str) -> f32 {
        // Simple readability based on average sentence length
        let sentences: Vec<&str> = summary.split(|c: char| c == '.' || c == '!' || c == '?').collect();
        let avg_sentence_length = summary.len() as f32 / sentences.len() as f32;

        // Optimal sentence length around 15-20 words
        let optimal_length = 75.0; // Rough character count estimate
        1.0 - (avg_sentence_length - optimal_length).abs() / optimal_length
    }

    fn calculate_coherence(&self, summary: &str) -> f32 {
        // Simplified coherence based on presence of transition words
        let transition_words = ["however", "therefore", "thus", "consequently", "furthermore"];
        let word_count = summary.split_whitespace().count();

        if word_count == 0 {
            return 0.0;
        }

        let transition_count = transition_words.iter()
            .filter(|word| summary.to_lowercase().contains(*word))
            .count();

        transition_count as f32 / word_count as f32
    }
}
