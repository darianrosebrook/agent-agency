/**
 * Data Collector for Real-Time Performance Metric Collection
 *
 * @author @darianrosebrook
 * @module data-collector
 *
 * Collects comprehensive performance metrics from all agent interactions
 * in real-time with minimal performance impact and guaranteed data integrity.
 */

import { createHash } from "crypto";
import { EventEmitter } from "events";
import { Timestamp } from "../types/agent-registry";
import {
  AccuracyMetrics,
  ComplianceMetrics,
  CostMetrics,
  DataCollectionConfig,
  LatencyMetrics,
  PerformanceEvent,
  PerformanceEventType,
  PerformanceMetrics,
  ReliabilityMetrics,
  ResourceMetrics,
} from "../types/performance-tracking";

/**
 * Default data collection configuration.
 */
const DEFAULT_CONFIG: DataCollectionConfig = {
  enabled: true,
  samplingRate: 1.0, // Collect all events
  maxBufferSize: 10000,
  batchSize: 100,
  retentionDays: 90,
  anonymization: {
    enabled: true,
    level: "differential",
    preserveAgentIds: true,
    preserveTaskTypes: true,
  },
};

/**
 * Internal buffer entry for performance data.
 */
interface BufferEntry {
  event: PerformanceEvent;
  collectedAt: Timestamp;
  priority: "low" | "normal" | "high" | "critical";
}

/**
 * Data Collector for real-time performance metric collection.
 *
 * This component captures performance data from all agent interactions with:
 * - Minimal latency impact (< 1ms collection time)
 * - Guaranteed data integrity through cryptographic hashing
 * - Configurable sampling and anonymization
 * - Event-driven architecture for loose coupling
 */
export class DataCollector extends EventEmitter {
  private config: DataCollectionConfig;
  private buffer: BufferEntry[] = [];
  private isCollecting = false;
  private collectionStats = {
    eventsCollected: 0,
    eventsDropped: 0,
    averageCollectionTimeMs: 0,
    lastCollectionTime: 0,
  };

  /**
   * Creates a new Data Collector instance.
   *
   * @param config - Collection configuration. Uses defaults if not provided.
   */
  constructor(config: Partial<DataCollectionConfig> = {}) {
    super();
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.setupEventHandlers();
  }

  /**
   * Starts data collection.
   */
  startCollection(): void {
    if (this.config.enabled) {
      this.isCollecting = true;
      this.emit("collection_started");
    }
  }

  /**
   * Stops data collection.
   */
  stopCollection(): void {
    this.isCollecting = false;
    this.emit("collection_stopped");
  }

  /**
   * Records a task execution start event.
   *
   * @param taskId - Task identifier
   * @param agentId - Agent identifier
   * @param context - Additional execution context
   * @returns Event ID for correlation
   */
  recordTaskStart(
    taskId: string,
    agentId: string,
    context?: Record<string, unknown>
  ): string {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return "";
    }

    const eventId = this.generateEventId();
    const event: PerformanceEvent = {
      id: eventId,
      type: PerformanceEventType.TASK_EXECUTION_START,
      timestamp: new Date().toISOString(),
      agentId,
      taskId,
      metrics: {},
      context: this.anonymizeContext(context),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    this.addToBuffer(event, "normal");

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();

    return eventId;
  }

  /**
   * Records a task execution completion event.
   *
   * @param taskId - Task identifier
   * @param agentId - Agent identifier
   * @param metrics - Performance metrics from execution
   * @param context - Additional execution context
   */
  recordTaskCompletion(
    taskId: string,
    agentId: string,
    metrics: Partial<PerformanceMetrics>,
    context?: Record<string, unknown>
  ): void {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return;
    }

    const event: PerformanceEvent = {
      id: this.generateEventId(),
      type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
      timestamp: new Date().toISOString(),
      agentId,
      taskId,
      metrics: this.normalizeMetrics(metrics),
      context: this.anonymizeContext(context),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    this.addToBuffer(event, "high");

    // Emit completion event for real-time processing
    this.emit("task_completed", event);

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();
  }

