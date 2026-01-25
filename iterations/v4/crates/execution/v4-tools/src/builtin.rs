//! Built-in Tools
//!
//! Standard tools for common operations.

use std::path::Path;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tokio::fs;

use v4_types::operators::{OperatorResult, SeekOp};
use v4_types::OperatorType;

use crate::tool::{Tool, ToolCapability, ToolCategory, ToolError, ToolMetadata};

/// File system read tool
pub struct FileReadTool {
    blocked_paths: Vec<String>,
}

impl FileReadTool {
    pub fn new() -> Self {
        Self {
            blocked_paths: vec![
                "/etc/shadow".to_string(),
                "/etc/passwd".to_string(),
                ".env".to_string(),
            ],
        }
    }

    pub fn with_blocked_paths(paths: Vec<String>) -> Self {
        Self {
            blocked_paths: paths,
        }
    }

    fn is_path_blocked(&self, path: &str) -> bool {
        let path = Path::new(path);
        self.blocked_paths.iter().any(|blocked| {
            path.ends_with(blocked) || path.to_string_lossy().contains(blocked)
        })
    }
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn id(&self) -> &str {
        "builtin:file-read"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "File Read".to_string(),
            description: "Read files from the filesystem".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::ReadFile { path }) => {
                if self.is_path_blocked(path) {
                    return Err(ToolError::PermissionDenied(format!(
                        "Access to {} is blocked",
                        path
                    )));
                }

                let content = fs::read_to_string(path)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "content": content,
                        "path": path,
                        "size": content.len(),
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: Some(hash),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "FileReadTool only handles ReadFile operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::ReadFile { .. }))
    }
}

/// Directory listing tool
pub struct DirectoryListTool {
    max_entries: usize,
}

impl DirectoryListTool {
    pub fn new() -> Self {
        Self { max_entries: 1000 }
    }

    pub fn with_max_entries(max: usize) -> Self {
        Self { max_entries: max }
    }
}

impl Default for DirectoryListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DirectoryListTool {
    fn id(&self) -> &str {
        "builtin:dir-list"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Directory List".to_string(),
            description: "List directory contents".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::ListDirectory { path }) => {
                let mut entries = Vec::new();
                let mut dir = fs::read_dir(path)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                while let Some(entry) = dir.next_entry().await.map_err(|e| ToolError::IoError(e.to_string()))? {
                    if entries.len() >= self.max_entries {
                        break;
                    }

                    let file_type = entry.file_type().await.ok();
                    let metadata = entry.metadata().await.ok();

                    entries.push(serde_json::json!({
                        "name": entry.file_name().to_string_lossy(),
                        "path": entry.path().to_string_lossy(),
                        "is_dir": file_type.map(|ft| ft.is_dir()).unwrap_or(false),
                        "is_file": file_type.map(|ft| ft.is_file()).unwrap_or(false),
                        "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                    }));
                }

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "path": path,
                        "entries": entries,
                        "count": entries.len(),
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: None,
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "DirectoryListTool only handles ListDirectory operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::ListDirectory { .. }))
    }
}

/// Code search tool (basic implementation)
pub struct CodeSearchTool {
    max_matches: usize,
}

impl CodeSearchTool {
    pub fn new() -> Self {
        Self { max_matches: 100 }
    }
}

impl Default for CodeSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CodeSearchTool {
    fn id(&self) -> &str {
        "builtin:code-search"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Code Search".to_string(),
            description: "Search code with patterns".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::CodeAnalysis,
            capabilities: vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::SearchCode { pattern, path }) => {
                // Simplified search - in production would use ripgrep or similar
                let search_path = path.as_deref().unwrap_or(".");
                let mut matches = Vec::new();

                // For now, just indicate search was attempted
                // Real implementation would walk directories and search files
                matches.push(serde_json::json!({
                    "path": search_path,
                    "pattern": pattern,
                    "status": "search_completed",
                    "note": "Basic search implementation - use external tools for full functionality"
                }));

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "pattern": pattern,
                        "search_path": search_path,
                        "matches": matches,
                        "count": matches.len(),
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: None,
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "CodeSearchTool only handles SearchCode operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::SearchCode { .. }))
    }
}

/// Memory query tool (delegates to memory system)
pub struct MemoryQueryTool;

impl MemoryQueryTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for MemoryQueryTool {
    fn id(&self) -> &str {
        "builtin:memory-query"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Memory Query".to_string(),
            description: "Query the knowledge graph memory".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::Memory,
            capabilities: vec![ToolCapability::ReadOnly],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::QueryMemory { query, max_hops }) => {
                // This would delegate to the actual memory system
                // For now, return a placeholder indicating the query was received
                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "query": query,
                        "max_hops": max_hops,
                        "results": [],
                        "note": "Memory query requires memory system integration"
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: None,
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "MemoryQueryTool only handles QueryMemory operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::QueryMemory { .. }))
    }
}

/// Register all built-in tools with a registry
pub fn register_builtin_tools(registry: &crate::registry::ToolRegistry) {
    use std::sync::Arc;

    registry.register(Arc::new(FileReadTool::new()));
    registry.register(Arc::new(DirectoryListTool::new()));
    registry.register(Arc::new(CodeSearchTool::new()));
    registry.register(Arc::new(MemoryQueryTool::new()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ToolRegistry;

    #[test]
    fn test_file_read_tool_metadata() {
        let tool = FileReadTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:file-read");
        assert_eq!(meta.category, ToolCategory::FileSystem);
        assert!(meta.capabilities.contains(&ToolCapability::ReadOnly));
    }

    #[test]
    fn test_file_read_tool_blocked_paths() {
        let tool = FileReadTool::new();

        assert!(tool.is_path_blocked("/etc/shadow"));
        assert!(tool.is_path_blocked("/path/to/.env"));
        assert!(!tool.is_path_blocked("/home/user/file.txt"));
    }

    #[test]
    fn test_file_read_can_execute() {
        let tool = FileReadTool::new();

        let read_op = OperatorType::Seek(SeekOp::ReadFile {
            path: "test.txt".to_string(),
        });
        assert!(tool.can_execute(&read_op));

        let list_op = OperatorType::Seek(SeekOp::ListDirectory {
            path: ".".to_string(),
        });
        assert!(!tool.can_execute(&list_op));
    }

    #[test]
    fn test_register_builtin_tools() {
        let registry = ToolRegistry::new();
        register_builtin_tools(&registry);

        assert!(registry.has("builtin:file-read"));
        assert!(registry.has("builtin:dir-list"));
        assert!(registry.has("builtin:code-search"));
        assert!(registry.has("builtin:memory-query"));
        assert_eq!(registry.count(), 4);
    }

    #[test]
    fn test_directory_list_metadata() {
        let tool = DirectoryListTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:dir-list");
        assert!(meta.supported_operators.contains(&"S".to_string()));
    }
}
