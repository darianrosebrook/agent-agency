//! Hierarchical Context Manager
//!
//! Manages multi-level context with working memory, episodic memory, and semantic memory.
//! Integrates with vector search and context compression for efficient long-context handling.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::types::{Task, ModelContext, IterationContext, EvalReport};
use crate::evaluation::EvalContext;

/// Context budget allocation
#[derive(Clone, Debug)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub headroom: f32, // keep 20% slack
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_tokens: 8192, // 8K tokens default
            headroom: 0.2,    // 20% headroom
        }
    }
}

/// Context allocation result
#[derive(Clone, Debug)]
pub struct Allocation {
    pub working: usize,
    pub episodic: usize,
    pub semantic: usize,
    pub citations: bool,
}

/// Context utility scores for allocation
#[derive(Clone, Debug)]
pub struct ContextUtility {
    pub working: f64,
    pub episodic: f64,
    pub semantic: f64,
}

/// Context token estimates
#[derive(Clone, Debug)]
pub struct ContextTokens {
    pub working: usize,
    pub episodic: usize,
    pub semantic: usize,
}

/// Context bundle with multiple memory types
#[derive(Clone, Debug)]
pub struct ContextBundle {
    pub working_memory: String,
    pub episodic_memory: Vec<String>,
    pub semantic_memory: Vec<String>,
    pub citations: Vec<String>,
    pub total_tokens: usize,
    pub compression_applied: bool,
}

impl ContextBundle {
    pub fn new(
        working: String,
        episodic: Vec<String>,
        semantic: Vec<String>,
        citations: Vec<String>,
    ) -> Self {
        let total_tokens = estimate_tokens(&working) +
                          episodic.iter().map(|s| estimate_tokens(s)).sum::<usize>() +
                          semantic.iter().map(|s| estimate_tokens(s)).sum::<usize>();

        Self {
            working_memory: working,
            episodic_memory: episodic,
            semantic_memory: semantic,
            citations,
            total_tokens,
            compression_applied: false,
        }
    }

    pub fn compressed(mut self, compressed_text: String, citations: Vec<String>) -> Self {
        self.working_memory = compressed_text;
        self.episodic_memory.clear();
        self.semantic_memory.clear();
        self.citations = citations;
        self.total_tokens = estimate_tokens(&compressed_text);
        self.compression_applied = true;
        self
    }
}

/// Hierarchical context manager
pub struct HierarchicalContextManager {
    context_allocator: ContextAllocator,
    retriever: CalibratedRetriever,
    compressor: ContextCompressor,
    working_memory: WorkingMemory,
    episodic_memory: EpisodicMemory,
    semantic_memory: SemanticMemory,
}

impl HierarchicalContextManager {
    pub fn new(
        retriever: CalibratedRetriever,
        compressor: ContextCompressor,
    ) -> Self {
        Self {
            context_allocator: ContextAllocator,
            retriever,
            compressor,
            working_memory: WorkingMemory::new(),
            episodic_memory: EpisodicMemory::new(),
            semantic_memory: SemanticMemory::new(),
        }
    }

    /// Build complete context for a task
    pub async fn build_context(&self, task: &Task, budget: &ContextBudget) -> Result<ContextBundle, ContextError> {
        info!("Building hierarchical context for task: {}", task.description);

        // 1. Estimate utility costs for each memory type
        let est_costs = self.estimate_context_costs(task).await?;

        // 2. Allocate tokens using knapsack optimization
        let allocation = self.context_allocator.allocate(budget, &est_costs);

        debug!("Context allocation: working={}, episodic={}, semantic={}, citations={}",
               allocation.working, allocation.episodic, allocation.semantic, allocation.citations);

        // 3. Retrieve from each memory with allocated budgets
        let working = self.working_memory.get_within_budget(allocation.working);
        let episodic = self.episodic_memory.retrieve_similar(task, allocation.episodic).await?;
        let semantic = self.retriever.retrieve_with_calibration(
            &task.description,
            allocation.episodic,
            budget
        ).await?;

        // 4. Check if compression is needed
        let total_tokens = estimate_tokens(&working) +
                          episodic.iter().map(|s| estimate_tokens(s)).sum::<usize>() +
                          semantic.iter().map(|s| estimate_tokens(s)).sum::<usize>();

        if total_tokens > budget.max_tokens {
            info!("Context exceeds budget ({} > {}), applying compression", total_tokens, budget.max_tokens);
            let compressed = self.compressor.compress_with_attribution(
                &[working.as_str()],
                &episodic,
                &semantic,
                budget
            ).await?;
            Ok(compressed)
        } else {
            let citations = if allocation.citations {
                vec!["working_memory".to_string(), "episodic_memory".to_string(), "semantic_memory".to_string()]
            } else {
                vec![]
            };
            Ok(ContextBundle::new(working, episodic, semantic, citations))
        }
    }

