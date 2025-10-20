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

/// Technical feasibility assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibilityAssessment {
    /// Overall feasibility score (0.0 = impossible, 1.0 = highly feasible)
    pub feasibility_score: f32,
    /// Specific feasibility concerns identified
    pub feasibility_concerns: Vec<FeasibilityConcern>,
    /// Domain expertise requirements
    pub domain_expertise: Vec<DomainExpertise>,
    /// Resource requirements assessment
    pub resource_requirements: ResourceRequirements,
    /// Technical complexity metrics
    pub complexity_metrics: ComplexityMetrics,
    /// Performance feasibility analysis
    pub performance_analysis: PerformanceAnalysis,
    /// Recommended risk mitigation strategies
    pub risk_mitigations: Vec<String>,
    /// Assessment confidence
    pub assessment_confidence: f32,
}

/// Types of feasibility concerns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeasibilityConcern {
    /// Requires domain expertise we don't have
    DomainExpertiseGap,
    /// Technically impossible within constraints
    TechnicalImpossibility,
    /// Performance requirements unrealistic
    PerformanceUnrealistic,
    /// Resource requirements exceed available capacity
    ResourceConstraints,
    /// Dependencies cannot be satisfied
    DependencyConflicts,
    /// Security requirements conflict with functionality
    SecurityConstraints,
    /// Timeline too aggressive for scope
    TimelineConstraints,
}

/// Domain expertise requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExpertise {
    /// Domain area required
    pub domain: String,
    /// Expertise level needed (1-5)
    pub expertise_level: u8,
    /// Whether we have this expertise
    pub available_internally: bool,
    /// Estimated time to acquire if needed
    pub acquisition_time_weeks: Option<u8>,
}

/// Resource requirements assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Estimated development time in hours
    pub development_hours: u16,
    /// Required technical skills
    pub required_skills: Vec<String>,
    /// Hardware/software requirements
    pub infrastructure_needs: Vec<String>,
    /// External dependencies required
    pub external_dependencies: Vec<String>,
    /// Cost estimate range
    pub cost_estimate_usd: (u32, u32),
}

/// Technical complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity estimate
    pub cyclomatic_complexity: u8,
    /// Number of integration points
    pub integration_points: u8,
    /// Data transformation complexity
    pub data_complexity: u8,
    /// Algorithmic complexity class
    pub algorithmic_complexity: String,
    /// Testing complexity factor
    pub testing_complexity: f32,
}

/// Performance feasibility analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Required latency in microseconds
    pub required_latency_us: Option<u64>,
    /// Required throughput (ops/sec)
    pub required_throughput: Option<u32>,
    /// Memory requirements
    pub memory_requirements_gb: Option<f32>,
    /// Network requirements
    pub network_requirements_mbps: Option<u32>,
    /// Feasibility assessment
    pub feasibility_assessment: PerformanceFeasibility,
    /// Performance risk factors
    pub risk_factors: Vec<String>,
}

/// Performance feasibility levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceFeasibility {
    /// Highly feasible with current technology
    Feasible,
    /// Challenging but achievable with optimization
    Challenging,
    /// Requires specialized techniques/hardware
    Specialized,
    /// Currently impossible with known technology
    Impossible,
}

/// Recommended implementation approach based on risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedApproach {
    DirectImplementation,
    PhasedImplementation,
    PrototypeFirst,
    ReconsiderRequirements,
}

/// Comprehensive risk assessment combining all dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveRiskAssessment {
    /// Overall risk score (0.0 = low risk, 1.0 = high risk)
    pub overall_risk_score: f32,
    /// Ambiguity assessment results
    pub ambiguity_assessment: AmbiguityAssessment,
    /// Feasibility assessment results
    pub feasibility_assessment: FeasibilityAssessment,
    /// Identified risk factors
    pub risk_factors: Vec<RiskFactor>,
    /// Recommended mitigation strategies
    pub mitigation_strategies: Vec<String>,
    /// Contingency plans for high-risk scenarios
    pub contingency_plans: Vec<String>,
    /// Recommended implementation approach
    pub recommended_approach: RecommendedApproach,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk factor
    pub factor_type: RiskFactorType,
    /// Severity level
    pub severity: RiskSeverity,
    /// Description of the risk
    pub description: String,
    /// Probability of impact (0.0-1.0)
    pub impact_probability: f32,
}

/// Types of risk factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    /// Ambiguity-related risk
    Ambiguity,
    /// Feasibility-related risk
    Feasibility,
    /// Technical complexity risk
    Technical,
    /// Resource availability risk
    Resource,
    /// Timeline risk
    Timeline,
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    /// Low impact, easily mitigated
    Low,
    /// Moderate impact, requires planning
    Medium,
    /// High impact, significant mitigation needed
    High,
    /// Critical impact, may require project changes
    Critical,
}

/// Domain expertise validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExpertiseValidation {
    /// Domains required for the task
    pub required_domains: Vec<String>,
    /// Expertise level required for each domain (1-5)
    pub expertise_levels: HashMap<String, u8>,
    /// Whether expertise is available internally
    pub available_expertise: HashMap<String, bool>,
    /// Whether expertise acquisition is feasible
    pub acquisition_feasible: bool,
    /// Time required to acquire missing expertise (weeks)
    pub acquisition_time_weeks: Option<u8>,
    /// Cost estimate for expertise acquisition
    pub acquisition_cost: Option<String>,
}

/// Domain requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRequirement {
    /// Description of the domain expertise
    pub description: String,
    /// Typical expertise levels needed (1-5 scale)
    pub typical_expertise_levels: Vec<u8>,
    /// Common technologies in this domain
    pub common_technologies: Vec<String>,
}

/// Mathematical complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicalComplexity {
    /// Computational complexity class
    pub complexity_class: ComputationalComplexity,
    /// Mathematical maturity level required (1-5)
    pub mathematical_maturity_level: u8,
    /// Complexity of mathematical proofs required
    pub proof_complexity: String,
    /// Whether numerical stability is a concern
    pub numerical_stability_concerns: bool,
    /// Implementation challenges identified
    pub implementation_challenges: Vec<String>,
    /// Complexity patterns identified in the task
    pub identified_patterns: Vec<String>,
}

/// Computational complexity classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputationalComplexity {
    Constant,
    Logarithmic,
    Linear,
    Polynomial,
    Exponential,
    Undecidable,
}

