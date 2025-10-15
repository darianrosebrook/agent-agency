/**
 * Unit tests for Argument Structure
 *
 * @author @darianrosebrook
 */

import { ArgumentStructure } from "@/reasoning/ArgumentStructure";
import { Argument, Evidence, InvalidArgumentError } from "@/types/reasoning";

describe("ArgumentStructure", () => {
  afterEach(() => {
    jest.clearAllMocks();
  });

  afterAll(() => {
    // Final cleanup
    jest.clearAllMocks();
  });

  describe("createArgument", () => {
    it("should create valid argument with all fields", () => {
      const evidence = [createTestEvidence()];
      const argument = ArgumentStructure.createArgument(
        "agent-1",
        "Test claim",
        evidence,
        "This is a detailed reasoning explanation for the test claim."
      );

      expect(argument.id).toBeDefined();
      expect(argument.agentId).toBe("agent-1");
      expect(argument.claim).toBe("Test claim");
      expect(argument.evidence).toEqual(evidence);
      expect(argument.timestamp).toBeInstanceOf(Date);
      expect(argument.credibilityScore).toBeGreaterThan(0);
    });

    it("should throw error for empty claim", () => {
      expect(() =>
        ArgumentStructure.createArgument("agent-1", "", [], "Some reasoning")
      ).toThrow(InvalidArgumentError);
      expect(() =>
        ArgumentStructure.createArgument("agent-1", "   ", [], "Some reasoning")
      ).toThrow(/cannot be empty/);
    });

    it("should throw error for empty reasoning", () => {
      expect(() =>
        ArgumentStructure.createArgument("agent-1", "Test claim", [], "")
      ).toThrow(InvalidArgumentError);
      expect(() =>
        ArgumentStructure.createArgument("agent-1", "Test claim", [], "   ")
      ).toThrow(/cannot be empty/);
    });

    it("should throw error for claim exceeding 1000 characters", () => {
      const longClaim = "a".repeat(1001);

      expect(() =>
        ArgumentStructure.createArgument("agent-1", longClaim, [], "Reasoning")
      ).toThrow(InvalidArgumentError);
      expect(() =>
        ArgumentStructure.createArgument("agent-1", longClaim, [], "Reasoning")
      ).toThrow(/maximum length/);
    });

    it("should throw error for reasoning exceeding 5000 characters", () => {
      const longReasoning = "a".repeat(5001);

      expect(() =>
        ArgumentStructure.createArgument(
          "agent-1",
          "Test claim",
          [],
          longReasoning
        )
      ).toThrow(InvalidArgumentError);
      expect(() =>
        ArgumentStructure.createArgument(
          "agent-1",
          "Test claim",
          [],
          longReasoning
        )
      ).toThrow(/maximum length/);
    });

    it("should calculate credibility score on creation", () => {
      const evidence = [createTestEvidence(0.9, "verified")];
      const argument = ArgumentStructure.createArgument(
        "agent-1",
        "Well-formed claim that is neither too short nor too long",
        evidence,
        "This is a comprehensive reasoning explanation that demonstrates understanding of the topic and provides sufficient justification for the claim being made."
      );

      expect(argument.credibilityScore).toBeDefined();
      expect(argument.credibilityScore).toBeGreaterThan(0);
      expect(argument.credibilityScore).toBeLessThanOrEqual(1);
    });
  });

  describe("validateArgument", () => {
    it("should validate well-formed argument", () => {
      const argument = createTestArgument();
      const result = ArgumentStructure.validateArgument(argument);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should detect empty claim", () => {
      const argument = createTestArgument();
      argument.claim = "";

      const result = ArgumentStructure.validateArgument(argument);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain("Claim is empty");
    });

    it("should warn on very short claim", () => {
      const argument = createTestArgument();
      argument.claim = "Short";

      const result = ArgumentStructure.validateArgument(argument);

      expect(result.warnings).toContain(
        "Claim is very short (< 10 characters)"
      );
    });

    it("should detect empty reasoning", () => {
      const argument = createTestArgument();
      argument.reasoning = "";

      const result = ArgumentStructure.validateArgument(argument);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain("Reasoning is empty");
    });

    it("should warn on brief reasoning", () => {
      const argument = createTestArgument();
      argument.reasoning = "Too brief";

      const result = ArgumentStructure.validateArgument(argument);

      expect(result.warnings).toContain(
        "Reasoning is very brief (< 50 characters)"
      );
    });

    it("should warn when no evidence provided", () => {
      const argument = createTestArgument();
      argument.evidence = [];

      const result = ArgumentStructure.validateArgument(argument);

      expect(result.warnings).toContain("No evidence provided");
    });

    it("should detect missing evidence source", () => {
      const argument = createTestArgument();
      argument.evidence = [{ ...createTestEvidence(), source: "" }];

      const result = ArgumentStructure.validateArgument(argument);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.includes("missing source"))).toBe(
        true
      );
    });

    it("should detect invalid evidence credibility score", () => {
      const argument = createTestArgument();
      argument.evidence = [{ ...createTestEvidence(), credibilityScore: 1.5 }];

      const result = ArgumentStructure.validateArgument(argument);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.includes("invalid credibility"))).toBe(
        true
      );
    });
  });

  describe("calculateCredibilityScore", () => {
    it("should assign base score of 0.5 with no evidence", () => {
      const score = ArgumentStructure.calculateCredibilityScore(
        "Test claim",
        [],
        "Some reasoning"
      );

      expect(score).toBeLessThan(0.6); // Base 0.5 with penalties
    });

    it("should increase score with high-quality evidence", () => {
      const highQualityEvidence = [
        createTestEvidence(0.9, "verified"),
        createTestEvidence(0.85, "verified"),
      ];

      const score = ArgumentStructure.calculateCredibilityScore(
        "Test claim",
        highQualityEvidence,
        "Some reasoning"
      );

      expect(score).toBeGreaterThan(0.7);
    });

    it("should decrease score with disputed evidence", () => {
      const disputedEvidence = [
        createTestEvidence(0.8, "disputed"),
        createTestEvidence(0.7, "disputed"),
      ];

      const score = ArgumentStructure.calculateCredibilityScore(
        "Test claim",
        disputedEvidence,
        "Some reasoning"
      );

      expect(score).toBeLessThan(0.6);
    });

    it("should increase score for comprehensive reasoning", () => {
      const comprehensiveReasoning = "a".repeat(500);
      const score = ArgumentStructure.calculateCredibilityScore(
        "Test claim",
        [createTestEvidence()],
        comprehensiveReasoning
      );

      expect(score).toBeGreaterThan(0.5);
    });

    it("should increase score for well-sized claim", () => {
      const wellSizedClaim =
        "This is a well-sized claim that is neither too short nor too long";
      const score = ArgumentStructure.calculateCredibilityScore(
        wellSizedClaim,
        [createTestEvidence()],
        "Some reasoning"
      );

      expect(score).toBeGreaterThan(0.4);
    });

    it("should ensure score stays within [0, 1] range", () => {
      const maxEvidence = Array(10)
        .fill(null)
        .map(() => createTestEvidence(1.0, "verified"));
      const score = ArgumentStructure.calculateCredibilityScore(
        "Test claim",
        maxEvidence,
        "a".repeat(1000)
      );

      expect(score).toBeLessThanOrEqual(1.0);
      expect(score).toBeGreaterThanOrEqual(0);
    });
  });

  describe("compareArguments", () => {
    it("should rank higher credibility argument first", () => {
      const arg1 = createTestArgument();
      arg1.credibilityScore = 0.9;

      const arg2 = createTestArgument();
      arg2.credibilityScore = 0.5;

      const comparison = ArgumentStructure.compareArguments(arg1, arg2);

      expect(comparison).toBeLessThan(0); // arg1 has higher score, should come first (negative = a < b)
    });

    it("should calculate credibility if not present", () => {
      const arg1 = createTestArgument();
      delete arg1.credibilityScore;

      const arg2 = createTestArgument();
      delete arg2.credibilityScore;

      const comparison = ArgumentStructure.compareArguments(arg1, arg2);

      expect(typeof comparison).toBe("number");
    });

    it("should handle equal credibility scores", () => {
      const arg1 = createTestArgument();
      arg1.credibilityScore = 0.8;

      const arg2 = createTestArgument();
      arg2.credibilityScore = 0.8;

      const comparison = ArgumentStructure.compareArguments(arg1, arg2);

      expect(comparison).toBe(0);
    });
  });

  describe("extractKeyPoints", () => {
    it("should extract sentences from reasoning", () => {
      const argument = createTestArgument();
      argument.reasoning =
        "First key point about the topic. Second important point. Third crucial detail. Fourth significant point. Fifth relevant aspect.";

      const keyPoints = ArgumentStructure.extractKeyPoints(argument);

      expect(keyPoints.length).toBeGreaterThan(0);
      expect(keyPoints.length).toBeLessThanOrEqual(5);
    });

    it("should filter out very short sentences", () => {
      const argument = createTestArgument();
      argument.reasoning =
        "Short. This is a longer sentence that should be included.";

      const keyPoints = ArgumentStructure.extractKeyPoints(argument);

      expect(keyPoints.every((point) => point.length > 20)).toBe(true);
    });

    it("should return empty array for brief reasoning", () => {
      const argument = createTestArgument();
      argument.reasoning = "Too brief.";

      const keyPoints = ArgumentStructure.extractKeyPoints(argument);

      expect(keyPoints).toHaveLength(0);
    });
  });

  describe("summarizeArgument", () => {
    it("should include claim in summary", () => {
      const argument = createTestArgument();
      argument.claim = "Unique test claim";

      const summary = ArgumentStructure.summarizeArgument(argument);

      expect(summary).toContain("Unique test claim");
    });

    it("should include credibility score in summary", () => {
      const argument = createTestArgument();
      argument.credibilityScore = 0.85;

      const summary = ArgumentStructure.summarizeArgument(argument);

      expect(summary).toContain("0.85");
    });

    it("should include evidence count in summary", () => {
      const argument = createTestArgument();
      argument.evidence = [createTestEvidence(), createTestEvidence()];

      const summary = ArgumentStructure.summarizeArgument(argument);

      expect(summary).toContain("2 items");
    });

    it("should handle missing credibility score", () => {
      const argument = createTestArgument();
      delete argument.credibilityScore;

      const summary = ArgumentStructure.summarizeArgument(argument);

      expect(summary).toContain("N/A");
    });
  });

  describe("detectConflict", () => {
    it("should detect conflict when one claim has negation", () => {
      const arg1 = createTestArgument();
      arg1.claim = "The system works correctly and efficiently";

      const arg2 = createTestArgument();
      arg2.claim = "The system does not work correctly and efficiently";

      const hasConflict = ArgumentStructure.detectConflict(arg1, arg2);

      expect(hasConflict).toBe(true);
    });

    it("should not detect conflict for unrelated claims", () => {
      const arg1 = createTestArgument();
      arg1.claim = "The database is optimized";

      const arg2 = createTestArgument();
      arg2.claim = "The frontend needs improvement";

      const hasConflict = ArgumentStructure.detectConflict(arg1, arg2);

      expect(hasConflict).toBe(false);
    });

    it("should not detect conflict when both have negations", () => {
      const arg1 = createTestArgument();
      arg1.claim = "The system cannot handle this load";

      const arg2 = createTestArgument();
      arg2.claim = "The database cannot process requests";

      const hasConflict = ArgumentStructure.detectConflict(arg1, arg2);

      expect(hasConflict).toBe(false);
    });

    it("should require shared terms for conflict detection", () => {
      const arg1 = createTestArgument();
      arg1.claim = "Weather is not sunny today";

      const arg2 = createTestArgument();
      arg2.claim = "Traffic is heavy downtown";

      const hasConflict = ArgumentStructure.detectConflict(arg1, arg2);

      // Should not conflict as they share no significant terms
      expect(hasConflict).toBe(false);
    });
  });
});

// Test Helper Functions

function createTestArgument(): Argument {
  return {
    id: `arg-${Math.random().toString(36).substring(2, 9)}`,
    agentId: "agent-1",
    claim: "This is a test claim that is of appropriate length for validation",
    evidence: [createTestEvidence()],
    reasoning:
      "This is a detailed reasoning explanation that provides sufficient justification for the claim being made. It includes multiple sentences to ensure it meets the minimum length requirements for validation.",
    timestamp: new Date(),
    credibilityScore: 0.7,
  };
}

function createTestEvidence(
  credibilityScore = 0.8,
  verificationStatus: "verified" | "unverified" | "disputed" = "verified"
): Evidence {
  return {
    id: `ev-${Math.random().toString(36).substring(2, 9)}`,
    source: "https://example.com/source",
    content: "This is test evidence content that supports the claim.",
    credibilityScore,
    verificationStatus,
    timestamp: new Date(),
  };
}
