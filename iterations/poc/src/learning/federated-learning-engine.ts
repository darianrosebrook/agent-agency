/**
 * Federated Learning Engine
 *
 * Enables privacy-preserving learning across multiple tenants without
 * exposing individual tenant data. Uses differential privacy techniques
 * to aggregate insights while maintaining confidentiality.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { MultiTenantMemoryManager } from "../memory/MultiTenantMemoryManager";

export interface PrivacyParameters {
  epsilon: number; // Differential privacy parameter (smaller = more private)
  delta: number; // Privacy failure probability
  noiseScale: number; // Amount of noise to add
  clippingThreshold: number; // Gradient clipping for privacy
}

export interface FederatedLearningTask {
  id: string;
  name: string;
  description: string;
  domain: string;
  learningObjective: string;
  privacyParams: PrivacyParameters;
  minimumParticipants: number;
  maximumRounds: number;
  aggregationMethod: "fedavg" | "fednova" | "scaffold";
  created: Date;
  status: "pending" | "active" | "completed" | "failed";
}

export interface ParticipantUpdate {
  tenantId: string;
  taskId: string;
  round: number;
  localModel: any; // Privacy-preserved model updates
  sampleCount: number;
  qualityMetrics: {
    loss: number;
    accuracy: number;
    privacyBudgetUsed: number;
  };
  timestamp: Date;
}

export interface GlobalModel {
  taskId: string;
  round: number;
  aggregatedModel: any;
  participantCount: number;
  qualityMetrics: {
    globalLoss: number;
    globalAccuracy: number;
    privacyBudgetRemaining: number;
  };
  created: Date;
}

export interface LearningPattern {
  id: string;
  pattern: string;
  confidence: number;
  support: number; // Number of tenants that exhibit this pattern
  privacyScore: number; // How well the pattern preserves privacy
  quality: number; // Quality/usefulness of the pattern
  domain: string;
  discoveredBy: string[]; // Participating tenant IDs (anonymized)
  created: Date;
}

/**
 * Federated Learning Engine for cross-tenant learning
 */
export class FederatedLearningEngine extends EventEmitter {
  private activeTasks = new Map<string, FederatedLearningTask>();
  private participantUpdates = new Map<string, ParticipantUpdate[]>();
  private globalModels = new Map<string, GlobalModel[]>();
  private discoveredPatterns: LearningPattern[] = [];
  private memoryManager: MultiTenantMemoryManager;

  constructor(memoryManager: MultiTenantMemoryManager) {
    super();
    this.memoryManager = memoryManager;
  }

  /**
   * Create a new federated learning task
   */
  async createTask(
    tenantId: string,
    task: Omit<FederatedLearningTask, "id" | "created" | "status">
  ): Promise<FederatedLearningTask> {
    const taskId = `fed-task-${Date.now()}-${Math.random()
      .toString(36)
      .substr(2, 9)}`;
    const federatedTask: FederatedLearningTask = {
      ...task,
      id: taskId,
      created: new Date(),
      status: "pending",
    };

    this.activeTasks.set(taskId, federatedTask);
    this.participantUpdates.set(taskId, []);

    // Store task in memory
    await this.memoryManager.storeExperience(tenantId, {
      memoryId: `fed-task-${taskId}`,
      relevanceScore: 1.0,
      contextMatch: {
        similarityScore: 1.0,
        keywordMatches: ["federated", "learning", "task", taskId],
        semanticMatches: [
          "federated learning",
          "privacy-preserving",
          task.domain,
        ],
        temporalAlignment: 1.0,
      },
      content: {
        type: "federated-learning-task",
        taskId,
        data: federatedTask,
      },
    });

    this.emit("task-created", federatedTask);
    return federatedTask;
  }

  /**
   * Submit a privacy-preserved update from a participating tenant
   */
  async submitUpdate(
    tenantId: string,
    update: Omit<ParticipantUpdate, "tenantId" | "timestamp">
  ): Promise<boolean> {
    const task = this.activeTasks.get(update.taskId);
    if (!task || task.status !== "active") {
      return false;
    }

    const participantUpdate: ParticipantUpdate = {
      ...update,
      tenantId,
      timestamp: new Date(),
    };

    // Add differential privacy noise
    participantUpdate.localModel = this.addDifferentialPrivacyNoise(
      participantUpdate.localModel,
      task.privacyParams
    );

    // Clip gradients for additional privacy
    participantUpdate.localModel = this.clipGradients(
      participantUpdate.localModel,
      task.privacyParams.clippingThreshold
    );

    // Store the privacy-preserved update
    const updates = this.participantUpdates.get(update.taskId) || [];
    updates.push(participantUpdate);
    this.participantUpdates.set(update.taskId, updates);

    // Store in tenant's memory (privacy-preserved)
    await this.memoryManager.storeExperience(tenantId, {
      memoryId: `fed-update-${update.taskId}-${update.round}`,
      relevanceScore: 0.8,
      contextMatch: {
        similarityScore: 0.8,
        keywordMatches: ["federated", "update", update.taskId],
        semanticMatches: ["federated learning", "participant update"],
        temporalAlignment: 0.9,
      },
      content: {
        type: "federated-update",
        taskId: update.taskId,
        round: update.round,
        data: participantUpdate,
      },
    });

    this.emit("update-submitted", { task, update: participantUpdate });

    // Check if we have enough updates to aggregate
    await this.checkAggregationReady(update.taskId);

    return true;
  }

