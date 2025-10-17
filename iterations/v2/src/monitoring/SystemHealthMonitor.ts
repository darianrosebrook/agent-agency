/**
 * System Health Monitor - Comprehensive Health Assessment
 *
 * Monitors system health, collects metrics, assesses agent health, and provides
 * health scores for intelligent decision making in the Arbiter Orchestrator.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import {
  EmbeddingMetrics,
  EmbeddingMonitor,
} from "../embeddings/EmbeddingMonitor.js";
import { MetricsCollector } from "./MetricsCollector.js";
import {
  AgentHealthMetrics,
  HealthAlert,
  HealthMetrics,
  SystemHealthMonitorConfig,
  SystemMetrics,
} from "./types.js";

export class SystemHealthMonitor extends EventEmitter {
  private config: SystemHealthMonitorConfig;
  private metricsCollector: MetricsCollector;
  private databaseClient?: any; // PerformanceTrackerDatabaseClient
  private embeddingMonitor?: EmbeddingMonitor;
  private metricsHistory: SystemMetrics[] = [];
  private agentHealthMetrics: Map<string, AgentHealthMetrics> = new Map();
  private alerts: HealthAlert[] = [];
  private circuitBreakerFailureCount = 0;
  private circuitBreakerLastFailure = 0;
  private circuitBreakerState: "closed" | "open" | "half-open" = "closed";

  // Timers
  private metricsCollectionTimer?: ReturnType<typeof setInterval>;
  private healthCheckTimer?: ReturnType<typeof setInterval>;

  constructor(
    config: Partial<SystemHealthMonitorConfig> = {},
    databaseClient?: any
  ) {
    super();

    this.config = {
      collectionIntervalMs: 30000, // 30 seconds
      healthCheckIntervalMs: 60000, // 1 minute
      retentionPeriodMs: 3600000, // 1 hour
      enableCircuitBreaker: true,
      circuitBreakerFailureThreshold: 5,
      circuitBreakerRecoveryTimeoutMs: 300000, // 5 minutes
      thresholds: {
        cpuWarningThreshold: 70,
        cpuCriticalThreshold: 90,
        memoryWarningThreshold: 80,
        memoryCriticalThreshold: 95,
        diskWarningThreshold: 85,
        diskCriticalThreshold: 95,
        agentErrorRateThreshold: 5, // errors per minute
        agentResponseTimeThreshold: 5000, // 5 seconds
        systemErrorRateThreshold: 10, // errors per minute
        queueDepthThreshold: 100,
      },
      ...config,
    };

    this.databaseClient = databaseClient;
    this.metricsCollector = new MetricsCollector();
  }

  /**
   * Initialize the health monitor
   */
  async initialize(): Promise<void> {
    console.log("Initializing System Health Monitor...");

    // Initialize metrics collector with database client
    if (this.databaseClient) {
      await this.metricsCollector.initialize(this.databaseClient);
    }

    // Start metrics collection
    await this.startMetricsCollection();

    // Start health checks
    this.startHealthChecks();

    console.log("✅ System Health Monitor initialized");
  }

  /**
   * Shutdown the health monitor
   */
  async shutdown(): Promise<void> {
    console.log("Shutting down System Health Monitor...");

    if (this.metricsCollectionTimer) {
      clearInterval(this.metricsCollectionTimer);
      this.metricsCollectionTimer = undefined;
    }

    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
      this.healthCheckTimer = undefined;
    }

    console.log("✅ System Health Monitor shutdown complete");
  }

  /**
   * Register embedding monitor for integrated health monitoring
   */
  registerEmbeddingMonitor(monitor: EmbeddingMonitor): void {
    this.embeddingMonitor = monitor;
    console.log("✅ Embedding monitor registered with System Health Monitor");
  }

  /**
   * Get comprehensive health metrics
   */
  async getHealthMetrics(): Promise<HealthMetrics> {
    const systemMetrics =
      this.metricsHistory.length > 0
        ? this.metricsHistory[this.metricsHistory.length - 1]
        : await this.metricsCollector.collectSystemMetrics();

    const overallHealth = this.calculateOverallHealth(systemMetrics);

    // Get real system-wide metrics from various sources
    const errorRate = this.calculateSystemErrorRate();
    const queueDepth = this.getEstimatedQueueDepth();

    // Get embedding metrics if monitor is available
    let embeddingMetrics: EmbeddingMetrics | undefined;
    if (this.embeddingMonitor) {
      try {
        embeddingMetrics = await this.embeddingMonitor.collectMetrics();
      } catch (error) {
        console.warn("Failed to collect embedding metrics:", error);
      }
    }

    return {
      overallHealth,
      system: systemMetrics,
      agents: new Map(this.agentHealthMetrics),
      errorRate,
      queueDepth,
      circuitBreakerOpen: this.circuitBreakerState === "open",
      embedding: embeddingMetrics,
      timestamp: new Date(),
    };
  }

  /**
   * Get historical metrics summary with trends and alerts
   */
  async getHistoricalMetricsSummary(hoursBack: number = 24): Promise<{
    current: SystemMetrics | null;
    historical: {
      average: Partial<SystemMetrics>;
      trends: Record<string, any>;
      dataPoints: number;
    };
    alerts: string[];
    agentMetrics: Map<string, AgentHealthMetrics>;
    systemAlerts: HealthAlert[];
  }> {
    const summary = await this.metricsCollector.getMetricsSummary(hoursBack);

    // Add agent-specific trends and alerts
    const agentAlerts: string[] = [];
    for (const [agentId, metrics] of this.agentHealthMetrics) {
      if (metrics.healthScore < 0.5) {
        agentAlerts.push(
          `Agent ${agentId} health critically low (${(
            metrics.healthScore * 100
          ).toFixed(1)}%)`
        );
      } else if (metrics.healthScore < 0.7) {
        agentAlerts.push(
          `Agent ${agentId} health degraded (${(
            metrics.healthScore * 100
          ).toFixed(1)}%)`
        );
      }

      if (metrics.errorRate > this.config.thresholds.agentErrorRateThreshold) {
        agentAlerts.push(
          `Agent ${agentId} high error rate (${metrics.errorRate} errors/min)`
        );
      }

      if (
        metrics.responseTimeP95 >
        this.config.thresholds.agentResponseTimeThreshold
      ) {
        agentAlerts.push(
          `Agent ${agentId} slow response time (${metrics.responseTimeP95}ms)`
        );
      }
    }

    return {
      current: summary.current,
      historical: summary.historical,
      alerts: [...summary.alerts, ...agentAlerts],
      agentMetrics: new Map(this.agentHealthMetrics),
      systemAlerts: this.alerts.filter((alert) => !alert.resolved),
    };
  }

  /**
   * Get health metrics for a specific agent
   */
  getAgentHealth(agentId: string): AgentHealthMetrics | null {
    return this.agentHealthMetrics.get(agentId) || null;
  }

  /**
   * Update agent health metrics
   */
  updateAgentHealth(
    agentId: string,
    metrics: Partial<AgentHealthMetrics>
  ): void {
    const existing = this.agentHealthMetrics.get(agentId) || {
      agentId,
      healthScore: 0.8,
      reliabilityScore: 0.8,
      errorRate: 0,
      responseTimeP95: 1000,
      currentLoad: 0,
      maxLoad: 5,
      successRate: 0.95,
      lastActivity: new Date(),
      circuitBreakerStatus: "closed" as const,
    };

    const updated: AgentHealthMetrics = {
      ...existing,
      ...metrics,
      lastActivity: metrics.lastActivity || existing.lastActivity,
    };

    // Recalculate health score
    updated.healthScore = this.calculateAgentHealthScore(updated);

    this.agentHealthMetrics.set(agentId, updated);

    // Check for alerts
    this.checkAgentAlerts(agentId, updated);
  }

  /**
   * Record agent task completion
   */
  recordAgentTask(
    agentId: string,
    success: boolean,
    responseTime: number
  ): void {
    const metrics = this.agentHealthMetrics.get(agentId);
    if (!metrics) {
      // Initialize with default values
      this.updateAgentHealth(agentId, {
        currentLoad: 1,
        lastActivity: new Date(),
      });
      return;
    }

    // Update load and activity
    metrics.currentLoad = Math.max(0, metrics.currentLoad - 1); // Assume task completion reduces load
    metrics.lastActivity = new Date();

    // Update success rate (rolling average)
    const alpha = 0.1; // Learning rate
    metrics.successRate =
      metrics.successRate * (1 - alpha) + (success ? 1 : 0) * alpha;

    // Update response time (rolling average)
    metrics.responseTimeP95 =
      metrics.responseTimeP95 * (1 - alpha) + responseTime * alpha;

    this.agentHealthMetrics.set(agentId, metrics);
  }

  /**
   * Record agent error
   */
  recordAgentError(agentId: string, _errorType: string = "generic"): void {
    const metrics = this.agentHealthMetrics.get(agentId);
    if (!metrics) {
      this.updateAgentHealth(agentId, {
        errorRate: 1,
        lastActivity: new Date(),
      });
      return;
    }

    // Increment error rate (simplified - would use time-based window in production)
    metrics.errorRate += 1;
    metrics.lastActivity = new Date();

    this.agentHealthMetrics.set(agentId, metrics);

    // Check circuit breaker
    if (this.config.enableCircuitBreaker) {
      this.updateCircuitBreaker();
    }
  }

  /**
   * Get active health alerts
   */
  getActiveAlerts(): HealthAlert[] {
    return this.alerts.filter((alert) => !alert.resolved);
  }

  /**
   * Acknowledge an alert
   */
  acknowledgeAlert(alertId: string): boolean {
    const alert = this.alerts.find((a) => a.id === alertId);
    if (alert && !alert.acknowledged) {
      alert.acknowledged = true;
      return true;
    }
    return false;
  }

  /**
   * Simulate health degradation (for testing)
   */
  async simulateHealthDegradation(): Promise<void> {
    // Add artificial system load
    if (this.metricsHistory.length > 0) {
      const lastMetrics = this.metricsHistory[this.metricsHistory.length - 1];
      const degradedMetrics: SystemMetrics = {
        ...lastMetrics,
        cpuUsage: Math.min(100, lastMetrics.cpuUsage + 30),
        memoryUsage: Math.min(100, lastMetrics.memoryUsage + 20),
        timestamp: new Date(),
      };

      this.metricsHistory.push(degradedMetrics);
    }

    // Degrade agent health
    for (const [agentId, metrics] of this.agentHealthMetrics) {
      this.updateAgentHealth(agentId, {
        errorRate: metrics.errorRate + 2,
        healthScore: Math.max(0.1, metrics.healthScore - 0.2),
      });
    }
  }

  /**
   * Start metrics collection
   */
  private async startMetricsCollection(): Promise<void> {
    // Collect initial metrics
    const initialMetrics = await this.metricsCollector.collectSystemMetrics();
    this.metricsHistory.push(initialMetrics);

    // Set up periodic collection
    this.metricsCollectionTimer = setInterval(async () => {
      try {
        const metrics = await this.metricsCollector.collectSystemMetrics();
        this.metricsHistory.push(metrics);

        // Clean up old metrics
        this.cleanupOldMetrics();
      } catch (error) {
        console.error("Failed to collect system metrics:", error);
      }
    }, this.config.collectionIntervalMs);
  }

  /**
   * Start health checks
   */
  private startHealthChecks(): void {
    this.healthCheckTimer = setInterval(async () => {
      try {
        const healthMetrics = await this.getHealthMetrics();

        // Emit health update event
        this.emit("health-updated", healthMetrics);

        // Check system alerts
        this.checkSystemAlerts(healthMetrics);
      } catch (error) {
        console.error("Failed to perform health check:", error);
      }
    }, this.config.healthCheckIntervalMs);
  }

  /**
   * Calculate overall system health score
   */
  private calculateOverallHealth(systemMetrics: SystemMetrics): number {
    const weights = {
      cpu: 0.3,
      memory: 0.3,
      disk: 0.2,
      load: 0.2,
    };

    // Invert metrics (lower usage = higher health)
    const cpuHealth = Math.max(0, 1 - systemMetrics.cpuUsage / 100);
    const memoryHealth = Math.max(0, 1 - systemMetrics.memoryUsage / 100);
    const diskHealth = Math.max(0, 1 - systemMetrics.diskUsage / 100);

    // Load average health (normalize to 0-1 scale, assuming 4 CPUs)
    const loadHealth = Math.max(0, 1 - systemMetrics.loadAverage[0] / 4);

    const overallHealth =
      cpuHealth * weights.cpu +
      memoryHealth * weights.memory +
      diskHealth * weights.disk +
      loadHealth * weights.load;

    return Math.round(overallHealth * 100) / 100;
  }

  /**
   * Calculate agent health score
   */
  private calculateAgentHealthScore(metrics: AgentHealthMetrics): number {
    const weights = {
      reliability: 0.3,
      errorRate: 0.2,
      responseTime: 0.2,
      load: 0.15,
      successRate: 0.15,
    };

    // Normalize each metric to 0-1 scale (higher = healthier)
    const reliability = metrics.reliabilityScore;
    const errorRateHealth = Math.max(0, 1 - metrics.errorRate / 10); // Max 10 errors/min
    const responseTimeHealth = Math.max(0, 1 - metrics.responseTimeP95 / 10000); // Max 10 seconds
    const loadHealth = Math.max(0, 1 - metrics.currentLoad / metrics.maxLoad);
    const successRate = metrics.successRate;

    const healthScore =
      reliability * weights.reliability +
      errorRateHealth * weights.errorRate +
      responseTimeHealth * weights.responseTime +
      loadHealth * weights.load +
      successRate * weights.successRate;

    return Math.round(healthScore * 100) / 100;
  }

  /**
   * Calculate system error rate
   */
  private calculateSystemErrorRate(): number {
    // Sum error rates from all agents
    let totalErrorRate = 0;
    for (const metrics of this.agentHealthMetrics.values()) {
      totalErrorRate += metrics.errorRate;
    }

    return Math.round(totalErrorRate * 100) / 100;
  }

  /**
   * Get estimated queue depth
   */
  private getEstimatedQueueDepth(): number {
    // Estimate based on agent load (simplified)
    let totalLoad = 0;
    let totalCapacity = 0;

    for (const metrics of this.agentHealthMetrics.values()) {
      totalLoad += metrics.currentLoad;
      totalCapacity += metrics.maxLoad;
    }

    if (totalCapacity === 0) return 0;

    // If system is at capacity, estimate queue depth
    const utilization = totalLoad / totalCapacity;
    return utilization > 0.8 ? Math.round((utilization - 0.8) * 100) : 0;
  }

  /**
   * Check for system alerts
   */
  private checkSystemAlerts(healthMetrics: HealthMetrics): void {
    const { system } = healthMetrics;
    const { thresholds } = this.config;

    // CPU alerts
    if (system.cpuUsage >= thresholds.cpuCriticalThreshold) {
      this.createAlert(
        "critical",
        "system",
        "cpu",
        system.cpuUsage,
        thresholds.cpuCriticalThreshold,
        "Critical CPU usage"
      );
    } else if (system.cpuUsage >= thresholds.cpuWarningThreshold) {
      this.createAlert(
        "warning",
        "system",
        "cpu",
        system.cpuUsage,
        thresholds.cpuWarningThreshold,
        "High CPU usage"
      );
    }

    // Memory alerts
    if (system.memoryUsage >= thresholds.memoryCriticalThreshold) {
      this.createAlert(
        "critical",
        "system",
        "memory",
        system.memoryUsage,
        thresholds.memoryCriticalThreshold,
        "Critical memory usage"
      );
    } else if (system.memoryUsage >= thresholds.memoryWarningThreshold) {
      this.createAlert(
        "warning",
        "system",
        "memory",
        system.memoryUsage,
        thresholds.memoryWarningThreshold,
        "High memory usage"
      );
    }

    // Error rate alerts
    if (healthMetrics.errorRate >= thresholds.systemErrorRateThreshold) {
      this.createAlert(
        "error",
        "system",
        "error_rate",
        healthMetrics.errorRate,
        thresholds.systemErrorRateThreshold,
        "High system error rate"
      );
    }

    // Queue depth alerts
    if (healthMetrics.queueDepth >= thresholds.queueDepthThreshold) {
      this.createAlert(
        "warning",
        "system",
        "queue_depth",
        healthMetrics.queueDepth,
        thresholds.queueDepthThreshold,
        "High task queue depth"
      );
    }
  }

  /**
   * Check for agent alerts
   */
  private checkAgentAlerts(agentId: string, metrics: AgentHealthMetrics): void {
    const { thresholds } = this.config;

    // Error rate alerts
    if (metrics.errorRate >= thresholds.agentErrorRateThreshold) {
      this.createAlert(
        "error",
        "agent",
        "error_rate",
        metrics.errorRate,
        thresholds.agentErrorRateThreshold,
        `High error rate for agent ${agentId}`,
        agentId
      );
    }

    // Response time alerts
    if (metrics.responseTimeP95 >= thresholds.agentResponseTimeThreshold) {
      this.createAlert(
        "warning",
        "agent",
        "response_time",
        metrics.responseTimeP95,
        thresholds.agentResponseTimeThreshold,
        `Slow response time for agent ${agentId}`,
        agentId
      );
    }

    // Health score alerts
    if (metrics.healthScore < 0.5) {
      this.createAlert(
        "critical",
        "agent",
        "health_score",
        metrics.healthScore,
        0.5,
        `Agent ${agentId} health critically low`,
        agentId
      );
    } else if (metrics.healthScore < 0.7) {
      this.createAlert(
        "warning",
        "agent",
        "health_score",
        metrics.healthScore,
        0.7,
        `Agent ${agentId} health degraded`,
        agentId
      );
    }
  }

  /**
   * Create a health alert
   */
  private createAlert(
    level: HealthAlert["level"],
    component: HealthAlert["component"],
    metric: string,
    value: number,
    threshold: number,
    message: string,
    componentId?: string
  ): void {
    // Check if similar alert already exists
    const existingAlert = this.alerts.find(
      (alert) =>
        alert.component === component &&
        alert.componentId === componentId &&
        alert.metric === metric &&
        !alert.resolved
    );

    if (existingAlert) {
      // Update existing alert
      existingAlert.value = value;
      existingAlert.timestamp = new Date();
      return;
    }

    const alert: HealthAlert = {
      id: `alert-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      level,
      component,
      componentId,
      metric,
      value,
      threshold,
      message,
      timestamp: new Date(),
      acknowledged: false,
      resolved: false,
    };

    this.alerts.push(alert);
    this.emit("alert-created", alert);
  }

  /**
   * Update circuit breaker state
   */
  private updateCircuitBreaker(): void {
    const now = Date.now();

    // Track failures in sliding window
    if (now - this.circuitBreakerLastFailure > 60000) {
      // Reset after 1 minute
      this.circuitBreakerFailureCount = 0;
    }

    this.circuitBreakerFailureCount++;
    this.circuitBreakerLastFailure = now;

    // Update circuit breaker state
    if (
      this.circuitBreakerFailureCount >=
      this.config.circuitBreakerFailureThreshold
    ) {
      if (this.circuitBreakerState === "closed") {
        this.circuitBreakerState = "open";
        console.warn("Circuit breaker opened due to high error rate");
        this.emit("circuit-breaker-opened");
      }
    } else if (
      this.circuitBreakerState === "open" &&
      now - this.circuitBreakerLastFailure >
        this.config.circuitBreakerRecoveryTimeoutMs
    ) {
      this.circuitBreakerState = "half-open";
      console.log("Circuit breaker transitioning to half-open");
    }
  }

  /**
   * Clean up old metrics
   */
  private cleanupOldMetrics(): void {
    const cutoffTime = Date.now() - this.config.retentionPeriodMs;
    this.metricsHistory = this.metricsHistory.filter(
      (metrics) => metrics.timestamp.getTime() > cutoffTime
    );
  }
}