  /**
   * Records an agent registration event for baseline performance tracking.
   *
   * @param agentId - Agent identifier
   * @param agentData - Agent registration data
   */
  recordAgentRegistration(
    agentId: string,
    agentData: {
      capabilities: string[];
      baselineMetrics: {
        latencyMs: number;
        accuracy: number;
        costPerTask: number;
        reliability: number;
      };
      registrationTimestamp: string;
    }
  ): void {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return;
    }

    const event: PerformanceEvent = {
      id: this.generateEventId(),
      type: PerformanceEventType.AGENT_REGISTRATION,
      timestamp: agentData.registrationTimestamp,
      agentId,
      metrics: {
        capabilities: agentData.capabilities,
        baselineLatencyMs: agentData.baselineMetrics.latencyMs,
        baselineAccuracy: agentData.baselineMetrics.accuracy,
        baselineCostPerTask: agentData.baselineMetrics.costPerTask,
        baselineReliability: agentData.baselineMetrics.reliability,
      },
      context: this.anonymizeContext({
        eventType: "agent_registration",
        capabilities: agentData.capabilities,
      }),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    this.addToBuffer(event, "normal");

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();
  }

  /**
   * Records an agent status change event for availability tracking.
   *
   * @param agentId - Agent identifier
   * @param status - New availability status
   * @param context - Status change context
   */
  recordAgentStatusChange(
    agentId: string,
    status: "available" | "busy" | "offline" | "maintenance",
    context: { previousStatus?: string; reason?: string }
  ): void {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return;
    }

    const event: PerformanceEvent = {
      id: this.generateEventId(),
      type: PerformanceEventType.AGENT_STATUS_CHANGE,
      timestamp: new Date().toISOString(),
      agentId,
      metrics: {
        status,
        previousStatus: context.previousStatus,
        reason: context.reason,
      },
      context: this.anonymizeContext({
        eventType: "agent_status_change",
        status,
        previousStatus: context.previousStatus,
        reason: context.reason,
      }),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    this.addToBuffer(event, "normal");

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();
  }

  /**
   * Records a routing decision event.
   *
   * @param taskId - Task identifier
   * @param selectedAgentId - Selected agent identifier
   * @param alternatives - Alternative agents considered
   * @param routingContext - Routing decision context
   */
  recordRoutingDecision(
    taskId: string,
    selectedAgentId: string,
    alternatives: Array<{ agentId: string; score: number }>,
    routingContext?: Record<string, unknown>
  ): void {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return;
    }

    const event: PerformanceEvent = {
      id: this.generateEventId(),
      type: PerformanceEventType.ROUTING_DECISION,
      timestamp: new Date().toISOString(),
      agentId: selectedAgentId,
      taskId,
      metrics: {},
      context: this.anonymizeContext({
        alternatives,
        ...routingContext,
      }),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    this.addToBuffer(event, "high");

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();
  }

  /**
   * Records an evaluation outcome event.
   *
   * @param taskId - Task identifier
   * @param agentId - Agent identifier
   * @param evaluationScore - Overall evaluation score (0-1)
   * @param evaluationDetails - Detailed evaluation results
   */
  recordEvaluationOutcome(
    taskId: string,
    agentId: string,
    evaluationScore: number,
    evaluationDetails?: Record<string, unknown>
  ): void {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return;
    }

    const accuracyMetrics: Partial<AccuracyMetrics> = {
      qualityScore: evaluationScore,
      evaluationScore,
    };

    const event: PerformanceEvent = {
      id: this.generateEventId(),
      type: PerformanceEventType.EVALUATION_OUTCOME,
      timestamp: new Date().toISOString(),
      agentId,
      taskId,
      metrics: {
        accuracy: accuracyMetrics as AccuracyMetrics,
      },
      context: this.anonymizeContext(evaluationDetails),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    this.addToBuffer(event, "normal");

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();
  }

  /**
   * Records a constitutional validation event.
   *
   * @param validationData - Complete CAWS validation result data
   */
  recordConstitutionalValidation(validationData: {
    taskId: string;
    agentId: string;
    validationResult: {
      valid: boolean;
      violations: Array<{
        severity: "low" | "medium" | "high" | "critical";
        message: string;
        rule?: string;
      }>;
      complianceScore: number;
      processingTimeMs: number;
      ruleCount: number;
    };
  }): void {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return;
    }

