/**
 * @fileoverview Embedding Infrastructure Monitor
 *
 * Monitoring integration for embedding infrastructure with existing SystemHealthMonitor.
 * Tracks performance, cache metrics, and knowledge base health.
 *
 * @author @darianrosebrook
 */

import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient";
import { SystemHealthMonitor } from "../monitoring/SystemHealthMonitor";
import { EmbeddingService } from "./EmbeddingService";

/**
 * Metrics collected by the embedding monitor
 */
export interface EmbeddingMetrics {
  embeddingService: {
    available: boolean;
    cacheStats: {
      size: number;
      maxSize: number;
      hitRate: number;
    };
    performance: PerformanceStats;
  };
  knowledgeBase: KnowledgeBaseMetrics;
  semanticSearch: SemanticSearchMetrics;
  timestamp?: Date;
}

/**
 * Performance statistics for embedding operations
 */
export interface PerformanceStats {
  averageLatency: number;
  p95Latency: number;
  throughput: number;
  targetGemma3nTokensPerSec: number;
}

/**
 * Knowledge base health metrics
 */
export interface KnowledgeBaseMetrics {
  workspaceFiles: number;
  externalKnowledge: number;
  averageConfidence: number;
  lastUpdated: Date;
  totalEntries: number;
  confidenceDistribution: {
    high: number; // > 0.8
    medium: number; // 0.5 - 0.8
    low: number; // < 0.5
  };
}

/**
 * Semantic search performance metrics
 */
export interface SemanticSearchMetrics {
  queriesPerMinute: number;
  averageQueryLatency: number;
  cacheHitRate: number;
  topEntityTypes: Array<{
    type: string;
    count: number;
    avgRelevance: number;
  }>;
}

/**
 * Embedding monitor for system health monitoring
 */
export class EmbeddingMonitor {
  private embeddingService: EmbeddingService;
  private dbClient: KnowledgeDatabaseClient;
  private systemHealthMonitor: SystemHealthMonitor;
  private metricsHistory: EmbeddingMetrics[] = [];
  private maxHistorySize = 100;

  constructor(
    embeddingService: EmbeddingService,
    dbClient: KnowledgeDatabaseClient,
    systemHealthMonitor: SystemHealthMonitor
  ) {
    this.embeddingService = embeddingService;
    this.dbClient = dbClient;
    this.systemHealthMonitor = systemHealthMonitor;

    // Note: Embedding monitor should be registered with SystemHealthMonitor
    // via systemHealthMonitor.registerEmbeddingMonitor(this)

    // Start periodic metrics collection
    this.startMetricsCollection();
  }

  /**
   * Collect comprehensive embedding metrics
   */
  async collectMetrics(): Promise<EmbeddingMetrics> {
    const metrics: EmbeddingMetrics = {
      embeddingService: {
        available: await this.embeddingService.isAvailable(),
        cacheStats: this.embeddingService.getCacheStats(),
        performance: await this.measureEmbeddingPerformance(),
      },
      knowledgeBase: await this.collectKnowledgeBaseMetrics(),
      semanticSearch: await this.collectSemanticSearchMetrics(),
    };

    // Add timestamp and store in history for trend analysis
    const metricsWithTimestamp = { ...metrics, timestamp: new Date() };
    this.metricsHistory.push(metricsWithTimestamp);
    if (this.metricsHistory.length > this.maxHistorySize) {
      this.metricsHistory.shift();
    }

    return metricsWithTimestamp;
  }

  /**
   * Measure embedding generation performance against benchmarks
   */
  private async measureEmbeddingPerformance(): Promise<PerformanceStats> {
    const testTexts = [
      "short", // ~1 token
      "medium length text for testing".repeat(3), // ~12 tokens
      "long text for comprehensive performance evaluation".repeat(8), // ~64 tokens
    ];

    const latencies: number[] = [];
    let totalTokens = 0;

    for (const text of testTexts) {
      const startTime = Date.now();
      try {
        await this.embeddingService.generateEmbedding(text);
        latencies.push(Date.now() - startTime);
        totalTokens += Math.ceil(text.length / 4); // Rough token estimation
      } catch (error) {
        console.warn(`Embedding performance test failed: ${error.message}`);
        latencies.push(1000); // Penalize failures with high latency
      }
    }

    const sortedLatencies = latencies.sort((a, b) => a - b);
    const averageLatency = latencies.reduce((a, b) => a + b) / latencies.length;
    const p95Latency = sortedLatencies[Math.floor(latencies.length * 0.95)];
    const throughput = totalTokens / (averageLatency / 1000);

    return {
      averageLatency,
      p95Latency,
      throughput,
      targetGemma3nTokensPerSec: 36.02, // Your established benchmark
    };
  }

