//! Interactive clarification system for handling ambiguous tasks
//!
//! Provides capabilities to detect ambiguity, request clarification from users,
//! and incorporate clarified information into the planning process.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Clarification request for ambiguous tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationRequest {
    pub id: String,
    pub task_description: String,
    pub ambiguity_detected: DateTime<Utc>,
    pub questions: Vec<ClarificationQuestion>,
    pub context_provided: HashMap<String, String>,
    pub priority: ClarificationPriority,
    pub estimated_importance: f32, // 0.0 to 1.0
}

/// Individual clarification question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub id: String,
    pub question: String,
    pub question_type: QuestionType,
    pub context_hint: Option<String>,
    pub suggested_answers: Vec<String>,
    pub required: bool,
    pub validation_rules: Vec<ValidationRule>,
}

/// Types of clarification questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    /// What specific functionality is needed?
    FunctionalRequirements,
    /// What are the technical constraints?
    TechnicalConstraints,
    /// What is the expected scope/boundaries?
    ScopeBoundaries,
    /// What are the success criteria?
    SuccessCriteria,
    /// What technology stack to use?
    TechnologyStack,
    /// What is the priority/urgency level?
    PriorityLevel,
    /// What are the performance requirements?
    PerformanceRequirements,
    /// What are the security requirements?
    SecurityRequirements,
    /// What is the target audience/users?
    TargetAudience,
    /// What are the integration requirements?
    IntegrationRequirements,
}

/// Validation rules for clarification answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    NotEmpty,
    MinLength(usize),
    MaxLength(usize),
    Contains(Vec<String>), // Must contain one of these words
    Excludes(Vec<String>), // Must not contain these words
    MatchesPattern(String), // Regex pattern
    Custom(String), // Custom validation logic
}

/// Priority levels for clarification requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClarificationPriority {
    Critical,   // Must be answered before proceeding
    High,       // Strongly recommended
    Medium,     // Helpful but not essential
    Low,        // Nice to have
}

/// Clarification response from user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationResponse {
    pub request_id: String,
    pub answers: HashMap<String, String>,
    pub responded_at: DateTime<Utc>,
    pub confidence_level: f32, // User's confidence in answers (0.0 to 1.0)
}

/// Enhanced task context with clarification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifiedTaskContext {
    pub original_description: String,
    pub clarified_description: String,
    pub clarification_responses: Vec<ClarificationResponse>,
    pub ambiguity_score: f32, // 0.0 (clear) to 1.0 (highly ambiguous)
    pub confidence_score: f32, // Overall confidence in understanding
    pub unresolved_questions: Vec<String>, // Questions that couldn't be answered
}

/// Clarification system for detecting and handling ambiguity
pub struct ClarificationSystem {
    ambiguity_detector: AmbiguityDetector,
    question_generator: QuestionGenerator,
    response_validator: ResponseValidator,
    config: ClarificationConfig,
}

#[derive(Debug, Clone)]
pub struct ClarificationConfig {
    pub enable_auto_clarification: bool,
    pub ambiguity_threshold: f32, // Threshold for triggering clarification
    pub max_clarification_rounds: usize,
    pub timeout_minutes: u64,
    pub require_critical_questions: bool,
}

impl ClarificationSystem {
    pub fn new(config: ClarificationConfig) -> Self {
        Self {
            ambiguity_detector: AmbiguityDetector::new(),
            question_generator: QuestionGenerator::new(),
            response_validator: ResponseValidator::new(),
            config,
        }
    }

    /// Analyze task description for ambiguity and generate clarification requests
    pub async fn analyze_and_clarify(
        &self,
        task_description: &str,
        context: &super::agent::TaskContext,
    ) -> Result<ClarifiedTaskContext, ClarificationError> {
        // Detect ambiguity in the task description
        let ambiguity_analysis = self.ambiguity_detector
            .analyze_ambiguity(task_description, context)
            .await?;

        // If ambiguity is below threshold, return original description
        if ambiguity_analysis.overall_score < self.config.ambiguity_threshold {
            return Ok(ClarifiedTaskContext {
                original_description: task_description.to_string(),
                clarified_description: task_description.to_string(),
                clarification_responses: vec![],
                ambiguity_score: ambiguity_analysis.overall_score,
                confidence_score: 1.0 - ambiguity_analysis.overall_score,
                unresolved_questions: vec![],
            });
        }

        // Generate clarification questions
        let clarification_request = self.question_generator
            .generate_questions(&ambiguity_analysis, context)
            .await?;

        // For this implementation, we'll simulate user responses
        // In a real system, this would involve user interaction
        let simulated_responses = self.simulate_user_responses(&clarification_request);

        // Validate responses
        let validated_responses = self.response_validator
            .validate_responses(&clarification_request, &simulated_responses)?;

        // Build clarified context
        let clarified_description = self.build_clarified_description(
            task_description,
            &validated_responses,
        );

        Ok(ClarifiedTaskContext {
            original_description: task_description.to_string(),
            clarified_description,
            clarification_responses: validated_responses,
            ambiguity_score: ambiguity_analysis.overall_score,
            confidence_score: self.calculate_confidence_score(&validated_responses),
            unresolved_questions: vec![], // All questions answered in simulation
        })
    }

