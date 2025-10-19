//! Cross-reference generation between Wikidata and WordNet
//!
//! Links Wikidata entities to WordNet synsets based on lemma matching,
//! POS agreement, and semantic similarity.
//!
//! @author @darianrosebrook

use crate::types::*;
use crate::{IngestionStats, KnowledgeIngestor};
use agent_agency_database::models::*;
use anyhow::Result;
use tracing::{debug, info};
use unicode_normalization::UnicodeNormalization;

/// Minimum semantic similarity threshold for cross-references
const MIN_SIMILARITY_THRESHOLD: f32 = 0.7;

/// Generate cross-references between Wikidata and WordNet entities
pub async fn generate_cross_references(
    ingestor: &KnowledgeIngestor,
) -> Result<IngestionStats> {
    let start_time = std::time::Instant::now();
    let mut stats = IngestionStats::new();
    
    info!("Generating cross-references between Wikidata and WordNet");
    
    // Get all Wikidata entities
    let wikidata_entities = ingestor
        .db_client()
        .kb_get_entities_by_source("wikidata", None)
        .await?;
    
    info!("Found {} Wikidata entities", wikidata_entities.len());
    
    // Get all WordNet entities
    let wordnet_entities = ingestor
        .db_client()
        .kb_get_entities_by_source("wordnet", None)
        .await?;
    
    info!("Found {} WordNet entities", wordnet_entities.len());
    
    // Build WordNet lookup index by canonical name
    let mut wordnet_by_name = std::collections::HashMap::new();
    for entity in &wordnet_entities {
        let normalized = normalize_for_matching(&entity.canonical_name);
        wordnet_by_name
            .entry(normalized)
            .or_insert_with(Vec::new)
            .push(entity);
    }
    
    // Match Wikidata entities to WordNet synsets
    for wikidata_entity in &wikidata_entities {
        match find_wordnet_matches(wikidata_entity, &wordnet_by_name, ingestor).await {
            Ok(matches) => {
                for match_result in matches {
                    // Create relationship
                    match create_cross_reference(ingestor, &match_result).await {
                        Ok(_) => {
                            stats.relationships_created += 1;
                            debug!(
                                "Created cross-reference: {} <-> {} (confidence: {:.2})",
                                wikidata_entity.entity_key,
                                match_result.wordnet_id,
                                match_result.confidence
                            );
                        }
                        Err(e) => {
                            stats.errors.push(format!(
                                "Failed to create cross-reference: {}",
                                e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Failed to find matches for {}: {}", wikidata_entity.entity_key, e);
            }
        }
        
        if stats.relationships_created % 100 == 0 && stats.relationships_created > 0 {
            info!("Created {} cross-references", stats.relationships_created);
        }
    }
    
    stats.processing_time_ms = start_time.elapsed().as_millis() as u64;
    
    info!(
        "Cross-reference generation complete: {} relationships created",
        stats.relationships_created
    );
    
    Ok(stats)
}

/// Find matching WordNet synsets for a Wikidata entity
async fn find_wordnet_matches(
    wikidata_entity: &ExternalKnowledgeEntity,
    wordnet_by_name: &std::collections::HashMap<String, Vec<&ExternalKnowledgeEntity>>,
    ingestor: &KnowledgeIngestor,
) -> Result<Vec<CrossReferenceMatch>> {
    let mut matches = Vec::new();
    
    // Extract lemmas from Wikidata properties
    let lemmas = extract_wikidata_lemmas(wikidata_entity)?;
    
    // Try lexical matching first
    for lemma in &lemmas {
        let normalized = normalize_for_matching(lemma);
        
        if let Some(candidates) = wordnet_by_name.get(&normalized) {
            for candidate in candidates {
                // Check POS agreement if available
                if let Some(pos_match) = check_pos_agreement(wikidata_entity, candidate) {
                    if !pos_match {
                        continue;
                    }
                }
                
                // Calculate semantic similarity
                let similarity = calculate_semantic_similarity(
                    wikidata_entity,
                    candidate,
                    ingestor,
                )
                .await?;
                
                if similarity >= MIN_SIMILARITY_THRESHOLD {
                    matches.push(CrossReferenceMatch {
                        wikidata_id: wikidata_entity.id.unwrap(),
                        wordnet_id: candidate.id.unwrap(),
                        confidence: similarity as f64,
                        matching_method: "lemma+pos+semantic".to_string(),
                    });
                }
            }
        }
    }
    
    Ok(matches)
}

/// Extract lemmas from Wikidata entity properties
fn extract_wikidata_lemmas(entity: &ExternalKnowledgeEntity) -> Result<Vec<String>> {
    let mut lemmas = vec![entity.canonical_name.clone()];
    
    // Extract from properties
    if let Some(lemma_obj) = entity.properties.get("lemma") {
        if let Some(lemma_map) = lemma_obj.as_object() {
            for (_, value) in lemma_map {
                if let Some(lemma) = value.as_str() {
                    lemmas.push(lemma.to_string());
                }
            }
        }
    }
    
    // Extract from forms
    if let Some(forms) = entity.properties.get("forms") {
        if let Some(forms_array) = forms.as_array() {
            for form in forms_array {
                if let Some(form_str) = form.as_str() {
                    lemmas.push(form_str.to_string());
                }
            }
        }
    }
    
    Ok(lemmas)
}

/// Normalize text for matching
fn normalize_for_matching(text: &str) -> String {
    text.nfc()
        .collect::<String>()
        .to_lowercase()
        .replace('_', " ")
        .trim()
        .to_string()
}

/// Check if POS (part of speech) agrees between entities
fn check_pos_agreement(
    wikidata_entity: &ExternalKnowledgeEntity,
    wordnet_entity: &ExternalKnowledgeEntity,
) -> Option<bool> {
    // Extract Wikidata lexical category
    let wikidata_category = wikidata_entity
        .properties
        .get("lexical_category")
        .and_then(|v| v.as_str())?;
    
    // Extract WordNet POS
    let wordnet_pos = wordnet_entity
        .properties
        .get("pos")
        .and_then(|v| v.as_str())?;
    
    // Map Wikidata categories to WordNet POS
    let matches = match (wikidata_category, wordnet_pos) {
        ("Q1084", "noun") => true,  // Wikidata noun
        ("Q24905", "verb") => true, // Wikidata verb
        ("Q34698", "adjective") => true, // Wikidata adjective
        ("Q380057", "adverb") => true, // Wikidata adverb
        _ => false,
    };
    
    Some(matches)
}

/// Calculate semantic similarity between entities using embeddings
async fn calculate_semantic_similarity(
    wikidata_entity: &ExternalKnowledgeEntity,
    wordnet_entity: &ExternalKnowledgeEntity,
    ingestor: &KnowledgeIngestor,
) -> Result<f32> {
    // Get vectors for both entities
    let wikidata_vector = ingestor
        .db_client()
        .kb_get_entity_vector(
            wikidata_entity.id.unwrap(),
            &ingestor.config().model_id,
        )
        .await?;
    
    let wordnet_vector = ingestor
        .db_client()
        .kb_get_entity_vector(
            wordnet_entity.id.unwrap(),
            &ingestor.config().model_id,
        )
        .await?;
    
    // Calculate cosine similarity
    let similarity = cosine_similarity(&wikidata_vector, &wordnet_vector);
    
    Ok(similarity)
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (magnitude_a * magnitude_b)
}

/// Create a cross-reference relationship in the database
async fn create_cross_reference(
    ingestor: &KnowledgeIngestor,
    match_result: &CrossReferenceMatch,
) -> Result<uuid::Uuid> {
    let relationship = KnowledgeRelationship {
        id: None,
        source_entity_id: match_result.wikidata_id,
        target_entity_id: match_result.wordnet_id,
        relationship_type: "equivalent".to_string(),
        confidence: match_result.confidence,
        metadata: Some(serde_json::json!({
            "matching_method": match_result.matching_method,
        })),
    };
    
    ingestor
        .db_client()
        .kb_create_relationship(relationship)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_for_matching() {
        assert_eq!(normalize_for_matching("Database"), "database");
        assert_eq!(normalize_for_matching("data_base"), "data base");
        assert_eq!(normalize_for_matching("  HELLO  "), "hello");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
        
        let a = vec![1.0, 1.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let expected = 1.0 / 2.0_f32.sqrt();
        assert!((cosine_similarity(&a, &b) - expected).abs() < 0.001);
    }
}

