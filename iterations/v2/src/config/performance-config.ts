/**
 * Shared Performance Configuration for ARBITER-004
 *
 * Centralized configuration for all performance tracking components
 * across ARBITER-001, 002, 003, and 004 integration points.
 *
 * @author @darianrosebrook
 */

// Environment-aware configuration loader
function loadConfigFromEnvironment(): Record<string, any> {
  return {
    // Collection settings
    collection: {
      enabled: process.env.PERFORMANCE_TRACKING_ENABLED === "true",
      samplingRate: parseFloat(process.env.PERFORMANCE_SAMPLING_RATE || "1.0"),
      maxCollectionLatencyMs: parseInt(
        process.env.PERFORMANCE_MAX_LATENCY_MS || "10"
      ),
      batchSize: parseInt(process.env.PERFORMANCE_BATCH_SIZE || "100"),
      flushIntervalMs: parseInt(
        process.env.PERFORMANCE_FLUSH_INTERVAL_MS || "5000"
      ),
    },

    // Aggregation settings
    aggregation: {
      enabled: process.env.PERFORMANCE_AGGREGATION_ENABLED === "true",
      windowSizes: {
        realtime: parseInt(
          process.env.PERFORMANCE_WINDOW_REALTIME_MS || "300000"
        ), // 5 minutes
        short: parseInt(process.env.PERFORMANCE_WINDOW_SHORT_MS || "3600000"), // 1 hour
        medium: parseInt(
          process.env.PERFORMANCE_WINDOW_MEDIUM_MS || "86400000"
        ), // 24 hours
        long: parseInt(process.env.PERFORMANCE_WINDOW_LONG_MS || "604800000"), // 7 days
      },
      outlierThresholds: {
        zScoreThreshold: parseFloat(
          process.env.PERFORMANCE_ZSCORE_THRESHOLD || "2.5"
        ),
        iqrMultiplier: parseFloat(
          process.env.PERFORMANCE_IQR_MULTIPLIER || "1.8"
        ),
      },
      anonymization: {
        enabled: process.env.PERFORMANCE_ANONYMIZATION_ENABLED !== "false",
        noiseLevel: parseFloat(process.env.PERFORMANCE_NOISE_LEVEL || "0.05"),
        preserveAgentIds:
          process.env.PERFORMANCE_PRESERVE_AGENT_IDS !== "false",
        preserveTaskTypes:
          process.env.PERFORMANCE_PRESERVE_TASK_TYPES !== "false",
      },
      trendAnalysis: {
        minDataPoints: parseInt(
          process.env.PERFORMANCE_MIN_DATA_POINTS || "10"
        ),
        confidenceThreshold: parseFloat(
          process.env.PERFORMANCE_CONFIDENCE_THRESHOLD || "0.8"
        ),
      },
    },

    // RL Training settings
    rl: {
      enabled: process.env.RL_TRAINING_ENABLED === "true",
      batching: {
        maxBatchSize: parseInt(process.env.RL_MAX_BATCH_SIZE || "32"),
        maxBatchAgeMinutes: parseInt(
          process.env.RL_MAX_BATCH_AGE_MINUTES || "5"
        ),
        minBatchSize: parseInt(process.env.RL_MIN_BATCH_SIZE || "8"),
      },
      qualityThresholds: {
        minSampleDiversity: parseFloat(
          process.env.RL_MIN_SAMPLE_DIVERSITY || "0.7"
        ),
        maxTemporalGapMinutes: parseFloat(
          process.env.RL_MAX_TEMPORAL_GAP_MINUTES || "30"
        ),
        minRewardVariance: parseFloat(
          process.env.RL_MIN_REWARD_VARIANCE || "0.1"
        ),
        maxDuplicateRatio: parseFloat(
          process.env.RL_MAX_DUPLICATE_RATIO || "0.15"
        ),
      },
      stateRepresentation: {
        includeHistoricalMetrics: process.env.RL_INCLUDE_HISTORICAL !== "false",
        includeAgentLoad: process.env.RL_INCLUDE_AGENT_LOAD === "true",
        includeTaskContext: process.env.RL_INCLUDE_TASK_CONTEXT !== "false",
        temporalWindowSize: parseInt(
          process.env.RL_TEMPORAL_WINDOW_SIZE || "5"
        ),
      },
      rewardFunction: {
        latencyWeight: parseFloat(process.env.RL_LATENCY_WEIGHT || "0.3"),
        accuracyWeight: parseFloat(process.env.RL_ACCURACY_WEIGHT || "0.4"),
        costWeight: parseFloat(process.env.RL_COST_WEIGHT || "0.1"),
        complianceWeight: parseFloat(process.env.RL_COMPLIANCE_WEIGHT || "0.2"),
        temporalDecayFactor: parseFloat(
          process.env.RL_TEMPORAL_DECAY || "0.95"
        ),
      },
      retention: {
        maxSamplesInMemory: parseInt(
          process.env.RL_MAX_SAMPLES_MEMORY || "10000"
        ),
        maxBatchesInMemory: parseInt(
          process.env.RL_MAX_BATCHES_MEMORY || "100"
        ),
        cleanupIntervalMinutes: parseInt(
          process.env.RL_CLEANUP_INTERVAL_MINUTES || "15"
        ),
      },
    },

    // Analysis & Alerting settings
    analysis: {
      enabled: process.env.PERFORMANCE_ANALYSIS_ENABLED === "true",
      anomalyThresholds: {
        latencySpikeMultiplier: parseFloat(
          process.env.ANOMALY_LATENCY_SPIKE_MULTIPLIER || "3.0"
        ),
        accuracyDropPercent: parseFloat(
          process.env.ANOMALY_ACCURACY_DROP_PERCENT || "15"
        ),
        errorRateIncreasePercent: parseFloat(
          process.env.ANOMALY_ERROR_RATE_INCREASE || "25"
        ),
        resourceSaturationPercent: parseFloat(
          process.env.ANOMALY_RESOURCE_SATURATION || "95"
        ),
      },
      alertThresholds: {
        criticalLatencyMs: parseInt(
          process.env.ALERT_CRITICAL_LATENCY_MS || "5000"
        ),
        criticalErrorRatePercent: parseFloat(
          process.env.ALERT_CRITICAL_ERROR_RATE || "10"
        ),
        criticalAccuracyDropPercent: parseFloat(
          process.env.ALERT_CRITICAL_ACCURACY_DROP || "25"
        ),
      },
      alertChannels: {
        slack: process.env.ALERT_SLACK_WEBHOOK,
        email: process.env.ALERT_EMAIL_RECIPIENTS,
        webhook: process.env.ALERT_WEBHOOK_URL,
      },
      monitoring: {
        healthCheckIntervalMs: parseInt(
          process.env.MONITORING_HEALTH_CHECK_INTERVAL || "30000"
        ),
        performanceSnapshotIntervalMs: parseInt(
          process.env.MONITORING_SNAPSHOT_INTERVAL || "60000"
        ),
        alertCooldownMs: parseInt(
          process.env.MONITORING_ALERT_COOLDOWN || "300000"
        ), // 5 minutes
      },
    },

    // Storage settings
    storage: {
      retentionDays: parseInt(process.env.PERFORMANCE_RETENTION_DAYS || "90"),
      compressionEnabled:
        process.env.PERFORMANCE_COMPRESSION_ENABLED !== "false",
      maxStorageSizeGB: parseInt(
        process.env.PERFORMANCE_MAX_STORAGE_GB || "100"
      ),
      backupEnabled: process.env.PERFORMANCE_BACKUP_ENABLED === "true",
      backupIntervalHours: parseInt(
        process.env.PERFORMANCE_BACKUP_INTERVAL_HOURS || "24"
      ),
    },

    // Integration settings for other ARBITER components
    integration: {
      arbiter001: {
        enabled: process.env.INTEGRATION_ARBITER_001_ENABLED !== "false",
        emitRegistrationEvents:
          process.env.INTEGRATION_EMIT_REGISTRATION_EVENTS !== "false",
        trackAgentLifecycle:
          process.env.INTEGRATION_TRACK_AGENT_LIFECYCLE !== "false",
      },
      arbiter002: {
        enabled: process.env.INTEGRATION_ARBITER_002_ENABLED !== "false",
        performanceWeightedRouting:
          process.env.INTEGRATION_PERFORMANCE_WEIGHTED_ROUTING !== "false",
        routingFeedbackLoop:
          process.env.INTEGRATION_ROUTING_FEEDBACK_LOOP !== "false",
      },
      arbiter003: {
        enabled: process.env.INTEGRATION_ARBITER_003_ENABLED !== "false",
        constitutionalMetrics:
          process.env.INTEGRATION_CONSTITUTIONAL_METRICS !== "false",
        complianceScoring:
          process.env.INTEGRATION_COMPLIANCE_SCORING !== "false",
      },
    },

    // Feature flags for gradual rollout
    features: {
      dataCollection: process.env.FEATURE_DATA_COLLECTION !== "false",
      realTimeAggregation: process.env.FEATURE_REALTIME_AGGREGATION !== "false",
      rlTrainingPipeline: process.env.FEATURE_RL_TRAINING_PIPELINE !== "false",
      performanceAnalysis: process.env.FEATURE_PERFORMANCE_ANALYSIS !== "false",
      alerting: process.env.FEATURE_ALERTING !== "false",
      anonymization: process.env.FEATURE_ANONYMIZATION !== "false",
      apiEndpoints: process.env.FEATURE_API_ENDPOINTS !== "false",
    },
  };
}