    /// Simulate user responses for demonstration
    fn simulate_user_responses(&self, request: &ClarificationRequest) -> Vec<ClarificationResponse> {
        // Simulate realistic user responses based on question types
        let mut responses = Vec::new();

        for question in &request.questions {
            let answer = match question.question_type {
                QuestionType::FunctionalRequirements => {
                    "I need a user authentication system with login, logout, and password reset functionality".to_string()
                },
                QuestionType::TechnicalConstraints => {
                    "Should work with React frontend, Node.js backend, PostgreSQL database".to_string()
                },
                QuestionType::ScopeBoundaries => {
                    "Focus on core authentication features, exclude social login and MFA for now".to_string()
                },
                QuestionType::SuccessCriteria => {
                    "Users can register, login securely, and reset passwords when forgotten".to_string()
                },
                QuestionType::TechnologyStack => {
                    "React, Express.js, PostgreSQL, JWT for tokens".to_string()
                },
                QuestionType::PriorityLevel => {
                    "High priority - needed for product launch".to_string()
                },
                QuestionType::PerformanceRequirements => {
                    "Should handle 1000 concurrent users with <2 second response times".to_string()
                },
                QuestionType::SecurityRequirements => {
                    "Passwords must be hashed, JWT tokens should expire, HTTPS required".to_string()
                },
                QuestionType::TargetAudience => {
                    "End users of our SaaS platform, technical skill level varies".to_string()
                },
                QuestionType::IntegrationRequirements => {
                    "Should integrate with existing user management API".to_string()
                },
            };

            responses.push(ClarificationResponse {
                request_id: request.id.clone(),
                answers: std::iter::once((question.id.clone(), answer)).collect(),
                responded_at: Utc::now(),
                confidence_level: 0.8, // Simulated user confidence
            });
        }

        responses
    }

    /// Build clarified description from original + responses
    fn build_clarified_description(
        &self,
        original: &str,
        responses: &[ClarificationResponse],
    ) -> String {
        let mut clarified = original.to_string();

        // Add clarification context
        clarified.push_str("\n\nClarified Requirements:");
        for response in responses {
            for (question_id, answer) in &response.answers {
                clarified.push_str(&format!("\n• {}: {}", question_id, answer));
            }
        }

        clarified
    }

    /// Calculate overall confidence score from responses
    fn calculate_confidence_score(&self, responses: &[ClarificationResponse]) -> f32 {
        if responses.is_empty() {
            return 0.5;
        }

        let avg_confidence: f32 = responses.iter()
            .map(|r| r.confidence_level)
            .sum::<f32>() / responses.len() as f32;

        // Weight by number of responses (more clarification = higher confidence)
        let response_weight = (responses.len() as f32 / 10.0).min(1.0);

        (avg_confidence * 0.7) + (response_weight * 0.3)
    }
}

/// Ambiguity detection system
pub struct AmbiguityDetector;