/// Performance feasibility modeling result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceFeasibilityModel {
    /// Extracted performance requirements
    pub extracted_requirements: PerformanceRequirements,
    /// Hardware constraints analysis
    pub hardware_constraints: HardwareConstraints,
    /// Theoretical performance bounds
    pub theoretical_bounds: TheoreticalBounds,
    /// Practical achievability assessment
    pub practical_assessment: PracticalAssessment,
    /// Performance optimization recommendations
    pub optimization_recommendations: Vec<String>,
}

/// Performance requirements extracted from task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Required latency in microseconds
    pub latency_microseconds: Option<u64>,
    /// Required throughput in operations per second
    pub throughput_operations_per_second: Option<u32>,
    /// Memory requirements in GB
    pub memory_requirements_gb: Option<f32>,
    /// Network bandwidth requirements in Mbps
    pub network_bandwidth_mbps: Option<u32>,
}

/// Hardware constraints analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConstraints {
    /// Identified hardware constraints
    pub identified_constraints: Vec<String>,
    /// Recommended hardware configurations
    pub recommended_hardware: Vec<String>,
    /// Cost implications of hardware requirements
    pub cost_implications: String,
}

/// Theoretical performance bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoreticalBounds {
    /// Latency theoretical bounds
    pub latency_bounds: Option<TheoreticalLatency>,
    /// Throughput theoretical bounds
    pub throughput_bounds: Option<TheoreticalThroughput>,
}

/// Theoretical latency bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoreticalLatency {
    /// Requested latency
    pub requested: u64,
    /// Theoretical minimum latency (GHz limit)
    pub theoretical_minimum: u64,
    /// Light speed limit for distributed systems
    pub light_speed_limit: u64,
    /// Whether the requirement is theoretically achievable
    pub achievable: bool,
}

/// Theoretical throughput bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoreticalThroughput {
    /// Requested throughput
    pub requested: u32,
    /// Theoretical maximum throughput
    pub theoretical_maximum: u32,
    /// Practical throughput limit
    pub practical_limit: u32,
    /// Whether the requirement is theoretically achievable
    pub achievable: bool,
}

/// Practical achievability assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticalAssessment {
    /// Overall feasibility score (0.0 = impossible, 1.0 = highly feasible)
    pub feasibility_score: f32,
    /// Identified practical concerns
    pub identified_concerns: Vec<String>,
    /// Recommended implementation approach
    pub recommended_approach: String,
}

