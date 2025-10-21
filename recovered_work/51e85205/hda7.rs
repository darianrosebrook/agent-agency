use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::caws_runtime::{CawsRuntimeValidator, WorkingSpec as CawsWorkingSpec};
use crate::planning::context_builder::ContextBuilder;
use crate::planning::llm_client::{LLMClient, Message, MessageRole, GenerationRequest};
use crate::planning::spec_generator::SpecGenerator;
use crate::planning::validation_loop::ValidationLoop;

/// Task context for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Repository information
    pub repo_info: RepositoryInfo,
    /// Recent incidents or issues
    pub recent_incidents: Vec<Incident>,
    /// Current team constraints
    pub team_constraints: Vec<String>,
    /// Technology stack information
    pub tech_stack: TechStack,
    /// Historical task completion data
    pub historical_data: HistoricalData,
}

/// Ambiguity assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityAssessment {
    /// Overall ambiguity score (0.0 = clear, 1.0 = highly ambiguous)
    pub ambiguity_score: f32,
    /// Specific ambiguity types detected
    pub ambiguity_types: Vec<AmbiguityType>,
    /// Questions needed to clarify the task
    pub clarification_questions: Vec<ClarificationQuestion>,
    /// Whether clarification is required before proceeding
    pub clarification_required: bool,
    /// Confidence in the assessment
    pub assessment_confidence: f32,
}

/// Types of ambiguity detected in task descriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmbiguityType {
    /// Missing specific subject or object
    VagueSubject,
    /// Undefined success criteria
    UnclearObjectives,
    /// Missing scope boundaries
    UndefinedScope,
    /// Ambiguous technical requirements
    TechnicalAmbiguity,
    /// Missing context about existing systems
    ContextualGaps,
    /// Multiple possible interpretations
    MultipleInterpretations,
    /// Incomplete requirement specification
    IncompleteRequirements,
}

/// Clarification question for user interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    /// Unique question ID
    pub id: String,
    /// The question text
    pub question: String,
    /// Type of information being requested
    pub question_type: QuestionType,
    /// Suggested answers (if applicable)
    pub suggested_answers: Vec<String>,
    /// Whether the question is required
    pub required: bool,
    /// Priority level
    pub priority: QuestionPriority,
}

/// Types of clarification questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    /// Free-form text response
    FreeForm,
    /// Multiple choice selection
    MultipleChoice,
    /// Yes/No question
    Boolean,
    /// Specific technical choices
    TechnicalChoice,
    /// Scope definition
    ScopeDefinition,
}

/// Priority levels for clarification questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionPriority {
    /// Critical for proceeding
    Critical,
    /// Important for quality
    Important,
    /// Nice to have
    Optional,
}

/// User clarification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationResponse {
    /// Question ID being answered
    pub question_id: String,
    /// User's response
    pub response: String,
    /// Response timestamp
    pub responded_at: DateTime<Utc>,
    /// Additional context or notes
    pub notes: Option<String>,
}

/// Interactive clarification session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationSession {
    /// Session ID
    pub session_id: String,
    /// Original task description
    pub original_task: String,
    /// Ambiguity assessment
    pub assessment: AmbiguityAssessment,
    /// Questions asked
    pub questions_asked: Vec<ClarificationQuestion>,
    /// Responses received
    pub responses: Vec<ClarificationResponse>,
    /// Session status
    pub status: SessionStatus,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Session completion time
    pub completed_at: Option<DateTime<Utc>>,
}

/// Clarification session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session initiated, waiting for responses
    Active,
    /// All required questions answered
    Completed,
    /// Session timed out or cancelled
    Terminated,
    /// Clarification provided, ready for planning
    ReadyForPlanning,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub name: String,
    pub description: Option<String>,
    pub primary_language: String,
    pub size_kb: u64,
    pub last_commit: DateTime<Utc>,
    pub contributors: Vec<String>,
}

/// Historical incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub resolved: bool,
    pub tags: Vec<String>,
}

/// Technology stack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub deployment: Vec<String>,
}

/// Historical task data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalData {
    pub completed_tasks: Vec<TaskHistory>,
    pub average_completion_time: std::time::Duration,
    pub success_rate: f64,
}

/// Task completion history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistory {
    pub task_type: String,
    pub risk_tier: u8,
    pub completion_time: std::time::Duration,
    pub success: bool,
    pub quality_score: Option<f64>,
}

/// Planning agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningAgentConfig {
    /// Maximum planning iterations
    pub max_iterations: u32,
    /// Timeout for planning operations
    pub planning_timeout: std::time::Duration,
    /// Risk tier inference confidence threshold
    pub risk_confidence_threshold: f64,
    /// Enable context enrichment
    pub enable_context_enrichment: bool,
}

/// Planning agent that generates working specs from natural language
pub struct PlanningAgent {
    llm_client: Box<dyn LLMClient>,
    spec_generator: SpecGenerator,
    context_builder: ContextBuilder,
    validator: Arc<dyn CawsRuntimeValidator>,
    config: PlanningAgentConfig,
}

impl PlanningAgent {
    pub fn new(
        llm_client: Box<dyn LLMClient>,
        spec_generator: SpecGenerator,
        context_builder: ContextBuilder,
        validator: Arc<dyn CawsRuntimeValidator>,
        config: PlanningAgentConfig,
    ) -> Self {
        Self {
            llm_client,
            spec_generator,
            context_builder,
            validator,
            config,
        }
    }

