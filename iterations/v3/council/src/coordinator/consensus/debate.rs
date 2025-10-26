//! Debate protocol for consensus building

use crate::council_types::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Debate coordinator for resolving conflicts
pub struct DebateCoordinator {
    max_rounds: u32,
    convergence_threshold: f32,
    active_debates: HashMap<Uuid, DebateSession>,
}

impl DebateCoordinator {
    pub fn new() -> Self {
        Self {
            max_rounds: 5,
            convergence_threshold: 0.8,
            active_debates: HashMap::new(),
        }
    }

    /// Start a debate session for conflicting verdicts
    pub async fn start_debate(&mut self, session_id: Uuid, conflicting_verdicts: Vec<JudgeEvaluationResult>) -> Result<DebateSession> {
        let debate = DebateSession {
            id: Uuid::new_v4(),
            session_id,
            round: 1,
            max_rounds: self.max_rounds,
            participants: conflicting_verdicts.into_iter()
                .map(|v| DebateParticipant {
                    judge_id: v.judge_id,
                    initial_verdict: v.verdict,
                    current_verdict: v.verdict,
                    confidence: v.confidence,
                    arguments: vec![DebateArgument {
                        round: 1,
                        content: v.reasoning,
                        timestamp: Utc::now(),
                    }],
                })
                .collect(),
            status: DebateStatus::Active,
            start_time: Utc::now(),
        };

        self.active_debates.insert(debate.id, debate.clone());
        Ok(debate)
    }

    /// Execute a debate round
    pub async fn execute_round(&mut self, debate_id: Uuid) -> Result<DebateRoundResult> {
        let debate = self.active_debates.get_mut(&debate_id)
            .ok_or_else(|| anyhow::anyhow!("Debate session not found"))?;

        if debate.status != DebateStatus::Active {
            return Err(anyhow::anyhow!("Debate is not active"));
        }

        debate.round += 1;

        // Generate arguments for this round
        let mut new_arguments = Vec::new();
        for participant in &mut debate.participants {
            let argument = self.generate_argument(participant, debate.round).await?;
            participant.arguments.push(argument.clone());
            new_arguments.push(argument);
        }

        // Check for convergence
        let convergence_score = self.calculate_convergence(debate);
        let consensus_reached = convergence_score >= self.convergence_threshold;

        if consensus_reached || debate.round >= debate.max_rounds {
            debate.status = if consensus_reached {
                DebateStatus::Converged
            } else {
                DebateStatus::Stalemate
            };
        }

        Ok(DebateRoundResult {
            debate_id,
            round: debate.round,
            arguments: new_arguments,
            convergence_score,
            consensus_reached,
            status: debate.status.clone(),
        })
    }

    /// Generate an argument for a participant
    async fn generate_argument(&self, participant: &DebateParticipant, round: u32) -> Result<DebateArgument> {
        // Simplified argument generation
        let content = format!("Round {} argument: Maintaining verdict {} with confidence {:.2}",
                            round, participant.current_verdict, participant.confidence);

        Ok(DebateArgument {
            round,
            content,
            timestamp: Utc::now(),
        })
    }

    /// Calculate convergence score
    fn calculate_convergence(&self, debate: &DebateSession) -> f32 {
        let total_participants = debate.participants.len();
        if total_participants <= 1 {
            return 1.0;
        }

        // Count agreements
        let mut agreements = 0;
        for i in 0..total_participants {
            for j in (i + 1)..total_participants {
                if debate.participants[i].current_verdict == debate.participants[j].current_verdict {
                    agreements += 1;
                }
            }
        }

        let total_possible_agreements = total_participants * (total_participants - 1) / 2;
        agreements as f32 / total_possible_agreements as f32
    }

    /// Get debate status
    pub fn get_debate_status(&self, debate_id: Uuid) -> Option<&DebateSession> {
        self.active_debates.get(&debate_id)
    }
}

/// Debate session
#[derive(Debug, Clone)]
pub struct DebateSession {
    /// Unique debate ID
    pub id: Uuid,
    /// Associated consensus session
    pub session_id: Uuid,
    /// Current debate round
    pub round: u32,
    /// Maximum debate rounds
    pub max_rounds: u32,
    /// Debate participants
    pub participants: Vec<DebateParticipant>,
    /// Debate status
    pub status: DebateStatus,
    /// Start time
    pub start_time: DateTime<Utc>,
}

/// Debate participant
#[derive(Debug, Clone)]
pub struct DebateParticipant {
    /// Judge ID
    pub judge_id: Uuid,
    /// Initial verdict
    pub initial_verdict: FinalVerdict,
    /// Current verdict (may change during debate)
    pub current_verdict: FinalVerdict,
    /// Current confidence
    pub confidence: f32,
    /// Arguments presented
    pub arguments: Vec<DebateArgument>,
}

/// Debate argument
#[derive(Debug, Clone)]
pub struct DebateArgument {
    /// Debate round when argument was made
    pub round: u32,
    /// Argument content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Debate status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebateStatus {
    /// Debate is active
    Active,
    /// Debate reached convergence
    Converged,
    /// Debate reached stalemate
    Stalemate,
    /// Debate was cancelled
    Cancelled,
}

/// Result of a debate round
#[derive(Debug, Clone)]
pub struct DebateRoundResult {
    /// Debate ID
    pub debate_id: Uuid,
    /// Round number
    pub round: u32,
    /// Arguments generated this round
    pub arguments: Vec<DebateArgument>,
    /// Convergence score (0.0-1.0)
    pub convergence_score: f32,
    /// Whether consensus was reached
    pub consensus_reached: bool,
    /// Current debate status
    pub status: DebateStatus,
}
