//! Core Planning Agent implementation
//!
//! The PlanningAgent is responsible for transforming TaskRequests into
//! validated WorkingSpecs through a multi-stage planning pipeline.

use schemars::JsonSchema;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use std::collections::{HashMap, HashSet, VecDeque};
use regex::Regex;
use once_cell::sync::Lazy;
use strsim::{jaro_winkler, levenshtein};
use priority_queue::PriorityQueue;
use lru::LruCache;
use serde::{Deserialize, Serialize};

use crate::planning_agent::planning_errors::{PlanningError, PlanningResult};
use crate::planning_agent::planning_caws_integration::CawsValidator;
use crate::planning_agent::validation_pipeline::ValidationPipeline;
use crate::planning_agent::refinement_engine::RefinementEngine;
use crate::planning_agent::types::{
    PlanningConfig, PlanningRequest, PlanningResponse, PlanningMetadata,
    ValidationResults, RefinementRecord, RiskAssessment,
};
use system_configuration::types::*;
use crate::planning_agent::validation::*;
use crate::planning_agent::spec_generation::*;

/// The main Planning Agent
pub struct PlanningAgent {
    config: PlanningConfig,
    caws_validator: Arc<dyn CawsValidator>,
    validation_pipeline: Arc<ValidationPipeline>,
    refinement_engine: Arc<dyn RefinementEngine>,
}

impl PlanningAgent {
    /// Create a new planning agent
    pub fn new(
        config: PlanningConfig,
        caws_validator: Arc<dyn CawsValidator>,
        validation_pipeline: Arc<ValidationPipeline>,
        refinement_engine: Arc<dyn RefinementEngine>,
    ) -> Self {
        Self {
            config,
            caws_validator,
            validation_pipeline,
            refinement_engine,
        }
    }

    /// Plan a task by transforming TaskRequest into validated WorkingSpec
    pub async fn plan_task(&self, request: PlanningRequest) -> PlanningResult<PlanningResponse> {
        let start_time = std::time::Instant::now();
        let config = request.config_override.unwrap_or_else(|| self.config.clone());

        // Validate input task request
        validate_task_request(&request.task_request)?;

        // Perform risk assessment
        let risk_assessment = assess_risk(&request.task_request)?;

        // Check for immediate escalation
        if risk_assessment.escalation_recommended {
            return Err(PlanningError::RiskEscalation {
                reason: format!("Risk assessment indicates escalation required: {:?}", risk_assessment.risk_factors),
            });
        }

        // Generate initial working specification
        let mut working_spec = generate_working_spec(&request.task_request).await?;

        // Run planning pipeline with validation and refinement
        let planning_result = timeout(
            Duration::from_secs(config.max_planning_time_seconds),
            self.run_planning_pipeline(&mut working_spec, &config)
        ).await
        .map_err(|_| PlanningError::Timeout(format!("Planning exceeded {} seconds", config.max_planning_time_seconds)))?;

        let (validation_results, refinement_history) = planning_result?;

        // Create response metadata
        let metadata = PlanningMetadata {
            planning_duration: start_time.elapsed(),
            refinement_iterations: refinement_history.len() as u32,
            human_intervention_required: validation_results.overall_status == crate::planning_agent::types::planning_types::ValidationStatus::EscalationRequired,
            risk_assessment,
        };

        Ok(PlanningResponse {
            working_spec,
            metadata,
            validation_results,
            refinement_history,
        })
    }


    /// Run the complete planning pipeline with validation and refinement
    async fn run_planning_pipeline(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        config: &PlanningConfig,
    ) -> PlanningResult<(ValidationResults, Vec<RefinementRecord>)> {
        let mut refinement_history = Vec::new();
        let mut iteration = 0u32;

        loop {
            iteration += 1;

            // Run validation pipeline
            let validation_result = self.validation_pipeline.validate_working_spec(working_spec).await?;

            // Check if validation passed
            if validation_result.overall_status == crate::planning_agent::types::planning_types::ValidationStatus::Passed {
                return Ok((validation_result, refinement_history));
            }

            // Check if we've exceeded max iterations
            if iteration >= config.max_refinement_iterations {
                let mut validation_results = validation_result;
                validation_results.overall_status = crate::planning_agent::types::planning_types::ValidationStatus::EscalationRequired;
                return Ok((validation_results, refinement_history));
            }

            // Check if refinement is disabled
            if !config.enable_auto_refinement {
                let mut validation_results = validation_result;
                validation_results.overall_status = crate::planning_agent::types::planning_types::ValidationStatus::EscalationRequired;
                return Ok((validation_results, refinement_history));
            }

            // Attempt refinement
            let refinement_result = self.refinement_engine.refine_working_spec(
                working_spec,
                &validation_result.issues
            ).await?;

            let record = RefinementRecord {
                iteration,
                triggering_issues: validation_result.issues.iter().map(|i| i.description.clone()).collect(),
                applied_actions: refinement_result.applied_actions,
                successful: refinement_result.successful,
            };

            refinement_history.push(record);

            if !refinement_result.successful {
                // Refinement failed, require escalation
                let mut validation_results = validation_result;
                validation_results.overall_status = crate::planning_agent::types::planning_types::ValidationStatus::EscalationRequired;
                return Ok((validation_results, refinement_history));
            }
        }
    }

    // Helper methods for working spec generation...

    fn extract_goals_from_description(&self, description: &str) -> PlanningResult<Vec<String>> {
        // Extract goals using pattern matching and sentence analysis
        let mut goals: Vec<String> = Vec::new();
        
        // Split into sentences for better goal extraction
        let sentences: Vec<&str> = description.split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for sentence in sentences {
            // Look for action verbs and goal indicators
            let sentence_lower = sentence.to_lowercase();
            
            // Goal patterns: "need to", "should", "must", "implement", "create", "add", "build"
            let goal_keywords = ["need to", "should", "must", "implement", "create", "add", "build", 
                                 "develop", "establish", "set up", "configure", "integrate"];
            
            let is_goal_sentence = goal_keywords.iter().any(|keyword| sentence_lower.contains(keyword));
            
            if is_goal_sentence {
                // Extract goal by finding the core action
                let goal = self.extract_core_goal_from_sentence(sentence);
                if !goal.is_empty() && !goals.contains(&goal) {
                    goals.push(goal);
                }
            }
        }

        // If no goals found via pattern matching, create a high-level goal from description
        if goals.is_empty() {
            // Try to extract key phrases
            let key_phrases = self.extract_key_phrases(description);
            if !key_phrases.is_empty() {
                goals.extend(key_phrases);
            } else {
                // Fallback: create a single goal from the description
                goals.push(format!("Successfully complete: {}", description.trim()));
            }
        }

        // Validate and deduplicate goals
        goals = self.validate_and_deduplicate_goals(goals);

        Ok(goals)
    }

    /// Extract core goal from a sentence
    fn extract_core_goal_from_sentence(&self, sentence: &str) -> String {
        // Remove common prefixes and extract the action
        let mut goal = sentence.trim().to_string();
        
        // Remove leading "we need to", "we should", etc.
        let prefixes = ["we need to ", "we should ", "we must ", "need to ", "should ", "must "];
        for prefix in &prefixes {
            if goal.to_lowercase().starts_with(prefix) {
                goal = goal[prefix.len()..].trim().to_string();
                break;
            }
        }
        
        // Capitalize first letter
        if !goal.is_empty() {
            let mut chars: Vec<char> = goal.chars().collect();
            chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
            goal = chars.into_iter().collect();
        }
        
        goal
    }

