// @ts-nocheck
import { EventEmitter } from "events";
import { ConfigManager } from "../config/ConfigManager";
import { Logger } from "../observability/Logger";
import {
  FeedbackEvent,
  FeedbackPipelineConfig,
  FeedbackSource,
} from "../types/feedback-loop";

export interface TrainingDataBatch {
  id: string;
  timestamp: string;
  events: FeedbackEvent[];
  features: Record<string, any>;
  qualityScore: number;
  metadata: {
    batchSize: number;
    timeRange: { start: string; end: string };
    entityTypes: string[];
    sources: FeedbackSource[];
  };
}

export class FeedbackPipeline extends EventEmitter {
  private config: FeedbackPipelineConfig;
  private logger: Logger;

  // Pipeline state
  private pendingBatches: TrainingDataBatch[] = [];
  private processedBatches: TrainingDataBatch[] = [];
  private qualityScores: number[] = [];

  // Statistics
  private stats = {
    totalBatchesProcessed: 0,
    totalEventsProcessed: 0,
    averageQualityScore: 0,
    batchesByQuality: { high: 0, medium: 0, low: 0 },
    processingErrors: 0,
    lastProcessingTime: undefined as string | undefined,
  };

  constructor(configManager: ConfigManager) {
    super();
    this.config = configManager.get("feedbackLoop.pipeline");
    this.logger = new Logger("FeedbackPipeline");
  }

