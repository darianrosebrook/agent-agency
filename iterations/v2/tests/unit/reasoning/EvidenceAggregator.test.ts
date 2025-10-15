/**
 * Unit tests for Evidence Aggregator
 *
 * @author @darianrosebrook
 */

import { EvidenceAggregator } from "@/reasoning/EvidenceAggregator";
import { Argument, Evidence } from "@/types/reasoning";

describe("EvidenceAggregator", () => {
  afterEach(() => {
    jest.clearAllMocks();
  });

  afterAll(() => {
    // Final cleanup
    jest.clearAllMocks();
  });
  describe("aggregateEvidence", () => {
    it("should aggregate evidence from multiple arguments", () => {
      const args = [
        createTestArgument([createTestEvidence(), createTestEvidence()]),
        createTestArgument([createTestEvidence()]),
      ];

      const aggregation = EvidenceAggregator.aggregateEvidence(args);

      expect(aggregation.totalEvidence).toBe(3);
      expect(aggregation.sources.length).toBeGreaterThan(0);
    });

    it("should calculate average credibility", () => {
      const args = [
        createTestArgument([createTestEvidence(0.8), createTestEvidence(0.6)]),
      ];

      const aggregation = EvidenceAggregator.aggregateEvidence(args);

      expect(aggregation.averageCredibility).toBe(0.7);
    });

    it("should count verified evidence", () => {
      const args = [
        createTestArgument([
          createTestEvidence(0.8, "verified"),
          createTestEvidence(0.6, "unverified"),
          createTestEvidence(0.9, "verified"),
        ]),
      ];

      const aggregation = EvidenceAggregator.aggregateEvidence(args);

      expect(aggregation.verifiedCount).toBe(2);
    });

    it("should count disputed evidence", () => {
      const args = [
        createTestArgument([
          createTestEvidence(0.8, "verified"),
          createTestEvidence(0.6, "disputed"),
        ]),
      ];

      const aggregation = EvidenceAggregator.aggregateEvidence(args);

      expect(aggregation.disputedCount).toBe(1);
    });

    it("should track unique sources", () => {
      const ev1 = createTestEvidence();
      ev1.source = "source1";
      const ev2 = createTestEvidence();
      ev2.source = "source2";
      const ev3 = createTestEvidence();
      ev3.source = "source1"; // duplicate

      const args = [createTestArgument([ev1, ev2, ev3])];

      const aggregation = EvidenceAggregator.aggregateEvidence(args);

      expect(aggregation.sources.length).toBe(2);
      expect(aggregation.sources).toContain("source1");
      expect(aggregation.sources).toContain("source2");
    });

    it("should generate summary", () => {
      const args = [createTestArgument([createTestEvidence()])];

      const aggregation = EvidenceAggregator.aggregateEvidence(args);

      expect(aggregation.summary).toContain("Total:");
      expect(aggregation.summary).toContain("Verified:");
      expect(aggregation.summary).toContain("Average credibility:");
    });

    it("should handle empty arguments array", () => {
      const aggregation = EvidenceAggregator.aggregateEvidence([]);

      expect(aggregation.totalEvidence).toBe(0);
      expect(aggregation.averageCredibility).toBe(0);
      expect(aggregation.sources).toEqual([]);
    });
  });

  describe("weighEvidence", () => {
    it("should assign base weight from credibility score", () => {
      const evidence = [createTestEvidence(0.8, "unverified")]; // Use unverified to avoid boost

      const weights = EvidenceAggregator.weighEvidence(evidence);

      expect(weights.get(evidence[0].id)).toBeCloseTo(0.8, 1);
    });

    it("should boost verified evidence weight", () => {
      const evidence = [createTestEvidence(0.8, "verified")];

      const weights = EvidenceAggregator.weighEvidence(evidence);

      expect(weights.get(evidence[0].id)).toBeGreaterThan(0.8);
    });

    it("should reduce disputed evidence weight", () => {
      const evidence = [createTestEvidence(0.8, "disputed")];

      const weights = EvidenceAggregator.weighEvidence(evidence);

      expect(weights.get(evidence[0].id)).toBeLessThan(0.8);
    });

    it("should cap weights at 1.0", () => {
      const evidence = [createTestEvidence(0.95, "verified")];

      const weights = EvidenceAggregator.weighEvidence(evidence);

      expect(weights.get(evidence[0].id)).toBeLessThanOrEqual(1.0);
    });

    it("should floor weights at 0.0", () => {
      const evidence = [createTestEvidence(0.1, "disputed")];

      const weights = EvidenceAggregator.weighEvidence(evidence);

      expect(weights.get(evidence[0].id)).toBeGreaterThanOrEqual(0.0);
    });
  });

  describe("detectConflicts", () => {
    it("should detect disputed evidence conflicts", () => {
      const ev1 = createTestEvidence(0.8, "disputed");
      const ev2 = createTestEvidence(0.7, "disputed");

      const conflicts = EvidenceAggregator.detectConflicts([ev1, ev2]);

      expect(conflicts.length).toBeGreaterThan(0);
      expect(conflicts[0].conflictType).toBe("disputed");
    });

    it("should return empty array for no conflicts", () => {
      const ev1 = createTestEvidence(0.8, "verified");
      const ev2 = createTestEvidence(0.7, "verified");

      const conflicts = EvidenceAggregator.detectConflicts([ev1, ev2]);

      expect(conflicts.length).toBe(0);
    });
  });

  describe("filterByCredibility", () => {
    it("should filter evidence by minimum credibility", () => {
      const evidence = [
        createTestEvidence(0.9),
        createTestEvidence(0.5),
        createTestEvidence(0.3),
      ];

      const filtered = EvidenceAggregator.filterByCredibility(evidence, 0.6);

      expect(filtered.length).toBe(1);
      expect(filtered[0].credibilityScore).toBe(0.9);
    });

    it("should include evidence at exact threshold", () => {
      const evidence = [createTestEvidence(0.7), createTestEvidence(0.6)];

      const filtered = EvidenceAggregator.filterByCredibility(evidence, 0.6);

      expect(filtered.length).toBe(2);
    });
  });

  describe("groupBySource", () => {
    it("should group evidence by source", () => {
      const ev1 = createTestEvidence();
      ev1.source = "source1";
      const ev2 = createTestEvidence();
      ev2.source = "source1";
      const ev3 = createTestEvidence();
      ev3.source = "source2";

      const groups = EvidenceAggregator.groupBySource([ev1, ev2, ev3]);

      expect(groups.size).toBe(2);
      expect(groups.get("source1")?.length).toBe(2);
      expect(groups.get("source2")?.length).toBe(1);
    });
  });

  describe("calculateSourceDiversity", () => {
    it("should return 1.0 for all unique sources", () => {
      const evidence = [
        { ...createTestEvidence(), source: "source1" },
        { ...createTestEvidence(), source: "source2" },
        { ...createTestEvidence(), source: "source3" },
      ];

      const diversity = EvidenceAggregator.calculateSourceDiversity(evidence);

      expect(diversity).toBe(1.0);
    });

    it("should return lower value for duplicate sources", () => {
      const evidence = [
        { ...createTestEvidence(), source: "source1" },
        { ...createTestEvidence(), source: "source1" },
        { ...createTestEvidence(), source: "source2" },
      ];

      const diversity = EvidenceAggregator.calculateSourceDiversity(evidence);

      expect(diversity).toBeLessThan(1.0);
      expect(diversity).toBeCloseTo(0.667, 2);
    });

    it("should return 0 for empty evidence array", () => {
      const diversity = EvidenceAggregator.calculateSourceDiversity([]);

      expect(diversity).toBe(0);
    });
  });

  describe("getMostCredibleEvidence", () => {
    it("should return top N evidence by credibility", () => {
      const evidence = [
        createTestEvidence(0.5),
        createTestEvidence(0.9),
        createTestEvidence(0.7),
        createTestEvidence(0.8),
      ];

      const top = EvidenceAggregator.getMostCredibleEvidence(evidence, 2);

      expect(top.length).toBe(2);
      expect(top[0].credibilityScore).toBe(0.9);
      expect(top[1].credibilityScore).toBe(0.8);
    });

    it("should default to top 5", () => {
      const evidence = Array(10)
        .fill(null)
        .map((_, i) => createTestEvidence(i / 10));

      const top = EvidenceAggregator.getMostCredibleEvidence(evidence);

      expect(top.length).toBe(5);
    });
  });

  describe("validateEvidenceQuality", () => {
    it("should validate high-quality evidence", () => {
      const evidence = [
        createTestEvidence(0.9, "verified"),
        createTestEvidence(0.85, "verified"),
        { ...createTestEvidence(0.8, "verified"), source: "source2" },
      ];

      const validation = EvidenceAggregator.validateEvidenceQuality(evidence);

      expect(validation.valid).toBe(true);
      expect(validation.issues).toHaveLength(0);
    });

    it("should detect no evidence provided", () => {
      const validation = EvidenceAggregator.validateEvidenceQuality([]);

      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain("No evidence provided");
    });

    it("should detect low credibility evidence", () => {
      const evidence = [
        createTestEvidence(0.2),
        createTestEvidence(0.3),
        createTestEvidence(0.1),
      ];

      const validation = EvidenceAggregator.validateEvidenceQuality(evidence);

      expect(validation.valid).toBe(false);
      expect(validation.issues.some((i) => i.includes("low credibility"))).toBe(
        true
      );
    });

    it("should detect too much disputed evidence", () => {
      const evidence = [
        createTestEvidence(0.8, "disputed"),
        createTestEvidence(0.7, "disputed"),
        createTestEvidence(0.6, "verified"),
      ];

      const validation = EvidenceAggregator.validateEvidenceQuality(evidence);

      expect(validation.valid).toBe(false);
      expect(validation.issues.some((i) => i.includes("disputed"))).toBe(true);
    });

    it("should detect low source diversity", () => {
      const evidence = [
        { ...createTestEvidence(), source: "same" },
        { ...createTestEvidence(), source: "same" },
        { ...createTestEvidence(), source: "same" },
        { ...createTestEvidence(), source: "same" }, // 4 items = 0.25 diversity
      ];

      const validation = EvidenceAggregator.validateEvidenceQuality(evidence);

      expect(validation.valid).toBe(false);
      expect(validation.issues.some((i) => i.includes("diversity"))).toBe(true);
    });
  });
});

// Test Helper Functions

function createTestArgument(evidence: Evidence[] = []): Argument {
  return {
    id: `arg-${Math.random().toString(36).substring(2, 9)}`,
    agentId: "agent-1",
    claim: "Test claim",
    evidence,
    reasoning: "Test reasoning",
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
    content: "Test evidence content",
    credibilityScore,
    verificationStatus,
    timestamp: new Date(),
  };
}
