/**
 * @fileoverview Context Gathering Coordinator - Parallelized Discovery
 *
 * Coordinates context gathering operations with parallelization, early stopping,
 * and configurable search depth to optimize information discovery efficiency.
 *
 * @author @darianrosebrook
 */

import { ContextGatheringConfig } from "../../types/agent-prompting";

/**
 * Search query for context gathering
 */
export interface SearchQuery {
  /** Query identifier */
  id: string;

  /** Search terms */
  terms: string[];

  /** Search strategy */
  strategy: "breadth" | "depth" | "focused";

  /** Priority level */
  priority: "high" | "medium" | "low";

  /** Expected result type */
  expectedType: "code" | "documentation" | "data" | "mixed";
}

/**
 * Query result from context gathering
 */
export interface QueryResult {
  /** Query identifier */
  queryId: string;

  /** Search results */
  results: SearchResult[];

  /** Processing time */
  processingTimeMs: number;

  /** Quality score (0-1) */
  qualityScore: number;

  /** Completion status */
  status: "success" | "partial" | "failed";

  /** Error message if failed */
  error?: string;
}

/**
 * Individual search result
 */
export interface SearchResult {
  /** Result identifier */
  id: string;

  /** Content snippet */
  content: string;

  /** Relevance score (0-1) */
  relevanceScore: number;

  /** Source location */
  source: {
    type: "file" | "database" | "api" | "memory";
    location: string;
    lineNumber?: number;
  };

  /** Metadata */
  metadata: {
    confidence: number;
    freshness: number; // How recent the information is
    authority: number; // Source reliability
  };
}

/**
 * Parallel execution result
 */
export interface ParallelExecutionResult {
  /** All query results */
  results: QueryResult[];

  /** Overall execution time */
  totalTimeMs: number;

  /** Quality metrics */
  qualityMetrics: {
    averageQualityScore: number;
    coverageScore: number;
    redundancyScore: number;
  };

  /** Early stopping information */
  earlyStopping?: {
    triggered: boolean;
    reason: string;
    convergenceThreshold: number;
  };
}

/**
 * Context Gathering Coordinator
 *
 * Orchestrates parallel context gathering with intelligent early stopping
 * and quality-based result selection.
 */
export class ContextGatheringCoordinator {
  private config: ContextGatheringConfig;
  private activeQueries: Map<string, Promise<QueryResult>>;
  private resultCache: Map<string, QueryResult>;

  /**
   * Create a new ContextGatheringCoordinator
   */
  constructor(config: ContextGatheringConfig) {
    this.config = config;
    this.activeQueries = new Map();
    this.resultCache = new Map();
  }

  /**
   * Execute queries in parallel with early stopping
   */
  async parallelizeQueries(queries: SearchQuery[]): Promise<QueryResult[]> {
    const startTime = Date.now();

    // Prioritize queries
    const prioritizedQueries = this.prioritizeQueries(queries);

    // Execute queries based on strategy
    const results = await this.executeWithStrategy(
      prioritizedQueries,
      startTime
    );

    return results;
  }

  /**
   * Create context gathering configuration for task complexity
   */
  createConfig(
    taskComplexity: string,
    reasoningEffort: string
  ): ContextGatheringConfig {
    // Adjust configuration based on task characteristics
    const baseConfig = { ...this.config };

    // Adjust depth limits based on complexity
    switch (taskComplexity) {
      case "trivial":
        baseConfig.depthLimits.veryLow = 3;
        baseConfig.depthLimits.low = 5;
        break;
      case "complex":
      case "expert":
        baseConfig.depthLimits.high = 20;
        break;
    }

    // Adjust early stopping based on reasoning effort
    if (reasoningEffort === "low") {
      baseConfig.earlyStopCriteria.convergenceThreshold = 0.6; // Stop sooner
    } else if (reasoningEffort === "high") {
      baseConfig.earlyStopCriteria.convergenceThreshold = 0.8; // More thorough
    }

    return baseConfig;
  }

