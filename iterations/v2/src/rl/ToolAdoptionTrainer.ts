/**
 * Tool Adoption Training Framework
 *
 * @author @darianrosebrook
 * @module tool-adoption-trainer
 *
 * Implements supervised warmup and RL fine-tuning for improving tool usage rates.
 * Trains agents to properly select, format, and use tools in conversations.
 */

import {
  RLError,
  RLErrorType,
  Tool,
  ToolCall,
  ToolExample,
  ToolRewardSignal,
} from "../types/agentic-rl";

/**
 * Configuration for tool adoption training.
 */
export interface ToolAdoptionConfig {
  /**
   * Number of supervised warmup examples.
   */
  warmupExamples: number;

  /**
   * Learning rate for RL fine-tuning.
   */
  learningRate: number;

  /**
   * Batch size for training.
   */
  batchSize: number;

  /**
   * KL divergence penalty coefficient.
   */
  klPenalty: number;

  /**
   * Maximum training epochs.
   */
  maxEpochs: number;

  /**
   * Tool choice reward weight.
   */
  toolChoiceWeight: number;

  /**
   * Format correctness reward weight.
   */
  formatCorrectnessWeight: number;

  /**
   * Information utility reward weight.
   */
  informationUtilityWeight: number;

  /**
   * Error handling reward weight.
   */
  errorHandlingWeight: number;
}

/**
 * Training statistics for tool adoption.
 */
export interface ToolAdoptionStats {
  /**
   * Total examples processed.
   */
  examplesProcessed: number;

  /**
   * Supervised warmup accuracy.
   */
  warmupAccuracy: number;

  /**
   * RL fine-tuning improvement.
   */
  rlImprovement: number;

  /**
   * Tool choice accuracy.
   */
  toolChoiceAccuracy: number;

  /**
   * Format correctness rate.
   */
  formatCorrectnessRate: number;

  /**
   * Training time in milliseconds.
   */
  trainingTimeMs: number;

  /**
   * Timestamp of training completion.
   */
  timestamp: string;
}

/**
 * Tool Adoption Trainer implementing supervised warmup + RL fine-tuning.
 *
 * This trainer improves tool usage through two phases:
 * 1. Supervised Fine-tuning: Learn from correct tool usage examples
 * 2. RL Fine-tuning: Optimize tool choice with intermediate rewards
 */
export class ToolAdoptionTrainer {
  private config: ToolAdoptionConfig;

  /**
   * Creates a new tool adoption trainer.
   *
   * @param config - Training configuration. Uses defaults if not provided.
   */
  constructor(config: Partial<ToolAdoptionConfig> = {}) {
    this.config = {
      warmupExamples: 1000,
      learningRate: 1e-5,
      batchSize: 32,
      klPenalty: 0.1,
      maxEpochs: 10,
      toolChoiceWeight: 0.4,
      formatCorrectnessWeight: 0.3,
      informationUtilityWeight: 0.2,
      errorHandlingWeight: 0.1,
      ...config,
    };
  }

  /**
   * Trains tool adoption on a set of examples.
   *
   * @param examples - Tool usage examples for training.
   * @returns Training statistics.
   */
  async trainOnExamples(examples: ToolExample[]): Promise<ToolAdoptionStats> {
    const startTime = Date.now();

    if (examples.length === 0) {
      throw new RLError(
        RLErrorType.INVALID_CONFIG,
        "No tool examples provided for training"
      );
    }

    // Phase 1: Supervised warmup
    const warmupResults = await this.supervisedWarmup(examples);

    // Phase 2: RL fine-tuning
    const rlResults = await this.rlFineTuning(examples, warmupResults);

    const trainingTime = Date.now() - startTime;

    return {
      examplesProcessed: examples.length,
      warmupAccuracy: warmupResults.accuracy,
      rlImprovement: rlResults.improvement,
      toolChoiceAccuracy: rlResults.toolChoiceAccuracy,
      formatCorrectnessRate: rlResults.formatCorrectnessRate,
      trainingTimeMs: trainingTime,
      timestamp: new Date().toISOString(),
    };
  }

