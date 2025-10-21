//! WordNet synset parser and normalizer
//!
//! Parses WordNet 3.1 dictionary files and extracts synsets with their definitions,
//! examples, and relationships (synonyms, hypernyms, hyponyms).
//!
//! @author @darianrosebrook

use crate::types::*;
use crate::{IngestionConfig, IngestionStats, KnowledgeIngestor};
use agent_agency_database::models::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tar::Archive;
use tracing::{debug, info, warn};

/// Parse WordNet synsets from tar.gz archive
pub async fn parse_wordnet_dump<P: AsRef<Path>>(
    ingestor: &KnowledgeIngestor,
    path: P,
) -> Result<IngestionStats> {
    let start_time = std::time::Instant::now();
    let mut stats = IngestionStats::new();
    
    info!("Opening WordNet dump: {:?}", path.as_ref());
    
    let file = File::open(path.as_ref())
        .context("Failed to open WordNet dump file")?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    
    let config = ingestor.config();
    let limit = config.limit.unwrap_or(usize::MAX);
    let mut count = 0;
    
    // Extract and parse data files
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        
        // Look for data.* files (noun, verb, adj, adv)
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.starts_with("data.") && !filename_str.ends_with(".exc") {
                info!("Processing WordNet file: {}", filename_str);
                
                let reader = BufReader::new(&mut entry);
                let file_stats = parse_wordnet_data_file(ingestor, reader, &mut count, limit).await?;
                stats.merge(file_stats);
                
                if count >= limit {
                    info!("Reached limit of {} entities", limit);
                    break;
                }
            }
        }
    }
    
    stats.processing_time_ms = start_time.elapsed().as_millis() as u64;
    
    info!(
        "WordNet ingestion complete: {} inserted, {} skipped, {} errors",
        stats.entities_inserted,
        stats.entities_skipped,
        stats.errors.len()
    );
    
    Ok(stats)
}

