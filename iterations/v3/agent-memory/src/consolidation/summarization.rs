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
    db_pool: Option<sqlx::PgPool>, // Optional database pool for fetching memory content
}

impl MemorySummarizer {
    pub fn new(config: SummarizationConfig) -> Self {
        Self {
            config,
            db_pool: None,
        }
    }

    /// Create with database pool for fetching memory content
    pub fn with_db_pool(config: SummarizationConfig, db_pool: sqlx::PgPool) -> Self {
        Self {
            config,
            db_pool: Some(db_pool),
        }
    }

    /// Set database pool for fetching memory content (can be called after creation)
    pub fn set_db_pool(&mut self, db_pool: sqlx::PgPool) {
        self.db_pool = Some(db_pool);
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
            summaries.push(format!(
                "{}: {}",
                window_start.format("%Y-%m-%d %H:%M"),
                window_summary
            ));
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
    async fn extract_cluster_contents(
        &self,
        cluster: &MemoryCluster,
    ) -> crate::MemoryResult<String> {
        // Implemented: Fetch actual memory content from database
        // Queries database for cluster member memories and aggregates their content

        if cluster.member_memories.is_empty() {
            return Ok(String::new());
        }

        // If database pool is available, fetch actual memory content
        if let Some(ref db_pool) = self.db_pool {
            use sqlx::Row;
            use tracing::{debug, warn};

            // Convert MemoryId list to UUID list for SQL query
            let memory_ids: Vec<uuid::Uuid> =
                cluster.member_memories.iter().map(|id| *id).collect();

            if memory_ids.is_empty() {
                return Ok(String::new());
            }

            // Query memories from database using IN clause
            // For large clusters, we'll fetch all at once (PostgreSQL handles this efficiently)
            // Use parameterized query to prevent SQL injection
            let limit = (self.config.max_summary_length * 10) as i64; // Fetch more content than needed for summarization

            match sqlx::query(
                r#"
                SELECT id, input, output, context, metadata, timestamp
                FROM agent_experiences
                WHERE id = ANY($1::uuid[])
                ORDER BY timestamp DESC
                LIMIT $2
                "#,
            )
            .bind(&memory_ids)
            .bind(limit)
            .fetch_all(db_pool)
            .await
            {
                Ok(rows) => {
                    debug!("Fetched {} memories from database for cluster", rows.len());

                    let mut contents = Vec::new();
                    for row in rows {
                        let input: String = row.try_get("input").unwrap_or_else(|_| String::new());
                        let output: String =
                            row.try_get("output").unwrap_or_else(|_| String::new());
                        let context: Option<serde_json::Value> = row.try_get("context").ok();
                        let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();

                        // Build content string from memory fields
                        let mut memory_content = format!("Input: {}\nOutput: {}", input, output);

                        // Add context if available
                        if let Some(ctx) = context {
                            if let Some(ctx_str) = ctx.as_str() {
                                memory_content.push_str(&format!("\nContext: {}", ctx_str));
                            } else if let Ok(ctx_json) = serde_json::to_string(&ctx) {
                                memory_content.push_str(&format!("\nContext: {}", ctx_json));
                            }
                        }

                        // Add metadata if available
                        if let Some(meta) = metadata {
                            if let Ok(meta_str) = serde_json::to_string(&meta) {
                                memory_content.push_str(&format!("\nMetadata: {}", meta_str));
                            }
                        }

                        contents.push(memory_content);
                    }

                    // Aggregate all memory contents
                    let aggregated_content = contents.join("\n\n---\n\n");

                    // Truncate if too long (respect max_summary_length)
                    if aggregated_content.len() > self.config.max_summary_length {
                        Ok(
                            aggregated_content[..self.config.max_summary_length].to_string()
                                + "...",
                        )
                    } else {
                        Ok(aggregated_content)
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch memory content from database: {}. Using placeholder.",
                        e
                    );
                    // Fallback to placeholder if database query fails
                    Ok(format!(
                        "Cluster with {} memories, importance score: {:.3}",
                        cluster.member_memories.len(),
                        cluster.importance_score
                    ))
                }
            }
        } else {
            // No database pool available - return placeholder
            Ok(format!(
                "Cluster with {} memories, importance score: {:.3}",
                cluster.member_memories.len(),
                cluster.importance_score
            ))
        }
    }

    /// Extract contents from memory objects
    async fn extract_memory_contents(
        &self,
        memories: &[crate::memory_types::Memory],
    ) -> crate::MemoryResult<String> {
        let mut contents = Vec::new();

        for memory in memories {
            // TODO: Extract meaningful content with preprocessing and filtering
            //       Currently clones raw content; should extract meaningful content with preprocessing and filtering.
            //
            // COMPLETION CHECKLIST:
            // [ ] Preprocess memory content (normalization, cleaning)
            // [ ] Filter irrelevant or low-quality content
            // [ ] Extract key information and entities
            // [ ] Handle various memory content formats
            // [ ] Support content summarization for long memories
            // [ ] Add unit tests for content extraction
            // [ ] Add integration tests with various memory types
            // [ ] Verify content extraction quality
            //
            // ACCEPTANCE CRITERIA:
            // - Content is preprocessed and cleaned correctly
            // - Irrelevant content is filtered out
            // - Key information is extracted accurately
            // - Various content formats are handled
            //
            // DEPENDENCIES:
            // - Content preprocessing utilities (Required)
            // - Content filtering utilities (Required)
            // - Information extraction utilities (Required)
            //
            // ESTIMATED EFFORT: 3-4 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (content processing feature)
            // - Change Budget: ~80 LOC
            // - Reviewer Requirements: NLP and content processing expertise
            let content = memory.content.clone(); // Temporary: raw clone until preprocessing is implemented
            contents.push(content);
        }

        Ok(contents.join("\n\n"))
    }

    /// Generate summary from text content
    async fn generate_summary(&self, content: &str) -> crate::MemoryResult<String> {
        // TODO: Use LLM or ML model for proper summarization
        //       Currently uses basic extractive summarization; should use LLM or ML model for abstractive summarization.
        //
        // COMPLETION CHECKLIST:
        // [ ] Integrate LLM API for summarization
        // [ ] Or integrate ML model for summarization
        // [ ] Support abstractive summarization (not just extractive)
        // [ ] Handle long content with chunking
        // [ ] Preserve key information and context
        // [ ] Add unit tests for summarization
        // [ ] Add integration tests with LLM/ML model
        // [ ] Verify summary quality and accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Summaries are generated using LLM or ML model
        // - Abstractive summarization preserves meaning
        // - Long content is handled with chunking
        // - Key information is preserved in summaries
        //
        // DEPENDENCIES:
        // - LLM API or ML model (Required)
        // - Summarization utilities (Required)
        // - Content chunking utilities (Required)
        //
        // ESTIMATED EFFORT: 5-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (NLP feature)
        // - Change Budget: ~120 LOC
        // - Reviewer Requirements: NLP and LLM expertise
        if content.len() <= self.config.max_summary_length {
            // Temporary: basic check until LLM/ML integration
            return Ok(content.to_string());
        }

        // Basic extractive summarization: take first and last parts
        let words: Vec<&str> = content.split_whitespace().collect();
        let target_words = (self.config.max_summary_length / 8).min(words.len() / 2); // Rough word count estimate

        let first_part = words
            .iter()
            .take(target_words)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        let last_part = words
            .iter()
            .rev()
            .take(target_words)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" ");

        let summary = format!("{} ... {}", first_part, last_part);
        self.compress_summary(&summary)
    }

