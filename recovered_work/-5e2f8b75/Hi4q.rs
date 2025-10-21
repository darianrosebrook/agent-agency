//! Knowledge base query methods for external knowledge entities
//!
//! @author @darianrosebrook

use crate::client::DatabaseClient;
use crate::models::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

impl DatabaseClient {
    /// Search external knowledge by semantic similarity
    pub async fn kb_semantic_search(
        &self,
        query_vec: &[f32],
        model_id: &str,
        source: Option<&str>,
        k: usize,
        min_conf: f32,
    ) -> Result<Vec<ExternalKnowledgeEntity>> {
        let query = "SELECT * FROM kb_semantic_search($1::vector, $2, $3, $4, $5)";
        
        let rows = sqlx::query(query)
            .bind(query_vec)
            .bind(model_id)
            .bind(source)
            .bind(k as i32)
            .bind(min_conf as f64)
            .fetch_all(&self.pool())
            .await
            .context("Failed to execute semantic search")?;
        
        let mut entities = Vec::new();
        for row in rows {
            entities.push(ExternalKnowledgeEntity {
                id: Some(row.try_get("entity_id")?),
                source: match row.try_get::<String, _>("source")?.as_str() {
                    "wikidata" => KnowledgeSource::Wikidata,
                    "wordnet" => KnowledgeSource::WordNet,
                    _ => continue,
                },
                entity_key: row.try_get("entity_key")?,
                canonical_name: row.try_get("canonical_name")?,
                lang: None,
                entity_type: row.try_get("entity_type")?,
                properties: row.try_get("properties")?,
                confidence: row.try_get::<f64, _>("confidence")? as f64,
                usage_count: row.try_get("usage_count")?,
                usage_decay: None,
                last_accessed: None,
                created_at: None,
                dump_version: None,
                toolchain: None,
                license: None,
            });
        }
        
        Ok(entities)
    }
    
    /// Get entity by source and key
    pub async fn kb_get_entity(
        &self,
        source: &str,
        key: &str,
    ) -> Result<Option<ExternalKnowledgeEntity>> {
        let query = "
            SELECT id, source, entity_key, canonical_name, lang, entity_type,
                   properties, confidence, usage_count, usage_decay, last_accessed,
                   created_at, dump_version, toolchain, license
            FROM external_knowledge_entities
            WHERE source = $1 AND entity_key = $2
        ";
        
        let row = sqlx::query(query)
            .bind(source)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get entity")?;
        
        match row {
            Some(row) => Ok(Some(ExternalKnowledgeEntity {
                id: Some(row.try_get("id")?),
                source: match row.try_get::<String, _>("source")?.as_str() {
                    "wikidata" => KnowledgeSource::Wikidata,
                    "wordnet" => KnowledgeSource::WordNet,
                    _ => return Ok(None),
                },
                entity_key: row.try_get("entity_key")?,
                canonical_name: row.try_get("canonical_name")?,
                lang: row.try_get("lang")?,
                entity_type: row.try_get("entity_type")?,
                properties: row.try_get("properties")?,
                confidence: row.try_get("confidence")?,
                usage_count: row.try_get("usage_count")?,
                usage_decay: row.try_get("usage_decay")?,
                last_accessed: row.try_get("last_accessed")?,
                created_at: row.try_get("created_at")?,
                dump_version: row.try_get("dump_version")?,
                toolchain: row.try_get("toolchain")?,
                license: row.try_get("license")?,
            })),
            None => Ok(None),
        }
    }
    
    /// Get related entities via relationships
    pub async fn kb_get_related(
        &self,
        entity_id: Uuid,
        types: Option<Vec<String>>,
        max_depth: usize,
    ) -> Result<Vec<ExternalKnowledgeEntity>> {
        let query = "SELECT * FROM kb_get_related($1, $2, $3)";
        
        let rows = sqlx::query(query)
            .bind(entity_id)
            .bind(types.as_ref().map(|v| v.as_slice()))
            .bind(max_depth as i32)
            .fetch_all(&self.pool())
            .await
            .context("Failed to get related entities")?;
        
        let mut entities = Vec::new();
        for row in rows {
            let entity_id: Uuid = row.try_get("entity_id")?;
            
            // Fetch full entity details
            if let Some(entity) = self.kb_get_entity_by_id(entity_id).await? {
                entities.push(entity);
            }
        }
        
        Ok(entities)
    }
    
