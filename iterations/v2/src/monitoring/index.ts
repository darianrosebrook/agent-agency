/**
 * System Health Monitor
 *
 * Provides comprehensive system health monitoring, metrics collection,
 * and health assessment for intelligent decision making.
 *
 * @author @darianrosebrook
 */

// Core types
export * from "./types.js";

// Core components
export { MetricsCollector } from "./MetricsCollector.js";
export { SystemHealthMonitor } from "./SystemHealthMonitor.js";

// Default configuration
export const DEFAULT_HEALTH_CONFIG = {
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
} as const;