    /// Generate a working spec from a natural language task description
    pub async fn generate_working_spec(
        &self,
        task_description: &str,
        context: TaskContext,
    ) -> Result<WorkingSpec> {
        tracing::info!("Generating working spec for task: {}", task_description);

        // Build enriched context
        let enriched_context = if self.config.enable_context_enrichment {
            self.context_builder.enrich_context(context).await?
        } else {
            context
        };

        // Generate initial spec using LLM
        let initial_spec = self.spec_generator
            .generate_spec(task_description, &enriched_context)
            .await?;

        // Validate and repair the spec
        let validation_loop = ValidationLoop::new(
            self.validator.clone(),
            self.llm_client.as_ref(),
            self.config.max_iterations,
        );

        let validated_spec = validation_loop
            .validate_and_repair(initial_spec, task_description, &enriched_context)
            .await?;

        // Add metadata and provenance
        let final_spec = WorkingSpec {
            id: format!("SPEC-{}", Uuid::new_v4().simple()),
            title: self.extract_title_from_description(task_description),
            description: task_description.to_string(),
            risk_tier: self.infer_risk_tier(&validated_spec, &enriched_context),
            scope: validated_spec.scope,
            acceptance_criteria: validated_spec.acceptance_criteria,
            test_plan: validated_spec.test_plan,
            rollback_plan: validated_spec.rollback_plan,
            constraints: validated_spec.constraints,
            estimated_effort: self.estimate_effort(&validated_spec, &enriched_context),
            generated_at: Utc::now(),
            context_hash: self.hash_context(&enriched_context),
        };

        tracing::info!("Generated working spec: {} (risk tier: {})", final_spec.id, final_spec.risk_tier);
        Ok(final_spec)
    }

    /// Extract a concise title from the task description
    fn extract_title_from_description(&self, description: &str) -> String {
        // Use LLM to generate a concise title
        // For now, use a simple heuristic
        let words: Vec<&str> = description.split_whitespace().take(8).collect();
        format!("{}...", words.join(" "))
    }

    /// Infer risk tier based on spec content and context
    fn infer_risk_tier(&self, spec: &CawsWorkingSpec, context: &TaskContext) -> u8 {
        // Risk tier inference logic
        // Tier 1: Critical (authentication, billing, data integrity)
        // Tier 2: High (API changes, database schema)
        // Tier 3: Standard (UI changes, internal tools)

        let description = spec.title.to_lowercase();

        if description.contains("auth") || description.contains("security") ||
           description.contains("billing") || description.contains("payment") ||
           description.contains("database") || description.contains("migration") {
            1
        } else if description.contains("api") || description.contains("endpoint") ||
                  description.contains("schema") || description.contains("breaking") {
            2
        } else {
            3
        }
    }

    /// Estimate effort in hours based on spec and historical data
    fn estimate_effort(&self, spec: &CawsWorkingSpec, context: &TaskContext) -> std::time::Duration {
        // Simple estimation based on risk tier and historical data
        let base_hours = match spec.risk_tier {
            1 => 16.0, // 2 days
            2 => 8.0,  // 1 day
            3 => 4.0,  // 0.5 days
            _ => 4.0,
        };

        // Adjust based on historical data
        let adjustment_factor = if context.historical_data.completed_tasks.len() > 5 {
            let avg_completion_hours = context.historical_data.average_completion_time.as_secs() as f64 / 3600.0;
            (avg_completion_hours / base_hours).min(2.0).max(0.5)
        } else {
            1.0
        };

        let estimated_hours = base_hours * adjustment_factor;
        std::time::Duration::from_secs((estimated_hours * 3600.0) as u64)
    }

    /// Generate a hash of the context for provenance
    fn hash_context(&self, context: &TaskContext) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get planning agent health status
    pub async fn health_check(&self) -> Result<()> {
        self.llm_client.health_check().await?;
        Ok(())
    }
}

/// Working spec with additional metadata for autonomous execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    pub id: String,
    pub title: String,
    pub description: String,
    pub risk_tier: u8,
    pub scope: Option<crate::caws_runtime::WorkingSpecScope>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub test_plan: Option<TestPlan>,
    pub rollback_plan: Option<RollbackPlan>,
    pub constraints: Vec<String>,
    pub estimated_effort: std::time::Duration,
    pub generated_at: DateTime<Utc>,
    pub context_hash: String,
}

/// Acceptance criterion for the working spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
    pub priority: CriterionPriority,
}

/// Priority levels for acceptance criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionPriority {
    MustHave,
    ShouldHave,
    CouldHave,
}

/// Test plan for the working spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlan {
    pub unit_tests: Vec<String>,
    pub integration_tests: Vec<String>,
    pub e2e_tests: Vec<String>,
    pub coverage_target: f64,
    pub mutation_score_target: f64,
}

/// Rollback plan for the working spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub steps: Vec<String>,
    pub data_backup_required: bool,
    pub downtime_expected: std::time::Duration,
    pub risk_level: RollbackRisk,
}

/// Risk levels for rollback operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackRisk {
    Low,
    Medium,
    High,
    Critical,
}

pub type Result<T> = std::result::Result<T, PlanningError>;

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("LLM generation failed: {0}")]
    LLMError(#[from] anyhow::Error),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Context building failed: {0}")]
    ContextError(String),

    #[error("Spec generation failed: {0}")]
    SpecGenerationError(String),

    #[error("Planning timeout exceeded")]
    TimeoutError,

    #[error("Maximum iterations exceeded")]
    MaxIterationsExceeded,
}
