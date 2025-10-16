/**
 * @fileoverview
 * Arbiter model manager - integrates hot-swap with arbiter decision-making.
 * This is how the arbiter picks and chooses best performing LLMs.
 *
 * @author @darianrosebrook
 */

import type {
  GenerationRequest,
  GenerationResponse,
  ModelSelectionCriteria,
} from "@/types/model-registry";
import type { ComputeCostTracker } from "./ComputeCostTracker";
import type { LocalModelSelector } from "./LocalModelSelector";
import type { ModelHotSwapManager } from "./ModelHotSwap";
import type { ModelRegistry } from "./ModelRegistry";

/**
 * Arbiter model manager error
 */
export class ArbiterModelManagerError extends Error {
  constructor(message: string, public _code: string) {
    super(message);
    this.name = "ArbiterModelManagerError";
  }
}

/**
 * Task execution result with performance tracking
 */
interface TaskExecutionResult {
  /** Generated response */
  response: GenerationResponse;

  /** Model used */
  modelId: string;

  /** Performance metrics */
  performance: {
    latencyMs: number;
    quality: number;
    memoryMB: number;
    success: boolean;
  };

  /** Swap occurred during execution */
  swapped: boolean;

  /** Swap details if applicable */
  swapDetails?: {
    fromModelId: string;
    toModelId: string;
    reason: string;
  };
}

/**
 * Arbiter model manager
 *
 * High-level interface for arbiter to:
 * 1. Execute tasks with optimal model selection
 * 2. Automatically swap models based on performance
 * 3. Track and learn from task outcomes
 * 4. Maintain zero-downtime operations
 */
export class ArbiterModelManager {
  private currentModelByTask: Map<string, string> = new Map();

  constructor(
    private _registry: ModelRegistry,
    private _selector: LocalModelSelector,
    private _costTracker: ComputeCostTracker,
    private _hotSwap: ModelHotSwapManager
  ) {}

  /**
   * Execute task with automatic model selection
   *
   * This is the main entry point for the arbiter:
   * 1. Select optimal model (or use cached selection)
   * 2. Execute task
   * 3. Track performance
   * 4. Consider swap if underperforming
   *
   * @param request Generation request
   * @param criteria Selection criteria
   * @returns Execution result with performance tracking
   */
  async executeTask(
    request: GenerationRequest,
    criteria: ModelSelectionCriteria
  ): Promise<TaskExecutionResult> {
    const startTime = Date.now();

    // 1. Get or select model
    let modelId = this.currentModelByTask.get(criteria.taskType);

    if (!modelId) {
      const selected = await this._selector.selectModel(criteria);
      modelId = selected.primary.id || "";
      this.currentModelByTask.set(criteria.taskType, modelId);
    }

    // 2. Get provider
    const provider = this._hotSwap.getProvider(modelId);

    if (!provider) {
      throw new ArbiterModelManagerError(
        `Provider not found for model: ${modelId}`,
        "PROVIDER_NOT_FOUND"
      );
    }

    // 3. Execute
    const response = await provider.generate(request);

    // 4. Track performance
    const latencyMs = Date.now() - startTime;
    const quality = this.estimateQuality(response);
    const memoryMB = response.computeCost?.peakMemoryMB || 0;

    // Record in cost tracker
    if (response.computeCost) {
      this._costTracker.recordOperation(response.computeCost);
    }

    // Record in learning layer (model-agnostic)
    this._hotSwap.recordTaskCompletion(criteria.taskType, {
      latencyMs,
      quality,
      memoryMB,
      success: true,
    });

    // Record in selector (model-specific)
    this._selector.updatePerformanceHistory(modelId, criteria.taskType, {
      quality,
      latencyMs,
      memoryMB,
      success: true,
    });

    // 5. Check if swap needed
    const swapResult = await this._hotSwap.autoSwap(modelId, criteria);

    if (swapResult?.swapped && swapResult.newModelId) {
      // Update current model
      this.currentModelByTask.set(criteria.taskType, swapResult.newModelId);

      return {
        response,
        modelId: modelId!,
        performance: {
          latencyMs,
          quality,
          memoryMB,
          success: true,
        },
        swapped: true,
        swapDetails: {
          fromModelId: modelId!,
          toModelId: swapResult.newModelId,
          reason: swapResult.reason || "Performance-based auto-swap",
        },
      };
    }

    return {
      response,
      modelId: modelId!,
      performance: {
        latencyMs,
        quality,
        memoryMB,
        success: true,
      },
      swapped: false,
    };
  }

