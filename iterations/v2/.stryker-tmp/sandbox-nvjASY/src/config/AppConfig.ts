/**
 * Centralized Application Configuration
 *
 * Provides type-safe, environment-aware configuration management.
 * All configuration is loaded from environment variables with sensible defaults.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { z } from "zod";

/**
 * Application configuration schema with validation
 */
const configSchema = z.object({
  // Environment
  env: z
    .enum(["development", "staging", "production", "test"])
    .default("development"),

  // Server
  server: z.object({
    port: z.number().min(1).max(65535).default(3000),
    host: z.string().default("localhost"),
  }),

  // Agent Registry
  registry: z.object({
    maxAgents: z.number().min(1).default(1000),
    cacheEnabled: z.boolean().default(true),
    cacheTTLSeconds: z.number().min(0).default(300),
  }),

  // Task Routing
  routing: z.object({
    maxRoutingTimeMs: z.number().min(1).default(100),
    explorationRate: z.number().min(0).max(1).default(0.1),
    capabilityMatchWeight: z.number().min(0).max(1).default(0.7),
    loadBalancingWeight: z.number().min(0).max(1).default(0.3),
  }),

  // Performance Tracking
  performance: z.object({
    bufferSize: z.number().min(1).default(1000),
    flushIntervalMs: z.number().min(100).default(5000),
    enableAnonymization: z.boolean().default(true),
  }),

  // Observability
  observability: z.object({
    tracingEnabled: z.boolean().default(true),
    metricsEnabled: z.boolean().default(true),
    logLevel: z.enum(["debug", "info", "warn", "error"]).default("info"),
  }),

  // Resilience
  resilience: z.object({
    circuitBreakerEnabled: z.boolean().default(true),
    failureThreshold: z.number().min(1).default(5),
    successThreshold: z.number().min(1).default(2),
    timeoutMs: z.number().min(100).default(5000),
    retryBackoffMs: z.number().min(10).default(1000),
  }),

  // Health Monitoring
  health: z.object({
    checkIntervalMs: z.number().min(1000).default(30000),
    timeoutMs: z.number().min(100).default(5000),
  }),
});

export type AppConfig = z.infer<typeof configSchema>;

/**
 * Singleton configuration manager
 *
 * Loads configuration from environment variables on initialization.
 * Provides type-safe access to all configuration values.
 */
export class ConfigManager {
  private static instance: ConfigManager;
  private config: AppConfig;

  private constructor() {
    this.config = this.loadConfig();
  }

  /**
   * Get singleton instance
   */
  static getInstance(): ConfigManager {
    if (!ConfigManager.instance) {
      ConfigManager.instance = new ConfigManager();
    }
    return ConfigManager.instance;
  }

  /**
   * Load configuration from environment variables
   */
  private loadConfig(): AppConfig {
    const raw = {
      env: process.env.NODE_ENV || "development",
      server: {
        port: this.parseNumber(process.env.PORT, 3000),
        host: process.env.HOST || "localhost",
      },
      registry: {
        maxAgents: this.parseNumber(process.env.MAX_AGENTS, 1000),
        cacheEnabled: this.parseBoolean(process.env.CACHE_ENABLED, true),
        cacheTTLSeconds: this.parseNumber(process.env.CACHE_TTL, 300),
      },
      routing: {
        maxRoutingTimeMs: this.parseNumber(
          process.env.MAX_ROUTING_TIME_MS,
          100
        ),
        explorationRate: this.parseNumber(process.env.EXPLORATION_RATE, 0.1),
        capabilityMatchWeight: this.parseNumber(
          process.env.CAPABILITY_MATCH_WEIGHT,
          0.7
        ),
        loadBalancingWeight: this.parseNumber(
          process.env.LOAD_BALANCING_WEIGHT,
          0.3
        ),
      },
      performance: {
        bufferSize: this.parseNumber(process.env.PERF_BUFFER_SIZE, 1000),
        flushIntervalMs: this.parseNumber(
          process.env.PERF_FLUSH_INTERVAL_MS,
          5000
        ),
        enableAnonymization: this.parseBoolean(
          process.env.PERF_ENABLE_ANONYMIZATION,
          true
        ),
      },
      observability: {
        tracingEnabled: this.parseBoolean(process.env.TRACING_ENABLED, true),
        metricsEnabled: this.parseBoolean(process.env.METRICS_ENABLED, true),
        logLevel: (process.env.LOG_LEVEL as any) || "info",
      },
      resilience: {
        circuitBreakerEnabled: this.parseBoolean(
          process.env.CIRCUIT_BREAKER_ENABLED,
          true
        ),
        failureThreshold: this.parseNumber(process.env.FAILURE_THRESHOLD, 5),
        successThreshold: this.parseNumber(process.env.SUCCESS_THRESHOLD, 2),
        timeoutMs: this.parseNumber(process.env.TIMEOUT_MS, 5000),
        retryBackoffMs: this.parseNumber(process.env.RETRY_BACKOFF_MS, 1000),
      },
      health: {
        checkIntervalMs: this.parseNumber(
          process.env.HEALTH_CHECK_INTERVAL_MS,
          30000
        ),
        timeoutMs: this.parseNumber(process.env.HEALTH_TIMEOUT_MS, 5000),
      },
    };

    try {
      return configSchema.parse(raw);
    } catch (error) {
      console.error("Configuration validation failed:", error);
      throw new Error(
        `Invalid configuration: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }
  }

  /**
   * Parse number from string with fallback
   */
  private parseNumber(value: string | undefined, fallback: number): number {
    if (!value) return fallback;
    const parsed = Number(value);
    return isNaN(parsed) ? fallback : parsed;
  }

  /**
   * Parse boolean from string with fallback
   */
  private parseBoolean(value: string | undefined, fallback: boolean): boolean {
    if (!value) return fallback;
    return value.toLowerCase() === "true" || value === "1";
  }

  /**
   * Get current configuration
   */
  get(): AppConfig {
    return this.config;
  }

  /**
   * Reload configuration from environment
   */
  reload(): void {
    this.config = this.loadConfig();
  }

  /**
   * Get specific configuration section
   */
  getSection<K extends keyof AppConfig>(section: K): AppConfig[K] {
    return this.config[section];
  }
}

/**
 * Get global configuration instance
 */
export const getConfig = (): AppConfig => ConfigManager.getInstance().get();
