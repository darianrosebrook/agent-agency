/**
 * Unit tests for Debate State Machine
 *
 * @author @darianrosebrook
 */

import { DebateStateMachine } from "@/reasoning/DebateStateMachine";
import {
  DebateSession,
  DebateState,
  ReasoningEngineError,
} from "@/types/reasoning";

describe("DebateStateMachine", () => {
  afterEach(() => {
    jest.clearAllMocks();
  });

  afterAll(() => {
    // Final cleanup
    jest.clearAllMocks();
  });
  describe("canTransition", () => {
    it("should allow valid state transitions", () => {
      const session = createTestSession(DebateState.INITIALIZED);

      expect(
        DebateStateMachine.canTransition(session, DebateState.AGENTS_ASSIGNED)
      ).toBe(true);
    });

    it("should reject invalid state transitions", () => {
      const session = createTestSession(DebateState.INITIALIZED);

      expect(
        DebateStateMachine.canTransition(session, DebateState.CONSENSUS_REACHED)
      ).toBe(false);
    });

    it("should allow transition to failed state from any non-terminal state", () => {
      const states = [
        DebateState.INITIALIZED,
        DebateState.AGENTS_ASSIGNED,
        DebateState.ARGUMENTS_PRESENTED,
        DebateState.EVIDENCE_AGGREGATED,
        DebateState.DELIBERATION,
        DebateState.DEADLOCKED,
      ];

      states.forEach((state) => {
        const session = createTestSession(state);
        expect(
          DebateStateMachine.canTransition(session, DebateState.FAILED)
        ).toBe(true);
      });
    });

    it("should reject transitions from terminal states", () => {
      const completedSession = createTestSession(DebateState.COMPLETED);
      expect(
        DebateStateMachine.canTransition(
          completedSession,
          DebateState.DELIBERATION
        )
      ).toBe(false);

      const failedSession = createTestSession(DebateState.FAILED);
      expect(
        DebateStateMachine.canTransition(failedSession, DebateState.INITIALIZED)
      ).toBe(false);
    });
  });

  describe("transition", () => {
    it("should successfully transition to valid next state", () => {
      const session = createTestSession(DebateState.INITIALIZED);

      const transitioned = DebateStateMachine.transition(
        session,
        DebateState.AGENTS_ASSIGNED
      );

      expect(transitioned.state).toBe(DebateState.AGENTS_ASSIGNED);
      expect(transitioned.reasoningChain).toContain(
        "Transitioned from initialized to agents_assigned"
      );
    });

    it("should throw error for invalid transition", () => {
      const session = createTestSession(DebateState.INITIALIZED);

      expect(() =>
        DebateStateMachine.transition(session, DebateState.CONSENSUS_REACHED)
      ).toThrow(ReasoningEngineError);
      expect(() =>
        DebateStateMachine.transition(session, DebateState.CONSENSUS_REACHED)
      ).toThrow(/Invalid state transition/);
    });

    it("should set end time when reaching terminal state", () => {
      const session = createTestSession(DebateState.CONSENSUS_REACHED);

      const completed = DebateStateMachine.transition(
        session,
        DebateState.COMPLETED
      );

      expect(completed.endTime).toBeDefined();
      expect(completed.endTime).toBeInstanceOf(Date);
    });

    it("should preserve existing session data during transition", () => {
      const session = createTestSession(DebateState.INITIALIZED);
      session.arguments = [createTestArgument()];
      session.participants = [createTestParticipant()];

      const transitioned = DebateStateMachine.transition(
        session,
        DebateState.AGENTS_ASSIGNED
      );

      expect(transitioned.arguments).toEqual(session.arguments);
      expect(transitioned.participants).toEqual(session.participants);
    });
  });

  describe("isTerminalState", () => {
    it("should identify completed as terminal state", () => {
      expect(DebateStateMachine.isTerminalState(DebateState.COMPLETED)).toBe(
        true
      );
    });

    it("should identify failed as terminal state", () => {
      expect(DebateStateMachine.isTerminalState(DebateState.FAILED)).toBe(true);
    });

    it("should not identify non-terminal states as terminal", () => {
      const nonTerminalStates = [
        DebateState.INITIALIZED,
        DebateState.AGENTS_ASSIGNED,
        DebateState.ARGUMENTS_PRESENTED,
        DebateState.EVIDENCE_AGGREGATED,
        DebateState.DELIBERATION,
        DebateState.CONSENSUS_FORMING,
        DebateState.CONSENSUS_REACHED,
        DebateState.DEADLOCKED,
        DebateState.RESOLUTION_IN_PROGRESS,
      ];

      nonTerminalStates.forEach((state) => {
        expect(DebateStateMachine.isTerminalState(state)).toBe(false);
      });
    });
  });

  describe("getValidNextStates", () => {
    it("should return all valid next states for initialized", () => {
      const nextStates = DebateStateMachine.getValidNextStates(
        DebateState.INITIALIZED
      );

      expect(nextStates).toContain(DebateState.AGENTS_ASSIGNED);
      expect(nextStates).toContain(DebateState.FAILED);
      expect(nextStates.length).toBeGreaterThan(0);
    });

    it("should return empty array for terminal states", () => {
      const completedNext = DebateStateMachine.getValidNextStates(
        DebateState.COMPLETED
      );
      const failedNext = DebateStateMachine.getValidNextStates(
        DebateState.FAILED
      );

      expect(completedNext).toEqual([]);
      expect(failedNext).toEqual([]);
    });

    it("should include both normal flow and failure transitions", () => {
      const nextStates = DebateStateMachine.getValidNextStates(
        DebateState.ARGUMENTS_PRESENTED
      );

      expect(nextStates).toContain(DebateState.EVIDENCE_AGGREGATED);
      expect(nextStates).toContain(DebateState.FAILED);
    });
  });

  describe("validateInvariants", () => {
    it("should validate session with at least 2 participants", () => {
      const session = createTestSession(DebateState.INITIALIZED);
      session.participants = [createTestParticipant(), createTestParticipant()];

      expect(() =>
        DebateStateMachine.validateInvariants(session)
      ).not.toThrow();
    });

    it("should reject session with less than 2 participants", () => {
      const session = createTestSession(DebateState.INITIALIZED);
      session.participants = [createTestParticipant()];

      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        ReasoningEngineError
      );
      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        /at least 2 participants/
      );
    });

    it("should reject consensus result in non-consensus state", () => {
      const session = createTestSession(DebateState.DELIBERATION);
      session.participants = [createTestParticipant(), createTestParticipant()];
      session.consensusResult = createTestConsensusResult();

      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        ReasoningEngineError
      );
      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        /not in consensus state/
      );
    });

    it("should allow consensus result in consensus reached state", () => {
      const session = createTestSession(DebateState.CONSENSUS_REACHED);
      session.participants = [createTestParticipant(), createTestParticipant()];
      session.consensusResult = createTestConsensusResult();

      expect(() =>
        DebateStateMachine.validateInvariants(session)
      ).not.toThrow();
    });

    it("should reject end time in non-terminal state", () => {
      const session = createTestSession(DebateState.DELIBERATION);
      session.participants = [createTestParticipant(), createTestParticipant()];
      session.endTime = new Date();

      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        ReasoningEngineError
      );
      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        /not terminal/
      );
    });

    it("should require consensus result for completed debates", () => {
      const session = createTestSession(DebateState.COMPLETED);
      session.participants = [createTestParticipant(), createTestParticipant()];
      // Missing consensusResult

      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        ReasoningEngineError
      );
      expect(() => DebateStateMachine.validateInvariants(session)).toThrow(
        /missing consensus/
      );
    });
  });

  describe("initializeSession", () => {
    it("should create new session with initialized state", () => {
      const config = createTestConfig();
      const session = DebateStateMachine.initializeSession("debate-1", config);

      expect(session.id).toBe("debate-1");
      expect(session.state).toBe(DebateState.INITIALIZED);
      expect(session.config).toEqual(config);
    });

    it("should initialize with empty arrays", () => {
      const config = createTestConfig();
      const session = DebateStateMachine.initializeSession("debate-1", config);

      expect(session.participants).toEqual([]);
      expect(session.arguments).toEqual([]);
    });

    it("should set start time to current time", () => {
      const before = Date.now();
      const config = createTestConfig();
      const session = DebateStateMachine.initializeSession("debate-1", config);
      const after = Date.now();

      expect(session.startTime.getTime()).toBeGreaterThanOrEqual(before);
      expect(session.startTime.getTime()).toBeLessThanOrEqual(after);
    });

    it("should initialize reasoning chain", () => {
      const config = createTestConfig();
      const session = DebateStateMachine.initializeSession("debate-1", config);

      expect(session.reasoningChain).toHaveLength(1);
      expect(session.reasoningChain[0]).toContain("initialized");
    });
  });

  describe("isExpired", () => {
    it("should return false for debate within time limit", () => {
      const session = createTestSession(DebateState.DELIBERATION);
      session.config.maxDuration = 60000; // 1 minute
      session.startTime = new Date(); // Just started

      expect(DebateStateMachine.isExpired(session)).toBe(false);
    });

    it("should return true for debate exceeding time limit", () => {
      const session = createTestSession(DebateState.DELIBERATION);
      session.config.maxDuration = 1000; // 1 second
      session.startTime = new Date(Date.now() - 2000); // Started 2 seconds ago

      expect(DebateStateMachine.isExpired(session)).toBe(true);
    });

    it("should handle edge case at exact time limit", () => {
      const session = createTestSession(DebateState.DELIBERATION);
      session.config.maxDuration = 1000;
      session.startTime = new Date(Date.now() - 1001); // Slightly past to avoid timing precision issues

      // Should be expired at or past the limit
      expect(DebateStateMachine.isExpired(session)).toBe(true);
    });
  });

  describe("getStateSummary", () => {
    it("should generate summary with debate details", () => {
      const session = createTestSession(DebateState.DELIBERATION);
      session.participants = [createTestParticipant(), createTestParticipant()];
      session.arguments = [createTestArgument()];

      const summary = DebateStateMachine.getStateSummary(session);

      expect(summary).toContain(session.id);
      expect(summary).toContain("deliberation");
      expect(summary).toContain("2 participants");
      expect(summary).toContain("1 arguments");
    });

    it("should include elapsed time in summary", () => {
      const session = createTestSession(DebateState.DELIBERATION);
      session.participants = [createTestParticipant(), createTestParticipant()];
      session.startTime = new Date(Date.now() - 5000); // 5 seconds ago

      const summary = DebateStateMachine.getStateSummary(session);

      expect(summary).toMatch(/elapsed \d+s/);
    });
  });
});

