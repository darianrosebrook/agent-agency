/**
 * @fileoverview GPT-5 Prompting Engine - Main Coordination Component
 *
 * Central orchestrator for GPT-5 prompting techniques including reasoning effort control,
 * agent eagerness management, tool budgeting, and structured prompt processing.
 *
 * @author @darianrosebrook
 */

import {
  AgentControlConfig,
  AgentEagerness,
  ContextGatheringConfig,
  PromptingResult,
  ReasoningEffort,
  Task,
  TaskComplexity,
  TaskContext,
} from "../../types/agent-prompting";

import { AgentEagernessManager } from "./AgentEagernessManager";
import { ContextGatheringCoordinator } from "./ContextGatheringCoordinator";
import { ReasoningEffortController } from "./ReasoningEffortController";
import { SelfReflectionManager } from "./SelfReflectionManager";
import { ToolBudgetManager } from "./ToolBudgetManager";
import { XMLPromptProcessor } from "./XMLPromptProcessor";

/**
 * Prompting Engine Configuration
 */
export interface PromptingEngineConfig {
  /** Enable prompting optimizations */
  enabled: boolean;

  /** Reasoning effort controller settings */
  reasoningEffort: AgentControlConfig["reasoningEffort"];

  /** Agent eagerness settings */
  eagerness: AgentControlConfig["eagerness"];

  /** Tool budget management settings */
  toolBudget: AgentControlConfig["toolBudget"];

  /** Context gathering configuration */
  contextGathering: ContextGatheringConfig;

  /** Self-reflection settings */
  selfReflection: AgentControlConfig["selfReflection"];

  /** Performance monitoring */
  monitoring: {
    enableMetrics: boolean;
    enableTracing: boolean;
    metricsPrefix: string;
  };
}

/**
 * Prompting Engine Status
 */
export interface PromptingEngineStatus {
  /** Engine health */
  healthy: boolean;

  /** Component statuses */
  components: {
    reasoningController: boolean;
    eagernessManager: boolean;
    budgetManager: boolean;
    contextCoordinator: boolean;
    reflectionManager: boolean;
    xmlProcessor: boolean;
  };

  /** Active configurations */
  activeConfigs: {
    totalProcessed: number;
    currentComplexity: TaskComplexity;
    currentEffort: ReasoningEffort;
    activeBudgets: number;
  };
}

/**
 * Main GPT-5 Prompting Engine
 *
 * Orchestrates all prompting techniques for optimal agent behavior control.
 */
export class PromptingEngine {
  private config: PromptingEngineConfig;
  private reasoningController: ReasoningEffortController;
  private eagernessManager: AgentEagernessManager;
  private budgetManager: ToolBudgetManager;
  private contextCoordinator: ContextGatheringCoordinator;
  private reflectionManager: SelfReflectionManager;
  private xmlProcessor: XMLPromptProcessor;

  private processingStats = {
    totalProcessed: 0,
    averageProcessingTime: 0,
    successRate: 1.0,
    lastProcessedAt: new Date(),
  };

  /**
   * Create a new PromptingEngine instance
   */
  constructor(config: PromptingEngineConfig) {
    this.config = config;

    // Initialize all prompting components
    this.reasoningController = new ReasoningEffortController(
      config.reasoningEffort
    );

    this.eagernessManager = new AgentEagernessManager(config.eagerness);

    this.budgetManager = new ToolBudgetManager(config.toolBudget);

    this.contextCoordinator = new ContextGatheringCoordinator(
      config.contextGathering
    );

    this.reflectionManager = new SelfReflectionManager(config.selfReflection);

    this.xmlProcessor = new XMLPromptProcessor();
  }

