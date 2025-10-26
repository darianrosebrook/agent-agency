//! Filesystem-based tool discovery

use crate::mcp_types::*;
use super::core::{DiscoveryError, DiscoveryErrorType};
use anyhow::Result;
use glob;
use std::collections::HashSet;
use std::path::Path;

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
            if !Path::new(base_path).exists() {
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
                                            if let Some(max) = self.config.max_tools {
                                                if tools.len() >= max {
                                                    break;
                                                }
                                            }
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
            id: format!("tool_{}", path.file_stem().unwrap_or_default().to_string_lossy()),
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
            schema: ToolSchema {
                input_schema: manifest.get("input_schema")
                    .unwrap_or(&serde_json::json!({}))
                    .clone(),
                output_schema: manifest.get("output_schema")
                    .unwrap_or(&serde_json::json!({}))
                    .clone(),
            },
            capabilities: manifest.get("capabilities")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            metadata: manifest.get("metadata")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
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
            id: manifest.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
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
            schema: ToolSchema {
                input_schema: manifest.get("schema")
                    .and_then(|s| s.get("input"))
                    .unwrap_or(&serde_json::json!({}))
                    .clone(),
                output_schema: manifest.get("schema")
                    .and_then(|s| s.get("output"))
                    .unwrap_or(&serde_json::json!({}))
                    .clone(),
            },
            capabilities: manifest.get("capabilities")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            metadata: manifest.get("metadata")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
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
