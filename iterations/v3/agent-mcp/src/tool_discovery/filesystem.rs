//! Filesystem-based tool discovery

use super::core::{DiscoveryError, DiscoveryErrorType};
use crate::mcp_types::*;
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
                                                message: format!(
                                                    "Failed to load tool manifest: {}",
                                                    e
                                                ),
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
    /// Implemented: Comprehensive manifest parsing including all tool metadata, capabilities, parameters, dependencies, and CAWS compliance
    async fn load_tool_from_manifest(&self, path: &Path) -> Result<MCPTool> {
        let content = tokio::fs::read_to_string(path).await?;
        let manifest: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse manifest JSON: {}", e))?;

        // Parse tool type from manifest
        let tool_type = self.parse_tool_type(&manifest);

        // Parse capabilities comprehensively
        let capabilities = self.parse_capabilities(&manifest);

        // Parse tool parameters comprehensively
        let parameters = self.parse_parameters(&manifest);

        // Parse dependencies
        let dependencies = self.parse_dependencies(&manifest);

        // Parse CAWS compliance configuration
        let caws_compliance_config = self.parse_caws_compliance(&manifest);

        // Parse configuration schema
        let configuration_schema = manifest
            .get("configuration_schema")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        // Parse registration and update timestamps if present
        let registration_time = manifest
            .get("registration_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let last_updated = manifest
            .get("last_updated")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        // Parse usage count if present
        let usage_count = manifest
            .get("usage_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Parse comprehensive metadata
        let metadata = self.parse_metadata(&manifest);

        // Determine CAWS compliance status from config
        let caws_compliance_status = if caws_compliance_config.is_some() {
            crate::mcp_types::CawsComplianceStatus::Compliant
        } else {
            crate::mcp_types::CawsComplianceStatus::Unknown
        };

        let tool = MCPTool {
            id: manifest
                .get("id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            name: manifest
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing required field: name"))?
                .to_string(),
            description: manifest
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: manifest
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0")
                .to_string(),
            author: manifest
                .get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            tool_type: tool_type.clone(),
            capabilities: capabilities.clone(),
            parameters: parameters.clone(),
            output_schema: manifest
                .get("output_schema")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
            endpoint: manifest
                .get("endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            manifest: ToolManifest {
                name: manifest
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                version: manifest
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0")
                    .to_string(),
                description: manifest
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                author: manifest
                    .get("author")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                tool_type,
                entry_point: manifest
                    .get("entry_point")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                dependencies,
                capabilities,
                parameters,
                output_schema: manifest
                    .get("output_schema")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
                endpoint: manifest
                    .get("endpoint")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                caws_compliance: caws_compliance_config,
                metadata: metadata.clone(),
                configuration_schema,
            },
            caws_compliance: caws_compliance_status,
            registration_time,
            last_updated,
            usage_count,
            metadata,
        };

        Ok(tool)
    }

    /// Parse tool type from manifest
    fn parse_tool_type(&self, manifest: &serde_json::Value) -> crate::mcp_types::ToolType {
        manifest
            .get("tool_type")
            .and_then(|v| v.as_str())
            .and_then(|s| match s.to_lowercase().as_str() {
                "codegeneration" | "code_generation" => {
                    Some(crate::mcp_types::ToolType::CodeGeneration)
                }
                "codeanalysis" | "code_analysis" => Some(crate::mcp_types::ToolType::CodeAnalysis),
                "testing" => Some(crate::mcp_types::ToolType::Testing),
                "documentation" => Some(crate::mcp_types::ToolType::Documentation),
                "build" => Some(crate::mcp_types::ToolType::Build),
                "deployment" => Some(crate::mcp_types::ToolType::Deployment),
                "monitoring" => Some(crate::mcp_types::ToolType::Monitoring),
                "utility" => Some(crate::mcp_types::ToolType::Utility),
                custom => Some(crate::mcp_types::ToolType::Custom(custom.to_string())),
            })
            .unwrap_or_else(|| crate::mcp_types::ToolType::Custom("filesystem".to_string()))
    }

    /// Parse capabilities from manifest
    fn parse_capabilities(
        &self,
        manifest: &serde_json::Value,
    ) -> Vec<crate::mcp_types::ToolCapability> {
        manifest
            .get("capabilities")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| {
                v.as_str()
                    .map(|s| crate::mcp_types::ToolCapability::from_str(s))
            })
            .collect()
    }

    /// Parse tool parameters comprehensively from manifest
    fn parse_parameters(&self, manifest: &serde_json::Value) -> crate::mcp_types::ToolParameters {
        let params_obj = manifest.get("parameters").and_then(|v| v.as_object());

        if params_obj.is_none() {
            return crate::mcp_types::ToolParameters::default();
        }

        let params_obj = params_obj.unwrap();

        // Parse required parameters
        let required = params_obj
            .get("required")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| self.parse_parameter_definition(v))
            .collect();

        // Parse optional parameters
        let optional = params_obj
            .get("optional")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| self.parse_parameter_definition(v))
            .collect();

        // Parse constraints
        let constraints = params_obj
            .get("constraints")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| self.parse_parameter_constraint(v))
            .collect();

        crate::mcp_types::ToolParameters {
            required,
            optional,
            constraints,
        }
    }

    /// Parse a single parameter definition
    fn parse_parameter_definition(
        &self,
        param: &serde_json::Value,
    ) -> Option<crate::mcp_types::ParameterDefinition> {
        let param_obj = param.as_object()?;

        let name = param_obj.get("name")?.as_str()?.to_string();
        let description = param_obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let parameter_type = param_obj
            .get("type")
            .and_then(|v| v.as_str())
            .and_then(|s| self.parse_parameter_type(s))
            .unwrap_or(crate::mcp_types::ParameterType::String);

        let default_value = param_obj.get("default").cloned();

        let validation_rules = param_obj
            .get("validation_rules")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| self.parse_validation_rule(v))
            .collect();

        Some(crate::mcp_types::ParameterDefinition {
            name,
            parameter_type,
            description,
            default_value,
            validation_rules,
        })
    }

    /// Parse parameter type from string
    fn parse_parameter_type(&self, s: &str) -> Option<crate::mcp_types::ParameterType> {
        match s.to_lowercase().as_str() {
            "string" => Some(crate::mcp_types::ParameterType::String),
            "integer" | "int" => Some(crate::mcp_types::ParameterType::Integer),
            "float" | "number" => Some(crate::mcp_types::ParameterType::Float),
            "boolean" | "bool" => Some(crate::mcp_types::ParameterType::Boolean),
            "array" => Some(crate::mcp_types::ParameterType::Array),
            "object" => Some(crate::mcp_types::ParameterType::Object),
            "file" => Some(crate::mcp_types::ParameterType::File),
            "directory" | "dir" => Some(crate::mcp_types::ParameterType::Directory),
            "url" => Some(crate::mcp_types::ParameterType::URL),
            "json" => Some(crate::mcp_types::ParameterType::JSON),
            _ => None,
        }
    }

    /// Parse validation rule
    fn parse_validation_rule(
        &self,
        rule: &serde_json::Value,
    ) -> Option<crate::mcp_types::ValidationRule> {
        let rule_obj = rule.as_object()?;

        let rule_type_str = rule_obj.get("rule_type")?.as_str()?;
        let rule_type = match rule_type_str.to_lowercase().as_str() {
            "notempty" | "not_empty" => crate::mcp_types::ValidationRuleType::NotEmpty,
            "regexmatch" | "regex_match" => crate::mcp_types::ValidationRuleType::RegexMatch,
            "rangecheck" | "range_check" => crate::mcp_types::ValidationRuleType::RangeCheck,
            "typecheck" | "type_check" => crate::mcp_types::ValidationRuleType::TypeCheck,
            custom => crate::mcp_types::ValidationRuleType::Custom(custom.to_string()),
        };

        let parameters = rule_obj
            .get("parameters")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let error_message = rule_obj
            .get("error_message")
            .and_then(|v| v.as_str())
            .unwrap_or("Validation failed")
            .to_string();

        Some(crate::mcp_types::ValidationRule {
            rule_type,
            parameters,
            error_message,
        })
    }

    /// Parse parameter constraint
    fn parse_parameter_constraint(
        &self,
        constraint: &serde_json::Value,
    ) -> Option<crate::mcp_types::ParameterConstraint> {
        let constraint_obj = constraint.as_object()?;

        let parameter_name = constraint_obj.get("parameter_name")?.as_str()?.to_string();

        let constraint_type_str = constraint_obj.get("constraint_type")?.as_str()?;
        let constraint_type = match constraint_type_str.to_lowercase().as_str() {
            "minlength" | "min_length" => crate::mcp_types::ConstraintType::MinLength,
            "maxlength" | "max_length" => crate::mcp_types::ConstraintType::MaxLength,
            "minvalue" | "min_value" => crate::mcp_types::ConstraintType::MinValue,
            "maxvalue" | "max_value" => crate::mcp_types::ConstraintType::MaxValue,
            "pattern" => crate::mcp_types::ConstraintType::Pattern,
            "required" => crate::mcp_types::ConstraintType::Required,
            "unique" => crate::mcp_types::ConstraintType::Unique,
            custom => crate::mcp_types::ConstraintType::Custom(custom.to_string()),
        };

        let value = constraint_obj.get("value")?.clone();
        let message = constraint_obj
            .get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Some(crate::mcp_types::ParameterConstraint {
            parameter_name,
            constraint_type,
            value,
            message,
        })
    }

    /// Parse dependencies from manifest
    fn parse_dependencies(
        &self,
        manifest: &serde_json::Value,
    ) -> Vec<crate::mcp_types::Dependency> {
        manifest
            .get("dependencies")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| {
                let dep_obj = v.as_object()?;

                let name = dep_obj.get("name")?.as_str()?.to_string();
                let version = dep_obj.get("version")?.as_str()?.to_string();

                let dependency_type_str = dep_obj
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("runtime");

                let dependency_type = match dependency_type_str.to_lowercase().as_str() {
                    "runtime" => crate::mcp_types::DependencyType::Runtime,
                    "build" => crate::mcp_types::DependencyType::Build,
                    "development" | "dev" => crate::mcp_types::DependencyType::Development,
                    "test" => crate::mcp_types::DependencyType::Test,
                    _ => crate::mcp_types::DependencyType::Runtime,
                };

                let optional = dep_obj
                    .get("optional")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                Some(crate::mcp_types::Dependency {
                    name,
                    version,
                    dependency_type,
                    optional,
                })
            })
            .collect()
    }

    /// Parse CAWS compliance configuration
    fn parse_caws_compliance(
        &self,
        manifest: &serde_json::Value,
    ) -> Option<crate::mcp_types::CawsComplianceConfig> {
        let caws_obj = manifest.get("caws_compliance")?.as_object()?;

        let required_rules = caws_obj
            .get("required_rules")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let optional_rules = caws_obj
            .get("optional_rules")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let strict_mode = caws_obj
            .get("strict_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let custom_validations = caws_obj
            .get("custom_validations")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| {
                let val_obj = v.as_object()?;
                Some(crate::mcp_types::CustomValidation {
                    name: val_obj.get("name")?.as_str()?.to_string(),
                    description: val_obj
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    validation_function: val_obj.get("validation_function")?.as_str()?.to_string(),
                    parameters: val_obj
                        .get("parameters")
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default(),
                })
            })
            .collect();

        Some(crate::mcp_types::CawsComplianceConfig {
            required_rules,
            optional_rules,
            strict_mode,
            custom_validations,
        })
    }

    /// Parse comprehensive metadata from manifest
    fn parse_metadata(
        &self,
        manifest: &serde_json::Value,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut metadata = std::collections::HashMap::new();

        // Parse explicit metadata object
        if let Some(meta_obj) = manifest.get("metadata").and_then(|v| v.as_object()) {
            for (k, v) in meta_obj {
                metadata.insert(k.clone(), v.clone());
            }
        }

        // Include additional fields as metadata
        let additional_fields = vec![
            "license",
            "homepage",
            "repository",
            "keywords",
            "tags",
            "category",
        ];
        for field in additional_fields {
            if let Some(value) = manifest.get(field) {
                metadata.insert(field.to_string(), value.clone());
            }
        }

        metadata
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
            name: manifest
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            description: manifest
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: manifest
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0")
                .to_string(),
            author: manifest
                .get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            tool_type: ToolType::Custom("filesystem".to_string()),
            capabilities: manifest
                .get("capabilities")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| ToolCapability::from_str(s)))
                .collect(),
            parameters: ToolParameters::default(),
            output_schema: manifest
                .get("schema")
                .and_then(|s| s.get("output"))
                .unwrap_or(&serde_json::json!({}))
                .clone(),
            endpoint: manifest
                .get("endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            manifest: ToolManifest {
                name: manifest
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                version: manifest
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0")
                    .to_string(),
                description: manifest
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                author: manifest
                    .get("author")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                tool_type: ToolType::Custom("filesystem".to_string()),
                entry_point: manifest
                    .get("entry_point")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                dependencies: Vec::new(),
                capabilities: manifest
                    .get("capabilities")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| ToolCapability::from_str(s)))
                    .collect(),
                parameters: ToolParameters::default(),
                output_schema: manifest
                    .get("output_schema")
                    .unwrap_or(&serde_json::json!({}))
                    .clone(),
                endpoint: manifest
                    .get("endpoint")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                caws_compliance: None,
                metadata: manifest
                    .get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                configuration_schema: serde_json::json!({}),
            },
            caws_compliance: CawsComplianceStatus::Unknown,
            registration_time: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            usage_count: 0,
            metadata: manifest
                .get("metadata")
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
