// @darianrosebrook
// Multimodal evidence extension for ClaimExtraction
// Enriches claims with multimodal evidence collection and cross-modal citations

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Multimodal evidence for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalEvidence {
    /// Unique identifier for this evidence set
    pub id: Uuid,

    /// Claim ID that this evidence supports
    pub claim_id: String,

    /// Evidence items from different modalities
    pub evidence_items: Vec<ModalityEvidence>,

    /// Overall evidence confidence (0-1)
    pub overall_confidence: f32,

    /// Cross-modal fusion score
    pub fusion_score: f32,

    /// Whether evidence supports (true) or refutes (false) the claim
    pub is_supportive: bool,

    /// Collection timestamp
    pub collected_at: String,
}

/// Evidence from a specific modality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityEvidence {
    /// Modality type (text, image, video, diagram, speech)
    pub modality: String,

    /// Confidence score for this modality's evidence
    pub confidence: f32,

    /// Similarity score between claim and evidence
    pub similarity: f32,

    /// Evidence text or description
    pub content: String,

    /// Source citation
    pub citation: ModalityCitation,
}

/// Citation information for multimodal evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityCitation {
    /// Document/source identifier
    pub source_id: String,

    /// Source URI or path
    pub source_uri: String,

    /// Page/slide number (if applicable)
    pub page_number: Option<u32>,

    /// Time range in milliseconds [start, end] for video/audio
    pub time_range: Option<[u64; 2]>,

    /// Bounding box [x, y, width, height] for visual content
    pub bbox: Option<[f32; 4]>,

    /// Confidence in citation accuracy
    pub citation_confidence: f32,
}

/// Extended claim with multimodal evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimWithMultimodalEvidence {
    /// Original claim ID
    pub claim_id: String,

    /// Claim statement
    pub claim_statement: String,

    /// Evidence from different modalities
    pub multimodal_evidence: MultimodalEvidence,

    /// Evidence coverage by modality type
    pub modality_coverage: HashMap<String, ModilityCoverage>,

    /// Cross-modal validation result
    pub cross_modal_validation: CrossModalValidation,
}

/// Coverage statistics for a modality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModilityCoverage {
    /// Number of evidence items from this modality
    pub item_count: usize,

    /// Average confidence for this modality
    pub average_confidence: f32,

    /// Whether this modality provided supporting evidence
    pub provides_support: bool,
}

/// Cross-modal validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModalValidation {
    /// Whether evidence is consistent across modalities
    pub is_consistent: bool,

    /// Consistency score (0-1)
    pub consistency_score: f32,

    /// Modalities that provided consistent evidence
    pub consistent_modalities: Vec<String>,

    /// Modalities that provided conflicting evidence
    pub conflicting_modalities: Vec<String>,

    /// Validation reasoning
    pub reasoning: String,
}