/// Result of working spec generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkingSpecResult {
    /// Working spec successfully generated
    Success(WorkingSpec),
    /// Clarification needed before proceeding
    ClarificationNeeded {
        assessment: AmbiguityAssessment,
        session: ClarificationSession,
    },
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

    /// Assess task ambiguity and determine if clarification is needed
    pub async fn assess_ambiguity(&self, task_description: &str) -> Result<AmbiguityAssessment> {
        tracing::info!("Assessing ambiguity for task: {}", task_description);

        // Use LLM to analyze task description for ambiguity
        let analysis_prompt = format!(
            "Analyze the following task description for ambiguity and clarity issues. \
             Identify specific areas that need clarification before implementation can begin. \
             Consider: subject/object clarity, success criteria, scope boundaries, technical requirements, \
             context dependencies, and completeness of requirements.\n\n\
             Task: {}\n\n\
             Provide analysis in JSON format with ambiguity_score (0.0-1.0), \
             ambiguity_types array, clarification_questions array, and clarification_required boolean.",
            task_description
        );

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are an expert at analyzing task descriptions for ambiguity and identifying clarification needs. Always respond with valid JSON.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: analysis_prompt,
            }
        ];

        let request = GenerationRequest {
            messages,
            temperature: 0.1, // Low temperature for consistent analysis
            max_tokens: 1000,
            ..Default::default()
        };

        let response = self.llm_client.generate(request).await?;
        let analysis: serde_json::Value = serde_json::from_str(&response.content)
            .map_err(|e| PlanningError::LLMError(anyhow::anyhow!("Failed to parse ambiguity analysis: {}", e)))?;

        // Parse the LLM response into our structured format
        let ambiguity_score = analysis["ambiguity_score"].as_f64().unwrap_or(0.0) as f32;
        let clarification_required = analysis["clarification_required"].as_bool().unwrap_or(false);

        let ambiguity_types = analysis["ambiguity_types"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|t| {
                match t.as_str()? {
                    "vague_subject" => Some(AmbiguityType::VagueSubject),
                    "unclear_objectives" => Some(AmbiguityType::UnclearObjectives),
                    "undefined_scope" => Some(AmbiguityType::UndefinedScope),
                    "technical_ambiguity" => Some(AmbiguityType::TechnicalAmbiguity),
                    "contextual_gaps" => Some(AmbiguityType::ContextualGaps),
                    "multiple_interpretations" => Some(AmbiguityType::MultipleInterpretations),
                    "incomplete_requirements" => Some(AmbiguityType::IncompleteRequirements),
                    _ => None,
                }
            })
            .collect();

        let clarification_questions = analysis["clarification_questions"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let question_text = q["question"].as_str().unwrap_or("Please clarify this aspect");
                let question_type = match q["type"].as_str().unwrap_or("free_form") {
                    "multiple_choice" => QuestionType::MultipleChoice,
                    "boolean" => QuestionType::Boolean,
                    "technical_choice" => QuestionType::TechnicalChoice,
                    "scope_definition" => QuestionType::ScopeDefinition,
                    _ => QuestionType::FreeForm,
                };

                let suggested_answers = q["suggested_answers"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|a| a.as_str().map(|s| s.to_string()))
                    .collect();

                let priority = match q["priority"].as_str().unwrap_or("important") {
                    "critical" => QuestionPriority::Critical,
                    "important" => QuestionPriority::Important,
                    _ => QuestionPriority::Optional,
                };

                ClarificationQuestion {
                    id: format!("Q{}", i + 1),
                    question: question_text.to_string(),
                    question_type,
                    suggested_answers,
                    required: q["required"].as_bool().unwrap_or(true),
                    priority,
                }
            })
            .collect();

        Ok(AmbiguityAssessment {
            ambiguity_score,
            ambiguity_types,
            clarification_questions,
            clarification_required,
            assessment_confidence: 0.85, // LLM-based assessment confidence
        })
    }

    /// Initiate an interactive clarification session
    pub async fn initiate_clarification(
        &self,
        task_description: &str,
        assessment: &AmbiguityAssessment,
    ) -> Result<ClarificationSession> {
        let session_id = format!("SESSION-{}", Uuid::new_v4().simple());

        let session = ClarificationSession {
            session_id: session_id.clone(),
            original_task: task_description.to_string(),
            assessment: assessment.clone(),
            questions_asked: assessment.clarification_questions.clone(),
            responses: vec![],
            status: SessionStatus::Active,
            started_at: Utc::now(),
            completed_at: None,
        };

        tracing::info!("Initiated clarification session {} for ambiguous task", session_id);
        Ok(session)
    }

    /// Process a clarification response
    pub async fn process_clarification_response(
        &self,
        session: &mut ClarificationSession,
        response: ClarificationResponse,
    ) -> Result<()> {
        // Validate response belongs to this session
        if !session.questions_asked.iter().any(|q| q.id == response.question_id) {
            return Err(PlanningError::ValidationError(
                format!("Question ID {} not found in session", response.question_id)
            ));
        }

        // Add response to session
        session.responses.push(response);

        // Check if all required questions are answered
        let required_questions: std::collections::HashSet<&str> = session
            .questions_asked
            .iter()
            .filter(|q| q.required)
            .map(|q| q.id.as_str())
            .collect();

        let answered_questions: std::collections::HashSet<&str> = session
            .responses
            .iter()
            .map(|r| r.question_id.as_str())
            .collect();

        let all_required_answered = required_questions.is_subset(&answered_questions);

        // Update session status
        if all_required_answered {
            session.status = SessionStatus::ReadyForPlanning;
            session.completed_at = Some(Utc::now());
            tracing::info!("Clarification session {} ready for planning", session.session_id);
        }

        Ok(())
    }

    /// Generate enriched task description from clarification responses
    pub fn enrich_task_description(
        &self,
        original_task: &str,
        session: &ClarificationSession,
    ) -> String {
        let mut enriched = original_task.to_string();

        // Add clarification responses as structured context
        enriched.push_str("\n\nClarification Context:");
        for response in &session.responses {
            if let Some(question) = session.questions_asked.iter().find(|q| q.id == response.question_id) {
                enriched.push_str(&format!("\n{}: {}", question.question, response.response));
                if let Some(notes) = &response.notes {
                    enriched.push_str(&format!(" (Note: {})", notes));
                }
            }
        }

        enriched
    }

    /// Assess technical feasibility of a task
    pub async fn assess_feasibility(&self, task_description: &str) -> Result<FeasibilityAssessment> {
        tracing::info!("Assessing technical feasibility for task: {}", task_description);

        // Use LLM to analyze technical feasibility
        let feasibility_prompt = format!(
            "Analyze the technical feasibility of the following task. Consider: \
             domain expertise requirements, performance constraints, resource needs, \
             technical complexity, dependencies, and timeline feasibility.\n\n\
             Task: {}\n\n\
             Provide analysis in JSON format with feasibility_score (0.0-1.0), \
             feasibility_concerns array, domain_expertise array, resource_requirements, \
             complexity_metrics, performance_analysis, and risk_mitigations array.",
            task_description
        );

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are an expert technical architect analyzing project feasibility. Always respond with valid JSON.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: feasibility_prompt,
            }
        ];

        let request = GenerationRequest {
            messages,
            temperature: 0.1, // Low temperature for consistent analysis
            max_tokens: 1200,
            ..Default::default()
        };

        let response = self.llm_client.generate(request).await?;
        let analysis: serde_json::Value = serde_json::from_str(&response.content)
            .map_err(|e| PlanningError::LLMError(anyhow::anyhow!("Failed to parse feasibility analysis: {}", e)))?;

        // Parse the LLM response into our structured format
        let feasibility_score = analysis["feasibility_score"].as_f64().unwrap_or(0.5) as f32;

        let feasibility_concerns = analysis["feasibility_concerns"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|c| {
                match c.as_str()? {
                    "domain_expertise_gap" => Some(FeasibilityConcern::DomainExpertiseGap),
                    "technical_impossibility" => Some(FeasibilityConcern::TechnicalImpossibility),
                    "performance_unrealistic" => Some(FeasibilityConcern::PerformanceUnrealistic),
                    "resource_constraints" => Some(FeasibilityConcern::ResourceConstraints),
                    "dependency_conflicts" => Some(FeasibilityConcern::DependencyConflicts),
                    "security_constraints" => Some(FeasibilityConcern::SecurityConstraints),
                    "timeline_constraints" => Some(FeasibilityConcern::TimelineConstraints),
                    _ => None,
                }
            })
            .collect();

        let domain_expertise = analysis["domain_expertise"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|de| {
                DomainExpertise {
                    domain: de["domain"].as_str().unwrap_or("Unknown").to_string(),
                    expertise_level: de["expertise_level"].as_u64().unwrap_or(3) as u8,
                    available_internally: de["available_internally"].as_bool().unwrap_or(true),
                    acquisition_time_weeks: de["acquisition_time_weeks"].as_u64().map(|w| w as u8),
                }
            })
            .collect();

        // Parse resource requirements
        let rr = &analysis["resource_requirements"];
        let resource_requirements = ResourceRequirements {
            development_hours: rr["development_hours"].as_u64().unwrap_or(40) as u16,
            required_skills: rr["required_skills"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect(),
            infrastructure_needs: rr["infrastructure_needs"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect(),
            external_dependencies: rr["external_dependencies"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect(),
            cost_estimate_usd: (
                rr["cost_min"].as_u64().unwrap_or(1000) as u32,
                rr["cost_max"].as_u64().unwrap_or(5000) as u32,
            ),
        };

        // Parse complexity metrics
        let cm = &analysis["complexity_metrics"];
        let complexity_metrics = ComplexityMetrics {
            cyclomatic_complexity: cm["cyclomatic_complexity"].as_u64().unwrap_or(5) as u8,
            integration_points: cm["integration_points"].as_u64().unwrap_or(2) as u8,
            data_complexity: cm["data_complexity"].as_u64().unwrap_or(3) as u8,
            algorithmic_complexity: cm["algorithmic_complexity"].as_str().unwrap_or("O(n)").to_string(),
            testing_complexity: cm["testing_complexity"].as_f64().unwrap_or(1.0) as f32,
        };

        // Parse performance analysis
        let pa = &analysis["performance_analysis"];
        let performance_feasibility = match pa["feasibility_assessment"].as_str().unwrap_or("feasible") {
            "feasible" => PerformanceFeasibility::Feasible,
            "challenging" => PerformanceFeasibility::Challenging,
            "specialized" => PerformanceFeasibility::Specialized,
            "impossible" => PerformanceFeasibility::Impossible,
            _ => PerformanceFeasibility::Feasible,
        };

        let performance_analysis = PerformanceAnalysis {
            required_latency_us: pa["required_latency_us"].as_u64(),
            required_throughput: pa["required_throughput"].as_u64().map(|t| t as u32),
            memory_requirements_gb: pa["memory_requirements_gb"].as_f64().map(|m| m as f32),
            network_requirements_mbps: pa["network_requirements_mbps"].as_u64().map(|n| n as u32),
            feasibility_assessment: performance_feasibility,
            risk_factors: pa["risk_factors"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|r| r.as_str().map(|s| s.to_string()))
                .collect(),
        };

        let risk_mitigations = analysis["risk_mitigations"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| r.as_str().map(|s| s.to_string()))
            .collect();

        Ok(FeasibilityAssessment {
            feasibility_score,
            feasibility_concerns,
            domain_expertise,
            resource_requirements,
            complexity_metrics,
            performance_analysis,
            risk_mitigations,
            assessment_confidence: 0.82, // LLM-based assessment confidence
        })
    }

    /// Generate comprehensive risk assessment combining ambiguity and feasibility
    pub async fn assess_risks(&self, task_description: &str) -> Result<ComprehensiveRiskAssessment> {
        let ambiguity = self.assess_ambiguity(task_description).await?;
        let feasibility = self.assess_feasibility(task_description).await?;

        // Calculate overall risk score based on multiple factors
        let overall_risk_score = self.calculate_overall_risk(&ambiguity, &feasibility);

        let risk_factors = self.identify_risk_factors(&ambiguity, &feasibility);
        let mitigation_strategies = self.generate_mitigation_strategies(&ambiguity, &feasibility);
        let contingency_plans = self.generate_contingency_plans(&ambiguity, &feasibility);

        Ok(ComprehensiveRiskAssessment {
            overall_risk_score,
            ambiguity_assessment: ambiguity,
            feasibility_assessment: feasibility,
            risk_factors,
            mitigation_strategies,
            contingency_plans,
            recommended_approach: self.recommend_approach(overall_risk_score),
        })
    }

    /// Calculate overall risk score from multiple assessment dimensions
    fn calculate_overall_risk(&self, ambiguity: &AmbiguityAssessment, feasibility: &FeasibilityAssessment) -> f32 {
        // Weighted combination of different risk factors
        let ambiguity_weight = 0.3;
        let feasibility_weight = 0.4;
        let concern_weight = 0.3;

        let ambiguity_risk = ambiguity.ambiguity_score;
        let feasibility_risk = 1.0 - feasibility.feasibility_score;
        let concern_risk = (feasibility.feasibility_concerns.len() as f32).min(1.0);

        (ambiguity_risk * ambiguity_weight) +
        (feasibility_risk * feasibility_weight) +
        (concern_risk * concern_weight)
    }

    /// Identify all risk factors across assessments
    fn identify_risk_factors(&self, ambiguity: &AmbiguityAssessment, feasibility: &FeasibilityAssessment) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        // Add ambiguity-based risk factors
        for ambiguity_type in &ambiguity.ambiguity_types {
            let (severity, description) = match ambiguity_type {
                AmbiguityType::VagueSubject => (RiskSeverity::High, "Unclear subject/object may lead to incorrect implementation".to_string()),
                AmbiguityType::UnclearObjectives => (RiskSeverity::High, "Undefined success criteria may result in unmet expectations".to_string()),
                AmbiguityType::UndefinedScope => (RiskSeverity::Medium, "Missing scope boundaries may cause scope creep".to_string()),
                AmbiguityType::TechnicalAmbiguity => (RiskSeverity::Medium, "Technical requirements unclear may require rework".to_string()),
                AmbiguityType::ContextualGaps => (RiskSeverity::Medium, "Missing context about existing systems".to_string()),
                AmbiguityType::MultipleInterpretations => (RiskSeverity::High, "Multiple interpretations possible".to_string()),
                AmbiguityType::IncompleteRequirements => (RiskSeverity::High, "Requirements incomplete, high risk of missing functionality".to_string()),
            };
            factors.push(RiskFactor {
                factor_type: RiskFactorType::Ambiguity,
                severity,
                description,
                impact_probability: 0.8,
            });
        }

        // Add feasibility-based risk factors
        for concern in &feasibility.feasibility_concerns {
            let (severity, description) = match concern {
                FeasibilityConcern::DomainExpertiseGap => (RiskSeverity::Critical, "Required domain expertise not available".to_string()),
                FeasibilityConcern::TechnicalImpossibility => (RiskSeverity::Critical, "Technical implementation impossible".to_string()),
                FeasibilityConcern::PerformanceUnrealistic => (RiskSeverity::High, "Performance requirements unrealistic".to_string()),
                FeasibilityConcern::ResourceConstraints => (RiskSeverity::High, "Resource requirements exceed capacity".to_string()),
                FeasibilityConcern::DependencyConflicts => (RiskSeverity::High, "Required dependencies incompatible".to_string()),
                FeasibilityConcern::SecurityConstraints => (RiskSeverity::Medium, "Security requirements may limit functionality".to_string()),
                FeasibilityConcern::TimelineConstraints => (RiskSeverity::Medium, "Timeline too aggressive for scope".to_string()),
            };
            factors.push(RiskFactor {
                factor_type: RiskFactorType::Feasibility,
                severity,
                description,
                impact_probability: 0.9,
            });
        }

        factors
    }

    /// Generate mitigation strategies for identified risks
    fn generate_mitigation_strategies(&self, ambiguity: &AmbiguityAssessment, feasibility: &FeasibilityAssessment) -> Vec<String> {
        let mut strategies = Vec::new();

        if ambiguity.clarification_required {
            strategies.push("Implement clarification workflow to resolve ambiguity before implementation".to_string());
        }

        if ambiguity.ambiguity_score > 0.7 {
            strategies.push("Conduct stakeholder workshops to align on requirements and objectives".to_string());
        }

        for concern in &feasibility.feasibility_concerns {
            match concern {
                FeasibilityConcern::DomainExpertiseGap => {
                    strategies.push("Engage domain experts or provide training for required expertise".to_string());
                },
                FeasibilityConcern::TechnicalImpossibility => {
                    strategies.push("Re-evaluate requirements and consider alternative approaches".to_string());
                },
                FeasibilityConcern::PerformanceUnrealistic => {
                    strategies.push("Conduct performance prototyping and optimization planning".to_string());
                },
                FeasibilityConcern::ResourceConstraints => {
                    strategies.push("Scale up infrastructure or optimize resource usage patterns".to_string());
                },
                FeasibilityConcern::DependencyConflicts => {
                    strategies.push("Research alternative dependencies or fork/modify incompatible libraries".to_string());
                },
                _ => {}
            }
        }

        strategies
    }

    /// Generate contingency plans for high-risk scenarios
    fn generate_contingency_plans(&self, ambiguity: &AmbiguityAssessment, feasibility: &FeasibilityAssessment) -> Vec<String> {
        let mut plans = Vec::new();

        if ambiguity.ambiguity_score > 0.8 {
            plans.push("Contingency: If clarification fails, implement MVP with assumption documentation and validation checkpoints".to_string());
        }

        if feasibility.feasibility_score < 0.3 {
            plans.push("Contingency: If feasibility assessment shows high risk, prepare fallback implementation with reduced scope".to_string());
        }

        if feasibility.feasibility_concerns.iter().any(|c| matches!(c, FeasibilityConcern::DomainExpertiseGap)) {
            plans.push("Contingency: If expertise gap cannot be filled, partner with external consultants or reduce scope".to_string());
        }

        if feasibility.performance_analysis.feasibility_assessment == PerformanceFeasibility::Impossible {
            plans.push("Contingency: If performance requirements impossible, negotiate relaxed SLAs with stakeholders".to_string());
        }

        plans
    }

    /// Recommend implementation approach based on overall risk
    fn recommend_approach(&self, risk_score: f32) -> RecommendedApproach {
        match risk_score {
            r if r < 0.3 => RecommendedApproach::DirectImplementation,
            r if r < 0.6 => RecommendedApproach::PhasedImplementation,
            r if r < 0.8 => RecommendedApproach::PrototypeFirst,
            _ => RecommendedApproach::ReconsiderRequirements,
        }
    }

    /// Validate domain expertise requirements for a task
    pub async fn validate_domain_expertise(&self, task_description: &str) -> Result<DomainExpertiseValidation> {
        tracing::info!("Validating domain expertise for task: {}", task_description);

        // Define domain expertise requirements for common technical areas
        let domain_requirements = self.get_domain_expertise_requirements();

        // Analyze task for domain requirements using LLM
        let analysis_prompt = format!(
            "Analyze the following task for required domain expertise. \
             Consider specialized knowledge areas, technical domains, and expertise levels needed. \
             Map to these expertise areas: {}\n\n\
             Task: {}\n\n\
             Provide analysis in JSON format with required_domains array, expertise_levels object, \
             available_expertise object, and acquisition_assessment.",
            domain_requirements.keys().collect::<Vec<_>>().join(", "),
            task_description
        );

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a technical architect assessing domain expertise requirements. Always respond with valid JSON.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: analysis_prompt,
            }
        ];

        let request = GenerationRequest {
            messages,
            temperature: 0.1,
            max_tokens: 800,
            ..Default::default()
        };

        let response = self.llm_client.generate(request).await?;
        let analysis: serde_json::Value = serde_json::from_str(&response.content)
            .map_err(|e| PlanningError::LLMError(anyhow::anyhow!("Failed to parse expertise analysis: {}", e)))?;

        // Parse domain requirements
        let required_domains = analysis["required_domains"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|d| d.as_str().map(|s| s.to_string()))
            .collect::<Vec<String>>();

        // Parse expertise levels
        let expertise_levels = analysis["expertise_levels"]
            .as_object()
            .unwrap_or(&serde_json::Map::new())
            .iter()
            .filter_map(|(domain, level)| {
                level.as_u64().map(|l| (domain.clone(), l as u8))
            })
            .collect::<HashMap<String, u8>>();

        // Parse available expertise (assume current team capabilities)
        let available_expertise = analysis["available_expertise"]
            .as_object()
            .unwrap_or(&serde_json::Map::new())
            .iter()
            .filter_map(|(domain, available)| {
                available.as_bool().map(|a| (domain.clone(), a))
            })
            .collect::<HashMap<String, bool>>();

        // Parse acquisition assessment
        let acquisition_assessment = analysis["acquisition_assessment"]
            .as_object()
            .unwrap_or(&serde_json::Map::new());

        let acquisition_feasible = acquisition_assessment["feasible"].as_bool().unwrap_or(true);
        let acquisition_time_weeks = acquisition_assessment["time_weeks"].as_u64().map(|w| w as u8);
        let acquisition_cost = acquisition_assessment["cost_estimate"].as_str().map(|s| s.to_string());

        Ok(DomainExpertiseValidation {
            required_domains,
            expertise_levels,
            available_expertise,
            acquisition_feasible,
            acquisition_time_weeks,
            acquisition_cost,
        })
    }

    /// Get predefined domain expertise requirements
    fn get_domain_expertise_requirements(&self) -> HashMap<String, DomainRequirement> {
        let mut requirements = HashMap::new();

        requirements.insert("cryptography".to_string(), DomainRequirement {
            description: "Cryptographic algorithms, security protocols, key management".to_string(),
            typical_expertise_levels: vec![3, 4, 5], // Expert to world-class
            common_technologies: vec!["AES", "RSA", "ECC", "quantum-resistant", "PKI"].into_iter().map(|s| s.to_string()).collect(),
        });

        requirements.insert("quantum_computing".to_string(), DomainRequirement {
            description: "Quantum algorithms, quantum information theory, quantum hardware".to_string(),
            typical_expertise_levels: vec![5], // World-class only
            common_technologies: vec!["Shor's algorithm", "Grover's algorithm", "quantum gates", "QKD"].into_iter().map(|s| s.to_string()).collect(),
        });

        requirements.insert("distributed_systems".to_string(), DomainRequirement {
            description: "Distributed consensus, fault tolerance, scalability patterns".to_string(),
            typical_expertise_levels: vec![2, 3, 4, 5],
            common_technologies: vec!["Raft", "Paxos", "CAP theorem", " Byzantine fault tolerance"].into_iter().map(|s| s.to_string()).collect(),
        });

        requirements.insert("blockchain".to_string(), DomainRequirement {
            description: "Blockchain protocols, smart contracts, decentralized systems".to_string(),
            typical_expertise_levels: vec![2, 3, 4],
            common_technologies: vec!["Bitcoin", "Ethereum", "smart contracts", "consensus mechanisms"].into_iter().map(|s| s.to_string()).collect(),
        });

        requirements.insert("machine_learning".to_string(), DomainRequirement {
            description: "ML algorithms, model training, optimization techniques".to_string(),
            typical_expertise_levels: vec![2, 3, 4, 5],
            common_technologies: vec!["neural networks", "gradient descent", "backpropagation", "transformers"].into_iter().map(|s| s.to_string()).collect(),
        });

        requirements.insert("performance_engineering".to_string(), DomainRequirement {
            description: "High-performance systems, optimization, low-latency design".to_string(),
            typical_expertise_levels: vec![3, 4, 5],
            common_technologies: vec!["JIT compilation", "SIMD", "memory hierarchies", "lock-free algorithms"].into_iter().map(|s| s.to_string()).collect(),
        });

        requirements.insert("security_hardening".to_string(), DomainRequirement {
            description: "Security architecture, threat modeling, secure coding practices".to_string(),
            typical_expertise_levels: vec![2, 3, 4],
            common_technologies: vec!["OWASP", "threat modeling", "secure SDLC", "cryptography"].into_iter().map(|s| s.to_string()).collect(),
        });

        requirements
    }

    /// Evaluate mathematical complexity of a task
    pub async fn evaluate_mathematical_complexity(&self, task_description: &str) -> Result<MathematicalComplexity> {
        tracing::info!("Evaluating mathematical complexity for task: {}", task_description);

        // Analyze task for mathematical complexity patterns
        let complexity_patterns = self.identify_complexity_patterns(task_description);

        // Use LLM to assess mathematical complexity
        let analysis_prompt = format!(
            "Analyze the mathematical complexity of the following task. \
             Consider algorithmic complexity, mathematical proofs required, computational complexity classes, \
             numerical stability, and mathematical maturity needed.\n\n\
             Task: {}\n\n\
             Identified patterns: {}\n\n\
             Provide analysis in JSON format with complexity_class (constant|logarithmic|linear|polynomial|exponential|undecidable), \
             mathematical_maturity_level (1-5), proof_complexity, numerical_stability_concerns, \
             and implementation_challenges array.",
            task_description,
            complexity_patterns.join(", ")
        );

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a computational complexity theorist analyzing mathematical requirements. Always respond with valid JSON.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: analysis_prompt,
            }
        ];

        let request = GenerationRequest {
            messages,
            temperature: 0.1,
            max_tokens: 600,
            ..Default::default()
        };

        let response = self.llm_client.generate(request).await?;
        let analysis: serde_json::Value = serde_json::from_str(&response.content)
            .map_err(|e| PlanningError::LLMError(anyhow::anyhow!("Failed to parse complexity analysis: {}", e)))?;

        let complexity_class = match analysis["complexity_class"].as_str().unwrap_or("polynomial") {
            "constant" => ComputationalComplexity::Constant,
            "logarithmic" => ComputationalComplexity::Logarithmic,
            "linear" => ComputationalComplexity::Linear,
            "polynomial" => ComputationalComplexity::Polynomial,
            "exponential" => ComputationalComplexity::Exponential,
            "undecidable" => ComputationalComplexity::Undecidable,
            _ => ComputationalComplexity::Polynomial,
        };

        let mathematical_maturity = analysis["mathematical_maturity_level"].as_u64().unwrap_or(3) as u8;
        let proof_complexity = analysis["proof_complexity"].as_str().unwrap_or("moderate").to_string();
        let numerical_stability = analysis["numerical_stability_concerns"].as_bool().unwrap_or(false);

        let implementation_challenges = analysis["implementation_challenges"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|c| c.as_str().map(|s| s.to_string()))
            .collect();

        Ok(MathematicalComplexity {
            complexity_class,
            mathematical_maturity_level: mathematical_maturity,
            proof_complexity,
            numerical_stability_concerns: numerical_stability,
            implementation_challenges,
            identified_patterns: complexity_patterns,
        })
    }

    /// Identify complexity patterns in task description
    fn identify_complexity_patterns(&self, task_description: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let desc = task_description.to_lowercase();

        // Cryptographic complexity patterns
        if desc.contains("quantum") && desc.contains("resistant") {
            patterns.push("post-quantum cryptography".to_string());
        }
        if desc.contains("zero-knowledge") || desc.contains("zkp") {
            patterns.push("zero-knowledge proofs".to_string());
        }
        if desc.contains("homomorphic") {
            patterns.push("homomorphic encryption".to_string());
        }

        // Algorithmic complexity patterns
        if desc.contains("optimization") && desc.contains("np") {
            patterns.push("NP-complete optimization".to_string());
        }
        if desc.contains("graph") && (desc.contains("algorithm") || desc.contains("traversal")) {
            patterns.push("graph algorithms".to_string());
        }
        if desc.contains("sorting") && desc.contains("comparison") {
            patterns.push("comparison-based sorting".to_string());
        }

        // Mathematical complexity patterns
        if desc.contains("prime") && desc.contains("generation") {
            patterns.push("prime number generation".to_string());
        }
        if desc.contains("floating") && desc.contains("precision") {
            patterns.push("floating-point precision".to_string());
        }
        if desc.contains("numerical") && desc.contains("stability") {
            patterns.push("numerical stability".to_string());
        }

        // Performance complexity patterns
        if desc.contains("microsecond") || desc.contains("nanosecond") {
            patterns.push("extreme low-latency requirements".to_string());
        }
        if desc.contains("real-time") && desc.contains("processing") {
            patterns.push("real-time processing constraints".to_string());
        }

        if patterns.is_empty() {
            patterns.push("general algorithmic complexity".to_string());
        }

        patterns
    }

    /// Model performance feasibility for a task
    pub async fn model_performance_feasibility(&self, task_description: &str) -> Result<PerformanceFeasibilityModel> {
        tracing::info!("Modeling performance feasibility for task: {}", task_description);

        // Extract performance requirements from task
        let perf_requirements = self.extract_performance_requirements(task_description);

        // Analyze hardware constraints
        let hardware_constraints = self.analyze_hardware_constraints(&perf_requirements);

        // Model theoretical performance bounds
        let theoretical_bounds = self.calculate_theoretical_bounds(&perf_requirements);

        // Assess practical achievability
        let practical_assessment = self.assess_practical_achievability(&perf_requirements, &hardware_constraints);

        // Generate optimization recommendations
        let optimization_recommendations = self.generate_performance_optimizations(&perf_requirements);

        Ok(PerformanceFeasibilityModel {
            extracted_requirements: perf_requirements,
            hardware_constraints,
            theoretical_bounds,
            practical_assessment,
            optimization_recommendations,
        })
    }

    /// Extract performance requirements from task description
    fn extract_performance_requirements(&self, task_description: &str) -> PerformanceRequirements {
        let desc = task_description.to_lowercase();

        // Extract latency requirements
        let latency_us = if desc.contains("microsecond") {
            desc.split_whitespace()
                .find(|w| w.parse::<u64>().is_ok() && desc.contains("microsecond"))
                .and_then(|w| w.parse().ok())
                .unwrap_or(1000) // 1ms default for microsecond mentions
        } else if desc.contains("millisecond") || desc.contains("ms") {
            desc.split_whitespace()
                .find(|w| w.parse::<u64>().is_ok())
                .and_then(|w| w.parse().ok())
                .map(|ms| ms * 1000) // convert to microseconds
                .unwrap_or(1000000) // 1 second default
        } else {
            None
        };

        // Extract throughput requirements
        let throughput_ops = if desc.contains("per second") || desc.contains("/s") {
            desc.split_whitespace()
                .find(|w| w.parse::<u32>().is_ok())
                .and_then(|w| w.parse().ok())
        } else {
            None
        };

        // Extract memory requirements
        let memory_gb = if desc.contains("gb") || desc.contains("gigabyte") {
            desc.split_whitespace()
                .find(|w| w.parse::<f32>().is_ok())
                .and_then(|w| w.parse().ok())
        } else {
            None
        };

        // Extract network requirements
        let network_mbps = if desc.contains("mbps") || desc.contains("mb/s") {
            desc.split_whitespace()
                .find(|w| w.parse::<u32>().is_ok())
                .and_then(|w| w.parse().ok())
        } else {
            None
        };

        PerformanceRequirements {
            latency_microseconds: latency_us,
            throughput_operations_per_second: throughput_ops,
            memory_requirements_gb: memory_gb,
            network_bandwidth_mbps: network_mbps,
        }
    }

    /// Analyze hardware constraints for performance requirements
    fn analyze_hardware_constraints(&self, requirements: &PerformanceRequirements) -> HardwareConstraints {
        let mut constraints = Vec::new();

        if let Some(latency) = requirements.latency_microseconds {
            if latency < 1000 { // Sub-millisecond
                constraints.push("Requires specialized hardware (FPGA, ASIC, or custom silicon)".to_string());
            } else if latency < 10000 { // Sub-10ms
                constraints.push("Requires high-performance CPU with optimized memory access".to_string());
            }
        }

        if let Some(throughput) = requirements.throughput_operations_per_second {
            if throughput > 1000000 { // 1M+ ops/sec
                constraints.push("Requires parallel processing (GPU, multi-core CPU)".to_string());
            } else if throughput > 100000 { // 100K+ ops/sec
                constraints.push("Requires optimized single-threaded performance".to_string());
            }
        }

        if let Some(memory) = requirements.memory_requirements_gb {
            if memory > 512.0 {
                constraints.push("Requires high-memory server or distributed architecture".to_string());
            } else if memory > 64.0 {
                constraints.push("Requires workstation-grade memory (64GB+)".to_string());
            }
        }

        if let Some(network) = requirements.network_bandwidth_mbps {
            if network > 10000 { // 10Gbps+
                constraints.push("Requires high-bandwidth network infrastructure".to_string());
            } else if network > 1000 { // 1Gbps+
                constraints.push("Requires enterprise-grade networking".to_string());
            }
        }

        HardwareConstraints {
            identified_constraints: constraints,
            recommended_hardware: self.recommend_hardware(requirements),
            cost_implications: self.estimate_hardware_cost(requirements),
        }
    }

    /// Recommend hardware for performance requirements
    fn recommend_hardware(&self, requirements: &PerformanceRequirements) -> Vec<String> {
        let mut recommendations = Vec::new();

        if requirements.latency_microseconds.unwrap_or(u64::MAX) < 10000 {
            recommendations.push("Apple Silicon M2/M3 Ultra (for Neural Engine acceleration)".to_string());
            recommendations.push("NVIDIA RTX 40-series GPU (for CUDA acceleration)".to_string());
        }

        if requirements.throughput_operations_per_second.unwrap_or(0) > 500000 {
            recommendations.push("Multi-core CPU (16+ cores) with high clock speed".to_string());
            recommendations.push("NVIDIA A100/H100 GPU for parallel processing".to_string());
        }

        if requirements.memory_requirements_gb.unwrap_or(0.0) > 128.0 {
            recommendations.push("Server with 512GB+ RAM".to_string());
            recommendations.push("Consider distributed architecture".to_string());
        }

        if requirements.network_bandwidth_mbps.unwrap_or(0) > 5000 {
            recommendations.push("10Gbps+ network infrastructure".to_string());
            recommendations.push("RDMA-enabled networking for low latency".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Standard workstation/server hardware sufficient".to_string());
        }

        recommendations
    }

    /// Estimate hardware cost implications
    fn estimate_hardware_cost(&self, requirements: &PerformanceRequirements) -> String {
        let base_cost = if requirements.latency_microseconds.unwrap_or(u64::MAX) < 1000 {
            50000..150000 // Custom hardware
        } else if requirements.memory_requirements_gb.unwrap_or(0.0) > 256.0 {
            10000..30000 // High-memory server
        } else if requirements.throughput_operations_per_second.unwrap_or(0) > 1000000 {
            5000..15000 // GPU workstation
        } else {
            2000..8000 // Standard workstation
        };

        format!("${}-${}", base_cost.start, base_cost.end)
    }

    /// Calculate theoretical performance bounds
    fn calculate_theoretical_bounds(&self, requirements: &PerformanceRequirements) -> TheoreticalBounds {
        // Simplified theoretical calculations
        let theoretical_latency = requirements.latency_microseconds
            .map(|req| {
                // Light speed limit for network operations
                let light_speed_limit = 300000000.0 / 2.0; // speed of light round trip / 2
                let theoretical_min = (1.0 / 3000000000.0) * 1000000.0; // 1 GHz theoretical min

                TheoreticalLatency {
                    requested: req,
                    theoretical_minimum: theoretical_min as u64,
                    light_speed_limit: light_speed_limit as u64,
                    achievable: req >= theoretical_min as u64,
                }
            });

        let theoretical_throughput = requirements.throughput_operations_per_second
            .map(|req| {
                // CPU throughput limits
                let max_cpu_throughput = 100000000; // 100M ops/sec theoretical max
                let practical_cpu_limit = 10000000; // 10M ops/sec practical limit

                TheoreticalThroughput {
                    requested: req,
                    theoretical_maximum: max_cpu_throughput,
                    practical_limit: practical_cpu_limit,
                    achievable: req <= practical_cpu_limit,
                }
            });

        TheoreticalBounds {
            latency_bounds: theoretical_latency,
            throughput_bounds: theoretical_throughput,
        }
    }

    /// Assess practical achievability of performance requirements
    fn assess_practical_achievability(&self, requirements: &PerformanceRequirements, constraints: &HardwareConstraints) -> PracticalAssessment {
        let mut feasibility_score = 1.0;
        let mut concerns = Vec::new();

        // Assess latency feasibility
        if let Some(latency) = requirements.latency_microseconds {
            if latency < 100 { // Sub-100μs
                feasibility_score *= 0.1;
                concerns.push("Sub-100μs latency requires custom hardware or extreme optimization".to_string());
            } else if latency < 1000 { // Sub-1ms
                feasibility_score *= 0.5;
                concerns.push("Sub-1ms latency challenging but achievable with optimization".to_string());
            }
        }

        // Assess throughput feasibility
        if let Some(throughput) = requirements.throughput_operations_per_second {
            if throughput > 50000000 { // 50M+ ops/sec
                feasibility_score *= 0.2;
                concerns.push("50M+ ops/sec requires specialized parallel hardware".to_string());
            } else if throughput > 10000000 { // 10M+ ops/sec
                feasibility_score *= 0.6;
                concerns.push("10M+ ops/sec requires high-end parallel processing".to_string());
            }
        }

        // Assess memory feasibility
        if let Some(memory) = requirements.memory_requirements_gb {
            if memory > 1024.0 { // 1TB+
                feasibility_score *= 0.3;
                concerns.push("1TB+ memory requires specialized server hardware".to_string());
            } else if memory > 256.0 { // 256GB+
                feasibility_score *= 0.7;
                concerns.push("256GB+ memory requires high-end server".to_string());
            }
        }

        PracticalAssessment {
            feasibility_score,
            identified_concerns: concerns,
            recommended_approach: if feasibility_score > 0.8 {
                "Standard implementation with performance monitoring".to_string()
            } else if feasibility_score > 0.5 {
                "Optimized implementation with performance profiling".to_string()
            } else {
                "Prototype and benchmark before full implementation".to_string()
            },
        }
    }

    /// Generate performance optimization recommendations
    fn generate_performance_optimizations(&self, requirements: &PerformanceRequirements) -> Vec<String> {
        let mut recommendations = Vec::new();

        if requirements.latency_microseconds.is_some() {
            recommendations.push("Use zero-copy data structures".to_string());
            recommendations.push("Implement SIMD instructions where applicable".to_string());
            recommendations.push("Optimize memory access patterns (cache-friendly)".to_string());
            recommendations.push("Consider lock-free algorithms for concurrency".to_string());
        }

        if requirements.throughput_operations_per_second.is_some() {
            recommendations.push("Implement parallel processing with async/await".to_string());
            recommendations.push("Use vectorized operations (SIMD)".to_string());
            recommendations.push("Consider GPU acceleration for compute-intensive tasks".to_string());
            recommendations.push("Implement work-stealing schedulers".to_string());
        }

        if requirements.memory_requirements_gb.is_some() {
            recommendations.push("Implement memory pooling to reduce allocation overhead".to_string());
            recommendations.push("Use compact data structures".to_string());
            recommendations.push("Consider memory-mapped files for large datasets".to_string());
        }

        if requirements.network_bandwidth_mbps.is_some() {
            recommendations.push("Implement connection pooling".to_string());
            recommendations.push("Use compression for data transfer".to_string());
            recommendations.push("Consider protocol buffers over JSON".to_string());
        }

        recommendations
    }

    /// Generate a working spec from a natural language task description with ambiguity handling
    pub async fn generate_working_spec(
        &self,
        task_description: &str,
        context: TaskContext,
    ) -> Result<WorkingSpecResult> {
        tracing::info!("Generating working spec for task: {}", task_description);

        // First, assess task ambiguity
        let ambiguity_assessment = self.assess_ambiguity(task_description).await?;

        // If clarification is required, return clarification request
        if ambiguity_assessment.clarification_required {
            let session = self.initiate_clarification(task_description, &ambiguity_assessment).await?;
            return Ok(WorkingSpecResult::ClarificationNeeded {
                assessment: ambiguity_assessment,
                session,
            });
        }

        // If no clarification needed, proceed with normal planning
        self.generate_working_spec_internal(task_description, context).await
    }

    /// Generate working spec with clarified task description
    pub async fn generate_working_spec_with_clarification(
        &self,
        session: &ClarificationSession,
        context: TaskContext,
    ) -> Result<WorkingSpec> {
        // Enrich the task description with clarification responses
        let enriched_task = self.enrich_task_description(&session.original_task, session);
        tracing::info!("Generating working spec with clarified task: {}", enriched_task);

        self.generate_working_spec_internal(&enriched_task, context).await
            .map(|result| {
                if let WorkingSpecResult::Success(spec) = result {
                    spec
                } else {
                    panic!("Internal error: expected success result from clarified task");
                }
            })
    }

    /// Internal method for generating working spec (assumes clarification is handled)
    async fn generate_working_spec_internal(
        &self,
        task_description: &str,
        context: TaskContext,
    ) -> Result<WorkingSpecResult> {
        tracing::info!("Generating working spec internally for task: {}", task_description);

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
        Ok(WorkingSpecResult::Success(final_spec))
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
