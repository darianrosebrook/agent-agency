// @darianrosebrook
// Multimodal context provider for Council integration
// Supplies multimodal evidence and context to council decision-making

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Result from multimodal context query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalContext {
    /// Query identifier for tracking
    pub query_id: String,
    
    /// Retrieved evidence items with citations
    pub evidence_items: Vec<EvidenceItem>,
    
    /// Deduplication metadata
    pub dedup_score: f32,
    
    /// Context budget usage (0-1, where 1 = full budget used)
    pub budget_utilization: f32,
    
    /// Global vs project-specific evidence ratio
    pub global_to_project_ratio: f32,
    
    /// Processing metadata
    pub processing_time_ms: u64,
    
    /// Retrieval metadata for audit trail
    pub retrieval_metadata: RetrievalMetadata,
}

/// Individual evidence item retrieved from multimodal indices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Evidence identifier
    pub id: Uuid,
    
    /// Evidence text or description
    pub content: String,
    
    /// Modality source (text, image, video, diagram, speech)
    pub modality: String,
    
    /// Confidence score (0-1)
    pub confidence: f32,
    
    /// Semantic similarity score
    pub similarity_score: f32,
    
    /// Citation information
    pub citation: Citation,
    
    /// Whether this is global or project-specific
    pub is_global: bool,
    
    /// Deduplication hash for redundancy detection
    pub dedup_hash: String,
}

/// Citation information for retrieved evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Document URI or file path
    pub source_uri: String,
    
    /// Document identifier
    pub document_id: Uuid,
    
    /// Page/slide number (if applicable)
    pub page_number: Option<u32>,
    
    /// Bounding box for visual content [x, y, width, height]
    pub bbox: Option<[f32; 4]>,
    
    /// Time range for video/audio [start_ms, end_ms]
    pub time_range: Option<[u64; 2]>,
}

/// Metadata for retrieval operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalMetadata {
    /// Number of indices queried
    pub indices_queried: usize,
    
    /// Fusion method used (late_fusion, early_fusion, etc.)
    pub fusion_method: String,
    
    /// Reranking applied (true/false)
    pub reranking_applied: bool,
    
    /// Query expansion terms used
    pub query_expansions: Vec<String>,
}

/// Context budget for controlling evidence volume
#[derive(Debug, Clone)]
pub struct ContextBudget {
    /// Maximum tokens to use for evidence
    pub max_tokens: usize,
    
    /// Maximum items to retrieve
    pub max_items: usize,
    
    /// Minimum confidence threshold
    pub min_confidence: f32,
    
    /// Prefer global (non-project) evidence
    pub prefer_global: bool,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_tokens: 8000,      // 8K tokens
            max_items: 50,
            min_confidence: 0.5,
            prefer_global: false,
        }
    }
}

/// Multimodal context provider for Council
pub struct MultimodalContextProvider {
    retriever: crate::multimodal_retriever::MultimodalRetriever,
    dedup_cache: HashMap<String, Uuid>,
}

impl MultimodalContextProvider {
    /// Create new multimodal context provider
    pub fn new(retriever: crate::multimodal_retriever::MultimodalRetriever) -> Self {
        Self {
            retriever,
            dedup_cache: HashMap::new(),
        }
    }

    /// Provide multimodal context for a query
    ///
    /// Retrieves multimodal evidence, deduplicates, respects context budgets,
    /// and provides proper citations for council decision-making.
    ///
    /// # Arguments
    /// * `query` - Query text
    /// * `budget` - Context budget constraints
    /// * `project_scope` - Optional project scope for filtering
    ///
    /// # Returns
    /// `MultimodalContext` with evidence and metadata
    pub async fn provide_context(
        &mut self,
        query: &str,
        budget: Option<ContextBudget>,
        project_scope: Option<&str>,
    ) -> Result<MultimodalContext> {
        let query_id = Uuid::new_v4().to_string();
        let budget = budget.unwrap_or_default();
        let start_time = std::time::Instant::now();

        // Retrieve multimodal evidence
        let retrieval_result = self.retriever
            .search_multimodal(query, 10, project_scope)
            .await
            .map_err(|e| anyhow!("Multimodal search failed: {}", e))?;

        // Convert to evidence items with deduplication
        let mut evidence_items = Vec::new();
        let mut seen_hashes = HashSet::new();
        let mut token_count = 0;
        let mut global_count = 0;
        let mut project_count = 0;

        for result in retrieval_result.results {
            // Check budget constraints
            if evidence_items.len() >= budget.max_items {
                break;
            }

            if result.confidence < budget.min_confidence {
                continue;
            }

            // Estimate token count (rough approximation)
            let token_estimate = result.text.len() / 4;
            if token_count + token_estimate > budget.max_tokens {
                break;
            }

            // Deduplication using content hash
            let dedup_hash = Self::hash_content(&result.text);
            if seen_hashes.contains(&dedup_hash) {
                continue; // Skip duplicate
            }
            seen_hashes.insert(dedup_hash.clone());

            let is_global = project_scope.is_none() || !result.text.contains("project:");
            if is_global {
                global_count += 1;
            } else {
                project_count += 1;
            }

            let evidence = EvidenceItem {
                id: Uuid::new_v4(),
                content: result.text.clone(),
                modality: result.modality.clone(),
                confidence: result.confidence,
                similarity_score: result.score,
                citation: Citation {
                    source_uri: format!("doc:{}", result.document_id),
                    document_id: result.document_id,
                    page_number: result.page_number,
                    bbox: result.bbox.map(|b| [b.0, b.1, b.2, b.3]),
                    time_range: result.time_range.map(|t| [t.0, t.1]),
                },
                is_global,
                dedup_hash,
            };

            evidence_items.push(evidence);
            token_count += token_estimate;
        }

        // Calculate budget utilization
        let budget_utilization = (token_count as f32 / budget.max_tokens as f32).min(1.0);

        // Calculate global to project ratio
        let total = global_count + project_count;
        let global_to_project_ratio = if total > 0 {
            global_count as f32 / total as f32
        } else {
            0.5 // Default ratio if no items
        };

        // Calculate deduplication score (items retained / items processed)
        let dedup_score = if !retrieval_result.results.is_empty() {
            evidence_items.len() as f32 / retrieval_result.results.len() as f32
        } else {
            1.0
        };

        Ok(MultimodalContext {
            query_id,
            evidence_items,
            dedup_score,
            budget_utilization,
            global_to_project_ratio,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            retrieval_metadata: RetrievalMetadata {
                indices_queried: 3, // Text (BM25), Dense, Image
                fusion_method: "late_fusion_rrf".to_string(),
                reranking_applied: true,
                query_expansions: vec![], // Would be populated by retriever
            },
        })
    }

