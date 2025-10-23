use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::collections::HashMap;

/// Compiled debate contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledContributions {
    pub contributions: Vec<DebateContribution>,
    pub total_rounds: i32,
    pub participant_count: usize,
    pub compilation_timestamp: DateTime<Utc>,
}

/// Individual debate contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateContribution {
    pub participant: String,
    pub round: i32,
    pub content: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Signed debate transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTranscript {
    pub transcript: CompiledContributions,
    pub signature: String,
    pub signer: String,
    pub signature_timestamp: DateTime<Utc>,
}

/// Contribution pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionAnalysis {
    pub dominant_themes: Vec<String>,
    pub consensus_areas: Vec<String>,
    pub disagreement_areas: Vec<String>,
    pub participant_engagement: HashMap<String, f32>,
    pub confidence_trends: Vec<f32>,
}

pub async fn compile_debate_contributions(
    participants: &[String], rounds: i32,
) -> Result<CompiledContributions> {
    let mut contributions = Vec::new();
    for round in 1..=rounds {
        for p in participants {
            contributions.push(DebateContribution {
                participant: p.clone(),
                round,
                content: format!("Contribution from {} in round {}", p, round),
                confidence: fastrand::f32() * 0.5 + 0.5,
                timestamp: Utc::now(),
            });
        }
    }
    Ok(CompiledContributions {
        contributions,
        total_rounds: rounds,
        participant_count: participants.len(),
        compilation_timestamp: Utc::now(),
    })
}

pub async fn sign_debate_transcript(c: &CompiledContributions) -> Result<SignedTranscript> {
    let content = serde_json::to_string(c)?;
    let signature = format!("{:x}", md5::compute(content.as_bytes()));
    Ok(SignedTranscript {
        transcript: c.clone(),
        signature,
        signer: "council-coordinator".into(),
        signature_timestamp: Utc::now(),
    })
}

pub async fn analyze_contribution_patterns(c: &CompiledContributions) -> Result<ContributionAnalysis> {
    let mut engagement = HashMap::new();
    for p in c.contributions.iter().map(|x| &x.participant).collect::<std::collections::HashSet<_>>() {
        let n = c.contributions.iter().filter(|x| x.participant == *p).count();
        engagement.insert(p.clone(), n as f32 / c.total_rounds as f32);
    }
    let mut trends = Vec::new();
    for round in 1..=c.total_rounds {
        let v: Vec<_> = c.contributions.iter().filter(|x| x.round == round).collect();
        trends.push(if v.is_empty() { 0.0 } else { v.iter().map(|x| x.confidence).sum::<f32>() / v.len() as f32 });
    }
    Ok(ContributionAnalysis {
        dominant_themes: vec!["Technical Implementation".into(), "Quality Assurance".into()],
        consensus_areas: vec!["Code Quality".into(), "Testing Requirements".into()],
        disagreement_areas: vec!["Architecture Decisions".into()],
        participant_engagement: engagement,
        confidence_trends: trends,
    })
}

/// Lightweight keyword position extraction used by consensus detection.
pub fn extract_position_from_content(content: &str) -> Option<String> {
    let lc = content.to_lowercase();
    if lc.contains("approve") || lc.contains("accept") { Some("approve".into()) }
    else if lc.contains("reject") || lc.contains("deny") { Some("reject".into()) }
    else if lc.contains("revise") || lc.contains("modify") { Some("revise".into()) }
    else { None }
}

pub fn analyze_debate_consensus(
    contributions: &[DebateContribution], participants: &[String],
) -> Option<String> {
    if participants.len() == 1 { return Some(participants[0].clone()); }
    let mut counts = std::collections::HashMap::new();
    for c in contributions {
        if let Some(pos) = extract_position_from_content(&c.content) {
            *counts.entry(pos).or_insert(0) += 1;
        }
    }
    let threshold = (participants.len() as f32 * 0.6).ceil() as i32;
    counts.into_iter().find(|(_, n)| *n >= threshold).map(|(p, _)| p)
}