// Configuration validation
function validateConfiguration(config: any): void {
  const errors: string[] = [];

  // Validate collection settings
  if (
    config.collection.samplingRate < 0 ||
    config.collection.samplingRate > 1
  ) {
    errors.push("Collection sampling rate must be between 0 and 1");
  }

  // Validate aggregation windows
  const windows = config.aggregation.windowSizes;
  if (
    windows.realtime >= windows.short ||
    windows.short >= windows.medium ||
    windows.medium >= windows.long
  ) {
    errors.push(
      "Aggregation windows must be in ascending order: realtime < short < medium < long"
    );
  }

  // Validate RL weights
  const rlWeights = config.rl.rewardFunction;
  const totalWeight =
    rlWeights.latencyWeight +
    rlWeights.accuracyWeight +
    rlWeights.costWeight +
    rlWeights.complianceWeight;
  if (Math.abs(totalWeight - 1.0) > 0.01) {
    errors.push("RL reward function weights must sum to 1.0");
  }

  if (errors.length > 0) {
    throw new Error(`Invalid performance configuration:\n${errors.join("\n")}`);
  }
}

// Load and validate configuration
const rawConfig = loadConfigFromEnvironment();
validateConfiguration(rawConfig);

// Export typed configuration
export const PERFORMANCE_CONFIG = rawConfig as {
  collection: {
    enabled: boolean;
    samplingRate: number;
    maxCollectionLatencyMs: number;
    batchSize: number;
    flushIntervalMs: number;
  };
  aggregation: {
    enabled: boolean;
    windowSizes: {
      realtime: number;
      short: number;
      medium: number;
      long: number;
    };
    outlierThresholds: {
      zScoreThreshold: number;
      iqrMultiplier: number;
    };
    anonymization: {
      enabled: boolean;
      noiseLevel: number;
      preserveAgentIds: boolean;
      preserveTaskTypes: boolean;
    };
    trendAnalysis: {
      minDataPoints: number;
      confidenceThreshold: number;
    };
  };
  rl: {
    enabled: boolean;
    batching: {
      maxBatchSize: number;
      maxBatchAgeMinutes: number;
      minBatchSize: number;
    };
    qualityThresholds: {
      minSampleDiversity: number;
      maxTemporalGapMinutes: number;
      minRewardVariance: number;
      maxDuplicateRatio: number;
    };
    stateRepresentation: {
      includeHistoricalMetrics: boolean;
      includeAgentLoad: boolean;
      includeTaskContext: boolean;
      temporalWindowSize: number;
    };
    rewardFunction: {
      latencyWeight: number;
      accuracyWeight: number;
      costWeight: number;
      complianceWeight: number;
      temporalDecayFactor: number;
    };
    retention: {
      maxSamplesInMemory: number;
      maxBatchesInMemory: number;
      cleanupIntervalMinutes: number;
    };
  };
  analysis: {
    enabled: boolean;
    anomalyThresholds: {
      latencySpikeMultiplier: number;
      accuracyDropPercent: number;
      errorRateIncreasePercent: number;
      resourceSaturationPercent: number;
    };
    alertThresholds: {
      criticalLatencyMs: number;
      criticalErrorRatePercent: number;
      criticalAccuracyDropPercent: number;
    };
    alertChannels: {
      slack?: string;
      email?: string;
      webhook?: string;
    };
    monitoring: {
      healthCheckIntervalMs: number;
      performanceSnapshotIntervalMs: number;
      alertCooldownMs: number;
    };
  };
  storage: {
    retentionDays: number;
    compressionEnabled: boolean;
    maxStorageSizeGB: number;
    backupEnabled: boolean;
    backupIntervalHours: number;
  };
  integration: {
    arbiter001: {
      enabled: boolean;
      emitRegistrationEvents: boolean;
      trackAgentLifecycle: boolean;
    };
    arbiter002: {
      enabled: boolean;
      performanceWeightedRouting: boolean;
      routingFeedbackLoop: boolean;
    };
    arbiter003: {
      enabled: boolean;
      constitutionalMetrics: boolean;
      complianceScoring: boolean;
    };
  };
  features: {
    dataCollection: boolean;
    realTimeAggregation: boolean;
    rlTrainingPipeline: boolean;
    performanceAnalysis: boolean;
    alerting: boolean;
    anonymization: boolean;
    apiEndpoints: boolean;
  };
};

