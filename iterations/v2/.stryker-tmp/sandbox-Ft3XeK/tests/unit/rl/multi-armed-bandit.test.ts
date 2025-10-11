/**
 * Multi-Armed Bandit Unit Tests
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { MultiArmedBandit } from "../../../src/rl/MultiArmedBandit";
import {
  AgentProfile,
  AgentQueryResult,
  TaskType,
} from "../../../src/types/agent-registry";

// Mock data for testing
const createMockAgent = (
  id: string,
  name: string,
  successRate: number,
  taskCount: number,
  utilizationPercent: number = 30
): AgentProfile => ({
  id,
  name,
  modelFamily: "gpt-4",
  capabilities: {
    taskTypes: ["code-editing" as TaskType],
    languages: ["TypeScript"],
    specializations: [],
  },
  performanceHistory: {
    successRate,
    averageQuality: successRate,
    averageLatency: 1000,
    taskCount,
  },
  currentLoad: {
    activeTasks: 1,
    queuedTasks: 0,
    utilizationPercent,
  },
  registeredAt: new Date().toISOString(),
  lastActiveAt: new Date().toISOString(),
});

const createMockCandidate = (
  agent: AgentProfile,
  matchScore: number = 1.0
): AgentQueryResult => ({
  agent,
  matchScore,
  matchReason: `Good match for ${agent.capabilities.taskTypes[0]}`,
});

describe("MultiArmedBandit", () => {
  let bandit: MultiArmedBandit;
  let mockAgents: AgentProfile[];
  let mockCandidates: AgentQueryResult[];

  beforeEach(() => {
    bandit = new MultiArmedBandit();
    bandit.reset(); // Reset for deterministic testing

    mockAgents = [
      createMockAgent("agent-1", "Agent One", 0.9, 100, 20), // High performer, low utilization
      createMockAgent("agent-2", "Agent Two", 0.7, 50, 40), // Medium performer, medium utilization
      createMockAgent("agent-3", "Agent Three", 0.5, 10, 80), // Low performer, high utilization
      createMockAgent("agent-4", "Agent Four", 0.0, 0, 0), // New agent, no history
    ];

    mockCandidates = mockAgents.map((agent) => createMockCandidate(agent));
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const bandit = new MultiArmedBandit();
      const stats = bandit.getStats();

      expect(stats.config.explorationRate).toBe(0.2);
      expect(stats.config.decayFactor).toBe(0.995);
      expect(stats.config.minSampleSize).toBe(10);
      expect(stats.config.useUCB).toBe(true);
    });

    it("should override default config", () => {
      const customConfig = {
        explorationRate: 0.1,
        decayFactor: 0.99,
        useUCB: false,
      };

      const bandit = new MultiArmedBandit(customConfig);
      const stats = bandit.getStats();

      expect(stats.config.explorationRate).toBe(0.1);
      expect(stats.config.decayFactor).toBe(0.99);
      expect(stats.config.useUCB).toBe(false);
    });
  });

  describe("select", () => {
    it("should throw error for empty candidates", async () => {
      await expect(bandit.select([], "code-editing")).rejects.toThrow(
        "No candidate agents provided for routing"
      );
    });

    it("should select an agent from candidates", async () => {
      const result = await bandit.select(mockCandidates, "code-editing");

      expect(mockAgents.map((a) => a.id)).toContain(result.id);
    });

    it("should select agents with varying frequencies", async () => {
      // Run enough selections to see pattern emerge
      const selections: string[] = [];

      for (let i = 0; i < 200; i++) {
        const result = await bandit.select(mockCandidates, "code-editing");
        selections.push(result.id);
      }

      // Should have selected multiple different agents (exploration working)
      const uniqueSelections = new Set(selections);
      expect(uniqueSelections.size).toBeGreaterThan(1);

      // Should have some distribution (not all same agent)
      const agentOneSelections = selections.filter(
        (id) => id === "agent-1"
      ).length;
      expect(agentOneSelections).toBeLessThan(selections.length); // Not all selections
      expect(agentOneSelections).toBeGreaterThan(0); // At least some selections
    });
  });

  describe("createRoutingDecision", () => {
    it("should create complete routing decision", () => {
      const taskId = "task-123";
      const selectedAgent = mockAgents[0];
      const taskType = "code-editing" as TaskType;

      const decision = bandit.createRoutingDecision(
        taskId,
        mockCandidates,
        selectedAgent,
        taskType
      );

      expect(decision.taskId).toBe(taskId);
      expect(decision.selectedAgent).toBe(selectedAgent.id);
      expect(decision.routingStrategy).toBe("multi-armed-bandit");
      expect(decision.confidence).toBeGreaterThan(0);
      expect(decision.confidence).toBeLessThanOrEqual(1);
      expect(decision.alternativesConsidered).toHaveLength(
        mockCandidates.length
      );
      expect(decision.rationale).toContain(selectedAgent.name);
      expect(decision.timestamp).toBeDefined();
    });

    it("should include all candidates in alternatives", () => {
      const decision = bandit.createRoutingDecision(
        "task-123",
        mockCandidates,
        mockAgents[0],
        "code-editing"
      );

      const alternativeIds = decision.alternativesConsidered.map(
        (a: any) => a.agentId
      );
      const expectedIds = mockAgents.map((a: AgentProfile) => a.id);

      expect(alternativeIds).toEqual(expect.arrayContaining(expectedIds));
      expect(alternativeIds).toHaveLength(expectedIds.length);
    });

    it("should calculate reasonable confidence scores", () => {
      // Test confidence for experienced agent
      const experiencedAgent = mockAgents[0]; // 100 tasks, 0.9 success rate
      const decision1 = bandit.createRoutingDecision(
        "task-1",
        mockCandidates,
        experiencedAgent,
        "code-editing"
      );

      // Test confidence for new agent
      const newAgent = mockAgents[3]; // 0 tasks, 0.0 success rate
      const decision2 = bandit.createRoutingDecision(
        "task-2",
        mockCandidates,
        newAgent,
        "code-editing"
      );

      expect(decision1.confidence).toBeGreaterThan(decision2.confidence);
      expect(decision1.confidence).toBeGreaterThan(0.5); // Experienced agent should have decent confidence
      expect(decision2.confidence).toBeLessThan(0.5); // New agent should have low confidence
    });
  });

  describe("exploration vs exploitation", () => {
    it("should use exploration for new agents", async () => {
      // Mock Math.random to force exploration
      const originalRandom = Math.random;
      const mockRandom = jest.fn().mockReturnValue(0.1); // Below exploration rate
      Object.defineProperty(Math, "random", {
        value: mockRandom,
        writable: true,
      });

      const result = await bandit.select(mockCandidates, "code-editing");

      // Should explore and potentially pick a less optimal agent
      expect(mockAgents.map((a) => a.id)).toContain(result.id);

      // Restore original Math.random
      Object.defineProperty(Math, "random", {
        value: originalRandom,
        writable: true,
      });
    });

    it("should use exploitation when forced", async () => {
      // Create bandit with no exploration to force exploitation
      const exploitOnlyBandit = new MultiArmedBandit({ explorationRate: 0 });

      // Run multiple times to see exploitation behavior
      const results: string[] = [];
      for (let i = 0; i < 10; i++) {
        const result = await exploitOnlyBandit.select(
          mockCandidates,
          "code-editing"
        );
        results.push(result.id);
      }

      // With no exploration, should select from high-performing agents only
      const validSelections = results.filter((id) =>
        ["agent-1", "agent-2", "agent-3"].includes(id)
      );
      expect(validSelections.length).toBe(results.length); // All selections should be from top performers

      // Should not select very low-performing or new agents
      const invalidSelections = results.filter((id) => id === "agent-4"); // New agent
      expect(invalidSelections.length).toBe(0);
    });
  });

  describe("UCB calculation", () => {
    it("should calculate UCB scores correctly", () => {
      const agent = mockAgents[0]; // 100 tasks, 0.9 success rate

      // Mock bandit with many total tasks
      const testBandit = new MultiArmedBandit();
      (testBandit as any).totalTasks = 1000;

      const ucbScore = (testBandit as any).calculateUCB(agent, "code-editing");

      // UCB should be mean + exploration bonus
      const expectedMean = agent.performanceHistory.successRate;
      const ucbConstant = testBandit.getStats().config.ucbConstant;
      const expectedBonus = ucbConstant * Math.sqrt(Math.log(1001) / 101);

      expect(ucbScore).toBeGreaterThan(expectedMean);
      expect(ucbScore).toBeCloseTo(expectedMean + expectedBonus, 5);
    });

    it("should boost exploration for new agents", () => {
      const newAgent = mockAgents[3]; // 0 tasks
      const experiencedAgent = mockAgents[0]; // 100 tasks

      const newScore = (bandit as any).calculateUCB(newAgent, "code-editing");
      const experiencedScore = (bandit as any).calculateUCB(
        experiencedAgent,
        "code-editing"
      );

      // New agent should get exploration boost
      expect(newScore).toBeGreaterThan(newAgent.performanceHistory.successRate);
      expect(newScore).toBeGreaterThan(experiencedScore);
    });
  });

  describe("exploration decay", () => {
    it("should decay exploration rate over time", () => {
      const initialEpsilon = (bandit as any).calculateEpsilon();

      // Simulate many tasks
      (bandit as any).totalTasks = 1000;
      const laterEpsilon = (bandit as any).calculateEpsilon();

      expect(laterEpsilon).toBeLessThan(initialEpsilon);
    });

    it("should maintain minimum exploration rate", () => {
      // Simulate many tasks to decay exploration
      (bandit as any).totalTasks = 10000;
      const epsilon = (bandit as any).calculateEpsilon();

      expect(epsilon).toBeGreaterThanOrEqual(0.01); // Minimum exploration rate
    });
  });

  describe("statistics", () => {
    it("should track total tasks", () => {
      expect(bandit.getStats().totalTasks).toBe(0);

      // Run some selections
      bandit.select(mockCandidates, "code-editing");
      bandit.select(mockCandidates, "code-editing");

      expect(bandit.getStats().totalTasks).toBe(2);
    });

    it("should provide current epsilon", () => {
      const stats = bandit.getStats();

      expect(stats.currentEpsilon).toBeGreaterThan(0);
      expect(stats.currentEpsilon).toBeLessThanOrEqual(0.2); // Initial exploration rate
    });
  });

  describe("updateWithOutcome", () => {
    it("should accept outcome updates without error", () => {
      expect(() => {
        bandit.updateWithOutcome("agent-1", true, 0.9, 1000);
      }).not.toThrow();
    });

    // In a full implementation, this would test actual learning
    // For now, it's a no-op that provides a hook for future learning
  });

  describe("edge cases", () => {
    it("should handle single candidate", async () => {
      const singleCandidate = [mockCandidates[0]];
      const result = await bandit.select(singleCandidate, "code-editing");

      expect(result.id).toBe(singleCandidate[0].agent.id);
    });

    it("should handle candidates with same performance", async () => {
      const samePerformanceCandidates = [
        createMockCandidate(createMockAgent("agent-a", "Agent A", 0.8, 50)),
        createMockCandidate(createMockAgent("agent-b", "Agent B", 0.8, 50)),
        createMockCandidate(createMockAgent("agent-c", "Agent C", 0.8, 50)),
      ];

      // Should select one of them without error
      const result = await bandit.select(
        samePerformanceCandidates,
        "code-editing"
      );
      expect(["agent-a", "agent-b", "agent-c"]).toContain(result.id);
    });

    it("should handle very low utilization agents", async () => {
      const lowUtilCandidates = mockCandidates.map((candidate) =>
        createMockCandidate({
          ...candidate.agent,
          currentLoad: {
            ...candidate.agent.currentLoad,
            utilizationPercent: 10,
          },
        })
      );

      const result = await bandit.select(lowUtilCandidates, "code-editing");
      expect(mockAgents.map((a) => a.id)).toContain(result.id);
    });
  });
});
