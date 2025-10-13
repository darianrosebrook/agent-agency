/**
 * Unit tests for Arbiter Reasoning Engine
 *
 * @author @darianrosebrook
 */

import { ArbiterReasoningEngine } from "@/reasoning/ArbiterReasoningEngine";
import {
  AgentRole,
  ConsensusAlgorithm,
  DebateState,
  ReasoningEngineError,
  InvalidArgumentError,
  ConsensusImpossibleError,
  Evidence,
} from "@/types/reasoning";

describe("ArbiterReasoningEngine", () => {
  let engine: ArbiterReasoningEngine;

  beforeEach(() => {
    engine = new ArbiterReasoningEngine({
      maximumParticipants: 10,
      minimumParticipants: 2,
      maxDebateDuration: 60000,
      defaultConsensusAlgorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
      enableDeadlockDetection: true,
      deadlockDetectionRounds: 3,
    });
  });

  describe("initiateDebate", () => {
    it("should initiate debate with valid participants", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
        { agentId: "agent-3", role: AgentRole.MEDIATOR },
      ];

      const session = await engine.initiateDebate("Test topic", participants);

      expect(session).toBeDefined();
      expect(session.config.topic).toBe("Test topic");
      expect(session.state).toBe(DebateState.AGENTS_ASSIGNED);
      expect(session.participants).toHaveLength(3);
    });

    it("should throw error for too few participants", async () => {
      const participants = [{ agentId: "agent-1", role: AgentRole.PROPONENT }];

      await expect(engine.initiateDebate("Test topic", participants)).rejects.toThrow(
        ReasoningEngineError
      );
      await expect(engine.initiateDebate("Test topic", participants)).rejects.toThrow(
        /minimum.*2/i
      );
    });

    it("should throw error for too many participants", async () => {
      const participants = Array(15)
        .fill(null)
        .map((_, i) => ({ agentId: `agent-${i}`, role: AgentRole.PROPONENT }));

      await expect(engine.initiateDebate("Test topic", participants)).rejects.toThrow(
        ReasoningEngineError
      );
      await expect(engine.initiateDebate("Test topic", participants)).rejects.toThrow(
        /maximum.*10/i
      );
    });

    it("should throw error for empty topic", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
      ];

      await expect(engine.initiateDebate("", participants)).rejects.toThrow(
        ReasoningEngineError
      );
      await expect(engine.initiateDebate("   ", participants)).rejects.toThrow(
        /cannot be empty/i
      );
    });

    it("should throw error for duplicate participant IDs", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-1", role: AgentRole.OPPONENT }, // Duplicate
        { agentId: "agent-2", role: AgentRole.MEDIATOR },
      ];

      await expect(engine.initiateDebate("Test topic", participants)).rejects.toThrow(
        ReasoningEngineError
      );
      await expect(engine.initiateDebate("Test topic", participants)).rejects.toThrow(
        /duplicate/i
      );
    });

    it("should assign default weight if not specified", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
      ];

      const session = await engine.initiateDebate("Test topic", participants);

      expect(session.participants[0].weight).toBe(1);
      expect(session.participants[1].weight).toBe(1);
    });

    it("should use custom weight if specified", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT, weight: 2 },
        { agentId: "agent-2", role: AgentRole.OPPONENT, weight: 1 },
      ];

      const session = await engine.initiateDebate("Test topic", participants);

      expect(session.participants[0].weight).toBe(2);
      expect(session.participants[1].weight).toBe(1);
    });
  });

  describe("submitArgument", () => {
    let debateId: string;

    beforeEach(async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
        { agentId: "agent-3", role: AgentRole.MEDIATOR },
      ];
      const session = await engine.initiateDebate("Test topic", participants);
      debateId = session.config.id;
    });

    it("should accept valid argument", async () => {
      const session = await engine.submitArgument(
        debateId,
        "agent-1",
        "This is a valid claim",
        [createEvidence()],
        "This is comprehensive reasoning for the claim"
      );

      expect(session.arguments).toHaveLength(1);
      expect(session.arguments[0].agentId).toBe("agent-1");
      expect(session.arguments[0].claim).toBe("This is a valid claim");
    });

    it("should reject empty claim", async () => {
      await expect(
        engine.submitArgument(
          debateId,
          "agent-1",
          "",
          [createEvidence()],
          "Some reasoning"
        )
      ).rejects.toThrow(InvalidArgumentError);
    });

    it("should reject empty reasoning", async () => {
      await expect(
        engine.submitArgument(debateId, "agent-1", "Valid claim", [createEvidence()], "")
      ).rejects.toThrow(InvalidArgumentError);
    });

    it("should reject argument from non-participant", async () => {
      await expect(
        engine.submitArgument(
          debateId,
          "unknown-agent",
          "Valid claim",
          [createEvidence()],
          "Valid reasoning"
        )
      ).rejects.toThrow(ReasoningEngineError);
      await expect(
        engine.submitArgument(
          debateId,
          "unknown-agent",
          "Valid claim",
          [createEvidence()],
          "Valid reasoning"
        )
      ).rejects.toThrow(/not a participant/i);
    });

    it("should reject argument for invalid debate ID", async () => {
      await expect(
        engine.submitArgument(
          "invalid-debate",
          "agent-1",
          "Valid claim",
          [createEvidence()],
          "Valid reasoning"
        )
      ).rejects.toThrow(ReasoningEngineError);
    });

    it("should handle multiple arguments from different agents", async () => {
      await engine.submitArgument(
        debateId,
        "agent-1",
        "First claim",
        [createEvidence()],
        "First reasoning"
      );
      const session = await engine.submitArgument(
        debateId,
        "agent-2",
        "Second claim",
        [createEvidence()],
        "Second reasoning"
      );

      expect(session.arguments).toHaveLength(2);
    });
  });

  describe("submitVote", () => {
    let debateId: string;

    beforeEach(async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
        { agentId: "agent-3", role: AgentRole.MEDIATOR },
      ];
      const session = await engine.initiateDebate("Test topic", participants);
      debateId = session.config.id;

      // Submit some arguments first
      await engine.submitArgument(
        debateId,
        "agent-1",
        "First claim",
        [createEvidence()],
        "First reasoning"
      );
      await engine.submitArgument(
        debateId,
        "agent-2",
        "Second claim",
        [createEvidence()],
        "Second reasoning"
      );
    });

    it("should accept valid vote", async () => {
      const session = await engine.submitVote(
        debateId,
        "agent-1",
        "for",
        0.9,
        "I agree"
      );

      const participant = session.participants.find((p) => p.agentId === "agent-1");
      expect(participant?.votesCast).toHaveLength(1);
      expect(participant?.votesCast[0].position).toBe("for");
    });

    it("should reject vote with invalid confidence", async () => {
      await expect(
        engine.submitVote(debateId, "agent-1", "for", 1.5, "Too high")
      ).rejects.toThrow(ReasoningEngineError);
      await expect(
        engine.submitVote(debateId, "agent-1", "for", -0.1, "Too low")
      ).rejects.toThrow(ReasoningEngineError);
    });

    it("should reject vote from non-participant", async () => {
      await expect(
        engine.submitVote(debateId, "unknown-agent", "for", 0.9, "I agree")
      ).rejects.toThrow(ReasoningEngineError);
    });

    it("should handle abstention vote", async () => {
      const session = await engine.submitVote(
        debateId,
        "agent-1",
        "abstain",
        0.5,
        "Neutral"
      );

      const participant = session.participants.find((p) => p.agentId === "agent-1");
      expect(participant?.votesCast[0].position).toBe("abstain");
    });

    it("should reject vote for invalid debate ID", async () => {
      await expect(
        engine.submitVote("invalid-debate", "agent-1", "for", 0.9, "I agree")
      ).rejects.toThrow(ReasoningEngineError);
    });
  });

  describe("formConsensus", () => {
    let debateId: string;

    beforeEach(async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
        { agentId: "agent-3", role: AgentRole.MEDIATOR },
      ];
      const session = await engine.initiateDebate("Test topic", participants);
      debateId = session.config.id;

      await engine.submitArgument(
        debateId,
        "agent-1",
        "First claim",
        [createEvidence()],
        "First reasoning"
      );
      await engine.submitArgument(
        debateId,
        "agent-2",
        "Second claim",
        [createEvidence()],
        "Second reasoning"
      );
    });

    it("should form consensus with majority votes", async () => {
      await engine.submitVote(debateId, "agent-1", "for", 0.9, "Strong support");
      await engine.submitVote(debateId, "agent-2", "for", 0.8, "Support");
      await engine.submitVote(debateId, "agent-3", "against", 0.7, "Against");

      const session = await engine.formConsensus(debateId);

      expect(session.consensusResult).toBeDefined();
      expect(session.consensusResult?.reached).toBe(true);
      expect(session.consensusResult?.outcome).toBe("accepted");
    });

    it("should fail to form consensus without votes", async () => {
      await expect(engine.formConsensus(debateId)).rejects.toThrow(
        ConsensusImpossibleError
      );
    });

    it("should fail to form consensus without majority", async () => {
      await engine.submitVote(debateId, "agent-1", "for", 0.9, "Support");
      await engine.submitVote(debateId, "agent-2", "against", 0.8, "Against");

      const session = await engine.formConsensus(debateId);

      expect(session.consensusResult?.reached).toBe(false);
      expect(session.consensusResult?.outcome).toBe("rejected");
    });

    it("should handle insufficient participation", async () => {
      await engine.submitVote(debateId, "agent-1", "for", 0.9, "Only vote");

      // Only 1 out of 3 voted (33% < 50% minimum)
      await expect(engine.formConsensus(debateId)).rejects.toThrow(
        ConsensusImpossibleError
      );
    });
  });

  describe("getDebateResults", () => {
    it("should return comprehensive debate results", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
        { agentId: "agent-3", role: AgentRole.MEDIATOR },
      ];
      const session = await engine.initiateDebate("Test topic", participants);
      const debateId = session.config.id;

      await engine.submitArgument(
        debateId,
        "agent-1",
        "First claim",
        [createEvidence()],
        "First reasoning"
      );
      await engine.submitArgument(
        debateId,
        "agent-2",
        "Second claim",
        [createEvidence()],
        "Second reasoning"
      );

      await engine.submitVote(debateId, "agent-1", "for", 0.9, "Support");
      await engine.submitVote(debateId, "agent-2", "for", 0.8, "Support");
      await engine.submitVote(debateId, "agent-3", "against", 0.7, "Against");

      await engine.formConsensus(debateId);

      const results = await engine.getDebateResults(debateId);

      expect(results.session).toBeDefined();
      expect(results.consensus).toBeDefined();
      expect(results.topArguments).toBeDefined();
      expect(results.evidenceSummary).toBeDefined();
    });

    it("should throw error for invalid debate ID", async () => {
      await expect(engine.getDebateResults("invalid-debate")).rejects.toThrow(
        ReasoningEngineError
      );
    });
  });

  describe("closeDebate", () => {
    it("should close completed debate", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
        { agentId: "agent-3", role: AgentRole.MEDIATOR },
      ];
      const session = await engine.initiateDebate("Test topic", participants);
      const debateId = session.config.id;

      // Complete the debate flow
      await engine.submitArgument(
        debateId,
        "agent-1",
        "Test claim",
        [createEvidence()],
        "Test reasoning"
      );
      await engine.submitVote(debateId, "agent-1", "for", 0.9, "Support");
      await engine.submitVote(debateId, "agent-2", "for", 0.8, "Support");
      await engine.submitVote(debateId, "agent-3", "for", 0.7, "Support");
      await engine.formConsensus(debateId);

      expect(engine.getActiveDebateIds()).toContain(debateId);

      await engine.closeDebate(debateId);

      expect(engine.getActiveDebateIds()).not.toContain(debateId);
    });

    it("should throw error for invalid debate ID", async () => {
      await expect(engine.closeDebate("invalid-debate")).rejects.toThrow(
        ReasoningEngineError
      );
    });
  });

  describe("getActiveDebateCount and getActiveDebateIds", () => {
    it("should track active debates", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
      ];

      expect(engine.getActiveDebateCount()).toBe(0);
      expect(engine.getActiveDebateIds()).toHaveLength(0);

      const session1 = await engine.initiateDebate("Topic 1", participants);
      expect(engine.getActiveDebateCount()).toBe(1);

      const session2 = await engine.initiateDebate("Topic 2", participants);
      expect(engine.getActiveDebateCount()).toBe(2);
      expect(engine.getActiveDebateIds()).toContain(session1.config.id);
      expect(engine.getActiveDebateIds()).toContain(session2.config.id);
    });

    it("should remove closed debates from active list", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
        { agentId: "agent-3", role: AgentRole.MEDIATOR },
      ];

      const session = await engine.initiateDebate("Test topic", participants);
      const debateId = session.config.id;
      expect(engine.getActiveDebateCount()).toBe(1);

      // Complete the debate before closing
      await engine.submitArgument(
        debateId,
        "agent-1",
        "Test claim",
        [createEvidence()],
        "Test reasoning"
      );
      await engine.submitVote(debateId, "agent-1", "for", 0.9, "Support");
      await engine.submitVote(debateId, "agent-2", "for", 0.8, "Support");
      await engine.submitVote(debateId, "agent-3", "for", 0.7, "Support");
      await engine.formConsensus(debateId);

      await engine.closeDebate(debateId);
      expect(engine.getActiveDebateCount()).toBe(0);
    });
  });

  describe("edge cases and error handling", () => {
    it("should handle multiple debates simultaneously", async () => {
      const participants1 = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
      ];
      const participants2 = [
        { agentId: "agent-3", role: AgentRole.PROPONENT },
        { agentId: "agent-4", role: AgentRole.OPPONENT },
      ];

      const session1 = await engine.initiateDebate("Topic 1", participants1);
      const session2 = await engine.initiateDebate("Topic 2", participants2);

      expect(session1.config.id).not.toBe(session2.config.id);
      expect(session1.state).toBe(DebateState.AGENTS_ASSIGNED);
      expect(session2.state).toBe(DebateState.AGENTS_ASSIGNED);
    });

    it("should generate unique debate IDs", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
      ];

      const session1 = await engine.initiateDebate("Topic", participants);
      const session2 = await engine.initiateDebate("Topic", participants);

      expect(session1.config.id).not.toBe(session2.config.id);
    });

    it("should handle empty evidence array", async () => {
      const participants = [
        { agentId: "agent-1", role: AgentRole.PROPONENT },
        { agentId: "agent-2", role: AgentRole.OPPONENT },
      ];
      const session = await engine.initiateDebate("Test topic", participants);

      const updatedSession = await engine.submitArgument(
        session.config.id,
        "agent-1",
        "Claim without evidence",
        [],
        "Reasoning without supporting evidence"
      );

      expect(updatedSession.arguments).toHaveLength(1);
      expect(updatedSession.arguments[0].evidence).toHaveLength(0);
    });
  });
});

// Test Helper Functions

function createEvidence(
  credibilityScore: number = 0.8,
  verificationStatus: "verified" | "unverified" | "disputed" = "verified"
): Evidence {
  return {
    id: `evidence-${Math.random().toString(36).substr(2, 9)}`,
    source: `Source ${Math.random().toString(36).substr(2, 5)}`,
    content: "Test evidence content",
    credibilityScore,
    verificationStatus,
    timestamp: new Date(),
  };
}
