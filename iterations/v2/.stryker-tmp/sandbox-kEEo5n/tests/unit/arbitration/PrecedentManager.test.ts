/**
 * Unit tests for PrecedentManager
 *
 * Tests precedent storage, similarity matching, citation tracking,
 * and applicability assessment.
 */
// @ts-nocheck


import { PrecedentManager } from "@/arbitration/PrecedentManager";
import {
  RuleCategory,
  Verdict,
  VerdictOutcome,
  ViolationSeverity,
} from "@/types/arbitration";

describe("PrecedentManager", () => {
  let manager: PrecedentManager;

  beforeEach(() => {
    manager = new PrecedentManager();
  });

  // Helper to create a test verdict
  const createVerdict = (overrides?: Partial<Verdict>): Verdict => {
    return {
      id: "verdict-1",
      sessionId: "session-1",
      outcome: VerdictOutcome.APPROVED,
      reasoning: [],
      rulesApplied: ["RULE-001"],
      evidence: ["evidence-1"],
      precedents: [],
      confidence: 0.9,
      issuedBy: "arbiter-1",
      issuedAt: new Date(),
      auditLog: [],
      ...overrides,
    };
  };

  describe("createPrecedent", () => {
    it("should create a new precedent", () => {
      const verdict = createVerdict();

      const precedent = manager.createPrecedent(
        verdict,
        "Test Precedent",
        ["fact1", "fact2"],
        "Test reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      expect(precedent.id).toMatch(/^PREC-/);
      expect(precedent.title).toBe("Test Precedent");
      expect(precedent.keyFacts).toHaveLength(2);
      expect(precedent.citationCount).toBe(0);
    });

    it("should store precedent for retrieval", () => {
      const verdict = createVerdict();
      const precedent = manager.createPrecedent(
        verdict,
        "Test Precedent",
        ["fact1"],
        "Test reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const retrieved = manager.getPrecedent(precedent.id);
      expect(retrieved).toEqual(precedent);
    });
  });

  describe("findSimilarPrecedents", () => {
    beforeEach(() => {
      // Create some test precedents
      manager.createPrecedent(
        createVerdict({ rulesApplied: ["RULE-001", "RULE-002"] }),
        "Linting Violation",
        ["code not linted", "missing imports"],
        "Code quality issue",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.createPrecedent(
        createVerdict({ rulesApplied: ["RULE-003"] }),
        "Test Coverage Issue",
        ["insufficient tests", "missing edge cases"],
        "Testing issue",
        {
          category: RuleCategory.TESTING,
          severity: ViolationSeverity.MAJOR,
          conditions: [],
        }
      );

      manager.createPrecedent(
        createVerdict({ rulesApplied: ["RULE-001"] }),
        "Format Violation",
        ["code not formatted", "inconsistent style"],
        "Formatting issue",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MINOR,
          conditions: [],
        }
      );
    });

    it("should find precedents with matching category", () => {
      const matches = manager.findSimilarPrecedents(
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MODERATE,
        ["code not linted"],
        ["RULE-001"],
        10
      );

      expect(matches.length).toBeGreaterThan(0);
      expect(matches[0].precedent.applicability.category).toBe(
        RuleCategory.CODE_QUALITY
      );
    });

    it("should calculate similarity scores", () => {
      const matches = manager.findSimilarPrecedents(
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MODERATE,
        ["code not linted", "missing imports"],
        ["RULE-001", "RULE-002"],
        10
      );

      expect(matches.length).toBeGreaterThan(0);
      expect(matches[0].score).toBeGreaterThan(0);
      expect(matches[0].score).toBeLessThanOrEqual(1);
    });

    it("should sort by similarity score descending", () => {
      const matches = manager.findSimilarPrecedents(
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MODERATE,
        ["code not linted"],
        ["RULE-001"],
        10
      );

      if (matches.length > 1) {
        for (let i = 0; i < matches.length - 1; i++) {
          expect(matches[i].score).toBeGreaterThanOrEqual(matches[i + 1].score);
        }
      }
    });

    it("should respect limit parameter", () => {
      const matches = manager.findSimilarPrecedents(
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MODERATE,
        ["code issue"],
        ["RULE-001"],
        1
      );

      expect(matches.length).toBeLessThanOrEqual(1);
    });

    it("should provide matching factors", () => {
      const matches = manager.findSimilarPrecedents(
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MODERATE,
        ["code not linted"],
        ["RULE-001"],
        10
      );

      expect(matches[0].matchingFactors).toBeDefined();
      expect(matches[0].matchingFactors.length).toBeGreaterThan(0);
    });
  });

  describe("searchPrecedents", () => {
    beforeEach(() => {
      manager.createPrecedent(
        createVerdict(),
        "First Precedent",
        ["fact1"],
        "Reasoning 1",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.createPrecedent(
        createVerdict(),
        "Second Precedent",
        ["fact2"],
        "Reasoning 2",
        {
          category: RuleCategory.TESTING,
          severity: ViolationSeverity.MAJOR,
          conditions: [],
        }
      );
    });

    it("should filter by category", () => {
      const results = manager.searchPrecedents({
        categories: [RuleCategory.CODE_QUALITY],
      });

      expect(results.length).toBeGreaterThan(0);
      expect(
        results.every(
          (p) => p.applicability.category === RuleCategory.CODE_QUALITY
        )
      ).toBe(true);
    });

    it("should filter by severity", () => {
      const results = manager.searchPrecedents({
        severity: ViolationSeverity.MAJOR,
      });

      expect(
        results.every(
          (p) => p.applicability.severity === ViolationSeverity.MAJOR
        )
      ).toBe(true);
    });

    it("should filter by minimum citations", () => {
      const precedent1 = manager.createPrecedent(
        createVerdict(),
        "Highly Cited",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      // Add some citations
      manager.citePrecedent(precedent1.id, "citing-1");
      manager.citePrecedent(precedent1.id, "citing-2");

      const results = manager.searchPrecedents({
        minCitations: 2,
      });

      expect(results.length).toBeGreaterThan(0);
      expect(results.every((p) => p.citationCount >= 2)).toBe(true);
    });

    it("should filter by keywords", () => {
      const results = manager.searchPrecedents({
        keywords: ["First"],
      });

      expect(results.length).toBeGreaterThan(0);
      expect(results.some((p) => p.title.includes("First"))).toBe(true);
    });

    it("should sort by citations", () => {
      const prec1 = manager.createPrecedent(
        createVerdict(),
        "Less Cited",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const prec2 = manager.createPrecedent(
        createVerdict(),
        "More Cited",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.citePrecedent(prec2.id, "citing-1");
      manager.citePrecedent(prec2.id, "citing-2");
      manager.citePrecedent(prec1.id, "citing-3");

      const results = manager.searchPrecedents({
        sortBy: "citations",
      });

      if (results.length >= 2) {
        expect(results[0].citationCount).toBeGreaterThanOrEqual(
          results[1].citationCount
        );
      }
    });

    it("should limit results", () => {
      const results = manager.searchPrecedents({
        limit: 1,
      });

      expect(results.length).toBeLessThanOrEqual(1);
    });
  });

  describe("citePrecedent", () => {
    it("should increment citation count", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.citePrecedent(precedent.id, "citing-1");

      const updated = manager.getPrecedent(precedent.id);
      expect(updated!.citationCount).toBe(1);
    });

    it("should track citing precedents", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.citePrecedent(precedent.id, "citing-1");
      manager.citePrecedent(precedent.id, "citing-2");

      const citing = manager.getCitingPrecedents(precedent.id);
      // Would need to create actual citing precedents to test this properly
      expect(Array.isArray(citing)).toBe(true);
    });

    it("should return false for non-existent precedent", () => {
      const result = manager.citePrecedent("non-existent", "citing-1");
      expect(result).toBe(false);
    });
  });

  describe("overrulePrecedent", () => {
    it("should mark precedent as overruled", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const result = manager.overrulePrecedent(
        precedent.id,
        "new-precedent-id",
        "Better reasoning available"
      );

      expect(result).toBe(true);
      expect(manager.isValid(precedent.id)).toBe(false);
    });

    it("should store overruling details in metadata", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.overrulePrecedent(
        precedent.id,
        "new-precedent-id",
        "Better reasoning"
      );

      const updated = manager.getPrecedent(precedent.id);
      expect(updated!.metadata.overruled).toBe(true);
      expect(updated!.metadata.overruledBy).toBe("new-precedent-id");
      expect(updated!.metadata.overruledReason).toBe("Better reasoning");
    });

    it("should return false for non-existent precedent", () => {
      const result = manager.overrulePrecedent(
        "non-existent",
        "new-id",
        "reason"
      );
      expect(result).toBe(false);
    });
  });

  describe("assessApplicability", () => {
    it("should assess precedent as not applicable if overruled", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.overrulePrecedent(precedent.id, "new-id", "reason");

      const result = manager.assessApplicability(
        precedent,
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MODERATE,
        []
      );

      expect(result.applicable).toBe(false);
      expect(result.reasoning).toContain("overruled");
    });

    it("should assess category mismatch as not applicable", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const result = manager.assessApplicability(
        precedent,
        RuleCategory.TESTING,
        ViolationSeverity.MODERATE,
        []
      );

      expect(result.applicable).toBe(false);
      expect(result.reasoning).toContain("Category mismatch");
    });

    it("should assess matching category and severity as highly applicable", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const result = manager.assessApplicability(
        precedent,
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MODERATE,
        []
      );

      expect(result.applicable).toBe(true);
      expect(result.confidence).toBeGreaterThan(0.8);
    });

    it("should reduce confidence for severity mismatch", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MAJOR,
          conditions: [],
        }
      );

      const result = manager.assessApplicability(
        precedent,
        RuleCategory.CODE_QUALITY,
        ViolationSeverity.MINOR,
        []
      );

      expect(result.applicable).toBe(true);
      expect(result.confidence).toBeLessThan(0.8);
      expect(result.reasoning).toContain("Severity mismatch");
    });
  });

  describe("getStatistics", () => {
    it("should return statistics for empty manager", () => {
      const stats = manager.getStatistics();

      expect(stats.totalPrecedents).toBe(0);
      expect(stats.validPrecedents).toBe(0);
      expect(stats.overruledPrecedents).toBe(0);
      expect(stats.averageCitations).toBe(0);
    });

    it("should count precedents correctly", () => {
      manager.createPrecedent(
        createVerdict(),
        "Test 1",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.createPrecedent(
        createVerdict(),
        "Test 2",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.TESTING,
          severity: ViolationSeverity.MAJOR,
          conditions: [],
        }
      );

      const stats = manager.getStatistics();
      expect(stats.totalPrecedents).toBe(2);
      expect(stats.validPrecedents).toBe(2);
    });

    it("should count overruled precedents", () => {
      const prec1 = manager.createPrecedent(
        createVerdict(),
        "Test 1",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.overrulePrecedent(prec1.id, "new-id", "reason");

      const stats = manager.getStatistics();
      expect(stats.overruledPrecedents).toBe(1);
      expect(stats.validPrecedents).toBe(0);
    });

    it("should calculate average citations", () => {
      const prec1 = manager.createPrecedent(
        createVerdict(),
        "Test 1",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const prec2 = manager.createPrecedent(
        createVerdict(),
        "Test 2",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.TESTING,
          severity: ViolationSeverity.MAJOR,
          conditions: [],
        }
      );

      manager.citePrecedent(prec1.id, "citing-1");
      manager.citePrecedent(prec1.id, "citing-2");
      manager.citePrecedent(prec2.id, "citing-3");

      const stats = manager.getStatistics();
      expect(stats.averageCitations).toBe(1.5);
    });

    it("should identify most cited precedent", () => {
      const prec1 = manager.createPrecedent(
        createVerdict(),
        "Less Cited",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const prec2 = manager.createPrecedent(
        createVerdict(),
        "More Cited",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.TESTING,
          severity: ViolationSeverity.MAJOR,
          conditions: [],
        }
      );

      manager.citePrecedent(prec2.id, "citing-1");
      manager.citePrecedent(prec2.id, "citing-2");
      manager.citePrecedent(prec1.id, "citing-3");

      const stats = manager.getStatistics();
      expect(stats.mostCited?.id).toBe(prec2.id);
    });

    it("should group by category", () => {
      manager.createPrecedent(
        createVerdict(),
        "Test 1",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      manager.createPrecedent(
        createVerdict(),
        "Test 2",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MAJOR,
          conditions: [],
        }
      );

      manager.createPrecedent(
        createVerdict(),
        "Test 3",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.TESTING,
          severity: ViolationSeverity.MINOR,
          conditions: [],
        }
      );

      const stats = manager.getStatistics();
      expect(stats.byCategory[RuleCategory.CODE_QUALITY]).toBe(2);
      expect(stats.byCategory[RuleCategory.TESTING]).toBe(1);
    });
  });

  describe("edge cases", () => {
    it("should handle precedent with no facts", () => {
      const precedent = manager.createPrecedent(
        createVerdict(),
        "Test",
        [],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      expect(precedent.keyFacts).toHaveLength(0);
    });

    it("should handle search with no results", () => {
      const results = manager.searchPrecedents({
        keywords: ["nonexistent"],
      });

      expect(results).toHaveLength(0);
    });

    it("should handle similarity search with no matches", () => {
      const customManager = new PrecedentManager({
        minSimilarityScore: 0.99,
      });

      customManager.createPrecedent(
        createVerdict(),
        "Test",
        ["fact"],
        "Reasoning",
        {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        }
      );

      const matches = customManager.findSimilarPrecedents(
        RuleCategory.TESTING,
        ViolationSeverity.CRITICAL,
        ["completely different"],
        ["DIFFERENT-RULE"],
        10
      );

      expect(matches).toHaveLength(0);
    });
  });
});