    /// Get context for council evidence enrichment
    ///
    /// Provides evidence context for enriching claims with multimodal citations.
    pub async fn get_evidence_context(
        &mut self,
        claim: &str,
        context_type: &str, // "citation", "support", "refutation"
    ) -> Result<MultimodalContext> {
        let query = match context_type {
            "citation" => format!("Evidence for: {}", claim),
            "support" => format!("Support for: {}", claim),
            "refutation" => format!("Counter-evidence to: {}", claim),
            _ => claim.to_string(),
        };

        let mut budget = ContextBudget::default();
        budget.max_items = 20; // Smaller set for claim enrichment
        budget.prefer_global = true;

        self.provide_context(&query, Some(budget), None).await
    }

    /// Get context for decision-making
    ///
    /// Provides focused context for council decision points.
    pub async fn get_decision_context(
        &mut self,
        decision_point: &str,
        project_scope: Option<&str>,
    ) -> Result<MultimodalContext> {
        let query = format!("Context for decision: {}", decision_point);
        let budget = ContextBudget {
            max_tokens: 12000, // Larger budget for decisions
            max_items: 100,
            min_confidence: 0.4, // Lower threshold for decisions
            prefer_global: false,
        };

        self.provide_context(&query, Some(budget), project_scope).await
    }

    /// Hash content for deduplication
    fn hash_content(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get deduplication statistics
    pub fn get_dedup_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("cache_size".to_string(), self.dedup_cache.len());
        stats
    }

    /// Clear deduplication cache (for memory management)
    pub fn clear_dedup_cache(&mut self) {
        self.dedup_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_budget_defaults() {
        let budget = ContextBudget::default();
        assert_eq!(budget.max_tokens, 8000);
        assert_eq!(budget.max_items, 50);
        assert_eq!(budget.min_confidence, 0.5);
        assert!(!budget.prefer_global);
    }

    #[test]
    fn test_content_hashing() {
        let content1 = "This is sample content";
        let content2 = "This is sample content";
        let content3 = "Different content";

        let hash1 = MultimodalContextProvider::hash_content(content1);
        let hash2 = MultimodalContextProvider::hash_content(content2);
        let hash3 = MultimodalContextProvider::hash_content(content3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_evidence_item_creation() {
        let evidence = EvidenceItem {
            id: Uuid::new_v4(),
            content: "Test evidence".to_string(),
            modality: "text".to_string(),
            confidence: 0.85,
            similarity_score: 0.92,
            citation: Citation {
                source_uri: "doc:123".to_string(),
                document_id: Uuid::new_v4(),
                page_number: Some(5),
                bbox: Some([0.1, 0.2, 0.3, 0.4]),
                time_range: Some([1000, 5000]),
            },
            is_global: true,
            dedup_hash: "abc123".to_string(),
        };

        assert_eq!(evidence.modality, "text");
        assert_eq!(evidence.confidence, 0.85);
        assert!(evidence.is_global);
    }

    #[test]
    fn test_citation_information() {
        let citation = Citation {
            source_uri: "doc:test-file".to_string(),
            document_id: Uuid::new_v4(),
            page_number: Some(10),
            bbox: Some([0.0, 0.0, 1.0, 1.0]),
            time_range: Some([0, 60000]),
        };

        assert_eq!(citation.page_number, Some(10));
        assert!(citation.bbox.is_some());
        assert!(citation.time_range.is_some());
    }

    #[test]
    fn test_retrieval_metadata() {
        let metadata = RetrievalMetadata {
            indices_queried: 3,
            fusion_method: "late_fusion_rrf".to_string(),
            reranking_applied: true,
            query_expansions: vec!["expansion1".to_string()],
        };

        assert_eq!(metadata.indices_queried, 3);
        assert_eq!(metadata.query_expansions.len(), 1);
        assert!(metadata.reranking_applied);
    }
}
