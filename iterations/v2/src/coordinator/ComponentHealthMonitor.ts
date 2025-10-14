/**
 * Component Health Monitor
 *
 * Monitors health of all registered components through regular health checks.
 * Provides real-time health status and failure detection.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import {
  ComponentHealth,
  ComponentRegistration,
  HealthCheckConfig,
  HealthStatus,
} from "../types/coordinator";

export class ComponentHealthMonitor extends EventEmitter {
  private componentHealth = new Map<string, ComponentHealth>();
  private healthChecks = new Map<string, HealthCheckConfig>();
  private checkIntervals = new Map<string, ReturnType<typeof setInterval>>();
  private isRunning = false;

  /**
   * Register component for health monitoring
   */
  async registerComponent(component: ComponentRegistration): Promise<void> {
    this.healthChecks.set(component.id, component.healthCheck);

    // Initialize health status
    this.componentHealth.set(component.id, {
      id: component.id,
      status: HealthStatus.UNKNOWN,
      lastCheck: new Date(),
      responseTime: 0,
      errorCount: 0,
    });

    // Start periodic health checks if running
    if (this.isRunning) {
      this.startHealthChecks(component.id);
    }

    this.emit("component:registered", {
      componentId: component.id,
      timestamp: new Date(),
    });
  }

  /**
   * Unregister component from health monitoring
   */
  async unregisterComponent(componentId: string): Promise<void> {
    // Stop health checks
    const interval = this.checkIntervals.get(componentId);
    if (interval) {
      clearInterval(interval);
      this.checkIntervals.delete(componentId);
    }

    // Remove from monitoring
    this.healthChecks.delete(componentId);
    this.componentHealth.delete(componentId);

    this.emit("component:unregistered", {
      componentId,
      timestamp: new Date(),
    });
  }

  /**
   * Start health monitoring for all registered components
   */
  start(): void {
    if (this.isRunning) return;

    this.isRunning = true;

    // Start health checks for all registered components
    for (const componentId of this.healthChecks.keys()) {
      this.startHealthChecks(componentId);
    }

    this.emit("monitor:started");
  }

  /**
   * Stop health monitoring
   */
  async stop(): Promise<void> {
    if (!this.isRunning) return;

    this.isRunning = false;

    // Stop all health check intervals
    for (const interval of this.checkIntervals.values()) {
      clearInterval(interval);
    }
    this.checkIntervals.clear();

    this.emit("monitor:stopped");
  }

  /**
   * Check health of specific component
   */
  async checkComponentHealth(componentId: string): Promise<ComponentHealth> {
    const config = this.healthChecks.get(componentId);
    if (!config) {
      throw new Error(`No health check config for component ${componentId}`);
    }

    const startTime = Date.now();

    try {
      const response = await this.performHealthCheck(config);
      const responseTime = Date.now() - startTime;

      const currentHealth = this.componentHealth.get(componentId)!;
      const newStatus = this.determineHealthStatus(
        response,
        currentHealth,
        responseTime
      );

      const updatedHealth: ComponentHealth = {
        id: componentId,
        status: newStatus,
        lastCheck: new Date(),
        responseTime,
        errorCount:
          newStatus === HealthStatus.HEALTHY
            ? 0
            : currentHealth.errorCount +
              (newStatus === HealthStatus.UNHEALTHY ? 1 : 0),
        details: response,
      };

      // Emit event if status changed
      if (currentHealth.status !== newStatus) {
        this.emit("component:health-changed", {
          componentId,
          oldStatus: currentHealth.status,
          newStatus,
          timestamp: new Date(),
          responseTime,
          details: response,
        });
      }

      this.componentHealth.set(componentId, updatedHealth);
      return updatedHealth;
    } catch (error) {
      const responseTime = Date.now() - startTime;
      const currentHealth = this.componentHealth.get(componentId)!;

      const updatedHealth: ComponentHealth = {
        id: componentId,
        status: HealthStatus.UNHEALTHY,
        lastCheck: new Date(),
        responseTime,
        errorCount: currentHealth.errorCount + 1,
        details: {
          error: error instanceof Error ? error.message : "Unknown error",
          stack: error instanceof Error ? error.stack : undefined,
        },
      };

      // Emit health change event
      this.emit("component:health-changed", {
        componentId,
        oldStatus: currentHealth.status,
        newStatus: HealthStatus.UNHEALTHY,
        timestamp: new Date(),
        responseTime,
        error: error instanceof Error ? error.message : "Unknown error",
      });

      this.componentHealth.set(componentId, updatedHealth);
      return updatedHealth;
    }
  }

  /**
   * Get health status for all components
   */
  getAllComponentHealth(): ComponentHealth[] {
    return Array.from(this.componentHealth.values());
  }

  /**
   * Get health status for specific component
   */
  getComponentHealth(componentId: string): ComponentHealth | undefined {
    return this.componentHealth.get(componentId);
  }

  /**
   * Get health statistics
   */
  getHealthStats(): {
    total: number;
    healthy: number;
    degraded: number;
    unhealthy: number;
    averageResponseTime: number;
  } {
    const allHealth = Array.from(this.componentHealth.values());

    const healthy = allHealth.filter(
      (h) => h.status === HealthStatus.HEALTHY
    ).length;
    const degraded = allHealth.filter(
      (h) => h.status === HealthStatus.DEGRADED
    ).length;
    const unhealthy = allHealth.filter(
      (h) => h.status === HealthStatus.UNHEALTHY
    ).length;

    const averageResponseTime =
      allHealth.length > 0
        ? allHealth.reduce((sum, h) => sum + h.responseTime, 0) /
          allHealth.length
        : 0;

    return {
      total: allHealth.length,
      healthy,
      degraded,
      unhealthy,
      averageResponseTime,
    };
  }

  /**
   * Force immediate health check for all components
   */
  async checkAllComponents(): Promise<ComponentHealth[]> {
    const results: ComponentHealth[] = [];

    for (const componentId of this.healthChecks.keys()) {
      try {
        const health = await this.checkComponentHealth(componentId);
        results.push(health);
      } catch (error) {
        console.error(`Failed to check health for ${componentId}:`, error);
      }
    }

    return results;
  }

  /**
   * Perform actual health check HTTP request
   */
  private async performHealthCheck(config: HealthCheckConfig): Promise<any> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), config.timeout);

    try {
      const response = await fetch(config.endpoint, {
        method: config.method,
        signal: controller.signal,
        headers: {
          "Content-Type": "application/json",
          "User-Agent": "ARBITER-SystemCoordinator/1.0",
        },
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(
          `Health check failed: ${response.status} ${response.statusText}`
        );
      }

      // Try to parse JSON response
      try {
        return await response.json();
      } catch {
        // If not JSON, return basic success info
        return {
          status: "healthy",
          statusCode: response.status,
          responseTime: Date.now(),
        };
      }
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof Error && error.name === "AbortError") {
        throw new Error(`Health check timeout after ${config.timeout}ms`);
      }

      throw error;
    }
  }

  /**
   * Determine health status based on response and current state
   */
  private determineHealthStatus(
    response: any,
    currentHealth: ComponentHealth,
    responseTime: number
  ): HealthStatus {
    // Check explicit health indicators
    if (response.status === "healthy" || response.healthy === true) {
      return HealthStatus.HEALTHY;
    }

    if (response.status === "degraded" || response.degraded === true) {
      return HealthStatus.DEGRADED;
    }

    if (response.status === "unhealthy" || response.unhealthy === true) {
      return HealthStatus.UNHEALTHY;
    }

    // Check HTTP status codes
    if (response.statusCode >= 200 && response.statusCode < 300) {
      // Additional checks for degraded status
      if (responseTime > 5000) {
        // Slow response
        return HealthStatus.DEGRADED;
      }

      if (currentHealth.errorCount > 3) {
        // Recent errors
        return HealthStatus.DEGRADED;
      }

      return HealthStatus.HEALTHY;
    }

    if (response.statusCode >= 400 && response.statusCode < 500) {
      return HealthStatus.DEGRADED; // Client errors might be temporary
    }

    if (response.statusCode >= 500) {
      return HealthStatus.UNHEALTHY; // Server errors indicate unhealthy
    }

    // Default to healthy if we get here and have a valid response
    if (response && typeof response === "object") {
      return HealthStatus.HEALTHY;
    }

    return HealthStatus.UNHEALTHY;
  }

  /**
   * Start periodic health checks for a component
   */
  private startHealthChecks(componentId: string): void {
    const config = this.healthChecks.get(componentId);
    if (!config) return;

    // Clear any existing interval
    const existingInterval = this.checkIntervals.get(componentId);
    if (existingInterval) {
      clearInterval(existingInterval);
    }

    // Start new interval
    const interval = setInterval(async () => {
      try {
        await this.checkComponentHealth(componentId);
      } catch (error) {
        console.error(`Health check failed for ${componentId}:`, error);
      }
    }, config.interval);

    this.checkIntervals.set(componentId, interval);
  }
}
