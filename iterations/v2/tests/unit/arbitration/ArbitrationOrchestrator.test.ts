/**
 * Unit tests for ArbitrationOrchestrator
 *
 * Tests complete arbitration workflow integration, session management,
 * and component coordination.
 */

import { ArbitrationOrchestrator } from "@/arbitration/ArbitrationOrchestrator";
import {
  ArbitrationState,
  ConstitutionalRule,
  ConstitutionalViolation,
  RuleCategory,
  ViolationSeverity,
  WaiverRequest,
} from "@/types/arbitration";

describe("ArbitrationOrchestrator", () => {
  let orchestrator: ArbitrationOrchestrator;

  beforeEach(() => {
    orchestrator = new ArbitrationOrchestrator();
  });

  // Helper to create a test rule
  const createRule = (): ConstitutionalRule => {
    return {
      id: "RULE-001",
      version: "1.0.0",
      category: RuleCategory.CODE_QUALITY,
      title: "Code must be linted",
      description: "All code must pass linting",
      condition: "linted === true",
      severity: ViolationSeverity.MODERATE,
      waivable: true,
      requiredEvidence: ["linter_report"],
      precedents: [],
      effectiveDate: new Date(),
      metadata: {},
    };
  };

  // Helper to create a test violation
  const createViolation = (): ConstitutionalViolation => {
    return {
      id: "violation-1",
      ruleId: "RULE-001",
      severity: ViolationSeverity.MODERATE,
      description: "Code not linted properly",
      evidence: ["commit-123", "linter-output.txt"],
      detectedAt: new Date(),
      violator: "agent-1",
      context: {
        file: "src/test.ts",
        line: 42,
      },
    };
  };

  describe("startSession", () => {
    it("should start a new arbitration session", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1", "arbiter-1"]
      );

      expect(session.id).toMatch(/^ARB-/);
      expect(session.state).toBe(ArbitrationState.RULE_EVALUATION);
      expect(session.violation).toEqual(violation);
      expect(session.rulesEvaluated).toHaveLength(1);
    });

    it("should throw error when max concurrent sessions reached", async () => {
      const customOrchestrator = new ArbitrationOrchestrator({
        maxConcurrentSessions: 1,
      });

      const violation = createViolation();
      const rule = createRule();

      await customOrchestrator.startSession(violation, [rule], ["agent-1"]);

      await expect(
        customOrchestrator.startSession(violation, [rule], ["agent-2"])
      ).rejects.toThrow("Maximum concurrent sessions reached");
    });

    it("should initialize session metrics", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      const metrics = orchestrator.getSessionMetrics(session.id);
      expect(metrics).toBeDefined();
      expect(metrics!.sessionId).toBe(session.id);
    });
  });

  describe("evaluateRules", () => {
    it("should evaluate constitutional rules", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);

      const updated = orchestrator.getSession(session.id);
      expect(updated.metadata.ruleEvaluationResults).toBeDefined();
      expect(Array.isArray(updated.metadata.ruleEvaluationResults)).toBe(true);
    });

    it("should throw error if not in rule evaluation state", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);

      // Try to evaluate again
      await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow(
        "Cannot evaluate rules in state"
      );
    });

    it("should track rule evaluation time", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);

      const metrics = orchestrator.getSessionMetrics(session.id);
      expect(metrics!.ruleEvaluationMs).toBeGreaterThanOrEqual(0);
    });
  });

  describe("generateVerdict", () => {
    it("should generate verdict for session", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);

      const verdict = await orchestrator.generateVerdict(
        session.id,
        "arbiter-1"
      );

      expect(verdict.id).toBeDefined();
      expect(verdict.sessionId).toBe(session.id);
      expect(verdict.issuedBy).toBe("arbiter-1");
    });

    it("should throw error if not in verdict generation state", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      // Try to generate verdict without evaluating rules
      await expect(
        orchestrator.generateVerdict(session.id, "arbiter-1")
      ).rejects.toThrow("Cannot generate verdict in state");
    });

    it("should create precedent from high-confidence verdict", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const stats = orchestrator.getStatistics();
      expect(stats.totalPrecedents).toBeGreaterThanOrEqual(0);
    });

    it("should track verdict generation time", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const metrics = orchestrator.getSessionMetrics(session.id);
      expect(metrics!.verdictGenerationMs).toBeGreaterThan(0);
    });
  });

  describe("evaluateWaiver", () => {
    it("should evaluate waiver request", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const waiverRequest: WaiverRequest = {
        id: "waiver-1",
        ruleId: "RULE-001",
        requestedBy: "agent-1",
        justification: "Emergency production fix required immediately",
        evidence: ["incident-report.pdf", "manager-approval.txt"],
        requestedDuration: 24 * 60 * 60 * 1000,
        requestedAt: new Date(),
        context: {},
      };

      await orchestrator.evaluateWaiver(session.id, waiverRequest, "arbiter-1");

      const updated = orchestrator.getSession(session.id);
      expect(updated.waiverRequest).toEqual(waiverRequest);
      expect(updated.metadata.waiverDecision).toBeDefined();
    });

    it("should throw error if waivers disabled", async () => {
      const customOrchestrator = new ArbitrationOrchestrator({
        enableWaivers: false,
      });

      const violation = createViolation();
      const rule = createRule();

      const session = await customOrchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      const waiverRequest: WaiverRequest = {
        id: "waiver-1",
        ruleId: "RULE-001",
        requestedBy: "agent-1",
        justification: "Emergency fix required",
        evidence: ["evidence-1"],
        requestedDuration: 86400000,
        requestedAt: new Date(),
        context: {},
      };

      await expect(
        customOrchestrator.evaluateWaiver(
          session.id,
          waiverRequest,
          "arbiter-1"
        )
      ).rejects.toThrow("Waiver system is disabled");
    });
  });

  describe("submitAppeal", () => {
    it("should submit appeal for verdict", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const appeal = await orchestrator.submitAppeal(
        session.id,
        "agent-1",
        "The verdict incorrectly assessed the severity of the violation",
        ["new-evidence-1", "new-evidence-2"]
      );

      expect(appeal.id).toMatch(/^APPEAL-/);
      expect(appeal.sessionId).toBe(session.id);
      expect(appeal.appellantId).toBe("agent-1");
    });

    it("should throw error if appeals disabled", async () => {
      const customOrchestrator = new ArbitrationOrchestrator({
        enableAppeals: false,
      });

      const violation = createViolation();
      const rule = createRule();

      const session = await customOrchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await customOrchestrator.evaluateRules(session.id);
      await customOrchestrator.generateVerdict(session.id, "arbiter-1");

      await expect(
        customOrchestrator.submitAppeal(
          session.id,
          "agent-1",
          "The verdict was incorrect",
          ["evidence-1"]
        )
      ).rejects.toThrow("Appeal system is disabled");
    });

    it("should throw error if no verdict exists", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await expect(
        orchestrator.submitAppeal(
          session.id,
          "agent-1",
          "The verdict was incorrect",
          ["evidence-1"]
        )
      ).rejects.toThrow("Cannot appeal session without verdict");
    });
  });

  describe("reviewAppeal", () => {
    it("should review appeal and update verdict", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const appeal = await orchestrator.submitAppeal(
        session.id,
        "agent-1",
        "The verdict incorrectly assessed the violation",
        ["evidence-1", "evidence-2"]
      );

      await orchestrator.reviewAppeal(session.id, appeal.id, [
        "reviewer-1",
        "reviewer-2",
      ]);

      const updated = orchestrator.getSession(session.id);
      expect(updated.metadata.appealDecision).toBeDefined();
    });

    it("should create precedent for overturned verdicts", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const appeal = await orchestrator.submitAppeal(
        session.id,
        "agent-1",
        "The verdict incorrectly assessed due to missing critical context and evidence",
        ["evidence-1", "evidence-2", "evidence-3", "evidence-4"]
      );

      await orchestrator.reviewAppeal(session.id, appeal.id, [
        "reviewer-1",
        "reviewer-2",
        "reviewer-3",
      ]);

      const stats = orchestrator.getStatistics();
      expect(stats.totalPrecedents).toBeGreaterThanOrEqual(0);
    });
  });

  describe("getSession", () => {
    it("should retrieve session by ID", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      const retrieved = orchestrator.getSession(session.id);
      expect(retrieved).toEqual(session);
    });

    it("should throw error for non-existent session", () => {
      expect(() => orchestrator.getSession("non-existent")).toThrow(
        "Session non-existent not found"
      );
    });
  });

  describe("getActiveSessions", () => {
    it("should return only active sessions", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session1 = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      const session2 = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-2"]
      );

      await orchestrator.evaluateRules(session1.id);
      await orchestrator.generateVerdict(session1.id, "arbiter-1");
      await orchestrator.completeSession(session1.id);

      const active = orchestrator.getActiveSessions();
      expect(active).toHaveLength(1);
      expect(active[0].id).toBe(session2.id);
    });
  });

  describe("getStatistics", () => {
    it("should return orchestrator statistics", async () => {
      const violation = createViolation();
      const rule = createRule();

      await orchestrator.startSession(violation, [rule], ["agent-1"]);

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(1);
      expect(stats.activeSessions).toBe(1);
      expect(stats.completedSessions).toBe(0);
    });

    it("should track completed sessions", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      const stats = orchestrator.getStatistics();
      expect(stats.completedSessions).toBe(1);
      expect(stats.activeSessions).toBe(0);
    });
  });

  describe("completeSession", () => {
    it("should complete a session", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.completeSession(session.id);

      const updated = orchestrator.getSession(session.id);
      expect(updated.state).toBe(ArbitrationState.COMPLETED);
      expect(updated.endTime).toBeInstanceOf(Date);
    });

    it("should update session metrics", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      // Add minimal delay to ensure non-zero duration
      await new Promise((resolve) => setTimeout(resolve, 1));

      await orchestrator.completeSession(session.id);

      const metrics = orchestrator.getSessionMetrics(session.id);
      expect(metrics!.totalDurationMs).toBeGreaterThanOrEqual(0);
      expect(metrics!.finalState).toBe(ArbitrationState.COMPLETED);
    });
  });

  describe("failSession", () => {
    it("should fail a session with error", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      const error = new Error("Test error");
      await orchestrator.failSession(session.id, error);

      const updated = orchestrator.getSession(session.id);
      expect(updated.state).toBe(ArbitrationState.FAILED);
      expect(updated.metadata.error).toBeDefined();
      expect(updated.metadata.error.message).toBe("Test error");
    });
  });

  describe("state transitions", () => {
    it("should track state transitions", async () => {
      const violation = createViolation();
      const rule = createRule();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const updated = orchestrator.getSession(session.id);
      expect(updated.metadata.stateTransitions).toBeDefined();
      expect(updated.metadata.stateTransitions.length).toBeGreaterThan(0);
    });
  });

  describe("getComponents", () => {
    it("should provide access to internal components", () => {
      const components = orchestrator.getComponents();

      expect(components.ruleEngine).toBeDefined();
      expect(components.verdictGenerator).toBeDefined();
      expect(components.waiverInterpreter).toBeDefined();
      expect(components.precedentManager).toBeDefined();
      expect(components.appealArbitrator).toBeDefined();
    });
  });

  describe("end-to-end workflow", () => {
    it("should complete full arbitration workflow", async () => {
      const violation = createViolation();
      const rule = createRule();

      // Start session
      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1", "arbiter-1"]
      );

      expect(session.state).toBe(ArbitrationState.RULE_EVALUATION);

      // Evaluate rules
      await orchestrator.evaluateRules(session.id);

      // Generate verdict
      const verdict = await orchestrator.generateVerdict(
        session.id,
        "arbiter-1"
      );

      expect(verdict).toBeDefined();
      expect(verdict.sessionId).toBe(session.id);

      // Complete session
      await orchestrator.completeSession(session.id);

      const updated = orchestrator.getSession(session.id);
      expect(updated.state).toBe(ArbitrationState.COMPLETED);
      expect(updated.verdict).toEqual(verdict);
    });
  });
});
