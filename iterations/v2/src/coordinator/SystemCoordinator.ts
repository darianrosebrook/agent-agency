/**
 * System Coordinator
 *
 * Centralized coordination system that manages component registration,
 * health monitoring, load balancing, and failure recovery across
 * all ARBITER components.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import {
  ComponentRegistration,
  ComponentType,
  CoordinatorStats,
  HealthStatus,
  RoutingPreferences,
  SystemCoordinatorConfig,
  SystemHealth,
} from "../types/coordinator";

import { ComponentHealthMonitor } from "./ComponentHealthMonitor";
import { FailureManager } from "./FailureManager";
import { LoadBalancer } from "./LoadBalancer";

export class SystemCoordinator extends EventEmitter {
  private components = new Map<string, ComponentRegistration>();
  private componentHealth = new Map<string, ComponentHealth>();
  private loadBalancer: LoadBalancer;
  private failureManager: FailureManager;
  private healthMonitor: ComponentHealthMonitor;
  private healthCheckInterval?: NodeJS.Timeout;
  private stats = {
    totalRequests: 0,
    responseTimes: [] as number[],
  };

  constructor(
    private config: SystemCoordinatorConfig,
    healthMonitor: ComponentHealthMonitor
  ) {
    super();

    this.healthMonitor = healthMonitor;
    this.loadBalancer = new LoadBalancer(this);
    this.failureManager = new FailureManager(this, config);

    this.setupEventHandlers();
    this.startHealthMonitoring();
  }

  /**
   * Register a component with the coordinator
   */
  async registerComponent(registration: ComponentRegistration): Promise<void> {
    // Validate component dependencies
    await this.validateDependencies(registration);

    // Register component
    this.components.set(registration.id, registration);
    this.componentHealth.set(registration.id, {
      id: registration.id,
      status: HealthStatus.UNKNOWN,
      lastCheck: new Date(),
      responseTime: 0,
      errorCount: 0,
    });

    // Start health monitoring for component
    await this.healthMonitor.registerComponent(registration);

    this.emit("component:registered", {
      componentId: registration.id,
      type: registration.type,
      timestamp: new Date(),
    });

    // Notify dependent components
    await this.notifyDependents(registration);
  }

  /**
   * Unregister a component
   */
  async unregisterComponent(componentId: string): Promise<void> {
    const component = this.components.get(componentId);
    if (!component) {
      throw new Error(`Component ${componentId} not found`);
    }

    // Stop health monitoring
    await this.healthMonitor.unregisterComponent(componentId);

    // Remove from registry
    this.components.delete(componentId);
    this.componentHealth.delete(componentId);

    // Redistribute load
    await this.loadBalancer.handleComponentRemoval(component);

    this.emit("component:unregistered", {
      componentId,
      type: component.type,
      timestamp: new Date(),
    });
  }

  /**
   * Get component by ID
   */
  getComponent(componentId: string): ComponentRegistration | undefined {
    return this.components.get(componentId);
  }

  /**
   * Get all components
   */
  getAllComponents(): ComponentRegistration[] {
    return Array.from(this.components.values());
  }

  /**
   * Get components by type
   */
  getComponentsByType(type: ComponentType): ComponentRegistration[] {
    return Array.from(this.components.values()).filter((c) => c.type === type);
  }

  /**
   * Get healthy components of a specific type
   */
  getHealthyComponents(type: ComponentType): ComponentRegistration[] {
    return this.getComponentsByType(type).filter((component) => {
      const health = this.componentHealth.get(component.id);
      return health?.status === HealthStatus.HEALTHY;
    });
  }

  /**
   * Get component health status
   */
  getComponentHealth(componentId: string) {
    return this.componentHealth.get(componentId);
  }

  /**
   * Route request to appropriate component
   */
  async routeRequest(
    requestType: string,
    payload: any,
    preferences?: RoutingPreferences
  ): Promise<ComponentRegistration> {
    const startTime = Date.now();

    try {
      const candidates = this.getHealthyComponents(
        this.getComponentTypeForRequest(requestType)
      );

      if (candidates.length === 0) {
        throw new Error(`No healthy components available for ${requestType}`);
      }

      const selected = await this.loadBalancer.selectComponent(
        candidates,
        payload,
        preferences
      );

      // Track metrics
      this.stats.totalRequests++;
      const responseTime = Date.now() - startTime;
      this.stats.responseTimes.push(responseTime);

      // Keep only last 100 response times
      if (this.stats.responseTimes.length > 100) {
        this.stats.responseTimes.shift();
      }

      this.emit("request:routed", {
        requestType,
        componentId: selected.id,
        responseTime,
        timestamp: new Date(),
      });

      return selected;
    } catch (error) {
      this.emit("request:routing-failed", {
        requestType,
        error: error instanceof Error ? error.message : "Unknown error",
        timestamp: new Date(),
      });
      throw error;
    }
  }

  /**
   * Get system health status
   */
  getSystemHealth(): SystemHealth {
    const componentHealth = Array.from(this.componentHealth.values());
    const healthy = componentHealth.filter(
      (h) => h.status === HealthStatus.HEALTHY
    ).length;
    const degraded = componentHealth.filter(
      (h) => h.status === HealthStatus.DEGRADED
    ).length;
    const unhealthy = componentHealth.filter(
      (h) => h.status === HealthStatus.UNHEALTHY
    ).length;
    const total = componentHealth.length;

    const overallStatus = this.calculateOverallStatus(componentHealth);

    return {
      status: overallStatus,
      components: {
        total,
        healthy,
        degraded,
        unhealthy,
      },
      lastUpdate: new Date(),
      issues: this.getCurrentIssues(),
    };
  }

  /**
   * Get coordinator statistics
   */
  getStats(): CoordinatorStats {
    const components = Array.from(this.components.values());
    const componentHealth = Array.from(this.componentHealth.values());

    // Count by type
    const byType: Record<ComponentType, number> = {
      [ComponentType.AGENT_REGISTRY]: 0,
      [ComponentType.TASK_ROUTER]: 0,
      [ComponentType.CAWS_VALIDATOR]: 0,
      [ComponentType.PERFORMANCE_TRACKER]: 0,
      [ComponentType.TASK_ORCHESTRATOR]: 0,
      [ComponentType.CONSTITUTIONAL_RUNTIME]: 0,
    };

    for (const component of components) {
      byType[component.type]++;
    }

    // Health counts
    const healthy = componentHealth.filter(
      (h) => h.status === HealthStatus.HEALTHY
    ).length;
    const degraded = componentHealth.filter(
      (h) => h.status === HealthStatus.DEGRADED
    ).length;
    const unhealthy = componentHealth.filter(
      (h) => h.status === HealthStatus.UNHEALTHY
    ).length;

    // Load metrics
    const averageResponseTime =
      this.stats.responseTimes.length > 0
        ? this.stats.responseTimes.reduce((sum, time) => sum + time, 0) /
          this.stats.responseTimes.length
        : 0;

    // Failure stats
    const failureStats = this.failureManager.getFailureStats();

    return {
      components: {
        total: components.length,
        byType,
      },
      health: {
        healthy,
        degraded,
        unhealthy,
      },
      load: {
        totalRequests: this.stats.totalRequests,
        averageResponseTime,
      },
      failures: failureStats,
    };
  }

  /**
   * Handle component failure
   */
  async handleComponentFailure(componentId: string, error: any): Promise<void> {
    await this.failureManager.handleFailure(componentId, error);
  }

  /**
   * Redistribute load after component changes
   */
  async redistributeLoad(): Promise<void> {
    await this.loadBalancer.redistributeLoad();
  }

  /**
   * Stop the coordinator
   */
  async stop(): Promise<void> {
    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
      this.healthCheckInterval = undefined;
    }

    // Stop health monitoring
    await this.healthMonitor.stop();

    this.emit("coordinator:stopped");
  }

  /**
   * Map request type to component type
   */
  private getComponentTypeForRequest(requestType: string): ComponentType {
    switch (requestType) {
      case "agent_registration":
      case "agent_lookup":
      case "agent_update":
        return ComponentType.AGENT_REGISTRY;

      case "task_routing":
      case "task_assignment":
        return ComponentType.TASK_ROUTER;

      case "caws_validation":
      case "spec_validation":
        return ComponentType.CAWS_VALIDATOR;

      case "performance_tracking":
      case "metrics_collection":
        return ComponentType.PERFORMANCE_TRACKER;

      case "task_orchestration":
      case "task_execution":
        return ComponentType.TASK_ORCHESTRATOR;

      case "constitutional_validation":
      case "compliance_check":
        return ComponentType.CONSTITUTIONAL_RUNTIME;

      default:
        throw new Error(`Unknown request type: ${requestType}`);
    }
  }

  /**
   * Calculate overall system health status
   */
  private calculateOverallStatus(componentHealth: any[]): HealthStatus {
    if (componentHealth.length === 0) return HealthStatus.UNKNOWN;

    const unhealthy = componentHealth.filter(
      (h) => h.status === HealthStatus.UNHEALTHY
    ).length;
    const degraded = componentHealth.filter(
      (h) => h.status === HealthStatus.DEGRADED
    ).length;

    if (unhealthy > 0) return HealthStatus.UNHEALTHY;
    if (degraded > componentHealth.length * 0.5) return HealthStatus.DEGRADED;
    if (degraded > 0) return HealthStatus.DEGRADED;

    return HealthStatus.HEALTHY;
  }

  /**
   * Get current health issues
   */
  private getCurrentIssues(): any[] {
    const issues: any[] = [];

    for (const [componentId, health] of this.componentHealth) {
      if (health.status !== HealthStatus.HEALTHY) {
        issues.push({
          componentId,
          type: "health_check",
          severity:
            health.status === HealthStatus.UNHEALTHY ? "high" : "medium",
          message: `Component ${componentId} is ${health.status}`,
          timestamp: health.lastCheck,
        });
      }
    }

    return issues;
  }

  /**
   * Validate component dependencies
   */
  private async validateDependencies(
    component: ComponentRegistration
  ): Promise<void> {
    for (const depId of component.dependencies) {
      if (!this.components.has(depId)) {
        throw new Error(
          `Dependency ${depId} not registered for component ${component.id}`
        );
      }
    }
  }

  /**
   * Notify dependent components
   */
  private async notifyDependents(
    component: ComponentRegistration
  ): Promise<void> {
    // Notify components that depend on this one
    for (const [id, otherComponent] of this.components) {
      if (otherComponent.dependencies.includes(component.id)) {
        this.emit("component:dependency-available", {
          dependentId: id,
          dependencyId: component.id,
          timestamp: new Date(),
        });
      }
    }
  }

  /**
   * Setup event handlers
   */
  private setupEventHandlers(): void {
    // Forward health events
    this.healthMonitor.on("component:health-changed", (event) => {
      this.emit("component:health-changed", event);
    });

    // Handle load balancing events
    this.loadBalancer.on("load:redistributed", (event) => {
      this.emit("load:redistributed", event);
    });

    // Handle failure recovery events
    this.failureManager.on("component:recovered", (event) => {
      this.emit("component:recovered", event);
    });

    this.failureManager.on("recovery:failed", (event) => {
      this.emit("recovery:failed", event);
    });
  }

  /**
   * Start health monitoring
   */
  private startHealthMonitoring(): void {
    this.healthCheckInterval = setInterval(async () => {
      await this.performHealthChecks();
    }, this.config.healthCheckInterval);
  }

  /**
   * Perform health checks for all components
   */
  private async performHealthChecks(): Promise<void> {
    for (const component of this.components.values()) {
      try {
        const health = await this.healthMonitor.checkComponentHealth(
          component.id
        );
        this.componentHealth.set(component.id, health);

        // Update load balancer with health info
        await this.loadBalancer.updateComponentHealth(component.id, health);
      } catch (error) {
        console.error(`Health check failed for ${component.id}:`, error);
      }
    }
  }
}

