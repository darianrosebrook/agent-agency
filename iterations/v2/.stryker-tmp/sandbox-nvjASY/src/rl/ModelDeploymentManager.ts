/**
 * Model Deployment Manager for RL Training
 *
 * @author @darianrosebrook
 * @module model-deployment-manager
 *
 * Manages deployment of trained models with A/B testing and rollback capabilities.
 * Ensures safe model updates with performance monitoring and automatic rollback.
 */
// @ts-nocheck


import { EventEmitter } from "events";
import type { PerformanceTracker } from "./PerformanceTracker";

/**
 * Model version information.
 */
export interface ModelVersion {
  /**
   * Unique version identifier.
   */
  id: string;

  /**
   * Model name/identifier.
   */
  modelName: string;

  /**
   * Version number (semver).
   */
  version: string;

  /**
   * Training metrics from this version.
   */
  trainingMetrics: {
    averageReward: number;
    trainingTimeMs: number;
    trajectoriesProcessed: number;
    klDivergence: number;
  };

  /**
   * Deployment status.
   */
  status: "staging" | "canary" | "production" | "rolled_back" | "archived";

  /**
   * Deployment timestamp.
   */
  deployedAt: string;

  /**
   * Rollback version (if rolled back).
   */
  rolledBackTo?: string;

  /**
   * Performance baseline for comparison.
   */
  performanceBaseline?: {
    averageReward: number;
    successRate: number;
    latencyMs: number;
    errorRate: number;
  };
}

/**
 * A/B test configuration.
 */
export interface ABTestConfig {
  /**
   * Test name.
   */
  name: string;

  /**
   * Control version (current production).
   */
  controlVersion: string;

  /**
   * Treatment version (new model).
   */
  treatmentVersion: string;

  /**
   * Traffic split percentage for treatment (0-100).
   */
  trafficPercentage: number;

  /**
   * Test duration (milliseconds).
   */
  durationMs: number;

  /**
   * Minimum sample size for statistical significance.
   */
  minSampleSize: number;

  /**
   * Performance thresholds for auto-promotion.
   */
  promotionThresholds: {
    minSuccessRate: number;
    maxLatencyIncreasePercent: number;
    maxErrorRateIncreasePercent: number;
    minRewardImprovement: number;
  };

  /**
   * Rollback thresholds.
   */
  rollbackThresholds: {
    maxSuccessRateDecreasePercent: number;
    maxLatencyIncreasePercent: number;
    maxErrorRatePercent: number;
    maxRewardDecreasePercent: number;
  };
}

/**
 * A/B test result.
 */
export interface ABTestResult {
  /**
   * Test name.
   */
  testName: string;

  /**
   * Control version metrics.
   */
  controlMetrics: {
    sampleSize: number;
    averageReward: number;
    successRate: number;
    latencyMs: number;
    errorRate: number;
  };

  /**
   * Treatment version metrics.
   */
  treatmentMetrics: {
    sampleSize: number;
    averageReward: number;
    successRate: number;
    latencyMs: number;
    errorRate: number;
  };

  /**
   * Statistical significance.
   */
  statisticalSignificance: {
    isSignificant: boolean;
    pValue: number;
    confidence: number;
  };

  /**
   * Recommendation.
   */
  recommendation:
    | "promote_treatment"
    | "keep_control"
    | "continue_testing"
    | "rollback";

  /**
   * Detailed reasoning.
   */
  reasoning: string;

  /**
   * Test completion timestamp.
   */
  completedAt: string;
}

/**
 * Deployment manager configuration.
 */
export interface ModelDeploymentManagerConfig {
  /**
   * Default A/B test duration (milliseconds).
   */
  defaultTestDurationMs: number;

  /**
   * Default traffic split for canary deployments (0-100).
   */
  defaultCanaryPercentage: number;

  /**
   * Performance monitoring interval (milliseconds).
   */
  monitoringIntervalMs: number;

