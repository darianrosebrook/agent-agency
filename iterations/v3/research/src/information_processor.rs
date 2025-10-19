//! Information Processor for Knowledge Results
//!
//! Processes search results through filtering, ranking, and deduplication.
//! Implements sophisticated quality assessment and diversity constraints.
//!
//! Ported from V2 InformationProcessor.ts with Rust optimizations.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Information processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationProcessorConfig {
    pub quality: QualityConfig,
    pub min_relevance_score: f64,
    pub min_credibility_score: f64,
    pub max_results_to_process: usize,
    pub enable_duplicate_detection: bool,
    pub enable_relevance_filtering: bool,
    pub enable_credibility_filtering: bool,
    pub enable_diversity_constraints: bool,
}

/// Quality assessment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    pub enable_duplicate_detection: bool,
    pub enable_relevance_filtering: bool,
    pub enable_credibility_filtering: bool,
    pub enable_diversity_constraints: bool,
    pub similarity_threshold: f64,
    pub diversity_threshold: f64,
}

impl Default for InformationProcessorConfig {
    fn default() -> Self {
        Self {
            quality: QualityConfig {
                enable_duplicate_detection: true,
                enable_relevance_filtering: true,
                enable_credibility_filtering: true,
                enable_diversity_constraints: true,
                similarity_threshold: 0.8,
                diversity_threshold: 0.3,
            },
            min_relevance_score: 0.3,
            min_credibility_score: 0.2,
            max_results_to_process: 50,
            enable_duplicate_detection: true,
            enable_relevance_filtering: true,
            enable_credibility_filtering: true,
            enable_diversity_constraints: true,
        }
    }
}

/// Enhanced search result with quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedSearchResult {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub source: String,
    pub relevance_score: f64,
    pub credibility_score: f64,
    pub quality: QualityLevel,
    pub combined_score: f64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub processing_metadata: ProcessingMetadata,
}

/// Quality level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityLevel {
    High,
    Medium,
    Low,
    Poor,
}

/// Processing metadata for result tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub original_relevance: f64,
    pub original_credibility: f64,
    pub duplicate_count: u32,
    pub diversity_score: f64,
    pub processing_time_ms: u64,
    pub filters_applied: Vec<String>,
}

/// Information processor trait
#[async_trait]
pub trait IInformationProcessor: Send + Sync {
    /// Process search results through filtering, ranking, and deduplication
    async fn process_results(
        &self,
        query: &KnowledgeQuery,
        results: Vec<SearchResult>,
    ) -> Result<Vec<ProcessedSearchResult>>;

    /// Score relevance of a result for a given query
    fn score_relevance(&self, query: &KnowledgeQuery, result: &SearchResult) -> f64;

    /// Assess credibility of a result
    fn assess_credibility(&self, result: &SearchResult) -> f64;

    /// Determine quality level based on scores
    fn determine_quality(&self, relevance: f64, credibility: f64) -> QualityLevel;

    /// Calculate combined score for ranking
    fn calculate_combined_score(&self, result: &ProcessedSearchResult) -> f64;
}

/// Information processor implementation
#[derive(Debug)]
pub struct InformationProcessor {
    config: InformationProcessorConfig,
    processing_stats: Arc<RwLock<ProcessingStats>>,
}

/// Processing statistics
#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_processed: u64,
    pub duplicates_removed: u64,
    pub relevance_filtered: u64,
    pub credibility_filtered: u64,
    pub diversity_constrained: u64,
    pub average_processing_time_ms: f64,
}

impl InformationProcessor {
    /// Create a new information processor
    pub fn new(config: InformationProcessorConfig) -> Self {
        Self {
            config,
            processing_stats: Arc::new(RwLock::new(ProcessingStats::default())),
        }
    }

