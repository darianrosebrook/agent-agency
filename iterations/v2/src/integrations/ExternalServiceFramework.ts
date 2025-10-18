/**
 * External Service Integration Framework
 *
 * Unified framework for integrating with external services including
 * notifications, monitoring, incident management, and infrastructure.
 * Provides a consistent interface for all external service integrations.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { Logger } from "../utils/Logger";

/**
 * Service integration types
 */
export type ServiceType =
  | "notification"
  | "monitoring"
  | "infrastructure"
  | "incident";

/**
 * Service configuration interface
 */
export interface ServiceConfig {
  name: string;
  type: ServiceType;
  enabled: boolean;
  timeout: number;
  retries: number;
  credentials?: Record<string, any>;
  endpoints?: Record<string, string>;
  [key: string]: any;
}

/**
 * Service operation result
 */
export interface ServiceOperationResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  duration: number;
  timestamp: Date;
}

/**
 * Health check result
 */
export interface HealthCheckResult {
  healthy: boolean;
  status: "healthy" | "degraded" | "unhealthy";
  message?: string;
  responseTime?: number;
  lastCheck: Date;
}

/**
 * Base service integration interface
 */
export interface ServiceIntegration {
  readonly name: string;
  readonly type: ServiceType;
  readonly config: ServiceConfig;

  /**
   * Initialize the service integration
   */
  initialize(): Promise<void>;

  /**
   * Perform health check
   */
  healthCheck(): Promise<HealthCheckResult>;

  /**
   * Execute a service operation
   */
  execute<T = any>(
    operation: string,
    params?: any
  ): Promise<ServiceOperationResult<T>>;

  /**
   * Cleanup resources
   */
  cleanup(): Promise<void>;
}

/**
 * Service integration manager
 */
export class ServiceIntegrationManager extends EventEmitter {
  private readonly logger = new Logger("ServiceIntegrationManager");
  private integrations = new Map<string, ServiceIntegration>();
  private healthCheckInterval?: NodeJS.Timeout;

  constructor(
    private config: {
      healthCheckIntervalMs?: number;
      enableHealthChecks?: boolean;
    } = {}
  ) {
    super();

    this.config = {
      healthCheckIntervalMs: 30000, // 30 seconds
      enableHealthChecks: true,
      ...config,
    };
  }

  /**
   * Register a service integration
   */
  async register(integration: ServiceIntegration): Promise<void> {
    try {
      await integration.initialize();
      this.integrations.set(integration.name, integration);
      this.logger.info(`Registered service integration: ${integration.name}`, {
        type: integration.type,
        enabled: integration.config.enabled,
      });

      this.emit("integration:registered", {
        name: integration.name,
        type: integration.type,
      });
    } catch (error) {
      this.logger.error(
        `Failed to register service integration: ${integration.name}`,
        { error }
      );
      throw error;
    }
  }

  /**
   * Get a service integration by name
   */
  get<T extends ServiceIntegration>(name: string): T | undefined {
    return this.integrations.get(name) as T;
  }

  /**
   * Get all integrations of a specific type
   */
  getByType<T extends ServiceIntegration>(type: ServiceType): T[] {
    return Array.from(this.integrations.values())
      .filter((integration) => integration.type === type)
      .map((integration) => integration as T);
  }

