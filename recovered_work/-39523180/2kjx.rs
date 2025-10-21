use std::sync::Arc;
use chrono::Utc;

use crate::caws_runtime::{CawsRuntimeValidator, DiffStats, TaskDescriptor, WorkingSpec as CawsWorkingSpec};
use crate::planning::agent::TaskContext;
use crate::planning::llm_client::{LLMClient, Message, MessageRole, GenerationRequest};

/// Validation loop configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationLoopConfig {
    /// Maximum repair iterations
    pub max_iterations: u32,
    /// Timeout for validation operations
    pub validation_timeout: std::time::Duration,
    /// Enable automatic repair suggestions
    pub enable_auto_repair: bool,
    /// Minimum compliance score to accept
    pub min_compliance_score: f64,
}

/// Handles CAWS validation and iterative repair of working specs
pub struct ValidationLoop {
    validator: Arc<dyn CawsRuntimeValidator>,
    llm_client: Box<dyn LLMClient>,
    config: ValidationLoopConfig,
}

impl ValidationLoop {
    pub fn new(
        validator: Arc<dyn CawsRuntimeValidator>,
        llm_client: Box<dyn LLMClient>,
        max_iterations: u32,
    ) -> Self {
        Self {
            validator,
            llm_client,
            config: ValidationLoopConfig {
                max_iterations,
                validation_timeout: std::time::Duration::from_secs(30),
                enable_auto_repair: true,
                min_compliance_score: 0.8,
            },
        }
    }

    /// Validate and repair a working spec until it passes CAWS validation
    pub async fn validate_and_repair(
        &self,
        mut spec: CawsWorkingSpec,
        task_description: &str,
        context: &TaskContext,
    ) -> Result<CawsWorkingSpec> {
        tracing::info!("Starting CAWS validation loop for spec");

        for iteration in 0..self.config.max_iterations {
            tracing::debug!("Validation iteration {}", iteration + 1);

            // Create a mock task descriptor for validation
            let task_desc = TaskDescriptor {
                task_id: format!("validation-{}", iteration + 1),
                scope_in: spec.scope_in.clone(),
                risk_tier: spec.risk_tier,
                acceptance: None,
                metadata: None,
            };

            // Create mock diff stats (assume no changes for planning phase)
            let diff_stats = DiffStats {
                files_changed: 0,
                lines_changed: 0,
                touched_paths: vec![],
            };

            // Run CAWS validation
            let validation_result = tokio::time::timeout(
                self.config.validation_timeout,
                self.validator.validate(
                    &spec,
                    &task_desc,
                    &diff_stats,
                    &[], // no patches in planning phase
                    &[], // no language hints
                    false, // no tests added yet
                    true,  // assume deterministic for planning
                    vec![], // no waivers
                )
            ).await??;

            // Check if validation passed
            if validation_result.violations.is_empty() {
                tracing::info!("CAWS validation passed on iteration {}", iteration + 1);
                return Ok(spec);
            }

            // Check if we've exceeded max iterations
            if iteration >= self.config.max_iterations - 1 {
                return Err(ValidationLoopError::MaxIterationsExceeded {
                    final_violations: validation_result.violations,
                });
            }

            // Generate repair suggestions
            if self.config.enable_auto_repair {
                spec = self.repair_spec(spec, &validation_result.violations, task_description, context).await?;
            } else {
                return Err(ValidationLoopError::ValidationFailed {
                    violations: validation_result.violations,
                });
            }
        }

        Err(ValidationLoopError::MaxIterationsExceeded {
            final_violations: vec![],
        })
    }

    /// Repair a working spec based on validation violations
    async fn repair_spec(
        &self,
        mut spec: CawsWorkingSpec,
        violations: &[crate::caws_runtime::Violation],
        task_description: &str,
        context: &TaskContext,
    ) -> Result<CawsWorkingSpec> {
        tracing::info!("Repairing spec with {} violations", violations.len());

        // Build repair prompt
        let repair_prompt = self.build_repair_prompt(&spec, violations, task_description, context);

        // Generate repair suggestions
        let messages = vec![
            Message {
                role: MessageRole::System,
                content: self.repair_system_prompt(),
            },
            Message {
                role: MessageRole::User,
                content: repair_prompt,
            },
        ];

        let request = GenerationRequest {
            messages,
            max_tokens: Some(2000),
            temperature: Some(0.3), // Lower temperature for more focused repairs
            stop_sequences: Some(vec!["---END---".to_string()]),
        };

        let response = self.llm_client.generate(&request).await?;

        // Parse and apply repairs
        self.apply_repairs(spec, &response.content)
    }