impl AmbiguityDetector {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_ambiguity(
        &self,
        task_description: &str,
        context: &super::agent::TaskContext,
    ) -> Result<AmbiguityAnalysis, ClarificationError> {
        let mut analysis = AmbiguityAnalysis {
            overall_score: 0.0,
            factors: vec![],
            detected_issues: vec![],
        };

        // Analyze task description length
        if task_description.len() < 20 {
            analysis.factors.push(AmbiguityFactor {
                factor_type: AmbiguityFactorType::TooBrief,
                score: 0.8,
                description: "Task description is too brief".to_string(),
            });
        }

        // Check for vague terms
        let vague_words = ["better", "improve", "enhance", "fix", "update", "make it"];
        for word in vague_words {
            if task_description.to_lowercase().contains(word) {
                analysis.factors.push(AmbiguityFactor {
                    factor_type: AmbiguityFactorType::VagueTerminology,
                    score: 0.6,
                    description: format!("Contains vague term: '{}'", word),
                });
                break;
            }
        }

        // Check for missing technical details
        if !task_description.to_lowercase().contains("api") &&
           !task_description.to_lowercase().contains("database") &&
           !task_description.to_lowercase().contains("frontend") &&
           !task_description.to_lowercase().contains("backend") {
            analysis.factors.push(AmbiguityFactor {
                factor_type: AmbiguityFactorType::MissingTechnicalDetails,
                score: 0.5,
                description: "No technical architecture details specified".to_string(),
            });
        }

        // Check for pronouns without context
        let pronouns = ["it", "this", "that", "these", "those"];
        for pronoun in pronouns {
            if task_description.to_lowercase().contains(&format!(" {}", pronoun)) {
                analysis.factors.push(AmbiguityFactor {
                    factor_type: AmbiguityFactorType::UnresolvedPronouns,
                    score: 0.4,
                    description: format!("Contains unresolved pronoun: '{}'", pronoun),
                });
                break;
            }
        }

        // Calculate overall score
        if !analysis.factors.is_empty() {
            analysis.overall_score = analysis.factors.iter()
                .map(|f| f.score)
                .sum::<f32>() / analysis.factors.len() as f32;
        }

        // Extract detected issues
        analysis.detected_issues = analysis.factors.iter()
            .map(|f| f.description.clone())
            .collect();

        Ok(analysis)
    }
}

/// Question generation system
pub struct QuestionGenerator;

impl QuestionGenerator {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_questions(
        &self,
        analysis: &AmbiguityAnalysis,
        context: &super::agent::TaskContext,
    ) -> Result<ClarificationRequest, ClarificationError> {
        let mut questions = Vec::new();

        // Generate questions based on detected ambiguity factors
        for factor in &analysis.factors {
            match factor.factor_type {
                AmbiguityFactorType::TooBrief => {
                    questions.push(ClarificationQuestion {
                        id: "func_req".to_string(),
                        question: "What specific functionality do you need implemented?".to_string(),
                        question_type: QuestionType::FunctionalRequirements,
                        context_hint: Some("Please describe the features and capabilities required".to_string()),
                        suggested_answers: vec![
                            "User authentication and authorization".to_string(),
                            "Data processing and analysis".to_string(),
                            "API endpoints for data access".to_string(),
                        ],
                        required: true,
                        validation_rules: vec![ValidationRule::NotEmpty, ValidationRule::MinLength(20)],
                    });
                },
                AmbiguityFactorType::VagueTerminology => {
                    questions.push(ClarificationQuestion {
                        id: "success_criteria".to_string(),
                        question: "What are the specific success criteria for this task?".to_string(),
                        question_type: QuestionType::SuccessCriteria,
                        context_hint: Some("How will you know when this task is complete?".to_string()),
                        suggested_answers: vec![
                            "All tests pass".to_string(),
                            "Performance meets requirements".to_string(),
                            "Users can perform required actions".to_string(),
                        ],
                        required: true,
                        validation_rules: vec![ValidationRule::NotEmpty],
                    });
                },
                AmbiguityFactorType::MissingTechnicalDetails => {
                    questions.push(ClarificationQuestion {
                        id: "tech_stack".to_string(),
                        question: "What technology stack should be used?".to_string(),
                        question_type: QuestionType::TechnologyStack,
                        context_hint: Some(format!("Based on your repo's primary language: {}", context.repo_info.primary_language)),
                        suggested_answers: vec![
                            format!("{} with standard frameworks", context.repo_info.primary_language),
                            "Modern web technologies (React, Node.js)".to_string(),
                            "Cloud-native (AWS, Docker, Kubernetes)".to_string(),
                        ],
                        required: false,
                        validation_rules: vec![ValidationRule::NotEmpty],
                    });
                },
                AmbiguityFactorType::UnresolvedPronouns => {
                    questions.push(ClarificationQuestion {
                        id: "scope_boundaries".to_string(),
                        question: "What are the boundaries and scope of this task?".to_string(),
                        question_type: QuestionType::ScopeBoundaries,
                        context_hint: Some("What should be included and what should be excluded?".to_string()),
                        suggested_answers: vec![
                            "Core functionality only".to_string(),
                            "Include related features".to_string(),
                            "Focus on specific components".to_string(),
                        ],
                        required: true,
                        validation_rules: vec![ValidationRule::NotEmpty],
                    });
                },
            }
        }

        // Add performance and security questions for high-risk tasks
        if analysis.overall_score > 0.6 {
            questions.push(ClarificationQuestion {
                id: "performance_req".to_string(),
                question: "What are the performance requirements?".to_string(),
                question_type: QuestionType::PerformanceRequirements,
                context_hint: Some("Response times, throughput, scalability needs".to_string()),
                suggested_answers: vec![
                    "Standard web application performance".to_string(),
                    "High-performance real-time system".to_string(),
                    "Batch processing with specific SLAs".to_string(),
                ],
                required: false,
                validation_rules: vec![ValidationRule::NotEmpty],
            });

            questions.push(ClarificationQuestion {
                id: "security_req".to_string(),
                question: "What are the security requirements?".to_string(),
                question_type: QuestionType::SecurityRequirements,
                context_hint: Some("Authentication, authorization, data protection needs".to_string()),
                suggested_answers: vec![
                    "Standard web security practices".to_string(),
                    "High-security with encryption and audit trails".to_string(),
                    "Compliance with specific security standards".to_string(),
                ],
                required: false,
                validation_rules: vec![ValidationRule::NotEmpty],
            });
        }

        Ok(ClarificationRequest {
            id: format!("CLARIFY-{}", uuid::Uuid::new_v4().simple()),
            task_description: "Task with detected ambiguity".to_string(),
            ambiguity_detected: Utc::now(),
            questions,
            context_provided: std::collections::HashMap::new(),
            priority: if analysis.overall_score > 0.7 {
                ClarificationPriority::Critical
            } else {
                ClarificationPriority::High
            },
            estimated_importance: analysis.overall_score,
        })
    }
}