    /// Extract key phrases from description
    fn extract_key_phrases(&self, description: &str) -> Vec<String> {
        let mut phrases = Vec::new();
        
        // Look for bullet points or numbered lists
        for line in description.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || 
               trimmed.matches(char::is_numeric).next().is_some() && trimmed.contains('.') {
                let phrase = trimmed
                    .trim_start_matches(|c: char| c == '-' || c == '*' || c == ' ' || c.is_numeric() || c == '.')
                    .trim()
                    .to_string();
                if !phrase.is_empty() {
                    phrases.push(phrase);
                }
            }
        }
        
        phrases
    }

    /// Validate and deduplicate goals
    fn validate_and_deduplicate_goals(&self, goals: Vec<String>) -> Vec<String> {
        let mut unique_goals = Vec::new();
        
        for goal in goals {
            // Skip empty or too short goals
            if goal.trim().len() < 5 {
                continue;
            }
            
            // Check for duplicates using similarity
            let is_duplicate = unique_goals.iter().any(|existing| {
                jaro_winkler(existing, &goal) > 0.85
            });
            
            if !is_duplicate {
                unique_goals.push(goal);
            }
        }
        
        unique_goals
    }

    fn generate_acceptance_criteria(&self, description: &str) -> PlanningResult<Vec<agent_agency_contracts::working_spec::AcceptanceCriterion>> {
        // Generate acceptance criteria based on description analysis
        let mut criteria = Vec::new();
        
        // Extract key actions and outcomes from description
        let sentences: Vec<&str> = description.split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for (idx, sentence) in sentences.iter().enumerate() {
            // Create acceptance criterion for each meaningful sentence
            let sentence_lower = sentence.to_lowercase();
            
            // Skip if it's just a conjunction or filler
            if sentence_lower.len() < 10 || 
               sentence_lower.starts_with("and ") || 
               sentence_lower.starts_with("or ") ||
               sentence_lower.starts_with("but ") {
                continue;
            }
            
            // Extract the "when" condition and "then" outcome
            let (given, when, then) = self.parse_acceptance_criterion(sentence);
            
            criteria.push(agent_agency_contracts::working_spec::AcceptanceCriterion {
                id: format!("A{}", idx + 1),
                given,
                when,
                then,
                priority: self.determine_criterion_priority(sentence),
            });
        }

        // If no criteria extracted, create a default one
        if criteria.is_empty() {
            criteria.push(agent_agency_contracts::working_spec::AcceptanceCriterion {
                id: "A1".to_string(),
                given: "Valid task request".to_string(),
                when: format!("Task is executed: {}", description),
                then: "Task completes successfully with all requirements met".to_string(),
                priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Must),
            });
        }

        Ok(criteria)
    }

    /// Parse a sentence into acceptance criterion components
    fn parse_acceptance_criterion(&self, sentence: &str) -> (String, String, String) {
        let sentence_lower = sentence.to_lowercase();
        
        // Try to find "when" and "then" patterns
        if let Some(when_idx) = sentence_lower.find("when ") {
            let when_part = &sentence[when_idx + 5..];
            let then_part = if let Some(then_idx) = when_part.to_lowercase().find(" then ") {
                &when_part[then_idx + 6..]
            } else {
                "expected outcome occurs"
            };
            
            return (
                "Given the system state".to_string(),
                when_part.trim().to_string(),
                then_part.trim().to_string(),
            );
        }
        
        // Fallback: split sentence into given/when/then
        let words: Vec<&str> = sentence.split_whitespace().collect();
        if words.len() >= 3 {
            let split_point = words.len() / 3;
            let given_words = words[..split_point].join(" ");
            let when_words = words[split_point..split_point * 2].join(" ");
            let then_words = words[split_point * 2..].join(" ");
            
            (
                format!("Given {}", given_words),
                format!("When {}", when_words),
                format!("Then {}", then_words),
            )
        } else {
            (
                "Given valid input".to_string(),
                format!("When {}", sentence),
                "Then the operation completes successfully".to_string(),
            )
        }
    }

    /// Determine priority for acceptance criterion
    fn determine_criterion_priority(&self, sentence: &str) -> Option<agent_agency_contracts::working_spec::MoSCoWPriority> {
        let sentence_lower = sentence.to_lowercase();
        
        // Check for priority indicators
        if sentence_lower.contains("must") || sentence_lower.contains("required") || 
           sentence_lower.contains("critical") || sentence_lower.contains("essential") {
            Some(agent_agency_contracts::working_spec::MoSCoWPriority::Must)
        } else if sentence_lower.contains("should") || sentence_lower.contains("important") {
            Some(agent_agency_contracts::working_spec::MoSCoWPriority::Should)
        } else if sentence_lower.contains("could") || sentence_lower.contains("nice to have") {
            Some(agent_agency_contracts::working_spec::MoSCoWPriority::Could)
        } else {
            Some(agent_agency_contracts::working_spec::MoSCoWPriority::Must) // Default
        }
    }

    fn generate_title_from_description(&self, description: &str) -> String {
        // Simple title generation - take first sentence or truncate
        let first_sentence = description.split('.').next().unwrap_or(description);
        if first_sentence.len() > 80 {
            format!("{}...", &first_sentence[..77])
        } else {
            first_sentence.to_string()
        }
    }

    fn create_working_spec_constraints(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpecConstraints> {
        let constraints = task_request.constraints.as_ref();

        Ok(agent_agency_contracts::working_spec::WorkingSpecConstraints {
            max_duration_minutes: constraints.and_then(|c| c.max_duration_minutes),
            max_iterations: constraints.and_then(|c| c.max_iterations),
            budget_limits: constraints.and_then(|c| c.budget_limits.as_ref()).map(|b| {
                agent_agency_contracts::working_spec::BudgetLimits {
                    max_files: b.max_files,
                    max_loc: b.max_loc,
                }
            }),
            scope_restrictions: constraints.and_then(|c| c.scope_restrictions.as_ref()).map(|s| {
                agent_agency_contracts::working_spec::ScopeRestrictions {
                    allowed_paths: s.allowed_paths.clone(),
                    blocked_paths: s.blocked_paths.clone(),
                }
            }),
        })
    }

    fn generate_test_plan(&self, task_request: &agent_agency_contracts::task_request::TaskRequest, risk_tier: u32) -> PlanningResult<agent_agency_contracts::working_spec::TestPlan> {
        let coverage_targets = match risk_tier {
            1 => Some(agent_agency_contracts::working_spec::CoverageTargets {
                line_coverage: Some(0.9),
                branch_coverage: Some(0.85),
                mutation_score: Some(0.7),
            }),
            2 => Some(agent_agency_contracts::working_spec::CoverageTargets {
                line_coverage: Some(0.8),
                branch_coverage: Some(0.75),
                mutation_score: Some(0.5),
            }),
            _ => Some(agent_agency_contracts::working_spec::CoverageTargets {
                line_coverage: Some(0.7),
                branch_coverage: None,
                mutation_score: None,
            }),
        };

        Ok(agent_agency_contracts::working_spec::TestPlan {
            unit_tests: vec![agent_agency_contracts::working_spec::UnitTestSpec {
                description: "Basic functionality test".to_string(),
                target_function: None,
                test_cases: vec!["happy_path".to_string(), "edge_cases".to_string()],
            }],
            integration_tests: vec![],
            e2e_scenarios: vec![],
            coverage_targets,
        })
    }

    fn generate_rollback_plan(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::RollbackPlan> {
        let risk_tier = task_request.constraints.as_ref()
            .map(|c| c.risk_tier.clone())
            .unwrap_or(agent_agency_contracts::task_request::RiskTier::Tier2);

        let (strategy, data_impact) = match risk_tier {
            agent_agency_contracts::task_request::RiskTier::Tier1 => (
                agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
                agent_agency_contracts::working_spec::DataImpact::Reversible,
            ),
            _ => (
                agent_agency_contracts::working_spec::RollbackStrategy::ManualRevert,
                agent_agency_contracts::working_spec::DataImpact::None,
            ),
        };

        Ok(agent_agency_contracts::working_spec::RollbackPlan {
            strategy,
            automated_steps: vec!["git revert".to_string()],
            manual_steps: vec!["Verify system state".to_string()],
            data_impact,
            downtime_required: Some(matches!(risk_tier, agent_agency_contracts::task_request::RiskTier::Tier1)),
            rollback_window_minutes: Some(30),
        })
    }

    fn create_working_spec_context(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpecContext> {
        let context = task_request.context.as_ref();

        Ok(agent_agency_contracts::working_spec::WorkingSpecContext {
            workspace_root: context
                .map(|c| c.workspace_root.clone())
                .unwrap_or_else(|| "/workspace".to_string()),
            git_branch: context
                .map(|c| c.git_branch.clone())
                .unwrap_or_else(|| "main".to_string()),
            recent_changes: context
                .map(|c| c.recent_changes.iter().map(|change| {
                    agent_agency_contracts::working_spec::FileChange {
                        file: change.file.clone(),
                        change_type: match change.change_type {
                            agent_agency_contracts::task_request::ChangeType::Added => agent_agency_contracts::working_spec::ChangeType::Added,
                            agent_agency_contracts::task_request::ChangeType::Modified => agent_agency_contracts::working_spec::ChangeType::Modified,
                            agent_agency_contracts::task_request::ChangeType::Deleted => agent_agency_contracts::working_spec::ChangeType::Deleted,
                        },
                        timestamp: change.timestamp,
                    }
                }).collect())
                .unwrap_or_default(),
            dependencies: context
                .map(|c| c.dependencies.clone())
                .unwrap_or_default(),
            environment: context
                .map(|c| c.environment.clone())
                .unwrap_or(agent_agency_contracts::task_request::Environment::Development),
        })
    }

}

/// Comprehensive Goal Extraction and Analysis Implementation

/// Configuration for goal analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalAnalysisConfig {
    /// Enable semantic analysis using BERT
    pub enable_semantic_analysis: bool,
    /// Enable dependency analysis
    pub enable_dependency_analysis: bool,
    /// Enable stakeholder analysis
    pub enable_stakeholder_analysis: bool,
    /// Enable constraint validation
    pub enable_constraint_validation: bool,
    /// Maximum goals to extract
    pub max_goals: usize,
    /// Minimum confidence for goal extraction
    pub min_confidence: f64,
    /// Enable ML-based prioritization
    pub enable_ml_prioritization: bool,
    /// Cache size for analysis results
    pub cache_size: usize,
}

/// Result of comprehensive goal analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalAnalysisResult {
    /// Extracted goals with metadata
    pub goals: Vec<ExtractedGoal>,
    /// Goal hierarchy and dependencies
    pub hierarchy: GoalHierarchy,
    /// Prioritized goal ranking
    pub prioritization: GoalPrioritization,
    /// Validation results
    pub validation_results: Vec<GoalValidation>,
    /// Analysis metadata
    pub metadata: GoalAnalysisMetadata,
}

