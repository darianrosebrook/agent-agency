/**
 * Budget Monitor
 *
 * Real-time budget monitoring with file system watching and threshold alerts.
 * Monitors file and LOC changes against CAWS budget limits.
 *
 * @author @darianrosebrook
 */

import * as chokidar from "chokidar";
import { EventEmitter } from "events";
import * as fs from "fs/promises";
import * as path from "path";
import { CAWSPolicyAdapter } from "../caws-integration/adapters/CAWSPolicyAdapter.js";
import type {
  AlertSeverity,
  BudgetAlert,
  BudgetMonitorConfig,
  BudgetMonitorEvents,
  BudgetRecommendation,
  BudgetStatistics,
  BudgetUsage,
  FileChangeEvent,
  MonitoringStatus,
} from "./types/budget-monitor-types.js";

/**
 * Real-time budget monitor with file watching
 *
 * Features:
 * - Real-time file system monitoring with chokidar
 * - Threshold alerts (warning, critical, exceeded)
 * - Budget usage tracking (files and LOC)
 * - Event-driven architecture
 * - Statistics and recommendations
 */
export class BudgetMonitor extends EventEmitter {
  private config: BudgetMonitorConfig & {
    thresholds: Required<NonNullable<BudgetMonitorConfig["thresholds"]>>;
    pollingInterval: number;
    useFileWatching: boolean;
    watchPatterns: string[];
    ignorePatterns: string[];
  };
  private watcher?: chokidar.FSWatcher;
  private policyAdapter: CAWSPolicyAdapter;

  private currentUsage: BudgetUsage;
  private alerts: BudgetAlert[] = [];
  private fileChanges: Map<string, number> = new Map();
  private startTime?: Date;
  private stopTime?: Date;
  private totalChanges: number = 0;
  private peakFilesUsage: number = 0;
  private peakLocUsage: number = 0;

  /** Default thresholds */
  private static readonly DEFAULT_THRESHOLDS = {
    warning: 0.5, // 50%
    critical: 0.8, // 80%
    exceeded: 0.95, // 95%
  };

  /** Default watch patterns */
  private static readonly DEFAULT_WATCH_PATTERNS = [
    "**/*.{ts,tsx,js,jsx,mjs,cjs}",
  ];

  /** Default ignore patterns */
  private static readonly DEFAULT_IGNORE_PATTERNS = [
    "**/node_modules/**",
    "**/dist/**",
    "**/build/**",
    "**/.git/**",
    "**/coverage/**",
    "**/*.test.{ts,tsx,js,jsx}",
    "**/*.spec.{ts,tsx,js,jsx}",
  ];