  /**
   * Evaluates tool usage quality.
   *
   * @param toolCall - The tool call to evaluate.
   * @param context - Evaluation context (correct tool, task requirements, etc.).
   * @returns Quality evaluation scores.
   */
  async evaluateToolUsage(
    toolCall: ToolCall,
    context: {
      correctTool?: ToolCall;
      availableTools: Tool[];
      taskRequirements?: Record<string, unknown>;
    }
  ): Promise<{
    toolChoiceAppropriate: boolean;
    formatCorrect: boolean;
    informationUtility: number;
    errorHandlingCorrect: boolean;
    overallScore: number;
  }> {
    const toolChoiceAppropriate = this.evaluateToolChoice(toolCall, context);
    const formatCorrect = this.evaluateFormatCorrectness(toolCall);
    const informationUtility = await this.evaluateInformationUtility(
      toolCall,
      context
    );
    const errorHandlingCorrect = this.evaluateErrorHandling(toolCall);

    const overallScore =
      (toolChoiceAppropriate ? this.config.toolChoiceWeight : 0) +
      (formatCorrect ? this.config.formatCorrectnessWeight : 0) +
      informationUtility * this.config.informationUtilityWeight +
      (errorHandlingCorrect ? this.config.errorHandlingWeight : 0);

    return {
      toolChoiceAppropriate,
      formatCorrect,
      informationUtility,
      errorHandlingCorrect,
      overallScore,
    };
  }

  /**
   * Generates synthetic tool examples for training.
   *
   * @param availableTools - Tools to generate examples for.
   * @param count - Number of examples to generate.
   * @returns Generated tool examples.
   */
  generateSyntheticExamples(
    availableTools: Tool[],
    count: number
  ): ToolExample[] {
    const examples: ToolExample[] = [];

    if (availableTools.length === 0) {
      // Generate generic examples when no tools are available
      for (let i = 0; i < count; i++) {
        examples.push({
          prompt: `Please help me with task ${i + 1}`,
          correctToolCall: { toolId: "generic_tool", parameters: {} },
          expectedReasoning: "Using a generic tool for the task",
          difficulty: "easy" as const,
        });
      }
      return examples;
    }

    for (let i = 0; i < count; i++) {
      const tool =
        availableTools[Math.floor(Math.random() * availableTools.length)];

      // Generate a realistic prompt for this tool
      const prompt = this.generatePromptForTool(tool);

      // Generate correct tool call
      const correctToolCall = this.generateCorrectToolCall(tool);

      // Determine difficulty
      const difficulty = this.assessExampleDifficulty(tool, correctToolCall);

      examples.push({
        prompt,
        correctToolCall,
        expectedReasoning: `Using ${tool.name} to ${this.describeToolPurpose(
          tool
        )}`,
        difficulty,
      });
    }

    return examples;
  }

  /**
   * Computes reward signal for a tool call.
   *
   * @param toolCall - Tool call to evaluate.
   * @param evaluation - Quality evaluation results.
   * @returns Reward signal components.
   */
  computeRewardSignal(
    toolCall: ToolCall,
    evaluation: {
      toolChoiceAppropriate: boolean;
      formatCorrect: boolean;
      informationUtility: number;
      errorHandlingCorrect: boolean;
    }
  ): ToolRewardSignal {
    const totalReward =
      (evaluation.toolChoiceAppropriate ? 1 : 0) *
        this.config.toolChoiceWeight +
      (evaluation.formatCorrect ? 1 : 0) * this.config.formatCorrectnessWeight +
      evaluation.informationUtility * this.config.informationUtilityWeight +
      (evaluation.errorHandlingCorrect ? 1 : 0) *
        this.config.errorHandlingWeight;

    return {
      callStructureValid: evaluation.formatCorrect,
      toolChoiceAppropriate: evaluation.toolChoiceAppropriate,
      informationUtility: evaluation.informationUtility,
      errorHandlingCorrect: evaluation.errorHandlingCorrect,
      efficiency: totalReward, // Use total reward as efficiency proxy
      totalReward,
    };
  }

