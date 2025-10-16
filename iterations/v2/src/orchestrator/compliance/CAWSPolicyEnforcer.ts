/**
 * @fileoverview CAWS Policy Enforcer - ARBITER-028
 *
 * Middleware for runtime CAWS compliance checks including budget tracking,
 * tool usage monitoring, and reasoning depth guards.
 *
 * @author @darianrosebrook
 */

export interface CAWSPolicyViolation {
  type:
    | "budget_exceeded"
    | "tool_usage_violation"
    | "reasoning_depth_exceeded"
    | "forbidden_operation";
  severity: "low" | "medium" | "high" | "critical";
  message: string;
  details: Record<string, any>;
  timestamp: Date;
  taskId: string;
  agentId: string;
}

export interface CAWSBudget {
  maxFiles: number;
  maxLinesOfCode: number;
  currentFiles: number;
  currentLinesOfCode: number;
  utilization: {
    files: number; // percentage
    lines: number; // percentage
  };
}

export interface CAWSToolUsage {
  allowedTools: string[];
  usedTools: Map<string, number>; // tool -> usage count
  maxUsagePerTool: Map<string, number>; // tool -> max allowed
  violations: string[];
}

export interface CAWSReasoningDepth {
  maxDepth: number;
  currentDepth: number;
  depthHistory: number[];
  violations: number;
}

export interface CAWSPolicyConfig {
  budgets: {
    defaultMaxFiles: number;
    defaultMaxLinesOfCode: number;
    taskTypeOverrides: Record<
      string,
      { maxFiles: number; maxLinesOfCode: number }
    >;
  };
  toolUsage: {
    defaultAllowedTools: string[];
    maxUsagePerTool: Record<string, number>;
    taskTypeOverrides: Record<
      string,
      { allowedTools: string[]; maxUsage: Record<string, number> }
    >;
  };
  reasoning: {
    defaultMaxDepth: number;
    taskTypeOverrides: Record<string, number>;
  };
  enforcement: {
    enableBudgetChecks: boolean;
    enableToolChecks: boolean;
    enableReasoningChecks: boolean;
    enableForbiddenOperationChecks: boolean;
  };
}

export interface CAWSPolicyEnforcer {
  /**
   * Initialize policy enforcement for a task
   */
  initializeTask(
    taskId: string,
    agentId: string,
    taskType: string
  ): Promise<void>;

  /**
   * Check file creation against budget
   */
  checkFileCreation(
    taskId: string,
    filePath: string,
    estimatedLines?: number
  ): Promise<boolean>;

  /**
   * Check tool usage against policy
   */
  checkToolUsage(taskId: string, toolName: string): Promise<boolean>;

  /**
   * Check reasoning depth against limits
   */
  checkReasoningDepth(taskId: string, currentDepth: number): Promise<boolean>;

  /**
   * Check for forbidden operations
   */
  checkForbiddenOperation(
    taskId: string,
    operation: string,
    details?: any
  ): Promise<boolean>;

  /**
   * Get current budget status for task
   */
  getBudgetStatus(taskId: string): Promise<CAWSBudget | null>;

  /**
   * Get tool usage status for task
   */
  getToolUsageStatus(taskId: string): Promise<CAWSToolUsage | null>;

  /**
   * Get reasoning depth status for task
   */
  getReasoningDepthStatus(taskId: string): Promise<CAWSReasoningDepth | null>;

  /**
   * Get all violations for a task
   */
  getTaskViolations(taskId: string): Promise<CAWSPolicyViolation[]>;

  /**
   * Finalize task and cleanup enforcement state
   */
  finalizeTask(taskId: string): Promise<{
    violations: CAWSPolicyViolation[];
    budgetUtilization: CAWSBudget | null;
    toolUsageSummary: CAWSToolUsage | null;
  }>;

  /**
   * Get enforcement statistics
   */
  getStatistics(): Promise<{
    activeTasks: number;
    totalViolations: number;
    violationBreakdown: Record<CAWSPolicyViolation["type"], number>;
    averageBudgetUtilization: number;
  }>;
}

/**
 * Implementation of CAWS Policy Enforcer
 */
