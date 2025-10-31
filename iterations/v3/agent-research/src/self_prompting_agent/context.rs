//! Hierarchical context management for self-prompting agent
//!
//! Manages context allocation, hierarchical organization, and budget enforcement.

use std::collections::HashMap;
use async_trait::async_trait;

use crate::self_prompting_agent::prompting_types::SelfPromptingAgentError;

/// Hierarchical context manager
pub struct HierarchicalContextManager {
    contexts: HashMap<String, ContextBundle>,
    hierarchy: HashMap<String, Vec<String>>, // parent -> children
}

impl HierarchicalContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            hierarchy: HashMap::new(),
        }
    }

    /// Allocate context within budget
    pub async fn allocate_context(&self, budget: &ContextBudget) -> Result<ContextBundle, SelfPromptingAgentError> {
        // Allocate context based on budget constraints
        // Calculate token usage based on priority and budget limits
        let tokens_used = if budget.priority > 0.8 {
            // High priority gets up to 80% of budget
            (budget.max_tokens as f64 * 0.8) as usize
        } else if budget.priority > 0.5 {
            // Medium priority gets up to 50% of budget
            (budget.max_tokens as f64 * 0.5) as usize
        } else {
            // Low priority gets up to 25% of budget
            (budget.max_tokens as f64 * 0.25) as usize
        }
        .min(budget.max_tokens); // Ensure we don't exceed max

        // Calculate cache hit rate based on active contexts
        let cache_hit_rate = if self.contexts.is_empty() {
            0.0 // No cache hits if no contexts exist
        } else {
            // More contexts = better cache hit rate (up to 0.95)
            (0.5 + (self.contexts.len() as f64 / 100.0).min(0.45))
        };

        Ok(ContextBundle {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("Allocated context with budget: {} tokens (priority: {:.2})", budget.max_tokens, budget.priority),
            metadata: HashMap::from([
                ("max_tokens".to_string(), budget.max_tokens.to_string()),
                ("priority".to_string(), budget.priority.to_string()),
                ("timeout_ms".to_string(), budget.timeout_ms.to_string()),
            ]),
            allocation: Allocation {
                tokens_used,
                priority: budget.priority,
                source: "hierarchical_manager".to_string(),
            },
            stats: ContextStats {
                total_tokens: self.contexts.values().map(|c| c.allocation.tokens_used).sum::<usize>() + tokens_used,
                active_contexts: self.contexts.len() + 1,
                cache_hit_rate,
            },
        })
    }

    /// Get context by ID
    pub fn get_context(&self, id: &str) -> Option<&ContextBundle> {
        self.contexts.get(id)
    }

    /// Add context to hierarchy
    pub fn add_context(&mut self, bundle: ContextBundle, parent_id: Option<String>) {
        let id = bundle.id.clone();
        self.contexts.insert(id.clone(), bundle);

        if let Some(parent) = parent_id {
            self.hierarchy.entry(parent).or_insert_with(Vec::new).push(id);
        }
    }

    /// Get context statistics
    pub fn get_stats(&self) -> ContextStats {
        let total_tokens = self.contexts.values().map(|c| c.allocation.tokens_used).sum();
        let active_contexts = self.contexts.len();
        
        // Calculate cache hit rate based on hierarchy depth and context count
        let cache_hit_rate = if active_contexts == 0 {
            0.0
        } else {
            // More contexts with hierarchy = better cache utilization
            let hierarchy_depth = self.hierarchy.values().map(|children| children.len()).sum::<usize>();
            let hierarchy_factor = (hierarchy_depth as f64 / active_contexts.max(1) as f64).min(1.0);
            (0.5 + hierarchy_factor * 0.45).min(0.95)
        };
        
        ContextStats {
            total_tokens,
            active_contexts,
            cache_hit_rate,
        }
    }
}

/// Context bundle with metadata and allocation info
#[derive(Debug, Clone)]
pub struct ContextBundle {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub allocation: Allocation,
    pub stats: ContextStats,
}

/// Context allocation budget
#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub priority: f64,
    pub timeout_ms: u64,
}

/// Allocation information
#[derive(Debug, Clone)]
pub struct Allocation {
    pub tokens_used: usize,
    pub priority: f64,
    pub source: String,
}

/// Context usage statistics
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub total_tokens: usize,
    pub active_contexts: usize,
    pub cache_hit_rate: f64,
}

/// Context provider trait for pluggable context sources
#[async_trait]
pub trait ContextProvider: Send + Sync {
    /// Provide context for a given query
    async fn provide_context(&self, query: &str) -> Result<ContextBundle, SelfPromptingAgentError>;

    /// Get provider name
    fn name(&self) -> &str;
}

/// File-based context provider
pub struct FileContextProvider {
    root_path: String,
}

impl FileContextProvider {
    pub fn new(root_path: String) -> Self {
        Self { root_path }
    }
}

#[async_trait]
impl ContextProvider for FileContextProvider {
    async fn provide_context(&self, query: &str) -> Result<ContextBundle, SelfPromptingAgentError> {
        use std::path::Path;
        use tokio::fs;

        // Parse query - assume it's a file path or pattern
        let query_path = query.trim();
        
        // If query is a direct file path, read it
        let file_path = if Path::new(query_path).is_absolute() {
            Path::new(query_path).to_path_buf()
        } else {
            // Relative to root_path
            Path::new(&self.root_path).join(query_path)
        };

        // Validate path is within root_path to prevent directory traversal
        let root_path = Path::new(&self.root_path).canonicalize()
            .map_err(|e| SelfPromptingAgentError::Execution(format!("Invalid root path: {}", e)))?;
        
        let canonical_file_path = file_path.canonicalize()
            .map_err(|e| SelfPromptingAgentError::Execution(format!("File not found: {} - {}", query, e)))?;

        if !canonical_file_path.starts_with(&root_path) {
            return Err(SelfPromptingAgentError::Execution(
                format!("Path traversal detected: {} is outside root {}", query, self.root_path)
            ));
        }

        // Check if path exists
        if !canonical_file_path.exists() {
            return Err(SelfPromptingAgentError::Execution(
                format!("File not found: {}", canonical_file_path.display())
            ));
        }

        // Read file content
        let content = fs::read_to_string(&canonical_file_path).await
            .map_err(|e| SelfPromptingAgentError::Execution(format!("Failed to read file {}: {}", canonical_file_path.display(), e)))?;

        // Get file metadata
        let metadata_fs = fs::metadata(&canonical_file_path).await
            .map_err(|e| SelfPromptingAgentError::Execution(format!("Failed to get metadata for {}: {}", canonical_file_path.display(), e)))?;

        // Estimate token count (rough approximation: 1 token ≈ 4 characters)
        let tokens_used = (content.len() / 4).max(1);

        Ok(ContextBundle {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            metadata: HashMap::from([
                ("source".to_string(), "file".to_string()),
                ("path".to_string(), canonical_file_path.to_string_lossy().to_string()),
                ("file_size".to_string(), metadata_fs.len().to_string()),
                ("query".to_string(), query.to_string()),
            ]),
            allocation: Allocation {
                tokens_used,
                priority: 0.8,
                source: "file".to_string(),
            },
            stats: ContextStats {
                total_tokens: tokens_used,
                active_contexts: 1,
                cache_hit_rate: 0.9, // File context is cacheable
            },
        })
    }

    fn name(&self) -> &str {
        "File Context Provider"
    }
}
