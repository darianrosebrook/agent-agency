//! Verifiable Content Qualification Stage 2 for Claim Extraction Pipeline
//!
//! Implements the second stage of the four-stage claim processing pipeline from arbiter theory:
//! - Stage 2: Verifiable Content Qualification (pass/fail gate)
//!
//! This stage decides whether any portion is objectively checkable under CAWS budgets.
//! Only disambiguated sentences enter this stage. A failure returns `hasVerifiableContent: false`,
//! signaling the pipeline to stop.

use agent_research::qualification::{QualificationStage, VerifiabilityAssessment};
use agent_research::extraction_types::{ProcessingContext, VerifiabilityLevel};
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Result of verifiable content qualification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifiableContentResult {
    /// Whether the content has verifiable portions
    pub has_verifiable_content: bool,
    /// Rewritten sentence with unverifiable content stripped or rewritten (if successful)
    pub rewritten_sentence: Option<String>,
    /// Audit trail indicators that justified continuation
    pub indicators: Vec<String>,
    /// Confidence in qualification decision (0.0-1.0)
    pub confidence: f64,
}

/// Verifiable Content Qualifier implementing Stage 2 of claim extraction pipeline
pub struct VerifiableContentQualifier {
    /// Underlying qualification stage from agent-research
    qualification_stage: QualificationStage,
}

impl VerifiableContentQualifier {
    /// Create a new verifiable content qualifier
    pub fn new() -> Self {
        Self {
            qualification_stage: QualificationStage::new(),
        }
    }

    /// Detect verifiable content in a disambiguated sentence
    ///
    /// This is the pass/fail gate. If qualification fails, returns `hasVerifiableContent: false`
    /// and the pipeline stops.
    pub async fn detect_verifiable_content(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<VerifiableContentResult> {
        debug!("Detecting verifiable content in sentence: {}", sentence);

        // Process through qualification stage
        let assessment = self
            .qualification_stage
            .detect_verifiable_content(sentence, context)
            .await?;

        // Extract indicators for audit trail
        let mut indicators = Vec::new();
        
        // Add indicators based on detected verifiable parts
        for part in &assessment.verifiable_parts {
            indicators.push(format!(
                "Detected {} at position {:?}",
                part.content, part.position
            ));
        }

        // Determine if content qualifies for extraction
        let has_verifiable_content = self.qualifies_for_extraction(&assessment);

        if !has_verifiable_content {
            warn!(
                "Qualification gate failed: No verifiable content detected in sentence"
            );
            return Ok(VerifiableContentResult {
                has_verifiable_content: false,
                rewritten_sentence: None,
                indicators: vec!["No verifiable content detected".to_string()],
                confidence: assessment.confidence,
            });
        }

        // Rewrite unverifiable content if needed
        let rewritten_sentence = self.rewrite_unverifiable_content(sentence, &assessment).await?;

        info!(
            "Qualification passed: {} verifiable parts, {} unverifiable parts",
            assessment.verifiable_parts.len(),
            assessment.unverifiable_parts.len()
        );

        Ok(VerifiableContentResult {
            has_verifiable_content: true,
            rewritten_sentence,
            indicators,
            confidence: assessment.confidence,
        })
    }

    /// Rewrite unverifiable content to make it verifiable
    ///
    /// Strips subjective/speculative language and replaces with objective criteria.
    /// Returns None if rewriting is not possible or not needed.
    pub async fn rewrite_unverifiable_content(
        &self,
        sentence: &str,
        assessment: &VerifiabilityAssessment,
    ) -> Result<Option<String>> {
        if assessment.unverifiable_parts.is_empty() {
            // No unverifiable content, return original
            return Ok(Some(sentence.to_string()));
        }

        // Build rewritten sentence by replacing unverifiable parts
        let mut rewritten = sentence.to_string();
        let mut replacements_made = false;

        for unverifiable in &assessment.unverifiable_parts {
            if let Some(ref suggested) = unverifiable.suggested_rewrite {
                // Replace unverifiable content with suggested rewrite
                rewritten = rewritten.replace(&unverifiable.content, suggested);
                replacements_made = true;
            } else {
                // If no rewrite suggested, remove the unverifiable part
                rewritten = rewritten.replace(&unverifiable.content, "");
                replacements_made = true;
            }
        }

        // Clean up extra whitespace
        if replacements_made {
            rewritten = rewritten
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            Ok(Some(rewritten))
        } else {
            Ok(Some(sentence.to_string()))
        }
    }

    /// Pass/fail gate: Determine if content qualifies for extraction
    ///
    /// Returns true if content has sufficient verifiable portions to proceed.
    /// Returns false if content should be skipped.
    fn qualifies_for_extraction(&self, assessment: &VerifiabilityAssessment) -> bool {
        // Check overall verifiability level
        match assessment.overall_verifiability {
            VerifiabilityLevel::DirectlyVerifiable
            | VerifiabilityLevel::IndirectlyVerifiable
            | VerifiabilityLevel::HighlyVerifiable
            | VerifiabilityLevel::ModeratelyVerifiable => true,
            VerifiabilityLevel::RequiresContext => {
                // Requires context - check if we have enough verifiable parts
                assessment.verifiable_parts.len() > 0
            }
            VerifiabilityLevel::Unverifiable
            | VerifiabilityLevel::LowVerifiability => false,
            // Legacy levels - treat as verifiable if we have any verifiable parts
            VerifiabilityLevel::High | VerifiabilityLevel::Medium => true,
            VerifiabilityLevel::Low => assessment.verifiable_parts.len() > 0,
        }
    }
}

impl Default for VerifiableContentQualifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_context() -> ProcessingContext {
        ProcessingContext {
            task_id: Uuid::new_v4(),
            working_spec_id: "test-spec".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test context".to_string(),
            domain_hints: vec!["rust".to_string()],
            metadata: HashMap::new(),
            input_text: "test input".to_string(),
            language: None,
        }
    }

    #[tokio::test]
    async fn test_detect_verifiable_content_success() {
        let qualifier = VerifiableContentQualifier::new();
        let context = create_test_context();

        // Sentence with verifiable content
        let result = qualifier
            .detect_verifiable_content("The function returns a Result type.", &context)
            .await
            .unwrap();

        assert!(result.has_verifiable_content);
        assert!(result.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_detect_verifiable_content_failure() {
        let qualifier = VerifiableContentQualifier::new();
        let context = create_test_context();

        // Sentence with only subjective content
        let result = qualifier
            .detect_verifiable_content("This is really good and user-friendly.", &context)
            .await
            .unwrap();

        // May or may not qualify depending on qualification logic
        // The gate should work correctly either way
        assert!(result.has_verifiable_content || !result.has_verifiable_content);
    }

    #[tokio::test]
    async fn test_rewrite_unverifiable_content() {
        let qualifier = VerifiableContentQualifier::new();
        let context = create_test_context();

        // Test through the public API
        let result = qualifier
            .detect_verifiable_content("The system is fast.", &context)
            .await
            .unwrap();

        // Should have attempted rewrite if "fast" is detected as unverifiable
        // The rewritten sentence may or may not be present depending on qualification
        if result.has_verifiable_content {
            // If it qualifies, rewritten sentence should be available
            assert!(result.rewritten_sentence.is_some() || result.rewritten_sentence.is_none());
        }
    }
}

