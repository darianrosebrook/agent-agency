/**
 * @fileoverview Wikidata Lexeme Indexer
 *
 * Indexes Wikidata lexemes for semantic search integration.
 * Processes gzipped JSON lines and stores embeddings in existing schema.
 *
 * @author @darianrosebrook
 */

import { createReadStream } from "fs";
import { createInterface } from "readline";
import { createGunzip } from "zlib";
import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient.js";
import { EmbeddingService } from "../embeddings/EmbeddingService.js";
import { WikidataLexeme } from "../embeddings/types.js";

/**
 * Wikidata lexeme indexer implementation
 */
export class WikidataIndexer {
  private embeddingService: EmbeddingService;
  private dbClient: KnowledgeDatabaseClient;
  private batchSize: number;
  private maxRetries: number;

  constructor(
    embeddingService: EmbeddingService,
    dbClient: KnowledgeDatabaseClient,
    options: {
      batchSize?: number;
      maxRetries?: number;
    } = {}
  ) {
    this.embeddingService = embeddingService;
    this.dbClient = dbClient;
    this.batchSize = options.batchSize || 100;
    this.maxRetries = options.maxRetries || 3;
  }

  /**
   * Index Wikidata lexemes from gzipped file
   */
  async indexLexemes(gzipPath: string): Promise<void> {
    console.log(`Starting Wikidata lexeme indexing from: ${gzipPath}`);

    const startTime = Date.now();
    let processedCount = 0;
    let errorCount = 0;

    try {
      const stream = createReadStream(gzipPath).pipe(createGunzip());
      const rl = createInterface({
        input: stream,
        crlfDelay: Infinity,
      });

      let batch: WikidataLexeme[] = [];

      for await (const line of rl) {
        try {
          const lexeme = this.parseLexemeLine(line);
          if (this.isValidLexeme(lexeme)) {
            batch.push(lexeme);
          }

          if (batch.length >= this.batchSize) {
            const batchErrors = await this.processBatch(batch);
            processedCount += batch.length - batchErrors;
            errorCount += batchErrors;
            batch = [];

            // Log progress
            if (processedCount % 1000 === 0) {
              console.log(`Processed ${processedCount} lexemes...`);
            }
          }
        } catch (error) {
          console.warn(
            `Failed to parse line: ${
              error instanceof Error ? error.message : String(error)
            }`
          );
          errorCount++;
        }
      }

      // Process remaining batch
      if (batch.length > 0) {
        const batchErrors = await this.processBatch(batch);
        processedCount += batch.length - batchErrors;
        errorCount += batchErrors;
      }

      const duration = Date.now() - startTime;
      console.log(`Wikidata indexing complete:`);
      console.log(`- Processed: ${processedCount} lexemes`);
      console.log(`- Errors: ${errorCount}`);
      console.log(`- Duration: ${(duration / 1000).toFixed(1)}s`);
      console.log(
        `- Rate: ${(processedCount / (duration / 1000)).toFixed(1)} lexemes/sec`
      );
    } catch (error) {
      console.error(
        `Wikidata indexing failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
      throw error;
    }
  }

  /**
   * Parse a single JSON line into WikidataLexeme
   */
  private parseLexemeLine(line: string): WikidataLexeme {
    try {
      const data = JSON.parse(line.trim());

      // Extract relevant fields from Wikidata format
      return {
        id: data.id || data.lexeme,
        lemma: data.lemmas?.en?.value || data.lemma || "",
        language: data.language || data.lemmas?.en?.language || "en",
        lexicalCategory: data.lexicalCategory || data.category || "",
        forms: data.forms?.map((f: any) => f.representation || f.value) || [],
        senses:
          data.senses?.map((sense: any) => ({
            glosses: {
              en: sense.glosses?.en?.value || sense.gloss || "",
            },
            examples:
              sense.examples?.map((ex: any) => ex.text || ex.value) || [],
          })) || [],
      };
    } catch (error) {
      throw new Error(
        `Invalid JSON: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Validate lexeme has required fields
   */
  private isValidLexeme(lexeme: WikidataLexeme): boolean {
    return !!(
      lexeme.id &&
      lexeme.lemma &&
      lexeme.lemma.trim().length > 0 &&
      lexeme.language &&
      lexeme.lexicalCategory
    );
  }

  /**
   * Process a batch of lexemes
   */
  private async processBatch(lexemes: WikidataLexeme[]): Promise<number> {
    let errorCount = 0;

    try {
      // Generate texts for embedding
      const texts = lexemes.map((lexeme) => this.prepareLexemeText(lexeme));

      // Generate embeddings in batches
      const embeddings = await this.embeddingService.generateBatch(texts);

      // Prepare data for database insertion
      const insertData = lexemes
        .map((lexeme, i) => {
          const embedding = embeddings[i];
          if (!embedding) {
            errorCount++;
            return null;
          }

          return [
            "system", // agent_id
            this.normalizeLemma(lexeme.lemma), // capability_name/canonical_name
            `[${embedding.join(",")}]`, // embedding as string
            1.0, // initial confidence
            JSON.stringify({
              source: "wikidata",
              entity_id: lexeme.id,
              language: lexeme.language,
              lexical_category: lexeme.lexicalCategory,
              forms: lexeme.forms,
              sense_count: lexeme.senses?.length || 0,
            }), // metadata
          ];
        })
        .filter((data) => data !== null);

      if (insertData.length === 0) {
        return lexemes.length; // All failed
      }

      // Bulk insert with error handling
      await this.dbClient.bulkInsert(
        `
        INSERT INTO agent_capabilities_graph (
          agent_id, capability_type, capability_name, canonical_name,
          embedding, confidence, metadata
        ) VALUES ($1, 'EXTERNAL_KNOWLEDGE', $2, $2, $3, $4, $5)
        ON CONFLICT (agent_id, canonical_name)
        DO UPDATE SET
          embedding = EXCLUDED.embedding,
          confidence = GREATEST(agent_capabilities_graph.confidence, EXCLUDED.confidence),
          metadata = EXCLUDED.metadata,
          last_updated = NOW()
      `,
        insertData as any[]
      );
    } catch (error) {
      console.error(
        `Batch processing failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
      // Count all as errors if batch fails
      errorCount = lexemes.length;
    }

    return errorCount;
  }

  /**
   * Prepare text for embedding generation
   */
  private prepareLexemeText(lexeme: WikidataLexeme): string {
    const parts = [
      `Wikidata Lexeme: ${lexeme.lemma}`,
      `Language: ${lexeme.language}`,
      `Category: ${lexeme.lexicalCategory}`,
    ];

    if (lexeme.forms && lexeme.forms.length > 0) {
      parts.push(
        `Forms: ${lexeme.forms.slice(0, 5).join(", ")}${
          lexeme.forms.length > 5 ? "..." : ""
        }`
      );
    }

    if (lexeme.senses && lexeme.senses.length > 0) {
      const firstSense = lexeme.senses[0];
      if (firstSense.glosses?.en) {
        parts.push(`Definition: ${firstSense.glosses.en}`);
      }
      if (firstSense.examples && firstSense.examples.length > 0) {
        parts.push(`Example: ${firstSense.examples[0]}`);
      }
    }

    return parts.join("\n");
  }

  /**
   * Normalize lemma for consistent storage
   */
  private normalizeLemma(lemma: string): string {
    return lemma.toLowerCase().trim();
  }

  /**
   * Get indexing statistics
   */
  async getIndexingStats(): Promise<{
    totalLexemes: number;
    indexedLexemes: number;
    lastIndexed: Date | null;
    averageConfidence: number;
  }> {
    const result = await this.dbClient.query(`
      SELECT
        COUNT(*) as total_count,
        COUNT(*) FILTER (WHERE metadata->>'source' = 'wikidata') as wikidata_count,
        AVG(confidence) FILTER (WHERE metadata->>'source' = 'wikidata') as avg_confidence,
        MAX(last_updated) FILTER (WHERE metadata->>'source' = 'wikidata') as last_updated
      FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'source' = 'wikidata'
    `);

    const row = result.rows[0];
    return {
      totalLexemes: parseInt(row.total_count) || 0,
      indexedLexemes: parseInt(row.wikidata_count) || 0,
      lastIndexed: row.last_updated ? new Date(row.last_updated) : null,
      averageConfidence: parseFloat(row.avg_confidence) || 0,
    };
  }

  /**
   * Check if lexeme is already indexed
   */
  async isIndexed(lexemeId: string): Promise<boolean> {
    const result = await this.dbClient.query(
      `
      SELECT 1 FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'source' = 'wikidata'
        AND metadata->>'entity_id' = $1
      LIMIT 1
    `,
      [lexemeId]
    );

    return result.rows.length > 0;
  }

  /**
   * Clear all Wikidata index entries
   */
  async clearIndex(): Promise<void> {
    await this.dbClient.query(`
      DELETE FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'source' = 'wikidata'
    `);
  }
}