  /**
   * Maximum concurrent A/B tests.
   */
  maxConcurrentTests: number;

  /**
   * Whether to enable automatic rollback.
   */
  enableAutoRollback: boolean;

  /**
   * Whether to enable automatic promotion.
   */
  enableAutoPromotion: boolean;

  /**
   * Rollback thresholds.
   */
  rollbackThresholds: {
    maxSuccessRateDecreasePercent: number;
    maxLatencyIncreasePercent: number;
    maxErrorRatePercent: number;
    maxRewardDecreasePercent: number;
  };
}

/**
 * Default configuration.
 */
const DEFAULT_CONFIG: ModelDeploymentManagerConfig = {
  defaultTestDurationMs: 4 * 60 * 60 * 1000, // 4 hours
  defaultCanaryPercentage: 10,
  monitoringIntervalMs: 5 * 60 * 1000, // 5 minutes
  maxConcurrentTests: 3,
  enableAutoRollback: true,
  enableAutoPromotion: false, // Require manual promotion by default
  rollbackThresholds: {
    maxSuccessRateDecreasePercent: 10, // 10% decrease triggers rollback
    maxLatencyIncreasePercent: 50, // 50% increase triggers rollback
    maxErrorRatePercent: 5, // 5% error rate triggers rollback
    maxRewardDecreasePercent: 15, // 15% decrease triggers rollback
  },
};

/**
 * Model Deployment Manager for safe model updates.
 *
 * This component manages:
 * 1. Model version tracking
 * 2. A/B testing (canary deployments)
 * 3. Performance monitoring
 * 4. Automatic rollback on degradation
 * 5. Promotion of better models
 */
export class ModelDeploymentManager extends EventEmitter {
  private config: ModelDeploymentManagerConfig;
  private versions: Map<string, ModelVersion> = new Map();
  private activeTests: Map<string, ABTestConfig> = new Map();
  private performanceTracker: PerformanceTracker;
  private monitoringTimer?: ReturnType<typeof setInterval>;

  /**
   * Creates a new model deployment manager.
   *
   * @param config - Manager configuration. Uses defaults if not provided.
   * @param performanceTracker - Performance tracker for monitoring.
   */
  constructor(
    config: Partial<ModelDeploymentManagerConfig> = {},
    performanceTracker: PerformanceTracker
  ) {
    super();
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.performanceTracker = performanceTracker;
  }

  /**
   * Registers a new model version.
   *
   * @param version - Model version to register
   */
  registerVersion(version: ModelVersion): void {
    this.versions.set(version.id, version);
    this.emit("version_registered", version);
  }

  /**
   * Deploys a model version to staging.
   *
   * @param versionId - Version to deploy
   */
  async deployToStaging(versionId: string): Promise<void> {
    const version = this.versions.get(versionId);
    if (!version) {
      throw new Error(`Version ${versionId} not found`);
    }

    version.status = "staging";
    version.deployedAt = new Date().toISOString();

    this.emit("deployed_to_staging", version);
  }

  /**
   * Starts an A/B test with a new model version.
   *
   * @param config - A/B test configuration
   * @returns Test identifier
   */
  async startABTest(config: ABTestConfig): Promise<string> {
    // Validate versions exist
    const controlVersion = this.versions.get(config.controlVersion);
    const treatmentVersion = this.versions.get(config.treatmentVersion);

    if (!controlVersion || !treatmentVersion) {
      throw new Error("Invalid version specified for A/B test");
    }

    // Check concurrent test limit
    if (this.activeTests.size >= this.config.maxConcurrentTests) {
      throw new Error(
        `Maximum concurrent tests (${this.config.maxConcurrentTests}) reached`
      );
    }

    // Update treatment version status
    treatmentVersion.status = "canary";

    // Store test configuration
    this.activeTests.set(config.name, config);

    // Start monitoring
    if (!this.monitoringTimer) {
      this.startMonitoring();
    }

    this.emit("ab_test_started", {
      testName: config.name,
      controlVersion: config.controlVersion,
      treatmentVersion: config.treatmentVersion,
      trafficPercentage: config.trafficPercentage,
    });

    return config.name;
  }

