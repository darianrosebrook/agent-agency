//! Wikidata lexeme parser and normalizer
//!
//! Parses Wikidata JSON dump (gzipped) and extracts lexemes with their senses,
//! forms, and glosses. Normalizes to database-ready format with embeddings.
//!
//! @author @darianrosebrook

use crate::types::*;
use crate::{IngestionConfig, IngestionStats, KnowledgeIngestor};
use agent_agency_database::models::*;
use agent_agency_database as database;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{debug, info, warn};
use unicode_normalization::UnicodeNormalization;

/// Parse Wikidata lexemes from gzipped JSON dump
pub async fn parse_wikidata_dump<P: AsRef<Path>>(
    ingestor: &KnowledgeIngestor,
    path: P,
) -> Result<IngestionStats> {
    let start_time = std::time::Instant::now();
    let mut stats = IngestionStats::new();
    
    info!("Opening Wikidata dump: {:?}", path.as_ref());
    
    let file = File::open(path.as_ref())
        .context("Failed to open Wikidata dump file")?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    
    let config = ingestor.config();
    let limit = config.limit.unwrap_or(usize::MAX);
    let mut count = 0;
    
    // Stream parse line by line to avoid loading entire file into memory
    for line in reader.lines() {
        if count >= limit {
            info!("Reached limit of {} entities", limit);
            break;
        }
        
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                warn!("Failed to read line: {}", e);
                stats.errors.push(format!("Line read error: {}", e));
                continue;
            }
        };
        
        // Skip empty lines and array markers
        let line = line.trim();
        if line.is_empty() || line == "[" || line == "]" {
            continue;
        }
        
        // Remove trailing comma if present
        let line = line.trim_end_matches(',');
        
        // Parse JSON
        let json: Value = match serde_json::from_str(line) {
            Ok(j) => j,
            Err(e) => {
                debug!("Failed to parse JSON: {}", e);
                continue;
            }
        };
        
        // Check if this is a lexeme (starts with 'L')
        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
            if !id.starts_with('L') {
                continue;
            }
        } else {
            continue;
        }
        
        stats.entities_processed += 1;
        
        // Parse and normalize lexeme
        match parse_lexeme(&json, config) {
            Ok(Some(parsed)) => {
                // Generate embedding
                match generate_embedding(ingestor, &parsed).await {
                    Ok(embedding) => {
                        // Insert into database
                        match insert_entity(ingestor, parsed, embedding).await {
                            Ok(_) => {
                                stats.entities_inserted += 1;
                                stats.vectors_generated += 1;
                                count += 1;
                                
                                if count % 100 == 0 {
                                    info!("Processed {} Wikidata lexemes", count);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to insert entity: {}", e);
                                stats.entities_skipped += 1;
                                stats.errors.push(format!("Insert error: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to generate embedding: {}", e);
                        stats.entities_skipped += 1;
                        stats.errors.push(format!("Embedding error: {}", e));
                    }
                }
            }
            Ok(None) => {
                stats.entities_skipped += 1;
            }
            Err(e) => {
                warn!("Failed to parse lexeme: {}", e);
                stats.entities_skipped += 1;
                stats.errors.push(format!("Parse error: {}", e));
            }
        }
    }
    
    stats.processing_time_ms = start_time.elapsed().as_millis() as u64;
    
    info!(
        "Wikidata ingestion complete: {} inserted, {} skipped, {} errors",
        stats.entities_inserted,
        stats.entities_skipped,
        stats.errors.len()
    );
    
    Ok(stats)
}

/// Parse a single Wikidata lexeme JSON object
fn parse_lexeme(json: &Value, config: &IngestionConfig) -> Result<Option<ParsedEntity>> {
    let id = json.get("id")
        .and_then(|v| v.as_str())
        .context("Missing lexeme ID")?
        .to_string();
    
    // Extract lemmas (surface forms in different languages)
    let lemmas = extract_lemmas(json)?;
    if lemmas.is_empty() {
        return Ok(None);
    }
    
    // Get preferred language lemma
    let canonical_name = get_preferred_lemma(&lemmas, &config.languages)?;
    let lang = config.languages.first().cloned();
    
    // Extract language and lexical category
    let language = json.get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let lexical_category = json.get("lexicalCategory")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Extract senses (meanings)
    let senses = extract_senses(json, &config.languages)?;
    
    // Extract forms (inflections)
    let forms = extract_forms(json)?;
    
    // Build properties JSONB
    let properties = serde_json::json!({
        "lexeme_id": id,
        "lemma": lemmas,
        "language": language,
        "lexical_category": lexical_category,
        "senses": senses,
        "forms": forms,
    });
    
    // Build embedding text from glosses
    let embedding_text = build_embedding_text(&canonical_name, &senses);
    
    let entity = ExternalKnowledgeEntity {
        id: None,
        source: KnowledgeSource::Wikidata,
        entity_key: id,
        canonical_name: normalize_text(&canonical_name),
        lang,
        entity_type: Some("lexeme".to_string()),
        properties,
        confidence: 1.0,
        usage_count: 0,
        usage_decay: Some(1.0),
        last_accessed: None,
        created_at: None,
        dump_version: Some("wikidata-2025-09-24".to_string()),
        toolchain: Some(format!("knowledge-ingestor-v{}", env!("CARGO_PKG_VERSION"))),
        license: Some("CC0".to_string()),
    };
    
    Ok(Some(ParsedEntity {
        entity,
        embedding_text,
    }))
}

/// Extract lemmas from lexeme JSON
fn extract_lemmas(json: &Value) -> Result<HashMap<String, String>> {
    let mut lemmas = HashMap::new();
    
    if let Some(lemmas_obj) = json.get("lemmas").and_then(|v| v.as_object()) {
        for (lang, value) in lemmas_obj {
            if let Some(text) = value.get("value").and_then(|v| v.as_str()) {
                lemmas.insert(lang.clone(), text.to_string());
            }
        }
    }
    
    Ok(lemmas)
}

/// Get preferred lemma based on language priority
fn get_preferred_lemma(lemmas: &HashMap<String, String>, lang_priority: &[String]) -> Result<String> {
    // Try each language in priority order
    for lang in lang_priority {
        if let Some(lemma) = lemmas.get(lang) {
            return Ok(lemma.clone());
        }
    }
    
    // Fallback to first available lemma
    lemmas.values()
        .next()
        .cloned()
        .context("No lemmas available")
}

/// Extract senses (meanings) from lexeme JSON
fn extract_senses(json: &Value, lang_priority: &[String]) -> Result<Vec<Value>> {
    let mut senses = Vec::new();
    
    if let Some(senses_array) = json.get("senses").and_then(|v| v.as_array()) {
        for sense in senses_array {
            let sense_id = sense.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            // Extract glosses
            let mut glosses = HashMap::new();
            if let Some(glosses_obj) = sense.get("glosses").and_then(|v| v.as_object()) {
                for (lang, value) in glosses_obj {
                    if let Some(text) = value.get("value").and_then(|v| v.as_str()) {
                        glosses.insert(lang.clone(), text.to_string());
                    }
                }
            }
            
            if !glosses.is_empty() {
                senses.push(serde_json::json!({
                    "id": sense_id,
                    "glosses": glosses,
                }));
            }
        }
    }
    
    Ok(senses)
}

/// Extract forms (inflections) from lexeme JSON
fn extract_forms(json: &Value) -> Result<Vec<String>> {
    let mut forms = Vec::new();
    
    if let Some(forms_array) = json.get("forms").and_then(|v| v.as_array()) {
        for form in forms_array {
            if let Some(representations) = form.get("representations").and_then(|v| v.as_object()) {
                for (_, value) in representations {
                    if let Some(text) = value.get("value").and_then(|v| v.as_str()) {
                        forms.push(text.to_string());
                    }
                }
            }
        }
    }
    
    Ok(forms)
}

/// Build embedding text from canonical name and senses
fn build_embedding_text(canonical_name: &str, senses: &[Value]) -> String {
    let mut text = canonical_name.to_string();
    
    // Add glosses from senses
    for sense in senses {
        if let Some(glosses) = sense.get("glosses").and_then(|v| v.as_object()) {
            for (_, gloss_value) in glosses {
                if let Some(gloss) = gloss_value.as_str() {
                    text.push_str(". ");
                    text.push_str(gloss);
                }
            }
        }
    }
    
    text
}

/// Normalize text for consistent matching
fn normalize_text(text: &str) -> String {
    text.nfc().collect::<String>().to_lowercase()
}

/// Generate embedding for parsed entity
#[cfg(feature = "embeddings")]
async fn generate_embedding(
    ingestor: &KnowledgeIngestor,
    parsed: &ParsedEntity,
) -> Result<Vec<f32>> {
    let embedding = ingestor
        .embedding_service()
        .generate_embedding(
            &parsed.embedding_text,
            embedding_service::ContentType::Knowledge,
            "wikidata",
        )
        .await?;
    
    Ok(embedding.vector)
}

/// Insert entity and vector into database
async fn insert_entity(
    ingestor: &KnowledgeIngestor,
    parsed: ParsedEntity,
    embedding: Vec<f32>,
) -> Result<uuid::Uuid> {
    ingestor
        .db_client()
        .kb_upsert_entity(parsed.entity, vec![(ingestor.config().model_id.clone(), embedding)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("Database"), "database");
        assert_eq!(normalize_text("Café"), "café");
        assert_eq!(normalize_text("HELLO WORLD"), "hello world");
    }

    #[test]
    fn test_get_preferred_lemma() {
        let mut lemmas = HashMap::new();
        lemmas.insert("en".to_string(), "database".to_string());
        lemmas.insert("de".to_string(), "Datenbank".to_string());
        
        let priority = vec!["en".to_string(), "de".to_string()];
        let result = get_preferred_lemma(&lemmas, &priority).unwrap();
        assert_eq!(result, "database");
        
        let priority = vec!["de".to_string(), "en".to_string()];
        let result = get_preferred_lemma(&lemmas, &priority).unwrap();
        assert_eq!(result, "Datenbank");
    }

    #[test]
    fn test_build_embedding_text() {
        let senses = vec![
            serde_json::json!({
                "id": "S1",
                "glosses": {
                    "en": "organized collection of data"
                }
            }),
        ];
        
        let text = build_embedding_text("database", &senses);
        assert!(text.contains("database"));
        assert!(text.contains("organized collection of data"));
    }
}

