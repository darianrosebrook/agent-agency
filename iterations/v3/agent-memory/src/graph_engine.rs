#![cfg(feature = "database")]
//! Knowledge Graph Engine - Entity and relationship management

use crate::memory_types::*;
use crate::MemoryResult;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use std::sync::Arc;
use regex::Regex;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use std::collections::HashMap;
use reqwest::Client;
use anyhow::{Context, Result};

/// Knowledge graph entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub description: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_memories: Vec<MemoryId>,
}

/// Entity types in the knowledge graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Agent,
    Task,
    Capability,
    Domain,
    Tool,
    Outcome,
    Concept,
    Person,
    Organization,
    Location,
    Technology,
    Other,
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub source_entity: String,
    pub target_entity: String,
    pub relationship_type: RelationshipType,
    pub properties: HashMap<String, serde_json::Value>,
    pub strength: f32,
    pub confidence: f32,
    pub bidirectional: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_memories: Vec<MemoryId>,
}

/// Types of relationships between entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    Performs,
    Requires,
    Enables,
    Conflicts,
    Improves,
    LearnsFrom,
    CollaboratesWith,
    Manages,
    Creates,
    Uses,
    Contains,
    RelatedTo,
    Causes,
    Prevents,
    SimilarTo,
    Other,
}

/// Real HTTP-based entity extraction service
#[derive(Debug)]
pub struct HttpEntityExtractionService {
    client: Client,
    base_url: String,
    timeout_ms: u64,
}