/// Individual extracted goal with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedGoal {
    /// Unique goal identifier
    pub id: String,
    /// Goal text/description
    pub text: String,
    /// Goal type classification
    pub goal_type: GoalType,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Priority score
    pub priority_score: f64,
    /// Business value score
    pub business_value: f64,
    /// Stakeholder importance
    pub stakeholder_importance: f64,
    /// Estimated effort (person-hours)
    pub estimated_effort: f64,
    /// Dependencies on other goals
    pub dependencies: Vec<String>,
    /// Conflicting goals
    pub conflicts: Vec<String>,
    /// Decomposition into sub-goals
    pub sub_goals: Vec<String>,
    /// Success criteria
    pub success_criteria: Vec<String>,
    /// Associated risks
    pub risks: Vec<String>,
    /// Required resources
    pub required_resources: Vec<String>,
    /// Timeline constraints
    pub timeline_constraints: Option<GoalTimeline>,
}

/// Goal type classifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum GoalType {
    /// Functional requirements
    Functional,
    /// Non-functional requirements (performance, security, etc.)
    NonFunctional,
    /// Technical implementation goals
    Technical,
    /// Business/stakeholder goals
    Business,
    /// Quality assurance goals
    Quality,
    /// Integration goals
    Integration,
    /// Maintenance goals
    Maintenance,
    /// Innovation/research goals
    Innovation,
    /// Custom goal type
    Custom(String),
}

/// Timeline constraints for goals
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalTimeline {
    /// Target completion date
    pub target_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Estimated duration (days)
    pub estimated_duration_days: f64,
    /// Critical path indicator
    pub is_critical_path: bool,
    /// Milestone dependencies
    pub milestone_dependencies: Vec<String>,
}

/// Goal hierarchy with dependencies
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalHierarchy {
    /// Root goals (no dependencies)
    pub root_goals: Vec<String>,
    /// Goal dependency graph
    pub dependency_graph: HashMap<String, Vec<String>>,
    /// Goal conflict graph
    pub conflict_graph: HashMap<String, Vec<String>>,
    /// Hierarchical levels
    pub hierarchy_levels: Vec<Vec<String>>,
    /// Circular dependency alerts
    pub circular_dependencies: Vec<Vec<String>>,
}

/// Goal prioritization results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalPrioritization {
    /// Goals ranked by priority score
    pub ranked_goals: Vec<String>,
    /// Priority score breakdown
    pub priority_breakdown: HashMap<String, PriorityBreakdown>,
    /// Stakeholder priority matrix
    pub stakeholder_matrix: HashMap<String, HashMap<String, f64>>,
    /// Business value analysis
    pub business_value_analysis: BusinessValueAnalysis,
}

/// Priority score breakdown
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PriorityBreakdown {
    /// Business value component
    pub business_value: f64,
    /// Technical feasibility component
    pub technical_feasibility: f64,
    /// Stakeholder importance component
    pub stakeholder_importance: f64,
    /// Risk factor component
    pub risk_factor: f64,
    /// Effort vs impact ratio
    pub effort_impact_ratio: f64,
    /// Total priority score
    pub total_score: f64,
}

/// Business value analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BusinessValueAnalysis {
    /// ROI estimates for each goal
    pub roi_estimates: HashMap<String, f64>,
    /// Cost-benefit analysis results
    pub cost_benefit_results: HashMap<String, CostBenefitResult>,
    /// Market impact assessment
    pub market_impact: HashMap<String, f64>,
    /// Competitive advantage potential
    pub competitive_advantage: HashMap<String, f64>,
}

/// Cost-benefit analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CostBenefitResult {
    /// Estimated costs
    pub estimated_costs: f64,
    /// Estimated benefits
    pub estimated_benefits: f64,
    /// Net present value
    pub net_present_value: f64,
    /// Benefit-cost ratio
    pub benefit_cost_ratio: f64,
    /// Payback period (months)
    pub payback_period_months: f64,
}