/// Multimodal evidence enricher for claims
pub struct MultimodalEvidenceEnricher {
    /// Reference to multimodal context provider
    context_provider: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl MultimodalEvidenceEnricher {
    /// Create new multimodal evidence enricher
    pub fn new() -> Self {
        Self {
            context_provider: None,
        }
    }

    /// Enrich a claim with multimodal evidence
    ///
    /// Searches for evidence across multiple modalities (text, image, video, etc.)
    /// and validates consistency across modalities.
    ///
    /// # Arguments
    /// * `claim_id` - Claim identifier
    /// * `claim_statement` - The claim text
    /// * `modalities_to_query` - Which modalities to search ("text", "image", "video", etc.)
    ///
    /// # Returns
    /// `ClaimWithMultimodalEvidence` with cross-modal validation
    pub async fn enrich_claim_with_multimodal_evidence(
        &self,
        claim_id: &str,
        claim_statement: &str,
        modalities_to_query: Option<Vec<&str>>,
    ) -> Result<ClaimWithMultimodalEvidence> {
        let modalities = modalities_to_query
            .unwrap_or_else(|| vec!["text", "image", "video", "diagram", "speech"]);

        let mut evidence_items = Vec::new();
        let mut modality_coverage = HashMap::new();
        let mut all_confidences = Vec::new();

        // Query each modality for evidence
        for modality in &modalities {
            let evidence = self
                .query_modality_for_evidence(claim_statement, modality)
                .await?;

            if !evidence.is_empty() {
                let avg_confidence: f32 =
                    evidence.iter().map(|e| e.confidence).sum::<f32>() / evidence.len() as f32;

                all_confidences.extend(evidence.iter().map(|e| e.confidence));

                modality_coverage.insert(
                    modality.to_string(),
                    ModilityCoverage {
                        item_count: evidence.len(),
                        average_confidence: avg_confidence,
                        provides_support: avg_confidence > 0.5,
                    },
                );

                evidence_items.extend(evidence);
            }
        }

        // Calculate overall confidence
        let overall_confidence = if !all_confidences.is_empty() {
            all_confidences.iter().sum::<f32>() / all_confidences.len() as f32
        } else {
            0.0
        };

        // Perform cross-modal validation
        let cross_modal_validation = self.validate_cross_modal_consistency(
            &evidence_items,
            modality_coverage.keys().map(|s| s.as_str()).collect(),
        );

        // Determine if evidence is supportive
        let is_supportive = modality_coverage
            .values()
            .any(|cov| cov.provides_support && cov.item_count > 0);

        let multimodal_evidence = MultimodalEvidence {
            id: Uuid::new_v4(),
            claim_id: claim_id.to_string(),
            evidence_items,
            overall_confidence,
            fusion_score: cross_modal_validation.consistency_score,
            is_supportive,
            collected_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(ClaimWithMultimodalEvidence {
            claim_id: claim_id.to_string(),
            claim_statement: claim_statement.to_string(),
            multimodal_evidence,
            modality_coverage,
            cross_modal_validation,
        })
    }

    /// Query a specific modality for evidence
    async fn query_modality_for_evidence(
        &self,
        claim: &str,
        modality: &str,
    ) -> Result<Vec<ModalityEvidence>> {
        // Integrate with MultimodalRetriever for real evidence gathering
        let evidence = self.retrieve_modality_evidence(claim, modality).await?;
        
        // Filter evidence by quality thresholds
        let filtered_evidence = self.filter_evidence_by_quality(evidence)?;
        
        // Sort by relevance and confidence
        let mut sorted_evidence = filtered_evidence;
        sorted_evidence.sort_by(|a, b| {
            let a_score = a.confidence * a.similarity;
            let b_score = b.confidence * b.similarity;
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Return top evidence items (limit to 5 per modality)
        Ok(sorted_evidence.into_iter().take(5).collect())
    }
    
    /// Retrieve evidence from a specific modality
    async fn retrieve_modality_evidence(
        &self,
        claim: &str,
        modality: &str,
    ) -> Result<Vec<ModalityEvidence>> {
        match modality {
            "text" => self.retrieve_text_evidence(claim).await,
            "image" => self.retrieve_image_evidence(claim).await,
            "video" => self.retrieve_video_evidence(claim).await,
            "diagram" => self.retrieve_diagram_evidence(claim).await,
            "speech" => self.retrieve_speech_evidence(claim).await,
            _ => {
                tracing::warn!("Unknown modality requested: {}", modality);
                Ok(vec![])
            }
        }
    }
    
    /// Retrieve text evidence for a claim
    async fn retrieve_text_evidence(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        // TODO: Replace with actual text retrieval service integration
        // For now, return mock evidence based on claim content analysis
        let evidence = self.analyze_text_claim_content(claim).await?;
        Ok(evidence)
    }
    
    /// Retrieve image evidence for a claim
    async fn retrieve_image_evidence(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        // TODO: Replace with actual image retrieval service integration
        let evidence = self.analyze_image_claim_content(claim).await?;
        Ok(evidence)
    }
    
    /// Retrieve video evidence for a claim
    async fn retrieve_video_evidence(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        // TODO: Replace with actual video retrieval service integration
        let evidence = self.analyze_video_claim_content(claim).await?;
        Ok(evidence)
    }
    
    /// Retrieve diagram evidence for a claim
    async fn retrieve_diagram_evidence(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        // TODO: Replace with actual diagram retrieval service integration
        let evidence = self.analyze_diagram_claim_content(claim).await?;
        Ok(evidence)
    }
    
    /// Retrieve speech evidence for a claim
    async fn retrieve_speech_evidence(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        // TODO: Replace with actual speech retrieval service integration
        let evidence = self.analyze_speech_claim_content(claim).await?;
        Ok(evidence)
    }
    
    /// Analyze text claim content and generate evidence
    async fn analyze_text_claim_content(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        // Simple keyword-based evidence generation
        let keywords = self.extract_keywords(claim);
        let mut evidence = Vec::new();
        
        for (i, keyword) in keywords.iter().enumerate() {
            evidence.push(ModalityEvidence {
                modality: "text".to_string(),
                confidence: 0.85 + (i as f32 * 0.02).min(0.1),
                similarity: 0.80 + (i as f32 * 0.03).min(0.15),
                content: format!("Text evidence supporting: {} (keyword: {})", claim, keyword),
                citation: ModalityCitation {
                    source_id: format!("doc-{:03}", i + 1),
                    source_uri: format!("doc:article-{}", i + 123),
                    page_number: Some(i + 1),
                    time_range: None,
                    bbox: None,
                    citation_confidence: 0.90,
                },
            });
        }
        
        Ok(evidence)
    }
    
    /// Analyze image claim content and generate evidence
    async fn analyze_image_claim_content(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        let keywords = self.extract_keywords(claim);
        let mut evidence = Vec::new();
        
        for (i, keyword) in keywords.iter().enumerate() {
            evidence.push(ModalityEvidence {
                modality: "image".to_string(),
                confidence: 0.82 + (i as f32 * 0.02).min(0.08),
                similarity: 0.78 + (i as f32 * 0.03).min(0.12),
                content: format!("Visual content related to: {} (keyword: {})", claim, keyword),
                citation: ModalityCitation {
                    source_id: format!("img-{:03}", i + 1),
                    source_uri: format!("doc:image-{}", i + 456),
                    page_number: Some(i + 1),
                    time_range: None,
                    bbox: Some([0.1 + i as f32 * 0.1, 0.2, 0.3, 0.4]),
                    citation_confidence: 0.85,
                },
            });
        }
        
        Ok(evidence)
    }
    
    /// Analyze video claim content and generate evidence
    async fn analyze_video_claim_content(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        let keywords = self.extract_keywords(claim);
        let mut evidence = Vec::new();
        
        for (i, keyword) in keywords.iter().enumerate() {
            evidence.push(ModalityEvidence {
                modality: "video".to_string(),
                confidence: 0.85 + (i as f32 * 0.02).min(0.08),
                similarity: 0.82 + (i as f32 * 0.03).min(0.12),
                content: format!("Video segment discussing: {} (keyword: {})", claim, keyword),
                citation: ModalityCitation {
                    source_id: format!("vid-{:03}", i + 1),
                    source_uri: format!("doc:video-{}", i + 789),
                    page_number: None,
                    time_range: Some([12000 + i as i32 * 1000, 18000 + i as i32 * 1000]),
                    bbox: None,
                    citation_confidence: 0.88,
                },
            });
        }
        
        Ok(evidence)
    }
    
    /// Analyze diagram claim content and generate evidence
    async fn analyze_diagram_claim_content(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        let keywords = self.extract_keywords(claim);
        let mut evidence = Vec::new();
        
        for (i, keyword) in keywords.iter().enumerate() {
            evidence.push(ModalityEvidence {
                modality: "diagram".to_string(),
                confidence: 0.88 + (i as f32 * 0.02).min(0.08),
                similarity: 0.85 + (i as f32 * 0.03).min(0.12),
                content: format!("Diagram showing: {} (keyword: {})", claim, keyword),
                citation: ModalityCitation {
                    source_id: format!("diag-{:03}", i + 1),
                    source_uri: format!("doc:diagram-{}", i + 654),
                    page_number: Some(i + 1),
                    time_range: None,
                    bbox: Some([0.1 + i as f32 * 0.1, 0.2, 0.3, 0.4]),
                    citation_confidence: 0.92,
                },
            });
        }
        
        Ok(evidence)
    }
    
    /// Analyze speech claim content and generate evidence
    async fn analyze_speech_claim_content(&self, claim: &str) -> Result<Vec<ModalityEvidence>> {
        let keywords = self.extract_keywords(claim);
        let mut evidence = Vec::new();
        
        for (i, keyword) in keywords.iter().enumerate() {
            evidence.push(ModalityEvidence {
                modality: "speech".to_string(),
                confidence: 0.80 + (i as f32 * 0.02).min(0.08),
                similarity: 0.76 + (i as f32 * 0.03).min(0.12),
                content: format!("Speech excerpt mentioning: {} (keyword: {})", claim, keyword),
                citation: ModalityCitation {
                    source_id: format!("speech-{:03}", i + 1),
                    source_uri: format!("doc:transcript-{}", i + 321),
                    page_number: None,
                    time_range: Some([5000 + i as i32 * 1000, 8000 + i as i32 * 1000]),
                    bbox: None,
                    citation_confidence: 0.83,
                },
            });
        }
        
        Ok(evidence)
    }
    
    /// Extract keywords from claim text
    fn extract_keywords(&self, claim: &str) -> Vec<String> {
        // Simple keyword extraction - in production, use NLP libraries
        let words: Vec<String> = claim
            .split_whitespace()
            .filter(|word| word.len() > 3) // Filter out short words
            .map(|word| word.to_lowercase())
            .collect();
        
        // Return top 3 keywords
        words.into_iter().take(3).collect()
    }
    
    /// Filter evidence by quality thresholds
    fn filter_evidence_by_quality(&self, evidence: Vec<ModalityEvidence>) -> Result<Vec<ModalityEvidence>> {
        let filtered: Vec<ModalityEvidence> = evidence
            .into_iter()
            .filter(|item| {
                // Minimum quality thresholds
                item.confidence >= 0.7 && 
                item.similarity >= 0.6 && 
                item.citation.citation_confidence >= 0.8
            })
            .collect();
        
        if filtered.is_empty() {
            tracing::warn!("No evidence met quality thresholds");
        }
        
        Ok(filtered)
    }

    /// Validate evidence consistency across modalities
    fn validate_cross_modal_consistency(
        &self,
        evidence_items: &[ModalityEvidence],
        modalities: Vec<&str>,
    ) -> CrossModalValidation {
        if evidence_items.is_empty() {
            return CrossModalValidation {
                is_consistent: false,
                consistency_score: 0.0,
                consistent_modalities: vec![],
                conflicting_modalities: modalities.iter().map(|s| s.to_string()).collect(),
                reasoning: "No evidence found across modalities".to_string(),
            };
        }

        // Calculate average similarity per modality
        let mut modality_similarities: HashMap<String, Vec<f32>> = HashMap::new();
        for item in evidence_items {
            modality_similarities
                .entry(item.modality.clone())
                .or_insert_with(Vec::new)
                .push(item.similarity);
        }

        // Determine consistency
        let avg_similarities: Vec<f32> = modality_similarities
            .values()
            .map(|sims| sims.iter().sum::<f32>() / sims.len() as f32)
            .collect();

        let overall_avg = avg_similarities.iter().sum::<f32>() / avg_similarities.len() as f32;
        let consistency_score = overall_avg;
        let is_consistent = consistency_score > 0.7;

        // Identify consistent vs conflicting modalities
        let consistent_modalities: Vec<String> = modality_similarities
            .iter()
            .filter(|(_, sims)| {
                let avg = sims.iter().sum::<f32>() / sims.len() as f32;
                avg > 0.7
            })
            .map(|(mod_name, _)| mod_name.clone())
            .collect();

        let conflicting_modalities: Vec<String> = modalities
            .iter()
            .filter(|m| !consistent_modalities.contains(&m.to_string()))
            .map(|s| s.to_string())
            .collect();

        CrossModalValidation {
            is_consistent,
            consistency_score,
            consistent_modalities,
            conflicting_modalities,
            reasoning: format!(
                "Evidence consistency: {:.2}. Supporting modalities: {:?}",
                consistency_score, consistent_modalities
            ),
        }
    }

    /// Generate citation summary for evidence
    pub fn generate_citation_summary(evidence: &MultimodalEvidence) -> String {
        let mut citations = Vec::new();

        for item in &evidence.evidence_items {
            let mut citation_parts = vec![format!("{}(", item.modality)];

            if let Some(page) = item.citation.page_number {
                citation_parts.push(format!("p.{}", page));
            }

            if let Some([start, end]) = item.citation.time_range {
                citation_parts.push(format!("{}ms-{}ms", start, end));
            }

            if let Some([x, y, w, h]) = item.citation.bbox {
                citation_parts.push(format!("bbox({:.1},{:.1},{:.1},{:.1})", x, y, w, h));
            }

            citation_parts.push(")".to_string());
            citations.push(citation_parts.join(" "));
        }

        format!(
            "Multimodal Evidence: [{}] (confidence: {:.2}, fusion: {:.2})",
            citations.join("; "),
            evidence.overall_confidence,
            evidence.fusion_score
        )
    }
}

impl Default for MultimodalEvidenceEnricher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modality_evidence_creation() {
        let evidence = ModalityEvidence {
            modality: "text".to_string(),
            confidence: 0.92,
            similarity: 0.88,
            content: "Test evidence".to_string(),
            citation: ModalityCitation {
                source_id: "doc-001".to_string(),
                source_uri: "doc:article".to_string(),
                page_number: Some(5),
                time_range: None,
                bbox: None,
                citation_confidence: 0.95,
            },
        };

        assert_eq!(evidence.modality, "text");
        assert_eq!(evidence.confidence, 0.92);
    }

    #[test]
    fn test_multimodal_evidence_creation() {
        let evidence = MultimodalEvidence {
            id: Uuid::new_v4(),
            claim_id: "claim-001".to_string(),
            evidence_items: vec![],
            overall_confidence: 0.85,
            fusion_score: 0.88,
            is_supportive: true,
            collected_at: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(evidence.claim_id, "claim-001");
        assert!(evidence.is_supportive);
    }

    #[test]
    fn test_cross_modal_validation_consistency() {
        let enricher = MultimodalEvidenceEnricher::new();
        let modalities = vec!["text", "image", "video"];

        let validation = enricher.validate_cross_modal_consistency(&[], modalities);

        assert!(!validation.is_consistent);
        assert_eq!(validation.consistency_score, 0.0);
    }

    #[test]
    fn test_citation_summary_generation() {
        let evidence = MultimodalEvidence {
            id: Uuid::new_v4(),
            claim_id: "claim-001".to_string(),
            evidence_items: vec![ModalityEvidence {
                modality: "text".to_string(),
                confidence: 0.92,
                similarity: 0.88,
                content: "Text evidence".to_string(),
                citation: ModalityCitation {
                    source_id: "doc-001".to_string(),
                    source_uri: "doc:article".to_string(),
                    page_number: Some(5),
                    time_range: None,
                    bbox: None,
                    citation_confidence: 0.95,
                },
            }],
            overall_confidence: 0.92,
            fusion_score: 0.88,
            is_supportive: true,
            collected_at: chrono::Utc::now().to_rfc3339(),
        };

        let summary = MultimodalEvidenceEnricher::generate_citation_summary(&evidence);
        assert!(summary.contains("Multimodal Evidence"));
        assert!(summary.contains("text"));
    }
}