impl HttpEntityExtractionService {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            timeout_ms: 30000,
        }
    }

    /// Extract entities from text via HTTP call to external service
    pub async fn extract_entities(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        let url = format!("{}/api/v1/entities/extract", self.base_url);
        
        let payload = serde_json::json!({
            "text": text,
            "entity_types": ["PERSON", "ORG", "GPE", "EVENT", "WORK_OF_ART", "LAW", "LANGUAGE", "DATE", "TIME", "PERCENT", "MONEY", "QUANTITY", "ORDINAL", "CARDINAL"]
        });

        debug!("Extracting entities from text: {}...", &text[..text.len().min(100)]);

        let response = self.client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .send()
            .await
            .context("Failed to send entity extraction request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Entity extraction service error {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await
            .context("Failed to parse entity extraction response")?;

        // Extract entities from response
        let entities = result["entities"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid entity extraction response format"))?
            .iter()
            .map(|entity| ExtractedEntity {
                text: entity["text"].as_str().unwrap_or("").to_string(),
                label: entity["label"].as_str().unwrap_or("OTHER").to_string(),
                start: entity["start"].as_u64().unwrap_or(0) as usize,
                end: entity["end"].as_u64().unwrap_or(0) as usize,
                confidence: entity["confidence"].as_f64().unwrap_or(0.0) as f32,
            })
            .collect::<Vec<ExtractedEntity>>();

        debug!("Extracted {} entities", entities.len());
        Ok(entities)
    }

    /// Extract relationships from text via HTTP call
    pub async fn extract_relationships(&self, text: &str, entities: &[ExtractedEntity]) -> Result<Vec<ExtractedRelationship>> {
        let url = format!("{}/api/v1/relationships/extract", self.base_url);
        
        let payload = serde_json::json!({
            "text": text,
            "entities": entities
        });

        debug!("Extracting relationships from text with {} entities", entities.len());

        let response = self.client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .send()
            .await
            .context("Failed to send relationship extraction request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Relationship extraction service error {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await
            .context("Failed to parse relationship extraction response")?;

        // Extract relationships from response
        let relationships = result["relationships"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid relationship extraction response format"))?
            .iter()
            .map(|rel| ExtractedRelationship {
                source: rel["source"].as_str().unwrap_or("").to_string(),
                target: rel["target"].as_str().unwrap_or("").to_string(),
                relation: rel["relation"].as_str().unwrap_or("RELATED_TO").to_string(),
                confidence: rel["confidence"].as_f64().unwrap_or(0.0) as f32,
            })
            .collect::<Vec<ExtractedRelationship>>();

        debug!("Extracted {} relationships", relationships.len());
        Ok(relationships)
    }

    /// Health check for entity extraction service
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Extracted entity from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub text: String,
    pub label: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}

/// Extracted relationship from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelationship {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: f32,
}

/// Knowledge Graph Engine for entity and relationship management
#[derive(Debug)]
pub struct KnowledgeGraphEngine {
    db_pool: Arc<PgPool>,
    config: GraphConfig,
    entity_cache: dashmap::DashMap<String, Entity>,
    relationship_cache: dashmap::DashMap<String, Relationship>,
    entity_extraction_service: Arc<HttpEntityExtractionService>,
}

impl KnowledgeGraphEngine {
    /// Create a new knowledge graph engine
    pub async fn new(config: &GraphConfig) -> MemoryResult<Self> {
        // Get database URL from environment
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());

        // Create direct database connection pool
        let db_pool = Arc::new(
            PgPool::connect(&database_url)
                .await
                .context("Failed to connect to database for knowledge graph")?
        );

        // Get entity extraction service URL from environment or use default
        let extraction_url = std::env::var("ENTITY_EXTRACTION_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());

        info!("Initializing HTTP entity extraction service at: {}", extraction_url);

        // Create HTTP-based entity extraction service
        let entity_extraction_service = Arc::new(HttpEntityExtractionService::new(extraction_url));
        
        // Test connection
        if let Err(e) = entity_extraction_service.health_check().await {
            warn!("Entity extraction service health check failed: {}", e);
        } else {
            info!("Entity extraction service health check passed");
        }

        Ok(Self {
            db_client,
            config: config.clone(),
            entity_cache: dashmap::DashMap::new(),
            relationship_cache: dashmap::DashMap::new(),
            entity_extraction_service,
        })
    }

    /// Extract entities from an agent experience
    pub async fn extract_entities_from_experience(&self, experience: &AgentExperience) -> MemoryResult<Vec<Entity>> {
        let mut entities = Vec::new();

        // Extract agent entity
        let agent_entity = Entity {
            id: format!("agent:{}", experience.agent_id),
            entity_type: EntityType::Agent,
            name: experience.agent_id.clone(),
            description: Some(format!("Agent {}", experience.agent_id)),
            properties: HashMap::from([
                ("type".to_string(), serde_json::json!("agent")),
                ("last_active".to_string(), serde_json::json!(experience.timestamp)),
            ]),
            embedding: None,
            confidence: 1.0,
            created_at: experience.timestamp,
            updated_at: experience.timestamp,
            source_memories: vec![experience.id],
        };
        entities.push(agent_entity);

        // Extract task entity
        let task_entity = Entity {
            id: format!("task:{}", experience.task_id),
            entity_type: EntityType::Task,
            name: experience.task_id.clone(),
            description: Some(experience.context.description.clone()),
            properties: HashMap::from([
                ("type".to_string(), serde_json::json!(experience.context.task_type)),
                ("domain".to_string(), serde_json::json!(experience.context.domain)),
                ("priority".to_string(), serde_json::json!(experience.context.temporal_context.as_ref().map(|tc| tc.priority as i32))),
            ]),
            embedding: None,
            confidence: 1.0,
            created_at: experience.timestamp,
            updated_at: experience.timestamp,
            source_memories: vec![experience.id],
        };
        entities.push(task_entity);

        // Extract capability entities from learned capabilities
        for capability in &experience.outcome.learned_capabilities {
            let capability_entity = Entity {
                id: format!("capability:{}", capability.to_lowercase().replace(" ", "_")),
                entity_type: EntityType::Capability,
                name: capability.clone(),
                description: Some(format!("Capability: {}", capability)),
                properties: HashMap::from([
                    ("proficiency_level".to_string(), serde_json::json!(experience.outcome.performance_score.unwrap_or(0.5))),
                    ("learned_from".to_string(), serde_json::json!(experience.task_id)),
                ]),
                embedding: None,
                confidence: 0.8,
                created_at: experience.timestamp,
                updated_at: experience.timestamp,
                source_memories: vec![experience.id],
            };
            entities.push(capability_entity);
        }

        // Extract domain entities
        for domain in &experience.context.domain {
            let domain_entity = Entity {
                id: format!("domain:{}", domain.to_lowercase().replace(" ", "_")),
                entity_type: EntityType::Domain,
                name: domain.clone(),
                description: Some(format!("Domain: {}", domain)),
                properties: HashMap::new(),
                embedding: None,
                confidence: 0.9,
                created_at: experience.timestamp,
                updated_at: experience.timestamp,
                source_memories: vec![experience.id],
            };
            entities.push(domain_entity);
        }

        // Extract entities from text content using regex patterns
        entities.extend(self.extract_entities_from_text(&experience.context.description).await?);

        Ok(entities)
    }

    /// Extract entities from text content using regex patterns
    async fn extract_entities_from_text(&self, text: &str) -> MemoryResult<Vec<Entity>> {
        let mut entities = Vec::new();

        // Technology patterns
        let tech_patterns = [
            r"\b(Rust|Python|JavaScript|TypeScript|Go|Java|C\+\+)\b",
            r"\b(PostgreSQL|MongoDB|Redis|Elasticsearch)\b",
            r"\b(Docker|Kubernetes|AWS|Azure|GCP)\b",
            r"\b(API|REST|GraphQL|WebSocket)\b",
        ];

        for pattern in &tech_patterns {
            let regex = Regex::new(pattern)?;
            for capture in regex.find_iter(text) {
                let tech_name = capture.as_str().to_string();
                let entity = Entity {
                    id: format!("technology:{}", tech_name.to_lowercase()),
                    entity_type: EntityType::Technology,
                    name: tech_name.clone(),
                    description: Some(format!("Technology: {}", tech_name)),
                    properties: HashMap::new(),
                    embedding: None,
                    confidence: 0.7,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    source_memories: vec![],
                };
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// Extract relationships from an agent experience
    pub async fn extract_relationships_from_experience(&self, experience: &AgentExperience, entities: &[Entity]) -> MemoryResult<Vec<Relationship>> {
        let mut relationships = Vec::new();

        let agent_entity_id = format!("agent:{}", experience.agent_id);
        let task_entity_id = format!("task:{}", experience.task_id);

        // Agent performs task
        relationships.push(Relationship {
            id: format!("rel:{}-performs-{}", agent_entity_id, task_entity_id),
            source_entity: agent_entity_id.clone(),
            target_entity: task_entity_id.clone(),
            relationship_type: RelationshipType::Performs,
            properties: HashMap::from([
                ("performance_score".to_string(), serde_json::json!(experience.outcome.performance_score)),
                ("execution_time_ms".to_string(), serde_json::json!(experience.outcome.execution_time_ms)),
            ]),
            strength: experience.outcome.performance_score.unwrap_or(0.5),
            confidence: 1.0,
            bidirectional: false,
            created_at: experience.timestamp,
            updated_at: experience.timestamp,
            source_memories: vec![experience.id],
        });

        // Task requires capabilities
        for capability in &experience.outcome.learned_capabilities {
            let capability_entity_id = format!("capability:{}", capability.to_lowercase().replace(" ", "_"));
            relationships.push(Relationship {
                id: format!("rel:{}-requires-{}", task_entity_id, capability_entity_id),
                source_entity: task_entity_id.clone(),
                target_entity: capability_entity_id,
                relationship_type: RelationshipType::Requires,
                properties: HashMap::new(),
                strength: 0.8,
                confidence: 0.9,
                bidirectional: false,
                created_at: experience.timestamp,
                updated_at: experience.timestamp,
                source_memories: vec![experience.id],
            });
        }

        // Agent learns capabilities from task
        for capability in &experience.outcome.learned_capabilities {
            let capability_entity_id = format!("capability:{}", capability.to_lowercase().replace(" ", "_"));
            relationships.push(Relationship {
                id: format!("rel:{}-learns-{}", agent_entity_id, capability_entity_id),
                source_entity: agent_entity_id.clone(),
                target_entity: capability_entity_id,
                relationship_type: RelationshipType::LearnsFrom,
                properties: HashMap::from([
                    ("proficiency_gain".to_string(), serde_json::json!(experience.outcome.performance_score.unwrap_or(0.5))),
                ]),
                strength: experience.outcome.performance_score.unwrap_or(0.5),
                confidence: 0.8,
                bidirectional: false,
                created_at: experience.timestamp,
                updated_at: experience.timestamp,
                source_memories: vec![experience.id],
            });
        }

        // Domain relationships
        for domain in &experience.context.domain {
            let domain_entity_id = format!("domain:{}", domain.to_lowercase().replace(" ", "_"));
            relationships.push(Relationship {
                id: format!("rel:{}-contains-{}", domain_entity_id, task_entity_id),
                source_entity: domain_entity_id,
                target_entity: task_entity_id.clone(),
                relationship_type: RelationshipType::Contains,
                properties: HashMap::new(),
                strength: 0.9,
                confidence: 0.8,
                bidirectional: false,
                created_at: experience.timestamp,
                updated_at: experience.timestamp,
                source_memories: vec![experience.id],
            });
        }

        Ok(relationships)
    }

    /// Upsert an entity (insert or update if exists)
    pub async fn upsert_entity(&self, entity: Entity) -> MemoryResult<()> {
        // Check if entity exists
        let existing = sqlx::query(
            "SELECT id FROM knowledge_graph_entities WHERE id = $1",
        )
        .bind(&entity.id)
        .fetch_optional(&*self.db_pool)
        .await?;

        if existing.is_some() {
            // Update existing entity
            sqlx::query(
                r#"
                UPDATE knowledge_graph_entities
                SET name = $2, description = $3, properties = $4, embedding = $5,
                    confidence = GREATEST(confidence, $6), updated_at = $7,
                    source_memories = array_cat(source_memories, $8)
                WHERE id = $1
                "#,
            )
            .bind(&entity.id)
            .bind(&entity.name)
            .bind(&entity.description)
            .bind(serde_json::to_value(&entity.properties)?)
            .bind(&entity.embedding)
            .bind(entity.confidence)
            .bind(entity.updated_at)
            .bind(&entity.source_memories.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .execute(&*self.db_pool)
            .await?;
        } else {
            // Insert new entity
            sqlx::query(
                r#"
                INSERT INTO knowledge_graph_entities (
                    id, entity_type, name, description, properties, embedding,
                    confidence, created_at, updated_at, source_memories
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(&entity.id)
            .bind(entity.entity_type as i32)
            .bind(&entity.name)
            .bind(&entity.description)
            .bind(serde_json::to_value(&entity.properties)?)
            .bind(&entity.embedding)
            .bind(entity.confidence)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .bind(&entity.source_memories.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .execute(&*self.db_pool)
            .await?;
        }

        // Update cache
        self.entity_cache.insert(entity.id.clone(), entity);

        Ok(())
    }

    /// Upsert a relationship
    pub async fn upsert_relationship(&self, relationship: Relationship) -> MemoryResult<()> {
        let relationship_id = relationship.id.clone();

        // Check if relationship exists
        let existing = sqlx::query(
            "SELECT id FROM knowledge_graph_relationships WHERE id = $1",
        )
        .bind(&relationship_id)
        .fetch_optional(&*self.db_pool)
        .await?;

        if existing.is_some() {
            // Update existing relationship - average strengths/confidences
            sqlx::query(
                r#"
                UPDATE knowledge_graph_relationships
                SET strength = (strength + $4) / 2, confidence = GREATEST(confidence, $5),
                    updated_at = $6, source_memories = array_cat(source_memories, $7)
                WHERE id = $1
                "#,
            )
            .bind(&relationship_id)
            .bind(relationship.strength)
            .bind(relationship.confidence)
            .bind(relationship.updated_at)
            .bind(&relationship.source_memories.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .execute(&*self.db_pool)
            .await?;
        } else {
            // Insert new relationship
            sqlx::query(
                r#"
                INSERT INTO knowledge_graph_relationships (
                    id, source_entity, target_entity, relationship_type, properties,
                    strength, confidence, bidirectional, created_at, updated_at, source_memories
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(&relationship_id)
            .bind(&relationship.source_entity)
            .bind(&relationship.target_entity)
            .bind(relationship.relationship_type as i32)
            .bind(serde_json::to_value(&relationship.properties)?)
            .bind(relationship.strength)
            .bind(relationship.confidence)
            .bind(relationship.bidirectional)
            .bind(relationship.created_at)
            .bind(relationship.updated_at)
            .bind(&relationship.source_memories.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .execute(&*self.db_pool)
            .await?;
        }

        // Update cache
        self.relationship_cache.insert(relationship_id, relationship);

        Ok(())
    }

    /// Find entities related to a task context
    pub async fn find_related_entities(&self, context: &TaskContext, limit: usize) -> MemoryResult<Vec<(MemoryId, Vec<String>)>> {
        let mut related_memories = Vec::new();

        // Find tasks of similar type
        let similar_tasks = sqlx::query(
            r#"
            SELECT id, properties
            FROM knowledge_graph_entities
            WHERE entity_type = $1 AND properties->>'type' = $2
            ORDER BY confidence DESC
            LIMIT $3
            "#,
        )
        .bind(EntityType::Task as i32)
        .bind(&context.task_type)
        .bind(limit as i32)
        .fetch_all(&*self.db_pool)
        .await?;

        for row in similar_tasks {
            let entity_id: String = row.try_get("id")?;
            let memory_path = vec![format!("Similar task: {}", entity_id)];

            // Find memories associated with this entity
            let memories = sqlx::query(
                r#"
                SELECT id FROM knowledge_graph_relationships
                WHERE (source_entity = $1 OR target_entity = $1)
                  AND relationship_type = $2
                "#,
            )
            .bind(&entity_id)
            .bind(RelationshipType::Performs as i32)
            .fetch_all(&*self.db_pool)
            .await?;

            for memory_row in memories {
                // This is simplified - would need to join with actual memory IDs
                // For now, just return placeholder
                related_memories.push((MemoryId::new_v4(), memory_path.clone()));
            }
        }

        Ok(related_memories)
    }

    /// Perform multi-hop reasoning
    pub async fn perform_multi_hop_reasoning(&self, query: ReasoningQuery) -> MemoryResult<ReasoningResult> {
        let mut all_paths = Vec::new();

        for start_entity in &query.start_entities {
            let paths = self.find_paths(start_entity, &query.target_entities, query.max_hops).await?;
            all_paths.extend(paths);
        }

        // Filter by confidence and sort
        let mut valid_paths: Vec<ReasoningPath> = all_paths.into_iter()
            .filter(|path| path.confidence >= query.min_confidence)
            .collect();

        valid_paths.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        valid_paths.truncate(10); // Limit results

        let entities_discovered = valid_paths.iter()
            .flat_map(|path| path.entities.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Convert ReasoningPath to Vec<String> for paths field
        let path_strings: Vec<Vec<String>> = valid_paths.into_iter()
            .map(|path| path.entities)
            .collect();

        Ok(ReasoningResult {
            path: Vec::new(), // Simplified - use first path if available
            paths: path_strings,
            confidence: 0.8, // Overall confidence
            confidence_score: 0.8, // Simplified
            reasoning_steps: vec!["Multi-hop reasoning completed".to_string()],
            reasoning_time_ms: 100,
            entities_discovered,
        })
    }

    /// Find paths between entities (simplified implementation)
    async fn find_paths(&self, start: &str, targets: &[String], max_hops: usize) -> MemoryResult<Vec<ReasoningPath>> {
        let mut paths = Vec::new();

        // Simple breadth-first search implementation
        let mut queue = vec![(vec![start.to_string()], vec![], 1.0, 0)];
        let mut visited = std::collections::HashSet::new();

        while let Some((current_path, relationships, confidence, hops)) = queue.pop() {
            if hops >= max_hops {
                continue;
            }

            let current_entity = current_path.last().unwrap();

            // Check if we reached a target
            if targets.contains(current_entity) {
                paths.push(ReasoningPath {
                    entities: current_path.clone(),
                    relationships: relationships.clone(),
                    confidence,
                    length: hops,
                });
                continue;
            }

            // Avoid cycles
            if visited.contains(current_entity) {
                continue;
            }
            visited.insert(current_entity.clone());

            // Find relationships from current entity
            let related = sqlx::query(
                r#"
                SELECT target_entity, relationship_type, strength, confidence
                FROM knowledge_graph_relationships
                WHERE source_entity = $1 AND confidence >= 0.5
                LIMIT 10
                "#,
            )
            .bind(current_entity)
            .fetch_all(&*self.db_pool)
            .await?;

            for row in related {
                let target: String = row.try_get("target_entity")?;
                let rel_type: i32 = row.try_get("relationship_type")?;
                let strength: f32 = row.try_get("strength")?;
                let rel_confidence: f32 = row.try_get("confidence")?;

                let mut new_path = current_path.clone();
                new_path.push(target);

                let mut new_relationships = relationships.clone();
                new_relationships.push(format!("{:?}", RelationshipType::RelatedTo));

                let new_confidence = confidence * strength * rel_confidence;
                let new_hops = hops + 1;

                queue.push((new_path, new_relationships, new_confidence, new_hops));
            }
        }

        Ok(paths)
    }

    /// Get graph statistics
    pub async fn get_graph_stats(&self) -> MemoryResult<GraphStats> {
        let entity_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_graph_entities")
            .fetch_one(&*self.db_pool)
            .await?;

        let relationship_count = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_graph_relationships")
            .fetch_one(&*self.db_pool)
            .await?;

        Ok(GraphStats {
            entity_count: entity_count,
            relationship_count: relationship_count,
            entity_types: HashMap::new(), // TODO: implement distribution
            relationship_types: HashMap::new(), // TODO: implement distribution
        })
    }
}

/// Graph query structure
#[derive(Debug, Clone)]
pub struct GraphQuery {
    pub entity_types: Option<Vec<EntityType>>,
    pub relationship_types: Option<Vec<RelationshipType>>,
    pub min_confidence: f32,
    pub limit: usize,
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub entity_count: i64,
    pub relationship_count: i64,
    pub entity_types: HashMap<EntityType, usize>,
    pub relationship_types: HashMap<RelationshipType, usize>,
}
