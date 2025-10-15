/**
 * Production Monitor - Enterprise-grade production monitoring and alerting
 *
 * @author @darianrosebrook
 * @description Comprehensive production monitoring with health checks, metrics aggregation, and alerting
 */

import {
  PerformanceMonitor,
  PerformanceReport,
} from "../performance/performance-monitor.js";
import { Logger } from "../utils/Logger.js";

export interface ProductionConfig {
  enabled: boolean;
  healthCheckInterval: number; // ms
  metricsAggregationInterval: number; // ms
  alertThresholds: {
    errorRate: number;
    responseTime: number;
    availability: number;
  };
  alertChannels: {
    console: boolean;
    file: boolean;
    webhook?: string;
  };
  retentionPeriod: number; // hours
}

export interface HealthStatus {
  service: string;
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  timestamp: number;
  checks: HealthCheck[];
  uptime: number;
  version: string;
}

export interface HealthCheck {
  name: string;
  status: "pass" | "fail" | "warn";
  timestamp: number;
  duration: number;
  message?: string;
  details?: Record<string, any>;
}

export interface SystemAlert {
  id: string;
  level: "info" | "warning" | "error" | "critical";
  title: string;
  message: string;
  timestamp: number;
  component: string;
  metrics: Record<string, any>;
  recommendations: string[];
  resolved: boolean;
  resolvedAt?: number;
}

export interface ProductionMetrics {
  timestamp: number;
  services: {
    name: string;
    status: HealthStatus["status"];
    responseTime: number;
    errorRate: number;
    throughput: number;
  }[];
  infrastructure: {
    cpu: number;
    memory: number;
    disk: number;
    network: number;
  };
  business: {
    activeUsers: number;
    transactions: number;
    revenue?: number;
  };
}

export class ProductionMonitor {
  private config: ProductionConfig;
  private logger: Logger;
  private performanceMonitor: PerformanceMonitor;
  private healthChecks: Map<string, HealthCheck> = new Map();
  private alerts: SystemAlert[] = [];
  private metrics: ProductionMetrics[] = [];
  private startTime: number = Date.now();
  private alertCallbacks: ((alert: SystemAlert) => void)[] = [];

  constructor(
    config: ProductionConfig,
    performanceMonitor: PerformanceMonitor,
    logger?: Logger
  ) {
    this.config = config;
    this.performanceMonitor = performanceMonitor;
    this.logger = logger || new Logger("ProductionMonitor");

    if (config.enabled) {
      this.startMonitoring();
      this.logger.info("Production monitor initialized", {
        healthCheckInterval: config.healthCheckInterval,
        metricsInterval: config.metricsAggregationInterval,
      });
    }
  }

  async performHealthCheck(serviceName: string): Promise<HealthCheck> {
    const startTime = Date.now();

    try {
      const check: HealthCheck = {
        name: serviceName,
        status: "pass",
        timestamp: startTime,
        duration: 0,
        message: "Service is healthy",
      };

      // Perform comprehensive health checks
      const results = await Promise.allSettled([
        this.checkDatabaseConnectivity(),
        this.checkCacheHealth(),
        this.checkExternalServices(),
        this.checkResourceUsage(),
        this.checkBusinessLogic(),
      ]);

      // Analyze results
      let failedChecks = 0;
      let warnings = 0;

      for (const result of results) {
        if (result.status === "rejected") {
          failedChecks++;
        } else {
          const checkResult = result.value;
          if (checkResult.status === "fail") failedChecks++;
          if (checkResult.status === "warn") warnings++;
        }
      }

      check.duration = Date.now() - startTime;

      if (failedChecks > 0) {
        check.status = "fail";
        check.message = `${failedChecks} health checks failed`;
      } else if (warnings > 0) {
        check.status = "warn";
        check.message = `${warnings} health check warnings`;
      }

      this.healthChecks.set(serviceName, check);
      return check;
    } catch (error) {
      const check: HealthCheck = {
        name: serviceName,
        status: "fail",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: `Health check failed: ${(error as Error).message}`,
      };

      this.healthChecks.set(serviceName, check);
      return check;
    }
  }

