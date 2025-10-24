//! Service Registration and Tool Discovery
//!
//! This module provides the infrastructure for services from other crates
//! to register their capabilities as MCP tools. Services can dynamically
//! register tools at runtime, making the system highly modular and extensible.
//!
//! ## Service Registration Pattern
//!
//! Services implement the `ToolProvider` trait and register their tools:
//!
//! ```rust
//! struct KnowledgeSeekerService;
//!
//! impl ToolProvider for KnowledgeSeekerService {
//!     fn register_tools(&self, registry: &MCPIntegration) -> Vec<MCPTool> {
//!         vec![
//!             create_tool_definition(
//!                 "knowledge_search",
//!                 "Search knowledge base for information",
//!                 ToolType::Utility,
//!                 vec![ToolCapability::TextProcessing],
//!                 vec![create_parameter("query", "Search query", "string", true, None)],
//!                 vec![],
//!             )
//!         ]
//!     }
//! }
//! ```

use crate::mcp_integration::{MCPIntegration, create_tool_definition, create_parameter};
use agent_mcp::{MCPTool, ToolType, ToolCapability, ParameterDefinition};
use std::sync::Arc;
use tracing::{info, warn};

/// Trait for services that provide MCP tools
#[async_trait::async_trait]
pub trait ToolProvider: Send + Sync {
    /// Get the service name
    fn service_name(&self) -> &'static str;

    /// Register tools provided by this service
    fn register_tools(&self) -> Vec<MCPTool>;

    /// Optional: Initialize the service
    async fn initialize(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    /// Optional: Check if service is healthy
    async fn health_check(&self) -> ServiceHealth {
        ServiceHealth::Healthy
    }
}

/// Service registry for managing tool providers
pub struct ServiceRegistry {
    mcp_integration: Arc<MCPIntegration>,
    registered_services: std::collections::HashMap<String, Box<dyn ToolProvider>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(mcp_integration: Arc<MCPIntegration>) -> Self {
        Self {
            mcp_integration,
            registered_services: std::collections::HashMap::new(),
        }
    }

    /// Register a service and its tools
    pub async fn register_service(&mut self, service: Box<dyn ToolProvider>) -> Result<(), ServiceError> {
        let service_name = service.service_name().to_string();

        info!("Registering service: {}", service_name);

        // Initialize the service
        service.initialize().await?;

        // Register service tools with MCP
        let tools = service.register_tools();
        self.mcp_integration.register_service_tools(&service_name, tools).await
            .map_err(|e| ServiceError::ToolRegistrationFailed(format!("{}: {}", service_name, e)))?;

        // Store the service
        self.registered_services.insert(service_name.clone(), service);

        info!("Successfully registered service: {} with {} tools", service_name, tools.len());
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister_service(&mut self, service_name: &str) -> Result<(), ServiceError> {
        if let Some(service) = self.registered_services.remove(service_name) {
            info!("Unregistered service: {}", service_name);
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound(service_name.to_string()))
        }
    }

    /// Get all registered services
    pub fn get_registered_services(&self) -> Vec<String> {
        self.registered_services.keys().cloned().collect()
    }

    /// Check health of all services
    pub async fn health_check_all(&self) -> std::collections::HashMap<String, ServiceHealth> {
        let mut results = std::collections::HashMap::new();

        for (name, service) in &self.registered_services {
            let health = service.health_check().await;
            results.insert(name.clone(), health);
        }

        results
    }
}

/// Service health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Errors from service operations
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Service initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Tool registration failed: {0}")]
    ToolRegistrationFailed(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Service health check failed: {0}")]
    HealthCheckFailed(String),
}

/// Example service implementations for key capabilities

/// Knowledge seeker service (from research crate)
pub struct KnowledgeSeekerService;

#[async_trait::async_trait]
impl ToolProvider for KnowledgeSeekerService {
    fn service_name(&self) -> &'static str {
        "knowledge_seeker"
    }

