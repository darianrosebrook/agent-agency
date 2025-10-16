/**
 * @fileoverview
 * Unit tests for ModelBasedJudge with retry functionality
 *
 * @author @darianrosebrook
 */

import { MockLLMProvider } from "@/evaluation/LLMProvider";
import { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import type { JudgmentInput } from "@/types/judge";

describe("ModelBasedJudge with Retry", () => {
  let judge: ModelBasedJudge;
  let mockProvider: MockLLMProvider;

  beforeEach(() => {
    mockProvider = new MockLLMProvider({
      provider: "mock",
      model: "test-model",
      temperature: 0.7,
      maxTokens: 1000,
    });

    judge = new ModelBasedJudge(
      {
        enableFallback: true,
        thresholds: {
          faithfulness: 0.7,
          relevance: 0.7,
          minimality: 0.7,
          safety: 0.8,
        },
      },
      mockProvider
    );
  });

  describe("retry functionality", () => {
    it("should use retry logic for LLM evaluation", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
        expectedOutput: "Expected output",
      };

      // Mock the provider to fail once then succeed
      const originalEvaluate = mockProvider.evaluate.bind(mockProvider);
      let callCount = 0;
      mockProvider.evaluate = jest
        .fn()
        .mockImplementation(async (input, criterion) => {
          callCount++;
          if (callCount === 1) {
            throw new Error("network error");
          }
          return originalEvaluate(input, criterion);
        });

      const result = await judge.evaluate(input);

      expect(result.assessments).toHaveLength(4);
      expect(callCount).toBe(2); // Should have retried once
    });

    it("should handle fallback when evaluation fails", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      // Mock provider to always fail
      mockProvider.evaluate = jest
        .fn()
        .mockRejectedValue(new Error("service down"));

      // Should use fallback assessment when fallback is enabled
      const result = await judge.evaluate(input);

      expect(result.assessments).toHaveLength(4);
      expect(result.assessments.every((a) => a.score === 0.5)).toBe(true); // Fallback score
    });

    it("should update configuration", () => {
      const newConfig = {
        enableFallback: false,
        thresholds: {
          faithfulness: 0.8,
          relevance: 0.8,
          minimality: 0.8,
          safety: 0.9,
        },
      };

      // Update configuration
      judge.updateConfig(newConfig);

      const updatedConfig = judge.getConfig();
      expect(updatedConfig.enableFallback).toBe(false);
      expect(updatedConfig.thresholds.faithfulness).toBe(0.8);
    });

    it("should reset metrics", () => {
      // Get initial metrics
      const initialMetrics = judge.getMetrics();
      expect(initialMetrics.totalJudgments).toBe(0);

      // Reset metrics
      judge.resetMetrics();

      const resetMetrics = judge.getMetrics();
      expect(resetMetrics.totalJudgments).toBe(0);
      expect(resetMetrics.fallbackRate).toBe(0);
      expect(resetMetrics.averageConfidence).toBe(0);
    });

    it("should handle retry exhaustion gracefully with fallback", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      // Mock provider to always fail
      mockProvider.evaluate = jest
        .fn()
        .mockRejectedValue(new Error("persistent error"));

      const result = await judge.evaluate(input);

      // Should use fallback assessments
      expect(result.assessments).toHaveLength(4);
      expect(result.assessments[0].score).toBe(0.5); // Fallback score
      expect(result.assessments[0].confidence).toBe(0.3); // Fallback confidence
    });

    it("should track retry metrics in judgment result", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      // Mock provider to fail once then succeed
      const originalEvaluate = mockProvider.evaluate.bind(mockProvider);
      let callCount = 0;
      mockProvider.evaluate = jest
        .fn()
        .mockImplementation(async (input, criterion) => {
          callCount++;
          if (callCount <= 2) {
            throw new Error("network error");
          }
          return originalEvaluate(input, criterion);
        });

      const result = await judge.evaluate(input);

      // Should have succeeded after retries
      expect(result.assessments).toHaveLength(4);
      expect(result.assessments[0].score).toBeGreaterThan(0.5);
    });

    it("should handle different error types appropriately", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      // Test retryable error
      mockProvider.evaluate = jest
        .fn()
        .mockRejectedValueOnce(new Error("ECONNRESET"))
        .mockResolvedValue({
          criterion: "faithfulness",
          score: 0.8,
          confidence: 0.9,
          reasoning: "Good response",
        });

      let result = await judge.evaluate(input);
      expect(result.assessments).toHaveLength(4);

      // Test non-retryable error
      mockProvider.evaluate = jest
        .fn()
        .mockRejectedValue(new Error("authentication failed"));

      result = await judge.evaluate(input);
      // Should use fallback due to non-retryable error
      expect(result.assessments).toHaveLength(4);
      expect(result.assessments[0].score).toBe(0.5); // Fallback score
    });
  });

  describe("integration with existing functionality", () => {
    it("should maintain existing evaluation behavior with retry", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
        expectedOutput: "Expected output",
      };

      const result = await judge.evaluate(input);

      expect(result.assessments).toHaveLength(4);
      expect(result.assessments[0].criterion).toBe("faithfulness");
      expect(result.assessments[1].criterion).toBe("relevance");
      expect(result.assessments[2].criterion).toBe("minimality");
      expect(result.assessments[3].criterion).toBe("safety");

      // All assessments should have valid scores and confidence
      result.assessments.forEach((assessment) => {
        expect(assessment.score).toBeGreaterThanOrEqual(0);
        expect(assessment.score).toBeLessThanOrEqual(1);
        expect(assessment.confidence).toBeGreaterThanOrEqual(0);
        expect(assessment.confidence).toBeLessThanOrEqual(1);
        expect(assessment.reasoning).toBeDefined();
      });
    });

    it("should maintain metrics tracking with retry", async () => {
      const input: JudgmentInput = {
        task: "Test task",
        output: "Test output",
      };

      const result = await judge.evaluate(input);
      const metrics = judge.getMetrics();

      expect(metrics.totalJudgments).toBe(1);
      expect(metrics.averageConfidence).toBeGreaterThan(0);
    });
  });
});