  /**
   * Aggregate participant updates into a new global model
   */
  private async checkAggregationReady(taskId: string): Promise<void> {
    const task = this.activeTasks.get(taskId);
    const updates = this.participantUpdates.get(taskId) || [];

    if (!task) return;

    // Group updates by round
    const roundUpdates = updates.filter((u) => u.round === task.maximumRounds);

    if (roundUpdates.length >= task.minimumParticipants) {
      await this.aggregateUpdates(task, roundUpdates);
    }
  }

  /**
   * Aggregate updates using federated averaging
   */
  private async aggregateUpdates(
    task: FederatedLearningTask,
    roundUpdates: ParticipantUpdate[]
  ): Promise<void> {
    const totalSamples = roundUpdates.reduce(
      (sum, update) => sum + update.sampleCount,
      0
    );
    const globalModels = this.globalModels.get(task.id) || [];
    const currentRound = globalModels.length;

    // Federated Averaging (FedAvg)
    const aggregatedModel: any = {};

    if (task.aggregationMethod === "fedavg") {
      // Weight each participant's model by their sample count
      const modelKeys = new Set<string>();
      roundUpdates.forEach((update) => {
        Object.keys(update.localModel).forEach((key) => modelKeys.add(key));
      });

      for (const key of modelKeys) {
        let weightedSum = 0;
        let totalWeight = 0;

        roundUpdates.forEach((update) => {
          if (update.localModel[key] !== undefined) {
            const weight = update.sampleCount / totalSamples;
            weightedSum += update.localModel[key] * weight;
            totalWeight += weight;
          }
        });

        aggregatedModel[key] = totalWeight > 0 ? weightedSum / totalWeight : 0;
      }
    }

    // Calculate global metrics
    const avgLoss =
      roundUpdates.reduce((sum, u) => sum + u.qualityMetrics.loss, 0) /
      roundUpdates.length;
    const avgAccuracy =
      roundUpdates.reduce((sum, u) => sum + u.qualityMetrics.accuracy, 0) /
      roundUpdates.length;
    const totalPrivacyUsed = roundUpdates.reduce(
      (sum, u) => sum + u.qualityMetrics.privacyBudgetUsed,
      0
    );

    const globalModel: GlobalModel = {
      taskId: task.id,
      round: currentRound + 1,
      aggregatedModel,
      participantCount: roundUpdates.length,
      qualityMetrics: {
        globalLoss: avgLoss,
        globalAccuracy: avgAccuracy,
        privacyBudgetRemaining: task.privacyParams.epsilon - totalPrivacyUsed,
      },
      created: new Date(),
    };

    globalModels.push(globalModel);
    this.globalModels.set(task.id, globalModels);

    // Store global model (shared across all participants)
    for (const update of roundUpdates) {
      await this.memoryManager.storeExperience(update.tenantId, {
        memoryId: `global-model-${task.id}-${globalModel.round}`,
        relevanceScore: 0.95,
        contextMatch: {
          similarityScore: 0.95,
          keywordMatches: ["global", "model", task.id],
          semanticMatches: [
            "federated learning",
            "aggregated model",
            "global knowledge",
          ],
          temporalAlignment: 0.9,
        },
        content: {
          type: "global-model",
          taskId: task.id,
          round: globalModel.round,
          data: globalModel,
        },
      });
    }

    this.emit("model-aggregated", {
      task,
      globalModel,
      participants: roundUpdates.length,
    });

    // Check for learning patterns
    await this.discoverPatterns(task, globalModel);
  }

