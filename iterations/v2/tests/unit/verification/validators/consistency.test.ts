/**
 * @fileoverview Unit tests for ConsistencyValidator
 *
 * Tests internal consistency checking for ARBITER-007
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "@/types/verification";
import { ConsistencyValidator } from "@/verification/validators/ConsistencyValidator";

describe("ConsistencyValidator", () => {
  let validator: ConsistencyValidator;

  beforeEach(() => {
    validator = new ConsistencyValidator({
      logicEngine: "default",
      strictMode: false,
    });
  });

  describe("Basic Consistency Checking", () => {
    it("should verify internally consistent content", async () => {
      const request: VerificationRequest = {
        id: "consistency-1",
        content:
          "John is 30 years old. He was born in 1993. He is currently in his thirties.",
        source: "https://example.com",
        context: "Consistent statements",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.7);
      expect(result.method).toBe(VerificationType.CONSISTENCY_CHECK);
    });

    it("should detect direct contradictions", async () => {
      const request: VerificationRequest = {
        id: "consistency-2",
        content:
          "The company was founded in 2010. The CEO started the company in 2015.",
        source: "https://example.com",
        context: "Contradictory dates",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.confidence).toBeGreaterThan(0.6);
    });

    it("should handle neutral content appropriately", async () => {
      const request: VerificationRequest = {
        id: "consistency-3",
        content: "The sky is blue. Dogs are animals.",
        source: "https://example.com",
        context: "Unrelated statements",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.5);
    });
  });

  describe("Temporal Consistency", () => {
    it("should verify correct temporal sequences", async () => {
      const request: VerificationRequest = {
        id: "temporal-1",
        content:
          "The event started at 9 AM and ended at 5 PM. It lasted 8 hours.",
        source: "https://example.com",
        context: "Time sequence",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.7);
    });

    it("should detect temporal inconsistencies", async () => {
      const request: VerificationRequest = {
        id: "temporal-2",
        content:
          "The meeting ended before it started. It was scheduled for 2 PM but concluded at 1 PM.",
        source: "https://example.com",
        context: "Time contradiction",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
    });

    it("should handle date ranges correctly", async () => {
      const request: VerificationRequest = {
        id: "temporal-3",
        content:
          "The project ran from January 2020 to March 2020. It lasted 3 months.",
        source: "https://example.com",
        context: "Date range",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });
  });

  describe("Numerical Consistency", () => {
    it("should verify numerical relationships", async () => {
      const request: VerificationRequest = {
        id: "numerical-1",
        content:
          "The budget is $100,000. Marketing gets 40% and Development gets 60%.",
        source: "https://example.com",
        context: "Budget allocation",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });

    it("should detect numerical contradictions", async () => {
      const request: VerificationRequest = {
        id: "numerical-2",
        content: "The total is 100. A is 60, B is 50.",
        source: "https://example.com",
        context: "Math error",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.confidence).toBeGreaterThan(0.7);
    });
  });

  describe("Circular Reasoning Detection", () => {
    it("should detect circular arguments", async () => {
      const request: VerificationRequest = {
        id: "circular-1",
        content: "A is true because B is true. B is true because A is true.",
        source: "https://example.com",
        context: "Circular reasoning",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.circularReasoning).toBe(true);
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty content", async () => {
      const request: VerificationRequest = {
        id: "empty",
        content: "",
        source: "https://example.com",
        context: "Empty content",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.UNVERIFIED);
      expect(result.confidence).toBe(0);
    });

    it("should handle single statement", async () => {
      const request: VerificationRequest = {
        id: "single-statement",
        content: "The Earth orbits the Sun.",
        source: "https://example.com",
        context: "Single fact",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      // Single statement should be verified (no contradiction possible)
      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });
  });

  describe("Strict Mode", () => {
    it("should be more rigorous in strict mode", async () => {
      const strictValidator = new ConsistencyValidator({
        logicEngine: "default",
        strictMode: true,
      });

      const request: VerificationRequest = {
        id: "strict-mode",
        content: "The product is good. The product has some issues.",
        source: "https://example.com",
        context: "Slightly contradictory",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const strictResult = await strictValidator.verify(request);
      const normalResult = await validator.verify(request);

      // Strict mode should be more critical
      expect(strictResult.confidence).toBeLessThanOrEqual(
        normalResult.confidence
      );
    });
  });

  describe("Performance", () => {
    it("should complete within reasonable time", async () => {
      const request: VerificationRequest = {
        id: "perf-test",
        content: "A ".repeat(1000) + "B is consistent with C.",
        source: "https://example.com",
        context: "Performance test",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const startTime = Date.now();
      const result = await validator.verify(request);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(5000); // 5 second timeout
      expect(result.processingTimeMs).toBeDefined();
    });

    it("should handle concurrent verifications", async () => {
      const requests: VerificationRequest[] = Array.from(
        { length: 5 },
        (_, i) => ({
          id: `concurrent-${i}`,
          content: `Statement ${i} is consistent with statement ${i + 1}.`,
          source: `https://example${i}.com`,
          context: "Concurrent test",
          priority: VerificationPriority.LOW,
          verificationTypes: [VerificationType.CONSISTENCY_CHECK],
          metadata: {},
        })
      );

      const results = await Promise.all(
        requests.map((req) => validator.verify(req))
      );

      expect(results.length).toBe(5);
      expect(results.every((r) => r.verdict !== undefined)).toBe(true);
    });
  });

  describe("Evidence Reporting", () => {
    it("should provide evidence for contradictions", async () => {
      const request: VerificationRequest = {
        id: "evidence-test",
        content: "The temperature is 100 degrees. It is freezing cold outside.",
        source: "https://example.com",
        context: "Temperature contradiction",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.metadata?.contradictions).toBeDefined();
    });

    it("should identify specific contradictory statements", async () => {
      const request: VerificationRequest = {
        id: "specific-contradiction",
        content:
          "Point 1: All birds can fly. Point 2: Penguins are birds. Point 3: Penguins cannot fly.",
        source: "https://example.com",
        context: "Logical contradiction",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.CONSISTENCY_CHECK],
        metadata: {},
      };

      const result = await validator.verify(request);

      if (result.verdict === VerificationVerdict.VERIFIED_FALSE) {
        expect(result.metadata?.contradictions).toBeDefined();
      }
    });
  });
});
