# Phase 2.1: System Coordinator Implementation Plan

**Date**: October 12, 2025
**Status**: 🔄 In Progress
**Expected Duration**: 4-5 hours

---

## Overview

Implement the System Coordinator that provides centralized coordination, health monitoring, and recovery management across all ARBITER components. This creates a unified system view and enables autonomous operation with automatic failure handling.

---

## Goals

1. **System Coordination**: Centralized coordination across all ARBITER components
2. **Health Monitoring**: Comprehensive health monitoring and alerting
3. **Failure Detection**: Automatic detection of component failures
4. **Recovery Management**: Automated recovery and failover procedures
5. **Load Balancing**: System-wide load distribution and optimization
6. **Unified State**: Centralized system state management

---

## System Coordinator Architecture

```
┌─────────────────────────────────────────────────┐
│              System Coordinator                 │
│                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
│  │ Health       │  │ Load        │  │ Failure │ │
│  │ Monitor      │  │ Balancer    │  │ Manager │ │
│  └─────────────┘  └─────────────┘  └─────────┘ │
│                                                 │
│  ┌─────────────────────────────────────────────┐ │
│  │         Component Registry                 │ │
│  │                                             │ │
│  │  • ARBITER-001: Agent Registry              │ │
│  │  • ARBITER-002: Task Routing                │ │
│  │  │  • ARBITER-003: CAWS Validation           │ │
│  │  • ARBITER-004: Performance Tracking        │ │
│  │  • ARBITER-005: Task Orchestrator           │ │
│  └─────────────────────────────────────────────┘ │
│                                                 │
│  ┌─────────────────────────────────────────────┐ │
│  │         Coordination Services              │ │
│  │                                             │ │
│  │  • State Synchronization                     │ │
│  │  • Event Coordination                        │ │
│  │  • Resource Allocation                       │ │
│  │  • Configuration Distribution                │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

---

## Implementation Components

### 1. System Coordinator Core

```typescript
// src/coordinator/SystemCoordinator.ts

export interface ComponentRegistration {
  id: string;
  name: string;
  type: ComponentType;
  endpoint: string;
  healthCheck: HealthCheckConfig;
  capabilities: ComponentCapabilities;
  dependencies: string[];
  metadata: Record<string, any>;
}

export enum ComponentType {
  AGENT_REGISTRY = "agent_registry",
  TASK_ROUTER = "task_router",
  CAWS_VALIDATOR = "caws_validator",
  PERFORMANCE_TRACKER = "performance_tracker",
  TASK_ORCHESTRATOR = "task_orchestrator",
  CONSTITUTIONAL_RUNTIME = "constitutional_runtime",
}

export interface ComponentCapabilities {
  maxConcurrentTasks?: number;
  supportedTaskTypes?: string[];
  performanceMetrics?: boolean;
  healthMonitoring?: boolean;
  constitutionalCompliance?: boolean;
}

export interface SystemCoordinatorConfig {
  healthCheckInterval: number;
  failureThreshold: number;
  recoveryTimeout: number;
  loadBalancingEnabled: boolean;
  autoScalingEnabled: boolean;
  maxComponentsPerType: number;
}

export class SystemCoordinator extends EventEmitter {
  private components = new Map<string, ComponentRegistration>();
  private componentHealth = new Map<string, ComponentHealth>();
  private loadBalancer: LoadBalancer;
  private failureManager: FailureManager;
  private healthMonitor: HealthMonitor;

