/**
 * Context Manager - Selects relevant workspace files for agents
 *
 * Intelligently selects and ranks files based on relevance to agent tasks,
 * considering file types, recency, size constraints, and task-specific keywords.
 * Security-focused: never exposes sensitive file content.
 *
 * @author @darianrosebrook
 */

import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient.js";
import { EmbeddingService } from "../embeddings/EmbeddingService.js";
import {
  ContextCriteria,
  FileMetadata,
  WorkspaceContext,
  WorkspaceSnapshot,
} from "./types/workspace-state.js";

export class ContextManager {
  private defaultCriteria: ContextCriteria;

  constructor(defaultCriteria?: Partial<ContextCriteria>) {
    this.defaultCriteria = {
      maxFiles: 20,
      maxSizeBytes: 1024 * 1024, // 1MB
      priorityExtensions: [".ts", ".js", ".json", ".md", ".txt"],
      excludeExtensions: [".log", ".tmp", ".temp", ".lock"],
      excludeDirectories: ["node_modules", "dist", "build", ".git", "coverage"],
      includeBinaryFiles: false,
      relevanceKeywords: [],
      recencyWeight: 0.3,
      ...defaultCriteria,
    };
  }

  /**
   * Generate semantic context for an agent using vector similarity
   */
  async generateSemanticContext(
    taskDescription: string,
    criteria: Partial<ContextCriteria> = {},
    embeddingService?: EmbeddingService,
    dbClient?: KnowledgeDatabaseClient
  ): Promise<WorkspaceContext> {
    if (!embeddingService || !dbClient) {
      // Fall back to traditional context generation if semantic search not available
      throw new Error(
        "Semantic search requires embedding service and database client"
      );
    }

    // Generate embedding for task description
    const taskEmbedding = await embeddingService.generateEmbedding(
      taskDescription
    );

    // Query for semantically similar workspace files
    const results = await dbClient.query(
      `
      SELECT * FROM hybrid_search(
        $1::vector(768),
        $2,
        $3,
        0, -- no graph hops for workspace files
        ARRAY['workspace_file']::VARCHAR(50)[],
        NULL,
        0.7
      )
    `,
      [`[${taskEmbedding.join(",")}]`, taskDescription, criteria.maxFiles || 20]
    );

    // Convert database results to workspace context
    return this.buildContextFromSemanticResults(
      results.rows,
      criteria,
      taskDescription
    );
  }

  /**
   * Generate context for an agent based on criteria
   */
  generateContext(
    snapshot: WorkspaceSnapshot,
    criteria: Partial<ContextCriteria> = {},
    agentId?: string,
    taskId?: string
  ): WorkspaceContext {
    const mergedCriteria = { ...this.defaultCriteria, ...criteria };

    // Filter eligible files
    const eligibleFiles = this.filterEligibleFiles(
      snapshot.files,
      mergedCriteria
    );

    // Score files by relevance
    const scoredFiles = this.scoreFilesByRelevance(
      eligibleFiles,
      mergedCriteria
    );

    // Sort by score (descending)
    scoredFiles.sort((a, b) => b.score - a.score);

    // Select top files within constraints
    const selectedFiles = this.selectFilesWithinConstraints(
      scoredFiles,
      mergedCriteria
    );

    // Calculate relevance scores map
    const relevanceScores = new Map<string, number>();
    for (const { file, score } of scoredFiles) {
      relevanceScores.set(file.path, score);
    }

    const context: WorkspaceContext = {
      files: selectedFiles.map((item) => item.file),
      totalSize: selectedFiles.reduce((sum, item) => sum + item.file.size, 0),
      criteria: mergedCriteria,
      relevanceScores,
      timestamp: new Date(),
      agentId,
      taskId,
    };

    return context;
  }

  /**
   * Filter files based on eligibility criteria
   */
  private filterEligibleFiles(
    files: FileMetadata[],
    criteria: ContextCriteria
  ): FileMetadata[] {
    return files.filter((file) => {
      // Check file size
      if (file.size > criteria.maxSizeBytes / criteria.maxFiles) {
        return false; // Too large for individual file
      }

      // Check extensions
      if (criteria.excludeExtensions.includes(file.extension)) {
        return false;
      }

      // Check directories
      const relativePath = file.relativePath;
      for (const excludedDir of criteria.excludeDirectories) {
        if (
          relativePath.startsWith(excludedDir + "/") ||
          relativePath.includes("/" + excludedDir + "/")
        ) {
          return false;
        }
      }

      // Check binary files
      if (!criteria.includeBinaryFiles && file.isBinary) {
        return false;
      }

      return true;
    });
  }

