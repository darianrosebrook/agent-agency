/**
 * @fileoverview
 * Model hot-swap mechanism without retraining.
 * Enables dynamic model replacement while preserving system learnings.
 *
 * Key Design Principles:
 * 1. System knowledge (routing, performance) is separate from model
 * 2. Models are interchangeable plugins
 * 3. Learnings are preserved across swaps
 * 4. Zero-downtime swaps with fallback
 * 5. Compatibility validation before swap
 *
 * @author @darianrosebrook
 */

import type {
  CompatibilityResult,
  HotSwapConfig,
  LocalModelConfig,
  ModelSelectionCriteria,
  SwapEvent,
} from "@/types/model-registry";
import type { ComputeCostTracker } from "./ComputeCostTracker";
import type { LocalModelSelector } from "./LocalModelSelector";
import type { ModelRegistry } from "./ModelRegistry";
import type { LocalModelProvider } from "./providers/LocalModelProvider";

/**
 * Hot-swap error
 */
export class HotSwapError extends Error {
  constructor(message: string, public _code: string) {
    super(message);
    this.name = "HotSwapError";
  }
}

/**
 * Learning preservation layer
 *
 * Stores system knowledge independent of specific models:
 * - Task type → performance patterns
 * - Task type → optimal model characteristics
 * - Task type → fallback strategies
 */
export class LearningPreservationLayer {
  private taskPerformance: Map<
    string,
    {
      avgLatencyMs: number;
      avgQuality: number;
      avgMemoryMB: number;
      successRate: number;
      samples: number;
      lastUpdated: Date;
    }
  > = new Map();

  private taskCharacteristics: Map<
    string,
    {
      preferFast: boolean;
      preferQuality: boolean;
      preferLowMemory: boolean;
      complexity: "low" | "medium" | "high";
    }
  > = new Map();

  /**
   * Record task performance (model-agnostic)
   *
   * @param taskType Task type
   * @param metrics Performance metrics
   */
  recordTaskPerformance(
    taskType: string,
    metrics: {
      latencyMs: number;
      quality: number;
      memoryMB: number;
      success: boolean;
    }
  ): void {
    const existing = this.taskPerformance.get(taskType);

    if (!existing) {
      this.taskPerformance.set(taskType, {
        avgLatencyMs: metrics.latencyMs,
        avgQuality: metrics.quality,
        avgMemoryMB: metrics.memoryMB,
        successRate: metrics.success ? 1 : 0,
        samples: 1,
        lastUpdated: new Date(),
      });
    } else {
      const newSamples = existing.samples + 1;
      const alpha = 1 / newSamples;

      this.taskPerformance.set(taskType, {
        avgLatencyMs:
          existing.avgLatencyMs * (1 - alpha) + metrics.latencyMs * alpha,
        avgQuality: existing.avgQuality * (1 - alpha) + metrics.quality * alpha,
        avgMemoryMB:
          existing.avgMemoryMB * (1 - alpha) + metrics.memoryMB * alpha,
        successRate:
          existing.successRate * (1 - alpha) +
          (metrics.success ? 1 : 0) * alpha,
        samples: newSamples,
        lastUpdated: new Date(),
      });
    }
  }

  /**
   * Get task performance patterns
   *
   * @param taskType Task type
   * @returns Performance patterns or undefined
   */
  getTaskPerformance(taskType: string) {
    return this.taskPerformance.get(taskType);
  }

  /**
   * Learn task characteristics
   *
   * @param taskType Task type
   * @param characteristics Task characteristics
   */
  learnTaskCharacteristics(
    taskType: string,
    characteristics: {
      preferFast?: boolean;
      preferQuality?: boolean;
      preferLowMemory?: boolean;
      complexity?: "low" | "medium" | "high";
    }
  ): void {
    const existing = this.taskCharacteristics.get(taskType);

    this.taskCharacteristics.set(taskType, {
      preferFast: characteristics.preferFast ?? existing?.preferFast ?? false,
      preferQuality:
        characteristics.preferQuality ?? existing?.preferQuality ?? false,
      preferLowMemory:
        characteristics.preferLowMemory ?? existing?.preferLowMemory ?? false,
      complexity:
        characteristics.complexity ?? existing?.complexity ?? "medium",
    });
  }

  /**
   * Get task characteristics
   *
   * @param taskType Task type
   * @returns Task characteristics or undefined
   */
  getTaskCharacteristics(taskType: string) {
    return this.taskCharacteristics.get(taskType);
  }

