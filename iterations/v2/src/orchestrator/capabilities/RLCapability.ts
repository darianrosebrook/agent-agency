/**
 * @fileoverview RL Capability - Reinforcement Learning Integration for Orchestrator
 *
 * Provides reinforcement learning capabilities for task routing, performance tracking,
 * and continuous improvement. Extracted from EnhancedArbiterOrchestrator to follow
 * composition over inheritance pattern.
 *
 * @author @darianrosebrook
 */

import { MultiArmedBandit } from "../../rl/MultiArmedBandit";
import { PerformanceTracker } from "../../rl/PerformanceTracker";
import { ToolAdoptionTrainer } from "../../rl/ToolAdoptionTrainer";
import { TurnLevelRLTrainer } from "../../rl/TurnLevelRLTrainer";
import {
  ConversationTrajectory,
  RoutingDecision as RLRoutingDecision,
  TaskOutcome,
  ToolExample,
  TurnLevelReward,
} from "../../types/agentic-rl";
import { Task, TaskResult } from "../../types/arbiter-orchestration";
import { AgentRegistryManager } from "../AgentRegistryManager";
import { RoutingOutcome, TaskRoutingManager } from "../TaskRoutingManager";

/**
 * RL Capability Configuration
 */
export interface RLCapabilityConfig {
  /** Enable multi-armed bandit routing */
  enableMultiArmedBandit: boolean;

  /** Enable performance tracking */
  enablePerformanceTracking: boolean;

  /** Enable RL training */
  enableRLTraining: boolean;

  /** Enable tool adoption training */
  enableToolAdoption: boolean;

  /** Multi-armed bandit configuration */
  banditConfig?: any;

  /** Performance tracker configuration */
  performanceTrackerConfig?: any;

  /** RL training configuration */
  rlTrainingConfig?: any;

  /** Tool adoption configuration */
  toolAdoptionConfig?: any;
}

/**
 * RL Capability - Adds reinforcement learning to orchestrator
 *
 * Follows composition pattern to avoid forbidden "enhanced" inheritance approach.
 * Can be integrated into any orchestrator that needs RL capabilities.
 */
export class RLCapability {
  private components:
    | {
        multiArmedBandit: MultiArmedBandit;
        performanceTracker: PerformanceTracker;
        rlTrainer: TurnLevelRLTrainer;
        toolAdoptionTrainer: ToolAdoptionTrainer;
        taskRoutingManager: TaskRoutingManager;
      }
    | undefined;

  constructor(private config: RLCapabilityConfig) {}

  /**
   * Initialize RL components
   */
  async initialize(agentRegistry: AgentRegistryManager): Promise<void> {
    if (
      !this.config.enableMultiArmedBandit &&
      !this.config.enablePerformanceTracking &&
      !this.config.enableRLTraining &&
      !this.config.enableToolAdoption
    ) {
      return;
    }

    const taskRoutingManager = new TaskRoutingManager(agentRegistry, {
      enableBandit: this.config.enableMultiArmedBandit,
      ...this.config.banditConfig,
    });

    this.components = {
      multiArmedBandit: this.config.enableMultiArmedBandit
        ? new MultiArmedBandit(this.config.banditConfig)
        : (null as any),

      performanceTracker: this.config.enablePerformanceTracking
        ? new PerformanceTracker(this.config.performanceTrackerConfig)
        : (null as any),

      rlTrainer: this.config.enableRLTraining
        ? new TurnLevelRLTrainer(this.config.rlTrainingConfig)
        : (null as any),

      toolAdoptionTrainer: this.config.enableToolAdoption
        ? new ToolAdoptionTrainer(this.config.toolAdoptionConfig)
        : (null as any),

      taskRoutingManager,
    };

    if (this.components.performanceTracker) {
      this.components.performanceTracker.startCollection();
    }
  }

  /**
   * Check if RL is enabled
   */
  isEnabled(): boolean {
    return this.components !== undefined;
  }