export class CAWSPolicyEnforcerImpl implements CAWSPolicyEnforcer {
  private taskStates: Map<
    string,
    {
      agentId: string;
      taskType: string;
      budget: CAWSBudget;
      toolUsage: CAWSToolUsage;
      reasoningDepth: CAWSReasoningDepth;
      violations: CAWSPolicyViolation[];
      config: CAWSPolicyConfig;
    }
  > = new Map();

  private readonly defaultConfig: CAWSPolicyConfig = {
    budgets: {
      defaultMaxFiles: 25,
      defaultMaxLinesOfCode: 1000,
      taskTypeOverrides: {
        refactor: { maxFiles: 15, maxLinesOfCode: 500 },
        fix: { maxFiles: 10, maxLinesOfCode: 200 },
        feature: { maxFiles: 30, maxLinesOfCode: 1500 },
      },
    },
    toolUsage: {
      defaultAllowedTools: [
        "read_file",
        "write",
        "search_replace",
        "grep",
        "list_dir",
      ],
      maxUsagePerTool: {
        read_file: 50,
        write: 20,
        search_replace: 30,
        grep: 100,
        list_dir: 25,
      },
      taskTypeOverrides: {
        refactor: {
          allowedTools: ["read_file", "search_replace", "grep", "list_dir"],
          maxUsage: {
            read_file: 30,
            search_replace: 20,
            grep: 50,
            list_dir: 15,
          },
        },
      },
    },
    reasoning: {
      defaultMaxDepth: 10,
      taskTypeOverrides: {
        fix: 5,
        refactor: 8,
      },
    },
    enforcement: {
      enableBudgetChecks: true,
      enableToolChecks: true,
      enableReasoningChecks: true,
      enableForbiddenOperationChecks: true,
    },
  };

  constructor(private config: Partial<CAWSPolicyConfig> = {}) {
    this.config = { ...this.defaultConfig, ...config };
  }

  async initializeTask(
    taskId: string,
    agentId: string,
    taskType: string
  ): Promise<void> {
    const taskConfig = this.getTaskConfig(taskType);

    const taskState = {
      agentId,
      taskType,
      budget: {
        maxFiles: taskConfig.maxFiles,
        maxLinesOfCode: taskConfig.maxLinesOfCode,
        currentFiles: 0,
        currentLinesOfCode: 0,
        utilization: { files: 0, lines: 0 },
      },
      toolUsage: {
        allowedTools: taskConfig.allowedTools,
        usedTools: new Map(),
        maxUsagePerTool: new Map(Object.entries(taskConfig.maxUsagePerTool)),
        violations: [],
      },
      reasoningDepth: {
        maxDepth: taskConfig.maxDepth,
        currentDepth: 0,
        depthHistory: [],
        violations: 0,
      },
      violations: [],
      config: this.config as CAWSPolicyConfig,
    };

    this.taskStates.set(taskId, taskState);
  }

  async checkFileCreation(
    taskId: string,
    filePath: string,
    estimatedLines: number = 50
  ): Promise<boolean> {
    if (!this.config.enforcement!.enableBudgetChecks) {
      return true;
    }

    const taskState = this.taskStates.get(taskId);
    if (!taskState) {
      throw new Error(`Task not initialized: ${taskId}`);
    }

    const { budget } = taskState;

    // Check file count limit
    if (budget.currentFiles >= budget.maxFiles) {
      this.recordViolation(taskId, {
        type: "budget_exceeded",
        severity: "high",
        message: `File count limit exceeded: ${budget.currentFiles}/${budget.maxFiles}`,
        details: {
          filePath,
          currentFiles: budget.currentFiles,
          maxFiles: budget.maxFiles,
        },
        timestamp: new Date(),
        taskId,
        agentId: taskState.agentId,
      });
      return false;
    }

    // Check lines of code limit
    const projectedLines = budget.currentLinesOfCode + estimatedLines;
    if (projectedLines > budget.maxLinesOfCode) {
      this.recordViolation(taskId, {
        type: "budget_exceeded",
        severity: "high",
        message: `Lines of code limit would be exceeded: ${projectedLines}/${budget.maxLinesOfCode}`,
        details: {
          filePath,
          estimatedLines,
          projectedLines,
          maxLines: budget.maxLinesOfCode,
        },
        timestamp: new Date(),
        taskId,
        agentId: taskState.agentId,
      });
      return false;
    }

    // Update budget
    budget.currentFiles++;
    budget.currentLinesOfCode += estimatedLines;
    budget.utilization.files = (budget.currentFiles / budget.maxFiles) * 100;
    budget.utilization.lines =
      (budget.currentLinesOfCode / budget.maxLinesOfCode) * 100;

    return true;
  }

