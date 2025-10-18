/**
 * Registry Provider Factory
 *
 * @author @darianrosebrook
 * @module orchestrator/RegistryProvider
 *
 * Factory for creating and initializing agent registries with seeding support.
 * Provides idempotent initialization and event emission for registry readiness.
 */

import type {
  AgentProfile,
  AgentRegistry,
  AgentRegistryConfig,
  RegistryInitOptions,
} from "../types/agent-registry.js";
import { AgentRegistryManager } from "./AgentRegistryManager.js";

/**
 * Registry provider factory that creates and initializes agent registries.
 * Handles seeding, idempotent initialization, and readiness events.
 */
export class RegistryProvider {
  private eventListeners: Map<string, (() => void)[]> = new Map();

  private static readonly DEFAULT_CONFIG: AgentRegistryConfig = {
    maxAgents: 1000,
    staleAgentThresholdMs: 24 * 60 * 60 * 1000, // 24 hours
    enableAutoCleanup: true,
    cleanupIntervalMs: 60 * 60 * 1000, // 1 hour
    enablePersistence: false, // Disabled by default for backward compatibility
    enableSecurity: false, // Disabled for runtime initialization
  };

  /**
   * Create and initialize an agent registry with optional seeding.
   *
   * @param options - Registry configuration and initialization options
   * @param provider - Provider instance for event emission (internal)
   * @returns Promise resolving to initialized registry
   */
  static async createAgentRegistry(
    options: {
      config?: Partial<AgentRegistryConfig>;
      initOptions?: RegistryInitOptions;
    } = {},
    provider?: RegistryProvider
  ): Promise<AgentRegistry> {
    const { config = {}, initOptions = {} } = options;

    // Merge with defaults
    const registryConfig: AgentRegistryConfig = {
      ...RegistryProvider.DEFAULT_CONFIG,
      ...config,
    } as AgentRegistryConfig;

    // Create registry manager
    const registry = new AgentRegistryManager(registryConfig);

    // Initialize with seeding if provided
    if (initOptions.seeds && initOptions.seeds.length > 0) {
      await RegistryProvider.seedRegistry(registry, initOptions);
    }

    // Initialize the registry
    await registry.initialize();

    // Emit ready event if requested
    if (initOptions.emitReady !== false && provider) {
      // Small delay to ensure initialization is complete
      setImmediate(() => {
        provider.emit("registry.ready", {
          timestamp: new Date().toISOString(),
          agentCount: initOptions.seeds?.length || 0,
        });
      });
    }

    return registry as AgentRegistry;
  }

  /**
   * Seed the registry with initial agent profiles.
   * Handles idempotent seeding to avoid duplicates.
   *
   * @param registry - Registry to seed
   * @param options - Seeding options
   */
  private static async seedRegistry(
    registry: AgentRegistryManager,
    options: RegistryInitOptions
  ): Promise<void> {
    const { seeds = [], mode = "idempotent" } = options;

    if (seeds.length === 0) {
      return;
    }

    console.log(`Seeding registry with ${seeds.length} agents (mode: ${mode})`);

    for (const seed of seeds) {
      try {
        // Complete the partial profile with defaults
        const profile = RegistryProvider.completeAgentProfile(seed);

        await registry.registerAgent(profile);
        console.log(`Registered agent: ${profile.id} (${profile.name})`);
      } catch (error) {
        if (
          mode === "idempotent" &&
          error instanceof Error &&
          error.message.includes("already exists")
        ) {
          console.log(
            `Agent ${seed.id} already exists, skipping (idempotent mode)`
          );
          continue;
        }

        console.error(`Failed to register agent ${seed.id}:`, error);

        if (mode === "strict") {
          throw error;
        }
      }
    }

    console.log(`Registry seeding completed`);
  }

  /**
   * Complete a partial agent profile with default values.
   *
   * @param partial - Partial agent profile from seed data
   * @returns Complete agent profile
   */
  private static completeAgentProfile(
    partial: Partial<AgentProfile>
  ): AgentProfile {
    const now = new Date().toISOString();

    return {
      id: partial.id!,
      name: partial.name || `Agent ${partial.id}`,
      modelFamily: partial.modelFamily || "gpt-4",
      capabilities: {
        taskTypes: partial.capabilities?.taskTypes || ["file_editing"],
        languages: partial.capabilities?.languages || ["TypeScript"],
        specializations: partial.capabilities?.specializations || [],
      },
      performanceHistory: {
        successRate: partial.performanceHistory?.successRate || 0.85,
        averageQuality: partial.performanceHistory?.averageQuality || 0.8,
        averageLatency: partial.performanceHistory?.averageLatency || 5000,
        taskCount: partial.performanceHistory?.taskCount || 0,
      },
      currentLoad: {
        activeTasks: partial.currentLoad?.activeTasks || 0,
        queuedTasks: partial.currentLoad?.queuedTasks || 0,
        utilizationPercent: partial.currentLoad?.utilizationPercent || 0,
      },
      registeredAt: partial.registeredAt || now,
      lastActiveAt: partial.lastActiveAt || now,
    };
  }

  /**
   * Create a registry provider instance for dependency injection.
   *
   * @param config - Registry configuration
   * @returns Registry provider instance
   */
  static createProvider(
    config?: Partial<AgentRegistryConfig>
  ): RegistryProvider {
    return new RegistryProvider(config);
  }

  /**
   * Listen for a single occurrence of an event.
   *
   * @param eventName - Name of the event to listen for
   * @param callback - Callback to invoke when event occurs
   */
  once(eventName: string, callback: () => void): void {
    if (!this.eventListeners.has(eventName)) {
      this.eventListeners.set(eventName, []);
    }
    this.eventListeners.get(eventName)!.push(callback);
  }

  /**
   * Emit an event to all registered listeners.
   *
   * @param eventName - Name of the event
   * @param data - Event data (currently unused)
   */
  private emit(eventName: string, _data?: any): void {
    const listeners = this.eventListeners.get(eventName);
    if (listeners) {
      listeners.forEach((callback) => callback());
      // Clear listeners after emitting (once behavior)
      this.eventListeners.delete(eventName);
    }
  }

  private constructor(private _config?: Partial<AgentRegistryConfig>) {}

  /**
   * Create a registry with this provider's configuration.
   *
   * @param initOptions - Initialization options
   * @returns Promise resolving to initialized registry
   */
  async createRegistry(
    initOptions?: RegistryInitOptions
  ): Promise<AgentRegistry> {
    return RegistryProvider.createAgentRegistry(
      {
        config: this._config,
        initOptions,
      },
      this
    );
  }
}
