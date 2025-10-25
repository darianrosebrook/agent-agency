//! Task Ambiguity Assessment and Clarification System

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid;

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

/// Ambiguity assessment service
pub struct AmbiguityAssessor {
    rule_patterns: HashMap<String, Vec<String>>,
}

impl AmbiguityAssessor {
    pub fn new() -> Self {
        let mut rule_patterns = HashMap::new();

        // Initialize rule-based ambiguity detection patterns
        rule_patterns.insert(
            "vague_subjects".to_string(),
            vec![
                "it".to_string(), "this".to_string(), "that".to_string(),
                "the system".to_string(), "the app".to_string(), "the code".to_string()
            ]
        );

        rule_patterns.insert(
            "missing_success_criteria".to_string(),
            vec![
                "should work".to_string(), "needs to be done".to_string(),
                "make it better".to_string(), "fix it".to_string()
            ]
        );

        rule_patterns.insert(
            "technical_ambiguity".to_string(),
            vec![
                "integrate with".to_string(), "connect to".to_string(),
                "use the API".to_string(), "implement feature".to_string()
            ]
        );

        Self { rule_patterns }
    }

    /// Assess task ambiguity using rule-based detection
    pub fn assess_ambiguity(&self, task_description: &str) -> AmbiguityAssessment {
        let mut ambiguity_score = 0.0;
        let mut ambiguity_types = Vec::new();
        let mut questions = Vec::new();

        // Rule-based ambiguity detection
        let rule_based_issues = self.detect_rule_based_ambiguity(task_description);

        // Calculate score based on detected issues
        if !rule_based_issues.is_empty() {
            ambiguity_score = (rule_based_issues.len() as f32).min(1.0);

            // Convert rule-based issues to ambiguity types
            if rule_based_issues.contains(&"vague_subjects".to_string()) {
                ambiguity_types.push(AmbiguityType::VagueSubject);
                questions.push(ClarificationQuestion {
                    id: "subject_clarification".to_string(),
                    question: "What specific component, feature, or system are you referring to?".to_string(),
                    question_type: QuestionType::FreeForm,
                    suggested_answers: vec![],
                    required: true,
                    priority: QuestionPriority::Critical,
                });
            }

            if rule_based_issues.contains(&"missing_success_criteria".to_string()) {
                ambiguity_types.push(AmbiguityType::UnclearObjectives);
                questions.push(ClarificationQuestion {
                    id: "success_criteria".to_string(),
                    question: "What are the specific success criteria or acceptance conditions?".to_string(),
                    question_type: QuestionType::FreeForm,
                    suggested_answers: vec![],
                    required: true,
                    priority: QuestionPriority::Critical,
                });
            }

            if rule_based_issues.contains(&"technical_ambiguity".to_string()) {
                ambiguity_types.push(AmbiguityType::TechnicalAmbiguity);
                questions.push(ClarificationQuestion {
                    id: "technical_requirements".to_string(),
                    question: "What are the specific technical requirements or constraints?".to_string(),
                    question_type: QuestionType::TechnicalChoice,
                    suggested_answers: vec![
                        "REST API integration".to_string(),
                        "Database schema changes".to_string(),
                        "UI/UX updates".to_string(),
                        "Performance optimization".to_string(),
                    ],
                    required: true,
                    priority: QuestionPriority::Important,
                });
            }
        }

        // Default question if no specific issues detected but some ambiguity
        if questions.is_empty() && ambiguity_score > 0.3 {
            ambiguity_types.push(AmbiguityType::IncompleteRequirements);
            questions.push(ClarificationQuestion {
                id: "general_clarification".to_string(),
                question: "Can you provide more specific details about what needs to be implemented?".to_string(),
                question_type: QuestionType::FreeForm,
                suggested_answers: vec![],
                required: false,
                priority: QuestionPriority::Important,
            });
        }

        AmbiguityAssessment {
            ambiguity_score,
            ambiguity_types,
            clarification_questions: questions,
            clarification_required: ambiguity_score > 0.5,
            assessment_confidence: 0.8, // Rule-based assessment confidence
        }
    }

