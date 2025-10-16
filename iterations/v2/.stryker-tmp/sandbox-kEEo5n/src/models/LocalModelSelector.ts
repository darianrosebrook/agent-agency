/**
 * @fileoverview
 * Local model selector for performance-based selection.
 * Selects optimal local model based on task requirements and historical performance.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import type {
  AvailableHardware,
  LocalModelConfig,
  ModelSelectionCriteria,
  PerformanceHistory,
  SelectedModel,
} from "@/types/model-registry";
import type { ComputeCostTracker } from "./ComputeCostTracker";
import type { ModelRegistry } from "./ModelRegistry";

/**
 * Model selector error
 */
export class ModelSelectorError extends Error {
  constructor(message: string, public code: string) {
    super(message);
    this.name = "ModelSelectorError";
  }
}

/**
 * Local model selector
 *
 * Intelligently selects the best local model for a task based on:
 * - Task requirements (capabilities, performance, resource limits)
 * - Historical performance data
 * - Hardware availability
 * - Cost efficiency (local compute resources)
 */
export class LocalModelSelector {
  private performanceHistory: Map<string, Map<string, PerformanceHistory>> =
    new Map();

  constructor(
    private registry: ModelRegistry,
    private costTracker: ComputeCostTracker
  ) {}

  /**
   * Select best local model for task
   *
   * @param criteria Selection criteria
   * @returns Selected model with fallback
   * @throws ModelSelectorError if no capable models available
   */
  async selectModel(criteria: ModelSelectionCriteria): Promise<SelectedModel> {
    // 1. Filter by capabilities
    const capable = this.registry.findByCapabilities(
      criteria.requiredCapabilities
    );

    if (capable.length === 0) {
      throw new ModelSelectorError(
        `No models found with capabilities: ${criteria.requiredCapabilities.join(
          ", "
        )}`,
        "NO_CAPABLE_MODELS"
      );
    }

    // 2. Filter by hardware compatibility
    const compatible = capable.filter((model) =>
      this.isHardwareCompatible(model, criteria.availableHardware)
    );

    if (compatible.length === 0) {
      throw new ModelSelectorError(
        "No models compatible with available hardware",
        "NO_COMPATIBLE_HARDWARE"
      );
    }

    // 3. Score each model
    const scored = await Promise.all(
      compatible.map(async (model) => ({
        model,
        score: await this.scoreModel(model, criteria),
      }))
    );

    // 4. Sort by score (higher = better)
    scored.sort((a, b) => b.score - a.score);

    if (scored.length === 0) {
      throw new ModelSelectorError(
        "No suitable models found after scoring",
        "NO_SUITABLE_MODELS"
      );
    }

    // 5. Select primary and fallback
    const primary = scored[0].model;
    const fallback = scored[1]?.model;

    // 6. Get expected performance
    const expectedPerformance = await this.getExpectedPerformance(
      primary,
      criteria
    );

    // 7. Build reasoning
    const reasoning = this.buildReasoning(primary, scored[0].score, criteria);

    return {
      primary,
      fallback,
      reasoning,
      confidence: this.calculateConfidence(scored[0].score, criteria),
      expectedPerformance,
    };
  }

  /**
   * Score a model for given criteria
   *
   * @param model Model to score
   * @param criteria Selection criteria
   * @returns Score (0-1 range, higher is better)
   */
  async scoreModel(
    model: LocalModelConfig,
    criteria: ModelSelectionCriteria
  ): Promise<number> {
    const history = this.getPerformanceHistory(model.id, criteria.taskType);

    // If no history, use conservative score
    if (!history || history.samples < 1) {
      return this.scoreNewModel(model, criteria);
    }

    let score = 0;
    const weights = this.getWeights(criteria);

    // Quality score (0-1)
    const qualityScore = Math.min(
      history.avgQuality / criteria.qualityThreshold,
      1
    );
    score += qualityScore * weights.quality;

    // Latency score (0-1, inverted - lower is better)
    const latencyScore = Math.max(
      0,
      1 - history.avgLatencyMs / criteria.maxLatencyMs
    );
    score += latencyScore * weights.latency;

    // Resource efficiency score (0-1)
    const memoryScore = Math.max(
      0,
      1 - history.avgMemoryMB / criteria.maxMemoryMB
    );
    score += memoryScore * weights.memory;

    // Reliability score (0-1)
    score += history.successRate * weights.reliability;

    // Recent performance bonus (models that improve over time)
    const recentBonus = this.calculateRecentPerformanceBonus(model.id);
    score += recentBonus * weights.recency;

    return Math.min(1, score); // Normalize to 0-1
  }

