//! Curriculum Learning System
//!
//! Structured skill progression and difficulty adjustment for agents.
//! Provides systematic learning paths that gradually increase task difficulty
//! as agents develop capabilities.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::planning::thinking_budget::TaskComplexity;
use agent_agency_contracts::WorkingSpec;

/// Skill domain classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SkillDomain {
    /// Code generation and implementation
    CodeGeneration,

    /// Testing and quality assurance
    Testing,

    /// Documentation and communication
    Documentation,

    /// Code refactoring and optimization
    Refactoring,

    /// Bug fixing and debugging
    BugFixing,

    /// Security and compliance
    Security,

    /// Performance optimization
    Performance,

    /// Architecture and design
    Architecture,

    /// Data processing and analysis
    DataProcessing,

    /// Infrastructure and DevOps
    Infrastructure,
}

/// Skill proficiency level
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SkillLevel {
    /// Beginner - just starting to learn
    Beginner,

    /// Novice - basic understanding
    Novice,

    /// Intermediate - can handle standard tasks
    Intermediate,

    /// Advanced - can handle complex tasks
    Advanced,

    /// Expert - can handle very complex tasks
    Expert,
}

impl SkillLevel {
    /// Get numeric level (0.0-1.0)
    pub fn as_f64(&self) -> f64 {
        match self {
            SkillLevel::Beginner => 0.2,
            SkillLevel::Novice => 0.4,
            SkillLevel::Intermediate => 0.6,
            SkillLevel::Advanced => 0.8,
            SkillLevel::Expert => 1.0,
        }
    }

    /// Get next level
    pub fn next(&self) -> Option<Self> {
        match self {
            SkillLevel::Beginner => Some(SkillLevel::Novice),
            SkillLevel::Novice => Some(SkillLevel::Intermediate),
            SkillLevel::Intermediate => Some(SkillLevel::Advanced),
            SkillLevel::Advanced => Some(SkillLevel::Expert),
            SkillLevel::Expert => None,
        }
    }

    /// Get previous level
    pub fn previous(&self) -> Option<Self> {
        match self {
            SkillLevel::Beginner => None,
            SkillLevel::Novice => Some(SkillLevel::Beginner),
            SkillLevel::Intermediate => Some(SkillLevel::Novice),
            SkillLevel::Advanced => Some(SkillLevel::Intermediate),
            SkillLevel::Expert => Some(SkillLevel::Advanced),
        }
    }
}

/// Agent skill profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkillProfile {
    pub agent_id: Uuid,
    pub skills: HashMap<SkillDomain, SkillLevel>,
    pub overall_level: SkillLevel,
    pub total_tasks_completed: usize,
    pub total_tasks_succeeded: usize,
    pub last_updated: DateTime<Utc>,
}

impl AgentSkillProfile {
    /// Create new skill profile
    pub fn new(agent_id: Uuid) -> Self {
        let mut skills = HashMap::new();
        // Initialize all domains at Beginner level
        for domain in [
            SkillDomain::CodeGeneration,
            SkillDomain::Testing,
            SkillDomain::Documentation,
            SkillDomain::Refactoring,
            SkillDomain::BugFixing,
            SkillDomain::Security,
            SkillDomain::Performance,
            SkillDomain::Architecture,
            SkillDomain::DataProcessing,
            SkillDomain::Infrastructure,
        ] {
            skills.insert(domain, SkillLevel::Beginner);
        }

        Self {
            agent_id,
            skills,
            overall_level: SkillLevel::Beginner,
            total_tasks_completed: 0,
            total_tasks_succeeded: 0,
            last_updated: Utc::now(),
        }
    }

    /// Update skill level for a domain
    pub fn update_skill(&mut self, domain: SkillDomain, new_level: SkillLevel) {
        self.skills.insert(domain, new_level);
        self.recalculate_overall_level();
        self.last_updated = Utc::now();
    }

    /// Recalculate overall skill level
    fn recalculate_overall_level(&mut self) {
        if self.skills.is_empty() {
            self.overall_level = SkillLevel::Beginner;
            return;
        }

        // Calculate average skill level
        let avg_level: f64 = self
            .skills
            .values()
            .map(|level| level.as_f64())
            .sum::<f64>()
            / self.skills.len() as f64;

        // Map to skill level
        self.overall_level = if avg_level < 0.3 {
            SkillLevel::Beginner
        } else if avg_level < 0.5 {
            SkillLevel::Novice
        } else if avg_level < 0.7 {
            SkillLevel::Intermediate
        } else if avg_level < 0.9 {
            SkillLevel::Advanced
        } else {
            SkillLevel::Expert
        };
    }

