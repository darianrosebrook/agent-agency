/**
 * Context Offloader - Efficient LLM context management and retrieval
 *
 * This component addresses the fundamental limitations of LLM context windows by
 * offloading context information to persistent storage and retrieving only the
 * most relevant data when needed. This prevents "context rot" and enables
 * virtually unlimited memory depth.
 *
 * @author @darianrosebrook
 */

import type {
  ContextOffloadingConfig,
  ContextualMemory,
  OffloadedContext,
  ReconstructedContext,
  SummarizedContext,
  TaskContext,
} from "../types/index.js";
import { Logger } from "../utils/Logger";

interface ContextAnalysis {
  complexity: number;
  size: number;
  entities: string[];
  relationships: string[];
  temporalPatterns: TemporalPattern[];
  compressionPotential: number;
}

interface TemporalPattern {
  pattern: string;
  frequency: number;
  recency: number;
  importance: number;
}

/**
 * ContextOffloader - Manages efficient context storage and retrieval
 */
export class ContextOffloader {
  private logger: Logger;
  private config: ContextOffloadingConfig;
  private contextStore: Map<string, OffloadedContext> = new Map();
  private summarizer: ContextSummarizer;
  private quarantineEngine: ContextQuarantineEngine;

  constructor(config: ContextOffloadingConfig, logger?: Logger) {
    this.config = config;
    this.logger = logger || new Logger("ContextOffloader");
    this.summarizer = new ContextSummarizer();
    this.quarantineEngine = new ContextQuarantineEngine();
  }

  /**
   * Offload context to external storage
   */
  async offloadContext(
    context: TaskContext,
    tenantId: string
  ): Promise<OffloadedContext> {
    this.logger.debug(`Offloading context for task ${context.taskId}`, {
      tenantId,
    });

    // Analyze context complexity
    const analysis = await this.analyzeContext(context);

    // Determine if offloading is needed
    if (!this.shouldOffload(analysis)) {
      throw new Error("Context does not meet offloading criteria");
    }

    // Apply context quarantine if enabled
    const quarantinedContext = this.config.quarantineEnabled
      ? await this.quarantineEngine.quarantineContext(context, tenantId)
      : context;

    // Summarize context if enabled
    const summarizedContext = this.config.summarizationEnabled
      ? await this.summarizer.summarizeContext(quarantinedContext, analysis)
      : this.createMinimalSummary(quarantinedContext);

    // Generate embedding for semantic retrieval
    const contextEmbedding = await this.generateContextEmbedding(
      summarizedContext
    );

    // Create offloaded context record
    const offloadedContext: OffloadedContext = {
      id: `offloaded_${context.taskId}_${Date.now()}`,
      tenantId,
      originalContext: context,
      summarizedContext,
      embedding: contextEmbedding,
      compressionRatio: this.calculateCompressionRatio(
        context,
        summarizedContext
      ),
      retrievalMetadata: {
        relevanceThreshold: this.config.relevanceThreshold,
        retrievalStrategy: this.determineRetrievalStrategy(analysis),
        contextQuarantine: this.config.quarantineEnabled,
        summarizationApplied: this.config.summarizationEnabled,
        expectedRetrievalTime: this.estimateRetrievalTime(analysis),
      },
      createdAt: new Date(),
      accessCount: 0,
    };

    // Store offloaded context
    this.contextStore.set(offloadedContext.id, offloadedContext);

    this.logger.info(`Context offloaded successfully`, {
      contextId: offloadedContext.id,
      tenantId,
      compressionRatio: offloadedContext.compressionRatio,
    });

    return offloadedContext;
  }

  /**
   * Retrieve and reconstruct context
   */
  async retrieveContext(
    contextId: string,
    tenantId: string,
    queryContext?: TaskContext
  ): Promise<ReconstructedContext> {
    const offloadedContext = this.contextStore.get(contextId);

    if (!offloadedContext) {
      return {
        context: null,
        relevanceScore: 0,
        reconstructionMethod: "direct",
        confidence: 0,
        metadata: {
          relevanceThreshold: this.config.relevanceThreshold,
          retrievalStrategy: "semantic",
          contextQuarantine: false,
          summarizationApplied: false,
          expectedRetrievalTime: 0,
        },
      };
    }

    // Verify tenant access
    if (offloadedContext.tenantId !== tenantId) {
      this.logger.warn(`Tenant access denied for context ${contextId}`, {
        tenantId,
        contextTenant: offloadedContext.tenantId,
      });
      return {
        context: null,
        relevanceScore: 0,
        reconstructionMethod: "direct",
        confidence: 0,
        metadata: offloadedContext.retrievalMetadata,
      };
    }

    // Calculate relevance if query context provided
    const relevanceScore = queryContext
      ? await this.calculateContextRelevance(offloadedContext, queryContext)
      : 1.0;

    // Check if context meets relevance threshold
    if (relevanceScore < this.config.relevanceThreshold) {
      this.logger.debug(`Context ${contextId} below relevance threshold`, {
        relevanceScore,
        threshold: this.config.relevanceThreshold,
      });
      return {
        context: null,
        relevanceScore,
        reconstructionMethod: "direct",
        confidence: 0,
        metadata: offloadedContext.retrievalMetadata,
      };
    }

    // Reconstruct context
    const reconstructedContext = await this.reconstructContext(
      offloadedContext,
      queryContext
    );

    // Update access statistics
    offloadedContext.lastAccessed = new Date();
    offloadedContext.accessCount++;
    this.contextStore.set(contextId, offloadedContext);

    this.logger.debug(`Context retrieved successfully`, {
      contextId,
      relevanceScore,
      reconstructionMethod: reconstructedContext.reconstructionMethod,
    });

    return reconstructedContext;
  }

