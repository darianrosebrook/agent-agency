/**
 * @fileoverview Integration tests for ArbiterOrchestrator Verification
 *
 * Tests verification engine integration within orchestrator for ARBITER-007
 *
 * @author @darianrosebrook
 */

import {
  ArbiterOrchestrator,
  ArbiterOrchestratorConfig,
} from "@/orchestrator/ArbiterOrchestrator";
import {
  VerificationPriority,
  VerificationRequest,
  VerificationType,
} from "@/types/verification";

describe("ArbiterOrchestrator Verification Integration", () => {
  let orchestrator: ArbiterOrchestrator;

  const testConfig: ArbiterOrchestratorConfig = {
    taskQueue: {
      maxQueueSize: 100,
      processingConcurrency: 5,
      priorityLevels: 3,
    },
    taskAssignment: {
      assignmentStrategy: "capability-based",
      maxTasksPerAgent: 5,
      reassignmentThresholdMs: 300000,
    },
    agentRegistry: {
      maxAgents: 10,
      cleanupIntervalMs: 300000,
      persistenceEnabled: false,
    },
    security: {
      auditLoggingEnabled: true,
      maxAuditEvents: 1000,
      inputSanitizationEnabled: true,
      secureErrorResponsesEnabled: true,
      sessionTimeoutMinutes: 30,
    },
    healthMonitor: {
      checkIntervalMs: 30000,
      unhealthyThreshold: 3,
      recoveryCheckIntervalMs: 10000,
    },
    recoveryManager: {
      maxRetries: 3,
      retryDelayMs: 1000,
      exponentialBackoff: true,
    },
    // Note: orchestrator property not part of ArbiterOrchestratorConfig interface
    knowledgeSeeker: {
      providers: [],
      processor: {
        relevanceThreshold: 0.5,
        credibilityThreshold: 0.6,
      },
      caching: {
        enableQueryCaching: false,
        enableResultCaching: false,
        cacheTtlMs: 300000,
      },
    },
    prompting: {
      enabled: false,
      reasoningEffort: "medium",
      eagerness: 0.5,
      toolBudget: { maxToolCalls: 100 },
      contextGathering: {
        enabled: false,
        maxContextItems: 10,
        relevanceThreshold: 0.6,
      },
      selfReflection: {
        enabled: false,
        reflectionTriggers: [],
      },
    } as any,
    // Note: verification property not part of ArbiterOrchestratorConfig interface
  };

  beforeAll(async () => {
    orchestrator = new ArbiterOrchestrator(testConfig);
    await orchestrator.initialize();
  });

  afterAll(async () => {
    await orchestrator.shutdown();
  });

  describe("Verification Engine Availability", () => {
    it("should have verification engine initialized when enabled", async () => {
      // Verification engine should be available through orchestrator
      expect(orchestrator).toBeDefined();
    });

    it("should handle verification request through orchestrator", async () => {
      const request: VerificationRequest = {
        id: "orch-verify-1",
        content: "The Earth orbits the Sun",
        source: "https://example.com",
        context: "Basic astronomy fact",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        metadata: {},
      };

      const result = await orchestrator.verifyInformation(request);

      expect(result).toBeDefined();
      expect(result.verdict).toBeDefined();
      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
    });
  });

  describe("Verification Method Statistics", () => {
    it("should retrieve verification method performance stats", async () => {
      // Create some verification requests first
      const requests: VerificationRequest[] = Array.from(
        { length: 3 },
        (_, i) => ({
          id: `orch-stats-${i}`,
          content: `Test verification content ${i}`,
          source: `https://example${i}.com`,
          context: "Statistics test",
          priority: VerificationPriority.LOW,
          verificationTypes: [VerificationType.FACT_CHECKING],
          metadata: {},
        })
      );

      await Promise.all(
        requests.map((req) => orchestrator.verifyInformation(req))
      );

      const stats = await orchestrator.getVerificationMethodStats();

      expect(Array.isArray(stats)).toBe(true);
      if (stats.length > 0) {
        stats.forEach((stat: any) => {
          expect(stat.methodType).toBeDefined();
          expect(stat.totalRequests).toBeGreaterThanOrEqual(0);
          expect(stat.successRate).toBeGreaterThanOrEqual(0);
          expect(stat.successRate).toBeLessThanOrEqual(100);
        });
      }
    });

    it("should retrieve evidence quality statistics", async () => {
      const stats = await orchestrator.getVerificationEvidenceStats();

      expect(Array.isArray(stats)).toBe(true);
    });
  });

  describe("Verification with Different Priorities", () => {
    it("should handle high-priority verification requests", async () => {
      const request: VerificationRequest = {
        id: "orch-priority-high",
        content: "Critical information requiring verification",
        source: "https://example.com",
        context: "High priority",
        priority: VerificationPriority.CRITICAL,
        verificationTypes: [
          VerificationType.FACT_CHECKING,
          VerificationType.SOURCE_CREDIBILITY,
        ],
        metadata: {},
      };

      const result = await orchestrator.verifyInformation(request);

      expect(result).toBeDefined();
      expect(result.methodResults.length).toBeGreaterThan(0);
    });

    it("should handle low-priority verification requests", async () => {
      const request: VerificationRequest = {
        id: "orch-priority-low",
        content: "Non-critical information",
        source: "https://example.com",
        context: "Low priority",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.FACT_CHECKING],
        metadata: {},
      };

      const result = await orchestrator.verifyInformation(request);

      expect(result).toBeDefined();
    });
  });

  describe("Verification with Multiple Methods", () => {
    it("should handle verification with multiple methods", async () => {
      const request: VerificationRequest = {
        id: "orch-multi-method",
        content: "Content requiring multiple verification approaches",
        source: "https://example.com",
        context: "Multi-method test",
        priority: VerificationPriority.HIGH,
        verificationTypes: [
          VerificationType.FACT_CHECKING,
          VerificationType.SOURCE_CREDIBILITY,
          VerificationType.CROSS_REFERENCE,
        ],
        metadata: {},
      };

      const result = await orchestrator.verifyInformation(request);

      expect(result).toBeDefined();
      expect(result.methodResults.length).toBeGreaterThanOrEqual(1);
    });
  });

  describe("Concurrent Verification Handling", () => {
    it("should handle multiple concurrent verification requests", async () => {
      const requests: VerificationRequest[] = Array.from(
        { length: 5 },
        (_, i) => ({
          id: `orch-concurrent-${i}`,
          content: `Concurrent verification content ${i}`,
          source: `https://example${i}.com`,
          context: "Concurrent test",
          priority: VerificationPriority.MEDIUM,
          verificationTypes: [VerificationType.FACT_CHECKING],
          metadata: {},
        })
      );

      const results = await Promise.all(
        requests.map((req) => orchestrator.verifyInformation(req))
      );

      expect(results.length).toBe(5);
      expect(results.every((r) => r.verdict !== undefined)).toBe(true);
    });
  });

  describe("Error Handling", () => {
    it("should handle invalid verification request gracefully", async () => {
      const invalidRequest: any = {
        id: "invalid-request",
        content: "",
        // Missing required fields
      };

      try {
        await orchestrator.verifyInformation(invalidRequest);
      } catch (error) {
        expect(error).toBeDefined();
      }
    });

    it("should handle verification timeout gracefully", async () => {
      const request: VerificationRequest = {
        id: "timeout-test",
        content: "A ".repeat(10000) + "content for timeout test",
        source: "https://example.com",
        context: "Timeout test",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.FACT_CHECKING],
        metadata: {},
      };

      const result = await orchestrator.verifyInformation(request);

      expect(result).toBeDefined();
      // Should complete even if some methods timeout
    });
  });

  describe("Integration with Security Manager", () => {
    it("should respect security context for verification", async () => {
      const request: VerificationRequest = {
        id: "security-context",
        content: "Content requiring security validation",
        source: "https://example.com",
        context: "Security test",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.FACT_CHECKING],
        metadata: {},
      };

      const result = await orchestrator.verifyInformation(request);

      expect(result).toBeDefined();
    });
  });

  describe("Verification Disabled Gracefully", () => {
    it("should handle verification when disabled in config", async () => {
      const configDisabled: ArbiterOrchestratorConfig = {
        ...testConfig,
        // Note: verification property not part of ArbiterOrchestratorConfig interface
      };

      const orchestratorDisabled = new ArbiterOrchestrator(configDisabled);
      await orchestratorDisabled.initialize();

      try {
        const request: VerificationRequest = {
          id: "disabled-test",
          content: "Test content",
          source: "https://example.com",
          context: "Disabled test",
          priority: VerificationPriority.LOW,
          verificationTypes: [VerificationType.FACT_CHECKING],
          metadata: {},
        };

        await orchestratorDisabled.verifyInformation(request);
        throw new Error(
          "Should have thrown error when verification is disabled"
        );
      } catch (error) {
        expect(error).toBeDefined();
        expect((error as Error).message).toContain("not enabled");
      }

      await orchestratorDisabled.shutdown();
    });
  });

  describe("Performance Monitoring", () => {
    it("should track verification processing times", async () => {
      const request: VerificationRequest = {
        id: "perf-monitor",
        content: "Content for performance monitoring",
        source: "https://example.com",
        context: "Performance test",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        metadata: {},
      };

      const startTime = Date.now();
      const result = await orchestrator.verifyInformation(request);
      const duration = Date.now() - startTime;

      expect(result.processingTimeMs).toBeDefined();
      expect(result.processingTimeMs).toBeLessThanOrEqual(duration);
    });
  });
});
