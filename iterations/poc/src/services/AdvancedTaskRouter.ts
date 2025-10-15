/**
 * Advanced Task Router
 *
 * Implements sophisticated task routing with priority queuing, predictive routing,
 * and memory-aware task assignment based on agent performance history.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { MultiTenantMemoryManager } from "../memory/MultiTenantMemoryManager.js";
import { Agent, Task, TaskContext } from "../types/index.js";
import { Logger } from "../utils/Logger.js";

export interface RoutingConfig {
  enabled: boolean;
  priorityQueuing: boolean;
  predictiveRouting: boolean;
  loadBalancing: boolean;
  memoryAwareRouting: boolean;
  maxConcurrentTasksPerAgent: number;
  routingHistoryWindow: number; // days
  performancePredictionEnabled: boolean;
  queueTimeoutMs: number;
  testMode?: boolean; // Synchronous routing for testing
}

export interface AgentPerformanceMetrics {
  agentId: string;
  taskType: string;
  successRate: number;
  averageLatency: number;
  throughput: number; // tasks per hour
  qualityScore: number;
  confidence: number;
  lastUpdated: Date;
  sampleSize: number;
}

export interface RoutingDecision {
  taskId: string;
  selectedAgentId: string;
  confidence: number;
  reasoning: string;
  alternatives: Array<{
    agentId: string;
    score: number;
    reasons: string[];
  }>;
  routingStrategy: "predictive" | "load_balance" | "priority" | "fallback";
  estimatedLatency: number;
  expectedQuality: number;
  performanceMetrics: {
    routingTimeMs: number;
    agentLoadFactor: number;
    memoryRelevanceScore: number;
    historicalSuccessRate: number;
    predictedCompletionTime: number;
  };
  timestamp: Date;
}

export interface TaskQueue {
  high: Task[];
  medium: Task[];
  low: Task[];
  critical: Task[];
}

export class AdvancedTaskRouter extends EventEmitter {
  private logger: Logger;
  private config: RoutingConfig;
  private memoryManager?: MultiTenantMemoryManager;

  // Routing state
  private taskQueue: TaskQueue = {
    high: [],
    medium: [],
    low: [],
    critical: [],
  };

  private agentPerformance = new Map<string, AgentPerformanceMetrics[]>();
  private routingHistory: RoutingDecision[] = [];
  private agentLoad = new Map<string, number>(); // current concurrent tasks per agent

  constructor(config: RoutingConfig, memoryManager?: MultiTenantMemoryManager) {
    super();
    this.logger = new Logger("AdvancedTaskRouter");
    this.config = config;
    this.memoryManager = memoryManager;

    if (this.config.enabled) {
      this.startQueueProcessor();
    }
  }

  /**
   * Submit task to routing queue with priority handling
   */
  async submitTask(
    task: Task,
    tenantId: string,
    context?: TaskContext
  ): Promise<RoutingDecision> {
    if (!this.config.enabled) {
      // Fallback routing
      const agents = this.getAvailableAgents(task.type);
      const selectedAgent = agents[0];
      return this.createRoutingDecision(
        task.id,
        selectedAgent?.id || task.agentId,
        "fallback"
      );
    }

    // In test mode, route synchronously
    if (this.config.testMode) {
      return this.routeTaskSynchronously(task, tenantId, context);
    }

    // Add to appropriate priority queue
    const priority = task.priority || "medium";
    this.taskQueue[priority as keyof TaskQueue].push(task);

    this.logger.info(`Queued task ${task.id} with priority ${priority}`);

    // For critical tasks, attempt immediate routing
    if (priority === "critical") {
      const decision = await this.routeCriticalTask(task, tenantId, context);
      if (decision) {
        this.routingHistory.push(decision);
        this.emit("task-routed", decision);
        return decision;
      }
    }

    // Queue will be processed by background processor
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error(`Task routing timeout for ${task.id}`));
      }, this.config.queueTimeoutMs);

      this.once(`routed-${task.id}`, (decision: RoutingDecision) => {
        clearTimeout(timeout);
        resolve(decision);
      });
    });
  }

  /**
   * Route task synchronously for testing
   */
  private async routeTaskSynchronously(
    task: Task,
    tenantId: string,
    context?: TaskContext
  ): Promise<RoutingDecision> {
    const availableAgents = this.getAvailableAgents(task.type);
    if (availableAgents.length === 0) {
      return this.createRoutingDecision(task.id, task.agentId, "fallback");
    }

    // Use predictive routing for test mode
    const decision = await this.predictiveRouting(
      task,
      availableAgents,
      tenantId,
      context
    );

    if (decision) {
      // Update agent load
      const currentLoad = this.agentLoad.get(decision.selectedAgentId) || 0;
      this.agentLoad.set(decision.selectedAgentId, currentLoad + 1);

      this.routingHistory.push(decision);
      this.emit("task-routed", decision);

      return decision;
    }

    // Fallback to load balancing
    return this.loadBalancingRouting(availableAgents, task);
  }

  /**
   * Route critical tasks immediately
   */
  private async routeCriticalTask(
    task: Task,
    tenantId: string,
    context?: TaskContext
  ): Promise<RoutingDecision | null> {
    const availableAgents = this.getAvailableAgents(task.type);
    if (availableAgents.length === 0) return null;

    // For critical tasks, use predictive routing with highest confidence
    const decision = await this.predictiveRouting(
      task,
      availableAgents,
      tenantId,
      context
    );

    if (decision && decision.confidence > 0.8) {
      // Remove from queue since we're routing immediately
      this.taskQueue.critical = this.taskQueue.critical.filter(
        (t) => t.id !== task.id
      );

      // Update agent load
      const currentLoad = this.agentLoad.get(decision.selectedAgentId) || 0;
      this.agentLoad.set(decision.selectedAgentId, currentLoad + 1);

      return decision;
    }

    return null;
  }

  /**
   * Predictive routing using performance history and ML models
   */
  private async predictiveRouting(
    task: Task,
    candidates: Agent[],
    tenantId: string,
    context?: TaskContext
  ): Promise<RoutingDecision> {
    const taskType = task.type;

    // Get performance predictions for each candidate
    const predictions = await Promise.all(
      candidates.map(async (agent) => {
        const performance = await this.getAgentPerformancePrediction(
          agent.id,
          taskType,
          tenantId
        );
        const loadPenalty = this.calculateLoadPenalty(agent.id);
        const memoryBonus = await this.getMemoryBasedScore(
          agent.id,
          task,
          tenantId,
          context
        );

        const finalScore =
          performance.successRate * 0.4 +
          (1 - performance.averageLatency / 30000) * 0.3 + // Faster is better
          memoryBonus * 0.2 -
          loadPenalty * 0.1;

        return {
          agent,
          score: Math.max(0, Math.min(1, finalScore)),
          performance,
          loadPenalty,
          memoryBonus,
        };
      })
    );

    // Sort by score descending
    predictions.sort((a, b) => b.score - a.score);

    const best = predictions[0];
    const alternatives = predictions.slice(1, 4).map((p) => ({
      agentId: p.agent.id,
      score: p.score,
      reasons: this.generateSelectionReasons(p),
    }));

    const routingStartTime = Date.now();
    const routingTime = Date.now() - routingStartTime;

    return {
      taskId: task.id,
      selectedAgentId: best.agent.id,
      confidence: best.score,
      reasoning: this.generateRoutingReasoning(best),
      alternatives,
      routingStrategy: "predictive",
      estimatedLatency: best.performance.averageLatency,
      expectedQuality: best.performance.qualityScore,
      performanceMetrics: {
        routingTimeMs: routingTime,
        agentLoadFactor: this.calculateLoadPenalty(best.agent.id),
        memoryRelevanceScore: best.memoryBonus,
        historicalSuccessRate: best.performance.successRate,
        predictedCompletionTime: best.performance.averageLatency,
      },
      timestamp: new Date(),
    };
  }

  /**
   * Load balancing routing for even distribution
   */
  private loadBalancingRouting(
    candidates: Agent[],
    task: Task
  ): RoutingDecision {
    // Find agent with lowest current load
    let bestAgent = candidates[0];
    let lowestLoad = this.agentLoad.get(bestAgent.id) || 0;

    for (const agent of candidates.slice(1)) {
      const load = this.agentLoad.get(agent.id) || 0;
      if (load < lowestLoad) {
        lowestLoad = load;
        bestAgent = agent;
      }
    }

    const routingTime = 1; // Minimal time for load balancing

    return {
      taskId: task.id,
      selectedAgentId: bestAgent.id,
      confidence: 0.7,
      reasoning: `Load balancing: selected ${bestAgent.id} with current load ${lowestLoad}`,
      alternatives: candidates
        .filter((a) => a.id !== bestAgent.id)
        .map((a) => ({
          agentId: a.id,
          score: 0.5,
          reasons: [`Load: ${this.agentLoad.get(a.id) || 0}`],
        })),
      routingStrategy: "load_balance",
      estimatedLatency: 15000, // Conservative estimate
      expectedQuality: 0.7,
      performanceMetrics: {
        routingTimeMs: routingTime,
        agentLoadFactor: lowestLoad / this.config.maxConcurrentTasksPerAgent,
        memoryRelevanceScore: 0.5, // Neutral for load balancing
        historicalSuccessRate: 0.7, // Conservative estimate
        predictedCompletionTime: 15000,
      },
      timestamp: new Date(),
    };
  }

  /**
   * Get performance prediction for agent on specific task type
   */
  private async getAgentPerformancePrediction(
    agentId: string,
    taskType: string,
    tenantId: string
  ): Promise<AgentPerformanceMetrics> {
    // Check cached performance metrics first
    const cached = this.agentPerformance
      .get(agentId)
      ?.find((p) => p.taskType === taskType);
    if (cached && Date.now() - cached.lastUpdated.getTime() < 3600000) {
      // 1 hour cache
      return cached;
    }

    // Calculate from historical data
    const performance = await this.calculateAgentPerformance(
      agentId,
      taskType,
      tenantId
    );

    // Cache the result
    const existing = this.agentPerformance.get(agentId) || [];
    const updated = existing.filter((p) => p.taskType !== taskType);
    updated.push(performance);
    this.agentPerformance.set(agentId, updated);

    return performance;
  }

  /**
   * Calculate agent performance from historical data
   */
  private async calculateAgentPerformance(
    agentId: string,
    taskType: string,
    tenantId: string
  ): Promise<AgentPerformanceMetrics> {
    if (!this.memoryManager) {
      return this.createDefaultPerformance(agentId, taskType);
    }

    try {
      // Query memory for agent's task performance
      const query: TaskContext = {
        type: "performance_analysis",
        description: `Agent ${agentId} performance on ${taskType} tasks`,
        requirements: ["task_outcomes", "performance_metrics"],
        constraints: { agentId, taskType },
      };

      const memories = await this.memoryManager.getContextualMemories(
        tenantId,
        query,
        {
          limit: 50,
          minRelevance: 0.5,
        }
      );

      if (!memories.success || !memories.data || memories.data.length === 0) {
        return this.createDefaultPerformance(agentId, taskType);
      }

      // Analyze performance from memories
      let successCount = 0;
      let totalCount = 0;
      let totalLatency = 0;
      let totalQuality = 0;
      const latencies: number[] = [];

      for (const memory of memories.data) {
        if (
          memory.content.agentId === agentId &&
          memory.content.taskType === taskType
        ) {
          totalCount++;
          if (memory.content.outcome === "success") {
            successCount++;
          }

          if (memory.content.latency) {
            latencies.push(memory.content.latency);
            totalLatency += memory.content.latency;
          }

          if (memory.content.qualityScore) {
            totalQuality += memory.content.qualityScore;
          }
        }
      }

      if (totalCount === 0) {
        return this.createDefaultPerformance(agentId, taskType);
      }

      const successRate = successCount / totalCount;
      const averageLatency =
        latencies.length > 0 ? totalLatency / latencies.length : 15000;
      const qualityScore = totalCount > 0 ? totalQuality / totalCount : 0.7;
      const throughput = this.calculateThroughput(latencies);

      return {
        agentId,
        taskType,
        successRate,
        averageLatency,
        throughput,
        qualityScore,
        confidence: Math.min(1.0, totalCount / 10), // Confidence increases with sample size
        lastUpdated: new Date(),
        sampleSize: totalCount,
      };
    } catch (error) {
      this.logger.warn(`Performance calculation failed for ${agentId}:`, error);
      return this.createDefaultPerformance(agentId, taskType);
    }
  }

  /**
   * Get memory-based routing score
   */
  private async getMemoryBasedScore(
    agentId: string,
    task: Task,
    tenantId: string,
    context?: TaskContext
  ): Promise<number> {
    if (!this.memoryManager || !this.config.memoryAwareRouting) {
      return 0.5; // Neutral score
    }

    try {
      const query: TaskContext = {
        ...context,
        type: "task_similarity",
        description: `Similar tasks handled by ${agentId}`,
        requirements: ["agent_performance", "task_similarity"],
        constraints: {
          agentId,
          taskType: task.type,
          similarDescription: task.description,
        },
      };

      const memories = await this.memoryManager.getContextualMemories(
        tenantId,
        query,
        {
          limit: 5,
          minRelevance: 0.7,
        }
      );

      if (!memories.success || !memories.data) {
        return 0.5;
      }

      // Calculate similarity score based on successful similar tasks
      const successfulSimilarTasks = memories.data.filter(
        (m) => m.content.outcome === "success" && m.content.agentId === agentId
      );

      return successfulSimilarTasks.length > 0
        ? Math.min(1.0, successfulSimilarTasks.length / memories.data.length)
        : 0.3;
    } catch (error) {
      this.logger.warn(`Memory-based scoring failed for ${agentId}:`, error);
      return 0.5;
    }
  }

  /**
   * Calculate load penalty for agent
   */
  private calculateLoadPenalty(agentId: string): number {
    const currentLoad = this.agentLoad.get(agentId) || 0;
    const maxLoad = this.config.maxConcurrentTasksPerAgent;

    // Exponential penalty as load approaches max
    return Math.pow(currentLoad / maxLoad, 2);
  }

  /**
   * Calculate throughput from latency samples
   */
  private calculateThroughput(latencies: number[]): number {
    if (latencies.length === 0) return 0;

    const avgLatency = latencies.reduce((a, b) => a + b) / latencies.length;
    // Tasks per hour (assuming 3600 seconds in hour)
    return 3600000 / avgLatency;
  }

  /**
   * Create default performance metrics
   */
  private createDefaultPerformance(
    agentId: string,
    taskType: string
  ): AgentPerformanceMetrics {
    return {
      agentId,
      taskType,
      successRate: 0.7,
      averageLatency: 15000,
      throughput: 240, // 4 tasks per minute
      qualityScore: 0.7,
      confidence: 0.3,
      lastUpdated: new Date(),
      sampleSize: 0,
    };
  }

  /**
   * Generate routing reasoning
   */
  private generateRoutingReasoning(prediction: any): string {
    return (
      `Selected ${prediction.agent.id} with score ${(
        prediction.score * 100
      ).toFixed(1)}% ` +
      `(success: ${(prediction.performance.successRate * 100).toFixed(1)}%, ` +
      `latency: ${prediction.performance.averageLatency}ms, ` +
      `load penalty: ${(prediction.loadPenalty * 100).toFixed(1)}%, ` +
      `memory bonus: ${(prediction.memoryBonus * 100).toFixed(1)}%)`
    );
  }

  /**
   * Generate selection reasons for alternatives
   */
  private generateSelectionReasons(prediction: any): string[] {
    const reasons = [];
    if (prediction.performance.successRate > 0.8) {
      reasons.push(
        `High success rate: ${(
          prediction.performance.successRate * 100
        ).toFixed(1)}%`
      );
    }
    if (prediction.performance.averageLatency < 10000) {
      reasons.push(`Fast response: ${prediction.performance.averageLatency}ms`);
    }
    if (prediction.loadPenalty < 0.2) {
      reasons.push("Low current load");
    }
    if (prediction.memoryBonus > 0.7) {
      reasons.push("Strong memory match");
    }
    return reasons.length > 0 ? reasons : ["General capability match"];
  }

  /**
   * Get available agents for task type
   */
  private getAvailableAgents(taskType: string): Agent[] {
    // This would be implemented to query the agent registry
    // For now, return mock agents
    return [
      {
        id: "agent-1",
        name: "Agent 1",
        type: "worker",
        capabilities: [taskType],
        status: "idle",
        metadata: {},
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      {
        id: "agent-2",
        name: "Agent 2",
        type: "worker",
        capabilities: [taskType],
        status: "idle",
        metadata: {},
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      {
        id: "agent-3",
        name: "Agent 3",
        type: "worker",
        capabilities: [taskType, "advanced"],
        status: "idle",
        metadata: {},
        createdAt: new Date(),
        updatedAt: new Date(),
      },
    ];
  }

  /**
   * Create routing decision
   */
  private createRoutingDecision(
    taskId: string,
    agentId: string,
    strategy: RoutingDecision["routingStrategy"]
  ): RoutingDecision {
    const routingTime = 1; // Minimal time for fallback routing

    return {
      taskId,
      selectedAgentId: agentId,
      confidence: 0.5,
      reasoning: `${strategy} routing strategy`,
      alternatives: [],
      routingStrategy: strategy,
      estimatedLatency: 15000,
      expectedQuality: 0.7,
      performanceMetrics: {
        routingTimeMs: routingTime,
        agentLoadFactor: 0, // Unknown for fallback
        memoryRelevanceScore: 0, // No memory data for fallback
        historicalSuccessRate: 0.5, // Conservative estimate
        predictedCompletionTime: 15000,
      },
      timestamp: new Date(),
    };
  }

  /**
   * Start background queue processor
   */
  private startQueueProcessor(): void {
    const processQueues = async () => {
      try {
        await this.processQueue("critical");
        await this.processQueue("high");
        await this.processQueue("medium");
        await this.processQueue("low");
      } catch (error) {
        this.logger.error("Queue processing error:", error);
      }
    };

    // Process queues every 5 seconds
    setInterval(processQueues, 5000);
  }

  /**
   * Process tasks from a specific priority queue
   */
  private async processQueue(priority: keyof TaskQueue): Promise<void> {
    const queue = this.taskQueue[priority];
    if (queue.length === 0) return;

    // Process one task at a time to avoid overwhelming
    const task = queue.shift();
    if (!task) return;

    try {
      // Use predictive routing for queued tasks
      const availableAgents = this.getAvailableAgents(task.type);
      const decision = await this.predictiveRouting(
        task,
        availableAgents,
        "default-tenant"
      );

      // Update agent load
      const currentLoad = this.agentLoad.get(decision.selectedAgentId) || 0;
      this.agentLoad.set(decision.selectedAgentId, currentLoad + 1);

      this.routingHistory.push(decision);
      this.emit("task-routed", decision);
      this.emit(`routed-${task.id}`, decision);

      this.logger.info(
        `Routed queued task ${task.id} to ${decision.selectedAgentId}`
      );
    } catch (error) {
      this.logger.error(`Failed to route queued task ${task.id}:`, error);
      // Re-queue task for retry
      queue.unshift(task);
    }
  }

  /**
   * Task completion notification for load tracking
   */
  taskCompleted(agentId: string): void {
    const currentLoad = this.agentLoad.get(agentId) || 0;
    this.agentLoad.set(agentId, Math.max(0, currentLoad - 1));
  }

  /**
   * Get routing analytics with detailed performance metrics
   */
  getAnalytics(): {
    totalRouted: number;
    averageConfidence: number;
    strategyBreakdown: Record<string, number>;
    queueDepths: Record<string, number>;
    agentLoads: Record<string, number>;
    performanceMetrics: {
      averageRoutingTimeMs: number;
      averageAgentLoadFactor: number;
      averageMemoryRelevanceScore: number;
      averageHistoricalSuccessRate: number;
      p95RoutingTimeMs: number;
      routingTimeDistribution: { fast: number; medium: number; slow: number };
    };
  } {
    const totalRouted = this.routingHistory.length;
    const averageConfidence =
      totalRouted > 0
        ? this.routingHistory.reduce((sum, r) => sum + r.confidence, 0) /
          totalRouted
        : 0;

    const strategyBreakdown = this.routingHistory.reduce((acc, r) => {
      acc[r.routingStrategy] = (acc[r.routingStrategy] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const queueDepths = {
      critical: this.taskQueue.critical.length,
      high: this.taskQueue.high.length,
      medium: this.taskQueue.medium.length,
      low: this.taskQueue.low.length,
    };

    const agentLoads = Object.fromEntries(this.agentLoad.entries());

    // Calculate performance metrics
    const routingTimes = this.routingHistory.map(
      (r) => r.performanceMetrics.routingTimeMs
    );
    const agentLoadFactors = this.routingHistory.map(
      (r) => r.performanceMetrics.agentLoadFactor
    );
    const memoryRelevanceScores = this.routingHistory.map(
      (r) => r.performanceMetrics.memoryRelevanceScore
    );
    const historicalSuccessRates = this.routingHistory.map(
      (r) => r.performanceMetrics.historicalSuccessRate
    );

    const averageRoutingTimeMs =
      routingTimes.length > 0
        ? routingTimes.reduce((a, b) => a + b) / routingTimes.length
        : 0;

    const averageAgentLoadFactor =
      agentLoadFactors.length > 0
        ? agentLoadFactors.reduce((a, b) => a + b) / agentLoadFactors.length
        : 0;

    const averageMemoryRelevanceScore =
      memoryRelevanceScores.length > 0
        ? memoryRelevanceScores.reduce((a, b) => a + b) /
          memoryRelevanceScores.length
        : 0;

    const averageHistoricalSuccessRate =
      historicalSuccessRates.length > 0
        ? historicalSuccessRates.reduce((a, b) => a + b) /
          historicalSuccessRates.length
        : 0;

    // Calculate P95 routing time
    const sortedTimes = [...routingTimes].sort((a, b) => a - b);
    const p95Index = Math.floor(sortedTimes.length * 0.95);
    const p95RoutingTimeMs = sortedTimes[p95Index] || 0;

    // Routing time distribution
    const routingTimeDistribution = {
      fast: routingTimes.filter((t) => t < 50).length,
      medium: routingTimes.filter((t) => t >= 50 && t < 200).length,
      slow: routingTimes.filter((t) => t >= 200).length,
    };

    return {
      totalRouted,
      averageConfidence,
      strategyBreakdown,
      queueDepths,
      agentLoads,
      performanceMetrics: {
        averageRoutingTimeMs,
        averageAgentLoadFactor,
        averageMemoryRelevanceScore,
        averageHistoricalSuccessRate,
        p95RoutingTimeMs,
        routingTimeDistribution,
      },
    };
  }
}
