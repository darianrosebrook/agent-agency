//! Filesystem-based tool discovery

use crate::mcp_types::*;
use super::core::{DiscoveryError, DiscoveryErrorType};
use anyhow::Result;
use glob;
use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

/// Filesystem scanner for tool manifests
pub struct FilesystemScanner {
    config: ToolDiscoveryConfig,
}

impl FilesystemScanner {
    pub fn new(config: ToolDiscoveryConfig) -> Self {
        Self { config }
    }

    /// Scan filesystem for tool manifests
    pub async fn scan_manifests(&self) -> Result<(Vec<MCPTool>, Vec<DiscoveryError>)> {
        let mut tools = Vec::new();
        let mut errors = Vec::new();
        let mut seen_paths = HashSet::new();

        for base_path in &self.config.discovery_paths {
            if !Path::new(base_path as &str).exists() {
                continue;
            }

            for pattern in &self.config.manifest_patterns {
                let full_pattern = format!("{}/{}", base_path.trim_end_matches('/'), pattern);

                match glob::glob(&full_pattern) {
                    Ok(paths) => {
                        for entry in paths {
                            match entry {
                                Ok(path) => {
                                    let path_str = path.to_string_lossy().to_string();

                                    if seen_paths.contains(&path_str) {
                                        continue; // Skip duplicates
                                    }
                                    seen_paths.insert(path_str.clone());

                                    match self.load_tool_from_manifest(&path).await {
                                        Ok(tool) => {
                                            tools.push(tool);
                                        }
                                        Err(e) => {
                                            errors.push(DiscoveryError {
                                                path: path_str,
                                                error_type: DiscoveryErrorType::InvalidManifest,
                                                message: format!("Failed to load tool manifest: {}", e),
                                                details: Some(e.to_string()),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    errors.push(DiscoveryError {
                                        path: full_pattern.clone(),
                                        error_type: DiscoveryErrorType::Unknown,
                                        message: format!("Glob pattern error: {}", e),
                                        details: None,
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(DiscoveryError {
                            path: full_pattern,
                            error_type: DiscoveryErrorType::Unknown,
                            message: format!("Invalid glob pattern: {}", e),
                            details: None,
                        });
                    }
                }
            }
        }

        Ok((tools, errors))
    }

    /// Load a tool from a manifest file
    async fn load_tool_from_manifest(&self, path: &Path) -> Result<MCPTool> {
        let content = tokio::fs::read_to_string(path).await?;
        let manifest: serde_json::Value = serde_json::from_str(&content)?;

        // Parse the manifest into an MCPTool
        // This is a simplified implementation - real implementation would be more complex
        let tool = MCPTool {
            id: Uuid::new_v4(),
            name: manifest.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            description: manifest.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: manifest.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0")
                .to_string(),
            author: manifest.get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            tool_type: ToolType::Custom("filesystem".to_string()),
            capabilities: manifest.get("capabilities")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| ToolCapability::from_str(s)))
                .collect(),
            parameters: ToolParameters::default(),
            output_schema: manifest.get("output_schema")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
            endpoint: manifest.get("endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            manifest: ToolManifest {
                name: manifest.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                version: manifest.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0")
                    .to_string(),
                description: manifest.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                author: manifest.get("author")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                tool_type: ToolType::Custom("filesystem".to_string()),
                entry_point: manifest.get("entry_point")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                dependencies: Vec::new(),
                capabilities: manifest.get("capabilities")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| ToolCapability::from_str(s)))
                    .collect(),
                parameters: ToolParameters::default(),
                output_schema: manifest.get("output_schema")
                    .unwrap_or(&serde_json::json!({}))
                    .clone(),
                endpoint: manifest.get("endpoint")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                caws_compliance: None,
                metadata: manifest.get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                configuration_schema: serde_json::json!({}),
            },
            caws_compliance: CawsComplianceStatus::Unknown,
            registration_time: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            usage_count: 0,
            metadata: manifest.get("metadata")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
        };

        Ok(tool)
    }
}

/// Tool manifest loader
pub struct ManifestLoader;

impl ManifestLoader {
    pub fn new() -> Self {
        Self
    }

    /// Load tool from JSON manifest
    pub fn load_from_json(&self, content: &str) -> Result<MCPTool> {
        let manifest: serde_json::Value = serde_json::from_str(content)?;

        let tool = MCPTool {
            id: Uuid::new_v4(),
            name: manifest.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            description: manifest.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: manifest.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0")
                .to_string(),
            author: manifest.get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            tool_type: ToolType::Custom("filesystem".to_string()),
            capabilities: manifest.get("capabilities")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| ToolCapability::from_str(s)))
                .collect(),
            parameters: ToolParameters::default(),
            output_schema: manifest.get("schema")
                .and_then(|s| s.get("output"))
                .unwrap_or(&serde_json::json!({}))
                .clone(),
            endpoint: manifest.get("endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            manifest: ToolManifest {
                name: manifest.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                version: manifest.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0")
                    .to_string(),
                description: manifest.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                author: manifest.get("author")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                tool_type: ToolType::Custom("filesystem".to_string()),
                entry_point: manifest.get("entry_point")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                dependencies: Vec::new(),
                capabilities: manifest.get("capabilities")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| ToolCapability::from_str(s)))
                    .collect(),
                parameters: ToolParameters::default(),
                output_schema: manifest.get("output_schema")
                    .unwrap_or(&serde_json::json!({}))
                    .clone(),
                endpoint: manifest.get("endpoint")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                caws_compliance: None,
                metadata: manifest.get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                configuration_schema: serde_json::json!({}),
            },
            caws_compliance: CawsComplianceStatus::Unknown,
            registration_time: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            usage_count: 0,
            metadata: manifest.get("metadata")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
        };

        Ok(tool)
    }

    /// Validate manifest format
    pub fn validate_manifest(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();

        match serde_json::from_str::<serde_json::Value>(content) {
            Ok(manifest) => {
                if !manifest.is_object() {
                    errors.push("Manifest must be a JSON object".to_string());
                    return errors;
                }

                let obj = manifest.as_object().unwrap();

                if !obj.contains_key("name") {
                    errors.push("Manifest missing required 'name' field".to_string());
                }

                if !obj.contains_key("schema") {
                    errors.push("Manifest missing required 'schema' field".to_string());
                }
            }
            Err(e) => {
                errors.push(format!("Invalid JSON: {}", e));
            }
        }

        errors
    }
}