    /// Estimate context costs and utilities
    async fn estimate_context_costs(&self, task: &Task) -> Result<ContextCosts, ContextError> {
        let utility = ContextUtility {
            working: self.working_memory.estimate_utility(task),
            episodic: self.episodic_memory.estimate_utility(task).await?,
            semantic: self.retriever.estimate_utility(&task.description).await?,
        };

        let tokens = ContextTokens {
            working: self.working_memory.estimate_tokens(),
            episodic: self.episodic_memory.estimate_tokens(),
            semantic: self.retriever.estimate_tokens(),
        };

        Ok(ContextCosts { utility, tokens })
    }

    /// Update memories with new task execution
    pub async fn update_memories(&mut self, task: &Task, result: &IterationContext) -> Result<(), ContextError> {
        // Update episodic memory with task execution
        self.episodic_memory.store_execution(task, result).await?;

        // Update working memory with latest context
        self.working_memory.update_from_iteration(result);

        // Update semantic memory with key learnings
        if let Some(learnings) = self.extract_key_learnings(result) {
            self.semantic_memory.store_learnings(&learnings).await?;
        }

        Ok(())
    }

    /// Extract key learnings from iteration
    fn extract_key_learnings(&self, iteration: &IterationContext) -> Option<Vec<String>> {
        // Extract successful patterns, error fixes, etc.
        let mut learnings = Vec::new();

        if iteration.eval_report.score > 0.8 {
            learnings.push(format!("Successful pattern: {}", iteration.prompt));
        }

        if !iteration.refinement_prompt.is_empty() {
            learnings.push(format!("Improvement needed: {}", iteration.refinement_prompt));
        }

        if learnings.is_empty() {
            None
        } else {
            Some(learnings)
        }
    }

    /// Get context statistics
    pub async fn get_stats(&self) -> ContextStats {
        ContextStats {
            working_memory_size: self.working_memory.size(),
            episodic_memory_entries: self.episodic_memory.size().await,
            semantic_memory_entries: self.semantic_memory.size().await,
            total_tokens_estimated: self.working_memory.estimate_tokens() +
                                   self.episodic_memory.estimate_tokens() +
                                   self.semantic_memory.estimate_tokens(),
        }
    }
}

/// Context costs combining utility and token estimates
#[derive(Clone, Debug)]
pub struct ContextCosts {
    pub utility: ContextUtility,
    pub estimated_tokens: ContextTokens,
}

/// Context statistics
#[derive(Clone, Debug)]
pub struct ContextStats {
    pub working_memory_size: usize,
    pub episodic_memory_entries: usize,
    pub semantic_memory_entries: usize,
    pub total_tokens_estimated: usize,
}

/// Context allocator using knapsack optimization
pub struct ContextAllocator;

impl ContextAllocator {
    pub fn allocate(&self, budget: &ContextBudget, costs: &ContextCosts) -> Allocation {
        let cap = (budget.max_tokens as f32 * (1.0 - budget.headroom)).round() as usize;
        let u = &costs.utility;

        // Simple proportional allocation based on utility scores
        let total_utility = u.working + u.episodic + u.semantic + 1e-6; // avoid division by zero

        let working_tokens = ((u.working / total_utility) * cap as f64) as usize;
        let episodic_tokens = ((u.episodic / total_utility) * cap as f64) as usize;
        let semantic_tokens = ((u.semantic / total_utility) * cap as f64) as usize;

        // Ensure minimum allocations
        let working_tokens = working_tokens.max(100); // At least 100 tokens for working memory
        let remaining = cap.saturating_sub(working_tokens);
        let episodic_tokens = episodic_tokens.min(remaining / 2);
        let semantic_tokens = remaining.saturating_sub(episodic_tokens);

        Allocation {
            working: working_tokens,
            episodic: episodic_tokens,
            semantic: semantic_tokens,
            citations: true, // Enable for critical tasks
        }
    }
}

