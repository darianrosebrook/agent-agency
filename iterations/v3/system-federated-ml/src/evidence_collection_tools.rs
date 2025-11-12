//! Evidence Collection Tools - Claim Extraction and Fact Verification
//!
//! Implements CAWS-compliant evidence collection through claim extraction,
//! fact verification, and source validation mechanisms.

use anyhow::Result;
use std::sync::Arc;
use chrono::Utc;
use tracing::{info, debug};

use crate::claim_extraction::*;
use crate::fact_verification::*;
use crate::source_validation::*;
use crate::evidence_types::{
    ProcessingContext, ProcessingConfig, ClaimExtractionResult, VerificationResult,
    VerificationStatus, SourceCredibility, EvidenceResult, EvidenceMetadata,
};

/// Evidence collection tool suite
#[derive(Debug)]
pub struct EvidenceCollectionTool {
    /// Claim extractor for atomic claim decomposition
    pub claim_extractor: Arc<ClaimExtractor>,
    /// Fact verifier for evidence validation
    pub fact_verifier: Arc<FactVerifier>,
    /// Source validator for evidence credibility assessment
    pub source_validator: Arc<SourceValidator>,
}

impl EvidenceCollectionTool {
    /// Create a new evidence collection tool
    pub async fn new() -> Result<Self> {
        let claim_extractor = Arc::new(ClaimExtractor::new().await?);
        let fact_verifier = Arc::new(FactVerifier::new().await?);
        let source_validator = Arc::new(SourceValidator::new().await?);

        Ok(Self {
            claim_extractor,
            fact_verifier,
            source_validator,
        })
    }

    /// Collect evidence from tasks using claim extraction, fact verification, and source validation
    /// 
    /// This implementation:
    /// 1. Extracts claims from each task's content
    /// 2. Verifies claims using fact verification
    /// 3. Assesses source credibility
    /// 4. Returns comprehensive evidence results
    pub async fn collect_evidence(&self, tasks: &[serde_json::Value], context: &str) -> Result<Vec<serde_json::Value>> {
        info!("Collecting evidence from {} tasks", tasks.len());
        
        let start_time = Utc::now();
        let mut all_evidence = Vec::new();
        let mut total_claims_extracted = 0;
        let mut total_claims_verified = 0;
        
        // Create processing context
        let processing_context = ProcessingContext {
            source_id: "evidence_collection".to_string(),
            timestamp: Utc::now(),
            config: ProcessingConfig {
                max_claims: 100,
                confidence_threshold: 0.7,
                enable_verification: true,
                enable_source_validation: true,
            },
        };
        
        for (task_idx, task) in tasks.iter().enumerate() {
            debug!("Processing task {} of {}", task_idx + 1, tasks.len());
            
            // Extract content from task JSON
            let content = self.extract_content_from_task(task)?;
            let content_type = self.determine_content_type(task, &content);
            let source_id = self.extract_source_id(task, task_idx);
            
            // Step 1: Extract claims from task content
            let extraction_result = self.claim_extractor
                .extract_claims(&content, &content_type, &processing_context)
                .await?;
            
            total_claims_extracted += extraction_result.claims.len();
            
            if extraction_result.claims.is_empty() {
                debug!("No claims extracted from task {}", task_idx);
                continue;
            }
            
            // Step 2: Verify claims
            let verification_results = self.fact_verifier
                .verify_claims(&extraction_result.claims, &processing_context)
                .await?;
            
            total_claims_verified += verification_results.iter()
                .filter(|r| matches!(r.status, VerificationStatus::Verified))
                .count();
            
            // Step 3: Assess source credibility
            let source_credibility = self.source_validator
                .assess_source(&source_id, Some(&content), &processing_context)
                .await?;
            
            // Step 4: Build evidence result
            let overall_confidence = self.calculate_overall_confidence(
                &extraction_result,
                &verification_results,
                &source_credibility,
            );
            
            let evidence_result = EvidenceResult {
                claims: extraction_result.claims.clone(),
                verifications: verification_results,
                source_credibility,
                overall_confidence,
                metadata: EvidenceMetadata {
                    start_time: start_time,
                    processing_time_ms: (Utc::now() - start_time).num_milliseconds() as u64,
                    sources_processed: 1,
                    claims_extracted: extraction_result.claims.len(),
                    claims_verified: total_claims_verified,
                },
            };
            
            // Convert to JSON for return
            let evidence_json = serde_json::to_value(&evidence_result)
                .map_err(|e| anyhow::anyhow!("Failed to serialize evidence result: {}", e))?;
            
            all_evidence.push(evidence_json);
        }
        
        info!(
            "Evidence collection completed: {} tasks processed, {} claims extracted, {} verified",
            tasks.len(),
            total_claims_extracted,
            total_claims_verified
        );
        
        Ok(all_evidence)
    }
    
