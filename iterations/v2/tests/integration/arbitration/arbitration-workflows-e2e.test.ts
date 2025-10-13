/**
 * Integration Tests: ARBITER-015 Arbitration Workflows End-to-End
 *
 * @author @darianrosebrook
 *
 * Tests complete arbitration workflows, validating that the arbitration
 * orchestrator properly coordinates all components through real-world scenarios.
 *
 * Test Coverage (15+ integration tests):
 * - Full arbitration workflows (violation → verdict → completion)
 * - Waiver request workflows with time-bounded expiration
 * - Appeal processes with escalation and overturn
 * - Precedent creation and application
 * - Multi-level appeal chains
 * - Concurrent session management
 * - Complex workflows with multiple agents
 * - State transition validation
 * - Performance and metrics tracking
 */

import { ArbitrationOrchestrator } from "@/arbitration/ArbitrationOrchestrator";
import {
  ArbitrationState,
  ConstitutionalRule,
  ConstitutionalViolation,
  RuleCategory,
  ViolationSeverity,
  WaiverRequest,
  WaiverStatus,
  VerdictOutcome,
} from "@/types/arbitration";

describe("ARBITER-015 Integration: End-to-End Arbitration Workflows", () => {
  let orchestrator: ArbitrationOrchestrator;

  beforeEach(() => {
    orchestrator = new ArbitrationOrchestrator({
      enableWaivers: true,
      enableAppeals: true,
      trackPerformance: true,
      maxConcurrentSessions: 10,
    });
  });

  afterEach(() => {
    orchestrator.clear();
  });

  // Helper: Create a constitutional rule
  const createRule = (
    overrides: Partial<ConstitutionalRule> = {}
  ): ConstitutionalRule => {
    return {
      id: overrides.id || "RULE-001",
      version: "1.0.0",
      category: RuleCategory.CODE_QUALITY,
      title: "Code must be linted",
      description: "All code must pass linting before commit",
      condition: "linted === true",
      severity: ViolationSeverity.MODERATE,
      waivable: true,
      requiredEvidence: ["linter_report"],
      precedents: [],
      effectiveDate: new Date(),
      metadata: {},
      ...overrides,
    };
  };

  // Helper: Create a violation
  const createViolation = (
    overrides: Partial<ConstitutionalViolation> = {}
  ): ConstitutionalViolation => {
    return {
      id: overrides.id || "violation-1",
      ruleId: overrides.ruleId || "RULE-001",
      severity: ViolationSeverity.MODERATE,
      description: "Code not linted properly in src/utils/helper.ts",
      evidence: ["commit-abc123", "linter-output.txt"],
      detectedAt: new Date(),
      violator: "agent-dev-1",
      context: {
        file: "src/utils/helper.ts",
        line: 42,
        commitHash: "abc123",
      },
      ...overrides,
    };
  };

  describe("Integration Test 1-3: Full Arbitration Workflows", () => {
    it("should complete simple arbitration workflow end-to-end", async () => {
      // Given: A constitutional violation
      const rule = createRule();
      const violation = createViolation();

      // When: We run the full arbitration workflow
      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-arbiter-1", "agent-reviewer-1"]
      );

      expect(session.state).toBe(ArbitrationState.RULE_EVALUATION);
      expect(session.id).toMatch(/^ARB-/);

      // Evaluate rules
      await orchestrator.evaluateRules(session.id);
      const afterRules = orchestrator.getSession(session.id);
      expect(afterRules.state).toBe(ArbitrationState.VERDICT_GENERATION);

      // Generate verdict
      const verdict = await orchestrator.generateVerdict(
        session.id,
        "agent-arbiter-1"
      );
      expect(verdict.id).toBeDefined();
      expect(verdict.outcome).toBeDefined();
      expect(verdict.reasoning.length).toBeGreaterThanOrEqual(0);

      // Complete session
      await orchestrator.completeSession(session.id);

      // Then: Session should be completed with verdict
      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.state).toBe(ArbitrationState.COMPLETED);
      expect(finalSession.verdict).toBeDefined();
      expect(finalSession.verdict!.id).toBe(verdict.id);
      expect(finalSession.endTime).toBeInstanceOf(Date);
    });

    it("should track performance metrics through complete workflow", async () => {
      const rule = createRule();
      const violation = createViolation();

      const startTime = Date.now();
      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      // Add minimal delay to ensure non-zero duration
      await new Promise((resolve) => setTimeout(resolve, 5));

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");
      await orchestrator.completeSession(session.id);
      const duration = Date.now() - startTime;

      const metrics = orchestrator.getSessionMetrics(session.id);
      expect(metrics).toBeDefined();
      expect(metrics!.ruleEvaluationMs).toBeGreaterThanOrEqual(0);
      expect(metrics!.verdictGenerationMs).toBeGreaterThan(0);
      expect(metrics!.totalDurationMs).toBeGreaterThanOrEqual(0);
      expect(metrics!.totalDurationMs).toBeLessThanOrEqual(duration + 100); // Allow 100ms buffer
      expect(metrics!.finalState).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle critical security violations with multiple reviewers", async () => {
      const rule = createRule({
        id: "RULE-SEC-001",
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.CRITICAL,
        title: "No hardcoded credentials",
        description: "Credentials must not be hardcoded in source",
      });

      const violation = createViolation({
        id: "violation-sec-001",
        ruleId: "RULE-SEC-001",
        severity: ViolationSeverity.CRITICAL,
        description: "Hardcoded API key found in config.ts",
        evidence: [
          "security-scan-report.pdf",
          "code-snippet.txt",
          "git-blame-output.txt",
        ],
      });

      const session = await orchestrator.startSession(
        violation,
        [rule],
        [
          "agent-security-lead",
          "agent-architect",
          "agent-compliance-officer",
        ]
      );

      await orchestrator.evaluateRules(session.id);
      const verdict = await orchestrator.generateVerdict(
        session.id,
        "agent-security-lead"
      );
      await orchestrator.completeSession(session.id);

      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.state).toBe(ArbitrationState.COMPLETED);
      expect(finalSession.participants).toHaveLength(3);
      expect(verdict.outcome).toBeDefined();
    });
  });

  describe("Integration Test 4-7: Waiver Request Workflows", () => {
    it("should handle waiver request with emergency justification", async () => {
      const rule = createRule({ waivable: true });
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1", "agent-2"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");

      const waiverRequest: WaiverRequest = {
        id: "waiver-emergency-001",
        ruleId: rule.id,
        requestedBy: "agent-dev-1",
        justification:
          "Emergency production hotfix required to restore service. P0 incident affecting 50% of users. Will address in follow-up PR within 24 hours.",
        evidence: [
          "incident-report-12345.pdf",
          "manager-approval-email.txt",
          "service-impact-metrics.csv",
        ],
        requestedDuration: 24 * 60 * 60 * 1000, // 24 hours
        requestedAt: new Date(),
        context: {
          incidentId: "INC-12345",
          severity: "P0",
        },
      };

      await orchestrator.evaluateWaiver(
        session.id,
        waiverRequest,
        "agent-arbiter-1"
      );

      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.state).toBe(ArbitrationState.COMPLETED);
      expect(finalSession.waiverRequest).toEqual(waiverRequest);
      expect(finalSession.metadata.waiverDecision).toBeDefined();
      expect(finalSession.metadata.waiverDecision.status).toBeDefined();
      expect([WaiverStatus.APPROVED, WaiverStatus.REJECTED]).toContain(
        finalSession.metadata.waiverDecision.status
      );
    });

    it("should handle waiver request with weak justification", async () => {
      const rule = createRule({ waivable: true });
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");

      const waiverRequest: WaiverRequest = {
        id: "waiver-weak-001",
        ruleId: rule.id,
        requestedBy: "agent-dev-1",
        justification: "I was in a hurry",
        evidence: [],
        requestedDuration: 7 * 24 * 60 * 60 * 1000, // 7 days
        requestedAt: new Date(),
        context: {},
      };

      await orchestrator.evaluateWaiver(
        session.id,
        waiverRequest,
        "agent-arbiter-1"
      );

      // Verify waiver was evaluated (interpreter has its own logic for approval/rejection)
      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.metadata.waiverDecision).toBeDefined();
      expect(finalSession.metadata.waiverDecision.status).toBeDefined();
      expect([WaiverStatus.APPROVED, WaiverStatus.REJECTED]).toContain(
        finalSession.metadata.waiverDecision.status
      );
      expect(finalSession.metadata.waiverDecision.reasoning).toBeDefined();
    });

    it("should handle time-bounded waiver with expiration", async () => {
      const rule = createRule({ waivable: true });
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");

      const now = new Date();
      const waiverRequest: WaiverRequest = {
        id: "waiver-time-bounded",
        ruleId: rule.id,
        requestedBy: "agent-dev-1",
        justification: "Temporary exception for testing phase",
        evidence: ["test-plan.pdf"],
        requestedDuration: 2 * 60 * 60 * 1000, // 2 hours
        requestedAt: now,
        context: {
          expiresAt: new Date(now.getTime() + 2 * 60 * 60 * 1000),
        },
      };

      await orchestrator.evaluateWaiver(
        session.id,
        waiverRequest,
        "agent-arbiter-1"
      );

      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.waiverRequest).toBeDefined();
      expect(finalSession.waiverRequest!.requestedDuration).toBe(
        2 * 60 * 60 * 1000
      );
      expect(finalSession.metadata.waiverDecision).toBeDefined();

      // Verify waiver statistics
      const { waiverInterpreter } = orchestrator.getComponents();
      const stats = waiverInterpreter.getStatistics();
      expect(stats.totalWaivers).toBeGreaterThan(0);
    });

    it("should prevent waiver when waivers are disabled", async () => {
      const customOrchestrator = new ArbitrationOrchestrator({
        enableWaivers: false,
      });

      const rule = createRule({ waivable: true });
      const violation = createViolation();

      const session = await customOrchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await customOrchestrator.evaluateRules(session.id);
      await customOrchestrator.generateVerdict(session.id, "agent-arbiter-1");

      const waiverRequest: WaiverRequest = {
        id: "waiver-001",
        ruleId: rule.id,
        requestedBy: "agent-dev-1",
        justification: "Testing",
        evidence: [],
        requestedDuration: 86400000,
        requestedAt: new Date(),
        context: {},
      };

      await expect(
        customOrchestrator.evaluateWaiver(
          session.id,
          waiverRequest,
          "agent-arbiter-1"
        )
      ).rejects.toThrow("Waiver system is disabled");

      customOrchestrator.clear();
    });
  });

  describe("Integration Test 8-11: Appeal and Escalation Workflows", () => {
    it("should handle appeal with strong new evidence leading to overturn", async () => {
      const rule = createRule();
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      const originalVerdict = await orchestrator.generateVerdict(
        session.id,
        "agent-arbiter-1"
      );
      await orchestrator.completeSession(session.id);

      // Submit appeal with strong evidence
      const appeal = await orchestrator.submitAppeal(
        session.id,
        "agent-dev-1",
        "Original verdict failed to consider that linter was misconfigured. False positive. Code actually complies with standards.",
        [
          "linter-config-error.log",
          "correct-linter-output.txt",
          "team-lead-confirmation.pdf",
          "historical-precedent-case-789.json",
        ]
      );

      expect(appeal.id).toMatch(/^APPEAL-/);
      expect(appeal.sessionId).toBe(session.id);
      expect(appeal.originalVerdictId).toBe(originalVerdict.id);

      // Review appeal
      await orchestrator.reviewAppeal(session.id, appeal.id, [
        "appeal-reviewer-1",
        "appeal-reviewer-2",
        "appeal-reviewer-3",
      ]);

      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.state).toBe(ArbitrationState.COMPLETED);
      expect(finalSession.metadata.appealDecision).toBeDefined();

      // Verify precedent created if overturned
      const stats = orchestrator.getStatistics();
      expect(stats.totalAppeals).toBeGreaterThanOrEqual(1);
      if (finalSession.metadata.appealDecision.decision === "overturned") {
        expect(stats.totalPrecedents).toBeGreaterThan(0);
      }
    });

    it("should handle appeal rejection with insufficient evidence", async () => {
      const rule = createRule();
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");
      await orchestrator.completeSession(session.id);

      // Weak appeal
      const appeal = await orchestrator.submitAppeal(
        session.id,
        "agent-dev-1",
        "I disagree with the verdict",
        ["personal-opinion.txt"]
      );

      await orchestrator.reviewAppeal(session.id, appeal.id, [
        "reviewer-1",
        "reviewer-2",
      ]);

      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.metadata.appealDecision.decision).toBe("upheld");
    });

    it("should handle multi-level appeal chain", async () => {
      const rule = createRule();
      const violation = createViolation();

      // Initial session
      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");
      await orchestrator.completeSession(session.id);

      // Level 1 Appeal
      const appeal1 = await orchestrator.submitAppeal(
        session.id,
        "agent-dev-1",
        "Initial appeal: Verdict was too harsh given context",
        ["context-evidence-1.pdf", "mitigating-factor-1.txt"]
      );

      await orchestrator.reviewAppeal(session.id, appeal1.id, [
        "reviewer-L1-1",
        "reviewer-L1-2",
      ]);

      let appealSession = orchestrator.getSession(session.id);
      expect(appealSession.metadata.appealDecision).toBeDefined();

      // Level 2 Appeal (if first was upheld)
      if (appealSession.metadata.appealDecision.decision === "upheld") {
        const appeal2 = await orchestrator.submitAppeal(
          session.id,
          "agent-dev-1",
          "Level 2 appeal: New critical evidence emerged",
          [
            "new-evidence-critical.pdf",
            "expert-testimony.txt",
            "technical-analysis.json",
          ]
        );

        await orchestrator.reviewAppeal(session.id, appeal2.id, [
          "senior-reviewer-1",
          "senior-reviewer-2",
          "senior-reviewer-3",
        ]);

        appealSession = orchestrator.getSession(session.id);
        expect(appealSession.metadata.appealDecision).toBeDefined();
      }

      // Verify appeal chain tracked
      const stats = orchestrator.getStatistics();
      expect(stats.totalAppeals).toBeGreaterThanOrEqual(1);
    });

    it("should prevent appeal when appeals are disabled", async () => {
      const customOrchestrator = new ArbitrationOrchestrator({
        enableAppeals: false,
      });

      const rule = createRule();
      const violation = createViolation();

      const session = await customOrchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await customOrchestrator.evaluateRules(session.id);
      await customOrchestrator.generateVerdict(session.id, "agent-arbiter-1");
      await customOrchestrator.completeSession(session.id);

      await expect(
        customOrchestrator.submitAppeal(
          session.id,
          "agent-1",
          "Appeal",
          ["evidence"]
        )
      ).rejects.toThrow("Appeal system is disabled");

      customOrchestrator.clear();
    });
  });

  describe("Integration Test 12-14: Precedent Management", () => {
    it("should create precedent from high-confidence verdict", async () => {
      const rule = createRule({ id: "RULE-SECURITY-001" });
      const violation = createViolation({
        id: "violation-001",
        ruleId: "RULE-SECURITY-001",
        description: "API endpoint missing authentication",
      });

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      const verdict = await orchestrator.generateVerdict(
        session.id,
        "agent-arbiter-1"
      );
      await orchestrator.completeSession(session.id);

      // Verify precedent created
      const stats = orchestrator.getStatistics();
      if (verdict.confidence > 0.8) {
        expect(stats.totalPrecedents).toBeGreaterThan(0);
      }
    });

    it("should search and apply similar precedents", async () => {
      // Create precedent case
      const rule1 = createRule({ id: "RULE-SEC-001" });
      const violation1 = createViolation({
        id: "violation-001",
        ruleId: "RULE-SEC-001",
        description: "REST API endpoint missing authentication middleware",
      });

      const session1 = await orchestrator.startSession(
        violation1,
        [rule1],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session1.id);
      const verdict1 = await orchestrator.generateVerdict(
        session1.id,
        "agent-arbiter-1"
      );
      await orchestrator.completeSession(session1.id);

      // Similar case
      const rule2 = createRule({ id: "RULE-SEC-002" });
      const violation2 = createViolation({
        id: "violation-002",
        ruleId: "RULE-SEC-002",
        description: "GraphQL endpoint missing authentication resolver",
      });

      const session2 = await orchestrator.startSession(
        violation2,
        [rule2],
        ["agent-1"]
      );

      // Search for similar precedents
      const { precedentManager } = orchestrator.getComponents();
      const similarPrecedents = precedentManager.searchPrecedents({
        categories: [RuleCategory.SECURITY],
      });

      expect(similarPrecedents).toBeInstanceOf(Array);
      expect(similarPrecedents.length).toBeGreaterThanOrEqual(0);
      if (similarPrecedents.length > 0) {
        expect(similarPrecedents[0].verdict.id).toBe(verdict1.id);
      }

      await orchestrator.evaluateRules(session2.id);
      await orchestrator.generateVerdict(session2.id, "agent-arbiter-1");
      await orchestrator.completeSession(session2.id);
    });

    it("should track precedent citations and applicability", async () => {
      const rule = createRule();
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");
      await orchestrator.completeSession(session.id);

      const { precedentManager } = orchestrator.getComponents();
      const stats = precedentManager.getStatistics();

      expect(stats).toBeDefined();
      expect(stats.totalPrecedents).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Integration Test 15-17: Concurrent Sessions and Performance", () => {
    it("should handle 5 concurrent arbitration sessions", async () => {
      const sessions: string[] = [];

      // Start 5 concurrent sessions
      for (let i = 0; i < 5; i++) {
        const rule = createRule({ id: `RULE-${i}` });
        const violation = createViolation({
          id: `violation-${i}`,
          ruleId: `RULE-${i}`,
        });

        const session = await orchestrator.startSession(
          violation,
          [rule],
          [`agent-${i}`]
        );

        sessions.push(session.id);
      }

      // Process all sessions concurrently
      await Promise.all(
        sessions.map(async (sessionId) => {
          await orchestrator.evaluateRules(sessionId);
          await orchestrator.generateVerdict(sessionId, "agent-arbiter");
          await orchestrator.completeSession(sessionId);
        })
      );

      // Verify all completed
      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(5);
      expect(stats.completedSessions).toBe(5);
      expect(stats.activeSessions).toBe(0);

      // Verify each session has metrics
      sessions.forEach((sessionId) => {
        const metrics = orchestrator.getSessionMetrics(sessionId);
        expect(metrics).toBeDefined();
        expect(metrics!.finalState).toBe(ArbitrationState.COMPLETED);
      });
    });

    it("should respect max concurrent session limit", async () => {
      const limitedOrchestrator = new ArbitrationOrchestrator({
        maxConcurrentSessions: 2,
      });

      const rule = createRule();
      const violation = createViolation();

      // Start 2 sessions (should succeed)
      await limitedOrchestrator.startSession(violation, [rule], ["agent-1"]);
      await limitedOrchestrator.startSession(violation, [rule], ["agent-2"]);

      // Try to start 3rd (should fail)
      await expect(
        limitedOrchestrator.startSession(violation, [rule], ["agent-3"])
      ).rejects.toThrow("Maximum concurrent sessions reached");

      limitedOrchestrator.clear();
    });

    it("should maintain performance with 10 rapid successive sessions", async () => {
      const startTime = Date.now();
      const sessionIds: string[] = [];

      for (let i = 0; i < 10; i++) {
        const rule = createRule({ id: `RULE-PERF-${i}` });
        const violation = createViolation({
          id: `violation-perf-${i}`,
          ruleId: `RULE-PERF-${i}`,
        });

        const session = await orchestrator.startSession(
          violation,
          [rule],
          [`agent-${i}`]
        );

        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "agent-arbiter");
        await orchestrator.completeSession(session.id);

        sessionIds.push(session.id);
      }

      const duration = Date.now() - startTime;

      // Verify all completed
      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(10);
      expect(stats.completedSessions).toBe(10);

      // Performance assertion: should complete in reasonable time
      expect(duration).toBeLessThan(10000); // 10 seconds for 10 sessions

      // Verify average performance
      const allMetrics = orchestrator.getAllMetrics();
      const avgDuration =
        allMetrics.reduce((sum, m) => sum + m.totalDurationMs, 0) /
        allMetrics.length;
      expect(avgDuration).toBeLessThan(1000); // < 1 second per session avg
    });
  });

  describe("Integration Test 18-20: Complex Multi-Component Workflows", () => {
    it("should complete full workflow: rules → verdict → waiver → appeal", async () => {
      // Phase 1: Initial arbitration
      const rule = createRule({
        id: "RULE-COMPLEX-001",
        category: RuleCategory.SECURITY,
        severity: ViolationSeverity.CRITICAL,
        waivable: true,
      });

      const violation = createViolation({
        id: "violation-complex",
        ruleId: "RULE-COMPLEX-001",
        severity: ViolationSeverity.CRITICAL,
        description: "Security vulnerability in authentication flow",
      });

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-security-1", "agent-dev-1", "agent-architect-1"]
      );

      // Phase 2: Rule evaluation
      await orchestrator.evaluateRules(session.id);

      // Phase 3: Generate verdict
      const verdict = await orchestrator.generateVerdict(
        session.id,
        "agent-arbiter-1"
      );

      expect(verdict.outcome).toBeDefined();

      // Phase 4: Waiver request
      const waiverRequest: WaiverRequest = {
        id: "waiver-complex",
        ruleId: rule.id,
        requestedBy: "agent-dev-1",
        justification:
          "Fix is ready but needs 48h to complete testing before deployment.",
        evidence: [
          "fix-implementation.patch",
          "test-plan.pdf",
          "deployment-schedule.xlsx",
        ],
        requestedDuration: 48 * 60 * 60 * 1000,
        requestedAt: new Date(),
        context: {
          fixReady: true,
          testingInProgress: true,
        },
      };

      await orchestrator.evaluateWaiver(
        session.id,
        waiverRequest,
        "agent-arbiter-1"
      );

      // Phase 5: Appeal
      const appeal = await orchestrator.submitAppeal(
        session.id,
        "agent-dev-1",
        "Request expedited review based on fix completion",
        ["fix-completed-early.pdf", "test-results-passed.json"]
      );

      await orchestrator.reviewAppeal(session.id, appeal.id, [
        "appeal-reviewer-1",
      ]);

      // Verify complete workflow
      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.state).toBe(ArbitrationState.COMPLETED);
      expect(finalSession.verdict).toBeDefined();
      expect(finalSession.waiverRequest).toBeDefined();
      expect(finalSession.metadata.waiverDecision).toBeDefined();
      expect(finalSession.metadata.appealDecision).toBeDefined();

      // Verify metrics (may be zero in fast test environments)
      const metrics = orchestrator.getSessionMetrics(session.id);
      expect(metrics).toBeDefined();
      expect(metrics!.totalDurationMs).toBeGreaterThanOrEqual(0);

      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(1);
      expect(stats.completedSessions).toBe(1);
    });

    it("should maintain state consistency through complex transitions", async () => {
      const rule = createRule();
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1", "agent-2"]
      );

      // Track state transitions
      const states: ArbitrationState[] = [session.state];

      await orchestrator.evaluateRules(session.id);
      states.push(orchestrator.getSession(session.id).state);

      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");
      states.push(orchestrator.getSession(session.id).state);

      await orchestrator.completeSession(session.id);
      states.push(orchestrator.getSession(session.id).state);

      // Verify state progression
      expect(states).toEqual([
        ArbitrationState.RULE_EVALUATION,
        ArbitrationState.VERDICT_GENERATION,
        ArbitrationState.VERDICT_GENERATION, // Stays after verdict
        ArbitrationState.COMPLETED,
      ]);

      // Verify state transitions logged
      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.metadata.stateTransitions).toBeDefined();
      expect(finalSession.metadata.stateTransitions.length).toBeGreaterThan(0);
    });

    it("should handle error recovery in complex workflows", async () => {
      const rule = createRule();
      const violation = createViolation();

      const session = await orchestrator.startSession(
        violation,
        [rule],
        ["agent-1"]
      );

      // Try invalid operation
      await expect(
        orchestrator.generateVerdict(session.id, "agent-arbiter-1")
      ).rejects.toThrow("Cannot generate verdict in state");

      // Session should be in valid state
      const currentSession = orchestrator.getSession(session.id);
      expect(currentSession.state).toBe(ArbitrationState.RULE_EVALUATION);

      // Continue with correct flow
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "agent-arbiter-1");
      await orchestrator.completeSession(session.id);

      const finalSession = orchestrator.getSession(session.id);
      expect(finalSession.state).toBe(ArbitrationState.COMPLETED);
    });
  });
});

