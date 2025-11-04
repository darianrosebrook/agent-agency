//! State management for learning coordination
//!
//! Learning session state, progress tracking, and state persistence
//! for multi-turn learning coordination.

use schemars::JsonSchema;
use std::collections::HashMap;
use uuid::Uuid;

/// Learning session state

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningSession {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub state: SessionState,
    pub progress: LearningProgress,
    pub history: Vec<LearningEvent>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SessionState {
    Initializing,
    Active,
    Paused,
    Completed,
    Failed,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningProgresss {
    pub completed_steps: u32,
    pub total_steps: u32,
    pub current_quality_score: f64,
    pub improvement_trend: Vec<f64>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: EventType,
    pub description: String,
    pub data: HashMap<String, String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EventType {
    Started,
    Progress,
    QualityImproved,
    QualityDegraded,
    ResourceWarning,
    Failure,
    Recovery,
    Completed,
}

/// State manager for learning sessions

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StateManager {
    sessions: HashMap<Uuid, LearningSession>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create new learning session
    pub fn create_session(&mut self) -> Uuid {
        let session_id = Uuid::new_v4();
        let session = LearningSession {
            id: session_id,
            state: SessionState::Initializing,
            progress: LearningProgress {
                completed_steps: 0,
                total_steps: 10, // Placeholder
                current_quality_score: 0.0,
                improvement_trend: vec![],
            },
            history: vec![LearningEvent {
                timestamp: chrono::Utc::now(),
                event_type: EventType::Started,
                description: "Learning session initialized".to_string(),
                data: HashMap::new(),
            }],
        };

        self.sessions.insert(session_id, session);
        session_id
    }

    /// Update session state
    pub fn update_session(&mut self, session_id: Uuid, update: SessionUpdate) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            match update {
                SessionUpdate::State(new_state) => {
                    session.state = new_state;
                }
                SessionUpdate::Progress(progress) => {
                    session.progress = progress;
                }
                SessionUpdate::AddEvent(event) => {
                    session.history.push(event);
                }
            }
        }
    }

    /// Get session state
    pub fn get_session(&self, session_id: Uuid) -> Option<&LearningSession> {
        self.sessions.get(&session_id)
    }
}


#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum SessionUpdatee {
    State(SessionState),
    Progress(LearningProgress),
    AddEvent(LearningEvent),
}


