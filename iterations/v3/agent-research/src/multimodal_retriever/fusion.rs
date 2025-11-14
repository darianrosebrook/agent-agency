//! Result fusion engine for combining multimodal search results

use anyhow::Result;

use super::core::MultimodalSearchResult;

use schemars::JsonSchema;
/// Fusion engine for combining results from different modalities
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct FusionEngine {
    config: super::core::MultimodalRetrieverConfig,
}

impl FusionEngine {
    /// Create a new fusion engine
    pub fn new(config: super::core::MultimodalRetrieverConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Fuse results from multiple modalities
    pub fn fuse_results(
        &self,
        modality_results: Vec<Vec<MultimodalSearchResult>>,
        max_results: usize,
    ) -> Result<Vec<MultimodalSearchResult>> {
        let mut fused_results = Vec::new();

        // Collect all results
        for results in modality_results {
            fused_results.extend(results);
        }

        // Apply fusion strategy
        match self.config.fusion_method {
            crate::research_types::FusionMethod::RRF => self.apply_rrf_fusion(&mut fused_results),
            _ => self.apply_weighted_fusion(&mut fused_results),
        }

        // Sort by combined score and limit results
        fused_results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        fused_results.truncate(max_results);

        Ok(fused_results)
    }

    /// Apply Reciprocal Rank Fusion
    fn apply_rrf_fusion(&self, results: &mut [MultimodalSearchResult]) {
        // Collect RRF scores first (immutable borrow)
        let mut rrf_scores = std::collections::HashMap::new();

        for (rank, result) in results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32); // k=60 is standard
            *rrf_scores.entry(result.id.clone()).or_insert(0.0) += rrf_score;
        }

        // Then mutate (mutable borrow)
        for result in results.iter_mut() {
            if let Some(rrf_score) = rrf_scores.get(&result.id) {
                result.combined_score = *rrf_score;
            }
        }
    }

    /// Apply weighted fusion
    fn apply_weighted_fusion(&self, results: &mut [MultimodalSearchResult]) {
        for result in results.iter_mut() {
            let mut combined_score = 0.0;

            for (modality, score) in &result.modality_scores {
                let weight = match modality.as_str() {
                    "text" => self.config.text_weight,
                    "visual" => self.config.visual_weight,
                    "code" => self.config.code_weight,
                    _ => 0.1,
                };
                combined_score += score * weight;
            }

            result.combined_score = combined_score;
        }
    }
}
