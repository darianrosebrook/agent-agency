import { EventEmitter } from "events";
import {
  FeedbackEvent,
  FeedbackAnalysis,
  FeedbackRecommendation,
  FeedbackLoopConfig,
  FeedbackStats,
  FeedbackProcessingResult,
  FeedbackSource,
  FeedbackType,
} from "../types/feedback-loop";
import { FeedbackCollector } from "./FeedbackCollector";
import { FeedbackAnalyzer } from "./FeedbackAnalyzer";
import { ImprovementEngine } from "./ImprovementEngine";
import { FeedbackPipeline } from "./FeedbackPipeline";
import { ConfigManager } from "../config/ConfigManager";
import { Logger } from "../observability/Logger";

export class FeedbackLoopManager extends EventEmitter {
  private config: FeedbackLoopConfig;
  private logger: Logger;

  private collector: FeedbackCollector;
  private analyzer: FeedbackAnalyzer;
  private improvementEngine: ImprovementEngine;
  private pipeline: FeedbackPipeline;

  private isRunning: boolean = false;
  private analysisTimer: NodeJS.Timeout | null = null;
  private processingTimer: NodeJS.Timeout | null = null;

  // Statistics
  private stats: FeedbackStats = {
    totalEvents: 0,
    eventsBySource: {
      [FeedbackSource.PERFORMANCE_METRICS]: 0,
      [FeedbackSource.TASK_OUTCOMES]: 0,
      [FeedbackSource.USER_RATINGS]: 0,
      [FeedbackSource.SYSTEM_EVENTS]: 0,
      [FeedbackSource.CONSTITUTIONAL_VIOLATIONS]: 0,
      [FeedbackSource.COMPONENT_HEALTH]: 0,
      [FeedbackSource.ROUTING_DECISIONS]: 0,
      [FeedbackSource.AGENT_FEEDBACK]: 0,
    },
    eventsByType: {
      [FeedbackType.NUMERIC_METRIC]: 0,
      [FeedbackType.CATEGORICAL_EVENT]: 0,
      [FeedbackType.TEXT_FEEDBACK]: 0,
      [FeedbackType.RATING_SCALE]: 0,
      [FeedbackType.BINARY_OUTCOME]: 0,
    },
    analysisCount: 0,
    recommendationsGenerated: 0,
    recommendationsApplied: 0,
    averageProcessingTimeMs: 0,
    dataQualityScore: 0,
    uptimeSeconds: 0,
  };

  private startTime: Date = new Date();
  private processingTimes: number[] = [];

  constructor(configManager: ConfigManager) {
    super();
    this.config = configManager.get("feedbackLoop");
    this.logger = new Logger("FeedbackLoopManager");

    // Initialize components
    this.collector = new FeedbackCollector(configManager);
    this.analyzer = new FeedbackAnalyzer(configManager);
    this.improvementEngine = new ImprovementEngine(configManager);
    this.pipeline = new FeedbackPipeline(configManager);

    this.setupEventHandlers();
  }

  public async initialize(): Promise<void> {
    this.logger.info("Initializing FeedbackLoopManager...");

    if (this.config.enabled) {
      this.collector.start();
      this.startAnalysisTimer();
      this.startProcessingTimer();
      this.isRunning = true;
    }

    this.emit("feedback-loop:initialized", { timestamp: new Date() });
    this.logger.info("FeedbackLoopManager initialized and running");
  }

  public async shutdown(): Promise<void> {
    this.logger.info("Shutting down FeedbackLoopManager...");

    this.isRunning = false;

    if (this.analysisTimer) {
      clearInterval(this.analysisTimer);
      this.analysisTimer = null;
    }

    if (this.processingTimer) {
      clearInterval(this.processingTimer);
      this.processingTimer = null;
    }

    this.collector.stop();
    await this.pipeline.flush();

    this.emit("feedback-loop:shutdown", { timestamp: new Date() });
    this.logger.info("FeedbackLoopManager shut down");
  }

  // Public API for collecting feedback
  public collectPerformanceMetrics(entityId: string, entityType: string, metrics: any): void {
    this.collector.collectPerformanceMetrics(entityId, entityType, metrics);
  }

  public collectTaskOutcome(taskId: string, outcome: any, executionTimeMs: number, retryCount: number, errorDetails?: string): void {
    this.collector.collectTaskOutcome(taskId, outcome, executionTimeMs, retryCount, errorDetails);
  }

  public collectUserRating(entityId: string, entityType: string, rating: number, criteria: any, comments?: string): void {
    this.collector.collectUserRating(entityId, entityType, rating, criteria, comments);
  }

  public collectSystemEvent(eventType: string, severity: string, description: string, impact: any): void {
    this.collector.collectSystemEvent(eventType, severity as any, description, impact);
  }

  public collectConstitutionalViolation(violation: any, policyImpact: any): void {
    this.collector.collectConstitutionalViolation(violation, policyImpact);
  }