  /**
   * Check if model is compatible with available hardware
   *
   * @param model Model to check
   * @param hardware Available hardware
   * @returns True if compatible
   */
  isHardwareCompatible(
    model: LocalModelConfig,
    hardware: AvailableHardware
  ): boolean {
    switch (model.type) {
      case "ollama":
        // Ollama models can run on CPU or GPU
        return hardware.cpu || hardware.gpu;

      case "custom":
        // Check custom model requirements
        const customReqs = model.hardwareRequirements;
        if (customReqs?.requiresGpu && !hardware.gpu) {
          return false;
        }
        return true;

      case "hardware-optimized":
        // Check target hardware availability
        switch (model.targetHardware) {
          case "apple-silicon":
            return hardware.ane ?? false;
          case "nvidia-gpu":
          case "amd-gpu":
            return hardware.gpu;
          case "cpu-only":
            return hardware.cpu;
          case "custom-server":
            return true; // Assume custom server is available
          default:
            return false;
        }

      default:
        return false;
    }
  }

  /**
   * Update performance history for a model
   *
   * @param modelId Model ID
   * @param taskType Task type
   * @param metrics Performance metrics
   */
  updatePerformanceHistory(
    modelId: string,
    taskType: string,
    metrics: {
      quality: number;
      latencyMs: number;
      memoryMB: number;
      success: boolean;
    }
  ): void {
    if (!this.performanceHistory.has(modelId)) {
      this.performanceHistory.set(modelId, new Map());
    }

    const modelHistory = this.performanceHistory.get(modelId)!;
    const existing = modelHistory.get(taskType);

    if (!existing) {
      // Create new history
      modelHistory.set(taskType, {
        modelId,
        taskType,
        samples: 1,
        avgQuality: metrics.quality,
        avgLatencyMs: metrics.latencyMs,
        p95LatencyMs: metrics.latencyMs,
        avgMemoryMB: metrics.memoryMB,
        successRate: metrics.success ? 1 : 0,
        lastUpdated: new Date(),
      });
    } else {
      // Update existing history (moving average)
      const newSamples = existing.samples + 1;
      const alpha = 1 / newSamples; // Simple moving average

      modelHistory.set(taskType, {
        ...existing,
        samples: newSamples,
        avgQuality: existing.avgQuality * (1 - alpha) + metrics.quality * alpha,
        avgLatencyMs:
          existing.avgLatencyMs * (1 - alpha) + metrics.latencyMs * alpha,
        p95LatencyMs: Math.max(existing.p95LatencyMs, metrics.latencyMs),
        avgMemoryMB:
          existing.avgMemoryMB * (1 - alpha) + metrics.memoryMB * alpha,
        successRate:
          existing.successRate * (1 - alpha) +
          (metrics.success ? 1 : 0) * alpha,
        lastUpdated: new Date(),
      });
    }
  }

  /**
   * Get performance history for model and task
   *
   * @param modelId Model ID
   * @param taskType Task type
   * @returns Performance history or undefined
   */
  getPerformanceHistory(
    modelId: string,
    taskType: string
  ): PerformanceHistory | undefined {
    return this.performanceHistory.get(modelId)?.get(taskType);
  }

  /**
   * Clear all performance history
   */
  clearHistory(): void {
    this.performanceHistory.clear();
  }

  /**
   * Clear history for specific model
   *
   * @param modelId Model ID
   */
  clearModelHistory(modelId: string): void {
    this.performanceHistory.delete(modelId);
  }

  /**
   * Score a new model without history
   *
   * @param model Model to score
   * @param criteria Selection criteria
   * @returns Conservative score
   */
  private scoreNewModel(
    model: LocalModelConfig,
    criteria: ModelSelectionCriteria
  ): number {
    let score = 0.5; // Base conservative score

    // Boost for preferred categories
    if (criteria.preferences?.preferFast && model.category === "fast") {
      score += 0.2;
    }
    if (criteria.preferences?.preferQuality && model.category === "quality") {
      score += 0.2;
    }

    // Boost for local models if preferred
    if (criteria.preferLocal) {
      score += 0.1;
    }

    // Consider quality threshold - quality models should score higher when quality is important
    if (criteria.qualityThreshold > 0.8 && model.category === "quality") {
      score += 0.3;
    }

    // Consider latency requirements - fast models should score higher when latency is critical
    if (criteria.maxLatencyMs < 1000 && model.category === "fast") {
      score += 0.2;
    }

    // Check cost profile if available
    const costProfile = this.costTracker.getCostProfile(model.id);
    if (costProfile) {
      // Boost for good performance in cost profile
      if (costProfile.avgTokensPerSec > 50) {
        score += 0.1;
      }
      if (costProfile.avgWallClockMs < 1000) {
        score += 0.1;
      }
    }

    return Math.min(1, score);
  }