  /**
   * Transfer learnings to selection criteria
   *
   * This is how we preserve learnings across model swaps:
   * System knowledge informs selection, not specific model history
   *
   * @param taskType Task type
   * @param baseCriteria Base selection criteria
   * @returns Enhanced criteria with learnings
   */
  enhanceCriteriaWithLearnings(
    taskType: string,
    baseCriteria: ModelSelectionCriteria
  ): ModelSelectionCriteria {
    const performance = this.getTaskPerformance(taskType);
    const characteristics = this.getTaskCharacteristics(taskType);

    const enhanced = { ...baseCriteria };

    // Apply learned performance patterns
    if (performance) {
      // Tighten latency if task has been consistently fast
      if (performance.avgLatencyMs < baseCriteria.maxLatencyMs * 0.7) {
        enhanced.maxLatencyMs = performance.avgLatencyMs * 1.2;
      }

      // Increase quality threshold if task has achieved high quality
      if (performance.avgQuality > baseCriteria.qualityThreshold) {
        enhanced.qualityThreshold = performance.avgQuality * 0.95;
      }

      // Optimize memory if task has been memory-efficient
      if (performance.avgMemoryMB < baseCriteria.maxMemoryMB * 0.7) {
        enhanced.maxMemoryMB = performance.avgMemoryMB * 1.2;
      }
    }

    // Apply learned characteristics
    if (characteristics) {
      enhanced.preferences = {
        ...baseCriteria.preferences,
        preferFast: characteristics.preferFast,
        preferQuality: characteristics.preferQuality,
        preferLowMemory: characteristics.preferLowMemory,
      };
    }

    return enhanced;
  }

  /**
   * Clear all learnings (for testing)
   */
  clearLearnings(): void {
    this.taskPerformance.clear();
    this.taskCharacteristics.clear();
  }
}

/**
 * Model hot-swap manager
 *
 * Coordinates zero-downtime model swaps with learning preservation
 */
export class ModelHotSwapManager {
  private activeProviders: Map<string, LocalModelProvider> = new Map();
  private swapHistory: SwapEvent[] = [];
  private learningLayer: LearningPreservationLayer;

  constructor(
    private registry: ModelRegistry,
    private selector: LocalModelSelector,
    private costTracker: ComputeCostTracker,
    private config: HotSwapConfig = {
      enableAutoSwap: true,
      swapCooldownMs: 300000, // 5 minutes
      minSamplesBeforeSwap: 10,
      performanceThreshold: 0.8,
      compatibilityCheckStrict: true,
    }
  ) {
    this.learningLayer = new LearningPreservationLayer();
  }

  /**
   * Register an active provider
   *
   * @param modelId Model ID
   * @param provider Provider instance
   */
  registerProvider(modelId: string, provider: LocalModelProvider): void {
    this.activeProviders.set(modelId, provider);
  }

  /**
   * Get active provider
   *
   * @param modelId Model ID
   * @returns Provider or undefined
   */
  getProvider(modelId: string): LocalModelProvider | undefined {
    return this.activeProviders.get(modelId);
  }

  /**
   * Hot-swap to new model
   *
   * Key features:
   * - Zero downtime (new model warmed up before swap)
   * - Compatibility validation
   * - Learning preservation
   * - Rollback capability
   * - Event tracking
   *
   * @param currentModelId Current model ID
   * @param newModelId New model ID
   * @param taskType Task type for context
   * @returns Swap success status
   */
  async hotSwap(
    currentModelId: string,
    newModelId: string,
    taskType: string
  ): Promise<{
    success: boolean;
    event: SwapEvent;
    rollbackAvailable: boolean;
  }> {
    const startTime = Date.now();

    try {
      // 1. Get models
      const currentModel = this.registry.getModel(currentModelId);
      const newModel = this.registry.getModel(newModelId);

      if (!currentModel || !newModel) {
        throw new HotSwapError(
          `Model not found: ${!currentModel ? currentModelId : newModelId}`,
          "MODEL_NOT_FOUND"
        );
      }

      // 2. Check compatibility
      const compatibility = await this.checkCompatibility(
        currentModel as LocalModelConfig,
        newModel as LocalModelConfig
      );

      if (!compatibility.canReplace) {
        throw new HotSwapError(
          `Incompatible models: ${compatibility.reason}`,
          "INCOMPATIBLE_MODELS"
        );
      }

      // 3. Get or create new provider
      const newProvider = this.activeProviders.get(newModelId);

      if (!newProvider) {
        // Provider needs to be created externally and registered
        throw new HotSwapError(
          `Provider not registered for model: ${newModelId}`,
          "PROVIDER_NOT_FOUND"
        );
      }

      // 4. Warm up new model
      await newProvider.warmUp();

      // 5. Health check
      const health = await newProvider.getHealth();

      if (health.status !== "healthy") {
        throw new HotSwapError(
          `New model unhealthy: ${health.message}`,
          "UNHEALTHY_MODEL"
        );
      }

      // 6. Create swap event
      const event: SwapEvent = {
        timestamp: new Date(),
        fromModelId: currentModelId,
        toModelId: newModelId,
        taskType,
        reason: "manual_swap",
        success: true,
        durationMs: Date.now() - startTime,
        compatibilityWarnings: compatibility.warnings,
      };

      // 7. Record swap
      this.swapHistory.push(event);

      // 8. Update selector with learning transfer
      // The key insight: learnings stay at task level, not model level
      const taskPerformance = this.learningLayer.getTaskPerformance(taskType);

      if (taskPerformance) {
        // Transfer learnings to new model context
        this.selector.updatePerformanceHistory(newModelId, taskType, {
          quality: taskPerformance.avgQuality,
          latencyMs: taskPerformance.avgLatencyMs,
          memoryMB: taskPerformance.avgMemoryMB,
          success: taskPerformance.successRate > 0.8,
        });
      }

      return {
        success: true,
        event,
        rollbackAvailable: this.activeProviders.has(currentModelId),
      };
    } catch (error) {
      // Record failed swap
      const event: SwapEvent = {
        timestamp: new Date(),
        fromModelId: currentModelId,
        toModelId: newModelId,
        taskType,
        reason: "manual_swap",
        success: false,
        durationMs: Date.now() - startTime,
        error: error instanceof Error ? error.message : "Unknown error",
      };

      this.swapHistory.push(event);

      throw error;
    }
  }

