/**
 * @fileoverview Runtime Optimizer - Main Optimization Engine
 *
 * Coordinates performance monitoring, bottleneck detection, and
 * optimization recommendations.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import {
  BottleneckSeverity,
  MetricType,
  RecommendationType,
  type CacheStatistics,
  type IRuntimeOptimizer,
  type OptimizationAnalysis,
  type OptimizationEngineConfig,
  type OptimizationRecommendation,
  type PerformanceTrend,
} from "@/types/optimization-types";
import { v4 as uuidv4 } from "uuid";
import { BottleneckDetector } from "./BottleneckDetector";
import { PerformanceMonitor } from "./PerformanceMonitor";

/**
 * Default optimization engine configuration
 */
const DEFAULT_CONFIG: OptimizationEngineConfig = {
  enabled: true,
  collectionIntervalMs: 10000, // 10 seconds
  analysisWindowMs: 300000, // 5 minutes
  maxOverheadPct: 5,
  thresholds: {
    [MetricType.CPU]: 80,
    [MetricType.MEMORY]: 85,
    [MetricType.LATENCY]: 1000,
    [MetricType.CACHE_HIT_RATE]: 70,
  },
  enableCacheOptimization: true,
  enableTrendAnalysis: true,
  minDataPointsForTrend: 10,
};

/**
 * Runtime Optimizer
 *
 * Main optimization engine that:
 * - Monitors system performance continuously
 * - Detects bottlenecks and issues
 * - Generates actionable recommendations
 * - Analyzes cache performance
 * - Tracks performance trends
 */
export class RuntimeOptimizer implements IRuntimeOptimizer {
  private logger: Logger;
  private config: OptimizationEngineConfig;
  private performanceMonitor: PerformanceMonitor;
  private bottleneckDetector: BottleneckDetector;
  private isRunning = false;
  private analysisTimer?: ReturnType<typeof setInterval>;
  private lastAnalysisTime?: Date;
  private analysisHistory: OptimizationAnalysis[] = [];

  constructor(config: Partial<OptimizationEngineConfig> = {}) {
    this.logger = new Logger("RuntimeOptimizer");
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.performanceMonitor = new PerformanceMonitor({
      maxMetrics: 10000,
      autoCleanOlderThanMs: this.config.analysisWindowMs * 2,
    });
    this.bottleneckDetector = new BottleneckDetector(this.config.thresholds);
  }

  /**
   * Initialize the optimizer
   */
  async initialize(): Promise<void> {
    await this.performanceMonitor.start();
    this.logger.info("Runtime optimizer initialized", {
      enabled: this.config.enabled,
      collectionInterval: this.config.collectionIntervalMs,
      analysisWindow: this.config.analysisWindowMs,
    });
  }

  /**
   * Start optimization monitoring
   */
  async start(): Promise<void> {
    if (this.isRunning) {
      this.logger.warn("Runtime optimizer already running");
      return;
    }

    if (!this.config.enabled) {
      this.logger.info("Runtime optimizer disabled, not starting");
      return;
    }

    this.isRunning = true;

    // Start periodic analysis
    this.analysisTimer = setInterval(async () => {
      try {
        await this.analyze();
      } catch (error) {
        this.logger.error("Analysis failed", { error });
      }
    }, this.config.collectionIntervalMs);

    this.logger.info("Runtime optimizer started");
  }

  /**
   * Stop optimization monitoring
   */
  async stop(): Promise<void> {
    if (!this.isRunning) {
      return;
    }

    if (this.analysisTimer) {
      clearInterval(this.analysisTimer);
      this.analysisTimer = undefined;
    }

    await this.performanceMonitor.stop();
    this.isRunning = false;

    this.logger.info("Runtime optimizer stopped");
  }

