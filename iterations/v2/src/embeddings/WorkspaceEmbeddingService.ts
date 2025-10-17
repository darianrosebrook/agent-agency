/**
 * @fileoverview Workspace Embedding Service
 *
 * Manages embedding generation and indexing for workspace files.
 * Provides batch processing and incremental updates for semantic search.
 *
 * @author @darianrosebrook
 */

import { readFile, readdir, stat } from "fs/promises";
import { extname, join, relative } from "path";
import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient.js";
import { EmbeddingService } from "./EmbeddingService.js";

export class WorkspaceEmbeddingService {
  private embeddingService: EmbeddingService;
  private dbClient: KnowledgeDatabaseClient;
  private supportedExtensions: Set<string>;

  constructor(
    embeddingService: EmbeddingService,
    dbClient: KnowledgeDatabaseClient
  ) {
    this.embeddingService = embeddingService;
    this.dbClient = dbClient;

    // Supported file types for embedding
    this.supportedExtensions = new Set([
      ".ts",
      ".js",
      ".tsx",
      ".jsx",
      ".py",
      ".java",
      ".cpp",
      ".c",
      ".h",
      ".hpp",
      ".md",
      ".txt",
      ".json",
      ".yaml",
      ".yml",
      ".rs",
      ".go",
      ".php",
      ".rb",
      ".scala",
      ".kt",
      ".swift",
    ]);
  }

  /**
   * Index all supported files in a workspace
   */
  async indexWorkspace(workspaceRoot: string): Promise<void> {
    console.log(`🔍 Scanning workspace: ${workspaceRoot}`);

    const files = await this.scanWorkspaceFiles(workspaceRoot);
    console.log(`📁 Found ${files.length} supported files`);

    if (files.length === 0) {
      console.log("ℹ️  No supported files found");
      return;
    }

    // Process in batches to avoid overwhelming the embedding service
    const batchSize = 50;
    let processed = 0;
    let errors = 0;

    for (let i = 0; i < files.length; i += batchSize) {
      const batch = files.slice(i, i + batchSize);
      console.log(
        `📦 Processing batch ${Math.floor(i / batchSize) + 1}/${Math.ceil(
          files.length / batchSize
        )} (${batch.length} files)`
      );

      const batchErrors = await this.processBatch(batch);
      processed += batch.length - batchErrors;
      errors += batchErrors;

      // Progress update
      console.log(
        `   ✅ ${processed}/${files.length} files processed (${errors} errors)`
      );
    }

    console.log(
      `🎉 Workspace indexing complete: ${processed} files indexed, ${errors} errors`
    );
  }

  /**
   * Update embedding for a single file
   */
  async updateFileEmbedding(filePath: string): Promise<void> {
    try {
      const content = await readFile(filePath, "utf-8");
      const relativePath = relative(process.cwd(), filePath);

      // Prepare text for embedding
      const textForEmbedding = this.prepareFileText(relativePath, content);

      // Generate embedding
      const embedding = await this.embeddingService.generateEmbedding(
        textForEmbedding
      );

      // Store in database
      await this.storeFileEmbedding(relativePath, embedding, content);
    } catch (error) {
      console.warn(`Failed to update embedding for ${filePath}:`, error);
      throw error;
    }
  }

  /**
   * Check if a file is already indexed
   */
  async isFileIndexed(filePath: string): Promise<boolean> {
    const relativePath = relative(process.cwd(), filePath);
    const result = await this.dbClient.query(
      `
      SELECT 1 FROM agent_capabilities_graph
      WHERE capability_type = 'TECHNOLOGY'
        AND metadata->>'source' = 'workspace_file'
        AND metadata->>'file_path' = $1
        AND validation_status = 'validated'
      LIMIT 1
    `,
      [relativePath]
    );

    return result.rows.length > 0;
  }

  /**
   * Get indexing statistics for workspace
   */
  async getIndexingStats(): Promise<{
    totalFiles: number;
    indexedFiles: number;
    lastIndexed: Date | null;
    averageConfidence: number;
  }> {
    const result = await this.dbClient.query(`
      SELECT
        COUNT(*) as total_count,
        COUNT(*) FILTER (WHERE validation_status = 'validated') as indexed_count,
        AVG(confidence) FILTER (WHERE validation_status = 'validated') as avg_confidence,
        MAX(last_updated) FILTER (WHERE validation_status = 'validated') as last_updated
      FROM agent_capabilities_graph
      WHERE capability_type = 'TECHNOLOGY'
        AND metadata->>'source' = 'workspace_file'
    `);

    const row = result.rows[0];
    return {
      totalFiles: parseInt(row.total_count) || 0,
      indexedFiles: parseInt(row.indexed_count) || 0,
      lastIndexed: row.last_updated ? new Date(row.last_updated) : null,
      averageConfidence: parseFloat(row.avg_confidence) || 0,
    };
  }

  /**
   * Clear all workspace file embeddings
   */
  async clearIndex(): Promise<void> {
    await this.dbClient.query(`
      DELETE FROM agent_capabilities_graph
      WHERE capability_type = 'TECHNOLOGY'
        AND metadata->>'source' = 'workspace_file'
    `);
  }

  /**
   * Private helper methods
   */

