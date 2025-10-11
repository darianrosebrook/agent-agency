/**
 * Enhanced Arbiter Orchestrator with RL Integration
 *
 * @author @darianrosebrook
 * @module enhanced-arbiter-orchestrator
 *
 * Extends the base ArbiterOrchestrator with reinforcement learning capabilities
 * for intelligent task routing, performance tracking, and continuous improvement.
 */

import { MultiArmedBandit } from "../rl/MultiArmedBandit";
import { PerformanceTracker } from "../rl/PerformanceTracker";
import { ToolAdoptionTrainer } from "../rl/ToolAdoptionTrainer";
import { TurnLevelRLTrainer } from "../rl/TurnLevelRLTrainer";
import {
  ConversationTrajectory,
  RoutingDecision,
  TaskOutcome,
  ToolExample,
  TurnLevelReward,
} from "../types/agentic-rl";
import { Task, TaskResult } from "../types/arbiter-orchestration";
import {
  ArbiterOrchestrator,
  ArbiterOrchestratorConfig,
} from "./ArbiterOrchestrator";

/**
 * Enhanced Arbiter Orchestrator Configuration
 */
export interface EnhancedArbiterOrchestratorConfig
  extends ArbiterOrchestratorConfig {
  /** RL Components Configuration */
  rl: {
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
  };
}

/**
 * Enhanced Arbiter Orchestrator with RL Capabilities
 *
 * This orchestrator extends the base ArbiterOrchestrator with reinforcement learning
 * components for intelligent task routing, performance tracking, and continuous improvement.
 */
export class EnhancedArbiterOrchestrator extends ArbiterOrchestrator {
  private rlComponents:
    | {
        multiArmedBandit: MultiArmedBandit;
        performanceTracker: PerformanceTracker;
        rlTrainer: TurnLevelRLTrainer;
        toolAdoptionTrainer: ToolAdoptionTrainer;
      }
    | undefined;

  private rlConfig: EnhancedArbiterOrchestratorConfig["rl"];

  constructor(config: EnhancedArbiterOrchestratorConfig) {
    super(config);
    this.rlConfig = config.rl;
  }

  /**
   * Initialize the enhanced orchestrator with RL components
   */
  async initialize(): Promise<void> {
    // Initialize base components first
    await super.initialize();

    // Initialize RL components if enabled
    if (
      this.rlConfig.enableMultiArmedBandit ||
      this.rlConfig.enablePerformanceTracking ||
      this.rlConfig.enableRLTraining ||
      this.rlConfig.enableToolAdoption
    ) {
      this.rlComponents = {
        multiArmedBandit: this.rlConfig.enableMultiArmedBandit
          ? new MultiArmedBandit(this.rlConfig.banditConfig)
          : (null as any),

        performanceTracker: this.rlConfig.enablePerformanceTracking
          ? new PerformanceTracker(this.rlConfig.performanceTrackerConfig)
          : (null as any),

        rlTrainer: this.rlConfig.enableRLTraining
          ? new TurnLevelRLTrainer(this.rlConfig.rlTrainingConfig)
          : (null as any),

        toolAdoptionTrainer: this.rlConfig.enableToolAdoption
          ? new ToolAdoptionTrainer(this.rlConfig.toolAdoptionConfig)
          : (null as any),
      };

      // Start performance tracking if enabled
      if (this.rlComponents.performanceTracker) {
        this.rlComponents.performanceTracker.startCollection();
      }
    }
  }

  /**
   * Enhanced task submission with RL-based routing
   */
  async submitTask(
    task: Task,
    credentials?: any
  ): Promise<{ taskId: string; assignmentId?: string }> {
    // Call parent implementation first
    const result = await super.submitTask(task, credentials);

    // If RL components are available and task was assigned, enhance with RL tracking
    if (this.rlComponents?.performanceTracker && result.assignmentId) {
      // Record the routing decision for RL training
      await this.recordRoutingDecisionForRL(task, result.assignmentId);
    }

    return result;
  }