// Configuration utilities
export class PerformanceConfigManager {
  /**
   * Get current configuration snapshot
   */
  static getConfig(): typeof PERFORMANCE_CONFIG {
    return { ...PERFORMANCE_CONFIG };
  }

  /**
   * Check if a specific feature is enabled
   */
  static isFeatureEnabled(
    feature: keyof typeof PERFORMANCE_CONFIG.features
  ): boolean {
    return PERFORMANCE_CONFIG.features[feature];
  }

  /**
   * Check if integration with specific ARBITER component is enabled
   */
  static isIntegrationEnabled(
    arbiter: keyof typeof PERFORMANCE_CONFIG.integration
  ): boolean {
    return PERFORMANCE_CONFIG.integration[arbiter].enabled;
  }

  /**
   * Get environment-specific overrides
   */
  static getEnvironmentOverrides(): Record<string, any> {
    const overrides: Record<string, any> = {};

    // Development overrides
    if (process.env.NODE_ENV === "development") {
      overrides.collection = {
        ...PERFORMANCE_CONFIG.collection,
        samplingRate: 0.1, // Reduce sampling in dev
      };
      overrides.analysis = {
        ...PERFORMANCE_CONFIG.analysis,
        enabled: false, // Disable alerting in dev
      };
    }

    // Production overrides
    if (process.env.NODE_ENV === "production") {
      overrides.collection = {
        ...PERFORMANCE_CONFIG.collection,
        samplingRate: 1.0, // Full sampling in prod
      };
      overrides.analysis = {
        ...PERFORMANCE_CONFIG.analysis,
        enabled: true, // Enable all monitoring in prod
      };
    }

    return overrides;
  }

