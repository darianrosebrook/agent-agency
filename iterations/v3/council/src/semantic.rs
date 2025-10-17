//! Semantic evaluation integration for council judges

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;

/// Semantic context for judge evaluation
#[derive(Debug, Clone)]
pub struct JudgeSemanticContext {
    pub task_description: String,
    pub task_vector: Vec<f32>,
    pub evidence_embeddings: Vec<EvidenceEmbedding>,
    pub knowledge_embeddings: Vec<KnowledgeEmbedding>,
    pub confidence: f32,
}

/// Evidence with semantic embedding
#[derive(Debug, Clone)]
pub struct EvidenceEmbedding {
    pub evidence: String,
    pub embedding: Vec<f32>,
    pub source: String,
    pub relevance_score: f32,
}

/// Knowledge with semantic embedding
#[derive(Debug, Clone)]
pub struct KnowledgeEmbedding {
    pub knowledge: String,
    pub embedding: Vec<f32>,
    pub source: String,
    pub confidence: f32,
}

/// Semantic evaluator for council judges
pub struct SemanticEvaluator {
    embedding_service: Box<dyn embedding_service::EmbeddingService>,
    context_cache: HashMap<String, JudgeSemanticContext>,
}

impl SemanticEvaluator {
    pub fn new(embedding_service: Box<dyn embedding_service::EmbeddingService>) -> Self {
        Self {
            embedding_service,
            context_cache: HashMap::new(),
        }
    }

    /// Generate semantic context for a task
    pub async fn generate_task_context(&mut self, task_spec: &TaskSpec) -> Result<JudgeSemanticContext> {
        let task_description = format!("{}: {}", task_spec.title, task_spec.description);
        let task_id = task_spec.id.to_string();
        
        // Check cache first
        if let Some(cached) = self.context_cache.get(&task_id) {
            return Ok(cached.clone());
        }

        // Generate task embedding
        let task_embedding = self.embedding_service
            .generate_embedding(
                &task_description,
                embedding_service::ContentType::TaskDescription,
                "council_semantic",
            )
            .await?;

        // Generate context
        let context = JudgeSemanticContext {
            task_description: task_description.clone(),
            task_vector: task_embedding.vector,
            evidence_embeddings: Vec::new(),
            knowledge_embeddings: Vec::new(),
            confidence: 0.0,
        };

        // Cache the context
        self.context_cache.insert(task_id, context.clone());

        Ok(context)
    }

    /// Add evidence to semantic context
    pub async fn add_evidence(
        &mut self,
        task_id: &str,
        evidence: &str,
        source: &str,
    ) -> Result<()> {
        let evidence_embedding = self.embedding_service
            .generate_embedding(
                evidence,
                embedding_service::ContentType::Evidence,
                source,
            )
            .await?;

        // Calculate relevance to task
        let relevance_score = self.calculate_relevance(
            &evidence_embedding.vector,
            task_id,
        ).await?;

        let evidence_emb = EvidenceEmbedding {
            evidence: evidence.to_string(),
            embedding: evidence_embedding.vector,
            source: source.to_string(),
            relevance_score,
        };

        // Update context
        if let Some(context) = self.context_cache.get_mut(task_id) {
            context.evidence_embeddings.push(evidence_emb);
            // Calculate confidence after adding evidence
            let evidence_count = context.evidence_embeddings.len();
            let knowledge_count = context.knowledge_embeddings.len();
            
            if evidence_count > 0 || knowledge_count > 0 {
                let evidence_confidence = if evidence_count > 0 {
                    let avg_relevance: f32 = context.evidence_embeddings
                        .iter()
                        .map(|e| e.relevance_score)
                        .sum::<f32>() / evidence_count as f32;
                    avg_relevance
                } else {
                    0.0
                };

                let knowledge_confidence = if knowledge_count > 0 {
                    let avg_confidence: f32 = context.knowledge_embeddings
                        .iter()
                        .map(|k| k.confidence)
                        .sum::<f32>() / knowledge_count as f32;
                    avg_confidence
                } else {
                    0.0
                };

                let total_items = evidence_count + knowledge_count;
                context.confidence = (evidence_confidence * evidence_count as f32 + knowledge_confidence * knowledge_count as f32)
                    / total_items as f32;
            }
        }

        Ok(())
    }

