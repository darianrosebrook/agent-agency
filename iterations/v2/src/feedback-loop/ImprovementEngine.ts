import { EventEmitter } from "events";
import { ConfigManager } from "../config/ConfigManager";
import { Logger } from "../observability/Logger";
import {
  FeedbackRecommendation,
  ImprovementEngineConfig,
} from "../types/feedback-loop";

export class ImprovementEngine extends EventEmitter {
  private config: ImprovementEngineConfig;
  private logger: Logger;

  // Track active improvements and cooldowns
  private activeImprovements: Map<
    string,
    { recommendation: FeedbackRecommendation; startTime: Date }
  > = new Map();
  private cooldowns: Map<string, Date> = new Map(); // entityId -> cooldown end time
  private improvementHistory: FeedbackRecommendation[] = [];

  // Success tracking
  private successRates: Map<string, { successes: number; attempts: number }> =
    new Map();

  constructor(configManager: ConfigManager) {
    super();
    this.config = configManager.get("feedbackLoop.improvements");
    this.logger = new Logger("ImprovementEngine");
  }

  public async evaluateRecommendation(
    recommendation: FeedbackRecommendation
  ): Promise<boolean> {
    // Check if entity is in cooldown
    if (this.isInCooldown(recommendation.action.targetEntity)) {
      this.logger.debug(
        `Entity ${recommendation.action.targetEntity} is in cooldown, skipping recommendation`
      );
      return false;
    }

    // Check confidence threshold
    if (
      recommendation.expectedImpact.improvementPercent / 100 <
      this.config.autoApplyThreshold
    ) {
      this.logger.debug(
        `Recommendation confidence below threshold: ${recommendation.expectedImpact.improvementPercent}%`
      );
      return false;
    }

    // Check concurrent improvement limit
    const activeForEntity = Array.from(this.activeImprovements.values()).filter(
      (imp) =>
        imp.recommendation.action.targetEntity ===
        recommendation.action.targetEntity
    );

    if (activeForEntity.length >= this.config.maxConcurrentImprovements) {
      this.logger.debug(
        `Too many concurrent improvements for entity ${recommendation.action.targetEntity}`
      );
      return false;
    }

    // Check prerequisites
    if (!this.checkPrerequisites(recommendation)) {
      this.logger.debug(
        `Prerequisites not met for recommendation ${recommendation.id}`
      );
      return false;
    }

    return true;
  }

  public async applyRecommendation(
    recommendation: FeedbackRecommendation
  ): Promise<boolean> {
    if (!(await this.evaluateRecommendation(recommendation))) {
      return false;
    }

    const startTime = new Date();
    this.activeImprovements.set(recommendation.id, {
      recommendation,
      startTime,
    });
    recommendation.implementationStatus = "in_progress";

    this.logger.info(`Applying recommendation: ${recommendation.description}`, {
      recommendationId: recommendation.id,
      targetEntity: recommendation.action.targetEntity,
      operation: recommendation.action.operation,
    });

    this.emit("improvement:started", {
      recommendationId: recommendation.id,
      targetEntity: recommendation.action.targetEntity,
      operation: recommendation.action.operation,
      startTime,
    });

    try {
      const success = await this.executeImprovement(recommendation);

      if (success) {
        recommendation.implementationStatus = "implemented";
        this.updateSuccessRate(recommendation.action.operation, true);

        // Set cooldown for the entity
        this.setCooldown(recommendation.action.targetEntity);

        this.emit("improvement:completed", {
          recommendationId: recommendation.id,
          targetEntity: recommendation.action.targetEntity,
          completionTime: new Date(),
          expectedImpact: recommendation.expectedImpact,
        });

        this.logger.info(
          `Successfully applied recommendation ${recommendation.id}`
        );
      } else {
        recommendation.implementationStatus = "failed";
        this.updateSuccessRate(recommendation.action.operation, false);

        if (this.config.rollbackOnFailure) {
          await this.rollbackImprovement(recommendation);
        }

        this.emit("improvement:failed", {
          recommendationId: recommendation.id,
          targetEntity: recommendation.action.targetEntity,
          failureTime: new Date(),
          error: "Improvement execution failed",
        });

        this.logger.error(
          `Failed to apply recommendation ${recommendation.id}`
        );
      }

      this.improvementHistory.push({ ...recommendation });
      this.activeImprovements.delete(recommendation.id);

      return success;
    } catch (error) {
      recommendation.implementationStatus = "failed";
      this.updateSuccessRate(recommendation.action.operation, false);
      this.activeImprovements.delete(recommendation.id);

      this.emit("improvement:error", {
        recommendationId: recommendation.id,
        targetEntity: recommendation.action.targetEntity,
        errorTime: new Date(),
        error: error.message,
      });

      this.logger.error(
        `Error applying recommendation ${recommendation.id}: ${error.message}`
      );
      return false;
    }
  }