  /**
   * Score files by relevance to the task
   */
  private scoreFilesByRelevance(
    files: FileMetadata[],
    criteria: ContextCriteria
  ): Array<{ file: FileMetadata; score: number }> {
    const now = Date.now();
    const maxAge = 30 * 24 * 60 * 60 * 1000; // 30 days in milliseconds

    return files.map((file) => {
      let score = 0;

      // Extension priority score (0-0.4)
      const extensionIndex = criteria.priorityExtensions.indexOf(
        file.extension
      );
      if (extensionIndex >= 0) {
        score +=
          0.4 * (1 - extensionIndex / criteria.priorityExtensions.length);
      }

      // Recency score (0-0.3) - newer files get higher scores
      const age = now - file.mtime.getTime();
      const recencyScore =
        Math.max(0, (maxAge - age) / maxAge) * criteria.recencyWeight;
      score += recencyScore;

      // Keyword relevance score (0-0.3)
      if (criteria.relevanceKeywords.length > 0) {
        const keywordScore = this.calculateKeywordRelevance(
          file,
          criteria.relevanceKeywords
        );
        score += keywordScore * 0.3;
      }

      // Size penalty - prefer smaller files (slight preference)
      const sizePenalty =
        Math.min(1, file.size / (criteria.maxSizeBytes / criteria.maxFiles)) *
        0.1;
      score -= sizePenalty;

      return { file, score: Math.max(0, Math.min(1, score)) };
    });
  }

  /**
   * Calculate keyword relevance based on file path and metadata
   */
  private calculateKeywordRelevance(
    file: FileMetadata,
    keywords: string[]
  ): number {
    const searchableText = [
      file.relativePath.toLowerCase(),
      file.extension,
      file.mimeType || "",
    ].join(" ");

    let matches = 0;
    for (const keyword of keywords) {
      if (searchableText.includes(keyword.toLowerCase())) {
        matches++;
      }
    }

    return matches / keywords.length; // Normalize to 0-1
  }

  /**
   * Select files within size and count constraints
   */
  private selectFilesWithinConstraints(
    scoredFiles: Array<{ file: FileMetadata; score: number }>,
    criteria: ContextCriteria
  ): Array<{ file: FileMetadata; score: number }> {
    const selected: Array<{ file: FileMetadata; score: number }> = [];
    let totalSize = 0;

    for (const item of scoredFiles) {
      // Check count limit
      if (selected.length >= criteria.maxFiles) {
        break;
      }

      // Check size limit
      if (totalSize + item.file.size > criteria.maxSizeBytes) {
        continue; // Skip this file, but continue looking for smaller ones
      }

      selected.push(item);
      totalSize += item.file.size;
    }

    return selected;
  }

  /**
   * Generate context for code-related tasks
   */
  generateCodeContext(
    snapshot: WorkspaceSnapshot,
    language?: string,
    framework?: string,
    agentId?: string,
    taskId?: string
  ): WorkspaceContext {
    const keywords = [];

    if (language) keywords.push(language);
    if (framework) keywords.push(framework);

    // Add common development keywords
    keywords.push("src", "lib", "component", "service", "util", "helper");

    const criteria: Partial<ContextCriteria> = {
      priorityExtensions: this.getLanguageExtensions(language),
      relevanceKeywords: keywords,
      recencyWeight: 0.4, // Favor recent changes for code tasks
      maxFiles: 15,
      maxSizeBytes: 512 * 1024, // 512KB for code context
    };

    return this.generateContext(snapshot, criteria, agentId, taskId);
  }

  /**
   * Generate context for documentation tasks
   */
  generateDocumentationContext(
    snapshot: WorkspaceSnapshot,
    agentId?: string,
    taskId?: string
  ): WorkspaceContext {
    const criteria: Partial<ContextCriteria> = {
      priorityExtensions: [".md", ".txt", ".rst", ".adoc"],
      relevanceKeywords: ["readme", "doc", "guide", "tutorial", "api", "spec"],
      recencyWeight: 0.2, // Documentation changes less frequently
      maxFiles: 10,
      maxSizeBytes: 256 * 1024, // 256KB for docs
    };

    return this.generateContext(snapshot, criteria, agentId, taskId);
  }

