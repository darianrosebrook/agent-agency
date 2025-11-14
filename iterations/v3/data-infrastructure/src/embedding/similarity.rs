//! Similarity search and vector operations

use super::embedding_types::*;
use anyhow::Result;

/// Calculate cosine similarity between two embedding vectors
pub fn cosine_similarity(a: &EmbeddingVector, b: &EmbeddingVector) -> Result<f32> {
    if a.dimensions != b.dimensions {
        return Err(anyhow::anyhow!("Vector dimensions must match"));
    }

    let dot_product: f32 = a
        .values
        .iter()
        .zip(b.values.iter())
        .map(|(x, y)| x * y)
        .sum();
    let norm_a: f32 = a.values.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.values.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return Ok(0.0);
    }

    Ok(dot_product / (norm_a * norm_b))
}

/// Calculate Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    if a.len() != b.len() {
        return Err(anyhow::anyhow!("Vector dimensions must match"));
    }

    let sum_squared_diff: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();

    Ok(sum_squared_diff.sqrt())
}

/// Normalize a vector to unit length
pub fn normalize_vector(vector: &mut [f32]) -> Result<()> {
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm == 0.0 {
        return Err(anyhow::anyhow!("Cannot normalize zero vector"));
    }

    for element in vector.iter_mut() {
        *element /= norm;
    }

    Ok(())
}

/// Find most similar embeddings to a query vector
pub fn find_similar_embeddings(
    query_vector: &EmbeddingVector,
    embeddings: &[StoredEmbedding],
    limit: usize,
    threshold: f32,
    content_types: &[ContentType],
    tags: &[String],
) -> Result<Vec<SimilarityResult>> {
    let mut results = Vec::new();

    for embedding in embeddings {
        // Filter by content type if specified
        if !content_types.is_empty() && !content_types.contains(&embedding.metadata.content_type) {
            continue;
        }

        // Filter by tags if specified
        if !tags.is_empty() {
            let has_matching_tag = tags.iter().any(|tag| embedding.metadata.tags.contains(tag));
            if !has_matching_tag {
                continue;
            }
        }

        // Calculate similarity
        let similarity = cosine_similarity(query_vector, &embedding.vector)?;

        if similarity >= threshold {
            results.push(SimilarityResult {
                embedding: embedding.clone(),
                similarity_score: similarity,
            });
        }
    }

    // Sort by similarity score (descending)
    results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

    // Limit results
    results.truncate(limit);

    Ok(results)
}

/// Calculate average vector from a collection of embeddings
pub fn average_embedding(embeddings: &[EmbeddingVector]) -> Result<EmbeddingVector> {
    if embeddings.is_empty() {
        return Err(anyhow::anyhow!("Cannot average empty embedding collection"));
    }

    let dimension = embeddings[0].dimensions;

    // Verify all embeddings have the same dimension
    for embedding in embeddings {
        if embedding.dimensions != dimension {
            return Err(anyhow::anyhow!(
                "All embeddings must have the same dimension"
            ));
        }
    }

    let mut average_values = vec![0.0; dimension];

    for embedding in embeddings {
        for (i, value) in embedding.values.iter().enumerate() {
            average_values[i] += value;
        }
    }

    let count = embeddings.len() as f32;
    for value in average_values.iter_mut() {
        *value /= count;
    }

    Ok(EmbeddingVector {
        values: average_values,
        model: "averaged".to_string(),
        dimensions: dimension,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = EmbeddingVector::from_values(vec![1.0, 0.0, 0.0]);
        let b = EmbeddingVector::from_values(vec![1.0, 0.0, 0.0]);
        assert_eq!(cosine_similarity(&a, &b).unwrap(), 1.0);

        let a = EmbeddingVector::from_values(vec![1.0, 0.0, 0.0]);
        let b = EmbeddingVector::from_values(vec![0.0, 1.0, 0.0]);
        assert_eq!(cosine_similarity(&a, &b).unwrap(), 0.0);

        let a = EmbeddingVector::from_values(vec![1.0, 1.0, 0.0]);
        let b = EmbeddingVector::from_values(vec![1.0, 0.0, 0.0]);
        let similarity = cosine_similarity(&a, &b).unwrap();
        assert!((similarity - 0.707).abs() < 0.01);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        assert_eq!(euclidean_distance(&a, &b).unwrap(), 5.0);
    }

    #[test]
    fn test_normalize_vector() {
        let mut vector = vec![3.0, 4.0, 0.0];
        normalize_vector(&mut vector).unwrap();

        let expected_norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((expected_norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_average_embedding() {
        let embeddings = vec![
            EmbeddingVector::new(vec![1.0, 2.0, 3.0], "test".to_string()),
            EmbeddingVector::new(vec![3.0, 4.0, 5.0], "test".to_string()),
        ];

        let average = average_embedding(&embeddings).unwrap();
        assert_eq!(average.values, vec![2.0, 3.0, 4.0]);
    }
}