    /// Build the repair prompt
    fn build_repair_prompt(
        &self,
        spec: &CawsWorkingSpec,
        violations: &[crate::caws_runtime::Violation],
        task_description: &str,
        context: &TaskContext,
    ) -> String {
        let mut prompt = format!(
            "The following working spec has CAWS validation violations. Please repair it:\n\nTASK: {}\n\nCURRENT SPEC:\n",
            task_description
        );

        // Add current spec in YAML format
        prompt.push_str(&format!("risk_tier: {}\n", spec.risk_tier));
        prompt.push_str(&format!("change_budget_max_files: {}\n", spec.change_budget_max_files));
        prompt.push_str(&format!("change_budget_max_loc: {}\n", spec.change_budget_max_loc));
        prompt.push_str("scope_in:\n");
        for path in &spec.scope_in {
            prompt.push_str(&format!("  - \"{}\"\n", path));
        }

        prompt.push_str("\nVIOLATIONS:\n");
        for violation in violations {
            prompt.push_str(&format!("- {}: {}\n", violation.code, violation.message));
            if let Some(remediation) = &violation.remediation {
                prompt.push_str(&format!("  Suggested fix: {}\n", remediation));
            }
        }

        prompt.push_str("\nCONTEXT:\n");
        prompt.push_str(&format!("Repository: {} ({})\n", context.repo_info.name, context.repo_info.primary_language));
        prompt.push_str(&format!("Tech Stack: {}\n", context.tech_stack.languages.join(", ")));

        prompt.push_str("\nPlease provide a repaired version of the working spec that addresses these violations.");
        prompt.push_str("Output only the YAML specification, no additional text.\n\n---END---");

        prompt
    }

    /// System prompt for repair operations
    fn repair_system_prompt(&self) -> &'static str {
        r#"You are a CAWS compliance repair specialist.

Your task is to fix working specifications that have CAWS validation violations.

Common repairs:
- RiskOutOfBounds: Adjust risk_tier to 1, 2, or 3 based on task impact
- ScopeViolation: Add missing paths to scope_in or create appropriate boundaries
- BudgetExceeded: Increase change_budget_max_files/max_loc based on task complexity
- MissingTests: Ensure test coverage requirements are reasonable
- NonDeterministic: Remove references to random/time functions

Guidelines:
1. Be conservative - err on the side of higher risk tiers and larger budgets
2. Maintain task intent - don't change the fundamental scope
3. Use reasonable defaults - base estimates on similar historical tasks
4. Ensure compliance - address all violations with specific fixes

Output only valid YAML that can be parsed as a WorkingSpec."#
    }

    /// Apply repairs from LLM response
    fn apply_repairs(&self, mut original_spec: CawsWorkingSpec, response: &str) -> Result<CawsWorkingSpec> {
        // Extract YAML from response
        let yaml_content = self.extract_yaml_from_response(response)?;

        // Parse repair data
        let repair_data: RepairData = serde_yaml::from_str(&yaml_content)?;

        // Apply repairs to original spec
        let repaired_spec = CawsWorkingSpec {
            risk_tier: repair_data.risk_tier.unwrap_or(original_spec.risk_tier),
            scope_in: repair_data.scope_in.unwrap_or(original_spec.scope_in),
            change_budget_max_files: repair_data.change_budget_max_files.unwrap_or(original_spec.change_budget_max_files),
            change_budget_max_loc: repair_data.change_budget_max_loc.unwrap_or(original_spec.change_budget_max_loc),
        };

        tracing::debug!("Applied repairs: risk_tier {} -> {}, budget {} -> {} files, {} -> {} loc",
            original_spec.risk_tier, repaired_spec.risk_tier,
            original_spec.change_budget_max_files, repaired_spec.change_budget_max_files,
            original_spec.change_budget_max_loc, repaired_spec.change_budget_max_loc);

        Ok(repaired_spec)
    }

    /// Extract YAML content from repair response
    fn extract_yaml_from_response(&self, response: &str) -> Result<String> {
        // Similar logic to spec_generator
        if let Some(start) = response.find("```yaml") {
            if let Some(end) = response[start + 7..].find("```") {
                return Ok(response[start + 7..start + 7 + end].trim().to_string());
            }
        }

        if let Some(start) = response.find("```") {
            if let Some(end) = response[start + 3..].find("```") {
                let content = &response[start + 3..start + 3 + end];
                if content.contains("risk_tier:") || content.contains("scope_in:") {
                    return Ok(content.trim().to_string());
                }
            }
        }

        Ok(response.trim().to_string())
    }
}

/// Data structure for parsing repair responses
#[derive(Debug, serde::Deserialize)]
struct RepairData {
    risk_tier: Option<u8>,
    scope_in: Option<Vec<String>>,
    change_budget_max_files: Option<u32>,
    change_budget_max_loc: Option<u32>,
}

pub type Result<T> = std::result::Result<T, ValidationLoopError>;

#[derive(Debug, thiserror::Error)]
pub enum ValidationLoopError {
    #[error("Validation timeout exceeded")]
    TimeoutError(#[from] tokio::time::error::Elapsed),

    #[error("LLM repair generation failed: {0}")]
    LLMError(#[from] anyhow::Error),

    #[error("YAML parsing failed: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Validation failed with violations: {violations:?}")]
    ValidationFailed {
        violations: Vec<crate::caws_runtime::Violation>,
    },

    #[error("Maximum repair iterations exceeded with final violations: {final_violations:?}")]
    MaxIterationsExceeded {
        final_violations: Vec<crate::caws_runtime::Violation>,
    },

    #[error("Repair application failed: {0}")]
    RepairError(String),
}