  /**
   * Process a task to generate optimal prompting configuration
   *
   * This is the main entry point that coordinates all GPT-5 prompting techniques
   * to determine the optimal agent behavior for a given task.
   */
  async processTask(
    task: Task,
    context: TaskContext,
    xmlInstructions?: string
  ): Promise<PromptingResult> {
    const startTime = Date.now();

    try {
      // Step 1: Assess task characteristics
      const taskAssessment = await this.assessTaskCharacteristics(
        task,
        context
      );

      // Step 2: Select optimal reasoning effort
      const reasoningEffort =
        await this.reasoningController.selectOptimalEffort(
          task,
          context,
          taskAssessment
        );

      // Step 3: Calibrate agent eagerness
      const eagerness = await this.eagernessManager.calibrateEagerness(
        task.type,
        task.complexity,
        context
      );

      // Step 4: Allocate tool budget
      const toolBudget = await this.budgetManager.allocateBudget(
        task,
        taskAssessment
      );

      // Step 5: Configure context gathering
      const contextConfig = this.contextCoordinator.createConfig(
        task.complexity,
        reasoningEffort
      );

      // Step 6: Generate self-reflection rubric (if needed)
      const reflectionRubric =
        task.complexity === "expert" || task.complexity === "complex"
          ? await this.reflectionManager.createRubric(task, context)
          : undefined;

      // Step 7: Process XML instructions (if provided)
      const structuredInstructions = xmlInstructions
        ? await this.xmlProcessor.parseInstructions(xmlInstructions)
        : [];

      // Step 8: Apply optimizations based on task characteristics
      const optimizations = this.determineOptimizations(
        taskAssessment,
        context
      );

      // Calculate processing time
      const processingTime = Date.now() - startTime;

      // Update statistics
      this.updateProcessingStats(processingTime, true);

      // Record metrics if enabled
      if (this.config.monitoring.enableMetrics) {
        await this.recordMetrics(
          task,
          reasoningEffort,
          eagerness,
          processingTime
        );
      }

      return {
        reasoningEffort,
        eagerness,
        toolBudget,
        contextConfig,
        reflectionRubric,
        structuredInstructions,
        metadata: {
          processingTimeMs: processingTime,
          confidence: this.calculateConfidence(taskAssessment),
          appliedOptimizations: optimizations,
        },
      };
    } catch (error) {
      // Update statistics on failure
      this.updateProcessingStats(Date.now() - startTime, false);

      // Log error for monitoring
      console.error(
        `PromptingEngine: Failed to process task ${task.id}:`,
        error
      );

      // Return conservative defaults on failure
      return this.getConservativeDefaults(task);
    }
  }

  /**
   * Get engine status and health information
   */
  async getStatus(): Promise<PromptingEngineStatus> {
    const componentStatuses = await Promise.all([
      this.reasoningController.isHealthy(),
      this.eagernessManager.isHealthy(),
      this.budgetManager.isHealthy(),
      this.contextCoordinator.isHealthy(),
      this.reflectionManager.isHealthy(),
      this.xmlProcessor.isHealthy(),
    ]);

    return {
      healthy: componentStatuses.every((status) => status),
      components: {
        reasoningController: componentStatuses[0],
        eagernessManager: componentStatuses[1],
        budgetManager: componentStatuses[2],
        contextCoordinator: componentStatuses[3],
        reflectionManager: componentStatuses[4],
        xmlProcessor: componentStatuses[5],
      },
      activeConfigs: {
        totalProcessed: this.processingStats.totalProcessed,
        currentComplexity: "standard", // Would track current task complexity
        currentEffort: "medium", // Would track current effort level
        activeBudgets: await this.budgetManager.getActiveBudgetCount(),
      },
    };
  }

  /**
   * Update engine configuration
   */
  async updateConfig(newConfig: Partial<PromptingEngineConfig>): Promise<void> {
    this.config = { ...this.config, ...newConfig };

    // Update individual components with new config
    if (newConfig.reasoningEffort) {
      await this.reasoningController.updateConfig(newConfig.reasoningEffort);
    }

    if (newConfig.eagerness) {
      await this.eagernessManager.updateConfig(newConfig.eagerness);
    }

    if (newConfig.toolBudget) {
      await this.budgetManager.updateConfig(newConfig.toolBudget);
    }

    if (newConfig.contextGathering) {
      await this.contextCoordinator.updateConfig(newConfig.contextGathering);
    }

    if (newConfig.selfReflection) {
      await this.reflectionManager.updateConfig(newConfig.selfReflection);
    }
  }

  /**
   * Assess task characteristics for prompting decisions
   */
  private async assessTaskCharacteristics(
    task: Task,
    context: TaskContext
  ): Promise<TaskAssessment> {
    // Analyze task description for complexity indicators
    const complexityIndicators = this.analyzeComplexityIndicators(
      task.description
    );

    // Consider historical performance if available
    const historicalAdjustment = context.historicalMetrics
      ? this.calculateHistoricalAdjustment(context.historicalMetrics)
      : 0;

    // Assess time pressure
    const timePressure = context.timeBudgetMs
      ? this.assessTimePressure(context.timeBudgetMs, task.complexity)
      : "normal";

    return {
      adjustedComplexity: this.adjustComplexity(
        task.complexity,
        complexityIndicators,
        historicalAdjustment
      ),
      timePressure,
      riskLevel: this.assessRiskLevel(task, context),
      optimizationOpportunities: this.identifyOptimizations(task, context),
    };
  }

  /**
   * Analyze task description for complexity indicators
   */
  private analyzeComplexityIndicators(
    description: string
  ): ComplexityIndicators {
    const words = description.toLowerCase().split(/\s+/);

    return {
      technicalTerms: this.countTechnicalTerms(words),
      conditionalLogic: this.detectConditionalLogic(description),
      multiStep: this.detectMultiStep(description),
      researchRequired: this.detectResearchRequirements(description),
    };
  }

