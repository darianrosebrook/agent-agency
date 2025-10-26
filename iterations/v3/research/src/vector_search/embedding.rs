//! Embedding Generation and Processing
//!
//! Handles vector embeddings, processing, validation, and quality assurance.

use anyhow::Result;
use tracing::{debug, warn};

/// Embedding processor for vector operations
pub struct EmbeddingProcessor;

impl EmbeddingProcessor {
    /// Create a new embedding processor
    pub fn new() -> Self {
        Self
    }

    /// Process embedding after generation (normalization, filtering, etc.)
    pub fn process_embedding(&self, mut embedding: Vec<f32>) -> Result<Vec<f32>> {
        // Normalize embedding
        self.normalize_embedding(&mut embedding);

        // Apply quality filters
        self.filter_embedding_quality(&mut embedding)?;

        // Ensure consistency
        self.ensure_embedding_consistency(&mut embedding)?;

        Ok(embedding)
    }

    /// Validate embedding quality
    pub fn validate_embedding_quality(&self, embedding: &[f32]) -> Result<()> {
        if embedding.is_empty() {
            return Err(anyhow::anyhow!("Embedding is empty"));
        }

        if embedding.len() < 64 {
            return Err(anyhow::anyhow!("Embedding too short: {} < 64", embedding.len()));
        }

        if embedding.iter().any(|&x| !x.is_finite()) {
            return Err(anyhow::anyhow!("Embedding contains non-finite values"));
        }

        let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude < 0.1 {
            return Err(anyhow::anyhow!("Embedding magnitude too small: {}", magnitude));
        }

        Ok(())
    }

    /// Normalize embedding to unit length
    pub fn normalize_embedding(&self, embedding: &mut [f32]) {
        let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for value in embedding.iter_mut() {
                *value /= magnitude;
            }
        }
    }

    /// Apply quality filters to embedding
    pub fn filter_embedding_quality(&self, embedding: &mut [f32]) -> Result<()> {
        // Remove extreme outliers
        let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
        let variance = embedding.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / embedding.len() as f32;
        let std_dev = variance.sqrt();

        for value in embedding.iter_mut() {
            if (*value - mean).abs() > 3.0 * std_dev {
                *value = mean + 3.0 * std_dev * (*value - mean).signum();
            }
        }

        Ok(())
    }

    /// Ensure embedding consistency
    pub fn ensure_embedding_consistency(&self, embedding: &mut [f32]) -> Result<()> {
        // Ensure all values are finite
        for value in embedding.iter_mut() {
            if !value.is_finite() {
                *value = 0.0;
            }
        }

        // Re-normalize after any modifications
        self.normalize_embedding(embedding);

        Ok(())
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            0.0
        } else {
            dot_product / (magnitude_a * magnitude_b)
        }
    }

    /// Generate mock embedding for testing
    pub fn generate_mock_embedding(&self, size: usize) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect()
    }

    /// Calculate embedding statistics
    pub fn calculate_embedding_stats(&self, embedding: &[f32]) -> (f32, f32, f32, f32) {
        if embedding.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
        let variance = embedding.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / embedding.len() as f32;
        let std_dev = variance.sqrt();
        let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        (mean, std_dev, magnitude, variance)
    }
}

impl Default for EmbeddingProcessor {
    fn default() -> Self {
        Self::new()
    }
}
