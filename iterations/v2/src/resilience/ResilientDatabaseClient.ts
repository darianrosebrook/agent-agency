/**
 * @fileoverview Resilient Database Client Wrapper
 *
 * Wraps AgentRegistryDatabaseClient with circuit breaker and retry logic
 * for production reliability. Provides graceful degradation to in-memory fallback.
 *
 * @author @darianrosebrook
 */

import { AgentRegistryDatabaseClient } from "../database/AgentRegistryDatabaseClient";
import { AgentRegistryManager } from "../orchestrator/AgentRegistryManager";
import {
  AgentId,
  AgentProfile,
  AgentQuery,
  PerformanceMetrics,
  RegistryStats,
} from "../types/agent-registry";
import { CircuitBreaker, CircuitState } from "./CircuitBreaker";
import { RetryPolicies } from "./RetryPolicy";

export interface ResilientClientConfig {
  /** Use in-memory fallback when database is unavailable */
  enableFallback: boolean;

  /** Circuit breaker config */
  circuitBreaker: {
    failureThreshold: number;
    failureWindowMs: number;
    resetTimeoutMs: number;
    successThreshold: number;
  };

  /** Enable retry logic */
  enableRetry: boolean;
}

/**
 * Resilient wrapper for AgentRegistryDatabaseClient
 *
 * Provides:
 * - Circuit breaker to prevent cascading failures
 * - Retry logic with exponential backoff
 * - Graceful degradation to in-memory storage
 */
export class ResilientDatabaseClient {
  private circuitBreaker: CircuitBreaker;
  private retryPolicy = RetryPolicies.database();
  private fallbackRegistry?: AgentRegistryManager;
  private usingFallback = false;

  constructor(
    private databaseClient: AgentRegistryDatabaseClient,
    private config: ResilientClientConfig = {
      enableFallback: true,
      circuitBreaker: {
        failureThreshold: 5,
        failureWindowMs: 60000, // 1 minute
        resetTimeoutMs: 30000, // 30 seconds
        successThreshold: 2,
      },
      enableRetry: true,
    },
    fallbackRegistry?: AgentRegistryManager
  ) {
    this.circuitBreaker = new CircuitBreaker({
      name: "AgentRegistryDatabase",
      ...config.circuitBreaker,
    });

    // Initialize fallback registry if enabled
    if (config.enableFallback) {
      this.fallbackRegistry =
        fallbackRegistry ||
        new AgentRegistryManager({
          maxAgents: 10000,
          staleAgentThresholdMs: 3600000,
        });
    }
  }

  /**
   * Initialize database connection
   */
  async initialize(): Promise<void> {
    try {
      await this.executeWithResilience(() => this.databaseClient.initialize());
      console.log("[ResilientDatabaseClient] Database connection established");
    } catch (error) {
      if (this.config.enableFallback && this.fallbackRegistry) {
        console.warn(
          "[ResilientDatabaseClient] Database unavailable, using in-memory fallback"
        );
        this.usingFallback = true;
      } else {
        throw error;
      }
    }
  }

  /**
   * Register agent
   */
  async registerAgent(agent: AgentProfile): Promise<AgentProfile> {
    if (this.usingFallback && this.fallbackRegistry) {
      return this.fallbackRegistry.registerAgent(agent);
    }

    await this.executeWithResilience(() =>
      this.databaseClient.registerAgent(agent)
    );
    return agent; // Database client doesn't return the agent, so return the input
  }

  /**
   * Get agent by ID
   */
  async getAgent(agentId: AgentId): Promise<AgentProfile | null> {
    if (this.usingFallback && this.fallbackRegistry) {
      return this.fallbackRegistry.getProfile(agentId);
    }

    return this.executeWithResilience(() =>
      this.databaseClient.getAgent(agentId)
    );
  }

  /**
   * Query agents by capability
   */
  async queryAgentsByCapability(query: AgentQuery): Promise<AgentProfile[]> {
    if (this.usingFallback && this.fallbackRegistry) {
      const results = await this.fallbackRegistry.getAgentsByCapability(query);
      return results.map((r) => r.agent);
    }

    return this.executeWithResilience(() =>
      this.databaseClient.queryAgentsByCapability(query)
    );
  }

