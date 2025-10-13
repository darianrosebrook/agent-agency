/**
 * Unit tests for AppealHandler
 *
 * Tests appeal submission, review, finalization, and outcome determination.
 */

import {
  AppealHandler,
  AppealRequest,
  AppealStatus,
} from "@/reasoning/AppealHandler";
import {
  AgentRole,
  DebateSession,
  DebateState,
  ReasoningEngineError,
} from "@/types/reasoning";

describe("AppealHandler", () => {
  let handler: AppealHandler;

  beforeEach(() => {
    handler = new AppealHandler();
  });

  // Helper function to create a test debate session
  const createSession = (): DebateSession => {
    return {
      id: "debate-1",
      config: {
        id: "debate-1",
        topic: "Test debate",
        maxParticipants: 10,
        maxDuration: 60000,
        consensusAlgorithm: "simple_majority" as any,
        deadlockStrategy: "timeout_default" as any,
        requiresUnanimous: false,
        allowAppeals: true,
      },
      state: DebateState.DELIBERATION,
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
        {
          agentId: "mediator",
          role: AgentRole.MEDIATOR,
          weight: 1.0,
          votesCast: [],
          argumentsPresented: [],
        },
      ],
      arguments: [],
      reasoningChain: [],
      startTime: new Date(),
    };
  };

  // Helper function to create an appeal request
  const createAppealRequest = (
    overrides?: Partial<AppealRequest>
  ): AppealRequest => {
    return {
      appealId: "appeal-1",
      agentId: "agent-1",
      targetDecision: "Previous vote",
      reason: "Lack of evidence",
      evidence: ["evidence-1", "evidence-2"],
      timestamp: new Date(),
      ...overrides,
    };
  };

  describe("submitAppeal", () => {
    it("should submit a valid appeal", () => {
      const session = createSession();
      const request = createAppealRequest();

      const record = handler.submitAppeal(session, request);

      expect(record.request).toEqual(request);
      expect(record.status).toBe(AppealStatus.SUBMITTED);
      expect(record.reviews).toHaveLength(0);
    });

    it("should throw error for non-participant", () => {
      const session = createSession();
      const request = createAppealRequest({ agentId: "non-participant" });

      expect(() => handler.submitAppeal(session, request)).toThrow(
        ReasoningEngineError
      );
      expect(() => handler.submitAppeal(session, request)).toThrow(
        "not a debate participant"
      );
    });

    it("should enforce appeal limit per agent", () => {
      const session = createSession();
      const customHandler = new AppealHandler({ maxAppealsPerAgent: 2 });

      // Submit first appeal
      customHandler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-1" })
      );

      // Submit second appeal
      customHandler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-2" })
      );

      // Third appeal should fail
      expect(() =>
        customHandler.submitAppeal(
          session,
          createAppealRequest({ appealId: "appeal-3" })
        )
      ).toThrow("maximum appeals");
    });

    it("should reject appeals during voting if configured", () => {
      const customHandler = new AppealHandler({ allowDuringVoting: false });
      const session = createSession();
      session.state = DebateState.CONSENSUS_FORMING;

      const request = createAppealRequest();

      expect(() => customHandler.submitAppeal(session, request)).toThrow(
        "Appeals not allowed during consensus forming"
      );
    });

    it("should allow appeals during voting if configured", () => {
      const customHandler = new AppealHandler({ allowDuringVoting: true });
      const session = createSession();
      session.state = DebateState.CONSENSUS_FORMING;

      const request = createAppealRequest();
      const record = customHandler.submitAppeal(session, request);

      expect(record.status).toBe(AppealStatus.SUBMITTED);
    });
  });

  describe("reviewAppeal", () => {
    it("should add review to appeal", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);
      const record = handler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "uphold",
        "Original decision was correct"
      );

      expect(record.reviews).toHaveLength(1);
      expect(record.reviews[0].reviewer).toBe("mediator");
      expect(record.reviews[0].recommendation).toBe("uphold");
      expect(record.status).toBe(AppealStatus.UNDER_REVIEW);
    });

    it("should throw error for non-participant reviewer", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);

      expect(() =>
        handler.reviewAppeal(
          session,
          "appeal-1",
          "non-participant",
          "uphold",
          "Reason"
        )
      ).toThrow("not a debate participant");
    });

    it("should require mediator role when configured", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);

      expect(() =>
        handler.reviewAppeal(session, "appeal-1", "agent-1", "uphold", "Reason")
      ).toThrow("mediator role");
    });

    it("should allow non-mediator review when not required", () => {
      const customHandler = new AppealHandler({
        requireMediatorApproval: false,
      });
      const session = createSession();
      const request = createAppealRequest();

      customHandler.submitAppeal(session, request);
      const record = customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-2",
        "overturn",
        "Valid concerns raised"
      );

      expect(record.reviews).toHaveLength(1);
      expect(record.reviews[0].reviewer).toBe("agent-2");
    });
  });

  describe("finalizeAppeal", () => {
    it("should finalize appeal with unanimous review", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);
      handler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "uphold",
        "Correct"
      );

      const outcome = handler.finalizeAppeal(
        session,
        "appeal-1",
        "uphold",
        "Original decision stands"
      );

      expect(outcome.decision).toBe("uphold");
      expect(outcome.status).toBe(AppealStatus.REJECTED);
      expect(outcome.confidence).toBe(1.0);
      expect(outcome.reviewers).toContain("mediator");
    });

    it("should throw error if no reviews exist", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);

      expect(() =>
        handler.finalizeAppeal(session, "appeal-1", "uphold", "Reason")
      ).toThrow("must be reviewed before finalization");
    });

    it("should calculate confidence from review consensus", () => {
      const customHandler = new AppealHandler({
        requireMediatorApproval: false,
      });
      const session = createSession();
      const request = createAppealRequest();

      customHandler.submitAppeal(session, request);
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-1",
        "overturn",
        "Valid"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-2",
        "overturn",
        "Agree"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "uphold",
        "Disagree"
      );

      const outcome = customHandler.finalizeAppeal(
        session,
        "appeal-1",
        "overturn",
        "Majority supports overturn"
      );

      expect(outcome.confidence).toBeCloseTo(0.667, 2);
      expect(outcome.status).toBe(AppealStatus.APPROVED);
    });

    it("should escalate appeal with low confidence", () => {
      const customHandler = new AppealHandler({
        requireMediatorApproval: false,
        minConfidenceForApproval: 0.8,
      });
      const session = createSession();
      const request = createAppealRequest();

      customHandler.submitAppeal(session, request);
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-1",
        "uphold",
        "Original correct"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-2",
        "overturn",
        "Should change"
      );

      const outcome = customHandler.finalizeAppeal(
        session,
        "appeal-1",
        "uphold",
        "Low consensus"
      );

      expect(outcome.status).toBe(AppealStatus.ESCALATED);
      expect(outcome.confidence).toBe(0.5);
    });

    it("should approve overturn decisions", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);
      handler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "overturn",
        "Valid concerns"
      );

      const outcome = handler.finalizeAppeal(
        session,
        "appeal-1",
        "overturn",
        "Decision overturned"
      );

      expect(outcome.status).toBe(AppealStatus.APPROVED);
      expect(outcome.decision).toBe("overturn");
    });

    it("should include modifications in outcome", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);
      handler.reviewAppeal(session, "appeal-1", "mediator", "modify", "Adjust");

      const outcome = handler.finalizeAppeal(
        session,
        "appeal-1",
        "modify",
        "Modified decision",
        "Change voting weight to 0.8"
      );

      expect(outcome.modifications).toBe("Change voting weight to 0.8");
      expect(outcome.status).toBe(AppealStatus.APPROVED);
    });
  });

  describe("withdrawAppeal", () => {
    it("should withdraw submitted appeal", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);
      handler.withdrawAppeal(session, "appeal-1");

      const appeals = handler.getAppeals("debate-1");
      expect(appeals[0].status).toBe(AppealStatus.WITHDRAWN);
    });

    it("should throw error when withdrawing completed appeal", () => {
      const session = createSession();
      const request = createAppealRequest();

      handler.submitAppeal(session, request);
      handler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "uphold",
        "Correct"
      );
      handler.finalizeAppeal(session, "appeal-1", "uphold", "Decision stands");

      expect(() => handler.withdrawAppeal(session, "appeal-1")).toThrow(
        "Cannot withdraw completed appeal"
      );
    });
  });

  describe("getAppeals", () => {
    it("should return all appeals for a debate", () => {
      const session = createSession();

      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-1" })
      );
      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-2", agentId: "agent-2" })
      );

      const appeals = handler.getAppeals("debate-1");
      expect(appeals).toHaveLength(2);
    });

    it("should return empty array for debate with no appeals", () => {
      const appeals = handler.getAppeals("debate-1");
      expect(appeals).toHaveLength(0);
    });
  });

  describe("getAppealsByAgent", () => {
    it("should filter appeals by agent", () => {
      const session = createSession();

      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-1", agentId: "agent-1" })
      );
      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-2", agentId: "agent-2" })
      );
      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-3", agentId: "agent-1" })
      );

      const agent1Appeals = handler.getAppealsByAgent("debate-1", "agent-1");
      expect(agent1Appeals).toHaveLength(2);
      expect(agent1Appeals.every((a) => a.request.agentId === "agent-1")).toBe(
        true
      );
    });
  });

  describe("getAppealsByStatus", () => {
    it("should filter appeals by status", () => {
      const session = createSession();

      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-1" })
      );
      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-2", agentId: "agent-2" })
      );
      handler.reviewAppeal(
        session,
        "appeal-2",
        "mediator",
        "uphold",
        "Correct"
      );

      const submitted = handler.getAppealsByStatus(
        "debate-1",
        AppealStatus.SUBMITTED
      );
      const underReview = handler.getAppealsByStatus(
        "debate-1",
        AppealStatus.UNDER_REVIEW
      );

      expect(submitted).toHaveLength(1);
      expect(underReview).toHaveLength(1);
    });
  });

  describe("shouldEscalate", () => {
    it("should return false for appeal with no reviews", () => {
      const session = createSession();
      const request = createAppealRequest();

      const record = handler.submitAppeal(session, request);
      expect(handler.shouldEscalate(record)).toBe(false);
    });

    it("should return true for complete disagreement", () => {
      const customHandler = new AppealHandler({
        requireMediatorApproval: false,
      });
      const session = createSession();
      const request = createAppealRequest();

      customHandler.submitAppeal(session, request);
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-1",
        "uphold",
        "Correct"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-2",
        "overturn",
        "Wrong"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "modify",
        "Adjust"
      );

      const appeals = customHandler.getAppeals("debate-1");
      expect(customHandler.shouldEscalate(appeals[0])).toBe(true);
    });

    it("should return true for low confidence", () => {
      const customHandler = new AppealHandler({
        requireMediatorApproval: false,
        escalationThreshold: 0.75,
      });
      const session = createSession();
      const request = createAppealRequest();

      customHandler.submitAppeal(session, request);
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-1",
        "uphold",
        "Correct"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-2",
        "uphold",
        "Agree"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "overturn",
        "Disagree"
      );

      const appeals = customHandler.getAppeals("debate-1");
      expect(customHandler.shouldEscalate(appeals[0])).toBe(true); // 0.667 < 0.75
    });

    it("should return false for high consensus", () => {
      const customHandler = new AppealHandler({
        requireMediatorApproval: false,
      });
      const session = createSession();
      const request = createAppealRequest();

      customHandler.submitAppeal(session, request);
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-1",
        "uphold",
        "Correct"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-2",
        "uphold",
        "Agree"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "uphold",
        "Confirm"
      );

      const appeals = customHandler.getAppeals("debate-1");
      expect(customHandler.shouldEscalate(appeals[0])).toBe(false);
    });
  });

  describe("clearAppeals", () => {
    it("should clear all appeals for a debate", () => {
      const session = createSession();

      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-1" })
      );
      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-2", agentId: "agent-2" })
      );

      handler.clearAppeals("debate-1");
      const appeals = handler.getAppeals("debate-1");

      expect(appeals).toHaveLength(0);
    });
  });

  describe("getAppealStatistics", () => {
    it("should return correct statistics", () => {
      const session = createSession();

      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-1" })
      );
      handler.submitAppeal(
        session,
        createAppealRequest({ appealId: "appeal-2", agentId: "agent-2" })
      );
      handler.reviewAppeal(
        session,
        "appeal-1",
        "mediator",
        "uphold",
        "Correct"
      );
      handler.reviewAppeal(
        session,
        "appeal-2",
        "mediator",
        "overturn",
        "Change"
      );

      const stats = handler.getAppealStatistics("debate-1");

      expect(stats.total).toBe(2);
      expect(stats.byStatus[AppealStatus.UNDER_REVIEW]).toBe(2);
      expect(stats.byAgent["agent-1"]).toBe(1);
      expect(stats.byAgent["agent-2"]).toBe(1);
      expect(stats.averageReviews).toBe(1);
    });

    it("should handle debate with no appeals", () => {
      const stats = handler.getAppealStatistics("debate-1");

      expect(stats.total).toBe(0);
      expect(stats.averageReviews).toBe(0);
    });
  });

  describe("edge cases", () => {
    it("should handle appeal for non-existent debate", () => {
      expect(() =>
        handler.reviewAppeal(
          createSession(),
          "non-existent",
          "mediator",
          "uphold",
          "Reason"
        )
      ).toThrow("No appeals found");
    });

    it("should handle non-existent appeal ID", () => {
      const session = createSession();
      handler.submitAppeal(session, createAppealRequest());

      expect(() =>
        handler.reviewAppeal(
          session,
          "non-existent",
          "mediator",
          "uphold",
          "Reason"
        )
      ).toThrow("Appeal non-existent not found");
    });

    it("should handle multiple reviews from same reviewer", () => {
      const customHandler = new AppealHandler({
        requireMediatorApproval: false,
      });
      const session = createSession();

      customHandler.submitAppeal(session, createAppealRequest());
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-1",
        "uphold",
        "First review"
      );
      customHandler.reviewAppeal(
        session,
        "appeal-1",
        "agent-1",
        "overturn",
        "Changed mind"
      );

      const appeals = customHandler.getAppeals("debate-1");
      expect(appeals[0].reviews).toHaveLength(2);
    });
  });
});