  /**
   * Generate context for configuration tasks
   */
  generateConfigContext(
    snapshot: WorkspaceSnapshot,
    agentId?: string,
    taskId?: string
  ): WorkspaceContext {
    const criteria: Partial<ContextCriteria> = {
      priorityExtensions: [
        ".json",
        ".yaml",
        ".yml",
        ".toml",
        ".ini",
        ".cfg",
        ".conf",
      ],
      relevanceKeywords: ["config", "setting", "env", "package", "docker"],
      excludeDirectories: ["node_modules", "dist", "build", ".git"],
      recencyWeight: 0.1, // Config changes infrequently but are important
      maxFiles: 8,
      maxSizeBytes: 128 * 1024, // 128KB for configs
    };

    return this.generateContext(snapshot, criteria, agentId, taskId);
  }

  /**
   * Get file extensions for a programming language
   */
  private getLanguageExtensions(language?: string): string[] {
    const extensionMap: Record<string, string[]> = {
      typescript: [".ts", ".tsx", ".d.ts"],
      javascript: [".js", ".jsx", ".mjs", ".cjs"],
      python: [".py", ".pyx", ".pxd"],
      java: [".java", ".jar"],
      csharp: [".cs", ".csx"],
      cpp: [".cpp", ".cc", ".cxx", ".c++", ".hpp", ".hxx"],
      c: [".c", ".h"],
      go: [".go"],
      rust: [".rs"],
      php: [".php", ".phtml"],
      ruby: [".rb", ".erb"],
      swift: [".swift"],
      kotlin: [".kt", ".kts"],
      scala: [".scala"],
      clojure: [".clj", ".cljs"],
      haskell: [".hs", ".lhs"],
      erlang: [".erl", ".hrl"],
      elixir: [".ex", ".exs"],
    };

    return language
      ? extensionMap[language.toLowerCase()] || [".ts", ".js"]
      : [".ts", ".js"];
  }

  /**
   * Update default criteria
   */
  updateDefaultCriteria(criteria: Partial<ContextCriteria>): void {
    this.defaultCriteria = { ...this.defaultCriteria, ...criteria };
  }

  /**
   * Get current default criteria
   */
  getDefaultCriteria(): ContextCriteria {
    return { ...this.defaultCriteria };
  }

  /**
   * Analyze context effectiveness (for debugging/optimization)
   */
  analyzeContext(context: WorkspaceContext): {
    coverage: number;
    averageRelevance: number;
    sizeEfficiency: number;
    extensionDistribution: Record<string, number>;
  } {
    const totalFiles = context.files.length;
    const totalRelevance = Array.from(context.relevanceScores.values()).reduce(
      (sum, score) => sum + score,
      0
    );

    const extensionDistribution: Record<string, number> = {};
    for (const file of context.files) {
      extensionDistribution[file.extension] =
        (extensionDistribution[file.extension] || 0) + 1;
    }

    return {
      coverage: totalFiles / Math.max(1, context.criteria.maxFiles),
      averageRelevance: totalRelevance / Math.max(1, totalFiles),
      sizeEfficiency: context.totalSize / context.criteria.maxSizeBytes,
      extensionDistribution,
    };
  }

  /**
   * Build workspace context from semantic search results
   */
  private buildContextFromSemanticResults(
    semanticResults: any[],
    criteria: Partial<ContextCriteria>,
    taskDescription: string
  ): WorkspaceContext {
    const mergedCriteria = { ...this.defaultCriteria, ...criteria };
    const relevanceScores = new Map<string, number>();

    // Convert semantic results to file metadata
    const files: FileMetadata[] = [];
    let totalSize = 0;

    for (const result of semanticResults) {
      // Extract file path from result metadata
      const filePath = result.metadata?.file_path || result.name;
      const fileType =
        result.metadata?.file_type || filePath.split(".").pop() || "";

      // Create file metadata (we don't have full file info from semantic search)
      const fileMetadata: FileMetadata = {
        path: filePath,
        name: filePath.split("/").pop() || filePath,
        extension: fileType,
        size: result.metadata?.size || 0,
        lastModified: new Date(result.metadata?.last_modified || Date.now()),
        hash: result.metadata?.hash || "",
        isBinary: false, // Assume text files for semantic search
      };

      files.push(fileMetadata);
      totalSize += fileMetadata.size;

      // Use similarity score as relevance
      relevanceScores.set(
        filePath,
        result.relevance_score || result.similarity_score || 0.5
      );
    }

    return {
      files,
      criteria: mergedCriteria,
      totalSize,
      relevanceScores,
      generatedAt: new Date(),
      taskDescription,
      searchType: "semantic",
    };
  }
}