  /**
   * Perform analysis and generate recommendations
   */
  async analyze(): Promise<OptimizationAnalysis> {
    const startTime = Date.now();

    // Get metrics for analysis window
    const windowEnd = new Date();
    const windowStart = new Date(
      windowEnd.getTime() - this.config.analysisWindowMs
    );
    const metrics = await this.performanceMonitor.getMetrics(
      windowStart,
      windowEnd
    );

    // Detect bottlenecks
    const bottlenecks = await this.bottleneckDetector.detectBottlenecks(
      metrics
    );

    // Generate recommendations based on bottlenecks
    const recommendations = await this.generateRecommendations(bottlenecks);

    // Analyze trends
    const trends = this.config.enableTrendAnalysis
      ? await this.analyzePerformanceTrends(metrics)
      : [];

    // Analyze cache performance
    const cacheStats = this.config.enableCacheOptimization
      ? await this.analyzeCachePerformance(metrics)
      : [];

    // Calculate health score
    const healthScore = this.calculateHealthScore(bottlenecks);

    const analysis: OptimizationAnalysis = {
      timestamp: new Date(),
      windowMs: this.config.analysisWindowMs,
      bottlenecks,
      recommendations,
      trends,
      cacheStats,
      healthScore,
      analysisDurationMs: Date.now() - startTime,
    };

    this.lastAnalysisTime = analysis.timestamp;
    this.analysisHistory.push(analysis);

    // Keep only last 100 analyses
    if (this.analysisHistory.length > 100) {
      this.analysisHistory.shift();
    }

    this.logger.debug("Analysis completed", {
      metricsAnalyzed: metrics.length,
      bottlenecksDetected: bottlenecks.length,
      recommendationsGenerated: recommendations.length,
      healthScore,
      durationMs: analysis.analysisDurationMs,
    });

    return analysis;
  }

  /**
   * Get cache statistics
   */
  async getCacheStatistics(): Promise<CacheStatistics[]> {
    const windowEnd = new Date();
    const windowStart = new Date(
      windowEnd.getTime() - this.config.analysisWindowMs
    );
    const metrics = await this.performanceMonitor.getMetrics(
      windowStart,
      windowEnd
    );

    return this.analyzeCachePerformance(metrics);
  }

  /**
   * Get performance trends
   */
  async getPerformanceTrends(): Promise<PerformanceTrend[]> {
    const windowEnd = new Date();
    const windowStart = new Date(
      windowEnd.getTime() - this.config.analysisWindowMs
    );
    const metrics = await this.performanceMonitor.getMetrics(
      windowStart,
      windowEnd
    );

    return this.analyzePerformanceTrends(metrics);
  }

  /**
   * Get current configuration
   */
  getConfig(): OptimizationEngineConfig {
    return { ...this.config };
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<OptimizationEngineConfig>): void {
    const wasRunning = this.isRunning;

    // Stop if running
    if (wasRunning) {
      this.stop();
    }

    // Update config
    this.config = { ...this.config, ...config };

    // Update sub-components
    if (config.thresholds) {
      this.bottleneckDetector.updateThresholds(config.thresholds);
    }

    // Restart if was running
    if (wasRunning && this.config.enabled) {
      this.start();
    }

    this.logger.info("Configuration updated", this.config);
  }

  /**
   * Get health status
   */
  getHealthStatus(): {
    isRunning: boolean;
    lastAnalysisTime?: Date;
    metricsCollected: number;
    bottlenecksDetected: number;
    recommendationsGenerated: number;
  } {
    const latestAnalysis =
      this.analysisHistory.length > 0
        ? this.analysisHistory[this.analysisHistory.length - 1]
        : null;

    return {
      isRunning: this.isRunning,
      lastAnalysisTime: this.lastAnalysisTime,
      metricsCollected: this.performanceMonitor.getMetricCount(),
      bottlenecksDetected:
        this.bottleneckDetector.getActiveBottlenecks().length,
      recommendationsGenerated: latestAnalysis?.recommendations.length ?? 0,
    };
  }

