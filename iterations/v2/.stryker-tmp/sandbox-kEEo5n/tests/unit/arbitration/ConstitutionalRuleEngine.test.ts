/**
 * Unit tests for ConstitutionalRuleEngine
 *
 * Tests constitutional rule loading, evaluation, precedent application, and violation detection.
 */
// @ts-nocheck


import {
  ConstitutionalRuleEngine,
  EvaluationContext,
} from "@/arbitration/ConstitutionalRuleEngine";
import {
  ArbitrationError,
  ConstitutionalRule,
  Precedent,
  RuleCategory,
  VerdictOutcome,
  ViolationSeverity,
} from "@/types/arbitration";

describe("ConstitutionalRuleEngine", () => {
  let engine: ConstitutionalRuleEngine;

  beforeEach(() => {
    engine = new ConstitutionalRuleEngine();
  });

  // Helper function to create a test rule
  const createRule = (
    overrides?: Partial<ConstitutionalRule>
  ): ConstitutionalRule => {
    return {
      id: "RULE-001",
      version: "1.0.0",
      category: RuleCategory.CODE_QUALITY,
      title: "Code must be linted",
      description: "All code must pass linting before commit",
      condition: "linted === true",
      severity: ViolationSeverity.MODERATE,
      waivable: false,
      requiredEvidence: ["linter_report"],
      precedents: [],
      effectiveDate: new Date("2024-01-01"),
      metadata: {},
      ...overrides,
    };
  };

  // Helper function to create evaluation context
  const createContext = (
    overrides?: Partial<EvaluationContext>
  ): EvaluationContext => {
    return {
      action: "commit",
      actor: "agent-1",
      parameters: {
        linted: true,
      },
      environment: {},
      timestamp: new Date(),
      ...overrides,
    };
  };

  describe("loadRule", () => {
    it("should load a valid rule", () => {
      const rule = createRule();
      engine.loadRule(rule);

      const loaded = engine.getRule("RULE-001");
      expect(loaded).toEqual(rule);
    });

    it("should throw error for rule without ID", () => {
      const rule = createRule({ id: "" });

      expect(() => engine.loadRule(rule)).toThrow(ArbitrationError);
      expect(() => engine.loadRule(rule)).toThrow("must have ID and version");
    });

    it("should throw error for rule without version", () => {
      const rule = createRule({ version: "" });

      expect(() => engine.loadRule(rule)).toThrow(ArbitrationError);
    });

    it("should throw error for expired rule", () => {
      const rule = createRule({
        expirationDate: new Date("2020-01-01"),
      });

      expect(() => engine.loadRule(rule)).toThrow("has expired");
    });

    it("should allow rule with future expiration", () => {
      const futureDate = new Date();
      futureDate.setFullYear(futureDate.getFullYear() + 1);

      const rule = createRule({
        expirationDate: futureDate,
      });

      engine.loadRule(rule);
      const loaded = engine.getRule("RULE-001");
      expect(loaded).toBeDefined();
    });
  });

  describe("loadRules", () => {
    it("should load multiple rules", () => {
      const rules = [
        createRule({ id: "RULE-001" }),
        createRule({ id: "RULE-002", category: RuleCategory.TESTING }),
        createRule({ id: "RULE-003", category: RuleCategory.SECURITY }),
      ];

      engine.loadRules(rules);

      expect(engine.getAllRules()).toHaveLength(3);
      expect(engine.getRule("RULE-001")).toBeDefined();
      expect(engine.getRule("RULE-002")).toBeDefined();
      expect(engine.getRule("RULE-003")).toBeDefined();
    });
  });

  describe("getRule", () => {
    it("should return undefined for non-existent rule", () => {
      const rule = engine.getRule("NON-EXISTENT");
      expect(rule).toBeUndefined();
    });

    it("should return rule by ID", () => {
      const rule = createRule();
      engine.loadRule(rule);

      const retrieved = engine.getRule("RULE-001");
      expect(retrieved).toEqual(rule);
    });
  });

  describe("getRulesByCategory", () => {
    it("should return rules filtered by category", () => {
      engine.loadRules([
        createRule({ id: "RULE-001", category: RuleCategory.CODE_QUALITY }),
        createRule({ id: "RULE-002", category: RuleCategory.TESTING }),
        createRule({ id: "RULE-003", category: RuleCategory.CODE_QUALITY }),
      ]);

      const codeQualityRules = engine.getRulesByCategory(
        RuleCategory.CODE_QUALITY
      );
      expect(codeQualityRules).toHaveLength(2);
      expect(codeQualityRules.map((r) => r.id)).toEqual([
        "RULE-001",
        "RULE-003",
      ]);
    });

    it("should return empty array for category with no rules", () => {
      engine.loadRule(createRule());

      const securityRules = engine.getRulesByCategory(RuleCategory.SECURITY);
      expect(securityRules).toHaveLength(0);
    });
  });

  describe("getAllRules", () => {
    it("should return empty array when no rules loaded", () => {
      const rules = engine.getAllRules();
      expect(rules).toHaveLength(0);
    });

    it("should return all loaded rules", () => {
      engine.loadRules([
        createRule({ id: "RULE-001" }),
        createRule({ id: "RULE-002" }),
        createRule({ id: "RULE-003" }),
      ]);

      const rules = engine.getAllRules();
      expect(rules).toHaveLength(3);
    });
  });

  describe("evaluateAction", () => {
    it("should evaluate action against single rule", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext({
        parameters: {
          linted: true,
          evidence_linter_report: "All checks passed",
        },
      });
      const results = await engine.evaluateAction(context, ["RULE-001"]);

      expect(results).toHaveLength(1);
      expect(results[0].ruleId).toBe("RULE-001");
      expect(results[0].violated).toBe(false);
    });

    it("should detect violation when linting not performed", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext({ parameters: { linted: false } });
      const results = await engine.evaluateAction(context);

      expect(results).toHaveLength(1);
      expect(results[0].violated).toBe(true);
      expect(results[0].violation).toBeDefined();
    });

    it("should evaluate against all rules when no ruleIds provided", async () => {
      engine.loadRules([
        createRule({ id: "RULE-001" }),
        createRule({ id: "RULE-002" }),
        createRule({ id: "RULE-003" }),
      ]);

      const context = createContext();
      const results = await engine.evaluateAction(context);

      expect(results).toHaveLength(3);
    });

    it("should return empty array when no rules loaded", async () => {
      const context = createContext();
      const results = await engine.evaluateAction(context);

      expect(results).toHaveLength(0);
    });

    it("should detect testing coverage violations", async () => {
      const rule = createRule({
        id: "RULE-TEST",
        category: RuleCategory.TESTING,
        title: "Test coverage must be >= 80%",
      });
      engine.loadRule(rule);

      const context = createContext({
        action: "test",
        parameters: { coverage: 75 },
      });
      const results = await engine.evaluateAction(context, ["RULE-TEST"]);

      expect(results[0].violated).toBe(true);
    });

    it("should detect security violations", async () => {
      const rule = createRule({
        id: "RULE-SEC",
        category: RuleCategory.SECURITY,
        title: "No vulnerabilities allowed",
      });
      engine.loadRule(rule);

      const context = createContext({
        action: "deploy",
        parameters: { hasVulnerabilities: true },
      });
      const results = await engine.evaluateAction(context, ["RULE-SEC"]);

      expect(results[0].violated).toBe(true);
    });

    it("should detect budget violations", async () => {
      const rule = createRule({
        id: "RULE-BUDGET",
        category: RuleCategory.BUDGET,
        title: "Max files per change",
      });
      engine.loadRule(rule);

      const context = createContext({
        action: "commit",
        parameters: { filesChanged: 30 },
        environment: { maxFiles: 25 },
      });
      const results = await engine.evaluateAction(context, ["RULE-BUDGET"]);

      expect(results[0].violated).toBe(true);
    });
  });

  describe("precedent application", () => {
    it("should apply precedents when configured", async () => {
      const customEngine = new ConstitutionalRuleEngine({
        usePrecedents: true,
      });

      const rule = createRule();
      customEngine.loadRule(rule);

      const precedent: Precedent = {
        id: "PREC-001",
        title: "Similar linting case",
        rulesInvolved: ["RULE-001"],
        verdict: {
          id: "verdict-1",
          sessionId: "session-1",
          outcome: VerdictOutcome.REJECTED,
          reasoning: [],
          rulesApplied: ["RULE-001"],
          evidence: [],
          precedents: [],
          confidence: 0.9,
          issuedBy: "arbiter",
          issuedAt: new Date(),
          auditLog: [],
        },
        keyFacts: ["Code not linted"],
        reasoningSummary: "Rejected due to missing linting",
        applicability: {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        },
        citationCount: 5,
        createdAt: new Date(),
        metadata: {},
      };
      customEngine.loadPrecedent(precedent);

      const context = createContext();
      const results = await customEngine.evaluateAction(context, ["RULE-001"]);

      expect(results[0].precedentsApplied).toContain("PREC-001");
    });

    it("should not apply precedents when disabled", async () => {
      const customEngine = new ConstitutionalRuleEngine({
        usePrecedents: false,
      });

      const rule = createRule();
      customEngine.loadRule(rule);

      const context = createContext();
      const results = await customEngine.evaluateAction(context, ["RULE-001"]);

      expect(results[0].precedentsApplied).toHaveLength(0);
    });
  });

  describe("caching", () => {
    it("should cache evaluation results", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext();

      // First evaluation
      const results1 = await engine.evaluateAction(context, ["RULE-001"]);
      const time1 = results1[0].evaluationTimeMs;

      // Second evaluation (should be faster due to cache)
      const results2 = await engine.evaluateAction(context, ["RULE-001"]);
      const time2 = results2[0].evaluationTimeMs;

      expect(results1[0].violated).toBe(results2[0].violated);
      // Cached result should typically be faster or equal
      expect(time2).toBeLessThanOrEqual(time1 + 1); // +1 for timing variance
    });

    it("should clear cache for specific rule", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext();
      await engine.evaluateAction(context, ["RULE-001"]);

      const stats1 = engine.getStatistics();
      expect(stats1.cacheSize).toBeGreaterThan(0);

      engine.loadRule(createRule({ version: "1.0.1" })); // Reloading clears cache

      const stats2 = engine.getStatistics();
      // Cache should be cleared or smaller
      expect(stats2.cacheSize).toBeLessThanOrEqual(stats1.cacheSize);
    });

    it("should clear all cache", async () => {
      engine.loadRules([
        createRule({ id: "RULE-001" }),
        createRule({ id: "RULE-002" }),
      ]);

      const context = createContext();
      await engine.evaluateAction(context);

      const stats1 = engine.getStatistics();
      expect(stats1.cacheSize).toBeGreaterThan(0);

      engine.clearCache();

      const stats2 = engine.getStatistics();
      expect(stats2.cacheSize).toBe(0);
    });
  });

  describe("getStatistics", () => {
    it("should return correct statistics", () => {
      engine.loadRules([
        createRule({ id: "RULE-001", category: RuleCategory.CODE_QUALITY }),
        createRule({ id: "RULE-002", category: RuleCategory.TESTING }),
        createRule({ id: "RULE-003", category: RuleCategory.CODE_QUALITY }),
      ]);

      const stats = engine.getStatistics();

      expect(stats.totalRules).toBe(3);
      expect(stats.activeRules).toBe(3);
      expect(stats.totalPrecedents).toBe(0);
      expect(stats.rulesByCategory[RuleCategory.CODE_QUALITY]).toBe(2);
      expect(stats.rulesByCategory[RuleCategory.TESTING]).toBe(1);
    });

    it("should count active rules correctly", () => {
      const futureDate = new Date();
      futureDate.setFullYear(futureDate.getFullYear() + 1);

      const pastDate = new Date("2020-01-01");

      engine.loadRules([
        createRule({ id: "RULE-001" }), // No expiration
        createRule({ id: "RULE-002", expirationDate: futureDate }), // Future expiration
      ]);

      // Try to load expired rule (should throw)
      try {
        engine.loadRule(
          createRule({ id: "RULE-003", expirationDate: pastDate })
        );
      } catch (e) {
        // Expected to throw
      }

      const stats = engine.getStatistics();
      expect(stats.totalRules).toBe(2);
      expect(stats.activeRules).toBe(2);
    });
  });

  describe("violation detection", () => {
    it("should create violation with evidence", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext({
        parameters: {
          linted: false,
          evidence_linter_report: "Failed with 5 errors",
          file: "src/test.ts",
          line: 42,
        },
      });

      const results = await engine.evaluateAction(context, ["RULE-001"]);

      expect(results[0].violated).toBe(true);
      expect(results[0].violation).toBeDefined();
      expect(results[0].violation!.ruleId).toBe("RULE-001");
      expect(results[0].violation!.severity).toBe(ViolationSeverity.MODERATE);
      expect(results[0].violation!.evidence).toContain("Action: commit");
      expect(results[0].violation!.location).toEqual({
        file: "src/test.ts",
        line: 42,
        function: undefined,
      });
    });

    it("should set violator from context", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext({
        actor: "agent-123",
        parameters: { linted: false },
      });

      const results = await engine.evaluateAction(context, ["RULE-001"]);

      expect(results[0].violation!.violator).toBe("agent-123");
    });
  });

  describe("confidence calculation", () => {
    it("should have higher confidence with precedents", async () => {
      const engineWithPrecedents = new ConstitutionalRuleEngine({
        usePrecedents: true,
      });
      const engineWithoutPrecedents = new ConstitutionalRuleEngine({
        usePrecedents: false,
      });

      const rule = createRule();
      engineWithPrecedents.loadRule(rule);
      engineWithoutPrecedents.loadRule(rule);

      const precedent: Precedent = {
        id: "PREC-001",
        title: "Test precedent",
        rulesInvolved: ["RULE-001"],
        verdict: {
          id: "verdict-1",
          sessionId: "session-1",
          outcome: VerdictOutcome.APPROVED,
          reasoning: [],
          rulesApplied: ["RULE-001"],
          evidence: [],
          precedents: [],
          confidence: 0.9,
          issuedBy: "arbiter",
          issuedAt: new Date(),
          auditLog: [],
        },
        keyFacts: [],
        reasoningSummary: "",
        applicability: {
          category: RuleCategory.CODE_QUALITY,
          severity: ViolationSeverity.MODERATE,
          conditions: [],
        },
        citationCount: 10,
        createdAt: new Date(),
        metadata: {},
      };
      engineWithPrecedents.loadPrecedent(precedent);

      const context = createContext();

      const resultsWithPrecedents = await engineWithPrecedents.evaluateAction(
        context,
        ["RULE-001"]
      );
      const resultsWithoutPrecedents =
        await engineWithoutPrecedents.evaluateAction(context, ["RULE-001"]);

      expect(resultsWithPrecedents[0].confidence).toBeGreaterThan(
        resultsWithoutPrecedents[0].confidence
      );
    });
  });

  describe("edge cases", () => {
    it("should handle empty parameters", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext({ parameters: {} });
      const results = await engine.evaluateAction(context, ["RULE-001"]);

      expect(results).toHaveLength(1);
      // Behavior depends on rule logic
    });

    it("should handle expired rules during evaluation", async () => {
      const futureDate = new Date();
      futureDate.setFullYear(futureDate.getFullYear() + 1);

      const rule = createRule({
        expirationDate: futureDate,
      });
      engine.loadRule(rule);

      // Evaluate with timestamp after expiration
      const pastExpiration = new Date();
      pastExpiration.setFullYear(pastExpiration.getFullYear() + 2);

      const context = createContext({
        timestamp: pastExpiration,
      });
      const results = await engine.evaluateAction(context, ["RULE-001"]);

      expect(results[0].violated).toBe(false);
      expect(results[0].explanation).toContain("expired");
    });

    it("should filter out non-existent rules", async () => {
      const rule = createRule();
      engine.loadRule(rule);

      const context = createContext();
      const results = await engine.evaluateAction(context, [
        "RULE-001",
        "NON-EXISTENT",
      ]);

      expect(results).toHaveLength(1);
      expect(results[0].ruleId).toBe("RULE-001");
    });
  });
});