    /// Get skill level for a domain
    pub fn get_skill_level(&self, domain: &SkillDomain) -> SkillLevel {
        self.skills
            .get(domain)
            .copied()
            .unwrap_or(SkillLevel::Beginner)
    }

    /// Check if agent is ready for a complexity level
    pub fn can_handle_complexity(&self, complexity: TaskComplexity, domain: &SkillDomain) -> bool {
        let skill_level = self.get_skill_level(domain);

        match complexity {
            TaskComplexity::Simple => skill_level >= SkillLevel::Beginner,
            TaskComplexity::Moderate => skill_level >= SkillLevel::Novice,
            TaskComplexity::Complex => skill_level >= SkillLevel::Intermediate,
            TaskComplexity::VeryComplex => skill_level >= SkillLevel::Advanced,
        }
    }
}

/// Learning milestone in curriculum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMilestone {
    pub milestone_id: String,
    pub domain: SkillDomain,
    pub required_level: SkillLevel,
    pub target_level: SkillLevel,
    pub complexity: TaskComplexity,
    pub description: String,
    pub prerequisites: Vec<String>, // IDs of prerequisite milestones
    pub success_criteria: SuccessCriteria,
}

/// Success criteria for a learning milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Minimum success rate (0.0-1.0)
    pub min_success_rate: f64,

    /// Minimum quality score (0.0-1.0)
    pub min_quality_score: f64,

    /// Number of successful completions required
    pub min_completions: usize,

    /// Maximum attempts allowed
    pub max_attempts: Option<usize>,
}

/// Curriculum path definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumPath {
    pub path_id: String,
    pub name: String,
    pub description: String,
    pub domains: Vec<SkillDomain>,
    pub milestones: Vec<LearningMilestone>,
    pub difficulty_progression: DifficultyProgression,
}

/// Difficulty progression strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyProgression {
    /// Linear progression (constant difficulty increase)
    Linear {
        /// Difficulty increment per milestone
        increment: f64,
    },

    /// Exponential progression (accelerating difficulty)
    Exponential {
        /// Base multiplier
        base: f64,
    },

    /// Adaptive progression (adjusts based on performance)
    Adaptive {
        /// Initial difficulty
        initial_difficulty: f64,
        /// Success rate threshold for advancement
        success_threshold: f64,
        /// Failure rate threshold for regression
        failure_threshold: f64,
    },
}

/// Task difficulty adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyAdjustment {
    /// Adjusted complexity level
    pub adjusted_complexity: TaskComplexity,

    /// Difficulty multiplier (1.0 = no change, >1.0 = harder, <1.0 = easier)
    pub difficulty_multiplier: f64,

    /// Reason for adjustment
    pub reason: String,
}

/// Curriculum Learning Engine
pub struct CurriculumLearningEngine {
    /// Agent skill profiles
    skill_profiles: Arc<RwLock<HashMap<Uuid, AgentSkillProfile>>>,

    /// Curriculum paths
    curriculum_paths: Arc<RwLock<HashMap<String, CurriculumPath>>>,

    /// Learning history
    learning_history: Arc<RwLock<Vec<LearningRecord>>>,

    /// Configuration
    config: CurriculumConfig,
}

/// Learning record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecord {
    pub agent_id: Uuid,
    pub task_id: Uuid,
    pub domain: SkillDomain,
    pub complexity: TaskComplexity,
    pub adjusted_complexity: Option<TaskComplexity>,
    pub skill_level_before: SkillLevel,
    pub skill_level_after: Option<SkillLevel>,
    pub success: bool,
    pub quality_score: f64,
    pub timestamp: DateTime<Utc>,
}

/// Curriculum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumConfig {
    /// Enable curriculum learning
    pub enabled: bool,

    /// Minimum tasks before skill level advancement
    pub min_tasks_for_advancement: usize,

    /// Success rate threshold for advancement (0.0-1.0)
    pub advancement_success_threshold: f64,

    /// Failure rate threshold for regression (0.0-1.0)
    pub regression_failure_threshold: f64,

    /// Enable difficulty adjustment
    pub enable_difficulty_adjustment: bool,

    /// Difficulty adjustment learning rate (0.0-1.0)
    pub difficulty_adjustment_rate: f64,
}