    /// Detect and remove duplicate results
    fn detect_duplicates(&self, results: &[SearchResult]) -> Vec<SearchResult> {
        if !self.config.enable_duplicate_detection {
            return results.to_vec();
        }

        let mut seen_content = HashSet::new();
        let mut seen_urls = HashSet::new();
        let mut unique_results = Vec::new();

        for result in results {
            let content_hash = self.content_hash(&result.content);
            let url = result.url.as_ref().map(|u| u.as_str()).unwrap_or("");

            if !seen_content.contains(&content_hash) && !seen_urls.contains(url) {
                seen_content.insert(content_hash);
                if !url.is_empty() {
                    seen_urls.insert(url);
                }
                unique_results.push(result.clone());
            }
        }

        info!(
            "Removed {} duplicate results from {} total results",
            results.len() - unique_results.len(),
            results.len()
        );

        unique_results
    }

    /// Generate content hash for duplicate detection
    fn content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Apply diversity constraints to results
    fn apply_diversity_constraints(
        &self,
        results: Vec<ProcessedSearchResult>,
    ) -> Vec<ProcessedSearchResult> {
        if !self.config.enable_diversity_constraints {
            return results;
        }

        let mut diverse_results = Vec::new();
        let mut source_counts = HashMap::new();
        let max_per_source = (results.len() / 3).max(1); // Max 1/3 from any single source

        let results_len = results.len();
        for result in results {
            let source_count = source_counts.get(&result.source).unwrap_or(&0);
            if *source_count < max_per_source {
                diverse_results.push(result.clone());
                source_counts.insert(result.source.clone(), source_count + 1);
            }
        }

        info!(
            "Applied diversity constraints: {} -> {} results",
            results_len,
            diverse_results.len()
        );

        diverse_results
    }

    /// Calculate diversity score for a result
    fn calculate_diversity_score(
        &self,
        result: &SearchResult,
        other_results: &[SearchResult],
    ) -> f64 {
        if other_results.is_empty() {
            return 1.0; // First result is maximally diverse
        }

        let mut similarity_sum = 0.0;
        let mut count = 0;

        for other in other_results {
            if other.id != result.id {
                let similarity = self.calculate_content_similarity(&result.content, &other.content);
                similarity_sum += similarity;
                count += 1;
            }
        }

        if count == 0 {
            1.0
        } else {
            1.0 - (similarity_sum / count as f64)
        }
    }

