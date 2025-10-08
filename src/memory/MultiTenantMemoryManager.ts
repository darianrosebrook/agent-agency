/**
 * Multi-Tenant Memory Manager - Central Coordinator
 *
 * This is the main entry point for the multi-tenant memory system, coordinating
 * between tenant isolation, context offloading, and federated learning components.
 * It provides a unified API for memory operations while ensuring tenant security
 * and performance optimization.
 *
 * @author @darianrosebrook
 */

import type {
  ContextualMemory,
  OffloadedContext,
  ReconstructedContext,
  TaskContext,
  TenantConfig,
} from "../types/index.js";
import { Logger } from "../utils/Logger";
import { ContextOffloader } from "./ContextOffloader";
import { TenantIsolator } from "./TenantIsolator";

export interface MultiTenantMemoryConfig {
  tenantIsolation: {
    enabled: boolean;
    defaultIsolationLevel: "strict" | "shared" | "federated";
    auditLogging: boolean;
    maxTenants: number;
  };
  contextOffloading: {
    enabled: boolean;
    maxContextSize: number;
    compressionThreshold: number;
    relevanceThreshold: number;
    embeddingDimensions: number;
  };
  federatedLearning: {
    enabled: boolean;
    privacyLevel: "basic" | "differential" | "secure";
    aggregationFrequency: number;
    minParticipants: number;
  };
  performance: {
    cacheEnabled: boolean;
    cacheSize: number;
    batchProcessing: boolean;
    asyncOperations: boolean;
  };
}

export interface MemoryOperationResult<T> {
  success: boolean;
  data?: T;
  error?: string;
  tenantId: string;
  operationId: string;
  performance: {
    duration: number;
    cacheHit: boolean;
    offloaded: boolean;
  };
}

export interface FederatedInsights {
  insights: ContextualMemory[];
  confidence: number;
  sourceTenants: string[];
  aggregationMethod: "weighted" | "consensus" | "hybrid";
  privacyPreserved: boolean;
}

/**
 * MultiTenantMemoryManager - Central coordination service
 */
export class MultiTenantMemoryManager {
  private logger: Logger;
  private config: MultiTenantMemoryConfig;
  private tenantIsolator: TenantIsolator;
  private contextOffloader: ContextOffloader;
  private operationCache: Map<string, any> = new Map();
  private activeOperations: Map<string, Promise<any>> = new Map();

  constructor(config: MultiTenantMemoryConfig, logger?: Logger) {
    this.config = config;
    this.logger = logger || new Logger("MultiTenantMemoryManager");

    // Initialize core components
    this.tenantIsolator = new TenantIsolator(this.logger);
    this.contextOffloader = new ContextOffloader(
      {
        maxContextSize: config.contextOffloading.maxContextSize,
        compressionThreshold: config.contextOffloading.compressionThreshold,
        relevanceThreshold: config.contextOffloading.relevanceThreshold,
        quarantineEnabled: true,
        summarizationEnabled: true,
        temporalDecayEnabled: false,
        embeddingDimensions: config.contextOffloading.embeddingDimensions,
      },
      this.logger
    );

    this.logger.info("MultiTenantMemoryManager initialized", {
      tenantIsolation: config.tenantIsolation.enabled,
      contextOffloading: config.contextOffloading.enabled,
      federatedLearning: config.federatedLearning.enabled,
    });
  }