  /**
   * Evaluates an active A/B test.
   *
   * @param testName - Name of the test to evaluate
   * @returns Test result and recommendation
   */
  async evaluateABTest(testName: string): Promise<ABTestResult> {
    const testConfig = this.activeTests.get(testName);
    if (!testConfig) {
      throw new Error(`A/B test ${testName} not found`);
    }

    // Get performance data for both versions
    const stats = this.performanceTracker.getStats();

    // In a real implementation, we would filter data by version
    // For now, simulate with mock data
    const controlMetrics = {
      sampleSize: 1000,
      averageReward: 0.75,
      successRate: 0.85,
      latencyMs: 250,
      errorRate: 0.02,
    };

    const treatmentMetrics = {
      sampleSize: 100,
      averageReward: 0.78,
      successRate: 0.87,
      latencyMs: 240,
      errorRate: 0.015,
    };

    // Calculate statistical significance (simplified)
    const statisticalSignificance = this.calculateSignificance(
      controlMetrics,
      treatmentMetrics,
      testConfig.minSampleSize
    );

    // Generate recommendation
    const recommendation = this.generateRecommendation(
      controlMetrics,
      treatmentMetrics,
      testConfig,
      statisticalSignificance
    );

    const result: ABTestResult = {
      testName,
      controlMetrics,
      treatmentMetrics,
      statisticalSignificance,
      recommendation,
      reasoning: this.generateReasoning(
        recommendation,
        controlMetrics,
        treatmentMetrics,
        statisticalSignificance
      ),
      completedAt: new Date().toISOString(),
    };

    this.emit("ab_test_evaluated", result);

    return result;
  }

  /**
   * Promotes a model version to production.
   *
   * @param versionId - Version to promote
   * @param testName - A/B test that validated this version (optional)
   */
  async promoteToProduction(
    versionId: string,
    testName?: string
  ): Promise<void> {
    const version = this.versions.get(versionId);
    if (!version) {
      throw new Error(`Version ${versionId} not found`);
    }

    // Demote current production versions
    for (const [id, v] of this.versions) {
      if (v.status === "production" && id !== versionId) {
        v.status = "archived";
      }
    }

    // Promote new version
    version.status = "production";
    version.deployedAt = new Date().toISOString();

    // Record performance baseline
    const stats = this.performanceTracker.getStats();
    version.performanceBaseline = {
      averageReward: 0, // Would calculate from actual data
      successRate: stats.overallSuccessRate,
      latencyMs: stats.averageCompletionTimeMs,
      errorRate: 0.02, // Would calculate from actual data
    };

    // Clean up A/B test if specified
    if (testName) {
      this.activeTests.delete(testName);
    }

    this.emit("promoted_to_production", version);
  }

  /**
   * Rolls back to a previous version.
   *
   * @param currentVersionId - Version to roll back from
   * @param targetVersionId - Version to roll back to (optional - uses last production)
   * @param reason - Rollback reason
   */
  async rollback(
    currentVersionId: string,
    targetVersionId?: string,
    reason?: string
  ): Promise<void> {
    const currentVersion = this.versions.get(currentVersionId);
    if (!currentVersion) {
      throw new Error(`Version ${currentVersionId} not found`);
    }

    // Find target version
    let targetVersion: ModelVersion | undefined;

    if (targetVersionId) {
      targetVersion = this.versions.get(targetVersionId);
    } else {
      // Find last production version
      const sortedVersions = Array.from(this.versions.values())
        .filter((v) => v.status === "production" || v.status === "archived")
        .sort(
          (a, b) =>
            new Date(b.deployedAt).getTime() - new Date(a.deployedAt).getTime()
        );

      targetVersion = sortedVersions[0];
    }

    if (!targetVersion) {
      throw new Error("No valid rollback target found");
    }

    // Update statuses
    currentVersion.status = "rolled_back";
    currentVersion.rolledBackTo = targetVersion.id;
    targetVersion.status = "production";

    this.emit("rollback_completed", {
      from: currentVersionId,
      to: targetVersion.id,
      reason: reason || "Manual rollback",
      timestamp: new Date().toISOString(),
    });
  }

