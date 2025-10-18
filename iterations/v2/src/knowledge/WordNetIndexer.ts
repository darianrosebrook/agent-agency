/**
 * @fileoverview WordNet Dictionary Indexer
 *
 * Indexes Princeton WordNet synsets for semantic search integration.
 * Processes WordNet dictionary files and stores embeddings in existing schema.
 *
 * @author @darianrosebrook
 */

import { createReadStream } from "fs";
import { join } from "path";
import { createInterface } from "readline";
import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient.js";
import { EmbeddingService } from "../embeddings/EmbeddingService.js";
import { WordNetSynset } from "../embeddings/types.js";

/**
 * WordNet dictionary indexer implementation
 */
export class WordNetIndexer {
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
   * Index WordNet dictionary from tar.gz file
   */
  async indexDictionary(tarGzPath: string): Promise<void> {
    console.log(`Starting WordNet dictionary indexing from: ${tarGzPath}`);

    const startTime = Date.now();
    let processedCount = 0;
    let errorCount = 0;

    try {
      // Extract tar.gz and process files
      const extractedPath = await this.extractTarGz(tarGzPath);
      const synsets = await this.parseDictionaryFiles(extractedPath);

      console.log(`Parsed ${synsets.length} synsets from WordNet dictionary`);

      // Process in batches
      for (let i = 0; i < synsets.length; i += this.batchSize) {
        const batch = synsets.slice(i, i + this.batchSize);
        const batchErrors = await this.processBatch(batch);
        processedCount += batch.length - batchErrors;
        errorCount += batchErrors;

        // Log progress
        if (processedCount % 1000 === 0) {
          console.log(`Processed ${processedCount} synsets...`);
        }
      }

      const duration = Date.now() - startTime;
      console.log(`WordNet indexing complete:`);
      console.log(`- Processed: ${processedCount} synsets`);
      console.log(`- Errors: ${errorCount}`);
      console.log(`- Duration: ${(duration / 1000).toFixed(1)}s`);
      console.log(
        `- Rate: ${(processedCount / (duration / 1000)).toFixed(1)} synsets/sec`
      );
    } catch (error) {
      console.error(
        `WordNet indexing failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
      throw error;
    }
  }

  /**
   * Extract tar.gz file to temporary directory
   */
  private async extractTarGz(tarGzPath: string): Promise<string> {
    const { exec } = await import("child_process");
    const { promisify } = await import("util");
    const execAsync = promisify(exec);
    const { mkdtemp } = await import("fs/promises");
    const { tmpdir } = await import("os");

    const tempDir = await mkdtemp(join(tmpdir(), "wordnet-"));

    try {
      // Extract tar.gz
      await execAsync(`tar -xzf "${tarGzPath}" -C "${tempDir}"`);
      console.log(`Extracted WordNet to: ${tempDir}`);
      return tempDir;
    } catch (error) {
      // Cleanup on error
      await execAsync(`rm -rf "${tempDir}"`);
      throw new Error(
        `Failed to extract tar.gz: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Parse WordNet dictionary files
   */
  private async parseDictionaryFiles(
    basePath: string
  ): Promise<WordNetSynset[]> {
    const synsets: WordNetSynset[] = [];

    // Process data.noun, data.verb, data.adj, data.adv files
    const posFiles = ["noun", "verb", "adj", "adv"];

    for (const pos of posFiles) {
      const filePath = join(basePath, "dict", `data.${pos}`);
      try {
        const fileSynsets = await this.parseDataFile(filePath, pos);
        synsets.push(...fileSynsets);
        console.log(`Parsed ${fileSynsets.length} ${pos} synsets`);
      } catch (error) {
        console.warn(
          `Failed to parse ${pos} file: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
      }
    }

    return synsets;
  }

  /**
   * Parse individual WordNet data file
   */
  private async parseDataFile(
    filePath: string,
    pos: string
  ): Promise<WordNetSynset[]> {
    const synsets: WordNetSynset[] = [];

    try {
      const stream = createReadStream(filePath, "utf8");
      const rl = createInterface({
        input: stream,
        crlfDelay: Infinity,
      });

      for await (const line of rl) {
        if (line.startsWith(" ") || line.trim().length === 0) {
          continue; // Skip comments and empty lines
        }

        try {
          const synset = this.parseSynsetLine(line, pos);
          if (synset) {
            synsets.push(synset);
          }
        } catch (error) {
          // Skip malformed lines
          continue;
        }
      }
    } catch (error) {
      console.warn(
        `Error reading file ${filePath}: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }

    return synsets;
  }

  /**
   * Parse a single synset line from WordNet data file
   */
  private parseSynsetLine(line: string, pos: string): WordNetSynset | null {
    try {
      // WordNet format: synset_offset lex_filenum ss_type w_cnt word lex_id [word lex_id...] p_cnt [ptr...] [frames...] | gloss
      const parts = line.split(" | ");
      if (parts.length !== 2) return null;

      const [definitionPart, glossPart] = parts;
      const defParts = definitionPart.split(" ");
      if (defParts.length < 4) return null;

      const synsetOffset = defParts[0];
      const _lexFilenum = defParts[1]; // Lexical file number (unused)
      const _ssType = defParts[2]; // Synset type (unused)
      const wCnt = parseInt(defParts[3], 16); // Hexadecimal word count

      // Extract words
      const words: string[] = [];
      for (let i = 0; i < wCnt; i++) {
        const wordIndex = 4 + i * 2;
        if (wordIndex < defParts.length) {
          let word = defParts[wordIndex];
          // Remove trailing markers
          word = word.replace(/[\d\(\)]+$/, "");
          words.push(word);
        }
      }

      // Extract gloss (definition and examples)
      const gloss = glossPart.trim();
      const [definition, examplesStr] = gloss.split("; ");
      const examples = examplesStr
        ? examplesStr
            .split("; ")
            .filter((ex) => ex.startsWith('"') && ex.endsWith('"'))
            .map((ex) => ex.slice(1, -1))
        : [];

      return {
        id: `${pos}-${synsetOffset}`,
        lemmas: words,
        definition: definition || "",
        examples,
        partOfSpeech: pos,
        gloss: glossPart,
      };
    } catch (error) {
      return null;
    }
  }

  /**
   * Process a batch of synsets
   */
  private async processBatch(synsets: WordNetSynset[]): Promise<number> {
    let errorCount = 0;

    try {
      // Generate texts for embedding
      const texts = synsets.map((synset) => this.prepareSynsetText(synset));

      // Generate embeddings in batches
      const embeddings = await this.embeddingService.generateBatch(texts);

      // Prepare data for database insertion
      const insertData = synsets
        .map((synset, i) => {
          const embedding = embeddings[i];
          if (!embedding) {
            errorCount++;
            return null;
          }

          // Use first lemma as canonical name
          const canonicalName = synset.lemmas[0]?.toLowerCase() || synset.id;

          return [
            "system", // agent_id
            canonicalName, // capability_name/canonical_name
            `[${embedding.join(",")}]`, // embedding as string
            1.0, // initial confidence
            JSON.stringify({
              source: "wordnet",
              entity_id: synset.id,
              part_of_speech: synset.partOfSpeech,
              lemmas: synset.lemmas,
              example_count: synset.examples.length,
              definition_length: synset.definition.length,
            }), // metadata
          ];
        })
        .filter((data) => data !== null);

      if (insertData.length === 0) {
        return synsets.length; // All failed
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
      errorCount = synsets.length;
    }

    return errorCount;
  }

  /**
   * Prepare text for embedding generation
   */
  private prepareSynsetText(synset: WordNetSynset): string {
    const parts = [
      `WordNet Synset: ${synset.lemmas.join(", ")}`,
      `Part of Speech: ${synset.partOfSpeech}`,
    ];

    if (synset.definition) {
      parts.push(`Definition: ${synset.definition}`);
    }

    if (synset.examples && synset.examples.length > 0) {
      parts.push(
        `Examples: ${synset.examples.slice(0, 2).join("; ")}${
          synset.examples.length > 2 ? "..." : ""
        }`
      );
    }

    return parts.join("\n");
  }

  /**
   * Get indexing statistics
   */
  async getIndexingStats(): Promise<{
    totalSynsets: number;
    indexedSynsets: number;
    lastIndexed: Date | null;
    averageConfidence: number;
  }> {
    const result = await this.dbClient.query(`
      SELECT
        COUNT(*) as total_count,
        COUNT(*) FILTER (WHERE metadata->>'source' = 'wordnet') as wordnet_count,
        AVG(confidence) FILTER (WHERE metadata->>'source' = 'wordnet') as avg_confidence,
        MAX(last_updated) FILTER (WHERE metadata->>'source' = 'wordnet') as last_updated
      FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'source' = 'wordnet'
    `);

    const row = result.rows[0];
    return {
      totalSynsets: parseInt(row.total_count) || 0,
      indexedSynsets: parseInt(row.wordnet_count) || 0,
      lastIndexed: row.last_updated ? new Date(row.last_updated) : null,
      averageConfidence: parseFloat(row.avg_confidence) || 0,
    };
  }

  /**
   * Check if synset is already indexed
   */
  async isIndexed(synsetId: string): Promise<boolean> {
    const result = await this.dbClient.query(
      `
      SELECT 1 FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'source' = 'wordnet'
        AND metadata->>'entity_id' = $1
      LIMIT 1
    `,
      [synsetId]
    );

    return result.rows.length > 0;
  }

  /**
   * Clear all WordNet index entries
   */
  async clearIndex(): Promise<void> {
    await this.dbClient.query(`
      DELETE FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'source' = 'wordnet'
    `);
  }
}