// Test Helper Functions

function createTestSession(state: DebateState): DebateSession {
  return {
    id: "test-debate-1",
    config: createTestConfig(),
    state,
    participants: [],
    arguments: [],
    startTime: new Date(),
    reasoningChain: ["Test session initialized"],
  };
}

function createTestConfig(): DebateSession["config"] {
  return {
    id: "config-1",
    topic: "Test debate topic",
    maxParticipants: 5,
    maxDuration: 300000,
    consensusAlgorithm: "simple_majority" as any,
    deadlockStrategy: "mediator_decision" as any,
    requiresUnanimous: false,
    allowAppeals: true,
  };
}

function createTestParticipant(): any {
  return {
    agentId: `agent-${Math.random().toString(36).substr(2, 9)}`,
    role: "proponent" as any,
    weight: 1,
    argumentsPresented: [],
    votesCast: [],
  };
}

function createTestArgument(): any {
  return {
    id: `arg-${Math.random().toString(36).substr(2, 9)}`,
    agentId: "agent-1",
    claim: "Test claim",
    evidence: [],
    reasoning: "Test reasoning",
    timestamp: new Date(),
    credibilityScore: 0.8,
  };
}

function createTestConsensusResult(): any {
  return {
    reached: true,
    algorithm: "simple_majority" as any,
    outcome: "accepted" as any,
    confidence: 0.85,
    votingBreakdown: { for: 3, against: 1, abstain: 0 },
    reasoning: "Consensus reached with simple majority",
    timestamp: new Date(),
  };
}