  /**
   * Update coordinator configuration
   */
  async updateConfig(
    newConfig: Partial<ContextGatheringConfig>
  ): Promise<void> {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Check coordinator health
   */
  async isHealthy(): Promise<boolean> {
    try {
      // Basic health checks
      return !!(
        this.config &&
        this.config.depthLimits &&
        this.config.earlyStopCriteria &&
        this.config.strategy &&
        this.config.maxParallelQueries > 0
      );
    } catch (error) {
      console.error("ContextGatheringCoordinator health check failed:", error);
      return false;
    }
  }

  /**
   * Prioritize queries based on strategy and characteristics
   */
  private prioritizeQueries(queries: SearchQuery[]): SearchQuery[] {
    return queries.sort((a, b) => {
      // Priority first
      const priorityOrder = { high: 3, medium: 2, low: 1 };
      const priorityDiff =
        priorityOrder[b.priority] - priorityOrder[a.priority];
      if (priorityDiff !== 0) return priorityDiff;

      // Then by expected result type relevance
      const typeOrder = { code: 3, documentation: 2, data: 1, mixed: 2 };
      const typeDiff = typeOrder[b.expectedType] - typeOrder[a.expectedType];
      if (typeDiff !== 0) return typeDiff;

      // Finally by term count (more specific queries first)
      return b.terms.length - a.terms.length;
    });
  }

  /**
   * Execute queries using the configured strategy
   */
  private async executeWithStrategy(
    queries: SearchQuery[],
    startTime: number
  ): Promise<QueryResult[]> {
    switch (this.config.strategy) {
      case "parallel":
        return this.executeParallel(queries, startTime);
      case "serial":
        return this.executeSerial(queries, startTime);
      case "hybrid":
        return this.executeHybrid(queries, startTime);
      default:
        return this.executeParallel(queries, startTime);
    }
  }

  /**
   * Execute all queries in parallel
   */
  private async executeParallel(
    queries: SearchQuery[],
    startTime: number
  ): Promise<QueryResult[]> {
    const promises = queries.map((query) =>
      this.executeQuery(query, startTime)
    );
    const results = await Promise.allSettled(promises);

    return results.map((result, index) => {
      if (result.status === "fulfilled") {
        return result.value;
      } else {
        return this.createFailedResult(queries[index].id, result.reason);
      }
    });
  }

  /**
   * Execute queries serially
   */
  private async executeSerial(
    queries: SearchQuery[],
    startTime: number
  ): Promise<QueryResult[]> {
    const results: QueryResult[] = [];

    for (const query of queries) {
      try {
        const result = await this.executeQuery(query, startTime);
        results.push(result);
      } catch (error) {
        results.push(this.createFailedResult(query.id, error));
      }
    }

    return results;
  }

  /**
   * Execute queries using hybrid approach (parallel with early stopping)
   */
  private async executeHybrid(
    queries: SearchQuery[],
    startTime: number
  ): Promise<QueryResult[]> {
    const results: QueryResult[] = [];
    const batches = this.createQueryBatches(queries);

    for (const batch of batches) {
      // Execute batch in parallel
      const batchPromises = batch.map((query) =>
        this.executeQuery(query, startTime)
      );
      const batchResults = await Promise.allSettled(batchPromises);

      // Convert results
      const convertedResults = batchResults.map((result, index) => {
        if (result.status === "fulfilled") {
          return result.value;
        } else {
          return this.createFailedResult(batch[index].id, result.reason);
        }
      });

      results.push(...convertedResults);

      // Check for early stopping
      const shouldStop = this.shouldStopEarly(results, startTime);
      if (shouldStop.stop) {
        console.log(
          `ContextGatheringCoordinator: Early stopping - ${shouldStop.reason}`
        );
        break;
      }
    }

    return results;
  }

  /**
   * Execute a single query
   */
  private async executeQuery(
    query: SearchQuery,
    startTime: number
  ): Promise<QueryResult> {
    const queryStartTime = Date.now();

    try {
      // Check cache first
      const cacheKey = this.createCacheKey(query);
      const cached = this.resultCache.get(cacheKey);
      if (cached && this.isCacheValid(cached)) {
        return {
          ...cached,
          processingTimeMs: Date.now() - queryStartTime,
        };
      }

      // Execute the actual search
      const results = await this.performSearch(query);

      // Calculate quality score
      const qualityScore = this.calculateQualityScore(results);

      const result: QueryResult = {
        queryId: query.id,
        results,
        processingTimeMs: Date.now() - queryStartTime,
        qualityScore,
        status: results.length > 0 ? "success" : "partial",
      };

      // Cache successful results
      if (result.status === "success") {
        this.resultCache.set(cacheKey, result);
      }

      return result;
    } catch (error) {
      throw new Error(`Query execution failed: ${error}`);
    }
  }

  /**
   * Perform the actual search operation
   */
  private async performSearch(query: SearchQuery): Promise<SearchResult[]> {
    // This would integrate with actual search systems
    // For now, return mock results based on query characteristics

    const mockResults: SearchResult[] = [];

    // Generate mock results based on query terms
    for (let i = 0; i < Math.min(query.terms.length, 5); i++) {
      mockResults.push({
        id: `${query.id}-result-${i}`,
        content: `Mock content for term: ${query.terms[i]}`,
        relevanceScore: Math.random() * 0.5 + 0.5, // 0.5-1.0
        source: {
          type: query.expectedType === "code" ? "file" : "database",
          location: `/mock/path/${query.terms[i]}.txt`,
          lineNumber: Math.floor(Math.random() * 100),
        },
        metadata: {
          confidence: Math.random() * 0.3 + 0.7, // 0.7-1.0
          freshness: Math.random(),
          authority: Math.random() * 0.4 + 0.6, // 0.6-1.0
        },
      });
    }

    // Sort by relevance
    return mockResults.sort((a, b) => b.relevanceScore - a.relevanceScore);
  }

  /**
   * Calculate quality score for query results
   */
  private calculateQualityScore(results: SearchResult[]): number {
    if (results.length === 0) return 0;

    const avgRelevance =
      results.reduce((sum, r) => sum + r.relevanceScore, 0) / results.length;
    const avgConfidence =
      results.reduce((sum, r) => sum + r.metadata.confidence, 0) /
      results.length;
    const avgAuthority =
      results.reduce((sum, r) => sum + r.metadata.authority, 0) /
      results.length;

    // Weighted quality score
    return avgRelevance * 0.5 + avgConfidence * 0.3 + avgAuthority * 0.2;
  }

  /**
   * Create query batches for hybrid execution
   */
  private createQueryBatches(queries: SearchQuery[]): SearchQuery[][] {
    const batches: SearchQuery[][] = [];
    const batchSize = Math.min(this.config.maxParallelQueries, 5);

    for (let i = 0; i < queries.length; i += batchSize) {
      batches.push(queries.slice(i, i + batchSize));
    }

    return batches;
  }

  /**
   * Check if execution should stop early
   */
  private shouldStopEarly(
    currentResults: QueryResult[],
    startTime: number
  ): { stop: boolean; reason?: string } {
    const elapsed = Date.now() - startTime;

    // Time-based early stopping
    if (elapsed >= this.config.earlyStopCriteria.maxTimeMs) {
      return { stop: true, reason: "max_time_exceeded" };
    }

    // Quality-based early stopping
    const avgQuality =
      currentResults.reduce((sum, r) => sum + r.qualityScore, 0) /
      currentResults.length;

    if (avgQuality >= this.config.earlyStopCriteria.convergenceThreshold) {
      return { stop: true, reason: "quality_threshold_met" };
    }

    // Convergence-based early stopping
    const recentResults = currentResults.slice(-3);
    if (recentResults.length >= 3) {
      const convergenceScore = this.calculateConvergenceScore(recentResults);
      if (
        convergenceScore >= this.config.earlyStopCriteria.convergenceThreshold
      ) {
        return { stop: true, reason: "results_converged" };
      }
    }

    return { stop: false };
  }

  /**
   * Calculate convergence score for recent results
   */
  private calculateConvergenceScore(recentResults: QueryResult[]): number {
    // Simple convergence metric: average quality score of recent results
    return (
      recentResults.reduce((sum, r) => sum + r.qualityScore, 0) /
      recentResults.length
    );
  }

  /**
   * Create cache key for query
   */
  private createCacheKey(query: SearchQuery): string {
    return `${query.id}-${query.terms.join("-")}-${query.strategy}`;
  }

  /**
   * Check if cached result is still valid
   */
  private isCacheValid(cached: QueryResult): boolean {
    const cacheAge = Date.now() - cached.results[0]?.metadata?.freshness || 0;
    return cacheAge < 300000; // 5 minutes cache validity
  }

  /**
   * Create a failed query result
   */
  private createFailedResult(queryId: string, error: any): QueryResult {
    return {
      queryId,
      results: [],
      processingTimeMs: 0,
      qualityScore: 0,
      status: "failed",
      error: error?.message || "Unknown error",
    };
  }
}