impl Default for CurriculumConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_tasks_for_advancement: 5,
            advancement_success_threshold: 0.8,
            regression_failure_threshold: 0.5,
            enable_difficulty_adjustment: true,
            difficulty_adjustment_rate: 0.1,
        }
    }
}

impl CurriculumLearningEngine {
    /// Create new curriculum learning engine
    pub fn new() -> Self {
        let mut curriculum_paths = HashMap::new();

        // Create default curriculum paths for each domain
        for domain in [
            SkillDomain::CodeGeneration,
            SkillDomain::Testing,
            SkillDomain::Documentation,
            SkillDomain::Refactoring,
            SkillDomain::BugFixing,
        ] {
            let path = Self::create_default_curriculum_path(&domain);
            curriculum_paths.insert(path.path_id.clone(), path);
        }

        Self {
            skill_profiles: Arc::new(RwLock::new(HashMap::new())),
            curriculum_paths: Arc::new(RwLock::new(curriculum_paths)),
            learning_history: Arc::new(RwLock::new(Vec::new())),
            config: CurriculumConfig::default(),
        }
    }

    /// Create default curriculum path for a domain
    fn create_default_curriculum_path(domain: &SkillDomain) -> CurriculumPath {
        let domain_name = format!("{:?}", domain);

        CurriculumPath {
            path_id: format!("{}_default", domain_name.to_lowercase()),
            name: format!("{} Learning Path", domain_name),
            description: format!("Structured learning path for {}", domain_name),
            domains: vec![domain.clone()],
            milestones: vec![
                LearningMilestone {
                    milestone_id: format!("{}_beginner_1", domain_name.to_lowercase()),
                    domain: domain.clone(),
                    required_level: SkillLevel::Beginner,
                    target_level: SkillLevel::Novice,
                    complexity: TaskComplexity::Simple,
                    description: format!("Basic {} tasks", domain_name),
                    prerequisites: vec![],
                    success_criteria: SuccessCriteria {
                        min_success_rate: 0.7,
                        min_quality_score: 0.6,
                        min_completions: 3,
                        max_attempts: Some(10),
                    },
                },
                LearningMilestone {
                    milestone_id: format!("{}_novice_1", domain_name.to_lowercase()),
                    domain: domain.clone(),
                    required_level: SkillLevel::Novice,
                    target_level: SkillLevel::Intermediate,
                    complexity: TaskComplexity::Moderate,
                    description: format!("Intermediate {} tasks", domain_name),
                    prerequisites: vec![format!("{}_beginner_1", domain_name.to_lowercase())],
                    success_criteria: SuccessCriteria {
                        min_success_rate: 0.75,
                        min_quality_score: 0.7,
                        min_completions: 5,
                        max_attempts: Some(15),
                    },
                },
                LearningMilestone {
                    milestone_id: format!("{}_intermediate_1", domain_name.to_lowercase()),
                    domain: domain.clone(),
                    required_level: SkillLevel::Intermediate,
                    target_level: SkillLevel::Advanced,
                    complexity: TaskComplexity::Complex,
                    description: format!("Advanced {} tasks", domain_name),
                    prerequisites: vec![format!("{}_novice_1", domain_name.to_lowercase())],
                    success_criteria: SuccessCriteria {
                        min_success_rate: 0.8,
                        min_quality_score: 0.75,
                        min_completions: 5,
                        max_attempts: Some(20),
                    },
                },
                LearningMilestone {
                    milestone_id: format!("{}_advanced_1", domain_name.to_lowercase()),
                    domain: domain.clone(),
                    required_level: SkillLevel::Advanced,
                    target_level: SkillLevel::Expert,
                    complexity: TaskComplexity::VeryComplex,
                    description: format!("Expert {} tasks", domain_name),
                    prerequisites: vec![format!("{}_intermediate_1", domain_name.to_lowercase())],
                    success_criteria: SuccessCriteria {
                        min_success_rate: 0.85,
                        min_quality_score: 0.8,
                        min_completions: 5,
                        max_attempts: Some(25),
                    },
                },
            ],
            difficulty_progression: DifficultyProgression::Adaptive {
                initial_difficulty: 0.3,
                success_threshold: 0.8,
                failure_threshold: 0.5,
            },
        }
    }

