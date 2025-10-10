/**
 * Performance Monitor - Real-time system monitoring and alerting
 *
 * @author @darianrosebrook
 * @description Monitors system performance, detects bottlenecks, and provides optimization recommendations
 */

import { Logger } from "../utils/Logger.js";

export interface PerformanceMetrics {
  timestamp: number;
  cpu: {
    usage: number;
    loadAverage: number[];
  };
  memory: {
    used: number;
    total: number;
    usage: number;
  };
  disk: {
    readBytes: number;
    writeBytes: number;
    ioWait: number;
  };
  network: {
    bytesReceived: number;
    bytesSent: number;
    connections: number;
  };
  database: {
    connections: number;
    queryCount: number;
    slowQueries: number;
    cacheHitRate: number;
  };
  application: {
    activeRequests: number;
    responseTime: number;
    errorRate: number;
    throughput: number;
  };
}

export interface PerformanceThresholds {
  cpu: { warning: number; critical: number };
  memory: { warning: number; critical: number };
  responseTime: { warning: number; critical: number };
  errorRate: { warning: number; critical: number };
  cacheHitRate: { warning: number; critical: number };
}

export interface PerformanceAlert {
  id: string;
  type: "warning" | "critical";
  metric: string;
  value: number;
  threshold: number;
  timestamp: number;
  message: string;
  recommendations: string[];
}

export interface PerformanceReport {
  period: { start: number; end: number };
  metrics: PerformanceMetrics[];
  alerts: PerformanceAlert[];
  recommendations: PerformanceRecommendation[];
  summary: {
    averageCpuUsage: number;
    peakMemoryUsage: number;
    averageResponseTime: number;
    totalErrors: number;
    overallHealth: "excellent" | "good" | "fair" | "poor" | "critical";
  };
}

export interface PerformanceRecommendation {
  priority: "low" | "medium" | "high" | "critical";
  category: "infrastructure" | "database" | "application" | "caching";
  title: string;
  description: string;
  estimatedBenefit: number;
  complexity: "low" | "medium" | "high";
  implementationSteps: string[];
}

export class PerformanceMonitor {
  private logger: Logger;
  private thresholds: PerformanceThresholds;
  private metrics: PerformanceMetrics[] = [];
  private alerts: PerformanceAlert[] = [];
  private isMonitoring = false;
  private monitoringInterval?: NodeJS.Timeout;
  private alertCallbacks: ((alert: PerformanceAlert) => void)[] = [];

  constructor(
    thresholds: Partial<PerformanceThresholds> = {},
    logger?: Logger
  ) {
    this.logger = logger || new Logger("PerformanceMonitor");
    this.thresholds = {
      cpu: { warning: 70, critical: 90 },
      memory: { warning: 80, critical: 95 },
      responseTime: { warning: 1000, critical: 5000 }, // ms
      errorRate: { warning: 0.05, critical: 0.15 }, // 5%, 15%
      cacheHitRate: { warning: 0.7, critical: 0.5 }, // 70%, 50%
      ...thresholds,
    };
  }

  startMonitoring(intervalMs: number = 30000): void {
    if (this.isMonitoring) {
      this.logger.warn("Performance monitoring already running");
      return;
    }

    this.isMonitoring = true;
    this.logger.info("Starting performance monitoring", { intervalMs });

    this.monitoringInterval = setInterval(async () => {
      try {
        const metrics = await this.collectMetrics();
        this.metrics.push(metrics);

        // Keep only last 1000 metrics (about 8 hours at 30s intervals)
        if (this.metrics.length > 1000) {
          this.metrics.shift();
        }

        // Check thresholds and generate alerts
        await this.checkThresholds(metrics);

        // Analyze trends and generate recommendations
        if (this.metrics.length >= 10) {
          await this.analyzeTrends();
        }
      } catch (error) {
        this.logger.error("Failed to collect performance metrics", {
          error: (error as Error).message,
        });
      }
    }, intervalMs);
  }

