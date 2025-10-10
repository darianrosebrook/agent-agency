/**
 * Multi-Model AI Orchestrator - Intelligent model selection and routing
 *
 * @author @darianrosebrook
 * @description Orchestrates multiple AI models with intelligent routing based on task requirements
 */

import { Logger } from "../utils/Logger.js";
import type {
  AIModelClient,
  GenerateRequest,
  GenerateResponse,
} from "./types.js";

export interface ModelCapability {
  name: string;
  client: AIModelClient;
  strengths: string[]; // e.g., ["code_generation", "analysis", "creative_writing"]
  costPerToken: number;
  maxTokens: number;
  contextWindow: number;
  supportsToolCalling: boolean;
  priority: number; // Higher priority = preferred for matching tasks
}

export interface OrchestratorConfig {
  defaultModel: string;
  fallbackModels: string[];
  enableCostOptimization: boolean;
  enableQualityRouting: boolean;
  maxRetries: number;
  timeout: number;
}

export interface ModelSelectionCriteria {
  taskType?: string; // e.g., "code_generation", "analysis", "creative"
  requiredCapabilities?: string[];
  maxCost?: number;
  minQuality?: number;
  preferredModels?: string[];
  excludeModels?: string[];
  contextLength?: number;
  requiresToolCalling?: boolean;
}

export class MultiModelOrchestrator implements AIModelClient {
  private models: Map<string, ModelCapability> = new Map();
  private config: OrchestratorConfig;
  private logger: Logger;
  private performanceMetrics: Map<string, ModelPerformance> = new Map();

  constructor(config: OrchestratorConfig, logger?: Logger) {
    this.config = config;
    this.logger = logger || new Logger("MultiModelOrchestrator");
  }

  registerModel(model: ModelCapability): void {
    this.models.set(model.name, model);
    this.performanceMetrics.set(model.name, {
      totalRequests: 0,
      successfulRequests: 0,
      averageLatency: 0,
      averageCost: 0,
      errorRate: 0,
      lastUsed: null,
    });

    this.logger.info(`Model registered: ${model.name}`, {
      strengths: model.strengths,
      costPerToken: model.costPerToken,
      supportsToolCalling: model.supportsToolCalling,
    });
  }

  async generate(request: GenerateRequest): Promise<GenerateResponse> {
    const startTime = Date.now();

    try {
      // Select the best model for this request
      const selectedModel = this.selectModel(request);

      if (!selectedModel) {
        throw new Error("No suitable model available for the request");
      }

      this.logger.debug("Selected model for request", {
        model: selectedModel.name,
        promptLength: request.prompt.length,
        hasSystemPrompt: !!request.systemPrompt,
      });

      // Execute the request with retry logic
      let lastError: Error | null = null;
      let currentModel = selectedModel;

      for (let attempt = 0; attempt <= this.config.maxRetries; attempt++) {
        try {
          const response = await currentModel.client.generate(request);

          // Record performance metrics
          this.recordPerformance(currentModel.name, startTime, true, response);

          return response;
        } catch (error) {
          lastError = error as Error;

          this.logger.warn(
            `Model ${currentModel.name} attempt ${attempt + 1} failed`,
            {
              error: lastError.message,
            }
          );

          // Record failed attempt
          this.recordPerformance(currentModel.name, startTime, false);

          // Try fallback model on failure
          if (attempt < this.config.maxRetries) {
            const fallbackModel = this.selectFallbackModel(
              request,
              currentModel.name
            );
            if (fallbackModel) {
              this.logger.info(
                `Switching to fallback model: ${fallbackModel.name}`
              );
              currentModel = fallbackModel;
              continue;
            }
          }

          // Wait before retry (exponential backoff)
          if (attempt < this.config.maxRetries) {
            const delay = 1000 * Math.pow(2, attempt);
            await new Promise((resolve) => setTimeout(resolve, delay));
          }
        }
      }

      throw lastError || new Error("All model attempts failed");
    } catch (error) {
      this.logger.error("Multi-model orchestration failed", {
        error: (error as Error).message,
        duration: Date.now() - startTime,
      });
      throw error;
    }
  }

  private selectModel(request: GenerateRequest): ModelCapability | null {
    const criteria: ModelSelectionCriteria =
      this.extractCriteriaFromRequest(request);

    // Get available models
    const availableModels = Array.from(this.models.values()).filter((model) =>
      model.client.isAvailable()
    );

    if (availableModels.length === 0) {
      return null;
    }

    // Score and rank models
    const scoredModels = availableModels.map((model) => ({
      model,
      score: this.scoreModel(model, criteria),
    }));

    scoredModels.sort((a, b) => b.score - a.score);

    const bestModel = scoredModels[0]?.model;

    this.logger.debug("Model selection completed", {
      selectedModel: bestModel?.name,
      scores: scoredModels
        .slice(0, 3)
        .map((s) => `${s.model.name}: ${s.score.toFixed(2)}`),
    });

    return bestModel || null;
  }