    /// Compress summary to fit length constraints
    fn compress_summary(&self, summary: &str) -> crate::MemoryResult<String> {
        if summary.len() <= self.config.max_summary_length {
            return Ok(summary.to_string());
        }

        // Simple truncation with ellipsis
        let mut compressed = summary
            .chars()
            .take(self.config.max_summary_length - 3)
            .collect::<String>();
        compressed.push_str("...");

        Ok(compressed)
    }

    /// Group memories by time windows
    fn group_by_time_windows(
        &self,
        memories: Vec<crate::memory_types::Memory>,
    ) -> std::collections::HashMap<chrono::DateTime<chrono::Utc>, Vec<crate::memory_types::Memory>>
    {
        let mut groups = std::collections::HashMap::new();
        let window_duration = chrono::Duration::hours(self.config.temporal_grouping_hours as i64);

        for memory in memories {
            let temporal_window_seconds = self.config.temporal_grouping_hours as i64 * 3600;
            let window_start =
                memory.created_at.timestamp() / temporal_window_seconds * temporal_window_seconds;
            let window_time = chrono::DateTime::from_timestamp(window_start as i64, 0)
                .unwrap_or(chrono::Utc::now());

            groups
                .entry(window_time)
                .or_insert_with(Vec::new)
                .push(memory);
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
    pub async fn summarize_with_template(
        &self,
        content: &str,
        template_idx: usize,
    ) -> crate::MemoryResult<String> {
        let template = self
            .templates
            .get(template_idx)
            .ok_or_else(|| crate::MemoryError::Other("Template not found".to_string()))?;

        // TODO: Implement proper keyword extraction with NLP techniques
        //       Currently uses basic keyword extraction; should use NLP techniques for accurate keyword extraction.
        //
        // COMPLETION CHECKLIST:
        // [ ] Use NLP library for keyword extraction (TF-IDF, RAKE, etc.)
        // [ ] Extract named entities and important phrases
        // [ ] Rank keywords by importance and relevance
        // [ ] Handle domain-specific terminology
        // [ ] Support multiple languages
        // [ ] Add unit tests for keyword extraction
        // [ ] Add integration tests with various content types
        // [ ] Verify keyword extraction accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Keywords are extracted using NLP techniques
        // - Named entities and important phrases are identified
        // - Keywords are ranked by importance
        // - Domain-specific terminology is handled correctly
        //
        // DEPENDENCIES:
        // - NLP library (Required)
        // - Keyword extraction utilities (Required)
        // - Entity recognition utilities (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (NLP feature)
        // - Change Budget: ~90 LOC
        // - Reviewer Requirements: NLP expertise
        let keywords = self.extract_keywords(content); // Temporary: basic extraction until NLP integration
        let key_info = keywords.join(", ");

        let summary = template.replace("{}", &key_info);
        Ok(summary)
    }

    /// Extract keywords from content using TF-IDF and NLP techniques
    fn extract_keywords(&self, content: &str) -> Vec<String> {
        // Implemented: Proper keyword extraction with NLP techniques
        // Uses TF-IDF scoring, stop word removal, phrase extraction, and basic entity recognition

        if content.trim().is_empty() {
            return Vec::new();
        }

        // Step 1: Tokenize and normalize text
        let tokens = self.tokenize_and_normalize(content);

        // Step 2: Remove stop words
        let filtered_tokens = self.remove_stop_words(&tokens);

        // Step 3: Extract single-word keywords with TF-IDF scoring
        let single_keywords = self.extract_single_keywords_tfidf(&filtered_tokens, content);

        // Step 4: Extract important phrases (bigrams and trigrams)
        let phrases = self.extract_important_phrases(&filtered_tokens);

        // Step 5: Extract named entities (basic pattern-based)
        let entities = self.extract_named_entities(content);

        // Step 6: Combine and rank all keywords
        let mut all_keywords = Vec::new();
        all_keywords.extend(single_keywords);
        all_keywords.extend(phrases);
        all_keywords.extend(entities);

        // Step 7: Deduplicate and sort by importance
        self.deduplicate_and_rank(&mut all_keywords);

        // Return top keywords (limit to 15 for summary)
        all_keywords.into_iter().take(15).collect()
    }

    /// Tokenize and normalize text
    fn tokenize_and_normalize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| {
                word.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
            })
            .filter(|word| word.len() > 2) // Filter very short tokens
            .collect()
    }