  /**
   * Get analysis history
   */
  getAnalysisHistory(count: number = 10): OptimizationAnalysis[] {
    return this.analysisHistory.slice(-count);
  }

  /**
   * Generate optimization recommendations from bottlenecks
   */
  private async generateRecommendations(
    bottlenecks: any[]
  ): Promise<OptimizationRecommendation[]> {
    const recommendations: OptimizationRecommendation[] = [];

    for (const bottleneck of bottlenecks) {
      const recommendation =
        this.generateRecommendationForBottleneck(bottleneck);
      if (recommendation) {
        recommendations.push(recommendation);
      }
    }

    return recommendations;
  }

  /**
   * Generate recommendation for a specific bottleneck
   */
  private generateRecommendationForBottleneck(
    bottleneck: any
  ): OptimizationRecommendation | null {
    const baseRecommendation = {
      id: uuidv4(),
      component: bottleneck.component,
      relatedBottleneckId: bottleneck.id,
      generatedAt: new Date(),
      priority: this.mapSeverityToPriority(bottleneck.severity),
    };

    // Generate recommendation based on metric type
    switch (bottleneck.metricType) {
      case MetricType.CPU:
        return {
          ...baseRecommendation,
          type: RecommendationType.RESOURCE_ALLOCATION,
          title: "Optimize CPU Usage",
          description: `CPU usage at ${bottleneck.currentValue}% (threshold: ${bottleneck.threshold}%)`,
          expectedImpact: "Reduce CPU usage by 20-30% through optimization",
          estimatedImprovementPct: 25,
          implementationDifficulty: "moderate",
        };

      case MetricType.MEMORY:
        return {
          ...baseRecommendation,
          type: RecommendationType.MEMORY_MANAGEMENT,
          title: "Optimize Memory Usage",
          description: `Memory usage at ${bottleneck.currentValue}% (threshold: ${bottleneck.threshold}%)`,
          expectedImpact: "Reduce memory footprint and prevent leaks",
          estimatedImprovementPct: 20,
          implementationDifficulty: "moderate",
        };

      case MetricType.CACHE_HIT_RATE:
        return {
          ...baseRecommendation,
          type: RecommendationType.CACHE_OPTIMIZATION,
          title: "Improve Cache Performance",
          description: `Cache hit rate at ${bottleneck.currentValue}% (target: ${bottleneck.threshold}%)`,
          expectedImpact:
            "Improve cache hit rate through better eviction and warming",
          estimatedImprovementPct: 30,
          implementationDifficulty: "easy",
        };

      case MetricType.LATENCY:
        return {
          ...baseRecommendation,
          type: RecommendationType.CONCURRENCY_TUNING,
          title: "Reduce Latency",
          description: `Latency at ${bottleneck.currentValue}ms (threshold: ${bottleneck.threshold}ms)`,
          expectedImpact:
            "Reduce response times through concurrency optimization",
          estimatedImprovementPct: 35,
          implementationDifficulty: "hard",
        };

      default:
        return null;
    }
  }

  /**
   * Analyze cache performance from metrics
   */
  private async analyzeCachePerformance(
    metrics: any[]
  ): Promise<CacheStatistics[]> {
    const cacheMetrics = metrics.filter(
      (m) => m.type === MetricType.CACHE_HIT_RATE
    );
    const cacheStats: CacheStatistics[] = [];

    // Group by source (cache ID)
    const byCacheId = new Map<string, any[]>();
    for (const metric of cacheMetrics) {
      const cacheId = metric.source;
      const cacheMetrics = byCacheId.get(cacheId) ?? [];
      cacheMetrics.push(metric);
      byCacheId.set(cacheId, cacheMetrics);
    }

    for (const [cacheId, cacheMetrics] of byCacheId) {
      if (cacheMetrics.length === 0) continue;

      const hitRate =
        cacheMetrics.reduce((sum, m) => sum + m.value, 0) / cacheMetrics.length;

      cacheStats.push({
        cacheId,
        totalRequests: cacheMetrics.length,
        hits: Math.round((cacheMetrics.length * hitRate) / 100),
        misses: Math.round((cacheMetrics.length * (100 - hitRate)) / 100),
        hitRate,
        avgHitTimeMs: 10,
        avgMissTimeMs: 100,
        cacheSizeBytes: 0,
        evictionCount: 0,
        windowStartTime: cacheMetrics[0].timestamp,
        windowEndTime: cacheMetrics[cacheMetrics.length - 1].timestamp,
      });
    }

    return cacheStats;
  }

