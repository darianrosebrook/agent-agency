//! Source Validation Module
//!
//! Assesses the credibility and reliability of information sources
//! to determine trustworthiness for evidence collection.

use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::evidence_types::*;

/// Source credibility assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAssessment {
    /// Source identifier
    pub source_id: String,
    /// Credibility score
    pub credibility: SourceCredibility,
    /// Assessment timestamp
    pub assessed_at: DateTime<Utc>,
    /// Assessment factors
    pub assessment_factors: Vec<String>,
}

/// Source validator for evidence credibility assessment
#[derive(Debug)]
pub struct SourceValidator {
    /// Known source credibility ratings
    source_ratings: HashMap<String, f64>,
    /// Domain authority mappings
    domain_authority: HashMap<String, f64>,
}

impl SourceValidator {
    /// Create a new source validator
    pub async fn new() -> Result<Self> {
        let mut ratings = HashMap::new();
        let mut authority = HashMap::new();

        // Initialize with some known credible sources
        ratings.insert("reputable_journal.org".to_string(), 0.95);
        ratings.insert("academic_university.edu".to_string(), 0.90);
        ratings.insert("government_agency.gov".to_string(), 0.85);

        // Domain authority scores
        authority.insert("edu".to_string(), 0.9);
        authority.insert("gov".to_string(), 0.8);
        authority.insert("org".to_string(), 0.7);
        authority.insert("com".to_string(), 0.5);

        Ok(Self {
            source_ratings: ratings,
            domain_authority: authority,
        })
    }

    /// Assess credibility of a source
    pub async fn assess_source(&self, source_id: &str, content: Option<&str>, context: &ProcessingContext) -> Result<SourceCredibility> {
        debug!("Assessing credibility of source: {}", source_id);

        let mut authority_score = 0.5;
        let mut reliability_score = 0.5;
        let mut bias_score = 0.5;
        let mut recency_score = 1.0;

        // Authority assessment
        if let Some(domain) = self.extract_domain(source_id) {
            authority_score = self.domain_authority.get(&domain).copied().unwrap_or(0.5);
        }

        // Reliability assessment based on known ratings
        reliability_score = self.source_ratings.get(source_id).copied().unwrap_or(0.6);

        // Bias assessment (simplified)
        bias_score = if source_id.contains("news") {
            0.6 // News sources may have bias
        } else if source_id.contains("academic") {
            0.9 // Academic sources typically less biased
        } else {
            0.7 // Neutral assumption
        };

        // Recency assessment
        let age_hours = (Utc::now() - context.timestamp).num_hours();
        recency_score = if age_hours < 24 {
            1.0 // Very recent
        } else if age_hours < 168 { // 1 week
            0.8 // Recent
        } else if age_hours < 720 { // 1 month
            0.6 // Somewhat recent
        } else {
            0.3 // Old
        };

        // Calculate overall score
        let overall_score = (authority_score * 0.3) + (reliability_score * 0.3) + (bias_score * 0.2) + (recency_score * 0.2);

        // Determine supporting and detracting factors
        let mut supporting_factors = Vec::new();
        let mut detracting_factors = Vec::new();

        if authority_score > 0.8 {
            supporting_factors.push("High domain authority".to_string());
        }
        if reliability_score > 0.8 {
            supporting_factors.push("Established reputation".to_string());
        }
        if bias_score > 0.8 {
            supporting_factors.push("Low bias indicators".to_string());
        }
        if recency_score > 0.8 {
            supporting_factors.push("Recent publication".to_string());
        }

        if authority_score < 0.4 {
            detracting_factors.push("Low domain authority".to_string());
        }
        if reliability_score < 0.4 {
            detracting_factors.push("Limited reputation".to_string());
        }
        if bias_score < 0.4 {
            detracting_factors.push("Potential bias concerns".to_string());
        }
        if recency_score < 0.4 {
            detracting_factors.push("Outdated information".to_string());
        }

        Ok(SourceCredibility {
            overall_score,
            authority_score,
            reliability_score,
            bias_score,
            recency_score,
            supporting_factors,
            detracting_factors,
        })
    }

    /// Validate source against known credibility criteria
    pub async fn validate_source(&self, source_id: &str, content: Option<&str>, context: &ProcessingContext) -> Result<bool> {
        let credibility = self.assess_source(source_id, content, context).await?;

        // Accept sources with credibility above threshold
        Ok(credibility.overall_score >= context.config.confidence_threshold)
    }

    /// Extract domain from source identifier
    fn extract_domain(&self, source_id: &str) -> Option<String> {
        // Simple domain extraction - in reality this would be more sophisticated
        if let Some(at_pos) = source_id.find('@') {
            source_id[at_pos + 1..].split('.').last().map(|s| s.to_string())
        } else if source_id.contains('.') {
            source_id.split('.').last().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Get credibility score for a source
    pub fn get_credibility_score(&self, source_id: &str) -> f64 {
        self.source_ratings.get(source_id).copied().unwrap_or(0.5)
    }

    /// Update credibility rating for a source
    pub fn update_credibility(&mut self, source_id: String, new_rating: f64) {
        self.source_ratings.insert(source_id, new_rating.clamp(0.0, 1.0));
    }
}
