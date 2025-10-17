/**
 * @fileoverview
 * End-to-end integration tests for the complete RL training pipeline.
 * Tests the integration of ThinkingBudgetManager, MinimalDiffEvaluator,
 * ModelBasedJudge, TurnLevelRLTrainer, and PerformanceTracker.
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { PerformanceTracker } from "../../src/rl/PerformanceTracker";
import { TurnLevelRLTrainer } from "../../src/rl/TurnLevelRLTrainer";
import type { ConversationTrajectory } from "../../src/types/agentic-rl";

describe("RL Pipeline Integration", () => {
  let trainer: TurnLevelRLTrainer;
  let tracker: PerformanceTracker;

  beforeEach(() => {
    trainer = new TurnLevelRLTrainer();
    tracker = new PerformanceTracker();
    tracker.startCollection();
  });

  describe("End-to-End Training Flow", () => {
    it("should process complete training pipeline with all RL components", async () => {
      // Create a realistic conversation trajectory
      const trajectory: ConversationTrajectory = {
        conversationId: "test-conv-001",
        totalReward: 0,
        turns: [
          {
            turnNumber: 1,
            toolChoice: {
              toolId: "read_file",
              parameters: { path: "src/test.ts" },
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
              toolId: "search_replace",
              parameters: {
                old_string: "const x = 1;",
                new_string: "const x = 2;",
              },
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
          qualityScore: 0.9,
          efficiencyScore: 0.85,
          tokensConsumed: 1000,
          completionTimeMs: 5000,
        },
      };

      // Train on the trajectory
      const stats = await trainer.trainOnConversation(trajectory);

      // Verify training completed
      expect(stats.trajectoriesProcessed).toBe(1);
      expect(stats.averageReward).toBeGreaterThan(0);
      expect(stats.trainingTimeMs).toBeGreaterThan(0);

      // Verify RL components were engaged
      expect(stats.policyLoss).toBeDefined();
      expect(stats.valueLoss).toBeDefined();
      expect(stats.klDivergence).toBeDefined();
    }, 10000);

    it("should allocate thinking budgets based on task complexity", () => {
      // Get initial metrics
      const initialMetrics = trainer.getBudgetMetrics();
      const initialAllocations = initialMetrics.totalAllocations;

      // Allocate budget for complex task
      const budget = trainer.allocateThinkingBudget("task-001", {
        toolCount: 5,
        contextSize: 5000,
        stepCount: 10,
        multiTurn: true,
        hasExternalCalls: true,
      });

      // Should allocate complex budget (complex tier)
      expect(budget).toBeGreaterThan(2000);
      expect(budget).toBeLessThanOrEqual(8000);

      // Verify metrics updated
      const metrics = trainer.getBudgetMetrics();
      expect(metrics.totalAllocations).toBe(initialAllocations + 1);
      expect(metrics.averageTokensAllocated).toBeGreaterThan(0);
    });

    it("should evaluate minimality for code changes", async () => {
      const trajectory: ConversationTrajectory = {
        conversationId: "test-conv-002",
        totalReward: 0,
        turns: [
          {
            turnNumber: 1,
            toolChoice: {
              toolId: "search_replace",
              parameters: {
                old_string: "const x = 1;",
                new_string: "const x = 1; // comment",
              },
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
          qualityScore: 0.7,
          efficiencyScore: 0.75,
          tokensConsumed: 800,
          completionTimeMs: 4000,
        },
      };

      // Compute rewards (includes minimality evaluation)
      const rewardedTrajectory = await trainer.computeTurnRewards(trajectory);

      // Verify rewards were computed
      expect(rewardedTrajectory.turns[0].totalReward).toBeGreaterThan(0);
      expect(rewardedTrajectory.turns[0].informationGain).toBeDefined();
      expect(rewardedTrajectory.turns[0].formatCorrectness).toBeDefined();
    });

    it("should use model-based judgment for evaluation", async () => {
      const trajectory: ConversationTrajectory = {
        conversationId: "test-conv-003",
        totalReward: 0,
        turns: [
          {
            turnNumber: 1,
            toolChoice: {
              toolId: "grep",
              parameters: {
                pattern: "test",
                path: "src/",
              },
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
          efficiencyScore: 0.82,
          tokensConsumed: 1200,
          completionTimeMs: 6000,
        },
      };

      // Compute rewards (uses ModelBasedJudge for information gain)
      const rewardedTrajectory = await trainer.computeTurnRewards(trajectory);

      // Verify judgment was applied
      expect(rewardedTrajectory.turns[0].informationGain).toBeGreaterThan(0);
      expect(rewardedTrajectory.turns[0].informationGain).toBeLessThanOrEqual(
        1
      );
    });
  });

  describe("Performance Tracking Integration", () => {
    it("should track thinking budget allocations", async () => {
      await tracker.recordThinkingBudget("task-001", {
        allocatedTokens: 5000,
        complexityLevel: "complex",
        confidence: 0.9,
      });

      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBeGreaterThanOrEqual(0);

      // Verify data is exportable
      const data = tracker.exportTrainingData();
      expect(data.length).toBeGreaterThan(0);

      const budgetEvent = data.find(
        (e) => e.type === "thinking-budget-allocation"
      );
      expect(budgetEvent).toBeDefined();
    });

    it("should track budget usage", async () => {
      await tracker.recordBudgetUsage("task-001", {
        tokensUsed: 4500,
        tokensAllocated: 5000,
        utilizationRate: 0.9,
      });

      const data = tracker.exportTrainingData();
      const usageEvent = data.find((e) => e.type === "thinking-budget-usage");
      expect(usageEvent).toBeDefined();
    });

    it("should track minimality evaluations", async () => {
      await tracker.recordMinimalityEvaluation("task-001", {
        minimalityFactor: 0.85,
        astSimilarity: 0.92,
        scaffoldingPenalty: 0.05,
        linesChanged: 15,
        qualityAssessment: "minimal",
      });

      const data = tracker.exportTrainingData();
      const minimalityEvent = data.find(
        (e) => e.type === "minimality-evaluation"
      );
      expect(minimalityEvent).toBeDefined();
      expect((minimalityEvent!.data as any).minimalityFactor).toBe(0.85);
    });

    it("should track model judgments", async () => {
      await tracker.recordJudgment("task-001", {
        overallScore: 0.82,
        overallConfidence: 0.88,
        allCriteriaPass: true,
        criteriaScores: {
          faithfulness: 0.9,
          relevance: 0.85,
          minimality: 0.8,
          safety: 0.95,
        },
        evaluationTimeMs: 250,
      });

      const data = tracker.exportTrainingData();
      const judgmentEvent = data.find((e) => e.type === "model-judgment");
      expect(judgmentEvent).toBeDefined();
      expect((judgmentEvent!.data as any).overallScore).toBe(0.82);
    });

    it("should track RL training metrics", async () => {
      await tracker.recordRLTrainingMetrics({
        trajectoriesProcessed: 100,
        averageReward: 0.75,
        policyLoss: -0.05,
        valueLoss: 0.02,
        klDivergence: 0.01,
        trainingTimeMs: 5000,
      });

      const data = tracker.exportTrainingData();
      const metricsEvent = data.find((e) => e.type === "rl-training-metrics");
      expect(metricsEvent).toBeDefined();
      expect((metricsEvent!.data as any).trajectoriesProcessed).toBe(100);
    });
  });

  describe("Multi-Trajectory Training", () => {
    it("should handle batch training with multiple trajectories", async () => {
      const trajectories: ConversationTrajectory[] = [
        {
          conversationId: "batch-001",
          totalReward: 0,
          turns: [
            {
              turnNumber: 1,
              toolChoice: {
                toolId: "read_file",
                parameters: { path: "file1.ts" },
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
                toolId: "write",
                parameters: { content: "updated content" },
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
            qualityScore: 0.9,
            efficiencyScore: 0.85,
            tokensConsumed: 1000,
            completionTimeMs: 5000,
          },
        },
        {
          conversationId: "batch-002",
          totalReward: 0,
          turns: [
            {
              turnNumber: 1,
              toolChoice: {
                toolId: "search_replace",
                parameters: {
                  old_string: "old",
                  new_string: "new",
                },
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
                parameters: { pattern: "test" },
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
            efficiencyScore: 0.82,
            tokensConsumed: 1200,
            completionTimeMs: 6000,
          },
        },
      ];

      const stats = await trainer.trainOnTrajectories(trajectories);

      expect(stats.trajectoriesProcessed).toBe(2);
      expect(stats.averageReward).toBeGreaterThan(0);
    }, 15000);

    it("should compute group advantages using GRPO", async () => {
      const trajectories: ConversationTrajectory[] = Array.from(
        { length: 5 },
        (_, i) => ({
          conversationId: `grpo-${i}`,
          totalReward: 0,
          turns: [
            {
              turnNumber: 1,
              toolChoice: {
                toolId: "grep",
                parameters: { pattern: "test" },
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
                toolId: "read_file",
                parameters: { path: "test.ts" },
              },
              informationGain: 0,
              formatCorrectness: 0,
              taskProgress: 0,
              safetyScore: 0,
              totalReward: 0,
            },
          ],
          finalOutcome: {
            success: i % 2 === 0,
            qualityScore: 0.5 + i * 0.1,
            efficiencyScore: 0.6 + i * 0.05,
            tokensConsumed: 1000 + i * 100,
            completionTimeMs: 5000 + i * 500,
          },
        })
      );

      const stats = await trainer.trainOnTrajectories(trajectories);

      expect(stats.trajectoriesProcessed).toBe(5);
      expect(stats.policyLoss).toBeDefined();
      expect(stats.valueLoss).toBeDefined();
      expect(stats.klDivergence).toBeDefined();
    }, 20000);
  });

  describe("Performance Budget Validation", () => {
    it("should complete single trajectory training under 5s", async () => {
      const trajectory: ConversationTrajectory = {
        conversationId: "perf-test-001",
        totalReward: 0,
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
              toolId: "write",
              parameters: { content: "test" },
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
          qualityScore: 0.9,
          efficiencyScore: 0.85,
          tokensConsumed: 1000,
          completionTimeMs: 5000,
        },
      };

      const startTime = Date.now();
      await trainer.trainOnConversation(trajectory);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(5000);
    });

    it("should maintain performance metrics correctly", async () => {
      const executionId = await tracker.startTaskExecution(
        "perf-task-001",
        "agent-001",
        {
          taskId: "perf-task-001",
          selectedAgent: "agent-001",
          routingStrategy: "capability-match",
          confidence: 0.9,
          alternativesConsidered: [],
          rationale: "Best match for task",
          timestamp: new Date().toISOString(),
        }
      );

      expect(executionId).toBeTruthy();

      await tracker.completeTaskExecution(executionId, {
        success: true,
        qualityScore: 0.85,
        efficiencyScore: 0.88,
        tokensConsumed: 950,
        completionTimeMs: 4500,
      });

      const stats = tracker.getStats();
      expect(stats.totalTaskExecutions).toBe(1);
      expect(stats.overallSuccessRate).toBe(1);
    });
  });

  describe("Edge Cases & Error Handling", () => {
    describe("Trajectory Validation", () => {
      it("should reject trajectories with insufficient turns", async () => {
        const shortTrajectory: ConversationTrajectory = {
          conversationId: "short-001",
          totalReward: 0,
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
          ],
          finalOutcome: {
            success: true,
            qualityScore: 0.9,
            efficiencyScore: 0.85,
            tokensConsumed: 1000,
            completionTimeMs: 5000,
          },
        };

        await expect(
          trainer.trainOnConversation(shortTrajectory)
        ).rejects.toThrow(/no valid trajectories/i);
      });

      it("should handle trajectories at minimum length boundary", async () => {
        const minTrajectory: ConversationTrajectory = {
          conversationId: "min-001",
          totalReward: 0,
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
                toolId: "write",
                parameters: { content: "test" },
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
            qualityScore: 0.9,
            efficiencyScore: 0.85,
            tokensConsumed: 1000,
            completionTimeMs: 5000,
          },
        };

        const stats = await trainer.trainOnConversation(minTrajectory);
        expect(stats.trajectoriesProcessed).toBe(1);
      });

      it("should reject trajectories without final outcome", async () => {
        const noOutcomeTrajectory: any = {
          conversationId: "no-outcome-001",
          totalReward: 0,
          turns: [
            {
              turnNumber: 1,
              toolChoice: { toolId: "read_file", parameters: {} },
              informationGain: 0,
              formatCorrectness: 0,
              taskProgress: 0,
              safetyScore: 0,
              totalReward: 0,
            },
            {
              turnNumber: 2,
              toolChoice: { toolId: "write", parameters: {} },
              informationGain: 0,
              formatCorrectness: 0,
              taskProgress: 0,
              safetyScore: 0,
              totalReward: 0,
            },
          ],
          finalOutcome: null,
        };

        await expect(
          trainer.trainOnConversation(noOutcomeTrajectory)
        ).rejects.toThrow(/no valid trajectories/i);
      });
    });

    describe("Budget Allocation Edge Cases", () => {
      it("should allocate trivial budget for simple tasks", () => {
        const budget = trainer.allocateThinkingBudget("simple-task", {
          toolCount: 1,
          contextSize: 500,
          stepCount: 1,
          multiTurn: false,
          hasExternalCalls: false,
        });

        expect(budget).toBeGreaterThan(0);
        expect(budget).toBeLessThanOrEqual(500);
      });

      it("should allocate standard budget for moderate tasks", () => {
        const budget = trainer.allocateThinkingBudget("moderate-task", {
          toolCount: 3,
          contextSize: 2000,
          stepCount: 5,
          multiTurn: true,
          hasExternalCalls: false,
        });

        expect(budget).toBeGreaterThan(500);
        expect(budget).toBeLessThanOrEqual(2000);
      });

      it("should handle multiple consecutive allocations", () => {
        const budgets = [];
        for (let i = 0; i < 10; i++) {
          const budget = trainer.allocateThinkingBudget(`task-${i}`, {
            toolCount: 2,
            contextSize: 1000,
            stepCount: 3,
            multiTurn: false,
            hasExternalCalls: false,
          });
          budgets.push(budget);
        }

        expect(budgets.length).toBe(10);
        expect(budgets.every((b) => b > 0)).toBe(true);
      });
    });

    describe("Minimality Evaluation Edge Cases", () => {
      it("should handle non-code tool changes", async () => {
        const trajectory: ConversationTrajectory = {
          conversationId: "non-code-001",
          totalReward: 0,
          turns: [
            {
              turnNumber: 1,
              toolChoice: {
                toolId: "grep",
                parameters: { pattern: "test" },
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
                toolId: "list_dir",
                parameters: { path: "src/" },
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
            efficiencyScore: 0.82,
            tokensConsumed: 1200,
            completionTimeMs: 6000,
          },
        };

        const rewardedTrajectory = await trainer.computeTurnRewards(trajectory);

        // Non-code tools should not be penalized for minimality
        expect(rewardedTrajectory.turns[0].totalReward).toBeGreaterThan(0);
        expect(rewardedTrajectory.turns[1].totalReward).toBeGreaterThan(0);
      });

      it("should handle code changes without old_string", async () => {
        const trajectory: ConversationTrajectory = {
          conversationId: "new-file-001",
          totalReward: 0,
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
                toolId: "write",
                parameters: { content: "const x = 1;" },
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
            qualityScore: 0.9,
            efficiencyScore: 0.85,
            tokensConsumed: 1000,
            completionTimeMs: 5000,
          },
        };

        const rewardedTrajectory = await trainer.computeTurnRewards(trajectory);
        expect(rewardedTrajectory.turns[1].totalReward).toBeGreaterThan(0);
      });
    });

    describe("Safety Evaluation Edge Cases", () => {
      it("should penalize dangerous commands", async () => {
        const dangerousTrajectory: ConversationTrajectory = {
          conversationId: "dangerous-001",
          totalReward: 0,
          turns: [
            {
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
            },
            {
              turnNumber: 2,
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
          ],
          finalOutcome: {
            success: false,
            qualityScore: 0.3,
            efficiencyScore: 0.4,
            tokensConsumed: 500,
            completionTimeMs: 2000,
          },
        };

        const rewardedTrajectory = await trainer.computeTurnRewards(
          dangerousTrajectory
        );

        // Dangerous command should have low reward
        expect(rewardedTrajectory.turns[0].safetyScore).toBeLessThan(0.5);
        expect(rewardedTrajectory.turns[0].totalReward).toBeLessThan(
          rewardedTrajectory.turns[1].totalReward
        );
      });

      it("should reward safe commands", async () => {
        const safeTrajectory: ConversationTrajectory = {
          conversationId: "safe-001",
          totalReward: 0,
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
                toolId: "write",
                parameters: { content: "test" },
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
            qualityScore: 0.9,
            efficiencyScore: 0.85,
            tokensConsumed: 1000,
            completionTimeMs: 5000,
          },
        };

        const rewardedTrajectory = await trainer.computeTurnRewards(
          safeTrajectory
        );

        // Safe commands should have high safety scores
        expect(rewardedTrajectory.turns[0].safetyScore).toBeGreaterThan(0.8);
        expect(rewardedTrajectory.turns[1].safetyScore).toBeGreaterThan(0.8);
      });
    });

    describe("Performance Tracking Edge Cases", () => {
      it("should handle high-volume event tracking", async () => {
        const events = 100;
        for (let i = 0; i < events; i++) {
          await tracker.recordThinkingBudget(`task-${i}`, {
            allocatedTokens: 1000 + i * 10,
            complexityLevel: "standard",
            confidence: 0.9,
          });
        }

        const data = tracker.exportTrainingData();
        const budgetEvents = data.filter(
          (e) => e.type === "thinking-budget-allocation"
        );

        expect(budgetEvents.length).toBeGreaterThanOrEqual(events);
      });

      it("should handle concurrent metric recording", async () => {
        const promises = [];
        for (let i = 0; i < 10; i++) {
          promises.push(
            tracker.recordMinimalityEvaluation(`concurrent-${i}`, {
              minimalityFactor: 0.8,
              astSimilarity: 0.9,
              scaffoldingPenalty: 0.05,
              linesChanged: 10,
              qualityAssessment: "minimal",
            })
          );
        }

        await Promise.all(promises);

        const data = tracker.exportTrainingData();
        const minimalityEvents = data.filter(
          (e) => e.type === "minimality-evaluation"
        );

        expect(minimalityEvents.length).toBeGreaterThanOrEqual(10);
      });

      it("should export data filtered by timestamp", async () => {
        const before = new Date().toISOString();

        await tracker.recordJudgment("filter-test", {
          overallScore: 0.85,
          overallConfidence: 0.9,
          allCriteriaPass: true,
          criteriaScores: { test: 0.85 },
          evaluationTimeMs: 200,
        });

        const allData = tracker.exportTrainingData();
        const filteredData = tracker.exportTrainingData(before);

        expect(filteredData.length).toBeLessThanOrEqual(allData.length);
      });
    });

    describe("GRPO Algorithm Edge Cases", () => {
      it("should handle single trajectory group", async () => {
        const singleTrajectory: ConversationTrajectory = {
          conversationId: "single-grpo",
          totalReward: 0,
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
                toolId: "write",
                parameters: { content: "test" },
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
            qualityScore: 0.9,
            efficiencyScore: 0.85,
            tokensConsumed: 1000,
            completionTimeMs: 5000,
          },
        };

        const stats = await trainer.trainOnConversation(singleTrajectory);

        expect(stats.trajectoriesProcessed).toBe(1);
        expect(stats.averageReward).toBeDefined();
      });

      it("should handle trajectories with varying lengths", async () => {
        const trajectories: ConversationTrajectory[] = [
          {
            conversationId: "varying-1",
            totalReward: 0,
            turns: Array.from({ length: 2 }, (_, i) => ({
              turnNumber: i + 1,
              toolChoice: {
                toolId: "read_file",
                parameters: { path: `file${i}.ts` },
              },
              informationGain: 0,
              formatCorrectness: 0,
              taskProgress: 0,
              safetyScore: 0,
              totalReward: 0,
            })),
            finalOutcome: {
              success: true,
              qualityScore: 0.9,
              efficiencyScore: 0.85,
              tokensConsumed: 1000,
              completionTimeMs: 5000,
            },
          },
          {
            conversationId: "varying-2",
            totalReward: 0,
            turns: Array.from({ length: 5 }, (_, i) => ({
              turnNumber: i + 1,
              toolChoice: {
                toolId: "read_file",
                parameters: { path: `file${i}.ts` },
              },
              informationGain: 0,
              formatCorrectness: 0,
              taskProgress: 0,
              safetyScore: 0,
              totalReward: 0,
            })),
            finalOutcome: {
              success: true,
              qualityScore: 0.85,
              efficiencyScore: 0.82,
              tokensConsumed: 1200,
              completionTimeMs: 6000,
            },
          },
        ];

        const stats = await trainer.trainOnTrajectories(trajectories);

        expect(stats.trajectoriesProcessed).toBe(2);
        expect(stats.averageReward).toBeGreaterThan(0);
      }, 15000);
    });
  });
});