/// Parse a single WordNet data file
async fn parse_wordnet_data_file<R: BufRead>(
    ingestor: &KnowledgeIngestor,
    reader: R,
    count: &mut usize,
    limit: usize,
) -> Result<IngestionStats> {
    let mut stats = IngestionStats::new();
    
    for line in reader.lines() {
        if *count >= limit {
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
        
        // Skip comments and empty lines
        let line = line.trim();
        if line.is_empty() || line.starts_with("  ") {
            continue;
        }
        
        stats.entities_processed += 1;
        
        // Parse synset
        match parse_synset(&line, ingestor.config()) {
            Ok(Some(parsed)) => {
                // Generate embedding
                match generate_embedding(ingestor, &parsed).await {
                    Ok(embedding) => {
                        // Insert into database
                        match insert_entity(ingestor, parsed, embedding).await {
                            Ok(_) => {
                                stats.entities_inserted += 1;
                                stats.vectors_generated += 1;
                                *count += 1;
                                
                                if *count % 100 == 0 {
                                    debug!("Processed {} WordNet synsets", *count);
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
                debug!("Failed to parse synset: {}", e);
                stats.entities_skipped += 1;
            }
        }
    }
    
    Ok(stats)
}

/// Parse a single WordNet synset line
fn parse_synset(line: &str, config: &IngestionConfig) -> Result<Option<ParsedEntity>> {
    // WordNet data file format:
    // synset_offset lex_filenum ss_type w_cnt word lex_id [word lex_id...] p_cnt [ptr...] [frames...] | gloss
    
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 2 {
        return Ok(None);
    }
    
    let synset_data = parts[0].trim();
    let gloss = parts[1].trim();
    
    // Parse synset data
    let fields: Vec<&str> = synset_data.split_whitespace().collect();
    if fields.len() < 4 {
        return Ok(None);
    }
    
    let synset_offset = fields[0];
    let _lex_filenum = fields[1];
    let ss_type = fields[2];
    let w_cnt_hex = fields[3];
    
    // Parse word count
    let w_cnt = usize::from_str_radix(w_cnt_hex, 16)
        .context("Invalid word count")?;
    
    if fields.len() < 4 + (w_cnt * 2) {
        return Ok(None);
    }
    
    // Extract words (synonyms)
    let mut words = Vec::new();
    for i in 0..w_cnt {
        let word_idx = 4 + (i * 2);
        if word_idx < fields.len() {
            let word = fields[word_idx].replace('_', " ");
            words.push(word);
        }
    }
    
    if words.is_empty() {
        return Ok(None);
    }
    
    // Parse POS (part of speech)
    let pos = match ss_type {
        "n" => "noun",
        "v" => "verb",
        "a" | "s" => "adjective",
        "r" => "adverb",
        _ => "unknown",
    };
    
    // Parse gloss into definition and examples
    let (definition, examples) = parse_gloss(gloss);
    
    // Build synset ID
    let synset_id = format!("{}.{}.{}", words[0].replace(' ', "_"), pos.chars().next().unwrap(), synset_offset);
    
    // Get canonical name (first word)
    let canonical_name = words[0].clone();
    
    // Build properties JSONB
    let properties = serde_json::json!({
        "synset_id": synset_id,
        "synset_offset": synset_offset,
        "pos": pos,
        "words": words,
        "definition": definition,
        "examples": examples,
        "synonyms": words.clone(),
    });
    
    // Build embedding text
    let embedding_text = build_embedding_text(&canonical_name, &definition, &examples);
    
    let entity = ExternalKnowledgeEntity {
        id: None,
        source: KnowledgeSource::WordNet,
        entity_key: synset_id,
        canonical_name: canonical_name.to_lowercase(),
        lang: Some("en".to_string()),
        entity_type: Some("synset".to_string()),
        properties,
        confidence: 1.0,
        usage_count: 0,
        usage_decay: Some(1.0),
        last_accessed: None,
        created_at: None,
        dump_version: Some("wordnet-3.1".to_string()),
        toolchain: Some(format!("knowledge-ingestor-v{}", env!("CARGO_PKG_VERSION"))),
        license: Some("WordNet-3.1".to_string()),
    };
    
    Ok(Some(ParsedEntity {
        entity,
        embedding_text,
    }))
}

/// Parse gloss into definition and examples
fn parse_gloss(gloss: &str) -> (String, Vec<String>) {
    // Gloss format: "definition; example1; example2"
    // Examples are often in quotes
    
    let parts: Vec<&str> = gloss.split(';').collect();
    let definition = parts[0].trim().to_string();
    
    let mut examples = Vec::new();
    for part in parts.iter().skip(1) {
        let example = part.trim()
            .trim_matches('"')
            .trim()
            .to_string();
        if !example.is_empty() {
            examples.push(example);
        }
    }
    
    (definition, examples)
}

/// Build embedding text from canonical name, definition, and examples
fn build_embedding_text(canonical_name: &str, definition: &str, examples: &[String]) -> String {
    let mut text = format!("{}: {}", canonical_name, definition);
    
    if !examples.is_empty() {
        text.push_str(". Examples: ");
        text.push_str(&examples.join(". "));
    }
    
    text
}

/// Generate embedding for parsed entity
async fn generate_embedding(
    ingestor: &KnowledgeIngestor,
    parsed: &ParsedEntity,
) -> Result<Vec<f32>> {
    let embedding = ingestor
        .embedding_service()
        .generate_embedding(
            &parsed.embedding_text,
            embedding_service::ContentType::Knowledge,
            "wordnet",
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
    fn test_parse_gloss() {
        let gloss = "an organized body of related information; \"the database contains customer records\"";
        let (definition, examples) = parse_gloss(gloss);
        
        assert_eq!(definition, "an organized body of related information");
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0], "the database contains customer records");
    }

    #[test]
    fn test_build_embedding_text() {
        let text = build_embedding_text(
            "database",
            "an organized body of related information",
            &vec!["the database contains customer records".to_string()],
        );
        
        assert!(text.contains("database"));
        assert!(text.contains("organized body"));
        assert!(text.contains("Examples:"));
        assert!(text.contains("customer records"));
    }
}