  public async applyRecommendations(
    recommendations: FeedbackRecommendation[]
  ): Promise<{
    applied: FeedbackRecommendation[];
    skipped: FeedbackRecommendation[];
    failed: FeedbackRecommendation[];
  }> {
    const results = {
      applied: [] as FeedbackRecommendation[],
      skipped: [] as FeedbackRecommendation[],
      failed: [] as FeedbackRecommendation[],
    };

    // Sort by priority
    const priorityOrder = { critical: 4, high: 3, medium: 2, low: 1 };
    const sortedRecommendations = recommendations.sort(
      (a, b) => priorityOrder[b.priority] - priorityOrder[a.priority]
    );

    for (const recommendation of sortedRecommendations) {
      try {
        if (await this.evaluateRecommendation(recommendation)) {
          const success = await this.applyRecommendation(recommendation);
          if (success) {
            results.applied.push(recommendation);
          } else {
            results.failed.push(recommendation);
          }
        } else {
          results.skipped.push(recommendation);
        }
      } catch (error) {
        this.logger.error(
          `Error processing recommendation ${recommendation.id}: ${error.message}`
        );
        results.failed.push(recommendation);
      }
    }

    return results;
  }

  public getActiveImprovements(): FeedbackRecommendation[] {
    return Array.from(this.activeImprovements.values()).map(
      (imp) => imp.recommendation
    );
  }

  public getImprovementHistory(entityId?: string): FeedbackRecommendation[] {
    if (entityId) {
      return this.improvementHistory.filter(
        (imp) => imp.action.targetEntity === entityId
      );
    }
    return [...this.improvementHistory];
  }

  public getSuccessRates(): Record<
    string,
    { successRate: number; attempts: number }
  > {
    const rates: Record<string, { successRate: number; attempts: number }> = {};

    for (const [operation, stats] of this.successRates.entries()) {
      rates[operation] = {
        successRate: stats.attempts > 0 ? stats.successes / stats.attempts : 0,
        attempts: stats.attempts,
      };
    }

    return rates;
  }

  public async monitorImprovementEffects(
    recommendation: FeedbackRecommendation
  ): Promise<{
    effective: boolean;
    actualImpact?: number;
    monitoringComplete: boolean;
  }> {
    if (recommendation.implementationStatus !== "implemented") {
      return { effective: false, monitoringComplete: false };
    }

    const implementationTime = this.improvementHistory.find(
      (h) => h.id === recommendation.id
    )?.updatedAt;

    if (!implementationTime) {
      return { effective: false, monitoringComplete: false };
    }

    const timeSinceImplementation =
      Date.now() - new Date(implementationTime).getTime();
    const monitoringComplete =
      timeSinceImplementation >= this.config.monitoringPeriodMs;

    if (!monitoringComplete) {
      return { effective: false, monitoringComplete: false };
    }

    // In a real implementation, this would query actual metrics
    // For now, simulate based on expected impact
    const expectedImpact = recommendation.expectedImpact.improvementPercent;
    const actualImpact = expectedImpact * (0.8 + Math.random() * 0.4); // 80-120% of expected
    const effective = actualImpact > expectedImpact * 0.5; // At least 50% of expected improvement

    this.emit("improvement:monitored", {
      recommendationId: recommendation.id,
      targetEntity: recommendation.action.targetEntity,
      effective,
      actualImpact,
      expectedImpact,
    });

    return { effective, actualImpact, monitoringComplete: true };
  }