    fn register_tools(&self) -> Vec<MCPTool> {
        vec![
            create_tool_definition(
                "knowledge_search",
                "Search knowledge base for relevant information and context",
                ToolType::Utility,
                vec![ToolCapability::TextProcessing, ToolCapability::NetworkAccess],
                vec![
                    create_parameter("query", "Search query or topic", "string", true, None),
                    create_parameter("context_type", "Type of context needed", "string", false, Some(serde_json::json!("general"))),
                ],
                vec![
                    create_parameter("max_results", "Maximum number of results", "number", false, Some(serde_json::json!(10))),
                ],
            ),
            create_tool_definition(
                "context_synthesis",
                "Synthesize and combine multiple information sources into coherent context",
                ToolType::Utility,
                vec![ToolCapability::TextProcessing],
                vec![
                    create_parameter("sources", "Array of information sources to synthesize", "array", true, None),
                    create_parameter("topic", "Topic or focus area for synthesis", "string", true, None),
                ],
                vec![
                    create_parameter("synthesis_depth", "Depth of synthesis (shallow|medium|deep)", "string", false, Some(serde_json::json!("medium"))),
                ],
            ),
        ]
    }
}

/// Web search service
pub struct WebSearchService;

#[async_trait::async_trait]
impl ToolProvider for WebSearchService {
    fn service_name(&self) -> &'static str {
        "web_search"
    }

    fn register_tools(&self) -> Vec<MCPTool> {
        vec![
            create_tool_definition(
                "web_search",
                "Search the web for current information and resources",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess, ToolCapability::TextProcessing],
                vec![
                    create_parameter("query", "Search query", "string", true, None),
                ],
                vec![
                    create_parameter("max_results", "Maximum results to return", "number", false, Some(serde_json::json!(5))),
                    create_parameter("include_snippets", "Include content snippets", "boolean", false, Some(serde_json::json!(true))),
                ],
            ),
            create_tool_definition(
                "url_fetch",
                "Fetch and extract content from a specific URL",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess, ToolCapability::TextProcessing],
                vec![
                    create_parameter("url", "URL to fetch content from", "string", true, None),
                ],
                vec![
                    create_parameter("extract_text", "Extract readable text only", "boolean", false, Some(serde_json::json!(true))),
                    create_parameter("max_content_length", "Maximum content length to return", "number", false, Some(serde_json::json!(5000))),
                ],
            ),
        ]
    }
}

/// File system service (for file operations)
pub struct FileSystemService;

#[async_trait::async_trait]
impl ToolProvider for FileSystemService {
    fn service_name(&self) -> &'static str {
        "file_system"
    }

    fn register_tools(&self) -> Vec<MCPTool> {
        vec![
            create_tool_definition(
                "file_read",
                "Read content from a file",
                ToolType::Utility,
                vec![ToolCapability::FileRead, ToolCapability::FileSystemAccess],
                vec![
                    create_parameter("file_path", "Path to the file to read", "string", true, None),
                ],
                vec![
                    create_parameter("encoding", "File encoding", "string", false, Some(serde_json::json!("utf-8"))),
                    create_parameter("max_bytes", "Maximum bytes to read", "number", false, Some(serde_json::json!(1048576))), // 1MB
                ],
            ),
            create_tool_definition(
                "file_write",
                "Write content to a file",
                ToolType::Utility,
                vec![ToolCapability::FileWrite, ToolCapability::FileSystemAccess],
                vec![
                    create_parameter("file_path", "Path to the file to write", "string", true, None),
                    create_parameter("content", "Content to write", "string", true, None),
                ],
                vec![
                    create_parameter("encoding", "File encoding", "string", false, Some(serde_json::json!("utf-8"))),
                    create_parameter("create_dirs", "Create parent directories if needed", "boolean", false, Some(serde_json::json!(true))),
                ],
            ),
            create_tool_definition(
                "directory_list",
                "List contents of a directory",
                ToolType::Utility,
                vec![ToolCapability::FileSystemAccess],
                vec![
                    create_parameter("dir_path", "Path to the directory", "string", true, None),
                ],
                vec![
                    create_parameter("recursive", "List recursively", "boolean", false, Some(serde_json::json!(false))),
                    create_parameter("include_hidden", "Include hidden files", "boolean", false, Some(serde_json::json!(false))),
                ],
            ),
        ]
    }
}

