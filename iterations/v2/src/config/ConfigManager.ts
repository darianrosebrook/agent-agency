/**
 * Configuration Manager
 *
 * Provides centralized configuration management for all ARBITER components.
 * Environment-aware with validation and type safety.
 *
 * @author @darianrosebrook
 */

import { z } from "zod";

// Base configuration schemas
const orchestratorConfigSchema = z.object({
  maxConcurrentTasks: z.number().min(1).default(10),
  processingLoopIntervalMs: z.number().min(10).default(100),
  circuitBreaker: z.object({
    failureThreshold: z.number().min(1).default(5),
    resetTimeoutMs: z.number().min(1000).default(30000),
  }),
});

const feedbackLoopConfigSchema = z.object({
  enabled: z.boolean().default(true),
  collection: z.object({
    enabledSources: z
      .array(z.string())
      .default(["performance_metrics", "task_outcomes"]),
    batchSize: z.number().min(1).default(10),
    flushIntervalMs: z.number().min(10).default(1000),
    retentionPeriodDays: z.number().min(1).default(30),
    samplingRate: z.number().min(0).max(1).default(1.0),
    filters: z
      .object({
        minSeverity: z.string().optional(),
        excludeEntityTypes: z.array(z.string()).optional(),
        includeOnlyRecent: z.boolean().default(false),
      })
      .default({}),
  }),
  analysis: z.object({
    enabledAnalyzers: z.array(z.string()).default(["trend", "anomaly"]),
    analysisIntervalMs: z.number().min(100).default(5000),
    anomalyThreshold: z.number().min(0).default(2.0),
    trendWindowHours: z.number().min(1).default(24),
    minDataPoints: z.number().min(1).default(5),
    correlationThreshold: z.number().min(0).max(1).default(0.5),
    predictionHorizonHours: z.number().min(1).default(24),
  }),
  improvements: z.object({
    autoApplyThreshold: z.number().min(0).max(1).default(0.8),
    maxConcurrentImprovements: z.number().min(1).default(5),
    cooldownPeriodMs: z.number().min(0).default(300000),
    improvementTimeoutMs: z.number().min(1000).default(300000),
    rollbackOnFailure: z.boolean().default(true),
    monitoringPeriodMs: z.number().min(1000).default(600000),
  }),
  pipeline: z.object({
    batchSize: z.number().min(1).default(50),
    processingIntervalMs: z.number().min(10).default(10000),
    dataQualityThreshold: z.number().min(0).max(1).default(0.7),
    anonymizationLevel: z.enum(["none", "partial", "full"]).default("partial"),
    featureEngineering: z
      .object({
        timeWindowFeatures: z.boolean().default(true),
        correlationFeatures: z.boolean().default(true),
        trendFeatures: z.boolean().default(true),
      })
      .default({}),
    trainingDataFormat: z.enum(["json", "parquet", "csv"]).default("json"),
  }),
});

const violationHandlerConfigSchema = z.object({
  escalationThreshold: z.string().default("high"),
});

const cawsRuntimeConfigSchema = z.object({
  violationHandlingEnabled: z.boolean().default(true),
  waiverManagementEnabled: z.boolean().default(true),
});

const fullConfigSchema = z.object({
  orchestrator: orchestratorConfigSchema,
  feedbackLoop: feedbackLoopConfigSchema,
  violationHandler: violationHandlerConfigSchema,
  cawsRuntime: cawsRuntimeConfigSchema,
});

/**
 * Configuration manager singleton
 */
export class ConfigManager {
  private static instance: ConfigManager;
  private config: any;

  private constructor() {
    this.loadConfig();
  }

  public static getInstance(): ConfigManager {
    if (!ConfigManager.instance) {
      ConfigManager.instance = new ConfigManager();
    }
    return ConfigManager.instance;
  }

  /**
   * Get configuration value by path
   */
  public get<T = any>(path: string): T {
    const keys = path.split(".");
    let current = this.config;

    for (const key of keys) {
      if (current && typeof current === "object" && key in current) {
        current = current[key];
      } else {
        throw new Error(`Configuration path not found: ${path}`);
      }
    }

    return current as T;
  }