  private selectFallbackModel(
    request: GenerateRequest,
    failedModel: string
  ): ModelCapability | null {
    const criteria: ModelSelectionCriteria =
      this.extractCriteriaFromRequest(request);
    criteria.excludeModels = [failedModel];

    // Use fallback models from config
    const fallbackModels = this.config.fallbackModels
      .filter((name) => name !== failedModel && this.models.has(name))
      .map((name) => this.models.get(name)!)
      .filter((model) => model.client.isAvailable());

    if (fallbackModels.length === 0) {
      return null;
    }

    // Score fallback models
    const scoredModels = fallbackModels.map((model) => ({
      model,
      score: this.scoreModel(model, criteria),
    }));

    scoredModels.sort((a, b) => b.score - a.score);

    return scoredModels[0]?.model || null;
  }

  private extractCriteriaFromRequest(
    request: GenerateRequest
  ): ModelSelectionCriteria {
    const criteria: ModelSelectionCriteria = {};

    // Analyze prompt for task type hints
    const prompt = (request.systemPrompt || "") + " " + request.prompt;
    const lowerPrompt = prompt.toLowerCase();

    if (
      lowerPrompt.includes("code") ||
      lowerPrompt.includes("function") ||
      lowerPrompt.includes("class")
    ) {
      criteria.taskType = "code_generation";
    } else if (
      lowerPrompt.includes("analyze") ||
      lowerPrompt.includes("explain") ||
      lowerPrompt.includes("review")
    ) {
      criteria.taskType = "analysis";
    } else if (
      lowerPrompt.includes("write") ||
      lowerPrompt.includes("create") ||
      lowerPrompt.includes("design")
    ) {
      criteria.taskType = "creative";
    }

    // Check for tool calling requirements
    if (lowerPrompt.includes("tool") || lowerPrompt.includes("function_call")) {
      criteria.requiresToolCalling = true;
    }

    // Estimate context length
    criteria.contextLength = prompt.length;

    return criteria;
  }

  private scoreModel(
    model: ModelCapability,
    criteria: ModelSelectionCriteria
  ): number {
    let score = model.priority; // Base priority

    // Task type matching
    if (criteria.taskType && model.strengths.includes(criteria.taskType)) {
      score += 20;
    }

    // Required capabilities
    if (criteria.requiredCapabilities) {
      const matchingCapabilities = criteria.requiredCapabilities.filter((cap) =>
        model.strengths.includes(cap)
      );
      score += matchingCapabilities.length * 15;
    }

    // Tool calling requirement
    if (criteria.requiresToolCalling) {
      score += model.supportsToolCalling ? 25 : -50;
    }

    // Cost optimization
    if (this.config.enableCostOptimization && criteria.maxCost) {
      const estimatedCost =
        ((criteria.contextLength || 1000) * model.costPerToken) / 1000;
      if (estimatedCost <= criteria.maxCost) {
        score += 10;
      } else {
        score -= 20;
      }
    }

    // Context window check
    if (
      criteria.contextLength &&
      criteria.contextLength > model.contextWindow * 0.8
    ) {
      score -= 30; // Penalize models that might exceed context window
    }

    // Performance bonus (prefer recently successful models)
    const metrics = this.performanceMetrics.get(model.name);
    if (metrics && metrics.successfulRequests > 0) {
      const successRate = metrics.successfulRequests / metrics.totalRequests;
      score += successRate * 10;
    }

    // Exclude specific models
    if (criteria.excludeModels?.includes(model.name)) {
      score = -999;
    }

    // Prefer specific models
    if (criteria.preferredModels?.includes(model.name)) {
      score += 15;
    }

    return score;
  }

  private recordPerformance(
    modelName: string,
    startTime: number,
    success: boolean,
    response?: GenerateResponse
  ): void {
    const metrics = this.performanceMetrics.get(modelName);
    if (!metrics) return;

    metrics.totalRequests++;
    if (success) {
      metrics.successfulRequests++;
    }

    const duration = Date.now() - startTime;
    metrics.averageLatency =
      (metrics.averageLatency * (metrics.totalRequests - 1) + duration) /
      metrics.totalRequests;

    if (response?.usage) {
      const cost =
        (response.usage.totalTokens *
          (this.models.get(modelName)?.costPerToken || 0)) /
        1000;
      metrics.averageCost =
        (metrics.averageCost * (metrics.totalRequests - 1) + cost) /
        metrics.totalRequests;
    }

    metrics.errorRate =
      (metrics.totalRequests - metrics.successfulRequests) /
      metrics.totalRequests;
    metrics.lastUsed = new Date();
  }

  supportsToolCalling(): boolean {
    // Check if any registered model supports tool calling
    return Array.from(this.models.values()).some(
      (model) => model.supportsToolCalling
    );
  }

  getModelName(): string {
    // Return the default model name
    const defaultModel = this.models.get(this.config.defaultModel);
    return defaultModel?.name || "multi-model-orchestrator";
  }

  async isAvailable(): Promise<boolean> {
    // Check if at least one model is available
    for (const model of this.models.values()) {
      if (await model.client.isAvailable()) {
        return true;
      }
    }
    return false;
  }

  getPerformanceMetrics(): Map<string, ModelPerformance> {
    return new Map(this.performanceMetrics);
  }

  getRegisteredModels(): string[] {
    return Array.from(this.models.keys());
  }
}

interface ModelPerformance {
  totalRequests: number;
  successfulRequests: number;
  averageLatency: number;
  averageCost: number;
  errorRate: number;
  lastUsed: Date | null;
}