  /**
   * Discover common patterns from federated learning
   */
  private async discoverPatterns(
    task: FederatedLearningTask,
    globalModel: GlobalModel
  ): Promise<void> {
    // Analyze the global model for patterns
    const patterns = this.analyzeModelForPatterns(
      globalModel.aggregatedModel,
      task.domain
    );

    for (const pattern of patterns) {
      const learningPattern: LearningPattern = {
        id: `pattern-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        pattern: pattern.pattern,
        confidence: pattern.confidence,
        support: globalModel.participantCount,
        privacyScore: this.calculatePrivacyScore(pattern, task.privacyParams),
        quality: globalModel.qualityMetrics.globalAccuracy,
        domain: task.domain,
        discoveredBy: [], // Anonymized - we don't store actual tenant IDs
        created: new Date(),
      };

      this.discoveredPatterns.push(learningPattern);

      this.emit("pattern-discovered", { task, pattern: learningPattern });
    }
  }

  /**
   * Analyze model for learning patterns
   */
  private analyzeModelForPatterns(
    model: any,
    domain: string
  ): Array<{ pattern: string; confidence: number }> {
    const patterns: Array<{ pattern: string; confidence: number }> = [];

    // Domain-specific pattern analysis
    if (domain === "code-review") {
      // Look for common code review patterns
      if (model.errorHandling && model.errorHandling > 0.8) {
        patterns.push({
          pattern: "consistent-error-handling",
          confidence: model.errorHandling,
        });
      }

      if (model.typeSafety && model.typeSafety > 0.7) {
        patterns.push({
          pattern: "strong-type-checking",
          confidence: model.typeSafety,
        });
      }
    } else if (domain === "testing") {
      if (model.coverage && model.coverage > 0.8) {
        patterns.push({
          pattern: "comprehensive-test-coverage",
          confidence: model.coverage,
        });
      }
    }

    return patterns;
  }

  /**
   * Add differential privacy noise to model updates
   */
  private addDifferentialPrivacyNoise(
    model: any,
    privacy: PrivacyParameters
  ): any {
    const noisyModel = { ...model };

    // Add Laplacian noise to each parameter
    Object.keys(noisyModel).forEach((key) => {
      const sensitivity = 1.0; // Assumed sensitivity
      const noise = this.generateLaplacianNoise(sensitivity / privacy.epsilon);
      noisyModel[key] += noise * privacy.noiseScale;
    });

    return noisyModel;
  }

  /**
   * Clip gradients to bound sensitivity
   */
  private clipGradients(model: any, threshold: number): any {
    const clippedModel = { ...model };

    Object.keys(clippedModel).forEach((key) => {
      const value = clippedModel[key];
      if (Math.abs(value) > threshold) {
        clippedModel[key] = Math.sign(value) * threshold;
      }
    });

    return clippedModel;
  }

  /**
   * Generate Laplacian noise for differential privacy
   */
  private generateLaplacianNoise(scale: number): number {
    // Generate Laplacian distributed noise
    const u = Math.random() - 0.5;
    const noise = -scale * Math.sign(u) * Math.log(1 - 2 * Math.abs(u));
    return noise;
  }

  /**
   * Calculate privacy score for a pattern
   */
  private calculatePrivacyScore(
    pattern: { pattern: string; confidence: number },
    privacy: PrivacyParameters
  ): number {
    // Higher epsilon = less privacy, lower privacy score
    // Higher confidence = more reliable pattern
    const privacyFactor = Math.min(1.0, privacy.epsilon / 1.0); // Normalize epsilon
    const confidenceFactor = pattern.confidence;

    return (privacyFactor + confidenceFactor) / 2;
  }

  /**
   * Get current global model for a task
   */
  getGlobalModel(taskId: string): GlobalModel | null {
    const models = this.globalModels.get(taskId);
    return models && models.length > 0 ? models[models.length - 1] : null;
  }

  /**
   * Get discovered learning patterns
   */
  getDiscoveredPatterns(domain?: string): LearningPattern[] {
    return domain
      ? this.discoveredPatterns.filter((p) => p.domain === domain)
      : this.discoveredPatterns;
  }

  /**
   * Get task status and progress
   */
  getTaskStatus(taskId: string): {
    task: FederatedLearningTask | null;
    participants: number;
    roundsCompleted: number;
    currentRound: number;
  } | null {
    const task = this.activeTasks.get(taskId);
    if (!task) return null;

    const updates = this.participantUpdates.get(taskId) || [];
    const models = this.globalModels.get(taskId) || [];

    // Count unique participants
    const participants = new Set(updates.map((u) => u.tenantId)).size;

    return {
      task,
      participants,
      roundsCompleted: models.length,
      currentRound: models.length + 1,
    };
  }

  /**
   * Get federated learning analytics
   */
  getAnalytics(): {
    activeTasks: number;
    completedTasks: number;
    totalParticipants: number;
    discoveredPatterns: number;
    averagePrivacyBudget: number;
    averageModelQuality: number;
  } {
    const tasks = Array.from(this.activeTasks.values());
    const allUpdates = Array.from(this.participantUpdates.values()).flat();
    const allModels = Array.from(this.globalModels.values()).flat();

    const activeTasks = tasks.filter((t) => t.status === "active").length;
    const completedTasks = tasks.filter((t) => t.status === "completed").length;
    const totalParticipants = new Set(allUpdates.map((u) => u.tenantId)).size;

    const avgPrivacyBudget =
      allModels.length > 0
        ? allModels.reduce(
            (sum, m) => sum + m.qualityMetrics.privacyBudgetRemaining,
            0
          ) / allModels.length
        : 0;

    const avgQuality =
      allModels.length > 0
        ? allModels.reduce(
            (sum, m) => sum + m.qualityMetrics.globalAccuracy,
            0
          ) / allModels.length
        : 0;

    return {
      activeTasks,
      completedTasks,
      totalParticipants,
      discoveredPatterns: this.discoveredPatterns.length,
      averagePrivacyBudget: avgPrivacyBudget,
      averageModelQuality: avgQuality,
    };
  }
}
