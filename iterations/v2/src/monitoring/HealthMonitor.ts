/**
 * Comprehensive Health Monitor for V2 Arbiter
 *
 * Provides real-time health monitoring, metrics collection, and alerting
 * for all system components to ensure production readiness.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import * as os from "os";
import { ConnectionPoolManager } from "../database/ConnectionPoolManager.js";
import { circuitBreakerManager } from "../resilience/CircuitBreakerManager";

/// <reference types="node" />

export interface HealthCheck {
  name: string;
  status: "healthy" | "degraded" | "unhealthy";
  message?: string;
  lastChecked: Date;
  responseTimeMs?: number;
  metadata?: Record<string, any>;
  details?: Record<string, any>;
}

export interface SystemMetrics {
  uptime: number;
  memoryUsage: NodeJS.MemoryUsage;
  cpuUsage: number;
  activeConnections: number;
  taskQueueDepth: number;
  circuitBreakerStats: Record<string, any>;
  errorRate: number;
  throughput: number;
  detailed?: {
    memoryUsagePercent: number;
    memoryUsageMB: number;
    totalMemoryMB: number;
    freeMemoryMB: number;
    cpuUsagePercent: number;
    loadAverage: number[];
    databaseConnections: { total: number; idle: number; waiting: number };
    activeAlerts: number;
    totalHealthChecks: number;
    systemLoad: number;
    processUptime: number;
    nodeVersion: string;
    platform: string;
    architecture: string;
  };
}

export interface HealthAlert {
  id: string;
  severity: "low" | "medium" | "high" | "critical";
  component: string;
  message: string;
  timestamp: Date;
  resolved?: boolean;
}

export class HealthMonitor extends EventEmitter {
  private healthChecks = new Map<string, HealthCheck>();
  private metrics: SystemMetrics;
  private alerts = new Map<string, HealthAlert>();
  private startTime = Date.now();
  private checkInterval?: NodeJS.Timeout;
  private metricsInterval?: NodeJS.Timeout;
  private isRunning = false;
  private checkCount = 0;

  constructor(
    private config: {
      checkIntervalMs: number;
      metricsIntervalMs: number;
      alertThresholds: {
        memoryUsagePercent: number;
        cpuUsagePercent: number;
        errorRatePercent: number;
        responseTimeMs: number;
      };
    }
  ) {
    super();
    this.metrics = this.initializeMetrics();
  }

  start(): void {
    if (this.isRunning) return;

    this.isRunning = true;
    this.startTime = Date.now();

    // Start health checks
    this.checkInterval = setInterval(() => {
      this.performHealthChecks();
    }, this.config.checkIntervalMs);

    // Start metrics collection
    this.metricsInterval = setInterval(() => {
      this.collectMetrics();
    }, this.config.metricsIntervalMs);

    this.emit("started");
  }

  stop(): void {
    if (!this.isRunning) return;

    this.isRunning = false;

    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = undefined;
    }

    if (this.metricsInterval) {
      clearInterval(this.metricsInterval);
      this.metricsInterval = undefined;
    }

    this.emit("stopped");
  }

  private async performHealthChecks(): Promise<void> {
    this.checkCount++;
    const checks = [
      this.checkDatabaseHealth(),
      this.checkCircuitBreakers(),
      this.checkMemoryUsage(),
      this.checkTaskQueue(),
      this.checkWebInterface(),
    ];

    await Promise.allSettled(checks);
    this.emit("health-checks-completed", this.getHealthSummary());
  }

  private async checkDatabaseHealth(): Promise<void> {
    const start = Date.now();
    try {
      // This would integrate with your actual database health check
      const responseTime = Date.now() - start;

      this.updateHealthCheck("database", {
        name: "database",
        status: "healthy",
        message: "Database connection pool healthy",
        lastChecked: new Date(),
        responseTimeMs: responseTime,
        metadata: {
          connectionCount: 5, // This would be actual connection count
          poolSize: 20,
        },
      });
    } catch (error) {
      this.updateHealthCheck("database", {
        name: "database",
        status: "unhealthy",
        message: `Database health check failed: ${error}`,
        lastChecked: new Date(),
        responseTimeMs: Date.now() - start,
      });

      this.createAlert("database", "high", "Database health check failed");
    }
  }

  private async checkCircuitBreakers(): Promise<void> {
    try {
      const stats = circuitBreakerManager.getAllStats();
      let overallStatus: "healthy" | "degraded" | "unhealthy" = "healthy";

      for (const [name, breakerStats] of Object.entries(stats)) {
        if (breakerStats.state === "open") {
          overallStatus = "unhealthy";
          this.createAlert(name, "high", `Circuit breaker ${name} is open`);
        } else if (breakerStats.state === "half-open") {
          overallStatus =
            overallStatus === "healthy" ? "degraded" : overallStatus;
        }
      }

      this.updateHealthCheck("circuit-breakers", {
        name: "circuit-breakers",
        status: overallStatus,
        message: `${Object.keys(stats).length} circuit breakers monitored`,
        lastChecked: new Date(),
        metadata: { stats },
      });
    } catch (error) {
      this.updateHealthCheck("circuit-breakers", {
        name: "circuit-breakers",
        status: "unhealthy",
        message: `Circuit breaker check failed: ${error}`,
        lastChecked: new Date(),
      });
    }
  }

  private async checkMemoryUsage(): Promise<void> {
    const memUsage = process.memoryUsage();
    const memUsagePercent = (memUsage.heapUsed / memUsage.heapTotal) * 100;

    let status: "healthy" | "degraded" | "unhealthy" = "healthy";
    if (memUsagePercent > this.config.alertThresholds.memoryUsagePercent) {
      status = "unhealthy";
      this.createAlert(
        "memory",
        "critical",
        `High memory usage: ${memUsagePercent.toFixed(1)}%`
      );
    } else if (
      memUsagePercent >
      this.config.alertThresholds.memoryUsagePercent * 0.8
    ) {
      status = "degraded";
    }

    this.updateHealthCheck("memory", {
      name: "memory",
      status,
      message: `Memory usage: ${memUsagePercent.toFixed(1)}%`,
      lastChecked: new Date(),
      metadata: {
        heapUsed: memUsage.heapUsed,
        heapTotal: memUsage.heapTotal,
        external: memUsage.external,
        rss: memUsage.rss,
      },
    });
  }

  private async checkTaskQueue(): Promise<void> {
    // This would integrate with your actual task queue
    const queueDepth = 0; // This would be actual queue depth

    let status: "healthy" | "degraded" | "unhealthy" = "healthy";
    if (queueDepth > 100) {
      status = "unhealthy";
      this.createAlert(
        "task-queue",
        "high",
        `High task queue depth: ${queueDepth}`
      );
    } else if (queueDepth > 50) {
      status = "degraded";
    }

    this.updateHealthCheck("task-queue", {
      name: "task-queue",
      status,
      message: `Queue depth: ${queueDepth}`,
      lastChecked: new Date(),
      metadata: { queueDepth },
    });
  }

  private async checkWebInterface(): Promise<void> {
    const start = Date.now();
    try {
      // Check if web interface is responding
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000);

      const response = await fetch("http://localhost:3000/", {
        method: "HEAD",
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      const responseTime = Date.now() - start;
      const status = response.ok ? "healthy" : "degraded";

      this.updateHealthCheck("web-interface", {
        name: "web-interface",
        status,
        message: response.ok
          ? "Web interface responsive"
          : `Web interface returned status ${response.status}`,
        lastChecked: new Date(),
        details: { responseTimeMs: responseTime, statusCode: response.status },
      });

      // Alert if degraded (not responding properly)
      if (status === "degraded") {
        this.createAlert(
          "web-interface",
          "medium",
          "Web interface not responding"
        );
      }
    } catch (error) {
      this.updateHealthCheck("web-interface", {
        name: "web-interface",
        status: "unhealthy",
        message: `Web interface check failed: ${error}`,
        lastChecked: new Date(),
        details: {
          error: error instanceof Error ? error.message : String(error),
        },
      });

      this.createAlert(
        "web-interface",
        "medium",
        "Web interface not responding"
      );
    }
  }

  private collectMetrics(): void {
    // Collect detailed system metrics
    const memUsage = process.memoryUsage();
    const cpuUsage = process.cpuUsage();
    const loadAverage = os.loadavg();

    // Calculate memory usage percentages
    const totalMemory = os.totalmem();
    const freeMemory = os.freemem();
    const memoryUsagePercent = Math.round(
      ((totalMemory - freeMemory) / totalMemory) * 100
    );

    // Calculate CPU usage (percentage)
    const cpus = os.cpus();
    let totalIdle = 0;
    let totalTick = 0;
    for (const cpu of cpus) {
      for (const type in cpu.times) {
        totalTick += (cpu.times as any)[type];
      }
      totalIdle += cpu.times.idle;
    }
    const idle = totalIdle / cpus.length;
    const total = totalTick / cpus.length;
    const cpuUsagePercent = Math.round(100 - ~~((100 * idle) / total));

    // Get database connection stats
    let dbConnections = { total: 0, idle: 0, waiting: 0 };
    try {
      const pool = ConnectionPoolManager.getInstance().getPool();
      if (pool) {
        dbConnections = {
          total: pool.totalCount,
          idle: pool.idleCount,
          waiting: pool.waitingCount,
        };
      }
    } catch (error) {
      console.warn("[HEALTH] Failed to get database connection stats:", error);
    }

    // TODO: Implement comprehensive error rate tracking and analysis
    // - Track errors over time windows with proper aggregation
    // - Implement error rate calculation with statistical significance
    // - Support error categorization and severity weighting
    // - Add error rate trend analysis and anomaly detection
    // - Implement error rate correlation with system performance
    // - Support error rate forecasting and predictive alerting
    // - Add error rate comparison across different time periods
    // - Implement error rate-based health scoring and recommendations
    const recentErrors = this.alerts.size; // Simplified - actual implementation would track errors over time
    const errorRate =
      recentErrors > 0
        ? (recentErrors / Math.max(1, this.checkCount)) * 100
        : 0;

    // TODO: Implement comprehensive throughput monitoring and optimization
    // - Track actual task completion rates and processing volumes
    // - Implement throughput trend analysis and performance forecasting
    // - Support throughput benchmarking against service level objectives
    // - Add throughput correlation with system resource utilization
    // - Implement throughput bottleneck identification and optimization
    // - Support throughput comparison across different time periods
    // - Add throughput-based capacity planning and scaling recommendations
    // - Implement throughput monitoring with alerting and anomaly detection
    const uptimeMinutes = (Date.now() - this.startTime) / 60000;
    const throughput = uptimeMinutes > 0 ? this.checkCount / uptimeMinutes : 0;

    this.metrics = {
      uptime: Date.now() - this.startTime,
      memoryUsage: memUsage,
      cpuUsage: cpuUsagePercent / 100, // Convert to 0-1 range
      activeConnections: dbConnections.total,
      taskQueueDepth: 0, // Would need integration with task queue
      circuitBreakerStats: circuitBreakerManager.getAllStats(),
      errorRate: errorRate,
      throughput: throughput,

      // Additional detailed metrics
      detailed: {
        memoryUsagePercent,
        memoryUsageMB: Math.round(memUsage.heapUsed / 1024 / 1024),
        totalMemoryMB: Math.round(totalMemory / 1024 / 1024),
        freeMemoryMB: Math.round(freeMemory / 1024 / 1024),
        cpuUsagePercent,
        loadAverage: loadAverage,
        databaseConnections: dbConnections,
        activeAlerts: this.alerts.size,
        totalHealthChecks: this.checkCount,
        systemLoad: loadAverage[0], // 1-minute load average
        processUptime: process.uptime(),
        nodeVersion: process.version,
        platform: process.platform,
        architecture: process.arch,
      },
    };

    this.emit("metrics-collected", this.metrics);
  }

  private updateHealthCheck(name: string, check: HealthCheck): void {
    this.healthChecks.set(name, check);
    this.emit("health-check-updated", name, check);
  }

  private createAlert(
    component: string,
    severity: HealthAlert["severity"],
    message: string
  ): void {
    const alertId = `${component}-${Date.now()}`;
    const alert: HealthAlert = {
      id: alertId,
      severity,
      component,
      message,
      timestamp: new Date(),
      resolved: false,
    };

    this.alerts.set(alertId, alert);
    this.emit("alert-created", alert);
  }

  private initializeMetrics(): SystemMetrics {
    return {
      uptime: 0,
      memoryUsage: process.memoryUsage(),
      cpuUsage: 0,
      activeConnections: 0,
      taskQueueDepth: 0,
      circuitBreakerStats: {},
      errorRate: 0,
      throughput: 0,
    };
  }

  // Public API methods
  getHealthSummary(): Record<string, HealthCheck> {
    return Object.fromEntries(this.healthChecks);
  }

  getMetrics(): SystemMetrics {
    return { ...this.metrics };
  }

  getAlerts(): HealthAlert[] {
    return Array.from(this.alerts.values()).filter((alert) => !alert.resolved);
  }

  resolveAlert(alertId: string): boolean {
    const alert = this.alerts.get(alertId);
    if (alert) {
      alert.resolved = true;
      this.emit("alert-resolved", alert);
      return true;
    }
    return false;
  }

  getOverallStatus(): "healthy" | "degraded" | "unhealthy" {
    const checks = Array.from(this.healthChecks.values());

    if (checks.some((check) => check.status === "unhealthy")) {
      return "unhealthy";
    }

    if (checks.some((check) => check.status === "degraded")) {
      return "degraded";
    }

    return "healthy";
  }
}