  constructor(
    private config: SystemCoordinatorConfig,
    healthMonitor: HealthMonitor
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
   * Get all components of a specific type
   */
  getComponentsByType(type: ComponentType): ComponentRegistration[] {
    return Array.from(this.components.values()).filter(c => c.type === type);
  }

  /**
   * Get healthy components of a specific type
   */
  getHealthyComponents(type: ComponentType): ComponentRegistration[] {
    return this.getComponentsByType(type).filter(component => {
      const health = this.componentHealth.get(component.id);
      return health?.status === HealthStatus.HEALTHY;
    });
  }

  /**
   * Route request to appropriate component
   */
  async routeRequest(
    requestType: string,
    payload: any,
    preferences?: RoutingPreferences
  ): Promise<ComponentRegistration> {
    const candidates = this.getHealthyComponents(this.getComponentTypeForRequest(requestType));

    if (candidates.length === 0) {
      throw new Error(`No healthy components available for ${requestType}`);
    }

    return this.loadBalancer.selectComponent(candidates, payload, preferences);
  }

  /**
   * Get system health status
   */
  getSystemHealth(): SystemHealth {
    const componentHealth = Array.from(this.componentHealth.values());
    const healthy = componentHealth.filter(h => h.status === HealthStatus.HEALTHY).length;
    const total = componentHealth.length;

    const overallStatus = this.calculateOverallStatus(componentHealth);

    return {
      status: overallStatus,
      components: {
        total,
        healthy,
        degraded: componentHealth.filter(h => h.status === HealthStatus.DEGRADED).length,
        unhealthy: componentHealth.filter(h => h.status === HealthStatus.UNHEALTHY).length,
      },
      lastUpdate: new Date(),
      issues: this.getCurrentIssues(),
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

  private getComponentTypeForRequest(requestType: string): ComponentType {
    // Map request types to component types
    switch (requestType) {
      case "agent_registration":
      case "agent_lookup":
        return ComponentType.AGENT_REGISTRY;
      case "task_routing":
        return ComponentType.TASK_ROUTER;
      case "caws_validation":
        return ComponentType.CAWS_VALIDATOR;
      case "performance_tracking":
        return ComponentType.PERFORMANCE_TRACKER;
      case "task_orchestration":
        return ComponentType.TASK_ORCHESTRATOR;
      default:
        throw new Error(`Unknown request type: ${requestType}`);
    }
  }

  private calculateOverallStatus(componentHealth: ComponentHealth[]): HealthStatus {
    if (componentHealth.length === 0) return HealthStatus.UNKNOWN;

    const unhealthy = componentHealth.filter(h => h.status === HealthStatus.UNHEALTHY).length;
    const degraded = componentHealth.filter(h => h.status === HealthStatus.DEGRADED).length;

    if (unhealthy > 0) return HealthStatus.UNHEALTHY;
    if (degraded > componentHealth.length * 0.5) return HealthStatus.DEGRADED;
    if (degraded > 0) return HealthStatus.DEGRADED;

    return HealthStatus.HEALTHY;
  }

  private getCurrentIssues(): HealthIssue[] {
    const issues: HealthIssue[] = [];

    for (const [componentId, health] of this.componentHealth) {
      if (health.status !== HealthStatus.HEALTHY) {
        issues.push({
          componentId,
          type: "health_check",
          severity: health.status === HealthStatus.UNHEALTHY ? "high" : "medium",
          message: `Component ${componentId} is ${health.status}`,
          timestamp: health.lastCheck,
        });
      }
    }

    return issues;
  }

  private async validateDependencies(component: ComponentRegistration): Promise<void> {
    for (const depId of component.dependencies) {
      if (!this.components.has(depId)) {
        throw new Error(`Dependency ${depId} not registered for component ${component.id}`);
      }
    }
  }

  private async notifyDependents(component: ComponentRegistration): Promise<void> {
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
  }

  private startHealthMonitoring(): void {
    setInterval(async () => {
      await this.performHealthChecks();
    }, this.config.healthCheckInterval);
  }

  private async performHealthChecks(): Promise<void> {
    for (const component of this.components.values()) {
      try {
        const health = await this.healthMonitor.checkComponentHealth(component.id);
        this.componentHealth.set(component.id, health);

        // Update load balancer with health info
        await this.loadBalancer.updateComponentHealth(component.id, health);
      } catch (error) {
        console.error(`Health check failed for ${component.id}:`, error);
      }
    }
  }
}
```

### 2. Health Monitor Integration

```typescript
// src/coordinator/HealthMonitor.ts

export interface ComponentHealth {
  id: string;
  status: HealthStatus;
  lastCheck: Date;
  responseTime: number;
  errorCount: number;
  details?: Record<string, any>;
}

export enum HealthStatus {
  HEALTHY = "healthy",
  DEGRADED = "degraded",
  UNHEALTHY = "unhealthy",
  UNKNOWN = "unknown",
}

export interface HealthIssue {
  componentId: string;
  type: string;
  severity: "low" | "medium" | "high" | "critical";
  message: string;
  timestamp: Date;
}

export interface SystemHealth {
  status: HealthStatus;
  components: {
    total: number;
    healthy: number;
    degraded: number;
    unhealthy: number;
  };
  lastUpdate: Date;
  issues: HealthIssue[];
}

export interface HealthCheckConfig {
  endpoint: string;
  method: "GET" | "POST";
  timeout: number;
  interval: number;
  retries: number;
}

export class ComponentHealthMonitor extends EventEmitter {
  private componentHealth = new Map<string, ComponentHealth>();
  private healthChecks = new Map<string, HealthCheckConfig>();

  /**
   * Register component for health monitoring
   */
  async registerComponent(component: ComponentRegistration): Promise<void> {
    this.healthChecks.set(component.id, component.healthCheck);

    this.componentHealth.set(component.id, {
      id: component.id,
      status: HealthStatus.UNKNOWN,
      lastCheck: new Date(),
      responseTime: 0,
      errorCount: 0,
    });
  }

  /**
   * Unregister component from health monitoring
   */
  async unregisterComponent(componentId: string): Promise<void> {
    this.healthChecks.delete(componentId);
    this.componentHealth.delete(componentId);
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
      const newStatus = this.determineHealthStatus(response, currentHealth);

      const updatedHealth: ComponentHealth = {
        id: componentId,
        status: newStatus,
        lastCheck: new Date(),
        responseTime,
        errorCount: newStatus === HealthStatus.HEALTHY ? 0 : currentHealth.errorCount + (newStatus === HealthStatus.UNHEALTHY ? 1 : 0),
        details: response,
      };

      // Emit event if status changed
      if (currentHealth.status !== newStatus) {
        this.emit("component:health-changed", {
          componentId,
          oldStatus: currentHealth.status,
          newStatus,
          timestamp: new Date(),
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
        details: { error: error instanceof Error ? error.message : "Unknown error" },
      };

      this.emit("component:health-changed", {
        componentId,
        oldStatus: currentHealth.status,
        newStatus: HealthStatus.UNHEALTHY,
        timestamp: new Date(),
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

  private async performHealthCheck(config: HealthCheckConfig): Promise<any> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), config.timeout);

    try {
      const response = await fetch(config.endpoint, {
        method: config.method,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`Health check failed: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    }
  }

  private determineHealthStatus(response: any, currentHealth: ComponentHealth): HealthStatus {
    // Simple health determination - can be extended with custom logic
    if (response.status === "healthy" || response.healthy === true) {
      return HealthStatus.HEALTHY;
    }

    if (response.status === "degraded" || response.degraded === true) {
      return HealthStatus.DEGRADED;
    }

    // If response time is high or error count is increasing
    if (currentHealth.responseTime > 5000 || currentHealth.errorCount > 3) {
      return HealthStatus.DEGRADED;
    }

    return HealthStatus.HEALTHY;
  }
}
```

### 3. Load Balancer

```typescript
// src/coordinator/LoadBalancer.ts

export interface RoutingPreferences {
  preferredComponent?: string;
  avoidComponents?: string[];
  maxLoad?: number;
  location?: string;
  capabilities?: string[];
}

export interface LoadDistribution {
  componentId: string;
  loadPercentage: number;
  activeConnections: number;
  queueDepth: number;
}

export class LoadBalancer extends EventEmitter {
  private loadDistribution = new Map<string, LoadDistribution>();
  private componentLoads = new Map<string, number>();

  constructor(private coordinator: SystemCoordinator) {
    super();
  }

  /**
   * Select best component for request
   */
  async selectComponent(
    candidates: ComponentRegistration[],
    payload: any,
    preferences?: RoutingPreferences
  ): Promise<ComponentRegistration> {
    if (candidates.length === 1) {
      return candidates[0];
    }

    // Apply preferences first
    let filteredCandidates = this.applyPreferences(candidates, preferences);

    if (filteredCandidates.length === 0) {
      filteredCandidates = candidates; // Fallback to all candidates
    }

    // Score candidates based on load and capabilities
    const scoredCandidates = await Promise.all(
      filteredCandidates.map(async (candidate) => ({
        component: candidate,
        score: await this.calculateScore(candidate, payload),
      }))
    );

    // Sort by score (highest first)
    scoredCandidates.sort((a, b) => b.score - a.score);

    const selected = scoredCandidates[0].component;

    // Update load tracking
    this.updateLoadTracking(selected.id);

    return selected;
  }

  /**
   * Handle component removal (redistribute load)
   */
  async handleComponentRemoval(component: ComponentRegistration): Promise<void> {
    this.loadDistribution.delete(component.id);
    this.componentLoads.delete(component.id);

    await this.redistributeLoad();

    this.emit("load:redistributed", {
      reason: "component_removal",
      removedComponent: component.id,
      timestamp: new Date(),
    });
  }

  /**
   * Redistribute load across available components
   */
  async redistributeLoad(): Promise<void> {
    const allComponents = this.coordinator.getAllComponents();
    const healthyComponents = allComponents.filter(component => {
      const health = this.coordinator.getComponentHealth(component.id);
      return health?.status === HealthStatus.HEALTHY;
    });

    // Calculate equal distribution
    const loadPerComponent = 100 / healthyComponents.length;

    for (const component of healthyComponents) {
      this.loadDistribution.set(component.id, {
        componentId: component.id,
        loadPercentage: loadPerComponent,
        activeConnections: 0,
        queueDepth: 0,
      });
    }

    this.emit("load:redistributed", {
      reason: "manual_redistribution",
      componentCount: healthyComponents.length,
      timestamp: new Date(),
    });
  }

  /**
   * Update component health for load balancing
   */
  async updateComponentHealth(componentId: string, health: ComponentHealth): Promise<void> {
    if (health.status !== HealthStatus.HEALTHY) {
      // Reduce load on unhealthy components
      const distribution = this.loadDistribution.get(componentId);
      if (distribution) {
        distribution.loadPercentage *= 0.5; // Reduce to 50%
      }
    }
  }

  /**
   * Get current load distribution
   */
  getLoadDistribution(): LoadDistribution[] {
    return Array.from(this.loadDistribution.values());
  }

  private applyPreferences(
    candidates: ComponentRegistration[],
    preferences?: RoutingPreferences
  ): ComponentRegistration[] {
    if (!preferences) return candidates;

    let filtered = candidates;

    // Preferred component
    if (preferences.preferredComponent) {
      const preferred = filtered.find(c => c.id === preferences.preferredComponent);
      if (preferred) return [preferred];
    }

    // Avoid components
    if (preferences.avoidComponents?.length) {
      filtered = filtered.filter(c => !preferences.avoidComponents!.includes(c.id));
    }

    // Max load filter
    if (preferences.maxLoad !== undefined) {
      filtered = filtered.filter(c => {
        const load = this.componentLoads.get(c.id) || 0;
        return load < preferences!.maxLoad!;
      });
    }

    // Location filter
    if (preferences.location) {
      filtered = filtered.filter(c =>
        c.metadata?.location === preferences!.location
      );
    }

    // Capabilities filter
    if (preferences.capabilities?.length) {
      filtered = filtered.filter(c =>
        preferences!.capabilities!.every(cap =>
          c.capabilities.supportedTaskTypes?.includes(cap) ||
          c.capabilities[cap as keyof ComponentCapabilities]
        )
      );
    }

    return filtered;
  }

  private async calculateScore(component: ComponentRegistration, payload: any): Promise<number> {
    let score = 100;

    // Load factor (lower load = higher score)
    const currentLoad = this.componentLoads.get(component.id) || 0;
    const loadPenalty = currentLoad * 2; // 2 points penalty per load unit
    score -= loadPenalty;

    // Health factor
    const health = this.coordinator.getComponentHealth(component.id);
    if (health?.status === HealthStatus.DEGRADED) {
      score -= 20;
    } else if (health?.status === HealthStatus.UNHEALTHY) {
      score -= 50;
    }

    // Response time factor
    if (health?.responseTime) {
      const responsePenalty = Math.min(health.responseTime / 100, 10); // Max 10 point penalty
      score -= responsePenalty;
    }

    // Capability match bonus
    if (payload?.taskType && component.capabilities.supportedTaskTypes?.includes(payload.taskType)) {
      score += 15;
    }

    // Location bonus
    if (payload?.location && component.metadata?.location === payload.location) {
      score += 10;
    }

    return Math.max(0, score);
  }

  private updateLoadTracking(componentId: string): void {
    const currentLoad = this.componentLoads.get(componentId) || 0;
    this.componentLoads.set(componentId, currentLoad + 1);

    // Decay load over time (simulate completion)
    setTimeout(() => {
      const load = this.componentLoads.get(componentId) || 0;
      if (load > 0) {
        this.componentLoads.set(componentId, load - 1);
      }
    }, 30000); // Assume 30 second average task time
  }
}
```

### 4. Failure Manager

```typescript
// src/coordinator/FailureManager.ts

export interface FailureEvent {
  componentId: string;
  failureType: FailureType;
  error: any;
  timestamp: Date;
  context?: Record<string, any>;
}

export enum FailureType {
  HEALTH_CHECK_FAILURE = "health_check_failure",
  CONNECTION_FAILURE = "connection_failure",
  TIMEOUT_FAILURE = "timeout_failure",
  INTERNAL_ERROR = "internal_error",
  DEPENDENCY_FAILURE = "dependency_failure",
}

export interface RecoveryAction {
  type: "restart" | "switchover" | "scale_up" | "alert" | "isolate";
  target: string;
  parameters?: Record<string, any>;
}

export interface FailureRecovery {
  failure: FailureEvent;
  actions: RecoveryAction[];
  status: RecoveryStatus;
  startTime: Date;
  endTime?: Date;
  success: boolean;
}

export enum RecoveryStatus {
  IN_PROGRESS = "in_progress",
  SUCCESSFUL = "successful",
  FAILED = "failed",
  TIMEOUT = "timeout",
}

export class FailureManager extends EventEmitter {
  private activeRecoveries = new Map<string, FailureRecovery>();
  private failureHistory: FailureEvent[] = [];

  constructor(
    private coordinator: SystemCoordinator,
    private config: { failureThreshold: number; recoveryTimeout: number }
  ) {
    super();
  }

  /**
   * Handle component failure
   */
  async handleFailure(componentId: string, error: any, context?: Record<string, any>): Promise<void> {
    const failure: FailureEvent = {
      componentId,
      failureType: this.classifyFailure(error),
      error,
      timestamp: new Date(),
      context,
    };

    // Record failure
    this.failureHistory.push(failure);

    // Check failure threshold
    const recentFailures = this.getRecentFailures(componentId, 300000); // 5 minutes
    if (recentFailures.length >= this.config.failureThreshold) {
      await this.initiateRecovery(componentId, failure);
    }

    this.emit("component:failed", failure);
  }

  /**
   * Initiate recovery process
   */
  private async initiateRecovery(componentId: string, failure: FailureEvent): Promise<void> {
    if (this.activeRecoveries.has(componentId)) {
      return; // Recovery already in progress
    }

    const recovery: FailureRecovery = {
      failure,
      actions: this.determineRecoveryActions(failure),
      status: RecoveryStatus.IN_PROGRESS,
      startTime: new Date(),
      success: false,
    };

    this.activeRecoveries.set(componentId, recovery);

    try {
      await this.executeRecovery(recovery);

      recovery.status = RecoveryStatus.SUCCESSFUL;
      recovery.success = true;
      recovery.endTime = new Date();

      this.emit("component:recovered", {
        componentId,
        recoveryTime: recovery.endTime.getTime() - recovery.startTime.getTime(),
        actions: recovery.actions.length,
        timestamp: new Date(),
      });

    } catch (recoveryError) {
      recovery.status = RecoveryStatus.FAILED;
      recovery.success = false;
      recovery.endTime = new Date();

      this.emit("recovery:failed", {
        componentId,
        error: recoveryError,
        timestamp: new Date(),
      });

      // Escalate to human intervention
      await this.escalateFailure(failure, recoveryError);
    } finally {
      // Clean up after timeout
      setTimeout(() => {
        this.activeRecoveries.delete(componentId);
      }, this.config.recoveryTimeout);
    }
  }

  /**
   * Determine recovery actions based on failure
   */
  private determineRecoveryActions(failure: FailureEvent): RecoveryAction[] {
    const actions: RecoveryAction[] = [];

    switch (failure.failureType) {
      case FailureType.HEALTH_CHECK_FAILURE:
        actions.push({
          type: "restart",
          target: failure.componentId,
          parameters: { reason: "health_check_failure" },
        });
        break;

      case FailureType.CONNECTION_FAILURE:
        actions.push({
          type: "switchover",
          target: failure.componentId,
          parameters: { to: "backup_instance" },
        });
        break;

      case FailureType.TIMEOUT_FAILURE:
        actions.push({
          type: "scale_up",
          target: failure.componentId,
          parameters: { instances: 1 },
        });
        break;

      case FailureType.INTERNAL_ERROR:
        actions.push(
          {
            type: "restart",
            target: failure.componentId,
            parameters: { reason: "internal_error" },
          },
          {
            type: "alert",
            target: "engineering_team",
            parameters: { priority: "high" },
          }
        );
        break;

      case FailureType.DEPENDENCY_FAILURE:
        actions.push({
          type: "isolate",
          target: failure.componentId,
          parameters: { duration: 300000 }, // 5 minutes
        });
        break;
    }

    return actions;
  }

  /**
   * Execute recovery actions
   */
  private async executeRecovery(recovery: FailureRecovery): Promise<void> {
    for (const action of recovery.actions) {
      try {
        await this.executeRecoveryAction(action);
        action.executed = true;
      } catch (error) {
        action.executed = false;
        action.error = error;
        throw error;
      }
    }
  }

  /**
   * Execute individual recovery action
   */
  private async executeRecoveryAction(action: RecoveryAction): Promise<void> {
    switch (action.type) {
      case "restart":
        await this.restartComponent(action.target, action.parameters);
        break;

      case "switchover":
        await this.switchoverComponent(action.target, action.parameters);
        break;

      case "scale_up":
        await this.scaleUpComponent(action.target, action.parameters);
        break;

      case "alert":
        await this.sendAlert(action.target, action.parameters);
        break;

      case "isolate":
        await this.isolateComponent(action.target, action.parameters);
        break;

      default:
        throw new Error(`Unknown recovery action: ${action.type}`);
    }
  }

  /**
   * Escalate failure to human intervention
   */
  private async escalateFailure(failure: FailureEvent, recoveryError: any): Promise<void> {
    // In a real implementation, this would:
    // 1. Create incident ticket
    // 2. Notify on-call engineer
    // 3. Send detailed diagnostics

    console.error(`CRITICAL: Component ${failure.componentId} failed and recovery unsuccessful`, {
      failure,
      recoveryError,
      recentFailures: this.getRecentFailures(failure.componentId, 3600000), // Last hour
    });
  }

  /**
   * Get recent failures for component
   */
  private getRecentFailures(componentId: string, timeWindowMs: number): FailureEvent[] {
    const cutoff = new Date(Date.now() - timeWindowMs);
    return this.failureHistory.filter(
      f => f.componentId === componentId && f.timestamp > cutoff
    );
  }

  /**
   * Get failure statistics
   */
  getFailureStats(): {
    totalFailures: number;
    activeRecoveries: number;
    recentFailures: number;
    byComponent: Record<string, number>;
  } {
    const byComponent: Record<string, number> = {};
    const recentCutoff = new Date(Date.now() - 3600000); // Last hour

    for (const failure of this.failureHistory) {
      byComponent[failure.componentId] = (byComponent[failure.componentId] || 0) + 1;
    }

    const recentFailures = this.failureHistory.filter(f => f.timestamp > recentCutoff).length;

    return {
      totalFailures: this.failureHistory.length,
      activeRecoveries: this.activeRecoveries.size,
      recentFailures,
      byComponent,
    };
  }

  private classifyFailure(error: any): FailureType {
    if (error.code === 'ECONNREFUSED' || error.message?.includes('connection')) {
      return FailureType.CONNECTION_FAILURE;
    }

    if (error.code === 'ETIMEDOUT' || error.message?.includes('timeout')) {
      return FailureType.TIMEOUT_FAILURE;
    }

    if (error.message?.includes('health check')) {
      return FailureType.HEALTH_CHECK_FAILURE;
    }

    if (error.message?.includes('dependency')) {
      return FailureType.DEPENDENCY_FAILURE;
    }

    return FailureType.INTERNAL_ERROR;
  }

  // Placeholder methods for recovery actions
  private async restartComponent(componentId: string, params?: any): Promise<void> {
    console.log(`Restarting component ${componentId}`, params);
    // Implementation would restart the component
  }

  private async switchoverComponent(componentId: string, params?: any): Promise<void> {
    console.log(`Switching over component ${componentId}`, params);
    // Implementation would switch to backup
  }

  private async scaleUpComponent(componentId: string, params?: any): Promise<void> {
    console.log(`Scaling up component ${componentId}`, params);
    // Implementation would scale up instances
  }

  private async sendAlert(target: string, params?: any): Promise<void> {
    console.log(`Sending alert to ${target}`, params);
    // Implementation would send alerts
  }

  private async isolateComponent(componentId: string, params?: any): Promise<void> {
    console.log(`Isolating component ${componentId}`, params);
    // Implementation would isolate component
  }
}
```

---

## Integration with Existing Components

### Registration During Startup

```typescript
// In main application startup
const coordinator = new SystemCoordinator(config, healthMonitor);

// Register ARBITER components
await coordinator.registerComponent({
  id: "arbiter-001-agent-registry",
  name: "Agent Registry",
  type: ComponentType.AGENT_REGISTRY,
  endpoint: "http://localhost:3001",
  healthCheck: {
    endpoint: "http://localhost:3001/health",
    method: "GET",
    timeout: 5000,
    interval: 30000,
    retries: 3,
  },
  capabilities: {
    maxConcurrentTasks: 100,
    supportedTaskTypes: ["agent_registration", "agent_lookup"],
    performanceMetrics: true,
    healthMonitoring: true,
  },
  dependencies: [],
  metadata: { version: "1.0.0" },
});

// Register other components similarly...
```

### Request Routing

```typescript
// In TaskOrchestrator.routeTask()
const routingComponent = await this.coordinator.routeRequest(
  "task_routing",
  {
    taskId: task.id,
    taskType: task.type,
    requirements: task.requirements,
  },
  {
    capabilities: [task.type],
    maxLoad: 80,
  }
);

// Route to selected component
return await this.callComponent(routingComponent, "route", { task });
```

### Health Monitoring Integration

```typescript
// Health checks automatically monitored
coordinator.on("component:health-changed", (event) => {
  if (event.newStatus === HealthStatus.UNHEALTHY) {
    console.warn(`Component ${event.componentId} became unhealthy`);
    // Trigger load redistribution
    await coordinator.redistributeLoad();
  }
});
```

---

## Testing Strategy

### Unit Tests

```typescript
// tests/unit/coordinator/system-coordinator.test.ts

describe("SystemCoordinator", () => {
  let coordinator: SystemCoordinator;
  let healthMonitor: ComponentHealthMonitor;

  beforeEach(() => {
    healthMonitor = new ComponentHealthMonitor();
    coordinator = new SystemCoordinator({
      healthCheckInterval: 30000,
      failureThreshold: 3,
      recoveryTimeout: 300000,
      loadBalancingEnabled: true,
      autoScalingEnabled: false,
      maxComponentsPerType: 5,
    }, healthMonitor);
  });

  describe("component registration", () => {
    it("should register component successfully", async () => {
      const registration: ComponentRegistration = {
        id: "test-component",
        name: "Test Component",
        type: ComponentType.AGENT_REGISTRY,
        endpoint: "http://localhost:3001",
        healthCheck: {
          endpoint: "http://localhost:3001/health",
          method: "GET",
          timeout: 5000,
          interval: 30000,
          retries: 3,
        },
        capabilities: {},
        dependencies: [],
        metadata: {},
      };

      await coordinator.registerComponent(registration);

      const component = coordinator.getComponent("test-component");
      expect(component).toEqual(registration);
    });

    it("should validate dependencies", async () => {
      const registration: ComponentRegistration = {
        id: "dependent-component",
        name: "Dependent Component",
        type: ComponentType.TASK_ROUTER,
        endpoint: "http://localhost:3002",
        healthCheck: {
          endpoint: "http://localhost:3002/health",
          method: "GET",
          timeout: 5000,
          interval: 30000,
          retries: 3,
        },
        capabilities: {},
        dependencies: ["non-existent-dependency"],
        metadata: {},
      };

      await expect(coordinator.registerComponent(registration))
        .rejects.toThrow("Dependency non-existent-dependency not registered");
    });
  });

  describe("request routing", () => {
    it("should route requests to healthy components", async () => {
      // Register components
      await coordinator.registerComponent(agentRegistryComponent);
      await coordinator.registerComponent(taskRouterComponent);

      const routedComponent = await coordinator.routeRequest("task_routing", {
        taskId: "task-1",
        taskType: "code-review",
      });

      expect(routedComponent.type).toBe(ComponentType.TASK_ROUTER);
    });

    it("should throw error when no healthy components available", async () => {
      await expect(coordinator.routeRequest("task_routing", {}))
        .rejects.toThrow("No healthy components available");
    });
  });

  describe("health monitoring", () => {
    it("should report system health", () => {
      const health = coordinator.getSystemHealth();

      expect(health).toHaveProperty("status");
      expect(health).toHaveProperty("components");
      expect(health).toHaveProperty("lastUpdate");
      expect(health).toHaveProperty("issues");
    });

    it("should calculate overall health status", () => {
      // Register healthy component
      coordinator.registerComponent(healthyComponent);

      const health = coordinator.getSystemHealth();
      expect(health.status).toBe(HealthStatus.HEALTHY);
    });
  });
});
```

---

## Acceptance Criteria

1. ✅ System coordinator manages component registry and lifecycle
2. ✅ Health monitoring provides real-time component status
3. ✅ Load balancer distributes requests across healthy components
4. ✅ Failure manager detects and recovers from component failures
5. ✅ Request routing selects optimal components based on load and health
6. ✅ System health dashboard provides unified view
7. ✅ Automatic load redistribution on component changes
8. ✅ All tests passing (unit + integration)
9. ✅ Sub-1ms routing decisions
10. ✅ <30 second failure detection and recovery

---

## Implementation Checklist

- [ ] Create coordinator types and interfaces
- [ ] Implement SystemCoordinator core
- [ ] Build ComponentHealthMonitor
- [ ] Implement LoadBalancer with scoring
- [ ] Create FailureManager with recovery actions
- [ ] Add component registration API
- [ ] Implement request routing logic
- [ ] Add system health aggregation
- [ ] Integrate with existing ARBITER components
- [ ] Write comprehensive unit tests
- [ ] Add integration tests
- [ ] Performance validation
- [ ] Documentation and examples

---

**Status**: Ready to implement!
