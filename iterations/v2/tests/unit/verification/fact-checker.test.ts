/**
 * @fileoverview Tests for Fact Checker Component (ARBITER-007)
 *
 * @author @darianrosebrook
 */

import { FactChecker } from "../../../src/verification/FactChecker";
import {
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
  VerificationPriority,
} from "../../../src/types/verification";

describe("FactChecker", () => {
  let factChecker: FactChecker;

  beforeEach(() => {
    factChecker = new FactChecker([
      {
        type: VerificationType.FACT_CHECKING,
        enabled: true,
        priority: 1,
        timeoutMs: 5000,
        config: { providers: ["mock"] },
      },
    ]);
  });

  describe("Claim Extraction", () => {
    it("should extract verifiable claims from content", async () => {
      const request: VerificationRequest = {
        id: "test-extraction",
        content: "The Earth is round and orbits the Sun. Albert Einstein was born in 1879. Vaccines cause autism according to some sources.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.method).toBe(VerificationType.FACT_CHECKING);
      expect(typeof result.confidence).toBe("number");
      expect(result.reasoning).toBeInstanceOf(Array);
    });

    it("should handle content without verifiable claims", async () => {
      const request: VerificationRequest = {
        id: "test-no-claims",
        content: "This is just a simple statement without any facts to check.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.verdict).toBe(VerificationVerdict.INSUFFICIENT_DATA);
      expect(result.confidence).toBe(0);
      expect(result.reasoning).toContain("No verifiable claims found in content");
    });

    it("should identify factual statements", async () => {
      const factualContent = "The United States declared independence in 1776. Water boils at 100 degrees Celsius.";
      const request: VerificationRequest = {
        id: "test-factual",
        content: factualContent,
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.evidenceCount).toBeGreaterThan(0);
    });

    it("should categorize claims appropriately", async () => {
      const request: VerificationRequest = {
        id: "test-categorization",
        content: "Marie Curie discovered radium in 1898. The population of Tokyo is over 13 million.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
    });
  });

  describe("Fact Checking Logic", () => {
    it("should verify well-established facts positively", async () => {
      const request: VerificationRequest = {
        id: "test-established-fact",
        content: "The Earth revolves around the Sun.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.8);
    });

    it("should identify debunked claims", async () => {
      const request: VerificationRequest = {
        id: "test-debunked-claim",
        content: "Vaccines cause autism according to some studies.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.confidence).toBeGreaterThan(0.7);
    });

    it("should handle unverifiable statements", async () => {
      const request: VerificationRequest = {
        id: "test-unverifiable",
        content: "This is my personal opinion about the weather.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.confidence).toBeGreaterThanOrEqual(0);
    });

    it("should aggregate multiple claims", async () => {
      const request: VerificationRequest = {
        id: "test-multiple-claims",
        content: "The Earth revolves around the Sun. The Moon orbits Earth. Mars has two moons. Jupiter is the largest planet.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.evidenceCount).toBeGreaterThan(0);
      expect(result.reasoning.length).toBeGreaterThan(0);
    });
  });

  describe("Confidence Scoring", () => {
    it("should provide confidence scores between 0 and 1", async () => {
      const request: VerificationRequest = {
        id: "test-confidence-range",
        content: "This is a test statement for confidence scoring.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
    });

    it("should have higher confidence for established facts", async () => {
      const establishedFact: VerificationRequest = {
        id: "established",
        content: "Water is wet.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const controversialClaim: VerificationRequest = {
        id: "controversial",
        content: "This is a debated topic.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result1 = await factChecker.verify(establishedFact);
      const result2 = await factChecker.verify(controversialClaim);

      expect(result1.confidence).toBeGreaterThanOrEqual(result2.confidence);
    });
  });

  describe("Evidence and Reasoning", () => {
    it("should provide reasoning for verdicts", async () => {
      const request: VerificationRequest = {
        id: "test-reasoning",
        content: "The capital of France is Paris.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result.reasoning).toBeInstanceOf(Array);
      expect(result.reasoning.length).toBeGreaterThan(0);
      expect(result.reasoning[0]).toBeDefined();
    });

    it("should include evidence count", async () => {
      const request: VerificationRequest = {
        id: "test-evidence-count",
        content: "This statement has supporting evidence.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(typeof result.evidenceCount).toBe("number");
      expect(result.evidenceCount).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Error Handling", () => {
    it("should handle empty content gracefully", async () => {
      const request: VerificationRequest = {
        id: "test-empty",
        content: "",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.verdict).toBe(VerificationVerdict.INSUFFICIENT_DATA);
      expect(result.confidence).toBe(0);
    });

    it("should handle very long content", async () => {
      const longContent = "A".repeat(10000);
      const request: VerificationRequest = {
        id: "test-long-content",
        content: longContent,
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.processingTimeMs).toBeGreaterThan(0);
    });

    it("should handle malformed content", async () => {
      const request: VerificationRequest = {
        id: "test-malformed",
        content: "This content has no verifiable claims at all.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await factChecker.verify(request);

      expect(result).toBeDefined();
      expect(result.verdict).toBe(VerificationVerdict.INSUFFICIENT_DATA);
    });
  });

  describe("Performance", () => {
    it("should complete verification within timeout", async () => {
      const request: VerificationRequest = {
        id: "test-performance",
        content: "The sky is blue. Grass is green. The sun rises in the east.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const startTime = Date.now();
      const result = await factChecker.verify(request);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(1000); // Should be fast
      expect(result.processingTimeMs).toBeGreaterThan(0);
      expect(result.processingTimeMs).toBeLessThan(1000);
    });

    it("should handle concurrent requests", async () => {
      const requests = Array(5).fill(null).map((_, i) => ({
        id: `concurrent-${i}`,
        content: `Concurrent test content ${i}`,
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      }));

      const startTime = Date.now();
      const results = await Promise.all(requests.map(req => factChecker.verify(req)));
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(5);
      expect(duration).toBeLessThan(2000); // Should complete reasonably fast
    });
  });

  describe("Method Availability", () => {
    it("should report availability status", async () => {
      const available = await factChecker.isAvailable();
      expect(typeof available).toBe("boolean");
    });

    it("should provide health status", () => {
      const health = factChecker.getHealth();

      expect(health).toBeDefined();
      expect(typeof health.available).toBe("boolean");
      expect(typeof health.responseTime).toBe("number");
      expect(typeof health.errorRate).toBe("number");
    });
  });

  describe("Configuration", () => {
    it("should work with different configurations", () => {
      const customChecker = new FactChecker([
        {
          type: VerificationType.FACT_CHECKING,
          enabled: true,
          priority: 1,
          timeoutMs: 1000,
          config: { customProviders: ["test"] },
        },
      ]);

      expect(customChecker).toBeDefined();
    });

    it("should handle disabled configuration", () => {
      const disabledChecker = new FactChecker([
        {
          type: VerificationType.FACT_CHECKING,
          enabled: false,
          priority: 1,
          timeoutMs: 5000,
          config: {},
        },
      ]);

      expect(disabledChecker).toBeDefined();
    });
  });
});
