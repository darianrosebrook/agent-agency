//! Tests for embedding service

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::*;
    use crate::provider::*;
    use crate::service::*;
    use crate::similarity::*;
    use crate::types::*;

    #[tokio::test]
    async fn test_dummy_embedding_provider() {
        let provider = DummyEmbeddingProvider::new(768);

        // Test health check
        assert!(provider.health_check().await.unwrap());

        // Test embedding generation
        let texts = vec!["Hello world".to_string(), "Test embedding".to_string()];
        let embeddings = provider.generate_embeddings(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 768);
        assert_eq!(embeddings[1].len(), 768);

        // Test deterministic behavior
        let embeddings2 = provider.generate_embeddings(&texts).await.unwrap();
        assert_eq!(embeddings, embeddings2);
    }

    #[tokio::test]
    async fn test_embedding_service() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingServiceFactory::create_dummy_service(config);

        // Test single embedding generation
        let embedding = service
            .generate_embedding("Test text", ContentType::Text, "test_source")
            .await
            .unwrap();

        assert_eq!(embedding.vector.len(), 768);
        assert_eq!(embedding.metadata.content_type, ContentType::Text);
        assert_eq!(embedding.metadata.source, "test_source");

        // Test batch embedding generation
        let request = EmbeddingRequest {
            texts: vec!["Text 1".to_string(), "Text 2".to_string()],
            content_type: ContentType::Code,
            source: "batch_test".to_string(),
            tags: vec!["test".to_string()],
            context: std::collections::HashMap::new(),
        };

        let response = service.generate_embeddings(request).await.unwrap();
        assert_eq!(response.embeddings.len(), 2);
        assert!(response.processing_time_ms >= 0);

        // Store embeddings for similarity search
        for embedding in &response.embeddings {
            service.store_embedding(embedding.clone()).await.unwrap();
        }

        // Test similarity search
        let query_vector = response.embeddings[0].vector.clone();
        let similarity_request = SimilarityRequest {
            query_vector,
            limit: 5,
            threshold: 0.0,
            content_types: vec![ContentType::Code],
            tags: vec!["test".to_string()],
        };

        let results = service.search_similar(similarity_request).await.unwrap();
        assert!(!results.is_empty());

        // Test health check
        assert!(service.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_similarity_functions() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        // Test cosine similarity
        assert_eq!(cosine_similarity(&a, &b).unwrap(), 1.0);
        assert_eq!(cosine_similarity(&a, &c).unwrap(), 0.0);

        // Test euclidean distance
        assert_eq!(euclidean_distance(&a, &b).unwrap(), 0.0);
        assert_eq!(euclidean_distance(&a, &c).unwrap(), 2.0_f32.sqrt());

        // Test vector normalization
        let mut vector = vec![3.0, 4.0, 0.0];
        normalize_vector(&mut vector).unwrap();
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let cache = EmbeddingCache::new(10);

        let embedding = StoredEmbedding {
            id: EmbeddingId::new("test_id".to_string()),
            vector: vec![1.0, 2.0, 3.0],
            metadata: EmbeddingMetadata {
                source: "test".to_string(),
                content_type: ContentType::Text,
                created_at: chrono::Utc::now(),
                tags: vec![],
                context: std::collections::HashMap::new(),
            },
        };

        // Test cache operations
        assert!(!cache.contains("test_key").await);

        cache.put("test_key".to_string(), embedding.clone()).await;
        assert!(cache.contains("test_key").await);

        let retrieved = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved.id, embedding.id);

        // Test cache stats
        let stats = cache.stats().await;
        assert_eq!(stats.size, 1);
        assert_eq!(stats.max_size, 10);
    }

    #[tokio::test]
    async fn test_index_functionality() {
        let index = EmbeddingIndex::new();

        let embedding = StoredEmbedding {
            id: EmbeddingId::new("test_id".to_string()),
            vector: vec![1.0, 2.0, 3.0],
            metadata: EmbeddingMetadata {
                source: "test_source".to_string(),
                content_type: ContentType::Evidence,
                created_at: chrono::Utc::now(),
                tags: vec!["important".to_string(), "test".to_string()],
                context: std::collections::HashMap::new(),
            },
        };

        // Test indexing
        index.insert(embedding.clone());

        // Test retrieval by ID
        let retrieved = index.get_by_id("test_id").unwrap();
        assert_eq!(retrieved.id, embedding.id);

        // Test retrieval by content type
        let evidence_embeddings = index.get_by_content_type(&ContentType::Evidence);
        assert_eq!(evidence_embeddings.len(), 1);

        // Test retrieval by tag
        let tagged_embeddings = index.get_by_tag("important");
        assert_eq!(tagged_embeddings.len(), 1);

        // Test removal
        let removed = index.remove("test_id").unwrap();
        assert_eq!(removed.id, embedding.id);
        assert!(index.get_by_id("test_id").is_none());

        // Test stats
        let stats = index.stats();
        assert_eq!(stats.total_embeddings, 0);
    }
}
