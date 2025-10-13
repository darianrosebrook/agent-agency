/**
 * @fileoverview Unit tests for LogicalValidator
 *
 * Tests logical reasoning validation for ARBITER-007
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "@/types/verification";
import { LogicalValidator } from "@/verification/validators/LogicalValidator";

describe("LogicalValidator", () => {
  let validator: LogicalValidator;

  beforeEach(() => {
    validator = new LogicalValidator({
      reasoningEngine: "symbolic",
      detectFallacies: true,
    });
  });

  describe("Valid Logical Arguments", () => {
    it("should verify modus ponens", async () => {
      const request: VerificationRequest = {
        id: "modus-ponens",
        content:
          "If it rains, the ground gets wet. It is raining. Therefore, the ground is wet.",
        source: "https://example.com",
        context: "Modus ponens",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.8);
    });

    it("should verify modus tollens", async () => {
      const request: VerificationRequest = {
        id: "modus-tollens",
        content:
          "If it's sunny, the sky is blue. The sky is not blue. Therefore, it's not sunny.",
        source: "https://example.com",
        context: "Modus tollens",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.7);
    });

    it("should verify syllogisms", async () => {
      const request: VerificationRequest = {
        id: "syllogism",
        content:
          "All humans are mortal. Socrates is human. Therefore, Socrates is mortal.",
        source: "https://example.com",
        context: "Classic syllogism",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.8);
    });
  });

  describe("Invalid Logical Arguments", () => {
    it("should detect affirming the consequent fallacy", async () => {
      const request: VerificationRequest = {
        id: "affirm-consequent",
        content:
          "If it rains, the ground is wet. The ground is wet. Therefore, it rained.",
        source: "https://example.com",
        context: "Logical fallacy",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("affirming_consequent");
    });

    it("should detect denying the antecedent fallacy", async () => {
      const request: VerificationRequest = {
        id: "deny-antecedent",
        content:
          "If it's a cat, it's an animal. It's not a cat. Therefore, it's not an animal.",
        source: "https://example.com",
        context: "Logical fallacy",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("denying_antecedent");
    });

    it("should detect circular reasoning", async () => {
      const request: VerificationRequest = {
        id: "circular",
        content:
          "God exists because the Bible says so. The Bible is true because God wrote it.",
        source: "https://example.com",
        context: "Circular reasoning",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("circular_reasoning");
    });
  });

  describe("Fallacy Detection", () => {
    it("should detect ad hominem", async () => {
      const request: VerificationRequest = {
        id: "ad-hominem",
        content:
          "You can't trust his argument about climate change because he's not a scientist.",
        source: "https://example.com",
        context: "Ad hominem fallacy",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("ad_hominem");
    });

    it("should detect straw man fallacy", async () => {
      const request: VerificationRequest = {
        id: "straw-man",
        content:
          "They want to regulate guns, so they must want to take away all our freedoms.",
        source: "https://example.com",
        context: "Straw man",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("straw_man");
    });

    it("should detect false dichotomy", async () => {
      const request: VerificationRequest = {
        id: "false-dichotomy",
        content:
          "You're either with us or against us. There's no middle ground.",
        source: "https://example.com",
        context: "False dichotomy",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("false_dichotomy");
    });

    it("should detect slippery slope", async () => {
      const request: VerificationRequest = {
        id: "slippery-slope",
        content:
          "If we allow same-sex marriage, next people will want to marry animals.",
        source: "https://example.com",
        context: "Slippery slope",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("slippery_slope");
    });

    it("should detect appeal to authority", async () => {
      const request: VerificationRequest = {
        id: "appeal-authority",
        content: "This must be true because a famous person said it.",
        source: "https://example.com",
        context: "Appeal to authority",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.fallacies).toContain("appeal_to_authority");
    });
  });

  describe("Argument Structure Analysis", () => {
    it("should extract premises and conclusions", async () => {
      const request: VerificationRequest = {
        id: "structure-analysis",
        content:
          "All mammals are warm-blooded. Whales are mammals. Therefore, whales are warm-blooded.",
        source: "https://example.com",
        context: "Argument structure",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.metadata?.premises).toBeDefined();
      expect(result.metadata?.conclusion).toBeDefined();
    });

    it("should identify logical connectives", async () => {
      const request: VerificationRequest = {
        id: "connectives",
        content:
          "If it's warm and sunny, then we'll go to the beach. It's warm and sunny. Therefore, we'll go to the beach.",
        source: "https://example.com",
        context: "Logical connectives",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.metadata?.connectives).toBeDefined();
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
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.UNVERIFIED);
      expect(result.confidence).toBe(0);
    });

    it("should handle non-logical statements", async () => {
      const request: VerificationRequest = {
        id: "non-logical",
        content: "The sky is beautiful today.",
        source: "https://example.com",
        context: "Opinion statement",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.UNVERIFIED);
    });

    it("should handle complex nested logic", async () => {
      const request: VerificationRequest = {
        id: "nested-logic",
        content:
          "If (A and B) or (C and D), then E. If E, then F. A and B are true. Therefore, F is true.",
        source: "https://example.com",
        context: "Nested logic",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });
  });

  describe("Configuration", () => {
    it("should work with fallacy detection disabled", async () => {
      const validatorNoFallacies = new LogicalValidator({
        reasoningEngine: "symbolic",
        detectFallacies: false,
      });

      const request: VerificationRequest = {
        id: "no-fallacy-check",
        content:
          "If it rains, the ground is wet. The ground is wet. Therefore, it rained.",
        source: "https://example.com",
        context: "Fallacy detection off",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const result = await validatorNoFallacies.verify(request);

      // May not detect the fallacy
      expect(result.verdict).toBeDefined();
    });
  });

  describe("Performance", () => {
    it("should complete within timeout", async () => {
      const request: VerificationRequest = {
        id: "perf-test",
        content:
          "A ".repeat(500) + "If A then B. A is true. Therefore B is true.",
        source: "https://example.com",
        context: "Performance test",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.LOGICAL_VALIDATION],
        metadata: {},
      };

      const startTime = Date.now();
      const result = await validator.verify(request);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(5000);
      expect(result.processingTimeMs).toBeDefined();
    });

    it("should handle concurrent verifications", async () => {
      const requests: VerificationRequest[] = Array.from(
        { length: 5 },
        (_, i) => ({
          id: `concurrent-${i}`,
          content: `If A${i} then B${i}. A${i} is true. Therefore B${i} is true.`,
          source: `https://example${i}.com`,
          context: "Concurrent test",
          priority: VerificationPriority.LOW,
          verificationTypes: [VerificationType.LOGICAL_VALIDATION],
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
});