/// Federated learning coordination service
pub struct FederatedLearningService;

#[async_trait::async_trait]
impl ToolProvider for FederatedLearningService {
    fn service_name(&self) -> &'static str {
        "federated_learning"
    }

    fn register_tools(&self) -> Vec<MCPTool> {
        vec![
            create_tool_definition(
                "fl_submit_update",
                "Submit a model update for federated aggregation",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess, ToolCapability::DatabaseAccess],
                vec![
                    create_parameter("model_update", "Serialized model update data", "string", true, None),
                    create_parameter("participant_id", "Unique participant identifier", "string", true, None),
                ],
                vec![
                    create_parameter("round_id", "Federated learning round ID", "string", false, None),
                    create_parameter("metadata", "Additional metadata", "object", false, None),
                ],
            ),
            create_tool_definition(
                "fl_get_global_model",
                "Retrieve the current global model state",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess, ToolCapability::DatabaseAccess],
                vec![],
                vec![
                    create_parameter("round_id", "Specific round ID to retrieve", "string", false, None),
                ],
            ),
            create_tool_definition(
                "fl_participant_register",
                "Register as a federated learning participant",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess, ToolCapability::DatabaseAccess],
                vec![
                    create_parameter("participant_info", "Participant information and capabilities", "object", true, None),
                ],
                vec![],
            ),
        ]
    }
}

/// Model hotswap service
pub struct ModelHotswapService;

#[async_trait::async_trait]
impl ToolProvider for ModelHotswapService {
    fn service_name(&self) -> &'static str {
        "model_hotswap"
    }

    fn register_tools(&self) -> Vec<MCPTool> {
        vec![
            create_tool_definition(
                "model_deploy",
                "Deploy a new model version with traffic splitting",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess],
                vec![
                    create_parameter("model_id", "Unique model identifier", "string", true, None),
                    create_parameter("model_data", "Model data or path", "string", true, None),
                    create_parameter("initial_traffic_percent", "Initial traffic percentage", "number", false, Some(serde_json::json!(10))),
                ],
                vec![
                    create_parameter("canary_strategy", "Canary deployment strategy", "string", false, Some(serde_json::json!("linear"))),
                ],
            ),
            create_tool_definition(
                "model_promote",
                "Promote a model version to receive more traffic",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess],
                vec![
                    create_parameter("model_id", "Model identifier", "string", true, None),
                    create_parameter("target_traffic_percent", "Target traffic percentage", "number", true, None),
                ],
                vec![
                    create_parameter("gradual_rollout", "Gradual rollout over time", "boolean", false, Some(serde_json::json!(true))),
                ],
            ),
            create_tool_definition(
                "model_rollback",
                "Rollback to a previous model version",
                ToolType::Utility,
                vec![ToolCapability::NetworkAccess],
                vec![
                    create_parameter("model_id", "Model identifier", "string", true, None),
                    create_parameter("target_version", "Version to rollback to", "string", true, None),
                ],
                vec![],
            ),
        ]
    }
}

/// Create a default service registry with common services
pub async fn create_default_service_registry(mcp_integration: Arc<MCPIntegration>) -> Result<ServiceRegistry, ServiceError> {
    let mut registry = ServiceRegistry::new(mcp_integration);

    // Register core services
    registry.register_service(Box::new(KnowledgeSeekerService)).await?;
    registry.register_service(Box::new(WebSearchService)).await?;
    registry.register_service(Box::new(FileSystemService)).await?;
    registry.register_service(Box::new(FederatedLearningService)).await?;
    registry.register_service(Box::new(ModelHotswapService)).await?;

    info!("Registered {} default services", registry.get_registered_services().len());
    Ok(registry)
}
