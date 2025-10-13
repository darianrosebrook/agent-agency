/**
 * Unit tests for DeadlockResolver
 *
 * Tests deadlock detection, resolution strategies, and pattern tracking.
 */

import { DeadlockResolver } from "@/reasoning/DeadlockResolver";
import {
  AgentRole,
  DeadlockResolutionStrategy,
  DebateParticipant,
  DebateSession,
  DebateState,
  DebateVote,
  ReasoningEngineError,
} from "@/types/reasoning";

describe("DeadlockResolver", () => {
  let resolver: DeadlockResolver;

  beforeEach(() => {
    resolver = new DeadlockResolver({
      deadlockDetectionRounds: 3,
      minVotesForDeadlock: 2,
      votingPatternThreshold: 0.8,
      enablePatternTracking: true,
      defaultResolutionStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
    });
  });

  describe("detectDeadlock", () => {
    const createSession = (): DebateSession => ({
      id: "debate-1",
      config: {
        id: "debate-1",
        topic: "Test topic",
        maxParticipants: 10,
        maxDuration: 60000,
        consensusAlgorithm: "simple_majority" as any,
        deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
        requiresUnanimous: false,
        allowAppeals: true,
      },
      participants: [
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
      ],
      state: DebateState.CONSENSUS_FORMING,
      startTime: new Date(),
      arguments: [],
      reasoningChain: [],
    });

    it("should return not deadlocked for insufficient data", () => {
      const session = createSession();
      const votes: DebateVote[] = [
        {
          agentId: "agent-1",
          position: "for",
          reasoning: "Support",
          confidence: 0.8,
          timestamp: new Date(),
        },
      ];

      const detection = resolver.detectDeadlock(session, votes);

      expect(detection.isDeadlocked).toBe(false);
      expect(detection.votingPattern).toBe("insufficient_data");
    });

    it("should return not deadlocked for progressing debate", () => {
      const session = createSession();
      const votes: DebateVote[] = [
        {
          agentId: "agent-1",
          position: "for",
          reasoning: "Support",
          confidence: 0.8,
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "for",
          reasoning: "Agree",
          confidence: 0.7,
          timestamp: new Date(),
        },
      ];

      const detection = resolver.detectDeadlock(session, votes);

      expect(detection.isDeadlocked).toBe(false);
      expect(detection.votingPattern).toBe("progressing");
    });

    it("should detect deadlock with repeating patterns", () => {
      const session = createSession();
      const votes: DebateVote[] = [
        // Round 1
        {
          agentId: "agent-1",
          position: "for",
          reasoning: "Support",
          confidence: 0.8,
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          reasoning: "Oppose",
          confidence: 0.8,
          timestamp: new Date(),
        },
        // Round 2
        {
          agentId: "agent-1",
          position: "for",
          reasoning: "Support",
          confidence: 0.8,
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          reasoning: "Oppose",
          confidence: 0.8,
          timestamp: new Date(),
        },
        // Round 3
        {
          agentId: "agent-1",
          position: "for",
          reasoning: "Support",
          confidence: 0.8,
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          reasoning: "Oppose",
          confidence: 0.8,
          timestamp: new Date(),
        },
      ];

      const detection = resolver.detectDeadlock(session, votes);

      expect(detection.isDeadlocked).toBe(true);
      expect(detection.rounds).toBe(3);
      expect(detection.confidence).toBeCloseTo(0.9, 1);
      expect(detection.participantsInvolved).toContain("agent-1");
      expect(detection.participantsInvolved).toContain("agent-2");
    });

    it("should suggest appropriate resolution strategy", () => {
      const session = createSession();
      session.participants.push({
        agentId: "agent-3",
        role: AgentRole.MEDIATOR,
        weight: 1.0,
        votesCast: [],
        argumentsPresented: [],
      });

      const votes: DebateVote[] = [
        // Create deadlock pattern
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
      ];

      const detection = resolver.detectDeadlock(session, votes);

      expect(detection.suggestedResolution).toBe(
        DeadlockResolutionStrategy.MEDIATOR_DECISION
      );
    });

    it("should track deadlock patterns when enabled", () => {
      const session = createSession();
      const votes: DebateVote[] = [
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
      ];

      resolver.detectDeadlock(session, votes);

      const patterns = resolver.getDeadlockPatterns();
      expect(patterns.length).toBeGreaterThan(0);
    });
  });

  describe("resolveDeadlock", () => {
    const createSession = (withMediator: boolean = false): DebateSession => {
      const participants: DebateParticipant[] = [
        {
          agentId: "agent-1",
          role: AgentRole.PROPONENT,
          weight: 1.0,
          votesCast: [
            {
              agentId: "agent-1",
              position: "for",
              reasoning: "Support",
              confidence: 0.8,
              timestamp: new Date(),
            },
          ],
          argumentsPresented: [],
        },
        {
          agentId: "agent-2",
          role: AgentRole.OPPONENT,
          weight: 1.0,
          votesCast: [
            {
              agentId: "agent-2",
              position: "against",
              reasoning: "Oppose",
              confidence: 0.8,
              timestamp: new Date(),
            },
          ],
          argumentsPresented: [],
        },
      ];

      if (withMediator) {
        participants.push({
          agentId: "mediator",
          role: AgentRole.MEDIATOR,
          weight: 1.0,
          votesCast: [
            {
              agentId: "mediator",
              position: "for",
              reasoning: "Mediator approves",
              confidence: 0.9,
              timestamp: new Date(),
            },
          ],
          argumentsPresented: [],
        });
      }

      return {
        id: "debate-1",
        config: {
          id: "debate-1",
          topic: "Test topic",
          maxParticipants: 10,
          maxDuration: 60000,
          consensusAlgorithm: "simple_majority" as any,
          deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
          requiresUnanimous: false,
          allowAppeals: true,
        },
        participants,
        state: DebateState.DEADLOCKED,
        startTime: new Date(),
        arguments: [],
        reasoningChain: [],
      };
    };

    const createDetection = (): any => ({
      isDeadlocked: true,
      rounds: 3,
      votingPattern: "1:1:0,1:1:0,1:1:0",
      participantsInvolved: ["agent-1", "agent-2"],
      confidence: 0.9,
      suggestedResolution: DeadlockResolutionStrategy.MEDIATOR_DECISION,
    });

    describe("MEDIATOR_DECISION", () => {
      it("should resolve via mediator decision", () => {
        const session = createSession(true);
        const detection = createDetection();

        const resolution = resolver.resolveDeadlock(
          session,
          detection,
          DeadlockResolutionStrategy.MEDIATOR_DECISION
        );

        expect(resolution.strategy).toBe(
          DeadlockResolutionStrategy.MEDIATOR_DECISION
        );
        expect(resolution.decision).toBe("for");
        expect(resolution.mediatorOverride).toBe(true);
        expect(resolution.confidence).toBeCloseTo(0.9, 1);
      });

      it("should throw error when no mediator present", () => {
        const session = createSession(false);
        const detection = createDetection();

        expect(() =>
          resolver.resolveDeadlock(
            session,
            detection,
            DeadlockResolutionStrategy.MEDIATOR_DECISION
          )
        ).toThrow(ReasoningEngineError);
        expect(() =>
          resolver.resolveDeadlock(
            session,
            detection,
            DeadlockResolutionStrategy.MEDIATOR_DECISION
          )
        ).toThrow("No mediator available");
      });

      it("should handle mediator without vote", () => {
        const session = createSession(true);
        session.participants[2].votesCast = []; // Remove mediator vote
        const detection = createDetection();

        const resolution = resolver.resolveDeadlock(
          session,
          detection,
          DeadlockResolutionStrategy.MEDIATOR_DECISION
        );

        expect(resolution.decision).toBe("for");
        expect(resolution.confidence).toBeCloseTo(0.5, 1); // Low confidence
      });
    });

    describe("TIMEOUT_DEFAULT", () => {
      it("should resolve to conservative default on timeout", () => {
        const session = createSession();
        const detection = createDetection();

        const resolution = resolver.resolveDeadlock(
          session,
          detection,
          DeadlockResolutionStrategy.TIMEOUT_DEFAULT
        );

        expect(resolution.strategy).toBe(
          DeadlockResolutionStrategy.TIMEOUT_DEFAULT
        );
        expect(resolution.decision).toBe("against"); // Conservative
        expect(resolution.mediatorOverride).toBe(false);
        expect(resolution.reason).toContain("Timeout");
      });
    });

    describe("WEIGHTED_COMPROMISE", () => {
      it("should calculate weighted compromise", () => {
        const session = createSession();
        session.participants[0].weight = 2.0; // Higher weight for agent-1
        const detection = createDetection();

        const resolution = resolver.resolveDeadlock(
          session,
          detection,
          DeadlockResolutionStrategy.WEIGHTED_COMPROMISE
        );

        expect(resolution.strategy).toBe(
          DeadlockResolutionStrategy.WEIGHTED_COMPROMISE
        );
        expect(resolution.decision).toBe("for"); // agent-1 has higher weight
        expect(resolution.reason).toContain("Weighted compromise");
      });

      it("should consider vote confidence in weighting", () => {
        const session = createSession();
        session.participants[0].votesCast[0].confidence = 1.0; // High confidence
        session.participants[1].votesCast[0].confidence = 0.3; // Low confidence
        const detection = createDetection();

        const resolution = resolver.resolveDeadlock(
          session,
          detection,
          DeadlockResolutionStrategy.WEIGHTED_COMPROMISE
        );

        expect(resolution.decision).toBe("for"); // Higher confidence vote wins
      });
    });

    describe("ESCALATE_TO_ADMIN", () => {
      it("should escalate to administrator", () => {
        const session = createSession();
        const detection = createDetection();

        const resolution = resolver.resolveDeadlock(
          session,
          detection,
          DeadlockResolutionStrategy.ESCALATE_TO_ADMIN
        );

        expect(resolution.strategy).toBe(
          DeadlockResolutionStrategy.ESCALATE_TO_ADMIN
        );
        expect(resolution.decision).toBe("escalated");
        expect(resolution.confidence).toBe(1.0);
        expect(resolution.reason).toContain("Escalated");
      });
    });

    describe("SPLIT_DECISION", () => {
      it("should allow split decision", () => {
        const session = createSession();
        const detection = createDetection();

        const resolution = resolver.resolveDeadlock(
          session,
          detection,
          DeadlockResolutionStrategy.SPLIT_DECISION
        );

        expect(resolution.strategy).toBe(
          DeadlockResolutionStrategy.SPLIT_DECISION
        );
        expect(resolution.decision).toBe("split");
        expect(resolution.confidence).toBeCloseTo(0.5, 1);
        expect(resolution.reason).toContain("Split decision");
      });
    });

    it("should use suggested resolution from detection", () => {
      const session = createSession(true);
      const detection = createDetection();
      detection.suggestedResolution =
        DeadlockResolutionStrategy.MEDIATOR_DECISION;

      const resolution = resolver.resolveDeadlock(session, detection);

      expect(resolution.strategy).toBe(
        DeadlockResolutionStrategy.MEDIATOR_DECISION
      );
    });

    it("should throw error for unknown strategy", () => {
      const session = createSession();
      const detection = createDetection();

      expect(() =>
        resolver.resolveDeadlock(session, detection, "unknown" as any)
      ).toThrow(ReasoningEngineError);
    });
  });

  describe("getDeadlockPatterns", () => {
    it("should return empty array initially", () => {
      const patterns = resolver.getDeadlockPatterns();
      expect(patterns).toEqual([]);
    });

    it("should return tracked patterns", () => {
      const session: DebateSession = {
        id: "debate-1",
        config: {
          id: "debate-1",
          topic: "Test",
          maxParticipants: 10,
          maxDuration: 60000,
          consensusAlgorithm: "simple_majority" as any,
          deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
          requiresUnanimous: false,
          allowAppeals: true,
        },
        participants: [
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
        ],
        state: DebateState.CONSENSUS_FORMING,
        startTime: new Date(),
        arguments: [],
        reasoningChain: [],
      };

      const votes: DebateVote[] = [
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
      ];

      resolver.detectDeadlock(session, votes);

      const patterns = resolver.getDeadlockPatterns();
      expect(patterns.length).toBeGreaterThan(0);
      expect(patterns[0].occurrences).toBe(1);
    });

    it("should increment occurrence count for repeat patterns", () => {
      const session: DebateSession = {
        id: "debate-1",
        config: {
          id: "debate-1",
          topic: "Test",
          maxParticipants: 10,
          maxDuration: 60000,
          consensusAlgorithm: "simple_majority" as any,
          deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
          requiresUnanimous: false,
          allowAppeals: true,
        },
        participants: [
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
        ],
        state: DebateState.CONSENSUS_FORMING,
        startTime: new Date(),
        arguments: [],
        reasoningChain: [],
      };

      const votes: DebateVote[] = [
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
      ];

      resolver.detectDeadlock(session, votes);
      resolver.detectDeadlock(session, votes); // Second time

      const patterns = resolver.getDeadlockPatterns();
      expect(patterns[0].occurrences).toBe(2);
    });
  });

  describe("clearPatterns", () => {
    it("should clear all tracked patterns", () => {
      const session: DebateSession = {
        id: "debate-1",
        config: {
          id: "debate-1",
          topic: "Test",
          maxParticipants: 10,
          maxDuration: 60000,
          consensusAlgorithm: "simple_majority" as any,
          deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
          requiresUnanimous: false,
          allowAppeals: true,
        },
        participants: [
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
        ],
        state: DebateState.CONSENSUS_FORMING,
        startTime: new Date(),
        arguments: [],
        reasoningChain: [],
      };

      const votes: DebateVote[] = [
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "for",
          confidence: 0.8,
          reasoning: "Support",
          timestamp: new Date(),
        },
        {
          agentId: "agent-2",
          position: "against",
          confidence: 0.8,
          reasoning: "Oppose",
          timestamp: new Date(),
        },
      ];

      resolver.detectDeadlock(session, votes);
      expect(resolver.getDeadlockPatterns().length).toBeGreaterThan(0);

      resolver.clearPatterns();

      expect(resolver.getDeadlockPatterns()).toEqual([]);
    });
  });

  describe("edge cases", () => {
    it("should handle empty vote array", () => {
      const session: DebateSession = {
        id: "debate-1",
        config: {
          id: "debate-1",
          topic: "Test",
          maxParticipants: 10,
          maxDuration: 60000,
          consensusAlgorithm: "simple_majority" as any,
          deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
          requiresUnanimous: false,
          allowAppeals: true,
        },
        participants: [],
        state: DebateState.CONSENSUS_FORMING,
        startTime: new Date(),
        arguments: [],
        reasoningChain: [],
      };

      const detection = resolver.detectDeadlock(session, []);

      expect(detection.isDeadlocked).toBe(false);
      expect(detection.votingPattern).toBe("insufficient_data");
    });

    it("should handle all abstain votes", () => {
      const session: DebateSession = {
        id: "debate-1",
        config: {
          id: "debate-1",
          topic: "Test",
          maxParticipants: 10,
          maxDuration: 60000,
          consensusAlgorithm: "simple_majority" as any,
          deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
          requiresUnanimous: false,
          allowAppeals: true,
        },
        participants: [
          {
            agentId: "agent-1",
            role: AgentRole.PROPONENT,
            weight: 1.0,
            votesCast: [],
            argumentsPresented: [],
          },
        ],
        state: DebateState.CONSENSUS_FORMING,
        startTime: new Date(),
        arguments: [],
        reasoningChain: [],
      };

      const votes: DebateVote[] = [
        {
          agentId: "agent-1",
          position: "abstain",
          confidence: 0.8,
          reasoning: "Neutral",
          timestamp: new Date(),
        },
        {
          agentId: "agent-1",
          position: "abstain",
          confidence: 0.8,
          reasoning: "Neutral",
          timestamp: new Date(),
        },
      ];

      const detection = resolver.detectDeadlock(session, votes);

      // Should not detect deadlock for abstentions only
      expect(detection.isDeadlocked).toBe(false);
    });

    it("should handle single participant", () => {
      const session: DebateSession = {
        id: "debate-1",
        config: {
          id: "debate-1",
          topic: "Test",
          maxParticipants: 10,
          maxDuration: 60000,
          consensusAlgorithm: "simple_majority" as any,
          deadlockStrategy: DeadlockResolutionStrategy.MEDIATOR_DECISION,
          requiresUnanimous: false,
          allowAppeals: true,
        },
        participants: [
          {
            agentId: "agent-1",
            role: AgentRole.PROPONENT,
            weight: 1.0,
            votesCast: [
              {
                agentId: "agent-1",
                position: "for",
                reasoning: "Support",
                confidence: 0.8,
                timestamp: new Date(),
              },
            ],
            argumentsPresented: [],
          },
        ],
        state: DebateState.CONSENSUS_FORMING,
        startTime: new Date(),
        arguments: [],
        reasoningChain: [],
      };

      const detection = resolver.detectDeadlock(session, []);

      // Single participant can't be deadlocked
      expect(detection.isDeadlocked).toBe(false);
    });
  });
});