/// Working memory for immediate task context
pub struct WorkingMemory {
    current_task: Option<String>,
    recent_iterations: Vec<String>,
    max_entries: usize,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            current_task: None,
            recent_iterations: Vec::new(),
            max_entries: 10,
        }
    }

    pub fn update_from_task(&mut self, task: &Task) {
        self.current_task = Some(task.description.clone());
        self.recent_iterations.clear(); // Reset for new task
    }

    pub fn update_from_iteration(&mut self, iteration: &IterationContext) {
        let entry = format!(
            "Iteration {}: Score {:.2}, Prompt: {}, Feedback: {}",
            iteration.iteration,
            iteration.eval_report.score,
            iteration.prompt,
            iteration.refinement_prompt
        );

        self.recent_iterations.push(entry);
        if self.recent_iterations.len() > self.max_entries {
            self.recent_iterations.remove(0);
        }
    }

    pub fn get_within_budget(&self, max_tokens: usize) -> String {
        let mut context = String::new();

        // Add current task
        if let Some(task) = &self.current_task {
            let task_str = format!("Current Task: {}\n\n", task);
            if estimate_tokens(&context) + estimate_tokens(&task_str) <= max_tokens {
                context.push_str(&task_str);
            }
        }

        // Add recent iterations (most recent first)
        for iteration in self.recent_iterations.iter().rev() {
            let iteration_str = format!("{}\n", iteration);
            if estimate_tokens(&context) + estimate_tokens(&iteration_str) <= max_tokens {
                context.push_str(&iteration_str);
            } else {
                break;
            }
        }

        context
    }

    pub fn estimate_utility(&self, _task: &Task) -> f64 {
        // Working memory is always high utility for current task
        0.9
    }

    pub fn estimate_tokens(&self) -> usize {
        estimate_tokens(&self.get_within_budget(usize::MAX))
    }

    pub fn size(&self) -> usize {
        self.recent_iterations.len()
    }
}

/// Episodic memory for historical task executions
pub struct EpisodicMemory {
    episodes: Arc<RwLock<HashMap<String, Vec<TaskEpisode>>>>,
    max_episodes_per_type: usize,
}

