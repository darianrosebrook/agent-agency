//! Tests for hierarchical context management

use std::collections::HashMap;
use self_prompting_agent::context::*;
use self_prompting_agent::types::SelfPromptingAgentError;

#[tokio::test]
async fn test_hierarchical_context_manager_creation() {
    let manager = HierarchicalContextManager::new();

    // Test that a new manager has no contexts and no stats
    let stats = manager.get_stats();
    assert_eq!(stats.total_tokens, 0);
    assert_eq!(stats.active_contexts, 0);
}

#[tokio::test]
async fn test_allocate_context() {
    let manager = HierarchicalContextManager::new();
    let budget = ContextBudget {
        max_tokens: 1000,
        priority: 1.0,
        timeout_ms: 5000,
    };

    let result = manager.allocate_context(&budget).await.unwrap();

    assert!(!result.id.is_empty());
    assert!(result.content.contains("1000 tokens"));
    assert_eq!(result.allocation.tokens_used, 500);
    assert_eq!(result.allocation.priority, 1.0);
    assert_eq!(result.allocation.source, "stub");
    assert_eq!(result.stats.total_tokens, 1000);
    assert_eq!(result.stats.active_contexts, 5);
    assert_eq!(result.stats.cache_hit_rate, 0.85);
}

#[tokio::test]
async fn test_get_context_not_found() {
    let manager = HierarchicalContextManager::new();

    let result = manager.get_context("non-existent");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_add_context_and_get() {
    let mut manager = HierarchicalContextManager::new();

    let bundle = ContextBundle {
        id: "test-context".to_string(),
        content: "Test content".to_string(),
        metadata: HashMap::new(),
        allocation: Allocation {
            tokens_used: 100,
            priority: 0.5,
            source: "test".to_string(),
        },
        stats: ContextStats {
            total_tokens: 100,
            active_contexts: 1,
            cache_hit_rate: 0.8,
        },
    };

    manager.add_context(bundle, None);

    let retrieved = manager.get_context("test-context");
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, "test-context");
    assert_eq!(retrieved.content, "Test content");
}

#[tokio::test]
async fn test_add_context_with_parent() {
    let mut manager = HierarchicalContextManager::new();

    // Add parent context
    let parent_bundle = ContextBundle {
        id: "parent".to_string(),
        content: "Parent content".to_string(),
        metadata: HashMap::new(),
        allocation: Allocation {
            tokens_used: 50,
            priority: 0.8,
            source: "test".to_string(),
        },
        stats: ContextStats {
            total_tokens: 50,
            active_contexts: 1,
            cache_hit_rate: 0.9,
        },
    };

    manager.add_context(parent_bundle, None);

    // Add child context
    let child_bundle = ContextBundle {
        id: "child".to_string(),
        content: "Child content".to_string(),
        metadata: HashMap::new(),
        allocation: Allocation {
            tokens_used: 75,
            priority: 0.6,
            source: "test".to_string(),
        },
        stats: ContextStats {
            total_tokens: 75,
            active_contexts: 1,
            cache_hit_rate: 0.7,
        },
    };

    manager.add_context(child_bundle, Some("parent".to_string()));

    // Check that we can retrieve the child context
    let retrieved_child = manager.get_context("child");
    assert!(retrieved_child.is_some());
    assert_eq!(retrieved_child.unwrap().content, "Child content");
}

#[tokio::test]
async fn test_get_stats_empty() {
    let manager = HierarchicalContextManager::new();

    let stats = manager.get_stats();

    assert_eq!(stats.total_tokens, 0);
    assert_eq!(stats.active_contexts, 0);
    assert_eq!(stats.cache_hit_rate, 0.85);
}

#[tokio::test]
async fn test_get_stats_with_contexts() {
    let mut manager = HierarchicalContextManager::new();

    let bundle1 = ContextBundle {
        id: "ctx1".to_string(),
        content: "Content 1".to_string(),
        metadata: HashMap::new(),
        allocation: Allocation {
            tokens_used: 100,
            priority: 0.8,
            source: "test".to_string(),
        },
        stats: ContextStats {
            total_tokens: 100,
            active_contexts: 1,
            cache_hit_rate: 0.8,
        },
    };

    let bundle2 = ContextBundle {
        id: "ctx2".to_string(),
        content: "Content 2".to_string(),
        metadata: HashMap::new(),
        allocation: Allocation {
            tokens_used: 200,
            priority: 0.6,
            source: "test".to_string(),
        },
        stats: ContextStats {
            total_tokens: 200,
            active_contexts: 1,
            cache_hit_rate: 0.9,
        },
    };

    manager.add_context(bundle1, None);
    manager.add_context(bundle2, None);

    let stats = manager.get_stats();

    assert_eq!(stats.total_tokens, 300); // 100 + 200
    assert_eq!(stats.active_contexts, 2);
    assert_eq!(stats.cache_hit_rate, 0.85); // Stub value
}