  stopMonitoring(): void {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = undefined;
    }
    this.isMonitoring = false;
    this.logger.info("Performance monitoring stopped");
  }

  async getCurrentMetrics(): Promise<PerformanceMetrics> {
    return this.collectMetrics();
  }

  getRecentMetrics(count: number = 10): PerformanceMetrics[] {
    return this.metrics.slice(-count);
  }

  getActiveAlerts(): PerformanceAlert[] {
    // Return alerts from the last hour
    const oneHourAgo = Date.now() - 60 * 60 * 1000;
    return this.alerts.filter((alert) => alert.timestamp > oneHourAgo);
  }

  generateReport(hours: number = 1): PerformanceReport {
    const endTime = Date.now();
    const startTime = endTime - hours * 60 * 60 * 1000;

    const relevantMetrics = this.metrics.filter(
      (m) => m.timestamp >= startTime
    );
    const relevantAlerts = this.alerts.filter((a) => a.timestamp >= startTime);

    return {
      period: { start: startTime, end: endTime },
      metrics: relevantMetrics,
      alerts: relevantAlerts,
      recommendations: this.generateRecommendations(
        relevantMetrics,
        relevantAlerts
      ),
      summary: this.calculateSummary(relevantMetrics),
    };
  }

  onAlert(callback: (alert: PerformanceAlert) => void): void {
    this.alertCallbacks.push(callback);
  }

  private async collectMetrics(): Promise<PerformanceMetrics> {
    const timestamp = Date.now();

    // In a real implementation, these would collect actual system metrics
    // For now, we'll simulate realistic values
    const metrics: PerformanceMetrics = {
      timestamp,
      cpu: {
        usage: 45 + Math.random() * 30, // 45-75%
        loadAverage: [1.2, 1.1, 1.0],
      },
      memory: {
        used: 2.1 * 1024 * 1024 * 1024, // 2.1 GB
        total: 8 * 1024 * 1024 * 1024, // 8 GB
        usage: 26.25,
      },
      disk: {
        readBytes: 1024 * 1024, // 1 MB
        writeBytes: 2 * 1024 * 1024, // 2 MB
        ioWait: 2.5,
      },
      network: {
        bytesReceived: 10 * 1024 * 1024, // 10 MB
        bytesSent: 5 * 1024 * 1024, // 5 MB
        connections: 150,
      },
      database: {
        connections: 25,
        queryCount: 120,
        slowQueries: 3,
        cacheHitRate: 0.85,
      },
      application: {
        activeRequests: 12,
        responseTime: 245,
        errorRate: 0.02,
        throughput: 45,
      },
    };

    return metrics;
  }

  private async checkThresholds(metrics: PerformanceMetrics): Promise<void> {
    const checks = [
      {
        metric: "cpu.usage",
        value: metrics.cpu.usage,
        thresholds: this.thresholds.cpu,
        message: `CPU usage is ${metrics.cpu.usage.toFixed(1)}%`,
      },
      {
        metric: "memory.usage",
        value: metrics.memory.usage,
        thresholds: this.thresholds.memory,
        message: `Memory usage is ${metrics.memory.usage.toFixed(1)}%`,
      },
      {
        metric: "application.responseTime",
        value: metrics.application.responseTime,
        thresholds: this.thresholds.responseTime,
        message: `Average response time is ${metrics.application.responseTime.toFixed(
          0
        )}ms`,
      },
      {
        metric: "application.errorRate",
        value: metrics.application.errorRate,
        thresholds: this.thresholds.errorRate,
        message: `Error rate is ${(metrics.application.errorRate * 100).toFixed(
          2
        )}%`,
      },
      {
        metric: "database.cacheHitRate",
        value: metrics.database.cacheHitRate,
        thresholds: this.thresholds.cacheHitRate,
        message: `Cache hit rate is ${(
          metrics.database.cacheHitRate * 100
        ).toFixed(1)}%`,
        invert: true, // Lower values are worse for cache hit rate
      },
    ];

    for (const check of checks) {
      const alert = this.evaluateThreshold(check);
      if (alert) {
        this.alerts.push(alert);
        this.notifyAlert(alert);
      }
    }

    // Keep only recent alerts (last 24 hours)
    const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000;
    this.alerts = this.alerts.filter((alert) => alert.timestamp > oneDayAgo);
  }

  private evaluateThreshold(check: {
    metric: string;
    value: number;
    thresholds: { warning: number; critical: number };
    message: string;
    invert?: boolean;
  }): PerformanceAlert | null {
    const { metric, value, thresholds, message, invert = false } = check;

    let alertType: "warning" | "critical" | null = null;
    let threshold: number = 0;

    if (invert) {
      if (value <= thresholds.critical) {
        alertType = "critical";
        threshold = thresholds.critical;
      } else if (value <= thresholds.warning) {
        alertType = "warning";
        threshold = thresholds.warning;
      }
    } else {
      if (value >= thresholds.critical) {
        alertType = "critical";
        threshold = thresholds.critical;
      } else if (value >= thresholds.warning) {
        alertType = "warning";
        threshold = thresholds.warning;
      }
    }

    if (!alertType) return null;

    return {
      id: `${metric}_${alertType}_${Date.now()}`,
      type: alertType,
      metric,
      value,
      threshold,
      timestamp: Date.now(),
      message,
      recommendations: this.getRecommendationsForMetric(metric, alertType),
    };
  }

  private getRecommendationsForMetric(
    metric: string,
    _alertType: "warning" | "critical"
  ): string[] {
    const recommendations: Record<string, string[]> = {
      "cpu.usage": [
        "Consider scaling horizontally by adding more instances",
        "Review and optimize CPU-intensive operations",
        "Check for memory leaks that might cause GC pressure",
      ],
      "memory.usage": [
        "Increase memory allocation for the application",
        "Implement memory-efficient data structures",
        "Add memory monitoring and alerts",
      ],
      "application.responseTime": [
        "Optimize database queries with proper indexing",
        "Implement caching for frequently accessed data",
        "Review application code for performance bottlenecks",
      ],
      "application.errorRate": [
        "Check application logs for error patterns",
        "Implement circuit breakers for external services",
        "Add comprehensive error handling and retries",
      ],
      "database.cacheHitRate": [
        "Increase cache size or implement better cache strategies",
        "Review cache invalidation policies",
        "Consider read replicas for better cache performance",
      ],
    };

    return (
      recommendations[metric] || [
        "Investigate the root cause and consider scaling resources",
      ]
    );
  }

  private async analyzeTrends(): Promise<void> {
    const recentMetrics = this.metrics.slice(-20); // Last 20 measurements

    if (recentMetrics.length < 10) return;

    // Analyze CPU usage trend
    const cpuTrend = this.calculateTrend(recentMetrics.map((m) => m.cpu.usage));
    if (cpuTrend > 0.5) {
      // Increasing trend
      this.logger.warn("CPU usage showing upward trend", { trend: cpuTrend });
    }

    // Analyze memory usage trend
    const memoryTrend = this.calculateTrend(
      recentMetrics.map((m) => m.memory.usage)
    );
    if (memoryTrend > 0.3) {
      this.logger.warn("Memory usage showing upward trend", {
        trend: memoryTrend,
      });
    }

    // Analyze response time trend
    const responseTimeTrend = this.calculateTrend(
      recentMetrics.map((m) => m.application.responseTime)
    );
    if (responseTimeTrend > 0.2) {
      this.logger.warn("Response time showing upward trend", {
        trend: responseTimeTrend,
      });
    }
  }

  private calculateTrend(values: number[]): number {
    if (values.length < 2) return 0;

    const n = values.length;
    const sumX = (n * (n - 1)) / 2;
    const sumY = values.reduce((sum, val) => sum + val, 0);
    const sumXY = values.reduce((sum, val, index) => sum + val * index, 0);
    const sumXX = (n * (n - 1) * (2 * n - 1)) / 6;

    const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
    return slope;
  }

  private generateRecommendations(
    metrics: PerformanceMetrics[],
    _alerts: PerformanceAlert[]
  ): PerformanceRecommendation[] {
    const recommendations: PerformanceRecommendation[] = [];

    if (metrics.length === 0) return recommendations;

    const avgMetrics = this.calculateAverageMetrics(metrics);

    // CPU recommendations
    if (avgMetrics.cpu.usage > this.thresholds.cpu.warning) {
      recommendations.push({
        priority:
          avgMetrics.cpu.usage > this.thresholds.cpu.critical
            ? "critical"
            : "high",
        category: "infrastructure",
        title: "High CPU Usage Detected",
        description: `Average CPU usage is ${avgMetrics.cpu.usage.toFixed(1)}%`,
        estimatedBenefit: 30,
        complexity: "medium",
        implementationSteps: [
          "Implement horizontal scaling with load balancer",
          "Profile application code for CPU hotspots",
          "Optimize database queries and indexes",
        ],
      });
    }

    // Memory recommendations
    if (avgMetrics.memory.usage > this.thresholds.memory.warning) {
      recommendations.push({
        priority:
          avgMetrics.memory.usage > this.thresholds.memory.critical
            ? "critical"
            : "high",
        category: "application",
        title: "High Memory Usage Detected",
        description: `Average memory usage is ${avgMetrics.memory.usage.toFixed(
          1
        )}%`,
        estimatedBenefit: 25,
        complexity: "medium",
        implementationSteps: [
          "Implement memory-efficient data structures",
          "Add memory monitoring and garbage collection tuning",
          "Consider increasing memory allocation",
        ],
      });
    }

    // Database recommendations
    if (
      avgMetrics.database.cacheHitRate < this.thresholds.cacheHitRate.warning
    ) {
      recommendations.push({
        priority: "medium",
        category: "database",
        title: "Low Cache Hit Rate",
        description: `Database cache hit rate is ${(
          avgMetrics.database.cacheHitRate * 100
        ).toFixed(1)}%`,
        estimatedBenefit: 40,
        complexity: "low",
        implementationSteps: [
          "Increase database cache size",
          "Implement query result caching",
          "Review cache invalidation policies",
        ],
      });
    }

    return recommendations;
  }

  private calculateAverageMetrics(metrics: PerformanceMetrics[]): {
    cpu: { usage: number };
    memory: { usage: number };
    database: { cacheHitRate: number };
    application: { responseTime: number; errorRate: number };
  } {
    const sums = metrics.reduce(
      (acc, m) => ({
        cpu: acc.cpu + m.cpu.usage,
        memory: acc.memory + m.memory.usage,
        cacheHitRate: acc.cacheHitRate + m.database.cacheHitRate,
        responseTime: acc.responseTime + m.application.responseTime,
        errorRate: acc.errorRate + m.application.errorRate,
      }),
      { cpu: 0, memory: 0, cacheHitRate: 0, responseTime: 0, errorRate: 0 }
    );

    const count = metrics.length;
    return {
      cpu: { usage: sums.cpu / count },
      memory: { usage: sums.memory / count },
      database: { cacheHitRate: sums.cacheHitRate / count },
      application: {
        responseTime: sums.responseTime / count,
        errorRate: sums.errorRate / count,
      },
    };
  }

  private calculateSummary(
    metrics: PerformanceMetrics[]
  ): PerformanceReport["summary"] {
    if (metrics.length === 0) {
      return {
        averageCpuUsage: 0,
        peakMemoryUsage: 0,
        averageResponseTime: 0,
        totalErrors: 0,
        overallHealth: "critical",
      };
    }

    const avgMetrics = this.calculateAverageMetrics(metrics);
    const peakMemoryUsage = Math.max(...metrics.map((m) => m.memory.usage));
    const totalErrors = metrics.reduce(
      (sum, m) => sum + m.application.errorRate * 100,
      0
    );

    // Calculate overall health score
    let healthScore = 100;

    // CPU health
    if (avgMetrics.cpu.usage > this.thresholds.cpu.critical) healthScore -= 30;
    else if (avgMetrics.cpu.usage > this.thresholds.cpu.warning)
      healthScore -= 15;

    // Memory health
    if (avgMetrics.memory.usage > this.thresholds.memory.critical)
      healthScore -= 30;
    else if (avgMetrics.memory.usage > this.thresholds.memory.warning)
      healthScore -= 15;

    // Response time health
    if (
      avgMetrics.application.responseTime >
      this.thresholds.responseTime.critical
    )
      healthScore -= 25;
    else if (
      avgMetrics.application.responseTime > this.thresholds.responseTime.warning
    )
      healthScore -= 10;

    // Error rate health
    if (avgMetrics.application.errorRate > this.thresholds.errorRate.critical)
      healthScore -= 25;
    else if (
      avgMetrics.application.errorRate > this.thresholds.errorRate.warning
    )
      healthScore -= 10;

    // Cache health
    if (
      avgMetrics.database.cacheHitRate < this.thresholds.cacheHitRate.critical
    )
      healthScore -= 20;
    else if (
      avgMetrics.database.cacheHitRate < this.thresholds.cacheHitRate.warning
    )
      healthScore -= 10;

    let overallHealth: PerformanceReport["summary"]["overallHealth"];
    if (healthScore >= 90) overallHealth = "excellent";
    else if (healthScore >= 75) overallHealth = "good";
    else if (healthScore >= 60) overallHealth = "fair";
    else if (healthScore >= 40) overallHealth = "poor";
    else overallHealth = "critical";

    return {
      averageCpuUsage: avgMetrics.cpu.usage,
      peakMemoryUsage,
      averageResponseTime: avgMetrics.application.responseTime,
      totalErrors,
      overallHealth,
    };
  }

  private notifyAlert(alert: PerformanceAlert): void {
    this.logger.warn(`Performance alert: ${alert.message}`, {
      type: alert.type,
      metric: alert.metric,
      value: alert.value,
      threshold: alert.threshold,
    });

    for (const callback of this.alertCallbacks) {
      try {
        callback(alert);
      } catch (error) {
        this.logger.error("Error in alert callback", {
          error: (error as Error).message,
        });
      }
    }
  }
}