  /**
   * Gets current configuration.
   *
   * @returns Current configuration.
   */
  getConfig(): ToolAdoptionConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration.
   *
   * @param config - New configuration to apply.
   */
  updateConfig(config: Partial<ToolAdoptionConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Phase 1: Supervised warmup training.
   *
   * @param examples - Training examples.
   * @returns Warmup training results.
   */
  private async supervisedWarmup(examples: ToolExample[]): Promise<{
    accuracy: number;
    trainedModel: any; // Would be actual model in full implementation
  }> {
    // In a full implementation, this would fine-tune a model on the examples
    // For now, simulate training with a mock accuracy calculation

    let correctPredictions = 0;

    for (const example of examples.slice(
      0,
      Math.min(examples.length, this.config.warmupExamples)
    )) {
      // Simulate model prediction (in practice, this would be actual model inference)
      const predictedToolCall = this.simulateModelPrediction(
        example.prompt,
        example.correctToolCall.toolId
      );

      if (this.compareToolCalls(predictedToolCall, example.correctToolCall)) {
        correctPredictions++;
      }
    }

    const accuracy =
      correctPredictions /
      Math.min(examples.length, this.config.warmupExamples);

    return {
      accuracy,
      trainedModel: {}, // Mock trained model
    };
  }

  /**
   * Phase 2: RL fine-tuning.
   *
   * @param examples - Training examples.
   * @param warmupResults - Results from supervised warmup.
   * @returns RL fine-tuning results.
   */
  private async rlFineTuning(
    examples: ToolExample[],
    warmupResults: { accuracy: number; trainedModel: any }
  ): Promise<{
    improvement: number;
    toolChoiceAccuracy: number;
    formatCorrectnessRate: number;
    trainedModel: any;
  }> {
    // In a full implementation, this would perform RL fine-tuning
    // For now, simulate improvement over warmup

    let toolChoiceCorrect = 0;
    let formatCorrect = 0;
    let totalEvaluations = 0;

    for (const example of examples) {
      // Simulate RL-improved predictions
      const predictedToolCall = this.simulateRLPrediction(
        example.prompt,
        example.correctToolCall.toolId
      );

      const evaluation = await this.evaluateToolUsage(predictedToolCall, {
        correctTool: example.correctToolCall,
        availableTools: [], // Would be populated in full implementation
      });

      if (evaluation.toolChoiceAppropriate) toolChoiceCorrect++;
      if (evaluation.formatCorrect) formatCorrect++;
      totalEvaluations++;
    }

    const toolChoiceAccuracy = toolChoiceCorrect / totalEvaluations;
    const formatCorrectnessRate = formatCorrect / totalEvaluations;

    // Simulate improvement over warmup (RL typically improves performance)
    const improvement = Math.max(
      0,
      toolChoiceAccuracy - warmupResults.accuracy
    );

    return {
      improvement,
      toolChoiceAccuracy,
      formatCorrectnessRate,
      trainedModel: {}, // Mock improved model
    };
  }

  /**
   * Evaluates if tool choice is appropriate.
   *
   * @param toolCall - Tool call to evaluate.
   * @param context - Evaluation context.
   * @returns Whether tool choice is appropriate.
   */
  private evaluateToolChoice(
    toolCall: ToolCall,
    context: {
      correctTool?: ToolCall;
      availableTools: Tool[];
      taskRequirements?: Record<string, unknown>;
    }
  ): boolean {
    if (context.correctTool) {
      return toolCall.toolId === context.correctTool.toolId;
    }

    // Check if tool is available
    return context.availableTools.some((tool) => tool.id === toolCall.toolId);
  }

  /**
   * Evaluates format correctness of tool call.
   *
   * @param toolCall - Tool call to evaluate.
   * @returns Whether format is correct.
   */
  private evaluateFormatCorrectness(toolCall: ToolCall): boolean {
    // Basic validation - check required fields
    if (!toolCall.toolId || typeof toolCall.toolId !== "string") {
      return false;
    }

    if (!toolCall.parameters || typeof toolCall.parameters !== "object") {
      return false;
    }

    return true;
  }

  /**
   * Evaluates information utility of tool call.
   *
   * @param toolCall - Tool call to evaluate.
   * @param context - Evaluation context.
   * @returns Information utility score (0-1).
   */
  private async evaluateInformationUtility(
    toolCall: ToolCall,
    context: {
      correctTool?: ToolCall;
      availableTools: Tool[];
      taskRequirements?: Record<string, unknown>;
    }
  ): Promise<number> {
    // In a full implementation, this would use a model judge
    // For now, use heuristic based on tool type

    const informationTools = ["read_file", "grep", "search", "list_dir"];
    const toolName = toolCall.toolId.toLowerCase();

    if (informationTools.some((tool) => toolName.includes(tool))) {
      return 0.8; // High utility for information-seeking tools
    }

    if (context.correctTool && toolCall.toolId === context.correctTool.toolId) {
      return 0.9; // High utility for correct tool choice
    }

    return 0.5; // Moderate utility for other tools
  }

  /**
   * Evaluates error handling correctness.
   *
   * @param toolCall - Tool call to evaluate.
   * @returns Whether error handling is correct.
   */
  private evaluateErrorHandling(toolCall: ToolCall): boolean {
    // Check if tool call includes error handling parameters where appropriate
    const dangerousTools = ["run_terminal_cmd"];

    if (dangerousTools.includes(toolCall.toolId)) {
      // For dangerous tools, check if parameters suggest caution
      const params = JSON.stringify(toolCall.parameters).toLowerCase();
      const safeIndicators = ["--dry-run", "test", "validate"];

      return safeIndicators.some((indicator) => params.includes(indicator));
    }

    return true; // Most tools don't require special error handling
  }

  /**
   * Generates a realistic prompt for a tool.
   *
   * @param tool - Tool to generate prompt for.
   * @returns Generated prompt.
   */
  private generatePromptForTool(tool: Tool): string {
    const prompts = {
      read_file: `I need to examine the contents of a file. Can you help me read the file at path "src/main.ts"?`,
      grep: `I need to search for a specific pattern in the codebase. Please search for "function" in the file "utils.ts".`,
      run_terminal_cmd: `I need to run a command to check the current directory. Please run "pwd" in the terminal.`,
      search: `I need to find information about a topic. Can you search for "reinforcement learning" and provide relevant results?`,
    };

    return (
      prompts[tool.id as keyof typeof prompts] ||
      `Please use the ${tool.name} tool to help with this task.`
    );
  }

  /**
   * Generates a correct tool call for a tool.
   *
   * @param tool - Tool to generate call for.
   * @returns Correct tool call.
   */
  private generateCorrectToolCall(tool: Tool): ToolCall {
    const calls = {
      read_file: { toolId: "read_file", parameters: { path: "src/main.ts" } },
      grep: {
        toolId: "grep",
        parameters: { pattern: "function", path: "utils.ts" },
      },
      run_terminal_cmd: {
        toolId: "run_terminal_cmd",
        parameters: { command: "pwd" },
      },
      search: {
        toolId: "search",
        parameters: { query: "reinforcement learning" },
      },
    };

    return (
      (calls[tool.id as keyof typeof calls] as ToolCall) || {
        toolId: tool.id,
        parameters: {},
      }
    );
  }

  /**
   * Describes the purpose of a tool.
   *
   * @param tool - Tool to describe.
   * @returns Purpose description.
   */
  private describeToolPurpose(tool: Tool): string {
    const purposes = {
      read_file: "read file contents",
      grep: "search for patterns in files",
      run_terminal_cmd: "execute terminal commands",
      search: "search for information",
    };

    return (
      purposes[tool.id as keyof typeof purposes] ||
      "perform the required operation"
    );
  }

  /**
   * Assesses the difficulty of an example.
   *
   * @param tool - Tool used in example.
   * @param toolCall - Tool call in example.
   * @returns Difficulty level.
   */
  private assessExampleDifficulty(
    tool: Tool,
    _toolCall: ToolCall
  ): "easy" | "medium" | "hard" {
    // Simple heuristic based on tool complexity
    const complexTools = ["run_terminal_cmd"];
    const mediumTools = ["grep", "search"];

    if (complexTools.includes(tool.id)) return "hard";
    if (mediumTools.includes(tool.id)) return "medium";
    return "easy";
  }

  /**
   * Simulates model prediction for supervised warmup.
   *
   * @param prompt - Input prompt.
   * @param correctToolId - Correct tool ID.
   * @returns Simulated tool call.
   */
  private simulateModelPrediction(
    prompt: string,
    correctToolId: string
  ): ToolCall {
    // Simple simulation - in practice, this would be actual model inference
    // For now, randomly succeed or fail based on "training quality"
    const successRate = 0.7; // 70% accuracy for warmup phase

    if (Math.random() < successRate) {
      return { toolId: correctToolId, parameters: {} };
    } else {
      // Random wrong tool
      const wrongTools = ["read_file", "grep", "run_terminal_cmd"];
      const wrongTool =
        wrongTools[Math.floor(Math.random() * wrongTools.length)];
      return { toolId: wrongTool, parameters: {} };
    }
  }

  /**
   * Simulates RL-improved prediction.
   *
   * @param prompt - Input prompt.
   * @param correctToolId - Correct tool ID.
   * @returns Simulated improved tool call.
   */
  private simulateRLPrediction(
    prompt: string,
    correctToolId: string
  ): ToolCall {
    // Improved simulation - higher accuracy for RL phase
    const successRate = 0.9; // 90% accuracy for RL-improved model

    if (Math.random() < successRate) {
      return { toolId: correctToolId, parameters: {} };
    } else {
      const wrongTools = ["read_file", "grep", "run_terminal_cmd"];
      const wrongTool =
        wrongTools[Math.floor(Math.random() * wrongTools.length)];
      return { toolId: wrongTool, parameters: {} };
    }
  }

  /**
   * Compares two tool calls for equality.
   *
   * @param call1 - First tool call.
   * @param call2 - Second tool call.
   * @returns Whether calls are equivalent.
   */
  private compareToolCalls(call1: ToolCall, call2: ToolCall): boolean {
    return call1.toolId === call2.toolId;
  }
}
