//! Hierarchical context management for self-prompting agent
//!
//! Manages context allocation, hierarchical organization, and budget enforcement.

use std::collections::HashMap;
use async_trait::async_trait;

use crate::types::SelfPromptingAgentError;

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
        // Stub implementation - would allocate context based on budget
        Ok(ContextBundle {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("Allocated context with budget: {} tokens", budget.max_tokens),
            metadata: HashMap::new(),
            allocation: Allocation {
                tokens_used: 500,
                priority: 1.0,
                source: "stub".to_string(),
            },
            stats: ContextStats {
                total_tokens: 1000,
                active_contexts: 5,
                cache_hit_rate: 0.85,
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
        ContextStats {
            total_tokens: self.contexts.values().map(|c| c.allocation.tokens_used).sum(),
            active_contexts: self.contexts.len(),
            cache_hit_rate: 0.85, // Stub value
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
        // Stub implementation - would read from files
        Ok(ContextBundle {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("File context for query: {}", query),
            metadata: HashMap::from([
                ("source".to_string(), "file".to_string()),
                ("path".to_string(), self.root_path.clone()),
            ]),
            allocation: Allocation {
                tokens_used: 200,
                priority: 0.8,
                source: "file".to_string(),
            },
            stats: ContextStats {
                total_tokens: 200,
                active_contexts: 1,
                cache_hit_rate: 0.9,
            },
        })
    }

    fn name(&self) -> &str {
        "File Context Provider"
    }
}
