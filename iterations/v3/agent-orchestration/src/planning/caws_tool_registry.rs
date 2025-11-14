//! CAWS Tool Registry
//!
//! Provides dynamic CAWS tool discovery and registration via MCP protocol.
//! This component integrates with the MCP ToolRegistry to discover and manage
//! CAWS-compliant tools for use during adjudication cycles.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

#[cfg(feature = "mcp")]
use agent_mcp::{MCPTool, ToolCapability, ToolRegistry, ToolType};

/// CAWS-specific tool category for adjudication
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CawsToolCategory {
    /// Policy validation tools
    PolicyValidation,
    /// Quality gate tools
    QualityGates,
    /// Compliance checking tools
    ComplianceChecking,
    /// Evidence collection tools
    EvidenceCollection,
    /// Verification tools
    Verification,
    /// Reporting tools
    Reporting,
}

/// CAWS tool metadata
#[derive(Debug, Clone)]
pub struct CawsToolMetadata {
    /// Tool ID
    pub tool_id: Uuid,

    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// CAWS tool category
    pub category: CawsToolCategory,

    /// CAWS compliance status
    pub is_caws_compliant: bool,

    /// Tool capabilities relevant to CAWS
    pub caws_capabilities: Vec<CawsToolCapability>,

    /// Last verification timestamp
    pub last_verified: Option<chrono::DateTime<Utc>>,

    /// Usage count in adjudication cycles
    pub usage_count: u64,
}

/// CAWS-specific tool capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CawsToolCapability {
    /// Can validate working specs
    WorkingSpecValidation,
    /// Can check code quality
    CodeQualityCheck,
    /// Can verify test coverage
    TestCoverageVerification,
    /// Can check security compliance
    SecurityCompliance,
    /// Can validate contracts
    ContractValidation,
    /// Can extract claims
    ClaimExtraction,
    /// Can verify evidence
    EvidenceVerification,
    /// Can generate reports
    ReportGeneration,
}

/// CAWS Tool Registry for dynamic tool discovery
pub struct CawsToolRegistry {
    /// Underlying MCP tool registry
    #[cfg(feature = "mcp")]
    mcp_registry: Arc<ToolRegistry>,

    /// CAWS tool metadata cache
    caws_tools: Arc<tokio::sync::RwLock<HashMap<Uuid, CawsToolMetadata>>>,

    /// Tool category index
    category_index: Arc<tokio::sync::RwLock<HashMap<CawsToolCategory, Vec<Uuid>>>>,
}

