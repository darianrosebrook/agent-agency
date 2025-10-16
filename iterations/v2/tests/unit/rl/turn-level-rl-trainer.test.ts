/**
 * Turn-Level RL Trainer Unit Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { TurnLevelRLTrainer } from "../../../src/rl/TurnLevelRLTrainer";
import {
  ConversationTrajectory,
  TaskOutcome,
  TurnLevelReward,
} from "../../../src/types/agentic-rl";

describe("TurnLevelRLTrainer", () => {
  let trainer: TurnLevelRLTrainer;
  let mockTrajectory: ConversationTrajectory;

  beforeEach(() => {
    trainer = new TurnLevelRLTrainer();

    // Create mock trajectory
    mockTrajectory = {
      conversationId: "conv-123",
      turns: [
        {
          turnNumber: 1,
          toolChoice: {
            toolId: "read_file",
            parameters: { path: "test.ts" },
          },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        },
        {
          turnNumber: 2,
          toolChoice: {
            toolId: "grep",
            parameters: { pattern: "function", path: "test.ts" },
          },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        },
      ],
      finalOutcome: {
        success: true,
        qualityScore: 0.85,
        efficiencyScore: 0.9,
        tokensConsumed: 1500,
        completionTimeMs: 2500,
      },
      totalReward: 0,
    };
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const config = trainer.getConfig();

      expect(config.learningRate).toBe(1e-5);
      expect(config.discountFactor).toBe(0.99);
      expect(config.batchSize).toBe(32);
      expect(config.minTrajectoryLength).toBe(2);
      expect(config.maxTrajectoryLength).toBe(50);
    });

    it("should override default config", () => {
      const customConfig = {
        learningRate: 1e-4,
        batchSize: 64,
      };

      const trainer = new TurnLevelRLTrainer(customConfig);
      const config = trainer.getConfig();

      expect(config.learningRate).toBe(1e-4);
      expect(config.batchSize).toBe(64);
      expect(config.discountFactor).toBe(0.99); // Unchanged default
    });
  });

  describe("computeTurnRewards", () => {
    it("should compute rewards for all turns", async () => {
      const result = await trainer.computeTurnRewards(mockTrajectory);

      expect(result.turns).toHaveLength(2);
      expect(result.turns[0].totalReward).toBeGreaterThan(0);
      expect(result.turns[1].totalReward).toBeGreaterThan(0);
      expect(result.turns[0].informationGain).toBeDefined();
      expect(result.turns[0].formatCorrectness).toBeDefined();
      expect(result.turns[0].taskProgress).toBeDefined();
      expect(result.turns[0].safetyScore).toBeDefined();
    });

    it("should give higher information gain to information-seeking tools", async () => {
      const result = await trainer.computeTurnRewards(mockTrajectory);

      // read_file and grep should have high information gain
      expect(result.turns[0].informationGain).toBeGreaterThan(0.7);
      expect(result.turns[1].informationGain).toBeGreaterThan(0.7);
    });

    it("should give perfect format correctness for valid tool calls", async () => {
      const result = await trainer.computeTurnRewards(mockTrajectory);

      expect(result.turns[0].formatCorrectness).toBe(1.0);
      expect(result.turns[1].formatCorrectness).toBe(1.0);
    });

    it("should assess task progress appropriately", async () => {
      const result = await trainer.computeTurnRewards(mockTrajectory);

      // Both turns should have reasonable progress scores
      expect(result.turns[0].taskProgress).toBeGreaterThan(0.3);
      expect(result.turns[1].taskProgress).toBeGreaterThan(0.3);
    });

    it("should give high safety scores for safe tools", async () => {
      const result = await trainer.computeTurnRewards(mockTrajectory);

      expect(result.turns[0].safetyScore).toBeGreaterThan(0.8);
      expect(result.turns[1].safetyScore).toBeGreaterThan(0.8);
    });
  });

  describe("trainOnTrajectories", () => {
    it("should train on valid trajectories", async () => {
      const trajectories = [mockTrajectory, mockTrajectory];
      const stats = await trainer.trainOnTrajectories(trajectories);

      expect(stats.trajectoriesProcessed).toBe(2);
      expect(stats.averageReward).toBeGreaterThanOrEqual(0);
      expect(stats.policyLoss).toBeDefined();
      expect(stats.valueLoss).toBeDefined();
      expect(stats.klDivergence).toBeDefined();
      expect(stats.trainingTimeMs).toBeGreaterThanOrEqual(0);
    });

    it("should reject empty trajectory list", async () => {
      await expect(trainer.trainOnTrajectories([])).rejects.toThrow(
        "No valid trajectories provided for training"
      );
    });

    it("should filter out invalid trajectories", async () => {
      const invalidTrajectory = {
        ...mockTrajectory,
        turns: [mockTrajectory.turns[0]], // Too short
      };

      const trajectories = [mockTrajectory, invalidTrajectory];
      const stats = await trainer.trainOnTrajectories(trajectories);

      // Should only process the valid trajectory
      expect(stats.trajectoriesProcessed).toBe(1);
    });

    it("should handle trajectories that are too long", async () => {
      const longTurns: TurnLevelReward[] = [];
      for (let i = 0; i < 60; i++) {
        // Exceeds maxTrajectoryLength
        longTurns.push({
          ...mockTrajectory.turns[0],
          turnNumber: i + 1,
        });
      }

      const longTrajectory = {
        ...mockTrajectory,
        turns: longTurns,
      };

      const trajectories = [mockTrajectory, longTrajectory];
      const stats = await trainer.trainOnTrajectories(trajectories);

      // Should only process the valid trajectory
      expect(stats.trajectoriesProcessed).toBe(1);
    });
  });

  describe("trainOnConversation", () => {
    it("should train on single conversation", async () => {
      const stats = await trainer.trainOnConversation(mockTrajectory);

      expect(stats.trajectoriesProcessed).toBe(1);
      expect(stats.averageReward).toBeGreaterThanOrEqual(0);
    });
  });

  describe("reward computation components", () => {
    describe("judgeInformationGain", () => {
      it("should give high scores to information-seeking tools", async () => {
        const turn: TurnLevelReward = {
          turnNumber: 1,
          toolChoice: { toolId: "read_file", parameters: {} },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        };

        const score = await (trainer as any).judgeInformationGain(turn);
        expect(score).toBeGreaterThan(0.7);
      });

      it("should give moderate scores to other tools", async () => {
        const turn: TurnLevelReward = {
          turnNumber: 1,
          toolChoice: { toolId: "run_terminal_cmd", parameters: {} },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        };

        const score = await (trainer as any).judgeInformationGain(turn);
        expect(score).toBe(0.5);
      });
    });

    describe("evaluateFormatCorrectness", () => {
      it("should give perfect score for valid tool calls", () => {
        const turn: TurnLevelReward = {
          turnNumber: 1,
          toolChoice: { toolId: "read_file", parameters: { path: "test.ts" } },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        };

        const score = (trainer as any).evaluateFormatCorrectness(turn);
        expect(score).toBe(1.0);
      });

      it("should give zero score for invalid tool calls", () => {
        const turn: TurnLevelReward = {
          turnNumber: 1,
          toolChoice: { toolId: "", parameters: {} },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        };

        const score = (trainer as any).evaluateFormatCorrectness(turn);
        expect(score).toBe(0.0);
      });
    });

    describe("evaluateSafety", () => {
      it("should penalize dangerous commands", () => {
        const turn: TurnLevelReward = {
          turnNumber: 1,
          toolChoice: {
            toolId: "run_terminal_cmd",
            parameters: { command: "rm -rf /" },
          },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        };

        const score = (trainer as any).evaluateSafety(turn);
        expect(score).toBeLessThan(0.5);
      });

      it("should give high scores for safe actions", () => {
        const turn: TurnLevelReward = {
          turnNumber: 1,
          toolChoice: {
            toolId: "read_file",
            parameters: { path: "test.ts" },
          },
          informationGain: 0,
          formatCorrectness: 0,
          taskProgress: 0,
          safetyScore: 0,
          totalReward: 0,
        };

        const score = (trainer as any).evaluateSafety(turn);
        expect(score).toBeGreaterThan(0.8);
      });
    });
  });

  describe("trajectory validation", () => {
    it("should accept valid trajectories", () => {
      const validTrajectories = (trainer as any).validateTrajectories([
        mockTrajectory,
      ]);
      expect(validTrajectories).toHaveLength(1);
    });

    it("should reject trajectories that are too short", () => {
      const shortTrajectory = {
        ...mockTrajectory,
        turns: [mockTrajectory.turns[0]],
      };

      const validTrajectories = (trainer as any).validateTrajectories([
        shortTrajectory,
      ]);
      expect(validTrajectories).toHaveLength(0);
    });

    it("should reject trajectories that are too long", () => {
      const longTurns: TurnLevelReward[] = [];
      for (let i = 0; i < 60; i++) {
        longTurns.push({ ...mockTrajectory.turns[0], turnNumber: i + 1 });
      }

      const longTrajectory = {
        ...mockTrajectory,
        turns: longTurns,
      };

      const validTrajectories = (trainer as any).validateTrajectories([
        longTrajectory,
      ]);
      expect(validTrajectories).toHaveLength(0);
    });

    it("should reject trajectories without outcomes", () => {
      const invalidTrajectory = {
        ...mockTrajectory,
        finalOutcome: undefined as any,
      };

      const validTrajectories = (trainer as any).validateTrajectories([
        invalidTrajectory,
      ]);
      expect(validTrajectories).toHaveLength(0);
    });
  });

  describe("grouping and advantage computation", () => {
    it("should group trajectories by similarity", () => {
      const trajectory1 = { ...mockTrajectory, turns: mockTrajectory.turns }; // length 2
      const trajectory2 = { ...mockTrajectory, turns: mockTrajectory.turns }; // length 2
      const trajectory3 = {
        ...mockTrajectory,
        turns: [
          ...mockTrajectory.turns,
          mockTrajectory.turns[0],
          mockTrajectory.turns[1],
        ],
      }; // length 4
      const trajectory4 = {
        ...mockTrajectory,
        turns: [
          ...mockTrajectory.turns,
          mockTrajectory.turns[0],
          mockTrajectory.turns[1],
        ],
      }; // length 4

      const trajectories = [trajectory1, trajectory2, trajectory3, trajectory4];
      const groups = (trainer as any).groupTrajectoriesBySimilarity(
        trajectories
      );

      expect(groups.length).toBeGreaterThan(0); // At least one group
      // Should have groups for different lengths (only groups with 2+ trajectories are kept)
      const totalTrajectoriesInGroups = groups.reduce(
        (sum: number, group: any) => sum + group.length,
        0
      );
      expect(totalTrajectoriesInGroups).toBeGreaterThan(0);
    });

    it("should compute group advantages", async () => {
      const rlTrajectory1 = {
        conversationId: "conv-1",
        turns: [
          {
            turnNumber: 1,
            state: {},
            action: {},
            reward: 1.0,
            advantage: 0,
            logProb: 0,
          },
          {
            turnNumber: 2,
            state: {},
            action: {},
            reward: 2.0,
            advantage: 0,
            logProb: 0,
          },
        ],
        finalOutcome: {} as TaskOutcome,
        totalReward: 3.0,
      };

      const rlTrajectory2 = {
        conversationId: "conv-2",
        turns: [
          {
            turnNumber: 1,
            state: {},
            action: {},
            reward: 1.5,
            advantage: 0,
            logProb: 0,
          },
          {
            turnNumber: 2,
            state: {},
            action: {},
            reward: 2.5,
            advantage: 0,
            logProb: 0,
          },
        ],
        finalOutcome: {} as TaskOutcome,
        totalReward: 4.0,
      };

      const groups = [[rlTrajectory1, rlTrajectory2]];
      const result = await (trainer as any).computeGroupAdvantages(groups);

      expect(result).toHaveLength(2);
      expect(result[0].turns[0].advantage).toBeDefined();
      expect(result[0].turns[1].advantage).toBeDefined();
      expect(result[1].turns[0].advantage).toBeDefined();
      expect(result[1].turns[1].advantage).toBeDefined();
    });
  });

  describe("statistics and configuration", () => {
    it("should provide training statistics", () => {
      const stats = trainer.getTrainingStats();

      expect(stats.trajectoriesProcessed).toBe(0);
      expect(typeof stats.averageReward).toBe("number");
      expect(typeof stats.trainingTimeMs).toBe("number");
      expect(stats.timestamp).toBeDefined();
    });

    it("should update statistics after training", async () => {
      const initialStats = trainer.getTrainingStats();
      await trainer.trainOnConversation(mockTrajectory);
      const updatedStats = trainer.getTrainingStats();

      expect(updatedStats.trajectoriesProcessed).toBe(
        initialStats.trajectoriesProcessed + 1
      );
    });

    it("should allow configuration updates", () => {
      const newConfig = { learningRate: 1e-3 };
      trainer.updateConfig(newConfig);

      const config = trainer.getConfig();
      expect(config.learningRate).toBe(1e-3);
    });
  });

  describe("edge cases", () => {
    it("should handle single turn trajectories at minimum length", () => {
      trainer.updateConfig({ minTrajectoryLength: 1 });

      const singleTurnTrajectory = {
        ...mockTrajectory,
        turns: [mockTrajectory.turns[0]],
      };

      const validTrajectories = (trainer as any).validateTrajectories([
        singleTurnTrajectory,
      ]);
      expect(validTrajectories).toHaveLength(1);
    });

    it("should handle trajectories with zero reward", async () => {
      const zeroRewardTrajectory = {
        ...mockTrajectory,
        turns: mockTrajectory.turns.map((turn) => ({
          ...turn,
          totalReward: 0,
        })),
        totalReward: 0,
      };

      const stats = await trainer.trainOnConversation(zeroRewardTrajectory);
      expect(stats.averageReward).toBe(0);
    });

    it("should handle concurrent training calls", async () => {
      const trainer1 = new TurnLevelRLTrainer();
      const trainer2 = new TurnLevelRLTrainer();

      const promises = [
        trainer1.trainOnConversation(mockTrajectory),
        trainer2.trainOnConversation(mockTrajectory),
      ];

      const results = await Promise.all(promises);

      expect(results).toHaveLength(2);
      expect(results[0].trajectoriesProcessed).toBe(1);
      expect(results[1].trajectoriesProcessed).toBe(1);
    });
  });
});
