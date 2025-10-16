/**
 * Unit tests for Consensus Engine
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { ConsensusEngine } from "@/reasoning/ConsensusEngine";
import {
  ConsensusAlgorithm,
  ConsensusImpossibleError,
  DebateParticipant,
  DebateVote,
} from "@/types/reasoning";

describe("ConsensusEngine", () => {
  describe("formConsensus", () => {
    it("should form consensus with simple majority", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
        createParticipant("agent-3", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
        createVote("agent-3", "against", 0.7),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
      });

      expect(result.reached).toBe(true);
      expect(result.outcome).toBe("accepted");
      expect(result.votingBreakdown.for).toBe(2);
      expect(result.votingBreakdown.against).toBe(1);
    });

    it("should fail consensus without majority", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "against", 0.8),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
      });

      expect(result.reached).toBe(false);
      expect(result.outcome).toBe("rejected");
    });

    it("should handle weighted majority algorithm", () => {
      const participants = [
        createParticipant("agent-1", 2), // weight 2
        createParticipant("agent-2", 1), // weight 1
        createParticipant("agent-3", 1), // weight 1
      ];

      const votes = [
        createVote("agent-1", "for", 0.9), // 2 weighted votes
        createVote("agent-2", "against", 0.8),
        createVote("agent-3", "against", 0.7),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.WEIGHTED_MAJORITY,
      });

      expect(result.reached).toBe(false); // 2 for vs 2 against = tie
    });

    it("should require unanimous agreement for unanimous algorithm", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
        createParticipant("agent-3", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
        createVote("agent-3", "for", 0.7),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.UNANIMOUS,
      });

      expect(result.reached).toBe(true);
      expect(result.outcome).toBe("accepted");
    });

    it("should fail unanimous with any against vote", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
        createParticipant("agent-3", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
        createVote("agent-3", "against", 0.7),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.UNANIMOUS,
      });

      expect(result.reached).toBe(false);
    });

    it("should handle supermajority algorithm", () => {
      const participants = Array(10)
        .fill(null)
        .map((_, i) => createParticipant(`agent-${i}`, 1));

      const votes = [
        ...Array(7)
          .fill(null)
          .map((_, i) => createVote(`agent-${i}`, "for", 0.8)),
        ...Array(3)
          .fill(null)
          .map((_, i) => createVote(`agent-${i + 7}`, "against", 0.7)),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SUPERMAJORITY,
        supermajorityThreshold: 0.67,
      });

      expect(result.reached).toBe(true); // 7/10 = 70% > 67%
    });

    it("should fail supermajority below threshold", () => {
      const participants = Array(10)
        .fill(null)
        .map((_, i) => createParticipant(`agent-${i}`, 1));

      const votes = [
        ...Array(6)
          .fill(null)
          .map((_, i) => createVote(`agent-${i}`, "for", 0.8)),
        ...Array(4)
          .fill(null)
          .map((_, i) => createVote(`agent-${i + 6}`, "against", 0.7)),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SUPERMAJORITY,
        supermajorityThreshold: 0.67,
      });

      expect(result.reached).toBe(false); // 6/10 = 60% < 67%
    });

    it("should throw error for insufficient participation", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
        createParticipant("agent-3", 1),
      ];

      const votes = [createVote("agent-1", "for", 0.9)];

      expect(() =>
        ConsensusEngine.formConsensus(votes, participants, {
          algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
          minimumParticipation: 0.67,
        })
      ).toThrow(ConsensusImpossibleError);
    });

    it("should mark as modified if confidence too low", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
        createParticipant("agent-3", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.4),
        createVote("agent-2", "for", 0.3),
        createVote("agent-3", "against", 0.5),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
        confidenceThreshold: 0.6,
      });

      expect(result.reached).toBe(true);
      expect(result.outcome).toBe("modified");
    });

    it("should handle abstentions", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
        createParticipant("agent-3", 1),
        createParticipant("agent-4", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.85),
        createVote("agent-3", "against", 0.8),
        createVote("agent-4", "abstain", 0.7),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
      });

      expect(result.votingBreakdown.abstain).toBe(1);
      expect(result.reached).toBe(true); // 2 for > 1 against (abstain doesn't count)
    });

    it("should generate comprehensive reasoning", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
      });

      expect(result.reasoning).toContain("simple_majority");
      expect(result.reasoning).toContain("%");
      expect(result.reasoning).toContain("Consensus reached");
    });
  });

  describe("canReachConsensus", () => {
    it("should detect when consensus is possible", () => {
      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
      ];

      const canReach = ConsensusEngine.canReachConsensus(
        votes,
        5,
        ConsensusAlgorithm.SIMPLE_MAJORITY
      );

      expect(canReach).toBe(true);
    });

    it("should detect when consensus is impossible", () => {
      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "against", 0.8),
        createVote("agent-3", "against", 0.7),
        createVote("agent-4", "against", 0.6),
      ];

      const canReach = ConsensusEngine.canReachConsensus(
        votes,
        5,
        ConsensusAlgorithm.SIMPLE_MAJORITY
      );

      expect(canReach).toBe(false); // 1 for + 1 remaining < 3 against
    });

    it("should detect unanimous consensus is impossible with any against", () => {
      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "against", 0.8),
      ];

      const canReach = ConsensusEngine.canReachConsensus(
        votes,
        5,
        ConsensusAlgorithm.UNANIMOUS
      );

      expect(canReach).toBe(false);
    });
  });

  describe("predictOutcome", () => {
    it("should predict likely accepted with strong majority", () => {
      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
        createVote("agent-3", "for", 0.7),
      ];

      const prediction = ConsensusEngine.predictOutcome(
        votes,
        5,
        ConsensusAlgorithm.SIMPLE_MAJORITY
      );

      expect(prediction).toBe("likely_accepted");
    });

    it("should predict likely rejected with strong opposition", () => {
      const votes = [
        createVote("agent-1", "against", 0.9),
        createVote("agent-2", "against", 0.8),
        createVote("agent-3", "against", 0.7),
      ];

      const prediction = ConsensusEngine.predictOutcome(
        votes,
        5,
        ConsensusAlgorithm.SIMPLE_MAJORITY
      );

      expect(prediction).toBe("likely_rejected");
    });

    it("should predict uncertain with low participation", () => {
      const votes = [createVote("agent-1", "for", 0.9)];

      const prediction = ConsensusEngine.predictOutcome(
        votes,
        10,
        ConsensusAlgorithm.SIMPLE_MAJORITY
      );

      expect(prediction).toBe("uncertain");
    });

    it("should predict uncertain with close vote", () => {
      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "against", 0.8),
        createVote("agent-3", "for", 0.7),
      ];

      const prediction = ConsensusEngine.predictOutcome(
        votes,
        5,
        ConsensusAlgorithm.SIMPLE_MAJORITY
      );

      expect(prediction).toBe("uncertain");
    });
  });

  describe("validateConsensusResult", () => {
    it("should validate correct simple majority result", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
        createParticipant("agent-3", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
        createVote("agent-3", "against", 0.7),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
      });

      const isValid = ConsensusEngine.validateConsensusResult(
        result,
        votes,
        participants
      );

      expect(isValid).toBe(true);
    });

    it("should reject incorrect vote counts", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.SIMPLE_MAJORITY,
      });

      // Tamper with result
      result.votingBreakdown.for = 5;

      const isValid = ConsensusEngine.validateConsensusResult(
        result,
        votes,
        participants
      );

      expect(isValid).toBe(false);
    });

    it("should validate unanimous result correctly", () => {
      const participants = [
        createParticipant("agent-1", 1),
        createParticipant("agent-2", 1),
      ];

      const votes = [
        createVote("agent-1", "for", 0.9),
        createVote("agent-2", "for", 0.8),
      ];

      const result = ConsensusEngine.formConsensus(votes, participants, {
        algorithm: ConsensusAlgorithm.UNANIMOUS,
      });

      const isValid = ConsensusEngine.validateConsensusResult(
        result,
        votes,
        participants
      );

      expect(isValid).toBe(true);
    });
  });
});

// Test Helper Functions

function createParticipant(agentId: string, weight: number): DebateParticipant {
  return {
    agentId,
    role: "proponent" as any,
    weight,
    argumentsPresented: [],
    votesCast: [],
  };
}

function createVote(
  agentId: string,
  position: "for" | "against" | "abstain",
  confidence: number
): DebateVote {
  return {
    agentId,
    position,
    confidence,
    reasoning: "Test reasoning",
    timestamp: new Date(),
  };
}