  /**
   * Record routing decision for RL training
   */
  async recordRoutingDecision(task: Task, assignmentId: string): Promise<void> {
    if (!this.components?.performanceTracker) {
      return;
    }

    // Extract agent ID from task metadata or assignment ID
    let selectedAgentId = "unknown-agent";
    
    // First, check if task has assignedAgent metadata
    if ((task as any).assignedAgent) {
      selectedAgentId = (task as any).assignedAgent;
    }
    // Second, try to extract from assignment ID if it contains agent info
    else if (assignmentId && assignmentId.includes("agent-")) {
      // Extract agent ID from assignment ID if it follows pattern with agent ID
      const agentMatch = assignmentId.match(/agent-\w+/);
      if (agentMatch) {
        selectedAgentId = agentMatch[0];
      }
    }
    // Third, check task metadata for routing decision
    else if ((task as any).routingDecision?.selectedAgent?.id) {
      selectedAgentId = (task as any).routingDecision.selectedAgent.id;
    }

    const mockDecision: RLRoutingDecision = {
      taskId: task.id,
      selectedAgent: selectedAgentId,
      routingStrategy: "multi-armed-bandit",
      confidence: 0.8,
      rationale: "RL-based routing decision",
      alternativesConsidered: [],
      timestamp: new Date().toISOString(),
    };

    await this.components.performanceTracker.recordRoutingDecision(
      mockDecision
    );
  }

  /**
   * Attempt RL-enhanced task routing
   */
  async routeTask(task: Task): Promise<any> {
    if (!this.components?.taskRoutingManager) {
      return null;
    }

    try {
      const routingDecision =
        await this.components.taskRoutingManager.routeTask(task);

      if (this.components.performanceTracker) {
        const rlRoutingDecision: RLRoutingDecision = {
          taskId: task.id,
          selectedAgent: routingDecision.selectedAgent.id,
          routingStrategy: routingDecision.strategy as any,
          confidence: routingDecision.confidence,
          rationale: routingDecision.reason,
          alternativesConsidered: routingDecision.alternatives.map((alt) => ({
            agentId: alt.agent.id,
            score: alt.score,
            reason: alt.reason,
          })),
          timestamp:
            typeof routingDecision.timestamp === "string"
              ? routingDecision.timestamp
              : routingDecision.timestamp.toISOString(),
        };

        await this.components.performanceTracker.recordRoutingDecision(
          rlRoutingDecision
        );
      }

      const assignment: any = {
        id: `assignment-${task.id}-${Date.now()}`,
        taskId: task.id,
        agentId: routingDecision.selectedAgent.id,
        assignedAt: new Date(),
        status: "assigned",
      };

      console.log(
        `RL-enhanced assignment: Task ${task.id} assigned to agent ${routingDecision.selectedAgent.id} ` +
          `with confidence ${(routingDecision.confidence * 100).toFixed(1)}% ` +
          `via ${routingDecision.strategy} strategy`
      );

      return assignment;
    } catch (error) {
      console.error(
        "RL-based assignment failed, falling back to queuing:",
        error
      );
      return null;
    }
  }

  /**
   * Record task completion for RL training
   */
  async recordTaskCompletion(
    taskId: string,
    taskResult: TaskResult,
    assignmentId?: string
  ): Promise<void> {
    if (!this.components) {
      return;
    }

    try {
      const outcome: TaskOutcome = {
        success: taskResult.success,
        qualityScore: taskResult.qualityScore,
        efficiencyScore: 0.8,
        tokensConsumed: taskResult.performance.tokensUsed || 0,
        completionTimeMs: taskResult.executionTimeMs,
      };

      if (this.components.performanceTracker) {
        // Extract agent ID from assignment ID or task metadata
        let agentId = "unknown-agent";

        if (assignmentId && this.components.taskAssignmentManager) {
          try {
            const assignment =
              await this.components.taskAssignmentManager.getAssignment(
                assignmentId
              );
            agentId = assignment?.agent?.id || agentId;
          } catch (error) {
            this.logger?.warn(
              "Failed to get assignment for performance tracking",
              { assignmentId, error }
            );
          }
        }

        // Fallback: check if task has assignedAgent metadata
        if (agentId === "unknown-agent" && (task as any).assignedAgent) {
          agentId = (task as any).assignedAgent;
        }

        const executionId =
          this.components.performanceTracker.startTaskExecution(
            taskId,
            agentId,
            {} as any
          );

        await this.components.performanceTracker.completeTaskExecution(
          executionId,
          outcome
        );

        await this.components.performanceTracker.recordEvaluationOutcome(
          taskId,
          {
            passed: taskResult.success,
            score: taskResult.qualityScore,
          }
        );
      }

      if (this.components.taskRoutingManager && assignmentId) {
        const routingStats =
          await this.components.taskRoutingManager.getRoutingStats();
        const routingDecision = routingStats.recentDecisions.find(
          (decision) => decision.taskId === taskId
        );

        if (routingDecision) {
          const routingOutcome: RoutingOutcome = {
            routingDecision: {
              id: assignmentId,
              taskId,
              selectedAgent: {
                id: routingDecision.agentId,
              } as any,
              confidence: routingDecision.confidence,
              reason: "Routing decision for task completion",
              strategy: routingDecision.strategy as any,
              alternatives: [],
              timestamp: routingDecision.timestamp,
            },
            success: taskResult.success,
            qualityScore: taskResult.qualityScore,
            latencyMs: taskResult.executionTimeMs,
            errorReason: taskResult.errors?.[0] || undefined,
          };

          await this.components.taskRoutingManager.recordRoutingOutcome(
            routingOutcome
          );
        }
      }
    } catch (error) {
      console.error("Failed to record task completion for RL:", error);
    }
  }