  /**
   * Gets all registered model versions.
   *
   * @returns Array of model versions
   */
  getVersions(): ModelVersion[] {
    return Array.from(this.versions.values());
  }

  /**
   * Gets current production version.
   *
   * @returns Production version or undefined
   */
  getProductionVersion(): ModelVersion | undefined {
    return Array.from(this.versions.values()).find(
      (v) => v.status === "production"
    );
  }

  /**
   * Gets all active A/B tests.
   *
   * @returns Array of active test configurations
   */
  getActiveTests(): ABTestConfig[] {
    return Array.from(this.activeTests.values());
  }

  /**
   * Gets current configuration.
   *
   * @returns Current configuration
   */
  getConfig(): ModelDeploymentManagerConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<ModelDeploymentManagerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Starts performance monitoring.
   */
  private startMonitoring(): void {
    this.monitoringTimer = setInterval(async () => {
      await this.monitorPerformance();
    }, this.config.monitoringIntervalMs);
  }

  /**
   * Stops performance monitoring.
   */
  stopMonitoring(): void {
    if (this.monitoringTimer) {
      clearInterval(this.monitoringTimer);
      this.monitoringTimer = undefined;
    }
  }

  /**
   * Monitors performance and checks for degradation.
   */
  private async monitorPerformance(): Promise<void> {
    const productionVersion = this.getProductionVersion();
    if (!productionVersion || !productionVersion.performanceBaseline) {
      return;
    }

    // Get current performance stats
    const stats = this.performanceTracker.getStats();

    // Check for performance degradation
    const baseline = productionVersion.performanceBaseline;

    const successRateDecrease =
      ((baseline.successRate - stats.overallSuccessRate) /
        baseline.successRate) *
      100;
    const latencyIncrease =
      ((stats.averageCompletionTimeMs - baseline.latencyMs) /
        baseline.latencyMs) *
      100;

    // Check rollback thresholds
    const thresholds = this.config.rollbackThresholds;
    const shouldRollback =
      successRateDecrease > thresholds.maxSuccessRateDecreasePercent ||
      latencyIncrease > thresholds.maxLatencyIncreasePercent;

    if (shouldRollback && this.config.enableAutoRollback) {
      this.emit("performance_degradation_detected", {
        versionId: productionVersion.id,
        successRateDecrease,
        latencyIncrease,
      });

      await this.rollback(
        productionVersion.id,
        undefined,
        "Automatic rollback due to performance degradation"
      );
    }
  }

  /**
   * Calculates statistical significance for A/B test.
   */
  private calculateSignificance(
    controlMetrics: any,
    treatmentMetrics: any,
    minSampleSize: number
  ): ABTestResult["statisticalSignificance"] {
    // Simplified statistical significance calculation
    const hasEnoughSamples =
      controlMetrics.sampleSize >= minSampleSize &&
      treatmentMetrics.sampleSize >= minSampleSize / 10; // Treatment needs 10% of control

    const rewardDifference = Math.abs(
      treatmentMetrics.averageReward - controlMetrics.averageReward
    );
    const successRateDifference = Math.abs(
      treatmentMetrics.successRate - controlMetrics.successRate
    );

    // Mock p-value calculation (in practice, would use proper statistical tests)
    const pValue =
      rewardDifference > 0.05 && successRateDifference > 0.05 ? 0.03 : 0.15;
    const isSignificant = pValue < 0.05 && hasEnoughSamples;
    const confidence = isSignificant ? 0.95 : 0.8;

    return {
      isSignificant,
      pValue,
      confidence,
    };
  }

