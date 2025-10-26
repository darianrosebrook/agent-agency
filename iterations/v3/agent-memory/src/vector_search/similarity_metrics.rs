//! Similarity Metrics
//!
//! Various similarity and distance metrics for vector comparison.

/// Similarity metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityMetric {
    Cosine,
    Euclidean,
    DotProduct,
    Manhattan,
    Jaccard,
    Hamming,
}

/// Similarity calculator
pub struct SimilarityCalculator;

impl SimilarityCalculator {
    /// Calculate similarity between two vectors
    pub fn calculate(metric: SimilarityMetric, a: &[f32], b: &[f32]) -> f32 {
        match metric {
            SimilarityMetric::Cosine => Self::cosine_similarity(a, b),
            SimilarityMetric::Euclidean => Self::euclidean_similarity(a, b),
            SimilarityMetric::DotProduct => Self::dot_product_similarity(a, b),
            SimilarityMetric::Manhattan => Self::manhattan_similarity(a, b),
            SimilarityMetric::Jaccard => Self::jaccard_similarity(a, b),
            SimilarityMetric::Hamming => Self::hamming_similarity(a, b),
        }
    }

    /// Cosine similarity: cos(θ) = (A·B) / (|A|·|B|)
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Euclidean similarity (inverse of distance)
    pub fn euclidean_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let distance = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();

        // Convert distance to similarity (higher values = more similar)
        if distance == 0.0 {
            1.0
        } else {
            1.0 / (1.0 + distance)
        }
    }

    /// Dot product similarity
    pub fn dot_product_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Manhattan similarity (inverse of distance)
    pub fn manhattan_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let distance: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();

        // Convert distance to similarity
        if distance == 0.0 {
            1.0
        } else {
            1.0 / (1.0 + distance)
        }
    }

    /// Jaccard similarity for binary vectors
    pub fn jaccard_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let mut intersection = 0;
        let mut union = 0;

        for (x, y) in a.iter().zip(b.iter()) {
            let a_bit = *x > 0.0;
            let b_bit = *y > 0.0;

            if a_bit && b_bit {
                intersection += 1;
            }
            if a_bit || b_bit {
                union += 1;
            }
        }

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Hamming similarity (inverse of distance)
    pub fn hamming_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let distance = a.iter()
            .zip(b.iter())
            .map(|(x, y)| if (*x - *y).abs() > f32::EPSILON { 1 } else { 0 })
            .sum::<i32>();

        // Convert distance to similarity
        let max_distance = a.len() as f32;
        1.0 - (distance as f32 / max_distance)
    }

    /// Calculate similarity matrix for multiple vectors
    pub fn similarity_matrix(metric: SimilarityMetric, vectors: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let n = vectors.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in i..n {
                let similarity = Self::calculate(metric, &vectors[i], &vectors[j]);
                matrix[i][j] = similarity;
                matrix[j][i] = similarity; // Symmetric
            }
        }

        matrix
    }

    /// Find k nearest neighbors
    pub fn knn(metric: SimilarityMetric, query: &[f32], vectors: &[Vec<f32>], k: usize) -> Vec<(usize, f32)> {
        let mut similarities: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(i, v)| (i, Self::calculate(metric, query, v)))
            .collect();

        // Sort by similarity (descending for similarity, ascending for distance)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        similarities.into_iter().take(k).collect()
    }

    /// Calculate percentile of similarities
    pub fn similarity_percentile(similarities: &[f32], percentile: f32) -> f32 {
        if similarities.is_empty() {
            return 0.0;
        }

        let mut sorted = similarities.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((percentile / 100.0) * (sorted.len() - 1) as f32) as usize;
        sorted[index]
    }

    /// Normalize similarities to [0, 1] range
    pub fn normalize_similarities(similarities: &mut [f32]) {
        if similarities.is_empty() {
            return;
        }

        let min_val = similarities.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = similarities.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let range = max_val - min_val;
        if range > 0.0 {
            for similarity in similarities.iter_mut() {
                *similarity = (*similarity - min_val) / range;
            }
        }
    }

    /// Calculate diversity score for a set of vectors
    pub fn diversity_score(metric: SimilarityMetric, vectors: &[Vec<f32>]) -> f32 {
        if vectors.len() < 2 {
            return 0.0;
        }

        let matrix = Self::similarity_matrix(metric, vectors);
        let mut total_similarity = 0.0;
        let mut count = 0;

        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                total_similarity += matrix[i][j];
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            // Lower average similarity = higher diversity
            1.0 - (total_similarity / count as f32)
        }
    }
}

/// Distance metrics (lower values = more similar)
pub struct DistanceCalculator;

impl DistanceCalculator {
    /// Calculate distance between two vectors
    pub fn calculate(metric: SimilarityMetric, a: &[f32], b: &[f32]) -> f32 {
        match metric {
            SimilarityMetric::Cosine => 1.0 - SimilarityCalculator::cosine_similarity(a, b),
            SimilarityMetric::Euclidean => {
                if a.len() != b.len() {
                    return f32::INFINITY;
                }
                a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f32>().sqrt()
            }
            SimilarityMetric::DotProduct => {
                // Convert similarity to distance (negative dot product becomes distance)
                let dot = SimilarityCalculator::dot_product_similarity(a, b);
                if dot >= 0.0 { 1.0 / (1.0 + dot) } else { 1.0 - dot }
            }
            SimilarityMetric::Manhattan => {
                if a.len() != b.len() {
                    return f32::INFINITY;
                }
                a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
            }
            SimilarityMetric::Jaccard => 1.0 - SimilarityCalculator::jaccard_similarity(a, b),
            SimilarityMetric::Hamming => {
                if a.len() != b.len() {
                    return f32::INFINITY;
                }
                a.iter().zip(b.iter())
                    .map(|(x, y)| if (*x - *y).abs() > f32::EPSILON { 1.0 } else { 0.0 })
                    .sum()
            }
        }
    }
}

/// Vector normalization utilities
pub struct VectorNormalizer;

impl VectorNormalizer {
    /// L2 normalization (unit vector)
    pub fn l2_normalize(vector: &mut [f32]) {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in vector.iter_mut() {
                *x /= norm;
            }
        }
    }

    /// L1 normalization
    pub fn l1_normalize(vector: &mut [f32]) {
        let sum: f32 = vector.iter().map(|x| x.abs()).sum();
        if sum > 0.0 {
            for x in vector.iter_mut() {
                *x /= sum;
            }
        }
    }

    /// Min-max normalization to [0, 1]
    pub fn min_max_normalize(vector: &mut [f32]) {
        if vector.is_empty() {
            return;
        }

        let min_val = vector.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = vector.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let range = max_val - min_val;
        if range > 0.0 {
            for x in vector.iter_mut() {
                *x = (*x - min_val) / range;
            }
        }
    }

    /// Z-score normalization
    pub fn z_score_normalize(vector: &mut [f32]) {
        if vector.len() < 2 {
            return;
        }

        let mean: f32 = vector.iter().sum::<f32>() / vector.len() as f32;
        let variance: f32 = vector.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / vector.len() as f32;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            for x in vector.iter_mut() {
                *x = (*x - mean) / std_dev;
            }
        }
    }
}
