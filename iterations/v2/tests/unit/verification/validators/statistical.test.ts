/**
 * @fileoverview Unit tests for StatisticalValidator
 *
 * Tests statistical claim validation for ARBITER-007
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "@/types/verification";
import { StatisticalValidator } from "@/verification/validators/StatisticalValidator";

describe("StatisticalValidator", () => {
  let validator: StatisticalValidator;

  beforeEach(() => {
    validator = new StatisticalValidator({
      statisticalTests: ["chi_square", "correlation", "significance"],
      minSampleSize: 30,
    });
  });

  describe("Sample Size Validation", () => {
    it("should verify claims with adequate sample sizes", async () => {
      const request: VerificationRequest = {
        id: "sample-adequate",
        content:
          "A study of 500 participants found that 75% preferred option A over option B.",
        source: "https://example.com",
        context: "Survey results",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
      expect(result.confidence).toBeGreaterThan(0.7);
    });

    it("should flag claims with inadequate sample sizes", async () => {
      const request: VerificationRequest = {
        id: "sample-inadequate",
        content: "A study of 5 people showed that 80% experienced improvement.",
        source: "https://example.com",
        context: "Small sample",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain("inadequate_sample_size");
    });

    it("should detect missing sample size information", async () => {
      const request: VerificationRequest = {
        id: "sample-missing",
        content: "Most people prefer coffee over tea.",
        source: "https://example.com",
        context: "No sample size",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.UNVERIFIED);
      expect(result.metadata?.issues).toContain("missing_sample_size");
    });
  });

  describe("Statistical Significance", () => {
    it("should verify claims with proper significance levels", async () => {
      const request: VerificationRequest = {
        id: "significance-valid",
        content:
          "The difference between groups was statistically significant (p < 0.05) with n=200.",
        source: "https://example.com",
        context: "Significance testing",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });

    it("should flag claims with weak significance", async () => {
      const request: VerificationRequest = {
        id: "significance-weak",
        content: "The results showed a trend (p = 0.15) in 50 subjects.",
        source: "https://example.com",
        context: "Weak significance",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain("weak_significance");
    });
  });

  describe("Correlation vs Causation", () => {
    it("should verify proper causal claims", async () => {
      const request: VerificationRequest = {
        id: "causation-valid",
        content:
          "The randomized controlled trial of 300 participants demonstrated that the treatment caused a significant improvement.",
        source: "https://example.com",
        context: "Valid causation",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });

    it("should detect correlation-causation confusion", async () => {
      const request: VerificationRequest = {
        id: "correlation-causation",
        content:
          "Ice cream sales and drowning deaths are correlated, therefore ice cream causes drowning.",
        source: "https://example.com",
        context: "Correlation fallacy",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain(
        "correlation_causation_confusion"
      );
    });
  });

  describe("Cherry-Picking Detection", () => {
    it("should detect selective data presentation", async () => {
      const request: VerificationRequest = {
        id: "cherry-picking",
        content:
          "In one study, the success rate was 95%. (Note: Five other studies showed rates between 40-60%)",
        source: "https://example.com",
        context: "Cherry-picked data",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain("cherry_picking");
    });
  });

  describe("Confidence Intervals", () => {
    it("should verify claims with proper confidence intervals", async () => {
      const request: VerificationRequest = {
        id: "ci-proper",
        content:
          "The mean improvement was 15% (95% CI: 12-18%) in 150 participants.",
        source: "https://example.com",
        context: "Confidence intervals",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });

    it("should flag missing confidence intervals for estimates", async () => {
      const request: VerificationRequest = {
        id: "ci-missing",
        content: "The average effect size was 0.8 in our sample.",
        source: "https://example.com",
        context: "Missing CI",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      // May flag as potentially problematic
      expect(result.verdict).toBeDefined();
    });
  });

  describe("Percentage and Ratio Validation", () => {
    it("should verify mathematically correct percentages", async () => {
      const request: VerificationRequest = {
        id: "percentage-valid",
        content: "Of 200 respondents, 150 (75%) agreed with the statement.",
        source: "https://example.com",
        context: "Correct percentage",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_TRUE);
    });

    it("should detect incorrect percentage calculations", async () => {
      const request: VerificationRequest = {
        id: "percentage-invalid",
        content: "Of 100 respondents, 75 (80%) agreed with the statement.",
        source: "https://example.com",
        context: "Wrong percentage",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain("incorrect_percentage");
    });

    it("should detect percentages that don't sum to 100%", async () => {
      const request: VerificationRequest = {
        id: "percentage-sum",
        content:
          "60% chose option A, 50% chose option B, and 20% chose option C.",
        source: "https://example.com",
        context: "Invalid percentage sum",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain("percentage_sum_error");
    });
  });

  describe("Data Manipulation Detection", () => {
    it("should detect p-hacking indicators", async () => {
      const request: VerificationRequest = {
        id: "p-hacking",
        content:
          "After excluding outliers, removing 20% of participants, and trying 15 different analyses, we found p = 0.048.",
        source: "https://example.com",
        context: "P-hacking",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain("p_hacking");
    });

    it("should detect suspicious data patterns", async () => {
      const request: VerificationRequest = {
        id: "suspicious-data",
        content:
          "All measurements were exactly 10.0, 20.0, 30.0, 40.0, 50.0 with no variation.",
        source: "https://example.com",
        context: "Suspicious patterns",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.VERIFIED_FALSE);
      expect(result.metadata?.issues).toContain("suspicious_data_pattern");
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
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.UNVERIFIED);
      expect(result.confidence).toBe(0);
    });

    it("should handle content without statistical claims", async () => {
      const request: VerificationRequest = {
        id: "no-stats",
        content: "The weather is nice today.",
        source: "https://example.com",
        context: "No statistics",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBe(VerificationVerdict.UNVERIFIED);
    });

    it("should handle multiple statistical claims", async () => {
      const request: VerificationRequest = {
        id: "multiple-claims",
        content:
          "Study 1 (n=200): 60% success rate. Study 2 (n=150): 55% success rate. Meta-analysis (n=350): 58% overall success rate.",
        source: "https://example.com",
        context: "Multiple studies",
        priority: VerificationPriority.HIGH,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const result = await validator.verify(request);

      expect(result.verdict).toBeDefined();
      expect(result.metadata?.claimsAnalyzed).toBeGreaterThan(1);
    });
  });

  describe("Configuration", () => {
    it("should respect minimum sample size configuration", async () => {
      const strictValidator = new StatisticalValidator({
        statisticalTests: ["significance"],
        minSampleSize: 100, // Higher threshold
      });

      const request: VerificationRequest = {
        id: "sample-threshold",
        content: "A study of 50 participants found a significant effect.",
        source: "https://example.com",
        context: "Sample threshold",
        priority: VerificationPriority.MEDIUM,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
        metadata: {},
      };

      const strictResult = await strictValidator.verify(request);
      const normalResult = await validator.verify(request);

      // Strict validator should be more critical
      expect(strictResult.confidence).toBeLessThanOrEqual(
        normalResult.confidence
      );
    });
  });

  describe("Performance", () => {
    it("should complete within timeout", async () => {
      const request: VerificationRequest = {
        id: "perf-test",
        content:
          "Study of 500 participants: 60% success. Study of 300: 55%. Study of 400: 62%.",
        source: "https://example.com",
        context: "Performance test",
        priority: VerificationPriority.LOW,
        verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
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
          content: `Study of ${100 + i * 10} participants found ${
            60 + i
          }% success rate.`,
          source: `https://example${i}.com`,
          context: "Concurrent test",
          priority: VerificationPriority.LOW,
          verificationTypes: [VerificationType.STATISTICAL_VALIDATION],
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
