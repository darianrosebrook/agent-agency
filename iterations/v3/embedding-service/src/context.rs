//! Semantic context generation for tasks and content

use crate::similarity::*;
use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;

/// Generate semantic context for a task description
pub async fn generate_semantic_context(
    task_description: &str,
    embedding_service: &dyn crate::service::EmbeddingService,
    context_embeddings: &[StoredEmbedding],
) -> Result<SemanticContext> {
    // Generate embedding for the task description
    let task_embedding = embedding_service
        .generate_embedding(
            task_description,
            ContentType::TaskDescription,
            "semantic_context",
        )
        .await?;

    // Find related embeddings
    let related_embeddings = find_similar_embeddings(
        &task_embedding.vector,
        context_embeddings,
        10,  // limit
        0.3, // threshold
        &[], // all content types
        &[], // all tags
    )?;

    // Calculate confidence based on similarity scores
    let confidence = if related_embeddings.is_empty() {
        0.0
    } else {
        related_embeddings
            .iter()
            .map(|r| r.similarity_score)
            .sum::<f32>()
            / related_embeddings.len() as f32
    };

    Ok(SemanticContext {
        task_description: task_description.to_string(),
        context_vector: task_embedding.vector,
        related_embeddings,
        confidence,
    })
}

/// Build context from multiple sources
pub struct ContextBuilder {
    embeddings: Vec<StoredEmbedding>,
    context_map: HashMap<String, Vec<StoredEmbedding>>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            embeddings: Vec::new(),
            context_map: HashMap::new(),
        }
    }

    /// Add embedding to context
    pub fn add_embedding(&mut self, embedding: StoredEmbedding) {
        self.embeddings.push(embedding.clone());

        // Group by source
        let source = embedding.metadata.source.clone();
        self.context_map
            .entry(source)
            .or_insert_with(Vec::new)
            .push(embedding);
    }

    /// Get embeddings by source
    pub fn get_by_source(&self, source: &str) -> Vec<StoredEmbedding> {
        self.context_map.get(source).cloned().unwrap_or_default()
    }

    /// Get embeddings by content type
    pub fn get_by_content_type(&self, content_type: &ContentType) -> Vec<StoredEmbedding> {
        self.embeddings
            .iter()
            .filter(|e| e.metadata.content_type == *content_type)
            .cloned()
            .collect()
    }

    /// Get embeddings by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<StoredEmbedding> {
        self.embeddings
            .iter()
            .filter(|e| e.metadata.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Build semantic context for a query
    pub async fn build_context(
        &self,
        query: &str,
        embedding_service: &dyn crate::service::EmbeddingService,
        _limit: usize,
        _threshold: f32,
    ) -> Result<SemanticContext> {
        generate_semantic_context(query, embedding_service, &self.embeddings).await
    }

    /// Get all embeddings
    pub fn get_all(&self) -> &[StoredEmbedding] {
        &self.embeddings
    }

    /// Get context statistics
    pub fn stats(&self) -> ContextStats {
        let mut content_type_counts = HashMap::new();
        let mut source_counts = HashMap::new();
        let mut tag_counts = HashMap::new();

        for embedding in &self.embeddings {
            // Count content types
            let content_type = format!("{:?}", embedding.metadata.content_type);
            *content_type_counts.entry(content_type).or_insert(0) += 1;

            // Count sources
            *source_counts
                .entry(embedding.metadata.source.clone())
                .or_insert(0) += 1;

            // Count tags
            for tag in &embedding.metadata.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        ContextStats {
            total_embeddings: self.embeddings.len(),
            content_type_counts,
            source_counts,
            tag_counts,
        }
    }
}

/// Context statistics
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub total_embeddings: usize,
    pub content_type_counts: HashMap<String, usize>,
    pub source_counts: HashMap<String, usize>,
    pub tag_counts: HashMap<String, usize>,
}

/// Context enrichment for council decisions
pub struct CouncilContextEnricher {
    context_builder: ContextBuilder,
    embedding_service: Box<dyn crate::service::EmbeddingService>,
}

impl CouncilContextEnricher {
    pub fn new(embedding_service: Box<dyn crate::service::EmbeddingService>) -> Self {
        Self {
            context_builder: ContextBuilder::new(),
            embedding_service,
        }
    }

    /// Add evidence to context
    pub async fn add_evidence(&mut self, evidence: &str, source: &str) -> Result<()> {
        let embedding = self
            .embedding_service
            .generate_embedding(evidence, ContentType::Evidence, source)
            .await?;

        self.context_builder.add_embedding(embedding);
        Ok(())
    }

    /// Add knowledge to context
    pub async fn add_knowledge(&mut self, knowledge: &str, source: &str) -> Result<()> {
        let embedding = self
            .embedding_service
            .generate_embedding(knowledge, ContentType::Knowledge, source)
            .await?;

        self.context_builder.add_embedding(embedding);
        Ok(())
    }

    /// Enrich task with semantic context
    pub async fn enrich_task(&self, task_description: &str) -> Result<SemanticContext> {
        self.context_builder
            .build_context(task_description, self.embedding_service.as_ref(), 10, 0.3)
            .await
    }

    /// Get relevant evidence for a claim
    pub async fn get_relevant_evidence(&self, claim: &str) -> Result<Vec<SimilarityResult>> {
        let claim_embedding = self
            .embedding_service
            .generate_embedding(claim, ContentType::Evidence, "claim_verification")
            .await?;

        let evidence_embeddings = self
            .context_builder
            .get_by_content_type(&ContentType::Evidence);

        find_similar_embeddings(
            &claim_embedding.vector,
            &evidence_embeddings,
            5,   // limit
            0.5, // threshold
            &[ContentType::Evidence],
            &[],
        )
    }
}