  /**
   * Collect knowledge base health metrics
   */
  private async collectKnowledgeBaseMetrics(): Promise<KnowledgeBaseMetrics> {
    const result = await this.dbClient.query(`
      SELECT
        capability_type,
        COUNT(*) as count,
        AVG(confidence) as avg_confidence,
        MAX(last_updated) as last_updated,
        COUNT(*) FILTER (WHERE confidence > 0.8) as high_confidence,
        COUNT(*) FILTER (WHERE confidence BETWEEN 0.5 AND 0.8) as medium_confidence,
        COUNT(*) FILTER (WHERE confidence < 0.5) as low_confidence
      FROM agent_capabilities_graph
      WHERE capability_type IN ('TECHNOLOGY', 'EXTERNAL_KNOWLEDGE')
      GROUP BY capability_type
    `);

    const workspaceStats = result.rows.find(
      (r) => r.capability_type === "TECHNOLOGY"
    ) || {
      count: 0,
      avg_confidence: 0,
      last_updated: null,
      high_confidence: 0,
      medium_confidence: 0,
      low_confidence: 0,
    };

    const externalStats = result.rows.find(
      (r) => r.capability_type === "EXTERNAL_KNOWLEDGE"
    ) || {
      count: 0,
      avg_confidence: 0,
      last_updated: null,
      high_confidence: 0,
      medium_confidence: 0,
      low_confidence: 0,
    };

    const totalEntries = workspaceStats.count + externalStats.count;
    const avgConfidence =
      totalEntries > 0
        ? (workspaceStats.avg_confidence * workspaceStats.count +
            externalStats.avg_confidence * externalStats.count) /
          totalEntries
        : 0;

    const lastUpdated =
      workspaceStats.last_updated && externalStats.last_updated
        ? new Date(
            Math.max(
              new Date(workspaceStats.last_updated).getTime(),
              new Date(externalStats.last_updated).getTime()
            )
          )
        : workspaceStats.last_updated
        ? new Date(workspaceStats.last_updated)
        : externalStats.last_updated
        ? new Date(externalStats.last_updated)
        : new Date();

    return {
      workspaceFiles: workspaceStats.count,
      externalKnowledge: externalStats.count,
      averageConfidence: avgConfidence,
      lastUpdated,
      totalEntries,
      confidenceDistribution: {
        high: workspaceStats.high_confidence + externalStats.high_confidence,
        medium:
          workspaceStats.medium_confidence + externalStats.medium_confidence,
        low: workspaceStats.low_confidence + externalStats.low_confidence,
      },
    };
  }

  /**
   * Collect semantic search performance metrics
   */
  private async collectSemanticSearchMetrics(): Promise<SemanticSearchMetrics> {
    // Get recent search sessions from the last hour
    const result = await this.dbClient.query(`
      SELECT
        COUNT(*) as total_queries,
        AVG(execution_time_ms) as avg_latency,
        COUNT(*) FILTER (WHERE search_type = 'hybrid') as semantic_queries,
        AVG(execution_time_ms) FILTER (WHERE search_type = 'hybrid') as semantic_avg_latency
      FROM graph_search_sessions
      WHERE created_at > NOW() - INTERVAL '1 hour'
    `);

    const stats = result.rows[0] || {
      total_queries: 0,
      avg_latency: 0,
      semantic_queries: 0,
      semantic_avg_latency: 0,
    };

    // Calculate queries per minute
    const queriesPerMinute = stats.total_queries * 60; // Assuming 1-hour window

    // Get top entity types by usage
    const entityTypeResult = await this.dbClient.query(`
      SELECT
        unnest(entity_type_filters) as entity_type,
        COUNT(*) as usage_count,
        AVG(result_count) as avg_results
      FROM graph_search_sessions
      WHERE created_at > NOW() - INTERVAL '24 hours'
        AND search_type = 'hybrid'
        AND entity_type_filters IS NOT NULL
      GROUP BY unnest(entity_type_filters)
      ORDER BY usage_count DESC
      LIMIT 5
    `);

    const topEntityTypes = entityTypeResult.rows.map((row) => ({
      type: row.entity_type,
      count: row.usage_count,
      avgRelevance: row.avg_results, // Using result count as proxy for relevance
    }));

    return {
      queriesPerMinute,
      averageQueryLatency: stats.semantic_avg_latency || 0,
      cacheHitRate: 0, // Would need to track in embedding service
      topEntityTypes,
    };
  }