  public collectComponentHealth(health: any, previousStatus?: any, statusChangeReason?: string): void {
    this.collector.collectComponentHealth(health, previousStatus, statusChangeReason);
  }

  public collectRoutingDecision(decision: any, outcome?: any): void {
    this.collector.collectRoutingDecision(decision, outcome);
  }

  public collectAgentFeedback(agentId: string, feedback: any): void {
    this.collector.collectAgentFeedback(agentId, feedback);
  }

  // Public API for analysis
  public analyzeEntity(entityId: string, entityType: string): FeedbackAnalysis {
    this.stats.analysisCount++;
    return this.analyzer.analyzeEntityFeedback(entityId, entityType);
  }

  public analyzeAllEntities(entityType?: string): FeedbackAnalysis[] {
    this.stats.analysisCount++;
    return this.analyzer.analyzeAllEntities(entityType);
  }

  // Public API for improvements
  public async applyRecommendation(recommendation: FeedbackRecommendation): Promise<boolean> {
    const applied = await this.improvementEngine.applyRecommendation(recommendation);
    if (applied) {
      this.stats.recommendationsApplied++;
    }
    return applied;
  }

  public async applyRecommendations(recommendations: FeedbackRecommendation[]): Promise<any> {
    const results = await this.improvementEngine.applyRecommendations(recommendations);
    this.stats.recommendationsApplied += results.applied.length;
    return results;
  }

  // Public API for getting data
  public getEntityAnalysis(entityId: string): FeedbackAnalysis | undefined {
    return this.analyzer.getAnalysisCache().get(entityId);
  }

  public getActiveImprovements(): FeedbackRecommendation[] {
    return this.improvementEngine.getActiveImprovements();
  }

  public getImprovementHistory(entityId?: string): FeedbackRecommendation[] {
    return this.improvementEngine.getImprovementHistory(entityId);
  }

  public getStats(): FeedbackStats {
    const uptime = Date.now() - this.startTime.getTime();
    const collectorStats = this.collector.getStats();
    const improvementStats = this.improvementEngine.getStats();
    const pipelineStats = this.pipeline.getStats();

    return {
      ...this.stats,
      totalEvents: collectorStats.totalEvents,
      eventsBySource: collectorStats.eventsBySource,
      eventsByType: collectorStats.eventsByType,
      averageProcessingTimeMs: this.processingTimes.length > 0
        ? this.processingTimes.reduce((sum, time) => sum + time, 0) / this.processingTimes.length
        : 0,
      dataQualityScore: pipelineStats.averageQualityScore || 0,
      uptimeSeconds: Math.floor(uptime / 1000),
    };
  }

  private setupEventHandlers(): void {
    // Collector events
    this.collector.on("feedback:collected", (event: FeedbackEvent) => {
      this.analyzer.addFeedbackEvent(event);
      this.emit("feedback:collected", event);
    });

    this.collector.on("feedback:batch-ready", async (batch: FeedbackEvent[]) => {
      await this.processFeedbackBatch(batch);
    });

    // Analyzer events (none currently defined)

    // Improvement Engine events
    this.improvementEngine.on("improvement:started", (data) => {
      this.emit("improvement:started", data);
    });

    this.improvementEngine.on("improvement:completed", (data) => {
      this.emit("improvement:completed", data);
    });

    this.improvementEngine.on("improvement:failed", (data) => {
      this.emit("improvement:failed", data);
    });

    this.improvementEngine.on("improvement:monitored", (data) => {
      this.emit("improvement:monitored", data);
    });

    // Pipeline events
    this.pipeline.on("pipeline:batch-processed", (data) => {
      this.emit("pipeline:batch-processed", data);
    });

    this.pipeline.on("pipeline:error", (error) => {
      this.emit("pipeline:error", error);
    });
  }

  private async processFeedbackBatch(batch: FeedbackEvent[]): Promise<void> {
    const startTime = Date.now();

    try {
      this.logger.debug(`Processing feedback batch of ${batch.length} events`);

      // Process batch through collector (validation, etc.)
      const processingResult = await this.collector.processBatch(batch);

      // Send high-quality data to pipeline
      if (processingResult.qualityScore >= this.config.pipeline.dataQualityThreshold) {
        await this.pipeline.processBatch(batch);
      }

      // Generate recommendations from analysis
      const analyses = this.analyzer.analyzeAllEntities();
      const allRecommendations: FeedbackRecommendation[] = [];

      for (const analysis of analyses) {
        allRecommendations.push(...analysis.recommendations);
      }

      this.stats.recommendationsGenerated += allRecommendations.length;

      // Auto-apply recommendations if enabled
      if (this.config.improvements.autoApplyThreshold > 0) {
        const applicationResults = await this.improvementEngine.applyRecommendations(allRecommendations);
        this.stats.recommendationsApplied += applicationResults.applied.length;

        this.emit("recommendations:processed", {
          generated: allRecommendations.length,
          applied: applicationResults.applied.length,
          skipped: applicationResults.skipped.length,
          failed: applicationResults.failed.length,
          timestamp: new Date(),
        });
      } else {
        this.emit("recommendations:generated", {
          recommendations: allRecommendations,
          timestamp: new Date(),
        });
      }

      const processingTime = Date.now() - startTime;
      this.processingTimes.push(processingTime);

      // Keep only last 100 processing times for average calculation
      if (this.processingTimes.length > 100) {
        this.processingTimes = this.processingTimes.slice(-100);
      }

      this.emit("feedback:batch-processed", {
        batchSize: batch.length,
        processingTimeMs: processingTime,
        qualityScore: processingResult.qualityScore,
        recommendationsGenerated: allRecommendations.length,
        timestamp: new Date(),
      });

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      this.logger.error(`Error processing feedback batch: ${errorMessage}`);
      this.emit("feedback:batch-error", {
        error: errorMessage,
        batchSize: batch.length,
        timestamp: new Date(),
      });
    }
  }

