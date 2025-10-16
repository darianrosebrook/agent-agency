/**
 * Budget Monitor Types
 *
 * Type definitions for real-time budget monitoring and alerting system.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import type { WorkingSpec } from "../../types/caws-types.js";

/**
 * Budget usage statistics
 */
export interface BudgetUsage {
  /** Current number of files changed */
  filesChanged: number;

  /** Maximum files allowed */
  maxFiles: number;

  /** Files usage percentage (0-100) */
  filesPercentage: number;

  /** Current lines of code changed */
  linesChanged: number;

  /** Maximum LOC allowed */
  maxLoc: number;

  /** LOC usage percentage (0-100) */
  locPercentage: number;

  /** List of changed files */
  changedFiles: string[];

  /** Timestamp of last update */
  lastUpdated: string;
}

/**
 * Budget alert severity levels
 */
export type AlertSeverity = "info" | "warning" | "critical";

/**
 * Budget threshold alert
 */
export interface BudgetAlert {
  /** Alert severity level */
  severity: AlertSeverity;

  /** Budget type (files or LOC) */
  type: "files" | "loc";

  /** Threshold that triggered alert (0-1) */
  threshold: number;

  /** Current usage percentage */
  currentPercentage: number;

  /** Human-readable message */
  message: string;

  /** Timestamp of alert */
  timestamp: string;

  /** Whether alert has been acknowledged */
  acknowledged?: boolean;
}

/**
 * Budget monitoring configuration
 */
export interface BudgetMonitorConfig {
  /** Project root directory to monitor */
  projectRoot: string;

  /** Working spec to monitor against */
  spec: WorkingSpec;

  /** Thresholds for alerts (0-1, e.g., 0.5 = 50%) */
  thresholds?: {
    /** Warning threshold (default: 0.5) */
    warning?: number;
    /** Critical threshold (default: 0.8) */
    critical?: number;
    /** Exceeded threshold (default: 0.95) */
    exceeded?: number;
  };

  /** Polling interval in ms (if not using file watching) */
  pollingInterval?: number;

  /** Whether to use file system watching (default: true) */
  useFileWatching?: boolean;

  /** File patterns to watch (globs) */
  watchPatterns?: string[];

  /** File patterns to ignore (globs) */
  ignorePatterns?: string[];

  /** Alert callback function */
  onAlert?: (alert: BudgetAlert) => void | Promise<void>;

  /** Budget update callback function */
  onBudgetUpdate?: (usage: BudgetUsage) => void | Promise<void>;
}

/**
 * File change event from watcher
 */
export interface FileChangeEvent {
  /** Type of change */
  type: "add" | "change" | "unlink";

  /** Absolute path to file */
  path: string;

  /** Relative path from project root */
  relativePath: string;

  /** File statistics */
  stats?: {
    /** File size in bytes */
    size: number;
    /** Number of lines */
    lines?: number;
    /** Last modified timestamp */
    mtime: Date;
  };

  /** Timestamp of event */
  timestamp: string;
}

/**
 * Budget monitoring status
 */
export interface MonitoringStatus {
  /** Whether monitoring is active */
  active: boolean;

  /** Monitoring start timestamp */
  startedAt?: string;

  /** Monitoring stop timestamp */
  stoppedAt?: string;

  /** Total number of file changes detected */
  totalChanges: number;

  /** Total number of alerts generated */
  totalAlerts: number;

  /** Current budget usage */
  currentUsage: BudgetUsage;

  /** Recent alerts (last 10) */
  recentAlerts: BudgetAlert[];

  /** Monitoring errors (if any) */
  errors?: Error[];
}

/**
 * Budget monitor event emitter events
 */
export interface BudgetMonitorEvents {
  /** Emitted when budget usage is updated */
  "budget:update": (usage: BudgetUsage) => void;

  /** Emitted when an alert is triggered */
  "budget:alert": (alert: BudgetAlert) => void;

  /** Emitted when a threshold is crossed */
  "budget:threshold": (threshold: number, type: "files" | "loc") => void;

  /** Emitted when budget is exceeded */
  "budget:exceeded": (type: "files" | "loc") => void;

  /** Emitted when a file change is detected */
  "file:change": (event: FileChangeEvent) => void;

  /** Emitted when monitoring starts */
  "monitor:start": () => void;

  /** Emitted when monitoring stops */
  "monitor:stop": () => void;

  /** Emitted on monitoring error */
  "monitor:error": (error: Error) => void;
}

/**
 * Budget statistics for reporting
 */
export interface BudgetStatistics {
  /** Total monitoring duration in ms */
  monitoringDuration: number;

  /** Total file changes detected */
  totalFileChanges: number;

  /** Total LOC changes */
  totalLocChanges: number;

  /** Peak files usage percentage */
  peakFilesUsage: number;

  /** Peak LOC usage percentage */
  peakLocUsage: number;

  /** Number of alerts by severity */
  alertsBySeverity: {
    info: number;
    warning: number;
    critical: number;
  };

  /** Average time between changes (ms) */
  averageTimeBetweenChanges?: number;

  /** Most frequently changed files */
  frequentlyChangedFiles: Array<{
    path: string;
    changeCount: number;
  }>;
}

/**
 * Budget recommendation for optimization
 */
export interface BudgetRecommendation {
  /** Recommendation type */
  type: "refactor" | "split" | "optimize" | "warning";

  /** Recommendation message */
  message: string;

  /** Affected files or areas */
  affectedAreas: string[];

  /** Priority level */
  priority: "low" | "medium" | "high";

  /** Estimated impact on budget */
  estimatedImpact?: {
    files?: number;
    loc?: number;
  };
}