/// Goal validation results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalValidation {
    /// Goal ID being validated
    pub goal_id: String,
    /// Validation type
    pub validation_type: ValidationType,
    /// Validation result
    pub result: ValidationResult,
    /// Validation message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggested fixes
    pub suggested_fixes: Vec<String>,
}

/// Validation types for goals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum ValidationType {
    /// SMART criteria validation
    SmartCriteria,
    /// Feasibility analysis
    Feasibility,
    /// Resource availability
    ResourceAvailability,
    /// Timeline constraints
    TimelineConstraints,
    /// Dependency conflicts
    DependencyConflicts,
    /// Stakeholder alignment
    StakeholderAlignment,
    /// Risk assessment
    RiskAssessment,
    /// Business value validation
    BusinessValue,
}

/// Validation results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ValidationResult {
    Pass,
    Warning,
    Fail,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Goal analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalAnalysisMetadata {
    /// Analysis timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Analysis duration (milliseconds)
    pub analysis_duration_ms: u64,
    /// Input text length
    pub input_length: usize,
    /// Number of goals extracted
    pub goals_extracted: usize,
    /// Analysis confidence score
    pub confidence_score: f64,
    /// Analysis method used
    pub analysis_method: String,
    /// Warnings generated during analysis
    pub warnings: Vec<String>,
}

/// Advanced goal analyzer with comprehensive NLP capabilities
pub struct AdvancedGoalAnalyzer {
    /// Configuration
    config: GoalAnalysisConfig,
    /// NLP patterns for goal extraction
    goal_patterns: HashMap<String, Regex>,
    /// Goal type classification patterns
    goal_type_patterns: HashMap<GoalType, Vec<Regex>>,
    /// Semantic similarity cache
    semantic_cache: LruCache<String, Vec<f64>>,
    /// Goal prioritization engine
    prioritization_engine: GoalPrioritizationEngine,
    /// Dependency analyzer
    dependency_analyzer: GoalDependencyAnalyzer,
    /// Validation engine
    validation_engine: GoalValidationEngine,
}

/// Goal prioritization engine

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GoalPrioritizationEngine {
    /// ML model for priority prediction (simplified)
    priority_weights: HashMap<String, f64>,
}

/// Goal dependency analyzer

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GoalDependencyAnalyzer {
    /// Dependency patterns
    dependency_patterns: Vec<String>,
}

/// Goal validation engine

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GoalValidationEngine {
    /// Validation rules
    validation_rules: Vec<GoalValidationRule>,
}

/// Goal validation rule

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GoalValidationRule {
    /// Rule type
    rule_type: ValidationType,
    /// Validation function (simplified)
    description: String,
}

/// Pre-compiled regex patterns for goal extraction
static GOAL_PATTERNS: Lazy<HashMap<&'static str, Lazy<Regex>>> = Lazy::new(|| {
    let mut patterns: HashMap<&'static str, Lazy<Regex>> = HashMap::new();

    // Goal indicators
    patterns.insert("goal_indicators", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:goal|objective|aim|target|purpose|mission|vision|strategy)\b").unwrap()
    }));

    // Action verbs
    patterns.insert("action_verbs", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:implement|create|build|develop|design|fix|improve|optimize|integrate|add|remove|update|modify|enhance|extend)\b").unwrap()
    }));

    // Functional requirements
    patterns.insert("functional_req", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:system|user|application|software|feature|function|capability|service)\b.*\b(?:shall|should|must|will|can|does)\b").unwrap()
    }));

    // Non-functional requirements
    patterns.insert("nonfunctional_req", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:performance|security|reliability|usability|scalability|maintainability|efficiency|compatibility|portability)\b").unwrap()
    }));

    // Stakeholder mentions
    patterns.insert("stakeholder", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:user|customer|client|stakeholder|team|organization|company|business|management|executive)\b").unwrap()
    }));

    // Constraint indicators
    patterns.insert("constraints", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:constraint|limitation|restriction|requirement|deadline|budget|resource|time|cost)\b").unwrap()
    }));

    // Dependency indicators
    patterns.insert("dependencies", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:depend|require|need|after|before|following|preceding|subsequent|prior)\b").unwrap()
    }));

    // Risk indicators
    patterns.insert("risks", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:risk|danger|threat|vulnerability|exposure|hazard|challenge|issue|problem|concern)\b").unwrap()
    }));

    // Success criteria
    patterns.insert("success_criteria", Lazy::new(|| {
        Regex::new(r"(?i)\b(?:success|complete|done|finished|achieved|delivered|implemented|working|operational|functional)\b").unwrap()
    }));

    patterns
});

impl AdvancedGoalAnalyzer {
    /// Create a new advanced goal analyzer
    pub fn new(config: GoalAnalysisConfig) -> Self {
        let goal_patterns = Self::compile_goal_patterns();
        let goal_type_patterns = Self::compile_goal_type_patterns();
        let semantic_cache = LruCache::new(std::num::NonZeroUsize::new(config.cache_size).unwrap_or(std::num::NonZeroUsize::new(100).unwrap()));
        let prioritization_engine = GoalPrioritizationEngine::new();
        let dependency_analyzer = GoalDependencyAnalyzer::new();
        let validation_engine = GoalValidationEngine::new();

        Self {
            config,
            goal_patterns,
            goal_type_patterns,
            semantic_cache,
            prioritization_engine,
            dependency_analyzer,
            validation_engine,
        }
    }

    /// Compile goal extraction patterns
    fn compile_goal_patterns() -> HashMap<String, Regex> {
        let mut patterns = HashMap::new();

        for (name, lazy_pattern) in &*GOAL_PATTERNS {
            patterns.insert(name.to_string(), (*lazy_pattern).clone());
        }

        patterns
    }

    /// Compile goal type classification patterns
    fn compile_goal_type_patterns() -> HashMap<GoalType, Vec<Regex>> {
        let mut type_patterns = HashMap::new();

        // Functional goals
        type_patterns.insert(GoalType::Functional, vec![
            Regex::new(r"(?i)\b(?:implement|create|build|add|develop|design)\b").unwrap(),
        ]);

        // Non-functional goals
        type_patterns.insert(GoalType::NonFunctional, vec![
            Regex::new(r"(?i)\b(?:performance|security|reliability|usability|scalability)\b").unwrap(),
        ]);

        // Technical goals
        type_patterns.insert(GoalType::Technical, vec![
            Regex::new(r"(?i)\b(?:code|algorithm|architecture|infrastructure|database|api)\b").unwrap(),
        ]);

        // Business goals
        type_patterns.insert(GoalType::Business, vec![
            Regex::new(r"(?i)\b(?:business|revenue|profit|market|customer|stakeholder)\b").unwrap(),
        ]);

        // Quality goals
        type_patterns.insert(GoalType::Quality, vec![
            Regex::new(r"(?i)\b(?:quality|test|testing|qa|review|audit|compliance)\b").unwrap(),
        ]);

        type_patterns
    }