  /**
   * Update agent performance
   */
  async updatePerformance(
    agentId: AgentId,
    metrics: PerformanceMetrics
  ): Promise<AgentProfile> {
    if (this.usingFallback && this.fallbackRegistry) {
      return this.fallbackRegistry.updatePerformance(agentId, metrics);
    }

    await this.executeWithResilience(() =>
      this.databaseClient.updatePerformance(agentId, metrics)
    );
    // We need to get the updated agent - this is a limitation of the current design
    return this.getAgent(agentId).then(
      (agent) =>
        agent || {
          id: agentId,
          name: "Unknown",
          modelFamily: "unknown" as any,
          capabilities: { taskTypes: [], languages: [], specializations: [] },
          performanceHistory: {
            successRate: 0,
            averageQuality: 0,
            averageLatency: 0,
            taskCount: 0,
          },
          currentLoad: {
            activeTasks: 0,
            queuedTasks: 0,
            utilizationPercent: 0,
          },
          registeredAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
        }
    );
  }

  /**
   * Unregister agent
   */
  async unregisterAgent(agentId: AgentId): Promise<boolean> {
    if (this.usingFallback && this.fallbackRegistry) {
      return this.fallbackRegistry.unregisterAgent(agentId);
    }

    return this.executeWithResilience(() =>
      this.databaseClient.unregisterAgent(agentId)
    );
  }

  /**
   * Get registry statistics
   */
  async getStats(): Promise<RegistryStats> {
    if (this.usingFallback && this.fallbackRegistry) {
      return this.fallbackRegistry.getStats
        ? this.fallbackRegistry.getStats()
        : {
            totalAgents: 0,
            activeAgents: 0,
            idleAgents: 0,
            averageUtilization: 0,
            averageSuccessRate: 0,
            lastUpdated: new Date().toISOString(),
          };
    }

    return this.executeWithResilience(() => this.databaseClient.getStats());
  }

  /**
   * Health check
   */
  async healthCheck(): Promise<boolean> {
    try {
      if (this.usingFallback) {
        return true; // Fallback is always "healthy"
      }

      const healthResult = await this.circuitBreaker.execute(() =>
        this.databaseClient.healthCheck()
      );

      // If circuit was open and health check passed, try to recover
      const healthy = typeof healthResult === "boolean" ? healthResult : true;
      if (healthy && this.usingFallback) {
        await this.attemptRecovery();
      }

      return healthy;
    } catch {
      return false;
    }
  }

  /**
   * Shutdown
   */
  async shutdown(): Promise<void> {
    // Shutdown the database client if it has a shutdown method
    if (
      this.databaseClient &&
      typeof (this.databaseClient as any).shutdown === "function"
    ) {
      await (this.databaseClient as any).shutdown();
    }
  }

  /**
   * Get resilience status
   */
  getStatus(): {
    circuitState: CircuitState;
    usingFallback: boolean;
    circuitStats: ReturnType<CircuitBreaker["getStats"]>;
  } {
    return {
      circuitState: this.circuitBreaker.getState(),
      usingFallback: this.usingFallback,
      circuitStats: this.circuitBreaker.getStats(),
    };
  }

  /**
   * Execute operation with circuit breaker and retry
   */
  private async executeWithResilience<T>(fn: () => Promise<T>): Promise<T> {
    try {
      if (this.config.enableRetry) {
        // Wrap with both circuit breaker and retry
        return await this.circuitBreaker.execute(() =>
          this.retryPolicy.execute(fn)
        );
      } else {
        // Just circuit breaker
        return await this.circuitBreaker.execute(fn);
      }
    } catch (error) {
      // If circuit opened and fallback enabled, switch to fallback
      if (
        this.config.enableFallback &&
        this.fallbackRegistry &&
        !this.usingFallback
      ) {
        console.warn(
          "[ResilientDatabaseClient] Switching to fallback mode due to errors"
        );
        this.usingFallback = true;
      }

      throw error;
    }
  }

  /**
   * Attempt to recover from fallback mode
   */
  private async attemptRecovery(): Promise<void> {
    console.log(
      "[ResilientDatabaseClient] Attempting recovery from fallback mode..."
    );

    try {
      // Test database connection
      const healthy = await this.databaseClient.healthCheck();

      if (healthy) {
        console.log(
          "[ResilientDatabaseClient] Database recovered, switching back from fallback"
        );
        this.usingFallback = false;
        this.circuitBreaker.reset();

        // TODO: Sync fallback data to database if needed
        // This would require tracking writes that happened during fallback
      }
    } catch (error) {
      console.warn(
        "[ResilientDatabaseClient] Recovery attempt failed, staying in fallback mode"
      );
    }
  }

  /**
   * Manual reset of circuit breaker
   */
  resetCircuitBreaker(): void {
    this.circuitBreaker.reset();
    console.log("[ResilientDatabaseClient] Circuit breaker manually reset");
  }
}
