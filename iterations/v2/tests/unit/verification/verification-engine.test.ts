/**
 * @fileoverview Tests for Verification Engine Component (ARBITER-007)
 *
 * @author @darianrosebrook
 */

import {
  VerificationEngineConfig,
  VerificationPriority,
  VerificationRequest,
  VerificationType,
} from "../../../src/types/verification";
import { VerificationEngineImpl } from "../../../src/verification/VerificationEngine";

describe("VerificationEngine", () => {
  let engine: VerificationEngineImpl;
  let config: VerificationEngineConfig;

  beforeEach(() => {
    config = {
      defaultTimeoutMs: 5000,
      maxConcurrentVerifications: 5,
      minConfidenceThreshold: 0.5,
      maxEvidencePerMethod: 10,
      methods: [
        {
          type: VerificationType.FACT_CHECKING,
          enabled: true,
          priority: 1,
          timeoutMs: 2000,
          config: { providers: ["mock"] },
        },
        {
          type: VerificationType.SOURCE_CREDIBILITY,
          enabled: true,
          priority: 2,
          timeoutMs: 1500,
          config: { database: "mock" },
        },
      ],
      cacheEnabled: true,
      cacheTtlMs: 300000,
      retryAttempts: 2,
      retryDelayMs: 1000,
    };

    engine = new VerificationEngineImpl(config);
  });

  describe("Request Validation", () => {
    it("should accept valid verification requests", async () => {
      const request: VerificationRequest = {
        id: "test-1",
        content: "The Earth is round and orbits the Sun.",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        timeoutMs: 3000,
        metadata: {},
      };

      const result = await engine.verify(request);

      expect(result).toBeDefined();
      expect(result.requestId).toBe(request.id);
      expect(typeof result.confidence).toBe("number");
      expect(result.verificationMethods).toBeInstanceOf(Array);
    });

    it("should reject requests without ID", async () => {
      const request = {
        id: "",
        content: "Test content",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      } as VerificationRequest;

      await expect(engine.verify(request)).rejects.toThrow();
    });

    it("should reject requests without content", async () => {
      const request = {
        id: "test-1",
        content: "",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      } as VerificationRequest;

      await expect(engine.verify(request)).rejects.toThrow();
    });

    it("should handle timeout constraints", async () => {
      const request: VerificationRequest = {
        id: "test-timeout",
        content: "Test content for timeout",
        priority: VerificationPriority.MEDIUM,
        timeoutMs: 1, // Very short timeout
        metadata: {},
      };

      // Should complete within reasonable time despite short timeout
      const startTime = Date.now();
      const result = await engine.verify(request);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(100); // Should be very fast
      expect(result).toBeDefined();
    });
  });

  describe("Method Selection", () => {
    it("should use specified verification methods", async () => {
      const request: VerificationRequest = {
        id: "test-methods",
        content: "Test content",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.FACT_CHECKING],
        metadata: {},
      };

      const result = await engine.verify(request);

      expect(result.verificationMethods.length).toBeGreaterThan(0);
      expect(
        result.verificationMethods.some(
          (m) => m.method === VerificationType.FACT_CHECKING
        )
      ).toBe(true);
    });

    it("should use all enabled methods when none specified", async () => {
      const request: VerificationRequest = {
        id: "test-all-methods",
        content: "Test content for all methods",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await engine.verify(request);

      expect(result.verificationMethods.length).toBeGreaterThanOrEqual(2);
    });

    it("should respect method priority", async () => {
      const request: VerificationRequest = {
        id: "test-priority",
        content: "Test content for priority",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [
          VerificationType.FACT_CHECKING,
          VerificationType.SOURCE_CREDIBILITY,
        ],
        metadata: {},
      };

      const result = await engine.verify(request);

      // Should have results from both methods
      const factChecking = result.verificationMethods.find(
        (m) => m.method === VerificationType.FACT_CHECKING
      );
      const credibility = result.verificationMethods.find(
        (m) => m.method === VerificationType.SOURCE_CREDIBILITY
      );

      expect(factChecking).toBeDefined();
      expect(credibility).toBeDefined();
    });
  });

  describe("Result Aggregation", () => {
    it("should aggregate results from multiple methods", async () => {
      const request: VerificationRequest = {
        id: "test-aggregation",
        content: "The Earth orbits the Sun. This is a scientific fact.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await engine.verify(request);

      expect(result.verdict).toBeDefined();
      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
      expect(result.reasoning).toBeInstanceOf(Array);
      expect(result.reasoning.length).toBeGreaterThan(0);
    });

    it("should provide confidence scores", async () => {
      const request: VerificationRequest = {
        id: "test-confidence",
        content: "This is a test statement.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await engine.verify(request);

      expect(typeof result.confidence).toBe("number");
      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
    });

    it("should include processing time", async () => {
      const request: VerificationRequest = {
        id: "test-timing",
        content: "Test content for timing",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await engine.verify(request);

      expect(result.processingTimeMs).toBeGreaterThanOrEqual(0);
      expect(typeof result.processingTimeMs).toBe("number");
    });
  });

  describe("Health Monitoring", () => {
    it("should provide health status", async () => {
      const health = await engine.healthCheck();

      expect(health).toBeDefined();
      expect(typeof health.healthy).toBe("boolean");
      expect(typeof health.totalMethods).toBe("number");
      expect(typeof health.enabledMethods).toBe("number");
      expect(typeof health.healthyMethods).toBe("number");
      expect(typeof health.cacheSize).toBe("number");
      expect(typeof health.activeVerifications).toBe("number");
    });

    it("should report supported methods", () => {
      const methods = engine.getSupportedMethods();

      expect(methods).toBeInstanceOf(Array);
      expect(methods.length).toBeGreaterThan(0);
      expect(methods).toContain(VerificationType.FACT_CHECKING);
      expect(methods).toContain(VerificationType.SOURCE_CREDIBILITY);
    });

    it("should provide method status", () => {
      const status = engine.getMethodStatus(VerificationType.FACT_CHECKING);

      expect(status).toBeDefined();
      expect(status.type).toBe(VerificationType.FACT_CHECKING);
      expect(typeof status.enabled).toBe("boolean");
      expect(typeof status.healthy).toBe("boolean");
    });
  });
});