    /// Perform comprehensive goal analysis
    pub async fn analyze_goals_comprehensive(&self, input_text: &str) -> PlanningResult<GoalAnalysisResult> {
        let start_time = std::time::Instant::now();

        // Extract goals using multiple methods
        let mut extracted_goals = self.extract_goals_multimodal(input_text)?;

        // Limit number of goals if specified
        if extracted_goals.len() > self.config.max_goals {
            extracted_goals.truncate(self.config.max_goals);
        }

        // Analyze goal hierarchy and dependencies
        let hierarchy = self.dependency_analyzer.analyze_hierarchy(&extracted_goals, input_text)?;

        // Prioritize goals
        let prioritization = self.prioritization_engine.prioritize_goals(&extracted_goals, input_text)?;

        // Validate goals
        let validation_results = self.validation_engine.validate_goals(&extracted_goals, &hierarchy)?;

        // Create analysis metadata
        let metadata = GoalAnalysisMetadata {
            timestamp: chrono::Utc::now(),
            analysis_duration_ms: start_time.elapsed().as_millis() as u64,
            input_length: input_text.len(),
            goals_extracted: extracted_goals.len(),
            confidence_score: self.calculate_overall_confidence(&extracted_goals),
            analysis_method: "multimodal_nlp".to_string(),
            warnings: Vec::new(), // Could be populated with analysis warnings
        };

        Ok(GoalAnalysisResult {
            goals: extracted_goals,
            hierarchy,
            prioritization,
            validation_results,
            metadata,
        })
    }

    /// Extract goals using multiple methods (pattern matching, semantic analysis, etc.)
    fn extract_goals_multimodal(&self, input_text: &str) -> PlanningResult<Vec<ExtractedGoal>> {
        let mut goals: Vec<ExtractedGoal> = Vec::new();
        let mut goal_id_counter = 0;

        // Method 1: Pattern-based extraction
        let pattern_goals = self.extract_goals_by_patterns(input_text)?;
        goals.extend(pattern_goals.into_iter().map(|text| {
            goal_id_counter += 1;
            self.create_goal_from_text(text, GoalType::Functional, goal_id_counter)
        }));

        // Method 2: Sentence-based extraction
        let sentence_goals = self.extract_goals_by_sentences(input_text)?;
        for sentence in sentence_goals {
            if !goals.iter().any(|g| jaro_winkler(&g.text, &sentence) > 0.8) {
                goal_id_counter += 1;
                let goal_type = self.classify_goal_type(&sentence);
                goals.push(self.create_goal_from_text(sentence.to_string(), goal_type, goal_id_counter));
            }
        }

        // Method 3: Stakeholder-based extraction
        if self.config.enable_stakeholder_analysis {
            let stakeholder_goals = self.extract_stakeholder_goals(input_text)?;
            for goal_text in stakeholder_goals {
                if !goals.iter().any(|g| jaro_winkler(&g.text, &goal_text) > 0.8) {
                    goal_id_counter += 1;
                    goals.push(self.create_goal_from_text(goal_text.to_string(), GoalType::Business, goal_id_counter));
                }
            }
        }

        // Enrich goals with additional metadata
        for goal in &mut goals {
            self.enrich_goal_metadata(goal, input_text);
        }

        // Filter by confidence
        goals.retain(|g| g.confidence >= self.config.min_confidence);

        Ok(goals)
    }

    /// Extract goals using pattern matching
    fn extract_goals_by_patterns(&self, input_text: &str) -> PlanningResult<Vec<String>> {
        let mut goals: Vec<String> = Vec::new();

        // Split into sentences and analyze each
        for sentence in input_text.split(|c: char| c == '.' || c == '!' || c == '?') {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            // Check if sentence contains goal indicators
            let has_goal_indicator = self.pattern_matches("goal_indicators", sentence);
            let has_action_verb = self.pattern_matches("action_verbs", sentence);
            let has_functional_req = self.pattern_matches("functional_req", sentence);

            if has_goal_indicator || (has_action_verb && has_functional_req) {
                goals.push(sentence.to_string());
            }
        }

        Ok(goals)
    }

    /// Extract goals by analyzing individual sentences
    fn extract_goals_by_sentences(&self, input_text: &str) -> PlanningResult<Vec<String>> {
        let mut goals: Vec<String> = Vec::new();

        for sentence in input_text.split(|c: char| c == '.' || c == '!' || c == '?') {
            let sentence = sentence.trim();
            if sentence.is_empty() || sentence.len() < 10 {
                continue;
            }

            // Score sentence as potential goal
            let score = self.score_sentence_as_goal(sentence);

            if score > 0.6 { // Threshold for goal classification
                goals.push(sentence.to_string());
            }
        }

        Ok(goals)
    }

    /// Extract stakeholder-specific goals
    fn extract_stakeholder_goals(&self, input_text: &str) -> PlanningResult<Vec<String>> {
        let mut goals: Vec<String> = Vec::new();

        for sentence in input_text.split(|c: char| c == '.' || c == '!' || c == '?') {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            // Check for stakeholder mentions
            if self.pattern_matches("stakeholder", sentence) {
                // Extract what the stakeholder wants
                if let Some(goal_text) = self.extract_stakeholder_requirement(sentence) {
                    goals.push(goal_text);
                }
            }
        }

        Ok(goals)
    }

    /// Check if pattern matches text
    fn pattern_matches(&self, pattern_name: &str, text: &str) -> bool {
        if let Some(pattern) = self.goal_patterns.get(pattern_name) {
            pattern.is_match(text)
        } else {
            false
        }
    }

    /// Score sentence as potential goal (0.0-1.0)
    fn score_sentence_as_goal(&self, sentence: &str) -> f64 {
        let mut score: f64 = 0.0;

        // Goal indicators (+0.4)
        if self.pattern_matches("goal_indicators", sentence) {
            score += 0.4;
        }

        // Action verbs (+0.3)
        if self.pattern_matches("action_verbs", sentence) {
            score += 0.3;
        }

        // Functional requirements (+0.2)
        if self.pattern_matches("functional_req", sentence) {
            score += 0.2;
        }

        // Stakeholder mentions (+0.2)
        if self.pattern_matches("stakeholder", sentence) {
            score += 0.2;
        }

        // Length bonus (prefer substantial sentences)
        let word_count = sentence.split_whitespace().count();
        if word_count > 5 && word_count < 50 {
            score += 0.1;
        }

        // Cap at 1.0
        score.min(1.0f64)
    }

