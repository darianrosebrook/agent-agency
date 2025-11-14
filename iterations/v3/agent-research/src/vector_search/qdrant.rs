//! Qdrant Database Integration
//!
//! Handles Qdrant client operations, payload conversion, and database interactions.

use crate::research_types::*;
use anyhow::{Context, Result};
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, DeletePoints, Distance, PointStruct, ScrollPoints,
    SearchPoints, UpsertPoints, VectorParams, VectorsConfig, WithPayloadSelector,
};
use qdrant_client::Qdrant;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Qdrant client wrapper for vector search operations
pub struct QdrantClient {
    client: Arc<Qdrant>,
    collection_name: String,
}

impl QdrantClient {
    /// Create a new Qdrant client wrapper
    pub fn new(client: Arc<Qdrant>, collection_name: impl Into<String>) -> Self {
        Self {
            client,
            collection_name: collection_name.into(),
        }
    }

    /// Ensure collection exists with proper configuration
    pub async fn ensure_collection(&self, vector_size: u32) -> Result<()> {
        debug!("Ensuring collection '{}' exists", self.collection_name);

        // Check if collection exists
        let collections = self.client.list_collections().await?;
        let collection_exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection_name);

        if !collection_exists {
            info!("Creating collection '{}'", self.collection_name);

            self.client
                .create_collection(CreateCollection {
                    collection_name: self.collection_name.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(Config::Params(VectorParams {
                            size: vector_size as u64,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                })
                .await
                .context("Failed to create Qdrant collection")?;
        }

        Ok(())
    }

    /// Search for similar vectors
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: u32,
        score_threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_embedding.to_vec(),
                limit: limit as u64,
                score_threshold: Some(score_threshold),
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(
                        qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
                    ),
                }),
                ..Default::default()
            })
            .await
            .context("Failed to search Qdrant")?;

        let results = search_result
            .result
            .into_iter()
            .filter_map(|point| self.qdrant_point_to_search_result(point).ok())
            .collect();

        Ok(results)
    }

    /// Add knowledge entry to the vector database
    pub async fn add_knowledge_entry(
        &self,
        entry: &KnowledgeEntry,
        embedding: &[f32],
    ) -> Result<()> {
        let point = PointStruct {
            id: Some(entry.id.to_string().into()),
            vectors: Some(embedding.to_vec().into()),
            payload: self.knowledge_entry_to_payload(entry),
        };

        self.client
            .upsert_points(UpsertPoints {
                collection_name: self.collection_name.clone(),
                points: vec![point],
                ..Default::default()
            })
            .await
            .context("Failed to add knowledge entry to Qdrant")?;

        debug!("Added knowledge entry {} to Qdrant", entry.id);
        Ok(())
    }

    /// Update existing knowledge entry
    pub async fn update_knowledge_entry(
        &self,
        entry: &KnowledgeEntry,
        embedding: &[f32],
    ) -> Result<()> {
        // For updates, we delete and re-insert
        self.delete_knowledge_entry(&entry.id).await?;
        self.add_knowledge_entry(entry, embedding).await
    }

    /// Delete knowledge entry from vector database
    pub async fn delete_knowledge_entry(&self, entry_id: &Uuid) -> Result<()> {
        use qdrant_client::qdrant::{PointId, PointsIdsList, PointsSelector};

        self.client
            .delete_points(DeletePoints {
                collection_name: self.collection_name.clone(),
                points: Some(PointsSelector {
                    points_selector_one_of: Some(
                        qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(
                            PointsIdsList {
                                ids: vec![PointId {
                                    point_id_options: Some(
                                        qdrant_client::qdrant::point_id::PointIdOptions::Uuid(
                                            entry_id.to_string(),
                                        ),
                                    ),
                                }],
                            },
                        ),
                    ),
                }),
                ..Default::default()
            })
            .await
            .context("Failed to delete knowledge entry from Qdrant")?;

        debug!("Deleted knowledge entry {} from Qdrant", entry_id);
        Ok(())
    }

    /// Fetch all knowledge entries (with pagination)
    pub async fn fetch_all_entries(&self, batch_size: Option<u32>) -> Result<Vec<KnowledgeEntry>> {
        let batch_size = batch_size.unwrap_or(100);
        let mut all_entries = Vec::new();
        let mut offset = None;

        loop {
            let scroll_result = self
                .client
                .scroll(ScrollPoints {
                    collection_name: self.collection_name.clone(),
                    limit: Some(batch_size),
                    offset,
                    with_payload: Some(WithPayloadSelector {
                        selector_options: Some(
                            qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(
                                true,
                            ),
                        ),
                    }),
                    ..Default::default()
                })
                .await
                .context("Failed to scroll Qdrant collection")?;

            let points = scroll_result.result;
            if points.is_empty() {
                break;
            }

            for point in points.clone() {
                if let Ok(entry) = self.qdrant_point_to_knowledge_entry(point) {
                    all_entries.push(entry);
                }
            }

            // Set offset for next batch
            if let Some(last_point) = points.last() {
                offset = Some(last_point.id.clone().unwrap_or_default());
            } else {
                break;
            }
        }

        Ok(all_entries)
    }

    /// Convert KnowledgeEntry to Qdrant payload
    fn knowledge_entry_to_payload(
        &self,
        entry: &KnowledgeEntry,
    ) -> HashMap<String, qdrant_client::qdrant::Value> {
        let mut payload = HashMap::new();

        payload.insert("id".to_string(), json!(entry.id).into());
        payload.insert("content".to_string(), json!(entry.content).into());
        payload.insert("title".to_string(), json!(entry.title).into());
        payload.insert("source".to_string(), json!(entry.source).into());
        payload.insert("content_type".to_string(), json!(entry.content_type).into());
        payload.insert("tags".to_string(), json!(entry.tags).into());
        payload.insert("metadata".to_string(), json!(entry.metadata).into());
        payload.insert("created_at".to_string(), json!(entry.created_at).into());
        payload.insert("updated_at".to_string(), json!(entry.updated_at).into());
        // Quality score and embedding_model not available in KnowledgeEntry, use defaults
        payload.insert("quality_score".to_string(), json!(0.8).into());
        payload.insert("embedding_model".to_string(), json!("unknown").into());

        payload
    }

    /// Convert Qdrant payload to KnowledgeEntry
    fn convert_payload_to_qdrant(
        &self,
        payload: &HashMap<String, qdrant_client::qdrant::Value>,
    ) -> Result<KnowledgeEntry> {
        let id = self
            .extract_string_value(payload.get("id"))
            .and_then(|s| Uuid::parse_str(&s).ok())
            .context("Missing or invalid id")?;

        let content = self
            .extract_string_value(payload.get("content"))
            .context("Missing content")?;

        let title = self
            .extract_string_value(payload.get("title"))
            .unwrap_or_default();

        let source = self
            .extract_string_value(payload.get("source"))
            .unwrap_or_default();

        let content_type_str = self
            .extract_string_value(payload.get("content_type"))
            .unwrap_or_else(|| "text".to_string());

        let content_type = match content_type_str.as_str() {
            "audio" => ContentType::Text, // Audio not available, use Text
            "image" => ContentType::Text, // Image not available, use Text
            "code" => ContentType::Code,
            _ => ContentType::Text,
        };

        let tags = payload
            .get("tags")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::ListValue(list) => Some(
                    list.values
                        .iter()
                        .filter_map(|v| self.extract_string_value(Some(v)))
                        .collect::<Vec<String>>(),
                ),
                _ => None,
            })
            .unwrap_or_default();

        let metadata = payload
            .get("metadata")
            .and_then(|v| serde_json::from_str(&serde_json::to_string(v).unwrap_or_default()).ok())
            .unwrap_or_default();

        // quality_score and embedding_model not available in KnowledgeEntry, skip them

        Ok(KnowledgeEntry {
            id,
            content,
            title,
            source: KnowledgeSource::InternalKnowledgeBase(source),
            content_type,
            tags,
            metadata,
            created_at: chrono::Utc::now(), // Would need to extract from payload
            updated_at: chrono::Utc::now(),
            access_count: 0,
            last_accessed: None,
            language: None,
            embedding: None,
            source_url: None,
        })
    }

    /// Extract string value from Qdrant Value
    fn extract_string_value(&self, value: Option<&qdrant_client::qdrant::Value>) -> Option<String> {
        value
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            })
    }

    /// Convert Qdrant point to SearchResult
    fn qdrant_point_to_search_result(
        &self,
        point: qdrant_client::qdrant::ScoredPoint,
    ) -> Result<SearchResult> {
        // Extract the point from ScoredPoint
        let retrieved_point = qdrant_client::qdrant::RetrievedPoint {
            id: point.id.clone(),
            payload: point.payload.clone(),
            vectors: point.vectors.clone(),
            shard_key: point.shard_key.clone(),
            order_value: point.order_value.clone(),
        };
        let entry = self.qdrant_point_to_knowledge_entry(retrieved_point)?;

        let score = point.score as f64;

        // Convert KnowledgeSource enum to string
        let source_str = match &entry.source {
            KnowledgeSource::WebPage(s) => s.clone(),
            KnowledgeSource::Documentation(s) => s.clone(),
            KnowledgeSource::CodeRepository(s) => s.clone(),
            KnowledgeSource::ApiDocumentation(s) => s.clone(),
            KnowledgeSource::CommunityPost(s) => s.clone(),
            KnowledgeSource::AcademicPaper(s) => s.clone(),
            KnowledgeSource::InternalKnowledgeBase(s) => s.clone(),
        };

        Ok(SearchResult {
            id: entry.id,
            title: entry.title.clone(),
            content: entry.content.clone(),
            url: entry
                .metadata
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            source: source_str,
            relevance_score: score,
            credibility_score: 0.8, // Default credibility score
            metadata: entry.metadata,
        })
    }

    /// Convert Qdrant point to KnowledgeEntry
    fn qdrant_point_to_knowledge_entry(
        &self,
        point: qdrant_client::qdrant::RetrievedPoint,
    ) -> Result<KnowledgeEntry> {
        let payload = point.payload;
        self.convert_payload_to_qdrant(&payload)
    }
}
