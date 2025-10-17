#!/usr/bin/env tsx

/**
 * Knowledge Indexing Script
 *
 * Indexes external knowledge sources (Wikidata, WordNet) and workspace files
 * for semantic search integration.
 *
 * Usage:
 *   npm run index:knowledge -- wikidata
 *   npm run index:knowledge -- wordnet
 *   npm run index:knowledge -- workspace
 *   npm run index:knowledge -- all
 *
 * @author @darianrosebrook
 */

import { KnowledgeDatabaseClient } from "../src/database/KnowledgeDatabaseClient.js";
import { EmbeddingService } from "../src/embeddings/EmbeddingService.js";
import { WorkspaceEmbeddingService } from "../src/embeddings/WorkspaceEmbeddingService.js";
import { WikidataIndexer } from "../src/knowledge/WikidataIndexer.js";
import { WordNetIndexer } from "../src/knowledge/WordNetIndexer.js";

async function main() {
  const command = process.argv[2];

  if (!command) {
    console.log("Usage: index-knowledge.ts [wikidata|wordnet|workspace|all]");
    console.log("");
    console.log("Commands:");
    console.log(
      "  wikidata  - Index Wikidata lexemes from wikidata-20250924-lexemes.json.gz"
    );
    console.log("  wordnet   - Index Princeton WordNet from wn3.1.dict.tar.gz");
    console.log("  workspace - Index workspace files for semantic search");
    console.log("  all       - Run all indexing operations in sequence");
    process.exit(1);
  }

  console.log("🚀 Starting knowledge indexing...");

  // Initialize services
  const embeddingService = new EmbeddingService({
    ollamaEndpoint: process.env.OLLAMA_HOST || "http://localhost:11434",
    cacheSize: 5000, // Larger cache for batch operations
  });

  const dbClient = new KnowledgeDatabaseClient(/* config */);

  // Check embedding service availability
  console.log("🔍 Checking embedding service availability...");
  const embeddingAvailable = await embeddingService.isAvailable();
  if (!embeddingAvailable) {
    console.error(
      "❌ Embedding service not available. Make sure Ollama is running with embeddinggemma model."
    );
    process.exit(1);
  }
  console.log("✅ Embedding service ready");

  try {
    switch (command) {
      case "wikidata":
        await indexWikidata(embeddingService, dbClient);
        break;

      case "wordnet":
        await indexWordNet(embeddingService, dbClient);
        break;

      case "workspace":
        await indexWorkspace(embeddingService, dbClient);
        break;

      case "all":
        console.log("📚 Starting comprehensive indexing...");
        await indexWikidata(embeddingService, dbClient);
        await indexWordNet(embeddingService, dbClient);
        await indexWorkspace(embeddingService, dbClient);
        console.log("🎉 All indexing operations completed successfully!");
        break;

      default:
        console.error(`❌ Unknown command: ${command}`);
        process.exit(1);
    }
  } catch (error) {
    console.error("❌ Indexing failed:", error);
    process.exit(1);
  }
}

async function indexWikidata(
  embeddingService: EmbeddingService,
  dbClient: KnowledgeDatabaseClient
): Promise<void> {
  console.log("🌍 Starting Wikidata lexeme indexing...");

  const wikidataPath = "./wikidata-20250924-lexemes.json.gz";
  const indexer = new WikidataIndexer(embeddingService, dbClient, {
    batchSize: 100,
    maxRetries: 3,
  });

  // Check if already indexed
  const stats = await indexer.getIndexingStats();
  if (stats.indexedLexemes > 0) {
    console.log(
      `ℹ️  Wikidata already indexed: ${stats.indexedLexemes} lexemes`
    );
    const proceed = process.env.FORCE_REINDEX === "true";
    if (!proceed) {
      console.log("   Use FORCE_REINDEX=true to reindex");
      return;
    }
    console.log("🔄 Reindexing Wikidata...");
    await indexer.clearIndex();
  }

  await indexer.indexLexemes(wikidataPath);
  console.log("✅ Wikidata indexing completed");
}

async function indexWordNet(
  embeddingService: EmbeddingService,
  dbClient: KnowledgeDatabaseClient
): Promise<void> {
  console.log("📖 Starting WordNet dictionary indexing...");

  const wordnetPath = "./wn3.1.dict.tar.gz";
  const indexer = new WordNetIndexer(embeddingService, dbClient, {
    batchSize: 100,
    maxRetries: 3,
  });

  // Check if already indexed
  const stats = await indexer.getIndexingStats();
  if (stats.indexedSynsets > 0) {
    console.log(`ℹ️  WordNet already indexed: ${stats.indexedSynsets} synsets`);
    const proceed = process.env.FORCE_REINDEX === "true";
    if (!proceed) {
      console.log("   Use FORCE_REINDEX=true to reindex");
      return;
    }
    console.log("🔄 Reindexing WordNet...");
    await indexer.clearIndex();
  }

  await indexer.indexDictionary(wordnetPath);
  console.log("✅ WordNet indexing completed");
}

async function indexWorkspace(
  embeddingService: EmbeddingService,
  dbClient: KnowledgeDatabaseClient
): Promise<void> {
  console.log("📁 Starting workspace file indexing...");

  const workspaceService = new WorkspaceEmbeddingService(
    embeddingService,
    dbClient
  );
  const workspaceRoot = process.cwd();

  console.log(`Indexing workspace: ${workspaceRoot}`);
  await workspaceService.indexWorkspace(workspaceRoot);

  console.log("✅ Workspace indexing completed");
}

// Graceful error handling
process.on("SIGINT", () => {
  console.log("\n⚠️  Indexing interrupted by user");
  process.exit(0);
});

process.on("unhandledRejection", (error) => {
  console.error("❌ Unhandled rejection:", error);
  process.exit(1);
});

main().catch((error) => {
  console.error("💥 Fatal error:", error);
  process.exit(1);
});