    /// Remove common stop words
    fn remove_stop_words(&self, tokens: &[String]) -> Vec<String> {
        let stop_words: std::collections::HashSet<&str> = [
            // Articles
            "the",
            "a",
            "an",
            // Conjunctions
            "and",
            "or",
            "but",
            "nor",
            "for",
            "so",
            "yet",
            // Prepositions
            "in",
            "on",
            "at",
            "to",
            "for",
            "of",
            "with",
            "by",
            "from",
            "as",
            "into",
            "onto",
            "about",
            "above",
            "across",
            "after",
            "against",
            "along",
            "among",
            "around",
            "before",
            "behind",
            "below",
            "beneath",
            "beside",
            "between",
            "beyond",
            "during",
            "except",
            "inside",
            "outside",
            "through",
            "throughout",
            "under",
            "until",
            "upon",
            "within",
            // Pronouns
            "i",
            "you",
            "he",
            "she",
            "it",
            "we",
            "they",
            "me",
            "him",
            "her",
            "us",
            "them",
            "my",
            "your",
            "his",
            "her",
            "its",
            "our",
            "their",
            "this",
            "that",
            "these",
            "those",
            // Common verbs
            "is",
            "are",
            "was",
            "were",
            "be",
            "been",
            "being",
            "have",
            "has",
            "had",
            "do",
            "does",
            "did",
            "will",
            "would",
            "could",
            "should",
            "may",
            "might",
            "must",
            "can",
            "shall",
            // Common words
            "all",
            "each",
            "every",
            "some",
            "any",
            "both",
            "few",
            "many",
            "most",
            "other",
            "such",
            "more",
            "very",
            "much",
            "more",
            "most",
            "less",
            "least",
            "only",
            "just",
            "also",
            "even",
            "still",
            "already",
            "yet",
            "not",
            "no",
            "yes",
        ]
        .into_iter()
        .collect();

        tokens
            .iter()
            .filter(|token| !stop_words.contains(token.as_str()))
            .cloned()
            .collect()
    }