  private startAnalysisTimer(): void {
    if (this.analysisTimer) {
      clearInterval(this.analysisTimer);
    }

    this.analysisTimer = setInterval(async () => {
      if (!this.isRunning) return;

      try {
        // Periodic analysis of all entities
        const analyses = this.analyzer.analyzeAllEntities();
        const totalRecommendations = analyses.reduce((sum, analysis) => sum + analysis.recommendations.length, 0);

        this.stats.analysisCount += analyses.length;
        this.stats.recommendationsGenerated += totalRecommendations;

        this.emit("analysis:completed", {
          entitiesAnalyzed: analyses.length,
          recommendationsGenerated: totalRecommendations,
          timestamp: new Date(),
        });

      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        this.logger.error(`Error in periodic analysis: ${errorMessage}`);
      }
    }, this.config.analysis.analysisIntervalMs);
  }

  private startProcessingTimer(): void {
    if (this.processingTimer) {
      clearInterval(this.processingTimer);
    }

    this.processingTimer = setInterval(async () => {
      if (!this.isRunning) return;

      try {
        // Periodic pipeline processing
        await this.pipeline.processPendingBatches();

        this.emit("processing:cycle-completed", {
          timestamp: new Date(),
        });

      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        this.logger.error(`Error in processing cycle: ${errorMessage}`);
      }
    }, this.config.pipeline.processingIntervalMs);
  }

  public clearStats(): void {
    this.stats = {
      totalEvents: 0,
      eventsBySource: {
        [FeedbackSource.PERFORMANCE_METRICS]: 0,
        [FeedbackSource.TASK_OUTCOMES]: 0,
        [FeedbackSource.USER_RATINGS]: 0,
        [FeedbackSource.SYSTEM_EVENTS]: 0,
        [FeedbackSource.CONSTITUTIONAL_VIOLATIONS]: 0,
        [FeedbackSource.COMPONENT_HEALTH]: 0,
        [FeedbackSource.ROUTING_DECISIONS]: 0,
        [FeedbackSource.AGENT_FEEDBACK]: 0,
      },
      eventsByType: {
        [FeedbackType.NUMERIC_METRIC]: 0,
        [FeedbackType.CATEGORICAL_EVENT]: 0,
        [FeedbackType.TEXT_FEEDBACK]: 0,
        [FeedbackType.RATING_SCALE]: 0,
        [FeedbackType.BINARY_OUTCOME]: 0,
      },
      analysisCount: 0,
      recommendationsGenerated: 0,
      recommendationsApplied: 0,
      averageProcessingTimeMs: 0,
      dataQualityScore: 0,
      uptimeSeconds: 0,
    };
    this.processingTimes = [];
    this.startTime = new Date();

    this.collector.clearStats();
    this.analyzer.clearAnalysisCache();
    this.improvementEngine.clearHistory();
  }

  public getHealthStatus(): {
    status: "healthy" | "degraded" | "unhealthy";
    details: Record<string, any>;
  } {
    const stats = this.getStats();
    const collectorStats = this.collector.getStats();

    const isHealthy = this.isRunning &&
                     stats.averageProcessingTimeMs < 1000 && // < 1s average processing
                     collectorStats.bufferSize < this.config.collection.batchSize * 2; // Not overflowing

    const isDegraded = this.isRunning &&
                      (stats.averageProcessingTimeMs > 1000 || collectorStats.bufferSize > this.config.collection.batchSize);

    return {
      status: isHealthy ? "healthy" : isDegraded ? "degraded" : "unhealthy",
      details: {
        isRunning: this.isRunning,
        bufferSize: collectorStats.bufferSize,
        averageProcessingTimeMs: stats.averageProcessingTimeMs,
        totalEvents: stats.totalEvents,
        activeImprovements: this.improvementEngine.getStats().activeImprovements,
        lastAnalysisTime: stats.lastAnalysisTime,
      },
    };
  }
}