  /**
   * Train RL models on collected data
   */
  async trainModels(): Promise<void> {
    if (!this.components) {
      return;
    }

    try {
      const trainingData =
        this.components.performanceTracker?.exportTrainingData();

      if (trainingData && trainingData.length > 0) {
        const trajectories =
          this.convertTrainingDataToTrajectories(trainingData);

        if (trajectories.length > 0 && this.components.rlTrainer) {
          await this.components.rlTrainer.trainOnTrajectories(trajectories);
          console.log(
            `Trained RL model on ${trajectories.length} trajectories`
          );
        }
      }

      if (this.components.toolAdoptionTrainer) {
        const toolExamples = this.generateToolExamplesForTraining();
        if (toolExamples.length > 0) {
          await this.components.toolAdoptionTrainer.trainOnExamples(
            toolExamples
          );
          console.log(
            `Trained tool adoption on ${toolExamples.length} examples`
          );
        }
      }
    } catch (error) {
      console.error("RL training failed:", error);
    }
  }

  /**
   * Get RL statistics
   */
  getStats(): {
    multiArmedBandit?: any;
    performanceTracker?: any;
    rlTraining?: any;
    toolAdoption?: any;
  } {
    if (!this.components) {
      return {};
    }

    return {
      multiArmedBandit: this.components.multiArmedBandit?.getStats(),
      performanceTracker: this.components.performanceTracker?.getStats(),
      rlTraining: this.components.rlTrainer?.getTrainingStats(),
      toolAdoption: this.components.toolAdoptionTrainer?.getConfig(),
    };
  }

  /**
   * Convert training data to conversation trajectories
   */
  private convertTrainingDataToTrajectories(
    trainingData: any[]
  ): ConversationTrajectory[] {
    const conversations: { [key: string]: any[] } = {};

    for (const event of trainingData) {
      const taskId =
        event.data?.taskId || event.data?.conversationId || "unknown";
      if (!conversations[taskId]) {
        conversations[taskId] = [];
      }
      conversations[taskId].push(event);
    }

    const trajectories: ConversationTrajectory[] = [];

    for (const [taskId, events] of Object.entries(conversations)) {
      const turns: TurnLevelReward[] = events
        .filter((e) => e.type === "task-execution")
        .map((e) => ({
          turnNumber: 1,
          toolChoice: { toolId: "generic_tool", parameters: {} },
          informationGain: e.data?.outcome?.success ? 0.8 : 0.2,
          formatCorrectness: 1,
          taskProgress: e.data?.outcome?.success ? 1 : 0,
          safetyScore: 0.9,
          totalReward: e.data?.outcome?.qualityScore || 0,
        }));

      if (turns.length > 0) {
        trajectories.push({
          conversationId: taskId,
          turns,
          finalOutcome: {
            success: turns[turns.length - 1].taskProgress > 0,
            qualityScore: turns[turns.length - 1].totalReward,
            efficiencyScore: 0.8,
            tokensConsumed: 1000,
            completionTimeMs: 5000,
          },
          totalReward: turns.reduce((sum, turn) => sum + turn.totalReward, 0),
        });
      }
    }

    return trajectories;
  }

  /**
   * Generate tool examples for training
   */
  private generateToolExamplesForTraining(): ToolExample[] {
    if (!this.components?.toolAdoptionTrainer) {
      return [];
    }

    const tools = [
      {
        id: "read_file",
        name: "Read File",
        description: "Reads file contents",
        parameters: { path: "string" },
      },
      {
        id: "grep",
        name: "Search",
        description: "Searches for patterns",
        parameters: { pattern: "string", path: "string" },
      },
      {
        id: "run_terminal_cmd",
        name: "Terminal",
        description: "Runs terminal commands",
        parameters: { command: "string" },
      },
    ];

    return this.components.toolAdoptionTrainer.generateSyntheticExamples(
      tools as any,
      50
    );
  }
}