impl CawsToolRegistry {
    /// Create a new CAWS tool registry
    #[cfg(feature = "mcp")]
    pub fn new(mcp_registry: Arc<ToolRegistry>) -> Self {
        Self {
            mcp_registry,
            caws_tools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            category_index: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Create a new CAWS tool registry without MCP (for testing)
    #[cfg(not(feature = "mcp"))]
    pub fn new() -> Self {
        Self {
            caws_tools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            category_index: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Discover and register CAWS-compliant tools from MCP registry
    #[cfg(feature = "mcp")]
    pub async fn discover_tools(&self) -> Result<usize> {
        info!("Discovering CAWS-compliant tools from MCP registry");

        // Get all tools from MCP registry
        let all_tools = self.mcp_registry.list_tools().await?;

        let mut discovered_count = 0;

        for tool in all_tools {
            // Check if tool is CAWS-relevant
            if self.is_caws_relevant(&tool) {
                let metadata = self.create_caws_metadata(&tool).await?;

                // Register in CAWS registry
                self.register_caws_tool(metadata.clone()).await?;

                discovered_count += 1;
                info!(
                    "Discovered CAWS tool: {} ({})",
                    metadata.name, metadata.tool_id
                );
            }
        }

        info!("Discovered {} CAWS-compliant tools", discovered_count);
        Ok(discovered_count)
    }

    /// Check if a tool is relevant for CAWS adjudication
    #[cfg(feature = "mcp")]
    fn is_caws_relevant(&self, tool: &MCPTool) -> bool {
        // Check tool type
        let relevant_type = matches!(
            tool.tool_type,
            ToolType::CodeAnalysis
                | ToolType::Testing
                | ToolType::Documentation
                | ToolType::Monitoring
                | ToolType::Utility
        );

        // Check capabilities
        let relevant_capabilities = tool.capabilities.iter().any(|cap| {
            matches!(
                cap,
                ToolCapability::CodeAnalysis
                    | ToolCapability::TestExecution
                    | ToolCapability::DocumentationGeneration
                    | ToolCapability::FileRead
                    | ToolCapability::TextProcessing
            )
        });

        // Check CAWS compliance status
        let is_compliant = matches!(
            tool.caws_compliance,
            agent_mcp::CawsComplianceStatus::Compliant
                | agent_mcp::CawsComplianceStatus::MinorViolations(_)
        );

        // Check tool name/description for CAWS keywords
        let has_caws_keywords = tool.name.to_lowercase().contains("caws")
            || tool.name.to_lowercase().contains("policy")
            || tool.name.to_lowercase().contains("compliance")
            || tool.name.to_lowercase().contains("quality")
            || tool.description.to_lowercase().contains("caws")
            || tool.description.to_lowercase().contains("compliance")
            || tool.description.to_lowercase().contains("validation");

        relevant_type || relevant_capabilities || is_compliant || has_caws_keywords
    }

    /// Create CAWS metadata from MCP tool
    #[cfg(feature = "mcp")]
    async fn create_caws_metadata(&self, tool: &MCPTool) -> Result<CawsToolMetadata> {
        // Determine category based on tool type and capabilities
        let category = self.determine_category(tool);

        // Extract CAWS capabilities
        let caws_capabilities = self.extract_caws_capabilities(tool);

        // Check CAWS compliance
        let is_compliant = matches!(
            tool.caws_compliance,
            agent_mcp::CawsComplianceStatus::Compliant
        );

        Ok(CawsToolMetadata {
            tool_id: tool.id,
            name: tool.name.clone(),
            description: tool.description.clone(),
            category,
            is_caws_compliant: is_compliant,
            caws_capabilities,
            last_verified: Some(Utc::now()),
            usage_count: tool.usage_count,
        })
    }

    /// Determine CAWS category from tool
    #[cfg(feature = "mcp")]
    fn determine_category(&self, tool: &MCPTool) -> CawsToolCategory {
        // Check tool name for category hints
        let name_lower = tool.name.to_lowercase();
        if name_lower.contains("policy") || name_lower.contains("validator") {
            return CawsToolCategory::PolicyValidation;
        }
        if name_lower.contains("quality") || name_lower.contains("gate") {
            return CawsToolCategory::QualityGates;
        }
        if name_lower.contains("compliance") || name_lower.contains("check") {
            return CawsToolCategory::ComplianceChecking;
        }
        if name_lower.contains("evidence") || name_lower.contains("collect") {
            return CawsToolCategory::EvidenceCollection;
        }
        if name_lower.contains("verify") || name_lower.contains("verification") {
            return CawsToolCategory::Verification;
        }
        if name_lower.contains("report") || name_lower.contains("generate") {
            return CawsToolCategory::Reporting;
        }

        // Default based on tool type
        match tool.tool_type {
            ToolType::CodeAnalysis => CawsToolCategory::QualityGates,
            ToolType::Testing => CawsToolCategory::Verification,
            ToolType::Documentation => CawsToolCategory::Reporting,
            _ => CawsToolCategory::ComplianceChecking,
        }
    }

    /// Extract CAWS capabilities from tool
    #[cfg(feature = "mcp")]
    fn extract_caws_capabilities(&self, tool: &MCPTool) -> Vec<CawsToolCapability> {
        let mut capabilities = Vec::new();

        // Check tool capabilities
        for cap in &tool.capabilities {
            match cap {
                ToolCapability::CodeAnalysis => {
                    capabilities.push(CawsToolCapability::CodeQualityCheck);
                }
                ToolCapability::TestExecution => {
                    capabilities.push(CawsToolCapability::TestCoverageVerification);
                }
                ToolCapability::DocumentationGeneration => {
                    capabilities.push(CawsToolCapability::ReportGeneration);
                }
                ToolCapability::FileRead | ToolCapability::TextProcessing => {
                    capabilities.push(CawsToolCapability::EvidenceVerification);
                }
                _ => {}
            }
        }

        // Check tool name/description for specific capabilities
        let name_lower = tool.name.to_lowercase();
        let desc_lower = tool.description.to_lowercase();

        if name_lower.contains("spec") || desc_lower.contains("working spec") {
            capabilities.push(CawsToolCapability::WorkingSpecValidation);
        }
        if name_lower.contains("contract") || desc_lower.contains("contract") {
            capabilities.push(CawsToolCapability::ContractValidation);
        }
        if name_lower.contains("claim") || desc_lower.contains("claim") {
            capabilities.push(CawsToolCapability::ClaimExtraction);
        }
        if name_lower.contains("security") || desc_lower.contains("security") {
            capabilities.push(CawsToolCapability::SecurityCompliance);
        }

        // Remove duplicates
        capabilities.sort();
        capabilities.dedup();

        capabilities
    }

    /// Register a CAWS tool
    pub async fn register_caws_tool(&self, metadata: CawsToolMetadata) -> Result<()> {
        let tool_id = metadata.tool_id;

        // Store metadata
        {
            let mut tools = self.caws_tools.write().await;
            tools.insert(tool_id, metadata.clone());
        }

        // Update category index
        {
            let mut index = self.category_index.write().await;
            index
                .entry(metadata.category.clone())
                .or_insert_with(Vec::new)
                .push(tool_id);
        }

        debug!("Registered CAWS tool: {} ({})", metadata.name, tool_id);
        Ok(())
    }

    /// Get tools by category
    pub async fn get_tools_by_category(
        &self,
        category: &CawsToolCategory,
    ) -> Vec<CawsToolMetadata> {
        let index = self.category_index.read().await;
        let tools = self.caws_tools.read().await;

        if let Some(tool_ids) = index.get(category) {
            tool_ids
                .iter()
                .filter_map(|id| tools.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get tools by capability
    pub async fn get_tools_by_capability(
        &self,
        capability: &CawsToolCapability,
    ) -> Vec<CawsToolMetadata> {
        let tools = self.caws_tools.read().await;

        tools
            .values()
            .filter(|metadata| metadata.caws_capabilities.contains(capability))
            .cloned()
            .collect()
    }

    /// Get a tool by ID
    pub async fn get_tool(&self, tool_id: &Uuid) -> Option<CawsToolMetadata> {
        let tools = self.caws_tools.read().await;
        tools.get(tool_id).cloned()
    }

    /// Increment usage count for a tool
    pub async fn increment_usage(&self, tool_id: &Uuid) {
        let mut tools = self.caws_tools.write().await;
        if let Some(metadata) = tools.get_mut(tool_id) {
            metadata.usage_count += 1;
        }
    }

    /// Get all registered CAWS tools
    pub async fn list_all_tools(&self) -> Vec<CawsToolMetadata> {
        let tools = self.caws_tools.read().await;
        tools.values().cloned().collect()
    }

    /// Get CAWS-compliant tools only
    pub async fn get_compliant_tools(&self) -> Vec<CawsToolMetadata> {
        let tools = self.caws_tools.read().await;
        tools
            .values()
            .filter(|metadata| metadata.is_caws_compliant)
            .cloned()
            .collect()
    }

    /// Invoke a CAWS tool for validation
    #[cfg(feature = "mcp")]
    pub async fn invoke_tool(
        &self,
        tool_id: &Uuid,
        parameters: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolInvocationResult> {
        use agent_mcp::{ExecutionPriority, ToolExecutionRequest};
        use chrono::Utc;

        // Create execution request
        let request = ToolExecutionRequest {
            id: Uuid::new_v4(),
            tool_id: *tool_id,
            parameters,
            context: None,
            priority: ExecutionPriority::Normal,
            timeout_seconds: Some(30), // 30 second timeout for validation tools
            created_at: Utc::now(),
            requested_by: Some("caws_adjudication_cycle".to_string()),
        };

        // Execute tool via MCP registry
        let result = self.mcp_registry.execute_tool(request).await?;

        // Convert to our result type
        Ok(ToolInvocationResult {
            tool_id: *tool_id,
            success: matches!(result.status, agent_mcp::ExecutionStatus::Completed),
            output: result.output,
            error: result.error,
            caws_compliant: result
                .caws_compliance_result
                .as_ref()
                .map(|r| matches!(r.status, agent_mcp::CawsComplianceStatus::Compliant))
                .unwrap_or(true), // Assume compliant if no compliance check
        })
    }
}

/// Result of tool invocation
#[derive(Debug, Clone)]
pub struct ToolInvocationResult {
    pub tool_id: Uuid,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub caws_compliant: bool,
}