  /**
   * Register a new tenant in the memory system
   */
  async registerTenant(
    config: TenantConfig
  ): Promise<MemoryOperationResult<void>> {
    const operationId = this.generateOperationId(
      "register_tenant",
      config.tenantId
    );
    const startTime = Date.now();

    try {
      this.logger.debug(`Registering tenant: ${config.tenantId}`, {
        operationId,
      });

      // Validate tenant configuration
      await this.validateTenantConfig(config);

      // Register with tenant isolator
      await this.tenantIsolator.registerTenant(config);

      // Cache tenant configuration if caching enabled
      if (this.config.performance.cacheEnabled) {
        this.operationCache.set(`tenant_config_${config.tenantId}`, config);
      }

      const duration = Date.now() - startTime;
      this.logger.info(`Tenant registered successfully: ${config.tenantId}`, {
        operationId,
        duration,
        isolationLevel: config.isolationLevel,
      });

      return {
        success: true,
        tenantId: config.tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      this.logger.error(`Tenant registration failed: ${config.tenantId}`, {
        operationId,
        error: errorMessage,
        duration,
      });

      return {
        success: false,
        error: errorMessage,
        tenantId: config.tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    }
  }

  /**
   * Store agent experience with tenant isolation
   */
  async storeExperience(
    tenantId: string,
    experience: ContextualMemory,
    options: {
      offloadContext?: boolean;
      sharingLevel?: "private" | "shared" | "federated";
      priority?: "low" | "normal" | "high";
    } = {}
  ): Promise<MemoryOperationResult<string>> {
    const operationId = this.generateOperationId("store_experience", tenantId);
    const startTime = Date.now();

    try {
      this.logger.debug(`Storing experience for tenant: ${tenantId}`, {
        operationId,
        experienceId: experience.memoryId,
      });

      // Validate tenant access
      const accessCheck = await this.tenantIsolator.validateTenantAccess(
        tenantId,
        "write",
        "memory"
      );

      if (!accessCheck.allowed) {
        throw new Error(accessCheck.reason || "Access denied");
      }

      // Handle context offloading if enabled
      let offloadedContext: OffloadedContext | undefined;
      const shouldOffload =
        options.offloadContext ?? this.config.contextOffloading.enabled;

      if (shouldOffload && experience.contextMatch) {
        // Convert ContextualMemory to TaskContext
        const taskContext: TaskContext = {
          taskId: experience.memoryId,
          agentId: tenantId,
          type: "experience_storage",
          description: `Storing experience: ${experience.memoryId}`,
          requirements: [],
          constraints: {
            tenantId,
            relevanceScore: experience.relevanceScore,
          },
          historicalContext: [experience],
          metadata: {
            sharingLevel: options.sharingLevel || "private",
            priority: options.priority || "normal",
          },
        };

        offloadedContext = await this.contextOffloader.offloadContext(
          taskContext,
          tenantId
        );
        this.logger.debug(
          `Context offloaded for experience: ${experience.memoryId}`,
          {
            operationId,
            compressionRatio: offloadedContext.compressionRatio,
          }
        );
      }

      // Store the experience (placeholder - would integrate with actual storage)
      const experienceId = await this.persistExperience(
        tenantId,
        experience,
        offloadedContext
      );

      // Handle sharing if applicable
      if (options.sharingLevel && options.sharingLevel !== "private") {
        await this.handleExperienceSharing(
          tenantId,
          experience,
          options.sharingLevel
        );
      }

      // Cache operation result if enabled
      if (this.config.performance.cacheEnabled) {
        this.operationCache.set(`experience_${experienceId}`, {
          tenantId,
          experience,
          offloadedContext,
          timestamp: Date.now(),
        });
      }

      const duration = Date.now() - startTime;
      this.logger.info(`Experience stored successfully: ${experienceId}`, {
        operationId,
        tenantId,
        duration,
        offloaded: !!offloadedContext,
      });

      return {
        success: true,
        data: experienceId,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: !!offloadedContext,
        },
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      this.logger.error(`Experience storage failed for tenant: ${tenantId}`, {
        operationId,
        experienceId: experience.memoryId,
        error: errorMessage,
        duration,
      });

      return {
        success: false,
        error: errorMessage,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    }
  }

  /**
   * Retrieve contextual memories for a tenant
   */
  async getContextualMemories(
    tenantId: string,
    queryContext: TaskContext,
    options: {
      limit?: number;
      includeShared?: boolean;
      includeFederated?: boolean;
      minRelevance?: number;
      useCache?: boolean;
    } = {}
  ): Promise<MemoryOperationResult<ContextualMemory[]>> {
    const operationId = this.generateOperationId("get_memories", tenantId);
    const startTime = Date.now();

    try {
      this.logger.debug(`Retrieving memories for tenant: ${tenantId}`, {
        operationId,
        queryType: queryContext.type,
        limit: options.limit,
      });

      // Check cache first if enabled
      if (options.useCache ?? this.config.performance.cacheEnabled) {
        const cacheKey = this.generateCacheKey(
          "memories",
          tenantId,
          queryContext
        );
        const cachedResult = this.operationCache.get(cacheKey);
        if (cachedResult && this.isCacheValid(cachedResult.timestamp)) {
          this.logger.debug("Returning cached memories", {
            operationId,
            tenantId,
          });
          return {
            success: true,
            data: cachedResult.memories,
            tenantId,
            operationId,
            performance: {
              duration: Date.now() - startTime,
              cacheHit: true,
              offloaded: false,
            },
          };
        }
      }

      // Validate tenant access
      const accessCheck = await this.tenantIsolator.validateTenantAccess(
        tenantId,
        "read",
        "memory"
      );

      if (!accessCheck.allowed) {
        throw new Error(accessCheck.reason || "Access denied");
      }

      // Get tenant-specific memories
      const tenantMemories = await this.retrieveTenantMemories(
        tenantId,
        queryContext,
        options
      );

      // Get shared memories if requested
      let sharedMemories: ContextualMemory[] = [];
      if (options.includeShared) {
        sharedMemories = await this.getSharedMemories(
          tenantId,
          queryContext,
          options
        );
      }

      // Get federated insights if requested and enabled
      let federatedMemories: ContextualMemory[] = [];
      if (options.includeFederated && this.config.federatedLearning.enabled) {
        const federatedResult = await this.getFederatedInsights(
          tenantId,
          queryContext
        );
        federatedMemories = federatedResult.insights;
      }

      // Combine and rank all memories
      const allMemories = [
        ...tenantMemories,
        ...sharedMemories,
        ...federatedMemories,
      ];
      const rankedMemories = await this.rankAndFilterMemories(
        allMemories,
        queryContext,
        options.limit || 10,
        options.minRelevance || 0.1
      );

      // Enhance with offloaded context if available
      const enhancedMemories = await this.contextOffloader.enrichMemories(
        rankedMemories,
        tenantId
      );

      // Cache result if enabled
      if (this.config.performance.cacheEnabled) {
        const cacheKey = this.generateCacheKey(
          "memories",
          tenantId,
          queryContext
        );
        this.operationCache.set(cacheKey, {
          memories: enhancedMemories,
          timestamp: Date.now(),
        });
      }

      const duration = Date.now() - startTime;
      this.logger.info(
        `Retrieved ${enhancedMemories.length} memories for tenant: ${tenantId}`,
        {
          operationId,
          duration,
          cacheHit: false,
          tenantMemories: tenantMemories.length,
          sharedMemories: sharedMemories.length,
          federatedMemories: federatedMemories.length,
        }
      );

      return {
        success: true,
        data: enhancedMemories,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      this.logger.error(`Memory retrieval failed for tenant: ${tenantId}`, {
        operationId,
        error: errorMessage,
        duration,
      });

      return {
        success: false,
        error: errorMessage,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    }
  }

  /**
   * Get federated insights from multiple tenants
   */
  async getFederatedInsights(
    tenantId: string,
    context: TaskContext
  ): Promise<FederatedInsights> {
    if (!this.config.federatedLearning.enabled) {
      return {
        insights: [],
        confidence: 0,
        sourceTenants: [],
        aggregationMethod: "weighted",
        privacyPreserved: true,
      };
    }

    this.logger.debug(`Getting federated insights for tenant: ${tenantId}`, {
      contextType: context.type,
    });

    // Get participating tenants (placeholder - would query federation network)
    const participatingTenants = await this.getParticipatingTenants(tenantId);

    // Collect insights from each tenant (with privacy preservation)
    const allInsights: ContextualMemory[] = [];
    const sourceTenants: string[] = [];

    for (const participantTenant of participatingTenants) {
      try {
        // Check if sharing is allowed
        const canShare = await this.tenantIsolator.canShareWithTenant(
          participantTenant,
          tenantId,
          "memory"
        );

        if (canShare.allowed) {
          // Get anonymized insights from participant
          const participantInsights = await this.getAnonymizedInsights(
            participantTenant,
            context
          );
          allInsights.push(...participantInsights);
          sourceTenants.push(participantTenant);
        }
      } catch (error) {
        this.logger.warn(
          `Failed to get insights from tenant: ${participantTenant}`,
          {
            error: error instanceof Error ? error.message : String(error),
          }
        );
      }
    }

    // Aggregate insights with privacy preservation
    const aggregatedInsights = await this.aggregateFederatedInsights(
      allInsights,
      context,
      this.config.federatedLearning.privacyLevel
    );

    return {
      insights: aggregatedInsights,
      confidence: this.calculateFederatedConfidence(
        aggregatedInsights,
        sourceTenants
      ),
      sourceTenants,
      aggregationMethod: "weighted",
      privacyPreserved: true,
    };
  }

  /**
   * Offload context for long-term storage
   */
  async offloadContext(
    tenantId: string,
    context: TaskContext
  ): Promise<MemoryOperationResult<OffloadedContext>> {
    const operationId = this.generateOperationId("offload_context", tenantId);
    const startTime = Date.now();

    try {
      this.logger.debug(`Offloading context for tenant: ${tenantId}`, {
        operationId,
        contextId: context.taskId,
      });

      // Validate tenant access
      const accessCheck = await this.tenantIsolator.validateTenantAccess(
        tenantId,
        "write",
        "memory"
      );

      if (!accessCheck.allowed) {
        throw new Error(accessCheck.reason || "Access denied");
      }

      // Offload context
      const offloadedContext = await this.contextOffloader.offloadContext(
        context,
        tenantId
      );

      const duration = Date.now() - startTime;
      this.logger.info(
        `Context offloaded successfully: ${offloadedContext.id}`,
        {
          operationId,
          tenantId,
          duration,
          compressionRatio: offloadedContext.compressionRatio,
        }
      );

      return {
        success: true,
        data: offloadedContext,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: true,
        },
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      this.logger.error(`Context offloading failed for tenant: ${tenantId}`, {
        operationId,
        contextId: context.taskId,
        error: errorMessage,
        duration,
      });

      return {
        success: false,
        error: errorMessage,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    }
  }

  /**
   * Retrieve offloaded context
   */
  async retrieveContext(
    tenantId: string,
    contextId: string
  ): Promise<MemoryOperationResult<ReconstructedContext>> {
    const operationId = this.generateOperationId("retrieve_context", tenantId);
    const startTime = Date.now();

    try {
      this.logger.debug(`Retrieving context for tenant: ${tenantId}`, {
        operationId,
        contextId,
      });

      // Validate tenant access
      const accessCheck = await this.tenantIsolator.validateTenantAccess(
        tenantId,
        "read",
        "memory"
      );

      if (!accessCheck.allowed) {
        throw new Error(accessCheck.reason || "Access denied");
      }

      // Find relevant contexts
      const relevantContexts = await this.contextOffloader.findRelevantContexts(
        tenantId,
        {
          taskId: "retrieval",
          agentId: tenantId,
          type: "context_retrieval",
          description: `Retrieve context: ${contextId}`,
          requirements: [],
          constraints: {},
          metadata: {},
        },
        5
      );

      // Find the specific context
      const targetContext = relevantContexts.find(
        (ctx) => ctx.id === contextId
      );
      if (!targetContext) {
        throw new Error(`Context not found: ${contextId}`);
      }

      // Reconstruct context
      const reconstructed = await this.contextOffloader.retrieveContext(
        contextId,
        tenantId
      );

      const duration = Date.now() - startTime;
      this.logger.info(`Context retrieved successfully: ${contextId}`, {
        operationId,
        tenantId,
        duration,
        reconstructionMethod: reconstructed.reconstructionMethod,
      });

      return {
        success: true,
        data: reconstructed,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      this.logger.error(`Context retrieval failed for tenant: ${tenantId}`, {
        operationId,
        contextId,
        error: errorMessage,
        duration,
      });

      return {
        success: false,
        error: errorMessage,
        tenantId,
        operationId,
        performance: {
          duration,
          cacheHit: false,
          offloaded: false,
        },
      };
    }
  }

  /**
   * Get system health and performance metrics
   */
  async getSystemHealth(): Promise<{
    tenants: number;
    activeOperations: number;
    cacheSize: number;
    offloadedContexts: number;
    federatedParticipants: number;
  }> {
    const tenants = this.tenantIsolator.listTenants().length;
    const activeOperations = this.activeOperations.size;
    const cacheSize = this.operationCache.size;

    // These would be implemented with actual metrics collection
    const offloadedContexts = 0; // Placeholder
    const federatedParticipants = 0; // Placeholder

    return {
      tenants,
      activeOperations,
      cacheSize,
      offloadedContexts,
      federatedParticipants,
    };
  }

  /**
   * Clean up expired data and optimize performance
   */
  async performMaintenance(): Promise<void> {
    this.logger.info("Starting maintenance operations");

    // Clean up expired cache entries
    if (this.config.performance.cacheEnabled) {
      const expiredKeys: string[] = [];
      const now = Date.now();
      const maxAge = 24 * 60 * 60 * 1000; // 24 hours

      for (const [key, value] of this.operationCache.entries()) {
        if (value.timestamp && now - value.timestamp > maxAge) {
          expiredKeys.push(key);
        }
      }

      expiredKeys.forEach((key) => this.operationCache.delete(key));
      this.logger.info(
        `Cleaned up ${expiredKeys.length} expired cache entries`
      );
    }

    // Clean up offloaded contexts for all tenants
    const tenants = this.tenantIsolator.listTenants();
    for (const tenantId of tenants) {
      try {
        const cleaned = await this.contextOffloader.cleanupContexts(tenantId);
        if (cleaned > 0) {
          this.logger.info(
            `Cleaned up ${cleaned} contexts for tenant: ${tenantId}`
          );
        }
      } catch (error) {
        this.logger.warn(`Failed to cleanup contexts for tenant: ${tenantId}`, {
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    this.logger.info("Maintenance operations completed");
  }

  // Private helper methods

  private async validateTenantConfig(config: TenantConfig): Promise<void> {
    if (!config.tenantId || !config.projectId) {
      throw new Error("Tenant ID and Project ID are required");
    }

    const maxTenants = this.config.tenantIsolation.maxTenants;
    if (
      maxTenants > 0 &&
      this.tenantIsolator.listTenants().length >= maxTenants
    ) {
      throw new Error(`Maximum tenant limit reached: ${maxTenants}`);
    }

    // Validate isolation level compatibility
    if (!["strict", "shared", "federated"].includes(config.isolationLevel)) {
      throw new Error("Invalid isolation level");
    }

    if (
      config.isolationLevel === "federated" &&
      !this.config.federatedLearning.enabled
    ) {
      throw new Error(
        "Federated learning must be enabled for federated isolation level"
      );
    }
  }

  private generateOperationId(operation: string, tenantId: string): string {
    return `${operation}_${tenantId}_${Date.now()}_${Math.random()
      .toString(36)
      .substr(2, 9)}`;
  }

  private generateCacheKey(
    operation: string,
    tenantId: string,
    context: TaskContext
  ): string {
    return `${operation}_${tenantId}_${context.taskId}_${context.type}`;
  }

  private isCacheValid(timestamp: number): boolean {
    const maxAge = 60 * 60 * 1000; // 1 hour
    return Date.now() - timestamp < maxAge;
  }

  private async persistExperience(
    tenantId: string,
    experience: ContextualMemory,
    offloadedContext?: OffloadedContext
  ): Promise<string> {
    // Placeholder - would integrate with actual persistence layer
    // For now, just return a generated ID
    return `exp_${tenantId}_${experience.memoryId}_${Date.now()}`;
  }

  private async handleExperienceSharing(
    tenantId: string,
    experience: ContextualMemory,
    sharingLevel: string
  ): Promise<void> {
    // Placeholder - would implement sharing logic
    this.logger.debug(`Handling experience sharing: ${sharingLevel}`, {
      tenantId,
      experienceId: experience.memoryId,
    });
  }

  private async retrieveTenantMemories(
    tenantId: string,
    queryContext: TaskContext,
    options: any
  ): Promise<ContextualMemory[]> {
    // Placeholder - would integrate with actual storage layer
    // Return empty array for now
    return [];
  }

  private async getSharedMemories(
    tenantId: string,
    queryContext: TaskContext,
    options: any
  ): Promise<ContextualMemory[]> {
    // Placeholder - would implement shared memory retrieval
    return [];
  }

  private async getParticipatingTenants(tenantId: string): Promise<string[]> {
    // Placeholder - would query federation network
    const allTenants = this.tenantIsolator.listTenants();
    return allTenants.filter((t) => t !== tenantId).slice(0, 5); // Max 5 participants
  }

  private async getAnonymizedInsights(
    tenantId: string,
    context: TaskContext
  ): Promise<ContextualMemory[]> {
    // Placeholder - would get anonymized insights from tenant
    // Return empty array for now
    return [];
  }

  private async aggregateFederatedInsights(
    insights: ContextualMemory[],
    context: TaskContext,
    privacyLevel: string
  ): Promise<ContextualMemory[]> {
    // Placeholder - would aggregate insights based on privacy level
    // For now, return top insights
    return insights.slice(0, 5);
  }

  private calculateFederatedConfidence(
    insights: ContextualMemory[],
    sourceTenants: string[]
  ): number {
    if (insights.length === 0 || sourceTenants.length === 0) return 0;

    // Simple confidence calculation based on number of sources
    const baseConfidence = Math.min(sourceTenants.length / 5, 1.0); // Max at 5 sources
    const averageRelevance =
      insights.reduce((sum, i) => sum + i.relevanceScore, 0) / insights.length;

    return (baseConfidence + averageRelevance) / 2;
  }

  private async rankAndFilterMemories(
    memories: ContextualMemory[],
    queryContext: TaskContext,
    limit: number,
    minRelevance: number
  ): Promise<ContextualMemory[]> {
    // Filter by minimum relevance
    const filtered = memories.filter((m) => m.relevanceScore >= minRelevance);

    // Sort by relevance score (descending)
    const sorted = filtered.sort((a, b) => b.relevanceScore - a.relevanceScore);

    // Return top results
    return sorted.slice(0, limit);
  }
}