  private async executeImprovement(
    recommendation: FeedbackRecommendation
  ): Promise<boolean> {
    const { action } = recommendation;

    // Simulate improvement execution based on type
    switch (recommendation.type) {
      case "agent_update":
        return await this.executeAgentUpdate(
          action.targetEntity,
          action.parameters
        );

      case "routing_adjustment":
        return await this.executeRoutingAdjustment(
          action.targetEntity,
          action.parameters
        );

      case "resource_allocation":
        return await this.executeResourceAllocation(
          action.targetEntity,
          action.parameters
        );

      case "policy_change":
        return await this.executePolicyChange(
          action.targetEntity,
          action.parameters
        );

      case "system_configuration":
        return await this.executeSystemConfiguration(
          action.targetEntity,
          action.parameters
        );

      default:
        this.logger.warn(`Unknown recommendation type: ${recommendation.type}`);
        return false;
    }
  }

  private async executeAgentUpdate(
    entityId: string,
    params: Record<string, any>
  ): Promise<boolean> {
    // In real implementation, would update agent registry
    this.logger.info(`Updating agent ${entityId} with params:`, params);
    await this.delay(100); // Simulate async operation
    return Math.random() > 0.1; // 90% success rate
  }

  private async executeRoutingAdjustment(
    entityId: string,
    params: Record<string, any>
  ): Promise<boolean> {
    // In real implementation, would update task router
    this.logger.info(`Adjusting routing for ${entityId} with params:`, params);
    await this.delay(50);
    return Math.random() > 0.05; // 95% success rate
  }

  private async executeResourceAllocation(
    entityId: string,
    params: Record<string, any>
  ): Promise<boolean> {
    // In real implementation, would update resource manager
    this.logger.info(
      `Allocating resources for ${entityId} with params:`,
      params
    );
    await this.delay(200);
    return Math.random() > 0.15; // 85% success rate
  }

  private async executePolicyChange(
    entityId: string,
    params: Record<string, any>
  ): Promise<boolean> {
    // In real implementation, would update constitutional runtime
    this.logger.info(`Changing policy for ${entityId} with params:`, params);
    await this.delay(150);
    return Math.random() > 0.2; // 80% success rate
  }

  private async executeSystemConfiguration(
    entityId: string,
    params: Record<string, any>
  ): Promise<boolean> {
    // In real implementation, would update system configuration
    this.logger.info(
      `Updating system configuration for ${entityId} with params:`,
      params
    );
    await this.delay(300);
    return Math.random() > 0.1; // 90% success rate
  }

  private async rollbackImprovement(
    recommendation: FeedbackRecommendation
  ): Promise<void> {
    // In real implementation, would undo the changes
    this.logger.info(`Rolling back improvement ${recommendation.id}`);
    await this.delay(100);
  }

  private checkPrerequisites(recommendation: FeedbackRecommendation): boolean {
    if (!recommendation.prerequisites) return true;

    // In real implementation, would check actual system state
    // For now, assume prerequisites are met
    return true;
  }

  private isInCooldown(entityId: string): boolean {
    const cooldownEnd = this.cooldowns.get(entityId);
    return cooldownEnd ? new Date() < cooldownEnd : false;
  }

  private setCooldown(entityId: string): void {
    const cooldownEnd = new Date(Date.now() + this.config.cooldownPeriodMs);
    this.cooldowns.set(entityId, cooldownEnd);
  }

  private updateSuccessRate(operation: string, success: boolean): void {
    const stats = this.successRates.get(operation) || {
      successes: 0,
      attempts: 0,
    };
    stats.attempts++;
    if (success) stats.successes++;
    this.successRates.set(operation, stats);
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  public clearHistory(): void {
    this.improvementHistory = [];
    this.activeImprovements.clear();
    this.cooldowns.clear();
    this.successRates.clear();
  }

  public getStats() {
    return {
      activeImprovements: this.activeImprovements.size,
      totalImprovements: this.improvementHistory.length,
      successRates: this.getSuccessRates(),
      cooldownsActive: this.cooldowns.size,
    };
  }
}