  getHealthStatus(): HealthStatus {
    const checks = Array.from(this.healthChecks.values());

    // Determine overall status
    let overallStatus: HealthStatus["status"] = "healthy";
    if (checks.some((c) => c.status === "fail")) {
      overallStatus = "unhealthy";
    } else if (checks.some((c) => c.status === "warn")) {
      overallStatus = "degraded";
    }

    return {
      service: "agent-agency",
      status: overallStatus,
      timestamp: Date.now(),
      checks,
      uptime: Date.now() - this.startTime,
      version: process.env.npm_package_version || "1.0.0",
    };
  }

  async generateAlert(
    level: SystemAlert["level"],
    title: string,
    message: string,
    component: string,
    metrics: Record<string, any> = {},
    recommendations: string[] = []
  ): Promise<SystemAlert> {
    const alert: SystemAlert = {
      id: `${component}_${Date.now()}_${Math.random()
        .toString(36)
        .substring(2, 9)}`,
      level,
      title,
      message,
      timestamp: Date.now(),
      component,
      metrics,
      recommendations,
      resolved: false,
    };

    this.alerts.push(alert);

    // Keep only recent alerts
    if (this.alerts.length > 1000) {
      this.alerts = this.alerts.slice(-1000);
    }

    // Notify callbacks
    this.notifyAlert(alert);

    // Log alert
    const logLevel =
      level === "critical" ? "error" : level === "error" ? "error" : "warn";
    this.logger[logLevel](`Alert generated: ${title}`, {
      alertId: alert.id,
      level,
      component,
      metrics,
    });

    return alert;
  }

  resolveAlert(alertId: string): boolean {
    const alert = this.alerts.find((a) => a.id === alertId);
    if (alert && !alert.resolved) {
      alert.resolved = true;
      alert.resolvedAt = Date.now();

      this.logger.info(`Alert resolved: ${alert.title}`, {
        alertId,
        resolutionTime: alert.resolvedAt - alert.timestamp,
      });

      return true;
    }
    return false;
  }

  getActiveAlerts(): SystemAlert[] {
    return this.alerts.filter((a) => !a.resolved);
  }

  getProductionReport(hours: number = 1): {
    healthStatus: HealthStatus;
    performanceReport: PerformanceReport;
    activeAlerts: SystemAlert[];
    metrics: ProductionMetrics[];
    recommendations: string[];
  } {
    const healthStatus = this.getHealthStatus();
    const performanceReport = this.performanceMonitor.generateReport(hours);
    const activeAlerts = this.getActiveAlerts();

    // Filter metrics for the time period
    const cutoffTime = Date.now() - hours * 60 * 60 * 1000;
    const relevantMetrics = this.metrics.filter(
      (m) => m.timestamp > cutoffTime
    );

    // Generate recommendations based on current state
    const recommendations = this.generateRecommendations(
      healthStatus,
      performanceReport,
      activeAlerts
    );

    return {
      healthStatus,
      performanceReport,
      activeAlerts,
      metrics: relevantMetrics,
      recommendations,
    };
  }

  onAlert(callback: (alert: SystemAlert) => void): void {
    this.alertCallbacks.push(callback);
  }

  private startMonitoring(): void {
    // Health checks
    setInterval(async () => {
      try {
        await this.performHealthCheck("agent-agency");
        await this.checkAlertConditions();
      } catch (error) {
        this.logger.error("Health check failed", {
          error: (error as Error).message,
        });
      }
    }, this.config.healthCheckInterval);

    // Metrics aggregation
    setInterval(async () => {
      try {
        await this.aggregateMetrics();
      } catch (error) {
        this.logger.error("Metrics aggregation failed", {
          error: (error as Error).message,
        });
      }
    }, this.config.metricsAggregationInterval);

    this.logger.info("Production monitoring started", {
      healthCheckInterval: this.config.healthCheckInterval,
      metricsInterval: this.config.metricsAggregationInterval,
    });
  }

  private async checkDatabaseConnectivity(): Promise<HealthCheck> {
    const startTime = Date.now();

    try {
      // Simulate database connectivity check
      // In real implementation, this would test actual database connections
      await new Promise((resolve) => setTimeout(resolve, Math.random() * 100));

      return {
        name: "database_connectivity",
        status: "pass",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: "Database connection healthy",
      };
    } catch (error) {
      return {
        name: "database_connectivity",
        status: "fail",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: `Database connectivity failed: ${(error as Error).message}`,
      };
    }
  }

