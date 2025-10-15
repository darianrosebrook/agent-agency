/**
 * @fileoverview
 * Compute cost tracker for local models.
 * Tracks time, memory, energy consumption for local inference.
 *
 * @author @darianrosebrook
 */

import type { CostProfile, LocalComputeCost } from "@/types/model-registry";

/**
 * Compute cost tracker for local models
 *
 * Tracks actual compute resources used (time, memory, energy)
 * rather than API costs. Essential for local-first optimization.
 */
export class ComputeCostTracker {
  private costs: Map<string, LocalComputeCost[]> = new Map();
  private readonly maxCostsPerModel: number;

  constructor(maxCostsPerModel: number = 1000) {
    this.maxCostsPerModel = maxCostsPerModel;
  }

  /**
   * Record compute cost for a model operation
   *
   * @param cost Compute cost details
   */
  recordOperation(cost: LocalComputeCost): void {
    const modelCosts = this.costs.get(cost.modelId) ?? [];

    // Add new cost
    modelCosts.push(cost);

    // Keep only recent costs (FIFO)
    if (modelCosts.length > this.maxCostsPerModel) {
      modelCosts.shift();
    }

    this.costs.set(cost.modelId, modelCosts);

    // Log optimization opportunities
    this.checkOptimizationOpportunities(cost);
  }

  /**
   * Get cost profile for a model
   *
   * @param modelId Model ID
   * @param sampleSize Number of recent operations to analyze
   * @returns Cost profile with averages
   */
  getCostProfile(
    modelId: string,
    sampleSize?: number
  ): CostProfile | undefined {
    const modelCosts = this.costs.get(modelId);

    if (!modelCosts || modelCosts.length === 0) {
      return undefined;
    }

    // Use most recent samples
    const samples = sampleSize ? modelCosts.slice(-sampleSize) : modelCosts;

    if (samples.length === 0) {
      return undefined;
    }

    // Calculate averages
    const avgWallClockMs = this.mean(samples.map((c) => c.wallClockMs));
    const avgEnergyMWh = this.mean(
      samples.map((c) => c.estimatedEnergyMWh ?? 0)
    );
    const avgTokensPerSec = this.mean(samples.map((c) => c.tokensPerSecond));
    const p95WallClockMs = this.percentile(
      samples.map((c) => c.wallClockMs),
      95
    );

    return {
      modelId,
      avgWallClockMs,
      avgEnergyMWh: avgEnergyMWh > 0 ? avgEnergyMWh : undefined,
      avgTokensPerSec,
      p95WallClockMs,
      totalOperations: samples.length,
      lastUpdated: new Date(),
    };
  }

  /**
   * Get all recorded costs for a model
   *
   * @param modelId Model ID
   * @param limit Maximum number of costs to return
   * @returns Array of compute costs
   */
  getModelCosts(modelId: string, limit?: number): LocalComputeCost[] {
    const costs = this.costs.get(modelId) ?? [];

    if (limit) {
      return costs.slice(-limit);
    }

    return [...costs];
  }

  /**
   * Get total operations tracked for a model
   *
   * @param modelId Model ID
   * @returns Number of operations
   */
  getTotalOperations(modelId: string): number {
    return this.costs.get(modelId)?.length ?? 0;
  }

  /**
   * Compare cost profiles between two models
   *
   * @param modelId1 First model ID
   * @param modelId2 Second model ID
   * @returns Comparison metrics
   */
  compareCosts(
    modelId1: string,
    modelId2: string
  ):
    | {
        latencyDiff: number; // Percentage difference
        energyDiff: number;
        throughputDiff: number;
        winner: string; // ID of better model
      }
    | undefined {
    const profile1 = this.getCostProfile(modelId1);
    const profile2 = this.getCostProfile(modelId2);

    if (!profile1 || !profile2) {
      return undefined;
    }

    const latencyDiff = this.percentageDiff(
      profile1.avgWallClockMs,
      profile2.avgWallClockMs
    );
    const energyDiff = this.percentageDiff(
      profile1.avgEnergyMWh ?? 0,
      profile2.avgEnergyMWh ?? 0
    );
    const throughputDiff = this.percentageDiff(
      profile1.avgTokensPerSec,
      profile2.avgTokensPerSec
    );

    // Lower latency and energy is better, higher throughput is better
    // Calculate scores where lower is better for latency/energy, higher is better for throughput
    const model1LatencyScore = profile1.avgWallClockMs;
    const model2LatencyScore = profile2.avgWallClockMs;
    const model1EnergyScore = profile1.avgEnergyMWh ?? 0;
    const model2EnergyScore = profile2.avgEnergyMWh ?? 0;
    const model1ThroughputScore = profile1.avgTokensPerSec;
    const model2ThroughputScore = profile2.avgTokensPerSec;

    // Normalize scores (lower latency/energy is better, higher throughput is better)
    const model1Score =
      (model2LatencyScore - model1LatencyScore) / model2LatencyScore + // Latency advantage
      (model2EnergyScore - model1EnergyScore) / (model2EnergyScore || 1) + // Energy advantage
      (model1ThroughputScore - model2ThroughputScore) /
        (model2ThroughputScore || 1); // Throughput advantage

    return {
      latencyDiff,
      energyDiff,
      throughputDiff,
      winner: model1Score > 0 ? modelId1 : modelId2,
    };
  }