    /// Extract stakeholder requirement from sentence with improved pattern matching
    fn extract_stakeholder_requirement(&self, sentence: &str) -> Option<String> {
        // Enhanced requirement extraction using multiple patterns
        let sentence_lower = sentence.to_lowercase();
        
        // Pattern 1: "X wants/needs Y"
        if let Some(want_idx) = sentence_lower.find(" wants ") {
            let before_want = &sentence[..want_idx];
            let after_want = &sentence[want_idx + 7..];
            
            // Extract stakeholder name
            let stakeholder = before_want.split_whitespace().last().unwrap_or("Stakeholder");
            let requirement = after_want.trim().trim_end_matches('.').trim();
            
            return Some(format!("{} wants {}", stakeholder, requirement));
        }
        
        if let Some(need_idx) = sentence_lower.find(" needs ") {
            let before_need = &sentence[..need_idx];
            let after_need = &sentence[need_idx + 7..];
            
            let stakeholder = before_need.split_whitespace().last().unwrap_or("Stakeholder");
            let requirement = after_need.trim().trim_end_matches('.').trim();
            
            return Some(format!("{} needs {}", stakeholder, requirement));
        }
        
        // Pattern 2: "Requirement: ..." or "Requirement is ..."
        if let Some(req_idx) = sentence_lower.find("requirement") {
            let after_req = &sentence[req_idx + 11..];
            if !after_req.trim().is_empty() {
                // Clean up common prefixes
                let cleaned = after_req
                    .trim_start_matches(|c: char| c == ':' || c == ' ' || c == 'i' || c == 's' || c == ' ')
                    .trim()
                    .trim_end_matches('.')
                    .to_string();
                
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
        
        // Pattern 3: "The system should/must/can ..."
        if let Some(should_idx) = sentence_lower.find("should ") {
            return Some(sentence[should_idx + 7..].trim().trim_end_matches('.').to_string());
        }
        
        if let Some(must_idx) = sentence_lower.find("must ") {
            return Some(sentence[must_idx + 5..].trim().trim_end_matches('.').to_string());
        }
        
        // Fallback: return the sentence if it's substantial enough
        if sentence.trim().len() > 10 {
            Some(sentence.trim().trim_end_matches('.').to_string())
        } else {
            None
        }
    }

    /// Classify goal type based on content
    fn classify_goal_type(&self, goal_text: &str) -> GoalType {
        for (goal_type, patterns) in &self.goal_type_patterns {
            for pattern in patterns {
                if pattern.is_match(goal_text) {
                    return goal_type.clone();
                }
            }
        }

        GoalType::Functional // Default
    }

    /// Create goal from text with basic metadata
    fn create_goal_from_text(&self, text: String, goal_type: GoalType, id: usize) -> ExtractedGoal {
        ExtractedGoal {
            id: format!("goal_{}", id),
            text,
            goal_type,
            confidence: 0.7, // Base confidence
            priority_score: 0.5, // Will be updated by prioritization
            business_value: 0.5,
            stakeholder_importance: 0.5,
            estimated_effort: 1.0, // Default 1 person-day
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            sub_goals: Vec::new(),
            success_criteria: Vec::new(),
            risks: Vec::new(),
            required_resources: Vec::new(),
            timeline_constraints: None,
        }
    }

    /// Enrich goal with additional metadata
    fn enrich_goal_metadata(&self, goal: &mut ExtractedGoal, input_text: &str) {
        // Extract success criteria
        goal.success_criteria = self.extract_success_criteria(&goal.text, input_text);

        // Extract risks
        goal.risks = self.extract_risks(&goal.text, input_text);

        // Estimate effort (simplified)
        goal.estimated_effort = self.estimate_effort(&goal.text);

        // Extract required resources
        goal.required_resources = self.extract_resources(&goal.text);
    }

    /// Extract success criteria for goal
    fn extract_success_criteria(&self, goal_text: &str, context: &str) -> Vec<String> {
        let mut criteria = Vec::new();

        // Look for success indicators in context
        if context.to_lowercase().contains("success") {
            criteria.push("Goal successfully implemented and tested".to_string());
        }

        if context.to_lowercase().contains("working") {
            criteria.push("Functionality is operational and working".to_string());
        }

        if criteria.is_empty() {
            criteria.push(format!("{} is completed successfully", goal_text));
        }

        criteria
    }

    /// Extract risks for goal
    fn extract_risks(&self, goal_text: &str, context: &str) -> Vec<String> {
        let mut risks = Vec::new();

        if self.pattern_matches("risks", context) {
            risks.push("Identified risks in requirements need to be addressed".to_string());
        }

        if context.to_lowercase().contains("complex") {
            risks.push("High complexity may lead to implementation challenges".to_string());
        }

        if context.to_lowercase().contains("time") && context.to_lowercase().contains("constraint") {
            risks.push("Timeline constraints may affect quality".to_string());
        }

        risks
    }

    /// Estimate effort for goal (person-hours)
    fn estimate_effort(&self, goal_text: &str) -> f64 {
        let word_count = goal_text.split_whitespace().count();
        let complexity_factor = if goal_text.to_lowercase().contains("complex") { 2.0 } else { 1.0 };
        let integration_factor = if goal_text.to_lowercase().contains("integrat") { 1.5 } else { 1.0 };

        (word_count as f64 * 0.5 * complexity_factor * integration_factor).max(1.0)
    }

    /// Extract required resources
    fn extract_resources(&self, goal_text: &str) -> Vec<String> {
        let mut resources = Vec::new();

        if goal_text.to_lowercase().contains("database") {
            resources.push("Database access".to_string());
        }

        if goal_text.to_lowercase().contains("api") {
            resources.push("API access".to_string());
        }

        if goal_text.to_lowercase().contains("user") && goal_text.to_lowercase().contains("interface") {
            resources.push("UI/UX design resources".to_string());
        }

        if resources.is_empty() {
            resources.push("Development resources".to_string());
        }

        resources
    }

    /// Calculate overall analysis confidence
    fn calculate_overall_confidence(&self, goals: &[ExtractedGoal]) -> f64 {
        if goals.is_empty() {
            return 0.0;
        }

        let avg_confidence = goals.iter().map(|g| g.confidence).sum::<f64>() / goals.len() as f64;
        let goal_count_bonus = if goals.len() >= 3 { 0.1 } else { 0.0 };

        (avg_confidence + goal_count_bonus).min(1.0f64)
    }
}

impl GoalPrioritizationEngine {
    /// Create new prioritization engine
    fn new() -> Self {
        let mut priority_weights = HashMap::new();
        priority_weights.insert("business_value".to_string(), 0.3);
        priority_weights.insert("technical_feasibility".to_string(), 0.2);
        priority_weights.insert("stakeholder_importance".to_string(), 0.25);
        priority_weights.insert("risk_factor".to_string(), 0.15);
        priority_weights.insert("effort_impact_ratio".to_string(), 0.1);

        Self { priority_weights }
    }

    /// Prioritize goals using multiple criteria
    fn prioritize_goals(&self, goals: &[ExtractedGoal], context: &str) -> PlanningResult<GoalPrioritization> {
        let mut ranked_goals = Vec::new();
        let mut priority_breakdown = HashMap::new();
        let mut stakeholder_matrix = HashMap::new();
        let mut business_value_analysis = BusinessValueAnalysis {
            roi_estimates: HashMap::new(),
            cost_benefit_results: HashMap::new(),
            market_impact: HashMap::new(),
            competitive_advantage: HashMap::new(),
        };

        // Calculate priority scores for each goal
        let mut goal_scores: Vec<(String, f64)> = goals.iter().map(|goal| {
            let priority_score = self.calculate_priority_score(goal, context);
            (goal.id.clone(), priority_score)
        }).collect();

        // Sort by priority score (descending)
        goal_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Extract ranked goal IDs
        ranked_goals = goal_scores.iter().map(|(id, _)| id.clone()).collect();

        // Create priority breakdown
        for goal in goals {
            let breakdown = self.calculate_priority_breakdown(goal, context);
            priority_breakdown.insert(goal.id.clone(), breakdown);

            // Calculate business value metrics
            business_value_analysis.roi_estimates.insert(
                goal.id.clone(),
                goal.business_value * 2.0 // Simplified ROI calculation
            );

            business_value_analysis.cost_benefit_results.insert(
                goal.id.clone(),
                CostBenefitResult {
                    estimated_costs: goal.estimated_effort * 100.0, // $100/hour assumption
                    estimated_benefits: goal.business_value * 1000.0,
                    net_present_value: goal.business_value * 800.0,
                    benefit_cost_ratio: goal.business_value * 10.0,
                    payback_period_months: goal.estimated_effort / 10.0, // Simplified
                }
            );
        }

        Ok(GoalPrioritization {
            ranked_goals,
            priority_breakdown,
            stakeholder_matrix,
            business_value_analysis,
        })
    }

    /// Calculate overall priority score for goal
    fn calculate_priority_score(&self, goal: &ExtractedGoal, context: &str) -> f64 {
        let business_value = goal.business_value;
        let stakeholder_importance = goal.stakeholder_importance;
        let technical_feasibility = 1.0 / (1.0 + goal.estimated_effort); // Higher effort = lower feasibility
        let risk_factor = 1.0 - (goal.risks.len() as f64 * 0.1).min(0.5f64); // Risk penalty
        let effort_impact_ratio = goal.business_value / goal.estimated_effort.max(0.1);

        // Weighted combination
        business_value * 0.3 +
        stakeholder_importance * 0.25 +
        technical_feasibility * 0.2 +
        risk_factor * 0.15 +
        effort_impact_ratio * 0.1
    }

    /// Calculate detailed priority breakdown
    fn calculate_priority_breakdown(&self, goal: &ExtractedGoal, context: &str) -> PriorityBreakdown {
        let business_value = goal.business_value;
        let technical_feasibility = 1.0 / (1.0 + goal.estimated_effort);
        let stakeholder_importance = goal.stakeholder_importance;
        let risk_factor = 1.0 - (goal.risks.len() as f64 * 0.1).min(0.5f64);
        let effort_impact_ratio = goal.business_value / goal.estimated_effort.max(0.1);

        let total_score = business_value * 0.3 +
                         stakeholder_importance * 0.25 +
                         technical_feasibility * 0.2 +
                         risk_factor * 0.15 +
                         effort_impact_ratio * 0.1;

        PriorityBreakdown {
            business_value,
            technical_feasibility,
            stakeholder_importance,
            risk_factor,
            effort_impact_ratio,
            total_score,
        }
    }
}

impl GoalDependencyAnalyzer {
    /// Create new dependency analyzer
    fn new() -> Self {
        let dependency_patterns = vec![
            r"(?i)\b(?:after|before|following|preceding|subsequent|prior to)\b".to_string(),
            r"(?i)\b(?:depend|require|need)\b".to_string(),
            r"(?i)\b(?:must|shall|should)\b.*\b(?:first|initially|before)\b".to_string(),
        ];

        Self { dependency_patterns }
    }

    /// Analyze goal hierarchy and dependencies
    fn analyze_hierarchy(&self, goals: &[ExtractedGoal], context: &str) -> PlanningResult<GoalHierarchy> {
        let mut dependency_graph = HashMap::new();
        let mut conflict_graph = HashMap::new();
        let mut root_goals = Vec::new();
        let mut hierarchy_levels = Vec::new();

        // Initialize graphs
        for goal in goals {
            dependency_graph.insert(goal.id.clone(), Vec::new());
            conflict_graph.insert(goal.id.clone(), Vec::new());
        }

        // Analyze dependencies between goals
        for i in 0..goals.len() {
            for j in 0..goals.len() {
                if i == j { continue; }

                let goal_a = &goals[i];
                let goal_b = &goals[j];

                // Check for explicit dependencies
                if self.goals_have_dependency(goal_a, goal_b, context) {
                    dependency_graph.get_mut(&goal_a.id).unwrap().push(goal_b.id.clone());
                }

                // Check for conflicts
                if self.goals_conflict(goal_a, goal_b) {
                    conflict_graph.get_mut(&goal_a.id).unwrap().push(goal_b.id.clone());
                    conflict_graph.get_mut(&goal_b.id).unwrap().push(goal_a.id.clone());
                }
            }
        }

        // Identify root goals (no dependencies)
        for goal in goals {
            if dependency_graph[&goal.id].is_empty() {
                root_goals.push(goal.id.clone());
            }
        }

        // Implement topological sort for goal hierarchy using Kahn's algorithm
        // This properly orders goals based on their dependencies
        // Note: dependency_graph[goal_id] = list of goals that goal_id depends on
        let mut sorted_levels = Vec::new();
        let mut in_degree = HashMap::new();
        let mut ready_queue = VecDeque::new();
        
        // Initialize in-degree counts
        // In-degree = number of goals that depend on this goal
        for goal_id in dependency_graph.keys() {
            let mut count = 0;
            // Count how many goals have this goal in their dependency list
            for (_, deps) in dependency_graph.iter() {
                if deps.contains(goal_id) {
                    count += 1;
                }
            }
            in_degree.insert(goal_id.clone(), count);
            
            if count == 0 {
                ready_queue.push_back(goal_id.clone());
            }
        }
        
        // Process goals level by level
        while !ready_queue.is_empty() {
            let current_level = Vec::from_iter(ready_queue.drain(..));
            sorted_levels.push(current_level.clone());
            
            // For each goal in current level, reduce in-degree of goals that depend on this goal
            for goal_id in &current_level {
                // Find all goals that have this goal in their dependency list
                // (i.e., goals that depend on this goal)
                for (dependent_goal_id, deps) in dependency_graph.iter() {
                    if deps.contains(goal_id) {
                        if let Some(current_in_degree) = in_degree.get_mut(dependent_goal_id) {
                            *current_in_degree -= 1;
                            
                            if *current_in_degree == 0 {
                                ready_queue.push_back(dependent_goal_id.clone());
                            }
                        }
                    }
                }
            }
        }
        
        // Update hierarchy_levels with properly sorted levels
        hierarchy_levels = sorted_levels;

        // Detect circular dependencies (simplified)
        let circular_dependencies = self.detect_circular_dependencies(&dependency_graph);

        Ok(GoalHierarchy {
            root_goals,
            dependency_graph,
            conflict_graph,
            hierarchy_levels,
            circular_dependencies,
        })
    }

    /// Check if two goals have a dependency relationship
    fn goals_have_dependency(&self, goal_a: &ExtractedGoal, goal_b: &ExtractedGoal, context: &str) -> bool {
        // Check text similarity (related goals might be dependent)
        let text_similarity = jaro_winkler(&goal_a.text, &goal_b.text);

        // Check for explicit dependency patterns in context
        let combined_text = format!("{} {}", goal_a.text, goal_b.text);
        let has_dependency_pattern = self.dependency_patterns.iter()
            .any(|pattern_str| {
                if let Ok(pattern) = Regex::new(pattern_str) {
                    pattern.is_match(&combined_text) || pattern.is_match(context)
                } else {
                    false
                }
            });

        text_similarity > 0.6 || has_dependency_pattern
    }

    /// Check if two goals conflict
    fn goals_conflict(&self, goal_a: &ExtractedGoal, goal_b: &ExtractedGoal) -> bool {
        // Simple conflict detection based on contradictory terms
        let a_lower = goal_a.text.to_lowercase();
        let b_lower = goal_b.text.to_lowercase();

        (a_lower.contains("fast") && b_lower.contains("thorough")) ||
        (a_lower.contains("simple") && b_lower.contains("complex")) ||
        (a_lower.contains("cheap") && b_lower.contains("high quality"))
    }

    /// Detect circular dependencies using depth-first search
    fn detect_circular_dependencies(&self, dependency_graph: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
        // Use DFS to detect cycles in the dependency graph
        let mut circular_deps = Vec::new();
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            node: &str,
            graph: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            recursion_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
            cycles: &mut Vec<Vec<String>>,
        ) {
            visited.insert(node.to_string());
            recursion_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(deps) = graph.get(node) {
                for dep in deps {
                    if !visited.contains(dep) {
                        dfs(dep, graph, visited, recursion_stack, path, cycles);
                    } else if recursion_stack.contains(dep) {
                        // Found a cycle - extract the cycle path
                        if let Some(cycle_start) = path.iter().position(|x| x == dep) {
                            let cycle = path[cycle_start..].to_vec();
                            cycles.push(cycle);
                        }
                    }
                }
            }

            recursion_stack.remove(node);
            path.pop();
        }

        for goal_id in dependency_graph.keys() {
            if !visited.contains(goal_id) {
                dfs(
                    goal_id,
                    dependency_graph,
                    &mut visited,
                    &mut recursion_stack,
                    &mut path,
                    &mut circular_deps,
                );
            }
        }

        // Deduplicate cycles (same cycle might be detected from different starting points)
        let mut unique_cycles = Vec::new();
        for cycle in circular_deps {
            let mut normalized_cycle = cycle.clone();
            // Rotate cycle to start from smallest ID for consistent representation
            if let Some(min_idx) = normalized_cycle.iter().enumerate()
                .min_by_key(|(_, id)| id.as_str())
                .map(|(idx, _)| idx) {
                normalized_cycle.rotate_left(min_idx);
            }
            
            // Check if this cycle is already in unique_cycles
            let is_duplicate = unique_cycles.iter().any(|existing: &Vec<String>| {
                existing.len() == normalized_cycle.len() &&
                existing.iter().zip(normalized_cycle.iter()).all(|(a, b)| a == b)
            });
            
            if !is_duplicate {
                unique_cycles.push(normalized_cycle);
            }
        }

        unique_cycles
    }
}

impl GoalValidationEngine {
    /// Create new validation engine
    fn new() -> Self {
        let validation_rules = vec![
            GoalValidationRule {
                rule_type: ValidationType::SmartCriteria,
                description: "Goal should be Specific, Measurable, Achievable, Relevant, Time-bound".to_string(),
            },
            GoalValidationRule {
                rule_type: ValidationType::Feasibility,
                description: "Goal should be technically and resource-wise feasible".to_string(),
            },
            GoalValidationRule {
                rule_type: ValidationType::ResourceAvailability,
                description: "Required resources should be available".to_string(),
            },
        ];

        Self { validation_rules }
    }