  /**
   * Find relevant offloaded contexts for a query
   */
  async findRelevantContexts(
    tenantId: string,
    queryContext: TaskContext,
    limit: number = 10
  ): Promise<OffloadedContext[]> {
    const relevantContexts: Array<{
      context: OffloadedContext;
      score: number;
    }> = [];

    for (const [_contextId, context] of this.contextStore.entries()) {
      if (context.tenantId !== tenantId) continue;

      const relevanceScore = await this.calculateContextRelevance(
        context,
        queryContext
      );
      if (relevanceScore >= this.config.relevanceThreshold) {
        relevantContexts.push({ context, score: relevanceScore });
      }
    }

    // Sort by relevance and return top results
    return relevantContexts
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
      .map((item) => item.context);
  }

  /**
   * Enrich memories with offloaded context
   */
  async enrichMemories(
    memories: ContextualMemory[],
    tenantId: string
  ): Promise<ContextualMemory[]> {
    const enrichedMemories: ContextualMemory[] = [];

    for (const memory of memories) {
      // Find relevant offloaded contexts
      const relevantContexts = await this.findRelevantContexts(
        tenantId,
        {
          taskId: "enrichment",
          agentId: "",
          type: "enrichment",
          description: "",
          requirements: [],
          constraints: {},
          metadata: {},
        },
        3
      );

      // Enhance memory with context
      const enhancedMemory = await this.enhanceMemoryWithContext(
        memory,
        relevantContexts
      );
      enrichedMemories.push(enhancedMemory);
    }

    return enrichedMemories;
  }

  /**
   * Clean up expired or irrelevant contexts
   */
  async cleanupContexts(tenantId: string): Promise<number> {
    let cleanedCount = 0;
    const now = Date.now();
    const maxAge = 30 * 24 * 60 * 60 * 1000; // 30 days

    for (const [contextId, context] of this.contextStore.entries()) {
      if (context.tenantId !== tenantId) continue;

      const age = now - context.createdAt.getTime();
      const isExpired = age > maxAge;
      const isUnused = !context.lastAccessed && context.accessCount === 0;

      if (isExpired || isUnused) {
        this.contextStore.delete(contextId);
        cleanedCount++;
      }
    }

    this.logger.info(
      `Cleaned up ${cleanedCount} contexts for tenant ${tenantId}`
    );
    return cleanedCount;
  }

  // Private helper methods

  private async analyzeContext(context: TaskContext): Promise<ContextAnalysis> {
    // Analyze context complexity and structure
    const content = JSON.stringify(context);
    const size = content.length;

    // TODO: Implement comprehensive entity and relationship extraction
    // - Use NLP libraries for named entity recognition (spaCy, Stanford NER)
    // - Implement relationship extraction algorithms (OpenIE, dependency parsing)
    // - Add entity disambiguation and coreference resolution
    // - Support multiple entity types (persons, organizations, locations, concepts)
    // - Implement confidence scoring for extracted entities and relationships
    // - Add entity linking to knowledge bases (Wikidata, DBPedia)
    // - Support multi-language entity extraction
    // - Implement entity relationship graph construction and analysis
    const entities = this.extractEntities(context);
    const relationships = this.extractRelationships(context);

    // Calculate complexity score
    const complexity = Math.min(
      1.0,
      entities.length * 0.1 + relationships.length * 0.05 + size / 10000
    );

    // Estimate compression potential
    const compressionPotential = Math.max(0.1, Math.min(0.9, complexity * 0.8));

    return {
      complexity,
      size,
      entities,
      relationships,
      temporalPatterns: [], // Would implement temporal pattern analysis
      compressionPotential,
    };
  }