  /**
   * Auto-swap based on performance
   *
   * The arbiter calls this periodically to optimize model selection
   *
   * @param currentModelId Current model ID
   * @param criteria Selection criteria
   * @returns Swap result or null if no swap needed
   */
  async autoSwap(
    currentModelId: string,
    criteria: ModelSelectionCriteria
  ): Promise<{
    swapped: boolean;
    newModelId?: string;
    reason?: string;
    event?: SwapEvent;
  } | null> {
    if (!this.config.enableAutoSwap) {
      return null;
    }

    // Check cooldown
    const lastSwap = this.getLastSwapForTask(criteria.taskType);

    if (lastSwap) {
      const timeSinceSwap = Date.now() - lastSwap.timestamp.getTime();

      if (timeSinceSwap < this.config.swapCooldownMs) {
        return null; // Still in cooldown
      }
    }

    // Get task performance
    const taskPerf = this.learningLayer.getTaskPerformance(criteria.taskType);

    if (!taskPerf || taskPerf.samples < this.config.minSamplesBeforeSwap) {
      return null; // Not enough data
    }

    // Check if current model is underperforming
    if (
      taskPerf.successRate >= this.config.performanceThreshold &&
      taskPerf.avgQuality >= criteria.qualityThreshold * 0.9
    ) {
      return null; // Current model performing well
    }

    // Enhance criteria with learnings
    const enhancedCriteria = this.learningLayer.enhanceCriteriaWithLearnings(
      criteria.taskType,
      criteria
    );

    // Select new model
    const selected = await this.selector.selectModel(enhancedCriteria);

    // Don't swap to same model
    if (selected.primary.id === currentModelId) {
      return { swapped: false, reason: "Same model selected" };
    }

    // Perform swap
    const result = await this.hotSwap(
      currentModelId,
      selected.primary.id,
      criteria.taskType
    );

    return {
      swapped: true,
      newModelId: selected.primary.id,
      reason: `Auto-swap: performance below threshold (${taskPerf.successRate.toFixed(
        2
      )} < ${this.config.performanceThreshold})`,
      event: result.event,
    };
  }