  async checkToolUsage(taskId: string, toolName: string): Promise<boolean> {
    if (!this.config.enforcement!.enableToolChecks) {
      return true;
    }

    const taskState = this.taskStates.get(taskId);
    if (!taskState) {
      throw new Error(`Task not initialized: ${taskId}`);
    }

    const { toolUsage } = taskState;

    // Check if tool is allowed
    if (!toolUsage.allowedTools.includes(toolName)) {
      this.recordViolation(taskId, {
        type: "tool_usage_violation",
        severity: "medium",
        message: `Tool not allowed: ${toolName}`,
        details: { toolName, allowedTools: toolUsage.allowedTools },
        timestamp: new Date(),
        taskId,
        agentId: taskState.agentId,
      });
      return false;
    }

    // Check usage limit
    const currentUsage = toolUsage.usedTools.get(toolName) || 0;
    const maxUsage = toolUsage.maxUsagePerTool.get(toolName) || 0;

    if (currentUsage >= maxUsage) {
      this.recordViolation(taskId, {
        type: "tool_usage_violation",
        severity: "medium",
        message: `Tool usage limit exceeded: ${toolName} (${currentUsage}/${maxUsage})`,
        details: { toolName, currentUsage, maxUsage },
        timestamp: new Date(),
        taskId,
        agentId: taskState.agentId,
      });
      return false;
    }

    // Update usage
    toolUsage.usedTools.set(toolName, currentUsage + 1);

    return true;
  }

  async checkReasoningDepth(
    taskId: string,
    currentDepth: number
  ): Promise<boolean> {
    if (!this.config.enforcement!.enableReasoningChecks) {
      return true;
    }

    const taskState = this.taskStates.get(taskId);
    if (!taskState) {
      throw new Error(`Task not initialized: ${taskId}`);
    }

    const { reasoningDepth } = taskState;

    // Update depth history
    reasoningDepth.depthHistory.push(currentDepth);
    reasoningDepth.currentDepth = currentDepth;

    // Check depth limit
    if (currentDepth > reasoningDepth.maxDepth) {
      reasoningDepth.violations++;

      this.recordViolation(taskId, {
        type: "reasoning_depth_exceeded",
        severity: "high",
        message: `Reasoning depth exceeded: ${currentDepth}/${reasoningDepth.maxDepth}`,
        details: {
          currentDepth,
          maxDepth: reasoningDepth.maxDepth,
          violations: reasoningDepth.violations,
        },
        timestamp: new Date(),
        taskId,
        agentId: taskState.agentId,
      });
      return false;
    }

    return true;
  }

  async checkForbiddenOperation(
    taskId: string,
    operation: string,
    details?: any
  ): Promise<boolean> {
    if (!this.config.enforcement!.enableForbiddenOperationChecks) {
      return true;
    }

    const taskState = this.taskStates.get(taskId);
    if (!taskState) {
      throw new Error(`Task not initialized: ${taskId}`);
    }

    const forbiddenOperations = [
      "rm -rf /",
      "delete_system_files",
      "access_secrets",
      "bypass_security",
      "infinite_loop",
      "fork_bomb",
    ];

    if (
      forbiddenOperations.some((forbidden) => operation.includes(forbidden))
    ) {
      this.recordViolation(taskId, {
        type: "forbidden_operation",
        severity: "critical",
        message: `Forbidden operation detected: ${operation}`,
        details: { operation, ...details },
        timestamp: new Date(),
        taskId,
        agentId: taskState.agentId,
      });
      return false;
    }

    return true;
  }

