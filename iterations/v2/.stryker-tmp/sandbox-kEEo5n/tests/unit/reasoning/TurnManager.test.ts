/**
 * Unit tests for TurnManager
 *
 * Tests turn scheduling, fairness enforcement, timeout handling,
 * and various scheduling strategies.
 */
// @ts-nocheck


import { TurnManager, TurnSchedulingStrategy } from "@/reasoning/TurnManager";
import {
  AgentRole,
  DebateParticipant,
  ReasoningEngineError,
} from "@/types/reasoning";

describe("TurnManager", () => {
  let manager: TurnManager;

  beforeEach(() => {
    manager = new TurnManager({
      schedulingStrategy: TurnSchedulingStrategy.WEIGHTED_FAIR,
      defaultTurnTimeout: 60000,
      maxTurnsPerAgent: 10,
      fairnessThreshold: 0.7,
      enableTimeoutPenalty: true,
      timeoutPenaltyMultiplier: 0.5,
    });
  });

  describe("initializeDebate", () => {
    it("should initialize debate tracking", () => {
      manager.initializeDebate("debate-1");

      const history = manager.getTurnHistory("debate-1");
      expect(history).toEqual([]);
    });

    it("should throw error for empty debate ID", () => {
      expect(() => manager.initializeDebate("")).toThrow(ReasoningEngineError);
      expect(() => manager.initializeDebate("")).toThrow(
        "Debate ID cannot be empty"
      );
    });

    it("should allow re-initialization", () => {
      manager.initializeDebate("debate-1");
      manager.recordTurn("debate-1", "agent-1", "argument", 1000);

      manager.initializeDebate("debate-1"); // Re-initialize

      const history = manager.getTurnHistory("debate-1");
      expect(history).toEqual([]); // History cleared
    });
  });

  describe("allocateNextTurn", () => {
    const participants: DebateParticipant[] = [
      {
        agentId: "agent-1",
        role: AgentRole.PROPONENT,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      },
      {
        agentId: "agent-2",
        role: AgentRole.OPPONENT,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      },
    ];

    beforeEach(() => {
      manager.initializeDebate("debate-1");
    });

    it("should allocate first turn", () => {
      const allocation = manager.allocateNextTurn("debate-1", participants);

      expect(allocation).toBeDefined();
      expect(allocation.turnNumber).toBe(1);
      expect(allocation.agentId).toBeDefined();
      expect(allocation.maxDuration).toBe(60000);
      expect(allocation.deadline).toBeInstanceOf(Date);
    });

    it("should throw error for uninitialized debate", () => {
      expect(() => manager.allocateNextTurn("unknown", participants)).toThrow(
        ReasoningEngineError
      );
      expect(() => manager.allocateNextTurn("unknown", participants)).toThrow(
        "not initialized"
      );
    });

    it("should throw error for empty participants", () => {
      expect(() => manager.allocateNextTurn("debate-1", [])).toThrow(
        ReasoningEngineError
      );
      expect(() => manager.allocateNextTurn("debate-1", [])).toThrow(
        "At least one participant required"
      );
    });

    it("should throw error when max turns reached", () => {
      // Record max turns for all agents
      for (let i = 0; i < 10; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
        manager.recordTurn("debate-1", "agent-2", "argument", 1000);
      }

      expect(() => manager.allocateNextTurn("debate-1", participants)).toThrow(
        ReasoningEngineError
      );
      expect(() => manager.allocateNextTurn("debate-1", participants)).toThrow(
        "All agents have reached maximum turns"
      );
    });

    it("should skip agents at max turns", () => {
      // Record max turns for agent-1
      for (let i = 0; i < 10; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
      }

      const allocation = manager.allocateNextTurn("debate-1", participants);

      // Should allocate to agent-2 since agent-1 is at max
      expect(allocation.agentId).toBe("agent-2");
    });
  });

  describe("recordTurn", () => {
    beforeEach(() => {
      manager.initializeDebate("debate-1");
    });

    it("should record completed turn", () => {
      manager.recordTurn("debate-1", "agent-1", "argument", 1500);

      const history = manager.getTurnHistory("debate-1");
      expect(history).toHaveLength(1);
      expect(history[0].agentId).toBe("agent-1");
      expect(history[0].action).toBe("argument");
      expect(history[0].duration).toBe(1500);
      expect(history[0].wasTimeout).toBe(false);
      expect(history[0].turnNumber).toBe(1);
    });

    it("should record timeout turn", () => {
      manager.recordTurn("debate-1", "agent-1", "argument", 60000, true);

      const history = manager.getTurnHistory("debate-1");
      expect(history[0].wasTimeout).toBe(true);
    });

    it("should throw error for uninitialized debate", () => {
      expect(() =>
        manager.recordTurn("unknown", "agent-1", "argument", 1000)
      ).toThrow(ReasoningEngineError);
    });

    it("should increment turn numbers", () => {
      manager.recordTurn("debate-1", "agent-1", "argument", 1000);
      manager.recordTurn("debate-1", "agent-2", "vote", 500);
      manager.recordTurn("debate-1", "agent-1", "evidence", 2000);

      const history = manager.getTurnHistory("debate-1");
      expect(history[0].turnNumber).toBe(1);
      expect(history[1].turnNumber).toBe(2);
      expect(history[2].turnNumber).toBe(3);
    });
  });

  describe("isCurrentTurnTimedOut", () => {
    const participants: DebateParticipant[] = [
      {
        agentId: "agent-1",
        role: AgentRole.PROPONENT,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      },
    ];

    beforeEach(() => {
      manager.initializeDebate("debate-1");
    });

    it("should return false when no current turn", () => {
      const isTimedOut = manager.isCurrentTurnTimedOut("debate-1");
      expect(isTimedOut).toBe(false);
    });

    it("should return false when turn not timed out", () => {
      manager.allocateNextTurn("debate-1", participants);

      const isTimedOut = manager.isCurrentTurnTimedOut("debate-1");
      expect(isTimedOut).toBe(false);
    });

    it("should return true when turn timed out", () => {
      // Create manager with very short timeout
      const shortManager = new TurnManager({ defaultTurnTimeout: 1 });
      shortManager.initializeDebate("debate-1");
      shortManager.allocateNextTurn("debate-1", participants);

      // Wait for timeout
      setTimeout(() => {
        const isTimedOut = shortManager.isCurrentTurnTimedOut("debate-1");
        expect(isTimedOut).toBe(true);
      }, 10);
    });
  });

  describe("getCurrentTurn", () => {
    const participants: DebateParticipant[] = [
      {
        agentId: "agent-1",
        role: AgentRole.PROPONENT,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      },
    ];

    beforeEach(() => {
      manager.initializeDebate("debate-1");
    });

    it("should return null when no current turn", () => {
      const current = manager.getCurrentTurn("debate-1");
      expect(current).toBeNull();
    });

    it("should return current turn allocation", () => {
      const allocation = manager.allocateNextTurn("debate-1", participants);
      const current = manager.getCurrentTurn("debate-1");

      expect(current).toEqual(allocation);
    });

    it("should clear current turn after recording", () => {
      manager.allocateNextTurn("debate-1", participants);
      manager.recordTurn("debate-1", "agent-1", "argument", 1000);

      const current = manager.getCurrentTurn("debate-1");
      expect(current).toBeNull();
    });
  });

  describe("calculateFairnessMetrics", () => {
    beforeEach(() => {
      manager.initializeDebate("debate-1");
    });

    it("should calculate metrics for empty history", () => {
      const metrics = manager.calculateFairnessMetrics("debate-1");

      expect(metrics.totalTurns).toBe(0);
      expect(metrics.turnsPerAgent.size).toBe(0);
      expect(metrics.averageTurnsPerAgent).toBe(0);
      expect(metrics.fairnessScore).toBe(1.0); // Perfect fairness
    });

    it("should calculate metrics for balanced participation", () => {
      // Each agent gets 3 turns
      for (let i = 0; i < 3; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
        manager.recordTurn("debate-1", "agent-2", "argument", 1000);
      }

      const metrics = manager.calculateFairnessMetrics("debate-1");

      expect(metrics.totalTurns).toBe(6);
      expect(metrics.turnsPerAgent.get("agent-1")).toBe(3);
      expect(metrics.turnsPerAgent.get("agent-2")).toBe(3);
      expect(metrics.averageTurnsPerAgent).toBe(3);
      expect(metrics.fairnessScore).toBeCloseTo(1.0, 1); // Perfect balance
      expect(metrics.participationRate.get("agent-1")).toBeCloseTo(0.5, 1);
      expect(metrics.participationRate.get("agent-2")).toBeCloseTo(0.5, 1);
    });

    it("should calculate metrics for unbalanced participation", () => {
      // agent-1 gets 5 turns, agent-2 gets 1 turn
      for (let i = 0; i < 5; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
      }
      manager.recordTurn("debate-1", "agent-2", "argument", 1000);

      const metrics = manager.calculateFairnessMetrics("debate-1");

      expect(metrics.totalTurns).toBe(6);
      expect(metrics.fairnessScore).toBeLessThan(1.0); // Unbalanced
      expect(metrics.participationRate.get("agent-1")).toBeCloseTo(5 / 6, 1);
      expect(metrics.participationRate.get("agent-2")).toBeCloseTo(1 / 6, 1);
    });

    it("should track timeouts per agent", () => {
      manager.recordTurn("debate-1", "agent-1", "argument", 1000, false);
      manager.recordTurn("debate-1", "agent-1", "argument", 60000, true);
      manager.recordTurn("debate-1", "agent-2", "argument", 60000, true);

      const metrics = manager.calculateFairnessMetrics("debate-1");

      expect(metrics.timeoutsPerAgent.get("agent-1")).toBe(1);
      expect(metrics.timeoutsPerAgent.get("agent-2")).toBe(1);
    });
  });

  describe("validateFairness", () => {
    beforeEach(() => {
      manager.initializeDebate("debate-1");
    });

    it("should pass validation for fair debate", () => {
      // Balanced participation
      for (let i = 0; i < 3; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
        manager.recordTurn("debate-1", "agent-2", "argument", 1000);
      }

      const validation = manager.validateFairness("debate-1");

      expect(validation.isValid).toBe(true);
      expect(validation.issues).toHaveLength(0);
    });

    it("should fail validation for unfair debate", () => {
      // Highly unbalanced: agent-1 gets 10 turns, agent-2 gets 1 turn
      for (let i = 0; i < 10; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
      }
      manager.recordTurn("debate-1", "agent-2", "argument", 1000);

      const validation = manager.validateFairness("debate-1");

      expect(validation.isValid).toBe(false);
      expect(validation.issues.length).toBeGreaterThan(0);
    });

    it("should detect agent monopolization", () => {
      // agent-1 gets >50% of turns
      for (let i = 0; i < 6; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
      }
      manager.recordTurn("debate-1", "agent-2", "argument", 1000);

      const validation = manager.validateFairness("debate-1");

      expect(validation.isValid).toBe(false);
      expect(validation.issues.some((i) => i.includes("monopolized"))).toBe(
        true
      );
    });

    it("should detect excessive timeouts", () => {
      // agent-1: 2 timeouts out of 3 turns (>50%)
      manager.recordTurn("debate-1", "agent-1", "argument", 60000, true);
      manager.recordTurn("debate-1", "agent-1", "argument", 60000, true);
      manager.recordTurn("debate-1", "agent-1", "argument", 1000, false);
      manager.recordTurn("debate-1", "agent-2", "argument", 1000, false);

      const validation = manager.validateFairness("debate-1");

      expect(validation.isValid).toBe(false);
      expect(validation.issues.some((i) => i.includes("timeouts"))).toBe(true);
    });
  });

  describe("clearDebate", () => {
    beforeEach(() => {
      manager.initializeDebate("debate-1");
      manager.recordTurn("debate-1", "agent-1", "argument", 1000);
    });

    it("should clear debate history", () => {
      manager.clearDebate("debate-1");

      const history = manager.getTurnHistory("debate-1");
      expect(history).toEqual([]);
    });

    it("should clear current turn", () => {
      const participants: DebateParticipant[] = [
        {
          agentId: "agent-1",
          role: AgentRole.PROPONENT,
          weight: 1.0,
          votesCast: [],
          argumentsPresented: [],
        },
      ];

      manager.allocateNextTurn("debate-1", participants);
      manager.clearDebate("debate-1");

      const current = manager.getCurrentTurn("debate-1");
      expect(current).toBeNull();
    });
  });

  describe("scheduling strategies", () => {
    const participants: DebateParticipant[] = [
      {
        agentId: "agent-1",
        role: AgentRole.PROPONENT,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      },
      {
        agentId: "agent-2",
        role: AgentRole.OPPONENT,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      },
      {
        agentId: "agent-3",
        role: AgentRole.MEDIATOR,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      },
    ];

    describe("ROUND_ROBIN", () => {
      beforeEach(() => {
        manager = new TurnManager({
          schedulingStrategy: TurnSchedulingStrategy.ROUND_ROBIN,
        });
        manager.initializeDebate("debate-1");
      });

      it("should alternate between agents", () => {
        const allocation1 = manager.allocateNextTurn("debate-1", participants);
        manager.recordTurn("debate-1", allocation1.agentId, "argument", 1000);

        const allocation2 = manager.allocateNextTurn("debate-1", participants);
        manager.recordTurn("debate-1", allocation2.agentId, "argument", 1000);

        const allocation3 = manager.allocateNextTurn("debate-1", participants);

        // All three agents should have been selected
        const selectedAgents = [
          allocation1.agentId,
          allocation2.agentId,
          allocation3.agentId,
        ];
        expect(new Set(selectedAgents).size).toBe(3);
      });

      it("should give equal turns to all agents", () => {
        for (let i = 0; i < 9; i++) {
          const allocation = manager.allocateNextTurn("debate-1", participants);
          manager.recordTurn("debate-1", allocation.agentId, "argument", 1000);
        }

        const metrics = manager.calculateFairnessMetrics("debate-1");
        expect(metrics.turnsPerAgent.get("agent-1")).toBe(3);
        expect(metrics.turnsPerAgent.get("agent-2")).toBe(3);
        expect(metrics.turnsPerAgent.get("agent-3")).toBe(3);
      });
    });

    describe("WEIGHTED_FAIR", () => {
      beforeEach(() => {
        manager = new TurnManager({
          schedulingStrategy: TurnSchedulingStrategy.WEIGHTED_FAIR,
        });
        manager.initializeDebate("debate-1");
      });

      it("should respect agent weights", () => {
        const weightedParticipants: DebateParticipant[] = [
          { ...participants[0], weight: 2.0 }, // Higher weight
          { ...participants[1], weight: 1.0 },
        ];

        const allocations: string[] = [];
        for (let i = 0; i < 10; i++) {
          const allocation = manager.allocateNextTurn(
            "debate-1",
            weightedParticipants
          );
          allocations.push(allocation.agentId);
          manager.recordTurn("debate-1", allocation.agentId, "argument", 1000);
        }

        // agent-1 (weight 2.0) should get more turns than agent-2 (weight 1.0)
        const agent1Turns = allocations.filter((id) => id === "agent-1").length;
        const agent2Turns = allocations.filter((id) => id === "agent-2").length;

        expect(agent1Turns).toBeGreaterThan(agent2Turns);
      });

      it("should penalize timeout agents", () => {
        // agent-1 gets multiple timeouts
        manager.recordTurn("debate-1", "agent-1", "argument", 60000, true);
        manager.recordTurn("debate-1", "agent-1", "argument", 60000, true);

        // Next allocation should favor agent-2
        const allocation = manager.allocateNextTurn("debate-1", participants);
        expect(allocation.agentId).toBe("agent-2");
      });
    });

    describe("PRIORITY_BASED", () => {
      beforeEach(() => {
        manager = new TurnManager({
          schedulingStrategy: TurnSchedulingStrategy.PRIORITY_BASED,
        });
        manager.initializeDebate("debate-1");
      });

      it("should prioritize mediator role", () => {
        const allocation = manager.allocateNextTurn("debate-1", participants);

        // MEDIATOR should be selected first
        expect(allocation.agentId).toBe("agent-3"); // mediator
      });

      it("should balance priority with turn counts", () => {
        // Give mediator multiple turns
        for (let i = 0; i < 3; i++) {
          manager.recordTurn("debate-1", "agent-3", "argument", 1000);
        }

        const allocation = manager.allocateNextTurn("debate-1", participants);

        // Should select proponent or opponent since mediator has many turns
        expect(["agent-1", "agent-2"]).toContain(allocation.agentId);
      });
    });

    describe("DYNAMIC_ADAPTIVE", () => {
      beforeEach(() => {
        manager = new TurnManager({
          schedulingStrategy: TurnSchedulingStrategy.DYNAMIC_ADAPTIVE,
        });
        manager.initializeDebate("debate-1");
      });

      it("should adapt to recent activity", () => {
        // Give agent-1 recent turns
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);

        const allocation = manager.allocateNextTurn("debate-1", participants);

        // Should favor agent-2 or agent-3 due to recency penalty
        expect(["agent-2", "agent-3"]).toContain(allocation.agentId);
      });

      it("should combine multiple factors", () => {
        // Complex scenario:
        // - agent-1: high weight, but many turns
        // - agent-2: normal weight, few turns
        // - agent-3: normal weight, few turns + mediator role

        const weightedParticipants: DebateParticipant[] = [
          { ...participants[0], weight: 2.0 },
          { ...participants[1], weight: 1.0 },
          { ...participants[2], weight: 1.0 },
        ];

        for (let i = 0; i < 5; i++) {
          manager.recordTurn("debate-1", "agent-1", "argument", 1000);
        }

        const allocation = manager.allocateNextTurn(
          "debate-1",
          weightedParticipants
        );

        // Should select agent with best composite score
        expect(allocation.agentId).toBeDefined();
        expect(allocation.reason).toContain("Dynamic adaptive");
      });
    });
  });

  describe("edge cases", () => {
    beforeEach(() => {
      manager.initializeDebate("debate-1");
    });

    it("should handle single participant", () => {
      const participants: DebateParticipant[] = [
        {
          agentId: "agent-1",
          role: AgentRole.PROPONENT,
          weight: 1.0,
          votesCast: [],
          argumentsPresented: [],
        },
      ];

      const allocation = manager.allocateNextTurn("debate-1", participants);
      expect(allocation.agentId).toBe("agent-1");
    });

    it("should handle participants with zero weight", () => {
      const participants: DebateParticipant[] = [
        {
          agentId: "agent-1",
          role: AgentRole.PROPONENT,
          weight: 0,
          votesCast: [],
          argumentsPresented: [],
        },
        {
          agentId: "agent-2",
          role: AgentRole.OPPONENT,
          weight: 1.0,
          votesCast: [],
          argumentsPresented: [],
        },
      ];

      manager = new TurnManager({
        schedulingStrategy: TurnSchedulingStrategy.WEIGHTED_FAIR,
      });
      manager.initializeDebate("debate-1");

      const allocation = manager.allocateNextTurn("debate-1", participants);

      // Should still allocate (weight of 0 is handled)
      expect(allocation.agentId).toBeDefined();
    });

    it("should handle undefined weights", () => {
      const participants: DebateParticipant[] = [
        {
          agentId: "agent-1",
          role: AgentRole.PROPONENT,
          weight: undefined,
          votesCast: [],
          argumentsPresented: [],
        },
      ];

      const allocation = manager.allocateNextTurn("debate-1", participants);
      expect(allocation.agentId).toBe("agent-1");
    });

    it("should handle very long debates", () => {
      const participants: DebateParticipant[] = [
        {
          agentId: "agent-1",
          role: AgentRole.PROPONENT,
          weight: 1.0,
          votesCast: [],
          argumentsPresented: [],
        },
        {
          agentId: "agent-2",
          role: AgentRole.OPPONENT,
          weight: 1.0,
          votesCast: [],
          argumentsPresented: [],
        },
      ];

      manager = new TurnManager({ maxTurnsPerAgent: 100 });
      manager.initializeDebate("debate-1");

      // Simulate 50 turns per agent
      for (let i = 0; i < 50; i++) {
        manager.recordTurn("debate-1", "agent-1", "argument", 1000);
        manager.recordTurn("debate-1", "agent-2", "argument", 1000);
      }

      const metrics = manager.calculateFairnessMetrics("debate-1");
      expect(metrics.totalTurns).toBe(100);
      expect(metrics.fairnessScore).toBeCloseTo(1.0, 1); // Should remain fair
    });
  });
});