  /**
   * Check model compatibility
   *
   * @param currentModel Current model config
   * @param newModel New model config
   * @returns Compatibility result
   */
  async checkCompatibility(
    currentModel: LocalModelConfig,
    newModel: LocalModelConfig
  ): Promise<CompatibilityResult> {
    const warnings: string[] = [];
    let canReplace = true;
    let reason: string | undefined;

    // 1. Check capabilities
    const currentCaps = new Set(currentModel.capabilities);
    const newCaps = new Set(newModel.capabilities);

    const missingCaps = [...currentCaps].filter((cap) => !newCaps.has(cap));

    if (missingCaps.length > 0) {
      if (this.config.compatibilityCheckStrict) {
        canReplace = false;
        reason = `Missing capabilities: ${missingCaps.join(", ")}`;
      } else {
        warnings.push(
          `New model lacks capabilities: ${missingCaps.join(", ")}`
        );
      }
    }

    // 2. Check hardware requirements
    if (
      newModel.hardwareRequirements?.minMemoryMB &&
      currentModel.hardwareRequirements?.minMemoryMB &&
      newModel.hardwareRequirements.minMemoryMB >
        currentModel.hardwareRequirements.minMemoryMB * 1.5
    ) {
      warnings.push(
        `New model requires ${newModel.hardwareRequirements.minMemoryMB}MB vs ${currentModel.hardwareRequirements.minMemoryMB}MB (50% increase)`
      );
    }

    // 3. Check performance characteristics
    if (currentModel.performanceProfile && newModel.performanceProfile) {
      const latencyIncrease =
        (newModel.performanceProfile.avgLatencyMs -
          currentModel.performanceProfile.avgLatencyMs) /
        currentModel.performanceProfile.avgLatencyMs;

      if (latencyIncrease > 0.3) {
        warnings.push(
          `New model is ${(latencyIncrease * 100).toFixed(0)}% slower`
        );
      }
    }

    return {
      canReplace,
      reason,
      warnings,
      compatibilityScore: canReplace ? 1 - warnings.length * 0.1 : 0,
    };
  }

  /**
   * Rollback to previous model
   *
   * @param currentModelId Current model ID
   * @param taskType Task type
   * @returns Rollback success
   */
  async rollback(
    currentModelId: string,
    taskType: string
  ): Promise<{
    success: boolean;
    previousModelId?: string;
    event?: SwapEvent;
  }> {
    const lastSwap = this.getLastSwapForTask(taskType);

    if (!lastSwap || !lastSwap.success) {
      throw new HotSwapError(
        "No successful swap found to rollback",
        "NO_ROLLBACK_AVAILABLE"
      );
    }

    if (lastSwap.toModelId !== currentModelId) {
      throw new HotSwapError(
        "Current model doesn't match last swap target",
        "ROLLBACK_MISMATCH"
      );
    }

    // Perform rollback (swap back)
    const result = await this.hotSwap(
      currentModelId,
      lastSwap.fromModelId,
      taskType
    );

    return {
      success: result.success,
      previousModelId: lastSwap.fromModelId,
      event: result.event,
    };
  }

  /**
   * Record task completion for learning
   *
   * This is how system learns independently of models
   *
   * @param taskType Task type
   * @param metrics Performance metrics
   */
  recordTaskCompletion(
    taskType: string,
    metrics: {
      latencyMs: number;
      quality: number;
      memoryMB: number;
      success: boolean;
    }
  ): void {
    this.learningLayer.recordTaskPerformance(taskType, metrics);
  }

  /**
   * Get swap history
   *
   * @param taskType Optional task type filter
   * @param limit Optional limit
   * @returns Swap events
   */
  getSwapHistory(taskType?: string, limit?: number): SwapEvent[] {
    let events = [...this.swapHistory];

    if (taskType) {
      events = events.filter((e) => e.taskType === taskType);
    }

    if (limit) {
      events = events.slice(-limit);
    }

    return events;
  }

  /**
   * Get last swap for task
   *
   * @param taskType Task type
   * @returns Last swap event or undefined
   */
  private getLastSwapForTask(taskType: string): SwapEvent | undefined {
    const events = this.swapHistory.filter((e) => e.taskType === taskType);
    return events.length > 0 ? events[events.length - 1] : undefined;
  }

  /**
   * Get learning layer (for testing/inspection)
   *
   * @returns Learning preservation layer
   */
  getLearningLayer(): LearningPreservationLayer {
    return this.learningLayer;
  }

  /**
   * Get swap statistics
   *
   * @returns Swap statistics
   */
  getSwapStatistics(): {
    totalSwaps: number;
    successfulSwaps: number;
    failedSwaps: number;
    avgSwapDurationMs: number;
    swapsByTaskType: Map<string, number>;
  } {
    const successfulSwaps = this.swapHistory.filter((e) => e.success).length;
    const failedSwaps = this.swapHistory.filter((e) => !e.success).length;

    const totalDuration = this.swapHistory.reduce(
      (sum, e) => sum + (e.durationMs ?? 0),
      0
    );

    const swapsByTaskType = new Map<string, number>();

    for (const event of this.swapHistory) {
      const count = swapsByTaskType.get(event.taskType) ?? 0;
      swapsByTaskType.set(event.taskType, count + 1);
    }

    return {
      totalSwaps: this.swapHistory.length,
      successfulSwaps,
      failedSwaps,
      avgSwapDurationMs:
        this.swapHistory.length > 0
          ? totalDuration / this.swapHistory.length
          : 0,
      swapsByTaskType,
    };
  }
}