  /**
   * Get health status for system monitoring
   */
  async getHealthStatus(): Promise<{
    status: "healthy" | "degraded" | "unhealthy";
    message: string;
    metrics: EmbeddingMetrics;
  }> {
    const metrics = await this.collectMetrics();

    let status: "healthy" | "degraded" | "unhealthy" = "healthy";
    const issues: string[] = [];

    // Check embedding service availability
    if (!metrics.embeddingService.available) {
      status = "unhealthy";
      issues.push("Embedding service unavailable");
    }

    // Check performance against targets
    if (metrics.embeddingService.performance.p95Latency > 500) {
      status = status === "unhealthy" ? "unhealthy" : "degraded";
      issues.push(
        `Embedding latency too high: ${metrics.embeddingService.performance.p95Latency}ms`
      );
    }

    // Check knowledge base health
    if (metrics.knowledgeBase.totalEntries === 0) {
      status = "degraded";
      issues.push("Knowledge base is empty");
    }

    if (metrics.knowledgeBase.averageConfidence < 0.5) {
      status = "degraded";
      issues.push(
        `Low average knowledge confidence: ${metrics.knowledgeBase.averageConfidence}`
      );
    }

    // Check if knowledge base is stale (>24 hours old)
    const hoursSinceUpdate =
      (Date.now() - metrics.knowledgeBase.lastUpdated.getTime()) /
      (1000 * 60 * 60);
    if (hoursSinceUpdate > 24) {
      status = "degraded";
      issues.push(
        `Knowledge base stale: ${hoursSinceUpdate.toFixed(
          1
        )} hours since last update`
      );
    }

    return {
      status,
      message:
        issues.length > 0 ? issues.join("; ") : "All systems operational",
      metrics,
    };
  }

  /**
   * Get performance trends over time
   */
  getPerformanceTrends(hours: number = 24): {
    embeddingLatency: number[];
    knowledgeConfidence: number[];
    searchThroughput: number[];
    timestamps: Date[];
  } {
    const cutoffTime = Date.now() - hours * 60 * 60 * 1000;
    const recentMetrics = this.metricsHistory.filter(
      (m) => m.timestamp && m.timestamp.getTime() > cutoffTime
    );

    return {
      embeddingLatency: recentMetrics.map(
        (m) => m.embeddingService.performance.averageLatency
      ),
      knowledgeConfidence: recentMetrics.map(
        (m) => m.knowledgeBase.averageConfidence
      ),
      searchThroughput: recentMetrics.map(
        (m) => m.semanticSearch.queriesPerMinute
      ),
      timestamps: recentMetrics.map((m) => m.timestamp!),
    };
  }

  /**
   * Start periodic metrics collection
   */
  private startMetricsCollection(): void {
    // Collect metrics every 5 minutes
    setInterval(async () => {
      try {
        await this.collectMetrics();
      } catch (error) {
        console.error("Failed to collect embedding metrics:", error);
      }
    }, 5 * 60 * 1000);
  }

  /**
   * Get detailed diagnostic information
   */
  async getDiagnostics(): Promise<{
    cacheEfficiency: number;
    indexFreshness: number;
    performanceDegradation: number;
    recommendations: string[];
  }> {
    const metrics = await this.collectMetrics();
    const trends = this.getPerformanceTrends(1); // Last hour

    const recommendations: string[] = [];

    // Cache efficiency
    const cacheEfficiency = metrics.embeddingService.cacheStats.hitRate;

    // Index freshness (how recent are updates)
    const indexFreshness =
      (Date.now() - metrics.knowledgeBase.lastUpdated.getTime()) /
      (1000 * 60 * 60); // hours

    // Performance degradation (compare to baseline)
    const recentLatency =
      trends.embeddingLatency[trends.embeddingLatency.length - 1] || 0;
    const baselineLatency = trends.embeddingLatency[0] || recentLatency;
    const performanceDegradation =
      ((recentLatency - baselineLatency) / baselineLatency) * 100;

    // Generate recommendations
    if (cacheEfficiency < 0.5) {
      recommendations.push(
        "Consider increasing embedding cache size for better hit rates"
      );
    }

    if (indexFreshness > 24) {
      recommendations.push(
        "Knowledge base is stale - consider running indexing scripts"
      );
    }

    if (performanceDegradation > 20) {
      recommendations.push(
        "Embedding performance degrading - check Ollama service health"
      );
    }

    if (
      metrics.knowledgeBase.confidenceDistribution.low >
      metrics.knowledgeBase.totalEntries * 0.3
    ) {
      recommendations.push(
        "High proportion of low-confidence knowledge - consider reinforcement or cleanup"
      );
    }

    return {
      cacheEfficiency,
      indexFreshness,
      performanceDegradation,
      recommendations,
    };
  }
}

// Export for integration with SystemHealthMonitor
export type EmbeddingMonitorComponent = EmbeddingMonitor;