  /**
   * Attempt RL-enhanced task assignment using multi-armed bandit
   */
  private async attemptRLAssignment(task: Task): Promise<any> {
    if (!this.rlComponents?.multiArmedBandit) {
      // Fall back to no assignment (will be queued)
      return null;
    }

    try {
      // Get available agents for the task
      const availableAgents = await this.getAvailableAgentsForTask(task);

      if (availableAgents.length === 0) {
        return null; // No agents available
      }

      // Use multi-armed bandit for intelligent selection
      const selectedAgent = await this.rlComponents.multiArmedBandit.select(
        availableAgents,
        task.type as any
      );

      // Create routing decision for tracking
      const routingDecision =
        this.rlComponents.multiArmedBandit.createRoutingDecision(
          task.id,
          availableAgents,
          selectedAgent,
          task.type as any
        );

      // Record decision for RL training
      if (this.rlComponents.performanceTracker) {
        await this.rlComponents.performanceTracker.recordRoutingDecision(
          routingDecision
        );
      }

      // Create task assignment
      const assignment: any = {
        id: `assignment-${task.id}-${Date.now()}`,
        taskId: task.id,
        agentId: selectedAgent.id,
        assignedAt: new Date(),
        status: "assigned",
      };

      console.log(
        `RL-enhanced assignment: Task ${task.id} assigned to agent ${selectedAgent.id} ` +
          `with confidence ${(routingDecision.confidence * 100).toFixed(1)}%`
      );

      return assignment;
    } catch (error) {
      console.error(
        "RL-based assignment failed, falling back to queuing:",
        error
      );
      return null; // Fall back to queuing
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
    if (!this.rlComponents?.performanceTracker) {
      return;
    }

    try {
      // Convert task result to outcome format
      const outcome: TaskOutcome = {
        success: taskResult.success,
        qualityScore: taskResult.qualityScore,
        efficiencyScore: 0.8, // Default efficiency
        tokensConsumed: taskResult.performance.tokensUsed || 0,
        completionTimeMs: taskResult.executionTimeMs,
      };

      // Start task execution tracking
      const executionId =
        this.rlComponents.performanceTracker.startTaskExecution(
          taskId,
          "agent-1", // Would need to get from assignment
          {} as any // Mock routing decision
        );

      // Complete the task execution tracking
      await this.rlComponents.performanceTracker.completeTaskExecution(
        executionId,
        outcome
      );

      // Record evaluation outcome
      await this.rlComponents.performanceTracker.recordEvaluationOutcome(
        taskId,
        {
          passed: taskResult.success,
          score: taskResult.qualityScore,
        }
      );
    } catch (error) {
      console.error("Failed to record task completion for RL:", error);
    }
  }

  /**
   * Train RL models on collected data
   */
  async trainRLModels(): Promise<void> {
    if (!this.rlComponents) {
      return;
    }

    try {
      // Export training data
      const trainingData =
        this.rlComponents.performanceTracker?.exportTrainingData();

      if (trainingData && trainingData.length > 0) {
        // Convert to conversation trajectories for RL training
        const trajectories =
          this.convertTrainingDataToTrajectories(trainingData);

        if (trajectories.length > 0 && this.rlComponents.rlTrainer) {
          // Train the RL model
          await this.rlComponents.rlTrainer.trainOnTrajectories(trajectories);
          console.log(
            `Trained RL model on ${trajectories.length} trajectories`
          );
        }
      }

      // Train tool adoption if enabled
      if (this.rlComponents.toolAdoptionTrainer) {
        // Generate synthetic tool examples for training
        const toolExamples = this.generateToolExamplesForTraining();
        if (toolExamples.length > 0) {
          await this.rlComponents.toolAdoptionTrainer.trainOnExamples(
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
   * Get RL training statistics
   */
  getRLStats(): {
    multiArmedBandit?: any;
    performanceTracker?: any;
    rlTraining?: any;
    toolAdoption?: any;
  } {
    if (!this.rlComponents) {
      return {};
    }

    return {
      multiArmedBandit: this.rlComponents.multiArmedBandit?.getStats(),
      performanceTracker: this.rlComponents.performanceTracker?.getStats(),
      rlTraining: this.rlComponents.rlTrainer?.getTrainingStats(),
      toolAdoption: this.rlComponents.toolAdoptionTrainer?.getConfig(),
    };
  }

  /**
   * Get available agents for a task
   */
  private async getAvailableAgentsForTask(task: Task): Promise<any[]> {
    // This would integrate with the agent registry to get available agents
    // For now, return mock data
    return [
      {
        agent: {
          id: "agent-1",
          name: "Agent One",
          modelFamily: "gpt-4" as any,
          capabilities: {
            taskTypes: [task.type],
            languages: [],
            specializations: [],
          },
          performanceHistory: {
            successRate: 0.9,
            averageQuality: 0.85,
            averageLatency: 1000,
            taskCount: 100,
          },
          currentLoad: {
            activeTasks: 1,
            queuedTasks: 0,
            utilizationPercent: 20,
          },
          registeredAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
        },
        matchScore: 0.9,
        matchReason: "Good match for task type",
      },
      {
        agent: {
          id: "agent-2",
          name: "Agent Two",
          modelFamily: "claude-3" as any,
          capabilities: {
            taskTypes: [task.type],
            languages: [],
            specializations: [],
          },
          performanceHistory: {
            successRate: 0.8,
            averageQuality: 0.8,
            averageLatency: 1200,
            taskCount: 50,
          },
          currentLoad: {
            activeTasks: 0,
            queuedTasks: 1,
            utilizationPercent: 10,
          },
          registeredAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
        },
        matchScore: 0.8,
        matchReason: "Decent match for task type",
      },
    ];
  }

  /**
   * Record routing decision for RL training
   */
  private async recordRoutingDecisionForRL(
    task: Task,
    assignmentId: string
  ): Promise<void> {
    if (!this.rlComponents?.performanceTracker) {
      return;
    }

    // Create a mock routing decision for tracking
    // In a full implementation, this would come from the actual assignment
    const mockDecision: RoutingDecision = {
      taskId: task.id,
      selectedAgent: "agent-1", // Would be extracted from assignment
      routingStrategy: "multi-armed-bandit",
      confidence: 0.8,
      alternativesConsidered: [],
      rationale: "RL-based routing decision",
      timestamp: new Date().toISOString(),
    };

    await this.rlComponents.performanceTracker.recordRoutingDecision(
      mockDecision
    );
  }

  /**
   * Convert training data to conversation trajectories
   */
  private convertTrainingDataToTrajectories(
    trainingData: any[]
  ): ConversationTrajectory[] {
    // Group events by task/conversation
    const conversations: { [key: string]: any[] } = {};

    for (const event of trainingData) {
      const taskId =
        event.data?.taskId || event.data?.conversationId || "unknown";
      if (!conversations[taskId]) {
        conversations[taskId] = [];
      }
      conversations[taskId].push(event);
    }

    // Convert to trajectories
    const trajectories: ConversationTrajectory[] = [];

    for (const [taskId, events] of Object.entries(conversations)) {
      const turns: TurnLevelReward[] = events
        .filter((e) => e.type === "task-execution")
        .map((e) => ({
          turnNumber: 1, // Simplified
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
    if (!this.rlComponents?.toolAdoptionTrainer) {
      return [];
    }

    // Generate examples for common tools
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

    return this.rlComponents.toolAdoptionTrainer.generateSyntheticExamples(
      tools as any,
      50
    );
  }
}
