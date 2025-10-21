/**
 * @fileoverview Health Monitor implementation for Arbiter Orchestration (ARBITER-005)
 *
 * Monitors agent and system health, performs proactive issue detection,
 * and provides comprehensive health status reporting.
 *
 * @author @darianrosebrook
 */

import {
  AgentHealth,
  AgentProfile,
  HealthFailureError,
  HealthStatus,
  IHealthMonitor,
  SystemHealth,
} from "../types/arbiter-orchestration";

/**
 * Health Monitor Configuration
 */
export interface HealthMonitorConfig {
  /** Health check interval in milliseconds */
  checkIntervalMs: number;

  /** Number of consecutive failures before marking unhealthy */
  failureThreshold: number;

  /** Response timeout for health checks in milliseconds */
  responseTimeoutMs: number;

  /** Enable proactive monitoring */
  proactiveMonitoring: boolean;

  /** Alert thresholds for different metrics */
  alertThresholds: Record<string, number>;

  /** Enable detailed health logging */
  detailedLogging: boolean;

  /** Maximum health history to retain */
  maxHistorySize: number;
}

/**
 * Health Check Result
 */
export interface HealthCheckResult {
  /** Component identifier */
  component: string;

  /** Check timestamp */
  timestamp: Date;

  /** Overall result */
  healthy: boolean;

  /** Response time in milliseconds */
  responseTimeMs: number;

  /** Detailed metrics */
  metrics: Record<string, any>;

  /** Issues detected */
  issues: Array<{
    severity: "low" | "medium" | "high" | "critical";
    message: string;
    details?: any;
  }>;

  /** Recommendations for improvement */
  recommendations: string[];
}

/**
 * Health Monitor Statistics
 */
export interface HealthMonitorStats {
  /** Total health checks performed */
  totalChecks: number;

  /** Successful health checks */
  successfulChecks: number;

  /** Failed health checks */
  failedChecks: number;

  /** Average response time */
  averageResponseTimeMs: number;

  /** Health status distribution */
  statusDistribution: Record<"healthy" | "degraded" | "unhealthy", number>;

  /** Active alerts */
  activeAlerts: number;

  /** Recovery actions taken */
  recoveryActions: number;
}

/**
 * Health Monitor Implementation
 *
 * Comprehensive health monitoring for agents and system components.
 * Provides proactive issue detection, alerting, and health status reporting.
 */
export class HealthMonitor implements IHealthMonitor {
  private config: HealthMonitorConfig;
  private stats: HealthMonitorStats;
  private agentHealth: Map<string, AgentHealth> = new Map();
  private systemHealth: SystemHealth;
  private healthHistory: HealthCheckResult[] = [];
  private checkInterval?: ReturnType<typeof setInterval>;
  private isMonitoring: boolean = false;

  constructor(config: Partial<HealthMonitorConfig> = {}) {
    this.config = {
      checkIntervalMs: 30000, // 30 seconds
      failureThreshold: 3,
      responseTimeoutMs: 5000, // 5 seconds
      proactiveMonitoring: true,
      alertThresholds: {
        cpuUtilization: 90,
        memoryUtilization: 85,
        responseTimeMs: 2000,
        consecutiveFailures: 5,
      },
      detailedLogging: false,
      maxHistorySize: 1000,
      ...config,
    };

    this.stats = {
      totalChecks: 0,
      successfulChecks: 0,
      failedChecks: 0,
      averageResponseTimeMs: 0,
      statusDistribution: {
        healthy: 0,
        degraded: 0,
        unhealthy: 0,
      },
      activeAlerts: 0,
      recoveryActions: 0,
    };

    this.systemHealth = {
      component: "system",
      status: "unknown",
      checkedAt: new Date(),
      metrics: {},
      issues: [],
      recoveryActions: [],
      queueDepth: 0,
      queueProcessingRate: 0,
      cpuUtilization: 0,
      memoryUtilization: 0,
      diskUtilization: 0,
      databaseHealthy: true,
      databaseLatencyMs: 0,
      externalServices: {},
      systemLoad: 0,
      processUptime: 0,
    };
  }

