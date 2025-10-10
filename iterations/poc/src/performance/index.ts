/**
 * Performance Optimization Module
 *
 * @author @darianrosebrook
 * @description High-performance caching, query optimization, and monitoring
 */

export { CacheManager } from "./cache-manager.js";
export type {
  CacheConfig,
  CacheEntry,
  CacheMetadata,
  CacheStats,
} from "./cache-manager.js";

export { QueryOptimizer } from "./query-optimizer.js";
export type {
  IndexRecommendation,
  JoinInfo,
  OptimizationOpportunity,
  QueryAnalysis,
  QueryCondition,
  QueryPlan,
  SortInfo,
} from "./query-optimizer.js";

export { PerformanceMonitor } from "./performance-monitor.js";
export type {
  PerformanceAlert,
  PerformanceMetrics,
  PerformanceRecommendation,
  PerformanceReport,
  PerformanceThresholds,
} from "./performance-monitor.js";