    const complianceMetrics: Partial<ComplianceMetrics> = {
      validationPassRate: validationData.validationResult.valid ? 1 : 0,
      violationSeverityScore: validationData.validationResult.violations.reduce(
        (score, violation) => {
          const severityScore =
            { low: 1, medium: 2, high: 3, critical: 4 }[violation.severity] ||
            0;
          return score + severityScore;
        },
        0
      ),
      complianceScore: validationData.validationResult.complianceScore,
    };

    const event: PerformanceEvent = {
      id: this.generateEventId(),
      type: PerformanceEventType.CONSTITUTIONAL_VALIDATION,
      timestamp: new Date().toISOString(),
      agentId: validationData.agentId,
      taskId: validationData.taskId,
      metrics: {
        compliance: complianceMetrics as ComplianceMetrics,
        latency: {
          totalTimeMs: validationData.validationResult.processingTimeMs,
        } as any,
      },
      context: this.anonymizeContext({
        violations: validationData.validationResult.violations,
        ruleCount: validationData.validationResult.ruleCount,
        eventType: "constitutional_validation",
      }),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    this.addToBuffer(event, "high");

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();
  }

  /**
   * Records a system performance anomaly.
   *
   * @param anomalyType - Type of anomaly detected
   * @param severity - Anomaly severity
   * @param affectedAgentId - Agent affected (if applicable)
   * @param anomalyContext - Additional anomaly context
   */
  recordAnomaly(
    anomalyType: string,
    severity: "low" | "medium" | "high" | "critical",
    affectedAgentId?: string,
    anomalyContext?: Record<string, unknown>
  ): void {
    const startTime = performance.now();

    if (!this.shouldCollect()) {
      return;
    }

    const event: PerformanceEvent = {
      id: this.generateEventId(),
      type: PerformanceEventType.ANOMALY_DETECTED,
      timestamp: new Date().toISOString(),
      agentId: affectedAgentId,
      metrics: {},
      context: this.anonymizeContext({
        anomalyType,
        severity,
        ...anomalyContext,
      }),
      integrityHash: "",
    };

    event.integrityHash = this.calculateIntegrityHash(event);

    // Anomalies are always high priority
    this.addToBuffer(event, "critical");

    this.emit("anomaly_detected", event);

    this.collectionStats.lastCollectionTime = performance.now() - startTime;
    this.updateAverageCollectionTime();
  }

  /**
   * Retrieves pending events for processing.
   *
   * @param maxEvents - Maximum number of events to retrieve
   * @returns Array of pending events
   */
  getPendingEvents(
    maxEvents: number = this.config.batchSize
  ): PerformanceEvent[] {
    // Sort by priority (critical first) and then by timestamp
    const sortedBuffer = this.buffer.sort((a, b) => {
      const priorityOrder = { critical: 4, high: 3, normal: 2, low: 1 };
      const priorityDiff =
        priorityOrder[b.priority] - priorityOrder[a.priority];
      if (priorityDiff !== 0) return priorityDiff;

      return (
        new Date(a.event.timestamp).getTime() -
        new Date(b.event.timestamp).getTime()
      );
    });

    const events = sortedBuffer.slice(0, maxEvents).map((entry) => entry.event);

    // Remove retrieved events from buffer
    this.buffer = sortedBuffer.slice(maxEvents);

    return events;
  }

  /**
   * Gets current collection statistics.
   */
  getStats() {
    return {
      ...this.collectionStats,
      bufferSize: this.buffer.length,
      isCollecting: this.isCollecting,
      config: this.config,
    };
  }

  /**
   * Updates collection configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<DataCollectionConfig>): void {
    this.config = { ...this.config, ...config };
    this.emit("config_updated", this.config);
  }

  /**
   * Clears the event buffer.
   */
  clearBuffer(): void {
    this.buffer = [];
    this.emit("buffer_cleared");
  }