  constructor(config: BudgetMonitorConfig) {
    super();

    // Merge with defaults
    this.config = {
      ...config,
      thresholds: { ...BudgetMonitor.DEFAULT_THRESHOLDS, ...config.thresholds },
      pollingInterval: config.pollingInterval ?? 1000,
      useFileWatching: config.useFileWatching ?? true,
      watchPatterns:
        config.watchPatterns ?? BudgetMonitor.DEFAULT_WATCH_PATTERNS,
      ignorePatterns:
        config.ignorePatterns ?? BudgetMonitor.DEFAULT_IGNORE_PATTERNS,
    };

    this.policyAdapter = new CAWSPolicyAdapter({
      projectRoot: config.projectRoot,
    });

    // Initialize current usage
    this.currentUsage = {
      filesChanged: 0,
      maxFiles: 0,
      filesPercentage: 0,
      linesChanged: 0,
      maxLoc: 0,
      locPercentage: 0,
      changedFiles: [],
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Start monitoring
   */
  async start(): Promise<void> {
    if (this.watcher) {
      throw new Error("Monitor is already running");
    }

    this.startTime = new Date();
    this.stopTime = undefined;

    // Get budget limits
    await this.updateBudgetLimits();

    if (this.config.useFileWatching) {
      await this.startFileWatching();
    }

    // Perform initial budget calculation
    await this.calculateCurrentUsage();

    this.emit("monitor:start");
  }

  /**
   * Stop monitoring
   */
  async stop(): Promise<void> {
    if (!this.watcher) {
      return;
    }

    await this.watcher.close();
    this.watcher = undefined;
    this.stopTime = new Date();

    this.emit("monitor:stop");
  }

  /**
   * Get current monitoring status
   */
  getStatus(): MonitoringStatus {
    return {
      active: this.watcher !== undefined,
      startedAt: this.startTime?.toISOString(),
      stoppedAt: this.stopTime?.toISOString(),
      totalChanges: this.totalChanges,
      totalAlerts: this.alerts.length,
      currentUsage: { ...this.currentUsage },
      recentAlerts: this.alerts.slice(-10),
    };
  }

  /**
   * Get budget statistics
   */
  getStatistics(): BudgetStatistics {
    const duration = this.stopTime
      ? this.stopTime.getTime() - (this.startTime?.getTime() ?? 0)
      : Date.now() - (this.startTime?.getTime() ?? Date.now());

    const alertsBySeverity = this.alerts.reduce(
      (acc, alert) => {
        acc[alert.severity]++;
        return acc;
      },
      { info: 0, warning: 0, critical: 0 } as Record<AlertSeverity, number>
    );

    // Calculate frequently changed files
    const frequentlyChangedFiles = Array.from(this.fileChanges.entries())
      .map(([path, count]) => ({ path, changeCount: count }))
      .sort((a, b) => b.changeCount - a.changeCount)
      .slice(0, 10);

    return {
      monitoringDuration: duration,
      totalFileChanges: this.currentUsage.filesChanged,
      totalLocChanges: this.currentUsage.linesChanged,
      peakFilesUsage: this.peakFilesUsage,
      peakLocUsage: this.peakLocUsage,
      alertsBySeverity,
      averageTimeBetweenChanges:
        this.totalChanges > 1 ? duration / this.totalChanges : undefined,
      frequentlyChangedFiles,
    };
  }

  /**
   * Get budget recommendations
   */
  getRecommendations(): BudgetRecommendation[] {
    const recommendations: BudgetRecommendation[] = [];
    const { filesPercentage, locPercentage } = this.currentUsage;

    // Check if approaching budget limits
    if (filesPercentage > 80) {
      recommendations.push({
        type: "warning",
        message: `Files budget at ${filesPercentage.toFixed(
          1
        )}% - consider splitting work into smaller tasks`,
        affectedAreas: ["budget:files"],
        priority: "high",
      });
    }

    if (locPercentage > 80) {
      recommendations.push({
        type: "warning",
        message: `LOC budget at ${locPercentage.toFixed(
          1
        )}% - consider refactoring or splitting changes`,
        affectedAreas: ["budget:loc"],
        priority: "high",
      });
    }

    // Check for frequently changed files
    const stats = this.getStatistics();
    const hotFiles = stats.frequentlyChangedFiles.filter(
      (f) => f.changeCount > 5
    );

    if (hotFiles.length > 0) {
      recommendations.push({
        type: "refactor",
        message: `${hotFiles.length} files changed frequently - consider refactoring`,
        affectedAreas: hotFiles.map((f) => f.path),
        priority: "medium",
        estimatedImpact: {
          files: hotFiles.length,
        },
      });
    }

    // Check if budget is being exceeded
    if (filesPercentage > 100 || locPercentage > 100) {
      recommendations.push({
        type: "split",
        message:
          "Budget exceeded - split work into multiple tasks with separate specs",
        affectedAreas: ["budget:overall"],
        priority: "high",
      });
    }

    return recommendations;
  }

  /**
   * Acknowledge an alert
   */
  acknowledgeAlert(alertIndex: number): void {
    if (alertIndex >= 0 && alertIndex < this.alerts.length) {
      this.alerts[alertIndex].acknowledged = true;
    }
  }

  /**
   * Reset monitoring state
   */
  reset(): void {
    this.alerts = [];
    this.fileChanges.clear();
    this.totalChanges = 0;
    this.peakFilesUsage = 0;
    this.peakLocUsage = 0;

    this.currentUsage = {
      filesChanged: 0,
      maxFiles: this.currentUsage.maxFiles,
      filesPercentage: 0,
      linesChanged: 0,
      maxLoc: this.currentUsage.maxLoc,
      locPercentage: 0,
      changedFiles: [],
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Start file system watching
   */
  private async startFileWatching(): Promise<void> {
    const watchPaths = this.config.spec.scope.in.map((p) =>
      path.join(this.config.projectRoot, p)
    );

    this.watcher = chokidar.watch(watchPaths, {
      ignored: this.config.ignorePatterns,
      persistent: true,
      ignoreInitial: true, // Don't trigger on initial scan
      awaitWriteFinish: {
        stabilityThreshold: 100,
        pollInterval: 50,
      },
    });

    this.watcher
      .on("add", (filePath) => this.handleFileChange("add", filePath))
      .on("change", (filePath) => this.handleFileChange("change", filePath))
      .on("unlink", (filePath) => this.handleFileChange("unlink", filePath))
      .on("error", (error) => this.handleError(error));
  }

  /**
   * Handle file change event
   */
  private async handleFileChange(
    type: "add" | "change" | "unlink",
    filePath: string
  ): Promise<void> {
    try {
      const relativePath = path.relative(this.config.projectRoot, filePath);

      // Track file changes
      const currentCount = this.fileChanges.get(relativePath) ?? 0;
      this.fileChanges.set(relativePath, currentCount + 1);
      this.totalChanges++;

      // Get file stats
      let stats: FileChangeEvent["stats"] | undefined;
      if (type !== "unlink") {
        const fileStats = await fs.stat(filePath);
        const content = await fs.readFile(filePath, "utf-8");
        const lines = content.split("\n").length;

        stats = {
          size: fileStats.size,
          lines,
          mtime: fileStats.mtime,
        };
      }

      const event: FileChangeEvent = {
        type,
        path: filePath,
        relativePath,
        stats,
        timestamp: new Date().toISOString(),
      };

      this.emit("file:change", event);

      // Recalculate budget usage
      await this.calculateCurrentUsage();
    } catch (error) {
      this.handleError(error as Error);
    }
  }

  /**
   * Update budget limits from policy
   */
  private async updateBudgetLimits(): Promise<void> {
    const budgetResult = await this.policyAdapter.deriveBudget({
      spec: this.config.spec,
      projectRoot: this.config.projectRoot,
      applyWaivers: true,
    });

    if (budgetResult.success && budgetResult.data) {
      this.currentUsage.maxFiles = budgetResult.data.effective.max_files;
      this.currentUsage.maxLoc = budgetResult.data.effective.max_loc;
    }
  }

  /**
   * Calculate current budget usage
   */
  private async calculateCurrentUsage(): Promise<void> {
    const scopePaths = this.config.spec.scope.in.map((p) =>
      path.join(this.config.projectRoot, p)
    );

    let totalFiles = 0;
    let totalLoc = 0;
    const changedFiles: string[] = [];

    // Scan all files in scope
    for (const scopePath of scopePaths) {
      try {
        const stats = await fs.stat(scopePath);

        if (stats.isDirectory()) {
          const files = await this.scanDirectory(scopePath);
          totalFiles += files.length;
          changedFiles.push(...files);

          // Count LOC
          for (const file of files) {
            const content = await fs.readFile(file, "utf-8");
            totalLoc += content.split("\n").length;
          }
        } else if (stats.isFile()) {
          totalFiles++;
          changedFiles.push(scopePath);
          const content = await fs.readFile(scopePath, "utf-8");
          totalLoc += content.split("\n").length;
        }
      } catch (error) {
        // Skip files that don't exist or can't be read
        continue;
      }
    }

    // Update usage
    const filesPercentage =
      this.currentUsage.maxFiles > 0
        ? (totalFiles / this.currentUsage.maxFiles) * 100
        : 0;
    const locPercentage =
      this.currentUsage.maxLoc > 0
        ? (totalLoc / this.currentUsage.maxLoc) * 100
        : 0;

    this.currentUsage = {
      filesChanged: totalFiles,
      maxFiles: this.currentUsage.maxFiles,
      filesPercentage,
      linesChanged: totalLoc,
      maxLoc: this.currentUsage.maxLoc,
      locPercentage,
      changedFiles: changedFiles.map((f) =>
        path.relative(this.config.projectRoot, f)
      ),
      lastUpdated: new Date().toISOString(),
    };

    // Update peaks
    this.peakFilesUsage = Math.max(this.peakFilesUsage, filesPercentage);
    this.peakLocUsage = Math.max(this.peakLocUsage, locPercentage);

    // Check thresholds and generate alerts
    this.checkThresholds();

    // Emit update event
    this.emit("budget:update", this.currentUsage);
    if (this.config.onBudgetUpdate) {
      await this.config.onBudgetUpdate(this.currentUsage);
    }
  }

  /**
   * Scan directory recursively for files
   */
  private async scanDirectory(dirPath: string): Promise<string[]> {
    const files: string[] = [];

    try {
      const entries = await fs.readdir(dirPath, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(dirPath, entry.name);

        // Check ignore patterns
        const shouldIgnore = this.config.ignorePatterns.some((pattern) => {
          const regex = new RegExp(pattern.replace(/\*/g, ".*"));
          return regex.test(fullPath);
        });

        if (shouldIgnore) {
          continue;
        }

        if (entry.isDirectory()) {
          const subFiles = await this.scanDirectory(fullPath);
          files.push(...subFiles);
        } else if (entry.isFile()) {
          // Check watch patterns
          const matchesPattern = this.config.watchPatterns.some((pattern) => {
            const regex = new RegExp(pattern.replace(/\*/g, ".*"));
            return regex.test(fullPath);
          });

          if (matchesPattern) {
            files.push(fullPath);
          }
        }
      }
    } catch (error) {
      // Skip directories that can't be read
    }

    return files;
  }

  /**
   * Check thresholds and generate alerts
   */
  private checkThresholds(): void {
    const { filesPercentage, locPercentage } = this.currentUsage;
    const { warning, critical, exceeded } = this.config.thresholds;

    // Check files budget
    this.checkThreshold("files", filesPercentage, warning, critical, exceeded);

    // Check LOC budget
    this.checkThreshold("loc", locPercentage, warning, critical, exceeded);
  }

  /**
   * Check individual threshold
   */
  private checkThreshold(
    type: "files" | "loc",
    percentage: number,
    warning: number,
    critical: number,
    exceeded: number
  ): void {
    const percentageDecimal = percentage / 100;

    let severity: AlertSeverity | null = null;
    let threshold: number | null = null;

    if (percentageDecimal >= exceeded) {
      severity = "critical";
      threshold = exceeded;
      this.emit("budget:exceeded", type);
    } else if (percentageDecimal >= critical) {
      severity = "critical";
      threshold = critical;
    } else if (percentageDecimal >= warning) {
      severity = "warning";
      threshold = warning;
    }

    if (severity && threshold) {
      // Check if we already have a recent alert for this threshold
      const recentAlert = this.alerts
        .slice(-5)
        .find(
          (a) =>
            a.type === type &&
            a.severity === severity &&
            Math.abs(a.threshold - threshold) < 0.01
        );

      if (!recentAlert) {
        const alert: BudgetAlert = {
          severity,
          type,
          threshold,
          currentPercentage: percentage,
          message:
            type === "files"
              ? `Files budget at ${percentage.toFixed(1)}% (${
                  this.currentUsage.filesChanged
                }/${this.currentUsage.maxFiles})`
              : `LOC budget at ${percentage.toFixed(1)}% (${
                  this.currentUsage.linesChanged
                }/${this.currentUsage.maxLoc})`,
          timestamp: new Date().toISOString(),
        };

        this.alerts.push(alert);
        this.emit("budget:alert", alert);
        this.emit("budget:threshold", threshold, type);

        if (this.config.onAlert) {
          void this.config.onAlert(alert);
        }
      }
    }
  }

  /**
   * Handle monitoring error
   */
  private handleError(error: Error): void {
    console.error("[BudgetMonitor] Error:", error);
    this.emit("monitor:error", error);
  }

  /**
   * Typed event emitter methods
   */
  on<K extends keyof BudgetMonitorEvents>(
    event: K,
    listener: BudgetMonitorEvents[K]
  ): this {
    return super.on(event, listener);
  }

  emit<K extends keyof BudgetMonitorEvents>(
    event: K,
    ...args: Parameters<BudgetMonitorEvents[K]>
  ): boolean {
    return super.emit(event, ...args);
  }
}
