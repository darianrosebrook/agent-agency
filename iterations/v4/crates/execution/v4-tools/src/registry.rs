//! Tool Registry
//!
//! Central registry for all available tools with capability discovery.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::tool::{Tool, ToolCapability, ToolCategory, ToolId, ToolMetadata};

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: RwLock<HashMap<ToolId, Arc<dyn Tool>>>,
    metadata: RwLock<HashMap<ToolId, ToolMetadata>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool
    pub fn register(&self, tool: Arc<dyn Tool>) {
        let id = tool.id().to_string();
        let metadata = tool.metadata();

        self.tools.write().unwrap().insert(id.clone(), tool);
        self.metadata.write().unwrap().insert(id, metadata);
    }

    /// Get a tool by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn Tool>> {
        self.tools.read().unwrap().get(id).cloned()
    }

    /// Get tool metadata
    pub fn get_metadata(&self, id: &str) -> Option<ToolMetadata> {
        self.metadata.read().unwrap().get(id).cloned()
    }

    /// List all registered tools
    pub fn list(&self) -> Vec<ToolMetadata> {
        self.metadata.read().unwrap().values().cloned().collect()
    }

    /// Find tools by category
    pub fn find_by_category(&self, category: ToolCategory) -> Vec<ToolMetadata> {
        self.metadata
            .read()
            .unwrap()
            .values()
            .filter(|m| m.category == category)
            .cloned()
            .collect()
    }

    /// Find tools by capability
    pub fn find_by_capability(&self, capability: ToolCapability) -> Vec<ToolMetadata> {
        self.metadata
            .read()
            .unwrap()
            .values()
            .filter(|m| m.capabilities.contains(&capability))
            .cloned()
            .collect()
    }

    /// Find tools that can execute a specific operator type
    pub fn find_for_operator(&self, operator_class: &str) -> Vec<ToolMetadata> {
        self.metadata
            .read()
            .unwrap()
            .values()
            .filter(|m| m.supported_operators.iter().any(|op| op == operator_class))
            .cloned()
            .collect()
    }

    /// Get total count of registered tools
    pub fn count(&self) -> usize {
        self.tools.read().unwrap().len()
    }

    /// Unregister a tool
    pub fn unregister(&self, id: &str) -> bool {
        let removed_tool = self.tools.write().unwrap().remove(id).is_some();
        let removed_meta = self.metadata.write().unwrap().remove(id).is_some();
        removed_tool && removed_meta
    }

    /// Check if a tool is registered
    pub fn has(&self, id: &str) -> bool {
        self.tools.read().unwrap().contains_key(id)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry snapshot for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub tools: Vec<ToolMetadata>,
    pub captured_at: chrono::DateTime<chrono::Utc>,
}

impl ToolRegistry {
    /// Take a snapshot of the registry
    pub fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            tools: self.list(),
            captured_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolError;
    use async_trait::async_trait;
    use v4_types::operators::OperatorResult;

    struct MockTool {
        id: String,
        category: ToolCategory,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn id(&self) -> &str {
            &self.id
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                id: self.id.clone(),
                name: format!("Mock {}", self.id),
                description: "A mock tool for testing".to_string(),
                version: "1.0.0".to_string(),
                category: self.category,
                capabilities: vec![ToolCapability::ReadOnly],
                supported_operators: vec!["S".to_string()],
                requires_sandbox: false,
            }
        }

        async fn execute(
            &self,
            _operator: &v4_types::OperatorType,
        ) -> Result<OperatorResult, ToolError> {
            unimplemented!()
        }

        fn can_execute(&self, _operator: &v4_types::OperatorType) -> bool {
            true
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockTool {
            id: "test-tool".to_string(),
            category: ToolCategory::FileSystem,
        });

        registry.register(tool.clone());

        assert!(registry.has("test-tool"));
        assert!(registry.get("test-tool").is_some());
        assert!(registry.get_metadata("test-tool").is_some());
    }

    #[test]
    fn test_find_by_category() {
        let registry = ToolRegistry::new();

        registry.register(Arc::new(MockTool {
            id: "fs-tool".to_string(),
            category: ToolCategory::FileSystem,
        }));

        registry.register(Arc::new(MockTool {
            id: "web-tool".to_string(),
            category: ToolCategory::Web,
        }));

        let fs_tools = registry.find_by_category(ToolCategory::FileSystem);
        assert_eq!(fs_tools.len(), 1);
        assert_eq!(fs_tools[0].id, "fs-tool");
    }

    #[test]
    fn test_unregister() {
        let registry = ToolRegistry::new();

        registry.register(Arc::new(MockTool {
            id: "temp-tool".to_string(),
            category: ToolCategory::Memory,
        }));

        assert!(registry.has("temp-tool"));
        assert!(registry.unregister("temp-tool"));
        assert!(!registry.has("temp-tool"));
    }

    #[test]
    fn test_snapshot() {
        let registry = ToolRegistry::new();

        registry.register(Arc::new(MockTool {
            id: "tool-1".to_string(),
            category: ToolCategory::FileSystem,
        }));

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.tools.len(), 1);
    }
}