  /**
   * Determines if data collection should occur based on configuration and sampling.
   */
  private shouldCollect(): boolean {
    return (
      this.isCollecting &&
      this.config.enabled &&
      Math.random() < this.config.samplingRate
    );
  }

  /**
   * Adds an event to the buffer with size management.
   */
  private addToBuffer(
    event: PerformanceEvent,
    priority: BufferEntry["priority"]
  ): void {
    const entry: BufferEntry = {
      event,
      collectedAt: new Date().toISOString(),
      priority,
    };

    this.buffer.push(entry);
    this.collectionStats.eventsCollected++;

    // Manage buffer size
    if (this.buffer.length > this.config.maxBufferSize) {
      // Remove oldest low-priority events first
      const lowPriorityEntries = this.buffer.filter(
        (e) => e.priority === "low"
      );
      if (lowPriorityEntries.length > 0) {
        const entryToRemove = lowPriorityEntries[0];
        const index = this.buffer.indexOf(entryToRemove);
        this.buffer.splice(index, 1);
        this.collectionStats.eventsDropped++;
      } else {
        // If no low-priority events, remove oldest event
        this.buffer.shift();
        this.collectionStats.eventsDropped++;
      }
    }

    // Emit buffer full warning if approaching capacity
    if (this.buffer.length > this.config.maxBufferSize * 0.9) {
      this.emit("buffer_high_water_mark", this.buffer.length);
    }
  }