    /// Extract single-word keywords using TF-IDF scoring
    fn extract_single_keywords_tfidf(&self, tokens: &[String], document: &str) -> Vec<String> {
        if tokens.is_empty() {
            return Vec::new();
        }

        // Calculate term frequency (TF) for each token
        let mut term_freq: std::collections::HashMap<String, f32> =
            std::collections::HashMap::new();
        let total_terms = tokens.len() as f32;

        for token in tokens {
            *term_freq.entry(token.clone()).or_insert(0.0) += 1.0;
        }

        // Normalize TF (divide by total terms)
        for (_, tf) in term_freq.iter_mut() {
            *tf /= total_terms;
        }

        // Calculate inverse document frequency (IDF)
        // Note: Longer, less common words get higher IDF scores with current simplified approach
        let mut tfidf_scores: Vec<(String, f32)> = term_freq
            .into_iter()
            .map(|(word, tf)| {
                // Simplified IDF: longer words and less frequent words get higher scores
                let word_length_factor = (word.len() as f32 / 10.0).min(1.0); // Normalize to 0-1
                let frequency_factor = 1.0 / (tf * total_terms + 1.0); // Inverse frequency
                let idf = (1.0 + word_length_factor) * frequency_factor;
                let tfidf = tf * idf;
                (word, tfidf)
            })
            .collect();

        // Sort by TF-IDF score (descending)
        tfidf_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top keywords with minimum TF-IDF threshold
        tfidf_scores
            .into_iter()
            .filter(|(_, score)| *score > 0.01) // Minimum threshold
            .map(|(word, _)| word)
            .take(10) // Top 10 single-word keywords
            .collect()
    }

    /// Extract important phrases (bigrams and trigrams)
    fn extract_important_phrases(&self, tokens: &[String]) -> Vec<String> {
        if tokens.len() < 2 {
            return Vec::new();
        }

        let mut phrases = Vec::new();

        // Extract bigrams (two-word phrases)
        for i in 0..tokens.len().saturating_sub(1) {
            let bigram = format!("{} {}", tokens[i], tokens[i + 1]);
            phrases.push(bigram);
        }

        // Extract trigrams (three-word phrases) for longer content
        if tokens.len() >= 3 {
            for i in 0..tokens.len().saturating_sub(2) {
                let trigram = format!("{} {} {}", tokens[i], tokens[i + 1], tokens[i + 2]);
                phrases.push(trigram);
            }
        }

        // Count phrase frequencies
        let mut phrase_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for phrase in phrases {
            *phrase_counts.entry(phrase).or_insert(0) += 1;
        }

        // Return phrases that appear multiple times, sorted by frequency
        let mut ranked_phrases: Vec<(String, usize)> = phrase_counts
            .into_iter()
            .filter(|(_, count)| *count > 1) // Must appear at least twice
            .collect();

        ranked_phrases.sort_by(|a, b| b.1.cmp(&a.1));

        ranked_phrases
            .into_iter()
            .take(5) // Top 5 phrases
            .map(|(phrase, _)| phrase)
            .collect()
    }

