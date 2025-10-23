//! CAWS Integration
//!
//! DEPRECATION NOTICE: CawsIntegration migrating to caws-runtime-validator
//! See: iterations/v3/caws/runtime-validator/src/integration.rs
//! TODO: Remove after migration complete (target: Phase 2.2)
//!
//! Integrates CAWS compliance checking with MCP tools and execution.

// Re-export from runtime-validator (primary implementation)
pub use caws_runtime_validator::integration::{
    McpIntegration, DefaultMcpIntegration, McpValidationResult,
    ToolExecutionContext, ToolExecutionRecord, McpIntegrationError, McpCawsIntegration
};

use crate::types::*;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// CAWS integration service
#[deprecated(note = "Use caws_runtime_validator::integration::McpCawsIntegration")]
#[derive(Debug)]
pub struct CawsIntegration {
    // DEPRECATED: Wrapper around runtime-validator implementation
    inner: McpCawsIntegration,
    // Keep config and cache for backward compatibility
    pub(crate) config: CawsIntegrationConfig,
    pub(crate) rulebook: Arc<RwLock<CawsRulebook>>,
    pub(crate) compliance_cache: Arc<RwLock<std::collections::HashMap<Uuid, CawsComplianceResult>>>,
}

#[derive(Debug, Clone)]
pub struct CawsRulebook {
    pub version: String,
    pub rules: Vec<CawsRule>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct CawsRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: ViolationSeverity,
    pub category: RuleCategory,
    pub validation_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleCategory {
    CodeQuality,
    Security,
    Performance,
    Documentation,
    Testing,
    Deployment,
    Governance,
}

impl CawsIntegration {
    /// Create a new CAWS integration service
    pub fn new() -> Self {
        Self {
            // DEPRECATED: Use runtime-validator implementation
            inner: McpCawsIntegration::new(),
            config: CawsIntegrationConfig::default(),
            rulebook: Arc::new(RwLock::new(CawsRulebook {
                version: "1.0.0".to_string(),
                rules: Vec::new(),
                last_updated: chrono::Utc::now(),
            })),
            compliance_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Initialize CAWS integration
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing CAWS integration");

        // Clear compliance cache
        self.clear_compliance_cache().await;

        // Load rulebook from configured path if it exists
        let path = self.config.caws_rulebook_path.clone();
        let path_exists = std::path::Path::new(&path).exists();

        if path_exists {
            info!("Loading CAWS rulebook from: {}", path);
            self.load_rulebook(&path).await?;
        } else {
            info!(
                "CAWS rulebook path does not exist: {} - using default empty rulebook",
                path
            );
            // Initialize with empty rulebook - this is fine for testing/development
            let mut rb = self.rulebook.write().await;
            *rb = CawsRulebook {
                version: "default".to_string(),
                rules: Vec::new(),
                last_updated: chrono::Utc::now(),
            };
        }

        Ok(())
    }

    /// Calculate compliance score from violations using consistent scoring logic
    fn calculate_compliance_score(
        &self,
        violations: &[crate::types::CawsViolation],
        rulebook: &CawsRulebook,
    ) -> (f32, bool) {
        let strict = matches!(
            self.config.validation_strictness,
            crate::types::ValidationStrictness::Strict
        );

        let mut penalty: f32 = 0.0;
        for v in violations {
            penalty += match v.severity {
                crate::types::ViolationSeverity::Info => 0.02,
                crate::types::ViolationSeverity::Warning => 0.05,
                crate::types::ViolationSeverity::Error => 0.2,
                crate::types::ViolationSeverity::Critical => 0.5,
            };
        }

        // Consider rulebook size for normalization (avoid 0 rules edge case)
        let base: f32 = if rulebook.rules.is_empty() {
            1.0
        } else {
            rulebook.rules.len() as f32 * 0.05
        };
        let compliance_score = (1.0 - (penalty / (1.0 + base))).clamp(0.0, 1.0);

        let is_compliant = if strict {
            violations.is_empty()
        } else {
            // Moderate/Lenient allow warnings/errors depending on score
            compliance_score > 0.6
        };

        (compliance_score, is_compliant)
    }

    /// Validate tool for CAWS compliance
    #[deprecated(note = "Use caws_runtime_validator::integration::McpCawsIntegration::validate_tool_manifest")]
    pub async fn validate_tool(&self, tool: &MCPTool) -> Result<CawsComplianceResult> {
        info!("Validating tool for CAWS compliance: {}", tool.name);

        // DEPRECATED: Delegate to runtime-validator implementation
        let runtime_result = self.inner.validate_tool_manifest(&tool.manifest, "2").await
            .map_err(|e| anyhow::anyhow!("Runtime validator error: {}", e))?;
        
        // Convert runtime-validator result to legacy format
        let violations: Vec<crate::types::CawsViolation> = runtime_result.violations
            .into_iter()
            .map(|v| crate::types::CawsViolation {
                rule_id: "RUNTIME-VALIDATOR".to_string(),
                rule_name: "Runtime Validator Check".to_string(),
                severity: if runtime_result.compliant { 
                    crate::types::ViolationSeverity::Info 
                } else { 
                    crate::types::ViolationSeverity::Error 
                },
                description: v,
                suggestion: None,
                line_number: None,
                column_number: None,
                file_path: None,
            })
            .collect();
        
        let compliance_score = if runtime_result.compliant { 1.0 } else { 0.5 };
        
        let result = CawsComplianceResult {
            tool_id: tool.id,
            compliant: runtime_result.compliant,
            compliance_score,
            violations,
            recommendations: runtime_result.recommendations,
            risk_assessment: runtime_result.risk_assessment,
            validated_at: chrono::Utc::now(),
        };

        // Cache the result
        {
            let mut cache = self.compliance_cache.write().await;
            cache.insert(tool.id, result.clone());
        }

        Ok(result)
    }

    /// Validate tool execution for CAWS compliance
    pub async fn validate_tool_execution(
        &self,
        tool: &MCPTool,
        request: &ToolExecutionRequest,
    ) -> Result<CawsComplianceResult> {
        info!(
            "Validating tool execution for CAWS compliance: {}",
            tool.name
        );

        // Execution-specific validation: timeout presence for network/command tools
        let mut exec_violations: Vec<crate::types::CawsViolation> = Vec::new();
        if (tool
            .capabilities
            .contains(&crate::types::ToolCapability::NetworkAccess)
            || tool
                .capabilities
                .contains(&crate::types::ToolCapability::CommandExecution))
            && request.timeout_seconds.is_none()
        {
            exec_violations.push(crate::types::CawsViolation {
                rule_id: "RUNTIME-001".into(),
                rule_name: "Dangerous capability requires timeout".into(),
                severity: crate::types::ViolationSeverity::Error,
                description: "Execution missing timeout for network/command tool".into(),
                suggestion: Some("Set an appropriate timeout_seconds".into()),
                line_number: None,
                column_number: None,
                file_path: None,
            });
        }
        // Use shared scoring logic for consistency
        let rb = self.rulebook.read().await.clone();
        let (compliance_score, is_compliant) =
            self.calculate_compliance_score(&exec_violations, &rb);

        Ok(CawsComplianceResult {
            is_compliant,
            violations: exec_violations,
            compliance_score,
            checked_at: chrono::Utc::now(),
            rulebook_version: rb.version,
        })
    }

    /// Load CAWS rulebook
    pub async fn load_rulebook(&self, rulebook_path: &str) -> Result<()> {
        info!("Loading CAWS rulebook from: {}", rulebook_path);

        // Load a minimal rulebook from a JSON or YAML file.
        // Supported minimal schema: { version: string, rules: [{id,name,description,severity,category}] }

        let p = Path::new(rulebook_path);
        if !p.exists() {
            anyhow::bail!("Rulebook path does not exist: {}", rulebook_path);
        }
        let content = fs::read_to_string(p)?;
        // Try JSON first, then YAML
        #[derive(serde::Deserialize)]
        struct RawRulebook {
            version: String,
            rules: Vec<RawRule>,
        }
        #[derive(serde::Deserialize)]
        struct RawRule {
            id: String,
            name: String,
            description: String,
            severity: String,
            category: String,
        }

        let parsed: RawRulebook = if let Ok(j) = serde_json::from_str(&content) {
            j
        } else {
            serde_yaml::from_str(&content)?
        };

        fn map_sev(s: &str) -> crate::types::ViolationSeverity {
            match s.to_lowercase().as_str() {
                "info" => crate::types::ViolationSeverity::Info,
                "warning" | "warn" => crate::types::ViolationSeverity::Warning,
                "error" => crate::types::ViolationSeverity::Error,
                "critical" => crate::types::ViolationSeverity::Critical,
                _ => crate::types::ViolationSeverity::Warning,
            }
        }
        fn map_cat(s: &str) -> RuleCategory {
            match s.to_lowercase().as_str() {
                "security" => RuleCategory::Security,
                "performance" => RuleCategory::Performance,
                "documentation" => RuleCategory::Documentation,
                "testing" => RuleCategory::Testing,
                "deployment" => RuleCategory::Deployment,
                "governance" => RuleCategory::Governance,
                _ => RuleCategory::CodeQuality,
            }
        }

        let rules = parsed
            .rules
            .into_iter()
            .map(|r| CawsRule {
                id: r.id,
                name: r.name,
                description: r.description,
                severity: map_sev(&r.severity),
                category: map_cat(&r.category),
                validation_function: String::new(),
            })
            .collect::<Vec<_>>();

        let mut rb = self.rulebook.write().await;
        *rb = CawsRulebook {
            version: parsed.version,
            rules,
            last_updated: chrono::Utc::now(),
        };
        Ok(())
    }

    /// Get CAWS rulebook
    pub async fn get_rulebook(&self) -> CawsRulebook {
        let rulebook = self.rulebook.read().await;
        rulebook.clone()
    }

    /// Clear compliance cache
    pub async fn clear_compliance_cache(&self) {
        let mut cache = self.compliance_cache.write().await;
        cache.clear();
        info!("CAWS compliance cache cleared");
    }

    /// Shutdown CAWS integration
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down CAWS integration");

        // Idempotent: just clear caches and leave
        self.clear_compliance_cache().await;
        Ok(())
    }
}

impl Default for CawsIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_caws_checking: true,
            caws_rulebook_path: "./caws".to_string(),
            enable_provenance: true,
            enable_quality_gates: true,
            validation_strictness: ValidationStrictness::Moderate,
        }
    }
}
