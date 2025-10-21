use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::caws_runtime::WorkingSpec as CawsWorkingSpec;
use crate::planning::agent::{TaskContext, AcceptanceCriterion, TestPlan, RollbackPlan, CriterionPriority, RollbackRisk};
use crate::planning::llm_client::{LLMClient, Message, MessageRole, GenerationRequest};

/// Spec generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecGeneratorConfig {
    /// Temperature for generation (lower = more deterministic)
    pub temperature: f32,
    /// Maximum tokens for spec generation
    pub max_tokens: u32,
    /// Enable step-by-step reasoning
    pub enable_reasoning: bool,
    /// Include examples in prompts
    pub include_examples: bool,
}

/// Generates working specs using LLM assistance
pub struct SpecGenerator {
    llm_client: Box<dyn LLMClient>,
    config: SpecGeneratorConfig,
}

impl SpecGenerator {
    pub fn new(llm_client: Box<dyn LLMClient>, config: SpecGeneratorConfig) -> Self {
        Self { llm_client, config }
    }

    /// Generate a working spec from task description and context
    pub async fn generate_spec(
        &self,
        task_description: &str,
        context: &TaskContext,
    ) -> Result<CawsWorkingSpec> {
        tracing::info!("Generating spec for task: {}", task_description);

        // Build the generation prompt
        let prompt = self.build_spec_generation_prompt(task_description, context);

        // Generate the spec
        let messages = vec![
            Message {
                role: MessageRole::System,
                content: self.system_prompt(),
            },
            Message {
                role: MessageRole::User,
                content: prompt,
            },
        ];

        let request = GenerationRequest {
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stop_sequences: Some(vec!["---END---".to_string()]),
        };

        let response = self.llm_client.generate(&request).await?;

        // Parse the response into a working spec
        self.parse_spec_response(&response.content)
    }

    /// Build the comprehensive prompt for spec generation
    fn build_spec_generation_prompt(&self, task_description: &str, context: &TaskContext) -> String {
        let mut prompt = format!(
            "Generate a CAWS working specification for the following task:\n\nTASK: {}\n\n",
            task_description
        );

        // Add context information
        prompt.push_str("CONTEXT INFORMATION:\n");
        prompt.push_str(&format!("Repository: {} ({})\n", context.repo_info.name, context.repo_info.primary_language));
        prompt.push_str(&format!("Tech Stack: Languages={}, Frameworks={}, Databases={}\n",
            context.tech_stack.languages.join(", "),
            context.tech_stack.frameworks.join(", "),
            context.tech_stack.databases.join(", ")
        ));

        // Add recent incidents if any
        if !context.recent_incidents.is_empty() {
            prompt.push_str("\nRECENT INCIDENTS:\n");
            for incident in &context.recent_incidents {
                prompt.push_str(&format!("- {} ({}, resolved: {})\n",
                    incident.title, incident.severity, incident.resolved));
            }
        }

        // Add team constraints
        if !context.team_constraints.is_empty() {
            prompt.push_str("\nTEAM CONSTRAINTS:\n");
            for constraint in &context.team_constraints {
                prompt.push_str(&format!("- {}\n", constraint));
            }
        }

        // Add historical data insights
        prompt.push_str(&format!("\nHISTORICAL INSIGHTS:\n"));
        prompt.push_str(&format!("- Success Rate: {:.1}%\n", context.historical_data.success_rate * 100.0));
        prompt.push_str(&format!("- Average Completion Time: {:.1} hours\n",
            context.historical_data.average_completion_time.as_secs() as f64 / 3600.0));

        prompt.push_str("\nREQUIREMENTS:\n");
        prompt.push_str("1. Define clear scope boundaries (scope.in and scope.out)\n");
        prompt.push_str("2. Specify acceptance criteria in Given/When/Then format\n");
        prompt.push_str("3. Include comprehensive test plan\n");
        prompt.push_str("4. Define rollback procedures\n");
        prompt.push_str("5. Identify constraints and assumptions\n");
        prompt.push_str("6. Estimate change budget (files and LOC)\n");

        if self.config.include_examples {
            prompt.push_str("\nEXAMPLE OUTPUT FORMAT:\n");
            prompt.push_str(self.example_spec());
        }

        prompt.push_str("\nGenerate the working specification following CAWS principles.");
        prompt.push_str("\n\n---END---");

        prompt
    }

    /// System prompt for the LLM
    fn system_prompt(&self) -> String {
        r#"You are an expert CAWS (Constitutional AI Workflow Standards) planning agent.

Your role is to generate high-quality working specifications that follow CAWS principles:
- Constitutional compliance and ethical considerations
- Risk-aware planning with appropriate tier assignment
- Comprehensive acceptance criteria
- Test-driven development approach
- Clear scope boundaries and rollback plans
- Realistic change budgets and constraints

Generate specifications that are:
1. Actionable - clear, specific, and implementable
2. Conservative - err on the side of caution for risk assessment
3. Complete - include all necessary components
4. Testable - define clear acceptance criteria and test plans
5. Reversible - include rollback procedures

Output must be valid YAML that can be parsed into a WorkingSpec structure."#.to_string()
    }

