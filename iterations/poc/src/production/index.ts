/**
 * Production Hardening Module
 *
 * @author @darianrosebrook
 * @description Enterprise-grade production features for reliability and observability
 */

export { ErrorRecoveryManager } from "./error-recovery.js";
export type {
  CircuitBreakerState,
  ErrorContext,
  RecoveryAction,
  RecoveryConfig,
} from "./error-recovery.js";

export { ProductionMonitor } from "./production-monitor.js";
export type {
  HealthCheck,
  HealthStatus,
  ProductionConfig,
  ProductionMetrics,
  SystemAlert,
} from "./production-monitor.js";