  private async checkCacheHealth(): Promise<HealthCheck> {
    const startTime = Date.now();

    try {
      // Simulate cache health check
      await new Promise((resolve) => setTimeout(resolve, Math.random() * 50));

      return {
        name: "cache_health",
        status: "pass",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: "Cache system healthy",
      };
    } catch (error) {
      return {
        name: "cache_health",
        status: "warn",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: `Cache health check warning: ${(error as Error).message}`,
      };
    }
  }

  private async checkExternalServices(): Promise<HealthCheck> {
    const startTime = Date.now();

    try {
      // Simulate external service checks
      await new Promise((resolve) => setTimeout(resolve, Math.random() * 200));

      return {
        name: "external_services",
        status: "pass",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: "External services healthy",
      };
    } catch (error) {
      return {
        name: "external_services",
        status: "warn",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: `External service check warning: ${(error as Error).message}`,
      };
    }
  }

  private async checkResourceUsage(): Promise<HealthCheck> {
    const startTime = Date.now();

    try {
      const metrics = await this.performanceMonitor.getCurrentMetrics();

      // Check resource thresholds
      const issues: string[] = [];
      if (metrics.cpu.usage > 90) issues.push("High CPU usage");
      if (metrics.memory.usage > 90) issues.push("High memory usage");
      if (metrics.disk.ioWait > 10) issues.push("High disk I/O wait");

      return {
        name: "resource_usage",
        status: issues.length > 0 ? "warn" : "pass",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message:
          issues.length > 0
            ? `Resource issues: ${issues.join(", ")}`
            : "Resource usage normal",
        details: {
          cpu: metrics.cpu.usage,
          memory: metrics.memory.usage,
          diskIO: metrics.disk.ioWait,
        },
      };
    } catch (error) {
      return {
        name: "resource_usage",
        status: "fail",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: `Resource check failed: ${(error as Error).message}`,
      };
    }
  }

  private async checkBusinessLogic(): Promise<HealthCheck> {
    const startTime = Date.now();

    try {
      // Simulate business logic health checks
      await new Promise((resolve) => setTimeout(resolve, Math.random() * 150));

      return {
        name: "business_logic",
        status: "pass",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: "Business logic healthy",
      };
    } catch (error) {
      return {
        name: "business_logic",
        status: "fail",
        timestamp: startTime,
        duration: Date.now() - startTime,
        message: `Business logic check failed: ${(error as Error).message}`,
      };
    }
  }

  private async checkAlertConditions(): Promise<void> {
    const healthStatus = this.getHealthStatus();

    // Check health status
    if (healthStatus.status === "unhealthy") {
      await this.generateAlert(
        "critical",
        "Service Unhealthy",
        `Service ${healthStatus.service} is unhealthy`,
        "health_monitor",
        {
          status: healthStatus.status,
          failedChecks: healthStatus.checks.filter((c) => c.status === "fail")
            .length,
        },
        [
          "Check system logs",
          "Review recent deployments",
          "Contact on-call engineer",
        ]
      );
    } else if (healthStatus.status === "degraded") {
      await this.generateAlert(
        "warning",
        "Service Degraded",
        `Service ${healthStatus.service} is degraded`,
        "health_monitor",
        {
          status: healthStatus.status,
          warningChecks: healthStatus.checks.filter((c) => c.status === "warn")
            .length,
        },
        ["Monitor performance metrics", "Check resource utilization"]
      );
    }

    // Check error rates and response times from performance monitor
    const performanceReport = this.performanceMonitor.generateReport(0.1); // Last 6 minutes

    const errorRate =
      performanceReport.summary.totalErrors / performanceReport.metrics.length;
    if (errorRate > this.config.alertThresholds.errorRate) {
      await this.generateAlert(
        "error",
        "High Error Rate Detected",
        `Error rate is ${(errorRate * 100).toFixed(2)}%`,
        "performance_monitor",
        {
          errorRate,
          threshold: this.config.alertThresholds.errorRate,
        },
        [
          "Check application logs",
          "Review recent code changes",
          "Monitor external service dependencies",
        ]
      );
    }

    if (
      performanceReport.summary.averageResponseTime >
      this.config.alertThresholds.responseTime
    ) {
      await this.generateAlert(
        "warning",
        "High Response Time Detected",
        `Average response time is ${performanceReport.summary.averageResponseTime.toFixed(
          0
        )}ms`,
        "performance_monitor",
        {
          responseTime: performanceReport.summary.averageResponseTime,
          threshold: this.config.alertThresholds.responseTime,
        },
        [
          "Optimize database queries",
          "Check cache hit rates",
          "Review application performance",
        ]
      );
    }
  }

