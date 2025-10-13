/**
 * Unit tests for AppealArbitrator
 *
 * Tests appeal submission, review, escalation, and decision making.
 */

import { AppealArbitrator } from "@/arbitration/AppealArbitrator";
import {
  AppealStatus,
  ArbitrationSession,
  ArbitrationState,
  ConstitutionalRule,
  RuleCategory,
  Verdict,
  VerdictOutcome,
  ViolationSeverity,
} from "@/types/arbitration";

describe("AppealArbitrator", () => {
  let arbitrator: AppealArbitrator;

  beforeEach(() => {
    arbitrator = new AppealArbitrator();
  });

  // Helper to create a test session
  const createSession = (): ArbitrationSession => {
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
        evidence: ["evidence-1"],
        detectedAt: new Date(),
        context: {},
      },
      rulesEvaluated: [rule],
      evidence: ["evidence-1"],
      participants: ["agent-1"],
      precedents: [],
      startTime: new Date(),
      metadata: {},
    };
  };

  // Helper to create a test verdict
  const createVerdict = (): Verdict => {
    return {
      id: "verdict-1",
      sessionId: "session-1",
      outcome: VerdictOutcome.REJECTED,
      reasoning: [],
      rulesApplied: ["RULE-001"],
      evidence: ["evidence-1"],
      precedents: [],
      confidence: 0.9,
      issuedBy: "arbiter-1",
      issuedAt: new Date(),
      auditLog: [],
    };
  };

  describe("submitAppeal", () => {
    it("should submit a valid appeal", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      expect(appeal.id).toMatch(/^APPEAL-/);
      expect(appeal.status).toBe(AppealStatus.SUBMITTED);
      expect(appeal.level).toBe(1);
      expect(appeal.appellantId).toBe("agent-1");
    });

    it("should reject appeal with insufficient grounds", async () => {
      const session = createSession();
      const verdict = createVerdict();

      await expect(
        arbitrator.submitAppeal(
          session,
          verdict,
          "agent-1",
          "too short",
          ["new-evidence-1"],
          {}
        )
      ).rejects.toThrow("at least 20 characters");
    });

    it("should reject appeal with insufficient evidence", async () => {
      const customArbitrator = new AppealArbitrator({
        minEvidenceForAppeal: 2,
      });

      const session = createSession();
      const verdict = createVerdict();

      await expect(
        customArbitrator.submitAppeal(
          session,
          verdict,
          "agent-1",
          "The verdict was incorrect due to missing context",
          ["new-evidence-1"], // Only 1 piece
          {}
        )
      ).rejects.toThrow("at least 2 piece(s)");
    });

    it("should store appeal for retrieval", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      const retrieved = arbitrator.getAppeal(appeal.id);
      expect(retrieved).toEqual(appeal);
    });
  });

  describe("reviewAppeal", () => {
    it("should review an appeal and make a decision", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict incorrectly assessed the linting errors",
        ["new-evidence-1", "new-evidence-2"],
        {}
      );

      const decision = await arbitrator.reviewAppeal(
        appeal.id,
        ["reviewer-1", "reviewer-2"],
        session,
        verdict
      );

      expect(decision.appealId).toBe(appeal.id);
      expect(decision.decision).toMatch(/^(upheld|overturned)$/);
      expect(decision.reasoning).toBeDefined();
      expect(decision.reviewers).toHaveLength(2);
    });

    it("should throw error for non-existent appeal", async () => {
      const session = createSession();
      const verdict = createVerdict();

      await expect(
        arbitrator.reviewAppeal(
          "non-existent",
          ["reviewer-1"],
          session,
          verdict
        )
      ).rejects.toThrow("not found");
    });

    it("should throw error for appeal not in submitted state", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      // Review once
      await arbitrator.reviewAppeal(
        appeal.id,
        ["reviewer-1"],
        session,
        verdict
      );

      // Try to review again
      await expect(
        arbitrator.reviewAppeal(appeal.id, ["reviewer-2"], session, verdict)
      ).rejects.toThrow("not in submitted state");
    });

    it("should update appeal status to under review", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      await arbitrator.reviewAppeal(
        appeal.id,
        ["reviewer-1"],
        session,
        verdict
      );

      const updated = arbitrator.getAppeal(appeal.id);
      expect(updated!.status).toMatch(/^(upheld|overturned)$/);
      expect(updated!.reviewers).toEqual(["reviewer-1"]);
      expect(updated!.reviewedAt).toBeInstanceOf(Date);
    });

    it("should generate new verdict when overturned", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict incorrectly assessed the error severity, overlooking critical context",
        [
          "new-evidence-1",
          "new-evidence-2",
          "new-evidence-3",
          "new-evidence-4",
        ],
        {}
      );

      const decision = await arbitrator.reviewAppeal(
        appeal.id,
        ["reviewer-1", "reviewer-2", "reviewer-3"],
        session,
        verdict
      );

      if (decision.decision === "overturned") {
        expect(decision.newVerdict).toBeDefined();
        expect(decision.newVerdict!.reasoning.length).toBeGreaterThan(
          verdict.reasoning.length
        );
      }
    });
  });

  describe("escalateAppeal", () => {
    it("should escalate appeal to next level", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      const escalated = await arbitrator.escalateAppeal(
        appeal.id,
        "Reviewer disagreement"
      );

      expect(escalated.level).toBe(2);
      expect(escalated.status).toBe(AppealStatus.SUBMITTED);
      expect(escalated.metadata.escalationReason).toBe("Reviewer disagreement");
    });

    it("should throw error for non-existent appeal", async () => {
      await expect(
        arbitrator.escalateAppeal("non-existent", "reason")
      ).rejects.toThrow("not found");
    });

    it("should throw error when max level reached", async () => {
      const customArbitrator = new AppealArbitrator({
        maxAppealLevels: 2,
      });

      const session = createSession();
      const verdict = createVerdict();

      const appeal = await customArbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      await customArbitrator.escalateAppeal(appeal.id, "reason 1");

      await expect(
        customArbitrator.escalateAppeal(appeal.id, "reason 2")
      ).rejects.toThrow("already at maximum level");
    });
  });

  describe("finalizeAppeal", () => {
    it("should finalize an appeal", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      const result = arbitrator.finalizeAppeal(appeal.id);
      expect(result).toBe(true);

      const updated = arbitrator.getAppeal(appeal.id);
      expect(updated!.status).toBe(AppealStatus.FINALIZED);
      expect(updated!.metadata.finalizedAt).toBeInstanceOf(Date);
    });

    it("should return false for non-existent appeal", () => {
      const result = arbitrator.finalizeAppeal("non-existent");
      expect(result).toBe(false);
    });
  });

  describe("getSessionAppeals", () => {
    it("should return all appeals for a session", async () => {
      const session = createSession();
      const verdict = createVerdict();

      await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "First appeal grounds that are long enough",
        ["evidence-1"],
        {}
      );

      await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-2",
        "Second appeal grounds that are long enough",
        ["evidence-2"],
        {}
      );

      const appeals = arbitrator.getSessionAppeals("session-1");
      expect(appeals).toHaveLength(2);
    });

    it("should return empty array for session with no appeals", () => {
      const appeals = arbitrator.getSessionAppeals("non-existent");
      expect(appeals).toHaveLength(0);
    });
  });

  describe("getDecision", () => {
    it("should return appeal decision", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      const decision = await arbitrator.reviewAppeal(
        appeal.id,
        ["reviewer-1"],
        session,
        verdict
      );

      const retrieved = arbitrator.getDecision(appeal.id);
      expect(retrieved).toEqual(decision);
    });

    it("should return undefined for non-existent appeal", () => {
      const decision = arbitrator.getDecision("non-existent");
      expect(decision).toBeUndefined();
    });
  });

  describe("getStatistics", () => {
    it("should return statistics for empty arbitrator", () => {
      const stats = arbitrator.getStatistics();

      expect(stats.totalAppeals).toBe(0);
      expect(stats.overturnRate).toBe(0);
      expect(stats.averageLevel).toBe(0);
    });

    it("should count appeals by status", async () => {
      const session = createSession();
      const verdict = createVerdict();

      await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "First appeal grounds that are long enough",
        ["evidence-1"],
        {}
      );

      await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-2",
        "Second appeal grounds that are long enough",
        ["evidence-2"],
        {}
      );

      const stats = arbitrator.getStatistics();
      expect(stats.totalAppeals).toBe(2);
      expect(stats.byStatus[AppealStatus.SUBMITTED]).toBe(2);
    });

    it("should calculate overturn rate", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal1 = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "First appeal grounds that are long enough with error mention",
        ["evidence-1", "evidence-2", "evidence-3"],
        {}
      );

      await arbitrator.reviewAppeal(
        appeal1.id,
        ["reviewer-1"],
        session,
        verdict
      );

      const stats = arbitrator.getStatistics();
      expect(stats.overturnRate).toBeGreaterThanOrEqual(0);
      expect(stats.overturnRate).toBeLessThanOrEqual(1);
    });

    it("should calculate average level", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal1 = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "First appeal grounds that are long enough",
        ["evidence-1"],
        {}
      );

      const appeal2 = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-2",
        "Second appeal grounds that are long enough",
        ["evidence-2"],
        {}
      );

      await arbitrator.escalateAppeal(appeal1.id, "reason");

      const stats = arbitrator.getStatistics();
      expect(stats.averageLevel).toBe(1.5);
    });

    it("should group by level", async () => {
      const session = createSession();
      const verdict = createVerdict();

      const appeal1 = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "First appeal grounds that are long enough",
        ["evidence-1"],
        {}
      );

      const appeal2 = await arbitrator.submitAppeal(
        session,
        verdict,
        "agent-2",
        "Second appeal grounds that are long enough",
        ["evidence-2"],
        {}
      );

      await arbitrator.escalateAppeal(appeal1.id, "reason");

      const stats = arbitrator.getStatistics();
      expect(stats.byLevel[1]).toBe(1);
      expect(stats.byLevel[2]).toBe(1);
    });
  });

  describe("edge cases", () => {
    it("should handle appeal with maximum level", async () => {
      const customArbitrator = new AppealArbitrator({
        maxAppealLevels: 1,
      });

      const session = createSession();
      const verdict = createVerdict();

      const appeal = await customArbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      await expect(
        customArbitrator.escalateAppeal(appeal.id, "reason")
      ).rejects.toThrow("already at maximum level");
    });

    it("should handle appeal with no new evidence (just meeting minimum)", async () => {
      const customArbitrator = new AppealArbitrator({
        minEvidenceForAppeal: 1,
      });

      const session = createSession();
      const verdict = createVerdict();

      const appeal = await customArbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect due to missing context",
        ["new-evidence-1"],
        {}
      );

      expect(appeal.newEvidence).toHaveLength(1);
    });

    it("should handle unanimous requirement for overturn", async () => {
      const customArbitrator = new AppealArbitrator({
        requireUnanimous: true,
      });

      const session = createSession();
      const verdict = createVerdict();

      const appeal = await customArbitrator.submitAppeal(
        session,
        verdict,
        "agent-1",
        "The verdict was incorrect",
        ["evidence-1"],
        {}
      );

      const decision = await customArbitrator.reviewAppeal(
        appeal.id,
        ["reviewer-1"],
        session,
        verdict
      );

      // With unanimous requirement, threshold is higher
      expect(decision).toBeDefined();
    });
  });
});