    /// Get entity by ID
    async fn kb_get_entity_by_id(&self, id: Uuid) -> Result<Option<ExternalKnowledgeEntity>> {
        let query = "
            SELECT id, source, entity_key, canonical_name, lang, entity_type,
                   properties, confidence, usage_count, usage_decay, last_accessed,
                   created_at, dump_version, toolchain, license
            FROM external_knowledge_entities
            WHERE id = $1
        ";
        
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(row) => Ok(Some(ExternalKnowledgeEntity {
                id: Some(row.try_get("id")?),
                source: match row.try_get::<String, _>("source")?.as_str() {
                    "wikidata" => KnowledgeSource::Wikidata,
                    "wordnet" => KnowledgeSource::WordNet,
                    _ => return Ok(None),
                },
                entity_key: row.try_get("entity_key")?,
                canonical_name: row.try_get("canonical_name")?,
                lang: row.try_get("lang")?,
                entity_type: row.try_get("entity_type")?,
                properties: row.try_get("properties")?,
                confidence: row.try_get("confidence")?,
                usage_count: row.try_get("usage_count")?,
                usage_decay: row.try_get("usage_decay")?,
                last_accessed: row.try_get("last_accessed")?,
                created_at: row.try_get("created_at")?,
                dump_version: row.try_get("dump_version")?,
                toolchain: row.try_get("toolchain")?,
                license: row.try_get("license")?,
            })),
            None => Ok(None),
        }
    }
    
    /// Record usage for relevance tracking
    pub async fn kb_record_usage(&self, entity_id: Uuid) -> Result<()> {
        let query = "SELECT record_knowledge_usage($1)";
        
        sqlx::query(query)
            .bind(entity_id)
            .execute(&self.pool)
            .await
            .context("Failed to record usage")?;
        
        Ok(())
    }
    
    /// Upsert entity with vectors
    pub async fn kb_upsert_entity(
        &self,
        entity: ExternalKnowledgeEntity,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<Uuid> {
        let mut tx = self.pool.begin().await?;
        
        // Insert or update entity
        let query = "
            INSERT INTO external_knowledge_entities 
            (source, entity_key, canonical_name, lang, entity_type, properties, 
             confidence, dump_version, toolchain, license)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (source, entity_key) 
            DO UPDATE SET
                canonical_name = EXCLUDED.canonical_name,
                lang = EXCLUDED.lang,
                entity_type = EXCLUDED.entity_type,
                properties = EXCLUDED.properties,
                confidence = EXCLUDED.confidence,
                dump_version = EXCLUDED.dump_version,
                toolchain = EXCLUDED.toolchain,
                license = EXCLUDED.license
            RETURNING id
        ";
        
        let row = sqlx::query(query)
            .bind(entity.source.as_str())
            .bind(&entity.entity_key)
            .bind(&entity.canonical_name)
            .bind(&entity.lang)
            .bind(&entity.entity_type)
            .bind(&entity.properties)
            .bind(entity.confidence)
            .bind(&entity.dump_version)
            .bind(&entity.toolchain)
            .bind(&entity.license)
            .fetch_one(&mut *tx)
            .await
            .context("Failed to upsert entity")?;
        
        let entity_id: Uuid = row.try_get("id")?;
        
        // Insert vectors
        for (model_id, vec) in vectors {
            let vec_query = "
                INSERT INTO knowledge_vectors (entity_id, model_id, vec)
                VALUES ($1, $2, $3)
                ON CONFLICT (entity_id, model_id)
                DO UPDATE SET vec = EXCLUDED.vec
            ";
            
            sqlx::query(vec_query)
                .bind(entity_id)
                .bind(&model_id)
                .bind(&vec)
                .execute(&mut *tx)
                .await
                .context("Failed to insert vector")?;
        }
        
        tx.commit().await?;
        
        Ok(entity_id)
    }
    
    /// Get entity vector
    pub async fn kb_get_entity_vector(
        &self,
        entity_id: Uuid,
        model_id: &str,
    ) -> Result<Vec<f32>> {
        let query = "SELECT vec FROM knowledge_vectors WHERE entity_id = $1 AND model_id = $2";
        
        let row = sqlx::query(query)
            .bind(entity_id)
            .bind(model_id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to get entity vector")?;
        
        let vec: Vec<f32> = row.try_get("vec")?;
        Ok(vec)
    }
    
    /// Get entities by source
    pub async fn kb_get_entities_by_source(
        &self,
        source: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ExternalKnowledgeEntity>> {
        let query = "
            SELECT id, source, entity_key, canonical_name, lang, entity_type,
                   properties, confidence, usage_count, usage_decay, last_accessed,
                   created_at, dump_version, toolchain, license
            FROM external_knowledge_entities
            WHERE source = $1
            ORDER BY usage_count DESC
            LIMIT $2
        ";
        
        let rows = sqlx::query(query)
            .bind(source)
            .bind(limit.unwrap_or(10000) as i64)
            .fetch_all(&self.pool())
            .await
            .context("Failed to get entities by source")?;
        
        let mut entities = Vec::new();
        for row in rows {
            entities.push(ExternalKnowledgeEntity {
                id: Some(row.try_get("id")?),
                source: match row.try_get::<String, _>("source")?.as_str() {
                    "wikidata" => KnowledgeSource::Wikidata,
                    "wordnet" => KnowledgeSource::WordNet,
                    _ => continue,
                },
                entity_key: row.try_get("entity_key")?,
                canonical_name: row.try_get("canonical_name")?,
                lang: row.try_get("lang")?,
                entity_type: row.try_get("entity_type")?,
                properties: row.try_get("properties")?,
                confidence: row.try_get("confidence")?,
                usage_count: row.try_get("usage_count")?,
                usage_decay: row.try_get("usage_decay")?,
                last_accessed: row.try_get("last_accessed")?,
                created_at: row.try_get("created_at")?,
                dump_version: row.try_get("dump_version")?,
                toolchain: row.try_get("toolchain")?,
                license: row.try_get("license")?,
            });
        }
        
        Ok(entities)
    }
    
    /// Create relationship
    pub async fn kb_create_relationship(
        &self,
        relationship: KnowledgeRelationship,
    ) -> Result<Uuid> {
        let query = "
            INSERT INTO knowledge_relationships 
            (source_entity_id, target_entity_id, relationship_type, confidence, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
        ";
        
        let row = sqlx::query(query)
            .bind(relationship.source_entity_id)
            .bind(relationship.target_entity_id)
            .bind(&relationship.relationship_type)
            .bind(relationship.confidence)
            .bind(&relationship.metadata)
            .fetch_one(&self.pool)
            .await
            .context("Failed to create relationship")?;
        
        let id: Uuid = row.try_get("id")?;
        Ok(id)
    }
    
    /// Get knowledge base statistics
    pub async fn kb_get_stats(&self) -> Result<Vec<KnowledgeStats>> {
        let query = "SELECT * FROM kb_get_stats()";
        
        let rows = sqlx::query(query)
            .fetch_all(&self.pool())
            .await
            .context("Failed to get stats")?;
        
        let mut stats = Vec::new();
        for row in rows {
            stats.push(KnowledgeStats {
                source: row.try_get("source")?,
                total_entities: row.try_get("total_entities")?,
                total_vectors: row.try_get("total_vectors")?,
                total_relationships: row.try_get("total_relationships")?,
                avg_confidence: row.try_get("avg_confidence")?,
                avg_usage_count: row.try_get("avg_usage_count")?,
                last_updated: row.try_get("last_updated")?,
            });
        }
        
        Ok(stats)
    }
}