  async getBudgetStatus(taskId: string): Promise<CAWSBudget | null> {
    const taskState = this.taskStates.get(taskId);
    return taskState ? taskState.budget : null;
  }

  async getToolUsageStatus(taskId: string): Promise<CAWSToolUsage | null> {
    const taskState = this.taskStates.get(taskId);
    if (!taskState) {
      return null;
    }

    // Return the tool usage as-is (Maps)
    return {
      ...taskState.toolUsage,
    };
  }

  async getReasoningDepthStatus(
    taskId: string
  ): Promise<CAWSReasoningDepth | null> {
    const taskState = this.taskStates.get(taskId);
    return taskState ? taskState.reasoningDepth : null;
  }

  async getTaskViolations(taskId: string): Promise<CAWSPolicyViolation[]> {
    const taskState = this.taskStates.get(taskId);
    return taskState ? taskState.violations : [];
  }

  async finalizeTask(taskId: string): Promise<{
    violations: CAWSPolicyViolation[];
    budgetUtilization: CAWSBudget | null;
    toolUsageSummary: CAWSToolUsage | null;
  }> {
    const taskState = this.taskStates.get(taskId);
    if (!taskState) {
      throw new Error(`Task not initialized: ${taskId}`);
    }

    const result = {
      violations: [...taskState.violations],
      budgetUtilization: taskState.budget,
      toolUsageSummary: {
        ...taskState.toolUsage,
        usedTools: taskState.toolUsage.usedTools,
        maxUsagePerTool: taskState.toolUsage.maxUsagePerTool,
      },
    };

    // Cleanup task state
    this.taskStates.delete(taskId);

    return result;
  }

  async getStatistics(): Promise<{
    activeTasks: number;
    totalViolations: number;
    violationBreakdown: Record<CAWSPolicyViolation["type"], number>;
    averageBudgetUtilization: number;
  }> {
    const activeTasks = this.taskStates.size;
    let totalViolations = 0;
    const violationBreakdown: Record<CAWSPolicyViolation["type"], number> = {
      budget_exceeded: 0,
      tool_usage_violation: 0,
      reasoning_depth_exceeded: 0,
      forbidden_operation: 0,
    };
    let totalBudgetUtilization = 0;

    for (const taskState of this.taskStates.values()) {
      totalViolations += taskState.violations.length;

      for (const violation of taskState.violations) {
        violationBreakdown[violation.type]++;
      }

      totalBudgetUtilization += taskState.budget.utilization.lines;
    }

    const averageBudgetUtilization =
      activeTasks > 0 ? totalBudgetUtilization / activeTasks : 0;

    return {
      activeTasks,
      totalViolations,
      violationBreakdown,
      averageBudgetUtilization,
    };
  }

  private getTaskConfig(taskType: string) {
    const config = this.config as CAWSPolicyConfig;

    // Get task-specific overrides or use defaults
    const budgetOverride = config.budgets.taskTypeOverrides[taskType];
    const toolOverride = config.toolUsage.taskTypeOverrides[taskType];
    const reasoningOverride = config.reasoning.taskTypeOverrides[taskType];

    return {
      maxFiles: budgetOverride?.maxFiles ?? config.budgets.defaultMaxFiles,
      maxLinesOfCode:
        budgetOverride?.maxLinesOfCode ?? config.budgets.defaultMaxLinesOfCode,
      allowedTools:
        toolOverride?.allowedTools ?? config.toolUsage.defaultAllowedTools,
      maxUsagePerTool:
        toolOverride?.maxUsage ?? config.toolUsage.maxUsagePerTool,
      maxDepth: reasoningOverride ?? config.reasoning.defaultMaxDepth,
    };
  }

  private recordViolation(
    taskId: string,
    violation: CAWSPolicyViolation
  ): void {
    const taskState = this.taskStates.get(taskId);
    if (taskState) {
      taskState.violations.push(violation);

      // Emit violation event (can be extended with EventEmitter)
      console.warn(`CAWS Policy Violation:`, violation);
    }
  }
}