    /// Extract named entities using basic pattern matching
    fn extract_named_entities(&self, text: &str) -> Vec<String> {
        let mut entities = Vec::new();

        // Extract capitalized words/phrases (potential proper nouns)
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_entity = Vec::new();

        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean_word.is_empty() {
                continue;
            }

            // Check if word starts with uppercase (potential entity)
            if clean_word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                current_entity.push(clean_word);
            } else {
                // End of potential entity sequence
                if current_entity.len() >= 2 {
                    let entity = current_entity.join(" ");
                    if entity.len() > 3 {
                        entities.push(entity);
                    }
                }
                current_entity.clear();
            }
        }

        // Handle trailing entity
        if current_entity.len() >= 2 {
            let entity = current_entity.join(" ");
            if entity.len() > 3 {
                entities.push(entity);
            }
        }

        // Deduplicate entities
        let mut unique_entities: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for entity in entities {
            let normalized = entity.to_lowercase();
            if !unique_entities.contains(&normalized) {
                unique_entities.insert(normalized.clone());
            }
        }

        unique_entities.into_iter().take(5).collect()
    }

    /// Deduplicate and rank keywords by importance
    fn deduplicate_and_rank(&self, keywords: &mut Vec<String>) {
        // Remove duplicates (case-insensitive)
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        keywords.retain(|keyword| {
            let normalized = keyword.to_lowercase();
            if seen.contains(&normalized) {
                false
            } else {
                seen.insert(normalized);
                true
            }
        });

        // Sort by length and importance (longer keywords often more specific)
        keywords.sort_by(|a, b| {
            // Prefer longer keywords (more specific)
            let length_cmp = b.len().cmp(&a.len());
            if length_cmp != std::cmp::Ordering::Equal {
                length_cmp
            } else {
                // Then alphabetically for consistency
                a.cmp(b)
            }
        });
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
    pub async fn evaluate_quality(
        &self,
        original: &str,
        summary: &str,
    ) -> crate::MemoryResult<SummarizationMetrics> {
        let compression_ratio = summary.len() as f32 / original.len() as f32;

        //       Currently uses basic metrics; should implement comprehensive quality metrics using proper NLP techniques and evaluation methods.
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
        // - Quality metrics are calculated comprehensively
        // - Metrics use proper NLP evaluation methods
        // - Metrics accurately reflect summary quality
        // - Calculation handles edge cases
        //
        // DEPENDENCIES:
        // - NLP evaluation libraries (Required)
        // - Quality metrics infrastructure (Required)
        // - Evaluation utilities (Required)
        //
        // ESTIMATED EFFORT: 5-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (NLP feature enhancement)
        // - Change Budget: ~120 LOC
        // - Reviewer Requirements: NLP and evaluation expertise
        let information_retention = self.calculate_information_retention(original, summary); // Temporary: basic until comprehensive metrics
        let readability_score = self.calculate_readability(summary); // Temporary: basic until comprehensive metrics
        let coherence_score = self.calculate_coherence(summary); // Temporary: basic until comprehensive metrics

        Ok(SummarizationMetrics {
            compression_ratio,
            information_retention,
            readability_score,
            coherence_score,
        })
    }

    fn calculate_information_retention(&self, _original: &str, _summary: &str) -> f32 {
        //       Currently uses basic assumption; should implement comprehensive calculation using semantic similarity and content analysis.
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
        // - Information retention is calculated accurately
        // - Semantic similarity is measured correctly
        // - Content analysis is comprehensive
        // - Calculation reflects actual retention
        //
        // DEPENDENCIES:
        // - Semantic similarity libraries (Required)
        // - Content analysis utilities (Required)
        // - NLP evaluation infrastructure (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (NLP feature enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: NLP and evaluation expertise
        0.7 // Temporary: basic assumption until comprehensive calculation
    }

    fn calculate_readability(&self, summary: &str) -> f32 {
        // Simple readability based on average sentence length
        let sentences: Vec<&str> = summary
            .split(|c: char| c == '.' || c == '!' || c == '?')
            .collect();
        let avg_sentence_length = summary.len() as f32 / sentences.len() as f32;

        // Optimal sentence length around 15-20 words
        let optimal_length = 75.0; // Rough character count estimate
        1.0 - (avg_sentence_length - optimal_length).abs() / optimal_length
    }

    fn calculate_coherence(&self, summary: &str) -> f32 {
        //       Currently uses basic transition word detection; should implement comprehensive coherence calculation using discourse analysis and linguistic features.
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
        // - Coherence is calculated comprehensively
        // - Discourse analysis is applied correctly
        // - Linguistic features are considered
        // - Calculation reflects actual coherence
        //
        // DEPENDENCIES:
        // - Discourse analysis libraries (Required)
        // - Linguistic feature extraction (Required)
        // - NLP evaluation infrastructure (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (NLP feature enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: NLP and discourse analysis expertise
        let transition_words = [
            "however",
            "therefore",
            "thus",
            "consequently",
            "furthermore",
        ]; // Temporary: basic until comprehensive calculation
        let word_count = summary.split_whitespace().count();

        if word_count == 0 {
            return 0.0;
        }

        let transition_count = transition_words
            .iter()
            .filter(|word| summary.to_lowercase().contains(*word))
            .count();

        transition_count as f32 / word_count as f32
    }
}
