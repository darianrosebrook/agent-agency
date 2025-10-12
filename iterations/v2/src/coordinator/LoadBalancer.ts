/**
 * Load Balancer
 *
 * Distributes requests across healthy components based on load,
 * capabilities, and performance metrics.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import {
  ComponentRegistration,
  RoutingPreferences,
  LoadDistribution,
  ComponentHealth,
  HealthStatus,
} from "../types/coordinator";

import { SystemCoordinator } from "./SystemCoordinator";

export class LoadBalancer extends EventEmitter {
  private loadDistribution = new Map<string, LoadDistribution>();
  private componentLoads = new Map<string, number>();
  private requestHistory: Array<{
    componentId: string;
    timestamp: number;
    responseTime: number;
  }> = [];

  constructor(private coordinator: SystemCoordinator) {
    super();
  }

  /**
   * Select best component for request based on load and preferences
   */
  async selectComponent(
    candidates: ComponentRegistration[],
    payload: any,
    preferences?: RoutingPreferences
  ): Promise<ComponentRegistration> {
    if (candidates.length === 1) {
      this.trackRequest(candidates[0].id);
      return candidates[0];
    }

    // Apply preferences first
    let filteredCandidates = this.applyPreferences(candidates, preferences);

    if (filteredCandidates.length === 0) {
      // Fallback to all candidates if preferences filtered everything
      filteredCandidates = candidates;
    }

    if (filteredCandidates.length === 1) {
      this.trackRequest(filteredCandidates[0].id);
      return filteredCandidates[0];
    }

    // Score candidates based on load, health, and capabilities
    const scoredCandidates = await Promise.all(
      filteredCandidates.map(async (candidate) => ({
        component: candidate,
        score: await this.calculateScore(candidate, payload),
      }))
    );

    // Sort by score (highest first)
    scoredCandidates.sort((a, b) => b.score - a.score);

    const selected = scoredCandidates[0].component;

    // Track the request
    this.trackRequest(selected.id);

    // Update load tracking
    this.updateLoadTracking(selected.id);

    this.emit("component:selected", {
      componentId: selected.id,
      requestType: payload?.type || 'unknown',
      score: scoredCandidates[0].score,
      totalCandidates: candidates.length,
      filteredCandidates: filteredCandidates.length,
      timestamp: new Date(),
    });

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

    if (healthyComponents.length === 0) {
      this.emit("load:redistribution-failed", {
        reason: "no_healthy_components",
        timestamp: new Date(),
      });
      return;
    }

    // Calculate equal distribution
    const loadPerComponent = 100 / healthyComponents.length;

    // Clear existing distribution
    this.loadDistribution.clear();

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
      loadPerComponent,
      timestamp: new Date(),
    });
  }

  /**
   * Update component health for load balancing decisions
   */
  async updateComponentHealth(componentId: string, health: ComponentHealth): Promise<void> {
    if (health.status !== HealthStatus.HEALTHY) {
      // Reduce load on unhealthy components
      const distribution = this.loadDistribution.get(componentId);
      if (distribution) {
        distribution.loadPercentage *= 0.5; // Reduce to 50%
      }
    }

    this.emit("component:health-updated", {
      componentId,
      status: health.status,
      loadPercentage: this.loadDistribution.get(componentId)?.loadPercentage || 0,
      timestamp: new Date(),
    });
  }

  /**
   * Get current load distribution
   */
  getLoadDistribution(): LoadDistribution[] {
    return Array.from(this.loadDistribution.values());
  }

  /**
   * Get load statistics
   */
  getLoadStats(): {
    totalRequests: number;
    averageResponseTime: number;
    requestsPerComponent: Record<string, number>;
    loadDistribution: LoadDistribution[];
  } {
    const requestsPerComponent: Record<string, number> = {};

    // Count requests per component (last 5 minutes)
    const fiveMinutesAgo = Date.now() - 300000;
    for (const request of this.requestHistory) {
      if (request.timestamp > fiveMinutesAgo) {
        requestsPerComponent[request.componentId] = (requestsPerComponent[request.componentId] || 0) + 1;
      }
    }

    // Calculate average response time
    const recentRequests = this.requestHistory.filter(r => r.timestamp > fiveMinutesAgo);
    const averageResponseTime = recentRequests.length > 0
      ? recentRequests.reduce((sum, r) => sum + r.responseTime, 0) / recentRequests.length
      : 0;

    return {
      totalRequests: recentRequests.length,
      averageResponseTime,
      requestsPerComponent,
      loadDistribution: this.getLoadDistribution(),
    };
  }

  /**
   * Apply routing preferences to filter candidates
   */
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
        const currentLoad = this.componentLoads.get(c.id) || 0;
        return currentLoad < preferences!.maxLoad!;
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
          (c.capabilities as any)[cap] === true
        )
      );
    }

    return filtered;
  }

  /**
   * Calculate scoring for component selection
   */
  private async calculateScore(component: ComponentRegistration, payload: any): Promise<number> {
    let score = 100;

    // Load factor (lower load = higher score)
    const currentLoad = this.componentLoads.get(component.id) || 0;
    const loadPenalty = Math.min(currentLoad * 2, 40); // Max 40 point penalty
    score -= loadPenalty;

    // Health factor
    const health = this.coordinator.getComponentHealth(component.id);
    if (health?.status === HealthStatus.DEGRADED) {
      score -= 20;
    } else if (health?.status === HealthStatus.UNHEALTHY) {
      score -= 50;
    }

    // Response time factor (based on recent performance)
    const recentResponseTime = this.getAverageResponseTime(component.id, 300000); // 5 minutes
    if (recentResponseTime > 0) {
      const responsePenalty = Math.min(recentResponseTime / 100, 15); // Max 15 point penalty
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

    // Concurrent capacity bonus
    if (component.capabilities.maxConcurrentTasks) {
      const utilization = currentLoad / component.capabilities.maxConcurrentTasks;
      if (utilization < 0.8) { // Less than 80% utilized
        score += 5;
      }
    }

    return Math.max(0, score);
  }

  /**
   * Track request for analytics
   */
  private trackRequest(componentId: string, responseTime: number = 0): void {
    this.requestHistory.push({
      componentId,
      timestamp: Date.now(),
      responseTime,
    });

    // Keep only last 1000 requests
    if (this.requestHistory.length > 1000) {
      this.requestHistory = this.requestHistory.slice(-1000);
    }
  }

  /**
   * Update load tracking for component
   */
  private updateLoadTracking(componentId: string): void {
    const currentLoad = this.componentLoads.get(componentId) || 0;
    this.componentLoads.set(componentId, currentLoad + 1);

    // Update distribution
    const distribution = this.loadDistribution.get(componentId);
    if (distribution) {
      distribution.activeConnections++;
    }

    // Decay load over time (simulate completion)
    setTimeout(() => {
      const load = this.componentLoads.get(componentId) || 0;
      if (load > 0) {
        this.componentLoads.set(componentId, load - 1);

        const distribution = this.loadDistribution.get(componentId);
        if (distribution && distribution.activeConnections > 0) {
          distribution.activeConnections--;
        }
      }
    }, 30000); // Assume 30 second average task time
  }

  /**
   * Get average response time for component over time window
   */
  private getAverageResponseTime(componentId: string, timeWindowMs: number): number {
    const cutoff = Date.now() - timeWindowMs;
    const componentRequests = this.requestHistory.filter(
      r => r.componentId === componentId && r.timestamp > cutoff && r.responseTime > 0
    );

    if (componentRequests.length === 0) return 0;

    return componentRequests.reduce((sum, r) => sum + r.responseTime, 0) / componentRequests.length;
  }
}