  /**
   * Calculate historical performance adjustment
   */
  private calculateHistoricalAdjustment(
    metrics: TaskContext["historicalMetrics"]
  ): number {
    if (!metrics) return 0;

    const { successRate, toolEfficiency } = metrics;

    // Adjust complexity based on historical performance
    // Higher success rate suggests task might be simpler than classified
    // Higher tool efficiency suggests good optimization opportunities
    return (successRate - 0.8) * 0.2 + (toolEfficiency - 0.7) * 0.1;
  }

  /**
   * Assess time pressure impact
   */
  private assessTimePressure(
    timeBudget: number,
    complexity: TaskComplexity
  ): "low" | "normal" | "high" {
    const estimatedTime = this.getEstimatedTaskTime(complexity);

    if (timeBudget < estimatedTime * 0.7) return "high";
    if (timeBudget > estimatedTime * 1.5) return "low";
    return "normal";
  }

  /**
   * Get estimated task completion time
   */
  private getEstimatedTaskTime(complexity: TaskComplexity): number {
    const baseTimes = {
      trivial: 30000, // 30 seconds
      standard: 300000, // 5 minutes
      complex: 1800000, // 30 minutes
      expert: 7200000, // 2 hours
    };

    return baseTimes[complexity] || baseTimes.standard;
  }

  /**
   * Assess risk level for prompting decisions
   */
  private assessRiskLevel(
    task: Task,
    context: TaskContext
  ): "low" | "medium" | "high" {
    let riskScore = 0;

    // Increase risk for complex tasks
    if (task.complexity === "expert") riskScore += 2;
    else if (task.complexity === "complex") riskScore += 1;

    // Increase risk for high accuracy requirements
    if (context.accuracyRequirement === "critical") riskScore += 2;
    else if (context.accuracyRequirement === "high") riskScore += 1;

    // Increase risk for short time budgets
    if (context.timeBudgetMs && context.timeBudgetMs < 60000) riskScore += 1;

    if (riskScore >= 3) return "high";
    if (riskScore >= 1) return "medium";
    return "low";
  }

  /**
   * Identify optimization opportunities
   */
  private identifyOptimizations(task: Task, context: TaskContext): string[] {
    const optimizations: string[] = [];

    if (task.complexity === "trivial") {
      optimizations.push("minimal-reasoning-effort");
      optimizations.push("reduced-tool-budget");
    }

    if (context.timeBudgetMs && context.timeBudgetMs < 120000) {
      optimizations.push("parallel-context-gathering");
      optimizations.push("early-stop-criteria");
    }

    if (
      context.historicalMetrics?.successRate &&
      context.historicalMetrics.successRate > 0.9
    ) {
      optimizations.push("trust-historical-performance");
    }

    if (task.type === "research" || task.type === "analysis") {
      optimizations.push("optimized-context-gathering");
    }

    return optimizations;
  }

  /**
   * Adjust task complexity based on analysis
   */
  private adjustComplexity(
    baseComplexity: TaskComplexity,
    indicators: ComplexityIndicators,
    historicalAdjustment: number
  ): TaskComplexity {
    let complexityScore = this.complexityToScore(baseComplexity);

    // Adjust based on indicators
    complexityScore += indicators.technicalTerms * 0.1;
    complexityScore += indicators.conditionalLogic ? 0.3 : 0;
    complexityScore += indicators.multiStep ? 0.2 : 0;
    complexityScore += indicators.researchRequired ? 0.4 : 0;

    // Apply historical adjustment
    complexityScore += historicalAdjustment;

    // Ensure bounds
    complexityScore = Math.max(0, Math.min(3, complexityScore));

    return this.scoreToComplexity(complexityScore);
  }

  /**
   * Convert complexity to numeric score
   */
  private complexityToScore(complexity: TaskComplexity): number {
    const scores = { trivial: 0, standard: 1, complex: 2, expert: 3 };
    return scores[complexity] || 1;
  }

  /**
   * Convert numeric score to complexity
   */
  private scoreToComplexity(score: number): TaskComplexity {
    if (score < 0.5) return "trivial";
    if (score < 1.5) return "standard";
    if (score < 2.5) return "complex";
    return "expert";
  }

  /**
   * Count technical terms in description
   */
  private countTechnicalTerms(words: string[]): number {
    const technicalTerms = [
      "algorithm",
      "api",
      "async",
      "authentication",
      "cache",
      "concurrency",
      "database",
      "encryption",
      "framework",
      "graphql",
      "http",
      "https",
      "inheritance",
      "interface",
      "middleware",
      "microservice",
      "oauth",
      "optimization",
      "polymorphism",
      "protocol",
      "query",
      "rest",
      "schema",
      "security",
      "serialization",
      "session",
      "synchronization",
      "transaction",
      "validation",
      "virtualization",
      "websocket",
    ];

    return words.filter((word) =>
      technicalTerms.some((term) => word.includes(term))
    ).length;
  }