  private shouldOffload(analysis: ContextAnalysis): boolean {
    return (
      analysis.size > this.config.maxContextSize ||
      analysis.complexity > this.config.compressionThreshold
    );
  }

  private extractEntities(context: TaskContext): string[] {
    const entities: string[] = [];

    // Extract from requirements
    if (context.requirements) {
      entities.push(...context.requirements);
    }

    // Extract from constraints keys
    if (context.constraints) {
      entities.push(...Object.keys(context.constraints));
    }

    // Extract from metadata
    if (context.metadata?.entities) {
      entities.push(...(context.metadata.entities as string[]));
    }

    return [...new Set(entities)]; // Remove duplicates
  }

  private extractRelationships(context: TaskContext): string[] {
    // Simplified relationship extraction
    const relationships: string[] = [];

    if (context.metadata?.relationships) {
      relationships.push(...(context.metadata.relationships as string[]));
    }

    return relationships;
  }

  private createMinimalSummary(context: TaskContext): SummarizedContext {
    return {
      coreTask: context.description || "Unknown task",
      keyRequirements: context.requirements?.slice(0, 5) || [],
      criticalConstraints: context.constraints || {},
      essentialEntities: this.extractEntities(context).slice(0, 10),
      summary: `Task: ${context.description || "Unknown task"}. Requirements: ${
        context.requirements?.join(", ") || "None"
      }`,
      compressionLevel: "minimal",
    };
  }

  private async generateContextEmbedding(
    summarizedContext: SummarizedContext
  ): Promise<number[]> {
    // Placeholder for embedding generation
    // In real implementation, this would use an embedding service like Ollama
    const _text = `${
      summarizedContext.coreTask
    } ${summarizedContext.keyRequirements.join(" ")} ${
      summarizedContext.summary
    }`;
    return new Array(this.config.embeddingDimensions)
      .fill(0)
      .map(() => Math.random());
  }

  private calculateCompressionRatio(
    original: TaskContext,
    summarized: SummarizedContext
  ): number {
    const originalSize = JSON.stringify(original).length;
    const summarizedSize = JSON.stringify(summarized).length;
    return summarizedSize / originalSize;
  }

  private determineRetrievalStrategy(
    analysis: ContextAnalysis
  ): "semantic" | "temporal" | "hybrid" {
    if (analysis.complexity > 0.7) return "hybrid";
    if (analysis.temporalPatterns.length > 0) return "temporal";
    return "semantic";
  }

  private estimateRetrievalTime(analysis: ContextAnalysis): number {
    // Estimate retrieval time in milliseconds
    return Math.max(50, analysis.complexity * 200);
  }

  private async calculateContextRelevance(
    offloadedContext: OffloadedContext,
    queryContext: TaskContext
  ): Promise<number> {
    // Simplified relevance calculation
    const queryText = `${queryContext.description || ""} ${
      queryContext.requirements?.join(" ") || ""
    }`;
    const contextText = offloadedContext.summarizedContext.summary;

    // Simple text similarity (would use embeddings in real implementation)
    const queryWords = new Set(queryText.toLowerCase().split(/\s+/));
    const contextWords = new Set(contextText.toLowerCase().split(/\s+/));

    const intersection = new Set(
      [...queryWords].filter((x) => contextWords.has(x))
    );
    const union = new Set([...queryWords, ...contextWords]);

    return intersection.size / union.size;
  }

  private async reconstructContext(
    offloadedContext: OffloadedContext,
    queryContext?: TaskContext
  ): Promise<ReconstructedContext> {
    // Determine reconstruction method
    let reconstructionMethod: "direct" | "summarized" | "hybrid" = "summarized";
    let confidence = 0.8;

    if (queryContext) {
      const relevanceScore = await this.calculateContextRelevance(
        offloadedContext,
        queryContext
      );
      reconstructionMethod =
        relevanceScore > 0.8
          ? "direct"
          : relevanceScore > 0.5
          ? "hybrid"
          : "summarized";
      confidence = relevanceScore;
    }

    // Reconstruct context based on method
    let reconstructedContext: TaskContext;

    switch (reconstructionMethod) {
      case "direct":
        reconstructedContext = offloadedContext.originalContext;
        break;
      case "summarized":
        reconstructedContext = this.contextFromSummary(
          offloadedContext.summarizedContext
        );
        break;
      case "hybrid":
        reconstructedContext = this.hybridReconstruction(
          offloadedContext,
          queryContext!
        );
        break;
    }

    return {
      context: reconstructedContext,
      relevanceScore: confidence,
      reconstructionMethod,
      confidence,
      metadata: offloadedContext.retrievalMetadata,
    };
  }