    /// Example spec for the prompt
    fn example_spec(&self) -> &'static str {
        r#"
risk_tier: 2
scope:
  in:
    - "src/api/"
    - "tests/api/"
  out:
    - "src/billing/"
    - "node_modules/"
change_budget_max_files: 15
change_budget_max_loc: 500
acceptance_criteria:
  - id: "A1"
    given: "User is authenticated"
    when: "User requests data via API"
    then: "Data is returned with 200 status"
  - id: "A2"
    given: "User is not authenticated"
    when: "User requests data via API"
    then: "401 Unauthorized is returned"
test_plan:
  unit_tests:
    - "Test authentication middleware"
    - "Test API endpoint validation"
  integration_tests:
    - "Test end-to-end API flow"
rollback_plan:
  steps:
    - "Revert API changes"
    - "Restore previous endpoint versions"
constraints:
  - "Must maintain backward compatibility"
  - "Cannot break existing integrations"
"#
    }

    /// Parse LLM response into a working spec
    fn parse_spec_response(&self, response: &str) -> Result<CawsWorkingSpec> {
        // Extract YAML from response (between ```yaml and ``` markers or just YAML content)
        let yaml_content = self.extract_yaml_from_response(response)?;

        // Parse YAML into our spec structure
        let spec_data: SpecData = serde_yaml::from_str(&yaml_content)?;

        // Convert to CAWS WorkingSpec
        let scope = if let (Some(scope_in), Some(scope_out)) = (spec_data.scope_in, spec_data.scope_out) {
            Some(crate::caws_runtime::WorkingSpecScope {
                r#in: Some(scope_in),
                out: Some(scope_out),
            })
        } else {
            None
        };

        let acceptance_criteria = spec_data.acceptance_criteria.into_iter()
            .enumerate()
            .map(|(i, ac)| AcceptanceCriterion {
                id: format!("A{}", i + 1),
                given: ac.given,
                when: ac.when,
                then: ac.then,
                priority: CriterionPriority::MustHave, // Default to must-have
            })
            .collect();

        let test_plan = spec_data.test_plan.map(|tp| TestPlan {
            unit_tests: tp.unit_tests.unwrap_or_default(),
            integration_tests: tp.integration_tests.unwrap_or_default(),
            e2e_tests: tp.e2e_tests.unwrap_or_default(),
            coverage_target: tp.coverage_target.unwrap_or(0.8),
            mutation_score_target: tp.mutation_score_target.unwrap_or(0.5),
        });

        let rollback_plan = spec_data.rollback_plan.map(|rp| RollbackPlan {
            steps: rp.steps,
            data_backup_required: rp.data_backup_required.unwrap_or(false),
            downtime_expected: rp.downtime_expected.unwrap_or(std::time::Duration::from_secs(0)),
            risk_level: rp.risk_level.unwrap_or(RollbackRisk::Low),
        });

        Ok(CawsWorkingSpec {
            risk_tier: spec_data.risk_tier,
            scope_in: spec_data.scope_in.unwrap_or_default(),
            change_budget_max_files: spec_data.change_budget_max_files,
            change_budget_max_loc: spec_data.change_budget_max_loc,
        })
    }

    /// Extract YAML content from LLM response
    fn extract_yaml_from_response(&self, response: &str) -> Result<String> {
        // Try to find YAML between ```yaml and ``` markers
        if let Some(start) = response.find("```yaml") {
            if let Some(end) = response[start + 7..].find("```") {
                return Ok(response[start + 7..start + 7 + end].trim().to_string());
            }
        }

        // Try to find YAML between ``` and ``` markers
        if let Some(start) = response.find("```") {
            if let Some(end) = response[start + 3..].find("```") {
                let content = &response[start + 3..start + 3 + end];
                // Check if it looks like YAML
                if content.contains("risk_tier:") || content.contains("scope:") {
                    return Ok(content.trim().to_string());
                }
            }
        }

        // Assume the entire response is YAML
        Ok(response.trim().to_string())
    }
}

/// Intermediate structure for parsing YAML spec data
#[derive(Debug, Deserialize)]
struct SpecData {
    risk_tier: u8,
    scope_in: Option<Vec<String>>,
    scope_out: Option<Vec<String>>,
    change_budget_max_files: u32,
    change_budget_max_loc: u32,
    acceptance_criteria: Vec<AcceptanceCriterionData>,
    test_plan: Option<TestPlanData>,
    rollback_plan: Option<RollbackPlanData>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriterionData {
    given: String,
    when: String,
    then: String,
}

#[derive(Debug, Deserialize)]
struct TestPlanData {
    unit_tests: Option<Vec<String>>,
    integration_tests: Option<Vec<String>>,
    e2e_tests: Option<Vec<String>>,
    coverage_target: Option<f64>,
    mutation_score_target: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RollbackPlanData {
    steps: Vec<String>,
    data_backup_required: Option<bool>,
    downtime_expected: Option<u64>, // seconds
    risk_level: Option<String>,
}

impl From<String> for RollbackRisk {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => RollbackRisk::Critical,
            "high" => RollbackRisk::High,
            "medium" => RollbackRisk::Medium,
            _ => RollbackRisk::Low,
        }
    }
}

impl From<RollbackPlanData> for RollbackPlan {
    fn from(data: RollbackPlanData) -> Self {
        RollbackPlan {
            steps: data.steps,
            data_backup_required: data.data_backup_required.unwrap_or(false),
            downtime_expected: std::time::Duration::from_secs(data.downtime_expected.unwrap_or(0)),
            risk_level: data.risk_level.map(|s| s.into()).unwrap_or(RollbackRisk::Low),
        }
    }
}

pub type Result<T> = std::result::Result<T, SpecGeneratorError>;

#[derive(Debug, thiserror::Error)]
pub enum SpecGeneratorError {
    #[error("LLM generation failed: {0}")]
    LLMError(#[from] anyhow::Error),

    #[error("YAML parsing failed: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Invalid spec format: {0}")]
    InvalidFormat(String),

    #[error("Spec validation failed: {0}")]
    ValidationError(String),
}