  /**
   * Detect conditional logic patterns
   */
  private detectConditionalLogic(description: string): boolean {
    const patterns = [
      /\b(if|when|unless|provided|assuming|given)\b/i,
      /\b(should|must|may|can|cannot)\b.*\b(if|when|unless)\b/i,
      /\b(depending on|based on|according to)\b/i,
    ];

    return patterns.some((pattern) => pattern.test(description));
  }

  /**
   * Detect multi-step processes
   */
  private detectMultiStep(description: string): boolean {
    const patterns = [
      /\b(first|second|third|next|then|after|finally)\b/i,
      /\b(step|phase|stage)\s+\d+/i,
      /\b(begin|end|start|finish|complete)\b.*\b(and|then|next)\b/i,
    ];

    return patterns.some((pattern) => pattern.test(description));
  }

  /**
   * Detect research requirements
   */
  private detectResearchRequirements(description: string): boolean {
    const patterns = [
      /\b(research|investigate|analyze|explore|discover|find out)\b/i,
      /\b(what|how|why|when|where|who)\s+.*\?/i,
      /\b(understand|learn|determine|identify|examine)\b/i,
    ];

    return patterns.some((pattern) => pattern.test(description));
  }

  /**
   * Determine applied optimizations
   */
  private determineOptimizations(
    assessment: TaskAssessment,
    context: TaskContext
  ): string[] {
    const optimizations: string[] = [];

    if (assessment.adjustedComplexity === "trivial") {
      optimizations.push("low-reasoning-effort");
      optimizations.push("minimal-tool-budget");
    }

    if (assessment.timePressure === "high") {
      optimizations.push("parallel-context-gathering");
      optimizations.push("reduced-depth-limits");
    }

    if (assessment.riskLevel === "high") {
      optimizations.push("high-reasoning-effort");
      optimizations.push("self-reflection-enabled");
    }

    return optimizations;
  }

  /**
   * Calculate confidence score for prompting decisions
   */
  private calculateConfidence(assessment: TaskAssessment): number {
    let confidence = 0.8; // Base confidence

    // Reduce confidence for high-risk tasks
    if (assessment.riskLevel === "high") confidence -= 0.2;

    // Increase confidence for clear optimization opportunities
    if (assessment.optimizationOpportunities.length > 2) confidence += 0.1;

    // Ensure bounds
    return Math.max(0.1, Math.min(1.0, confidence));
  }

  /**
   * Update processing statistics
   */
  private updateProcessingStats(
    processingTime: number,
    success: boolean
  ): void {
    this.processingStats.totalProcessed++;

    // Update average processing time
    const totalTime =
      this.processingStats.averageProcessingTime *
      (this.processingStats.totalProcessed - 1);
    this.processingStats.averageProcessingTime =
      (totalTime + processingTime) / this.processingStats.totalProcessed;

    // Update success rate
    const successCount = success ? 1 : 0;
    const previousSuccesses =
      this.processingStats.successRate *
      (this.processingStats.totalProcessed - 1);
    this.processingStats.successRate =
      (previousSuccesses + successCount) / this.processingStats.totalProcessed;

    this.processingStats.lastProcessedAt = new Date();
  }

  /**
   * Record metrics for monitoring
   */
  private async recordMetrics(
    task: Task,
    effort: ReasoningEffort,
    eagerness: AgentEagerness,
    processingTime: number
  ): Promise<void> {
    // This would integrate with your metrics system
    // For now, we'll just log the key metrics
    console.log(`PromptingEngine: Task ${task.id} processed`, {
      effort,
      eagerness,
      processingTime,
      complexity: task.complexity,
      type: task.type,
    });
  }

  /**
   * Get conservative default configuration for error recovery
   */
  private getConservativeDefaults(task: Task): PromptingResult {
    return {
      reasoningEffort: "medium",
      eagerness: "balanced",
      toolBudget: {
        maxCalls: 10,
        usedCalls: 0,
        resetIntervalMs: 3600000, // 1 hour
        lastResetAt: new Date(),
        escalationRules: [],
      },
      contextConfig: this.config.contextGathering,
      structuredInstructions: [],
      metadata: {
        processingTimeMs: 0,
        confidence: 0.5,
        appliedOptimizations: ["conservative-fallback"],
      },
    };
  }
}

/**
 * Internal task assessment result
 */
interface TaskAssessment {
  adjustedComplexity: TaskComplexity;
  timePressure: "low" | "normal" | "high";
  riskLevel: "low" | "medium" | "high";
  optimizationOpportunities: string[];
}

/**
 * Complexity analysis indicators
 */
interface ComplexityIndicators {
  technicalTerms: number;
  conditionalLogic: boolean;
  multiStep: boolean;
  researchRequired: boolean;
}