/// Response validation system
pub struct ResponseValidator;

impl ResponseValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_responses(
        &self,
        request: &ClarificationRequest,
        responses: &[ClarificationResponse],
    ) -> Result<Vec<ClarificationResponse>, ClarificationError> {
        let mut validated = Vec::new();

        for question in &request.questions {
            // Find response for this question
            let response = responses.iter()
                .find(|r| r.answers.contains_key(&question.id))
                .ok_or_else(|| ClarificationError::MissingResponse(question.id.clone()))?;

            let answer = response.answers.get(&question.id)
                .ok_or_else(|| ClarificationError::MissingResponse(question.id.clone()))?;

            // Validate answer against rules
            for rule in &question.validation_rules {
                match rule {
                    ValidationRule::NotEmpty => {
                        if answer.trim().is_empty() {
                            return Err(ClarificationError::ValidationFailed(
                                format!("Answer for '{}' cannot be empty", question.id)
                            ));
                        }
                    },
                    ValidationRule::MinLength(min) => {
                        if answer.len() < *min {
                            return Err(ClarificationError::ValidationFailed(
                                format!("Answer for '{}' must be at least {} characters", question.id, min)
                            ));
                        }
                    },
                    ValidationRule::MaxLength(max) => {
                        if answer.len() > *max {
                            return Err(ClarificationError::ValidationFailed(
                                format!("Answer for '{}' must be at most {} characters", question.id, max)
                            ));
                        }
                    },
                    ValidationRule::Contains(required) => {
                        let has_required = required.iter()
                            .any(|word| answer.to_lowercase().contains(&word.to_lowercase()));
                        if !has_required {
                            return Err(ClarificationError::ValidationFailed(
                                format!("Answer for '{}' must contain one of: {:?}", question.id, required)
                            ));
                        }
                    },
                    _ => {} // Other rules not implemented in demo
                }
            }

            validated.push(response.clone());
        }

        Ok(validated)
    }
}

/// Ambiguity analysis results
#[derive(Debug)]
pub struct AmbiguityAnalysis {
    pub overall_score: f32,
    pub factors: Vec<AmbiguityFactor>,
    pub detected_issues: Vec<String>,
}

/// Individual ambiguity factor
#[derive(Debug)]
pub struct AmbiguityFactor {
    pub factor_type: AmbiguityFactorType,
    pub score: f32,
    pub description: String,
}

/// Types of ambiguity factors
#[derive(Debug)]
pub enum AmbiguityFactorType {
    TooBrief,
    VagueTerminology,
    MissingTechnicalDetails,
    UnresolvedPronouns,
}

/// Clarification system errors
#[derive(Debug, thiserror::Error)]
pub enum ClarificationError {
    #[error("Missing response for question: {0}")]
    MissingResponse(String),

    #[error("Response validation failed: {0}")]
    ValidationFailed(String),

    #[error("Clarification timeout exceeded")]
    TimeoutExceeded,

    #[error("Maximum clarification rounds exceeded")]
    MaxRoundsExceeded,

    #[error("User cancelled clarification process")]
    UserCancelled,

    #[error("Internal error: {0}")]
    InternalError(String),
}
