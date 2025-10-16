/**
 * Health Monitoring System
 *
 * Tracks health status of all system components.
 * Provides liveness and readiness endpoints for Kubernetes/production.
 *
 * @author @darianrosebrook
 */

export enum HealthStatus {
  HEALTHY = "healthy",
  DEGRADED = "degraded",
  UNHEALTHY = "unhealthy",
}

export interface ComponentHealth {
  name: string;
  status: HealthStatus;
  message?: string;
  lastCheck: Date;
  details?: Record<string, any>;
}

export interface SystemHealth {
  status: HealthStatus;
  components: ComponentHealth[];
  timestamp: Date;
  uptime: number; // seconds
}

export type HealthCheck = () => Promise<ComponentHealth>;

/**
 * Health monitoring system
 *
 * Registers and executes health checks for system components.
 * Aggregates results to determine overall system health.
 */
export class HealthMonitor {
  private checks: Map<string, HealthCheck> = new Map();
  private lastResults: Map<string, ComponentHealth> = new Map();
  private startTime: Date = new Date();

  /**
   * Register a health check for a component
   */
  registerCheck(name: string, check: HealthCheck): void {
    this.checks.set(name, check);
  }

  /**
   * Unregister a health check
   */
  unregisterCheck(name: string): void {
    this.checks.delete(name);
    this.lastResults.delete(name);
  }

  /**
   * Run all health checks and aggregate results
   */
  async checkHealth(): Promise<SystemHealth> {
    const results: ComponentHealth[] = [];

    // Run all checks in parallel
    const checkPromises = Array.from(this.checks.entries()).map(
      async ([name, check]) => {
        try {
          const result = await Promise.race([
            check(),
            this.timeoutPromise(name, 5000),
          ]);
          this.lastResults.set(name, result);
          return result;
        } catch (error) {
          const failedCheck: ComponentHealth = {
            name,
            status: HealthStatus.UNHEALTHY,
            message: error instanceof Error ? error.message : "Check failed",
            lastCheck: new Date(),
          };
          this.lastResults.set(name, failedCheck);
          return failedCheck;
        }
      }
    );

    results.push(...(await Promise.all(checkPromises)));

    // Determine overall system health
    const unhealthy = results.some((r) => r.status === HealthStatus.UNHEALTHY);
    const degraded = results.some((r) => r.status === HealthStatus.DEGRADED);

    const status = unhealthy
      ? HealthStatus.UNHEALTHY
      : degraded
      ? HealthStatus.DEGRADED
      : HealthStatus.HEALTHY;

    return {
      status,
      components: results,
      timestamp: new Date(),
      uptime: this.getUptimeSeconds(),
    };
  }

  /**
   * Check if system is ready to accept traffic (readiness probe)
   */
  async isReady(): Promise<boolean> {
    const health = await this.checkHealth();
    return health.status !== HealthStatus.UNHEALTHY;
  }

  /**
   * Check if system is alive (liveness probe)
   */
  async isLive(): Promise<boolean> {
    // Liveness is less strict - just check if service is responsive
    // In practice, if we can execute this method, we're alive
    return true;
  }

  /**
   * Get last known health status (without re-checking)
   */
  getLastHealth(): SystemHealth | null {
    if (this.lastResults.size === 0) {
      return null;
    }

    const components = Array.from(this.lastResults.values());
    const unhealthy = components.some(
      (r) => r.status === HealthStatus.UNHEALTHY
    );
    const degraded = components.some((r) => r.status === HealthStatus.DEGRADED);

    const status = unhealthy
      ? HealthStatus.UNHEALTHY
      : degraded
      ? HealthStatus.DEGRADED
      : HealthStatus.HEALTHY;

    return {
      status,
      components,
      timestamp: new Date(),
      uptime: this.getUptimeSeconds(),
    };
  }

  /**
   * Check health of a specific component
   */
  async checkComponent(name: string): Promise<ComponentHealth> {
    const check = this.checks.get(name);
    if (!check) {
      return {
        name,
        status: HealthStatus.UNHEALTHY,
        message: `Component '${name}' not registered`,
        lastCheck: new Date(),
        responseTime: 0,
      };
    }

    try {
      const startTime = Date.now();
      const result = await check();
      const responseTime = Date.now() - startTime;

      const componentHealth: ComponentHealth = {
        name,
        status: result.status,
        message: result.message,
        lastCheck: new Date(),
        responseTime,
        details: result.details,
      };

      // Cache the result
      this.lastResults.set(name, componentHealth);

      return componentHealth;
    } catch (error) {
      const componentHealth: ComponentHealth = {
        name,
        status: HealthStatus.UNHEALTHY,
        message: error instanceof Error ? error.message : String(error),
        lastCheck: new Date(),
        responseTime: 0,
      };

      this.lastResults.set(name, componentHealth);
      return componentHealth;
    }
  }

  /**
   * Get uptime in seconds
   */
  private getUptimeSeconds(): number {
    return Math.floor((Date.now() - this.startTime.getTime()) / 1000);
  }

  /**
   * Create a timeout promise for health checks
   */
  private async timeoutPromise(
    name: string,
    timeoutMs: number
  ): Promise<ComponentHealth> {
    return new Promise<ComponentHealth>((_, reject) =>
      setTimeout(
        () =>
          reject(
            new Error(`Health check '${name}' timed out after ${timeoutMs}ms`)
          ),
        timeoutMs
      )
    );
  }

  /**
   * Get list of registered checks
   */
  getRegisteredChecks(): string[] {
    return Array.from(this.checks.keys());
  }

  /**
   * Clear all health checks
   */
  clear(): void {
    this.checks.clear();
    this.lastResults.clear();
  }
}