    /// Get or create skill profile for an agent
    pub async fn get_skill_profile(&self, agent_id: Uuid) -> AgentSkillProfile {
        let profiles = self.skill_profiles.read().await;
        profiles
            .get(&agent_id)
            .cloned()
            .unwrap_or_else(|| AgentSkillProfile::new(agent_id))
    }

    /// Adjust task difficulty based on agent skill level
    pub async fn adjust_task_difficulty(
        &self,
        agent_id: Uuid,
        working_spec: &WorkingSpec,
        domain: &SkillDomain,
    ) -> Result<DifficultyAdjustment> {
        if !self.config.enable_difficulty_adjustment {
            let complexity = TaskComplexity::assess(working_spec);
            return Ok(DifficultyAdjustment {
                adjusted_complexity: complexity,
                difficulty_multiplier: 1.0,
                reason: "Difficulty adjustment disabled".to_string(),
            });
        }

        let profile = self.get_skill_profile(agent_id).await;
        let skill_level = profile.get_skill_level(domain);
        let base_complexity = TaskComplexity::assess(working_spec);

        // Check if agent can handle the base complexity
        if profile.can_handle_complexity(base_complexity, domain) {
            // Agent can handle it - no adjustment needed
            Ok(DifficultyAdjustment {
                adjusted_complexity: base_complexity,
                difficulty_multiplier: 1.0,
                reason: format!(
                    "Agent skill level ({:?}) sufficient for task complexity ({:?})",
                    skill_level, base_complexity
                ),
            })
        } else {
            // Agent needs easier task - adjust complexity down
            let adjusted = match base_complexity {
                TaskComplexity::VeryComplex => TaskComplexity::Complex,
                TaskComplexity::Complex => TaskComplexity::Moderate,
                TaskComplexity::Moderate => TaskComplexity::Simple,
                TaskComplexity::Simple => TaskComplexity::Simple, // Can't go lower
            };

            Ok(DifficultyAdjustment {
                adjusted_complexity: adjusted,
                difficulty_multiplier: 0.7, // 30% easier
                reason: format!(
                    "Adjusted from {:?} to {:?} based on agent skill level ({:?})",
                    base_complexity, adjusted, skill_level
                ),
            })
        }
    }

    /// Record learning outcome and update skill profile
    pub async fn record_learning_outcome(
        &self,
        agent_id: Uuid,
        task_id: Uuid,
        domain: SkillDomain,
        complexity: TaskComplexity,
        adjusted_complexity: Option<TaskComplexity>,
        success: bool,
        quality_score: f64,
    ) -> Result<()> {
        let mut profiles = self.skill_profiles.write().await;
        let profile = profiles
            .entry(agent_id)
            .or_insert_with(|| AgentSkillProfile::new(agent_id));

        let skill_level_before = profile.get_skill_level(&domain);

        // Record learning history
        let record = LearningRecord {
            agent_id,
            task_id,
            domain: domain.clone(),
            complexity,
            adjusted_complexity,
            skill_level_before,
            skill_level_after: None,
            success,
            quality_score,
            timestamp: Utc::now(),
        };

        let mut history = self.learning_history.write().await;
        history.push(record);

        // Trim history to last 1000 entries
        if history.len() > 1000 {
            let excess = history.len() - 1000;
            history.drain(0..excess);
        }

        // Update task counts
        profile.total_tasks_completed += 1;
        if success {
            profile.total_tasks_succeeded += 1;
        }

        // Check if skill level should advance
        if self.config.enabled {
            let new_level = self
                .calculate_new_skill_level(agent_id, &domain, skill_level_before)
                .await;

            if new_level != skill_level_before {
                profile.update_skill(domain.clone(), new_level);
                info!(
                    "Agent {} advanced in {:?} from {:?} to {:?}",
                    agent_id, domain, skill_level_before, new_level
                );
            }
        }

        Ok(())
    }

