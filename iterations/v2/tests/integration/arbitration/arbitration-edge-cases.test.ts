/**
 * Integration Tests: Arbitration Edge Cases
 *
 * Comprehensive edge case testing for arbitration workflows, covering boundary conditions,
 * error scenarios, concurrent operations, and unusual but valid use cases.
 *
 * Test Coverage (20+ edge case tests):
 * - Boundary condition testing (empty inputs, maximum values, special characters)
 * - Error recovery and resilience testing
 * - Concurrent operation handling
 * - Timeout and performance edge cases
 * - State transition edge cases
 * - Data validation edge cases
 * - Resource limit testing
 * - Unusual but valid workflow patterns
 */

import { ArbitrationOrchestrator } from "@/arbitration/ArbitrationOrchestrator";
import {
  ArbitrationError,
  ArbitrationState,
  ConstitutionalRule,
  ConstitutionalViolation,
  RuleCategory,
  VerdictOutcome,
  ViolationSeverity,
  WaiverRequest,
  WaiverStatus,
} from "@/types/arbitration";

describe("ARBITER-015 Integration: Arbitration Edge Cases", () => {
  let orchestrator: ArbitrationOrchestrator;

  beforeEach(() => {
    orchestrator = new ArbitrationOrchestrator({
      enableWaivers: true,
      enableAppeals: true,
      trackPerformance: true,
      maxConcurrentSessions: 10,
      sessionTimeoutMs: 60000, // 1 minute for tests
    });
  });

  afterEach(async () => {
    // Clean up any active sessions
    const activeSessions = orchestrator.getActiveSessions();
    for (const session of activeSessions) {
      try {
        if (session.state !== ArbitrationState.COMPLETED &&
            session.state !== ArbitrationState.FAILED) {
          await orchestrator.completeSession(session.id);
        }
      } catch (e) {
        // Ignore cleanup errors
      }
    }
    orchestrator.clear();
  });

  // Helper: Create rule with custom properties
  const createRule = (overrides: Partial<ConstitutionalRule> = {}): ConstitutionalRule => {
    return {
      id: overrides.id || `rule-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      version: overrides.version || "1.0.0",
      category: overrides.category || RuleCategory.CODE_QUALITY,
      title: overrides.title || "Test rule",
      description: overrides.description || "Test rule description",
      condition: overrides.condition || "test === true",
      severity: overrides.severity || ViolationSeverity.MODERATE,
      waivable: overrides.waivable ?? true,
      requiredEvidence: overrides.requiredEvidence || ["test_evidence"],
      precedents: overrides.precedents || [],
      effectiveDate: overrides.effectiveDate || new Date(),
      expirationDate: overrides.expirationDate,
      metadata: overrides.metadata || {},
      ...overrides,
    };
  };

  // Helper: Create violation with custom properties
  const createViolation = (overrides: Partial<ConstitutionalViolation> = {}): ConstitutionalViolation => {
    return {
      id: overrides.id || `violation-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      ruleId: overrides.ruleId || "test-rule",
      severity: overrides.severity || ViolationSeverity.MODERATE,
      description: overrides.description || "Test violation description",
      evidence: overrides.evidence || ["test evidence"],
      detectedAt: overrides.detectedAt || new Date(),
      violator: overrides.violator || "test-agent",
      context: overrides.context || {},
      location: overrides.location,
      ...overrides,
    };
  };

  describe("Edge Case 1-5: Boundary Conditions", () => {
    it("should handle rules with empty required evidence", async () => {
      const rule = createRule({
        id: "rule-empty-evidence",
        requiredEvidence: [], // Empty evidence requirements
      });

      const violation = createViolation({
        ruleId: rule.id,
        evidence: [], // No evidence provided
      });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      // Should still work with no evidence requirements
      await orchestrator.evaluateRules(session.id);

      const updated = orchestrator.getSession(session.id);
      expect(updated.metadata.ruleEvaluationResults).toBeDefined();
      expect(updated.metadata.ruleEvaluationResults[0].violated).toBeDefined();

      await orchestrator.completeSession(session.id);
    });

    it("should handle extremely long rule descriptions and titles", async () => {
      const longTitle = "A".repeat(1000);
      const longDescription = "B".repeat(5000);

      const rule = createRule({
        title: longTitle,
        description: longDescription,
      });

      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(final.verdict!.reasoning.length).toBeGreaterThan(0);
    });

    it("should handle Unicode and special characters in rule content", async () => {
      const rule = createRule({
        id: "rule-unicode-🚀",
        title: "Rule with emoji 🚀 and unicode 🔥",
        description: "Description with special chars: àáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ",
        condition: "test ✅ === true",
      });

      const violation = createViolation({
        ruleId: rule.id,
        description: "Violation with unicode: 中文 español français русский",
        evidence: ["evidence 📊 with emoji"],
      });

      const session = await orchestrator.startSession(violation, [rule], ["agent-🤖"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-🎯");
      await orchestrator.completeSession(session.id);

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle rules with future effective dates", async () => {
      const futureDate = new Date();
      futureDate.setFullYear(futureDate.getFullYear() + 1);

      const rule = createRule({
        effectiveDate: futureDate, // Rule not yet effective
      });

      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);

      const updated = orchestrator.getSession(session.id);
      expect(updated.metadata.ruleEvaluationResults[0].violated).toBe(false);
      expect(updated.metadata.ruleEvaluationResults[0].explanation).toContain("not yet effective");

      await orchestrator.completeSession(session.id);
    });

    it("should handle expired rules correctly", async () => {
      const pastDate = new Date();
      pastDate.setFullYear(pastDate.getFullYear() - 1);

      const rule = createRule({
        expirationDate: pastDate, // Rule has expired
      });

      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);

      const updated = orchestrator.getSession(session.id);
      expect(updated.metadata.ruleEvaluationResults[0].violated).toBe(false);
      expect(updated.metadata.ruleEvaluationResults[0].explanation).toContain("expired");

      await orchestrator.completeSession(session.id);
    });
  });

  describe("Edge Case 6-10: Concurrent Operations", () => {
    it("should handle exactly maximum concurrent sessions", async () => {
      const maxConcurrent = 3;
      const limitedOrchestrator = new ArbitrationOrchestrator({
        maxConcurrentSessions: maxConcurrent,
      });

      const sessions: string[] = [];

      // Create exactly max concurrent sessions
      for (let i = 0; i < maxConcurrent; i++) {
        const rule = createRule({ id: `rule-concurrent-${i}` });
        const violation = createViolation({ ruleId: rule.id });

        const session = await limitedOrchestrator.startSession(
          violation,
          [rule],
          [`agent-${i}`]
        );
        sessions.push(session.id);
      }

      // Verify all were created
      expect(limitedOrchestrator.getActiveSessions()).toHaveLength(maxConcurrent);

      // Try one more - should fail
      const rule = createRule({ id: "rule-fail" });
      const violation = createViolation({ ruleId: rule.id });

      await expect(
        limitedOrchestrator.startSession(violation, [rule], ["agent-fail"])
      ).rejects.toThrow("Maximum concurrent sessions reached");

      // Clean up
      for (const sessionId of sessions) {
        await limitedOrchestrator.completeSession(sessionId);
      }

      limitedOrchestrator.clear();
    });

    it("should handle rapid session creation and completion", async () => {
      const sessionIds: string[] = [];

      // Rapidly create and complete sessions
      for (let i = 0; i < 10; i++) {
        const rule = createRule({ id: `rule-rapid-${i}` });
        const violation = createViolation({ ruleId: rule.id });

        const session = await orchestrator.startSession(
          violation,
          [rule],
          [`agent-${i}`]
        );

        await orchestrator.evaluateRules(session.id);
        await orchestrator.generateVerdict(session.id, "arbiter-1");
        await orchestrator.completeSession(session.id);

        sessionIds.push(session.id);
      }

      expect(orchestrator.getStatistics().totalSessions).toBe(10);
      expect(orchestrator.getStatistics().completedSessions).toBe(10);
      expect(orchestrator.getActiveSessions()).toHaveLength(0);
    });

    it("should handle interleaved operations across multiple sessions", async () => {
      const session1 = await orchestrator.startSession(
        createViolation({ ruleId: "rule-1" }),
        [createRule({ id: "rule-1" })],
        ["agent-1"]
      );

      const session2 = await orchestrator.startSession(
        createViolation({ ruleId: "rule-2" }),
        [createRule({ id: "rule-2" })],
        ["agent-2"]
      );

      // Interleave operations
      await orchestrator.evaluateRules(session1.id);
      await orchestrator.evaluateRules(session2.id);
      await orchestrator.generateVerdict(session1.id, "arbiter-1");
      await orchestrator.generateVerdict(session2.id, "arbiter-1");
      await orchestrator.completeSession(session2.id);
      await orchestrator.completeSession(session1.id);

      expect(orchestrator.getSession(session1.id).state).toBe(ArbitrationState.COMPLETED);
      expect(orchestrator.getSession(session2.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle session operations while other sessions are failing", async () => {
      const session1 = await orchestrator.startSession(
        createViolation({ ruleId: "rule-1" }),
        [createRule({ id: "rule-1" })],
        ["agent-1"]
      );

      const session2 = await orchestrator.startSession(
        createViolation({ ruleId: "rule-2" }),
        [createRule({ id: "rule-2" })],
        ["agent-2"]
      );

      // Fail session 2
      await orchestrator.failSession(session2.id, new Error("Test failure"));

      // Session 1 should still work
      await orchestrator.evaluateRules(session1.id);
      await orchestrator.generateVerdict(session1.id, "arbiter-1");
      await orchestrator.completeSession(session1.id);

      expect(orchestrator.getSession(session1.id).state).toBe(ArbitrationState.COMPLETED);
      expect(orchestrator.getSession(session2.id).state).toBe(ArbitrationState.FAILED);

      expect(orchestrator.getStatistics().completedSessions).toBe(1);
      expect(orchestrator.getStatistics().failedSessions).toBe(1);
    });

    it("should handle concurrent waiver requests", async () => {
      const session1 = await orchestrator.startSession(
        createViolation({ ruleId: "rule-1" }),
        [createRule({ id: "rule-1", waivable: true })],
        ["agent-1"]
      );

      const session2 = await orchestrator.startSession(
        createViolation({ ruleId: "rule-2" }),
        [createRule({ id: "rule-2", waivable: true })],
        ["agent-2"]
      );

      await orchestrator.evaluateRules(session1.id);
      await orchestrator.evaluateRules(session2.id);
      await orchestrator.generateVerdict(session1.id, "arbiter-1");
      await orchestrator.generateVerdict(session2.id, "arbiter-1");

      // Submit waivers concurrently
      const waiver1: WaiverRequest = {
        id: "waiver-1",
        ruleId: "rule-1",
        requestedBy: "agent-1",
        justification: "Concurrent waiver 1",
        evidence: ["evidence-1"],
        requestedDuration: 86400000,
        requestedAt: new Date(),
        context: {},
      };

      const waiver2: WaiverRequest = {
        id: "waiver-2",
        ruleId: "rule-2",
        requestedBy: "agent-2",
        justification: "Concurrent waiver 2",
        evidence: ["evidence-2"],
        requestedDuration: 86400000,
        requestedAt: new Date(),
        context: {},
      };

      await Promise.all([
        orchestrator.evaluateWaiver(session1.id, waiver1, "arbiter-1"),
        orchestrator.evaluateWaiver(session2.id, waiver2, "arbiter-1"),
      ]);

      expect(orchestrator.getSession(session1.id).state).toBe(ArbitrationState.COMPLETED);
      expect(orchestrator.getSession(session2.id).state).toBe(ArbitrationState.COMPLETED);
    });
  });

  describe("Edge Case 11-15: Error Recovery and Resilience", () => {
    it("should recover from invalid state transitions", async () => {
      const session = await orchestrator.startSession(
        createViolation(),
        [createRule()],
        ["agent-1"]
      );

      // Try to generate verdict before evaluating rules
      await expect(
        orchestrator.generateVerdict(session.id, "arbiter-1")
      ).rejects.toThrow("Cannot generate verdict in state");

      // Session should still be accessible and in valid state
      const current = orchestrator.getSession(session.id);
      expect(current.state).toBe(ArbitrationState.RULE_EVALUATION);

      // Should be able to continue with correct flow
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle invalid session IDs gracefully", async () => {
      const invalidIds = ["", "non-existent", "invalid-id", null, undefined];

      for (const invalidId of invalidIds) {
        if (invalidId !== null && invalidId !== undefined) {
          expect(() => orchestrator.getSession(invalidId)).toThrow("not found");

          await expect(
            orchestrator.evaluateRules(invalidId)
          ).rejects.toThrow("not found");

          await expect(
            orchestrator.generateVerdict(invalidId, "arbiter")
          ).rejects.toThrow("not found");
        }
      }
    });

    it("should handle malformed rule data", async () => {
      const malformedRules = [
        { ...createRule(), id: "" }, // Empty ID
        { ...createRule(), version: "" }, // Empty version
        { ...createRule(), category: "invalid" as any }, // Invalid category
        { ...createRule(), severity: "invalid" as any }, // Invalid severity
      ];

      for (const malformedRule of malformedRules) {
        const violation = createViolation({ ruleId: malformedRule.id });

        await expect(
          orchestrator.startSession(violation, [malformedRule], ["agent-1"])
        ).rejects.toThrow();
      }
    });

    it("should handle malformed violation data", async () => {
      const rule = createRule();
      const malformedViolations = [
        { ...createViolation(), id: "" }, // Empty ID
        { ...createViolation(), ruleId: "" }, // Empty rule ID
        { ...createViolation(), detectedAt: new Date("invalid") }, // Invalid date
        { ...createViolation(), severity: "invalid" as any }, // Invalid severity
      ];

      for (const malformedViolation of malformedViolations) {
        await expect(
          orchestrator.startSession(malformedViolation, [rule], ["agent-1"])
        ).rejects.toThrow();
      }
    });

    it("should recover from component failures", async () => {
      const session = await orchestrator.startSession(
        createViolation(),
        [createRule()],
        ["agent-1"]
      );

      // Simulate component failure by clearing components
      orchestrator.clear();

      // Operations should fail gracefully
      await expect(
        orchestrator.evaluateRules(session.id)
      ).rejects.toThrow();

      // But orchestrator should be in a recoverable state
      expect(() => orchestrator.getActiveSessions()).not.toThrow();

      // Should be able to start new sessions after clear
      const newSession = await orchestrator.startSession(
        createViolation({ ruleId: "new-rule" }),
        [createRule({ id: "new-rule" })],
        ["agent-2"]
      );

      expect(newSession.id).toBeDefined();
      await orchestrator.completeSession(newSession.id);
    });
  });

  describe("Edge Case 16-20: Performance and Resource Limits", () => {
    it("should handle large numbers of precedents", async () => {
      const rule = createRule();
      const violation = createViolation({ ruleId: rule.id });

      // Create many precedents
      const precedents = Array.from({ length: 50 }, (_, i) => ({
        id: `precedent-${i}`,
        title: `Precedent ${i}`,
        rulesInvolved: [rule.id],
        verdict: {
          id: `verdict-${i}`,
          sessionId: `session-${i}`,
          outcome: VerdictOutcome.REJECTED,
          reasoning: [{ step: 1, description: `Reasoning ${i}`, evidence: [], ruleReferences: [rule.id], confidence: 0.8 }],
          rulesApplied: [rule.id],
          evidence: [`evidence-${i}`],
          precedents: [],
          conditions: undefined,
          confidence: 0.8,
          issuedBy: "arbiter",
          issuedAt: new Date(),
          auditLog: [],
        },
        keyFacts: [`Fact ${i}`],
        reasoningSummary: `Summary ${i}`,
        applicability: {
          category: rule.category,
          severity: rule.severity,
          conditions: [`condition-${i}`],
        },
        citationCount: i,
        lastCitedAt: new Date(),
        createdAt: new Date(),
        metadata: {},
      }));

      // Load precedents into the precedent manager
      const { precedentManager } = orchestrator.getComponents();
      for (const precedent of precedents) {
        precedentManager.createPrecedent(
          precedent.verdict,
          precedent.title,
          precedent.keyFacts,
          precedent.reasoningSummary,
          precedent.applicability
        );
      }

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      // Should handle precedent lookup with many precedents
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle very long evidence lists", async () => {
      const longEvidence = Array.from({ length: 100 }, (_, i) => `Evidence item ${i} with substantial content that makes it quite long and detailed for testing purposes. This evidence contains important information about the violation and should be properly processed by the arbitration system.`.repeat(5));

      const rule = createRule();
      const violation = createViolation({
        ruleId: rule.id,
        evidence: longEvidence,
      });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(final.evidence).toHaveLength(100);
    });

    it("should handle deep metadata objects", async () => {
      const deepMetadata = {
        level1: {
          level2: {
            level3: {
              level4: {
                level5: {
                  data: "deep value",
                  array: [1, 2, { nested: "object" }],
                  date: new Date(),
                },
              },
            },
          },
        },
      };

      const rule = createRule({ metadata: deepMetadata });
      const violation = createViolation({
        ruleId: rule.id,
        context: deepMetadata,
      });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(final.metadata.ruleEvaluationResults).toBeDefined();
    });

    it("should handle zero-duration waiver requests", async () => {
      const rule = createRule({ waivable: true });
      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const zeroDurationWaiver: WaiverRequest = {
        id: "waiver-zero-duration",
        ruleId: rule.id,
        requestedBy: "agent-1",
        justification: "Zero duration waiver for testing",
        evidence: ["test evidence"],
        requestedDuration: 0, // Zero duration
        requestedAt: new Date(),
        context: {},
      };

      await orchestrator.evaluateWaiver(session.id, zeroDurationWaiver, "arbiter-1");

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(final.waiverRequest!.requestedDuration).toBe(0);
    });

    it("should handle very old violation timestamps", async () => {
      const oldDate = new Date();
      oldDate.setFullYear(oldDate.getFullYear() - 10); // 10 years ago

      const rule = createRule();
      const violation = createViolation({
        ruleId: rule.id,
        detectedAt: oldDate,
      });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(final.violation.detectedAt).toEqual(oldDate);
    });
  });

  describe("Edge Case 21-25: State Machine and Workflow Edge Cases", () => {
    it("should handle waiver evaluation in wrong state", async () => {
      const rule = createRule({ waivable: true });
      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      // Try waiver before verdict
      const waiverRequest: WaiverRequest = {
        id: "early-waiver",
        ruleId: rule.id,
        requestedBy: "agent-1",
        justification: "Early waiver attempt",
        evidence: ["evidence"],
        requestedDuration: 86400000,
        requestedAt: new Date(),
        context: {},
      };

      await expect(
        orchestrator.evaluateWaiver(session.id, waiverRequest, "arbiter-1")
      ).rejects.toThrow("Waiver system is disabled");

      // Should still be able to continue normally
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle appeal submission without verdict", async () => {
      const rule = createRule();
      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      // Try appeal without verdict
      await expect(
        orchestrator.submitAppeal(session.id, "agent-1", "No verdict appeal", ["evidence"])
      ).rejects.toThrow("Cannot appeal session without verdict");

      // Should still be able to continue normally
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle multiple state transition attempts", async () => {
      const session = await orchestrator.startSession(
        createViolation(),
        [createRule()],
        ["agent-1"]
      );

      // Try to evaluate rules multiple times
      await orchestrator.evaluateRules(session.id);

      await expect(
        orchestrator.evaluateRules(session.id)
      ).rejects.toThrow("Cannot evaluate rules in state");

      // Should still be able to continue
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle forced state transitions (admin operations)", async () => {
      const session = await orchestrator.startSession(
        createViolation(),
        [createRule()],
        ["agent-1"]
      );

      // Force complete session before normal flow
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Should not be able to perform further operations
      await expect(
        orchestrator.evaluateRules(session.id)
      ).rejects.toThrow("Cannot evaluate rules in state");

      await expect(
        orchestrator.generateVerdict(session.id, "arbiter-1")
      ).rejects.toThrow("Cannot generate verdict in state");
    });

    it("should handle appeal review without appeal", async () => {
      const rule = createRule();
      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      // Try to review non-existent appeal
      await expect(
        orchestrator.reviewAppeal(session.id, "non-existent-appeal", ["reviewer"])
      ).rejects.toThrow("Appeal not found");

      // Session should remain completed
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });
  });

  describe("Edge Case 26-30: Data Validation and Sanitization", () => {
    it("should handle rules with duplicate IDs", async () => {
      const rule1 = createRule({ id: "duplicate-rule" });
      const rule2 = createRule({ id: "duplicate-rule" }); // Same ID

      const violation1 = createViolation({ ruleId: "duplicate-rule" });
      const violation2 = createViolation({ ruleId: "duplicate-rule" });

      // First session should work
      const session1 = await orchestrator.startSession(
        violation1,
        [rule1],
        ["agent-1"]
      );

      // Second session with same rule ID should also work (different sessions)
      const session2 = await orchestrator.startSession(
        violation2,
        [rule2],
        ["agent-2"]
      );

      expect(session1.id).not.toBe(session2.id);

      await orchestrator.completeSession(session1.id);
      await orchestrator.completeSession(session2.id);

      expect(orchestrator.getStatistics().totalSessions).toBe(2);
    });

    it("should handle violations with null/undefined fields", async () => {
      const rule = createRule();
      const violation = createViolation({
        ruleId: rule.id,
        violator: undefined, // Null violator
        location: undefined, // Null location
        context: undefined, // Null context
      });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(final.violation.violator).toBeUndefined();
      expect(final.violation.location).toBeUndefined();
    });

    it("should handle waiver requests with missing optional fields", async () => {
      const rule = createRule({ waivable: true });
      const violation = createViolation({ ruleId: rule.id });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      const minimalWaiver: WaiverRequest = {
        id: "minimal-waiver",
        ruleId: rule.id,
        requestedBy: "agent-1",
        justification: "Minimal waiver",
        evidence: [], // Empty evidence
        requestedDuration: 86400000,
        requestedAt: new Date(),
        context: {}, // Empty context
      };

      await orchestrator.evaluateWaiver(session.id, minimalWaiver, "arbiter-1");

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(final.waiverRequest!.evidence).toHaveLength(0);
    });

    it("should handle empty participant lists", async () => {
      const rule = createRule();
      const violation = createViolation({ ruleId: rule.id });

      // This should fail validation, but let's see how it handles empty participants
      await expect(
        orchestrator.startSession(violation, [rule], [])
      ).rejects.toThrow();
    });

    it("should handle extremely large metadata objects", async () => {
      const largeMetadata = {};
      for (let i = 0; i < 1000; i++) {
        largeMetadata[`key${i}`] = `value${i}`.repeat(100); // Large values
      }

      const rule = createRule({ metadata: largeMetadata });
      const violation = createViolation({
        ruleId: rule.id,
        context: largeMetadata,
      });

      const session = await orchestrator.startSession(violation, [rule], ["agent-1"]);

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      const final = orchestrator.getSession(session.id);
      expect(final.state).toBe(ArbitrationState.COMPLETED);
      expect(Object.keys(final.metadata)).toBeDefined();
    });
  });
});