    /// Rule-based ambiguity detection
    fn detect_rule_based_ambiguity(&self, task_description: &str) -> Vec<String> {
        let mut issues = Vec::new();
        let lower_description = task_description.to_lowercase();

        for (issue_type, patterns) in &self.rule_patterns {
            for pattern in patterns {
                if lower_description.contains(pattern) {
                    issues.push(issue_type.clone());
                    break; // Only add each issue type once
                }
            }
        }

        issues
    }

    /// Process clarification responses and update assessment
    pub fn process_clarification_responses(
        &self,
        assessment: &AmbiguityAssessment,
        responses: &[ClarificationResponse],
    ) -> AmbiguityAssessment {
        let mut updated_assessment = assessment.clone();

        // Reduce ambiguity score based on responses received
        let response_count = responses.len() as f32;
        let question_count = assessment.clarification_questions.len() as f32;

        if question_count > 0.0 {
            let response_ratio = response_count / question_count;
            updated_assessment.ambiguity_score *= (1.0 - response_ratio * 0.7); // Reduce by up to 70%
        }

        // Update clarification required flag
        updated_assessment.clarification_required = updated_assessment.ambiguity_score > 0.3;

        updated_assessment
    }

    /// Create a clarification session
    pub fn create_clarification_session(
        &self,
        task_description: &str,
        assessment: AmbiguityAssessment,
    ) -> ClarificationSession {
        ClarificationSession {
            session_id: uuid::Uuid::new_v4().to_string(),
            original_task: task_description.to_string(),
            assessment,
            questions_asked: vec![],
            responses: vec![],
            status: SessionStatus::Active,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Add response to clarification session
    pub fn add_response_to_session(
        &self,
        mut session: ClarificationSession,
        response: ClarificationResponse,
    ) -> ClarificationSession {
        session.responses.push(response);

        // Check if all required questions have been answered
        let required_questions: Vec<_> = session.assessment.clarification_questions
            .iter()
            .filter(|q| q.required)
            .collect();

        let answered_required = required_questions.iter()
            .all(|q| session.responses.iter().any(|r| r.question_id == q.id));

        if answered_required {
            session.status = SessionStatus::ReadyForPlanning;
            session.completed_at = Some(Utc::now());
        }

        session
    }
}

impl Default for AmbiguityAssessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_based_ambiguity_detection() {
        let assessor = AmbiguityAssessor::new();

        let vague_task = "Fix it so it works better";
        let assessment = assessor.assess_ambiguity(vague_task);

        assert!(assessment.ambiguity_score > 0.0);
        assert!(assessment.clarification_required);
        assert!(!assessment.clarification_questions.is_empty());
    }

    #[test]
    fn test_clear_task_assessment() {
        let assessor = AmbiguityAssessor::new();

        let clear_task = "Add user authentication with JWT tokens, including login, logout, and token refresh endpoints";
        let assessment = assessor.assess_ambiguity(clear_task);

        assert_eq!(assessment.ambiguity_score, 0.0);
        assert!(!assessment.clarification_required);
        assert!(assessment.clarification_questions.is_empty());
    }

    #[test]
    fn test_clarification_session_workflow() {
        let assessor = AmbiguityAssessor::new();

        let task = "Implement the feature";
        let assessment = assessor.assess_ambiguity(task);
        let session = assessor.create_clarification_session(task, assessment);

        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(session.original_task, task);

        // Simulate user response
        let response = ClarificationResponse {
            question_id: "subject_clarification".to_string(),
            response: "Implement user registration feature".to_string(),
            responded_at: Utc::now(),
            notes: None,
        };

        let updated_session = assessor.add_response_to_session(session, response);
        assert_eq!(updated_session.status, SessionStatus::ReadyForPlanning);
    }
}