    /// Calculate content similarity using simple word overlap
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f64 {
        let content1_lower = content1.to_lowercase();
        let content2_lower = content2.to_lowercase();

        let words1: HashSet<&str> = content1_lower.split_whitespace().collect();
        let words2: HashSet<&str> = content2_lower.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Update processing statistics
    async fn update_stats(&self, processing_time_ms: u64, filters_applied: &[String]) {
        let mut stats = self.processing_stats.write().await;
        stats.total_processed += 1;
        stats.average_processing_time_ms = (stats.average_processing_time_ms
            * (stats.total_processed - 1) as f64
            + processing_time_ms as f64)
            / stats.total_processed as f64;

        for filter in filters_applied {
            match filter.as_str() {
                "duplicate_detection" => stats.duplicates_removed += 1,
                "relevance_filtering" => stats.relevance_filtered += 1,
                "credibility_filtering" => stats.credibility_filtered += 1,
                "diversity_constraints" => stats.diversity_constrained += 1,
                _ => {}
            }
        }
    }
}

#[async_trait]
impl IInformationProcessor for InformationProcessor {
    async fn process_results(
        &self,
        query: &KnowledgeQuery,
        results: Vec<SearchResult>,
    ) -> Result<Vec<ProcessedSearchResult>> {
        let start_time = std::time::Instant::now();
        let mut filters_applied = Vec::new();

        let mut processed_results = results;

        // Step 1: Remove duplicates
        if self.config.enable_duplicate_detection {
            processed_results = self.detect_duplicates(&processed_results);
            filters_applied.push("duplicate_detection".to_string());
        }

        // Step 2: Filter by relevance threshold
        if self.config.enable_relevance_filtering {
            let initial_count = processed_results.len();
            processed_results.retain(|result| {
                let relevance = self.score_relevance(query, result);
                relevance >= self.config.min_relevance_score
            });
            if processed_results.len() < initial_count {
                filters_applied.push("relevance_filtering".to_string());
            }
        }

        // Step 3: Filter by credibility
        if self.config.enable_credibility_filtering {
            let initial_count = processed_results.len();
            processed_results.retain(|result| {
                let credibility = self.assess_credibility(result);
                credibility >= self.config.min_credibility_score
            });
            if processed_results.len() < initial_count {
                filters_applied.push("credibility_filtering".to_string());
            }
        }

        // Step 4: Convert to processed results and assess quality
        let mut processed: Vec<ProcessedSearchResult> = processed_results
            .into_iter()
            .map(|result| {
                let relevance_score = self.score_relevance(query, &result);
                let credibility_score = self.assess_credibility(&result);
                let quality = self.determine_quality(relevance_score, credibility_score);
                let diversity_score = self.calculate_diversity_score(&result, &[]);

                ProcessedSearchResult {
                    id: result.id,
                    title: result.title,
                    content: result.content,
                    url: result.url,
                    source: result.source,
                    relevance_score,
                    credibility_score,
                    quality,
                    combined_score: 0.0, // Will be calculated below
                    metadata: result.metadata,
                    processing_metadata: ProcessingMetadata {
                        original_relevance: relevance_score,
                        original_credibility: credibility_score,
                        duplicate_count: 0,
                        diversity_score,
                        processing_time_ms: 0,
                        filters_applied: filters_applied.clone(),
                    },
                }
            })
            .collect();

        // Step 5: Calculate combined scores
        for result in &mut processed {
            result.combined_score = self.calculate_combined_score(result);
        }

        // Step 6: Sort by combined score
        processed.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

        // Step 7: Apply diversity constraints
        if self.config.enable_diversity_constraints {
            processed = self.apply_diversity_constraints(processed);
            filters_applied.push("diversity_constraints".to_string());
        }

        // Step 8: Limit results
        processed.truncate(self.config.max_results_to_process);

        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        self.update_stats(processing_time_ms, &filters_applied)
            .await;

        info!(
            "Processed {} results in {}ms, {} filters applied",
            processed.len(),
            processing_time_ms,
            filters_applied.len()
        );

        Ok(processed)
    }

    fn score_relevance(&self, query: &KnowledgeQuery, result: &SearchResult) -> f64 {
        let query_lower = query.query.to_lowercase();
        let content_lower = result.content.to_lowercase();
        let title_lower = result.title.to_lowercase();

        let mut score: f64 = 0.0;

        // Title relevance (higher weight)
        if title_lower.contains(&query_lower) {
            score += 0.4;
        }

        // Content relevance
        let content_words: HashSet<&str> = content_lower.split_whitespace().collect();
        let query_words: HashSet<&str> = query_lower.split_whitespace().collect();
        let intersection = content_words.intersection(&query_words).count();
        let query_word_count = query_words.len();

        if query_word_count > 0 {
            score += (intersection as f64 / query_word_count as f64) * 0.6;
        }

        // Source credibility bonus
        match result.source.as_str() {
            "academic" | "peer-reviewed" => score += 0.1,
            "official" | "government" => score += 0.05,
            "news" | "media" => score += 0.02,
            _ => {}
        }

        score.min(1.0f32)
    }