  // Instance methods
  private config: any = {};

  /**
   * Load configuration
   */
  async loadConfiguration(config: any): Promise<void> {
    this.config = { ...config };
  }

  /**
   * Record a performance metric
   */
  async recordMetric(name: string, value: number): Promise<void> {
    // Mock implementation for tests
    console.log(`Recording metric ${name}: ${value}`);
  }

  /**
   * Validate configuration against runtime constraints
   */
  static validateRuntimeConstraints(): { valid: boolean; warnings: string[] } {
    const warnings: string[] = [];

    // Check memory constraints
    const memUsage = process.memoryUsage();
    const heapMB = memUsage.heapUsed / 1024 / 1024;

    if (heapMB > 500) {
      warnings.push(`High memory usage detected: ${heapMB.toFixed(1)}MB heap`);
    }

    // Check if required environment variables are set
    if (
      PERFORMANCE_CONFIG.analysis.enabled &&
      !PERFORMANCE_CONFIG.analysis.alertChannels.slack &&
      !PERFORMANCE_CONFIG.analysis.alertChannels.email
    ) {
      warnings.push("Alerting enabled but no alert channels configured");
    }

    return {
      valid: warnings.length === 0,
      warnings,
    };
  }

  /**
   * Export configuration for external monitoring
   */
  static exportForMonitoring(): Record<string, any> {
    return {
      version: "2.0.0",
      timestamp: new Date().toISOString(),
      environment: process.env.NODE_ENV || "unknown",
      features: PERFORMANCE_CONFIG.features,
      integrations: PERFORMANCE_CONFIG.integration,
      performance: {
        collectionEnabled: PERFORMANCE_CONFIG.collection.enabled,
        aggregationEnabled: PERFORMANCE_CONFIG.aggregation.enabled,
        rlEnabled: PERFORMANCE_CONFIG.rl.enabled,
        analysisEnabled: PERFORMANCE_CONFIG.analysis.enabled,
      },
    };
  }
}

// Export for convenience
export default PERFORMANCE_CONFIG;