    /// Validate goals against constraints and best practices
    fn validate_goals(&self, goals: &[ExtractedGoal], hierarchy: &GoalHierarchy) -> PlanningResult<Vec<GoalValidation>> {
        let mut validations = Vec::new();

        for goal in goals {
            // SMART criteria validation
            let smart_validation = self.validate_smart_criteria(goal);
            validations.push(smart_validation);

            // Feasibility validation
            let feasibility_validation = self.validate_feasibility(goal);
            validations.push(feasibility_validation);

            // Resource validation
            let resource_validation = self.validate_resources(goal);
            validations.push(resource_validation);

            // Dependency validation
            let dependency_validation = self.validate_dependencies(goal, hierarchy);
            validations.push(dependency_validation);
        }

        Ok(validations)
    }

    /// Validate SMART criteria
    fn validate_smart_criteria(&self, goal: &ExtractedGoal) -> GoalValidation {
        let mut issues = Vec::new();

        // Specific: Check if goal is specific enough
        if goal.text.split_whitespace().count() < 3 {
            issues.push("Goal is too vague - add more specific details".to_string());
        }

        // Measurable: Check for success criteria
        if goal.success_criteria.is_empty() {
            issues.push("Goal lacks measurable success criteria".to_string());
        }

        // Achievable: Check effort estimate
        if goal.estimated_effort > 100.0 { // More than 100 person-hours
            issues.push("Goal may be too ambitious for estimated effort".to_string());
        }

        // Relevant: Check stakeholder importance
        if goal.stakeholder_importance < 0.3 {
            issues.push("Goal may not be sufficiently relevant to stakeholders".to_string());
        }

        // Time-bound: Check timeline constraints
        let has_time_constraint = goal.timeline_constraints.is_some();
        if !has_time_constraint {
            issues.push("Goal lacks timeline constraints".to_string());
        }

        let result = if issues.is_empty() {
            ValidationResult::Pass
        } else if issues.len() <= 2 {
            ValidationResult::Warning
        } else {
            ValidationResult::Fail
        };

        GoalValidation {
            goal_id: goal.id.clone(),
            validation_type: ValidationType::SmartCriteria,
            result,
            message: if issues.is_empty() {
                "Goal meets SMART criteria".to_string()
            } else {
                format!("SMART criteria issues: {}", issues.join(", "))
            },
            severity: if issues.len() > 2 { ValidationSeverity::Error } else { ValidationSeverity::Warning },
            suggested_fixes: issues.into_iter().map(|issue| format!("Fix: {}", issue)).collect(),
        }
    }