  /**
   * Execute operation on a specific service
   */
  async execute<T = any>(
    serviceName: string,
    operation: string,
    params?: any
  ): Promise<ServiceOperationResult<T>> {
    const integration = this.integrations.get(serviceName);
    if (!integration) {
      return {
        success: false,
        error: `Service integration not found: ${serviceName}`,
        duration: 0,
        timestamp: new Date(),
      };
    }

    if (!integration.config.enabled) {
      return {
        success: false,
        error: `Service integration disabled: ${serviceName}`,
        duration: 0,
        timestamp: new Date(),
      };
    }

    const startTime = Date.now();
    try {
      const result = await integration.execute<T>(operation, params);
      const duration = Date.now() - startTime;

      this.logger.debug(
        `Service operation completed: ${serviceName}.${operation}`,
        {
          duration,
          success: result.success,
        }
      );

      return {
        ...result,
        duration,
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      this.logger.error(
        `Service operation failed: ${serviceName}.${operation}`,
        {
          error: errorMessage,
          duration,
        }
      );

      return {
        success: false,
        error: errorMessage,
        duration,
        timestamp: new Date(),
      };
    }
  }

  /**
   * Execute operation on all services of a specific type
   */
  async executeOnAll<T = any>(
    type: ServiceType,
    operation: string,
    params?: any
  ): Promise<Map<string, ServiceOperationResult<T>>> {
    const results = new Map<string, ServiceOperationResult<T>>();
    const integrations = this.getByType(type);

    const promises = integrations.map(async (integration) => {
      const result = await this.execute<T>(integration.name, operation, params);
      results.set(integration.name, result);
    });

    await Promise.allSettled(promises);
    return results;
  }

  /**
   * Get health status of all integrations
   */
  async getHealthStatus(): Promise<Map<string, HealthCheckResult>> {
    const results = new Map<string, HealthCheckResult>();
    const integrations = Array.from(this.integrations.values());

    const promises = integrations.map(async (integration) => {
      try {
        const health = await integration.healthCheck();
        results.set(integration.name, health);
      } catch (error) {
        results.set(integration.name, {
          healthy: false,
          status: "unhealthy",
          message: error instanceof Error ? error.message : String(error),
          lastCheck: new Date(),
        });
      }
    });

    await Promise.allSettled(promises);
    return results;
  }

  /**
   * Start health check monitoring
   */
  startHealthChecks(): void {
    if (!this.config.enableHealthChecks || this.healthCheckInterval) {
      return;
    }

    this.healthCheckInterval = setInterval(async () => {
      try {
        const healthStatus = await this.getHealthStatus();
        this.emit("health:check", healthStatus);

        // Emit individual health events
        for (const [serviceName, health] of healthStatus) {
          this.emit(`health:${serviceName}`, health);
        }
      } catch (error) {
        this.logger.error("Health check failed", { error });
      }
    }, this.config.healthCheckIntervalMs);
  }

  /**
   * Stop health check monitoring
   */
  stopHealthChecks(): void {
    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
      this.healthCheckInterval = undefined;
    }
  }

  /**
   * Cleanup all integrations
   */
  async cleanup(): Promise<void> {
    this.stopHealthChecks();

    const cleanupPromises = Array.from(this.integrations.values()).map(
      async (integration) => {
        try {
          await integration.cleanup();
        } catch (error) {
          this.logger.error(
            `Failed to cleanup integration: ${integration.name}`,
            { error }
          );
        }
      }
    );

    await Promise.allSettled(cleanupPromises);
    this.integrations.clear();
  }
}

/**
 * Base service integration implementation
 */
export abstract class BaseServiceIntegration implements ServiceIntegration {
  protected readonly logger: Logger;

  constructor(
    public readonly name: string,
    public readonly type: ServiceType,
    public readonly config: ServiceConfig
  ) {
    this.logger = new Logger(`ServiceIntegration:${name}`);
  }

  abstract initialize(): Promise<void>;
  abstract healthCheck(): Promise<HealthCheckResult>;
  abstract execute<T = any>(
    operation: string,
    params?: any
  ): Promise<ServiceOperationResult<T>>;

  async cleanup(): Promise<void> {
    // Default cleanup implementation - override if needed
    this.logger.debug(`Cleaning up service integration: ${this.name}`);
  }

  /**
   * Helper method to create operation result
   */
  protected createResult<T = any>(
    success: boolean,
    data?: T,
    error?: string,
    duration?: number
  ): ServiceOperationResult<T> {
    return {
      success,
      data,
      error,
      duration: duration || 0,
      timestamp: new Date(),
    };
  }
}