  /**
   * Generates a unique event identifier.
   */
  private generateEventId(): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    return `perf_${timestamp}_${random}`;
  }

  /**
   * Calculates integrity hash for data integrity verification.
   */
  private calculateIntegrityHash(
    event: Omit<PerformanceEvent, "integrityHash">
  ): string {
    const data = JSON.stringify({
      id: event.id,
      type: event.type,
      timestamp: event.timestamp,
      agentId: event.agentId,
      taskId: event.taskId,
      metrics: event.metrics,
      context: event.context,
    });

    return createHash("sha256").update(data).digest("hex");
  }

  /**
   * Applies anonymization to context data based on configuration.
   */
  private anonymizeContext(
    context?: Record<string, unknown>
  ): Record<string, unknown> | undefined {
    if (!context || !this.config.anonymization.enabled) {
      return context;
    }

    const anonymized = { ...context };

    // Apply anonymization based on level
    switch (this.config.anonymization.level) {
      case "secure":
        // Remove all identifying information
        this.removeIdentifyingFields(anonymized);
        this.hashSensitiveFields(anonymized);
        break;
      case "differential":
        // Apply differential privacy techniques
        this.applyDifferentialPrivacy(anonymized);
        break;
      case "basic":
        // Basic anonymization
        this.hashSensitiveFields(anonymized);
        break;
    }

    return anonymized;
  }

  /**
   * Normalizes partial metrics to complete metrics structure.
   */
  private normalizeMetrics(
    partialMetrics: Partial<PerformanceMetrics>
  ): PerformanceMetrics {
    return {
      latency: this.normalizeLatencyMetrics(partialMetrics.latency),
      accuracy: this.normalizeAccuracyMetrics(partialMetrics.accuracy),
      resources: this.normalizeResourceMetrics(partialMetrics.resources),
      compliance: this.normalizeComplianceMetrics(partialMetrics.compliance),
      cost: this.normalizeCostMetrics(partialMetrics.cost),
      reliability: this.normalizeReliabilityMetrics(partialMetrics.reliability),
    };
  }

  private normalizeLatencyMetrics(
    metrics?: Partial<LatencyMetrics>
  ): LatencyMetrics {
    return {
      averageMs: metrics?.averageMs || 0,
      p95Ms: metrics?.p95Ms || 0,
      p99Ms: metrics?.p99Ms || 0,
      minMs: metrics?.minMs || 0,
      maxMs: metrics?.maxMs || 0,
    };
  }

  private normalizeAccuracyMetrics(
    metrics?: Partial<AccuracyMetrics>
  ): AccuracyMetrics {
    return {
      successRate: metrics?.successRate || 0,
      qualityScore: metrics?.qualityScore || 0,
      violationRate: metrics?.violationRate || 0,
      evaluationScore: metrics?.evaluationScore || 0,
    };
  }

  private normalizeResourceMetrics(
    metrics?: Partial<ResourceMetrics>
  ): ResourceMetrics {
    return {
      cpuUtilizationPercent: metrics?.cpuUtilizationPercent || 0,
      memoryUtilizationPercent: metrics?.memoryUtilizationPercent || 0,
      networkIoKbps: metrics?.networkIoKbps || 0,
      diskIoKbps: metrics?.diskIoKbps || 0,
    };
  }

  private normalizeComplianceMetrics(
    metrics?: Partial<ComplianceMetrics>
  ): ComplianceMetrics {
    return {
      validationPassRate: metrics?.validationPassRate || 0,
      violationSeverityScore: metrics?.violationSeverityScore || 0,
      clauseCitationRate: metrics?.clauseCitationRate || 0,
    };
  }

  private normalizeCostMetrics(metrics?: Partial<CostMetrics>): CostMetrics {
    return {
      costPerTask: metrics?.costPerTask || 0,
      efficiencyScore: metrics?.efficiencyScore || 0,
      resourceWastePercent: metrics?.resourceWastePercent || 0,
    };
  }

  private normalizeReliabilityMetrics(
    metrics?: Partial<ReliabilityMetrics>
  ): ReliabilityMetrics {
    return {
      mtbfHours: metrics?.mtbfHours || 0,
      availabilityPercent: metrics?.availabilityPercent || 100,
      errorRatePercent: metrics?.errorRatePercent || 0,
      recoveryTimeMinutes: metrics?.recoveryTimeMinutes || 0,
    };
  }

  /**
   * Removes identifying fields for secure anonymization.
   */
  private removeIdentifyingFields(obj: Record<string, unknown>): void {
    const identifyingFields = [
      "userId",
      "sessionId",
      "ipAddress",
      "userAgent",
      "email",
    ];

    for (const field of identifyingFields) {
      if (field in obj) {
        delete obj[field];
      }
    }
  }

  /**
   * Hashes sensitive fields for basic anonymization.
   */
  private hashSensitiveFields(obj: Record<string, unknown>): void {
    const sensitiveFields = ["agentId", "taskId", "userId"];

    for (const field of sensitiveFields) {
      if (typeof obj[field] === "string") {
        obj[field] = createHash("sha256")
          .update(obj[field] as string)
          .digest("hex")
          .substring(0, 16);
      }
    }
  }

  /**
   * Applies differential privacy techniques.
   */
  private applyDifferentialPrivacy(obj: Record<string, unknown>): void {
    // Add noise to numerical metrics to provide differential privacy
    const numericalFields = ["latencyMs", "score", "qualityScore", "cost"];

    for (const field of numericalFields) {
      if (typeof obj[field] === "number") {
        // Add Laplace noise with sensitivity 1 and epsilon 0.1
        const noise = this.generateLaplaceNoise(1, 0.1);
        obj[field] = (obj[field] as number) + noise;
      }
    }
  }

  /**
   * Generates Laplace noise for differential privacy.
   */
  private generateLaplaceNoise(sensitivity: number, epsilon: number): number {
    const uniform = Math.random() - 0.5;
    const sign = uniform > 0 ? 1 : -1;
    const magnitude =
      -Math.log(Math.abs(uniform) * 2) * (sensitivity / epsilon);
    return sign * magnitude;
  }

  /**
   * Updates average collection time for performance monitoring.
   */
  private updateAverageCollectionTime(): void {
    const alpha = 0.1; // Exponential moving average factor
    this.collectionStats.averageCollectionTimeMs =
      alpha * this.collectionStats.lastCollectionTime +
      (1 - alpha) * this.collectionStats.averageCollectionTimeMs;
  }

  /**
   * Sets up event handlers for internal events.
   */
  private setupEventHandlers(): void {
    this.on("buffer_high_water_mark", (_size: number) => {
      // Could trigger buffer flush or scaling
    });

    this.on("anomaly_detected", (_event: PerformanceEvent) => {
      // Could trigger alerting or mitigation
    });
  }
}