#[test]
fn test_context_bundle_creation() {
    let bundle = ContextBundle {
        id: "test-bundle".to_string(),
        content: "Test content".to_string(),
        metadata: HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]),
        allocation: Allocation {
            tokens_used: 150,
            priority: 0.75,
            source: "manual".to_string(),
        },
        stats: ContextStats {
            total_tokens: 150,
            active_contexts: 1,
            cache_hit_rate: 0.95,
        },
    };

    assert_eq!(bundle.id, "test-bundle");
    assert_eq!(bundle.content, "Test content");
    assert_eq!(bundle.metadata.len(), 2);
    assert_eq!(bundle.allocation.tokens_used, 150);
    assert_eq!(bundle.stats.total_tokens, 150);
}

#[test]
fn test_context_budget_creation() {
    let budget = ContextBudget {
        max_tokens: 2048,
        priority: 0.9,
        timeout_ms: 10000,
    };

    assert_eq!(budget.max_tokens, 2048);
    assert_eq!(budget.priority, 0.9);
    assert_eq!(budget.timeout_ms, 10000);
}

#[test]
fn test_allocation_creation() {
    let allocation = Allocation {
        tokens_used: 512,
        priority: 0.7,
        source: "api".to_string(),
    };

    assert_eq!(allocation.tokens_used, 512);
    assert_eq!(allocation.priority, 0.7);
    assert_eq!(allocation.source, "api");
}

#[test]
fn test_context_stats_creation() {
    let stats = ContextStats {
        total_tokens: 1024,
        active_contexts: 3,
        cache_hit_rate: 0.82,
    };

    assert_eq!(stats.total_tokens, 1024);
    assert_eq!(stats.active_contexts, 3);
    assert_eq!(stats.cache_hit_rate, 0.82);
}

#[test]
fn test_file_context_provider_creation() {
    let provider = FileContextProvider::new("/tmp/contexts".to_string());

    // Test that provider is created and has the expected name
    assert_eq!(provider.name(), "File Context Provider");
}

#[tokio::test]
async fn test_file_context_provider_provide_context() {
    let provider = FileContextProvider::new("/tmp/contexts".to_string());

    let result = provider.provide_context("test query").await.unwrap();

    assert!(!result.id.is_empty());
    assert_eq!(result.content, "File context for query: test query");
    assert_eq!(result.metadata.get("source"), Some(&"file".to_string()));
    assert_eq!(result.metadata.get("path"), Some(&"/tmp/contexts".to_string()));
    assert_eq!(result.allocation.tokens_used, 200);
    assert_eq!(result.allocation.priority, 0.8);
    assert_eq!(result.allocation.source, "file");
    assert_eq!(result.stats.total_tokens, 200);
    assert_eq!(result.stats.active_contexts, 1);
    assert_eq!(result.stats.cache_hit_rate, 0.9);
}

#[test]
fn test_file_context_provider_name() {
    let provider = FileContextProvider::new("/tmp/contexts".to_string());

    assert_eq!(provider.name(), "File Context Provider");
}

#[test]
fn test_context_provider_trait() {
    // Test that FileContextProvider implements ContextProvider trait
    let provider = FileContextProvider::new("/tmp/test".to_string());
    let _provider_trait: &dyn ContextProvider = &provider;
}

#[test]
fn test_debug_implementations() {
    let bundle = ContextBundle {
        id: "debug-test".to_string(),
        content: "Debug content".to_string(),
        metadata: HashMap::new(),
        allocation: Allocation {
            tokens_used: 42,
            priority: 0.42,
            source: "debug".to_string(),
        },
        stats: ContextStats {
            total_tokens: 42,
            active_contexts: 1,
            cache_hit_rate: 0.42,
        },
    };

    let debug_str = format!("{:?}", bundle);
    assert!(debug_str.contains("debug-test"));
    assert!(debug_str.contains("Debug content"));

    let budget = ContextBudget {
        max_tokens: 100,
        priority: 0.5,
        timeout_ms: 1000,
    };

    let debug_str = format!("{:?}", budget);
    assert!(debug_str.contains("100"));
    assert!(debug_str.contains("0.5"));
}

#[test]
fn test_clone_implementations() {
    let original_bundle = ContextBundle {
        id: "original".to_string(),
        content: "Original content".to_string(),
        metadata: HashMap::from([("key".to_string(), "value".to_string())]),
        allocation: Allocation {
            tokens_used: 100,
            priority: 0.8,
            source: "original".to_string(),
        },
        stats: ContextStats {
            total_tokens: 100,
            active_contexts: 1,
            cache_hit_rate: 0.8,
        },
    };

    let cloned_bundle = original_bundle.clone();

    assert_eq!(original_bundle.id, cloned_bundle.id);
    assert_eq!(original_bundle.content, cloned_bundle.content);
    assert_eq!(original_bundle.metadata, cloned_bundle.metadata);
}
