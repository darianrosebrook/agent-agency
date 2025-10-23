use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Types of resolution outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionType {
    Consensus,
    MajorityVote,
    ExpertOverride,
    RandomSelection,
    Deferred,
}

/// Result of CAWS tie-breaking resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsResolutionResult {
    pub resolution_type: ResolutionType,
    pub winning_participant: Option<String>,
    pub confidence_score: f32,
    pub rationale: String,
    pub applied_rules: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

pub async fn apply_caws_tie_breaking_rules(
    participants: &[String],
    rounds: i32,
) -> Result<CawsResolutionResult> {
    let random_participant = participants[fastrand::usize(..participants.len())].clone();
    Ok(CawsResolutionResult {
        resolution_type: ResolutionType::RandomSelection,
        winning_participant: Some(random_participant),
        confidence_score: 0.3,
        rationale: format!("Random selection applied after {} rounds", rounds),
        applied_rules: vec!["CAWS-RANDOM-004".into()],
        timestamp: Utc::now(),
    })
}

/// Generate resolution rationale
pub async fn generate_resolution_rationale(
    resolution: &CawsResolutionResult,
    participants: &[String],
    rounds: i32,
) -> Result<String> {
    let mut r = format!(
        "Resolution: {:?} | Participants: {} | Rounds: {} | Confidence: {:.2}",
        resolution.resolution_type, participants.len(), rounds, resolution.confidence_score
    );
    if let Some(w) = &resolution.winning_participant { r.push_str(&format!(" | Winner: {}", w)); }
    r.push_str(&format!(" | Rules: {:?}", resolution.applied_rules));
    Ok(r)
}