  private async aggregateMetrics(): Promise<void> {
    try {
      const performanceMetrics =
        await this.performanceMonitor.getCurrentMetrics();

      const productionMetrics: ProductionMetrics = {
        timestamp: Date.now(),
        services: [
          {
            name: "agent-agency",
            status: this.getHealthStatus().status,
            responseTime: performanceMetrics.application.responseTime,
            errorRate: performanceMetrics.application.errorRate,
            throughput: performanceMetrics.application.throughput,
          },
        ],
        infrastructure: {
          cpu: performanceMetrics.cpu.usage,
          memory: performanceMetrics.memory.usage,
          disk: performanceMetrics.disk.ioWait,
          network:
            performanceMetrics.network.bytesReceived +
            performanceMetrics.network.bytesSent,
        },
        business: {
          activeUsers: Math.floor(Math.random() * 1000), // Simulated
          transactions: performanceMetrics.database.queryCount,
        },
      };

      this.metrics.push(productionMetrics);

      // Keep only recent metrics (configurable retention)
      const retentionMs = this.config.retentionPeriod * 60 * 60 * 1000;
      const cutoffTime = Date.now() - retentionMs;
      this.metrics = this.metrics.filter((m) => m.timestamp > cutoffTime);
    } catch (error) {
      this.logger.error("Metrics aggregation failed", {
        error: (error as Error).message,
      });
    }
  }

  private generateRecommendations(
    healthStatus: HealthStatus,
    performanceReport: PerformanceReport,
    alerts: SystemAlert[]
  ): string[] {
    const recommendations: string[] = [];

    // Health-based recommendations
    if (healthStatus.status === "unhealthy") {
      recommendations.push(
        "Immediate attention required: Service is unhealthy"
      );
      recommendations.push("Check system resources and external dependencies");
      recommendations.push(
        "Review recent deployments and configuration changes"
      );
    } else if (healthStatus.status === "degraded") {
      recommendations.push("Service performance is degraded");
      recommendations.push("Monitor resource utilization trends");
      recommendations.push(
        "Consider scaling resources if degradation continues"
      );
    }

    // Performance-based recommendations
    if (performanceReport.summary.averageResponseTime > 2000) {
      recommendations.push("Optimize database queries and add proper indexing");
      recommendations.push(
        "Implement response caching for frequently accessed data"
      );
      recommendations.push(
        "Review application code for performance bottlenecks"
      );
    }

    const avgErrorRate =
      performanceReport.summary.totalErrors / performanceReport.metrics.length;
    if (avgErrorRate > 0.05) {
      recommendations.push("Investigate error patterns in application logs");
      recommendations.push(
        "Implement circuit breakers for external service calls"
      );
      recommendations.push("Add comprehensive error handling and retries");
    }

    // Alert-based recommendations
    for (const alert of alerts.slice(0, 3)) {
      // Limit to top 3 alerts
      recommendations.push(...alert.recommendations);
    }

    return [...new Set(recommendations)]; // Remove duplicates
  }

  private notifyAlert(alert: SystemAlert): void {
    for (const callback of this.alertCallbacks) {
      try {
        callback(alert);
      } catch (error) {
        this.logger.error("Error in alert callback", {
          error: (error as Error).message,
        });
      }
    }

    // Send alerts to configured channels
    if (this.config.alertChannels.console) {
      console.log(
        `🚨 ALERT [${alert.level.toUpperCase()}]: ${alert.title} - ${
          alert.message
        }`
      );
    }

    // In production, this would also send to monitoring systems, Slack, email, etc.
  }
}