  /**
   * Get optimization recommendations for a model
   *
   * @param modelId Model ID
   * @returns Array of recommendations
   */
  getOptimizationRecommendations(modelId: string): string[] {
    const profile = this.getCostProfile(modelId);
    const recommendations: string[] = [];

    if (!profile) {
      return recommendations;
    }

    // Check recent operations for patterns
    const recentCosts = this.getModelCosts(modelId, 100);

    if (recentCosts.length < 10) {
      recommendations.push(
        "Insufficient data for optimization recommendations"
      );
      return recommendations;
    }

    // Analyze CPU utilization
    const avgCpuUtilization = this.mean(
      recentCosts.map((c) => c.cpuUtilization)
    );

    if (avgCpuUtilization < 30) {
      recommendations.push(
        "Low CPU utilization detected - consider batch processing or increasing concurrency"
      );
    } else if (avgCpuUtilization > 90) {
      recommendations.push(
        "High CPU utilization - consider load balancing or adding instances"
      );
    }

    // Analyze GPU utilization (if available)
    const gpuUtilizations = recentCosts
      .map((c) => c.gpuUtilization)
      .filter((u): u is number => u !== undefined);

    if (gpuUtilizations.length > 0) {
      const avgGpuUtilization = this.mean(gpuUtilizations);

      if (avgGpuUtilization < 30) {
        recommendations.push(
          "Low GPU utilization - model may benefit from GPU-optimized inference"
        );
      }
    }

    // Analyze memory usage
    const avgMemory = this.mean(recentCosts.map((c) => c.avgMemoryMB));
    const peakMemory = Math.max(...recentCosts.map((c) => c.peakMemoryMB));

    if (peakMemory > avgMemory * 2) {
      recommendations.push(
        "Memory spikes detected - consider memory pooling or preallocation"
      );
    }

    // Analyze throughput
    if (profile.avgTokensPerSec < 10) {
      recommendations.push(
        "Low throughput detected - consider model quantization or hardware acceleration"
      );
    }

    // Analyze latency
    if (profile.p95WallClockMs > 5000) {
      recommendations.push(
        "High P95 latency - consider warm instance pooling or model caching"
      );
    }

    return recommendations;
  }

  /**
   * Clear all tracked costs
   */
  clear(): void {
    this.costs.clear();
  }

  /**
   * Clear costs for a specific model
   *
   * @param modelId Model ID
   */
  clearModel(modelId: string): void {
    this.costs.delete(modelId);
  }

  /**
   * Get all tracked model IDs
   *
   * @returns Array of model IDs
   */
  getTrackedModels(): string[] {
    return Array.from(this.costs.keys());
  }

  /**
   * Check for optimization opportunities
   *
   * @param cost Compute cost
   */
  private checkOptimizationOpportunities(cost: LocalComputeCost): void {
    const warnings: string[] = [];

    // Check CPU utilization
    if (cost.cpuUtilization < 20) {
      warnings.push("Very low CPU utilization");
    } else if (cost.cpuUtilization > 95) {
      warnings.push("CPU near saturation");
    }

    // Check GPU utilization
    if (cost.gpuUtilization !== undefined) {
      if (cost.gpuUtilization < 20) {
        warnings.push("Very low GPU utilization");
      } else if (cost.gpuUtilization > 95) {
        warnings.push("GPU near saturation");
      }
    }

    // Check memory
    if (cost.peakMemoryMB > cost.avgMemoryMB * 3) {
      warnings.push("High memory spike detected");
    }

    // Check latency
    if (cost.wallClockMs > 10000) {
      warnings.push("Very high latency (>10s)");
    }

    // Check throughput
    if (cost.tokensPerSecond < 5) {
      warnings.push("Very low throughput (<5 tokens/s)");
    }

    // Log warnings if any
    if (warnings.length > 0) {
      console.warn(`[ComputeCostTracker] Model ${cost.modelId}:`, {
        operationId: cost.operationId,
        warnings,
        metrics: {
          cpuUtilization: `${cost.cpuUtilization}%`,
          gpuUtilization: cost.gpuUtilization
            ? `${cost.gpuUtilization}%`
            : "N/A",
          wallClockMs: cost.wallClockMs,
          tokensPerSec: cost.tokensPerSecond,
        },
      });
    }
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

  /**
   * Calculate percentile
   *
   * @param values Array of numbers
   * @param percentile Percentile (0-100)
   * @returns Percentile value
   */
  private percentile(values: number[], percentile: number): number {
    if (values.length === 0) {
      return 0;
    }

    const sorted = [...values].sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * sorted.length) - 1;

    return sorted[Math.max(0, index)];
  }

  /**
   * Calculate percentage difference
   *
   * @param value1 First value
   * @param value2 Second value
   * @returns Percentage difference (positive if value2 > value1)
   */
  private percentageDiff(value1: number, value2: number): number {
    if (value1 === 0) {
      return value2 === 0 ? 0 : 100;
    }

    return ((value2 - value1) / value1) * 100;
  }
}
