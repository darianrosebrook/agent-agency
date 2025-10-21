//! CAWS Policy Management
//!
//! Consolidated policy definitions and validation logic
//! extracted from scattered implementations across the codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

/// Unified CAWS Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsPolicy {
    /// Risk tier configuration
    pub risk_tiers: HashMap<String, RiskTierConfig>,
    /// Budget limits by tier
    pub budget_limits: HashMap<String, BudgetLimits>,
    /// Validation rules
    pub validation_rules: Vec<ValidationRule>,
    /// Waiver policies
    pub waiver_policies: WaiverPolicies,
    /// Integration settings
    pub integrations: IntegrationSettings,
}

/// Risk tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTierConfig {
    pub name: String,
    pub level: u8,
    pub requires_review: bool,
    pub max_budget_multiplier: f64,
    pub mandatory_checks: Vec<String>,
}

/// Budget limits for different resource types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub max_files: u32,
    pub max_loc: u32,
    pub max_time_seconds: u64,
    pub max_memory_mb: u64,
}

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: ViolationSeverity,
    pub category: RuleCategory,
    pub enabled: bool,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Rule categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleCategory {
    Budget,
    Security,
    Quality,
    Compliance,
    Performance,
}

/// Waiver policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverPolicies {
    pub allow_budget_overruns: bool,
    pub max_waiver_duration_days: u32,
    pub require_approval_for: Vec<String>,
}

/// Integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSettings {
    pub mcp_enabled: bool,
    pub orchestration_enabled: bool,
    pub provenance_enabled: bool,
}

/// Policy validator - consolidated validation logic
pub struct PolicyValidator;

impl PolicyValidator {
    /// Validate a complete policy configuration
    pub fn validate_policy(policy: &CawsPolicy) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate risk tiers
        for (tier_name, config) in &policy.risk_tiers {
            if config.level == 0 || config.level > 3 {
                errors.push(format!("Invalid risk level {} for tier {}", config.level, tier_name));
            }
        }

        // Validate budget limits
        for (tier_name, limits) in &policy.budget_limits {
            if limits.max_files == 0 || limits.max_loc == 0 {
                errors.push(format!("Invalid budget limits for tier {}", tier_name));
            }
        }

        // Validate rules
        for rule in &policy.validation_rules {
            if rule.id.is_empty() || rule.name.is_empty() {
                errors.push(format!("Invalid rule: missing id or name"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Load policy from file
    pub fn load_from_file(path: &PathBuf) -> Result<CawsPolicy> {
        let content = std::fs::read_to_string(path)?;
        let policy: CawsPolicy = serde_yaml::from_str(&content)?;
        Ok(policy)
    }

    /// Save policy to file
    pub fn save_to_file(policy: &CawsPolicy, path: &PathBuf) -> Result<()> {
        let content = serde_yaml::to_string(policy)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for CawsPolicy {
    fn default() -> Self {
        let mut risk_tiers = HashMap::new();
        risk_tiers.insert("low".to_string(), RiskTierConfig {
            name: "Low Risk".to_string(),
            level: 1,
            requires_review: false,
            max_budget_multiplier: 1.0,
            mandatory_checks: vec!["syntax".to_string()],
        });
        risk_tiers.insert("medium".to_string(), RiskTierConfig {
            name: "Medium Risk".to_string(),
            level: 2,
            requires_review: false,
            max_budget_multiplier: 2.0,
            mandatory_checks: vec!["syntax".to_string(), "tests".to_string()],
        });
        risk_tiers.insert("high".to_string(), RiskTierConfig {
            name: "High Risk".to_string(),
            level: 3,
            requires_review: true,
            max_budget_multiplier: 3.0,
            mandatory_checks: vec!["syntax".to_string(), "tests".to_string(), "security".to_string()],
        });

        let mut budget_limits = HashMap::new();
        budget_limits.insert("low".to_string(), BudgetLimits {
            max_files: 10,
            max_loc: 500,
            max_time_seconds: 300,
            max_memory_mb: 512,
        });
        budget_limits.insert("medium".to_string(), BudgetLimits {
            max_files: 25,
            max_loc: 1000,
            max_time_seconds: 600,
            max_memory_mb: 1024,
        });
        budget_limits.insert("high".to_string(), BudgetLimits {
            max_files: 50,
            max_loc: 2000,
            max_time_seconds: 1800,
            max_memory_mb: 2048,
        });

        Self {
            risk_tiers,
            budget_limits,
            validation_rules: vec![
                ValidationRule {
                    id: "syntax-check".to_string(),
                    name: "Syntax Validation".to_string(),
                    description: "Ensure code has valid syntax".to_string(),
                    severity: ViolationSeverity::Error,
                    category: RuleCategory::Quality,
                    enabled: true,
                },
                ValidationRule {
                    id: "test-coverage".to_string(),
                    name: "Test Coverage".to_string(),
                    description: "Ensure adequate test coverage".to_string(),
                    severity: ViolationSeverity::Warning,
                    category: RuleCategory::Quality,
                    enabled: true,
                },
            ],
            waiver_policies: WaiverPolicies {
                allow_budget_overruns: false,
                max_waiver_duration_days: 7,
                require_approval_for: vec!["budget-overrun".to_string()],
            },
            integrations: IntegrationSettings {
                mcp_enabled: true,
                orchestration_enabled: true,
                provenance_enabled: true,
            },
        }
    }
}