  private async scanWorkspaceFiles(workspaceRoot: string): Promise<string[]> {
    const files: string[] = [];
    const excludeDirs = new Set([
      "node_modules",
      ".git",
      "dist",
      "build",
      "coverage",
      ".next",
      ".nuxt",
      ".vuepress",
      ".cache",
      ".temp",
    ]);

    const scanDir = async (dir: string): Promise<void> => {
      const entries = await readdir(dir, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = join(dir, entry.name);

        if (entry.isDirectory()) {
          // Skip excluded directories
          if (excludeDirs.has(entry.name)) {
            continue;
          }

          // Skip hidden directories
          if (entry.name.startsWith(".")) {
            continue;
          }

          await scanDir(fullPath);
        } else if (entry.isFile()) {
          // Check if file extension is supported
          const ext = extname(entry.name).toLowerCase();
          if (this.supportedExtensions.has(ext)) {
            try {
              const stats = await stat(fullPath);
              // Skip very large files (> 1MB)
              if (stats.size <= 1024 * 1024) {
                files.push(fullPath);
              }
            } catch (error) {
              // Skip files we can't stat
              continue;
            }
          }
        }
      }
    };

    await scanDir(workspaceRoot);
    return files;
  }

  private async processBatch(filePaths: string[]): Promise<number> {
    let errorCount = 0;

    try {
      // Read file contents
      const fileData: Array<{
        path: string;
        content: string;
        relativePath: string;
      }> = [];

      for (const filePath of filePaths) {
        try {
          const content = await readFile(filePath, "utf-8");
          const relativePath = relative(process.cwd(), filePath);
          fileData.push({ path: filePath, content, relativePath });
        } catch (error) {
          console.warn(`Failed to read ${filePath}:`, error);
          errorCount++;
        }
      }

      if (fileData.length === 0) {
        return errorCount;
      }

      // Prepare texts for embedding
      const texts = fileData.map(({ relativePath, content }) =>
        this.prepareFileText(relativePath, content)
      );

      // Generate embeddings in batch
      const embeddings = await this.embeddingService.generateBatch(texts);

      // Store embeddings
      for (let i = 0; i < fileData.length; i++) {
        const embedding = embeddings[i];
        if (embedding) {
          try {
            await this.storeFileEmbedding(
              fileData[i].relativePath,
              embedding,
              fileData[i].content
            );
          } catch (error) {
            console.warn(
              `Failed to store embedding for ${fileData[i].path}:`,
              error
            );
            errorCount++;
          }
        } else {
          errorCount++;
        }
      }
    } catch (error) {
      console.error("Batch processing failed:", error);
      errorCount = filePaths.length;
    }

    return errorCount;
  }

  private prepareFileText(filePath: string, content: string): string {
    const fileName = filePath.split("/").pop() || filePath;
    const extension = extname(filePath);

    // Get file type context
    const contexts: Record<string, string> = {
      ".ts": "TypeScript source code file",
      ".js": "JavaScript source code file",
      ".tsx": "TypeScript React component file",
      ".jsx": "JavaScript React component file",
      ".py": "Python source code file",
      ".java": "Java source code file",
      ".cpp": "C++ source code file",
      ".c": "C source code file",
      ".h": "C/C++ header file",
      ".hpp": "C++ header file",
      ".md": "Markdown documentation file",
      ".txt": "Plain text file",
      ".json": "JSON data file",
      ".yaml": "YAML configuration file",
      ".yml": "YAML configuration file",
      ".rs": "Rust source code file",
      ".go": "Go source code file",
      ".php": "PHP source code file",
      ".rb": "Ruby source code file",
      ".scala": "Scala source code file",
      ".kt": "Kotlin source code file",
      ".swift": "Swift source code file",
    };

    const context = contexts[extension] || "Source code file";

    // Limit content size to avoid token limits
    const maxContentLength = 8000; // Leave room for context and metadata
    const truncatedContent =
      content.length > maxContentLength
        ? content.substring(0, maxContentLength) + "..."
        : content;

    return `${context}\n\nFile: ${fileName}\nPath: ${filePath}\n\nContent:\n${truncatedContent}`;
  }

  private async storeFileEmbedding(
    filePath: string,
    embedding: number[],
    content: string
  ): Promise<void> {
    // Calculate content hash for change detection
    const crypto = await import("crypto");
    const contentHash = crypto
      .createHash("sha256")
      .update(content)
      .digest("hex");

    await this.dbClient.query(
      `
      INSERT INTO agent_capabilities_graph (
        agent_id, capability_type, capability_name, canonical_name,
        embedding, confidence, metadata
      ) VALUES ($1, 'TECHNOLOGY', $2, $2, $3, 1.0, $4)
      ON CONFLICT (agent_id, canonical_name)
      DO UPDATE SET
        embedding = EXCLUDED.embedding,
        confidence = EXCLUDED.confidence,
        metadata = EXCLUDED.metadata,
        last_updated = NOW()
    `,
      [
        "system",
        filePath,
        `[${embedding.join(",")}]`,
        JSON.stringify({
          source: "workspace_file",
          file_path: filePath,
          file_type: extname(filePath),
          content_hash: contentHash,
          last_modified: new Date().toISOString(),
          indexed_at: new Date().toISOString(),
        }),
      ]
    );
  }
}