  /**
   * Force model swap for task type
   *
   * @param taskType Task type
   * @param newModelId New model ID
   * @returns Swap result
   */
  async forceSwap(
    taskType: string,
    newModelId: string
  ): Promise<{
    success: boolean;
    fromModelId?: string;
    event?: any;
  }> {
    const currentModelId = this.currentModelByTask.get(taskType);

    if (!currentModelId) {
      throw new ArbiterModelManagerError(
        `No current model for task: ${taskType}`,
        "NO_CURRENT_MODEL"
      );
    }

    const result = await this._hotSwap.hotSwap(
      currentModelId,
      newModelId,
      taskType
    );

    if (result.success) {
      this.currentModelByTask.set(taskType, newModelId);
    }

    return {
      success: result.success,
      fromModelId: currentModelId,
      event: result.event,
    };
  }

  /**
   * Rollback task to previous model
   *
   * @param taskType Task type
   * @returns Rollback result
   */
  async rollback(taskType: string): Promise<{
    success: boolean;
    previousModelId?: string;
  }> {
    const currentModelId = this.currentModelByTask.get(taskType);

    if (!currentModelId) {
      throw new ArbiterModelManagerError(
        `No current model for task: ${taskType}`,
        "NO_CURRENT_MODEL"
      );
    }

    const result = await this._hotSwap.rollback(currentModelId, taskType);

    if (result.success && result.previousModelId) {
      this.currentModelByTask.set(taskType, result.previousModelId);
    }

    return result;
  }

  /**
   * Get current model for task
   *
   * @param taskType Task type
   * @returns Model ID or undefined
   */
  getCurrentModel(taskType: string): string | undefined {
    return this.currentModelByTask.get(taskType);
  }

  /**
   * Get performance summary for task
   *
   * @param taskType Task type
   * @returns Performance summary
   */
  getTaskPerformanceSummary(taskType: string): {
    taskType: string;
    currentModel?: string;
    learnings?: any;
    swapHistory: any[];
    costProfile?: any;
  } {
    const currentModel = this.currentModelByTask.get(taskType);
    const learningLayer = this._hotSwap.getLearningLayer();
    const learnings = learningLayer.getTaskPerformance(taskType);
    const swapHistory = this._hotSwap.getSwapHistory(taskType);

    let costProfile;

    if (currentModel) {
      costProfile = this._costTracker.getCostProfile(currentModel);
    }

    return {
      taskType,
      currentModel,
      learnings,
      swapHistory,
      costProfile,
    };
  }

  /**
   * Get swap statistics across all tasks
   *
   * @returns Comprehensive statistics
   */
  getStatistics(): {
    totalTasks: number;
    modelsByTask: Map<string, string>;
    swapStats: any;
    topModels: Array<{ modelId: string; taskTypes: string[] }>;
  } {
    const swapStats = this._hotSwap.getSwapStatistics();

    // Analyze model usage
    const modelUsage = new Map<string, Set<string>>();

    for (const [taskType, modelId] of this.currentModelByTask.entries()) {
      if (!modelUsage.has(modelId)) {
        modelUsage.set(modelId, new Set());
      }

      modelUsage.get(modelId)!.add(taskType);
    }

    const topModels = Array.from(modelUsage.entries())
      .map(([modelId, taskTypes]) => ({
        modelId,
        taskTypes: Array.from(taskTypes),
      }))
      .sort((a, b) => b.taskTypes.length - a.taskTypes.length);

    return {
      totalTasks: this.currentModelByTask.size,
      modelsByTask: new Map(this.currentModelByTask),
      swapStats,
      topModels,
    };
  }

  /**
   * Estimate quality from response
   *
   * This is a simple heuristic - in production, would use
   * more sophisticated quality estimation
   *
   * @param response Generation response
   * @returns Quality score 0-1
   */
  private estimateQuality(response: GenerationResponse): number {
    // Simple heuristics for quality estimation
    let quality = 0.8; // Base quality

    // Check response length
    if (response.text.length < 10) {
      quality -= 0.3; // Very short response
    } else if (response.text.length > 100) {
      quality += 0.1; // Detailed response
    }

    // Check tokens/second (efficiency indicator)
    if (response.tokensPerSecond > 50) {
      quality += 0.1; // Fast generation
    }

    return Math.max(0, Math.min(1, quality));
  }
}

/**
 * Create arbiter model manager with all dependencies
 *
 * Convenience factory function
 *
 * @param registry Model registry
 * @param selector Model selector
 * @param costTracker Cost tracker
 * @param hotSwap Hot-swap manager
 * @returns Arbiter model manager
 */
export function createArbiterModelManager(
  registry: ModelRegistry,
  selector: LocalModelSelector,
  costTracker: ComputeCostTracker,
  hotSwap: ModelHotSwapManager
): ArbiterModelManager {
  return new ArbiterModelManager(registry, selector, costTracker, hotSwap);
}