  /**
   * Get scoring weights based on criteria preferences
   *
   * @param criteria Selection criteria
   * @returns Weights object
   */
  private getWeights(criteria: ModelSelectionCriteria): {
    quality: number;
    latency: number;
    memory: number;
    reliability: number;
    recency: number;
  } {
    const weights = {
      quality: 0.5,
      latency: 0.2,
      memory: 0.1,
      reliability: 0.1,
      recency: 0.1,
    };

    // Adjust based on preferences
    if (criteria.preferences?.preferFast) {
      weights.latency += 0.2;
      weights.quality -= 0.1;
      weights.memory -= 0.1;
    }

    if (criteria.preferences?.preferQuality) {
      weights.quality += 0.2;
      weights.latency -= 0.1;
      weights.memory -= 0.1;
    }

    if (criteria.preferences?.preferLowMemory) {
      weights.memory += 0.2;
      weights.quality -= 0.1;
      weights.latency -= 0.1;
    }

    return weights;
  }

  /**
   * Calculate recent performance bonus
   *
   * @param modelId Model ID
   * @returns Bonus score (0-0.2)
   */
  private calculateRecentPerformanceBonus(modelId: string): number {
    const recentCosts = this.costTracker.getModelCosts(modelId, 100);

    if (recentCosts.length < 20) {
      return 0;
    }

    // Compare first half to second half
    const midpoint = Math.floor(recentCosts.length / 2);
    const firstHalf = recentCosts.slice(0, midpoint);
    const secondHalf = recentCosts.slice(midpoint);

    const firstAvgTokens = this.mean(firstHalf.map((c) => c.tokensPerSecond));
    const secondAvgTokens = this.mean(secondHalf.map((c) => c.tokensPerSecond));

    // If improving, give bonus
    if (secondAvgTokens > firstAvgTokens) {
      const improvement = (secondAvgTokens - firstAvgTokens) / firstAvgTokens;
      return Math.min(0.2, improvement);
    }

    return 0;
  }

  /**
   * Get expected performance for model
   *
   * @param model Selected model
   * @param criteria Selection criteria
   * @returns Expected performance characteristics
   */
  private async getExpectedPerformance(
    model: LocalModelConfig,
    criteria: ModelSelectionCriteria
  ) {
    const history = this.getPerformanceHistory(model.id, criteria.taskType);
    const costProfile = this.costTracker.getCostProfile(model.id);

    return {
      avgLatencyMs: history?.avgLatencyMs ?? 1000,
      p95LatencyMs: history?.p95LatencyMs ?? 1500,
      tokensPerSec: costProfile?.avgTokensPerSec ?? 50,
      memoryUsageMB: history?.avgMemoryMB ?? 1024,
      cpuUtilization: 50, // Estimate
    };
  }

  /**
   * Build selection reasoning
   *
   * @param model Selected model
   * @param score Model score
   * @param criteria Selection criteria
   * @returns Reasoning array
   */
  private buildReasoning(
    model: LocalModelConfig,
    score: number,
    criteria: ModelSelectionCriteria
  ): string[] {
    const reasons: string[] = [];

    reasons.push(`Selected ${model.name} (${model.type})`);
    reasons.push(`Score: ${(score * 100).toFixed(1)}%`);

    const history = this.getPerformanceHistory(model.id, criteria.taskType);
    if (history) {
      reasons.push(
        `Historical success rate: ${(history.successRate * 100).toFixed(1)}%`
      );
      reasons.push(`Avg latency: ${history.avgLatencyMs.toFixed(0)}ms`);
    }

    const costProfile = this.costTracker.getCostProfile(model.id);
    if (costProfile) {
      reasons.push(
        `Throughput: ${costProfile.avgTokensPerSec.toFixed(0)} tokens/s`
      );
    }

    // Add hardware compatibility reasoning
    if (criteria.availableHardware) {
      const hardwareTypes = [];
      if (criteria.availableHardware.cpu) hardwareTypes.push("CPU");
      if (criteria.availableHardware.gpu) hardwareTypes.push("GPU");
      if (hardwareTypes.length > 0) {
        reasons.push(`Hardware: ${hardwareTypes.join(", ")} compatible`);
      }
    }

    return reasons;
  }

  /**
   * Calculate confidence in selection
   *
   * @param score Model score
   * @param criteria Selection criteria
   * @returns Confidence (0-1)
   */
  private calculateConfidence(
    score: number,
    criteria: ModelSelectionCriteria
  ): number {
    // Higher score = higher confidence
    // More historical data = higher confidence
    let confidence = score;

    // Reduce confidence if no history
    const totalSamples = Array.from(this.performanceHistory.values()).reduce(
      (sum, modelHistory) => {
        const taskHistory = Array.from(modelHistory.values()).find(
          (h) => h.taskType === criteria.taskType
        );
        return sum + (taskHistory?.samples ?? 0);
      },
      0
    );

    if (totalSamples < 10) {
      confidence *= 0.5; // Reduce confidence for new tasks
    } else if (totalSamples < 50) {
      confidence *= 0.75;
    }

    return Math.min(1, confidence);
  }

  /**
   * Calculate mean of array
   *
   * @param values Array of numbers
   * @returns Mean value
   */
  private mean(values: number[]): number {
    if (values.length === 0) {
      return 0;
    }

    return values.reduce((sum, v) => sum + v, 0) / values.length;
  }
}