    /// Calculate new skill level based on performance
    async fn calculate_new_skill_level(
        &self,
        agent_id: Uuid,
        domain: &SkillDomain,
        current_level: SkillLevel,
    ) -> SkillLevel {
        let history = self.learning_history.read().await;
        let domain_history: Vec<&LearningRecord> = history
            .iter()
            .filter(|r| r.agent_id == agent_id && r.domain == *domain)
            .collect();

        if domain_history.len() < self.config.min_tasks_for_advancement {
            return current_level; // Not enough data
        }

        // Calculate success rate
        let success_count = domain_history.iter().filter(|r| r.success).count();
        let success_rate = success_count as f64 / domain_history.len() as f64;

        // Calculate average quality score
        let avg_quality = domain_history.iter().map(|r| r.quality_score).sum::<f64>()
            / domain_history.len() as f64;

        // Check for advancement
        if success_rate >= self.config.advancement_success_threshold
            && avg_quality >= 0.7
            && current_level != SkillLevel::Expert
        {
            if let Some(next_level) = current_level.next() {
                return next_level;
            }
        }

        // Check for regression
        if success_rate < self.config.regression_failure_threshold
            && current_level != SkillLevel::Beginner
        {
            if let Some(prev_level) = current_level.previous() {
                warn!(
                    "Agent {} regressed in {:?} from {:?} to {:?} (success_rate={:.2})",
                    agent_id, domain, current_level, prev_level, success_rate
                );
                return prev_level;
            }
        }

        current_level
    }

    /// Get recommended next milestone for an agent
    pub async fn get_recommended_milestone(
        &self,
        agent_id: Uuid,
        domain: &SkillDomain,
    ) -> Option<LearningMilestone> {
        let profile = self.get_skill_profile(agent_id).await;
        let skill_level = profile.get_skill_level(domain);

        let paths = self.curriculum_paths.read().await;

        // Find curriculum path for this domain
        for path in paths.values() {
            if path.domains.contains(domain) {
                // Find next milestone based on skill level
                for milestone in &path.milestones {
                    if milestone.required_level == skill_level {
                        // Check if prerequisites are met
                        if self
                            .check_prerequisites(agent_id, &milestone.prerequisites)
                            .await
                        {
                            return Some(milestone.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if prerequisites are met
    async fn check_prerequisites(&self, agent_id: Uuid, prerequisite_ids: &[String]) -> bool {
        if prerequisite_ids.is_empty() {
            return true;
        }

        let history = self.learning_history.read().await;
        // TODO: Use agent_history for prerequisite checking in v4
        let _agent_history: Vec<&LearningRecord> = history
            .iter()
            .filter(|r| r.agent_id == agent_id && r.success)
            .collect();

        // TODO: Implement proper prerequisite milestone tracking
        //       Currently assumes prerequisites are met; should track actual milestone completions for accurate prerequisite checking.
        true // Temporary: basic assumption until milestone tracking is implemented
    }

    /// Get skill progression statistics
    pub async fn get_progression_stats(&self, agent_id: Uuid) -> Option<ProgressionStats> {
        let profile = self.get_skill_profile(agent_id).await;
        let history = self.learning_history.read().await;
        let agent_history: Vec<&LearningRecord> =
            history.iter().filter(|r| r.agent_id == agent_id).collect();

        if agent_history.is_empty() {
            return None;
        }

        let success_rate =
            agent_history.iter().filter(|r| r.success).count() as f64 / agent_history.len() as f64;

        let avg_quality =
            agent_history.iter().map(|r| r.quality_score).sum::<f64>() / agent_history.len() as f64;

        // Count skill level changes
        let mut level_changes = 0;
        let mut last_level: Option<SkillLevel> = None;
        for record in &agent_history {
            if let Some(prev_level) = last_level {
                if record.skill_level_before != prev_level {
                    level_changes += 1;
                }
            }
            last_level = Some(record.skill_level_before);
        }

        Some(ProgressionStats {
            agent_id,
            overall_level: profile.overall_level,
            total_tasks: agent_history.len(),
            success_rate,
            average_quality: avg_quality,
            skill_level_changes: level_changes,
            skills_by_domain: profile.skills.clone(),
        })
    }
}

/// Progression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionStats {
    pub agent_id: Uuid,
    pub overall_level: SkillLevel,
    pub total_tasks: usize,
    pub success_rate: f64,
    pub average_quality: f64,
    pub skill_level_changes: usize,
    pub skills_by_domain: HashMap<SkillDomain, SkillLevel>,
}

impl Default for CurriculumLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}
