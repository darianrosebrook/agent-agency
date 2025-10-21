//! CLI tool for loading core vocabulary from Wikidata and WordNet
//!
//! @author @darianrosebrook

use agent_agency_database as database;
use anyhow::Result;
use clap::Parser;
use knowledge_ingestor::*;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "load_core_vocabulary")]
#[command(about = "Load core vocabulary from Wikidata and WordNet into knowledge base")]
struct Args {
    /// Path to Wikidata lexemes dump (gzipped JSON)
    #[arg(long)]
    wikidata_path: PathBuf,
    
    /// Path to WordNet dictionary archive (tar.gz)
    #[arg(long)]
    wordnet_path: PathBuf,
    
    /// Maximum number of entities to ingest per source
    #[arg(long, default_value = "10000")]
    limit: usize,
    
    /// Preferred languages (comma-separated, e.g., "en,de,fr")
    #[arg(long, default_value = "en")]
    langs: String,
    
    /// Embedding model ID to use
    #[arg(long, default_value = "kb-text-default")]
    model_id: String,
    
    /// Database URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
    
    /// Skip Wikidata ingestion
    #[arg(long)]
    skip_wikidata: bool,
    
    /// Skip WordNet ingestion
    #[arg(long)]
    skip_wordnet: bool,
    
    /// Skip cross-reference generation
    #[arg(long)]
    skip_cross_ref: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    let args = Args::parse();
    
    info!("=== Knowledge Base Core Vocabulary Loader ===");
    info!("Wikidata path: {:?}", args.wikidata_path);
    info!("WordNet path: {:?}", args.wordnet_path);
    info!("Limit per source: {}", args.limit);
    info!("Languages: {}", args.langs);
    info!("Model ID: {}", args.model_id);
    
    // Parse languages
    let languages: Vec<String> = args
        .langs
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    // Initialize database client
    info!("Connecting to database...");
    let db_config = database::DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "agent_agency".to_string(),
        username: "postgres".to_string(),
        password: "password".to_string(),
        pool_min: 1,
        pool_max: 10,
        connection_timeout_seconds: 30,
        idle_timeout_seconds: 300,
        max_lifetime_seconds: 3600,
    };
    let db_client = Arc::new(
        database::DatabaseClient::new(db_config).await?
    );
    
    // Initialize embedding service
    info!("Initializing embedding service...");
    let embedding_config = embedding_service::EmbeddingConfig {
        dimension: 768,
        model_name: args.model_id.clone(),
        ..Default::default()
    };
    let embedding_service: Arc<dyn embedding_service::EmbeddingService> = Arc::from(
        embedding_service::EmbeddingServiceFactory::create_ollama_service(embedding_config)?
    );
    
    // Create ingestion config
    let config = IngestionConfig {
        limit: Some(args.limit),
        languages,
        model_id: args.model_id,
        min_confidence: 0.5,
        batch_size: 100,
        parallel: true,
    };
    
    // Create ingestor
    let ingestor = KnowledgeIngestor::new(
        db_client.clone(),
        embedding_service,
        config,
    );
    
    let mut total_stats = IngestionStats::new();
    
    // Ingest Wikidata (requires embeddings feature)
    #[cfg(feature = "embeddings")]
    if !args.skip_wikidata {
        info!("\n=== Ingesting Wikidata Lexemes ===");
        match wikidata::parse_wikidata_dump(&ingestor, &args.wikidata_path).await {
            Ok(stats) => {
                stats.print_summary();
                total_stats.merge(stats);
            }
            Err(e) => {
                eprintln!("Error ingesting Wikidata: {}", e);
                return Err(e);
            }
        }
    }

    #[cfg(not(feature = "embeddings"))]
    if !args.skip_wikidata {
        info!("Skipping Wikidata ingestion - embeddings feature not enabled");
    }

    // Ingest WordNet (requires embeddings feature)
    #[cfg(feature = "embeddings")]
    if !args.skip_wordnet {
        info!("\n=== Ingesting WordNet Synsets ===");
        match wordnet::parse_wordnet_dump(&ingestor, &args.wordnet_path).await {
            Ok(stats) => {
                stats.print_summary();
                total_stats.merge(stats);
            }
            Err(e) => {
                eprintln!("Error ingesting WordNet: {}", e);
                return Err(e);
            }
        }
    }
    
    // Generate cross-references
    if !args.skip_cross_ref {
        info!("\n=== Generating Cross-References ===");
        match cross_reference::generate_cross_references(&ingestor).await {
            Ok(stats) => {
                stats.print_summary();
                total_stats.merge(stats);
            }
            Err(e) => {
                eprintln!("Error generating cross-references: {}", e);
                return Err(e);
            }
        }
    }
    
    // Print final summary
    info!("\n=== Final Summary ===");
    total_stats.print_summary();
    
    // Get knowledge base statistics
    info!("\n=== Knowledge Base Statistics ===");
    match db_client.kb_get_stats().await {
        Ok(stats) => {
            for stat in stats {
                println!("Source: {}", stat.source);
                println!("  Total entities: {}", stat.total_entities);
                println!("  Total vectors: {}", stat.total_vectors);
                println!("  Total relationships: {}", stat.total_relationships);
                println!("  Average confidence: {:.2}", stat.avg_confidence);
                println!("  Average usage: {:.2}", stat.avg_usage_count);
                println!("  Last updated: {}", stat.last_updated);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error getting stats: {}", e);
        }
    }
    
    info!("=== Core Vocabulary Loading Complete ===");
    
    Ok(())
}

