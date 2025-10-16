/**
 * Unit tests for VerdictGenerator
 *
 * Tests verdict generation, reasoning chain construction, outcome determination,
 * and confidence calculation.
 */
// @ts-nocheck


import { VerdictGenerator } from "@/arbitration/VerdictGenerator";
import {
  ArbitrationError,
  ArbitrationSession,
  ArbitrationState,
  ConstitutionalRule,
  RuleCategory,
  VerdictOutcome,
  ViolationSeverity,
} from "@/types/arbitration";

describe("VerdictGenerator", () => {
  let generator: VerdictGenerator;

  beforeEach(() => {
    generator = new VerdictGenerator();
  });

  // Helper to create a test session
  const createSession = (
    overrides?: Partial<ArbitrationSession>
  ): ArbitrationSession => {
    const rule: ConstitutionalRule = {
      id: "RULE-001",
      version: "1.0.0",
      category: RuleCategory.CODE_QUALITY,
      title: "Code must be linted",
      description: "All code must pass linting",
      condition: "linted === true",
      severity: ViolationSeverity.MODERATE,
      waivable: false,
      requiredEvidence: ["linter_report"],
      precedents: [],
      effectiveDate: new Date(),
      metadata: {},
    };

    return {
      id: "session-1",
      state: ArbitrationState.VERDICT_GENERATION,
      violation: {
        id: "violation-1",
        ruleId: "RULE-001",
        severity: ViolationSeverity.MODERATE,
        description: "Code not linted",
        evidence: ["commit-hash-123", "linter-output.txt"],
        detectedAt: new Date(),
        violator: "agent-1",
        context: {},
      },
      rulesEvaluated: [rule],
      evidence: ["commit-hash-123", "linter-output.txt"],
      participants: ["agent-1"],
      precedents: [],
      startTime: new Date(),
      metadata: {},
      ...overrides,
    };
  };

  describe("generateVerdict", () => {
    it("should generate a complete verdict", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict).toBeDefined();
      expect(result.verdict.id).toMatch(/^VERDICT-/);
      expect(result.verdict.sessionId).toBe("session-1");
      expect(result.verdict.issuedBy).toBe("arbiter-1");
      expect(result.verdict.reasoning.length).toBeGreaterThan(0);
    });

    it("should include complete reasoning chain", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.reasoning.length).toBeGreaterThanOrEqual(3);
      expect(result.verdict.reasoning[0].step).toBe(1);
      expect(result.verdict.reasoning[0].description).toContain("violation");
    });

    it("should reference applied rules", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.rulesApplied).toContain("RULE-001");
    });

    it("should include evidence in verdict", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.evidence).toHaveLength(2);
      expect(result.verdict.evidence).toContain("commit-hash-123");
    });

    it("should create audit log entry", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.auditLog).toHaveLength(1);
      expect(result.verdict.auditLog[0].action).toBe("verdict_generated");
      expect(result.verdict.auditLog[0].actor).toBe("arbiter-1");
    });

    it("should track generation time", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.generationTimeMs).toBeGreaterThanOrEqual(0);
      expect(result.generationTimeMs).toBeLessThan(1000);
    });
  });

  describe("outcome determination", () => {
    it("should reject critical violations", async () => {
      const session = createSession({
        violation: {
          id: "violation-1",
          ruleId: "RULE-001",
          severity: ViolationSeverity.CRITICAL,
          description: "Critical violation",
          evidence: ["evidence-1"],
          detectedAt: new Date(),
          context: {},
        },
      });

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.outcome).toBe(VerdictOutcome.REJECTED);
    });

    it("should approve with strong evidence and confidence", async () => {
      const session = createSession({
        evidence: ["evidence-1", "evidence-2", "evidence-3"],
        violation: {
          id: "violation-1",
          ruleId: "RULE-001",
          severity: ViolationSeverity.MINOR,
          description: "Minor violation",
          evidence: ["evidence-1", "evidence-2", "evidence-3"],
          detectedAt: new Date(),
          context: {},
        },
      });

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.outcome).toBe(VerdictOutcome.APPROVED);
    });

    it("should create conditional verdict for low confidence", async () => {
      const customGenerator = new VerdictGenerator({
        minConfidenceForApproval: 0.95,
      });

      const session = createSession();
      const result = await customGenerator.generateVerdict(
        session,
        "arbiter-1"
      );

      expect(result.verdict.outcome).toBe(VerdictOutcome.CONDITIONAL);
      expect(result.verdict.conditions).toBeDefined();
    });

    it("should mark as waived when waiver request exists", async () => {
      const session = createSession({
        waiverRequest: {
          id: "waiver-1",
          ruleId: "RULE-001",
          requestedBy: "agent-1",
          justification: "Emergency fix required",
          evidence: ["evidence-1"],
          requestedDuration: 86400000,
          requestedAt: new Date(),
          context: {},
        },
      });

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.outcome).toBe(VerdictOutcome.WAIVED);
    });
  });

  describe("reasoning chain", () => {
    it("should include violation assessment step", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      const step1 = result.verdict.reasoning.find((s) => s.step === 1);
      expect(step1).toBeDefined();
      expect(step1!.description).toContain("constitutional violation");
    });

    it("should include rule application steps", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      const ruleSteps = result.verdict.reasoning.filter((s) =>
        s.description.includes("constitutional rule")
      );
      expect(ruleSteps.length).toBeGreaterThan(0);
    });

    it("should include precedent analysis when precedents exist", async () => {
      const session = createSession({
        precedents: [
          {
            id: "PREC-001",
            title: "Similar case",
            rulesInvolved: ["RULE-001"],
            verdict: {
              id: "verdict-1",
              sessionId: "session-1",
              outcome: VerdictOutcome.APPROVED,
              reasoning: [],
              rulesApplied: [],
              evidence: [],
              precedents: [],
              confidence: 0.9,
              issuedBy: "arbiter",
              issuedAt: new Date(),
              auditLog: [],
            },
            keyFacts: [],
            reasoningSummary: "Test precedent",
            applicability: {
              category: RuleCategory.CODE_QUALITY,
              severity: ViolationSeverity.MODERATE,
              conditions: [],
            },
            citationCount: 5,
            createdAt: new Date(),
            metadata: {},
          },
        ],
      });

      const result = await generator.generateVerdict(session, "arbiter-1");

      const precedentSteps = result.verdict.reasoning.filter((s) =>
        s.description.includes("precedent")
      );
      expect(precedentSteps.length).toBeGreaterThan(0);
      expect(result.verdict.precedents).toContain("PREC-001");
    });

    it("should include evidence evaluation step", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      const evidenceSteps = result.verdict.reasoning.filter((s) =>
        s.description.includes("evidence")
      );
      expect(evidenceSteps.length).toBeGreaterThan(0);
    });

    it("should include final assessment", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      const lastStep =
        result.verdict.reasoning[result.verdict.reasoning.length - 1];
      expect(lastStep.description).toContain("Final assessment");
    });
  });

  describe("confidence calculation", () => {
    it("should calculate base confidence from reasoning steps", async () => {
      const session = createSession();

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.confidence).toBeGreaterThan(0);
      expect(result.verdict.confidence).toBeLessThanOrEqual(1);
    });

    it("should boost confidence with precedents", async () => {
      const sessionWithoutPrecedents = createSession();
      const sessionWithPrecedents = createSession({
        precedents: [
          {
            id: "PREC-001",
            title: "Similar case",
            rulesInvolved: ["RULE-001"],
            verdict: {} as any,
            keyFacts: [],
            reasoningSummary: "Test",
            applicability: {
              category: RuleCategory.CODE_QUALITY,
              severity: ViolationSeverity.MODERATE,
              conditions: [],
            },
            citationCount: 5,
            createdAt: new Date(),
            metadata: {},
          },
        ],
      });

      const result1 = await generator.generateVerdict(
        sessionWithoutPrecedents,
        "arbiter-1"
      );
      const result2 = await generator.generateVerdict(
        sessionWithPrecedents,
        "arbiter-1"
      );

      expect(result2.verdict.confidence).toBeGreaterThan(
        result1.verdict.confidence
      );
    });

    it("should boost confidence with strong evidence", async () => {
      const weakSession = createSession({
        evidence: ["evidence-1"],
      });
      const strongSession = createSession({
        evidence: ["evidence-1", "evidence-2", "evidence-3", "evidence-4"],
      });

      const result1 = await generator.generateVerdict(weakSession, "arbiter-1");
      const result2 = await generator.generateVerdict(
        strongSession,
        "arbiter-1"
      );

      expect(result2.verdict.confidence).toBeGreaterThan(
        result1.verdict.confidence
      );
    });

    it("should reduce confidence for waiver requests", async () => {
      const normalSession = createSession();
      const waiverSession = createSession({
        waiverRequest: {
          id: "waiver-1",
          ruleId: "RULE-001",
          requestedBy: "agent-1",
          justification: "Test",
          evidence: [],
          requestedDuration: 86400000,
          requestedAt: new Date(),
          context: {},
        },
      });

      const result1 = await generator.generateVerdict(
        normalSession,
        "arbiter-1"
      );
      const result2 = await generator.generateVerdict(
        waiverSession,
        "arbiter-1"
      );

      expect(result2.verdict.confidence).toBeLessThan(
        result1.verdict.confidence
      );
    });
  });

  describe("conditional verdicts", () => {
    it("should generate conditions for conditional verdict", async () => {
      const customGenerator = new VerdictGenerator({
        minConfidenceForApproval: 0.95,
      });

      const session = createSession();
      const result = await customGenerator.generateVerdict(
        session,
        "arbiter-1"
      );

      if (result.verdict.outcome === VerdictOutcome.CONDITIONAL) {
        expect(result.verdict.conditions).toBeDefined();
        expect(result.verdict.conditions!.length).toBeGreaterThan(0);
      }
    });

    it("should include severity-based conditions", async () => {
      const customGenerator = new VerdictGenerator({
        allowConditional: true,
        minConfidenceForApproval: 0.95,
      });

      const session = createSession({
        violation: {
          id: "violation-1",
          ruleId: "RULE-001",
          severity: ViolationSeverity.MAJOR,
          description: "Major violation",
          evidence: [],
          detectedAt: new Date(),
          context: {},
        },
      });

      const result = await customGenerator.generateVerdict(
        session,
        "arbiter-1"
      );

      if (result.verdict.outcome === VerdictOutcome.CONDITIONAL) {
        const hasTimeCondition = result.verdict.conditions?.some((c) =>
          c.includes("48 hours")
        );
        expect(hasTimeCondition).toBe(true);
      }
    });
  });

  describe("validation", () => {
    it("should throw error for session without ID", async () => {
      const session = createSession({ id: "" });

      await expect(
        generator.generateVerdict(session, "arbiter-1")
      ).rejects.toThrow(ArbitrationError);
    });

    it("should throw error for session without violation", async () => {
      const session = createSession({ violation: undefined as any });

      await expect(
        generator.generateVerdict(session, "arbiter-1")
      ).rejects.toThrow("must have violation");
    });

    it("should throw error for session without evaluated rules", async () => {
      const session = createSession({ rulesEvaluated: [] });

      await expect(
        generator.generateVerdict(session, "arbiter-1")
      ).rejects.toThrow("must have evaluated rules");
    });
  });

  describe("audit trail", () => {
    it("should add audit entry", async () => {
      const session = createSession();
      const result = await generator.generateVerdict(session, "arbiter-1");

      generator.addAuditEntry(
        result.verdict,
        "verdict_reviewed",
        "reviewer-1",
        "Reviewed and approved"
      );

      expect(result.verdict.auditLog).toHaveLength(2);
      expect(result.verdict.auditLog[1].action).toBe("verdict_reviewed");
    });
  });

  describe("warnings", () => {
    it("should warn about missing precedents when required", async () => {
      const customGenerator = new VerdictGenerator({
        requirePrecedents: true,
      });

      const session = createSession({ precedents: [] });
      const result = await customGenerator.generateVerdict(
        session,
        "arbiter-1"
      );

      expect(result.warnings.length).toBeGreaterThan(0);
      expect(result.warnings.some((w) => w.includes("precedents"))).toBe(true);
    });

    it("should warn about insufficient reasoning steps", async () => {
      const customGenerator = new VerdictGenerator({
        minReasoningSteps: 10,
      });

      const session = createSession();
      const result = await customGenerator.generateVerdict(
        session,
        "arbiter-1"
      );

      expect(result.warnings.some((w) => w.includes("reasoning steps"))).toBe(
        true
      );
    });
  });

  describe("statistics", () => {
    it("should track verdict count", async () => {
      const session = createSession();

      await generator.generateVerdict(session, "arbiter-1");
      await generator.generateVerdict(session, "arbiter-1");

      const stats = generator.getStatistics();
      expect(stats.totalVerdicts).toBe(2);
    });
  });

  describe("edge cases", () => {
    it("should handle session with no evidence", async () => {
      const session = createSession({ evidence: [] });

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict).toBeDefined();
      expect(result.verdict.evidence).toHaveLength(0);
    });

    it("should handle multiple rules", async () => {
      const rule2: ConstitutionalRule = {
        id: "RULE-002",
        version: "1.0.0",
        category: RuleCategory.TESTING,
        title: "Test coverage required",
        description: "Must have 80% coverage",
        condition: "coverage >= 0.8",
        severity: ViolationSeverity.MODERATE,
        waivable: true,
        requiredEvidence: ["coverage_report"],
        precedents: [],
        effectiveDate: new Date(),
        metadata: {},
      };

      const session = createSession({
        rulesEvaluated: [...createSession().rulesEvaluated, rule2],
      });

      const result = await generator.generateVerdict(session, "arbiter-1");

      expect(result.verdict.rulesApplied).toHaveLength(2);
      expect(result.verdict.rulesApplied).toContain("RULE-002");
    });
  });
});
