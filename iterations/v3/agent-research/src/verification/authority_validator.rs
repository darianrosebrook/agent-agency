//! Authority attribution and validation logic
//!
//! This module handles source credibility, authority scoring, and trust assessment.

use std::collections::HashMap;
use anyhow::Result;

/// Authority validator for source credibility assessment
pub struct AuthorityValidator {
    // Known authoritative domains
    authoritative_domains: HashMap<String, f64>,
    // Known unreliable domains
    unreliable_domains: HashMap<String, f64>,
}

impl AuthorityValidator {
    pub fn new() -> Self {
        let mut authoritative_domains = HashMap::new();
        let mut unreliable_domains = HashMap::new();
        
        // Initialize authoritative domains with high credibility scores
        authoritative_domains.insert("github.com".to_string(), 0.9);
        authoritative_domains.insert("stackoverflow.com".to_string(), 0.8);
        authoritative_domains.insert("docs.rs".to_string(), 0.95);
        authoritative_domains.insert("crates.io".to_string(), 0.9);
        authoritative_domains.insert("rust-lang.org".to_string(), 0.95);
        authoritative_domains.insert("developer.mozilla.org".to_string(), 0.9);
        authoritative_domains.insert("w3.org".to_string(), 0.95);
        authoritative_domains.insert("ietf.org".to_string(), 0.95);
        authoritative_domains.insert("rfc-editor.org".to_string(), 0.95);
        authoritative_domains.insert("nist.gov".to_string(), 0.95);
        
        // Initialize unreliable domains with low credibility scores
        unreliable_domains.insert("wikipedia.org".to_string(), 0.6);
        unreliable_domains.insert("reddit.com".to_string(), 0.4);
        unreliable_domains.insert("blogspot.com".to_string(), 0.3);
        unreliable_domains.insert("wordpress.com".to_string(), 0.3);
        unreliable_domains.insert("medium.com".to_string(), 0.5);
        
        Self {
            authoritative_domains,
            unreliable_domains,
        }
    }

    /// Validate authority attribution and source credibility
    pub async fn validate_authority(&self, claim: &str, sources: &[String]) -> Result<AuthorityValidation> {
        let mut source_scores = HashMap::new();
        let mut total_score = 0.0;
        let mut valid_sources = 0;
        
        for source in sources {
            let score = self.assess_source_credibility(source);
            source_scores.insert(source.clone(), score);
            
            if score > 0.0 {
                total_score += score;
                valid_sources += 1;
            }
        }
        
        let overall_score = if valid_sources > 0 {
            total_score / valid_sources as f64
        } else {
            0.0
        };
        
        let credibility_assessment = self.generate_credibility_assessment(claim, &source_scores, overall_score);
        
        Ok(AuthorityValidation {
            overall_score,
            source_scores,
            credibility_assessment,
        })
    }

    /// Assess credibility of a single source
    fn assess_source_credibility(&self, source: &str) -> f64 {
        let source_lower = source.to_lowercase();
        
        // Check domain-based credibility
        if let Some(domain) = self.extract_domain(&source_lower) {
            // Check authoritative domains
            if let Some(&score) = self.authoritative_domains.get(&domain) {
                return score;
            }
            
            // Check unreliable domains
            if let Some(&score) = self.unreliable_domains.get(&domain) {
                return score;
            }
        }
        
        // Check for HTTPS (more secure)
        let https_bonus = if source_lower.starts_with("https://") { 0.1 } else { 0.0 };
        
        // Check for official documentation patterns
        let official_bonus = if source_lower.contains("/docs/") || 
                               source_lower.contains("/documentation/") ||
                               source_lower.contains("/api/") {
            0.2
        } else {
            0.0
        };
        
        // Check for academic/research patterns
        let academic_bonus = if source_lower.contains(".edu") || 
                               source_lower.contains("/research/") ||
                               source_lower.contains("/papers/") {
            0.15
        } else {
            0.0
        };
        
        // Check for version control patterns (GitHub, GitLab, etc.)
        let vcs_bonus = if source_lower.contains("github.com") || 
                          source_lower.contains("gitlab.com") ||
                          source_lower.contains("bitbucket.org") {
            0.1
        } else {
            0.0
        };
        
        // Base score for unknown sources
        let base_score = 0.5;
        
        // Combine all factors
        (base_score + https_bonus + official_bonus + academic_bonus + vcs_bonus)
            .min(1.0)
            .max(0.0)
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &str) -> Option<String> {
        // Simple domain extraction - in practice would use a proper URL parser
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                Some(after_protocol[..end].to_string())
            } else {
                Some(after_protocol.to_string())
            }
        } else {
            None
        }
    }

    /// Generate credibility assessment text
    fn generate_credibility_assessment(&self, claim: &str, source_scores: &HashMap<String, f64>, overall_score: f64) -> String {
        let source_count = source_scores.len();
        let high_credibility_sources = source_scores.values().filter(|&&score| score >= 0.8).count();
        let low_credibility_sources = source_scores.values().filter(|&&score| score < 0.5).count();
        
        if overall_score >= 0.8 {
            format!(
                "High credibility: {} sources with {} high-quality references. Claim '{}' is well-supported.",
                source_count, high_credibility_sources, claim
            )
        } else if overall_score >= 0.6 {
            format!(
                "Moderate credibility: {} sources with mixed quality. Claim '{}' has reasonable support but could benefit from more authoritative sources.",
                source_count, claim
            )
        } else if overall_score >= 0.4 {
            format!(
                "Low credibility: {} sources with {} unreliable references. Claim '{}' has weak support and should be verified with more authoritative sources.",
                source_count, low_credibility_sources, claim
            )
        } else {
            format!(
                "Very low credibility: {} sources with {} unreliable references. Claim '{}' lacks credible support and should be treated with skepticism.",
                source_count, low_credibility_sources, claim
            )
        }
    }

    /// Add a new authoritative domain
    pub fn add_authoritative_domain(&mut self, domain: String, score: f64) {
        self.authoritative_domains.insert(domain, score.min(1.0).max(0.0));
    }

    /// Add a new unreliable domain
    pub fn add_unreliable_domain(&mut self, domain: String, score: f64) {
        self.unreliable_domains.insert(domain, score.min(1.0).max(0.0));
    }

    /// Get all authoritative domains
    pub fn get_authoritative_domains(&self) -> &HashMap<String, f64> {
        &self.authoritative_domains
    }

    /// Get all unreliable domains
    pub fn get_unreliable_domains(&self) -> &HashMap<String, f64> {
        &self.unreliable_domains
    }
}

impl Default for AuthorityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Authority validation result
pub struct AuthorityValidation {
    pub overall_score: f64,
    pub source_scores: HashMap<String, f64>,
    pub credibility_assessment: String,
}