    fn assess_credibility(&self, result: &SearchResult) -> f64 {
        let mut credibility: f64 = 0.5; // Base credibility

        // Source-based credibility
        match result.source.as_str() {
            "academic" | "peer-reviewed" => credibility += 0.3,
            "official" | "government" => credibility += 0.25,
            "news" | "media" => credibility += 0.1,
            "blog" | "personal" => credibility -= 0.2,
            "social-media" | "forum" => credibility -= 0.3,
            _ => {}
        }

        // Content quality indicators
        if result.content.len() > 500 {
            credibility += 0.1; // Longer content often more credible
        }

        if result.content.contains("citation") || result.content.contains("reference") {
            credibility += 0.05; // Citations indicate credibility
        }

        // URL-based credibility
        if let Some(url) = &result.url {
            if url.contains(".edu") || url.contains(".gov") {
                credibility += 0.1;
            } else if url.contains(".org") {
                credibility += 0.05;
            }
        }

        credibility.max(0.0f32).min(1.0f32)
    }

    fn determine_quality(&self, relevance: f64, credibility: f64) -> QualityLevel {
        let combined = (relevance + credibility) / 2.0;

        if combined >= 0.8 {
            QualityLevel::High
        } else if combined >= 0.6 {
            QualityLevel::Medium
        } else if combined >= 0.4 {
            QualityLevel::Low
        } else {
            QualityLevel::Poor
        }
    }

    fn calculate_combined_score(&self, result: &ProcessedSearchResult) -> f64 {
        // Weighted combination of relevance and credibility
        let relevance_weight = 0.6;
        let credibility_weight = 0.4;

        (result.relevance_score * relevance_weight + result.credibility_score * credibility_weight)
            * match result.quality {
                QualityLevel::High => 1.2,
                QualityLevel::Medium => 1.0,
                QualityLevel::Low => 0.8,
                QualityLevel::Poor => 0.6,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_query() -> KnowledgeQuery {
        KnowledgeQuery {
            id: Uuid::new_v4(),
            query: "artificial intelligence".to_string(),
            query_type: QueryType::Technical,
            max_results: Some(10),
            context: None,
            filters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_result(id: &str, content: &str, source: &str) -> SearchResult {
        SearchResult {
            id: Uuid::new_v4(),
            title: format!("Test Result {}", id),
            content: content.to_string(),
            url: Some(format!("https://example.com/{}", id)),
            source: source.to_string(),
            relevance_score: 0.5,
            credibility_score: 0.5,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_information_processor_basic_processing() {
        let config = InformationProcessorConfig::default();
        let processor = InformationProcessor::new(config);

        let query = create_test_query();
        let results = vec![
            create_test_result(
                "1",
                "This is about artificial intelligence and machine learning",
                "academic",
            ),
            create_test_result("2", "AI is transforming the world", "news"),
        ];

        let processed = processor.process_results(&query, results).await.unwrap();
        assert_eq!(processed.len(), 2);
        assert!(processed[0].combined_score > 0.0);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let config = InformationProcessorConfig::default();
        let processor = InformationProcessor::new(config);

        let query = create_test_query();
        let results = vec![
            create_test_result("1", "Same content", "source1"),
            create_test_result("2", "Same content", "source2"), // Duplicate
            create_test_result("3", "Different content", "source3"),
        ];

        let processed = processor.process_results(&query, results).await.unwrap();
        assert_eq!(processed.len(), 2); // One duplicate removed
    }

    #[tokio::test]
    async fn test_relevance_scoring() {
        let config = InformationProcessorConfig::default();
        let processor = InformationProcessor::new(config);

        let query = create_test_query();
        let result = create_test_result("1", "This is about artificial intelligence", "academic");

        let relevance = processor.score_relevance(&query, &result);
        assert!(relevance > 0.5); // Should be relevant
    }

    #[tokio::test]
    async fn test_credibility_assessment() {
        let config = InformationProcessorConfig::default();
        let processor = InformationProcessor::new(config);

        let academic_result = create_test_result("1", "Academic content", "academic");
        let blog_result = create_test_result("2", "Blog content", "blog");

        let academic_credibility = processor.assess_credibility(&academic_result);
        let blog_credibility = processor.assess_credibility(&blog_result);

        assert!(academic_credibility > blog_credibility);
    }
}