    /// Extract text content from a task JSON value
    fn extract_content_from_task(&self, task: &serde_json::Value) -> Result<String> {
        // Try multiple common fields for content
        if let Some(content) = task.get("content").and_then(|v| v.as_str()) {
            return Ok(content.to_string());
        }
        if let Some(description) = task.get("description").and_then(|v| v.as_str()) {
            return Ok(description.to_string());
        }
        if let Some(text) = task.get("text").and_then(|v| v.as_str()) {
            return Ok(text.to_string());
        }
        if let Some(body) = task.get("body").and_then(|v| v.as_str()) {
            return Ok(body.to_string());
        }
        
        // Fallback: serialize the entire task as JSON string
        Ok(serde_json::to_string(task)?)
    }
    
    /// Determine content type from task structure
    fn determine_content_type(&self, task: &serde_json::Value, content: &str) -> String {
        // Check explicit type field
        if let Some(type_str) = task.get("type").and_then(|v| v.as_str()) {
            return type_str.to_lowercase();
        }
        
        // Infer from content or task structure
        let content_lower = content.to_lowercase();
        if content_lower.contains("function") || content_lower.contains("class") || content_lower.contains("code") {
            "code".to_string()
        } else if content_lower.contains("research") || content_lower.contains("study") || content_lower.contains("finding") {
            "research".to_string()
        } else if task.get("documentation").is_some() || content_lower.contains("documentation") {
            "documentation".to_string()
        } else {
            "general".to_string()
        }
    }
    
    /// Extract source identifier from task
    fn extract_source_id(&self, task: &serde_json::Value, task_idx: usize) -> String {
        if let Some(source) = task.get("source").and_then(|v| v.as_str()) {
            return source.to_string();
        }
        if let Some(source_id) = task.get("source_id").and_then(|v| v.as_str()) {
            return source_id.to_string();
        }
        if let Some(id) = task.get("id").and_then(|v| v.as_str()) {
            return id.to_string();
        }
        
        format!("task_{}", task_idx)
    }
    
    /// Calculate overall confidence from extraction, verification, and source credibility
    fn calculate_overall_confidence(
        &self,
        extraction_result: &ClaimExtractionResult,
        verification_results: &[VerificationResult],
        source_credibility: &SourceCredibility,
    ) -> f64 {
        // Weighted average of:
        // - Extraction confidence (30%)
        // - Verification confidence (50%)
        // - Source credibility (20%)
        
        let extraction_score = extraction_result.metadata.confidence_score;
        
        let verification_score = if verification_results.is_empty() {
            0.5 // Default if no verifications
        } else {
            let avg_confidence: f64 = verification_results
                .iter()
                .map(|r| r.confidence)
                .sum::<f64>() / verification_results.len() as f64;
            avg_confidence
        };
        
        let source_score = source_credibility.overall_score;
        
        (extraction_score * 0.3) + (verification_score * 0.5) + (source_score * 0.2)
    }
}

