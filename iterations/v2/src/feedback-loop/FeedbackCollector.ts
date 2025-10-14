import { EventEmitter } from "events";
import { v4 as uuidv4 } from "uuid";
import { ConfigManager } from "../config/ConfigManager";
import { Logger } from "../observability/Logger";
import { TaskOutcome } from "../types/agentic-rl";
import { ConstitutionalViolation } from "../types/caws-constitutional";
import { ComponentHealth, HealthStatus } from "../types/coordinator";
import {
  AgentFeedback,
  ComponentHealthFeedback,
  ConstitutionalViolationFeedback,
  FeedbackCollectionConfig,
  FeedbackEvent,
  FeedbackProcessingResult,
  FeedbackSource,
  FeedbackType,
  PerformanceFeedback,
  PerformanceMetrics,
  RoutingDecisionFeedback,
  SystemEventFeedback,
  TaskOutcomeData,
  TaskOutcomeFeedback,
  UserRatingData,
  UserRatingFeedback,
} from "../types/feedback-loop";

export class FeedbackCollector extends EventEmitter {
  private config: FeedbackCollectionConfig;
  private logger: Logger;

  private eventBuffer: FeedbackEvent[] = [];
  private flushTimer: ReturnType<typeof setInterval> | null = null;
  private isRunning: boolean = false;

  // Statistics
  private stats = {
    totalEvents: 0,
    eventsBySource: {} as Record<FeedbackSource, number>,
    eventsByType: {} as Record<FeedbackType, number>,
    droppedEvents: 0,
    processingErrors: 0,
    lastFlushTime: undefined as string | undefined,
  };

  constructor(configManager: ConfigManager) {
    super();
    this.config = configManager.get("feedbackLoop.collection");
    this.logger = new Logger("FeedbackCollector");
  }

  public start(): void {
    if (this.isRunning) {
      this.logger.warn("FeedbackCollector is already running");
      return;
    }

    this.isRunning = true;
    this.scheduleFlush();
    this.logger.info("FeedbackCollector started", {
      batchSize: this.config.batchSize,
      flushIntervalMs: this.config.flushIntervalMs,
    });
  }

  public stop(): void {
    if (!this.isRunning) {
      this.logger.warn("FeedbackCollector is not running");
      return;
    }

    this.isRunning = false;
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }

    // Final flush
    this.flushBuffer();
    this.logger.info("FeedbackCollector stopped");
  }

  public collectPerformanceMetrics(
    entityId: string,
    entityType: string,
    metrics: PerformanceMetrics,
    context: Record<string, any> = {}
  ): void {
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.PERFORMANCE_METRICS,
      type: FeedbackType.NUMERIC_METRIC,
      entityId,
      entityType,
      timestamp: new Date().toISOString(),
      value: metrics,
      context: {
        ...context,
        collectionTime: Date.now(),
      },
    } as PerformanceFeedback);
  }

  public collectTaskOutcome(
    taskId: string,
    outcome: TaskOutcome,
    executionTimeMs: number,
    retryCount: number,
    errorDetails?: string,
    context: Record<string, any> = {}
  ): void {
    const value: TaskOutcomeData = {
      outcome,
      executionTimeMs,
      retryCount,
      errorDetails,
    };
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.TASK_OUTCOMES,
      type: FeedbackType.BINARY_OUTCOME,
      entityId: taskId,
      entityType: "task",
      timestamp: new Date().toISOString(),
      value,
      context: {
        ...context,
        taskCompletion: outcome.success,
        hasErrors: !!errorDetails,
      },
    } as TaskOutcomeFeedback);
  }

  public collectUserRating(
    entityId: string,
    entityType: string,
    rating: number,
    criteria: {
      accuracy: number;
      speed: number;
      reliability: number;
      communication: number;
    },
    comments?: string,
    context: Record<string, any> = {}
  ): void {
    const value: UserRatingData = { rating, criteria, comments };
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.USER_RATINGS,
      type: FeedbackType.RATING_SCALE,
      entityId,
      entityType,
      timestamp: new Date().toISOString(),
      value,
      context: {
        ...context,
        userId: context.userId || "anonymous",
        ratingScale: "1-5",
      },
    } as UserRatingFeedback);
  }

  public collectSystemEvent(
    eventType: string,
    severity: "low" | "medium" | "high" | "critical",
    description: string,
    impact: {
      affectedComponents: string[];
      estimatedDowntimeMinutes?: number;
      userImpact: "none" | "minor" | "major" | "critical";
    },
    context: Record<string, any> = {}
  ): void {
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.SYSTEM_EVENTS,
      type: FeedbackType.CATEGORICAL_EVENT,
      entityId: eventType,
      entityType: "system",
      timestamp: new Date().toISOString(),
      value: { eventType, severity, description, impact },
      context: {
        ...context,
        systemComponent: context.component || "unknown",
      },
    } as SystemEventFeedback);
  }

  public collectConstitutionalViolation(
    violation: ConstitutionalViolation,
    policyImpact: {
      affectedTasks: number;
      complianceScoreDelta: number;
      riskLevel: "low" | "medium" | "high";
    },
    context: Record<string, any> = {}
  ): void {
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.CONSTITUTIONAL_VIOLATIONS,
      type: FeedbackType.CATEGORICAL_EVENT,
      entityId: (violation as any).taskId || "unknown",
      entityType: "task",
      timestamp: new Date().toISOString(),
      value: { violation, policyImpact },
      context: {
        ...context,
        policyId: (violation as any).policyId,
        severity: (violation as any).severity,
      },
    } as ConstitutionalViolationFeedback);
  }

  public collectComponentHealth(
    health: ComponentHealth,
    previousStatus?: HealthStatus,
    statusChangeReason?: string,
    context: Record<string, any> = {}
  ): void {
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.COMPONENT_HEALTH,
      type: FeedbackType.CATEGORICAL_EVENT,
      entityId: health.id,
      entityType: "component",
      timestamp: new Date().toISOString(),
      value: { health, previousStatus, statusChangeReason },
      context: {
        ...context,
        healthStatus: health.status,
        responseTime: health.responseTime,
      },
    } as ComponentHealthFeedback);
  }

  public collectRoutingDecision(
    decision: {
      taskId: string;
      selectedAgentId: string;
      routingStrategy: string;
      confidence: number;
      alternativesCount: number;
      routingTimeMs: number;
    },
    outcome?: {
      success: boolean;
      executionTimeMs?: number;
      qualityScore?: number;
    },
    context: Record<string, any> = {}
  ): void {
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.ROUTING_DECISIONS,
      type: FeedbackType.CATEGORICAL_EVENT,
      entityId: decision.taskId,
      entityType: "task",
      timestamp: new Date().toISOString(),
      value: { decision, outcome },
      context: {
        ...context,
        selectedAgentId: decision.selectedAgentId,
        routingStrategy: decision.routingStrategy,
        confidence: decision.confidence,
      },
    } as RoutingDecisionFeedback);
  }

  public collectAgentFeedback(
    agentId: string,
    feedbackType:
      | "performance"
      | "capability"
      | "reliability"
      | "communication",
    rating: number,
    details?: string,
    suggestedImprovements?: string[],
    context: Record<string, any> = {}
  ): void {
    const feedback = {
      agentId,
      feedbackType,
      rating,
      details,
      suggestedImprovements,
    };
    this.collectEvent({
      id: uuidv4(),
      source: FeedbackSource.AGENT_FEEDBACK,
      type: FeedbackType.RATING_SCALE,
      entityId: agentId,
      entityType: "agent",
      timestamp: new Date().toISOString(),
      value: feedback,
      context: {
        ...context,
        feedbackType,
        rating,
      },
    } as AgentFeedback);
  }

  private collectEvent(event: FeedbackEvent): void {
    // Check if source is enabled
    if (!this.config.enabledSources.includes(event.source)) {
      this.stats.droppedEvents++;
      return;
    }

    // Apply sampling for high-volume sources
    if (
      this.config.samplingRate < 1.0 &&
      Math.random() > this.config.samplingRate
    ) {
      this.stats.droppedEvents++;
      return;
    }

    // Apply filters
    if (this.shouldFilterEvent(event)) {
      this.stats.droppedEvents++;
      return;
    }

    // Validate event
    if (!this.validateEvent(event)) {
      this.stats.processingErrors++;
      this.logger.warn("Invalid feedback event", {
        eventId: event.id,
        source: event.source,
      });
      return;
    }

    // Add to buffer
    this.eventBuffer.push(event);
    this.stats.totalEvents++;
    this.stats.eventsBySource[event.source] =
      (this.stats.eventsBySource[event.source] || 0) + 1;
    this.stats.eventsByType[event.type] =
      (this.stats.eventsByType[event.type] || 0) + 1;

    // Emit collection event
    this.emit("feedback:collected", event);

    // Check if buffer should be flushed
    if (this.eventBuffer.length >= this.config.batchSize) {
      this.flushBuffer();
    }
  }

  private shouldFilterEvent(event: FeedbackEvent): boolean {
    // Filter by entity type
    if (this.config.filters.excludeEntityTypes?.includes(event.entityType)) {
      return true;
    }

    // Filter by minimum severity for system events
    if (event.source === FeedbackSource.SYSTEM_EVENTS) {
      const severityLevels = { low: 1, medium: 2, high: 3, critical: 4 };
      const minSeverityLevel =
        severityLevels[
          this.config.filters.minSeverity as keyof typeof severityLevels
        ] || 0;
      const eventSeverity =
        severityLevels[
          (event.value as any).severity as keyof typeof severityLevels
        ] || 0;
      if (eventSeverity < minSeverityLevel) {
        return true;
      }
    }

    // Filter to recent events only
    if (this.config.filters.includeOnlyRecent) {
      const eventTime = new Date(event.timestamp).getTime();
      const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000;
      if (eventTime < oneDayAgo) {
        return true;
      }
    }

    return false;
  }

  private validateEvent(event: FeedbackEvent): boolean {
    try {
      // Basic validation
      if (
        !event.id ||
        !event.source ||
        !event.type ||
        !event.entityId ||
        !event.timestamp
      ) {
        return false;
      }

      // Timestamp validation
      const timestamp = new Date(event.timestamp);
      if (isNaN(timestamp.getTime())) {
        return false;
      }

      // Source-specific validation
      switch (event.source) {
        case FeedbackSource.PERFORMANCE_METRICS:
          return this.validatePerformanceMetrics(event as PerformanceFeedback);
        case FeedbackSource.USER_RATINGS:
          return this.validateUserRating(event as UserRatingFeedback);
        case FeedbackSource.TASK_OUTCOMES:
          return this.validateTaskOutcome(event as TaskOutcomeFeedback);
        default:
          return true; // Basic validation passed
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.logger.error("Event validation error", {
        error: errorMessage,
        eventId: event.id,
      });
      return false;
    }
  }

  private validatePerformanceMetrics(event: PerformanceFeedback): boolean {
    const metrics = event.value;
    if (
      metrics.latencyMs !== undefined &&
      (metrics.latencyMs < 0 || !isFinite(metrics.latencyMs))
    ) {
      return false;
    }
    if (
      metrics.throughput !== undefined &&
      (metrics.throughput < 0 || !isFinite(metrics.throughput))
    ) {
      return false;
    }
    if (
      metrics.errorRate !== undefined &&
      (metrics.errorRate < 0 || metrics.errorRate > 1)
    ) {
      return false;
    }
    return true;
  }

  private validateUserRating(event: UserRatingFeedback): boolean {
    if (
      event.value.rating < 1 ||
      event.value.rating > 5 ||
      !Number.isInteger(event.value.rating)
    ) {
      return false;
    }
    const criteria = event.value.criteria;
    const validCriteria = ["accuracy", "speed", "reliability", "communication"];
    for (const key of validCriteria) {
      const value = (criteria as any)[key];
      if (value < 1 || value > 5 || !Number.isInteger(value)) {
        return false;
      }
    }
    return true;
  }

  private validateTaskOutcome(event: TaskOutcomeFeedback): boolean {
    if (
      event.value.executionTimeMs < 0 ||
      !isFinite(event.value.executionTimeMs)
    ) {
      return false;
    }
    if (
      event.value.retryCount < 0 ||
      !Number.isInteger(event.value.retryCount)
    ) {
      return false;
    }
    return true;
  }

  private scheduleFlush(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }
    this.flushTimer = setInterval(() => {
      this.flushBuffer();
    }, this.config.flushIntervalMs);
  }

  private flushBuffer(): void {
    if (this.eventBuffer.length === 0) {
      return;
    }

    const batch = [...this.eventBuffer];
    this.eventBuffer = [];
    this.stats.lastFlushTime = new Date().toISOString();

    // Emit batch for processing
    this.emit("feedback:batch-ready", batch);

    this.logger.debug(`Flushed ${batch.length} feedback events`);
  }

  public getStats() {
    return {
      ...this.stats,
      bufferSize: this.eventBuffer.length,
      isRunning: this.isRunning,
    };
  }

  public clearStats(): void {
    this.stats = {
      totalEvents: 0,
      eventsBySource: {} as Record<FeedbackSource, number>,
      eventsByType: {} as Record<FeedbackType, number>,
      droppedEvents: 0,
      processingErrors: 0,
      lastFlushTime: this.stats.lastFlushTime,
    };
  }

  public async processBatch(
    batch: FeedbackEvent[]
  ): Promise<FeedbackProcessingResult> {
    const startTime = Date.now();
    let processedEvents = 0;
    const errors: string[] = [];
    const recommendations: any[] = []; // We'll define this properly later

    try {
      // Process batch (in real implementation, this would persist to database)
      for (const event of batch) {
        try {
          // Basic processing - in real system would do validation, enrichment, etc.
          processedEvents++;
        } catch (error) {
          const errorMessage =
            error instanceof Error ? error.message : String(error);
          errors.push(`Failed to process event ${event.id}: ${errorMessage}`);
        }
      }

      const processingTimeMs = Date.now() - startTime;
      const qualityScore =
        errors.length === 0
          ? 1.0
          : Math.max(0, 1.0 - errors.length / batch.length);

      return {
        success: errors.length === 0,
        processedEvents,
        analysisPerformed: false, // This would be done by FeedbackAnalyzer
        recommendations,
        errors,
        processingTimeMs,
        qualityScore,
      };
    } catch (error) {
      return {
        success: false,
        processedEvents: 0,
        analysisPerformed: false,
        recommendations: [],
        errors: [error instanceof Error ? error.message : String(error)],
        processingTimeMs: Date.now() - startTime,
        qualityScore: 0,
      };
    }
  }
}