  /**
   * Set configuration value by path
   */
  public set(path: string, value: any): void {
    const keys = path.split(".");
    let current = this.config;

    for (let i = 0; i < keys.length - 1; i++) {
      const key = keys[i];
      if (!(key in current) || typeof current[key] !== "object") {
        current[key] = {};
      }
      current = current[key];
    }

    current[keys[keys.length - 1]] = value;
  }

  /**
   * Get all configuration
   */
  public getAll(): any {
    return { ...this.config };
  }

  /**
   * Reload configuration from environment
   */
  public reload(): void {
    this.loadConfig();
  }

  /**
   * Validate configuration
   */
  public validate(): { valid: boolean; errors: string[] } {
    try {
      fullConfigSchema.parse(this.config);
      return { valid: true, errors: [] };
    } catch (error) {
      if (error instanceof z.ZodError) {
        const errors = error.errors.map(
          (err) => `${err.path.join(".")}: ${err.message}`
        );
        return { valid: false, errors };
      }
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      return { valid: false, errors: [errorMessage] };
    }
  }

  private loadConfig(): void {
    // Load from environment variables with defaults
    this.config = {
      orchestrator: {
        maxConcurrentTasks: parseInt(
          process.env.ORCHESTRATOR_MAX_CONCURRENT_TASKS || "10"
        ),
        processingLoopIntervalMs: parseInt(
          process.env.ORCHESTRATOR_PROCESSING_LOOP_INTERVAL_MS || "100"
        ),
        circuitBreaker: {
          failureThreshold: parseInt(
            process.env.ORCHESTRATOR_CIRCUIT_BREAKER_FAILURE_THRESHOLD || "5"
          ),
          resetTimeoutMs: parseInt(
            process.env.ORCHESTRATOR_CIRCUIT_BREAKER_RESET_TIMEOUT_MS || "30000"
          ),
        },
      },
      feedbackLoop: {
        enabled: process.env.FEEDBACK_LOOP_ENABLED !== "false",
        collection: {
          enabledSources: (
            process.env.FEEDBACK_LOOP_COLLECTION_ENABLED_SOURCES ||
            "performance_metrics,task_outcomes,user_ratings,system_events"
          ).split(","),
          batchSize: parseInt(
            process.env.FEEDBACK_LOOP_COLLECTION_BATCH_SIZE || "10"
          ),
          flushIntervalMs: parseInt(
            process.env.FEEDBACK_LOOP_COLLECTION_FLUSH_INTERVAL_MS || "1000"
          ),
          retentionPeriodDays: parseInt(
            process.env.FEEDBACK_LOOP_COLLECTION_RETENTION_PERIOD_DAYS || "30"
          ),
          samplingRate: parseFloat(
            process.env.FEEDBACK_LOOP_COLLECTION_SAMPLING_RATE || "1.0"
          ),
          filters: {
            minSeverity:
              process.env.FEEDBACK_LOOP_COLLECTION_FILTERS_MIN_SEVERITY,
            excludeEntityTypes:
              process.env.FEEDBACK_LOOP_COLLECTION_FILTERS_EXCLUDE_ENTITY_TYPES?.split(
                ","
              ),
            includeOnlyRecent:
              process.env
                .FEEDBACK_LOOP_COLLECTION_FILTERS_INCLUDE_ONLY_RECENT ===
              "true",
          },
        },
        analysis: {
          enabledAnalyzers: (
            process.env.FEEDBACK_LOOP_ANALYSIS_ENABLED_ANALYZERS ||
            "trend,anomaly"
          ).split(","),
          analysisIntervalMs: parseInt(
            process.env.FEEDBACK_LOOP_ANALYSIS_INTERVAL_MS || "5000"
          ),
          anomalyThreshold: parseFloat(
            process.env.FEEDBACK_LOOP_ANALYSIS_ANOMALY_THRESHOLD || "2.0"
          ),
          trendWindowHours: parseInt(
            process.env.FEEDBACK_LOOP_ANALYSIS_TREND_WINDOW_HOURS || "24"
          ),
          minDataPoints: parseInt(
            process.env.FEEDBACK_LOOP_ANALYSIS_MIN_DATA_POINTS || "5"
          ),
          correlationThreshold: parseFloat(
            process.env.FEEDBACK_LOOP_ANALYSIS_CORRELATION_THRESHOLD || "0.5"
          ),
          predictionHorizonHours: parseInt(
            process.env.FEEDBACK_LOOP_ANALYSIS_PREDICTION_HORIZON_HOURS || "24"
          ),
        },
        improvements: {
          autoApplyThreshold: parseFloat(
            process.env.FEEDBACK_LOOP_IMPROVEMENTS_AUTO_APPLY_THRESHOLD || "0.8"
          ),
          maxConcurrentImprovements: parseInt(
            process.env.FEEDBACK_LOOP_IMPROVEMENTS_MAX_CONCURRENT || "5"
          ),
          cooldownPeriodMs: parseInt(
            process.env.FEEDBACK_LOOP_IMPROVEMENTS_COOLDOWN_PERIOD_MS ||
              "300000"
          ),
          improvementTimeoutMs: parseInt(
            process.env.FEEDBACK_LOOP_IMPROVEMENTS_TIMEOUT_MS || "300000"
          ),
          rollbackOnFailure:
            process.env.FEEDBACK_LOOP_IMPROVEMENTS_ROLLBACK_ON_FAILURE !==
            "false",
          monitoringPeriodMs: parseInt(
            process.env.FEEDBACK_LOOP_IMPROVEMENTS_MONITORING_PERIOD_MS ||
              "600000"
          ),
        },
        pipeline: {
          batchSize: parseInt(
            process.env.FEEDBACK_LOOP_PIPELINE_BATCH_SIZE || "50"
          ),
          processingIntervalMs: parseInt(
            process.env.FEEDBACK_LOOP_PIPELINE_PROCESSING_INTERVAL_MS || "10000"
          ),
          dataQualityThreshold: parseFloat(
            process.env.FEEDBACK_LOOP_PIPELINE_DATA_QUALITY_THRESHOLD || "0.7"
          ),
          anonymizationLevel: (process.env
            .FEEDBACK_LOOP_PIPELINE_ANONYMIZATION_LEVEL || "partial") as
            | "none"
            | "partial"
            | "full",
          featureEngineering: {
            timeWindowFeatures:
              process.env
                .FEEDBACK_LOOP_PIPELINE_FEATURE_ENGINEERING_TIME_WINDOW !==
              "false",
            correlationFeatures:
              process.env
                .FEEDBACK_LOOP_PIPELINE_FEATURE_ENGINEERING_CORRELATION !==
              "false",
            trendFeatures:
              process.env.FEEDBACK_LOOP_PIPELINE_FEATURE_ENGINEERING_TREND !==
              "false",
          },
          trainingDataFormat: (process.env
            .FEEDBACK_LOOP_PIPELINE_TRAINING_DATA_FORMAT || "json") as
            | "json"
            | "parquet"
            | "csv",
        },
      },
      violationHandler: {
        escalationThreshold:
          process.env.VIOLATION_HANDLER_ESCALATION_THRESHOLD || "high",
      },
      cawsRuntime: {
        violationHandlingEnabled:
          process.env.CAWS_RUNTIME_VIOLATION_HANDLING_ENABLED !== "false",
        waiverManagementEnabled:
          process.env.CAWS_RUNTIME_WAIVER_MANAGEMENT_ENABLED !== "false",
      },
    };

    // Validate configuration
    const validation = this.validate();
    if (!validation.valid) {
      console.warn("Configuration validation errors:", validation.errors);
      // Continue with defaults for invalid values
    }
  }
}

// Export singleton instance
export const configManager = ConfigManager.getInstance();