#[derive(Clone, Debug)]
pub struct TaskEpisode {
    pub task_description: String,
    pub final_score: f64,
    pub execution_time: u64,
    pub successful: bool,
    pub key_learnings: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EpisodicMemory {
    pub fn new() -> Self {
        Self {
            episodes: Arc::new(RwLock::new(HashMap::new())),
            max_episodes_per_type: 50,
        }
    }

    pub async fn store_execution(&self, task: &Task, result: &IterationContext) -> Result<(), ContextError> {
        let episode = TaskEpisode {
            task_description: task.description.clone(),
            final_score: result.eval_report.score,
            execution_time: result.eval_report.execution_time_ms.unwrap_or(0),
            successful: result.eval_report.score > 0.7,
            key_learnings: vec![result.refinement_prompt.clone()],
            timestamp: chrono::Utc::now(),
        };

        let task_type = task.task_type.to_string();
        let mut episodes = self.episodes.write().await;
        let type_episodes = episodes.entry(task_type).or_insert_with(Vec::new);

        type_episodes.push(episode);
        if type_episodes.len() > self.max_episodes_per_type {
            // Keep most recent episodes
            type_episodes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            type_episodes.truncate(self.max_episodes_per_type);
        }

        Ok(())
    }

    pub async fn retrieve_similar(&self, task: &Task, max_tokens: usize) -> Result<Vec<String>, ContextError> {
        let episodes = self.episodes.read().await;
        let task_type = task.task_type.to_string();

        let similar_episodes = if let Some(type_episodes) = episodes.get(&task_type) {
            // Find episodes with similar success patterns
            type_episodes.iter()
                .filter(|ep| ep.successful)
                .take(3)
                .map(|ep| {
                    format!("Previous successful {} task (score: {:.2}): {}",
                           task_type, ep.final_score, ep.task_description)
                })
                .collect()
        } else {
            Vec::new()
        };

        // Truncate to fit token budget
        let mut result = Vec::new();
        let mut total_tokens = 0;

        for episode in similar_episodes {
            let episode_tokens = estimate_tokens(&episode);
            if total_tokens + episode_tokens <= max_tokens {
                result.push(episode);
                total_tokens += episode_tokens;
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub async fn estimate_utility(&self, task: &Task) -> Result<f64, ContextError> {
        let episodes = self.episodes.read().await;
        let task_type = task.task_type.to_string();

        if let Some(type_episodes) = episodes.get(&task_type) {
            let success_rate = type_episodes.iter()
                .filter(|ep| ep.successful)
                .count() as f64 / type_episodes.len() as f64;

            // Higher utility if we have successful examples
            Ok(success_rate * 0.7) // Scale to reasonable utility score
        } else {
            Ok(0.1) // Low utility if no historical data
        }
    }

    pub async fn estimate_tokens(&self) -> usize {
        let episodes = self.episodes.read().await;
        let total_episodes: usize = episodes.values().map(|v| v.len()).sum();
        // Rough estimate: 200 tokens per episode summary
        total_episodes * 200
    }

    pub async fn size(&self) -> usize {
        let episodes = self.episodes.read().await;
        episodes.values().map(|v| v.len()).sum()
    }
}

/// Semantic memory for knowledge base
pub struct SemanticMemory {
    knowledge_base: Arc<RwLock<HashMap<String, Vec<String>>>>,
    max_entries_per_topic: usize,
}

impl SemanticMemory {
    pub fn new() -> Self {
        Self {
            knowledge_base: Arc::new(RwLock::new(HashMap::new())),
            max_entries_per_topic: 20,
        }
    }

    pub async fn store_learnings(&self, learnings: &[String]) -> Result<(), ContextError> {
        let mut kb = self.knowledge_base.write().await;

        for learning in learnings {
            // Simple topic extraction (could be more sophisticated)
            let topic = self.extract_topic(learning);
            let topic_entries = kb.entry(topic).or_insert_with(Vec::new);

            topic_entries.push(learning.clone());
            if topic_entries.len() > self.max_entries_per_topic {
                topic_entries.remove(0); // Remove oldest
            }
        }

        Ok(())
    }

    pub async fn retrieve_relevant(&self, query: &str, max_tokens: usize) -> Result<Vec<String>, ContextError> {
        let kb = self.knowledge_base.read().await;

        // Simple keyword matching (would be vector search in real implementation)
        let mut relevant = Vec::new();
        let query_lower = query.to_lowercase();

        for (topic, entries) in kb.iter() {
            if topic.to_lowercase().contains(&query_lower) ||
               entries.iter().any(|e| e.to_lowercase().contains(&query_lower)) {

                for entry in entries.iter().rev().take(2) { // Most recent 2
                    let entry_tokens = estimate_tokens(entry);
                    if estimate_tokens(&relevant.concat()) + entry_tokens <= max_tokens {
                        relevant.push(entry.clone());
                    }
                }
            }
        }

        Ok(relevant)
    }

    pub async fn estimate_utility(&self, task: &Task) -> Result<f64, ContextError> {
        // Check if we have relevant knowledge
        let relevant = self.retrieve_relevant(&task.description, 1000).await?;
        let relevance_score = if relevant.is_empty() { 0.0 } else { 0.5 };

        Ok(relevance_score)
    }

    pub async fn estimate_tokens(&self) -> usize {
        let kb = self.knowledge_base.read().await;
        kb.values().map(|entries| {
            entries.iter().map(|e| estimate_tokens(e)).sum::<usize>()
        }).sum()
    }

    pub async fn size(&self) -> usize {
        let kb = self.knowledge_base.read().await;
        kb.values().map(|v| v.len()).sum()
    }

    fn extract_topic(&self, learning: &str) -> String {
        // Simple topic extraction - first few words
        learning.split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join("_")
            .to_lowercase()
    }
}

/// Calibrated retriever (placeholder - would integrate with VectorSearchEngine)
pub struct CalibratedRetriever;

impl CalibratedRetriever {
    pub async fn retrieve_with_calibration(
        &self,
        _query: &str,
        _max_results: usize,
        _budget: &ContextBudget,
    ) -> Result<Vec<String>, ContextError> {
        // Placeholder - would integrate with actual vector search
        Ok(vec![
            "Semantic knowledge: Error handling patterns".to_string(),
            "Semantic knowledge: Code structure best practices".to_string(),
        ])
    }

    pub async fn estimate_utility(&self, _query: &str) -> Result<f64, ContextError> {
        Ok(0.6) // Moderate utility for semantic search
    }

    pub async fn estimate_tokens(&self) -> usize {
        1000 // Rough estimate
    }
}

/// Context compressor (placeholder - would implement intelligent compression)
pub struct ContextCompressor;

impl ContextCompressor {
    pub async fn compress_with_attribution(
        &self,
        _working: &[&str],
        _episodic: &[String],
        _semantic: &[String],
        _budget: &ContextBudget,
    ) -> Result<ContextBundle, ContextError> {
        // Placeholder - would implement actual compression
        let compressed = "Compressed context: Previous successful patterns and key learnings applied.".to_string();
        let citations = vec![
            "working_memory_compressed".to_string(),
            "episodic_patterns".to_string(),
            "semantic_knowledge".to_string(),
        ];

        Ok(ContextBundle::new("".to_string(), vec![], vec![], vec![]).compressed(compressed, citations))
    }
}

/// Estimate token count (rough approximation)
fn estimate_tokens(text: &str) -> usize {
    // Rough approximation: 1 token per 4 characters
    (text.len() / 4).max(1)
}

/// Context errors
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Memory access error: {0}")]
    MemoryError(String),

    #[error("Retrieval failed: {0}")]
    RetrievalError(String),

    #[error("Compression failed: {0}")]
    CompressionError(String),

    #[error("Token budget exceeded")]
    BudgetExceeded,
}