  public async processBatch(
    events: FeedbackEvent[]
  ): Promise<TrainingDataBatch> {
    const startTime = new Date();

    try {
      this.logger.debug(`Processing batch of ${events.length} feedback events`);

      // Validate batch
      if (!this.validateBatch(events)) {
        throw new Error("Batch validation failed");
      }

      // Extract features
      const features = await this.extractFeatures(events);

      // Assess data quality
      const qualityScore = this.assessDataQuality(events, features);

      // Create training data batch
      const batch: TrainingDataBatch = {
        id: `batch-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        timestamp: startTime.toISOString(),
        events:
          this.config.anonymizationLevel === "full"
            ? this.anonymizeEvents(events)
            : events,
        features,
        qualityScore,
        metadata: {
          batchSize: events.length,
          timeRange: {
            start: events.reduce(
              (min, e) => (e.timestamp < min ? e.timestamp : min),
              events[0]?.timestamp || startTime.toISOString()
            ),
            end: events.reduce(
              (max, e) => (e.timestamp > max ? e.timestamp : max),
              events[0]?.timestamp || startTime.toISOString()
            ),
          },
          entityTypes: [...new Set(events.map((e) => e.entityType))],
          sources: [...new Set(events.map((e) => e.source))],
        },
      };

      // Quality gate check
      if (qualityScore >= this.config.dataQualityThreshold) {
        this.pendingBatches.push(batch);

        this.emit("pipeline:batch-ready", {
          batchId: batch.id,
          batchSize: events.length,
          qualityScore,
          timestamp: new Date(),
        });

        this.logger.debug(
          `Batch ${
            batch.id
          } ready for training (quality: ${qualityScore.toFixed(2)})`
        );
      } else {
        this.logger.warn(
          `Batch quality ${qualityScore.toFixed(2)} below threshold ${
            this.config.dataQualityThreshold
          }, discarding`
        );
        this.stats.batchesByQuality.low++;
      }

      // Update statistics
      this.stats.totalBatchesProcessed++;
      this.stats.totalEventsProcessed += events.length;
      this.qualityScores.push(qualityScore);
      this.stats.averageQualityScore =
        this.qualityScores.reduce((sum, score) => sum + score, 0) /
        this.qualityScores.length;

      // Categorize quality
      if (qualityScore >= 0.8) this.stats.batchesByQuality.high++;
      else if (qualityScore >= 0.6) this.stats.batchesByQuality.medium++;
      else this.stats.batchesByQuality.low++;

      this.stats.lastProcessingTime = new Date().toISOString();

      this.emit("pipeline:batch-processed", {
        batchId: batch.id,
        processingTimeMs: Date.now() - startTime.getTime(),
        qualityScore,
        eventsProcessed: events.length,
        featuresExtracted: Object.keys(features).length,
        timestamp: new Date(),
      });

      return batch;
    } catch (error) {
      this.stats.processingErrors++;
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error(`Error processing batch: ${errorMessage}`);

      this.emit("pipeline:error", {
        error: errorMessage,
        batchSize: events.length,
        timestamp: new Date(),
        processingTimeMs: Date.now() - startTime.getTime(),
      });

      throw error;
    }
  }

  public async processPendingBatches(): Promise<void> {
    if (this.pendingBatches.length === 0) {
      return;
    }

    const batchesToProcess = [...this.pendingBatches];
    this.pendingBatches = [];

    this.logger.info(
      `Processing ${batchesToProcess.length} pending training batches`
    );

    for (const batch of batchesToProcess) {
      try {
        await this.sendToTraining(batch);
        this.processedBatches.push(batch);

        this.emit("pipeline:batch-sent-to-training", {
          batchId: batch.id,
          batchSize: batch.events.length,
          qualityScore: batch.qualityScore,
          timestamp: new Date(),
        });
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        this.logger.error(
          `Failed to send batch ${batch.id} to training: ${errorMessage}`
        );
        // Put back in pending queue for retry
        this.pendingBatches.push(batch);

        this.emit("pipeline:training-send-failed", {
          batchId: batch.id,
          error: errorMessage,
          timestamp: new Date(),
        });
      }
    }

    // Keep only recent processed batches (memory management)
    if (this.processedBatches.length > 100) {
      this.processedBatches = this.processedBatches.slice(-100);
    }
  }

  public async flush(): Promise<void> {
    this.logger.info("Flushing all pending batches...");
    await this.processPendingBatches();
    this.logger.info("Flush complete");
  }

  private validateBatch(events: FeedbackEvent[]): boolean {
    if (!Array.isArray(events) || events.length === 0) {
      return false;
    }

    if (events.length > this.config.batchSize * 2) {
      this.logger.warn(
        `Batch size ${events.length} exceeds recommended maximum`
      );
    }

    // Check for required fields
    for (const event of events) {
      if (
        !event.id ||
        !event.source ||
        !event.type ||
        !event.entityId ||
        !event.timestamp
      ) {
        this.logger.warn(`Invalid event in batch: ${event.id || "unknown"}`);
        return false;
      }
    }

    return true;
  }

  private async extractFeatures(
    events: FeedbackEvent[]
  ): Promise<Record<string, any>> {
    const features: Record<string, any> = {};

    // Basic statistical features
    features.eventCount = events.length;
    features.uniqueEntities = new Set(events.map((e) => e.entityId)).size;
    features.uniqueEntityTypes = new Set(events.map((e) => e.entityType)).size;
    features.timeSpanHours = this.calculateTimeSpan(events);

    // Source distribution
    features.sourceDistribution = this.calculateDistribution(events, "source");

    // Type distribution
    features.typeDistribution = this.calculateDistribution(events, "type");

    // Entity type distribution
    features.entityTypeDistribution = this.calculateDistribution(
      events,
      "entityType"
    );

    // Time-based features
    if (this.config.featureEngineering.timeWindowFeatures) {
      features.hourlyDistribution = this.calculateHourlyDistribution(events);
      features.weekdayDistribution = this.calculateWeekdayDistribution(events);
    }

    // Correlation features
    if (this.config.featureEngineering.correlationFeatures) {
      features.sourceCorrelations = this.calculateSourceCorrelations(events);
    }

    // Trend features
    if (this.config.featureEngineering.trendFeatures) {
      features.trendFeatures = this.calculateTrendFeatures(events);
    }

    // Performance-specific features
    const performanceEvents = events.filter(
      (e) => e.source === FeedbackSource.PERFORMANCE_METRICS
    );
    if (performanceEvents.length > 0) {
      features.avgLatency = this.calculateAverageMetric(
        performanceEvents,
        "latencyMs"
      );
      features.avgThroughput = this.calculateAverageMetric(
        performanceEvents,
        "throughput"
      );
      features.avgQualityScore = this.calculateAverageMetric(
        performanceEvents,
        "qualityScore"
      );
      features.errorRate = this.calculateErrorRate(events);
    }

    // User feedback features
    const ratingEvents = events.filter(
      (e) => e.source === FeedbackSource.USER_RATINGS
    );
    if (ratingEvents.length > 0) {
      features.avgUserRating = this.calculateAverageRating(ratingEvents);
      features.ratingDistribution =
        this.calculateRatingDistribution(ratingEvents);
    }

    // Task outcome features
    const taskEvents = events.filter(
      (e) => e.source === FeedbackSource.TASK_OUTCOMES
    );
    if (taskEvents.length > 0) {
      features.taskSuccessRate = this.calculateTaskSuccessRate(taskEvents);
      features.avgExecutionTime =
        this.calculateAverageExecutionTime(taskEvents);
      features.retryRate = this.calculateRetryRate(taskEvents);
    }

    return features;
  }

  private assessDataQuality(
    events: FeedbackEvent[],
    features: Record<string, any>
  ): number {
    let score = 0;
    let maxScore = 0;

    // Completeness (20 points)
    maxScore += 20;
    const completenessRatio =
      events.filter((e) => e.context && Object.keys(e.context).length > 0)
        .length / events.length;
    score += completenessRatio * 20;

    // Diversity (15 points)
    maxScore += 15;
    const sourceDiversity = Object.keys(
      features.sourceDistribution || {}
    ).length;
    const typeDiversity = Object.keys(features.typeDistribution || {}).length;
    const diversityScore =
      Math.min((sourceDiversity + typeDiversity) / 10, 1) * 15;
    score += diversityScore;

    // Timeliness (15 points)
    maxScore += 15;
    const avgEventAge = this.calculateAverageEventAge(events);
    const timelinessScore =
      Math.max(0, 1 - avgEventAge / (24 * 60 * 60 * 1000)) * 15; // Prefer events < 24h old
    score += timelinessScore;

    // Consistency (15 points)
    maxScore += 15;
    const consistencyScore = this.assessConsistency(events) * 15;
    score += consistencyScore;

    // Representativeness (15 points)
    maxScore += 15;
    const representativenessScore = this.assessRepresentativeness(events) * 15;
    score += representativenessScore;

    // Performance metrics quality (20 points)
    maxScore += 20;
    if (
      features.avgLatency !== undefined &&
      features.taskSuccessRate !== undefined
    ) {
      const perfScore = 20; // Full points if we have key performance metrics
      score += perfScore;
    }

    return Math.min(score / maxScore, 1.0);
  }

  private async sendToTraining(batch: TrainingDataBatch): Promise<void> {
    // In a real implementation, this would send to RL training system
    // For now, simulate the operation

    this.logger.info(
      `Sending batch ${batch.id} to training system (${
        batch.events.length
      } events, quality: ${batch.qualityScore.toFixed(2)})`
    );

    // Simulate network call
    await this.delay(Math.random() * 500 + 100);

    // Simulate occasional failures
    if (Math.random() < 0.05) {
      // 5% failure rate
      throw new Error("Training system temporarily unavailable");
    }

    this.logger.debug(`Successfully sent batch ${batch.id} to training`);
  }

  private anonymizeEvents(events: FeedbackEvent[]): FeedbackEvent[] {
    return events.map((event) => ({
      ...event,
      entityId: this.hashString(event.entityId),
      context: this.anonymizeContext(event.context),
    }));
  }

  private hashString(str: string): string {
    // Simple hash for anonymization (not cryptographically secure)
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }

  private anonymizeContext(context: Record<string, any>): Record<string, any> {
    const anonymized: Record<string, any> = {};

    for (const [key, value] of Object.entries(context)) {
      if (typeof value === "string" && this.isPersonalData(key)) {
        anonymized[key] = this.hashString(value);
      } else if (typeof value === "object" && value !== null) {
        anonymized[key] = this.anonymizeContext(value);
      } else {
        anonymized[key] = value;
      }
    }

    return anonymized;
  }

  private isPersonalData(key: string): boolean {
    const personalDataKeys = [
      "userId",
      "email",
      "name",
      "user_id",
      "username",
      "ip",
      "address",
    ];
    return personalDataKeys.some((pdk) => key.toLowerCase().includes(pdk));
  }

  private calculateDistribution(
    events: FeedbackEvent[],
    field: keyof FeedbackEvent
  ): Record<string, number> {
    const distribution: Record<string, number> = {};
    for (const event of events) {
      const value = event[field] as string;
      distribution[value] = (distribution[value] || 0) + 1;
    }
    return distribution;
  }

  private calculateTimeSpan(events: FeedbackEvent[]): number {
    if (events.length === 0) return 0;
    const timestamps = events.map((e) => new Date(e.timestamp).getTime());
    const minTime = Math.min(...timestamps);
    const maxTime = Math.max(...timestamps);
    return (maxTime - minTime) / (1000 * 60 * 60); // hours
  }

  private calculateHourlyDistribution(
    events: FeedbackEvent[]
  ): Record<number, number> {
    const distribution: Record<number, number> = {};
    for (const event of events) {
      const hour = new Date(event.timestamp).getHours();
      distribution[hour] = (distribution[hour] || 0) + 1;
    }
    return distribution;
  }

  private calculateWeekdayDistribution(
    events: FeedbackEvent[]
  ): Record<number, number> {
    const distribution: Record<number, number> = {};
    for (const event of events) {
      const weekday = new Date(event.timestamp).getDay();
      distribution[weekday] = (distribution[weekday] || 0) + 1;
    }
    return distribution;
  }

  private calculateSourceCorrelations(
    events: FeedbackEvent[]
  ): Record<string, number> {
    // Simplified correlation calculation
    const correlations: Record<string, number> = {};
    const sources = Object.keys(this.calculateDistribution(events, "source"));

    for (let i = 0; i < sources.length; i++) {
      for (let j = i + 1; j < sources.length; j++) {
        const source1 = sources[i];
        const source2 = sources[j];
        const correlation = this.calculateCorrelationForSources(
          events,
          source1 as FeedbackSource,
          source2 as FeedbackSource
        );
        correlations[`${source1}_${source2}`] = correlation;
      }
    }

    return correlations;
  }

  private calculateCorrelationForSources(
    events: FeedbackEvent[],
    source1: FeedbackSource,
    source2: FeedbackSource
  ): number {
    const source1Events = events.filter((e) => e.source === source1);
    const source2Events = events.filter((e) => e.source === source2);

    if (source1Events.length === 0 || source2Events.length === 0) return 0;

    // Simple co-occurrence correlation
    const cooccurring = events.filter(
      (e) => e.source === source1 || e.source === source2
    ).length;

    const expectedCooccurrence =
      (source1Events.length + source2Events.length) / 2;
    return (cooccurring - expectedCooccurrence) / expectedCooccurrence;
  }

  private calculateTrendFeatures(events: FeedbackEvent[]): Record<string, any> {
    const sortedEvents = events.sort(
      (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    );

    const trends: Record<string, any> = {
      overallTrend: this.calculateSimpleTrend(sortedEvents),
      recentActivity: sortedEvents.slice(-10).length, // Activity in last 10 events
    };

    // Source-specific trends
    const sources = Object.keys(
      this.calculateDistribution(sortedEvents, "source")
    );
    trends.sourceTrends = {};
    for (const source of sources) {
      const sourceEvents = sortedEvents.filter((e) => e.source === source);
      trends.sourceTrends[source] = this.calculateSimpleTrend(sourceEvents);
    }

    return trends;
  }

  private calculateSimpleTrend(
    events: FeedbackEvent[]
  ): "increasing" | "decreasing" | "stable" {
    if (events.length < 3) return "stable";

    const firstHalf = events.slice(0, Math.floor(events.length / 2));
    const secondHalf = events.slice(Math.floor(events.length / 2));

    const firstHalfAvg = firstHalf.length;
    const secondHalfAvg = secondHalf.length;

    const change = (secondHalfAvg - firstHalfAvg) / firstHalfAvg;

    if (change > 0.1) return "increasing";
    if (change < -0.1) return "decreasing";
    return "stable";
  }

  private calculateAverageMetric(
    events: FeedbackEvent[],
    metricPath: string
  ): number {
    const values: number[] = [];
    for (const event of events) {
      const value = this.extractNestedValue(event.value, metricPath);
      if (typeof value === "number" && !isNaN(value)) {
        values.push(value);
      }
    }
    return values.length > 0
      ? values.reduce((sum, v) => sum + v, 0) / values.length
      : 0;
  }

  private calculateErrorRate(events: FeedbackEvent[]): number {
    const totalOutcomes = events.filter(
      (e) => e.source === FeedbackSource.TASK_OUTCOMES
    ).length;
    if (totalOutcomes === 0) return 0;

    const errorOutcomes = events.filter(
      (e) =>
        e.source === FeedbackSource.TASK_OUTCOMES &&
        !(e.value as any).outcome?.success
    ).length;

    return errorOutcomes / totalOutcomes;
  }

  private calculateAverageRating(events: FeedbackEvent[]): number {
    const ratings = events
      .map((e) => (e.value as any).rating)
      .filter((r) => typeof r === "number");
    return ratings.length > 0
      ? ratings.reduce((sum, r) => sum + r, 0) / ratings.length
      : 0;
  }

  private calculateRatingDistribution(
    events: FeedbackEvent[]
  ): Record<number, number> {
    const distribution: Record<number, number> = {};
    for (const event of events) {
      const rating = (event.value as any).rating;
      if (typeof rating === "number") {
        distribution[rating] = (distribution[rating] || 0) + 1;
      }
    }
    return distribution;
  }

  private calculateTaskSuccessRate(events: FeedbackEvent[]): number {
    const successful = events.filter(
      (e) => (e.value as any).outcome?.success
    ).length;
    return events.length > 0 ? successful / events.length : 0;
  }

  private calculateAverageExecutionTime(events: FeedbackEvent[]): number {
    const times = events
      .map((e) => (e.value as any).executionTimeMs)
      .filter((t) => typeof t === "number");
    return times.length > 0
      ? times.reduce((sum, t) => sum + t, 0) / times.length
      : 0;
  }

  private calculateRetryRate(events: FeedbackEvent[]): number {
    const totalRetries = events.reduce(
      (sum, e) => sum + ((e.value as any).retryCount || 0),
      0
    );
    return events.length > 0 ? totalRetries / events.length : 0;
  }

  private calculateAverageEventAge(events: FeedbackEvent[]): number {
    const now = Date.now();
    const ages = events.map((e) => now - new Date(e.timestamp).getTime());
    return ages.length > 0
      ? ages.reduce((sum, age) => sum + age, 0) / ages.length
      : 0;
  }

  private assessConsistency(events: FeedbackEvent[]): number {
    // Check for consistent data formats, reasonable value ranges, etc.
    let consistentEvents = 0;

    for (const event of events) {
      let isConsistent = true;

      // Check timestamp validity
      try {
        const timestamp = new Date(event.timestamp);
        if (isNaN(timestamp.getTime())) isConsistent = false;
      } catch {
        isConsistent = false;
      }

      // Check value ranges for known metrics
      if (event.source === FeedbackSource.PERFORMANCE_METRICS) {
        const metrics = event.value as any;
        if (
          metrics.latencyMs !== undefined &&
          (metrics.latencyMs < 0 || metrics.latencyMs > 3600000)
        ) {
          isConsistent = false; // Unreasonable latency
        }
      }

      if (isConsistent) consistentEvents++;
    }

    return events.length > 0 ? consistentEvents / events.length : 0;
  }

  private assessRepresentativeness(events: FeedbackEvent[]): number {
    // Check if the batch represents a good mix of sources, entity types, etc.
    const sources = new Set(events.map((e) => e.source));
    const entityTypes = new Set(events.map((e) => e.entityType));

    const sourceCoverage = sources.size / Object.keys(FeedbackSource).length;
    const entityTypeCoverage = Math.min(entityTypes.size / 3, 1); // Assume 3 main entity types

    return (sourceCoverage + entityTypeCoverage) / 2;
  }

  private extractNestedValue(obj: any, path: string): any {
    return path.split(".").reduce((current, key) => current?.[key], obj);
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  public getStats() {
    return {
      ...this.stats,
      pendingBatches: this.pendingBatches.length,
      processedBatches: this.processedBatches.length,
    };
  }

  public clearProcessedBatches(): void {
    this.processedBatches = [];
    this.logger.info("Cleared processed batches");
  }
}