    /// Validate goal feasibility
    fn validate_feasibility(&self, goal: &ExtractedGoal) -> GoalValidation {
        let mut issues = Vec::new();

        // Check for technical complexity indicators
        if goal.text.to_lowercase().contains("ai") || goal.text.to_lowercase().contains("machine learning") {
            if goal.estimated_effort < 10.0 {
                issues.push("AI/ML goals typically require more effort".to_string());
            }
        }

        // Check risk factors
        if goal.risks.len() > 3 {
            issues.push("Too many identified risks - goal may be too risky".to_string());
        }

        let result = if issues.is_empty() {
            ValidationResult::Pass
        } else {
            ValidationResult::Warning
        };

        GoalValidation {
            goal_id: goal.id.clone(),
            validation_type: ValidationType::Feasibility,
            result,
            message: if issues.is_empty() {
                "Goal appears feasible".to_string()
            } else {
                format!("Feasibility concerns: {}", issues.join(", "))
            },
            severity: ValidationSeverity::Warning,
            suggested_fixes: issues.into_iter().map(|issue| format!("Address: {}", issue)).collect(),
        }
    }

    /// Validate resource availability
    fn validate_resources(&self, goal: &ExtractedGoal) -> GoalValidation {
        let result = if goal.required_resources.is_empty() {
            ValidationResult::Warning
        } else {
            ValidationResult::Pass
        };

        GoalValidation {
            goal_id: goal.id.clone(),
            validation_type: ValidationType::ResourceAvailability,
            result: result.clone(),
            message: match result {
                ValidationResult::Pass => "Resources identified for goal".to_string(),
                _ => "No specific resources identified for goal".to_string(),
            },
            severity: ValidationSeverity::Info,
            suggested_fixes: vec!["Identify specific resources needed for goal implementation".to_string()],
        }
    }

    /// Validate goal dependencies
    fn validate_dependencies(&self, goal: &ExtractedGoal, hierarchy: &GoalHierarchy) -> GoalValidation {
        let deps = &hierarchy.dependency_graph[&goal.id];
        let conflicts = &hierarchy.conflict_graph[&goal.id];

        let mut issues = Vec::new();

        if deps.len() > 5 {
            issues.push("Too many dependencies - goal may be over-complex".to_string());
        }

        if !conflicts.is_empty() {
            issues.push(format!("Goal conflicts with {} other goals", conflicts.len()));
        }

        let result = if issues.is_empty() {
            ValidationResult::Pass
        } else {
            ValidationResult::Warning
        };

        GoalValidation {
            goal_id: goal.id.clone(),
            validation_type: ValidationType::DependencyConflicts,
            result,
            message: if issues.is_empty() {
                "No dependency issues detected".to_string()
            } else {
                format!("Dependency issues: {}", issues.join(", "))
            },
            severity: if conflicts.is_empty() { ValidationSeverity::Warning } else { ValidationSeverity::Error },
            suggested_fixes: issues.into_iter().map(|issue| format!("Resolve: {}", issue)).collect(),
        }
    }
}

impl Default for GoalAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_semantic_analysis: true,
            enable_dependency_analysis: true,
            enable_stakeholder_analysis: true,
            enable_constraint_validation: true,
            max_goals: 10,
            min_confidence: 0.5,
            enable_ml_prioritization: false, // Disabled by default for simplicity
            cache_size: 1000,
        }
    }
}