  private contextFromSummary(summary: SummarizedContext): TaskContext {
    return {
      taskId: "reconstructed",
      agentId: "context-offloader",
      type: "reconstructed",
      description: summary.coreTask,
      requirements: summary.keyRequirements,
      constraints: summary.criticalConstraints,
      metadata: {
        reconstructionMethod: "summarized",
        compressionLevel: summary.compressionLevel,
      },
    };
  }

  private hybridReconstruction(
    offloadedContext: OffloadedContext,
    queryContext: TaskContext
  ): TaskContext {
    // Combine original context with query-specific enhancements
    return {
      ...offloadedContext.originalContext,
      description: `${offloadedContext.originalContext.description} (enhanced for: ${queryContext.description})`,
      metadata: {
        ...offloadedContext.originalContext.metadata,
        reconstructionMethod: "hybrid",
        queryContext: queryContext.taskId,
      },
    };
  }

  private async enhanceMemoryWithContext(
    memory: ContextualMemory,
    contexts: OffloadedContext[]
  ): Promise<ContextualMemory> {
    // Enhance memory with relevant offloaded contexts
    const enhancements = contexts.map((ctx) => ctx.summarizedContext.summary);
    const enhancedContent = {
      ...memory.content,
      contextEnhancements: enhancements,
      enhancedAt: new Date(),
    };

    return {
      ...memory,
      content: enhancedContent,
      relevanceScore: Math.min(1.0, memory.relevanceScore * 1.2), // Boost relevance
    };
  }
}

/**
 * Context Summarizer - Intelligent context summarization
 */
export class ContextSummarizer {
  async summarizeContext(
    context: TaskContext,
    analysis: ContextAnalysis
  ): Promise<SummarizedContext> {
    // Determine compression level based on analysis
    const compressionLevel =
      analysis.compressionPotential > 0.7
        ? "aggressive"
        : analysis.compressionPotential > 0.4
        ? "moderate"
        : "minimal";

    // Extract core elements
    const coreTask = this.extractCoreTask(context);
    const keyRequirements = this.extractKeyRequirements(
      context,
      compressionLevel
    );
    const criticalConstraints = this.extractCriticalConstraints(
      context,
      compressionLevel
    );
    const essentialEntities = analysis.entities.slice(
      0,
      compressionLevel === "aggressive"
        ? 5
        : compressionLevel === "moderate"
        ? 10
        : 20
    );

    // Generate summary
    const summary = this.generateSummary(context, analysis, compressionLevel);

    return {
      coreTask,
      keyRequirements,
      criticalConstraints,
      essentialEntities,
      summary,
      compressionLevel,
    };
  }

  private extractCoreTask(context: TaskContext): string {
    const description = context.description || "";
    return description.length > 100
      ? `${description.substring(0, 100)}...`
      : description;
  }

  private extractKeyRequirements(
    context: TaskContext,
    level: string
  ): string[] {
    const maxRequirements =
      level === "aggressive" ? 3 : level === "moderate" ? 5 : 10;
    return context.requirements?.slice(0, maxRequirements) || [];
  }

  private extractCriticalConstraints(
    context: TaskContext,
    level: string
  ): Record<string, any> {
    if (level === "aggressive") {
      // Only keep most critical constraints
      const constraints = context.constraints || {};
      const criticalKeys = Object.keys(constraints).slice(0, 3);
      return criticalKeys.reduce((acc, key) => {
        acc[key] = constraints[key];
        return acc;
      }, {} as Record<string, any>);
    }
    return context.constraints || {};
  }

  private generateSummary(
    context: TaskContext,
    analysis: ContextAnalysis,
    level: string
  ): string {
    const parts = [
      `Task: ${context.description || "Unknown task"}`,
      `Requirements: ${context.requirements?.slice(0, 5).join(", ") || "None"}`,
      `Complexity: ${analysis.complexity.toFixed(2)}`,
    ];

    if (level !== "aggressive") {
      parts.push(`Entities: ${analysis.entities.slice(0, 5).join(", ")}`);
    }

    return parts.join(". ");
  }
}

/**
 * Context Quarantine Engine - Isolates sub-tasks for focused processing
 */
export class ContextQuarantineEngine {
  async quarantineContext(
    context: TaskContext,
    tenantId: string
  ): Promise<TaskContext> {
    // Create isolated sub-contexts for different aspects of the task
    const quarantinedContext = {
      ...context,
      metadata: {
        ...context.metadata,
        quarantined: true,
        quarantineTimestamp: new Date(),
        tenantId,
      },
    };

    // In a real implementation, this would split the context into focused sub-tasks
    // For now, we just mark it as quarantined
    return quarantinedContext;
  }
}
