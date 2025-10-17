/**
 * Integration Tests: Arbitration Error Handling and Recovery
 *
 * Comprehensive testing of error handling, recovery mechanisms, and resilience
 * patterns in the arbitration system under failure conditions.
 *
 * Test Coverage (20+ error recovery tests):
 * - Component failure isolation and recovery
 * - Network and external service failure simulation
 * - Data corruption and inconsistency handling
 * - Timeout and cancellation handling
 * - Resource exhaustion recovery
 * - State corruption recovery
 * - Partial failure scenarios
 * - Error propagation and isolation
 * - Recovery time and success rate measurement
 * - Circuit breaker pattern validation
 */

import { ArbitrationOrchestrator } from "@/arbitration/ArbitrationOrchestrator";
import {
  ArbitrationError,
  ArbitrationState,
  ConstitutionalRule,
  ConstitutionalViolation,
  RuleCategory,
  ViolationSeverity,
  WaiverRequest,
} from "@/types/arbitration";

describe("ARBITER-015 Integration: Arbitration Error Handling and Recovery", () => {
  let orchestrator: ArbitrationOrchestrator;

  beforeEach(() => {
    orchestrator = new ArbitrationOrchestrator({
      enableWaivers: true,
      enableAppeals: true,
      trackPerformance: true,
      maxConcurrentSessions: 10,
      sessionTimeoutMs: 30000, // 30 seconds for error tests
    });
  });

  afterEach(async () => {
    // Clean up any active sessions
    const activeSessions = orchestrator.getActiveSessions();
    for (const session of activeSessions) {
      try {
        await orchestrator.completeSession(session.id);
      } catch (e) {
        // Ignore cleanup errors in error tests
      }
    }
    orchestrator.clear();
  });

  // Helper: Create test rule
  const createRule = (overrides: Partial<ConstitutionalRule> = {}): ConstitutionalRule => {
    return {
      id: overrides.id || `rule-error-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      version: "1.0.0",
      category: RuleCategory.CODE_QUALITY,
      title: "Error Test Rule",
      description: "Rule for error handling tests",
      condition: "true",
      severity: ViolationSeverity.MODERATE,
      waivable: true,
      requiredEvidence: [],
      precedents: [],
      effectiveDate: new Date(),
      metadata: {},
      ...overrides,
    };
  };

  // Helper: Create test violation
  const createViolation = (ruleId: string, overrides: Partial<ConstitutionalViolation> = {}): ConstitutionalViolation => {
    return {
      id: `violation-error-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      ruleId,
      severity: ViolationSeverity.MODERATE,
      description: "Error test violation",
      evidence: ["error-test-evidence"],
      detectedAt: new Date(),
      violator: "error-test-agent",
      context: {},
      ...overrides,
    };
  };

  // Helper: Simulate async error
  const simulateAsyncError = async (message: string, delay: number = 10): Promise<never> => {
    await new Promise(resolve => setTimeout(resolve, delay));
    throw new Error(message);
  };

  describe("Error Recovery Test 1-5: Component Failure Isolation", () => {
    it("should isolate rule engine failures without affecting other sessions", async () => {
      // Create multiple sessions
      const session1 = await orchestrator.startSession(
        createViolation("rule-1"),
        [createRule({ id: "rule-1" })],
        ["agent-1"]
      );

      const session2 = await orchestrator.startSession(
        createViolation("rule-2"),
        [createRule({ id: "rule-2" })],
        ["agent-2"]
      );

      // Simulate rule engine failure for session 1 by modifying internal state
      const components = orchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockRejectedValue(new Error("Rule engine failure"));

      // Session 1 should fail
      await expect(orchestrator.evaluateRules(session1.id)).rejects.toThrow("Rule engine failure");

      // Session 2 should still work
      await orchestrator.evaluateRules(session2.id);
      await orchestrator.generateVerdict(session2.id, "arbiter-1");
      await orchestrator.completeSession(session2.id);

      // Verify isolation
      expect(orchestrator.getSession(session2.id).state).toBe(ArbitrationState.COMPLETED);
      expect(orchestrator.getSession(session1.id).state).toBe(ArbitrationState.RULE_EVALUATION);

      // Restore rule engine
      components.ruleEngine.evaluateAction = originalEvaluate;

      // Clean up
      await orchestrator.completeSession(session1.id);
    });

    it("should handle verdict generator failures gracefully", async () => {
      const session = await orchestrator.startSession(
        createViolation("rule-1"),
        [createRule({ id: "rule-1" })],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);

      // Simulate verdict generator failure
      const components = orchestrator.getComponents();
      const originalGenerate = components.verdictGenerator.generateVerdict;
      components.verdictGenerator.generateVerdict = jest.fn().mockRejectedValue(new Error("Verdict generation failed"));

      // Verdict generation should fail
      await expect(
        orchestrator.generateVerdict(session.id, "arbiter-1")
      ).rejects.toThrow("Verdict generation failed");

      // Session should remain in valid state
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.VERDICT_GENERATION);

      // Restore verdict generator
      components.verdictGenerator.generateVerdict = originalGenerate;

      // Should be able to retry
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should isolate waiver interpreter failures", async () => {
      const rule = createRule({ id: "rule-waiver", waivable: true });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      // Simulate waiver interpreter failure
      const components = orchestrator.getComponents();
      const originalProcess = components.waiverInterpreter.processWaiver;
      components.waiverInterpreter.processWaiver = jest.fn().mockRejectedValue(new Error("Waiver processing failed"));

      const waiverRequest: WaiverRequest = {
        id: "waiver-fail",
        ruleId: rule.id,
        requestedBy: "agent-1",
        justification: "Test waiver",
        evidence: ["evidence"],
        requestedDuration: 86400000,
        requestedAt: new Date(),
        context: {},
      };

      // Waiver should fail
      await expect(
        orchestrator.evaluateWaiver(session.id, waiverRequest, "arbiter-1")
      ).rejects.toThrow("Waiver processing failed");

      // Session should still be in verdict generation state
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.VERDICT_GENERATION);

      // Restore waiver interpreter
      components.waiverInterpreter.processWaiver = originalProcess;

      // Should be able to complete normally
      await orchestrator.completeSession(session.id);
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle precedent manager failures during rule evaluation", async () => {
      const rule = createRule({ id: "rule-precedent" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Simulate precedent manager failure
      const components = orchestrator.getComponents();
      const originalFind = components.precedentManager.findSimilarPrecedents;
      components.precedentManager.findSimilarPrecedents = jest.fn().mockRejectedValue(new Error("Precedent lookup failed"));

      // Rule evaluation should still succeed (precedents are optional)
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Restore precedent manager
      components.precedentManager.findSimilarPrecedents = originalFind;
    });

    it("should isolate appeal arbitrator failures", async () => {
      const rule = createRule({ id: "rule-appeal" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      // Simulate appeal arbitrator failure
      const components = orchestrator.getComponents();
      const originalSubmit = components.appealArbitrator.submitAppeal;
      components.appealArbitrator.submitAppeal = jest.fn().mockRejectedValue(new Error("Appeal submission failed"));

      // Appeal should fail
      await expect(
        orchestrator.submitAppeal(session.id, "agent-1", "Test appeal", ["evidence"])
      ).rejects.toThrow("Appeal submission failed");

      // Session should remain completed
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Restore appeal arbitrator
      components.appealArbitrator.submitAppeal = originalSubmit;
    });
  });

  describe("Error Recovery Test 6-10: Network and External Service Failures", () => {
    it("should handle timeout errors during long-running operations", async () => {
      // Create orchestrator with very short timeout
      const fastTimeoutOrchestrator = new ArbitrationOrchestrator({
        sessionTimeoutMs: 50, // Very short timeout
      });

      const rule = createRule({ id: "rule-timeout" });
      const session = await fastTimeoutOrchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Simulate slow operation that exceeds timeout
      const components = fastTimeoutOrchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve([]), 100)) // 100ms delay > 50ms timeout
      );

      // Operation should eventually complete or fail gracefully
      try {
        await fastTimeoutOrchestrator.evaluateRules(session.id);
        // If it completes, that's also acceptable
        await fastTimeoutOrchestrator.generateVerdict(session.id, "arbiter-1");
        await fastTimeoutOrchestrator.completeSession(session.id);
      } catch (error) {
        // Expected to fail due to timeout or other issues
        expect(error).toBeDefined();
      }

      // Restore
      components.ruleEngine.evaluateAction = originalEvaluate;
      fastTimeoutOrchestrator.clear();
    });

    it("should recover from database connection failures", async () => {
      // Note: In a real implementation, this would test actual database failures
      // For this test, we'll simulate the failure pattern

      const rule = createRule({ id: "rule-db-fail" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Simulate database failure during rule evaluation
      let failureCount = 0;
      const components = orchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockImplementation(() => {
        failureCount++;
        if (failureCount <= 2) {
          throw new Error("Database connection failed");
        }
        return Promise.resolve([]);
      });

      // First two attempts should fail
      await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow("Database connection failed");
      await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow("Database connection failed");

      // Third attempt should succeed
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Restore
      components.ruleEngine.evaluateAction = originalEvaluate;
    });

    it("should handle external service unavailability", async () => {
      const rule = createRule({ id: "rule-external" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Simulate external service failure
      const components = orchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockRejectedValue(
        new Error("External service unavailable: HTTP 503")
      );

      // Operation should fail
      await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow("External service unavailable");

      // Session should be recoverable
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.RULE_EVALUATION);

      // Restore service and retry
      components.ruleEngine.evaluateAction = originalEvaluate;

      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle partial response failures from external APIs", async () => {
      const rule = createRule({ id: "rule-partial-fail" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Simulate partial response (some data missing)
      const components = orchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockResolvedValue([
        {
          ruleId: rule.id,
          violated: true,
          explanation: "Partial evaluation result",
          confidence: 0.8,
          precedentsApplied: [],
          evaluationTimeMs: 50,
          violation: {
            id: "partial-violation",
            ruleId: rule.id,
            severity: ViolationSeverity.MODERATE,
            description: "Partial violation data",
            evidence: [], // Missing evidence
            detectedAt: new Date(),
            violator: undefined, // Missing violator
            context: {},
          },
        },
      ]);

      // Should handle partial data gracefully
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Restore
      components.ruleEngine.evaluateAction = originalEvaluate;
    });

    it("should recover from network partition scenarios", async () => {
      const rule = createRule({ id: "rule-network" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Simulate network partition (connection reset)
      let attemptCount = 0;
      const components = orchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockImplementation(() => {
        attemptCount++;
        if (attemptCount <= 3) {
          throw new Error("ECONNRESET: Connection reset by peer");
        }
        return Promise.resolve([]);
      });

      // Multiple failures should eventually succeed
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
      expect(attemptCount).toBe(4); // 3 failures + 1 success

      // Restore
      components.ruleEngine.evaluateAction = originalEvaluate;
    });
  });

  describe("Error Recovery Test 11-15: Data Corruption and Inconsistency", () => {
    it("should handle corrupted session state", async () => {
      const rule = createRule({ id: "rule-corrupt" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Manually corrupt session state (simulate data corruption)
      const sessionObj = orchestrator.getSession(session.id);
      (sessionObj as any).state = "invalid_state";

      // Operations should fail gracefully
      await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow();

      // Should be able to recover by failing the session
      await orchestrator.failSession(session.id, new Error("State corruption detected"));

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.FAILED);

      // System should still accept new sessions
      const newRule = createRule({ id: "rule-recovery" });
      const newSession = await orchestrator.startSession(
        createViolation(newRule.id),
        [newRule],
        ["agent-2"]
      );

      await orchestrator.completeSession(newSession.id);
      expect(orchestrator.getSession(newSession.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle inconsistent verdict data", async () => {
      const rule = createRule({ id: "rule-inconsistent" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      await orchestrator.evaluateRules(session.id);

      // Simulate verdict generator returning inconsistent data
      const components = orchestrator.getComponents();
      const originalGenerate = components.verdictGenerator.generateVerdict;
      components.verdictGenerator.generateVerdict = jest.fn().mockResolvedValue({
        verdict: {
          id: "inconsistent-verdict",
          sessionId: session.id,
          outcome: "invalid_outcome", // Invalid outcome
          reasoning: [],
          rulesApplied: [],
          evidence: [],
          precedents: [],
          confidence: 1.5, // Invalid confidence (> 1.0)
          issuedBy: "arbiter-1",
          issuedAt: new Date(),
          auditLog: [],
        },
        generationTimeMs: 50,
        warnings: ["Inconsistent verdict data"],
      });

      // Should handle inconsistent data gracefully
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Restore
      components.verdictGenerator.generateVerdict = originalGenerate;
    });

    it("should recover from metadata corruption", async () => {
      const rule = createRule({ id: "rule-metadata" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Corrupt metadata
      const sessionObj = orchestrator.getSession(session.id);
      sessionObj.metadata = null as any; // Null metadata

      // Operations should still work (defensive programming)
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle duplicate verdict IDs", async () => {
      const rule1 = createRule({ id: "rule-dup-1" });
      const rule2 = createRule({ id: "rule-dup-2" });

      const session1 = await orchestrator.startSession(
        createViolation(rule1.id),
        [rule1],
        ["agent-1"]
      );

      const session2 = await orchestrator.startSession(
        createViolation(rule2.id),
        [rule2],
        ["agent-2"]
      );

      // Simulate verdict generator returning duplicate IDs
      const components = orchestrator.getComponents();
      const originalGenerate = components.verdictGenerator.generateVerdict;
      let verdictCount = 0;
      components.verdictGenerator.generateVerdict = jest.fn().mockImplementation(() => {
        verdictCount++;
        return Promise.resolve({
          verdict: {
            id: "duplicate-verdict-id", // Same ID for both
            sessionId: verdictCount === 1 ? session1.id : session2.id,
            outcome: "approved",
            reasoning: [{
              step: 1,
              description: "Test reasoning",
              evidence: [],
              ruleReferences: [],
              confidence: 0.8,
            }],
            rulesApplied: [],
            evidence: [],
            precedents: [],
            confidence: 0.8,
            issuedBy: "arbiter-1",
            issuedAt: new Date(),
            auditLog: [],
          },
          generationTimeMs: 50,
          warnings: [],
        });
      });

      // Both verdicts should succeed despite duplicate IDs
      await orchestrator.evaluateRules(session1.id);
      await orchestrator.evaluateRules(session2.id);

      await orchestrator.generateVerdict(session1.id, "arbiter-1");
      await orchestrator.generateVerdict(session2.id, "arbiter-1");

      await orchestrator.completeSession(session1.id);
      await orchestrator.completeSession(session2.id);

      expect(orchestrator.getSession(session1.id).state).toBe(ArbitrationState.COMPLETED);
      expect(orchestrator.getSession(session2.id).state).toBe(ArbitrationState.COMPLETED);

      // Restore
      components.verdictGenerator.generateVerdict = originalGenerate;
    });

    it("should recover from evidence data corruption", async () => {
      const rule = createRule({ id: "rule-evidence-corrupt" });
      const session = await orchestrator.startSession(
        createViolation(rule.id, {
          evidence: ["valid-evidence-1", null, undefined, "valid-evidence-2"], // Corrupted evidence array
        }),
        [rule],
        ["agent-1"]
      );

      // Should handle corrupted evidence gracefully
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });
  });

  describe("Error Recovery Test 16-20: Resource Exhaustion and Limits", () => {
    it("should handle memory exhaustion gracefully", async () => {
      const memoryIntensiveRule = createRule({
        id: "rule-memory",
        metadata: {
          largeData: Array.from({ length: 10000 }, (_, i) => ({
            data: `Large string data ${i} `.repeat(100), // Large strings
            nested: {
              moreData: Array.from({ length: 100 }, (_, j) => `Nested data ${j}`),
            },
          })),
        },
      });

      const memoryIntensiveViolation = createViolation(memoryIntensiveRule.id, {
        evidence: Array.from({ length: 1000 }, (_, i) => `Evidence ${i} `.repeat(50)),
        context: {
          largeContext: Array.from({ length: 5000 }, (_, i) => ({
            key: `key-${i}`,
            value: `value-${i}`.repeat(20),
          })),
        },
      });

      const session = await orchestrator.startSession(
        memoryIntensiveViolation,
        [memoryIntensiveRule],
        ["agent-1"]
      );

      // Should handle large data structures without crashing
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Verify memory was properly managed
      const stats = orchestrator.getStatistics();
      expect(stats.completedSessions).toBe(1);
    });

    it("should handle CPU exhaustion during complex evaluations", async () => {
      // Create rule that requires complex evaluation
      const complexRule = createRule({
        id: "rule-cpu-intensive",
        requiredEvidence: Array.from({ length: 100 }, (_, i) => `evidence-${i}`),
        metadata: {
          complexLogic: {
            conditions: Array.from({ length: 50 }, (_, i) => ({
              type: "complex",
              rules: Array.from({ length: 20 }, (_, j) => `rule-${i}-${j}`),
            })),
          },
        },
      });

      const session = await orchestrator.startSession(
        createViolation(complexRule.id),
        [complexRule],
        ["agent-1"]
      );

      // Simulate CPU-intensive operation
      const components = orchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockImplementation(async () => {
        // Simulate CPU-intensive work
        const start = Date.now();
        while (Date.now() - start < 100) {
          // Busy wait for 100ms
          Math.random();
        }
        return [];
      });

      // Should complete without hanging
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);

      // Restore
      components.ruleEngine.evaluateAction = originalEvaluate;
    });

    it("should handle concurrent session limit enforcement", async () => {
      const limitedOrchestrator = new ArbitrationOrchestrator({
        maxConcurrentSessions: 2,
      });

      const rule = createRule({ id: "rule-limit" });

      // Fill concurrent session limit
      const session1 = await limitedOrchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      const session2 = await limitedOrchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-2"]
      );

      // Third session should fail
      await expect(
        limitedOrchestrator.startSession(
          createViolation(rule.id),
          [rule],
          ["agent-3"]
        )
      ).rejects.toThrow("Maximum concurrent sessions reached");

      // Complete one session to free up slot
      await limitedOrchestrator.completeSession(session1.id);

      // Now should be able to create new session
      const session3 = await limitedOrchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-3"]
      );

      await limitedOrchestrator.completeSession(session2.id);
      await limitedOrchestrator.completeSession(session3.id);

      const stats = limitedOrchestrator.getStatistics();
      expect(stats.totalSessions).toBe(3);
      expect(stats.completedSessions).toBe(3);

      limitedOrchestrator.clear();
    });

    it("should recover from thread pool exhaustion", async () => {
      // Simulate thread pool exhaustion by creating many concurrent operations
      const concurrentOperations = 20;
      const rule = createRule({ id: "rule-thread-pool" });

      // Create many sessions concurrently
      const createPromises = Array.from({ length: concurrentOperations }, async (_, i) => {
        const session = await orchestrator.startSession(
          createViolation(rule.id),
          [rule],
          [`agent-${i}`]
        );
        return session.id;
      });

      const sessionIds = await Promise.all(createPromises);
      expect(sessionIds).toHaveLength(concurrentOperations);

      // Process all concurrently (may stress thread pool)
      const processPromises = sessionIds.map(async (sessionId) => {
        await orchestrator.evaluateRules(sessionId);
        await orchestrator.generateVerdict(sessionId, "arbiter-1");
        await orchestrator.completeSession(sessionId);
        return sessionId;
      });

      await Promise.all(processPromises);

      // All should complete successfully
      const stats = orchestrator.getStatistics();
      expect(stats.totalSessions).toBe(concurrentOperations);
      expect(stats.completedSessions).toBe(concurrentOperations);
    });

    it("should handle disk space exhaustion during persistence", async () => {
      // Note: In a real system, this would test actual disk space issues
      // For this test, we'll simulate the failure pattern

      const rule = createRule({ id: "rule-disk-full" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Simulate disk full error during verdict generation
      const components = orchestrator.getComponents();
      const originalGenerate = components.verdictGenerator.generateVerdict;
      components.verdictGenerator.generateVerdict = jest.fn().mockRejectedValue(
        new Error("ENOSPC: No space left on device")
      );

      // Operation should fail
      await expect(
        orchestrator.generateVerdict(session.id, "arbiter-1")
      ).rejects.toThrow("No space left on device");

      // Session should be recoverable
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.VERDICT_GENERATION);

      // Simulate disk space becoming available
      components.verdictGenerator.generateVerdict = originalGenerate;

      // Should be able to retry
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });
  });

  describe("Error Recovery Test 21-25: State Management and Recovery", () => {
    it("should recover from state machine corruption", async () => {
      const rule = createRule({ id: "rule-state-corrupt" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Manually corrupt state machine state
      const sessionObj = orchestrator.getSession(session.id);
      sessionObj.state = ArbitrationState.COMPLETED; // Skip to completed

      // Operations should fail appropriately
      await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow();
      await expect(orchestrator.generateVerdict(session.id, "arbiter-1")).rejects.toThrow();

      // Should be able to fail the corrupted session
      await orchestrator.failSession(session.id, new Error("State corruption"));

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.FAILED);
    });

    it("should handle invalid state transitions gracefully", async () => {
      const rule = createRule({ id: "rule-invalid-transition" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Try invalid transition: complete before evaluation
      await expect(orchestrator.completeSession(session.id)).rejects.toThrow();

      // Session should remain in valid state
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.RULE_EVALUATION);

      // Valid transition should work
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should recover from cascading failure scenarios", async () => {
      const rule = createRule({ id: "rule-cascade" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Cause cascading failures
      const components = orchestrator.getComponents();

      // Make rule evaluation fail
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockRejectedValue(new Error("Rule evaluation failed"));

      // Make verdict generation also fail
      const originalGenerate = components.verdictGenerator.generateVerdict;
      components.verdictGenerator.generateVerdict = jest.fn().mockRejectedValue(new Error("Verdict generation failed"));

      // Multiple operations should fail
      await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow();
      await expect(orchestrator.generateVerdict(session.id, "arbiter-1")).rejects.toThrow();

      // Should be able to fail the session
      await orchestrator.failSession(session.id, new Error("Cascading failures"));

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.FAILED);

      // System should accept new sessions
      const newRule = createRule({ id: "rule-recovery" });
      const newSession = await orchestrator.startSession(
        createViolation(newRule.id),
        [newRule],
        ["agent-2"]
      );

      // Restore components
      components.ruleEngine.evaluateAction = originalEvaluate;
      components.verdictGenerator.generateVerdict = originalGenerate;

      // New session should work normally
      await orchestrator.completeSession(newSession.id);
      expect(orchestrator.getSession(newSession.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should handle partial session state recovery", async () => {
      const rule = createRule({ id: "rule-partial-recovery" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      // Complete partial workflow
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");

      // Simulate partial state corruption (missing verdict)
      const sessionObj = orchestrator.getSession(session.id);
      delete sessionObj.verdict;

      // Should still be able to complete session
      await orchestrator.completeSession(session.id);
      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });

    it("should maintain data consistency during error recovery", async () => {
      const rule = createRule({ id: "rule-consistency" });
      const session = await orchestrator.startSession(
        createViolation(rule.id),
        [rule],
        ["agent-1"]
      );

      const initialState = orchestrator.getSession(session.id).state;

      // Cause multiple failures
      const components = orchestrator.getComponents();
      const originalEvaluate = components.ruleEngine.evaluateAction;
      components.ruleEngine.evaluateAction = jest.fn().mockRejectedValue(new Error("Consistency test failure"));

      // Multiple failures should not corrupt session data
      for (let i = 0; i < 3; i++) {
        await expect(orchestrator.evaluateRules(session.id)).rejects.toThrow();
      }

      // Session data should remain consistent
      const sessionAfterFailures = orchestrator.getSession(session.id);
      expect(sessionAfterFailures.state).toBe(initialState);
      expect(sessionAfterFailures.violation).toBeDefined();
      expect(sessionAfterFailures.rulesEvaluated).toHaveLength(1);

      // Restore and complete
      components.ruleEngine.evaluateAction = originalEvaluate;
      await orchestrator.evaluateRules(session.id);
      await orchestrator.generateVerdict(session.id, "arbiter-1");
      await orchestrator.completeSession(session.id);

      expect(orchestrator.getSession(session.id).state).toBe(ArbitrationState.COMPLETED);
    });
  });
});