  /**
   * Start health monitoring
   */
  async startMonitoring(): Promise<void> {
    if (this.isMonitoring) {
      return;
    }

    this.isMonitoring = true;

    // Perform initial health check
    await this.performFullHealthCheck();

    // Start periodic monitoring
    this.checkInterval = setInterval(async () => {
      try {
        await this.performFullHealthCheck();
      } catch (error) {
        console.error("Health monitoring error:", error);
      }
    }, this.config.checkIntervalMs);
  }

  /**
   * Stop health monitoring
   */
  async stopMonitoring(): Promise<void> {
    if (!this.isMonitoring) {
      return;
    }

    this.isMonitoring = false;

    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = undefined;
    }
  }

  /**
   * Check overall system health
   */
  async checkHealth(): Promise<SystemHealth> {
    await this.performSystemHealthCheck();
    return { ...this.systemHealth };
  }

  /**
   * Check specific agent health
   */
  async checkAgentHealth(agentId: string): Promise<AgentHealth> {
    const agentHealth = await this.performAgentHealthCheck(agentId);
    if (agentHealth) {
      return { ...agentHealth };
    }

    throw new HealthFailureError(`agent-${agentId}`, ["Agent not found"]);
  }

  /**
   * Get all agent health statuses
   */
  getAllAgentHealth(): AgentHealth[] {
    return Array.from(this.agentHealth.values()).map((health) => ({
      ...health,
    }));
  }

  /**
   * Get health monitor statistics
   */
  getStats(): HealthMonitorStats {
    return { ...this.stats };
  }

  /**
   * Get health history
   */
  getHealthHistory(limit?: number): HealthCheckResult[] {
    const history = [...this.healthHistory];
    return limit ? history.slice(-limit) : history;
  }

  /**
   * Check if system is healthy
   */
  isSystemHealthy(): boolean {
    return this.systemHealth.status === "healthy";
  }

  /**
   * Get unhealthy components
   */
  getUnhealthyComponents(): Array<{
    component: string;
    status: HealthStatus["status"];
    issues: string[];
  }> {
    const unhealthy = [];

    // Check system health
    if (this.systemHealth.status !== "healthy") {
      unhealthy.push({
        component: this.systemHealth.component,
        status: this.systemHealth.status,
        issues: this.systemHealth.issues.map((issue) => issue.message),
      });
    }

    // Check agent health
    for (const [agentId, health] of Array.from(this.agentHealth)) {
      if (health.status !== "healthy") {
        unhealthy.push({
          component: `agent-${agentId}`,
          status: health.status,
          issues: health.issues.map((issue) => issue.message),
        });
      }
    }

    return unhealthy;
  }

  /**
   * Perform full health check on all components
   */
  private async performFullHealthCheck(): Promise<void> {
    const startTime = Date.now();

    try {
      // Check system health
      await this.performSystemHealthCheck();

      // Check all registered agents
      const agentIds = Array.from(this.agentHealth.keys());
      for (const agentId of agentIds) {
        await this.performAgentHealthCheck(agentId);
      }

      // Update statistics
      const responseTime = Date.now() - startTime;
      this.updateHealthStats(true, responseTime);

      // Store in history
      this.addToHistory({
        component: "full-system",
        timestamp: new Date(),
        healthy:
          this.isSystemHealthy() && this.getUnhealthyComponents().length === 0,
        responseTimeMs: responseTime,
        metrics: {
          systemStatus: this.systemHealth.status,
          agentCount: agentIds.length,
          unhealthyComponents: this.getUnhealthyComponents().length,
        },
        issues: this.getUnhealthyComponents().flatMap((comp) =>
          comp.issues.map((issue) => ({
            severity: "high" as const,
            message: `${comp.component}: ${issue}`,
          }))
        ),
        recommendations: this.generateRecommendations(),
      });
    } catch (error) {
      const responseTime = Date.now() - startTime;
      this.updateHealthStats(false, responseTime);
      console.error("Full health check failed:", error);
    }
  }

  /**
   * Perform system health check
   */
  private async performSystemHealthCheck(): Promise<void> {
    const startTime = Date.now();

    try {
      const metrics = await this.collectSystemMetrics();
      const issues = this.analyzeSystemMetrics(metrics);
      const status = this.determineHealthStatus(issues);

      this.systemHealth = {
        ...this.systemHealth,
        component: "system",
        status,
        checkedAt: new Date(),
        metrics,
        issues,
        recoveryActions: this.generateRecoveryActions("system", issues),
      };

      const responseTime = Date.now() - startTime;

      // Log if detailed logging enabled
      if (this.config.detailedLogging) {
        console.log(`System health check: ${status} (${responseTime}ms)`);
      }
    } catch (error) {
      this.systemHealth.status = "unhealthy";
      this.systemHealth.issues = [
        {
          severity: "critical",
          message: `System health check failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
          details: error,
        },
      ];
    }
  }

  /**
   * Perform agent health check
   */
  private async performAgentHealthCheck(
    agentId: string
  ): Promise<AgentHealth | null> {
    const startTime = Date.now();

    try {
      const agentHealth = this.agentHealth.get(agentId);
      if (!agentHealth) {
        return null;
      }

      const metrics = await this.collectAgentMetrics(agentId);
      const issues = this.analyzeAgentMetrics(agentId, metrics);
      const status = this.determineHealthStatus(issues);

      const updatedHealth: AgentHealth = {
        ...agentHealth,
        status,
        checkedAt: new Date(),
        metrics,
        issues,
        recoveryActions: this.generateRecoveryActions(
          `agent-${agentId}`,
          issues
        ),
      };

      this.agentHealth.set(agentId, updatedHealth);

      const responseTime = Date.now() - startTime;

      // Log if detailed logging enabled
      if (this.config.detailedLogging) {
        console.log(
          `Agent ${agentId} health check: ${status} (${responseTime}ms)`
        );
      }

      return updatedHealth;
    } catch (error) {
      // Mark agent as unhealthy on check failure
      const agentHealth = this.agentHealth.get(agentId);
      if (agentHealth) {
        agentHealth.status = "unhealthy";
        agentHealth.issues = [
          {
            severity: "high",
            message: `Health check failed: ${
              error instanceof Error ? error.message : String(error)
            }`,
            details: error,
          },
        ];
      }

      return agentHealth || null;
    }
  }

  /**
   * Register agent for health monitoring
   */
  registerAgent(agent: AgentProfile): void {
    const agentHealth: AgentHealth = {
      component: `agent-${agent.id}`,
      status: "unknown",
      checkedAt: new Date(),
      agentId: agent.id,
      responsiveness: 0,
      currentTasks: 0,
      successRate: 0,
      averageResponseTimeMs: 0,
      consecutiveFailures: 0,
      metrics: {},
      issues: [],
      recoveryActions: [],
    };

    this.agentHealth.set(agent.id, agentHealth);
  }

  /**
   * Unregister agent from health monitoring
   */
  unregisterAgent(agentId: string): void {
    this.agentHealth.delete(agentId);
  }

  /**
   * Update agent task status for health tracking
   */
  updateAgentTaskStatus(agentId: string, taskDelta: number): void {
    const agentHealth = this.agentHealth.get(agentId);
    if (agentHealth) {
      agentHealth.currentTasks = Math.max(
        0,
        agentHealth.currentTasks + taskDelta
      );
    }
  }

  /**
   * Record agent task result for success rate tracking
   */
  recordAgentTaskResult(
    agentId: string,
    success: boolean,
    responseTimeMs?: number
  ): void {
    const agentHealth = this.agentHealth.get(agentId);
    if (agentHealth) {
      // Update consecutive failures
      if (success) {
        agentHealth.consecutiveFailures = 0;
        agentHealth.lastSuccessfulTask = new Date();
      } else {
        agentHealth.consecutiveFailures++;
      }

      // Update success rate (simple moving average)
      const totalTasks = agentHealth.metrics.totalTasks || 0;
      const currentSuccessRate = agentHealth.successRate;
      agentHealth.successRate =
        (currentSuccessRate * totalTasks + (success ? 1 : 0)) /
        (totalTasks + 1);
      agentHealth.metrics.totalTasks = totalTasks + 1;

      // Update average response time
      if (responseTimeMs !== undefined) {
        const totalResponses = agentHealth.metrics.totalResponses || 0;
        const currentAverage = agentHealth.averageResponseTimeMs;
        agentHealth.averageResponseTimeMs =
          (currentAverage * totalResponses + responseTimeMs) /
          (totalResponses + 1);
        agentHealth.metrics.totalResponses = totalResponses + 1;
      }
    }
  }

  /**
   * Collect system metrics
   */
  private async collectSystemMetrics(): Promise<Record<string, any>> {
    const metrics: Record<string, any> = {};

    try {
      // TODO: Implement comprehensive system resource monitoring
      // - Integrate with OS-specific monitoring APIs (procfs, Windows Performance Counters, etc.)
      // - Monitor CPU usage per core and overall system load
      // - Track memory usage including virtual memory and swap
      // - Add disk I/O and network I/O monitoring
      // - Implement resource usage trending and forecasting
      // - Add resource utilization thresholds and alerting
      // - Support container and orchestration platform metrics
      // - Implement resource usage profiling and bottleneck analysis
      metrics.cpuUtilization = Math.random() * 100; // Placeholder

      // Memory utilization
      const memUsage = process.memoryUsage();
      metrics.memoryUtilization =
        (memUsage.heapUsed / memUsage.heapTotal) * 100;
      metrics.heapUsed = memUsage.heapUsed;
      metrics.heapTotal = memUsage.heapTotal;

      // Queue depth (would integrate with TaskQueue)
      metrics.queueDepth = 0; // Placeholder
      metrics.queueProcessingRate = 0; // Placeholder

      // Database connectivity (placeholder)
      metrics.databaseHealthy = true;
      metrics.databaseLatencyMs = Math.random() * 100;

      // External service health (placeholders)
      metrics.externalServices = {
        "agent-registry": true,
        "task-router": true,
        "caws-validator": true,
        "performance-tracker": true,
      };

      // System load
      metrics.systemLoad = process.uptime();
      metrics.processUptime = process.uptime();
    } catch (error) {
      metrics.collectionError =
        error instanceof Error ? error.message : String(error);
    }

    return metrics;
  }

  /**
   * Collect agent metrics
   */
  private async collectAgentMetrics(
    agentId: string
  ): Promise<Record<string, any>> {
    const agentHealth = this.agentHealth.get(agentId);
    if (!agentHealth) {
      return {};
    }

    const metrics: Record<string, any> = {};

    try {
      // Responsiveness check (placeholder - would send actual health ping)
      const responsivenessStart = Date.now();
      await new Promise((resolve) => setTimeout(resolve, Math.random() * 100)); // Simulate network call
      metrics.responsiveness = Date.now() - responsivenessStart;
      agentHealth.responsiveness = Math.max(
        0,
        1 - metrics.responsiveness / this.config.responseTimeoutMs
      );

      // Task load
      metrics.currentTasks = agentHealth.currentTasks;

      // Performance metrics
      metrics.successRate = agentHealth.successRate;
      metrics.averageResponseTimeMs = agentHealth.averageResponseTimeMs;
      metrics.consecutiveFailures = agentHealth.consecutiveFailures;
    } catch (error) {
      metrics.collectionError =
        error instanceof Error ? error.message : String(error);
    }

    return metrics;
  }

  /**
   * Analyze system metrics for issues
   */
  private analyzeSystemMetrics(
    metrics: Record<string, any>
  ): HealthStatus["issues"] {
    const issues: HealthStatus["issues"] = [];

    // Check CPU utilization
    if (metrics.cpuUtilization > this.config.alertThresholds.cpuUtilization) {
      issues.push({
        severity: "high",
        message: `High CPU utilization: ${metrics.cpuUtilization.toFixed(1)}%`,
        details: {
          threshold: this.config.alertThresholds.cpuUtilization,
          actual: metrics.cpuUtilization,
        },
      });
    }

    // Check memory utilization
    if (
      metrics.memoryUtilization > this.config.alertThresholds.memoryUtilization
    ) {
      issues.push({
        severity: "high",
        message: `High memory utilization: ${metrics.memoryUtilization.toFixed(
          1
        )}%`,
        details: {
          threshold: this.config.alertThresholds.memoryUtilization,
          actual: metrics.memoryUtilization,
        },
      });
    }

    // Check database connectivity
    if (!metrics.databaseHealthy) {
      issues.push({
        severity: "critical",
        message: "Database connectivity lost",
        details: { latency: metrics.databaseLatencyMs },
      });
    }

    // Check external services
    for (const [service, healthy] of Object.entries(
      metrics.externalServices || {}
    )) {
      if (!healthy) {
        issues.push({
          severity: "medium",
          message: `External service unhealthy: ${service}`,
          details: { service, healthy },
        });
      }
    }

    return issues;
  }

  /**
   * Analyze agent metrics for issues
   */
  private analyzeAgentMetrics(
    agentId: string,
    metrics: Record<string, any>
  ): HealthStatus["issues"] {
    const issues: HealthStatus["issues"] = [];

    // Check responsiveness
    if (metrics.responsiveness > this.config.alertThresholds.responseTimeMs) {
      issues.push({
        severity: "medium",
        message: `Slow response time: ${metrics.responsiveness}ms`,
        details: {
          threshold: this.config.alertThresholds.responseTimeMs,
          actual: metrics.responsiveness,
        },
      });
    }

    // Check consecutive failures
    if (
      metrics.consecutiveFailures >
      this.config.alertThresholds.consecutiveFailures
    ) {
      issues.push({
        severity: "high",
        message: `High consecutive failures: ${metrics.consecutiveFailures}`,
        details: {
          threshold: this.config.alertThresholds.consecutiveFailures,
          actual: metrics.consecutiveFailures,
        },
      });
    }

    // Check success rate
    if (metrics.successRate < 0.5) {
      issues.push({
        severity: "medium",
        message: `Low success rate: ${(metrics.successRate * 100).toFixed(1)}%`,
        details: { successRate: metrics.successRate },
      });
    }

    return issues;
  }

  /**
   * Determine health status from issues
   */
  private determineHealthStatus(
    issues: HealthStatus["issues"]
  ): HealthStatus["status"] {
    if (issues.length === 0) {
      return "healthy";
    }

    const hasCritical = issues.some((issue) => issue.severity === "critical");
    const hasHigh = issues.some((issue) => issue.severity === "high");

    if (hasCritical) {
      return "unhealthy";
    } else if (hasHigh) {
      return "degraded";
    } else {
      return "degraded";
    }
  }

  /**
   * Generate recovery actions for issues
   */
  private generateRecoveryActions(
    component: string,
    issues: HealthStatus["issues"]
  ): HealthStatus["recoveryActions"] {
    const actions: HealthStatus["recoveryActions"] = [];

    for (const issue of issues) {
      if (issue.severity === "critical") {
        actions.push({
          action: `Restart ${component}`,
          timestamp: new Date(),
          success: false,
          details: `Critical issue detected: ${issue.message}`,
        });
      } else if (issue.severity === "high") {
        actions.push({
          action: `Scale up ${component}`,
          timestamp: new Date(),
          success: false,
          details: `High severity issue: ${issue.message}`,
        });
      }
    }

    return actions;
  }

  /**
   * Generate health recommendations
   */
  private generateRecommendations(): string[] {
    const recommendations: string[] = [];

    if (this.systemHealth.metrics.cpuUtilization > 80) {
      recommendations.push(
        "Consider scaling system resources for better CPU utilization"
      );
    }

    if (this.systemHealth.metrics.memoryUtilization > 80) {
      recommendations.push(
        "Monitor memory usage and consider optimizing memory-intensive operations"
      );
    }

    const unhealthyAgents = this.getUnhealthyComponents().filter((comp) =>
      comp.component.startsWith("agent-")
    );
    if (unhealthyAgents.length > 0) {
      recommendations.push(
        `Review health of ${unhealthyAgents.length} unhealthy agents`
      );
    }

    return recommendations;
  }

  /**
   * Update health statistics
   */
  private updateHealthStats(success: boolean, responseTimeMs: number): void {
    this.stats.totalChecks++;

    if (success) {
      this.stats.successfulChecks++;
    } else {
      this.stats.failedChecks++;
    }

    // Update average response time
    const totalResponseTime =
      this.stats.averageResponseTimeMs * (this.stats.totalChecks - 1);
    this.stats.averageResponseTimeMs =
      (totalResponseTime + responseTimeMs) / this.stats.totalChecks;

    // Update status distribution
    this.stats.statusDistribution.healthy = this.agentHealth.size;
    this.stats.statusDistribution.degraded =
      this.getUnhealthyComponents().filter(
        (comp) => comp.status === "degraded"
      ).length;
    this.stats.statusDistribution.unhealthy =
      this.getUnhealthyComponents().filter(
        (comp) => comp.status === "unhealthy"
      ).length;
  }

  /**
   * Add result to health history
   */
  private addToHistory(result: HealthCheckResult): void {
    this.healthHistory.push(result);

    // Maintain history size limit
    if (this.healthHistory.length > this.config.maxHistorySize) {
      this.healthHistory.shift();
    }
  }
}