  /**
   * Analyze performance trends from metrics
   */
  private async analyzePerformanceTrends(
    metrics: any[]
  ): Promise<PerformanceTrend[]> {
    if (metrics.length < this.config.minDataPointsForTrend) {
      return [];
    }

    const trends: PerformanceTrend[] = [];

    // Group by component and metric type
    const groupedMetrics = new Map<string, any[]>();
    for (const metric of metrics) {
      const key = `${metric.source}-${metric.type}`;
      const group = groupedMetrics.get(key) ?? [];
      group.push(metric);
      groupedMetrics.set(key, group);
    }

    for (const [key, group] of groupedMetrics) {
      if (group.length < this.config.minDataPointsForTrend) continue;

      const values = group.map((m) => m.value);
      const avgValue = values.reduce((sum, v) => sum + v, 0) / values.length;
      const minValue = Math.min(...values);
      const maxValue = Math.max(...values);

      // Calculate standard deviation
      const variance =
        values.reduce((sum, v) => sum + Math.pow(v - avgValue, 2), 0) /
        values.length;
      const stdDev = Math.sqrt(variance);

      // Determine trend direction
      const firstHalf = values.slice(0, Math.floor(values.length / 2));
      const secondHalf = values.slice(Math.floor(values.length / 2));
      const firstAvg =
        firstHalf.reduce((sum, v) => sum + v, 0) / firstHalf.length;
      const secondAvg =
        secondHalf.reduce((sum, v) => sum + v, 0) / secondHalf.length;

      let direction: "improving" | "stable" | "degrading";
      if (Math.abs(secondAvg - firstAvg) / firstAvg < 0.1) {
        direction = "stable";
      } else if (secondAvg < firstAvg) {
        direction = "improving";
      } else {
        direction = "degrading";
      }

      trends.push({
        metricType: group[0].type,
        component: group[0].source,
        direction,
        averageValue: avgValue,
        minValue,
        maxValue,
        standardDeviation: stdDev,
        startTime: group[0].timestamp,
        endTime: group[group.length - 1].timestamp,
        dataPointCount: group.length,
      });
    }

    return trends;
  }

  /**
   * Calculate overall system health score
   */
  private calculateHealthScore(bottlenecks: any[]): number {
    if (bottlenecks.length === 0) {
      return 100;
    }

    let score = 100;

    for (const bottleneck of bottlenecks) {
      switch (bottleneck.severity) {
        case BottleneckSeverity.CRITICAL:
          score -= 20;
          break;
        case BottleneckSeverity.HIGH:
          score -= 10;
          break;
        case BottleneckSeverity.MEDIUM:
          score -= 5;
          break;
        case BottleneckSeverity.LOW:
          score -= 2;
          break;
      }
    }

    return Math.max(0, score);
  }

  /**
   * Map bottleneck severity to recommendation priority
   */
  private mapSeverityToPriority(
    severity: BottleneckSeverity
  ): "low" | "medium" | "high" {
    switch (severity) {
      case BottleneckSeverity.CRITICAL:
      case BottleneckSeverity.HIGH:
        return "high";
      case BottleneckSeverity.MEDIUM:
        return "medium";
      case BottleneckSeverity.LOW:
      default:
        return "low";
    }
  }
}