    /// Add knowledge to semantic context
    pub async fn add_knowledge(
        &mut self,
        task_id: &str,
        knowledge: &str,
        source: &str,
    ) -> Result<()> {
        let knowledge_embedding = self.embedding_service
            .generate_embedding(
                knowledge,
                embedding_service::ContentType::Knowledge,
                source,
            )
            .await?;

        let knowledge_emb = KnowledgeEmbedding {
            knowledge: knowledge.to_string(),
            embedding: knowledge_embedding.vector,
            source: source.to_string(),
            confidence: 0.8, // Default confidence for knowledge
        };

        // Update context
        if let Some(context) = self.context_cache.get_mut(task_id) {
            context.knowledge_embeddings.push(knowledge_emb);
            // Calculate confidence after adding knowledge
            let evidence_count = context.evidence_embeddings.len();
            let knowledge_count = context.knowledge_embeddings.len();
            
            if evidence_count > 0 || knowledge_count > 0 {
                let evidence_confidence = if evidence_count > 0 {
                    let avg_relevance: f32 = context.evidence_embeddings
                        .iter()
                        .map(|e| e.relevance_score)
                        .sum::<f32>() / evidence_count as f32;
                    avg_relevance
                } else {
                    0.0
                };

                let knowledge_confidence = if knowledge_count > 0 {
                    let avg_confidence: f32 = context.knowledge_embeddings
                        .iter()
                        .map(|k| k.confidence)
                        .sum::<f32>() / knowledge_count as f32;
                    avg_confidence
                } else {
                    0.0
                };

                let total_items = evidence_count + knowledge_count;
                context.confidence = (evidence_confidence * evidence_count as f32 + knowledge_confidence * knowledge_count as f32)
                    / total_items as f32;
            }
        }

        Ok(())
    }

    /// Calculate relevance score between evidence and task
    async fn calculate_relevance(
        &self,
        evidence_vector: &[f32],
        task_id: &str,
    ) -> Result<f32> {
        if let Some(context) = self.context_cache.get(task_id) {
            let similarity = embedding_service::cosine_similarity(
                evidence_vector,
                &context.task_vector,
            )?;
            Ok(similarity)
        } else {
            Ok(0.0)
        }
    }

    /// Calculate overall context confidence
    fn calculate_context_confidence(&self, context: &JudgeSemanticContext) -> f32 {
        let evidence_count = context.evidence_embeddings.len();
        let knowledge_count = context.knowledge_embeddings.len();
        
        if evidence_count == 0 && knowledge_count == 0 {
            return 0.0;
        }

        // Base confidence on evidence relevance and knowledge quality
        let evidence_confidence = if evidence_count > 0 {
            let avg_relevance: f32 = context.evidence_embeddings
                .iter()
                .map(|e| e.relevance_score)
                .sum::<f32>() / evidence_count as f32;
            avg_relevance
        } else {
            0.0
        };

        let knowledge_confidence = if knowledge_count > 0 {
            let avg_confidence: f32 = context.knowledge_embeddings
                .iter()
                .map(|k| k.confidence)
                .sum::<f32>() / knowledge_count as f32;
            avg_confidence
        } else {
            0.0
        };

        // Weighted average
        let total_items = evidence_count + knowledge_count;
        (evidence_confidence * evidence_count as f32 + knowledge_confidence * knowledge_count as f32)
            / total_items as f32
    }

    /// Get semantic context for a task
    pub fn get_context(&self, task_id: &str) -> Option<&JudgeSemanticContext> {
        self.context_cache.get(task_id)
    }

    /// Clear context cache
    pub fn clear_cache(&mut self) {
        self.context_cache.clear();
    }
}

/// Semantic enhancement for judge verdicts
pub struct SemanticVerdictEnhancer {
    evaluator: SemanticEvaluator,
}

impl SemanticVerdictEnhancer {
    pub fn new(evaluator: SemanticEvaluator) -> Self {
        Self { evaluator }
    }

    /// Enhance a judge verdict with semantic context
    pub async fn enhance_verdict(
        &mut self,
        verdict: &mut JudgeVerdict,
        task_id: &str,
    ) -> Result<()> {
        if let Some(context) = self.evaluator.get_context(task_id) {
            // Add semantic context to verdict reasoning
            let mut additional_reasoning = String::new();
            
            // Add evidence relevance scores
            if !context.evidence_embeddings.is_empty() {
                let evidence_summary = context.evidence_embeddings
                    .iter()
                    .map(|e| format!("{} (relevance: {:.2})", e.source, e.relevance_score))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                additional_reasoning.push_str(&format!(" Evidence: {}", evidence_summary));
            }

            // Add knowledge context
            if !context.knowledge_embeddings.is_empty() {
                let knowledge_sources = context.knowledge_embeddings
                    .iter()
                    .map(|k| k.source.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                additional_reasoning.push_str(&format!(" Knowledge: {}", knowledge_sources));
            }

            // Add semantic confidence
            additional_reasoning.push_str(&format!(" Semantic confidence: {:.2}", context.confidence));

            // Update verdict reasoning based on variant
            match verdict {
                JudgeVerdict::Pass { reasoning, .. } => {
                    *reasoning = format!("{}{}", reasoning, additional_reasoning);
                }
                JudgeVerdict::Fail { reasoning, .. } => {
                    *reasoning = format!("{}{}", reasoning, additional_reasoning);
                }
                JudgeVerdict::Uncertain { reasoning, .. } => {
                    *reasoning = format!("{}{}", reasoning, additional_reasoning);
                }
            }
        }

        Ok(())
    }

    /// Get semantic context for a task
    pub fn get_semantic_context(&self, task_id: &str) -> Option<&JudgeSemanticContext> {
        self.evaluator.get_context(task_id)
    }
}
