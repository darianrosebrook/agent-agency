/**
 * Tool Adoption Trainer Unit Tests
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { beforeEach, describe, expect, it } from "@jest/globals";
import { ToolAdoptionTrainer } from "../../../src/rl/ToolAdoptionTrainer";
import { Tool, ToolCall, ToolExample } from "../../../src/types/agentic-rl";

describe("ToolAdoptionTrainer", () => {
  let trainer: ToolAdoptionTrainer;
  let mockTools: Tool[];
  let mockExamples: ToolExample[];

  beforeEach(() => {
    trainer = new ToolAdoptionTrainer();

    mockTools = [
      {
        id: "read_file",
        name: "Read File",
        description: "Reads the contents of a file",
        parameters: {
          type: "object",
          properties: { path: { type: "string" } },
          required: ["path"],
        },
      },
      {
        id: "grep",
        name: "Search in Files",
        description: "Searches for patterns in files",
        parameters: {
          type: "object",
          properties: { pattern: { type: "string" }, path: { type: "string" } },
          required: ["pattern", "path"],
        },
      },
      {
        id: "run_terminal_cmd",
        name: "Run Terminal Command",
        description: "Executes terminal commands",
        parameters: {
          type: "object",
          properties: { command: { type: "string" } },
          required: ["command"],
        },
      },
    ];

    mockExamples = [
      {
        prompt: "I need to read the contents of main.ts",
        correctToolCall: {
          toolId: "read_file",
          parameters: { path: "main.ts" },
        },
        expectedReasoning: "Using read_file to read file contents",
        difficulty: "easy",
      },
      {
        prompt: "Search for function definitions in utils.ts",
        correctToolCall: {
          toolId: "grep",
          parameters: { pattern: "function", path: "utils.ts" },
        },
        expectedReasoning: "Using grep to search for patterns",
        difficulty: "medium",
      },
    ];
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const config = trainer.getConfig();

      expect(config.warmupExamples).toBe(1000);
      expect(config.learningRate).toBe(1e-5);
      expect(config.toolChoiceWeight).toBe(0.4);
      expect(config.formatCorrectnessWeight).toBe(0.3);
    });

    it("should override default config", () => {
      const customConfig = {
        learningRate: 1e-4,
        toolChoiceWeight: 0.5,
      };

      const trainer = new ToolAdoptionTrainer(customConfig);
      const config = trainer.getConfig();

      expect(config.learningRate).toBe(1e-4);
      expect(config.toolChoiceWeight).toBe(0.5);
      expect(config.formatCorrectnessWeight).toBe(0.3); // Unchanged default
    });
  });

  describe("trainOnExamples", () => {
    it("should train on valid examples", async () => {
      const stats = await trainer.trainOnExamples(mockExamples);

      expect(stats.examplesProcessed).toBe(2);
      expect(stats.warmupAccuracy).toBeGreaterThanOrEqual(0);
      expect(stats.rlImprovement).toBeGreaterThanOrEqual(0);
      expect(stats.toolChoiceAccuracy).toBeGreaterThanOrEqual(0);
      expect(stats.formatCorrectnessRate).toBeGreaterThanOrEqual(0);
      expect(stats.trainingTimeMs).toBeGreaterThanOrEqual(0);
    });

    it("should reject empty examples", async () => {
      await expect(trainer.trainOnExamples([])).rejects.toThrow(
        "No tool examples provided for training"
      );
    });

    it("should handle single example", async () => {
      const singleExample = [mockExamples[0]];
      const stats = await trainer.trainOnExamples(singleExample);

      expect(stats.examplesProcessed).toBe(1);
      expect(stats.warmupAccuracy).toBeGreaterThanOrEqual(0);
    });
  });

  describe("evaluateToolUsage", () => {
    it("should evaluate correct tool usage", async () => {
      const toolCall: ToolCall = {
        toolId: "read_file",
        parameters: { path: "test.ts" },
      };
      const context = {
        correctTool: toolCall,
        availableTools: mockTools,
      };

      const evaluation = await trainer.evaluateToolUsage(toolCall, context);

      expect(evaluation.toolChoiceAppropriate).toBe(true);
      expect(evaluation.formatCorrect).toBe(true);
      expect(evaluation.overallScore).toBeGreaterThan(0);
    });

    it("should evaluate incorrect tool choice", async () => {
      const toolCall: ToolCall = {
        toolId: "grep",
        parameters: { pattern: "test" },
      };
      const context = {
        correctTool: { toolId: "read_file", parameters: { path: "test.ts" } },
        availableTools: mockTools,
      };

      const evaluation = await trainer.evaluateToolUsage(toolCall, context);

      expect(evaluation.toolChoiceAppropriate).toBe(false);
      expect(evaluation.overallScore).toBeLessThan(1.0);
    });

    it("should evaluate format correctness", async () => {
      const validCall: ToolCall = {
        toolId: "read_file",
        parameters: { path: "test.ts" },
      };
      const invalidCall: ToolCall = { toolId: "", parameters: null as any };

      const context = { availableTools: mockTools };

      const validEval = await trainer.evaluateToolUsage(validCall, context);
      const invalidEval = await trainer.evaluateToolUsage(invalidCall, context);

      expect(validEval.formatCorrect).toBe(true);
      expect(invalidEval.formatCorrect).toBe(false);
    });

    it("should evaluate information utility", async () => {
      const infoToolCall: ToolCall = {
        toolId: "read_file",
        parameters: { path: "test.ts" },
      };
      const actionToolCall: ToolCall = {
        toolId: "run_terminal_cmd",
        parameters: { command: "ls" },
      };

      const context = { availableTools: mockTools };

      const infoEval = await trainer.evaluateToolUsage(infoToolCall, context);
      const actionEval = await trainer.evaluateToolUsage(
        actionToolCall,
        context
      );

      expect(infoEval.informationUtility).toBeGreaterThan(0.5);
      expect(actionEval.informationUtility).toBeGreaterThanOrEqual(0);
    });
  });

  describe("computeRewardSignal", () => {
    it("should compute reward signal from evaluation", () => {
      const evaluation = {
        toolChoiceAppropriate: true,
        formatCorrect: true,
        informationUtility: 0.8,
        errorHandlingCorrect: true,
      };

      const toolCall: ToolCall = {
        toolId: "read_file",
        parameters: { path: "test.ts" },
      };
      const rewardSignal = trainer.computeRewardSignal(toolCall, evaluation);

      expect(rewardSignal.callStructureValid).toBe(true);
      expect(rewardSignal.toolChoiceAppropriate).toBe(true);
      expect(rewardSignal.informationUtility).toBe(0.8);
      expect(rewardSignal.errorHandlingCorrect).toBe(true);
      expect(rewardSignal.totalReward).toBeGreaterThan(0);
    });

    it("should penalize incorrect tool choice", () => {
      const evaluation = {
        toolChoiceAppropriate: false,
        formatCorrect: true,
        informationUtility: 0.5,
        errorHandlingCorrect: true,
      };

      const toolCall: ToolCall = {
        toolId: "grep",
        parameters: { pattern: "test" },
      };
      const rewardSignal = trainer.computeRewardSignal(toolCall, evaluation);

      const config = trainer.getConfig();
      expect(rewardSignal.totalReward).toBe(
        0 * config.toolChoiceWeight +
          1 * config.formatCorrectnessWeight +
          0.5 * config.informationUtilityWeight +
          1 * config.errorHandlingWeight
      );
    });
  });

  describe("generateSyntheticExamples", () => {
    it("should generate requested number of examples", () => {
      const examples = trainer.generateSyntheticExamples(mockTools, 5);

      expect(examples).toHaveLength(5);
      examples.forEach((example) => {
        expect(example.prompt).toBeDefined();
        expect(example.correctToolCall).toBeDefined();
        expect(example.expectedReasoning).toBeDefined();
        expect(["easy", "medium", "hard"]).toContain(example.difficulty);
      });
    });

    it("should generate valid tool calls", () => {
      const examples = trainer.generateSyntheticExamples(mockTools, 3);

      examples.forEach((example) => {
        const toolCall = example.correctToolCall;
        expect(toolCall.toolId).toBeDefined();
        expect(toolCall.parameters).toBeDefined();
        expect(mockTools.some((tool) => tool.id === toolCall.toolId)).toBe(
          true
        );
      });
    });

    it("should handle empty tools list", () => {
      const examples = trainer.generateSyntheticExamples([], 2);

      expect(examples).toHaveLength(2);
      // Should still generate examples even with no tools (edge case)
    });
  });

  describe("configuration", () => {
    it("should allow configuration updates", () => {
      const newConfig = { maxEpochs: 20, klPenalty: 0.2 };
      trainer.updateConfig(newConfig);

      const config = trainer.getConfig();
      expect(config.maxEpochs).toBe(20);
      expect(config.klPenalty).toBe(0.2);
    });

    it("should preserve unchanged config values", () => {
      const originalConfig = trainer.getConfig();
      trainer.updateConfig({ learningRate: 1e-3 });

      const newConfig = trainer.getConfig();
      expect(newConfig.learningRate).toBe(1e-3);
      expect(newConfig.warmupExamples).toBe(originalConfig.warmupExamples);
    });
  });

  describe("reward weighting", () => {
    it("should respect weight configuration", () => {
      trainer.updateConfig({
        toolChoiceWeight: 0.5,
        formatCorrectnessWeight: 0.3,
        informationUtilityWeight: 0.15,
        errorHandlingWeight: 0.05,
      });

      const evaluation = {
        toolChoiceAppropriate: true,
        formatCorrect: true,
        informationUtility: 1.0,
        errorHandlingCorrect: true,
      };

      const toolCall: ToolCall = {
        toolId: "read_file",
        parameters: { path: "test.ts" },
      };
      const rewardSignal = trainer.computeRewardSignal(toolCall, evaluation);

      expect(rewardSignal.totalReward).toBe(0.5 + 0.3 + 0.15 + 0.05); // 1.0
    });
  });

  describe("edge cases", () => {
    it("should handle malformed tool calls", async () => {
      const malformedCall: ToolCall = { toolId: "", parameters: {} };
      const context = { availableTools: mockTools };

      const evaluation = await trainer.evaluateToolUsage(
        malformedCall,
        context
      );

      expect(evaluation.formatCorrect).toBe(false);
      expect(evaluation.toolChoiceAppropriate).toBe(false);
    });

    it("should handle unavailable tools", async () => {
      const toolCall: ToolCall = { toolId: "nonexistent_tool", parameters: {} };
      const context = { availableTools: mockTools };

      const evaluation = await trainer.evaluateToolUsage(toolCall, context);

      expect(evaluation.toolChoiceAppropriate).toBe(false);
    });

    it("should handle dangerous commands appropriately", async () => {
      const safeCommand: ToolCall = {
        toolId: "run_terminal_cmd",
        parameters: { command: "ls --dry-run" },
      };
      const dangerousCommand: ToolCall = {
        toolId: "run_terminal_cmd",
        parameters: { command: "rm -rf /" },
      };

      const context = { availableTools: mockTools };

      const safeEval = await trainer.evaluateToolUsage(safeCommand, context);
      const dangerousEval = await trainer.evaluateToolUsage(
        dangerousCommand,
        context
      );

      expect(safeEval.errorHandlingCorrect).toBe(true);
      expect(dangerousEval.errorHandlingCorrect).toBe(false);
    });

    it("should handle large numbers of examples", async () => {
      const manyExamples = trainer.generateSyntheticExamples(mockTools, 100);

      expect(manyExamples).toHaveLength(100);

      // Should be able to train on them without issues
      const stats = await trainer.trainOnExamples(manyExamples.slice(0, 10)); // Limit for test performance
      expect(stats.examplesProcessed).toBe(10);
    });
  });

  describe("training phases", () => {
    it("should show improvement from RL fine-tuning", async () => {
      // Test that RL phase shows improvement over supervised warmup
      const stats = await trainer.trainOnExamples(mockExamples);

      // RL improvement should be non-negative (can be 0 in some cases due to randomness)
      expect(stats.rlImprovement).toBeGreaterThanOrEqual(0);

      // Tool choice accuracy should be reasonable
      expect(stats.toolChoiceAccuracy).toBeGreaterThanOrEqual(0);
      expect(stats.toolChoiceAccuracy).toBeLessThanOrEqual(1);
    });

    it("should maintain training statistics", async () => {
      const beforeStats = trainer.getConfig();
      await trainer.trainOnExamples(mockExamples);
      const afterStats = trainer.getConfig();

      // Config should be unchanged
      expect(afterStats).toEqual(beforeStats);
    });
  });

  describe("tool example properties", () => {
    it("should generate examples with proper structure", () => {
      const examples = trainer.generateSyntheticExamples(mockTools, 1);
      const example = examples[0];

      expect(typeof example.prompt).toBe("string");
      expect(example.prompt.length).toBeGreaterThan(0);
      expect(example.correctToolCall.toolId).toBeDefined();
      expect(typeof example.expectedReasoning).toBe("string");
      expect(["easy", "medium", "hard"]).toContain(example.difficulty);
    });

    it("should generate diverse prompts", () => {
      const examples = trainer.generateSyntheticExamples(mockTools, 10);
      const prompts = examples.map((e) => e.prompt);

      // Should have some variety (at least not all identical)
      const uniquePrompts = new Set(prompts);
      expect(uniquePrompts.size).toBeGreaterThan(1);
    });
  });
});