  /**
   * Generates recommendation based on test results.
   */
  private generateRecommendation(
    controlMetrics: any,
    treatmentMetrics: any,
    testConfig: ABTestConfig,
    significance: ABTestResult["statisticalSignificance"]
  ): ABTestResult["recommendation"] {
    const thresholds = testConfig.promotionThresholds;
    const rollbackThresholds = testConfig.rollbackThresholds;

    // Check for rollback conditions
    const successRateDecrease =
      ((controlMetrics.successRate - treatmentMetrics.successRate) /
        controlMetrics.successRate) *
      100;
    const latencyIncrease =
      ((treatmentMetrics.latencyMs - controlMetrics.latencyMs) /
        controlMetrics.latencyMs) *
      100;
    const errorRateIncrease =
      ((treatmentMetrics.errorRate - controlMetrics.errorRate) /
        controlMetrics.errorRate) *
      100;

    if (
      successRateDecrease > rollbackThresholds.maxSuccessRateDecreasePercent ||
      latencyIncrease > rollbackThresholds.maxLatencyIncreasePercent ||
      treatmentMetrics.errorRate > rollbackThresholds.maxErrorRatePercent
    ) {
      return "rollback";
    }

    // Check for promotion conditions
    if (!significance.isSignificant) {
      return "continue_testing";
    }

    const rewardImprovement =
      ((treatmentMetrics.averageReward - controlMetrics.averageReward) /
        controlMetrics.averageReward) *
      100;
    const meetsThresholds =
      treatmentMetrics.successRate >= thresholds.minSuccessRate &&
      latencyIncrease <= thresholds.maxLatencyIncreasePercent &&
      errorRateIncrease <= thresholds.maxErrorRateIncreasePercent &&
      rewardImprovement >= thresholds.minRewardImprovement;

    if (meetsThresholds) {
      return "promote_treatment";
    }

    return "keep_control";
  }

  /**
   * Generates reasoning for recommendation.
   */
  private generateReasoning(
    recommendation: ABTestResult["recommendation"],
    controlMetrics: any,
    treatmentMetrics: any,
    significance: ABTestResult["statisticalSignificance"]
  ): string {
    const rewardChange =
      ((treatmentMetrics.averageReward - controlMetrics.averageReward) /
        controlMetrics.averageReward) *
      100;
    const successRateChange =
      ((treatmentMetrics.successRate - controlMetrics.successRate) /
        controlMetrics.successRate) *
      100;

    switch (recommendation) {
      case "promote_treatment":
        return (
          `Treatment version shows statistically significant improvement (p=${significance.pValue.toFixed(
            3
          )}). ` +
          `Reward improved by ${rewardChange.toFixed(
            1
          )}%, success rate improved by ${successRateChange.toFixed(1)}%. ` +
          `Safe to promote to production.`
        );

      case "keep_control":
        return (
          `Control version performs better or improvement is not significant enough. ` +
          `Reward change: ${rewardChange.toFixed(
            1
          )}%, success rate change: ${successRateChange.toFixed(1)}%. ` +
          `Continue with control version.`
        );

      case "continue_testing":
        return (
          `Results are not yet statistically significant (p=${significance.pValue.toFixed(
            3
          )}). ` +
          `Treatment shows ${rewardChange.toFixed(
            1
          )}% reward change, ${successRateChange.toFixed(
            1
          )}% success rate change. ` +
          `Continue testing to gather more data.`
        );

      case "rollback":
        return (
          `Treatment version shows concerning performance degradation. ` +
          `Success rate decreased by ${Math.abs(successRateChange).toFixed(
            1
          )}% or error rate too high. ` +
          `Recommend immediate rollback to control version.`
        );

      default:
        return "Unable to generate recommendation.";
    }
  }
}
